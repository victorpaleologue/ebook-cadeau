//! Fitting a raster image into a rectangle. The three ways of placing a cover are
//! compared in the crate documentation ("Placing the cover").

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
/// For a cover that must reach into the bleed use [`place_image_bleed`] instead of
/// passing a panel grown by [`Panels::with_bleed`](crate::Panels::with_bleed): that
/// rectangle is not centred on its panel.
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

/// Where to draw a **full-bleed** cover image on a cover panel: scaled without distortion
/// until it covers `panel` grown by `bleed` on all four sides, and centred on the panel.
///
/// The caller must clip the image to the panel grown into the bleed on its *outer* edges
/// only — `format.outside().with_bleed(bleed).right` for the front cover, `.left` for the
/// back ([`Panels::with_bleed`](crate::Panels::with_bleed)) — which the returned rectangle
/// always contains. On the crease side the clip cuts off the same strip that the trim
/// removes at the fore-edge, so what remains visible is centred on the panel: a frame or
/// a centred title on the cover stays centred on the finished card.
///
/// Every edge of the cover loses at least `bleed` (slightly more after scaling), so this
/// crops the artwork: use it only when that is allowed. The crate documentation
/// ("Placing the cover") has the uncropped default and the variant that keeps the cover's
/// spine-side edge on the crease.
///
/// A negative or non-finite `bleed` counts as zero. Degenerate input is handled as by
/// [`place_image`].
#[must_use]
pub fn place_image_bleed(image_px: (u32, u32), panel: Rect, bleed: f64) -> Rect {
    let bleed = if bleed.is_finite() { bleed.max(0.0) } else { 0.0 };
    place_image(image_px, panel.inflate(bleed), Fit::Cover)
}

/// The effective resolution, in pixels per inch, of an image of `image_px` pixels drawn
/// into `placed` (the rectangle returned by [`place_image`]): the lower of the horizontal
/// and vertical figures. Returns `0.0` for an empty or non-finite rectangle.
///
/// Compare the result with a profile using
/// [`PrintProfile::judge_ppi`](crate::PrintProfile::judge_ppi), which allows for the
/// last-bit rounding of this division.
#[must_use]
pub fn effective_ppi(image_px: (u32, u32), placed: Rect) -> f64 {
    let usable = |v: f64| v.is_finite() && v > 0.0;
    if !usable(placed.w) || !usable(placed.h) {
        return 0.0;
    }
    // Multiply before dividing: `1800 * 25.4 / 152.4` is closer to 300 than
    // `1800 / (152.4 / 25.4)`.
    let x = f64::from(image_px.0) * MM_PER_INCH / placed.w;
    let y = f64::from(image_px.1) * MM_PER_INCH / placed.h;
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
    fn full_bleed_placement_is_centred_on_the_panel_and_covers_its_bleed() {
        // The poche front panel and its clip rectangle (bleed on the three outer edges).
        let panel = Rect::new(120.0, 0.0, 110.0, 178.0);
        let clip = Rect::new(120.0, -3.0, 113.0, 184.0);
        // A scan in the panel's own proportions.
        let r = place_image_bleed((1100, 1780), panel, 3.0);
        assert!(r.contains_rect(&clip, 1e-9), "{r:?}");
        assert!(close(r.center().x, panel.center().x) && close(r.center().y, panel.center().y));
        // Scaled by 116/110 (the width decides), so the same 3 mm strip is hidden behind
        // the fold as is trimmed off at the fore-edge.
        assert!(close(r.w, 116.0) && close(r.h, 1780.0 * 116.0 / 1100.0));
        assert!(close(panel.x - r.x, 3.0) && close(r.right() - panel.right(), 3.0));
        // The recipe this replaces — centring on the clip rectangle — is 1.5 mm off.
        let naive = place_image((1100, 1780), clip, Fit::Cover);
        assert!(close(naive.center().x - panel.center().x, 1.5));
        // No bleed: plain cover fit on the panel. Hostile bleeds count as zero.
        for b in [0.0, -3.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                place_image_bleed((600, 900), panel, b),
                place_image((600, 900), panel, Fit::Cover),
                "bleed {b}"
            );
        }
        // Degenerate input comes back unchanged, as with `place_image`.
        assert_eq!(place_image_bleed((0, 0), panel, 3.0), panel.inflate(3.0));
    }

    #[test]
    fn crease_aligned_full_bleed_keeps_the_spine_side_edge_on_the_crease() {
        // The documented alternative to `place_image_bleed`, on the poche card.
        let (front, front_clip) =
            (Rect::new(120.0, 0.0, 110.0, 178.0), Rect::new(120.0, -3.0, 113.0, 184.0));
        let r = place_image_aligned((1100, 1780), front_clip, Fit::Cover, 0.0, 0.5);
        assert_eq!(r.x, front.x, "inner edge exactly on the crease");
        assert!(r.contains_rect(&front_clip, 1e-9));
        // The back cover mirrors it: its inner edge is its right edge.
        let (back, back_clip) =
            (Rect::new(0.0, 0.0, 110.0, 178.0), Rect::new(-3.0, -3.0, 113.0, 184.0));
        let r = place_image_aligned((1100, 1780), back_clip, Fit::Cover, 1.0, 0.5);
        assert!(close(r.right(), back.right()), "{r:?}");
        assert!(r.contains_rect(&back_clip, 1e-9));
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
        // Scans made at exactly 300 ppi on inch-based sizes: 6 x 9 in and 3 x 7 in.
        for (px, w, h) in [((1800, 2700), 152.4, 228.6), ((900, 2100), 76.2, 177.8)] {
            let ppi = effective_ppi(px, Rect::new(0.0, 0.0, w, h));
            assert!((ppi - 300.0).abs() < 1e-9, "{px:?}: {ppi:?}");
        }
    }
}
