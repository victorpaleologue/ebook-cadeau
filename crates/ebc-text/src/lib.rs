//! Font loading, shaping, measuring, fitting and outlining of text for print.
//!
//! This crate turns *(font, string, size)* into things the print pipeline can place on a
//! page, all in the units of [`ebc_scene`]: **millimetres, y down**, font sizes in points.
//!
//! | Need | Use |
//! |---|---|
//! | load a font, read its names and licence | [`Font::from_bytes`], [`Font::family_name`], [`Font::licence_description`] |
//! | may this font go into a PDF? | [`Font::embedding`], [`Font::embedding_flags`] (`OS/2.fsType`); rules: [`Font::allows_bundling`], [`Font::allows_subset_embedding`], [`Font::allows_outlines`] |
//! | line metrics, cap height | [`Font::metrics`] → [`FontMetrics`] and its `*_mm(size_pt)` helpers |
//! | pick an instance of a variable font | [`Font::with_variations`] |
//! | can the font draw this text? | [`Font::missing_chars`] |
//! | text → positioned glyphs | [`shape`] → [`ShapedText`] |
//! | real, selectable PDF text | [`ShapedText::to_glyph_item`] → [`ebc_scene::Item::Glyphs`] |
//! | text as vector outlines (spines, any font) | [`ShapedText::to_outline`] → [`ebc_scene::Path`] |
//! | largest size that fits a box | [`fit_size`] |
//! | paragraphs | [`wrap`], [`wrap_ranges`] |
//! | pixels (font identification, previews) | [`rasterize`], [`try_rasterize`] (feature `raster`, on by default) |
//!
//! Shaping is done by `harfrust` (the Rust port of HarfBuzz) on top of `read-fonts`, the
//! font parser of the fontations project: ligatures, kerning, mark positioning and
//! localised forms are applied. Everything is computed in
//! font units and scaled, without hinting, so results are deterministic and exactly
//! proportional to the font size.
//!
//! ```
//! # fn demo(font_bytes: Vec<u8>) -> Result<(), ebc_text::FontError> {
//! use ebc_scene::{Color, Item, Point};
//! use ebc_text::{Font, ShapeOptions, fit_size, shape};
//!
//! let font = Font::from_bytes(font_bytes, 0)?;
//! let opts = ShapeOptions::default();
//! // The largest size between 6 and 40 pt at which the title is at most 150 mm long
//! // with capitals at most 7 mm tall.
//! let size = fit_size(&font, "L'Étranger", 150.0, 7.0, 6.0, 40.0, &opts).unwrap_or(6.0);
//! let run = shape(&font, "L'Étranger", size, &opts);
//! let item = Item::fill(run.to_outline(&font, Point::new(20.0, 100.0)), Color::BLACK);
//! # let _ = item; Ok(()) }
//! ```
//!
//! # Untrusted fonts
//!
//! Font files are treated as hostile input. [`Font::from_bytes`] rejects what cannot be
//! parsed; anything that turns out to be malformed later (a glyph, a kerning table)
//! contributes nothing instead of failing. The underlying parsers do not panic on
//! malformed fonts in release builds; in builds with debug assertions they may, and this
//! crate then contains the panic and returns an error or an empty result — except where
//! panics cannot be caught at all (`wasm32-unknown-unknown`, `panic = "abort"`).
//!
//! The **amount of work** a font can ask for is bounded as well. Two outline formats let
//! a file of a kilobyte describe billions of operations — composite glyphs made of
//! composite glyphs (`glyf`) and subroutines calling subroutines (`CFF `, `CFF2`). The
//! font parser bounds the work of one glyph, but a long text multiplies that bound.
//! This crate therefore
//!
//! * measures the expanded size of a TrueType glyph before drawing it (65 536 points
//!   and component references at most);
//! * meters the charstring interpreter: the charstring of a PostScript glyph and every
//!   subroutine it calls are charged to a budget of 262 144 bytes per glyph;
//! * gives every [`ShapedText::to_outline`], [`rasterize`] and font load an overall
//!   budget (about one million points: some 20 000 letters per call), so a huge text
//!   cannot turn a merely expensive glyph into minutes of work or gigabytes of path;
//! * reads only the tables it needs — bitmap, colour (`COLR`, `SVG `, …) and variable
//!   composite (`VARC`) tables are never looked at — and shapes with a view of the font
//!   from which the tables that would make the *shaper* run glyph programs are removed
//!   (`CFF `, `CFF2`; also `glyf` for fonts that contain an oversized glyph — see
//!   [`Font::has_oversized_glyphs`]).
//!
//! A glyph beyond these limits draws nothing, like a malformed one;
//! [`ShapedText::try_to_outline`] and [`try_rasterize`] report it as
//! [`FontError::OutlineTooComplex`]. The cost of shaping, wrapping and fitting is
//! proportional to the length of the text; this crate sets no limit on that length.
//!
//! # Limits (by design)
//!
//! * One run has one font, one script and one direction: there is no font fallback, no
//!   script itemisation and no bidirectional reordering. Horizontal layout only.
//! * No hyphenation dictionary: [`wrap`] breaks at Unicode line-break opportunities and,
//!   as a last resort, inside words.
//! * Bitmap-only and colour glyphs (emoji) have no outline and draw nothing.
//! * WOFF/WOFF2 are not decoded.

#![forbid(unsafe_code)]

mod cff;
mod error;
mod fit;
mod font;
mod guard;
mod outline;
#[cfg(feature = "raster")]
mod raster;
mod sfnt;
mod shape;
mod wrap;

pub use error::FontError;
pub use fit::fit_size;
pub use font::{
    EmbeddingFlags, EmbeddingPermission, Font, FontMetrics, OutlineFormat, VariationAxis,
};
#[cfg(feature = "raster")]
pub use raster::{MAX_MASK_PIXELS, MAX_MASK_SIDE, Mask, RasterizedText, rasterize, try_rasterize};
pub use shape::{MAX_SIZE_PT, ShapeOptions, ShapedGlyph, ShapedText, outline_bounds, shape};
pub use wrap::{wrap, wrap_ranges};
