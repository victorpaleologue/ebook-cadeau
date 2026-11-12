//! Error type of the crate.

/// Everything that can go wrong when reading a serialised [`FontIndex`](crate::FontIndex).
///
/// The image functions ([`find_text_regions`](crate::find_text_regions),
/// [`identify`](crate::identify), [`rerank`](crate::rerank)) do not fail: an image or
/// region they cannot use yields an empty result.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FontIdError {
    /// The data does not start with the font-index magic number.
    #[error("not a font index (bad magic number)")]
    BadMagic,
    /// The index was written by a format version this build cannot read.
    #[error("unsupported font index version {found} (this build reads version {supported})")]
    UnsupportedVersion {
        /// Version found in the data.
        found: u16,
        /// Version this build reads and writes.
        supported: u16,
    },
    /// The index uses a descriptor layout this build does not implement.
    #[error("unsupported descriptor layout in font index")]
    UnsupportedLayout,
    /// The data ends before a field that the header promises.
    #[error("font index is truncated: {needed} byte(s) needed at offset {offset}")]
    Truncated {
        /// Offset of the field that could not be read.
        offset: usize,
        /// Number of bytes that field needs.
        needed: usize,
    },
    /// The trailing checksum does not match the content.
    #[error("font index checksum mismatch")]
    Checksum,
    /// A field holds a value that cannot be right.
    #[error("font index is corrupt: {0}")]
    Corrupt(&'static str),
}
