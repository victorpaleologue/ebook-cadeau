//! File names for downloads: the ASCII name used in URLs and as `filename=`, and the
//! `Content-Disposition` header.
//!
//! E-reader browsers take the name from the last URL segment and several of them break
//! on non-ASCII or unusual characters (`docs/research/notes/ereader-ux.devices.md` §3.1,
//! §3.2), so the URL name is restricted to `[A-Za-z0-9._-]`, at most 64 characters.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

/// Longest ASCII file name, extension included.
pub const MAX_ASCII_NAME: usize = 64;

/// Name used when nothing usable is left of the title.
pub const FALLBACK_STEM: &str = "livre";

/// ASCII replacement of a Latin letter with a diacritic, or of a ligature.
fn fold(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' => "A",
        'ç' | 'ć' | 'č' | 'ĉ' | 'ċ' => "c",
        'Ç' | 'Ć' | 'Č' | 'Ĉ' | 'Ċ' => "C",
        'ď' | 'đ' | 'ð' => "d",
        'Ď' | 'Đ' | 'Ð' => "D",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => "e",
        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ė' | 'Ę' | 'Ě' => "E",
        'ğ' | 'ĝ' | 'ġ' => "g",
        'Ğ' | 'Ĝ' | 'Ġ' => "G",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => "i",
        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ī' | 'Į' | 'İ' => "I",
        'ł' | 'ľ' | 'ĺ' => "l",
        'Ł' | 'Ľ' | 'Ĺ' => "L",
        'ñ' | 'ń' | 'ň' => "n",
        'Ñ' | 'Ń' | 'Ň' => "N",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => "o",
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ő' => "O",
        'ř' | 'ŕ' => "r",
        'Ř' | 'Ŕ' => "R",
        'ś' | 'š' | 'ş' | 'ș' => "s",
        'Ś' | 'Š' | 'Ş' | 'Ș' => "S",
        'ť' | 'ţ' | 'ț' => "t",
        'Ť' | 'Ţ' | 'Ț' => "T",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => "u",
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ū' | 'Ů' | 'Ű' | 'Ų' => "U",
        'ý' | 'ÿ' => "y",
        'Ý' | 'Ÿ' => "Y",
        'ź' | 'ż' | 'ž' => "z",
        'Ź' | 'Ż' | 'Ž' => "Z",
        'œ' => "oe",
        'Œ' => "OE",
        'æ' => "ae",
        'Æ' => "AE",
        'ß' => "ss",
        'þ' => "th",
        'Þ' => "Th",
        _ => return None,
    })
}

/// An ASCII file name derived from a title: letters and digits kept (diacritics folded),
/// every run of other characters turned into one hyphen, at most [`MAX_ASCII_NAME`]
/// characters with the extension, [`FALLBACK_STEM`] when nothing is left.
///
/// `extension` is given without dot and must itself be ASCII alphanumeric (anything else
/// is dropped from it).
#[must_use]
pub fn ascii_file_name(title: &str, extension: &str) -> String {
    let extension: String = extension
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(8)
        .collect::<String>()
        .to_ascii_lowercase();
    let budget = MAX_ASCII_NAME.saturating_sub(extension.len() + 1);
    let mut stem = String::new();
    let mut pending_hyphen = false;
    // Bound the work on hostile titles.
    for c in title.chars().take(1024) {
        let folded: Option<&str> = fold(c);
        let mut buffer = [0u8; 4];
        let piece: &str = match folded {
            Some(replacement) => replacement,
            None if c.is_ascii_alphanumeric() => c.encode_utf8(&mut buffer),
            None => {
                pending_hyphen = !stem.is_empty();
                continue;
            }
        };
        let needed = piece.len() + usize::from(pending_hyphen);
        if stem.len() + needed > budget {
            break;
        }
        if pending_hyphen {
            stem.push('-');
            pending_hyphen = false;
        }
        stem.push_str(piece);
    }
    if stem.is_empty() {
        stem.push_str(FALLBACK_STEM);
    }
    if extension.is_empty() { stem } else { format!("{stem}.{extension}") }
}

/// A display file name: the title without control characters or path separators, at
/// most 120 characters, plus the extension.
#[must_use]
pub fn display_file_name(title: &str, extension: &str) -> String {
    let cleaned: String =
        title.chars().filter(|c| !c.is_control() && !matches!(c, '/' | '\\')).take(120).collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    let stem = if cleaned.is_empty() { FALLBACK_STEM } else { cleaned };
    format!("{stem}.{extension}")
}

/// Characters of an RFC 8187 `attr-char` are left alone; everything else is
/// percent-encoded.
const ATTR_CHAR: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'#')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b'-')
    .remove(b'.')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'|')
    .remove(b'~');

/// The `Content-Disposition` value of a download.
///
/// `ascii_name` must come from [`ascii_file_name`] (anything outside `[A-Za-z0-9._-]` is
/// replaced, so the quoted string can never be broken out of). When `display_name` is
/// given and differs, it is added as `filename*` (RFC 6266 / RFC 8187), percent-encoded,
/// so the header stays pure ASCII whatever the title contains.
#[must_use]
pub fn content_disposition(ascii_name: &str, display_name: Option<&str>) -> String {
    let safe: String = ascii_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '_' })
        .collect();
    let mut value = format!("attachment; filename=\"{safe}\"");
    if let Some(display) = display_name.filter(|d| *d != safe) {
        value.push_str("; filename*=UTF-8''");
        value.extend(utf8_percent_encode(display, ATTR_CHAR));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_become_plain_ascii_names() {
        assert_eq!(ascii_file_name("L’Étranger", "epub"), "L-Etranger.epub");
        assert_eq!(
            ascii_file_name("Les Misérables — tome 1", "epub"),
            "Les-Miserables-tome-1.epub"
        );
        assert_eq!(
            ascii_file_name("Œuvres complètes: Maßstäbe", "pdf"),
            "OEuvres-completes-Massstabe.pdf"
        );
        assert_eq!(ascii_file_name("  ...Candide!!!  ", "epub"), "Candide.epub");
        assert_eq!(ascii_file_name("1984", "epub"), "1984.epub");
    }

    #[test]
    fn nothing_usable_falls_back_to_livre() {
        for title in ["", "   ", "戦争と平和", "../../..", "\u{0}\r\n", "💥"] {
            assert_eq!(ascii_file_name(title, "epub"), "livre.epub", "{title:?}");
        }
        assert_eq!(ascii_file_name("x", ""), "x");
        assert_eq!(ascii_file_name("x", "e/p\"u\nb"), "x.epub");
    }

    #[test]
    fn names_are_bounded_and_restricted_to_the_safe_alphabet() {
        let hostile = [
            "a".repeat(5_000),
            "é".repeat(5_000),
            "a b ".repeat(2_000),
            "\"; filename=evil.exe".to_owned(),
            "../../etc/passwd".to_owned(),
            "line\r\nSet-Cookie: x=1".to_owned(),
        ];
        for title in &hostile {
            let name = ascii_file_name(title, "epub");
            assert!(name.len() <= MAX_ASCII_NAME, "{name}");
            assert!(name.ends_with(".epub"));
            assert!(
                name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')),
                "{name}"
            );
            assert!(
                !name.contains("--") && !name.starts_with('-') && !name.contains("-."),
                "{name}"
            );
        }
    }

    #[test]
    fn content_disposition_is_ascii_and_cannot_be_broken_out_of() {
        assert_eq!(
            content_disposition("L-Etranger.epub", Some("L’Étranger — été.epub")),
            "attachment; filename=\"L-Etranger.epub\"; \
             filename*=UTF-8''L%E2%80%99%C3%89tranger%20%E2%80%94%20%C3%A9t%C3%A9.epub"
        );
        assert_eq!(content_disposition("a.epub", None), "attachment; filename=\"a.epub\"");
        assert_eq!(
            content_disposition("a.epub", Some("a.epub")),
            "attachment; filename=\"a.epub\""
        );
        let hostile =
            content_disposition("a\"\r\nX-Evil: 1.epub", Some("b\"\r\nX-Evil: 2\u{0}.epub"));
        assert!(hostile.is_ascii());
        assert!(!hostile.contains('\r') && !hostile.contains('\n'));
        assert_eq!(hostile.matches('"').count(), 2);
        assert!(hostile.starts_with(
            "attachment; filename=\"a___X-Evil__1.epub\"; filename*=UTF-8''b%22%0D%0AX-Evil"
        ));
    }

    #[test]
    fn display_names_keep_the_title_but_no_path_or_control_characters() {
        assert_eq!(display_file_name("L’Étranger", "epub"), "L’Étranger.epub");
        assert_eq!(display_file_name("a/b\\c\r\n", "pdf"), "abc.pdf");
        assert_eq!(display_file_name(" .. ", "epub"), "livre.epub");
        assert_eq!(display_file_name(&"x".repeat(500), "epub").chars().count(), 125);
    }
}
