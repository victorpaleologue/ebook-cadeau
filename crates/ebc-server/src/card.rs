//! The seam through which the server renders a print-ready card
//! (`GET /api/partner/v1/gifts/{id}/card.pdf`, `docs/api.md` §4.2).
//!
//! The composer (`ebc-card`) is not written yet, so the server ships with
//! [`NoCardRenderer`], which makes the endpoint answer `501 not_implemented`. A later
//! change implements [`CardRenderer`] on top of `ebc-card` and passes it to
//! [`AppStateBuilder::card_renderer`](crate::AppStateBuilder::card_renderer); nothing
//! else in the server needs to change.

use ebc_model::{BookMeta, GiftCode, Locale};

/// Card size, the `format` query parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardFormat {
    /// 110 × 178 mm covers.
    Poche,
    /// 105 × 148 mm covers.
    A6,
    /// 130 × 200 mm covers.
    Digest,
    /// 148 × 210 mm covers.
    A5,
}

impl CardFormat {
    /// Parses `poche|a6|digest|a5`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "poche" => Self::Poche,
            "a6" => Self::A6,
            "digest" => Self::Digest,
            "a5" => Self::A5,
            _ => return None,
        })
    }

    /// The parameter value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Poche => "poche",
            Self::A6 => "a6",
            Self::Digest => "digest",
            Self::A5 => "a5",
        }
    }
}

/// Print profile, the `profile` query parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardProfile {
    /// Professional printing: bleed, crop marks, PDF/X.
    Pro,
    /// Home printing on A4.
    HomeA4,
    /// Home printing on US Letter.
    HomeLetter,
}

impl CardProfile {
    /// Parses `pro|home-a4|home-letter`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "pro" => Self::Pro,
            "home-a4" => Self::HomeA4,
            "home-letter" => Self::HomeLetter,
            _ => return None,
        })
    }

    /// The parameter value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pro => "pro",
            Self::HomeA4 => "home-a4",
            Self::HomeLetter => "home-letter",
        }
    }
}

/// Everything a renderer needs to compose one card.
#[derive(Debug, Clone)]
pub struct CardRequest {
    /// Gift identifier (for the PDF metadata).
    pub gift_id: String,
    /// The book.
    pub book: BookMeta,
    /// Who gives.
    pub giver_name: Option<String>,
    /// Who receives.
    pub recipient_name: Option<String>,
    /// Personal message.
    pub message: Option<String>,
    /// Language of the card.
    pub locale: Locale,
    /// The code to print.
    pub code: GiftCode,
    /// The URL printed in clear.
    pub claim_url: String,
    /// The URL to encode in the QR code (upper case).
    pub claim_url_qr: String,
    /// `hosted_file`, `reseller_redirect` or `reseller_licence`: selects the printed steps.
    pub delivery_type: &'static str,
    /// The store's name, for reseller deliveries.
    pub store_name: Option<String>,
    /// The cover image the partner uploaded and its media type.
    pub cover: Option<(Vec<u8>, String)>,
    /// Card size.
    pub format: CardFormat,
    /// Print profile.
    pub profile: CardProfile,
    /// Creation time of the gift, for deterministic PDF dates.
    pub created_unix: i64,
}

/// Why a card could not be rendered.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CardRenderError {
    /// This server has no card composer (`501`).
    #[error("this server cannot render cards yet")]
    NotImplemented,
    /// The gift cannot be laid out as asked, e.g. no cover was supplied (`422`).
    #[error("the card cannot be composed: {0}")]
    Unprocessable(String),
    /// The composer failed (`500`).
    #[error("card rendering failed: {0}")]
    Failed(String),
}

/// Renders the print-ready PDF of a card.
pub trait CardRenderer: Send + Sync + 'static {
    /// Composes the card.
    ///
    /// # Errors
    /// See [`CardRenderError`].
    fn render(&self, request: &CardRequest) -> Result<Vec<u8>, CardRenderError>;
}

/// The default renderer: there is none, the endpoint answers `501`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCardRenderer;

impl CardRenderer for NoCardRenderer {
    fn render(&self, _request: &CardRequest) -> Result<Vec<u8>, CardRenderError> {
        Err(CardRenderError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_round_trip_and_reject_everything_else() {
        for format in [CardFormat::Poche, CardFormat::A6, CardFormat::Digest, CardFormat::A5] {
            assert_eq!(CardFormat::parse(format.as_str()), Some(format));
        }
        for profile in [CardProfile::Pro, CardProfile::HomeA4, CardProfile::HomeLetter] {
            assert_eq!(CardProfile::parse(profile.as_str()), Some(profile));
        }
        for bad in ["", "POCHE", "a4", "home", "pro ", "../x"] {
            assert_eq!(CardFormat::parse(bad), None, "{bad:?}");
            assert_eq!(CardProfile::parse(bad), None, "{bad:?}");
        }
    }
}
