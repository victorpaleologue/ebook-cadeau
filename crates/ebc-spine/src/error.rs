//! Error type of the crate.

/// Everything that can make [`design`](crate::design) fail.
///
/// Conditions that still allow a usable spine (a truncated title, a missing glyph, an
/// incompatible provided spine, a spine too narrow for text) are **not** errors: they are
/// reported in [`SpineDesign::notes`](crate::SpineDesign::notes).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SpineError {
    /// A title had to be typeset but [`BookMeta::title`](ebc_model::BookMeta) is empty or
    /// blank. The crate never invents text.
    #[error("the book has no title to set on the spine")]
    EmptyTitle,
    /// The spine rectangle, the bleed or the safe margin is not usable (not finite,
    /// negative, empty, or absurdly large).
    #[error("invalid spine geometry: {0}")]
    InvalidGeometry(String),
    /// A value of [`SpineOptions`](crate::SpineOptions) is not usable.
    #[error("invalid spine options: {0}")]
    InvalidOptions(String),
    /// The pixel range of [`SpineSource::FullWrap`](crate::SpineSource::FullWrap) is empty
    /// or lies outside the image.
    #[error(
        "spine pixel range {start}..{end} is empty or outside the {width} px wide wraparound image"
    )]
    InvalidSpineRange {
        /// First column of the requested range.
        start: u32,
        /// One past the last column of the requested range.
        end: u32,
        /// Width of the wraparound image in pixels.
        width: u32,
    },
}
