//! Palette extraction (k-means++ in CIELAB) and analysis of a cover's edges.
//!
//! # Pipeline
//!
//! 1. **Sample.** The region is divided into a grid of at most 16 384 square cells and one
//!    pixel is read from each cell, at a position chosen by a fixed hash of the cell index
//!    (stratified jittered sampling). Small regions are read in full. Jitter avoids the
//!    aliasing a regular stride would show on pinstripes and halftone screens, and reading
//!    real pixels (rather than block averages) keeps thin lettering in its true colour.
//!    Pixels are composited over white first, because that is what a transparent pixel
//!    becomes on paper.
//! 2. **Deduplicate.** Identical colours are counted once with a weight.
//! 3. **Cluster.** Greedy k-means++ seeding with a fixed seed, then Lloyd iterations, on the
//!    CIELAB coordinates, with ties broken by index.
//! 4. **Merge and order.** Clusters closer than [`PALETTE_MERGE_DELTA_E`] are merged, so a
//!    cover with two flat (or noisy but flat-looking) colours yields two swatches whatever
//!    `k` was asked for. Swatches are sorted by decreasing share, then by colour.

use ebc_scene::Pixmap;

use crate::Rgb8;
use crate::lab::Lab;

/// Upper bound on the number of swatches [`palette`] returns; larger `k` values are clamped.
pub const MAX_PALETTE_SIZE: usize = 64;

/// Clusters whose centres are closer than this CIE76 distance (ΔE\*ab) are reported as one
/// swatch. About 2.3 is a just-noticeable difference; 5 — "the same colour at a glance" —
/// also absorbs the way k-means splits one noisy flat area (JPEG artefacts, paper grain)
/// into two halves.
pub const PALETTE_MERGE_DELTA_E: f64 = 5.0;

/// The colour spread (RMS ΔE\*ab around the mean colour) at which [`edge_uniformity`]
/// reaches 0.0.
pub const EDGE_BUSY_SIGMA: f64 = 30.0;

/// Suggested threshold on [`edge_uniformity`]: at or above it (an RMS spread of at most
/// 3 ΔE\*ab) an edge is flat enough to be continued as a solid colour.
pub const UNIFORM_EDGE_THRESHOLD: f32 = 0.9;

/// Maximum number of pixels read from a region.
const MAX_SAMPLES: u64 = 16_384;
/// Fixed seed for sampling jitter and k-means++ ("ebc-colo" in ASCII).
const SEED: u64 = 0x6562_632D_636F_6C6F;
/// Upper bound on Lloyd iterations (convergence normally takes fewer than 20).
const MAX_ITERATIONS: usize = 64;

/// One colour of a palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swatch {
    /// The cluster's centre, as sRGB.
    pub color: Rgb8,
    /// Share of the analysed pixels belonging to this cluster, `0.0..=1.0`. The shares of
    /// a palette add up to 1.
    pub share: f32,
}

/// A side of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The left side (the binding edge of a front cover for left-to-right books).
    Left,
    /// The right side.
    Right,
    /// The top side.
    Top,
    /// The bottom side.
    Bottom,
}

impl Edge {
    /// All four edges, in the order left, right, top, bottom.
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];
}

/// Extracts up to `k` representative colours from an image, most frequent first.
///
/// The result is deterministic (fixed seed, fixed sampling pattern, stable ordering) and
/// has **at most** `min(k, MAX_PALETTE_SIZE)` entries: fewer when the image has fewer
/// distinct colours or when clusters are visually indistinguishable (see the module
/// documentation). `k = 0` yields an empty palette; any `k ≥ 1` yields at least one swatch.
/// Transparent pixels count as white.
///
/// Cost is bounded regardless of the image size (at most 16 384 pixels are read).
#[must_use]
pub fn palette(img: &Pixmap, k: usize) -> Vec<Swatch> {
    palette_of(img, Region::whole(img), k)
}

/// Like [`palette`], restricted to a strip along one edge of the image.
///
/// `strip_fraction` is the strip's thickness as a fraction of the image width (left/right
/// edges) or height (top/bottom edges). It is clamped to `0.0..=1.0`, NaN counts as 0, and
/// the strip is always at least one pixel thick.
#[must_use]
pub fn edge_palette(img: &Pixmap, edge: Edge, strip_fraction: f32, k: usize) -> Vec<Swatch> {
    palette_of(img, Region::strip(img, edge, strip_fraction), k)
}

/// How flat the strip along one edge is: 1.0 for a single colour, 0.0 for a busy edge.
///
/// The value is `1 − σ / EDGE_BUSY_SIGMA` clamped to `0.0..=1.0`, where σ is the RMS CIE76
/// distance of the strip's pixels to their mean colour. For orientation: light JPEG noise
/// on a flat colour gives about 0.95–0.99, a smooth gradient anything from 0.0 (black to
/// white) to about 0.6 (a gentle one), a half-black, half-white strip and most photographs
/// 0.0. Compare with
/// [`UNIFORM_EDGE_THRESHOLD`]. `strip_fraction` is interpreted as in [`edge_palette`].
#[must_use]
pub fn edge_uniformity(img: &Pixmap, edge: Edge, strip_fraction: f32) -> f32 {
    let samples = sample(img, Region::strip(img, edge, strip_fraction));
    let (_, sigma) = mean_and_sigma(&samples);
    (1.0 - sigma / EDGE_BUSY_SIGMA).clamp(0.0, 1.0) as f32
}

/// Tuning for [`uniform_border_with`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderOptions {
    /// Thickness of the examined band along each edge, as a fraction of the image's
    /// shorter side (clamped to `0.0..=0.5`; at least one pixel).
    pub band_fraction: f32,
    /// Maximum CIE76 distance (ΔE\*ab) from the border colour for a pixel to count as
    /// part of the border.
    pub tolerance_delta_e: f64,
    /// Minimum share of each edge's band, `0.0..=1.0`, that must be within the tolerance.
    pub min_coverage: f64,
}

impl Default for BorderOptions {
    /// A 2 % band, a tolerance of 5 ΔE\*ab and 97 % coverage: accepts a frame with JPEG
    /// noise and the odd speck, rejects one that the artwork touches.
    fn default() -> Self {
        Self { band_fraction: 0.02, tolerance_delta_e: 5.0, min_coverage: 0.97 }
    }
}

/// Detects a cover whose outer rim is one solid colour — typically a white or coloured
/// frame around the artwork — and returns that colour.
///
/// Such a border can be continued onto the spine and into the bleed as a flat tint instead
/// of mirroring or stretching the artwork. Uses [`BorderOptions::default`]. Note that an
/// image that is a single flat colour throughout also has a uniform border.
#[must_use]
pub fn has_uniform_border(img: &Pixmap) -> Option<Rgb8> {
    uniform_border_with(img, &BorderOptions::default())
}

/// [`has_uniform_border`] with explicit tuning.
///
/// The border colour is the per-channel median (in CIELAB) of the four edge bands taken
/// together; the border is accepted when, on **each** of the four edges, at least
/// `min_coverage` of the band lies within `tolerance_delta_e` of that colour. Hostile
/// options never panic: a NaN or negative tolerance, or a NaN coverage, rejects every
/// image.
#[must_use]
pub fn uniform_border_with(img: &Pixmap, options: &BorderOptions) -> Option<Rgb8> {
    let shorter = img.width().min(img.height());
    let fraction =
        if options.band_fraction.is_nan() { 0.0 } else { options.band_fraction.clamp(0.0, 0.5) };
    let band = ((f64::from(shorter) * f64::from(fraction)).round() as u32).clamp(1, shorter);

    let bands: Vec<Vec<Sample>> =
        Edge::ALL.iter().map(|&edge| sample(img, Region::band(img, edge, band))).collect();

    let channel_median = |channel: fn(&Lab) -> f64| {
        let mut values: Vec<(f64, f64)> =
            bands.iter().flatten().map(|s| (channel(&s.lab), s.weight)).collect();
        weighted_median(&mut values)
    };
    let centre =
        Lab::new(channel_median(|c| c.l)?, channel_median(|c| c.a)?, channel_median(|c| c.b)?);

    for samples in &bands {
        let total: f64 = samples.iter().map(|s| s.weight).sum();
        // A NaN or negative tolerance matches nothing.
        let inside: f64 = samples
            .iter()
            .filter(|s| s.lab.delta_e76(centre) <= options.tolerance_delta_e)
            .map(|s| s.weight)
            .sum();
        // Written so that a NaN on either side rejects.
        let covered = total > 0.0 && inside / total >= options.min_coverage;
        if !covered {
            return None;
        }
    }
    Some(centre.to_rgb8())
}

// ---------------------------------------------------------------------------------------
// Regions and sampling
// ---------------------------------------------------------------------------------------

/// A non-empty pixel rectangle inside an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Region {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Region {
    fn whole(img: &Pixmap) -> Self {
        Self { x: 0, y: 0, w: img.width(), h: img.height() }
    }

    /// A strip along `edge` whose thickness is `fraction` of the relevant dimension.
    fn strip(img: &Pixmap, edge: Edge, fraction: f32) -> Self {
        let dimension = match edge {
            Edge::Left | Edge::Right => img.width(),
            Edge::Top | Edge::Bottom => img.height(),
        };
        let fraction = if fraction.is_nan() { 0.0 } else { f64::from(fraction).clamp(0.0, 1.0) };
        let thickness = (f64::from(dimension) * fraction).round() as u32;
        Self::band(img, edge, thickness)
    }

    /// A strip along `edge`, `thickness` pixels thick (clamped to `1..=dimension`).
    fn band(img: &Pixmap, edge: Edge, thickness: u32) -> Self {
        let (w, h) = (img.width(), img.height());
        match edge {
            Edge::Left => Self { x: 0, y: 0, w: thickness.clamp(1, w), h },
            Edge::Right => {
                let t = thickness.clamp(1, w);
                Self { x: w - t, y: 0, w: t, h }
            }
            Edge::Top => Self { x: 0, y: 0, w, h: thickness.clamp(1, h) },
            Edge::Bottom => {
                let t = thickness.clamp(1, h);
                Self { x: 0, y: h - t, w, h: t }
            }
        }
    }
}

/// A distinct colour found in a region, with the number of sampled pixels that have it.
#[derive(Debug, Clone, Copy)]
struct Sample {
    lab: Lab,
    weight: f64,
}

/// The SplitMix64 finaliser: a stateless 64-bit mixing function.
fn mix(z: u64) -> u64 {
    let mut z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// SplitMix64 pseudo-random generator (public domain, Sebastiano Vigna).
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        let out = mix(self.0);
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        out
    }

    /// Uniform in `[0, 1)` with 53 bits of precision.
    fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / (1u64 << 53) as f64;
        (self.next_u64() >> 11) as f64 * SCALE
    }
}

/// The side of the square sampling cells: the smallest that keeps the number of cells
/// within [`MAX_SAMPLES`].
fn sampling_step(w: u32, h: u32) -> u32 {
    let (w, h) = (u64::from(w), u64::from(h));
    let total = w * h;
    if total <= MAX_SAMPLES {
        return 1;
    }
    let mut step = (total / MAX_SAMPLES).isqrt().max(1);
    while w.div_ceil(step) * h.div_ceil(step) > MAX_SAMPLES {
        step += 1;
    }
    // `step` never exceeds the larger dimension, which is a u32.
    u32::try_from(step).unwrap_or(u32::MAX)
}

/// Composites a non-premultiplied RGBA pixel over white.
fn over_white([r, g, b, a]: [u8; 4]) -> Rgb8 {
    if a == 255 {
        return Rgb8::new(r, g, b);
    }
    let alpha = u32::from(a);
    let blend = |c: u8| ((u32::from(c) * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
    Rgb8::new(blend(r), blend(g), blend(b))
}

/// Reads the region's pixels (all of them, or a jittered grid) and returns the distinct
/// colours in ascending RGB order with their counts.
fn sample(img: &Pixmap, region: Region) -> Vec<Sample> {
    let step = sampling_step(region.w, region.h);
    let (cols, rows) = (region.w.div_ceil(step), region.h.div_ceil(step));
    let data = img.data();
    let stride = img.width() as usize;

    let mut keys: Vec<u32> = Vec::with_capacity(cols as usize * rows as usize);
    for cy in 0..rows {
        let cell_h = step.min(region.h - cy * step);
        for cx in 0..cols {
            let cell_w = step.min(region.w - cx * step);
            let (dx, dy) = if step == 1 {
                (0, 0)
            } else {
                let hash = mix(SEED ^ (u64::from(cy) << 32 | u64::from(cx)));
                ((hash as u32) % cell_w, ((hash >> 32) as u32) % cell_h)
            };
            let x = (region.x + cx * step + dx) as usize;
            let y = (region.y + cy * step + dy) as usize;
            let offset = (y * stride + x) * 4;
            let pixel = data.get(offset..offset + 4).and_then(|p| <[u8; 4]>::try_from(p).ok());
            if let Some(rgba) = pixel {
                keys.push(over_white(rgba).to_key());
            }
        }
    }
    keys.sort_unstable();

    let mut samples: Vec<Sample> = Vec::new();
    let mut previous: Option<u32> = None;
    for key in keys {
        match samples.last_mut() {
            Some(last) if previous == Some(key) => last.weight += 1.0,
            _ => samples.push(Sample { lab: Lab::from_rgb8(Rgb8::from_key(key)), weight: 1.0 }),
        }
        previous = Some(key);
    }
    samples
}

/// Weighted mean colour and RMS distance to it. `(black, 0)` for an empty slice.
fn mean_and_sigma(samples: &[Sample]) -> (Lab, f64) {
    let total: f64 = samples.iter().map(|s| s.weight).sum();
    if total <= 0.0 {
        return (Lab::default(), 0.0);
    }
    let mut sum = [0.0_f64; 3];
    for s in samples {
        sum[0] += s.weight * s.lab.l;
        sum[1] += s.weight * s.lab.a;
        sum[2] += s.weight * s.lab.b;
    }
    let mean = Lab::new(sum[0] / total, sum[1] / total, sum[2] / total);
    let variance: f64 =
        samples.iter().map(|s| s.weight * s.lab.distance_squared(mean)).sum::<f64>() / total;
    (mean, variance.sqrt())
}

/// The weighted median of `(value, weight)` pairs (lower median); `None` when empty.
fn weighted_median(values: &mut [(f64, f64)]) -> Option<f64> {
    values.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    let half = values.iter().map(|v| v.1).sum::<f64>() / 2.0;
    let mut accumulated = 0.0;
    for &(value, weight) in values.iter() {
        accumulated += weight;
        if accumulated >= half {
            return Some(value);
        }
    }
    values.last().map(|v| v.0)
}

// ---------------------------------------------------------------------------------------
// Clustering
// ---------------------------------------------------------------------------------------

/// A cluster centre and the number of sampled pixels assigned to it.
#[derive(Debug, Clone, Copy)]
struct Cluster {
    lab: Lab,
    weight: f64,
}

fn palette_of(img: &Pixmap, region: Region, k: usize) -> Vec<Swatch> {
    if k == 0 {
        return Vec::new();
    }
    finalize(kmeans(&sample(img, region), k))
}

/// Picks an index with probability proportional to its weight. `total` must be the sum of
/// the weights; zero-weight entries are never picked unless every weight is zero.
fn pick_weighted(weights: impl Iterator<Item = f64>, total: f64, rng: &mut SplitMix64) -> usize {
    let target = rng.next_f64() * total;
    let mut accumulated = 0.0;
    let mut last_positive = 0;
    for (i, w) in weights.enumerate() {
        if w > 0.0 {
            accumulated += w;
            last_positive = i;
            if target < accumulated {
                return i;
            }
        }
    }
    last_positive
}

/// Weighted k-means in CIELAB: greedy k-means++ seeding, then Lloyd's algorithm.
fn kmeans(samples: &[Sample], k: usize) -> Vec<Cluster> {
    let k = k.min(samples.len()).min(MAX_PALETTE_SIZE);
    if k == 0 {
        return Vec::new();
    }
    if k == samples.len() {
        // No more distinct colours than clusters: every colour is its own cluster.
        return samples.iter().map(|s| Cluster { lab: s.lab, weight: s.weight }).collect();
    }

    // Seeding. The first centre is drawn by weight, every further one with probability
    // proportional to weight × squared distance to the nearest centre so far ("D²
    // sampling"); among a few such draws the one that lowers the total cost most is kept.
    let mut rng = SplitMix64(SEED);
    let total: f64 = samples.iter().map(|s| s.weight).sum();
    let first = pick_weighted(samples.iter().map(|s| s.weight), total, &mut rng);
    let mut centres: Vec<Lab> = Vec::with_capacity(k);
    centres.extend(samples.get(first).map(|s| s.lab));
    let mut nearest: Vec<f64> = match centres.first() {
        Some(&c) => samples.iter().map(|s| s.lab.distance_squared(c)).collect(),
        None => return Vec::new(),
    };
    let trials = 2 + k.ilog2() as usize;
    while centres.len() < k {
        let cost: f64 = samples.iter().zip(&nearest).map(|(s, d)| s.weight * d).sum();
        if cost.is_nan() || cost <= 0.0 {
            break; // every sample already coincides with a centre
        }
        let mut best: Option<(f64, Lab)> = None;
        for _ in 0..trials {
            let index = pick_weighted(
                samples.iter().zip(&nearest).map(|(s, d)| s.weight * d),
                cost,
                &mut rng,
            );
            let Some(candidate) = samples.get(index).map(|s| s.lab) else { continue };
            let new_cost: f64 = samples
                .iter()
                .zip(&nearest)
                .map(|(s, d)| s.weight * d.min(s.lab.distance_squared(candidate)))
                .sum();
            if best.is_none_or(|(c, _)| new_cost < c) {
                best = Some((new_cost, candidate));
            }
        }
        let Some((_, centre)) = best else { break };
        for (s, d) in samples.iter().zip(nearest.iter_mut()) {
            *d = d.min(s.lab.distance_squared(centre));
        }
        centres.push(centre);
    }

    // Lloyd iterations: assign every colour to its nearest centre (lowest index on ties),
    // move every centre to the weighted mean of its colours, until nothing changes.
    let mut assignment = vec![usize::MAX; samples.len()];
    let mut sums = vec![[0.0_f64; 4]; centres.len()];
    for _ in 0..MAX_ITERATIONS {
        let mut changed = false;
        for (s, slot) in samples.iter().zip(assignment.iter_mut()) {
            let mut best = 0;
            let mut best_distance = f64::INFINITY;
            for (c, centre) in centres.iter().enumerate() {
                let d = s.lab.distance_squared(*centre);
                if d < best_distance {
                    best_distance = d;
                    best = c;
                }
            }
            if *slot != best {
                *slot = best;
                changed = true;
            }
        }
        sums.iter_mut().for_each(|sum| *sum = [0.0; 4]);
        for (s, &a) in samples.iter().zip(&assignment) {
            if let Some(sum) = sums.get_mut(a) {
                sum[0] += s.weight * s.lab.l;
                sum[1] += s.weight * s.lab.a;
                sum[2] += s.weight * s.lab.b;
                sum[3] += s.weight;
            }
        }
        if !changed {
            break;
        }
        for (centre, sum) in centres.iter_mut().zip(&sums) {
            // An emptied cluster keeps its centre; it is dropped later if it stays empty.
            if sum[3] > 0.0 {
                *centre = Lab::new(sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3]);
            }
        }
    }

    centres.iter().zip(&sums).map(|(&lab, sum)| Cluster { lab, weight: sum[3] }).collect()
}

/// Drops empty clusters, merges near-duplicates, converts to sRGB and orders the result.
fn finalize(mut clusters: Vec<Cluster>) -> Vec<Swatch> {
    clusters.retain(|c| c.weight > 0.0);
    // Heaviest first so that small clusters merge into large ones; the Lab tie-break
    // makes the order total.
    clusters.sort_by(|x, y| {
        y.weight
            .total_cmp(&x.weight)
            .then(x.lab.l.total_cmp(&y.lab.l))
            .then(x.lab.a.total_cmp(&y.lab.a))
            .then(x.lab.b.total_cmp(&y.lab.b))
    });
    let threshold = PALETTE_MERGE_DELTA_E * PALETTE_MERGE_DELTA_E;
    let mut kept: Vec<Cluster> = Vec::with_capacity(clusters.len());
    for cluster in clusters {
        let nearest = kept
            .iter_mut()
            .map(|k| (k.lab.distance_squared(cluster.lab), k))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        match nearest {
            Some((distance, target)) if distance < threshold => {
                let weight = target.weight + cluster.weight;
                let blend = |a: f64, b: f64| (a * target.weight + b * cluster.weight) / weight;
                target.lab = Lab::new(
                    blend(target.lab.l, cluster.lab.l),
                    blend(target.lab.a, cluster.lab.a),
                    blend(target.lab.b, cluster.lab.b),
                );
                target.weight = weight;
            }
            _ => kept.push(cluster),
        }
    }

    let total: f64 = kept.iter().map(|c| c.weight).sum();
    let mut swatches: Vec<Swatch> = kept
        .iter()
        .map(|c| Swatch { color: c.lab.to_rgb8(), share: (c.weight / total) as f32 })
        .collect();
    swatches.sort_by(|x, y| y.share.total_cmp(&x.share).then(x.color.cmp(&y.color)));
    swatches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Pixmap {
        let mut data = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            for x in 0..w {
                data.extend_from_slice(&f(x, y));
            }
        }
        Pixmap::from_rgba8(w, h, data).expect("consistent test image")
    }

    #[test]
    fn sampling_step_keeps_the_sample_count_bounded_and_minimal() {
        let cells = |w: u32, h: u32, s: u32| u64::from(w.div_ceil(s)) * u64::from(h.div_ceil(s));
        for (w, h) in [
            (1, 1),
            (128, 128),
            (129, 128),
            (1000, 1500),
            (3000, 4500),
            (1, 1_000_000),
            (40, 3000),
            (65_535, 65_535),
            (u32::MAX, 1),
            (u32::MAX, u32::MAX),
        ] {
            let step = sampling_step(w, h);
            assert!(step >= 1);
            assert!(cells(w, h, step) <= MAX_SAMPLES, "{w}x{h}: step {step}");
            if step > 1 {
                assert!(cells(w, h, step - 1) > MAX_SAMPLES, "{w}x{h}: step {step} not minimal");
            }
        }
        assert_eq!(sampling_step(128, 128), 1, "exactly the budget: read every pixel");
        assert_eq!(sampling_step(129, 128), 2);
    }

    #[test]
    fn small_regions_are_read_exhaustively() {
        // 100 x 100 pixels, each with a distinct colour.
        let img = image(100, 100, |x, y| [x as u8, y as u8, 7, 255]);
        let samples = sample(&img, Region::whole(&img));
        assert_eq!(samples.len(), 10_000);
        assert!(samples.iter().all(|s| s.weight == 1.0));
    }

    #[test]
    fn large_regions_are_sampled_inside_every_cell_and_inside_the_region_only() {
        // Only the region (a 300-px strip on the right) is green; a sample leaking outside
        // would show up as red.
        let img =
            image(900, 600, |x, _| if x >= 600 { [0, 200, 0, 255] } else { [200, 0, 0, 255] });
        let region = Region::band(&img, Edge::Right, 300);
        let samples = sample(&img, region);
        assert_eq!(samples.len(), 1);
        let step = sampling_step(300, 600);
        assert!(step > 1);
        let expected = f64::from(300_u32.div_ceil(step) * 600_u32.div_ceil(step));
        assert_eq!(samples[0].weight, expected);
        assert_eq!(samples[0].lab, Lab::from_rgb8(Rgb8::new(0, 200, 0)));
    }

    #[test]
    fn regions_clamp_to_the_image() {
        let img = Pixmap::filled(10, 20, [0, 0, 0, 255]);
        assert_eq!(Region::strip(&img, Edge::Left, 0.3), Region { x: 0, y: 0, w: 3, h: 20 });
        assert_eq!(Region::strip(&img, Edge::Right, 0.3), Region { x: 7, y: 0, w: 3, h: 20 });
        assert_eq!(Region::strip(&img, Edge::Top, 0.25), Region { x: 0, y: 0, w: 10, h: 5 });
        assert_eq!(Region::strip(&img, Edge::Bottom, 0.25), Region { x: 0, y: 15, w: 10, h: 5 });
        for hostile in [f32::NAN, -1.0, 0.0, f32::NEG_INFINITY, 1e-9] {
            assert_eq!(
                Region::strip(&img, Edge::Right, hostile),
                Region { x: 9, y: 0, w: 1, h: 20 }
            );
        }
        for hostile in [1.0, 7.5, f32::INFINITY, f32::MAX] {
            assert_eq!(Region::strip(&img, Edge::Bottom, hostile), Region::whole(&img));
        }
        assert_eq!(Region::band(&img, Edge::Top, u32::MAX), Region::whole(&img));
        assert_eq!(Region::band(&img, Edge::Left, 0), Region { x: 0, y: 0, w: 1, h: 20 });
    }

    #[test]
    fn compositing_over_white() {
        assert_eq!(over_white([10, 20, 30, 255]), Rgb8::new(10, 20, 30));
        assert_eq!(over_white([10, 20, 30, 0]), Rgb8::WHITE);
        assert_eq!(over_white([0, 0, 0, 128]), Rgb8::new(127, 127, 127));
        assert_eq!(over_white([255, 255, 255, 77]), Rgb8::WHITE);
        assert_eq!(over_white([0, 255, 100, 51]), Rgb8::new(204, 255, 224));
    }

    #[test]
    fn weighted_median_respects_weights() {
        assert_eq!(weighted_median(&mut []), None);
        assert_eq!(weighted_median(&mut [(5.0, 1.0)]), Some(5.0));
        assert_eq!(weighted_median(&mut [(9.0, 1.0), (1.0, 1.0), (5.0, 1.0)]), Some(5.0));
        assert_eq!(weighted_median(&mut [(9.0, 10.0), (1.0, 1.0), (5.0, 1.0)]), Some(9.0));
        assert_eq!(weighted_median(&mut [(1.0, 1.0), (2.0, 1.0)]), Some(1.0), "lower median");
    }

    #[test]
    fn splitmix64_matches_the_reference_sequence() {
        // First outputs of SplitMix64 seeded with 1234567 (reference implementation).
        let mut rng = SplitMix64(1_234_567);
        assert_eq!(rng.next_u64(), 6_457_827_717_110_365_317);
        assert_eq!(rng.next_u64(), 3_203_168_211_198_807_973);
        assert_eq!(rng.next_u64(), 9_817_491_932_198_370_423);
        let mut rng = SplitMix64(SEED);
        for _ in 0..1_000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn pick_weighted_never_returns_a_zero_weight_entry() {
        let weights = [0.0, 3.0, 0.0, 1.0, 0.0];
        let mut rng = SplitMix64(42);
        let mut counts = [0u32; 5];
        for _ in 0..4_000 {
            counts[pick_weighted(weights.iter().copied(), 4.0, &mut rng)] += 1;
        }
        assert_eq!((counts[0], counts[2], counts[4]), (0, 0, 0));
        // 3:1 odds within a generous tolerance.
        assert!((2_800..3_200).contains(&counts[1]), "{counts:?}");
        // A total that overstates the weights (rounding) falls back to the last positive entry.
        let mut rng = SplitMix64(1);
        for _ in 0..100 {
            let i = pick_weighted(weights.iter().copied(), 1e9, &mut rng);
            assert!(i == 1 || i == 3);
        }
    }

    #[test]
    fn kmeans_separates_well_separated_blobs_exactly() {
        // Three tight groups of distinct colours; k-means must find exactly these groups.
        let groups: [(Rgb8, f64); 3] = [
            (Rgb8::new(200, 30, 30), 5.0),
            (Rgb8::new(30, 30, 200), 3.0),
            (Rgb8::new(240, 240, 240), 2.0),
        ];
        let mut samples = Vec::new();
        for (base, weight) in groups {
            for d in 0..4u8 {
                let c = Rgb8::new(base.r + d, base.g + d, base.b);
                samples.push(Sample { lab: Lab::from_rgb8(c), weight });
            }
        }
        let mut clusters = kmeans(&samples, 3);
        clusters.sort_by(|a, b| b.weight.total_cmp(&a.weight));
        assert_eq!(clusters.iter().map(|c| c.weight).collect::<Vec<_>>(), vec![20.0, 12.0, 8.0]);
        for (cluster, (base, _)) in clusters.iter().zip(groups) {
            assert!(cluster.lab.delta_e76(Lab::from_rgb8(base)) < 1.5, "{cluster:?}");
        }
    }

    #[test]
    fn kmeans_handles_degenerate_sizes() {
        assert!(kmeans(&[], 3).is_empty());
        let one = [Sample { lab: Lab::new(50.0, 0.0, 0.0), weight: 4.0 }];
        assert!(kmeans(&one, 0).is_empty());
        let clusters = kmeans(&one, 5);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].weight, 4.0);
    }

    #[test]
    fn finalize_merges_near_duplicates_and_orders_by_share() {
        let clusters = vec![
            Cluster { lab: Lab::new(50.0, 0.0, 0.0), weight: 10.0 },
            Cluster { lab: Lab::new(51.0, 0.0, 0.0), weight: 30.0 }, // 1 ΔE from the first
            Cluster { lab: Lab::new(80.0, 0.0, 0.0), weight: 50.0 },
            Cluster { lab: Lab::new(20.0, 0.0, 0.0), weight: 0.0 }, // empty: dropped
            // 2.75 from the centre the first two merge into (50.75).
            Cluster { lab: Lab::new(53.5, 0.0, 0.0), weight: 10.0 },
        ];
        let swatches = finalize(clusters);
        assert_eq!(swatches.len(), 2);
        assert_eq!(swatches[0].share, 0.5);
        assert_eq!(swatches[1].share, 0.5);
        // Equal shares: ordered by colour (darker grey first).
        assert!(swatches[0].color < swatches[1].color);
        // The merged centre is the weighted mean of its members: (10·50 + 30·51 + 10·53.5) / 50.
        let merged = swatches[0].color.to_lab().l;
        assert!((merged - 51.3).abs() < 0.3, "{merged}");
        assert!(finalize(Vec::new()).is_empty());
    }
}
