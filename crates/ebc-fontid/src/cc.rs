//! Local-contrast binarisation and connected components.

use crate::gray::{Gray, Integral};

/// Ink masks of both polarities from local contrast.
///
/// A pixel is *dark ink* when it is darker than the mean of its `window` x `window`
/// neighbourhood by more than `max(c_min, k * sd)`, where `sd` is the standard deviation
/// in that neighbourhood; *light ink* is the mirror image. This is Niblack's rule with an
/// absolute contrast floor, which keeps flat, noisy areas empty in both polarities and
/// treats dark and light backgrounds alike (Sauvola's rule does neither).
///
/// Returns `(dark, light)`, one byte per pixel, 1 = ink.
pub(crate) fn local_contrast_masks(
    g: &Gray,
    window: usize,
    k: f32,
    c_min: f32,
) -> (Vec<u8>, Vec<u8>) {
    let integral = Integral::new(g, None);
    let r = (window / 2).max(1);
    let mut dark = vec![0u8; g.w * g.h];
    let mut light = vec![0u8; g.w * g.h];
    for y in 0..g.h {
        for x in 0..g.w {
            let (_, mean, sd) = integral.window(x, y, r);
            let margin = c_min.max(k * sd);
            let v = f32::from(g.px[y * g.w + x]);
            if mean - v > margin {
                dark[y * g.w + x] = 1;
            } else if v - mean > margin {
                light[y * g.w + x] = 1;
            }
        }
    }
    (dark, light)
}

/// One 8-connected component of a binary mask.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Comp {
    /// Left edge (inclusive).
    pub(crate) x0: u32,
    /// Top edge (inclusive).
    pub(crate) y0: u32,
    /// Right edge (exclusive).
    pub(crate) x1: u32,
    /// Bottom edge (exclusive).
    pub(crate) y1: u32,
    /// Number of pixels.
    pub(crate) area: u32,
    /// Number of pixels with at least one 4-neighbour outside the component.
    pub(crate) perimeter: u32,
}

impl Comp {
    #[inline]
    pub(crate) fn w(&self) -> u32 {
        self.x1 - self.x0
    }

    #[inline]
    pub(crate) fn h(&self) -> u32 {
        self.y1 - self.y0
    }

    /// Mean stroke width estimated as `2 * area / perimeter` (exact for long strokes).
    #[inline]
    pub(crate) fn stroke_width(&self) -> f32 {
        2.0 * self.area as f32 / self.perimeter.max(1) as f32
    }

    #[inline]
    pub(crate) fn cx(&self) -> f32 {
        (self.x0 + self.x1) as f32 * 0.5
    }

    #[inline]
    pub(crate) fn cy(&self) -> f32 {
        (self.y0 + self.y1) as f32 * 0.5
    }
}

/// Labels the 8-connected components of `mask` (non-zero = ink).
///
/// Returns the label image (0 = background, `i + 1` = component `i`) and the components
/// in raster order of their first pixel.
pub(crate) fn label_components(mask: &[u8], w: usize, h: usize) -> (Vec<u32>, Vec<Comp>) {
    let mut labels = vec![0u32; w * h];
    let mut comps: Vec<Comp> = Vec::new();
    if w == 0 || h == 0 || mask.len() < w * h {
        return (labels, comps);
    }
    let mut stack: Vec<u32> = Vec::new();
    for start in 0..w * h {
        if mask[start] == 0 || labels[start] != 0 {
            continue;
        }
        let label = comps.len() as u32 + 1;
        let (sx, sy) = ((start % w) as u32, (start / w) as u32);
        let mut comp = Comp { x0: sx, y0: sy, x1: sx + 1, y1: sy + 1, area: 0, perimeter: 0 };
        labels[start] = label;
        stack.push(start as u32);
        while let Some(i) = stack.pop() {
            let i = i as usize;
            let (x, y) = (i % w, i / w);
            comp.area += 1;
            comp.x0 = comp.x0.min(x as u32);
            comp.y0 = comp.y0.min(y as u32);
            comp.x1 = comp.x1.max(x as u32 + 1);
            comp.y1 = comp.y1.max(y as u32 + 1);
            let on_edge = x == 0
                || y == 0
                || x + 1 == w
                || y + 1 == h
                || mask[i - 1] == 0
                || mask[i + 1] == 0
                || mask[i - w] == 0
                || mask[i + w] == 0;
            if on_edge {
                comp.perimeter += 1;
            }
            let (xa, xb) = (x.saturating_sub(1), (x + 1).min(w - 1));
            let (ya, yb) = (y.saturating_sub(1), (y + 1).min(h - 1));
            for ny in ya..=yb {
                for nx in xa..=xb {
                    let j = ny * w + nx;
                    if mask[j] != 0 && labels[j] == 0 {
                        labels[j] = label;
                        stack.push(j as u32);
                    }
                }
            }
        }
        comps.push(comp);
    }
    (labels, comps)
}

/// Median of a slice of floats (not NaN-aware: callers pass finite values). 0 if empty.
pub(crate) fn median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    let n = values.len();
    if n % 2 == 1 { values[n / 2] } else { 0.5 * (values[n / 2 - 1] + values[n / 2]) }
}

/// Theil–Sen slope of `y` over `x`: the median of all pairwise slopes. 0 for fewer than
/// two usable points.
pub(crate) fn theil_sen_slope(points: &[(f32, f32)]) -> f32 {
    let mut slopes = Vec::new();
    for (i, a) in points.iter().enumerate() {
        for b in &points[i + 1..] {
            let dx = b.0 - a.0;
            if dx.abs() > 1e-3 {
                slopes.push((b.1 - a.1) / dx);
            }
        }
    }
    median(&mut slopes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_are_eight_connected_with_exact_boxes() {
        #[rustfmt::skip]
        let mask = [
            1, 0, 0, 0, 0,
            0, 1, 0, 1, 1,
            0, 0, 0, 1, 1,
            0, 0, 0, 0, 0,
        ];
        let (labels, comps) = label_components(&mask, 5, 4);
        assert_eq!(comps.len(), 2);
        assert_eq!(labels[0], 1);
        assert_eq!(labels[6], 1, "diagonal neighbours join");
        assert_eq!((comps[0].x0, comps[0].y0, comps[0].x1, comps[0].y1), (0, 0, 2, 2));
        assert_eq!(comps[0].area, 2);
        assert_eq!((comps[1].x0, comps[1].y0, comps[1].x1, comps[1].y1), (3, 1, 5, 3));
        assert_eq!((comps[1].area, comps[1].perimeter), (4, 4));
        assert_eq!(labels[4 + 5], 2);
    }

    #[test]
    fn labelling_handles_empty_and_inconsistent_input() {
        assert!(label_components(&[], 0, 0).1.is_empty());
        assert!(label_components(&[1, 1], 4, 4).1.is_empty(), "short buffer is ignored");
        let (_, comps) = label_components(&[1; 12], 4, 3);
        assert_eq!(comps.len(), 1);
        assert_eq!((comps[0].area, comps[0].perimeter), (12, 10));
    }

    #[test]
    fn stroke_width_of_a_bar() {
        // A 40 x 4 bar: 2 * 160 / (2 * 40 + 2 * 2) = 3.8.
        let mut mask = vec![0u8; 50 * 10];
        for y in 3..7 {
            for x in 5..45 {
                mask[y * 50 + x] = 1;
            }
        }
        let (_, comps) = label_components(&mask, 50, 10);
        assert_eq!(comps.len(), 1);
        assert!((comps[0].stroke_width() - 3.8).abs() < 0.11, "{}", comps[0].stroke_width());
    }

    #[test]
    fn local_contrast_finds_both_polarities_and_ignores_flat_areas() {
        let mut g = Gray::filled(60, 40, 128);
        for y in 15..25 {
            for x in 10..14 {
                g.px[y * 60 + x] = 20;
            }
            for x in 40..44 {
                g.px[y * 60 + x] = 240;
            }
        }
        let (dark, light) = local_contrast_masks(&g, 21, 0.3, 12.0);
        assert_eq!(dark[20 * 60 + 12], 1);
        assert_eq!(light[20 * 60 + 12], 0);
        assert_eq!(light[20 * 60 + 42], 1);
        assert_eq!(dark[20 * 60 + 42], 0);
        // Far from both bars nothing is ink.
        assert_eq!(dark[2 * 60 + 28] + light[2 * 60 + 28], 0);
        let flat = Gray::filled(30, 30, 77);
        let (d, l) = local_contrast_masks(&flat, 15, 0.3, 12.0);
        assert!(d.iter().chain(&l).all(|&v| v == 0));
    }

    #[test]
    fn robust_statistics() {
        assert_eq!(median(&mut []), 0.0);
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&mut [4.0, 1.0, 2.0, 3.0]), 2.5);
        // A line y = 0.1 x + 5 with one wild outlier.
        let mut pts: Vec<(f32, f32)> = (0..10).map(|i| (i as f32 * 10.0, i as f32 + 5.0)).collect();
        pts[4].1 = 80.0;
        assert!((theil_sen_slope(&pts) - 0.1).abs() < 1e-4);
        assert_eq!(theil_sen_slope(&[(1.0, 2.0)]), 0.0);
        assert_eq!(theil_sen_slope(&[(1.0, 2.0), (1.0, 9.0)]), 0.0);
    }
}
