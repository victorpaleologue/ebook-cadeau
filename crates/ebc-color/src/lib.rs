//! Palette extraction, contrast and print colour helpers for cover images.
//!
//! The crate answers the colour questions the card composer and the spine designer ask
//! about a book cover:
//!
//! * *Which colours make up this cover?* — [`palette`], [`edge_palette`].
//! * *Is the binding edge a flat colour I can simply extend onto the spine?* —
//!   [`edge_uniformity`], [`has_uniform_border`].
//! * *Which colour can I set the title in so that it stays readable?* —
//!   [`contrast_ratio`] (WCAG 2.x), [`pick_text_color`], [`ensure_contrast`].
//! * *How different are two colours to the eye?* — [`delta_e2000`] (CIEDE2000).
//! * *How do I hand a colour to the PDF writer?* — [`to_cmyk_naive`], [`to_scene_rgb`].
//!
//! It is pure Rust, depends only on `ebc-scene` (images are [`ebc_scene::Pixmap`]s), performs
//! no I/O and compiles to `wasm32-unknown-unknown`.
//!
//! # Determinism
//!
//! Every function is a pure function of its arguments. The palette extraction uses a fixed
//! seed, a deterministic sampling pattern and stable tie-breaking, and all *discrete*
//! decisions (which pixel is sampled, which cluster a colour joins, which 8-bit value a
//! centroid rounds to) are computed with IEEE-754 basic operations only — the sRGB transfer
//! curve and the CIELAB cube root are evaluated by this crate's own Newton iterations instead
//! of the platform's `libm`. The same image therefore yields the same palette natively and
//! in the browser. Only [`delta_e2000`] calls transcendental functions (`atan2`, `sin`,
//! `cos`, `exp`) whose last bit may differ between platforms.
//!
//! # Colour-management caveat
//!
//! Inputs are assumed to be sRGB. [`to_cmyk_naive`] is **not** a colour-managed conversion:
//! it is the textbook device formula with grey-component replacement, good for text, rules
//! and flat tints where a predictable, K-only black matters more than a colorimetric match.
//! Cover photographs must stay RGB (or be converted with an ICC profile elsewhere).
//!
//! # Example
//!
//! ```
//! use ebc_color::{Rgb8, contrast_ratio, palette, pick_text_color};
//! use ebc_scene::Pixmap;
//!
//! let cover = Pixmap::filled(40, 60, [20, 40, 90, 255]);
//! let colours = palette(&cover, 4);
//! assert_eq!(colours[0].color, Rgb8::new(20, 40, 90));
//! assert_eq!(colours[0].share, 1.0);
//!
//! let text = pick_text_color(colours[0].color, &[Rgb8::new(30, 30, 30)], 4.5);
//! assert_eq!(text, Rgb8::WHITE); // the dark candidate fails 4.5:1, white wins
//! assert!(contrast_ratio(text, colours[0].color) >= 4.5);
//! ```

#![forbid(unsafe_code)]

mod cmyk;
mod contrast;
mod delta_e;
mod lab;
mod palette;
mod rgb;

pub use cmyk::{from_scene_color, limit_total_ink, to_cmyk_gcr, to_cmyk_naive, to_scene_rgb};
pub use contrast::{
    black_or_white, contrast_ratio, ensure_contrast, pick_text_color, relative_luminance,
};
pub use delta_e::{delta_e2000, delta_e2000_lab};
pub use lab::Lab;
pub use palette::{
    BorderOptions, EDGE_BUSY_SIGMA, Edge, MAX_PALETTE_SIZE, PALETTE_MERGE_DELTA_E, Swatch,
    UNIFORM_EDGE_THRESHOLD, edge_palette, edge_uniformity, has_uniform_border, palette,
    uniform_border_with,
};
pub use rgb::Rgb8;
