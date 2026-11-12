//! The render-and-compare stage: the known text is typeset with each candidate font,
//! fitted to the lettering on the cover and compared pixel by pixel.

use ebc_scene::Pixmap;
use ebc_text::{Font, ShapeOptions, try_rasterize};

use crate::descriptor::remap_coverage;
use crate::detect::TextRegion;
use crate::gray::Gray;
use crate::identify::{Identification, Match, Prepared, sort_matches};
use crate::index::{CharTable, FaceDescriptor, IndexEntry};
use crate::label::{CaseUsed, apply_case};

/// Height both images are normalised to before comparing, in pixels.
const COMPARE_H: usize = 48;
/// Largest width of the normalised images, in pixels.
const COMPARE_MAX_W: usize = 1024;
/// Resolution of the measuring render, in pixels per em.
const PROBE_PX_PER_EM: f32 = 48.0;
/// Range of letter-spacings that are fitted, in em.
const SPACING_RANGE: (f32, f32) = (-0.06, 0.5);
/// Letter-spacings smaller than this are treated as none (keeps ligatures and kerning).
const SPACING_DEAD_ZONE: f32 = 0.012;
/// Edge distances are truncated at this many pixels (at `COMPARE_H`).
const CHAMFER_TRUNCATE: f32 = 6.0;
/// Mean edge distance at which the edge similarity is 0.5, in pixels (at `COMPARE_H`).
const CHAMFER_HALF: f32 = 1.6;
/// Weight of the render similarity in the final score (the rest is the descriptor score).
const RENDER_WEIGHT: f32 = 0.5;
/// Most candidates that are rendered (the rest keep their descriptor score, scaled).
const MAX_RENDERED: usize = 64;

/// Re-ranks candidate fonts by rendering `known_text` with each of them and comparing
/// the result with the lettering in `region`.
///
/// For every candidate the text is rasterised with [`ebc_text::try_rasterize`], scaled
/// to the height of the lettering, letter-spaced to its width (a search over the fitted
/// spacing and none) and scored by normalised cross-correlation and a symmetric
/// truncated edge (chamfer) distance; the result is averaged with the descriptor score
/// of [`identify`](crate::identify). Candidates are typically the best 5–20 matches of
/// `identify` with their font files; an [`IndexEntry`] for a font outside any index
/// comes from [`IndexEntry::from_font`].
///
/// Returns the candidates sorted by decreasing score; an empty list when the region
/// holds no text. A candidate that cannot draw the text keeps half its descriptor score.
#[must_use]
pub fn rerank(
    img: &Pixmap,
    region: &TextRegion,
    known_text: &str,
    candidates: &[(IndexEntry, Font)],
) -> Vec<Match> {
    rerank_detailed(img, region, known_text, candidates).matches
}

/// Like [`rerank`], with the confidence signal of
/// [`identify_detailed`](crate::identify_detailed).
#[must_use]
pub fn rerank_detailed(
    img: &Pixmap,
    region: &TextRegion,
    known_text: &str,
    candidates: &[(IndexEntry, Font)],
) -> Identification {
    if candidates.is_empty() {
        return Identification::empty();
    }
    let table = CharTable::reference();
    let centroids =
        FaceDescriptor::centroid(&table, candidates.iter().map(|(e, _)| &*e.descriptor));
    let Some(prepared) = Prepared::new(img, region, Some(known_text), &centroids) else {
        return Identification::empty();
    };
    let scored = prepared.score(candidates.iter().map(|(e, _)| &*e.descriptor), &centroids);
    let mut out = prepared.identification(scored.labelled);
    // The candidates in decreasing order of their descriptor score.
    let descriptor = scored.scores;
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| descriptor[b].total_cmp(&descriptor[a]));

    // The lines to compare: (query image, the texts it may show).
    let mut jobs: Vec<(Gray, Vec<String>)> = Vec::new();
    if scored.labelled {
        for (i, line) in prepared.analysis.lines.iter().enumerate() {
            // The same line under every case hypothesis that survived.
            let mut texts: Vec<String> = Vec::new();
            for l in &prepared.labellings {
                if let Some(Some(text)) = l.line_texts.get(i)
                    && !texts.contains(text)
                {
                    texts.push(text.clone());
                }
            }
            let Some(first) = texts.first() else { continue };
            if let Some(query) = line_image(&prepared, line, first) {
                jobs.push((query, texts));
            }
        }
    } else if let [line] = prepared.analysis.lines.as_slice() {
        // Without labels only a single line can be compared as a whole; the case is
        // unknown, so both likely cases are tried.
        let mut texts = vec![known_text.trim().to_string()];
        let upper = apply_case(known_text.trim(), CaseUsed::Upper);
        if !texts.contains(&upper) {
            texts.push(upper);
        }
        if let Some(query) = line_image(&prepared, line, &texts[0]) {
            jobs.push((query, texts));
        }
    }

    let mut matches = Vec::with_capacity(candidates.len());
    for (position, &ci) in order.iter().enumerate() {
        let (entry, font) = &candidates[ci];
        let score = if jobs.is_empty() {
            descriptor[ci]
        } else {
            let render = if position < MAX_RENDERED {
                let (mut sum, mut weight) = (0.0f32, 0.0f32);
                for (query, texts) in &jobs {
                    let w = query.w as f32;
                    let best = texts
                        .iter()
                        .filter_map(|t| compare_line(query, font, t))
                        .fold(0.0f32, f32::max);
                    sum += best * w;
                    weight += w;
                }
                if weight > 0.0 { sum / weight } else { 0.0 }
            } else {
                0.0
            };
            (1.0 - RENDER_WEIGHT) * descriptor[ci] + RENDER_WEIGHT * render
        };
        matches.push(Match { entry: entry.clone(), score: score.clamp(0.0, 1.0) });
    }
    sort_matches(&mut matches);
    out.matches = matches;
    out.settle();
    out
}

/// The ink of one line, cut out of the analysed region and normalised to `COMPARE_H`.
fn line_image(prepared: &Prepared, line: &crate::segment::Line, text: &str) -> Option<Gray> {
    // Small components are kept only when the text has punctuation: otherwise they are
    // specks that would spoil the bounding box.
    let has_small = text.chars().any(|c| !c.is_whitespace() && !c.is_alphanumeric());
    let parts: Vec<u32> = line
        .glyphs
        .iter()
        .filter(|g| g.tall || has_small)
        .flat_map(|g| g.parts.iter().copied())
        .collect();
    let (crop, _) = prepared.analysis.crop_parts(&parts)?;
    if crop.w < 4 || crop.h < 4 {
        return None;
    }
    let scale = (COMPARE_H as f32 / crop.h as f32).min(COMPARE_MAX_W as f32 / crop.w as f32);
    let w = ((crop.w as f32 * scale).round() as usize).clamp(4, COMPARE_MAX_W);
    let h = ((crop.h as f32 * scale).round() as usize).clamp(4, COMPARE_H);
    Some(crop.resize(w, h))
}

/// Renders `text` with `font`, fits it to `query` and returns their similarity in
/// `0..=1`. `None` when the font cannot draw the text.
fn compare_line(query: &Gray, font: &Font, text: &str) -> Option<f32> {
    if !font.missing_chars(text).is_empty() {
        return None;
    }
    let plain = ShapeOptions::default();
    let probe = try_rasterize(font, text, PROBE_PX_PER_EM, &plain).ok()?;
    let (pw, ph) = (probe.mask.width as f32, probe.mask.height as f32);
    if pw < 1.0 || ph < 1.0 {
        return None;
    }
    // Pixels per em at which the render is as tall as the query.
    let px_per_em = (PROBE_PX_PER_EM * query.h as f32 / ph).clamp(4.0, 512.0);
    let natural_w = pw * px_per_em / PROBE_PX_PER_EM;
    let gaps = text.chars().count().saturating_sub(1).max(1) as f32;
    let fitted = (query.w as f32 - natural_w) / (gaps * px_per_em);

    let mut spacings = vec![0.0f32];
    if fitted.abs() >= SPACING_DEAD_ZONE {
        spacings.push(fitted.clamp(SPACING_RANGE.0, SPACING_RANGE.1));
    }
    let mut best: Option<f32> = None;
    for spacing in spacings {
        let opts = ShapeOptions { letter_spacing_em: spacing, ..ShapeOptions::default() };
        let Ok(render) = try_rasterize(font, text, px_per_em, &opts) else { continue };
        let (w, h) = (render.mask.width as usize, render.mask.height as usize);
        if w == 0 || h == 0 || render.mask.data.len() < w * h {
            continue;
        }
        let px = render.mask.data[..w * h].iter().map(|&v| remap_coverage(v)).collect();
        let rendered = Gray { w, h, px };
        // What the spacing could not fit is stretched away — and charged for.
        let stretch = (w as f32 / query.w as f32).max(query.w as f32 / w as f32) - 1.0;
        let fitted_render = rendered.resize(query.w, query.h);
        let similarity = similarity(query, &fitted_render) * (1.0 - (2.0 * stretch).min(0.6));
        if best.is_none_or(|b| similarity > b) {
            best = Some(similarity);
        }
    }
    best
}

/// Similarity of two coverage images of the same size in `0..=1`: the mean of their
/// normalised cross-correlation (after a light blur) and an edge similarity from the
/// symmetric truncated chamfer distance.
fn similarity(a: &Gray, b: &Gray) -> f32 {
    if a.w != b.w || a.h != b.h || a.px.is_empty() {
        return 0.0;
    }
    let ncc = ncc(&a.blurred(1.0), &b.blurred(1.0)).max(0.0);
    let chamfer = match chamfer(a, b) {
        Some(d) => 1.0 / (1.0 + (d / CHAMFER_HALF) * (d / CHAMFER_HALF)),
        None => 0.0,
    };
    0.5 * ncc + 0.5 * chamfer
}

/// Zero-mean normalised cross-correlation in `-1..=1` (0 for a flat image).
fn ncc(a: &Gray, b: &Gray) -> f32 {
    let n = a.px.len().min(b.px.len());
    if n == 0 {
        return 0.0;
    }
    let (mut sa, mut sb) = (0.0f64, 0.0f64);
    for (x, y) in a.px.iter().zip(&b.px) {
        sa += f64::from(*x);
        sb += f64::from(*y);
    }
    let (ma, mb) = (sa / n as f64, sb / n as f64);
    let (mut cov, mut va, mut vb) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in a.px.iter().zip(&b.px) {
        let (dx, dy) = (f64::from(*x) - ma, f64::from(*y) - mb);
        cov += dx * dy;
        va += dx * dx;
        vb += dy * dy;
    }
    if va <= 1e-9 || vb <= 1e-9 {
        return 0.0;
    }
    (cov / (va * vb).sqrt()) as f32
}

/// Symmetric mean edge-to-edge distance in pixels, truncated; `None` when one image has
/// no edges.
fn chamfer(a: &Gray, b: &Gray) -> Option<f32> {
    let (ea, eb) = (edges(a), edges(b));
    let (da, db) = (distance_transform(&ea, a.w, a.h), distance_transform(&eb, b.w, b.h));
    let one_way = |edge: &[bool], dist: &[f32]| -> Option<f32> {
        let (mut sum, mut n) = (0.0f32, 0u32);
        for (e, d) in edge.iter().zip(dist) {
            if *e {
                sum += d.min(CHAMFER_TRUNCATE);
                n += 1;
            }
        }
        (n > 0).then(|| sum / n as f32)
    };
    Some(0.5 * (one_way(&ea, &db)? + one_way(&eb, &da)?))
}

/// Ink pixels (coverage at least one half) with a non-ink 4-neighbour or on the border.
fn edges(g: &Gray) -> Vec<bool> {
    let (w, h) = (g.w, g.h);
    let ink = |x: usize, y: usize| g.px[y * w + x] >= 128;
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            if !ink(x, y) {
                continue;
            }
            let border = x == 0 || y == 0 || x + 1 == w || y + 1 == h;
            out[y * w + x] =
                border || !ink(x - 1, y) || !ink(x + 1, y) || !ink(x, y - 1) || !ink(x, y + 1);
        }
    }
    out
}

/// Two-pass 3-4 chamfer distance transform: approximate Euclidean distance of every
/// pixel to the nearest set pixel, in pixels.
fn distance_transform(set: &[bool], w: usize, h: usize) -> Vec<f32> {
    const FAR: u32 = u32::MAX / 2;
    let mut d: Vec<u32> = set.iter().map(|&s| if s { 0 } else { FAR }).collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut v = d[i];
            if x > 0 {
                v = v.min(d[i - 1] + 3);
            }
            if y > 0 {
                v = v.min(d[i - w] + 3);
                if x > 0 {
                    v = v.min(d[i - w - 1] + 4);
                }
                if x + 1 < w {
                    v = v.min(d[i - w + 1] + 4);
                }
            }
            d[i] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut v = d[i];
            if x + 1 < w {
                v = v.min(d[i + 1] + 3);
            }
            if y + 1 < h {
                v = v.min(d[i + w] + 3);
                if x + 1 < w {
                    v = v.min(d[i + w + 1] + 4);
                }
                if x > 0 {
                    v = v.min(d[i + w - 1] + 4);
                }
            }
            d[i] = v;
        }
    }
    d.iter().map(|&v| v.min(FAR) as f32 / 3.0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(size: usize, x0: usize, y0: usize, side: usize) -> Gray {
        let mut g = Gray::filled(size, size, 0);
        for y in y0..y0 + side {
            for x in x0..x0 + side {
                g.px[y * size + x] = 255;
            }
        }
        g
    }

    #[test]
    fn distance_transform_measures_steps() {
        let mut set = vec![false; 7 * 5];
        set[2 * 7 + 3] = true;
        let d = distance_transform(&set, 7, 5);
        assert_eq!(d[2 * 7 + 3], 0.0);
        assert_eq!(d[2 * 7 + 6], 3.0);
        assert!((d[0] - (2.0 * 4.0 + 3.0) / 3.0).abs() < 1e-6);
        // No set pixel: everything is far, nothing overflows.
        assert!(distance_transform(&[false; 12], 4, 3).iter().all(|&v| v > 1e6));
    }

    #[test]
    fn identical_images_are_fully_similar_and_shifted_ones_less() {
        let a = square(40, 10, 10, 16);
        assert!(similarity(&a, &a) > 0.99);
        let near = similarity(&a, &square(40, 11, 10, 16));
        let far = similarity(&a, &square(40, 18, 14, 16));
        assert!(near < 0.99 && near > far, "near {near}, far {far}");
        assert!(far < 0.5, "{far}");
        // Mismatched sizes and blank images do not compare.
        assert_eq!(similarity(&a, &square(41, 10, 10, 16)), 0.0);
        let blank = Gray::filled(40, 40, 0);
        assert_eq!(similarity(&a, &blank), 0.0);
        assert_eq!(chamfer(&blank, &a), None);
        assert_eq!(ncc(&blank, &blank), 0.0);
    }

    #[test]
    fn no_candidates_or_no_text_yield_nothing() {
        let img = Pixmap::filled(60, 40, [255, 255, 255, 255]);
        let region = TextRegion { x: 0, y: 0, w: 60, h: 40, score: 1.0 };
        assert!(rerank(&img, &region, "Title", &[]).is_empty());
    }
}
