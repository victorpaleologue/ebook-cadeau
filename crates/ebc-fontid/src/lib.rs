//! Image-based font identification against an index of openly licensed fonts.
//!
//! Given a cover image and (ideally) the known title, this crate finds the title on the
//! cover and ranks the fonts of a [`FontIndex`] by how closely they resemble the
//! lettering. It uses classical image processing only — no language model, no learned
//! weights — so it is small, deterministic and runs in the browser (`wasm32`).
//!
//! # Pipeline
//!
//! 1. [`find_text_regions`] / [`find_title_regions`]: local-contrast binarisation in both
//!    polarities, connected components, letter filtering, grouping into lines and
//!    blocks. With the known title, blocks whose letter count fits the title win.
//! 2. Inside a region the glyphs are segmented (accents and dots merged with their base)
//!    and, if their count fits the known text, labelled with their characters; the case
//!    actually used on the cover (as given, UPPER, lower, Title) is inferred.
//! 3. [`identify`]: every labelled glyph is reduced to a small descriptor (gradient
//!    orientation histogram + aspect, density, stroke width, stroke contrast, slant) and
//!    compared with the descriptor of the same character in every indexed font.
//! 4. [`rerank`]: the known text is rendered with each candidate font, fitted to the
//!    region (size, letter-spacing) and compared pixel-wise.
//!
//! # What the result means
//!
//! The answer is always *the nearest font of the index*, never "the font of the cover":
//! cover lettering is mostly commercial or hand-drawn, and the index must contain openly
//! licensed fonts only (each entry records its licence — see [`IndexEntryMeta`]). Use
//! [`Identification::confident`] to decide between the nearest font and a house face.
//! Measured accuracy and failure modes are in `docs/research/fontid-eval.md`.
//!
//! ```
//! use ebc_fontid::{FontIndex, TextRegion, identify};
//! use ebc_scene::Pixmap;
//!
//! // An empty index and a blank image: nothing to find, nothing panics.
//! let cover = Pixmap::filled(60, 90, [255, 255, 255, 255]);
//! let region = TextRegion { x: 5, y: 10, w: 50, h: 20, score: 1.0 };
//! assert!(identify(&cover, &region, Some("Title"), &FontIndex::default()).is_empty());
//! ```

#![forbid(unsafe_code)]

mod cc;
mod descriptor;
mod detect;
mod error;
mod gray;
mod identify;
mod index;
mod label;
mod rerank;
mod segment;

#[cfg(feature = "bundled-index")]
mod bundled;
#[cfg(feature = "synth")]
pub mod eval;
#[cfg(feature = "synth")]
pub mod synth;

#[cfg(feature = "bundled-index")]
pub use bundled::{BUNDLED_FONT_FILES, BUNDLED_INDEX_BYTES, bundled_index};
pub use detect::{TextRegion, find_text_regions, find_title_regions};
pub use error::FontIdError;
pub use identify::{CaseUsed, Identification, Match, identify, identify_detailed};
pub use index::{FontIndex, IndexEntry, IndexEntryMeta, OPEN_LICENCES, REFERENCE_CHARS};
pub use rerank::{rerank, rerank_detailed};
