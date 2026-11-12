//! Error type of the crate.

/// Everything that can go wrong when loading a font or rasterising text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FontError {
    /// The data is not a TrueType/OpenType font (or collection) that can be parsed.
    #[error("not a usable TrueType/OpenType font: {0}")]
    Parse(String),
    /// The requested face does not exist in the font collection.
    #[error("face index {index} is out of range (the file contains {count} face(s))")]
    FaceIndex {
        /// The face index that was asked for.
        index: u32,
        /// Number of faces in the file.
        count: u32,
    },
    /// A raster size that is not a positive, finite number of pixels per em.
    #[error("invalid raster size: {0} px per em")]
    InvalidRasterSize(String),
    /// The rasterised text would not fit in the pixel budget.
    #[error("rasterised text would be {width} x {height} px, above the supported maximum")]
    RasterTooLarge {
        /// Width the mask would have had, in pixels (saturated).
        width: u64,
        /// Height the mask would have had, in pixels (saturated).
        height: u64,
    },
    /// The font passed the initial checks but a table needed later is so malformed that
    /// the font libraries gave up while reading it.
    #[error("the font data is malformed")]
    Malformed,
    /// The font produced outline coordinates that are not finite numbers.
    #[error("the font produced non-finite outline coordinates")]
    NonFiniteOutline,
    /// A glyph was left out of an outline because drawing it would take an unreasonable
    /// amount of work: either the glyph itself expands to more than any real glyph needs
    /// (deeply nested composite glyphs or subroutines — a "glyph bomb"), or the text as
    /// a whole exceeded the outline budget of one call.
    #[error("glyph {glyph} was not drawn: the outline is too complex")]
    OutlineTooComplex {
        /// The first glyph that was left out.
        glyph: u16,
    },
}
