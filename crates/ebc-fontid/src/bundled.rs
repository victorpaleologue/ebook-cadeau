//! The index of the fonts bundled with eBook Cadeau (feature `bundled-index`).

use crate::error::FontIdError;
use crate::index::FontIndex;

/// The serialised index of the eight fonts in `assets/fonts` (all SIL OFL 1.1), as
/// written by `cargo run -p ebc-fontid --example build-index`.
pub const BUNDLED_INDEX_BYTES: &[u8] = include_bytes!("../data/bundled-index.bin");

/// File names (in `assets/fonts`) of the fonts in the bundled index, in index order.
/// Each entry's `source_url` is `assets/fonts/<file name>`.
pub const BUNDLED_FONT_FILES: [&str; 8] = [
    "AtkinsonHyperlegible-Regular.ttf",
    "AtkinsonHyperlegible-Bold.ttf",
    "AtkinsonHyperlegible-Italic.ttf",
    "Spectral-Regular.ttf",
    "Spectral-SemiBold.ttf",
    "Spectral-Italic.ttf",
    "BebasNeue-Regular.ttf",
    "CourierPrime-Bold.ttf",
];

/// The index of the bundled fonts: Atkinson Hyperlegible (Regular, Bold, Italic),
/// Spectral (Regular, SemiBold, Italic), Bebas Neue and Courier Prime Bold.
///
/// # Errors
/// Only if the embedded bytes were damaged or written by another format version
/// (a build problem; a unit test guards against it).
pub fn bundled_index() -> Result<FontIndex, FontIdError> {
    FontIndex::from_bytes(BUNDLED_INDEX_BYTES)
}
