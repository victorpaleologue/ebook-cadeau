//! Fitting a raster image into a rectangle.

use ebc_scene::{MM_PER_INCH, Rect};

/// How an image is scaled into a target rectangle. The aspect ratio is always kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Fit {
    /// Scale until the image **covers** the whole target; what sticks out is cropped by
    /// the caller's clip. No empty borders. The usual choice for a cover on its panel.
    #[default]
    Cover,
    /// Scale until the whole image is **contained** in the target; nothing is cropped,
    /// borders may remain.
    Contain,
}

/// Where to draw an image of `image_px` pixels so that it fills (`Fit::Cover`) or fits in
/// (`Fit::Contain`) `target`, centred, without distortion.
///
/// The returned rectangle is the destination of the *whole* image (use it as the `rect`
/// of an [`ebc_scene::Item::Image`]). With `Fit::Cover` it is at least as large as
/// `target` and the caller must clip to `target` (e.g. with
/// [`ebc_scene::Item::clipped`]); with `Fit::Contain` it lies inside `target`.
///
/// For a full-bleed cover pass the panel **with** its bleed
/// ([`Panels::with_bleed`](crate::Panels::with_bleed)) as `target`.
///
/// Degenerate input — a zero pixel dimension, or a target that is empty or not finite —
/// returns `target` unchanged.
#[must_use]
pub fn place_image(image_px: (u32, u32), target: Rect, fit: Fit) -> Rect {
    place_image_aligned(image_px, target, fit, 0.5, 0.5)
}

/// Like [`place_image`], but chooses which part of the image is kept (`Fit::Cover`) or
/// where the image sits in the target (`Fit::Contain`): `align_x` / `align_y` run from
/// `0.0` (left / top edges coincide) through `0.5` (centred) to `1.0` (right / bottom
/// edges coincide). Values outside `0.0..=1.0` are clamped; NaN means centred.
///
/// Example: a front cover that is wider than its panel loses width; `align_x = 0.0` keeps
/// its left (spine-side) edge and crops the fore-edge.
#[must_use]
pub fn place_image_aligned(
    image_px: (u32, u32),
    target: Rect,
    fit: Fit,
    align_x: f64,
    align_y: f64,
) -> Rect {
    let (iw, ih) = (f64::from(image_px.0), f64::from(image_px.1));
    let usable = |v: f64| v.is_finite() && v > 0.0;
    if image_px.0 == 0
        || image_px.1 == 0
        || !usable(target.w)
        || !usable(target.h)
        || !target.x.is_finite()
        || !target.y.is_finite()
    {
        return target;
    }
    let (sx, sy) = (target.w / iw, target.h / ih);
    // The dimension that decides the scale is kept bit-exact and the other one is clamped
    // to the right side of the target, so that a covering image never falls short of the
    // target (and a contained one never sticks out) by a rounding error.
    let (w, h) = match fit {
        Fit::Cover if sx >= sy => (target.w, (ih * sx).max(target.h)),
        Fit::Cover => ((iw * sy).max(target.w), target.h),
        Fit::Contain if sx <= sy => (target.w, (ih * sx).min(target.h)),
        Fit::Contain => ((iw * sy).min(target.w), target.h),
    };
    let align = |a: f64| if a.is_nan() { 0.5 } else { a.clamp(0.0, 1.0) };
    Rect::new(
        target.x + (target.w - w) * align(align_x),
        target.y + (target.h - h) * align(align_y),
        w,
        h,
    )
}

/// The effective resolution, in pixels per inch, of an image of `image_px` pixels drawn
/// into `placed` (the rectangle returned by [`place_image`]): the lower of the horizontal
/// and vertical figures. Returns `0.0` for an empty or non-finite rectangle.
///
/// Compare the result with a profile using
/// [`PrintProfile::judge_ppi`](crate::PrintProfile::judge_ppi).
#[must_use]
pub fn effective_ppi(image_px: (u32, u32), placed: Rect) -> f64 {
    let usable = |v: f64| v.is_finite() && v > 0.0;
    if !usable(placed.w) || !usable(placed.h) {
        return 0.0;
    }
    let x = f64::from(image_px.0) / (placed.w / MM_PER_INCH);
    let y = f64::from(image_px.1) / (placed.h / MM_PER_INCH);
    x.min(y)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn cover_fills_and_crops_the_long_side() {
        let target = Rect::new(120.0, -3.0, 113.0, 184.0);
        // A squarer image than the panel: full height, cropped left and right.
        let r = place_image((1000, 1000), target, Fit::Cover);
        assert_eq!((r.y, r.h), (target.y, target.h));
        assert!(close(r.w, 184.0));
        assert!(close(r.center().x, target.center().x));
        assert!(r.contains_rect(&target, 1e-9));
        // A taller image than the panel: full width, cropped top and bottom.
        let r = place_image((500, 2000), target, Fit::Cover);
        assert_eq!((r.x, r.w), (target.x, target.w));
        assert!(close(r.h, 452.0));
        assert!(close(r.center().y, target.center().y));
    }

    #[test]
    fn contain_fits_and_leaves_borders() {
        let target = Rect::new(10.0, 20.0, 100.0, 50.0);
        let r = place_image((400, 100), target, Fit::Contain);
        assert_eq!(r, Rect::new(10.0, 32.5, 100.0, 25.0));
        let r = place_image((100, 400), target, Fit::Contain);
        assert_eq!(r, Rect::new(53.75, 20.0, 12.5, 50.0));
        assert!(target.contains_rect(&r, 0.0));
    }

    #[test]
    fn same_aspect_gives_the_target_for_both_fits() {
        let target = Rect::new(-3.0, -3.0, 113.0, 184.0);
        for fit in [Fit::Cover, Fit::Contain] {
            let r = place_image((1130, 1840), target, fit);
            assert!(close(r.x, target.x) && close(r.y, target.y));
            assert!(close(r.w, target.w) && close(r.h, target.h));
        }
    }

    #[test]
    fn alignment_picks_the_kept_part() {
        let target = Rect::new(0.0, 0.0, 100.0, 100.0);
        // 2:1 image covering a square: 200 wide, 100 of it is cropped.
        let left = place_image_aligned((200, 100), target, Fit::Cover, 0.0, 0.5);
        assert_eq!(left, Rect::new(0.0, 0.0, 200.0, 100.0));
        let right = place_image_aligned((200, 100), target, Fit::Cover, 1.0, 0.5);
        assert_eq!(right, Rect::new(-100.0, 0.0, 200.0, 100.0));
        // Contain: the image sits at the top or the bottom.
        let top = place_image_aligned((200, 100), target, Fit::Contain, 0.5, 0.0);
        assert_eq!(top, Rect::new(0.0, 0.0, 100.0, 50.0));
        let bottom = place_image_aligned((200, 100), target, Fit::Contain, 0.5, 1.0);
        assert_eq!(bottom, Rect::new(0.0, 50.0, 100.0, 50.0));
        // Out-of-range and NaN alignments are clamped / centred.
        assert_eq!(place_image_aligned((200, 100), target, Fit::Cover, -7.0, 0.5), left);
        assert_eq!(place_image_aligned((200, 100), target, Fit::Cover, 7.0, 0.5), right);
        assert_eq!(
            place_image_aligned((200, 100), target, Fit::Cover, f64::NAN, f64::NAN),
            place_image((200, 100), target, Fit::Cover)
        );
    }

    #[test]
    fn degenerate_input_returns_the_target() {
        let target = Rect::new(1.0, 2.0, 3.0, 4.0);
        for fit in [Fit::Cover, Fit::Contain] {
            assert_eq!(place_image((0, 100), target, fit), target);
            assert_eq!(place_image((100, 0), target, fit), target);
            assert_eq!(place_image((0, 0), target, fit), target);
            for bad in [
                Rect::new(0.0, 0.0, 0.0, 10.0),
                Rect::new(0.0, 0.0, 10.0, 0.0),
                Rect::new(0.0, 0.0, -5.0, 10.0),
                Rect::new(0.0, 0.0, f64::INFINITY, 10.0),
                Rect::new(f64::NEG_INFINITY, 0.0, 10.0, 10.0),
            ] {
                assert_eq!(place_image((100, 100), bad, fit), bad);
            }
            // NaN is not equal to itself: check field by field.
            let nan = Rect::new(0.0, 0.0, f64::NAN, 10.0);
            let r = place_image((100, 100), nan, fit);
            assert!(r.w.is_nan() && r.h == 10.0);
        }
    }

    #[test]
    fn extreme_pixel_sizes_stay_finite() {
        let target = Rect::new(0.0, 0.0, 110.0, 178.0);
        for px in [(u32::MAX, 1), (1, u32::MAX), (u32::MAX, u32::MAX), (1, 1)] {
            for fit in [Fit::Cover, Fit::Contain] {
                let r = place_image(px, target, fit);
                assert!(r.x.is_finite() && r.y.is_finite() && r.w.is_finite() && r.h.is_finite());
                assert!(r.w > 0.0 && r.h > 0.0, "{px:?} {fit:?}: {r:?}");
            }
        }
    }

    #[test]
    fn effective_resolution() {
        // 1299 x 2102 px on 110 x 178 mm is 300 ppi.
        let ppi = effective_ppi((1299, 2102), Rect::new(0.0, 0.0, 110.0, 178.0));
        assert!((ppi - 300.0).abs() < 0.2, "{ppi}");
        // The lower of the two axes wins.
        let ppi = effective_ppi((254, 1000), Rect::new(0.0, 0.0, 25.4, 25.4));
        assert!(close(ppi, 254.0));
        // A typical 600 px wide thumbnail stretched over a poche cover with bleed.
        let placed = place_image((600, 900), Rect::new(0.0, 0.0, 113.0, 184.0), Fit::Cover);
        let ppi = effective_ppi((600, 900), placed);
        assert!(ppi > 120.0 && ppi < 130.0, "{ppi}");
        assert_eq!(effective_ppi((600, 900), Rect::new(0.0, 0.0, 0.0, 10.0)), 0.0);
        assert_eq!(effective_ppi((600, 900), Rect::new(0.0, 0.0, f64::NAN, 10.0)), 0.0);
        assert_eq!(effective_ppi((0, 0), Rect::new(0.0, 0.0, 10.0, 10.0)), 0.0);
    }
}
