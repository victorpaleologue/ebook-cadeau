//! Procedural test covers and image degradations (feature `synth`).
//!
//! Everything here is deterministic: all random choices come from a caller-supplied
//! seed. The covers are *synthetic* — a title typeset with a known font over a flat,
//! graded or textured background, an author line in another font, then rotated,
//! blurred, made noisy and JPEG-compressed. They measure how the pipeline degrades;
//! they do not stand in for real covers (see `docs/research/fontid-eval.md`).

use ebc_scene::Pixmap;
use ebc_text::{Font, ShapeOptions, try_rasterize};

use crate::detect::TextRegion;
use crate::gray::{Gray, quantize};

/// A small deterministic random number generator (SplitMix64).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A generator whose sequence is fully determined by `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `0.0..1.0`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A number in `lo..hi` (`lo` when the range is empty).
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        if hi > lo { lo + (hi - lo) * self.unit() } else { lo }
    }

    /// An index in `0..n` (0 when `n` is 0).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next_u64() % n as u64) as usize }
    }

    /// `true` with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    /// A standard normal deviate (Box–Muller).
    pub fn gauss(&mut self) -> f32 {
        let u = self.unit().max(1e-7);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos()
    }
}

/// The background of a synthetic cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Background {
    /// One colour.
    Flat,
    /// A vertical gradient between two colours of similar lightness.
    Gradient,
    /// Low-frequency clouds plus grain, at reduced contrast with the text.
    Textured,
}

/// What is done to a cover after it is drawn. `Default` is no degradation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Degradation {
    /// Rotation about the centre in degrees (positive = clockwise).
    pub rotation_degrees: f32,
    /// Standard deviation of a Gaussian blur, in pixels.
    pub blur_sigma: f32,
    /// Standard deviation of additive Gaussian noise, in grey levels.
    pub noise_sigma: f32,
    /// JPEG-like compression (8x8 DCT, quantisation, 2x2 chroma subsampling) at this
    /// quality, 1–100. `None` for no compression.
    pub jpeg_quality: Option<u8>,
}

/// The recipe of a synthetic cover.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverSpec {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The title, one string per line.
    pub title_lines: Vec<String>,
    /// Wanted height of the title's capital letters in pixels (reduced if a line would
    /// not fit in 90 % of the width).
    pub cap_height_px: f32,
    /// Letter-spacing of the title in em.
    pub letter_spacing_em: f32,
    /// Light text on a dark background.
    pub light_on_dark: bool,
    /// The background.
    pub background: Background,
    /// An author line, set smaller near the bottom in the second font.
    pub author: Option<String>,
    /// A thin rule under the title and a small disc: ornaments the detector must ignore.
    pub ornaments: bool,
    /// The degradations applied at the end.
    pub degradation: Degradation,
    /// Seed of all random choices (colours, texture, noise, sub-pixel offsets).
    pub seed: u64,
}

/// A synthetic cover and where its title is.
#[derive(Debug, Clone, PartialEq)]
pub struct SynthCover {
    /// The image.
    pub image: Pixmap,
    /// Bounding box of the title (all lines), after rotation; `score` is 1.
    pub title_region: TextRegion,
    /// Height of the title's capital letters as drawn, in pixels.
    pub cap_height_px: f32,
}

/// Draws the cover described by `spec`: the title in `title_font`, the author line in
/// `author_font`. `None` when the spec cannot be drawn (no title, zero size, a font
/// that cannot rasterise the text, a cover too small for the text).
#[must_use]
pub fn render_cover(spec: &CoverSpec, title_font: &Font, author_font: &Font) -> Option<SynthCover> {
    let (w, h) = (spec.width as usize, spec.height as usize);
    if w < 32 || h < 32 || w > 8192 || h > 8192 || spec.title_lines.is_empty() {
        return None;
    }
    let mut rng = Rng::new(spec.seed);

    // --- colours ---------------------------------------------------------------------
    let tint = |rng: &mut Rng, level: f32, spread: f32| -> [f32; 3] {
        let mut c = [0.0f32; 3];
        for v in &mut c {
            *v = (level + rng.range(-spread, spread)).clamp(0.0, 255.0);
        }
        c
    };
    let textured = spec.background == Background::Textured;
    let (bg_level, ink_level) = match (spec.light_on_dark, textured) {
        (false, false) => (rng.range(185.0, 245.0), rng.range(10.0, 60.0)),
        (true, false) => (rng.range(15.0, 75.0), rng.range(200.0, 250.0)),
        (false, true) => (rng.range(165.0, 205.0), rng.range(20.0, 60.0)),
        (true, true) => (rng.range(50.0, 90.0), rng.range(195.0, 240.0)),
    };
    let bg_top = tint(&mut rng, bg_level, 22.0);
    let bg_bottom = match spec.background {
        Background::Flat => bg_top,
        _ => {
            let level = bg_level + rng.range(-28.0, 28.0);
            tint(&mut rng, level, 22.0)
        }
    };
    let ink = tint(&mut rng, ink_level, 10.0);

    // --- background --------------------------------------------------------------------
    let mut planes: [Vec<f32>; 3] = [vec![0.0; w * h], vec![0.0; w * h], vec![0.0; w * h]];
    let clouds = if textured { Some(value_noise(w, h, &mut rng)) } else { None };
    for y in 0..h {
        let t = y as f32 / (h - 1) as f32;
        for x in 0..w {
            let i = y * w + x;
            let cloud = clouds.as_ref().map_or(0.0, |c| c[i]);
            for (plane, (top, bottom)) in planes.iter_mut().zip(bg_top.iter().zip(&bg_bottom)) {
                plane[i] = top + (bottom - top) * t + cloud;
            }
        }
    }
    if textured {
        for plane in &mut planes {
            for v in plane.iter_mut() {
                *v += 5.0 * rng.gauss();
            }
        }
    }

    // --- title -------------------------------------------------------------------------
    let metrics = title_font.metrics();
    let cap_ratio = if metrics.cap_height > 0 && metrics.units_per_em > 0 {
        f32::from(metrics.cap_height) / f32::from(metrics.units_per_em)
    } else {
        0.7
    };
    let opts =
        ShapeOptions { letter_spacing_em: spec.letter_spacing_em, ..ShapeOptions::default() };
    let mut px_per_em = (spec.cap_height_px / cap_ratio).clamp(4.0, 2000.0);
    // Shrink until the widest line fits.
    for _ in 0..4 {
        let mut widest = 0.0f32;
        for line in &spec.title_lines {
            let probe = try_rasterize(title_font, line, px_per_em, &opts).ok()?;
            widest = widest.max(probe.mask.width as f32);
        }
        if widest <= 0.9 * w as f32 {
            break;
        }
        px_per_em *= 0.9 * w as f32 / widest * 0.98;
    }
    let cap_px = px_per_em * cap_ratio;
    let line_step = cap_px * rng.range(1.5, 1.9);
    let block_h = line_step * (spec.title_lines.len() as f32 - 1.0) + cap_px;
    let top_min = 0.08 * h as f32 + cap_px * 0.4;
    let top_max = (0.55 * h as f32 - block_h).max(top_min);
    let mut baseline = rng.range(top_min, top_max) + cap_px;
    let align_left = rng.chance(0.3);
    let left_margin = rng.range(0.05, 0.08) * w as f32;

    let (mut tx0, mut ty0, mut tx1, mut ty1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for line in &spec.title_lines {
        let layer = supersampled(title_font, line, px_per_em, &opts, &mut rng)?;
        let x = if align_left { left_margin } else { (w as f32 - layer.mask.w as f32) * 0.5 };
        let (x, y) = (x.round() as i64, (baseline + layer.top).round() as i64);
        if x < 0 || y < 0 || x as usize + layer.mask.w > w || y as usize + layer.mask.h > h {
            return None;
        }
        composite(&mut planes, w, &layer.mask, x as usize, y as usize, ink);
        tx0 = tx0.min(x as f32);
        ty0 = ty0.min(y as f32);
        tx1 = tx1.max(x as f32 + layer.mask.w as f32);
        ty1 = ty1.max(y as f32 + layer.mask.h as f32);
        baseline += line_step;
    }

    // --- ornaments and author ------------------------------------------------------------
    if spec.ornaments {
        let rule_y = (ty1 + cap_px * 1.1).round() as usize;
        let thickness = (cap_px * 0.06).round().max(1.0) as usize;
        if rule_y + thickness + 2 < h {
            for y in rule_y..rule_y + thickness {
                for x in (0.2 * w as f32) as usize..(0.8 * w as f32) as usize {
                    for (plane, c) in planes.iter_mut().zip(&ink) {
                        plane[y * w + x] = *c;
                    }
                }
            }
        }
        let (cx, cy, r) = (0.5 * w as f32, 0.68 * h as f32, 0.035 * w as f32);
        for y in (cy - r).max(0.0) as usize..((cy + r) as usize).min(h) {
            for x in (cx - r).max(0.0) as usize..((cx + r) as usize).min(w) {
                if (x as f32 - cx).hypot(y as f32 - cy) <= r {
                    for (plane, c) in planes.iter_mut().zip(&ink) {
                        plane[y * w + x] = *c;
                    }
                }
            }
        }
    }
    if let Some(author) = &spec.author {
        let am = author_font.metrics();
        let ratio = if am.cap_height > 0 && am.units_per_em > 0 {
            f32::from(am.cap_height) / f32::from(am.units_per_em)
        } else {
            0.7
        };
        let author_cap = (cap_px * 0.42).max(9.0);
        let plain = ShapeOptions::default();
        if let Some(layer) = supersampled(author_font, author, author_cap / ratio, &plain, &mut rng)
        {
            let x = ((w as f32 - layer.mask.w as f32) * 0.5).round() as i64;
            let y = (0.86 * h as f32 + layer.top).round() as i64;
            if x >= 0 && y >= 0 && x as usize + layer.mask.w <= w && y as usize + layer.mask.h <= h
            {
                composite(&mut planes, w, &layer.mask, x as usize, y as usize, ink);
            }
        }
    }

    let mut data = Vec::with_capacity(w * h * 4);
    for i in 0..w * h {
        data.extend_from_slice(&[
            quantize(planes[0][i]),
            quantize(planes[1][i]),
            quantize(planes[2][i]),
            255,
        ]);
    }
    let clean = Pixmap::from_rgba8(spec.width, spec.height, data)?;
    let image = degrade(&clean, &spec.degradation, spec.seed ^ 0x5eed_c0de);

    // The title box, rotated like the image.
    let (sin, cos) = spec.degradation.rotation_degrees.to_radians().sin_cos();
    let (cx, cy) = ((w as f32 - 1.0) * 0.5, (h as f32 - 1.0) * 0.5);
    let (mut rx0, mut ry0, mut rx1, mut ry1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (px, py) in [(tx0, ty0), (tx1, ty0), (tx0, ty1), (tx1, ty1)] {
        let (dx, dy) = (px - cx, py - cy);
        let (ox, oy) = (cos * dx - sin * dy + cx, sin * dx + cos * dy + cy);
        rx0 = rx0.min(ox);
        ry0 = ry0.min(oy);
        rx1 = rx1.max(ox);
        ry1 = ry1.max(oy);
    }
    let x = rx0.floor().clamp(0.0, w as f32) as u32;
    let y = ry0.floor().clamp(0.0, h as f32) as u32;
    let title_region = TextRegion {
        x,
        y,
        w: (rx1.ceil().clamp(0.0, w as f32) as u32).saturating_sub(x),
        h: (ry1.ceil().clamp(0.0, h as f32) as u32).saturating_sub(y),
        score: 1.0,
    };
    Some(SynthCover { image, title_region, cap_height_px: cap_px })
}

/// A text layer: coverage and the offset of its top edge from the baseline.
struct Layer {
    mask: Gray,
    top: f32,
}

/// Rasterises text at twice the wanted resolution, shifts it by a random sub-pixel
/// offset and averages 2x2 blocks: the anti-aliasing differs from a direct render.
fn supersampled(
    font: &Font,
    text: &str,
    px_per_em: f32,
    opts: &ShapeOptions,
    rng: &mut Rng,
) -> Option<Layer> {
    let big = try_rasterize(font, text, px_per_em * 2.0, opts).ok()?;
    let (bw, bh) = (big.mask.width as usize, big.mask.height as usize);
    if bw == 0 || bh == 0 || big.mask.data.len() < bw * bh {
        return None;
    }
    let (ox, oy) = (rng.below(2), rng.below(2));
    let (pw, ph) = ((bw + ox).div_ceil(2) * 2, (bh + oy).div_ceil(2) * 2);
    let mut padded = Gray::filled(pw, ph, 0);
    for y in 0..bh {
        padded.px[(y + oy) * pw + ox..(y + oy) * pw + ox + bw]
            .copy_from_slice(&big.mask.data[y * bw..(y + 1) * bw]);
    }
    let mask = padded.resize(pw / 2, ph / 2);
    Some(Layer { mask, top: (big.top as f32 - oy as f32) * 0.5 })
}

/// Paints `ink` through a coverage mask.
fn composite(planes: &mut [Vec<f32>; 3], w: usize, mask: &Gray, x: usize, y: usize, ink: [f32; 3]) {
    for my in 0..mask.h {
        for mx in 0..mask.w {
            let a = f32::from(mask.px[my * mask.w + mx]) / 255.0;
            if a <= 0.0 {
                continue;
            }
            let i = (y + my) * w + x + mx;
            for (plane, c) in planes.iter_mut().zip(&ink) {
                plane[i] += (c - plane[i]) * a;
            }
        }
    }
}

/// Two octaves of bilinear value noise, roughly in `-28..28`.
fn value_noise(w: usize, h: usize, rng: &mut Rng) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    for (cell, amplitude) in [(48usize, 20.0f32), (14, 9.0)] {
        let (gw, gh) = (w / cell + 2, h / cell + 2);
        let lattice: Vec<f32> = (0..gw * gh).map(|_| rng.range(-1.0, 1.0)).collect();
        for y in 0..h {
            let (gy, ty) = (y / cell, (y % cell) as f32 / cell as f32);
            for x in 0..w {
                let (gx, tx) = (x / cell, (x % cell) as f32 / cell as f32);
                let at = |ix: usize, iy: usize| lattice[iy * gw + ix];
                let top = at(gx, gy) + (at(gx + 1, gy) - at(gx, gy)) * tx;
                let bottom = at(gx, gy + 1) + (at(gx + 1, gy + 1) - at(gx, gy + 1)) * tx;
                out[y * w + x] += amplitude * (top + (bottom - top) * ty);
            }
        }
    }
    out
}

/// Applies `degradation` to an image: rotation, blur, noise, then JPEG-like
/// compression. Alpha is discarded (the result is opaque).
#[must_use]
pub fn degrade(img: &Pixmap, degradation: &Degradation, seed: u64) -> Pixmap {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut planes: Vec<Gray> = (0..3)
        .map(|c| Gray { w, h, px: img.data().chunks_exact(4).map(|p| p[c]).collect() })
        .collect();
    let d = degradation;
    if d.rotation_degrees.is_finite() && d.rotation_degrees.abs() > 1e-3 {
        for p in &mut planes {
            *p = p.rotated(d.rotation_degrees.to_radians());
        }
    }
    if d.blur_sigma.is_finite() && d.blur_sigma > 0.05 {
        for p in &mut planes {
            *p = p.blurred(d.blur_sigma);
        }
    }
    if d.noise_sigma.is_finite() && d.noise_sigma > 0.0 {
        let mut rng = Rng::new(seed);
        for p in &mut planes {
            for v in &mut p.px {
                *v = quantize(f32::from(*v) + d.noise_sigma * rng.gauss());
            }
        }
    }
    if let Some(quality) = d.jpeg_quality {
        jpeg_like(&mut planes, quality);
    }
    let mut data = Vec::with_capacity(w * h * 4);
    for i in 0..w * h {
        data.extend_from_slice(&[planes[0].px[i], planes[1].px[i], planes[2].px[i], 255]);
    }
    // The dimensions are those of `img`, so this cannot fail; the fallback keeps the
    // function total.
    Pixmap::from_rgba8(img.width(), img.height(), data).unwrap_or_else(|| img.clone())
}

/// The standard JPEG luminance quantisation table (ITU-T T.81, Annex K).
#[rustfmt::skip]
const Q_LUMA: [u8; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61,
    12, 12, 14, 19, 26, 58, 60, 55,
    14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62,
    18, 22, 37, 56, 68, 109, 103, 77,
    24, 35, 55, 64, 81, 104, 113, 92,
    49, 64, 78, 87, 103, 121, 120, 101,
    72, 92, 95, 98, 112, 100, 103, 99,
];

/// The standard JPEG chrominance quantisation table.
#[rustfmt::skip]
const Q_CHROMA: [u8; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99,
    18, 21, 26, 66, 99, 99, 99, 99,
    24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
];

/// JPEG-like lossy round trip on RGB planes: YCbCr, 2x2 chroma subsampling, 8x8 DCT,
/// quantisation with the standard tables scaled as libjpeg does, and back.
fn jpeg_like(planes: &mut [Gray], quality: u8) {
    let (w, h) = (planes[0].w, planes[0].h);
    let n = w * h;
    let (mut yy, mut cb, mut cr) = (vec![0.0f32; n], vec![0.0f32; n], vec![0.0f32; n]);
    for i in 0..n {
        let (r, g, b) =
            (f32::from(planes[0].px[i]), f32::from(planes[1].px[i]), f32::from(planes[2].px[i]));
        yy[i] = 0.299 * r + 0.587 * g + 0.114 * b;
        cb[i] = 128.0 - 0.168_736 * r - 0.331_264 * g + 0.5 * b;
        cr[i] = 128.0 + 0.5 * r - 0.418_688 * g - 0.081_312 * b;
    }
    let quality = f32::from(quality.clamp(1, 100));
    let scale = if quality < 50.0 { 5000.0 / quality } else { 200.0 - 2.0 * quality };
    let table = |base: &[u8; 64]| -> [f32; 64] {
        let mut t = [1.0f32; 64];
        for (o, b) in t.iter_mut().zip(base) {
            *o = ((f32::from(*b) * scale + 50.0) / 100.0).floor().clamp(1.0, 255.0);
        }
        t
    };
    dct_quantise(&mut yy, w, h, &table(&Q_LUMA));
    let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
    for chroma in [&mut cb, &mut cr] {
        let mut small = crate::gray::resize_f32(chroma, w, h, cw, ch);
        dct_quantise(&mut small, cw, ch, &table(&Q_CHROMA));
        *chroma = crate::gray::resize_f32(&small, cw, ch, w, h);
    }
    for i in 0..n {
        let (y, u, v) = (yy[i], cb[i] - 128.0, cr[i] - 128.0);
        planes[0].px[i] = quantize(y + 1.402 * v);
        planes[1].px[i] = quantize(y - 0.344_136 * u - 0.714_136 * v);
        planes[2].px[i] = quantize(y + 1.772 * u);
    }
}

/// Forward 8x8 DCT, quantisation, dequantisation and inverse DCT of every block of a
/// plane (edge blocks replicate the last row and column).
fn dct_quantise(plane: &mut [f32], w: usize, h: usize, table: &[f32; 64]) {
    // basis[u][x] = c(u) * cos((2x + 1) u pi / 16)
    let mut basis = [[0.0f32; 8]; 8];
    for (u, row) in basis.iter_mut().enumerate() {
        let c = if u == 0 { (1.0f32 / 8.0).sqrt() } else { (2.0f32 / 8.0).sqrt() };
        for (x, v) in row.iter_mut().enumerate() {
            *v = c * ((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 16.0).cos();
        }
    }
    let mut block = [0.0f32; 64];
    let mut tmp = [0.0f32; 64];
    for by in (0..h).step_by(8) {
        for bx in (0..w).step_by(8) {
            for y in 0..8 {
                for x in 0..8 {
                    let (sx, sy) = ((bx + x).min(w - 1), (by + y).min(h - 1));
                    block[y * 8 + x] = plane[sy * w + sx] - 128.0;
                }
            }
            // Rows, then columns.
            for y in 0..8 {
                for u in 0..8 {
                    tmp[y * 8 + u] = (0..8).map(|x| block[y * 8 + x] * basis[u][x]).sum();
                }
            }
            for u in 0..8 {
                for v in 0..8 {
                    let coeff: f32 = (0..8).map(|y| tmp[y * 8 + u] * basis[v][y]).sum();
                    let q = table[v * 8 + u];
                    block[v * 8 + u] = (coeff / q).round() * q;
                }
            }
            // Inverse: columns, then rows.
            for u in 0..8 {
                for y in 0..8 {
                    tmp[y * 8 + u] = (0..8).map(|v| block[v * 8 + u] * basis[v][y]).sum();
                }
            }
            for y in 0..8 {
                if by + y >= h {
                    break;
                }
                for x in 0..8 {
                    if bx + x >= w {
                        break;
                    }
                    let value: f32 = (0..8).map(|u| tmp[y * 8 + u] * basis[u][x]).sum();
                    plane[(by + y) * w + bx + x] = value + 128.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        for _ in 0..1000 {
            let (x, y) = (a.unit(), b.unit());
            assert_eq!(x, y);
            assert!((0.0..1.0).contains(&x));
        }
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
        let mut r = Rng::new(3);
        assert_eq!(r.below(0), 0);
        assert!((0..100).all(|_| r.below(5) < 5));
        assert_eq!(r.range(2.0, 2.0), 2.0);
        let mean = (0..4000).map(|_| r.gauss()).sum::<f32>() / 4000.0;
        assert!(mean.abs() < 0.08, "{mean}");
    }

    #[test]
    fn no_degradation_is_the_identity_and_degradations_change_the_image() {
        let mut img = Pixmap::filled(40, 24, [200, 180, 160, 255]);
        for x in 10..30 {
            for y in 8..16 {
                img.set(x, y, [20, 30, 40, 255]);
            }
        }
        assert_eq!(degrade(&img, &Degradation::default(), 1), img);
        let blurred = degrade(&img, &Degradation { blur_sigma: 1.0, ..Default::default() }, 1);
        assert_ne!(blurred, img);
        assert_eq!(blurred.get(0, 0), Some([200, 180, 160, 255]));
        let noisy = degrade(&img, &Degradation { noise_sigma: 8.0, ..Default::default() }, 1);
        assert_ne!(noisy, img);
        assert_eq!(noisy, degrade(&img, &Degradation { noise_sigma: 8.0, ..Default::default() }, 1));
        assert_ne!(noisy, degrade(&img, &Degradation { noise_sigma: 8.0, ..Default::default() }, 2));
    }

    #[test]
    fn jpeg_round_trip_is_close_at_high_quality_and_blocky_at_low() {
        let mut img = Pixmap::filled(48, 32, [220, 220, 220, 255]);
        for x in 0..48 {
            for y in 0..32 {
                if (x / 5 + y / 7) % 2 == 0 {
                    img.set(x, y, [40, 60, 90, 255]);
                }
            }
        }
        let error = |q: u8| -> f32 {
            let out = degrade(&img, &Degradation { jpeg_quality: Some(q), ..Default::default() }, 0);
            img.data()
                .iter()
                .zip(out.data())
                .map(|(a, b)| f32::from(a.abs_diff(*b)))
                .sum::<f32>()
                / img.data().len() as f32
        };
        let (high, low) = (error(95), error(15));
        assert!(high < 6.0, "{high}");
        assert!(low > high * 1.5, "low {low}, high {high}");
        // A flat image survives any quality.
        let flat = Pixmap::filled(20, 20, [128, 128, 128, 255]);
        let out = degrade(&flat, &Degradation { jpeg_quality: Some(1), ..Default::default() }, 0);
        assert!(out.data().chunks_exact(4).all(|p| p[0].abs_diff(128) <= 2));
    }
}
