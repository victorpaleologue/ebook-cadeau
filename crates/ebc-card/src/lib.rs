//! Composes the complete gift card — outside spread, spine, inside with the QR code —
//! into a printable [`ebc_scene::Document`], and writes it as a PDF.
//!
//! ```text
//!   page 1, outside:   back cover   | spine |  front cover
//!   page 2, inside:    inside-front |       |  inside-back
//!                      (the gift:           (how to get the book: three steps,
//!                       for…, message,       QR code, address, code, the line for
//!                       from…)               people without a phone, legal lines)
//! ```
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "bundled-fonts")]
//! # fn main() -> Result<(), ebc_card::CardError> {
//! use ebc_card::{CardRequest, bundled_fonts, render_pdf};
//! use ebc_layout::{PrintProfile, Sheet};
//! use ebc_model::{BookMeta, Contributor, GiftCode};
//!
//! let mut book = BookMeta::titled("Les Vagues du soir");
//! book.contributors.push(Contributor::author("Jeanne Roy"));
//! let code = GiftCode::from_payload(123_456_789).expect("in range");
//!
//! let mut request = CardRequest::new(book, code, "https://ebk.example");
//! request.profile = PrintProfile::home(Sheet::A4);     // print at home, both sides
//! request.giver = Some("Alice".into());
//! request.recipient = Some("Bob".into());
//!
//! let (pdf, warnings) = render_pdf(&request, &bundled_fonts()?)?;
//! assert!(pdf.starts_with(b"%PDF-"));
//! assert!(warnings.is_empty());
//! # Ok(()) }
//! # #[cfg(not(feature = "bundled-fonts"))]
//! # fn main() {}
//! ```
//!
//! # Covers and rights
//!
//! A cover image only reaches the print file with a rights basis ([`RightsBasis`], the
//! type of `ebc-covers`) that opens the route of the profile ([`PrintRoute`]); see
//! [`rights`] for the table. With
//! no image, or with basis `none`, the output is the **typographic card**: title,
//! subtitle, author and publisher as text in the card's own fonts on a flat ground — no
//! logo, no blurb, no barcode. Requests that break the rules are refused with a typed
//! error ([`CardError::RightsRouteRefused`], [`CardError::AdaptationNotAuthorised`]),
//! never repaired silently.
//!
//! A printed cover is never altered: uniform scaling, no rotation and, by default, no
//! crop (it is letterboxed, at least 3 mm inside the trim, on the flat ground of the
//! outside).
//! `trim_authorised` allows full-bleed placement; `adaptation_authorised` — with a
//! `print_licence` or `rights_holder_declared` basis — allows the spine styles that
//! sample or re-fit artwork. A credit line for the cover is printed on the back panel.
//! These are conservative defaults from
//! `docs/research/07-legal-covers-and-reseller-rights.md`, pending a lawyer's review;
//! the layout decisions are documented on the two page modules' source.
//!
//! # The ground of the outside
//!
//! Back, spine and front share **one flat ground**, so that the card reads as one
//! object, like a book jacket. [`CardRequest::ground`] sets it; by default it is
//!
//! * for the typographic card, a colour picked from a fixed palette by a hash of the
//!   title;
//! * for a card with a cover, one colour *computed* from the cover — never pixels: the
//!   colour of its frame if it has a plain one, else of its plain binding edge, else its
//!   dominant colour — with the text on it black or white, whichever contrasts more;
//! * **paper white** when print argues against that colour: no process channel reaches
//!   [`GROUND_MIN_TINT_PERCENT`] (too faint to print), the text on it would stay under
//!   [`GROUND_MIN_CONTRAST`]:1 with black counted as ink on paper (saturated mid-tones),
//!   or its build under the printer's ink limit is more than [`GROUND_MAX_DELTA_E`]
//!   (ΔE2000) from the colour. A dark colour above the limit is otherwise **kept**, built
//!   with more black and less colour ink (see *Colour* below).
//!
//! In a PDF/X file the ground goes through the same ICC transform as the cover, so on
//! press it matches the cover's own frame or edge colour: see *Colour* below.
//!
//! A dark or heavy outside on a professional card — more than [`DARK_CREASE_INK_PERCENT`]
//! of ink or darker than L\* [`DARK_CREASE_LSTAR`] within [`CREASE_BAND_MM`] of a crease
//! — comes with [`CardWarning::DarkAcrossCreases`]: ask the printer for matt or
//! soft-touch lamination and the paper grain parallel to the spine, so that the folds do
//! not crack.
//!
//! # Colour
//!
//! Text, QR code and rules are black on the black plate only (grey in a home PDF).
//! In PDF/X files the cover, any image on the spine and the flat grounds are converted
//! to the output intent's CMYK by **one ICC transform** (`ebc_color::SrgbToCmyk`:
//! relative colorimetric with black-point compensation, from the output profile's own
//! tables), so that the ground and the cover's frame meet on press. That transform
//! respects the print profile's **total ink limit** (`PrintProfile::max_total_ink_percent`:
//! 240 % for upload/print-on-demand, 300 % for offset): every colour the profile would
//! build above it — a dark ground, the dark pixels of the cover or of an image spine — is
//! rebuilt by grey-component replacement, the build closest to the original (ΔE2000
//! through the profile; `ebc_color::ink`). Flat colours are then cleaned of tints under
//! 5 %. A CMYK JPEG cover is passed through unchanged (and not ink-limited: it is assumed
//! to be built for the output condition already). Only when the output profile cannot be read (an ICC v4
//! profile with `mBA` tables, the CC0 fallback) does the cover stay sRGB
//! ([`CardWarning::CoverNotConverted`]) — and then [`render_pdf`] refuses the file
//! ([`CardError::CoverColourSpace`]) unless the print profile allows RGB
//! (`PrintProfile::allow_rgb`, off for the professional profiles; never in PDF/X-1a).
//! A home PDF keeps sRGB images and colours.
//!
//! The output intent of PDF/X files is, by default, the bundled "ISO Coated v2 300%
//! (basICColor)" FOGRA39 profile ([`OutputProfile::bundled_fogra39`]); pass the profile
//! your printer names in [`CardRequest::output_profile`].
//!
//! # Features
//!
//! * `bundled-fonts` (default) — [`bundled_fonts`].
//! * `bundled-fogra39` (default) — the FOGRA39 output profile, about 1 MB.
//! * `bundled-icc` (default) — the 8 kB CC0 fallback profile, used only when
//!   `bundled-fogra39` is off, with [`CardWarning::FallbackOutputProfile`].
//!
//! # Not implemented
//!
//! Colour management handles `lut8`/`lut16` CMYK output profiles only, ignores the
//! profiles embedded in covers, judges a CMYK JPEG cover's ground from a naive RGB
//! decoding of it and does not ink-limit such a cover (it is passed through); no right-to-left or vertical
//! setting (one run, one direction, as in `ebc-text`); no font fallback for body text
//! (characters the font lacks are left out, with a warning); the single-sided home
//! variant with a flap and per-printer crease encodings of the print research are not
//! built.

#![forbid(unsafe_code)]

mod compose;
mod cover;
mod crease;
mod error;
mod ground;
mod inside;
pub mod inspect;
mod outside;
mod paint;
mod request;
pub mod rights;
pub mod svg;
mod text;

use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Document, ImageData, PixelFormat};

pub use compose::{MARK_CLEARANCE_MM, NOTES_LAYER};
pub use cover::{
    MAX_COVER_BYTES, MAX_COVER_PIXELS, MAX_COVER_SIDE_PX, MAX_EMBED_PPI, RESAMPLE_PPI,
};
pub use crease::{CREASE_BAND_MM, DARK_CREASE_INK_PERCENT, DARK_CREASE_LSTAR};
pub use error::{CardError, CardWarning, TextField};
pub use ground::{GROUND_MAX_DELTA_E, GROUND_MIN_CONTRAST, GROUND_MIN_TINT_PERCENT};
pub use inside::{CODE_PT, QR_BOX_MAX_MM, QR_BOX_MM, QR_MIN_SYMBOL_MM, STEPS_PT, qr_payload};
pub use outside::{MIN_COVER_INSET_MM, REDUCED_COVER_SHARE};
#[cfg(feature = "bundled-fonts")]
pub use request::bundled_fonts;
pub use request::{CardRequest, CoverInput, FontSet, OutputProfile, SpineChoice};
pub use rights::{Placement, PrintRoute, RightsBasis};

/// Composes the card: page 1 is the outside (back | spine | front), page 2 the inside.
///
/// Returns the document and what the caller should know about it. The same request
/// always gives the same document: nothing reads a clock or a random source.
///
/// # Errors
/// See [`CardError`]. In particular a cover whose rights basis does not open the
/// profile's print route, a spine style that adapts artwork without authorisation, a card
/// that does not fit its sheet, a blank title, a PDF/X profile without `created_unix`,
/// and cover bytes that are not a decodable JPEG, PNG or WebP image.
pub fn compose(
    req: &CardRequest,
    fonts: &FontSet,
) -> Result<(Document, Vec<CardWarning>), CardError> {
    compose::compose(req, fonts, compose::Purpose::Print)
}

/// [`compose`], then [`ebc_pdf::write_pdf`]: the print file and the warnings.
///
/// Layers are flattened, as printers ask — except for a proof with the guides overlay
/// ([`CardRequest::guides`]), which keeps them so that a viewer can switch them off, and
/// is therefore written as a plain PDF without the PDF/X label and output intent (a
/// proof is not a print file). The bytes are deterministic.
///
/// Two print gates apply to professional (PDF/X) files, which [`compose`] only reports
/// as warnings: a cover below the profile's minimum resolution, and RGB images in a
/// profile that does not allow RGB (when the output profile cannot be read by the
/// colour-management module, so nothing could be converted).
///
/// # Errors
/// Those of [`compose`]; [`CardError::CoverResolutionTooLow`] (offer the typographic
/// card instead); [`CardError::CoverColourSpace`]; and [`CardError::Pdf`] when the writer
/// refuses the document.
pub fn render_pdf(
    req: &CardRequest,
    fonts: &FontSet,
) -> Result<(Vec<u8>, Vec<CardWarning>), CardError> {
    let (doc, warnings) = compose(req, fonts)?;
    print_gates(req, &doc, &warnings)?;
    let options = WriteOptions { compress: true, keep_layers: req.guides };
    Ok((write_pdf(&doc, &options)?, warnings))
}

/// The checks that stop a professional print file from being written.
fn print_gates(
    req: &CardRequest,
    doc: &Document,
    warnings: &[CardWarning],
) -> Result<(), CardError> {
    if doc.standard == ebc_scene::PdfStandard::Plain {
        return Ok(());
    }
    // `docs/research/03-print-requirements.md` §4.2: below the error threshold a
    // professional order is blocked and the typographic card is offered instead.
    for warning in warnings {
        if let CardWarning::LowResolution { ppi, threshold_ppi, below_minimum: true } = warning {
            return Err(CardError::CoverResolutionTooLow {
                ppi: *ppi,
                minimum_ppi: *threshold_ppi,
            });
        }
    }
    // §3.2 / §3.4, implication 23: CMYK and grey only, unless the profile allows RGB.
    let rgb_allowed =
        req.profile.allow_rgb && req.profile.standard != ebc_scene::PdfStandard::X1a2001;
    let rgb = doc.images.iter().any(|image| match &image.data {
        ImageData::Jpeg { components, .. } => *components == 3,
        ImageData::Raw { format, .. } => matches!(format, PixelFormat::Rgb8 | PixelFormat::Rgba8),
    });
    if rgb && !rgb_allowed {
        return Err(CardError::CoverColourSpace { standard: req.profile.standard });
    }
    Ok(())
}

/// The card composed for the screen, then [`svg::render_page`] for each page:
/// `[outside, inside]`, for live previews. Nothing is converted to the output intent
/// (the screen shows the sRGB design), and the print gates of [`render_pdf`] do not
/// apply.
///
/// # Errors
/// Those of [`compose`].
pub fn preview_svg(req: &CardRequest, fonts: &FontSet) -> Result<Vec<String>, CardError> {
    let (doc, _) = compose::compose(req, fonts, compose::Purpose::Preview)?;
    Ok((0..doc.pages.len()).map(|index| svg::render_page(&doc, index)).collect())
}
