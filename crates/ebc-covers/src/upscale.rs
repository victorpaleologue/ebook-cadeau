//! Deterministic Lanczos-3 resampling of a [`Pixmap`], with an optional mild unsharp mask.
//!
//! This is the project's baseline and fallback upscaler (`docs/research/02-cover-sources.md`,
//! §3.4–3.5, implications 40–42): plain resampling to the placed size, which every print
//! workflow does. It adds no detail — [`crate::print_quality`] keeps rating the *native*
//! resolution and reports the factor separately.
//!
//! # Determinism
//!
//! The same input gives the same bytes on every target, including
//! `wasm32-unknown-unknown`: filter weights are computed with `+ − × ÷` only (no
//! platform `sin`/`exp`), rounded to 14-bit fixed point, and all pixel arithmetic is
//! integer.
//!
//! # Super-resolution hook
//!
//! Learned super-resolution (Real-ESRGAN, Swin2SR, waifu2x … — see the research table) is
//! **not** part of this crate: it runs browser-side on ONNX Runtime Web, is opt-in, capped
//! at 4×, and is disabled for licensed covers unless the reseller allows it. The hook is
//! the [`SuperResolution`] trait: the host wraps whatever model it has, and
//! [`upscale_with`] lets the model do the integer-factor step, then brings the result to
//! the exact pixel size with Lanczos-3. With no model the function is identical to
//! [`upscale`].

use ebc_scene::Pixmap;

use crate::CoversError;

/// Largest output side accepted by default (research implication 34).
pub const MAX_SIDE_PX: u32 = 10_000;
/// Largest output pixel count accepted by default (research implication 34).
pub const MAX_PIXELS: u64 = 64_000_000;
/// Largest integer factor a [`SuperResolution`] model is asked for (research
/// implication 41).
pub const MAX_MODEL_FACTOR: u32 = 4;

/// Fixed-point scale of the filter weights (2^14).
const WEIGHT_BITS: u32 = 14;
const WEIGHT_ONE: i32 = 1 << WEIGHT_BITS;
/// Fractional bits kept between the horizontal and the vertical pass.
const MID_BITS: u32 = 6;

/// Parameters of the unsharp mask applied after resampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnsharpMask {
    /// Standard deviation of the Gaussian blur, in output pixels (clamped to 0.3–10).
    pub sigma_px: f32,
    /// Strength: `out = in + amount × (in − blurred)` (clamped to 0–4).
    pub amount: f32,
    /// Differences smaller than this (0–255 scale) are left alone, so that flat areas
    /// and JPEG noise are not amplified.
    pub threshold: u8,
}

impl UnsharpMask {
    /// A mild mask: σ = 1 px, amount 0.35, threshold 2.
    pub const MILD: Self = Self { sigma_px: 1.0, amount: 0.35, threshold: 2 };
}

/// Options of [`upscale`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UpscaleOptions {
    /// Unsharp mask applied after resampling; `None` (the default) for plain resampling.
    pub unsharp: Option<UnsharpMask>,
    /// Refuse outputs with a side longer than this.
    pub max_side_px: u32,
    /// Refuse outputs with more pixels than this.
    pub max_pixels: u64,
}

impl Default for UpscaleOptions {
    fn default() -> Self {
        Self { unsharp: None, max_side_px: MAX_SIDE_PX, max_pixels: MAX_PIXELS }
    }
}

impl UpscaleOptions {
    /// Plain Lanczos-3, no sharpening (same as `default()`).
    #[must_use]
    pub fn plain() -> Self {
        Self::default()
    }

    /// Lanczos-3 followed by [`UnsharpMask::MILD`].
    #[must_use]
    pub fn mild_sharpen() -> Self {
        Self { unsharp: Some(UnsharpMask::MILD), ..Self::default() }
    }
}

/// A learned super-resolution model supplied by the host (see the module docs).
pub trait SuperResolution {
    /// Upscales `image` by the integer `factor` (2, 3 or 4). Return `None` when the model
    /// cannot do it (unsupported factor, image too large, model not loaded): the caller
    /// then falls back to Lanczos-3.
    fn upscale(&self, image: &Pixmap, factor: u32) -> Option<Pixmap>;
}

// ------------------------------------------------------------------------------------
// Deterministic elementary functions
// ------------------------------------------------------------------------------------

/// `sin(π·x)` from a Taylor series, using only exactly rounded operations.
fn sin_pi(x: f64) -> f64 {
    // Reduce to r in [-0.5, 0.5]: sin(π·x) = (−1)^n · sin(π·r) with n = round(x).
    let n = (x + 0.5).floor();
    let t = core::f64::consts::PI * (x - n);
    let t2 = t * t;
    let mut term = t;
    let mut sum = t;
    for k in 1..=12u32 {
        let d = f64::from(2 * k) * f64::from(2 * k + 1);
        term = -term * t2 / d;
        sum += term;
    }
    // `n` is a small integer here (|x| < 4), so the cast is exact.
    if (n as i64) % 2 == 0 { sum } else { -sum }
}

/// The Lanczos kernel with a = 3.
fn lanczos3(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 1e-12 {
        1.0
    } else if ax >= 3.0 {
        0.0
    } else {
        let pi = core::f64::consts::PI;
        3.0 * sin_pi(x) * sin_pi(x / 3.0) / (pi * pi * x * x)
    }
}

/// `e^(−x)` for `x ≥ 0`, from a Taylor series with argument halving.
fn exp_neg(x: f64) -> f64 {
    let mut y = x.max(0.0);
    let mut squarings = 0u32;
    while y > 0.25 && squarings < 64 {
        y /= 2.0;
        squarings += 1;
    }
    let mut term = 1.0;
    let mut sum = 1.0;
    for k in 1..=16u32 {
        term *= y / f64::from(k);
        sum += term;
    }
    for _ in 0..squarings {
        sum *= sum;
    }
    1.0 / sum
}

// ------------------------------------------------------------------------------------
// Filter taps
// ------------------------------------------------------------------------------------

/// For each output sample: the first input index and the fixed-point weights.
struct Taps {
    starts: Vec<usize>,
    /// `width` weights per output sample, zero-padded.
    weights: Vec<i32>,
    width: usize,
}

impl Taps {
    fn of(&self, i: usize) -> (usize, &[i32]) {
        (self.starts[i], &self.weights[i * self.width..(i + 1) * self.width])
    }
}

/// Turns real weights into integers summing exactly to [`WEIGHT_ONE`].
fn quantise(weights: &[f64], out: &mut [i32]) {
    let sum: f64 = weights.iter().sum();
    if sum.is_nan() || sum.abs() <= 1e-9 {
        // Cannot happen for the kernels used here; fall back to the middle tap.
        out.fill(0);
        if let Some(mid) = out.get_mut(weights.len() / 2) {
            *mid = WEIGHT_ONE;
        }
        return;
    }
    let mut total = 0i32;
    let mut largest = 0usize;
    for (i, w) in weights.iter().enumerate() {
        let q = (w / sum * f64::from(WEIGHT_ONE)).round() as i32;
        out[i] = q;
        total += q;
        if q > out[largest] {
            largest = i;
        }
    }
    out[largest] += WEIGHT_ONE - total;
}

/// Lanczos-3 taps for resampling `src` samples to `dst` samples. When shrinking, the
/// kernel is widened by the scale factor so that it also low-pass filters.
fn lanczos_taps(src: usize, dst: usize) -> Taps {
    let scale = src as f64 / dst as f64;
    let stretch = scale.max(1.0);
    let support = 3.0 * stretch;
    let width = ((2.0 * support).ceil() as usize + 1).min(src);
    let mut starts = Vec::with_capacity(dst);
    let mut weights = vec![0i32; dst * width];
    let mut real = vec![0.0f64; width];
    for i in 0..dst {
        let centre = (i as f64 + 0.5) * scale - 0.5;
        let first = (centre - support).ceil() as i64;
        let last = (centre + support).floor() as i64;
        // The window of input samples actually read, clamped to the image and to `width`.
        let lo = first.clamp(0, src as i64 - 1) as usize;
        let lo = lo.min(src - width);
        real.fill(0.0);
        for k in first..=last {
            let w = lanczos3((k as f64 - centre) / stretch);
            // Samples beyond an edge repeat the edge pixel.
            let idx = (k.clamp(0, src as i64 - 1) as usize).clamp(lo, lo + width - 1);
            real[idx - lo] += w;
        }
        quantise(&real, &mut weights[i * width..(i + 1) * width]);
        starts.push(lo);
    }
    Taps { starts, weights, width }
}

/// Normalised Gaussian taps for a blur of standard deviation `sigma`.
fn gaussian_kernel(sigma: f64) -> Vec<i32> {
    let radius = ((3.0 * sigma).ceil() as usize).clamp(1, 32);
    let real: Vec<f64> = (0..=2 * radius)
        .map(|i| {
            let d = i as f64 - radius as f64;
            exp_neg(d * d / (2.0 * sigma * sigma))
        })
        .collect();
    let mut out = vec![0i32; real.len()];
    quantise(&real, &mut out);
    out
}

// ------------------------------------------------------------------------------------
// Resampling
// ------------------------------------------------------------------------------------

fn check_size(width: u32, height: u32, opts: &UpscaleOptions) -> Result<(), CoversError> {
    if width == 0 || height == 0 {
        return Err(CoversError::EmptyImage);
    }
    let pixels = u64::from(width) * u64::from(height);
    if width > opts.max_side_px || height > opts.max_side_px || pixels > opts.max_pixels {
        return Err(CoversError::ImageTooLarge {
            width,
            height,
            max_side_px: opts.max_side_px,
            max_pixels: opts.max_pixels,
        });
    }
    Ok(())
}

fn premultiply(data: &mut [u8]) {
    for px in data.chunks_exact_mut(4) {
        let a = u32::from(px[3]);
        for c in &mut px[..3] {
            *c = ((u32::from(*c) * a + 127) / 255) as u8;
        }
    }
}

fn unpremultiply(data: &mut [u8]) {
    for px in data.chunks_exact_mut(4) {
        let a = u32::from(px[3]);
        if a == 0 {
            px[..3].fill(0);
        } else if a < 255 {
            for c in &mut px[..3] {
                *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
}

/// Separable Lanczos-3 resampling of RGBA bytes.
fn resample(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    let xt = lanczos_taps(sw, dw);
    let yt = lanczos_taps(sh, dh);

    // Horizontal pass: sw × sh → dw × sh, kept with MID_BITS fractional bits.
    let mut mid = vec![0i16; dw * sh * 4];
    for (src_row, mid_row) in src.chunks_exact(sw * 4).zip(mid.chunks_exact_mut(dw * 4)) {
        for (x, out) in mid_row.chunks_exact_mut(4).enumerate() {
            let (start, weights) = xt.of(x);
            let mut acc = [0i32; 4];
            for (k, &w) in weights.iter().enumerate() {
                let px = &src_row[(start + k) * 4..(start + k) * 4 + 4];
                for c in 0..4 {
                    acc[c] += w * i32::from(px[c]);
                }
            }
            for c in 0..4 {
                let v = (acc[c] + (1 << (WEIGHT_BITS - MID_BITS - 1))) >> (WEIGHT_BITS - MID_BITS);
                out[c] = v.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            }
        }
    }

    // Vertical pass: dw × sh → dw × dh.
    let mut out = vec![0u8; dw * dh * 4];
    let mut acc = vec![0i32; dw * 4];
    let shift = WEIGHT_BITS + MID_BITS;
    for (y, out_row) in out.chunks_exact_mut(dw * 4).enumerate() {
        let (start, weights) = yt.of(y);
        acc.fill(0);
        for (k, &w) in weights.iter().enumerate() {
            if w == 0 {
                continue;
            }
            let row = &mid[(start + k) * dw * 4..(start + k + 1) * dw * 4];
            for (a, &m) in acc.iter_mut().zip(row) {
                *a += w * i32::from(m);
            }
        }
        for (o, &a) in out_row.iter_mut().zip(&acc) {
            *o = ((a + (1 << (shift - 1))) >> shift).clamp(0, 255) as u8;
        }
    }
    out
}

/// Blurs the three colour channels of RGBA bytes along one axis, clamping at the edges.
fn blur_axis(src: &[u8], w: usize, h: usize, kernel: &[i32], horizontal: bool) -> Vec<u8> {
    let radius = kernel.len() / 2;
    let mut out = src.to_vec();
    let (len, lines, step, line_step) =
        if horizontal { (w, h, 4, w * 4) } else { (h, w, w * 4, 4) };
    for line in 0..lines {
        let base = line * line_step;
        for i in 0..len {
            let mut acc = [0i32; 3];
            for (k, &kw) in kernel.iter().enumerate() {
                let j = (i + k).saturating_sub(radius).min(len - 1);
                let at = base + j * step;
                for c in 0..3 {
                    acc[c] += kw * i32::from(src[at + c]);
                }
            }
            let at = base + i * step;
            for c in 0..3 {
                out[at + c] = ((acc[c] + WEIGHT_ONE / 2) >> WEIGHT_BITS).clamp(0, 255) as u8;
            }
        }
    }
    out
}

/// Applies an unsharp mask in place on the colour channels of RGBA bytes.
fn unsharp(data: &mut [u8], w: usize, h: usize, mask: &UnsharpMask) {
    let sigma =
        if mask.sigma_px.is_finite() { f64::from(mask.sigma_px).clamp(0.3, 10.0) } else { 1.0 };
    let amount = if mask.amount.is_finite() { f64::from(mask.amount).clamp(0.0, 4.0) } else { 0.0 };
    let amount_fixed = (amount * 256.0).round() as i32;
    if amount_fixed == 0 {
        return;
    }
    let kernel = gaussian_kernel(sigma);
    let blurred = blur_axis(&blur_axis(data, w, h, &kernel, true), w, h, &kernel, false);
    let threshold = i32::from(mask.threshold);
    for (px, soft) in data.chunks_exact_mut(4).zip(blurred.chunks_exact(4)) {
        for c in 0..3 {
            let original = i32::from(px[c]);
            let diff = original - i32::from(soft[c]);
            if diff.abs() >= threshold {
                px[c] = (original + ((amount_fixed * diff + 128) >> 8)).clamp(0, 255) as u8;
            }
        }
    }
}

/// Resamples `src` to exactly `width × height` pixels with a separable Lanczos-3 filter.
///
/// Works for enlarging and for shrinking (the kernel is widened when shrinking). Images
/// with transparency are filtered on premultiplied colour, so no dark fringes appear.
/// The aspect ratio is whatever the caller asks for; covers must be scaled uniformly, so
/// derive the size from one factor ([`upscale_by`]) or from
/// [`crate::Quality::target_px`].
///
/// # Errors
/// [`CoversError::EmptyImage`] for a zero dimension, [`CoversError::ImageTooLarge`] when
/// the output would exceed the limits in `options`.
pub fn upscale(
    src: &Pixmap,
    width: u32,
    height: u32,
    options: &UpscaleOptions,
) -> Result<Pixmap, CoversError> {
    check_size(width, height, options)?;
    let (sw, sh) = (src.width() as usize, src.height() as usize);
    let (dw, dh) = (width as usize, height as usize);
    let opaque = src.is_opaque();
    let mut data = if (sw, sh) == (dw, dh) {
        src.data().to_vec()
    } else if opaque {
        resample(src.data(), sw, sh, dw, dh)
    } else {
        let mut pre = src.data().to_vec();
        premultiply(&mut pre);
        let mut out = resample(&pre, sw, sh, dw, dh);
        // Premultiplied colour can overshoot alpha after filtering; clamp before dividing.
        for px in out.chunks_exact_mut(4) {
            let a = px[3];
            for c in &mut px[..3] {
                *c = (*c).min(a);
            }
        }
        unpremultiply(&mut out);
        out
    };
    if opaque {
        // Ringing must not make an opaque image translucent.
        for px in data.chunks_exact_mut(4) {
            px[3] = 255;
        }
    }
    if let Some(mask) = &options.unsharp {
        unsharp(&mut data, dw, dh, mask);
    }
    Pixmap::from_rgba8(width, height, data).ok_or(CoversError::EmptyImage)
}

/// Resamples `src` uniformly by `factor` (e.g. [`crate::Quality::upscale_factor`]); the
/// output size is the input size times the factor, rounded to the nearest pixel.
///
/// # Errors
/// [`CoversError::InvalidFactor`] when the factor is not a positive finite number, plus
/// the errors of [`upscale`].
pub fn upscale_by(
    src: &Pixmap,
    factor: f64,
    options: &UpscaleOptions,
) -> Result<Pixmap, CoversError> {
    if !factor.is_finite() || factor <= 0.0 {
        return Err(CoversError::InvalidFactor);
    }
    let scaled = |v: u32| {
        let s = (f64::from(v) * factor).round();
        if s >= f64::from(u32::MAX) { u32::MAX } else { (s as u32).max(1) }
    };
    upscale(src, scaled(src.width()), scaled(src.height()), options)
}

/// Like [`upscale`], letting a host-supplied [`SuperResolution`] model do the
/// integer-factor part first.
///
/// The model is asked for the smallest factor in `2..=4` that reaches the requested
/// size (4 when even that is not enough); its output — or the original image when the
/// model declines or returns an image of the wrong size — is then brought to the exact
/// size with Lanczos-3. The model is not called at all when the image is being shrunk
/// or enlarged by less than 1.5×.
///
/// # Errors
/// Same as [`upscale`].
pub fn upscale_with(
    src: &Pixmap,
    width: u32,
    height: u32,
    options: &UpscaleOptions,
    model: &dyn SuperResolution,
) -> Result<Pixmap, CoversError> {
    check_size(width, height, options)?;
    let ratio = (f64::from(width) / f64::from(src.width()))
        .max(f64::from(height) / f64::from(src.height()));
    if ratio >= 1.5 {
        let factor = (ratio.ceil() as u32).clamp(2, MAX_MODEL_FACTOR);
        let expected = (src.width().checked_mul(factor), src.height().checked_mul(factor));
        if let (Some(ew), Some(eh)) = expected
            && check_size(ew, eh, options).is_ok()
            && let Some(enhanced) = model.upscale(src, factor)
            && (enhanced.width(), enhanced.height()) == (ew, eh)
        {
            return upscale(&enhanced, width, height, options);
        }
    }
    upscale(src, width, height, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> Pixmap {
        let mut p = Pixmap::filled(w, h, [0, 0, 0, 255]);
        for y in 0..h {
            for x in 0..w {
                let r = (x * 255 / (w - 1).max(1)) as u8;
                let g = (y * 255 / (h - 1).max(1)) as u8;
                p.set(x, y, [r, g, ((x + y) % 256) as u8, 255]);
            }
        }
        p
    }

    /// FNV-1a, to pin exact output bytes without storing them.
    fn fnv(data: &[u8]) -> u64 {
        data.iter()
            .fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3))
    }

    #[test]
    fn elementary_functions_are_accurate() {
        for i in -80..=80 {
            let x = f64::from(i) * 0.05;
            let exact = (core::f64::consts::PI * x).sin();
            assert!((sin_pi(x) - exact).abs() < 1e-12, "sin_pi({x})");
        }
        for i in 0..=100 {
            let x = f64::from(i) * 0.1;
            assert!((exp_neg(x) - (-x).exp()).abs() < 1e-12, "exp_neg({x})");
        }
        assert_eq!(lanczos3(0.0), 1.0);
        for k in [1.0, 2.0, -1.0, -2.0] {
            assert!(lanczos3(k).abs() < 1e-12, "zero crossing at {k}");
        }
        assert_eq!(lanczos3(3.0), 0.0);
        assert_eq!(lanczos3(-7.5), 0.0);
        assert!((lanczos3(0.5) - 0.607_927_101_854_026_6).abs() < 1e-12);
    }

    #[test]
    fn taps_are_normalised_and_in_range() {
        for (src, dst) in
            [(1usize, 1usize), (1, 7), (2, 9), (5, 5), (10, 3), (100, 421), (640, 97), (3, 1000)]
        {
            let taps = lanczos_taps(src, dst);
            assert_eq!(taps.starts.len(), dst);
            for i in 0..dst {
                let (start, weights) = taps.of(i);
                assert_eq!(weights.iter().sum::<i32>(), WEIGHT_ONE, "{src}->{dst} sample {i}");
                assert!(start + weights.len() <= src, "{src}->{dst} sample {i}");
            }
        }
        let g = gaussian_kernel(1.0);
        assert_eq!(g.len(), 7);
        assert_eq!(g.iter().sum::<i32>(), WEIGHT_ONE);
        assert!(g[3] > g[2] && g[2] > g[1] && g[1] > g[0]);
        assert_eq!(g[0], g[6]);
    }

    #[test]
    fn flat_images_stay_exactly_flat() {
        for colour in [[0u8, 0, 0, 255], [255, 255, 255, 255], [17, 130, 201, 255]] {
            let src = Pixmap::filled(13, 21, colour);
            for (w, h) in [(13u32, 21u32), (52, 84), (7, 9), (100, 33)] {
                for opts in [UpscaleOptions::plain(), UpscaleOptions::mild_sharpen()] {
                    let out = upscale(&src, w, h, &opts).expect("resample");
                    assert_eq!((out.width(), out.height()), (w, h));
                    assert!(
                        out.data().chunks_exact(4).all(|px| px == colour),
                        "{colour:?} {w}x{h}"
                    );
                }
            }
        }
    }

    #[test]
    fn identity_size_returns_the_same_pixels() {
        let src = gradient(31, 17);
        let out = upscale(&src, 31, 17, &UpscaleOptions::plain()).expect("identity");
        assert_eq!(out, src);
    }

    #[test]
    fn gradient_is_preserved_when_enlarging() {
        let src = gradient(64, 48);
        let out = upscale(&src, 256, 192, &UpscaleOptions::plain()).expect("resample");
        // The red channel is a horizontal ramp: it must stay monotonic (within rounding)
        // and reach both ends.
        let row: Vec<i32> =
            (0..256).map(|x| i32::from(out.get(x, 96).expect("in range")[0])).collect();
        assert!(row.windows(2).all(|p| p[1] >= p[0] - 1), "not monotonic: {row:?}");
        assert!(row[0] <= 3 && row[255] >= 252, "ends: {} {}", row[0], row[255]);
        // Interior samples sit close to the ideal ramp.
        for (x, v) in row.iter().enumerate().skip(8).take(240) {
            let ideal = ((x as f64 + 0.5) / 4.0 - 0.5) * 255.0 / 63.0;
            assert!((f64::from(*v) - ideal).abs() < 3.0, "x={x}: {v} vs {ideal}");
        }
        assert!(out.is_opaque());
    }

    #[test]
    fn sharp_edge_stays_sharp_and_overshoot_is_clamped() {
        let mut src = Pixmap::filled(40, 10, [0, 0, 0, 255]);
        for y in 0..10 {
            for x in 20..40 {
                src.set(x, y, [255, 255, 255, 255]);
            }
        }
        let out = upscale(&src, 160, 40, &UpscaleOptions::plain()).expect("resample");
        let row: Vec<u8> = (0..160).map(|x| out.get(x, 20).expect("in range")[0]).collect();
        assert!(row[..70].iter().all(|v| *v <= 12), "left side: {:?}", &row[60..80]);
        assert!(row[90..].iter().all(|v| *v >= 243), "right side: {:?}", &row[80..100]);
        // The transition is a few pixels wide, not a long blur.
        let transition = row.iter().filter(|v| (30..=225).contains(*v)).count();
        assert!((1..=6).contains(&transition), "transition width {transition}");
        // Sharpening steepens the transition without changing flat areas.
        let sharp = upscale(&src, 160, 40, &UpscaleOptions::mild_sharpen()).expect("resample");
        let srow: Vec<u8> = (0..160).map(|x| sharp.get(x, 20).expect("in range")[0]).collect();
        assert_eq!(srow[..60], row[..60]);
        assert_eq!(srow[100..], row[100..]);
        let width = |r: &[u8]| r.iter().filter(|v| (30..=225).contains(*v)).count();
        assert!(width(&srow) <= width(&row));
        assert_ne!(srow, row);
    }

    #[test]
    fn shrinking_averages_instead_of_aliasing() {
        // One-pixel black and white stripes average to mid grey when shrunk 8x.
        let mut src = Pixmap::filled(160, 16, [0, 0, 0, 255]);
        for y in 0..16 {
            for x in (0..160).step_by(2) {
                src.set(x, y, [255, 255, 255, 255]);
            }
        }
        let out = upscale(&src, 20, 2, &UpscaleOptions::plain()).expect("resample");
        for px in out.data().chunks_exact(4).skip(2).take(16) {
            assert!((110..=145).contains(&px[0]), "{px:?}");
        }
    }

    #[test]
    fn transparency_does_not_bleed_colour() {
        // Left half opaque red, right half fully transparent *green*.
        let mut src = Pixmap::filled(20, 8, [255, 0, 0, 255]);
        for y in 0..8 {
            for x in 10..20 {
                src.set(x, y, [0, 255, 0, 0]);
            }
        }
        let out = upscale(&src, 80, 32, &UpscaleOptions::plain()).expect("resample");
        for px in out.data().chunks_exact(4) {
            // Whatever is visible is red: the hidden green never leaks in.
            if px[3] > 0 {
                assert!(px[1] <= 2, "green leaked: {px:?}");
                assert!(px[0] >= 250, "red lost: {px:?}");
            }
        }
        assert_eq!(out.get(2, 16).expect("in range"), [255, 0, 0, 255]);
        assert_eq!(out.get(77, 16).expect("in range")[3], 0);
    }

    #[test]
    fn output_is_pinned_bit_for_bit() {
        // Guards determinism: any change in the arithmetic shows up here, and the same
        // values must come out on every target.
        let src = gradient(23, 37);
        let plain = upscale(&src, 97, 151, &UpscaleOptions::plain()).expect("resample");
        let again = upscale(&src, 97, 151, &UpscaleOptions::plain()).expect("resample");
        assert_eq!(plain, again);
        // The unsharp mask only acts on edges, so pin it on an image that has some.
        let mut blocks = Pixmap::filled(19, 29, [250, 20, 30, 255]);
        for y in 0..29 {
            for x in 0..19 {
                if (x / 4 + y / 4) % 2 == 1 {
                    blocks.set(x, y, [10, 200, 90, 255]);
                }
            }
        }
        let sharp = upscale(&blocks, 97, 151, &UpscaleOptions::mild_sharpen()).expect("resample");
        let unsharpened = upscale(&blocks, 97, 151, &UpscaleOptions::plain()).expect("resample");
        assert_ne!(sharp, unsharpened);
        let small = upscale(&src, 9, 11, &UpscaleOptions::plain()).expect("resample");
        assert_eq!(
            [fnv(plain.data()), fnv(sharp.data()), fnv(small.data())],
            PINNED,
            "resampling output changed"
        );
    }

    // Recorded on x86-64 Linux; see `output_is_pinned_bit_for_bit`.
    const PINNED: [u64; 3] =
        [1_657_750_346_625_254_244, 12_354_784_532_352_411_889, 13_372_679_121_941_524_204];

    #[test]
    fn size_limits_and_bad_factors() {
        let src = gradient(10, 10);
        let opts = UpscaleOptions::plain();
        assert!(matches!(upscale(&src, 0, 10, &opts), Err(CoversError::EmptyImage)));
        assert!(matches!(
            upscale(&src, MAX_SIDE_PX + 1, 10, &opts),
            Err(CoversError::ImageTooLarge { .. })
        ));
        assert!(matches!(upscale(&src, 9000, 9000, &opts), Err(CoversError::ImageTooLarge { .. })));
        let tight = UpscaleOptions { max_pixels: 99, ..opts };
        assert!(upscale(&src, 10, 10, &tight).is_err());
        assert!(upscale(&src, 9, 11, &tight).is_ok());
        for bad in [0.0, -2.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(upscale_by(&src, bad, &opts), Err(CoversError::InvalidFactor)));
        }
        let out = upscale_by(&src, 2.55, &opts).expect("resample");
        assert_eq!((out.width(), out.height()), (26, 26));
        let tiny = upscale_by(&src, 0.01, &opts).expect("resample");
        assert_eq!((tiny.width(), tiny.height()), (1, 1));
    }

    #[test]
    fn odd_unsharp_parameters_are_harmless() {
        let src = gradient(12, 12);
        for mask in [
            UnsharpMask { sigma_px: f32::NAN, amount: 1.0, threshold: 0 },
            UnsharpMask { sigma_px: 1e9, amount: f32::INFINITY, threshold: 0 },
            UnsharpMask { sigma_px: -3.0, amount: -1.0, threshold: 255 },
            UnsharpMask { sigma_px: 0.0, amount: 100.0, threshold: 0 },
        ] {
            let opts = UpscaleOptions { unsharp: Some(mask), ..UpscaleOptions::default() };
            let out = upscale(&src, 30, 30, &opts).expect("resample");
            assert_eq!((out.width(), out.height()), (30, 30));
        }
    }

    struct Doubler {
        calls: std::cell::Cell<u32>,
        lie_about_size: bool,
    }

    impl SuperResolution for Doubler {
        fn upscale(&self, image: &Pixmap, factor: u32) -> Option<Pixmap> {
            self.calls.set(self.calls.get() + 1);
            if self.lie_about_size {
                return Some(Pixmap::filled(3, 3, [1, 2, 3, 255]));
            }
            // A recognisable "model": solid magenta at the requested size.
            Some(Pixmap::filled(
                image.width() * factor,
                image.height() * factor,
                [255, 0, 255, 255],
            ))
        }
    }

    #[test]
    fn super_resolution_hook_is_used_and_checked() {
        let src = gradient(20, 30);
        let opts = UpscaleOptions::plain();
        let model = Doubler { calls: std::cell::Cell::new(0), lie_about_size: false };

        // 3.4x: the model is asked for 4x, then Lanczos brings it down to size.
        let out = upscale_with(&src, 68, 102, &opts, &model).expect("resample");
        assert_eq!(model.calls.get(), 1);
        assert_eq!((out.width(), out.height()), (68, 102));
        assert!(out.data().chunks_exact(4).all(|px| px == [255, 0, 255, 255]));

        // Below 1.5x and when shrinking the model is not consulted.
        let out = upscale_with(&src, 24, 36, &opts, &model).expect("resample");
        assert_eq!(model.calls.get(), 1);
        assert_eq!(out, upscale(&src, 24, 36, &opts).expect("resample"));
        let _ = upscale_with(&src, 10, 15, &opts, &model).expect("resample");
        assert_eq!(model.calls.get(), 1);

        // A model returning the wrong size is ignored.
        let liar = Doubler { calls: std::cell::Cell::new(0), lie_about_size: true };
        let out = upscale_with(&src, 60, 90, &opts, &liar).expect("resample");
        assert_eq!(liar.calls.get(), 1);
        assert_eq!(out, upscale(&src, 60, 90, &opts).expect("resample"));

        // Limits apply before the model runs.
        assert!(upscale_with(&src, 0, 5, &opts, &model).is_err());
    }
}
