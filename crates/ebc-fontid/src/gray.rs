//! Single-channel image utilities: conversion, resampling, rotation, blur, integral images.

use ebc_scene::Pixmap;

/// Which channel of an RGBA image to read as grey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Channel {
    /// Rec. 601 luma of the (gamma-encoded) sRGB values.
    Luma,
    /// The red channel.
    Red,
    /// The green channel.
    Green,
    /// The blue channel.
    Blue,
}

/// An 8-bit single-channel image, row-major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Gray {
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) px: Vec<u8>,
}

impl Gray {
    /// A `w` x `h` image filled with `value`.
    pub(crate) fn filled(w: usize, h: usize, value: u8) -> Self {
        Self { w, h, px: vec![value; w * h] }
    }

    /// Reads the rectangle `(x, y, w, h)` of `img` (clamped to the image) as grey.
    /// Transparent pixels are composited over white. Returns `None` if the clamped
    /// rectangle is empty.
    #[cfg(test)]
    pub(crate) fn from_pixmap_rect(
        img: &Pixmap,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        channel: Channel,
    ) -> Option<Self> {
        Self::from_pixmap_rect_reduced(img, x, y, w, h, channel, 1)
    }

    /// Like reading the rectangle `(x, y, w, h)` of `img` (clamped to the image; pixels
    /// composited over white; `None` if empty), but every output pixel is the
    /// average of a `k` x `k` block of source pixels (`k >= 1`; a trailing partial block
    /// is dropped unless it is the only one). This reads a huge image at a reduced size
    /// without any full-size intermediate buffer.
    pub(crate) fn from_pixmap_rect_reduced(
        img: &Pixmap,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        channel: Channel,
        k: u32,
    ) -> Option<Self> {
        if x >= img.width() || y >= img.height() {
            return None;
        }
        let w = w.min(img.width() - x) as usize;
        let h = h.min(img.height() - y) as usize;
        if w == 0 || h == 0 {
            return None;
        }
        let k = (k.max(1) as usize).min(w).min(h);
        let (ow, oh) = (w / k, h / k);
        let stride = img.width() as usize * 4;
        let data = img.data();
        if data.len() < stride * img.height() as usize {
            return None;
        }
        let value = |p: &[u8]| -> u32 {
            let a = u32::from(p[3]);
            let over_white = |c: u8| (u32::from(c) * a + 255 * (255 - a) + 127) / 255;
            match channel {
                Channel::Luma => {
                    let (r, g, b) = (over_white(p[0]), over_white(p[1]), over_white(p[2]));
                    ((77 * r + 150 * g + 29 * b + 128) >> 8).min(255)
                }
                Channel::Red => over_white(p[0]),
                Channel::Green => over_white(p[1]),
                Channel::Blue => over_white(p[2]),
            }
        };
        let mut px = Vec::with_capacity(ow * oh);
        let mut acc = vec![0u32; ow];
        let block = (k * k) as u32;
        for row in 0..oh {
            acc.iter_mut().for_each(|a| *a = 0);
            for dy in 0..k {
                let start = (y as usize + row * k + dy) * stride + x as usize * 4;
                let line = &data[start..start + ow * k * 4];
                for (a, chunk) in acc.iter_mut().zip(line.chunks_exact(4 * k)) {
                    *a += chunk.chunks_exact(4).map(value).sum::<u32>();
                }
            }
            px.extend(acc.iter().map(|&a| ((a + block / 2) / block) as u8));
        }
        Some(Self { w: ow, h: oh, px })
    }

    /// The whole image as grey.
    #[cfg(test)]
    pub(crate) fn from_pixmap(img: &Pixmap, channel: Channel) -> Option<Self> {
        Self::from_pixmap_rect(img, 0, 0, img.width(), img.height(), channel)
    }

    /// Pixel value; coordinates are clamped to the image (edge replication).
    #[inline]
    pub(crate) fn at_clamped(&self, x: isize, y: isize) -> u8 {
        let xc = x.clamp(0, self.w as isize - 1) as usize;
        let yc = y.clamp(0, self.h as isize - 1) as usize;
        self.px[yc * self.w + xc]
    }

    /// `255 - v` for every pixel.
    pub(crate) fn inverted(&self) -> Self {
        Self { w: self.w, h: self.h, px: self.px.iter().map(|&v| 255 - v).collect() }
    }

    /// Resamples to `nw` x `nh`: exact area averaging along an axis that shrinks,
    /// linear interpolation along an axis that grows.
    pub(crate) fn resize(&self, nw: usize, nh: usize) -> Self {
        let (nw, nh) = (nw.max(1), nh.max(1));
        if nw == self.w && nh == self.h {
            return self.clone();
        }
        let src: Vec<f32> = self.px.iter().map(|&v| f32::from(v)).collect();
        let out = resize_f32(&src, self.w, self.h, nw, nh);
        Self { w: nw, h: nh, px: out.iter().map(|&v| quantize(v)).collect() }
    }

    /// Rotates the image by `radians` (positive = clockwise on screen, y down) about its
    /// centre, keeping the size; pixels that come from outside replicate the edge.
    pub(crate) fn rotated(&self, radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        let cx = (self.w as f32 - 1.0) * 0.5;
        let cy = (self.h as f32 - 1.0) * 0.5;
        let mut px = Vec::with_capacity(self.w * self.h);
        for y in 0..self.h {
            let dy = y as f32 - cy;
            for x in 0..self.w {
                let dx = x as f32 - cx;
                // Inverse mapping: where does output (x, y) come from?
                let sx = cos * dx + sin * dy + cx;
                let sy = -sin * dx + cos * dy + cy;
                px.push(quantize(self.sample_bilinear(sx, sy)));
            }
        }
        Self { w: self.w, h: self.h, px }
    }

    /// Bilinear sample at a fractional position, with edge replication.
    pub(crate) fn sample_bilinear(&self, x: f32, y: f32) -> f32 {
        let x = x.clamp(-1.0, self.w as f32);
        let y = y.clamp(-1.0, self.h as f32);
        let (x0, y0) = (x.floor(), y.floor());
        let (tx, ty) = (x - x0, y - y0);
        let (xi, yi) = (x0 as isize, y0 as isize);
        let p00 = f32::from(self.at_clamped(xi, yi));
        let p10 = f32::from(self.at_clamped(xi + 1, yi));
        let p01 = f32::from(self.at_clamped(xi, yi + 1));
        let p11 = f32::from(self.at_clamped(xi + 1, yi + 1));
        let top = p00 + (p10 - p00) * tx;
        let bottom = p01 + (p11 - p01) * tx;
        top + (bottom - top) * ty
    }

    /// Gaussian blur with standard deviation `sigma` pixels (edge replication).
    pub(crate) fn blurred(&self, sigma: f32) -> Self {
        if !(sigma.is_finite() && sigma > 0.05) {
            return self.clone();
        }
        let src: Vec<f32> = self.px.iter().map(|&v| f32::from(v)).collect();
        let out = gaussian_blur_f32(&src, self.w, self.h, sigma);
        Self { w: self.w, h: self.h, px: out.iter().map(|&v| quantize(v)).collect() }
    }
}

#[inline]
pub(crate) fn quantize(v: f32) -> u8 {
    // `as` saturates and maps NaN to 0.
    (v + 0.5).clamp(0.0, 255.0) as u8
}

/// Per-output-sample source window: first source index and the weights of consecutive
/// source samples.
fn axis_weights(src: usize, dst: usize) -> Vec<(usize, Vec<f32>)> {
    let scale = src as f32 / dst as f32;
    let mut table = Vec::with_capacity(dst);
    if scale >= 1.0 {
        for i in 0..dst {
            let a = i as f32 * scale;
            let b = ((i + 1) as f32 * scale).min(src as f32);
            let start = (a.floor() as usize).min(src - 1);
            let end = (b.ceil() as usize).clamp(start + 1, src);
            let mut weights = Vec::with_capacity(end - start);
            let mut total = 0.0;
            for j in start..end {
                let lo = a.max(j as f32);
                let hi = b.min((j + 1) as f32);
                let wgt = (hi - lo).max(0.0);
                weights.push(wgt);
                total += wgt;
            }
            if total <= 0.0 {
                weights.iter_mut().for_each(|w| *w = 1.0);
                total = weights.len() as f32;
            }
            weights.iter_mut().for_each(|w| *w /= total);
            table.push((start, weights));
        }
    } else {
        for i in 0..dst {
            let c = ((i as f32 + 0.5) * scale - 0.5).clamp(0.0, (src - 1) as f32);
            let j0 = (c.floor() as usize).min(src - 1);
            let t = c - j0 as f32;
            if j0 + 1 < src {
                table.push((j0, vec![1.0 - t, t]));
            } else {
                table.push((j0, vec![1.0]));
            }
        }
    }
    table
}

/// Separable resampling of an `f32` image (see [`Gray::resize`]).
pub(crate) fn resize_f32(src: &[f32], w: usize, h: usize, nw: usize, nh: usize) -> Vec<f32> {
    if w == 0 || h == 0 || nw == 0 || nh == 0 {
        return vec![0.0; nw * nh];
    }
    let xs = axis_weights(w, nw);
    let ys = axis_weights(h, nh);
    let mut tmp = vec![0.0f32; nw * h];
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let out = &mut tmp[y * nw..(y + 1) * nw];
        for (o, (start, weights)) in out.iter_mut().zip(&xs) {
            let mut acc = 0.0;
            for (k, wgt) in weights.iter().enumerate() {
                acc += row[start + k] * wgt;
            }
            *o = acc;
        }
    }
    let mut out = vec![0.0f32; nw * nh];
    for (y, (start, weights)) in ys.iter().enumerate() {
        let dst = &mut out[y * nw..(y + 1) * nw];
        for (k, wgt) in weights.iter().enumerate() {
            let row = &tmp[(start + k) * nw..(start + k + 1) * nw];
            for (d, s) in dst.iter_mut().zip(row) {
                *d += s * wgt;
            }
        }
    }
    out
}

/// Separable Gaussian blur of an `f32` image with edge replication.
pub(crate) fn gaussian_blur_f32(src: &[f32], w: usize, h: usize, sigma: f32) -> Vec<f32> {
    if w == 0 || h == 0 || !(sigma.is_finite() && sigma > 0.0) {
        return src.to_vec();
    }
    let radius = ((sigma * 3.0).ceil() as usize).clamp(1, 64);
    let mut kernel: Vec<f32> = (0..=2 * radius)
        .map(|i| {
            let d = i as f32 - radius as f32;
            (-0.5 * d * d / (sigma * sigma)).exp()
        })
        .collect();
    let total: f32 = kernel.iter().sum();
    kernel.iter_mut().for_each(|k| *k /= total);

    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        for x in 0..w {
            let mut acc = 0.0;
            for (k, wgt) in kernel.iter().enumerate() {
                let sx = (x + k).saturating_sub(radius).min(w - 1);
                acc += row[sx] * wgt;
            }
            tmp[y * w + x] = acc;
        }
    }
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        for (k, wgt) in kernel.iter().enumerate() {
            let sy = (y + k).saturating_sub(radius).min(h - 1);
            let row = &tmp[sy * w..(sy + 1) * w];
            let dst = &mut out[y * w..(y + 1) * w];
            for (d, s) in dst.iter_mut().zip(row) {
                *d += s * wgt;
            }
        }
    }
    out
}

/// Summed-area tables of a weighted image: sums of `v`, `v²` and of the weights.
pub(crate) struct Integral {
    w: usize,
    h: usize,
    sum: Vec<f64>,
    sq: Vec<f64>,
    count: Vec<f64>,
}

impl Integral {
    /// Tables over the pixels of `g` whose `keep` flag is non-zero (all pixels when
    /// `keep` is `None`).
    pub(crate) fn new(g: &Gray, keep: Option<&[u8]>) -> Self {
        let (w, h) = (g.w, g.h);
        let stride = w + 1;
        let mut sum = vec![0.0f64; stride * (h + 1)];
        let mut sq = vec![0.0f64; stride * (h + 1)];
        let mut count = vec![0.0f64; stride * (h + 1)];
        for y in 0..h {
            let (mut rs, mut rq, mut rc) = (0.0f64, 0.0f64, 0.0f64);
            for x in 0..w {
                let i = y * w + x;
                if keep.is_none_or(|k| k[i] != 0) {
                    let v = f64::from(g.px[i]);
                    rs += v;
                    rq += v * v;
                    rc += 1.0;
                }
                let o = (y + 1) * stride + x + 1;
                sum[o] = sum[o - stride] + rs;
                sq[o] = sq[o - stride] + rq;
                count[o] = count[o - stride] + rc;
            }
        }
        Self { w, h, sum, sq, count }
    }

    #[inline]
    fn rect(table: &[f64], stride: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> f64 {
        table[y1 * stride + x1] - table[y0 * stride + x1] - table[y1 * stride + x0]
            + table[y0 * stride + x0]
    }

    /// `(count, mean, standard deviation)` of the kept pixels in the window of half-size
    /// `r` centred on `(x, y)`, clipped to the image.
    #[inline]
    pub(crate) fn window(&self, x: usize, y: usize, r: usize) -> (f32, f32, f32) {
        let x0 = x.saturating_sub(r);
        let y0 = y.saturating_sub(r);
        let x1 = (x + r + 1).min(self.w);
        let y1 = (y + r + 1).min(self.h);
        let stride = self.w + 1;
        let n = Self::rect(&self.count, stride, x0, y0, x1, y1);
        if n <= 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let s = Self::rect(&self.sum, stride, x0, y0, x1, y1);
        let q = Self::rect(&self.sq, stride, x0, y0, x1, y1);
        let mean = s / n;
        let var = (q / n - mean * mean).max(0.0);
        (n as f32, mean as f32, var.sqrt() as f32)
    }
}

/// Robust estimate of the pixel-noise standard deviation: the median absolute difference
/// of horizontal neighbours, scaled for a Gaussian. Edges are sparse, so they do not
/// move the median.
pub(crate) fn noise_sigma(g: &Gray) -> f32 {
    let mut hist = [0u32; 256];
    let mut n = 0u32;
    let step = (g.h / 256).max(1);
    for y in (0..g.h).step_by(step) {
        let row = &g.px[y * g.w..(y + 1) * g.w];
        for pair in row.windows(2) {
            hist[usize::from(pair[0].abs_diff(pair[1]))] += 1;
            n += 1;
        }
    }
    if n == 0 {
        return 0.0;
    }
    let mut acc = 0u32;
    let mut median = 0usize;
    for (d, &c) in hist.iter().enumerate() {
        acc += c;
        if acc * 2 >= n {
            median = d;
            break;
        }
    }
    // median(|a - b|) = 0.6745 * sqrt(2) * sigma for Gaussian noise.
    median as f32 / 0.9539
}

/// Otsu's threshold of a histogram: pixels `<= t` form the dark class. Also returns the
/// means of the dark and light classes. `None` when the histogram has a single value.
pub(crate) fn otsu(hist: &[u32; 256]) -> Option<(u8, f32, f32)> {
    let total: u64 = hist.iter().map(|&c| u64::from(c)).sum();
    if total == 0 {
        return None;
    }
    let sum_all: f64 = hist.iter().enumerate().map(|(v, &c)| v as f64 * f64::from(c)).sum();
    let (mut w0, mut s0) = (0.0f64, 0.0f64);
    let mut best = (0.0f64, None);
    for t in 0..255usize {
        w0 += f64::from(hist[t]);
        s0 += t as f64 * f64::from(hist[t]);
        let w1 = total as f64 - w0;
        if w0 <= 0.0 || w1 <= 0.0 {
            continue;
        }
        let (m0, m1) = (s0 / w0, (sum_all - s0) / w1);
        let between = w0 * w1 * (m0 - m1) * (m0 - m1);
        if between > best.0 {
            best = (between, Some((t as u8, m0 as f32, m1 as f32)));
        }
    }
    best.1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(w: usize, h: usize) -> Gray {
        let px = (0..w * h).map(|i| ((i % w) * 255 / (w - 1)) as u8).collect();
        Gray { w, h, px }
    }

    #[test]
    fn luma_composites_transparency_over_white() {
        let mut img = Pixmap::filled(2, 1, [0, 0, 0, 255]);
        img.set(1, 0, [0, 0, 0, 0]);
        let g = Gray::from_pixmap(&img, Channel::Luma).expect("non-empty");
        assert_eq!(g.px, vec![0, 255]);
        let red = Pixmap::filled(1, 1, [255, 0, 0, 255]);
        assert_eq!(Gray::from_pixmap(&red, Channel::Luma).expect("non-empty").px, vec![77]);
        assert_eq!(Gray::from_pixmap(&red, Channel::Red).expect("non-empty").px, vec![255]);
        assert_eq!(Gray::from_pixmap(&red, Channel::Blue).expect("non-empty").px, vec![0]);
    }

    #[test]
    fn rect_is_clamped_and_empty_is_none() {
        let img = Pixmap::filled(4, 3, [10, 10, 10, 255]);
        let g = Gray::from_pixmap_rect(&img, 2, 1, 100, 100, Channel::Luma).expect("clamped");
        assert_eq!((g.w, g.h), (2, 2));
        assert!(Gray::from_pixmap_rect(&img, 4, 0, 1, 1, Channel::Luma).is_none());
        assert!(Gray::from_pixmap_rect(&img, 0, 0, 0, 1, Channel::Luma).is_none());
    }

    #[test]
    fn reduced_reading_averages_blocks() {
        let mut img = Pixmap::filled(5, 4, [0, 0, 0, 255]);
        for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            img.set(x, y, [200, 200, 200, 255]);
        }
        img.set(2, 2, [255, 255, 255, 255]);
        let g = Gray::from_pixmap_rect_reduced(&img, 0, 0, 5, 4, Channel::Luma, 2).expect("ok");
        assert_eq!((g.w, g.h), (2, 2));
        assert_eq!(g.px, vec![200, 0, 0, 64]);
        // A factor larger than the image collapses to one pixel instead of nothing.
        let one = Gray::from_pixmap_rect_reduced(&img, 0, 0, 5, 4, Channel::Luma, 99).expect("ok");
        assert_eq!((one.w, one.h), (1, 1));
        assert!(Gray::from_pixmap_rect_reduced(&img, 9, 0, 5, 4, Channel::Luma, 2).is_none());
    }

    #[test]
    fn area_downscale_preserves_the_mean_and_upscale_interpolates() {
        let g = ramp(64, 8);
        let small = g.resize(8, 2);
        let mean = |g: &Gray| g.px.iter().map(|&v| f64::from(v)).sum::<f64>() / g.px.len() as f64;
        assert!((mean(&g) - mean(&small)).abs() < 1.0);
        let big = small.resize(64, 8);
        assert_eq!((big.w, big.h), (64, 8));
        // Monotone along x, like the source.
        assert!(big.px[..64].windows(2).all(|p| p[0] <= p[1]));
        // Identity resize.
        assert_eq!(g.resize(64, 8), g);
    }

    #[test]
    fn rotation_by_zero_is_identity_and_by_pi_flips() {
        let g = ramp(9, 5);
        assert_eq!(g.rotated(0.0), g);
        let flipped = g.rotated(std::f32::consts::PI);
        for y in 0..5 {
            for x in 0..9 {
                let a = i32::from(flipped.px[y * 9 + x]);
                let b = i32::from(g.px[(4 - y) * 9 + (8 - x)]);
                assert!((a - b).abs() <= 1, "({x},{y}): {a} vs {b}");
            }
        }
    }

    #[test]
    fn blur_keeps_flat_images_flat_and_spreads_an_impulse() {
        let flat = Gray::filled(7, 7, 93);
        assert_eq!(flat.blurred(1.5), flat);
        let mut dot = Gray::filled(9, 9, 0);
        dot.px[4 * 9 + 4] = 255;
        let b = dot.blurred(1.0);
        assert!(b.px[4 * 9 + 4] < 255 && b.px[4 * 9 + 4] > b.px[4 * 9 + 5]);
        assert!(b.px[4 * 9 + 5] > 0);
        assert_eq!(b.px[4 * 9 + 5], b.px[5 * 9 + 4]);
    }

    #[test]
    fn integral_window_statistics() {
        let g = Gray { w: 4, h: 1, px: vec![0, 0, 200, 200] };
        let it = Integral::new(&g, None);
        let (n, mean, sd) = it.window(1, 0, 10);
        assert_eq!(n, 4.0);
        assert!((mean - 100.0).abs() < 1e-3 && (sd - 100.0).abs() < 1e-3);
        let keep = [1u8, 1, 0, 0];
        let masked = Integral::new(&g, Some(&keep));
        let (n, mean, sd) = masked.window(3, 0, 10);
        assert_eq!((n, mean, sd), (2.0, 0.0, 0.0));
        let (n, _, _) = masked.window(3, 0, 0);
        assert_eq!(n, 0.0);
    }

    #[test]
    fn otsu_separates_two_levels() {
        let mut hist = [0u32; 256];
        hist[40] = 500;
        hist[200] = 100;
        let (t, dark, light) = otsu(&hist).expect("two classes");
        assert!((40..200).contains(&t));
        assert!((dark - 40.0).abs() < 0.5 && (light - 200.0).abs() < 0.5);
        let mut single = [0u32; 256];
        single[7] = 10;
        assert!(otsu(&single).is_none());
        assert!(otsu(&[0u32; 256]).is_none());
    }

    #[test]
    fn noise_estimate_is_zero_on_flat_and_positive_on_noise() {
        assert_eq!(noise_sigma(&Gray::filled(32, 32, 120)), 0.0);
        let mut state = 12345u32;
        let px = (0..64 * 64)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                120 + ((state >> 24) % 21) as u8
            })
            .collect();
        let noisy = Gray { w: 64, h: 64, px };
        let sigma = noise_sigma(&noisy);
        assert!(sigma > 3.0 && sigma < 12.0, "sigma = {sigma}");
    }
}
