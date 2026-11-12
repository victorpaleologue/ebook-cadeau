//! Combining records: [`merge`] reconciles what several providers say about one ISBN,
//! [`combine`] assembles the pieces one provider returned over several requests.

use ebc_model::{BookMeta, Contributor, ContributorRole};

use crate::normalize::{fold_key, name_key, normalize_record};

/// At most this many subjects are kept.
const MAX_SUBJECTS: usize = 20;

// ---------------------------------------------------------------------------------------
// People
// ---------------------------------------------------------------------------------------

/// `true` when two display names designate the same person: equal up to case, accents and
/// word order, or one abbreviates the other (`"A. Camus"` / `"Albert Camus"`).
#[must_use]
pub fn same_person(a: &str, b: &str) -> bool {
    if name_key(a) == name_key(b) {
        return !name_key(a).is_empty();
    }
    let (a, b) = (fold_key(a), fold_key(b));
    let words_a: Vec<&str> = a.split(' ').filter(|w| !w.is_empty()).collect();
    let words_b: Vec<&str> = b.split(' ').filter(|w| !w.is_empty()).collect();
    let (short, long) =
        if words_a.len() <= words_b.len() { (&words_a, &words_b) } else { (&words_b, &words_a) };
    if short.is_empty() {
        return false;
    }
    let mut used = vec![false; long.len()];
    let mut full_match = false;
    for word in short.iter() {
        let initial = word.chars().count() == 1;
        let found = long.iter().enumerate().position(|(i, other)| {
            !used[i]
                && (other == word
                    || (initial && other.starts_with(word))
                    || (other.chars().count() == 1 && word.starts_with(other)))
        });
        match found {
            Some(i) => {
                used[i] = true;
                full_match |= long[i] == *word && !initial;
            }
            None => return false,
        }
    }
    // A shared surname alone is not enough ("Camus" / "Albert Camus" is; "A." / "B." is not).
    full_match
}

fn add_contributor(list: &mut Vec<Contributor>, candidate: &Contributor) {
    match list.iter_mut().find(|known| same_person(&known.name, &candidate.name)) {
        Some(known) => {
            // Keep the fuller spelling; the role already recorded wins.
            if candidate.name.chars().count() > known.name.chars().count()
                && name_key(&candidate.name) != name_key(&known.name)
            {
                known.name.clone_from(&candidate.name);
            }
            if matches!(known.role, ContributorRole::Other(_))
                && !matches!(candidate.role, ContributorRole::Other(_))
            {
                known.role = candidate.role.clone();
            }
        }
        None => list.push(candidate.clone()),
    }
}

/// Authors first, other roles after, each group in its original order.
fn authors_first(list: &mut [Contributor]) {
    list.sort_by_key(|c| c.role != ContributorRole::Author);
}

// ---------------------------------------------------------------------------------------
// Lists that are simply united
// ---------------------------------------------------------------------------------------

fn unite_lists(target: &mut BookMeta, record: &BookMeta) {
    for subject in &record.subjects {
        if target.subjects.len() < MAX_SUBJECTS
            && !target.subjects.iter().any(|s| fold_key(s) == fold_key(subject))
        {
            target.subjects.push(subject.clone());
        }
    }
    for cover in &record.covers {
        if !target.covers.iter().any(|c| c.url == cover.url) {
            target.covers.push(cover.clone());
        }
    }
    for source in &record.sources {
        if !target.sources.contains(source) {
            target.sources.push(source.clone());
        }
    }
    let main = target.isbn13;
    for isbn in record.isbn13.iter().chain(&record.related_isbns) {
        if Some(*isbn) != main && !target.related_isbns.contains(isbn) {
            target.related_isbns.push(*isbn);
        }
    }
}

// ---------------------------------------------------------------------------------------
// One provider, several responses
// ---------------------------------------------------------------------------------------

/// Assembles the records one provider returned for one ISBN — a main record and the
/// fragments its follow-up requests produced — in the order the requests were issued.
///
/// For single-valued fields the first record that has a value wins; contributors,
/// subjects, covers, sources and related ISBNs are united. Returns `None` when `records`
/// is empty.
#[must_use]
pub fn combine(records: &[BookMeta]) -> Option<BookMeta> {
    let (first, rest) = records.split_first()?;
    let mut out = first.clone();
    for record in rest {
        if out.title.trim().is_empty() {
            out.title.clone_from(&record.title);
        }
        macro_rules! fill {
            ($($field:ident),*) => {$(
                if out.$field.is_none() {
                    out.$field.clone_from(&record.$field);
                }
            )*};
        }
        fill!(
            isbn13,
            subtitle,
            publisher,
            imprint,
            series,
            published,
            language,
            page_count,
            description,
            format
        );
        for contributor in &record.contributors {
            add_contributor(&mut out.contributors, contributor);
        }
        unite_lists(&mut out, record);
    }
    authors_first(&mut out.contributors);
    normalize_record(&mut out);
    Some(out)
}

// ---------------------------------------------------------------------------------------
// Several providers
// ---------------------------------------------------------------------------------------

/// Words that do not distinguish one publisher from another.
const PUBLISHER_NOISE: [&str; 14] = [
    "editions",
    "edition",
    "ed",
    "les",
    "the",
    "verlag",
    "gmbh",
    "co",
    "kg",
    "inc",
    "ltd",
    "publishing",
    "publishers",
    "books",
];

fn publisher_key(name: &str) -> String {
    let folded = fold_key(name);
    let kept: Vec<&str> = folded.split(' ').filter(|w| !PUBLISHER_NOISE.contains(w)).collect();
    if kept.is_empty() { folded } else { kept.join(" ") }
}

fn date_key(date: &str) -> String {
    date.chars().take(4).collect()
}

fn language_key(tag: &str) -> String {
    tag.split('-').next().unwrap_or(tag).to_lowercase()
}

/// A title in capitals only (`"L'ETRANGER"`) is a poorer spelling than a mixed-case one.
fn shouting(text: &str) -> bool {
    let letters = text.chars().filter(|c| c.is_alphabetic()).count();
    letters >= 4 && text.chars().filter(|c| c.is_alphabetic()).all(char::is_uppercase)
}

/// Picks one value among the candidates `(record index, value)`, given in priority order.
///
/// Values are grouped by `key`. Each group scores two points per record that states it,
/// plus three when the highest-priority record that has the field is among them: the
/// preferred source is overruled only when at least three other sources agree against it.
/// Ties go to the group seen first. Within the winning group `better` chooses the
/// spelling; by default the highest-priority one.
fn vote<T: Clone>(
    candidates: &[(usize, T)],
    key: impl Fn(&T) -> String,
    better: impl Fn(&T, &T) -> bool,
) -> Option<T> {
    let mut groups: Vec<(String, u32, Vec<&T>)> = Vec::new();
    for (_, value) in candidates {
        let k = key(value);
        if k.is_empty() {
            continue;
        }
        let bonus = if groups.is_empty() { 3 } else { 0 };
        match groups.iter_mut().find(|(known, _, _)| *known == k) {
            Some((_, score, values)) => {
                *score += 2;
                values.push(value);
            }
            None => groups.push((k, 2 + bonus, vec![value])),
        }
    }
    let mut best: Option<&(String, u32, Vec<&T>)> = None;
    for group in &groups {
        if best.is_none_or(|b| group.1 > b.1) {
            best = Some(group);
        }
    }
    let (_, _, values) = best?;
    let mut chosen = *values.first()?;
    for value in values.iter().skip(1) {
        if better(value, chosen) {
            chosen = value;
        }
    }
    Some(chosen.clone())
}

fn text_field(
    records: &[BookMeta],
    get: impl Fn(&BookMeta) -> Option<&String>,
    key: impl Fn(&String) -> String,
) -> Option<String> {
    let candidates: Vec<(usize, String)> = records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| get(r).filter(|v| !v.trim().is_empty()).map(|v| (i, v.clone())))
        .collect();
    vote(&candidates, key, |new, old| shouting(old) && !shouting(new))
}

/// Merges the records several providers returned for the same ISBN into one.
///
/// `records` are in **priority order**, most trusted first (the order of
/// [`default_providers`](crate::default_providers) or
/// [`providers_for`](crate::providers_for)).
///
/// * Title, subtitle, publisher, imprint, series, date, language, page count and format
///   are decided field by field: values are compared after normalisation (case, accents,
///   punctuation; the year for dates; `"Éditions Gallimard"` equals `"Gallimard"`), the
///   most trusted source that has the field wins unless at least three other sources agree
///   on something else. Among equal dates the most precise one is kept.
/// * Contributors start from the most trusted record that names any. Other records add
///   people it lacks: always when they are not authors (a translator, say), and for
///   authors only when two records agree on the name — which keeps a translator that one
///   source mislabels as an author out of the author line.
/// * The description comes from the most trusted record that has one.
/// * Subjects, covers, sources and related ISBNs of every record are kept, without
///   duplicates. The ISBN of the first record that has one is the ISBN of the result; any
///   other becomes a related ISBN — it is never lost.
///
/// The result is normalised ([`normalize_record`]). Returns `None` when `records` is empty
/// or no record has a title.
#[must_use]
pub fn merge(records: &[BookMeta]) -> Option<BookMeta> {
    let cleaned: Vec<BookMeta> = records
        .iter()
        .map(|record| {
            let mut copy = record.clone();
            normalize_record(&mut copy);
            copy
        })
        .collect();
    let records = cleaned.as_slice();
    let title = text_field(records, |r| Some(&r.title), |t| fold_key(t))?;
    let mut out = BookMeta {
        isbn13: records.iter().find_map(|r| r.isbn13),
        title,
        subtitle: text_field(records, |r| r.subtitle.as_ref(), |t| fold_key(t)),
        publisher: text_field(records, |r| r.publisher.as_ref(), |p| publisher_key(p)),
        imprint: text_field(records, |r| r.imprint.as_ref(), |p| publisher_key(p)),
        series: text_field(records, |r| r.series.as_ref(), |s| fold_key(s)),
        language: text_field(records, |r| r.language.as_ref(), |l| language_key(l)),
        description: records.iter().find_map(|r| r.description.clone()),
        ..BookMeta::default()
    };
    let dates: Vec<(usize, String)> = records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.published.clone().map(|d| (i, d)))
        .collect();
    out.published = vote(&dates, |d| date_key(d), |new, old| new.len() > old.len());
    let pages: Vec<(usize, u32)> =
        records.iter().enumerate().filter_map(|(i, r)| r.page_count.map(|p| (i, p))).collect();
    out.page_count = vote(&pages, u32::to_string, |_, _| false);
    let formats: Vec<_> =
        records.iter().enumerate().filter_map(|(i, r)| r.format.map(|f| (i, f))).collect();
    out.format = vote(&formats, |f| format!("{f:?}"), |_, _| false);

    // Contributors: the most trusted list, completed by the others.
    let mut contributors: Vec<Contributor> = Vec::new();
    let mut base_has_author = false;
    if let Some(base) = records.iter().find(|r| !r.contributors.is_empty()) {
        for contributor in &base.contributors {
            add_contributor(&mut contributors, contributor);
        }
        base_has_author = base.authors().next().is_some();
    }
    for (index, record) in records.iter().enumerate() {
        for contributor in &record.contributors {
            if contributors.iter().any(|known| same_person(&known.name, &contributor.name)) {
                add_contributor(&mut contributors, contributor);
                continue;
            }
            let is_author = contributor.role == ContributorRole::Author;
            let confirmed = records.iter().enumerate().any(|(other, r)| {
                other != index
                    && r.contributors.iter().any(|c| same_person(&c.name, &contributor.name))
            });
            if !is_author || !base_has_author || confirmed {
                add_contributor(&mut contributors, contributor);
            }
        }
    }
    authors_first(&mut contributors);
    out.contributors = contributors;

    for record in records {
        unite_lists(&mut out, record);
    }
    normalize_record(&mut out);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn people_are_matched_across_spellings() {
        assert!(same_person("Albert Camus", "Camus, Albert"));
        assert!(same_person("A. Camus", "Albert Camus"));
        assert!(same_person("J. R. R. Tolkien", "John Ronald Reuel Tolkien"));
        assert!(same_person("Camus", "Albert Camus"));
        assert!(!same_person("Albert Camus", "Albert Cohen"));
        assert!(!same_person("A. C.", "Albert Camus"));
        assert!(!same_person("", "Albert Camus"));
        assert!(!same_person("Catherine Camus", "Albert Camus"));
    }

    #[test]
    fn publishers_compare_without_legal_noise() {
        assert_eq!(publisher_key("Éditions Gallimard"), publisher_key("Gallimard"));
        assert_eq!(publisher_key("S. Fischer Verlag GmbH"), publisher_key("S. Fischer"));
        assert_ne!(publisher_key("Gallimard"), publisher_key("Flammarion"));
        assert_eq!(publisher_key("Les Éditions"), "les editions");
    }

    #[test]
    fn the_preferred_source_yields_only_to_three_others() {
        let one = |v: &str| v.to_owned();
        let top = [(0, one("1971")), (1, one("1942")), (2, one("1942"))];
        assert_eq!(vote(&top, |d| date_key(d), |_, _| false).as_deref(), Some("1971"));
        let outvoted = [(0, one("1971")), (1, one("1942")), (2, one("1942")), (3, one("1942-06"))];
        assert_eq!(
            vote(&outvoted, |d| date_key(d), |new, old| new.len() > old.len()).as_deref(),
            Some("1942-06")
        );
        let empty: [(usize, String); 0] = [];
        assert_eq!(vote(&empty, |d| date_key(d), |_, _| false), None);
    }
}
