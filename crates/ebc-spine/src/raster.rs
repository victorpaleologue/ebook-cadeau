//! A minimal RGB raster and the few pixel operations the spine needs: reading a
//! [`Pixmap`] composited over white, box-filter downscaling, separable box blurs and
//! bilinear upscaling.
//!
//! Everything is computed with integer arithmetic or IEEE-754 basic operations (and
//! `sqrt`), never with `exp`/`pow`, so the generated pixels are identical on every
//! platform, including `wasm32-unknown-unknown`.

use ebc_color::Rgb8;
use ebc_scene::Pixmap;

/// An opaque 8-bit RGB image, row-major, 3 bytes per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Raster {
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) data: Vec<u8>,
}

impl Raster {
    /// A raster of `w × h` pixels, all black. Both dimensions are at least 1.
    pub(crate) fn new(w: usize, h: usize) -> Self {
        let (w, h) = (w.max(1), h.max(1));
        Self { w, h, data: vec![0; w * h * 3] }
    }

    /// The pixel at `(x, y)`; coordinates are clamped to the raster.
    pub(crate) fn px(&self, x: usize, y: usize) -> [u8; 3] {
        let x = x.min(self.w - 1);
        let y = y.min(self.h - 1);
        let i = (y * self.w + x) * 3;
        match self.data.get(i..i + 3) {
            Some(p) => [p[0], p[1], p[2]],
            None => [255, 255, 255],
        }
    }

    /// Sets the pixel at `(x, y)`; out-of-range coordinates are ignored.
    pub(crate) fn set(&mut self, x: usize, y: usize, rgb: [u8; 3]) {
        if x < self.w && y < self.h {
            let i = (y * self.w + x) * 3;
            if let Some(p) = self.data.get_mut(i..i + 3) {
                p.copy_from_slice(&rgb);
            }
        }
    }

    /// The mean colour of all pixels.
    pub(crate) fn mean(&self) -> Rgb8 {
        self.rows_mean(0, self.h)
    }

    /// The mean colour of the rows `y0..y1` (clamped; at least one row).
    pub(crate) fn rows_mean(&self, y0: usize, y1: usize) -> Rgb8 {
        let y0 = y0.min(self.h - 1);
        let y1 = y1.clamp(y0 + 1, self.h);
        let mut sum = [0u64; 3];
        let rows = self.data.get(y0 * self.w * 3..y1 * self.w * 3).unwrap_or(&[]);
        for p in rows.chunks_exact(3) {
            sum[0] += u64::from(p[0]);
            sum[1] += u64::from(p[1]);
            sum[2] += u64::from(p[2]);
        }
        let n = ((y1 - y0) * self.w).max(1) as u64;
        let avg = |s: u64| ((s + n / 2) / n).min(255) as u8;
        Rgb8::new(avg(sum[0]), avg(sum[1]), avg(sum[2]))
    }

    /// A copy with the first row repeated `top` times above and the last row repeated
    /// `bottom` times below (edge replication, used to build the bleed).
    pub(crate) fn extend_rows(&self, top: usize, bottom: usize) -> Self {
        let stride = self.w * 3;
        let first = self.data.get(..stride).unwrap_or(&[]);
        let last = self.data.get(self.data.len().saturating_sub(stride)..).unwrap_or(&[]);
        let mut data = Vec::with_capacity((self.h + top + bottom) * stride);
        for _ in 0..top {
            data.extend_from_slice(first);
        }
        data.extend_from_slice(&self.data);
        for _ in 0..bottom {
            data.extend_from_slice(last);
        }
        Self { w: self.w, h: self.h + top + bottom, data }
    }
}

/// Composites a non-premultiplied RGBA pixel over white — what a transparent pixel
/// becomes on paper (the same convention as `ebc-color`).
pub(crate) fn over_white([r, g, b, a]: [u8; 4]) -> [u8; 3] {
    if a == 255 {
        return [r, g, b];
    }
    let alpha = u32::from(a);
    let blend = |c: u8| ((u32::from(c) * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
    [blend(r), blend(g), blend(b)]
}

/// The pixel of `img` at `(x, y)` composited over white; coordinates are clamped.
pub(crate) fn pixel(img: &Pixmap, x: u32, y: u32) -> [u8; 3] {
    let x = x.min(img.width() - 1);
    let y = y.min(img.height() - 1);
    let i = (y as usize * img.width() as usize + x as usize) * 4;
    match img.data().get(i..i + 4) {
        Some(p) => over_white([p[0], p[1], p[2], p[3]]),
        None => [255, 255, 255],
    }
}

/// Sum of absolute channel differences between two colours (0–765).
pub(crate) fn rgb_distance(a: [u8; 3], b: [u8; 3]) -> u32 {
    (0..3).map(|c| u32::from(a[c].abs_diff(b[c]))).sum()
}

/// Rec. 601 luma of an 8-bit colour, 0–255 (integer arithmetic).
pub(crate) fn luma(p: [u8; 3]) -> u32 {
    (299 * u32::from(p[0]) + 587 * u32::from(p[1]) + 114 * u32::from(p[2]) + 500) / 1000
}

/// Box-filter downscale of the columns `x0..x1` of `img` (all rows) to at most
/// `max_w × max_h` pixels, keeping the aspect ratio. Never upscales. Every source pixel
/// contributes to exactly one destination pixel.
pub(crate) fn downscale(img: &Pixmap, x0: u32, x1: u32, max_w: usize, max_h: usize) -> Raster {
    let x1 = x1.min(img.width());
    let x0 = x0.min(x1.saturating_sub(1));
    let sw = (x1 - x0).max(1) as usize;
    let sh = img.height() as usize;
    // Uniform scale factor as a rational `num / den` (≤ 1) chosen so that both limits hold.
    let (dw, dh) = if sw <= max_w.max(1) && sh <= max_h.max(1) {
        (sw, sh)
    } else {
        let fw = max_w.max(1) as f64 / sw as f64;
        let fh = max_h.max(1) as f64 / sh as f64;
        let f = fw.min(fh).min(1.0);
        (((sw as f64 * f).round() as usize).clamp(1, sw), ((sh as f64 * f).round() as usize).clamp(1, sh))
    };
    let mut out = Raster::new(dw, dh);
    let mut sums = vec![0u32; dw * 3];
    let mut counts = vec![0u32; dw];
    let mut row_start = 0usize;
    for j in 0..dh {
        let row_end =
            (((j as u64 + 1) * sh as u64 / dh as u64) as usize).max(row_start + 1).min(sh);
        sums.iter_mut().for_each(|s| *s = 0);
        counts.iter_mut().for_each(|c| *c = 0);
        for y in row_start..row_end {
            for x in 0..sw {
                let i = ((x as u64 * dw as u64 / sw as u64) as usize).min(dw - 1);
                let p = pixel(img, x0 + x as u32, y as u32);
                sums[i * 3] += u32::from(p[0]);
                sums[i * 3 + 1] += u32::from(p[1]);
                sums[i * 3 + 2] += u32::from(p[2]);
                counts[i] += 1;
            }
        }
        for i in 0..dw {
            let n = counts[i].max(1);
            let avg = |s: u32| ((s + n / 2) / n).min(255) as u8;
            out.set(i, j, [avg(sums[i * 3]), avg(sums[i * 3 + 1]), avg(sums[i * 3 + 2])]);
        }
        row_start = row_end;
    }
    out
}

/// A floating-point RGB grid used while generating the continuation image.
#[derive(Debug, Clone)]
pub(crate) struct Grid {
    pub(crate) w: usize,
    pub(crate) h: usize,
    /// `w * h * 3` channel values in 0.0..=255.0.
    pub(crate) data: Vec<f32>,
}

impl Grid {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        let (w, h) = (w.max(1), h.max(1));
        Self { w, h, data: vec![0.0; w * h * 3] }
    }

    /// The raster as a grid, flipped left to right: column `x` of the grid is column
    /// `w - 1 - x` of the raster.
    pub(crate) fn mirrored(raster: &Raster) -> Self {
        let mut grid = Self::new(raster.w, raster.h);
        for y in 0..raster.h {
            for x in 0..raster.w {
                let p = raster.px(raster.w - 1 - x, y);
                let i = (y * raster.w + x) * 3;
                if let Some(cell) = grid.data.get_mut(i..i + 3) {
                    cell[0] = f32::from(p[0]);
                    cell[1] = f32::from(p[1]);
                    cell[2] = f32::from(p[2]);
                }
            }
        }
        grid
    }

    /// The cells with the lowest and the highest luma, as 8-bit colours.
    pub(crate) fn extremes(&self) -> ([u8; 3], [u8; 3]) {
        let to8 = |p: &[f32]| -> [u8; 3] {
            let c = |v: f32| (v.clamp(0.0, 255.0) + 0.5) as u8;
            [c(p[0]), c(p[1]), c(p[2])]
        };
        let mut darkest = [255u8; 3];
        let mut lightest = [0u8; 3];
        let (mut lo, mut hi) = (u32::MAX, 0u32);
        for p in self.data.chunks_exact(3) {
            let rgb = to8(p);
            let l = luma(rgb);
            if l < lo {
                lo = l;
                darkest = rgb;
            }
            if l >= hi {
                hi = l;
                lightest = rgb;
            }
        }
        (darkest, lightest)
    }

    fn at(&self, x: usize, y: usize, c: usize) -> f32 {
        self.data.get((y * self.w + x) * 3 + c).copied().unwrap_or(0.0)
    }

    /// Mean of every channel.
    pub(crate) fn mean(&self) -> [f32; 3] {
        let mut sum = [0.0f64; 3];
        for p in self.data.chunks_exact(3) {
            sum[0] += f64::from(p[0]);
            sum[1] += f64::from(p[1]);
            sum[2] += f64::from(p[2]);
        }
        let n = (self.w * self.h).max(1) as f64;
        [(sum[0] / n) as f32, (sum[1] / n) as f32, (sum[2] / n) as f32]
    }

    /// Approximates a Gaussian blur of standard deviation `sigma_x` / `sigma_y` (in grid
    /// cells) with three box blurs per axis; edges are replicated.
    pub(crate) fn blur(&mut self, sigma_x: f64, sigma_y: f64) {
        for radius in box_radii(sigma_x) {
            self.box_blur_rows(radius);
        }
        for radius in box_radii(sigma_y) {
            self.box_blur_columns(radius);
        }
    }

    fn box_blur_rows(&mut self, radius: usize) {
        if radius == 0 {
            return;
        }
        let (w, h) = (self.w, self.h);
        let mut line = vec![0.0f32; w];
        for y in 0..h {
            for c in 0..3 {
                for (x, slot) in line.iter_mut().enumerate() {
                    *slot = self.at(x, y, c);
                }
                let blurred = box_blur_line(&line, radius);
                for (x, v) in blurred.into_iter().enumerate() {
                    self.data[(y * w + x) * 3 + c] = v;
                }
            }
        }
    }

    fn box_blur_columns(&mut self, radius: usize) {
        if radius == 0 {
            return;
        }
        let (w, h) = (self.w, self.h);
        let mut line = vec![0.0f32; h];
        for x in 0..w {
            for c in 0..3 {
                for (y, slot) in line.iter_mut().enumerate() {
                    *slot = self.at(x, y, c);
                }
                let blurred = box_blur_line(&line, radius);
                for (y, v) in blurred.into_iter().enumerate() {
                    self.data[(y * w + x) * 3 + c] = v;
                }
            }
        }
    }

    /// Moves every value `amount` (0–1) of the way towards `target`.
    pub(crate) fn pull_towards(&mut self, target: [f32; 3], amount: f32) {
        let amount = amount.clamp(0.0, 1.0);
        for p in self.data.chunks_exact_mut(3) {
            for c in 0..3 {
                p[c] += (target[c] - p[c]) * amount;
            }
        }
    }

    /// Bilinear resampling to `out_w × out_h` 8-bit pixels (pixel centres aligned, edges
    /// replicated).
    pub(crate) fn upscale(&self, out_w: usize, out_h: usize) -> Raster {
        let mut out = Raster::new(out_w, out_h);
        let (out_w, out_h) = (out.w, out.h);
        // Source coordinate of destination centre `d`: (d + 0.5) * src / dst - 0.5.
        let axis = |dst: usize, src: usize| -> Vec<(usize, usize, f32)> {
            (0..dst)
                .map(|d| {
                    let pos = (d as f64 + 0.5) * src as f64 / dst as f64 - 0.5;
                    let pos = pos.clamp(0.0, (src - 1) as f64);
                    let i0 = pos.floor() as usize;
                    let i1 = (i0 + 1).min(src - 1);
                    (i0, i1, (pos - i0 as f64) as f32)
                })
                .collect()
        };
        let xs = axis(out_w, self.w);
        let ys = axis(out_h, self.h);
        for (j, &(y0, y1, ty)) in ys.iter().enumerate() {
            for (i, &(x0, x1, tx)) in xs.iter().enumerate() {
                let mut rgb = [0u8; 3];
                for (c, slot) in rgb.iter_mut().enumerate() {
                    let top = self.at(x0, y0, c) + (self.at(x1, y0, c) - self.at(x0, y0, c)) * tx;
                    let bottom =
                        self.at(x0, y1, c) + (self.at(x1, y1, c) - self.at(x0, y1, c)) * tx;
                    let v = top + (bottom - top) * ty;
                    *slot = (v.clamp(0.0, 255.0) + 0.5) as u8;
                }
                out.set(i, j, rgb);
            }
        }
        out
    }
}

/// The radii of three successive box blurs that approximate a Gaussian of standard
/// deviation `sigma` (the classic "boxes for Gauss" construction).
fn box_radii(sigma: f64) -> [usize; 3] {
    if !(sigma.is_finite() && sigma > 0.0) {
        return [0; 3];
    }
    const N: f64 = 3.0;
    let ideal = (12.0 * sigma * sigma / N + 1.0).sqrt();
    let mut lower = ideal.floor() as i64;
    if lower % 2 == 0 {
        lower -= 1;
    }
    let lower = lower.max(1);
    let upper = lower + 2;
    let l = lower as f64;
    let m = ((12.0 * sigma * sigma - N * l * l - 4.0 * N * l - 3.0 * N) / (-4.0 * l - 4.0)).round();
    let mut radii = [0usize; 3];
    for (i, r) in radii.iter_mut().enumerate() {
        let width = if (i as f64) < m { lower } else { upper };
        *r = ((width - 1) / 2).max(0) as usize;
    }
    radii
}

/// One box blur of a line with edge replication: every output is the mean of the
/// `2 * radius + 1` values centred on it.
fn box_blur_line(line: &[f32], radius: usize) -> Vec<f32> {
    let n = line.len();
    if n == 0 {
        return Vec::new();
    }
    let at = |i: i64| -> f64 { f64::from(line[i.clamp(0, n as i64 - 1) as usize]) };
    let r = radius as i64;
    let window = (2 * r + 1) as f64;
    let mut sum: f64 = (-r..=r).map(at).sum();
    let mut out = Vec::with_capacity(n);
    for i in 0..n as i64 {
        out.push((sum / window) as f32);
        sum += at(i + r + 1) - at(i - r);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_pixels_become_white_and_opaque_ones_are_kept() {
        assert_eq!(over_white([10, 20, 30, 255]), [10, 20, 30]);
        assert_eq!(over_white([10, 20, 30, 0]), [255, 255, 255]);
        assert_eq!(over_white([0, 0, 0, 128]), [127, 127, 127]);
    }

    #[test]
    fn downscale_averages_and_never_upscales() {
        let mut img = Pixmap::filled(4, 4, [0, 0, 0, 255]);
        for y in 0..4 {
            for x in 2..4 {
                img.set(x, y, [200, 100, 50, 255]);
            }
        }
        let half = downscale(&img, 0, 4, 2, 2);
        assert_eq!((half.w, half.h), (2, 2));
        assert_eq!(half.px(0, 0), [0, 0, 0]);
        assert_eq!(half.px(1, 1), [200, 100, 50]);
        let one = downscale(&img, 0, 4, 1, 1);
        assert_eq!(one.px(0, 0), [100, 50, 25]);
        let same = downscale(&img, 0, 4, 100, 100);
        assert_eq!((same.w, same.h), (4, 4));
        // A column range is honoured and clamped.
        let right = downscale(&img, 2, 99, 100, 100);
        assert_eq!((right.w, right.h), (2, 4));
        assert_eq!(right.px(0, 0), [200, 100, 50]);
    }

    #[test]
    fn box_radii_grow_with_sigma_and_blur_preserves_flat_areas_and_the_mean() {
        assert_eq!(box_radii(0.0), [0, 0, 0]);
        assert_eq!(box_radii(f64::NAN), [0, 0, 0]);
        let small: usize = box_radii(1.0).iter().sum();
        let large: usize = box_radii(10.0).iter().sum();
        assert!(large > small, "{small} vs {large}");

        let mut flat = Grid::new(5, 9);
        flat.data.iter_mut().for_each(|v| *v = 77.0);
        flat.blur(3.0, 2.0);
        assert!(flat.data.iter().all(|v| (v - 77.0).abs() < 1e-3));

        // A step edge becomes a monotonic ramp.
        let mut step = Grid::new(1, 40);
        for y in 20..40 {
            for c in 0..3 {
                step.data[y * 3 + c] = 255.0;
            }
        }
        step.blur(0.0, 4.0);
        let column: Vec<f32> = (0..40).map(|y| step.at(0, y, 0)).collect();
        assert!(column.windows(2).all(|w| w[1] >= w[0] - 1e-3), "{column:?}");
        assert!(column[0] < 1.0 && column[39] > 254.0);
        assert!(column[19] > 60.0 && column[20] < 195.0, "the edge is soft: {column:?}");
    }

    #[test]
    fn upscale_interpolates_between_cells() {
        let mut g = Grid::new(1, 2);
        g.data = vec![0.0, 0.0, 0.0, 200.0, 200.0, 200.0];
        let up = g.upscale(3, 8);
        assert_eq!((up.w, up.h), (3, 8));
        assert_eq!(up.px(1, 0), [0, 0, 0]);
        assert_eq!(up.px(1, 7), [200, 200, 200]);
        let middle = up.px(1, 4)[0];
        assert!((90..=160).contains(&middle), "{middle}");
        let column: Vec<u8> = (0..8).map(|y| up.px(0, y)[0]).collect();
        assert!(column.windows(2).all(|w| w[1] >= w[0]));
    }
}
