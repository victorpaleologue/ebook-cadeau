//! Glyph descriptors: a gradient-orientation histogram over a grid plus scalar shape
//! features, quantised to bytes.
//!
//! The same code describes a glyph cut out of a cover and a glyph rendered from a font
//! file, so that systematic biases cancel when the two are compared character by
//! character.

use crate::gray::{Gray, gaussian_blur_f32, resize_f32};

/// Side of the square box a glyph is normalised into, in pixels.
pub(crate) const BOX: usize = 40;
/// Largest dimension of the glyph inside the box (the rest is padding).
pub(crate) const CONTENT: usize = 36;
/// Histogram cells per side.
pub(crate) const CELLS: usize = 4;
/// Signed orientation bins per cell.
pub(crate) const BINS: usize = 8;
/// Bytes of the orientation histogram.
pub(crate) const HOG_LEN: usize = CELLS * CELLS * BINS;
/// Number of scalar features.
pub(crate) const SCALARS: usize = 5;
/// Bytes of a glyph's features: histogram + scalars.
pub(crate) const FEATURE_LEN: usize = HOG_LEN + SCALARS;

/// Index of the aspect feature, `w / (w + h)`.
pub(crate) const S_ASPECT: usize = 0;
/// Index of the ink-density feature (mean coverage of the bounding box).
pub(crate) const S_DENSITY: usize = 1;
/// Index of the stroke-width feature (relative to the glyph's larger dimension).
pub(crate) const S_STROKE: usize = 2;
/// Index of the thin/thick stroke-contrast feature.
pub(crate) const S_CONTRAST: usize = 3;
/// Index of the slant feature.
pub(crate) const S_SLANT: usize = 4;

/// Largest slant the slant feature can express, in radians (35 degrees).
const SLANT_RANGE: f32 = 0.610_865;

/// The features of one glyph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Features(pub(crate) [u8; FEATURE_LEN]);

impl Features {
    #[inline]
    pub(crate) fn hog(&self) -> &[u8] {
        &self.0[..HOG_LEN]
    }

    #[inline]
    pub(crate) fn scalars(&self) -> &[u8] {
        &self.0[HOG_LEN..]
    }
}

/// Maps raw coverage to the coverage used everywhere in this crate: a small dead zone at
/// both ends, so that faint halos (anti-aliasing fringes, residual background noise)
/// vanish and cores saturate. Applied to cover crops and to font renders alike.
#[inline]
pub(crate) fn remap_coverage(v: u8) -> u8 {
    let v = i32::from(v);
    (((v - 20) * 255) / 215).clamp(0, 255) as u8
}

/// Euclidean distance between two histograms, scaled to `0..=~1.4` (unit vectors are
/// stored multiplied by 512).
#[inline]
pub(crate) fn hog_distance(a: &[u8], b: &[u8]) -> f32 {
    let mut acc = 0u32;
    for (x, y) in a.iter().zip(b) {
        let d = u32::from(x.abs_diff(*y));
        acc += d * d;
    }
    (acc as f32).sqrt() / 512.0
}

/// Slant in degrees encoded by a slant feature byte (positive = leaning right).
#[inline]
pub(crate) fn slant_degrees(byte: u8) -> f32 {
    ((f32::from(byte) / 255.0) * 2.0 - 1.0) * SLANT_RANGE.to_degrees()
}

/// Describes a glyph given as a coverage crop (255 = ink) that is tight around the ink.
/// Returns `None` for an empty crop.
pub(crate) fn describe(crop: &Gray) -> Option<Features> {
    if crop.w == 0 || crop.h == 0 || crop.px.len() < crop.w * crop.h {
        return None;
    }
    let total: u64 = crop.px.iter().map(|&v| u64::from(v)).sum();
    if total == 0 {
        return None;
    }
    let mut out = [0u8; FEATURE_LEN];

    // --- normalise into the box, keeping the aspect ratio --------------------------
    let scale = CONTENT as f32 / crop.w.max(crop.h) as f32;
    let nw = ((crop.w as f32 * scale).round() as usize).clamp(1, CONTENT);
    let nh = ((crop.h as f32 * scale).round() as usize).clamp(1, CONTENT);
    let src: Vec<f32> = crop.px.iter().map(|&v| f32::from(v) / 255.0).collect();
    let small = resize_f32(&src, crop.w, crop.h, nw, nh);
    let (ox, oy) = ((BOX - nw) / 2, (BOX - nh) / 2);
    let mut boxed = vec![0.0f32; BOX * BOX];
    for y in 0..nh {
        boxed[(oy + y) * BOX + ox..(oy + y) * BOX + ox + nw]
            .copy_from_slice(&small[y * nw..(y + 1) * nw]);
    }

    // --- stroke width from area and total variation (no thresholding) ---------------
    let area: f32 = boxed.iter().sum();
    let (gx0, gy0) = gradients(&boxed);
    let tv: f32 = gx0.iter().zip(&gy0).map(|(a, b)| a.hypot(*b)).sum();
    let stroke = if tv > 1e-3 { 2.0 * area / tv } else { CONTENT as f32 * 0.5 };
    let stroke_rel = stroke / CONTENT as f32;

    // --- orientation histogram on a lightly smoothed copy -----------------------------
    let smooth = gaussian_blur_f32(&boxed, BOX, BOX, 1.0);
    let (gx, gy) = gradients(&smooth);
    let mut hist = [0.0f32; HOG_LEN];
    let (mut slant_sum, mut slant_weight) = (0.0f32, 0.0f32);
    let cell_scale = CELLS as f32 / BOX as f32;
    for y in 0..BOX {
        for x in 0..BOX {
            let (dx, dy) = (gx[y * BOX + x], gy[y * BOX + x]);
            let mag = dx.hypot(dy);
            if mag < 1e-4 {
                continue;
            }
            // Near-vertical edges vote for the slant (the gradient is normal to them).
            if dx.abs() > 1e-6 && (dy / dx).abs() < 0.700_2 {
                slant_sum += mag * (dy / dx).atan();
                slant_weight += mag;
            }
            let mut pos = dy.atan2(dx) / std::f32::consts::TAU * BINS as f32;
            if pos < 0.0 {
                pos += BINS as f32;
            }
            let b0 = (pos.floor() as usize) % BINS;
            let tb = pos - pos.floor();
            let b1 = (b0 + 1) % BINS;
            let fx = (x as f32 + 0.5) * cell_scale - 0.5;
            let fy = (y as f32 + 0.5) * cell_scale - 0.5;
            let (cx0, cy0) = (fx.floor(), fy.floor());
            let (tx, ty) = (fx - cx0, fy - cy0);
            for (cy, wy) in [(cy0 as isize, 1.0 - ty), (cy0 as isize + 1, ty)] {
                if cy < 0 || cy >= CELLS as isize {
                    continue;
                }
                for (cx, wx) in [(cx0 as isize, 1.0 - tx), (cx0 as isize + 1, tx)] {
                    if cx < 0 || cx >= CELLS as isize {
                        continue;
                    }
                    let base = (cy as usize * CELLS + cx as usize) * BINS;
                    let wgt = mag * wx * wy;
                    hist[base + b0] += wgt * (1.0 - tb);
                    hist[base + b1] += wgt * tb;
                }
            }
        }
    }
    // SIFT-style normalisation: unit length, clip, unit length again.
    let norm = hist.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 1e-6 {
        hist.iter_mut().for_each(|v| *v = (*v / norm).min(0.2));
        let norm2 = hist.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
        for (o, v) in out[..HOG_LEN].iter_mut().zip(&hist) {
            *o = ((v / norm2) * 512.0 + 0.5).clamp(0.0, 255.0) as u8;
        }
    }

    // --- scalars --------------------------------------------------------------------
    let scalars = &mut out[HOG_LEN..];
    let aspect = crop.w as f32 / (crop.w + crop.h) as f32;
    scalars[S_ASPECT] = unit_to_byte(aspect);
    scalars[S_DENSITY] = unit_to_byte(total as f32 / (255.0 * (crop.w * crop.h) as f32));
    scalars[S_STROKE] = unit_to_byte(stroke_rel * 2.0);
    scalars[S_CONTRAST] = unit_to_byte(stroke_contrast(crop));
    let slant = if slant_weight > 1e-6 { slant_sum / slant_weight } else { 0.0 };
    scalars[S_SLANT] = unit_to_byte((slant / SLANT_RANGE + 1.0) * 0.5);
    Some(Features(out))
}

#[inline]
fn unit_to_byte(v: f32) -> u8 {
    (v * 255.0 + 0.5).clamp(0.0, 255.0) as u8
}

/// Central-difference gradients of a `BOX` x `BOX` image, zero outside.
fn gradients(img: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut gx = vec![0.0f32; BOX * BOX];
    let mut gy = vec![0.0f32; BOX * BOX];
    for y in 0..BOX {
        for x in 0..BOX {
            let i = y * BOX + x;
            let left = if x > 0 { img[i - 1] } else { 0.0 };
            let right = if x + 1 < BOX { img[i + 1] } else { 0.0 };
            let up = if y > 0 { img[i - BOX] } else { 0.0 };
            let down = if y + 1 < BOX { img[i + BOX] } else { 0.0 };
            gx[i] = 0.5 * (right - left);
            gy[i] = 0.5 * (down - up);
        }
    }
    (gx, gy)
}

/// Ratio of thin to thick strokes in `0..=1` (1 = monoline), from run lengths of the
/// binarised glyph at its own resolution: every ink pixel gets the shorter of its
/// horizontal and vertical run as local thickness; the result is the ratio of the 20th
/// to the 85th percentile of that thickness.
fn stroke_contrast(crop: &Gray) -> f32 {
    let (w, h) = (crop.w, crop.h);
    let ink = |x: usize, y: usize| crop.px[y * w + x] >= 128;
    let mut hrun = vec![0u16; w * h];
    for y in 0..h {
        let mut x = 0;
        while x < w {
            if !ink(x, y) {
                x += 1;
                continue;
            }
            let start = x;
            while x < w && ink(x, y) {
                x += 1;
            }
            let len = (x - start).min(usize::from(u16::MAX)) as u16;
            hrun[y * w + start..y * w + x].iter_mut().for_each(|v| *v = len);
        }
    }
    let limit = w.max(h).min(usize::from(u16::MAX));
    let mut hist = vec![0u32; limit + 1];
    let mut count = 0u32;
    for x in 0..w {
        let mut y = 0;
        while y < h {
            if !ink(x, y) {
                y += 1;
                continue;
            }
            let start = y;
            while y < h && ink(x, y) {
                y += 1;
            }
            let vlen = y - start;
            for yy in start..y {
                let t = vlen.min(usize::from(hrun[yy * w + x])).min(limit);
                hist[t] += 1;
                count += 1;
            }
        }
    }
    if count == 0 {
        return 1.0;
    }
    let percentile = |p: f32| -> f32 {
        let target = (p * count as f32).ceil().max(1.0) as u32;
        let mut acc = 0u32;
        for (t, &c) in hist.iter().enumerate() {
            acc += c;
            if acc >= target {
                return t as f32;
            }
        }
        limit as f32
    };
    let (thin, thick) = (percentile(0.20), percentile(0.85));
    if thick <= 0.0 { 1.0 } else { (thin / thick).clamp(0.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A crop with a filled parallelogram: a bar of `bar_w` pixels sheared by `shear`
    /// pixels over the height.
    fn bar(w: usize, h: usize, bar_w: usize, shear: f32) -> Gray {
        let mut g = Gray::filled(w, h, 0);
        for y in 0..h {
            let off = shear * (h - 1 - y) as f32 / (h - 1) as f32;
            for x in 0..w {
                let xf = x as f32 - off;
                if xf >= 0.0 && xf < bar_w as f32 {
                    g.px[y * w + x] = 255;
                }
            }
        }
        g
    }

    fn ring(size: usize, thickness: f32) -> Gray {
        let mut g = Gray::filled(size, size, 0);
        let c = (size as f32 - 1.0) * 0.5;
        let outer = size as f32 * 0.5;
        for y in 0..size {
            for x in 0..size {
                let r = (x as f32 - c).hypot(y as f32 - c);
                if r <= outer && r >= outer - thickness {
                    g.px[y * size + x] = 255;
                }
            }
        }
        g
    }

    #[test]
    fn empty_crops_have_no_descriptor() {
        assert!(describe(&Gray { w: 0, h: 0, px: vec![] }).is_none());
        assert!(describe(&Gray::filled(8, 8, 0)).is_none());
        assert!(describe(&Gray { w: 8, h: 8, px: vec![255; 3] }).is_none());
    }

    #[test]
    fn descriptor_is_scale_invariant() {
        let a = describe(&ring(30, 5.0)).expect("ink");
        let b = describe(&ring(90, 15.0)).expect("ink");
        assert!(hog_distance(a.hog(), b.hog()) < 0.12, "{}", hog_distance(a.hog(), b.hog()));
        for k in 0..SCALARS {
            let d = a.scalars()[k].abs_diff(b.scalars()[k]);
            assert!(d <= 16, "scalar {k}: {} vs {}", a.scalars()[k], b.scalars()[k]);
        }
    }

    #[test]
    fn different_shapes_are_far_apart() {
        let o = describe(&ring(40, 6.0)).expect("ink");
        let i = describe(&bar(8, 40, 8, 0.0)).expect("ink");
        assert!(hog_distance(o.hog(), i.hog()) > 0.5);
        assert!(i.scalars()[S_ASPECT] < o.scalars()[S_ASPECT]);
        assert!(i.scalars()[S_DENSITY] > o.scalars()[S_DENSITY]);
    }

    #[test]
    fn stroke_width_feature_tracks_the_weight() {
        let light = describe(&ring(60, 5.0)).expect("ink");
        let bold = describe(&ring(60, 12.0)).expect("ink");
        assert!(bold.scalars()[S_STROKE] > light.scalars()[S_STROKE] + 20);
        // 5 px of a 60 px glyph: relative width 1/12, stored doubled.
        let expected = 255.0 * 2.0 * 5.0 / 60.0;
        let got = f32::from(light.scalars()[S_STROKE]);
        assert!((got - expected).abs() < 12.0, "got {got}, expected about {expected}");
    }

    #[test]
    fn slant_feature_has_the_right_sign_and_size() {
        let upright = describe(&bar(10, 60, 10, 0.0)).expect("ink");
        assert!(slant_degrees(upright.scalars()[S_SLANT]).abs() < 1.0);
        // Sheared by 12 px over 59 rows: atan(12 / 59) = 11.5 degrees to the right.
        let italic = describe(&bar(22, 60, 10, 12.0)).expect("ink");
        let deg = slant_degrees(italic.scalars()[S_SLANT]);
        assert!((deg - 11.5).abs() < 3.0, "slant {deg}");
        let mut mirrored = bar(22, 60, 10, 12.0);
        for row in mirrored.px.chunks_exact_mut(22) {
            row.reverse();
        }
        let deg = slant_degrees(describe(&mirrored).expect("ink").scalars()[S_SLANT]);
        assert!((deg + 11.5).abs() < 3.0, "slant {deg}");
    }

    #[test]
    fn contrast_feature_separates_monoline_from_modulated_strokes() {
        // An "L": vertical stem and horizontal foot of equal or unequal thickness.
        let l_shape = |stem: usize, foot: usize| {
            let mut g = Gray::filled(40, 60, 0);
            for y in 0..60 {
                for x in 0..40 {
                    if x < stem || y >= 60 - foot {
                        g.px[y * 40 + x] = 255;
                    }
                }
            }
            g
        };
        let mono = describe(&l_shape(8, 8)).expect("ink");
        let modulated = describe(&l_shape(12, 3)).expect("ink");
        assert!(mono.scalars()[S_CONTRAST] > 230, "{}", mono.scalars()[S_CONTRAST]);
        assert!(modulated.scalars()[S_CONTRAST] < 110, "{}", modulated.scalars()[S_CONTRAST]);
    }

    #[test]
    fn coverage_remap_has_dead_zones() {
        assert_eq!(remap_coverage(0), 0);
        assert_eq!(remap_coverage(20), 0);
        assert_eq!(remap_coverage(235), 255);
        assert_eq!(remap_coverage(255), 255);
        assert!(remap_coverage(128) > 120 && remap_coverage(128) < 136);
    }
}
