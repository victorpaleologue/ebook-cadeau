//! [`ComposedCardRenderer`]: the server's [`CardRenderer`] on top of the card composer
//! `ebc-card` (cargo feature `card`).
//!
//! | `format` | Card |
//! |---|---|
//! | `poche` / `a6` / `digest` / `a5` | [`ebc_layout::CardFormat`] presets of the same names, 10 mm spine |
//!
//! | `profile` | Print profile |
//! |---|---|
//! | `pro` | [`PrintProfile::upload_eu`]: trim + 3 mm bleed, no marks, PDF/X-4 with the composer's bundled FOGRA39 output intent — what web-to-print portals accept |
//! | `home-a4` / `home-letter` | [`PrintProfile::home`] on that sheet: the card centred on the page with cut and fold guides, plain PDF |
//!
//! The cover is handed to the composer **with the rights basis the partner declared**
//! ([`CoverRights`]); with `none` it is not handed over at all and the composer produces
//! the typographic card. The composer's rights gate (a promotion-only licence cannot go
//! to a printer) surfaces as [`CardRenderError::RightsRefused`].
//!
//! The output is deterministic for a given gift: the creation date of the PDF is the
//! gift's, nothing reads the clock.

use ebc_card::{CardError, CoverInput, FontSet, RightsBasis};
use ebc_layout::{PrintProfile, Sheet};

use super::{CardFormat, CardProfile, CardRenderError, CardRenderer, CardRequest, CoverRights};

/// Renders cards with `ebc-card` and its bundled fonts.
#[derive(Debug, Clone)]
pub struct ComposedCardRenderer {
    fonts: FontSet,
}

impl ComposedCardRenderer {
    /// A renderer using the fonts bundled with the composer (SIL OFL).
    ///
    /// # Errors
    /// [`CardRenderError::Failed`] if a bundled font cannot be parsed (a corrupted
    /// build; the composer's tests rule it out).
    pub fn new() -> Result<Self, CardRenderError> {
        let fonts =
            ebc_card::bundled_fonts().map_err(|e| CardRenderError::Failed(e.to_string()))?;
        Ok(Self { fonts })
    }

    /// A renderer with the caller's fonts.
    #[must_use]
    pub fn with_fonts(fonts: FontSet) -> Self {
        Self { fonts }
    }
}

/// The composer's format of a `format` parameter.
fn format_of(format: CardFormat) -> ebc_layout::CardFormat {
    match format {
        CardFormat::Poche => ebc_layout::CardFormat::poche(),
        CardFormat::A6 => ebc_layout::CardFormat::a6(),
        CardFormat::Digest => ebc_layout::CardFormat::digest(),
        CardFormat::A5 => ebc_layout::CardFormat::a5(),
    }
}

/// The composer's print profile of a `profile` parameter.
fn profile_of(profile: CardProfile) -> PrintProfile {
    match profile {
        CardProfile::Pro => PrintProfile::upload_eu(),
        CardProfile::HomeA4 => PrintProfile::home(Sheet::A4),
        CardProfile::HomeLetter => PrintProfile::home(Sheet::Letter),
    }
}

/// The composer's rights basis of a declaration; `None` when nothing may be printed.
fn basis_of(rights: CoverRights) -> Option<RightsBasis> {
    match rights {
        CoverRights::None => None,
        CoverRights::PrintLicence => Some(RightsBasis::PrintLicence),
        CoverRights::PromoAllMedia => Some(RightsBasis::PromoAllMedia),
        CoverRights::RightsHolderDeclared => Some(RightsBasis::RightsHolderDeclared),
        CoverRights::PublicDomain => Some(RightsBasis::PublicDomain),
    }
}

/// `host[:port]` of a base URL, as people type it.
fn typed_host(public_url: &str) -> String {
    let rest = public_url.trim();
    let rest = rest.split_once("://").map_or(rest, |(_, r)| r);
    rest.split(['/', '?', '#']).next().unwrap_or(rest).to_lowercase()
}

/// Sorts the composer's errors into what the API answers.
fn render_error(error: &CardError) -> CardRenderError {
    match error {
        CardError::RightsRouteRefused { .. } | CardError::AdaptationNotAuthorised { .. } => {
            CardRenderError::RightsRefused(error.to_string())
        }
        // The request cannot be laid out as asked: the caller can change it.
        CardError::EmptyTitle
        | CardError::InvalidClaimUrl(_)
        | CardError::Layout(_)
        | CardError::Qr(_)
        | CardError::CoverDecode(_)
        | CardError::CoverTooLarge { .. }
        | CardError::CoverColourSpace { .. }
        | CardError::FormatTooSmall { .. } => CardRenderError::Unprocessable(error.to_string()),
        _ => CardRenderError::Failed(error.to_string()),
    }
}

impl CardRenderer for ComposedCardRenderer {
    fn render(&self, request: &CardRequest) -> Result<Vec<u8>, CardRenderError> {
        let mut card =
            ebc_card::CardRequest::new(request.book.clone(), request.code, &request.public_url);
        // The composer derives a bare host from the base URL; a development server on a
        // port must print the port too, or the address typed on an e-reader leads nowhere.
        let host = typed_host(&request.public_url);
        card.claim_url.clone_from(&request.claim_url);
        card.help_url = Some(format!("{host}/aide"));
        card.display_url = host;
        card.giver.clone_from(&request.giver_name);
        card.recipient.clone_from(&request.recipient_name);
        card.message.clone_from(&request.message);
        card.store_name.clone_from(&request.store_name);
        card.locale = request.locale;
        card.format = format_of(request.format);
        card.profile = profile_of(request.profile);
        card.created_unix = Some(request.created_unix);
        card.expires_unix = request.expires_unix;
        card.cover = match (&request.cover, basis_of(request.cover_rights)) {
            (Some((bytes, _media_type)), Some(rights)) => {
                CoverInput::Encoded { bytes: bytes.clone(), rights }
            }
            // No cover, or a cover nobody declared a right to print: the typographic card.
            _ => CoverInput::None,
        };
        let (pdf, warnings) =
            ebc_card::render_pdf(&card, &self.fonts).map_err(|error| render_error(&error))?;
        if !warnings.is_empty() {
            tracing::debug!(
                gift = %request.gift_id,
                warnings = warnings.len(),
                first = %warnings[0],
                "card rendered with warnings"
            );
        }
        Ok(pdf)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ebc_model::{BookMeta, GiftCode, Locale};

    use super::*;

    fn request(format: CardFormat, profile: CardProfile) -> CardRequest {
        let code = GiftCode::from_payload(123_456_789).unwrap();
        let public_url = "http://127.0.0.1:8080".to_owned();
        CardRequest {
            gift_id: "g_test".to_owned(),
            book: BookMeta::titled("Le Horla"),
            giver_name: Some("Camille".to_owned()),
            recipient_name: Some("Sacha".to_owned()),
            message: Some("Joyeux anniversaire !".to_owned()),
            locale: Locale::Fr,
            code,
            claim_url: ebc_model::claim_url(&public_url, &code),
            claim_url_qr: ebc_model::claim_url_qr(&public_url, &code),
            public_url,
            delivery_type: "hosted_file",
            store_name: None,
            cover: None,
            cover_rights: CoverRights::None,
            expires_unix: None,
            format,
            profile,
            created_unix: 1_794_441_600,
        }
    }

    #[test]
    fn every_format_renders_for_the_printer_and_poche_and_a6_for_home_sheets() {
        let renderer = ComposedCardRenderer::new().unwrap();
        for format in [CardFormat::Poche, CardFormat::A6, CardFormat::Digest, CardFormat::A5] {
            let pdf = renderer.render(&request(format, CardProfile::Pro)).unwrap();
            assert!(pdf.starts_with(b"%PDF-"), "{format:?}");
            // Deterministic: the same gift gives the same bytes.
            assert_eq!(pdf, renderer.render(&request(format, CardProfile::Pro)).unwrap());
        }
        for profile in [CardProfile::HomeA4, CardProfile::HomeLetter] {
            for format in [CardFormat::Poche, CardFormat::A6] {
                let pdf = renderer.render(&request(format, profile)).unwrap();
                assert!(pdf.starts_with(b"%PDF-"), "{format:?} {profile:?}");
            }
        }
        // A card that does not fit the sheet is the caller's problem, not a failure.
        let too_big = renderer.render(&request(CardFormat::A5, CardProfile::HomeA4));
        assert!(matches!(too_big, Err(CardRenderError::Unprocessable(_))), "{too_big:?}");
    }

    #[test]
    fn a_cover_is_only_handed_to_the_composer_with_a_declared_basis() {
        let renderer = ComposedCardRenderer::new().unwrap();
        // Bytes that are not an image: decoding them fails — but only if they are used.
        let junk = Some((b"not an image".to_vec(), "image/png".to_owned()));
        let mut undeclared = request(CardFormat::Poche, CardProfile::Pro);
        undeclared.cover.clone_from(&junk);
        assert!(renderer.render(&undeclared).is_ok(), "an undeclared cover is never touched");
        let mut declared = undeclared.clone();
        declared.cover_rights = CoverRights::PrintLicence;
        assert!(matches!(renderer.render(&declared), Err(CardRenderError::Unprocessable(_))));
        // The gate: a promotion-only licence cannot go to a printer.
        let mut promo = declared.clone();
        promo.cover_rights = CoverRights::PromoAllMedia;
        assert!(matches!(renderer.render(&promo), Err(CardRenderError::RightsRefused(_))));
    }

    #[test]
    fn the_printed_address_keeps_the_port_of_a_development_server() {
        assert_eq!(typed_host("http://127.0.0.1:8080"), "127.0.0.1:8080");
        assert_eq!(typed_host("https://EBK.example/"), "ebk.example");
        assert_eq!(typed_host("ebk.example"), "ebk.example");
    }
}
