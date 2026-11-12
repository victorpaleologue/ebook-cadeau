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
//! crop (it is letterboxed on a flat neutral ground, at least 3 mm inside the trim).
//! `trim_authorised` allows full-bleed placement; `adaptation_authorised` — with a
//! `print_licence` or `rights_holder_declared` basis — allows the spine styles that
//! sample or re-fit artwork. A credit line for the cover is printed on the back panel.
//! These are conservative defaults from
//! `docs/research/07-legal-covers-and-reseller-rights.md`, pending a lawyer's review;
//! the layout decisions are documented on the two page modules' source.
//!
//! # Colour
//!
//! Text, QR code and rules are black on the black plate only (grey in a home PDF).
//! Flat grounds are device CMYK in PDF/X files, through a conversion that is **not
//! colour-managed**. The cover is embedded as sRGB and left to the printer's colour
//! management ([`CardWarning::CoverNotConverted`]); PDF/X-1a, which forbids RGB,
//! therefore cannot carry a cover ([`CardError::CoverColourSpace`]).
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
//! No colour-managed conversion of covers or grounds; no right-to-left or vertical
//! setting (one run, one direction, as in `ebc-text`); no font fallback for body text
//! (characters the font lacks are left out, with a warning); the single-sided home
//! variant with a flap and per-printer crease encodings of the print research are not
//! built.

#![forbid(unsafe_code)]

mod compose;
mod cover;
mod error;
mod inside;
pub mod inspect;
mod outside;
mod paint;
mod request;
pub mod rights;
mod strings;
pub mod svg;
mod text;

use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::Document;

pub use compose::{MARK_CLEARANCE_MM, NOTES_LAYER};
pub use cover::{
    MAX_COVER_BYTES, MAX_COVER_PIXELS, MAX_COVER_SIDE_PX, MAX_EMBED_PPI, RESAMPLE_PPI,
};
pub use error::{CardError, CardWarning, TextField};
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
    compose::compose(req, fonts)
}

/// [`compose`], then [`ebc_pdf::write_pdf`]: the print file and the warnings.
///
/// Layers are flattened, as printers ask — except for a proof with the guides overlay
/// ([`CardRequest::guides`]), which keeps them so that a viewer can switch them off.
/// The bytes are deterministic.
///
/// # Errors
/// Those of [`compose`], and [`CardError::Pdf`] when the writer refuses the document.
pub fn render_pdf(
    req: &CardRequest,
    fonts: &FontSet,
) -> Result<(Vec<u8>, Vec<CardWarning>), CardError> {
    let (doc, warnings) = compose(req, fonts)?;
    let options = WriteOptions { compress: true, keep_layers: req.guides };
    Ok((write_pdf(&doc, &options)?, warnings))
}

/// [`compose`], then [`svg::render_page`] for each page: `[outside, inside]`, for live
/// previews.
///
/// # Errors
/// Those of [`compose`].
pub fn preview_svg(req: &CardRequest, fonts: &FontSet) -> Result<Vec<String>, CardError> {
    let (doc, _) = compose(req, fonts)?;
    Ok((0..doc.pages.len()).map(|index| svg::render_page(&doc, index)).collect())
}
