//! Cleaning of catalogue artefacts.
//!
//! Library catalogues describe books with ISBD punctuation and MARC conventions: titles
//! carry a medium designation (`"[Texte imprimé]"`) and a statement of responsibility
//! (`"L'étranger / Albert Camus"`), names are inverted and dated
//! (`"Camus, Albert (1913-1960)"`), dates are qualified (`"DL 2015"`, `"cop. 1942"`),
//! extents are sentences (`"1 vol. (185 p.)"`), languages are three-letter codes. The
//! functions here turn those into what a person would write on a card. All of them are
//! total (no panics) and idempotent.

use ebc_model::{BookMeta, Contributor, ContributorRole};

use crate::compose_table::COMPOSITIONS;

// ---------------------------------------------------------------------------------------
// Unicode helpers
// ---------------------------------------------------------------------------------------

fn is_combining_mark(ch: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&ch)
}

fn compose_pair(base: char, mark: char) -> Option<char> {
    COMPOSITIONS.iter().find(|(_, b, m)| *b == base && *m == mark).map(|(c, _, _)| *c)
}

fn decompose_base(ch: char) -> Option<char> {
    COMPOSITIONS.binary_search_by(|(c, _, _)| c.cmp(&ch)).ok().map(|i| COMPOSITIONS[i].1)
}

/// Composes "letter + combining mark" sequences into single characters
/// (`"e\u{301}"` → `"é"`), as Unicode NFC does, for Latin, Greek and Cyrillic letters.
///
/// Catalogues converted from MARC-8 (Library of Congress) deliver decomposed accents, which
/// many fonts render badly and which defeat string comparison.
#[must_use]
pub fn compose(input: &str) -> String {
    if !input.chars().any(is_combining_mark) {
        return input.to_owned();
    }
    let mut out: Vec<char> = Vec::with_capacity(input.len());
    // Index in `out` of the last starter, as long as every mark after it was composed.
    let mut starter: Option<usize> = None;
    for ch in input.chars() {
        if is_combining_mark(ch) {
            if let Some(i) = starter
                && i + 1 == out.len()
                && let Some(composed) = out.get(i).and_then(|b| compose_pair(*b, ch))
            {
                out[i] = composed;
                continue;
            }
            out.push(ch);
        } else {
            out.push(ch);
            starter = Some(out.len() - 1);
        }
    }
    out.into_iter().collect()
}

/// A comparison key: lower case, accents and punctuation removed, ligatures expanded,
/// words separated by single spaces. `"L'Étranger"` and `"l etranger"` have the same key.
#[must_use]
pub fn fold_key(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pending_space = false;
    for ch in compose(input).chars() {
        for lower in ch.to_lowercase() {
            let mut base = lower;
            for _ in 0..4 {
                match decompose_base(base) {
                    Some(b) => base = b,
                    None => break,
                }
            }
            let expanded: &str = match base {
                'ß' => "ss",
                'œ' => "oe",
                'æ' => "ae",
                'ø' => "o",
                'đ' | 'ð' => "d",
                'ł' => "l",
                'ı' => "i",
                'þ' => "th",
                _ => "",
            };
            if !expanded.is_empty() {
                if pending_space && !out.is_empty() {
                    out.push(' ');
                }
                pending_space = false;
                out.push_str(expanded);
            } else if base.is_alphanumeric() {
                if pending_space && !out.is_empty() {
                    out.push(' ');
                }
                pending_space = false;
                out.push(base);
            } else if !is_combining_mark(base) {
                pending_space = true;
            }
        }
    }
    out
}

/// Repairs text that was UTF-8 but got decoded as Windows-1252 (`"L'Ã‰tranger"` →
/// `"L'Étranger"`). Text that is not such mojibake is returned unchanged.
#[must_use]
pub fn fix_mojibake(input: &str) -> String {
    if !input.contains(['Ã', 'Â', 'â', 'Å', 'Ä', 'Ð', 'Î', 'Ï']) {
        return input.to_owned();
    }
    let mut bytes = Vec::with_capacity(input.len());
    for ch in input.chars() {
        match cp1252_byte(ch) {
            Some(b) => bytes.push(b),
            None => return input.to_owned(),
        }
    }
    match String::from_utf8(bytes) {
        Ok(fixed)
            if fixed.chars().count() < input.chars().count()
                && !fixed.chars().any(|c| ('\u{80}'..='\u{9F}').contains(&c)) =>
        {
            fixed
        }
        _ => input.to_owned(),
    }
}

/// Windows-1252 code points 0x80–0x9F, in order (`'\0'` where the byte is unassigned).
pub(crate) const CP1252_HIGH: [char; 32] = [
    '€', '\0', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\0', 'Ž', '\0', '\0', '‘',
    '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\0', 'ž', 'Ÿ',
];

fn cp1252_byte(ch: char) -> Option<u8> {
    let cp = u32::from(ch);
    if cp < 0x80 || (0xA0..=0xFF).contains(&cp) {
        return u8::try_from(cp).ok();
    }
    if (0x80..=0x9F).contains(&cp) {
        // C1 controls: bytes that Windows-1252 leaves unassigned decode to themselves.
        return u8::try_from(cp).ok();
    }
    CP1252_HIGH
        .iter()
        .position(|c| *c == ch && *c != '\0')
        .and_then(|i| u8::try_from(0x80 + i).ok())
}

/// Composes accents, repairs mojibake, removes control characters and non-sorting marks,
/// and collapses white space. Line breaks are kept when `keep_newlines` is set.
fn tidy(input: &str, keep_newlines: bool) -> String {
    let fixed = fix_mojibake(&compose(input));
    let mut out = String::with_capacity(fixed.len());
    let mut pending: Option<char> = None;
    for ch in fixed.chars() {
        let ch = match ch {
            '\n' if keep_newlines => '\n',
            // No-break and thin spaces are layout, not content.
            '\u{00A0}' | '\u{2007}' | '\u{202F}' | '\u{2009}' => ' ',
            // Soft hyphen, zero-width characters, BOM, replacement of nothing useful.
            '\u{00AD}' | '\u{200B}' | '\u{FEFF}' => continue,
            c if c.is_whitespace() => ' ',
            c if c.is_control() => continue,
            c => c,
        };
        if ch == ' ' || ch == '\n' {
            pending = match (pending, ch) {
                (Some('\n'), _) | (_, '\n') => Some('\n'),
                _ => Some(' '),
            };
            continue;
        }
        if let Some(sep) = pending.take()
            && !out.is_empty()
        {
            out.push(sep);
        }
        out.push(ch);
    }
    out
}

/// Plain one-line text: accents composed, control characters and redundant spaces removed.
#[must_use]
pub fn text(input: &str) -> String {
    tidy(input, false)
}

// ---------------------------------------------------------------------------------------
// Titles
// ---------------------------------------------------------------------------------------

/// General material designations that catalogues insert into titles.
const GMD: &[&str] = &[
    "texte imprime",
    "texte",
    "texte manuscrit",
    "ressource electronique",
    "document electronique",
    "livre",
    "livre numerique",
    "enregistrement sonore",
    "images animees",
    "image fixe",
    "musique imprimee",
    "document cartographique",
    "multimedia multisupport",
    "microforme",
    "electronic resource",
    "computer file",
    "sound recording",
    "videorecording",
    "microform",
    "text",
    "text large print",
    "large print",
    "braille",
    "elektronische ressource",
    "tontrager",
    "online ressource",
    "computerdatei",
    "mikroform",
    "recurso electronico",
    "texto impreso",
    "risorsa elettronica",
    "testo a stampa",
    "elektronisk resurs",
];

fn is_gmd(inner: &str) -> bool {
    let key = fold_key(inner);
    GMD.contains(&key.as_str())
}

/// Removes non-sorting marks: the C1 controls NSB/NSE (U+0098/U+009C, U+0088/U+0089),
/// `<<…>>`, `¬…¬`, the `@` filing marker of PICA catalogues, and the `˜…œ` pair those
/// controls become when a Latin-1 response is decoded as Windows-1252.
fn strip_nonsorting(input: &str) -> String {
    let mut s: String = input
        .chars()
        .filter(|c| !matches!(c, '\u{0098}' | '\u{009C}' | '\u{0088}' | '\u{0089}' | '¬'))
        .collect();
    if s.contains("<<") && s.contains(">>") {
        s = s.replace("<<", "").replace(">>", "");
    }
    if let Some(start) = s.find('˜') {
        let after = start + '˜'.len_utf8();
        if let Some(rel) = s[after..].find('œ') {
            let end = after + rel;
            let mut fixed = String::with_capacity(s.len());
            fixed.push_str(&s[..start]);
            fixed.push_str(&s[after..end]);
            fixed.push_str(&s[end + 'œ'.len_utf8()..]);
            s = fixed;
        } else {
            s.remove(start);
        }
    }
    // PICA "@": marks the first filing word. Only when it directly precedes a letter and
    // sits at the start or after a space or apostrophe ("L'@étranger", "Der @Zauberberg").
    if let Some(at) = s.find('@') {
        let before = s[..at].chars().next_back();
        let after = s[at + 1..].chars().next();
        let boundary = before.is_none_or(|c| c == ' ' || c == '\'' || c == '’');
        if boundary && after.is_some_and(char::is_alphanumeric) && !s[at + 1..].contains('@') {
            s.remove(at);
        }
    }
    s
}

/// Removes bracketed medium designations (`"[Texte imprimé]"`, `"[electronic resource]"`).
/// Other bracketed text is kept.
fn strip_gmd(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(open) = rest.find('[') {
        let Some(close_rel) = rest[open..].find(']') else { break };
        let close = open + close_rel;
        let inner = &rest[open + 1..close];
        if is_gmd(inner) {
            out.push_str(rest[..open].trim_end());
            out.push(' ');
        } else {
            out.push_str(&rest[..=close]);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Words after which a final full stop is part of the word.
const ABBREVIATIONS: &[&str] = &[
    "verl", "ed", "éd", "eds", "inc", "ltd", "co", "corp", "publ", "pr", "univ", "libr", "impr",
    "cie", "bros", "gmbh", "etc", "jr", "sr", "st", "dr", "mr", "mrs", "vol", "hrsg", "aufl",
    "bd", "nr", "no", "ill", "coll", "trad", "pref", "préf", "u.a",
];

/// Removes ISBD separators left at the end of a subfield (`" /"`, `" :"`, `","`…) and a
/// final full stop that is not part of an abbreviation or an ellipsis.
fn strip_trailing_punctuation(input: &str) -> &str {
    let mut s = input.trim();
    loop {
        let trimmed = s.trim_end_matches([',', ';', ':', '/', '=']).trim_end();
        let trimmed = match trimmed.strip_suffix('.') {
            Some(body) if !body.ends_with('.') => {
                let last_word: String = body
                    .rsplit(|c: char| c.is_whitespace() || c == '-')
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                let letters = last_word.chars().filter(|c| c.is_alphabetic()).count();
                let keep = (1..3).contains(&letters)
                    || ABBREVIATIONS.contains(&last_word.as_str())
                    || last_word.contains('.');
                if keep { trimmed } else { body.trim_end() }
            }
            _ => trimmed,
        };
        if trimmed.len() == s.len() {
            return s;
        }
        s = trimmed;
    }
}

/// Cleans a title: removes non-sorting marks, medium designations, the statement of
/// responsibility after `" / "`, a parallel title after `" = "` and trailing ISBD
/// punctuation.
///
/// ```
/// use ebc_catalog::normalize::title;
/// assert_eq!(title("L'étranger [Texte imprimé] / Albert Camus"), "L'étranger");
/// assert_eq!(title("\u{98}Die \u{9c}Verwandlung"), "Die Verwandlung");
/// assert_eq!(title("<<Der>> Zauberberg :"), "Der Zauberberg");
/// assert_eq!(title("Pride and prejudice /"), "Pride and prejudice");
/// ```
#[must_use]
pub fn title(input: &str) -> String {
    let s = tidy(input, false);
    let s = strip_nonsorting(&s);
    let s = strip_gmd(&s);
    let s = tidy(&s, false);
    let mut cut = s.as_str();
    if let Some(i) = cut.find(" / ") {
        cut = &cut[..i];
    }
    if let Some(i) = cut.find(" = ") {
        cut = &cut[..i];
    }
    strip_trailing_punctuation(cut).to_owned()
}

/// Splits `"Title : other title information"` (ISBD, and French typography) into title and
/// subtitle. Input is cleaned with [`title`] first.
#[must_use]
pub fn split_title(input: &str) -> (String, Option<String>) {
    let cleaned = title(input);
    match cleaned.split_once(" : ") {
        Some((main, sub)) if !main.trim().is_empty() && !sub.trim().is_empty() => {
            (strip_trailing_punctuation(main).to_owned(), Some(sub.trim().to_owned()))
        }
        _ => (cleaned, None),
    }
}

// ---------------------------------------------------------------------------------------
// Names
// ---------------------------------------------------------------------------------------

const NAME_SUFFIXES: &[&str] = &["jr", "jr.", "sr", "sr.", "ii", "iii", "iv", "fils", "père"];

fn has_digit_or_gap(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit()) || s.contains("....") || s.contains("..-")
}

/// Removes parenthesised groups that hold dates (`"(1913-1960)"`, `"(19..-....)"`).
fn strip_date_parentheses(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(open) = rest.find('(') {
        let Some(close_rel) = rest[open..].find(')') else { break };
        let close = open + close_rel;
        if has_digit_or_gap(&rest[open + 1..close]) {
            out.push_str(rest[..open].trim_end());
        } else {
            out.push_str(&rest[..=close]);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Turns a catalogue heading into a display name in natural order.
///
/// ```
/// use ebc_catalog::normalize::person_name;
/// assert_eq!(person_name("Camus, Albert (1913-1960)"), "Albert Camus");
/// assert_eq!(person_name("Austen, Jane, 1775-1817."), "Jane Austen");
/// assert_eq!(person_name("La Fontaine, Jean de"), "Jean de La Fontaine");
/// assert_eq!(person_name("Albert Camus"), "Albert Camus");
/// ```
#[must_use]
pub fn person_name(input: &str) -> String {
    let s = strip_nonsorting(&tidy(input, false));
    let s = strip_date_parentheses(&s);
    let segments: Vec<&str> = s
        .split(',')
        .map(str::trim)
        .enumerate()
        // Later segments holding digits are dates ("1913-1960", "1952-").
        .filter(|(i, seg)| !seg.is_empty() && (*i == 0 || !has_digit_or_gap(seg)))
        .map(|(_, seg)| strip_name_end(seg))
        .filter(|seg| !seg.is_empty())
        .collect();
    let short = |seg: &str| seg.split_whitespace().count() <= 5;
    let is_suffix = |seg: &str| NAME_SUFFIXES.contains(&seg.to_lowercase().as_str());
    let joined = match segments.as_slice() {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [last, suffix] if is_suffix(suffix) => format!("{last} {suffix}"),
        [last, first] if short(last) && short(first) => format!("{first} {last}"),
        [last, first, suffix] if is_suffix(suffix) && short(last) && short(first) => {
            format!("{first} {last} {suffix}")
        }
        _ => segments.join(", "),
    };
    tidy(&joined, false)
}

fn strip_name_end(segment: &str) -> &str {
    let s = segment.trim().trim_end_matches([',', ';', ':']).trim_end();
    match s.strip_suffix('.') {
        Some(body) => {
            let last = body.rsplit(|c: char| c.is_whitespace() || c == '.').next().unwrap_or("");
            // Keep the stop of initials ("J. R. R.") and of short suffixes ("Jr.").
            if last.chars().count() >= 3 { body.trim_end() } else { s }
        }
        None => s,
    }
}

// ---------------------------------------------------------------------------------------
// Publisher, series
// ---------------------------------------------------------------------------------------

const NO_PUBLISHER: &[&str] = &[
    "s n",
    "sn",
    "s l",
    "s l s n",
    "sans nom",
    "o v",
    "ohne verlag",
    "publisher not identified",
    "editeur non identifie",
    "not identified",
    "unknown",
    "n a",
    "none",
    "null",
];

fn contains_year(s: &str) -> bool {
    year_candidates(s).next().is_some()
}

/// Extracts the publisher name from a publication statement.
///
/// ```
/// use ebc_catalog::normalize::publisher;
/// assert_eq!(publisher("[Paris] : Gallimard").as_deref(), Some("Gallimard"));
/// assert_eq!(publisher("Paris : Gallimard, DL 2015").as_deref(), Some("Gallimard"));
/// assert_eq!(publisher("Gallimard ([Paris])").as_deref(), Some("Gallimard"));
/// assert_eq!(publisher("Penguin Books,").as_deref(), Some("Penguin Books"));
/// assert_eq!(publisher("[s.n.]"), None);
/// ```
#[must_use]
pub fn publisher(input: &str) -> Option<String> {
    let s = strip_nonsorting(&tidy(input, false));
    // ISBD: "place ; place : publisher : publisher, date ; place : publisher".
    let mut part = match s.split_once(" : ") {
        Some((_, after)) => after,
        None => s.as_str(),
    };
    for sep in [" ; ", " : "] {
        if let Some(i) = part.find(sep) {
            part = &part[..i];
        }
    }
    // "Gallimard, 1971" / "Gallimard, impr. 2011, cop. 1942": drop the dates.
    while let Some(i) = part.rfind(", ")
        && contains_year(&part[i..])
    {
        part = &part[..i];
    }
    let mut name = part.trim().to_owned();
    // "Gallimard ([Paris])" / "Hachette (Paris)": drop the place.
    if name.ends_with(')')
        && let Some(open) = name.rfind('(')
    {
        let inner = &name[open + 1..name.len() - 1];
        let wordy = inner.split_whitespace().count() <= 3 && !inner.chars().any(|c| c.is_ascii_digit());
        if open > 0 && wordy {
            name.truncate(open);
        }
    }
    let name: String = name.chars().filter(|c| !matches!(c, '[' | ']')).collect();
    let name = strip_trailing_punctuation(&name).to_owned();
    let key = fold_key(&name);
    if key.is_empty() || NO_PUBLISHER.contains(&key.as_str()) {
        return None;
    }
    Some(name)
}

/// Cleans a series (collection) statement: drops the volume number and a leading
/// `"Collection"`.
///
/// ```
/// use ebc_catalog::normalize::series;
/// assert_eq!(series("Collection Folio ; 2").as_deref(), Some("Folio"));
/// assert_eq!(series("Penguin classics").as_deref(), Some("Penguin classics"));
/// assert_eq!(series("Reclams Universal-Bibliothek ; Nr. 9900").as_deref(), Some("Reclams Universal-Bibliothek"));
/// ```
#[must_use]
pub fn series(input: &str) -> Option<String> {
    let s = strip_nonsorting(&tidy(input, false));
    let mut part = s.trim();
    // "(Folio ; 2)" → "Folio ; 2"
    if part.len() >= 2 && part.starts_with('(') && part.ends_with(')') {
        part = part[1..part.len() - 1].trim();
    }
    for sep in [" ; ", " -- ", " ;"] {
        if let Some(i) = part.find(sep) {
            part = &part[..i];
        }
    }
    let part = part.trim_start_matches([';', ',', ':', ' ']);
    let mut name = strip_trailing_punctuation(part).to_owned();
    // "Folio, 2" / "Fischer Nr. 9431"
    if let Some(i) = name.rfind(' ') {
        let tail = name[i + 1..].trim();
        let head_raw = name[..i].trim_end();
        let comma = head_raw.ends_with(',');
        let head = head_raw.trim_end_matches([',', ' ']);
        let head_last = head.rsplit(' ').next().unwrap_or("").to_lowercase();
        let numbered = !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit());
        let marker = matches!(
            head_last.trim_end_matches('.'),
            "n°" | "no" | "nr" | "bd" | "vol" | "t" | "tome" | "band" | "#" | "numéro" | "volume"
        );
        if numbered && marker {
            let cut = head.rfind(' ').map_or("", |j| &head[..j]);
            name = cut.trim_end_matches([',', ' ', ';']).to_owned();
        } else if numbered && comma && !head.is_empty() {
            name = head.to_owned();
        }
    }
    for prefix in ["Collection ", "Coll. ", "collection ", "coll. "] {
        if let Some(rest) = name.strip_prefix(prefix)
            && rest.chars().next().is_some_and(|c| c.is_uppercase() || c.is_ascii_digit())
        {
            name = rest.to_owned();
            break;
        }
    }
    let name = strip_trailing_punctuation(&name).to_owned();
    if fold_key(&name).is_empty() || name.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(name)
    }
}

// ---------------------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------------------

/// How a year is qualified in a publication statement; lower is closer to "published".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum YearKind {
    Plain,
    LegalDeposit,
    Printing,
    Copyright,
}

/// Four-digit years (1000–2100) that are not part of a longer number, with the byte
/// offset where each starts.
fn year_candidates(s: &str) -> impl Iterator<Item = (usize, u16)> + '_ {
    let bytes = s.as_bytes();
    let mut i = 0;
    std::iter::from_fn(move || {
        while i < bytes.len() {
            if bytes[i].is_ascii_digit() {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i - start == 4 {
                    let year = bytes[start..i]
                        .iter()
                        .fold(0u16, |acc, b| acc * 10 + u16::from(b - b'0'));
                    if (1000..=2100).contains(&year) {
                        return Some((start, year));
                    }
                }
            } else {
                i += 1;
            }
        }
        None
    })
}

fn year_kind(before: &str) -> YearKind {
    let lowered = before.to_lowercase();
    let word: String = lowered
        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '©' && c != '℗')
        .chars()
        .rev()
        .take_while(|c| c.is_alphabetic() || matches!(c, '.' | '©' | '℗'))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let word = word.trim_matches('.');
    match word {
        "c" | "cop" | "copyright" | "©" | "p" | "℗" => YearKind::Copyright,
        "impr" | "réimpr" | "reimpr" | "printing" | "printed" | "tirage" | "druck" | "print" => {
            YearKind::Printing
        }
        "dl" | "d.l" | "légal" | "legal" => YearKind::LegalDeposit,
        _ if word.ends_with('©') => YearKind::Copyright,
        _ => YearKind::Plain,
    }
}

const MONTHS: &[(&str, u8)] = &[
    ("january", 1), ("jan", 1), ("janvier", 1), ("janv", 1), ("januar", 1), ("enero", 1),
    ("gennaio", 1), ("januari", 1), ("janeiro", 1),
    ("february", 2), ("feb", 2), ("février", 2), ("fevrier", 2), ("févr", 2), ("fevr", 2),
    ("februar", 2), ("febrero", 2), ("febbraio", 2), ("februari", 2), ("fevereiro", 2),
    ("march", 3), ("mar", 3), ("mars", 3), ("märz", 3), ("marz", 3), ("marzo", 3), ("maart", 3),
    ("março", 3),
    ("april", 4), ("apr", 4), ("avril", 4), ("avr", 4), ("abril", 4), ("aprile", 4),
    ("may", 5), ("mai", 5), ("mayo", 5), ("maggio", 5), ("mei", 5), ("maio", 5),
    ("june", 6), ("jun", 6), ("juin", 6), ("juni", 6), ("junio", 6), ("giugno", 6), ("junho", 6),
    ("july", 7), ("jul", 7), ("juillet", 7), ("juil", 7), ("juli", 7), ("julio", 7),
    ("luglio", 7), ("julho", 7),
    ("august", 8), ("aug", 8), ("août", 8), ("aout", 8), ("agosto", 8), ("augustus", 8),
    ("september", 9), ("sep", 9), ("sept", 9), ("septembre", 9), ("septiembre", 9),
    ("settembre", 9), ("setembro", 9),
    ("october", 10), ("oct", 10), ("octobre", 10), ("oktober", 10), ("okt", 10), ("octubre", 10),
    ("ottobre", 10), ("outubro", 10),
    ("november", 11), ("nov", 11), ("novembre", 11), ("noviembre", 11), ("novembro", 11),
    ("december", 12), ("dec", 12), ("décembre", 12), ("decembre", 12), ("déc", 12),
    ("dezember", 12), ("dez", 12), ("diciembre", 12), ("dicembre", 12), ("dezembro", 12),
];

fn month_number(word: &str) -> Option<u8> {
    let w = word.trim_matches('.').to_lowercase();
    MONTHS.iter().find(|(name, _)| *name == w).map(|(_, n)| *n)
}

fn iso(year: u16, month: Option<u8>, day: Option<u8>) -> String {
    match (month, day) {
        (Some(m), Some(d)) if (1..=12).contains(&m) && (1..=31).contains(&d) => {
            format!("{year:04}-{m:02}-{d:02}")
        }
        (Some(m), _) if (1..=12).contains(&m) => format!("{year:04}-{m:02}"),
        _ => format!("{year:04}"),
    }
}

fn small_number(token: &str, max: u8) -> Option<u8> {
    let t = token.trim_matches(|c: char| !c.is_ascii_digit());
    if t.is_empty() || t.len() > 2 || !t.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    t.parse::<u8>().ok().filter(|n| (1..=max).contains(n))
}

/// Turns a date as catalogues write it into a partial ISO 8601 date (`"1942"`,
/// `"1942-05"`, `"1942-05-19"`), or `None` when no year can be read.
///
/// When several years are present the publication year wins over a legal-deposit year,
/// which wins over a printing year, which wins over a copyright year.
///
/// ```
/// use ebc_catalog::normalize::date;
/// assert_eq!(date("DL 2015").as_deref(), Some("2015"));
/// assert_eq!(date("cop. 1942").as_deref(), Some("1942"));
/// assert_eq!(date("[1972]").as_deref(), Some("1972"));
/// assert_eq!(date("impr. 2011, cop. 1942").as_deref(), Some("2011"));
/// assert_eq!(date("March 2009").as_deref(), Some("2009-03"));
/// assert_eq!(date("5 janvier 1972").as_deref(), Some("1972-01-05"));
/// assert_eq!(date("2012-12-06T00:00:00Z").as_deref(), Some("2012-12-06"));
/// assert_eq!(date("19.."), None);
/// ```
#[must_use]
pub fn date(input: &str) -> Option<String> {
    let s = tidy(input, false);
    let s = s.trim();
    let b = s.as_bytes();
    let digits = |from: usize, len: usize| -> Option<u16> {
        let slice = b.get(from..from + len)?;
        if !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        Some(slice.iter().fold(0u16, |acc, d| acc * 10 + u16::from(d - b'0')))
    };
    // ISO: YYYY, YYYY-MM, YYYY-MM-DD (optionally followed by a time).
    if let Some(year) = digits(0, 4).filter(|y| (1000..=2100).contains(y)) {
        let after_year = b.get(4).copied();
        if after_year.is_none() {
            return Some(iso(year, None, None));
        }
        if after_year == Some(b'-')
            && let Some(month) = digits(5, 2).and_then(|m| u8::try_from(m).ok())
            && (1..=12).contains(&month)
        {
            let after_month = b.get(7).copied();
            if after_month.is_none() {
                return Some(iso(year, Some(month), None));
            }
            if after_month == Some(b'-')
                && let Some(day) = digits(8, 2).and_then(|d| u8::try_from(d).ok())
                && (1..=31).contains(&day)
                && !b.get(10).is_some_and(u8::is_ascii_digit)
            {
                return Some(iso(year, Some(month), Some(day)));
            }
        }
    }
    // Numeric day-first forms: "05.01.1972", "25/12/1999".
    let parts: Vec<&str> = s.split(['.', '/']).map(str::trim).collect();
    if let [d, m, y] = parts.as_slice()
        && y.len() == 4
        && let Ok(year) = y.parse::<u16>()
        && (1000..=2100).contains(&year)
        && let (Some(a), Some(c)) = (small_number(d, 31), small_number(m, 31))
        && d.bytes().all(|x| x.is_ascii_digit())
        && m.bytes().all(|x| x.is_ascii_digit())
    {
        return Some(match (a, c) {
            (day, month) if month <= 12 && (day > 12 || s.contains('.')) => {
                iso(year, Some(month), Some(day))
            }
            (month, day) if month <= 12 && day > 12 => iso(year, Some(month), Some(day)),
            _ => iso(year, None, None),
        });
    }
    // Month names: "March 2009", "Jan 05, 2010", "5 janvier 1972", "5. Januar 1972".
    let tokens: Vec<&str> =
        s.split(|c: char| c.is_whitespace() || c == ',').filter(|t| !t.is_empty()).collect();
    if let Some(mi) = tokens.iter().position(|t| month_number(t).is_some())
        && let Some(month) = tokens.get(mi).and_then(|t| month_number(t))
    {
        let year = year_candidates(s).next().map(|(_, y)| y);
        if let Some(year) = year {
            let before = mi.checked_sub(1).and_then(|i| tokens.get(i)).and_then(|t| small_number(t, 31));
            let after = tokens.get(mi + 1).and_then(|t| small_number(t, 31));
            return Some(iso(year, Some(month), before.or(after)));
        }
    }
    // Qualified years: "DL 2015", "cop. 1942", "[1972]", "impr. 2011".
    let mut best: Option<(YearKind, u16)> = None;
    let mut previous_end = 0;
    for (start, year) in year_candidates(s) {
        let kind = year_kind(s.get(previous_end..start).unwrap_or(""));
        if best.is_none_or(|(k, _)| kind < k) {
            best = Some((kind, year));
        }
        previous_end = start + 4;
    }
    best.map(|(_, year)| iso(year, None, None))
}

// ---------------------------------------------------------------------------------------
// Extent
// ---------------------------------------------------------------------------------------

const PAGE_UNITS: &[&str] = &[
    "p", "pp", "page", "pages", "s", "seite", "seiten", "sivua", "sidor", "sider", "blz", "pag",
    "pags", "pagine", "paginas", "str", "stron",
];

/// Reads a page count out of an extent statement.
///
/// ```
/// use ebc_catalog::normalize::page_count;
/// assert_eq!(page_count("1 vol. (185 p.)"), Some(185));
/// assert_eq!(page_count("xli, 435 p. ;"), Some(435));
/// assert_eq!(page_count("1 Online-Ressource (224 Seiten)"), Some(224));
/// assert_eq!(page_count("185"), Some(185));
/// assert_eq!(page_count("2 vol."), None);
/// ```
#[must_use]
pub fn page_count(input: &str) -> Option<u32> {
    let s = tidy(input, false);
    let trimmed = s.trim();
    if !trimmed.is_empty() && trimmed.len() <= 5 && trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return trimmed.parse::<u32>().ok().filter(|n| *n > 0);
    }
    let chars: Vec<char> = s.chars().collect();
    let mut best: Option<u32> = None;
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        let mut value: u32 = 0;
        while i < chars.len() && chars[i].is_ascii_digit() {
            value = value.saturating_mul(10).saturating_add(chars[i].to_digit(10).unwrap_or(0));
            i += 1;
        }
        let digits = i - start;
        let mut j = i;
        while j < chars.len() && matches!(chars[j], ' ' | ']' | ')') {
            j += 1;
        }
        let mut unit = String::new();
        while j < chars.len() && chars[j].is_alphabetic() {
            unit.push(chars[j]);
            j += 1;
        }
        let unit = fold_key(&unit);
        if digits <= 5 && value > 0 && PAGE_UNITS.contains(&unit.as_str()) {
            best = Some(best.map_or(value, |b| b.max(value)));
        }
    }
    best
}

// ---------------------------------------------------------------------------------------
// Languages
// ---------------------------------------------------------------------------------------

/// ISO 639-2 (bibliographic and terminologic) codes with their ISO 639-1 equivalent.
const LANG3: &[(&str, &str)] = &[
    ("fre", "fr"), ("fra", "fr"), ("eng", "en"), ("ger", "de"), ("deu", "de"), ("spa", "es"),
    ("ita", "it"), ("dut", "nl"), ("nld", "nl"), ("por", "pt"), ("lat", "la"), ("rus", "ru"),
    ("pol", "pl"), ("swe", "sv"), ("dan", "da"), ("nor", "no"), ("nob", "nb"), ("nno", "nn"),
    ("fin", "fi"), ("cze", "cs"), ("ces", "cs"), ("slo", "sk"), ("slk", "sk"), ("slv", "sl"),
    ("hun", "hu"), ("rum", "ro"), ("ron", "ro"), ("bul", "bg"), ("gre", "el"), ("ell", "el"),
    ("tur", "tr"), ("ara", "ar"), ("heb", "he"), ("jpn", "ja"), ("chi", "zh"), ("zho", "zh"),
    ("kor", "ko"), ("cat", "ca"), ("baq", "eu"), ("eus", "eu"), ("glg", "gl"), ("bre", "br"),
    ("oci", "oc"), ("cos", "co"), ("wel", "cy"), ("cym", "cy"), ("gle", "ga"), ("ice", "is"),
    ("isl", "is"), ("est", "et"), ("lav", "lv"), ("lit", "lt"), ("ukr", "uk"), ("srp", "sr"),
    ("hrv", "hr"), ("bos", "bs"), ("mac", "mk"), ("mkd", "mk"), ("alb", "sq"), ("sqi", "sq"),
    ("per", "fa"), ("fas", "fa"), ("hin", "hi"), ("vie", "vi"), ("tha", "th"), ("ind", "id"),
    ("epo", "eo"), ("ltz", "lb"), ("afr", "af"), ("arm", "hy"), ("hye", "hy"), ("geo", "ka"),
    ("kat", "ka"), ("yid", "yi"), ("san", "sa"), ("bel", "be"), ("mlt", "mt"), ("gla", "gd"),
];

const LANG_NAMES: &[(&str, &str)] = &[
    ("french", "fr"), ("francais", "fr"), ("franzosisch", "fr"), ("frances", "fr"),
    ("english", "en"), ("anglais", "en"), ("englisch", "en"), ("ingles", "en"),
    ("german", "de"), ("deutsch", "de"), ("allemand", "de"), ("aleman", "de"),
    ("spanish", "es"), ("espanol", "es"), ("espagnol", "es"), ("spanisch", "es"),
    ("italian", "it"), ("italiano", "it"), ("italien", "it"), ("italienisch", "it"),
    ("dutch", "nl"), ("nederlands", "nl"), ("neerlandais", "nl"),
    ("portuguese", "pt"), ("portugues", "pt"), ("portugais", "pt"),
    ("latin", "la"), ("swedish", "sv"), ("svenska", "sv"), ("finnish", "fi"), ("suomi", "fi"),
];

/// Normalises a language to a BCP 47 code: `"fre"`, `"fra"`, `"/languages/fre"`,
/// `"French"` → `"fr"`; `"pt_br"` → `"pt-BR"`. Three-letter codes without a two-letter
/// equivalent are kept (`"grc"`); undetermined values give `None`.
#[must_use]
pub fn language(input: &str) -> Option<String> {
    let s = tidy(input, false);
    let last = s.trim().trim_end_matches('/').rsplit('/').next().unwrap_or("").trim();
    let lowered = last.to_lowercase();
    if lowered.is_empty() {
        return None;
    }
    let alpha = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_lowercase());
    let (primary, region) = match lowered.split_once(['-', '_']) {
        Some((p, r)) => (p, Some(r)),
        None => (lowered.as_str(), None),
    };
    if alpha(primary) && primary.len() == 2 {
        return Some(match region {
            Some(r) if r.len() == 2 && r.bytes().all(|b| b.is_ascii_alphabetic()) => {
                format!("{primary}-{}", r.to_ascii_uppercase())
            }
            _ => primary.to_owned(),
        });
    }
    if alpha(primary) && primary.len() == 3 && region.is_none() {
        if matches!(primary, "und" | "zxx" | "mis" | "xxx" | "unk") {
            return None;
        }
        return Some(
            LANG3
                .iter()
                .find(|(three, _)| *three == primary)
                .map_or_else(|| primary.to_owned(), |(_, two)| (*two).to_owned()),
        );
    }
    let key = fold_key(&lowered);
    LANG_NAMES.iter().find(|(name, _)| *name == key).map(|(_, code)| (*code).to_owned())
}

// ---------------------------------------------------------------------------------------
// HTML
// ---------------------------------------------------------------------------------------

const ENTITIES: &[(&str, &str)] = &[
    ("amp", "&"), ("lt", "<"), ("gt", ">"), ("quot", "\""), ("apos", "'"), ("nbsp", " "),
    ("iexcl", "¡"), ("cent", "¢"), ("pound", "£"), ("yen", "¥"), ("sect", "§"), ("copy", "©"),
    ("laquo", "«"), ("raquo", "»"), ("reg", "®"), ("deg", "°"), ("plusmn", "±"), ("para", "¶"),
    ("middot", "·"), ("iquest", "¿"), ("times", "×"), ("divide", "÷"), ("szlig", "ß"),
    ("Agrave", "À"), ("Aacute", "Á"), ("Acirc", "Â"), ("Atilde", "Ã"), ("Auml", "Ä"),
    ("Aring", "Å"), ("AElig", "Æ"), ("Ccedil", "Ç"), ("Egrave", "È"), ("Eacute", "É"),
    ("Ecirc", "Ê"), ("Euml", "Ë"), ("Igrave", "Ì"), ("Iacute", "Í"), ("Icirc", "Î"),
    ("Iuml", "Ï"), ("Ntilde", "Ñ"), ("Ograve", "Ò"), ("Oacute", "Ó"), ("Ocirc", "Ô"),
    ("Otilde", "Õ"), ("Ouml", "Ö"), ("Oslash", "Ø"), ("Ugrave", "Ù"), ("Uacute", "Ú"),
    ("Ucirc", "Û"), ("Uuml", "Ü"), ("Yacute", "Ý"), ("agrave", "à"), ("aacute", "á"),
    ("acirc", "â"), ("atilde", "ã"), ("auml", "ä"), ("aring", "å"), ("aelig", "æ"),
    ("ccedil", "ç"), ("egrave", "è"), ("eacute", "é"), ("ecirc", "ê"), ("euml", "ë"),
    ("igrave", "ì"), ("iacute", "í"), ("icirc", "î"), ("iuml", "ï"), ("ntilde", "ñ"),
    ("ograve", "ò"), ("oacute", "ó"), ("ocirc", "ô"), ("otilde", "õ"), ("ouml", "ö"),
    ("oslash", "ø"), ("ugrave", "ù"), ("uacute", "ú"), ("ucirc", "û"), ("uuml", "ü"),
    ("yacute", "ý"), ("yuml", "ÿ"), ("OElig", "Œ"), ("oelig", "œ"), ("Scaron", "Š"),
    ("scaron", "š"), ("Yuml", "Ÿ"), ("ndash", "–"), ("mdash", "—"), ("lsquo", "‘"),
    ("rsquo", "’"), ("sbquo", "‚"), ("ldquo", "“"), ("rdquo", "”"), ("bdquo", "„"),
    ("bull", "•"), ("hellip", "…"), ("euro", "€"), ("trade", "™"), ("thinsp", " "),
    ("ensp", " "), ("emsp", " "), ("shy", ""), ("zwnj", ""), ("zwj", ""), ("lrm", ""),
    ("rlm", ""),
];

/// Resolves a character reference without its `&` and `;` (`"eacute"`, `"#233"`,
/// `"#xE9"`).
pub(crate) fn resolve_entity(name: &str) -> Option<String> {
    if let Some(num) = name.strip_prefix('#') {
        let code = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse::<u32>().ok()?,
        };
        // Windows-1252 bytes written as numeric references are a classic of the web.
        let ch = match code {
            0x80..=0x9F => CP1252_HIGH[(code - 0x80) as usize],
            _ => char::from_u32(code)?,
        };
        return if ch == '\0' { None } else { Some(ch.to_string()) };
    }
    ENTITIES.iter().find(|(n, _)| *n == name).map(|(_, v)| (*v).to_owned())
}

/// `true` when the text contains an HTML tag or a character reference.
#[must_use]
pub fn looks_like_html(input: &str) -> bool {
    let bytes = input.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'<' => {
                let next = bytes.get(i + 1).copied().unwrap_or(b' ');
                if (next.is_ascii_alphabetic() || next == b'/' || next == b'!')
                    && bytes[i + 1..].iter().take(400).any(|c| *c == b'>')
                {
                    return true;
                }
            }
            b'&' => {
                let rest = &bytes[i + 1..];
                let len = rest
                    .iter()
                    .take(12)
                    .take_while(|c| c.is_ascii_alphanumeric() || **c == b'#')
                    .count();
                if len > 0
                    && rest.get(len) == Some(&b';')
                    && std::str::from_utf8(&rest[..len]).ok().and_then(resolve_entity).is_some()
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Turns an HTML fragment into plain text: tags are removed (block-level ones become line
/// breaks, `<script>` and `<style>` lose their content), character references are
/// decoded, white space is collapsed.
///
/// ```
/// use ebc_catalog::normalize::strip_html;
/// assert_eq!(strip_html("<p>Un <b>roman</b> &eacute;trange.</p><p>1942</p>"), "Un roman étrange.\n1942");
/// assert_eq!(strip_html("a < b && c"), "a < b && c");
/// ```
#[must_use]
pub fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while !rest.is_empty() {
        let Some(pos) = rest.find(['<', '&']) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        if tail.starts_with('<') {
            let next = tail[1..].chars().next().unwrap_or(' ');
            let end = tail.char_indices().take(2000).find(|(_, c)| *c == '>').map(|(i, _)| i);
            match end {
                Some(end) if next.is_ascii_alphabetic() || next == '/' || next == '!' => {
                    let inside = &tail[1..end];
                    let name: String = inside
                        .trim_start_matches('/')
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == ':')
                        .collect::<String>()
                        .to_ascii_lowercase();
                    let closing = inside.starts_with('/');
                    rest = &tail[end + 1..];
                    if !closing && (name == "script" || name == "style") {
                        let lower = rest.to_ascii_lowercase();
                        let close_tag = format!("</{name}");
                        rest = match lower.find(&close_tag) {
                            Some(i) => {
                                let after = &rest[i..];
                                after.find('>').map_or("", |j| &after[j + 1..])
                            }
                            None => "",
                        };
                    } else if matches!(
                        name.as_str(),
                        "br" | "p" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                            | "jats:p" | "jats:title" | "blockquote"
                    ) {
                        out.push('\n');
                    }
                }
                _ => {
                    out.push('<');
                    rest = &tail[1..];
                }
            }
        } else {
            let body = &tail[1..];
            let len = body
                .chars()
                .take(12)
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '#')
                .count();
            let resolved = (len > 0 && body[len..].starts_with(';'))
                .then(|| resolve_entity(&body[..len]))
                .flatten();
            match resolved {
                Some(text) => {
                    out.push_str(&text);
                    rest = &body[len + 1..];
                }
                None => {
                    out.push('&');
                    rest = body;
                }
            }
        }
    }
    tidy(&out, true)
}

// ---------------------------------------------------------------------------------------
// Whole records
// ---------------------------------------------------------------------------------------

/// `true` for `http://` and `https://` URLs with a host.
fn is_web_url(url: &str) -> bool {
    ["https://", "http://"].iter().any(|scheme| {
        url.strip_prefix(scheme).is_some_and(|rest| {
            rest.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
                && !url.chars().any(|c| c.is_whitespace() || c.is_control())
        })
    })
}

/// Applies every cleaning rule of this module to a record, in place:
///
/// * title and subtitle: non-sorting marks, medium designations, statement of
///   responsibility, ISBD punctuation; `"Title : subtitle"` is split when no subtitle is set;
/// * contributor names in natural order, without dates, duplicates removed;
/// * publisher, imprint and series statements reduced to the name;
/// * date to a partial ISO 8601 date, language to a BCP 47 code;
/// * description to plain text (HTML removed when the text contains markup);
/// * subjects, covers (web URLs only), sources and related ISBNs de-duplicated.
///
/// The function is idempotent.
pub fn normalize_record(book: &mut BookMeta) {
    let (main, split_sub) = split_title(&book.title);
    book.title = main;
    let explicit = book.subtitle.as_deref().map(|s| {
        let cleaned = title(s);
        cleaned.trim_start_matches([':', '=', ' ']).to_owned()
    });
    book.subtitle = match explicit.filter(|s| !s.is_empty()) {
        Some(sub) => Some(match split_sub {
            // The title carried "main : extra" and a subtitle was given as well.
            Some(extra) if fold_key(&extra) != fold_key(&sub) => format!("{extra} : {sub}"),
            _ => sub,
        }),
        None => split_sub,
    }
    .filter(|sub| fold_key(sub) != fold_key(&book.title));

    let mut seen: Vec<String> = Vec::new();
    let mut contributors: Vec<Contributor> = Vec::with_capacity(book.contributors.len());
    for c in &book.contributors {
        let name = person_name(&c.name);
        let key = name_key(&name);
        if key.is_empty() || seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let role = match &c.role {
            ContributorRole::Other(label) => ContributorRole::Other(text(label)),
            other => other.clone(),
        };
        contributors.push(Contributor { name, role });
    }
    book.contributors = contributors;

    book.publisher = book.publisher.as_deref().and_then(publisher);
    book.imprint = book.imprint.as_deref().and_then(publisher);
    if let (Some(p), Some(i)) = (&book.publisher, &book.imprint)
        && fold_key(p) == fold_key(i)
    {
        book.imprint = None;
    }
    book.series = book.series.as_deref().and_then(series);
    book.published = book.published.as_deref().and_then(date);
    book.language = book.language.as_deref().and_then(language);
    book.page_count = book.page_count.filter(|n| (1..=30_000).contains(n));
    book.description = book
        .description
        .as_deref()
        .map(|d| if looks_like_html(d) { strip_html(d) } else { tidy(d, true) })
        .filter(|d| !d.is_empty());

    let mut subject_keys: Vec<String> = Vec::new();
    book.subjects = book
        .subjects
        .iter()
        .map(|s| strip_trailing_punctuation(&tidy(s, false)).to_owned())
        .filter(|s| {
            let key = fold_key(s);
            if key.is_empty() || subject_keys.contains(&key) {
                return false;
            }
            subject_keys.push(key);
            true
        })
        .collect();

    let mut urls: Vec<String> = Vec::new();
    book.covers.retain_mut(|cover| {
        cover.url = cover.url.trim().to_owned();
        if !is_web_url(&cover.url) || urls.contains(&cover.url) {
            return false;
        }
        urls.push(cover.url.clone());
        true
    });

    let mut sources = Vec::with_capacity(book.sources.len());
    for source in &book.sources {
        if !sources.contains(source) {
            sources.push(source.clone());
        }
    }
    book.sources = sources;

    let mut related = Vec::with_capacity(book.related_isbns.len());
    for isbn in &book.related_isbns {
        if Some(*isbn) != book.isbn13 && !related.contains(isbn) {
            related.push(*isbn);
        }
    }
    book.related_isbns = related;
}

/// Comparison key for a personal name: the folded words, sorted, so that
/// `"Camus, Albert"`, `"Albert Camus"` and `"CAMUS Albert"` compare equal.
#[must_use]
pub fn name_key(name: &str) -> String {
    let folded = fold_key(name);
    let mut words: Vec<&str> = folded.split(' ').filter(|w| !w.is_empty()).collect();
    words.sort_unstable();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_handles_decomposed_and_stacked_marks() {
        assert_eq!(compose("L'e\u{301}tranger"), "L'étranger");
        assert_eq!(compose("Vie\u{323}\u{302}t"), "Việt");
        assert_eq!(compose("\u{301}abc"), "\u{301}abc");
        assert_eq!(compose("x\u{301}"), "x\u{301}");
        // A mark that does not compose blocks later ones instead of corrupting the text.
        assert_eq!(compose("q\u{301}\u{302}"), "q\u{301}\u{302}");
        assert_eq!(compose("Ελλα\u{301}δα"), "Ελλάδα");
    }

    #[test]
    fn fold_key_ignores_case_accents_and_punctuation() {
        assert_eq!(fold_key("L'Étranger"), "l etranger");
        assert_eq!(fold_key("l’etranger !"), "l etranger");
        assert_eq!(fold_key("Œuvres — Straße"), "oeuvres strasse");
        assert_eq!(fold_key("  "), "");
        assert_eq!(fold_key("e\u{301}"), "e");
    }

    #[test]
    fn mojibake_is_repaired_only_when_it_is_mojibake() {
        assert_eq!(fix_mojibake("L'Ã‰tranger"), "L'Étranger");
        assert_eq!(fix_mojibake("Ã©tÃ©"), "été");
        assert_eq!(fix_mojibake("À bientôt"), "À bientôt");
        assert_eq!(fix_mojibake("naïve café"), "naïve café");
        assert_eq!(fix_mojibake("Ångström"), "Ångström");
        assert_eq!(fix_mojibake("Â"), "Â");
    }

    #[test]
    fn titles_lose_catalogue_artefacts() {
        assert_eq!(title("L'étranger [Texte imprimé] / Albert Camus"), "L'étranger");
        assert_eq!(title("\u{98}L'\u{9c}étranger"), "L'étranger");
        assert_eq!(title("˜L'œétranger"), "L'étranger");
        assert_eq!(title("˜Le œcœur"), "Le cœur");
        assert_eq!(title("<<Der>> Zauberberg"), "Der Zauberberg");
        assert_eq!(title("¬Die¬ Blechtrommel"), "Die Blechtrommel");
        assert_eq!(title("L'@étranger"), "L'étranger");
        assert_eq!(title("Der @Zauberberg"), "Der Zauberberg");
        assert_eq!(title("user@example.org"), "user@example.org");
        assert_eq!(title("Pride and prejudice [electronic resource] /"), "Pride and prejudice");
        assert_eq!(title("Poèmes [choisis]"), "Poèmes [choisis]");
        assert_eq!(title("The stranger."), "The stranger");
        assert_eq!(title("Qui a tué... ?"), "Qui a tué... ?");
        assert_eq!(title("Et alors..."), "Et alors...");
        assert_eq!(title("Hamlet = Hamlet, prince de Danemark"), "Hamlet");
        assert_eq!(title("AC/DC"), "AC/DC");
        assert_eq!(title("  "), "");
        assert_eq!(title("[Texte imprimé]"), "");
    }

    #[test]
    fn title_is_idempotent() {
        for raw in [
            "L'étranger [Texte imprimé] / Albert Camus",
            "˜L'œétranger",
            "Buddenbrooks : Verfall einer Familie /",
            "A. B.",
            "Mr.",
            "<<>>",
            "@",
            "...",
        ] {
            let once = title(raw);
            assert_eq!(title(&once), once, "{raw:?}");
        }
    }

    #[test]
    fn split_title_separates_subtitle() {
        assert_eq!(
            split_title("Buddenbrooks : Verfall einer Familie / Thomas Mann"),
            ("Buddenbrooks".to_owned(), Some("Verfall einer Familie".to_owned()))
        );
        assert_eq!(split_title("Ratio: 2:1"), ("Ratio: 2:1".to_owned(), None));
        assert_eq!(split_title(" : x"), (": x".to_owned(), None));
    }

    #[test]
    fn names_are_put_in_natural_order() {
        assert_eq!(person_name("Camus, Albert (1913-1960)"), "Albert Camus");
        assert_eq!(person_name("Camus, Albert, 1913-1960"), "Albert Camus");
        assert_eq!(person_name("Austen, Jane,"), "Jane Austen");
        assert_eq!(person_name("Mann, Thomas, 1875-1955."), "Thomas Mann");
        assert_eq!(person_name("Jones, Vivien, 1952-"), "Vivien Jones");
        assert_eq!(person_name("Tolkien, J. R. R."), "J. R. R. Tolkien");
        assert_eq!(person_name("King, Martin Luther, Jr."), "Martin Luther King Jr.");
        assert_eq!(person_name("Beauvoir, Simone de"), "Simone de Beauvoir");
        assert_eq!(person_name("Voltaire (1694-1778)"), "Voltaire");
        assert_eq!(person_name("Homère"), "Homère");
        assert_eq!(person_name("Colette (1873-1954)"), "Colette");
        assert_eq!(person_name("Dupont, Jean (19..-....)"), "Jean Dupont");
        assert_eq!(
            person_name("Centre national de la recherche scientifique (France)"),
            "Centre national de la recherche scientifique (France)"
        );
        assert_eq!(person_name("Kafka, Franz"), "Franz Kafka");
        assert_eq!(person_name(", ,"), "");
        assert_eq!(person_name("50 Cent"), "50 Cent");
        assert_eq!(person_name("3M, Company"), "Company 3M");
        assert_eq!(person_name("(1913-1960)"), "");
        let once = person_name("Camus, Albert (1913-1960)");
        assert_eq!(person_name(&once), once);
    }

    #[test]
    fn publisher_statements_reduce_to_the_name() {
        assert_eq!(publisher("[Paris] : Gallimard").as_deref(), Some("Gallimard"));
        assert_eq!(publisher("Paris : Gallimard, 1971").as_deref(), Some("Gallimard"));
        assert_eq!(publisher("Paris : Gallimard, impr. 2011, cop. 1942").as_deref(), Some("Gallimard"));
        assert_eq!(publisher("Gallimard ([Paris])").as_deref(), Some("Gallimard"));
        assert_eq!(publisher("[Gallimard]").as_deref(), Some("Gallimard"));
        assert_eq!(publisher("Penguin Books,").as_deref(), Some("Penguin Books"));
        assert_eq!(
            publisher("Fischer-Taschenbuch-Verl.").as_deref(),
            Some("Fischer-Taschenbuch-Verl.")
        );
        assert_eq!(
            publisher("London ; New York : Penguin Books,").as_deref(),
            Some("Penguin Books")
        );
        assert_eq!(
            publisher("Paris : Gallimard ; [Montréal] : Lacombe, 1990").as_deref(),
            Some("Gallimard")
        );
        assert_eq!(publisher("50 Cent Books").as_deref(), Some("50 Cent Books"));
        assert_eq!(publisher("Reclam").as_deref(), Some("Reclam"));
        assert_eq!(publisher("[s.n.]"), None);
        assert_eq!(publisher("[S.l.] : [s.n.]"), None);
        assert_eq!(publisher(""), None);
        let once = publisher("Paris : Gallimard, 1971").expect("publisher");
        assert_eq!(publisher(&once).as_deref(), Some(once.as_str()));
    }

    #[test]
    fn series_statements_lose_numbering() {
        assert_eq!(series("Collection Folio ; 2").as_deref(), Some("Folio"));
        assert_eq!(series("Folio, 2").as_deref(), Some("Folio"));
        assert_eq!(series("Folio ; n° 2").as_deref(), Some("Folio"));
        assert_eq!(series("Folio -- 2").as_deref(), Some("Folio"));
        assert_eq!(series("(Folio ; 2)").as_deref(), Some("Folio"));
        assert_eq!(series("Fischer Nr. 9431").as_deref(), Some("Fischer"));
        assert_eq!(series("Collection blanche").as_deref(), Some("Collection blanche"));
        assert_eq!(series("Penguin classics").as_deref(), Some("Penguin classics"));
        assert_eq!(series("10/18").as_deref(), Some("10/18"));
        assert_eq!(series("Que sais-je ? ; 123").as_deref(), Some("Que sais-je ?"));
        assert_eq!(series(" ; 2"), None);
    }

    #[test]
    fn dates_reduce_to_iso() {
        let cases: &[(&str, Option<&str>)] = &[
            ("1971", Some("1971")),
            ("DL 2015", Some("2015")),
            ("cop. 1942", Some("1942")),
            ("[1972]", Some("1972")),
            ("impr. 2011", Some("2011")),
            ("impr. 2005, cop. 1942", Some("2005")),
            ("1971, cop. 1942", Some("1971")),
            ("cop. 1942, 1971", Some("1971")),
            ("DL 2015, cop. 2014", Some("2015")),
            ("c1996", Some("1996")),
            ("©2015", Some("2015")),
            ("[ca 1950]", Some("1950")),
            ("1998-2001", Some("1998")),
            ("2015-", Some("2015")),
            ("1996?", Some("1996")),
            ("2012-12-06", Some("2012-12-06")),
            ("2012-12", Some("2012-12")),
            ("2012-13", Some("2012")),
            ("2012-00-00", Some("2012")),
            ("1971-01-01T00:00:00Z", Some("1971-01-01")),
            ("March 2009", Some("2009-03")),
            ("Jan 05, 2010", Some("2010-01-05")),
            ("5 janvier 1972", Some("1972-01-05")),
            ("5. Januar 1972", Some("1972-01-05")),
            ("août 1999", Some("1999-08")),
            ("05.01.1972", Some("1972-01-05")),
            ("25/12/1999", Some("1999-12-25")),
            ("12/25/1999", Some("1999-12-25")),
            ("03/04/1999", Some("1999")),
            ("19..", None),
            ("s.d.", None),
            ("", None),
            ("9782070360024", None),
            ("0000", None),
            ("3000", None),
        ];
        for (raw, expected) in cases {
            assert_eq!(date(raw).as_deref(), *expected, "{raw:?}");
            if let Some(once) = date(raw) {
                assert_eq!(date(&once).as_deref(), Some(once.as_str()), "idempotent {raw:?}");
            }
        }
    }

    #[test]
    fn page_counts_are_read_from_extents() {
        let cases: &[(&str, Option<u32>)] = &[
            ("1 vol. (185 p.)", Some(185)),
            ("185 p.", Some(185)),
            ("185p.", Some(185)),
            ("XII-345 p.", Some(345)),
            ("xli, 435 p. ;", Some(435)),
            ("xii, 345 p., [16] p. of plates", Some(345)),
            ("758 S.", Some(758)),
            ("1 Online-Ressource (224 Seiten)", Some(224)),
            ("476 sivua ; 21 cm", Some(476)),
            ("123 s.", Some(123)),
            ("300 pages : illustrations", Some(300)),
            ("185", Some(185)),
            ("2 vol.", None),
            ("18 cm", None),
            ("185 photos", None),
            ("0 p.", None),
            ("99999999999999999999 p.", None),
            ("", None),
        ];
        for (raw, expected) in cases {
            assert_eq!(page_count(raw), *expected, "{raw:?}");
        }
    }

    #[test]
    fn languages_become_bcp47() {
        let cases: &[(&str, Option<&str>)] = &[
            ("fre", Some("fr")),
            ("fra", Some("fr")),
            ("FRE", Some("fr")),
            ("ger", Some("de")),
            ("deu", Some("de")),
            ("eng", Some("en")),
            ("fr", Some("fr")),
            ("pt-br", Some("pt-BR")),
            ("pt_BR", Some("pt-BR")),
            ("/languages/fre", Some("fr")),
            ("http://id.loc.gov/vocabulary/iso639-2/ger", Some("de")),
            ("https://id.kb.se/language/swe", Some("sv")),
            ("French", Some("fr")),
            ("français", Some("fr")),
            ("Deutsch", Some("de")),
            ("grc", Some("grc")),
            ("mul", Some("mul")),
            ("und", None),
            ("zxx", None),
            ("", None),
            ("|||", None),
            ("klingon", None),
        ];
        for (raw, expected) in cases {
            assert_eq!(language(raw).as_deref(), *expected, "{raw:?}");
        }
    }

    #[test]
    fn html_is_stripped() {
        assert_eq!(
            strip_html("<p>Un <b>roman</b> &eacute;trange.</p><p>1942</p>"),
            "Un roman étrange.\n1942"
        );
        assert_eq!(strip_html("Ligne 1<br>Ligne 2<br/>Ligne 3"), "Ligne 1\nLigne 2\nLigne 3");
        assert_eq!(strip_html("a<script>alert('x')</script>b"), "ab");
        assert_eq!(strip_html("a<style>p{}</style>b<script>never closed"), "ab");
        assert_eq!(strip_html("a < b && c > d"), "a < b && c > d");
        assert_eq!(strip_html("AT&T &amp; R&D &#233;t&#xE9; &#150; fin"), "AT&T & R&D été – fin");
        assert_eq!(strip_html("<jats:p>Abstract.</jats:p>"), "Abstract.");
        assert_eq!(strip_html("&unknown; &#xD800; &#99999999;"), "&unknown; &#xD800; &#99999999;");
        assert_eq!(strip_html("<"), "<");
        assert_eq!(strip_html("<a href='x"), "<a href='x");
        assert!(looks_like_html("a<br>b"));
        assert!(looks_like_html("caf&eacute;"));
        assert!(!looks_like_html("a < b & c"));
        assert!(!looks_like_html("R&D; plus"));
    }

    #[test]
    fn records_are_normalised_once_and_for_all() {
        let mut book = BookMeta {
            title: "\u{98}L'\u{9c}étranger [Texte imprimé] : roman / Albert Camus".into(),
            contributors: vec![
                Contributor::author("Camus, Albert (1913-1960)"),
                Contributor::author("Albert CAMUS"),
                Contributor { name: " ".into(), role: ContributorRole::Translator },
            ],
            publisher: Some("[Paris] : Gallimard".into()),
            imprint: Some("Gallimard".into()),
            series: Some("Collection Folio ; 2".into()),
            published: Some("impr. 2011, cop. 1942".into()),
            language: Some("fre".into()),
            page_count: Some(0),
            description: Some("<p>Meursault&nbsp;&hellip;</p>".into()),
            subjects: vec!["Roman.".into(), "roman".into(), " ".into()],
            covers: vec![
                ebc_model::CoverRef {
                    url: "javascript:alert(1)".into(),
                    kind: ebc_model::CoverKind::Front,
                    width_px: None,
                    height_px: None,
                    source: "x".into(),
                },
                ebc_model::CoverRef {
                    url: " https://example.org/c.jpg ".into(),
                    kind: ebc_model::CoverKind::Front,
                    width_px: None,
                    height_px: None,
                    source: "x".into(),
                },
            ],
            ..BookMeta::default()
        };
        normalize_record(&mut book);
        assert_eq!(book.title, "L'étranger");
        assert_eq!(book.subtitle.as_deref(), Some("roman"));
        assert_eq!(book.contributors, vec![Contributor::author("Albert Camus")]);
        assert_eq!(book.publisher.as_deref(), Some("Gallimard"));
        assert_eq!(book.imprint, None);
        assert_eq!(book.series.as_deref(), Some("Folio"));
        assert_eq!(book.published.as_deref(), Some("2011"));
        assert_eq!(book.language.as_deref(), Some("fr"));
        assert_eq!(book.page_count, None);
        assert_eq!(book.description.as_deref(), Some("Meursault …"));
        assert_eq!(book.subjects, vec!["Roman".to_owned()]);
        assert_eq!(book.covers.len(), 1);
        assert_eq!(book.covers[0].url, "https://example.org/c.jpg");
        let once = book.clone();
        normalize_record(&mut book);
        assert_eq!(book, once);
    }
}
