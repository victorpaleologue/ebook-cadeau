//! The seam through which the server renders a print-ready card
//! (`GET /api/partner/v1/gifts/{id}/card.pdf`, `docs/api.md` §4.2).
//!
//! With the cargo feature `card` (on by default) the server renders cards with the card
//! composer `ebc-card` ([`ComposedCardRenderer`]). Without the feature it ships with
//! [`NoCardRenderer`], which makes the endpoint answer `501 not_implemented`; an
//! embedding application can plug in any other [`CardRenderer`] with
//! [`AppStateBuilder::card_renderer`](crate::AppStateBuilder::card_renderer).
//!
//! # Covers and rights
//!
//! A cover the partner uploaded (`PUT …/cover`) is **printed only under a rights basis
//! the partner declared** ([`CoverRights`]): in the gift creation body
//! (`"cover_rights": "print_licence"`) or on the upload (`PUT …/cover?cover_rights=…`,
//! which is covered by the request signature). Without a declaration the cover is kept
//! but never enters a print file: the card is the typographic one (title and author set
//! as text).

use std::sync::Arc;

use ebc_model::{BookMeta, GiftCode, Locale};
use serde::{Deserialize, Serialize};

#[cfg(feature = "card")]
mod composer;
#[cfg(feature = "card")]
pub use composer::ComposedCardRenderer;

/// The rights basis a partner declares for the cover it uploads: why this image may be
/// printed on a card. The names and their meaning are those of `ebc-covers`
/// (`docs/research/07-legal-covers-and-reseller-rights.md`); the conditions that go with
/// a basis are the partner's to meet — the server records the declaration and applies
/// the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverRights {
    /// Nothing declared: the cover is not printed, the card is typographic.
    #[default]
    None,
    /// A per-title print licence from the publisher, passed on by the partner: the full
    /// cover, at home and at a printer.
    PrintLicence,
    /// A licence for promotion on all media: reduced-size layout on home-printed cards
    /// only; refused for professional printing.
    PromoAllMedia,
    /// The partner is (or acts for) the author or publisher.
    RightsHolderDeclared,
    /// The artwork is in the public domain in the country of use.
    PublicDomain,
}

impl CoverRights {
    /// Parses `none|print_licence|promo_all_media|rights_holder_declared|public_domain`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "none" => Self::None,
            "print_licence" => Self::PrintLicence,
            "promo_all_media" => Self::PromoAllMedia,
            "rights_holder_declared" => Self::RightsHolderDeclared,
            "public_domain" => Self::PublicDomain,
            _ => return None,
        })
    }

    /// The name used in the API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PrintLicence => "print_licence",
            Self::PromoAllMedia => "promo_all_media",
            Self::RightsHolderDeclared => "rights_holder_declared",
            Self::PublicDomain => "public_domain",
        }
    }
}

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
    /// Professional printing: trim + 3 mm bleed, no marks, PDF/X-4 (what web-to-print
    /// portals ask for).
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
    /// The public base URL of the service (`EBC_PUBLIC_URL`), whose host is the address
    /// printed for people who type it.
    pub public_url: String,
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
    /// The rights basis the partner declared for that cover. With
    /// [`CoverRights::None`] the cover must not be printed.
    pub cover_rights: CoverRights,
    /// When the gift expires, if it does (printed on the card).
    pub expires_unix: Option<i64>,
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
    /// The declared rights basis of the cover does not allow this print route, e.g. a
    /// promotion-only licence on a professional profile (`422 cover_rights_refused`).
    #[error("the cover may not be printed this way: {0}")]
    RightsRefused(String),
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

/// The renderer a server uses unless another one is plugged in: the card composer with
/// the cargo feature `card`, [`NoCardRenderer`] without it (or if the composer's bundled
/// fonts cannot be loaded, which is logged).
#[must_use]
pub fn default_renderer() -> Arc<dyn CardRenderer> {
    #[cfg(feature = "card")]
    {
        match ComposedCardRenderer::new() {
            Ok(renderer) => return Arc::new(renderer),
            Err(error) => {
                tracing::error!(%error, "the card composer is unavailable: card.pdf answers 501");
            }
        }
    }
    Arc::new(NoCardRenderer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_rights_round_trip_and_default_to_none() {
        for rights in [
            CoverRights::None,
            CoverRights::PrintLicence,
            CoverRights::PromoAllMedia,
            CoverRights::RightsHolderDeclared,
            CoverRights::PublicDomain,
        ] {
            assert_eq!(CoverRights::parse(rights.as_str()), Some(rights));
            assert_eq!(
                serde_json::to_string(&rights).ok(),
                Some(format!("\"{}\"", rights.as_str()))
            );
        }
        assert_eq!(CoverRights::default(), CoverRights::None);
        // A private copy is a user's basis, never a partner's; anything else is unknown.
        for bad in ["", "user_private_copy", "PRINT_LICENCE", "print-licence", "all"] {
            assert_eq!(CoverRights::parse(bad), None, "{bad:?}");
        }
    }

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
