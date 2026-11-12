//! Print-ready PDF writer for [`ebc_scene`] documents.
//!
//! [`write_pdf`] turns a [`Document`] into the bytes of a PDF file that a professional
//! printer can take as it is:
//!
//! * **Page boxes** — `MediaBox` (= `CropBox`), `BleedBox` and `TrimBox` from
//!   [`ebc_scene::Page`]'s trim, bleed and slug (per direction when the page has a
//!   `slug_v`, e.g. a trim centred on an A4 sheet), with the scene's top-left/y-down
//!   millimetres mapped to PDF points.
//! * **Vector content** — paths (fill, even-odd fill, stroke, dashes, caps, joins),
//!   clipping groups and transforms.
//! * **Colour** — CMYK and grey as device colours, the registration colour as a
//!   `/Separation /All` space, RGB as ICC-based sRGB in colour-managed standards
//!   (PDF/X-3, PDF/X-4), as `DeviceRGB` in plain PDF, and as an **error** in PDF/X-1a.
//! * **Images** — JPEG files are embedded untouched (`DCTDecode`), raw pixels are
//!   Flate-compressed, alpha channels become soft masks where the standard allows
//!   transparency, ICC profiles are embedded.
//! * **Text** — fonts are embedded as subsets (Type 0 / `Identity-H`; `CIDFontType2` for
//!   TrueType outlines, `CIDFontType0` with a bare CFF program for CFF outlines) with a
//!   `ToUnicode` map so that the text can be extracted. Fonts whose `OS/2.fsType` forbids
//!   embedding are refused; fonts that forbid subsetting are embedded whole.
//! * **PDF/X** — output intent with the destination ICC profile, Info dictionary, XMP
//!   metadata, `Trapped`, file identifier, and the header version each standard is based
//!   on (1.3 for PDF/X-1a:2001 and PDF/X-3:2002, 1.6 for PDF/X-4, 1.7 for plain PDF).
//!
//! What a standard forbids is reported as a [`PdfError`], never silently converted.
//! The output is **deterministic**: the same document gives the same bytes; the writer
//! never reads the clock or a random source.
//!
//! # Example
//!
//! ```
//! use ebc_pdf::{WriteOptions, write_pdf};
//! use ebc_scene::{Color, Document, Item, Page, PdfStandard, Rect, Size};
//!
//! let mut page = Page::new(Size::new(230.0, 178.0), 3.0, 0.0);
//! page.items.push(Item::filled_rect(Rect::new(-3.0, -3.0, 236.0, 184.0), Color::cmyk(0.0, 0.2, 0.9, 0.0)));
//! let mut doc = Document { standard: PdfStandard::Plain, ..Document::default() };
//! doc.info.title = "A yellow card".into();
//! doc.pages.push(page);
//!
//! let pdf = write_pdf(&doc, &WriteOptions::default())?;
//! assert!(pdf.starts_with(b"%PDF-1.7"));
//! # Ok::<(), ebc_pdf::PdfError>(())
//! ```
//!
//! # Conventions worth knowing
//!
//! * Stroke widths and dash lengths live in the coordinate system of the group they are
//!   in: a scaling group transform scales them too (as in PDF and SVG).
//! * Groups with a singular transform, an empty clip path or no children draw nothing
//!   and are skipped.
//! * **Overprint**: text in pure black (C0 M0 Y0 K100) is set to overprint, so that it
//!   prints on top of a coloured ground instead of knocking it out (no white halo when
//!   the plates are slightly out of register). Nothing else overprints — in particular
//!   never anything white.
//! * When the same glyph stands for different texts in different runs (say a space and a
//!   no-break space), the `ToUnicode` map keeps the first one.
//! * Variable fonts are embedded at their default instance; [`ebc_scene::FontResource`]
//!   carries no variation coordinates.
//! * Resources that no page actually paints (unused fonts, images, ICC profiles) are not
//!   written, and not validated either.
//!
//! # What the writer does not check
//!
//! The writer enforces what each PDF/X part forbids outright. It does **not** judge print
//! quality (image resolution, ink coverage, minimum type size, hairlines) — that is the
//! job of the independent `ebc-preflight` crate — and it embeds the output-intent profile
//! as given: use [`icc::inspect`] to make sure it is a CMYK *output* (`prtr`) profile
//! before sending a file to a printer.

#![forbid(unsafe_code)]

mod content;
mod date;
mod error;
mod font;
pub mod icc;
mod image;
mod jpeg;
mod md5;
mod writer;
mod xmp;

pub use ebc_scene::{Document, PdfStandard};
pub use error::{EmbeddingRestriction, PdfError};

/// Options for [`write_pdf`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteOptions {
    /// Flate-compress content streams, fonts, ICC profiles, CMaps and raw images, and
    /// write dictionaries compactly. Turn it off to get a file that can be read in a
    /// text editor (JPEG data and the XMP packet are never recompressed either way).
    pub compress: bool,
    /// Keep named groups ([`ebc_scene::Item::Group`] with a `layer`) as PDF layers
    /// (optional content groups). Only PDF/X-4 and plain PDF can carry layers; for
    /// PDF/X-1a and PDF/X-3, or when this is `false`, layers are flattened into the page.
    pub keep_layers: bool,
}

impl Default for WriteOptions {
    /// Compressed output without layers — what online printers ask for.
    fn default() -> Self {
        Self { compress: true, keep_layers: false }
    }
}

/// Writes `doc` as a PDF file conforming to [`Document::standard`].
///
/// # Errors
///
/// Returns a [`PdfError`] when the document cannot be written as asked, in particular:
///
/// * any PDF/X level without an output intent, a title or a creation date
///   ([`PdfError::MissingOutputIntent`], [`PdfError::MissingTitle`],
///   [`PdfError::MissingCreationDate`]), PDF/X-4 without an embedded output profile
///   ([`PdfError::MissingOutputProfile`]);
/// * RGB colours or RGB images in PDF/X-1a ([`PdfError::RgbNotAllowed`]), ICC-tagged
///   images in PDF/X-1a ([`PdfError::IccNotAllowed`]);
/// * images with a non-opaque alpha channel in PDF/X-1a or PDF/X-3
///   ([`PdfError::TransparencyNotAllowed`]);
/// * fonts that must not be embedded ([`PdfError::FontEmbeddingForbidden`]), fonts or
///   images that are malformed, references to resources that are not in the document;
/// * geometry that cannot be written: non-finite numbers, paths that draw before their
///   first `MoveTo`, negative stroke widths, groups nested more than 24 levels deep.
pub fn write_pdf(doc: &Document, opts: &WriteOptions) -> Result<Vec<u8>, PdfError> {
    writer::write(doc, opts)
}
