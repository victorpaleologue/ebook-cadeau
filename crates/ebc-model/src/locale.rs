//! Supported user-interface locales.

use serde::{Deserialize, Serialize};

/// A user-interface language supported by eBook Cadeau.
///
/// Regional variants are folded into their language; the tag returned by
/// [`Locale::bcp47`] is the default region used for formatting.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// French (default).
    #[default]
    Fr,
    /// English.
    En,
    /// German.
    De,
    /// Spanish.
    Es,
    /// Italian.
    It,
    /// Dutch.
    Nl,
    /// Portuguese.
    Pt,
}

impl Locale {
    /// Every supported locale, in display order.
    pub const ALL: [Locale; 7] =
        [Locale::Fr, Locale::En, Locale::De, Locale::Es, Locale::It, Locale::Nl, Locale::Pt];

    /// Two-letter ISO 639-1 code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Fr => "fr",
            Self::En => "en",
            Self::De => "de",
            Self::Es => "es",
            Self::It => "it",
            Self::Nl => "nl",
            Self::Pt => "pt",
        }
    }

    /// Default BCP 47 tag including a region.
    #[must_use]
    pub const fn bcp47(self) -> &'static str {
        match self {
            Self::Fr => "fr-FR",
            Self::En => "en-GB",
            Self::De => "de-DE",
            Self::Es => "es-ES",
            Self::It => "it-IT",
            Self::Nl => "nl-NL",
            Self::Pt => "pt-PT",
        }
    }

    /// Parses a BCP 47 tag or ISO 639 code such as `"fr"`, `"fr-CA"`, `"FRA"`, `"de_AT"`.
    /// Returns `None` for unsupported languages.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        let lang = tag.trim().split(['-', '_']).next()?.to_ascii_lowercase();
        Some(match lang.as_str() {
            "fr" | "fra" | "fre" => Self::Fr,
            "en" | "eng" => Self::En,
            "de" | "deu" | "ger" => Self::De,
            "es" | "spa" => Self::Es,
            "it" | "ita" => Self::It,
            "nl" | "nld" | "dut" => Self::Nl,
            "pt" | "por" => Self::Pt,
            _ => return None,
        })
    }

    /// Picks the best supported locale from an HTTP `Accept-Language` header value,
    /// honouring `q` weights. Returns `None` if nothing matches.
    #[must_use]
    pub fn from_accept_language(header: &str) -> Option<Self> {
        let mut best: Option<(f32, Self)> = None;
        for part in header.split(',') {
            let mut it = part.split(';');
            let Some(locale) = it.next().and_then(Self::from_tag) else { continue };
            let q = it
                .find_map(|p| {
                    p.trim().strip_prefix("q=").and_then(|v| v.trim().parse::<f32>().ok())
                })
                .unwrap_or(1.0);
            if q > 0.0 && best.is_none_or(|(bq, _)| q > bq) {
                best = Some((q, locale));
            }
        }
        best.map(|(_, l)| l)
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags() {
        assert_eq!(Locale::from_tag("fr-CA"), Some(Locale::Fr));
        assert_eq!(Locale::from_tag("de_AT"), Some(Locale::De));
        assert_eq!(Locale::from_tag("ENG"), Some(Locale::En));
        assert_eq!(Locale::from_tag("ja"), None);
        assert_eq!(Locale::from_tag(""), None);
    }

    #[test]
    fn accept_language() {
        assert_eq!(Locale::from_accept_language("ja, de;q=0.7, en;q=0.9"), Some(Locale::En));
        assert_eq!(Locale::from_accept_language("fr-FR,fr;q=0.9,en;q=0.8"), Some(Locale::Fr));
        assert_eq!(Locale::from_accept_language("ja;q=1, en;q=0"), None);
        assert_eq!(Locale::from_accept_language("garbage;;q=x"), None);
    }

    #[test]
    fn serde_uses_lowercase_codes() {
        assert_eq!(serde_json::to_string(&Locale::Nl).expect("serialises"), "\"nl\"");
        assert_eq!(serde_json::from_str::<Locale>("\"it\"").expect("parses"), Locale::It);
    }
}
