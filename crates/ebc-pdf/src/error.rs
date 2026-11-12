//! The error type of the PDF writer.

use ebc_scene::{FontId, ImageId, PdfStandard};

/// Why a font's licence flags (`OS/2.fsType`) forbid embedding it in a PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EmbeddingRestriction {
    /// "Restricted License embedding": the font must not be embedded at all.
    Restricted,
    /// "Bitmap embedding only": outlines must not be embedded, and this writer only
    /// embeds outlines.
    BitmapOnly,
}

impl core::fmt::Display for EmbeddingRestriction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Restricted => "restricted-licence embedding (fsType bit 1)",
            Self::BitmapOnly => "bitmap-only embedding (fsType bit 9)",
        })
    }
}

/// Human-readable name of a conformance level, used in error messages.
pub(crate) fn standard_name(standard: PdfStandard) -> &'static str {
    match standard {
        PdfStandard::Plain => "plain PDF",
        PdfStandard::X1a2001 => "PDF/X-1a:2001",
        PdfStandard::X3_2002 => "PDF/X-3:2002",
        PdfStandard::X4 => "PDF/X-4",
    }
}

/// Everything that can go wrong while writing a PDF.
///
/// The writer never panics on malformed documents: geometry that cannot be expressed in a
/// PDF, resources that cannot be embedded and content that the requested
/// [`PdfStandard`] forbids are all reported here instead of being silently altered.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PdfError {
    /// The document has no pages.
    #[error("the document has no pages")]
    NoPages,

    /// A page has an unusable size, bleed or slug.
    #[error("page {page}: invalid page geometry: {reason}")]
    InvalidPage {
        /// 1-based page number.
        page: usize,
        /// What is wrong.
        reason: &'static str,
    },

    /// A PDF/X level was requested without [`ebc_scene::Document::output_intent`].
    #[error("{} requires an output intent", standard_name(*standard))]
    MissingOutputIntent {
        /// The requested conformance level.
        standard: PdfStandard,
    },

    /// The output intent carries no ICC profile although the standard needs one
    /// (always for PDF/X-4; for PDF/X-1a and PDF/X-3 when no registry name is given).
    #[error("{} requires the output intent's ICC profile to be embedded", standard_name(*standard))]
    MissingOutputProfile {
        /// The requested conformance level.
        standard: PdfStandard,
    },

    /// The output intent cannot be used with the requested standard.
    #[error("unsupported output intent: {reason}")]
    UnsupportedOutputIntent {
        /// What is wrong.
        reason: &'static str,
    },

    /// An ICC profile (output intent or image) is malformed or does not match its use.
    #[error("invalid ICC profile for {context}: {reason}")]
    InvalidIccProfile {
        /// Which profile, e.g. `"the output intent"` or `"image 3"`.
        context: String,
        /// What is wrong.
        reason: &'static str,
    },

    /// A PDF/X level was requested but [`ebc_scene::DocInfo::title`] is empty.
    #[error("{} requires a document title", standard_name(*standard))]
    MissingTitle {
        /// The requested conformance level.
        standard: PdfStandard,
    },

    /// A PDF/X level was requested but [`ebc_scene::DocInfo::created_unix`] is `None`
    /// (PDF/X requires `CreationDate` and `ModDate`; the writer never reads the clock).
    #[error("{} requires a creation date (DocInfo::created_unix)", standard_name(*standard))]
    MissingCreationDate {
        /// The requested conformance level.
        standard: PdfStandard,
    },

    /// `created_unix` is outside the years 0001–9999.
    #[error("creation time {0} s is outside the representable range (years 0001-9999)")]
    InvalidDate(i64),

    /// A key of [`ebc_scene::DocInfo::custom`] is empty, not printable ASCII, a duplicate, or
    /// collides with a key the writer sets itself.
    #[error("invalid custom Info key {key:?}: {reason}")]
    InvalidCustomKey {
        /// The offending key.
        key: String,
        /// What is wrong.
        reason: &'static str,
    },

    /// RGB content (a paint colour or an image) in a standard that only allows CMYK,
    /// grey and spot colours (PDF/X-1a).
    #[error("{} does not allow RGB content ({location})", standard_name(*standard))]
    RgbNotAllowed {
        /// The requested conformance level.
        standard: PdfStandard,
        /// Where the RGB content was found, e.g. `"page 1"` or `"image 3"`.
        location: String,
    },

    /// An image with a non-opaque alpha channel in a standard without transparency
    /// (PDF/X-1a, PDF/X-3).
    #[error("{} does not allow transparency (image {} has a non-opaque alpha channel)", standard_name(*standard), image.0)]
    TransparencyNotAllowed {
        /// The requested conformance level.
        standard: PdfStandard,
        /// The offending image.
        image: ImageId,
    },

    /// An ICC-tagged image in a standard without colour management (PDF/X-1a).
    #[error("{} does not allow ICC-tagged images (image {})", standard_name(*standard), image.0)]
    IccNotAllowed {
        /// The requested conformance level.
        standard: PdfStandard,
        /// The offending image.
        image: ImageId,
    },

    /// A page draws an image that is not registered in the document.
    #[error("unknown image id {}", .0.0)]
    UnknownImage(ImageId),

    /// A page sets glyphs in a font that is not registered in the document.
    #[error("unknown font id {}", .0.0)]
    UnknownFont(FontId),

    /// An image resource is malformed (bad dimensions, truncated pixels, unsupported or
    /// inconsistent JPEG).
    #[error("invalid image {}: {reason}", image.0)]
    InvalidImage {
        /// The offending image.
        image: ImageId,
        /// What is wrong.
        reason: String,
    },

    /// A font resource cannot be parsed or subset.
    #[error("invalid font {}: {reason}", font.0)]
    InvalidFont {
        /// The offending font.
        font: FontId,
        /// What is wrong.
        reason: String,
    },

    /// A font's `OS/2.fsType` forbids embedding it.
    #[error("font {} ({name}) must not be embedded: {restriction}", font.0)]
    FontEmbeddingForbidden {
        /// The offending font.
        font: FontId,
        /// The font's PostScript name, for the error message.
        name: String,
        /// Which licence flag forbids the embedding.
        restriction: EmbeddingRestriction,
    },

    /// A glyph run references a glyph that the font does not have.
    #[error("font {} has {glyph_count} glyphs, glyph id {glyph} does not exist", font.0)]
    InvalidGlyph {
        /// The font.
        font: FontId,
        /// The offending glyph id.
        glyph: u16,
        /// Number of glyphs in the font.
        glyph_count: u16,
    },

    /// Page content that cannot be written: non-finite or out-of-range numbers, a path
    /// that draws before its first `MoveTo`, a negative stroke width, a bad dash pattern
    /// or a non-positive font size.
    #[error("page {page}: invalid content: {reason}")]
    InvalidContent {
        /// 1-based page number.
        page: usize,
        /// What is wrong.
        reason: &'static str,
    },

    /// Groups are nested deeper than PDF consumers are required to support.
    #[error("page {page}: groups are nested deeper than {max} levels")]
    NestingTooDeep {
        /// 1-based page number.
        page: usize,
        /// The maximum supported nesting depth.
        max: usize,
    },

    /// A stream or the object count exceeds what a PDF file can hold.
    #[error("{what} is too large for a PDF file")]
    TooLarge {
        /// What is too large.
        what: &'static str,
    },
}
