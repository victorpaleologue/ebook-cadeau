//! Library record formats mapped to [`BookMeta`]: MARC 21, UNIMARC and Dublin Core.
//!
//! The mappers copy raw catalogue strings; [`normalize_record`](crate::normalize) cleans
//! them afterwards (ISBD punctuation, inverted names, qualified dates).

use ebc_isbn::Isbn13;
use ebc_model::{BookFormat, BookMeta, Contributor, ContributorRole};

use crate::normalize::{self, fold_key};
use crate::util::{isbns_in, push_isbn};
use crate::xml::{MarcField, MarcRecord, Node};

/// One catalogue record, not yet normalised.
#[derive(Debug, Clone, Default)]
pub(crate) struct Extracted {
    pub(crate) book: BookMeta,
    /// Every ISBN the record carries.
    pub(crate) isbns: Vec<Isbn13>,
    /// Link to the record on the catalogue's site.
    pub(crate) url: Option<String>,
}

// ---------------------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------------------

/// MARC relator codes (`$4` in MARC 21). `None` means "not a contributor to print".
fn role_from_marc_code(code: &str) -> Option<ContributorRole> {
    Some(match code.trim().to_ascii_lowercase().as_str() {
        "aut" | "cre" | "dub" => ContributorRole::Author,
        "trl" => ContributorRole::Translator,
        "ill" | "art" | "cov" => ContributorRole::Illustrator,
        "edt" | "edc" | "com" | "pbd" => ContributorRole::Editor,
        "aui" | "aft" | "wpr" | "win" => ContributorRole::Preface,
        "nrt" | "spk" => ContributorRole::Narrator,
        "pbl" | "prt" | "bsl" | "dst" | "fmo" | "dnr" | "own" => return None,
        "" => return Some(ContributorRole::Other("contributor".to_owned())),
        "ctb" | "oth" => ContributorRole::Other("contributor".to_owned()),
        other => ContributorRole::Other(other.to_owned()),
    })
}

/// UNIMARC relator codes (`$4`, three digits; Appendix C of the UNIMARC manual).
fn role_from_unimarc_code(code: &str) -> Option<ContributorRole> {
    Some(match code.trim() {
        "070" | "090" | "330" | "690" => ContributorRole::Author,
        "730" => ContributorRole::Translator,
        "440" | "040" => ContributorRole::Illustrator,
        "340" | "651" | "220" => ContributorRole::Editor,
        "080" | "075" => ContributorRole::Preface,
        "550" => ContributorRole::Narrator,
        // Publisher, printer, distributor, donor, former owner, bookseller, binder.
        "650" | "610" | "310" | "320" | "390" | "160" | "110" => return None,
        "010" => ContributorRole::Other("adapter".to_owned()),
        "100" => ContributorRole::Other("antecedent".to_owned()),
        "205" => ContributorRole::Other("contributor".to_owned()),
        other if other.is_empty() => return Some(ContributorRole::Author),
        other => ContributorRole::Other(format!("unimarc-{other}")),
    })
}

/// Relator terms in words, as catalogues write them in French, German and English.
pub(crate) fn role_from_term(term: &str) -> Option<ContributorRole> {
    let key = fold_key(term);
    if key.is_empty() {
        return None;
    }
    let has = |needles: &[&str]| needles.iter().any(|n| key.contains(n));
    Some(if has(&["ubers", "transl", "traduc", "trad ", "kaant", "oversatt"]) || key == "trad" {
        ContributorRole::Translator
    } else if has(&["illustr", "dessin", "zeichn", "cover", "couverture", "kuvitt"]) {
        ContributorRole::Illustrator
    } else if has(&["vorw", "nachw", "pref", "postf", "introd", "foreword", "afterword", "geleitw"])
    {
        ContributorRole::Preface
    } else if has(&["hrsg", "herausg", "editeur scientifique", "editor", "directeur", "compil", "toimitt"])
        || key == "ed"
    {
        ContributorRole::Editor
    } else if has(&["erzahl", "sprecher", "narrat", "lecteur", "read by"]) {
        ContributorRole::Narrator
    } else if has(&["verf", "author", "auteur", "autor", "kirjoittaja", "forfattare"]) {
        ContributorRole::Author
    } else if has(&["verlag", "publisher", "editeur commercial", "imprimeur", "drucker"]) {
        return None;
    } else {
        ContributorRole::Other(normalize::text(term).trim_end_matches(['.', ',']).to_lowercase())
    })
}

/// Guesses the role of a person from the statement of responsibility (`245 $c`,
/// `200 $f $g`): the words between the previous name and this surname.
fn role_from_statement(statement: &str, surname: &str) -> Option<ContributorRole> {
    let statement_key = fold_key(statement);
    let surname_key = fold_key(surname);
    if surname_key.is_empty() {
        return None;
    }
    let at = statement_key.find(&surname_key)?;
    let before = &statement_key[..at];
    // Keep the last clause: text after the previous " ; " equivalent is lost in the key, so
    // look at the last eight words only.
    let words: Vec<&str> = before.split(' ').filter(|w| !w.is_empty()).collect();
    let tail = words.iter().rev().take(8).rev().copied().collect::<Vec<_>>().join(" ");
    match role_from_term(&tail) {
        Some(ContributorRole::Other(_)) | None => Some(ContributorRole::Author),
        known => known,
    }
}

fn push_contributor(book: &mut BookMeta, name: String, role: ContributorRole) {
    if !name.trim().is_empty() {
        book.contributors.push(Contributor { name, role });
    }
}

fn join_title(main: &str, parts: &[&str]) -> String {
    let mut title = normalize::title(main);
    for part in parts {
        let part = normalize::title(part);
        if !part.is_empty() {
            if !title.is_empty() {
                title.push_str(". ");
            }
            title.push_str(&part);
        }
    }
    title
}

// ---------------------------------------------------------------------------------------
// MARC 21
// ---------------------------------------------------------------------------------------

fn marc21_person(field: &MarcField, default: Option<ContributorRole>, statement: &str) -> Option<Contributor> {
    // An added entry with a title ($t) points at another work, not at a contributor.
    if field.sub('t').is_some() {
        return None;
    }
    let heading = field.sub('a')?;
    let mut name = heading.to_owned();
    if let Some(numeration) = field.sub('b') {
        name = format!("{} {}", name.trim_end_matches(','), numeration.trim_end_matches([',', '.']));
    }
    let coded = field.subs('4').map(role_from_marc_code).collect::<Vec<_>>();
    let role = if let Some(first) = coded.first() {
        // Prefer a recognised role over "contributor" when several codes are given.
        coded
            .iter()
            .flatten()
            .find(|r| !matches!(r, ContributorRole::Other(_)))
            .cloned()
            .or_else(|| first.clone())?
    } else if let Some(term) = field.sub('e') {
        role_from_term(term)?
    } else if let Some(role) = default {
        role
    } else {
        let surname = heading.split(',').next().unwrap_or(heading);
        role_from_statement(statement, surname)
            .unwrap_or_else(|| ContributorRole::Other("contributor".to_owned()))
    };
    Some(Contributor { name, role })
}

/// Maps a MARC 21 bibliographic record (DNB, K10plus, Library of Congress).
pub(crate) fn marc21(record: &MarcRecord) -> Extracted {
    let mut book = BookMeta::default();
    let mut isbns = Vec::new();
    for field in record.fields("020") {
        for value in field.subs('a').chain(field.subs('9')) {
            for isbn in isbns_in(value) {
                push_isbn(&mut isbns, isbn);
            }
        }
    }
    for field in record.fields("024").filter(|f| f.ind1 == '3') {
        for isbn in field.subs('a').flat_map(isbns_in) {
            push_isbn(&mut isbns, isbn);
        }
    }

    let mut statement = String::new();
    if let Some(field) = record.fields("245").next() {
        let parts: Vec<&str> = field.subs('n').chain(field.subs('p')).collect();
        book.title = join_title(field.sub('a').unwrap_or(""), &parts);
        book.subtitle = field.sub('b').map(str::to_owned);
        statement = field.sub('c').unwrap_or("").to_owned();
    }

    for field in record.fields("100").chain(record.fields("110")) {
        if let Some(c) = marc21_person(field, Some(ContributorRole::Author), &statement) {
            book.contributors.push(c);
        }
    }
    for field in record.fields("700") {
        if let Some(c) = marc21_person(field, None, &statement) {
            book.contributors.push(c);
        }
    }

    let publication = record
        .fields("264")
        .find(|f| f.ind2 == '1')
        .or_else(|| record.fields("264").next())
        .or_else(|| record.fields("260").next());
    if let Some(field) = publication {
        book.publisher = field.sub('b').map(str::to_owned);
        book.published = field.subs('c').find(|c| normalize::date(c).is_some()).map(str::to_owned);
    }
    let fixed = record.control("008").unwrap_or("");
    if book.published.is_none() {
        book.published = fixed.get(7..11).and_then(normalize::date);
    }
    book.language = record
        .first("041", 'a')
        // Pre-1980 records concatenate codes: "gerfre".
        .map(|code| code.get(..3).unwrap_or(code).to_owned())
        .or_else(|| fixed.get(35..38).map(str::to_owned))
        .filter(|code| code.chars().all(|c| c.is_ascii_alphabetic()));
    let extent = record.first("300", 'a').unwrap_or("");
    book.page_count = normalize::page_count(extent);
    book.series = record
        .first("490", 'a')
        .or_else(|| record.first("830", 'a'))
        .or_else(|| record.first("800", 't'))
        .map(str::to_owned);
    let carrier = record.first("338", 'b').unwrap_or("");
    if carrier == "cr" || fold_key(extent).contains("online") {
        book.format = Some(BookFormat::Ebook);
    }
    Extracted { book, isbns, url: None }
}

// ---------------------------------------------------------------------------------------
// UNIMARC
// ---------------------------------------------------------------------------------------

fn unimarc_person(field: &MarcField, default: ContributorRole) -> Option<Contributor> {
    let entry = field.sub('a')?;
    let name = match field.sub('b') {
        // $a entry element (surname), $b rest of the name (forename).
        Some(forename) => format!("{} {}", forename.trim_end_matches(','), entry.trim_end_matches(',')),
        None => entry.to_owned(),
    };
    let coded: Vec<Option<ContributorRole>> = field.subs('4').map(role_from_unimarc_code).collect();
    let role = match coded.first() {
        Some(first) => coded
            .iter()
            .flatten()
            .find(|r| !matches!(r, ContributorRole::Other(_)))
            .cloned()
            .or_else(|| first.clone())?,
        None => default,
    };
    Some(Contributor { name, role })
}

/// Maps a UNIMARC bibliographic record (BnF, SUDOC).
pub(crate) fn unimarc(record: &MarcRecord) -> Extracted {
    let mut book = BookMeta::default();
    let mut isbns = Vec::new();
    for value in record.fields("010").chain(record.fields("073")).flat_map(|f| f.subs('a')) {
        for isbn in isbns_in(value) {
            push_isbn(&mut isbns, isbn);
        }
    }
    if let Some(field) = record.fields("200").next() {
        let parts: Vec<&str> = field.subs('h').chain(field.subs('i')).collect();
        book.title = join_title(field.sub('a').unwrap_or(""), &parts);
        let other: Vec<&str> = field.subs('e').collect();
        if !other.is_empty() {
            book.subtitle = Some(other.join(" : "));
        }
        let medium = fold_key(field.sub('b').unwrap_or(""));
        if medium.contains("electronique") || medium.contains("numerique") {
            book.format = Some(BookFormat::Ebook);
        }
    }
    for (tag, default) in [
        ("700", ContributorRole::Author),
        ("701", ContributorRole::Author),
        ("702", ContributorRole::Other("contributor".to_owned())),
    ] {
        for field in record.fields(tag) {
            if let Some(c) = unimarc_person(field, default.clone()) {
                book.contributors.push(c);
            }
        }
    }
    // Corporate bodies only when they are coded as authors.
    for field in record.fields("710").chain(record.fields("711")) {
        if field.subs('4').any(|code| code.trim() == "070")
            && let Some(name) = field.sub('a')
        {
            push_contributor(&mut book, name.to_owned(), ContributorRole::Author);
        }
    }
    let publication = record
        .fields("214")
        .find(|f| f.ind2 == '0')
        .or_else(|| record.fields("214").next())
        .or_else(|| record.fields("210").next())
        .or_else(|| record.fields("219").next());
    if let Some(field) = publication {
        book.publisher = field.sub('c').map(str::to_owned);
        book.published = field.subs('d').find(|d| normalize::date(d).is_some()).map(str::to_owned);
    }
    if book.published.is_none() {
        // 100 $a positions 9-12: first date of publication.
        book.published = record.first("100", 'a').and_then(|a| a.get(9..13)).and_then(normalize::date);
    }
    book.language = record.first("101", 'a').map(str::to_owned);
    book.page_count = record.first("215", 'a').and_then(normalize::page_count);
    book.series = record
        .first("225", 'a')
        .or_else(|| record.first("410", 't'))
        .or_else(|| record.first("461", 't'))
        .map(str::to_owned);
    if record.fields("135").next().is_some() && book.format.is_none() {
        book.format = Some(BookFormat::Ebook);
    }
    let url = record
        .control("003")
        .map(str::trim)
        .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
        .map(str::to_owned);
    Extracted { book, isbns, url }
}

// ---------------------------------------------------------------------------------------
// Dublin Core
// ---------------------------------------------------------------------------------------

/// Splits a BnF Dublin Core creator, `"Camus, Albert (1913-1960). Auteur du texte"`, into
/// the heading and the role.
fn dc_creator(value: &str) -> (String, Option<ContributorRole>) {
    let split = value
        .rfind("). ")
        .map(|i| (i + 1, i + 3))
        .or_else(|| value.rfind(". ").map(|i| (i, i + 2)));
    if let Some((name_end, role_start)) = split
        && let (Some(name), Some(role)) = (value.get(..name_end), value.get(role_start..))
        && !role.contains(',')
        && role.split_whitespace().count() <= 4
        && role.chars().next().is_some_and(char::is_uppercase)
        // "Tolkien, J. R. R." ends with an initial, not with a role.
        && role.split_whitespace().next().is_some_and(|w| w.chars().count() >= 4)
        && name.chars().count() > 3
        && let Some(parsed) = role_from_term(role)
    {
        return (name.to_owned(), Some(parsed));
    }
    (value.to_owned(), None)
}

/// Maps an `oai_dc:dc` (or SRU `dc`) element.
pub(crate) fn dublin_core(dc: &Node) -> Extracted {
    let mut book = BookMeta::default();
    let mut isbns = Vec::new();
    let mut url = None;
    let values = |name: &'static str| {
        dc.children_named(name).map(|n| n.text.trim()).filter(|t| !t.is_empty())
    };
    book.title = values("title").next().unwrap_or("").to_owned();
    for (element, default) in [
        ("creator", ContributorRole::Author),
        ("contributor", ContributorRole::Other("contributor".to_owned())),
    ] {
        for value in values(element) {
            let (name, role) = dc_creator(value);
            push_contributor(&mut book, name, role.unwrap_or_else(|| default.clone()));
        }
    }
    book.publisher = values("publisher").next().map(str::to_owned);
    book.published = values("date").find(|d| normalize::date(d).is_some()).map(str::to_owned);
    book.language = values("language").next().map(str::to_owned);
    for value in values("identifier") {
        if (value.starts_with("http://") || value.starts_with("https://")) && isbns_in(value).is_empty()
        {
            url.get_or_insert_with(|| value.to_owned());
        } else {
            for isbn in isbns_in(value) {
                push_isbn(&mut isbns, isbn);
            }
        }
    }
    book.page_count =
        values("format").chain(values("extent")).find_map(normalize::page_count);
    book.series = values("relation").chain(values("description")).find_map(|relation| {
        let rest = relation
            .strip_prefix("Appartient à : ")
            .or_else(|| relation.strip_prefix("Collection : "))?;
        Some(rest.to_owned())
    });
    Extracted { book, isbns, url }
}

// ---------------------------------------------------------------------------------------
// Choosing among several records
// ---------------------------------------------------------------------------------------

/// Picks the record to keep for `isbn`: records about another ISBN are dropped (and
/// mentioned in `notes`); among the rest, one that carries the ISBN beats one without any
/// ISBN, then the most recent publication date wins (research implication 9).
pub(crate) fn select(
    isbn: &Isbn13,
    candidates: Vec<Extracted>,
    notes: &mut Vec<String>,
) -> Option<Extracted> {
    let mut best: Option<(u8, u16, Extracted)> = None;
    for candidate in candidates {
        let exact = candidate.isbns.contains(isbn);
        if !exact && !candidate.isbns.is_empty() {
            notes.push(format!(
                "ignored a record about another ISBN ({})",
                candidate.isbns.iter().map(Isbn13::to_plain).collect::<Vec<_>>().join(", ")
            ));
            continue;
        }
        if candidate.book.title.trim().is_empty() {
            notes.push("ignored a record without a title".to_owned());
            continue;
        }
        if !exact {
            notes.push("kept a record that carries no ISBN".to_owned());
        }
        let year = candidate
            .book
            .published
            .as_deref()
            .and_then(normalize::date)
            .and_then(|d| d.get(..4).and_then(|y| y.parse::<u16>().ok()))
            .unwrap_or(0);
        let rank = u8::from(exact);
        if best.as_ref().is_none_or(|(r, y, _)| (rank, year) > (*r, *y)) {
            best = Some((rank, year, candidate));
        }
    }
    best.map(|(_, _, chosen)| chosen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_from_codes_terms_and_statements() {
        assert_eq!(role_from_marc_code("trl"), Some(ContributorRole::Translator));
        assert_eq!(role_from_marc_code("pbl"), None);
        assert_eq!(role_from_unimarc_code("070"), Some(ContributorRole::Author));
        assert_eq!(role_from_unimarc_code("730"), Some(ContributorRole::Translator));
        assert_eq!(role_from_term("Übersetzer"), Some(ContributorRole::Translator));
        assert_eq!(role_from_term("Auteur du texte"), Some(ContributorRole::Author));
        assert_eq!(role_from_term("Verfasser"), Some(ContributorRole::Author));
        assert_eq!(role_from_term("Préfacier"), Some(ContributorRole::Preface));
        assert_eq!(role_from_term("Herausgeber"), Some(ContributorRole::Editor));
        assert_eq!(role_from_term("Verlag"), None);
        assert_eq!(
            role_from_statement("Albert Camus ; translated from the French by Matthew Ward", "Ward"),
            Some(ContributorRole::Translator)
        );
        assert_eq!(
            role_from_statement("Terry Pratchett and Neil Gaiman", "Gaiman"),
            Some(ContributorRole::Author)
        );
        assert_eq!(role_from_statement("Albert Camus", "Ward"), None);
    }

    #[test]
    fn dublin_core_creators_split_name_and_role() {
        let (name, role) = dc_creator("Camus, Albert (1913-1960). Auteur du texte");
        assert_eq!(name, "Camus, Albert (1913-1960)");
        assert_eq!(role, Some(ContributorRole::Author));
        let (name, role) = dc_creator("Tolkien, J. R. R.");
        assert_eq!(name, "Tolkien, J. R. R.");
        assert_eq!(role, None);
    }
}
