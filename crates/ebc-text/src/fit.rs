//! Fitting a line of text into a box by choosing the font size.

use crate::font::Font;
use crate::guard::contain;
use crate::shape::{MAX_SIZE_PT, ShapeOptions, shape_units};

/// Finds the largest font size in `min_pt..=max_pt` at which `text`, set on one line,
/// is at most `max_width` millimetres wide **and** its capital letters are at most
/// `max_cap_height` millimetres tall.
///
/// Returns `None` when the text does not fit even at `min_pt`, or when the arguments
/// make no sense (`min_pt` not positive, `min_pt > max_pt`, a negative or NaN limit, a
/// non-finite size). Sizes above [`MAX_SIZE_PT`] are not considered. Pass `f64::INFINITY` for a limit that should not apply.
///
/// The result is guaranteed to fit: `shape(font, text, size, opts).width <= max_width`
/// and `font.metrics().cap_height_mm(size) <= max_cap_height` hold exactly for the
/// returned `size`, and no size more than a rounding error larger does.
///
/// The typical use is the spine of the card: `max_width` is the usable length of the
/// spine and `max_cap_height` its usable thickness.
///
/// ```
/// # fn demo(font: &ebc_text::Font) {
/// use ebc_text::{ShapeOptions, fit_size};
/// // A title along a 178 mm tall, 10 mm thick spine with 2 mm clear on each side.
/// let size = fit_size(font, "L'Étranger", 150.0, 6.0, 6.0, 28.0, &ShapeOptions::default());
/// # let _ = size; }
/// ```
#[must_use]
pub fn fit_size(
    font: &Font,
    text: &str,
    max_width: f64,
    max_cap_height: f64,
    min_pt: f64,
    max_pt: f64,
    opts: &ShapeOptions,
) -> Option<f64> {
    let sizes_ok = min_pt.is_finite()
        && max_pt.is_finite()
        && min_pt > 0.0
        && min_pt <= max_pt
        && min_pt <= MAX_SIZE_PT;
    let limits_ok = max_width >= 0.0 && max_cap_height >= 0.0; // false for NaN
    if !sizes_ok || !limits_ok {
        return None;
    }

    let max_pt = max_pt.min(MAX_SIZE_PT);

    // Shape once: glyph positions are proportional to the size.
    let run = contain(|| Some(shape_units(&font.hb_face()?, text, opts))).flatten()?;
    let metrics = font.metrics();
    let fits = |size: f64| {
        run.width_mm(size) <= max_width && metrics.cap_height_mm(size) <= max_cap_height
    };

    if !fits(min_pt) {
        return None;
    }
    if fits(max_pt) {
        return Some(max_pt);
    }
    // Invariant: fits(lo) && !fits(hi). Bisect until the two are adjacent floats.
    let (mut lo, mut hi) = (min_pt, max_pt);
    for _ in 0..128 {
        let mid = lo + (hi - lo) / 2.0;
        if mid <= lo || mid >= hi {
            break;
        }
        if fits(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(lo)
}
