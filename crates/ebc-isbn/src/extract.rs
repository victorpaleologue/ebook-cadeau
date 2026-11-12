//! Finding ISBNs in free text (web pages, catalogue records, file names, URLs).

use std::collections::HashSet;

use crate::{Isbn13, ten_to_thirteen, validate10, validate13};

/// How a digit group is connected to the group before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Join {
    /// Not connected: start of a chain.
    None,
    /// Exactly one hyphen-like character in between.
    Hyphen,
    /// Exactly one space-like character in between.
    Space,
}

/// A maximal run of ASCII digits, optionally terminated by an ISBN-10 `X`.
#[derive(Debug, Clone, Copy)]
struct Token {
    /// The first thirteen digits, as ASCII.
    digits: [u8; 13],
    /// Number of digits in the run, saturated at 14 ("too many").
    len: u8,
    /// The run is terminated by an `X` check character (which may be the whole token).
    x: bool,
    /// Connection to the previous token.
    join: Join,
    /// The character right before the run is an ASCII letter.
    after_letter: bool,
    /// The character right after the run is an ASCII letter (other than a terminating `X`).
    before_letter: bool,
    /// The run directly follows an `ISBN` label.
    labelled: bool,
}

/// What separates the scanner from the last kept token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Link {
    /// Something other than a single separator: the next token starts a new chain.
    Broken,
    /// Nothing yet: the scanner is right after the token's last digit.
    Direct,
    /// Exactly one separator.
    Separator(Join),
}

/// Whether an `ISBN` label has just been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Label {
    /// No label in effect.
    None,
    /// `ISBN` was read; `number_allowed` until its `10`/`13` suffix has been consumed.
    Seen { number_allowed: bool },
}

const fn hyphen_like(c: char) -> bool {
    matches!(c, '-' | '\u{2010}'..='\u{2013}' | '\u{2212}')
}

const fn space_like(c: char) -> bool {
    matches!(c, ' ' | '\u{a0}' | '\u{2007}' | '\u{2009}' | '\u{202f}')
}

/// Characters that may stand between an `ISBN` label and the number.
const fn label_filler(c: char) -> bool {
    hyphen_like(c)
        || space_like(c)
        || matches!(c, ':' | '.' | '#' | '=' | '(' | ')' | '\t' | '\u{ff1a}')
}

/// `"isbn"` as the big-endian packing of its four ASCII bytes.
const ISBN_LETTERS: u32 = u32::from_be_bytes(*b"isbn");

/// Splits `text` into digit tokens and records how they relate to each other.
fn tokenize(text: &str) -> Vec<Token> {
    let bytes = text.as_bytes();
    let mut tokens: Vec<Token> = Vec::new();
    let mut pos = 0usize;
    let mut link = Link::Broken;
    let mut label = Label::None;
    // The last four ASCII letters read, lower-cased; reset by anything that is not a letter.
    let mut letters = 0u32;
    let mut previous_is_letter = false;

    while let Some(&byte) = bytes.get(pos) {
        if byte.is_ascii_digit() {
            let start = pos;
            while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                pos += 1;
            }
            let run = bytes.get(start..pos).unwrap_or_default();
            letters = 0;
            if label == (Label::Seen { number_allowed: true }) && (run == b"10" || run == b"13") {
                // The numeric suffix of "ISBN-10" / "ISBN 13": part of the label.
                label = Label::Seen { number_allowed: false };
                link = Link::Broken;
                previous_is_letter = false;
                continue;
            }
            let mut digits = [0u8; 13];
            for (slot, d) in digits.iter_mut().zip(run) {
                *slot = *d;
            }
            let mut token = Token {
                digits,
                len: run.len().min(14) as u8,
                x: false,
                join: match link {
                    Link::Separator(join) => join,
                    Link::Broken | Link::Direct => Join::None,
                },
                after_letter: previous_is_letter,
                before_letter: false,
                labelled: label != Label::None,
            };
            let next = bytes.get(pos).copied();
            let after_next_is_alnum = bytes.get(pos + 1).is_some_and(u8::is_ascii_alphanumeric);
            // Also after a terminating `X`: what follows may still be "attached" to it.
            link = Link::Direct;
            if matches!(next, Some(b'X' | b'x')) && !after_next_is_alnum {
                token.x = true;
                pos += 1;
            } else if next.is_some_and(|n| n.is_ascii_alphabetic()) {
                token.before_letter = true;
            }
            tokens.push(token);
            label = Label::None;
            previous_is_letter = false;
            continue;
        }

        // Not a digit: decode one character.
        let c = if byte.is_ascii() {
            char::from(byte)
        } else {
            text.get(pos..).and_then(|rest| rest.chars().next()).unwrap_or('\u{fffd}')
        };
        pos += c.len_utf8();

        if c.is_ascii_alphabetic() {
            // A lone `X` joined to the previous digits is an ISBN-10 check character.
            if let Link::Separator(join) = link
                && matches!(c, 'X' | 'x')
                && !bytes.get(pos).is_some_and(u8::is_ascii_alphanumeric)
            {
                tokens.push(Token {
                    digits: [0; 13],
                    len: 0,
                    x: true,
                    join,
                    after_letter: false,
                    before_letter: false,
                    labelled: false,
                });
                link = Link::Direct;
            } else {
                link = Link::Broken;
            }
            letters = (letters << 8) | u32::from(byte.to_ascii_lowercase());
            label = if letters == ISBN_LETTERS {
                Label::Seen { number_allowed: true }
            } else {
                Label::None
            };
            previous_is_letter = true;
            continue;
        }

        letters = 0;
        previous_is_letter = false;
        link = match (link, hyphen_like(c), space_like(c)) {
            (Link::Direct, true, _) => Link::Separator(Join::Hyphen),
            (Link::Direct, _, true) => Link::Separator(Join::Space),
            _ => Link::Broken,
        };
        if !label_filler(c) {
            label = Label::None;
        }
    }
    tokens
}

/// Tries to read an ISBN starting at token `start`. Returns it with the number of tokens
/// it spans.
fn match_at(tokens: &[Token], start: usize) -> Option<(Isbn13, usize)> {
    let first = tokens.get(start)?;
    if first.len == 0 {
        return None;
    }
    let mut chars = [0u8; 13];
    let mut total = 0usize;
    let mut ten: Option<(Isbn13, usize)> = None;

    for (offset, token) in tokens.iter().skip(start).take(5).enumerate() {
        if offset > 0 && token.join == Join::None {
            break;
        }
        let len = usize::from(token.len);
        if len > 13 - total {
            break;
        }
        for (slot, d) in chars.iter_mut().skip(total).zip(token.digits.iter().take(len)) {
            *slot = *d;
        }
        total += len;
        let span = offset + 1;

        if total == 13 {
            // Strong evidence (prefix + check digit): no further conditions.
            return validate13(&chars).ok().map(|d| (Isbn13::from_valid_digits(d), span)).or(ten);
        }
        let complete_ten = (total == 10 && !token.x) || (total == 9 && token.x);
        if complete_ten && span <= 4 && ten_is_plausible(tokens, start, span) {
            let mut candidate = chars;
            if token.x {
                candidate[9] = b'X';
            }
            if let Ok(d) = validate10(&candidate) {
                ten = Some((Isbn13::from_valid_digits(ten_to_thirteen(&d)), span));
            }
        }
        if token.x {
            break;
        }
    }
    ten
}

/// The extra conditions an ISBN-10 candidate must meet. Ten digits with a modulo-11 check
/// are weak evidence (one random number in eleven passes), so the context has to look
/// like an ISBN: see [`extract_all`].
fn ten_is_plausible(tokens: &[Token], start: usize, span: usize) -> bool {
    let (Some(first), Some(last)) = (tokens.get(start), tokens.get(start + span - 1)) else {
        return false;
    };
    if last.before_letter || (first.after_letter && !first.labelled) {
        return false;
    }
    if span == 1 {
        return true;
    }
    if first.labelled {
        return true;
    }
    // Several groups: the canonical written form, with the check character on its own,
    // standing alone rather than inside a longer hyphenated number.
    let check_alone = (last.len == 1 && !last.x) || (last.len == 0 && last.x);
    let attached_before = first.join != Join::None;
    let attached_after = tokens.get(start + span).is_some_and(|next| next.join == Join::Hyphen);
    check_alone && !attached_before && !attached_after
}

/// Finds every valid ISBN in free text, converted to [`Isbn13`], deduplicated, in order of
/// first appearance.
///
/// The text may be anything: a catalogue record, an HTML page, a URL, a file name. Runs in
/// linear time and never panics.
///
/// What is recognised:
///
/// * **ISBN-13**: thirteen digits with prefix 978/979 and a correct check digit, written
///   contiguously or in up to five groups separated by single hyphens (including the
///   typographic ones) or single spaces: `9782070360024`, `978-2-07-036002-4`,
///   `978 2070360024`. Letters or other numbers may touch it (`ISBN9782070360024`,
///   `9782070360024-475x500.jpg`).
/// * **ISBN-10**: ten characters with a correct modulo-11 check (`X` allowed last), either
///   contiguous and not glued to letters (`2070360024`, `/dp/080442957X/`), or in the
///   canonical grouped form whose last group is the check character alone
///   (`2-07-036002-4`, `0 8044 2957 X`), or in any grouping of up to four groups right
///   after an `ISBN` label (`ISBN 207 036 0024`, `ISBN2070360024`).
///
/// What is deliberately *not* recognised, to keep false positives rare: ISBN-10s buried in
/// longer hyphenated numbers, phone-number-like groupings, digits in other scripts, and
/// numbers broken across lines. A ten-digit number that is not an ISBN still has one
/// chance in eleven of passing the check, so treat unlabelled ISBN-10 hits as candidates.
///
/// ```
/// let found = ebc_isbn::extract_all("L'Étranger — ISBN : 978-2-07-036002-4 (poche), 2070360024");
/// assert_eq!(found.len(), 1); // the same book twice
/// assert_eq!(found[0].to_string(), "9782070360024");
/// ```
#[must_use]
pub fn extract_all(text: &str) -> Vec<Isbn13> {
    let tokens = tokenize(text);
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut index = 0usize;
    while index < tokens.len() {
        match match_at(&tokens, index) {
            Some((isbn, span)) => {
                if seen.insert(isbn) {
                    found.push(isbn);
                }
                index += span.max(1);
            }
            None => index += 1,
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> Vec<String> {
        extract_all(text).iter().map(Isbn13::to_plain).collect()
    }

    const ETRANGER: &str = "9782070360024";

    #[test]
    fn finds_isbn13_in_common_notations() {
        for text in [
            "9782070360024",
            "ISBN 978-2-07-036002-4",
            "ISBN-13: 978-2-07-036002-4.",
            "ISBN-13 978-2-07-036002-4",
            "isbn13=9782070360024&x=1",
            "ISBN9782070360024",
            "(978 2 07 036002 4)",
            "978 2070360024",
            "978-2070360024",
            "EAN\u{a0}: 978\u{2011}2\u{2011}07\u{2011}036002\u{2011}4",
            "ISBN\u{202f}: 978\u{2013}2\u{2013}07\u{2013}036002\u{2013}4",
            "https://example.org/livre/l-etranger-9782070360024.html",
            "https://img.example.org/albert-camus/9782070360024-475x500-1.webp",
            "/covers/12345-9782070360024_L.jpg",
            "<meta property=\"book:isbn\" content=\"9782070360024\">",
            "{\"isbn\":\"9782070360024\",\"price\":7.5}",
            "prix 7,50 € 9782070360024 185 pages",
            "1 978-2-07-036002-4",
            "2021 9782070360024 300",
            "urn:isbn:9782070360024",
        ] {
            assert_eq!(plain(text), [ETRANGER], "text {text:?}");
        }
    }

    #[test]
    fn finds_isbn10_in_common_notations() {
        for text in [
            "2070360024",
            "ISBN 2-07-036002-4",
            "ISBN-10: 2-07-036002-4",
            "ISBN-10 2070360024",
            "ISBN2070360024",
            "isbn 207 036 0024",
            "2 07 036002 4",
            "(2-07-036002-4)",
            "https://www.example.com/dp/2070360024/ref=x",
            "l-etranger-2070360024",
            "product-12345-2070360024.html",
            "2019 2070360024",
            "207036002-4",
        ] {
            assert_eq!(plain(text), [ETRANGER], "text {text:?}");
        }
        for text in [
            "080442957X",
            "080442957x",
            "0-8044-2957-X",
            "0 8044 2957 x",
            "ISBN 0-8044-2957-X.",
            "/dp/080442957X/",
            "ISBN-10:080442957X",
        ] {
            assert_eq!(plain(text), ["9780804429573"], "text {text:?}");
        }
    }

    #[test]
    fn deduplicates_and_keeps_order() {
        let text = "ISBN-10: 0-306-40615-2 / ISBN-13: 978-0-306-40615-7; see also \
                    9782070360024, 2070360024 and 978-3-16-148410-0. Again 9780306406157.";
        assert_eq!(plain(text), ["9780306406157", "9782070360024", "9783161484100"]);
    }

    #[test]
    fn prefers_isbn13_over_an_embedded_isbn10() {
        // "978 2070360024": the second group alone is a valid ISBN-10 of the same book,
        // but "979 2070360024" is not a valid ISBN-13 and must still yield the ISBN-10.
        assert_eq!(plain("978 2070360024"), [ETRANGER]);
        assert_eq!(plain("979 2070360024"), [ETRANGER]);
        // An ISBN-10 followed by another number that completes thirteen digits.
        assert_eq!(plain("2070360024 978"), [ETRANGER]);
    }

    #[test]
    fn rejects_invalid_numbers() {
        for text in [
            "",
            "no digits at all",
            "9782070360025",       // wrong check digit
            "4006381333931",       // a valid EAN-13 that is not a book
            "97820703600244",      // fourteen digits
            "19782070360024",      // glued to another digit
            "97820703600240",      // glued to another digit
            "978-2-07-036002",     // truncated
            "2070360025",          // ISBN-10, wrong check digit
            "20703600240",         // eleven digits
            "tel. 01 23 45 67 89", // phone number
            "+33-1-23-45-67-89",
            "555-123-4567",
            "1-555-123-4567",
            "2023-10-05 12:30:00",
            "id2070360024", // glued to letters, no label
            "2070360024abc",
            "080442957Xyz",
            "sha256-a2070360024f",
            "X",
            "x-x-x-x",
            "978\n2070360025",
        ] {
            assert_eq!(plain(text), Vec::<String>::new(), "text {text:?}");
        }
    }

    #[test]
    fn isbn10_inside_a_longer_hyphenated_number_is_ignored() {
        // 6-381-33393-? would be read as an ISBN-10 if sub-windows of a hyphenated EAN
        // were considered. Build the variant whose embedded check digit is valid.
        let check = crate::isbn10_check_char(b"638133393");
        let embedded = format!("6-381-33393-{check}");
        assert_eq!(extract_all(&embedded).len(), 1, "sanity: valid when standing alone");
        assert_eq!(plain(&format!("400-{embedded}")), Vec::<String>::new());
        assert_eq!(plain(&format!("{embedded}-12")), Vec::<String>::new());
        // …but a space-separated neighbour is fine.
        assert_eq!(extract_all(&format!("{embedded} 12")).len(), 1);
    }

    #[test]
    fn line_breaks_do_not_join_groups() {
        assert_eq!(plain("978-2-07-\n036002-4"), Vec::<String>::new());
        assert_eq!(plain("9782070360024\n9780306406157\n"), [ETRANGER, "9780306406157"]);
        assert_eq!(plain("2070360024\t080442957X"), [ETRANGER, "9780804429573"]);
    }

    #[test]
    fn label_numbers_are_not_part_of_the_isbn() {
        assert_eq!(plain("ISBN-13-978-2-07-036002-4"), [ETRANGER]);
        assert_eq!(plain("ISBN 13 : 9782070360024"), [ETRANGER]);
        assert_eq!(plain("ISBN-10 0-306-40615-2 ISBN-13 978-0-306-40615-7"), ["9780306406157"]);
        // "ISBN 10" at the very end, or followed by text, is harmless.
        assert_eq!(plain("see ISBN 13"), Vec::<String>::new());
        assert_eq!(plain("ISBN 10 is the old format; 2070360024 is one"), [ETRANGER]);
    }

    #[test]
    fn realistic_html_page() {
        let html = r#"<!doctype html><html lang="fr"><head>
            <title>L'Étranger - Albert Camus - Folio - Poche</title>
            <meta property="og:image" content="https://cdn.example.net/img/9782070360024-475x500-1.webp">
            <script type="application/ld+json">{"@type":"Book","isbn":"9782070360024",
              "workExample":[{"isbn":"978-2-07-212286-6"},{"isbn":"2070360024"}],
              "offers":{"price":"7.50","sku":"000000123456"}}</script>
            <link rel="stylesheet" href="/a.css" integrity="sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC">
            </head><body><p>EAN : 9782070360024 — 185 pages — 11/01/1972 — 0,10 kg</p>
            <p>Service client : 01 23 45 67 89 (du lundi au vendredi, 9h-18h)</p></body></html>"#;
        assert_eq!(plain(html), [ETRANGER, "9782072122866"]);
    }

    #[test]
    fn hostile_inputs_are_handled_in_linear_time() {
        // One enormous digit run, many tiny groups, many separators, many labels.
        assert!(extract_all(&"9".repeat(1_000_000)).is_empty());
        assert!(extract_all(&"1 ".repeat(300_000)).is_empty());
        assert!(extract_all(&"9-7-8-".repeat(100_000)).is_empty());
        assert!(extract_all(&"ISBN".repeat(100_000)).is_empty());
        assert!(extract_all(&"ISBN-13 ".repeat(50_000)).is_empty());
        assert!(extract_all(&"X-".repeat(100_000)).is_empty());
        assert!(extract_all("\u{0}\u{ffff}\u{10ffff}é漢字978").is_empty());
        // Non-ASCII right around a match.
        assert_eq!(plain("書籍9782070360024円"), [ETRANGER]);
        assert_eq!(plain("é2070360024é"), [ETRANGER]);
        // Many distinct valid ISBNs: output is deduplicated, input order preserved.
        let mut text = String::new();
        let mut expected = Vec::new();
        for n in 0..2_000u32 {
            let body = format!("97820703{n:04}");
            let mut first12 = [0u8; 12];
            first12.copy_from_slice(body.as_bytes());
            let isbn = format!("{body}{}", crate::ean13_check_digit(&first12));
            text.push_str(&isbn);
            text.push(' ');
            expected.push(isbn);
        }
        let twice = format!("{text}{text}");
        assert_eq!(plain(&twice), expected);
    }
}
