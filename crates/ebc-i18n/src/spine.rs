//! Which way a title runs along the spine.
//!
//! Two conventions exist for a title set along the spine of an upright book:
//!
//! * **top-to-bottom** ("descending"): the text is rotated 90° clockwise and reads from
//!   head to tail. It is upright when the book lies flat with its front cover up. This is
//!   the rule of ISO 6357 and the practice in English-speaking countries, the
//!   Netherlands and Flanders, Denmark, Norway and Sweden.
//! * **bottom-to-top** ("ascending"): the text is rotated 90° counter-clockwise and reads
//!   from tail to head. This is the practice in France and most of continental Europe
//!   (German-, Italian-, Spanish- and Portuguese-language publishing) and Latin America.
//!
//! The direction belongs to the **book** (its language and market), not to the web site:
//! a French novel bought on a Dutch store still gets a bottom-to-top spine.
//!
//! Source: `docs/research/notes/spine-design.conventions.md` (to be consolidated in
//! `docs/research/11-spine-design.md`). The country table there is recalled from
//! reference works and was not verified against a sample of real spines; German and
//! Spanish practice in particular is a majority, not a rule. Treat the result as a
//! default the user can flip.

use ebc_model::{BookMeta, Locale};
use serde::{Deserialize, Serialize};

/// The reading direction of a title set along the spine of an upright book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpineDirection {
    /// Head to tail ("descending"): text rotated 90° clockwise. ISO 6357; English,
    /// Dutch and Scandinavian publishing.
    TopToBottom,
    /// Tail to head ("ascending"): text rotated 90° counter-clockwise. French, German,
    /// Italian, Spanish and Portuguese publishing.
    BottomToTop,
}

impl SpineDirection {
    /// The rotation to apply to horizontal text, in degrees **clockwise** on the page
    /// (the `ebc-scene` convention, y pointing down): `90.0` for top-to-bottom, `-90.0`
    /// for bottom-to-top.
    ///
    /// On the flat outside spread (back | spine | front), top-to-bottom text has the
    /// tops of its letters towards the front cover; bottom-to-top text towards the back
    /// cover.
    #[must_use]
    pub const fn rotation_degrees(self) -> f64 {
        match self {
            Self::TopToBottom => 90.0,
            Self::BottomToTop => -90.0,
        }
    }

    /// The other direction.
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::TopToBottom => Self::BottomToTop,
            Self::BottomToTop => Self::TopToBottom,
        }
    }

    /// The serialised name: `"top_to_bottom"` or `"bottom_to_top"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopToBottom => "top_to_bottom",
            Self::BottomToTop => "bottom_to_top",
        }
    }
}

/// The direction used when nothing is known about a book: ISO 6357's top-to-bottom.
const DEFAULT_DIRECTION: SpineDirection = SpineDirection::TopToBottom;

/// The spine convention of the publishing market a user-interface locale stands for:
/// top-to-bottom for English and Dutch; bottom-to-top for French, German, Spanish,
/// Italian and Portuguese.
///
/// Prefer [`spine_direction_for_book`] when a book is at hand: the direction follows the
/// book, not the site.
#[must_use]
pub const fn spine_direction(locale: Locale) -> SpineDirection {
    match locale {
        Locale::En | Locale::Nl => SpineDirection::TopToBottom,
        Locale::Fr | Locale::De | Locale::Es | Locale::It | Locale::Pt => {
            SpineDirection::BottomToTop
        }
    }
}

/// The spine convention for a book language given as a BCP 47 tag or ISO 639 code
/// (`"fr"`, `"fr-CA"`, `"de_AT"`, `"ENG"`, `"spa"`), or `None` when the language is
/// missing from the table below.
///
/// | direction | languages |
/// |---|---|
/// | bottom-to-top | French, German, Italian, Spanish, Portuguese, Catalan, Galician, Basque |
/// | top-to-bottom | English, Dutch, Danish, Swedish, Norwegian |
///
/// Region subtags are ignored: the convention follows the language area (French Canada
/// and Latin America print bottom-to-top like their European counterparts).
#[must_use]
pub fn known_spine_direction(bcp47: &str) -> Option<SpineDirection> {
    let primary = bcp47.trim().split(['-', '_']).next().unwrap_or_default();
    if primary.len() > 3 {
        return None;
    }
    let mut lower = [0u8; 3];
    for (slot, byte) in lower.iter_mut().zip(primary.bytes()) {
        *slot = byte.to_ascii_lowercase();
    }
    match lower.get(..primary.len())? {
        b"fr" | b"fra" | b"fre" | b"de" | b"deu" | b"ger" | b"it" | b"ita" | b"es" | b"spa"
        | b"pt" | b"por" | b"ca" | b"cat" | b"gl" | b"glg" | b"eu" | b"eus" | b"baq" => {
            Some(SpineDirection::BottomToTop)
        }
        b"en" | b"eng" | b"nl" | b"nld" | b"dut" | b"da" | b"dan" | b"sv" | b"swe" | b"nb"
        | b"nob" | b"nn" | b"nno" | b"no" | b"nor" => Some(SpineDirection::TopToBottom),
        _ => None,
    }
}

/// The spine convention for a book language (see [`known_spine_direction`] for the
/// table), defaulting to ISO 6357's top-to-bottom for any other, empty or malformed tag.
#[must_use]
pub fn spine_direction_for_language(bcp47: &str) -> SpineDirection {
    known_spine_direction(bcp47).unwrap_or(DEFAULT_DIRECTION)
}

/// The spine convention suggested by an ISBN registration group, as a proxy for the
/// publishing market: `prefix` is a group prefix such as `"978-2"`.
fn direction_for_isbn_group(prefix: &str) -> Option<SpineDirection> {
    match prefix {
        // English language; United States.
        "978-0" | "978-1" | "979-8" => Some(SpineDirection::TopToBottom),
        // Netherlands and Flanders; Denmark, Norway, Sweden.
        "978-90" | "978-94" | "978-87" | "978-82" | "978-91" => Some(SpineDirection::TopToBottom),
        // French language, France; German language; Spain; Italy; Portugal; Brazil.
        "978-2" | "979-10" | "978-3" | "978-84" | "979-13" | "978-88" | "979-12" | "978-972"
        | "978-989" | "978-85" | "978-65" => Some(SpineDirection::BottomToTop),
        _ => None,
    }
}

/// The spine direction to use by default for `book`, in decreasing order of evidence:
///
/// 1. the language of the text (`book.language`), when it is in the table of
///    [`known_spine_direction`];
/// 2. the registration group of the book's ISBN (then of its related ISBNs), as a proxy
///    for where it was published: `978-2` is French-language publishing, `978-0` English…
/// 3. the convention of `ui_locale` — in the absence of any information about the book,
///    the site's own market is the best guess.
///
/// A spine supplied with the cover, or an explicit user choice, takes precedence over
/// this default; that decision belongs to the caller.
#[must_use]
pub fn spine_direction_for_book(book: &BookMeta, ui_locale: Locale) -> SpineDirection {
    let by_language = book.language.as_deref().and_then(known_spine_direction);
    let by_isbn = || {
        book.isbn13
            .iter()
            .chain(book.related_isbns.iter())
            .filter_map(|isbn| isbn.registration_group())
            .find_map(|group| direction_for_isbn_group(group.prefix))
    };
    by_language.or_else(by_isbn).unwrap_or_else(|| spine_direction(ui_locale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use SpineDirection::{BottomToTop, TopToBottom};

    #[test]
    fn locale_conventions() {
        let expected = [
            (Locale::Fr, BottomToTop),
            (Locale::En, TopToBottom),
            (Locale::De, BottomToTop),
            (Locale::Es, BottomToTop),
            (Locale::It, BottomToTop),
            (Locale::Nl, TopToBottom),
            (Locale::Pt, BottomToTop),
        ];
        assert_eq!(expected.len(), Locale::ALL.len());
        for (locale, direction) in expected {
            assert_eq!(spine_direction(locale), direction, "{locale}");
            // The locale's own language tag must agree with the locale convention.
            assert_eq!(spine_direction_for_language(locale.code()), direction);
            assert_eq!(spine_direction_for_language(locale.bcp47()), direction);
        }
    }

    #[test]
    fn language_tags_in_all_their_spellings() {
        for tag in [
            "fr", "FR", "fr-FR", "fr-CA", "fr_BE", "fra", "fre", " fr ", "de", "de-AT", "de_CH",
            "deu", "GER", "it", "ita", "es", "es-MX", "es-419", "spa", "pt", "pt-BR", "por", "ca",
            "cat", "gl", "eu", "baq",
        ] {
            assert_eq!(known_spine_direction(tag), Some(BottomToTop), "{tag:?}");
        }
        for tag in [
            "en", "en-GB", "en_US", "ENG", "nl", "nl-BE", "nld", "dut", "da", "sv", "sv-FI", "nb",
            "nn", "no", "nor",
        ] {
            assert_eq!(known_spine_direction(tag), Some(TopToBottom), "{tag:?}");
        }
    }

    #[test]
    fn unknown_or_malformed_tags_use_the_iso_default() {
        for tag in [
            "",
            " ",
            "-",
            "_",
            "ja",
            "zh-Hant",
            "ru",
            "pl",
            "fi",
            "xx",
            "fren",
            "french",
            "f",
            "frfr",
            "é",
            "fr\u{0}",
            "-fr",
            "\u{0}",
            "🙂",
            "en\u{301}",
        ] {
            assert_eq!(known_spine_direction(tag), None, "{tag:?}");
            assert_eq!(spine_direction_for_language(tag), TopToBottom, "{tag:?}");
        }
        let long = "x".repeat(1_000_000);
        assert_eq!(spine_direction_for_language(&long), TopToBottom);
    }

    #[test]
    fn rotation_and_names() {
        assert_eq!(TopToBottom.rotation_degrees(), 90.0);
        assert_eq!(BottomToTop.rotation_degrees(), -90.0);
        assert_eq!(TopToBottom.flipped(), BottomToTop);
        assert_eq!(BottomToTop.flipped().flipped(), BottomToTop);
        assert_eq!(TopToBottom.as_str(), "top_to_bottom");
        for direction in [TopToBottom, BottomToTop] {
            let json = serde_json::to_string(&direction).expect("serialises");
            assert_eq!(json, format!("\"{}\"", direction.as_str()));
            assert_eq!(serde_json::from_str::<SpineDirection>(&json).expect("parses"), direction);
        }
        assert!(serde_json::from_str::<SpineDirection>("\"sideways\"").is_err());
    }

    fn book(language: Option<&str>, isbn: Option<&str>, related: &[&str]) -> BookMeta {
        let mut book = BookMeta::titled("T");
        book.language = language.map(str::to_owned);
        book.isbn13 = isbn.map(|s| s.parse().expect("valid test ISBN"));
        book.related_isbns = related.iter().map(|s| s.parse().expect("valid test ISBN")).collect();
        book
    }

    #[test]
    fn book_direction_follows_the_book_not_the_site() {
        // A French novel on a Dutch site: bottom-to-top.
        assert_eq!(spine_direction_for_book(&book(Some("fr"), None, &[]), Locale::Nl), BottomToTop);
        // An English novel on a French site: top-to-bottom.
        assert_eq!(
            spine_direction_for_book(&book(Some("en-GB"), None, &[]), Locale::Fr),
            TopToBottom
        );
        // Language beats ISBN: an English-language book from a French publisher.
        assert_eq!(
            spine_direction_for_book(&book(Some("en"), Some("9782070360024"), &[]), Locale::Fr),
            TopToBottom
        );
    }

    #[test]
    fn book_direction_falls_back_to_isbn_group_then_locale() {
        // No language: the ISBN group decides.
        for (isbn, direction) in [
            ("9782070360024", BottomToTop), // 978-2 French language
            ("9791090636071", BottomToTop), // 979-10 France
            ("9783161484100", BottomToTop), // 978-3 German language
            ("9788437604947", BottomToTop), // 978-84 Spain
            ("9788806219352", BottomToTop), // 978-88 Italy
            ("9789720046819", BottomToTop), // 978-972 Portugal
            ("9780306406157", TopToBottom), // 978-0 English language
            ("9781402894626", TopToBottom), // 978-1 English language
            ("9798602405453", TopToBottom), // 979-8 United States
            ("9789027239617", TopToBottom), // 978-90 Netherlands
        ] {
            for locale in Locale::ALL {
                assert_eq!(
                    spine_direction_for_book(&book(None, Some(isbn), &[]), locale),
                    direction,
                    "{isbn} on a {locale} site"
                );
            }
        }
        // Unknown language tag: same as no language.
        assert_eq!(
            spine_direction_for_book(&book(Some("ja"), Some("9782070360024"), &[]), Locale::En),
            BottomToTop
        );
        // An ISBN group without a known convention (Japan), then a related ISBN that has one.
        let japan = "9784873113364";
        assert_eq!(
            spine_direction_for_book(&book(None, Some(japan), &[]), Locale::Fr),
            BottomToTop
        );
        assert_eq!(
            spine_direction_for_book(&book(None, Some(japan), &[]), Locale::En),
            TopToBottom
        );
        assert_eq!(
            spine_direction_for_book(&book(None, Some(japan), &["9780306406157"]), Locale::Fr),
            TopToBottom
        );
        // Nothing at all: the site's convention.
        for locale in Locale::ALL {
            assert_eq!(
                spine_direction_for_book(&book(None, None, &[]), locale),
                spine_direction(locale)
            );
        }
    }
}
