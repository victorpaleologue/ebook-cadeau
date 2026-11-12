//! The descriptor stage: comparing the glyphs of a region with the fonts of an index.

use ebc_scene::Pixmap;

use crate::cc::median;
use crate::descriptor::{
    FEATURE_LEN, Features, HOG_LEN, S_ASPECT, S_CONTRAST, S_DENSITY, S_SLANT, S_STROKE,
    hog_distance,
};
use crate::detect::TextRegion;
use crate::index::{FaceDescriptor, FontIndex, IndexEntry, record_geometry};
pub use crate::label::CaseUsed;
use crate::label::{Label, Labelling, Observed, label, observe};
use crate::segment::{Analysis, analyse};

/// Weight of the aspect-ratio difference in a glyph distance.
const W_ASPECT: f32 = 1.2;
/// Weight of the ink-density difference.
const W_DENSITY: f32 = 0.6;
/// Weight of the stroke-width difference.
const W_STROKE: f32 = 1.2;
/// Weight of the stroke-contrast difference.
const W_CONTRAST: f32 = 0.3;
/// Weight of the slant difference.
const W_SLANT: f32 = 1.0;
/// Weight of the line-geometry residual (relative heights, widths, descender depths).
const W_GEOMETRY: f32 = 1.0;
/// Distance charged for a glyph whose character the font does not have.
const MISSING_GLYPH: f32 = 1.2;
/// Share of the glyphs (the best-matching ones) that make up a font's distance.
const KEEP: f32 = 0.75;
/// Distance at which the score is 0.5 (see [`score_from_distance`]).
const HALF_SCORE_DISTANCE: f32 = 0.62;
/// Factor applied to scores obtained without labelled glyphs.
const UNLABELLED_FACTOR: f32 = 0.8;
/// Smallest best score for a confident identification.
pub(crate) const CONFIDENT_SCORE: f32 = 0.8;
/// Smallest lead over the best font of another family for a confident identification.
pub(crate) const CONFIDENT_FAMILY_MARGIN: f32 = 0.04;
/// Fewest labelled glyphs for a confident identification.
pub(crate) const CONFIDENT_GLYPHS: usize = 4;
/// Most glyphs compared in the unlabelled fallback.
const MAX_UNLABELLED: usize = 40;

/// A font of the index with its similarity to the lettering.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// The font.
    pub entry: IndexEntry,
    /// Similarity in `0..=1`; higher is closer. Scores of at least 0.8 mean "the glyphs
    /// look like this font's"; see `docs/research/fontid-eval.md` for the calibration.
    pub score: f32,
}

/// The full result of [`identify_detailed`] or [`rerank_detailed`](crate::rerank_detailed).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Identification {
    /// The fonts, best first. Empty when the region holds nothing that looks like text
    /// or the index is empty.
    pub matches: Vec<Match>,
    /// Number of glyphs that were matched with a character of the known text and
    /// compared character by character. 0 means the unlabelled fallback was used.
    pub labelled_glyphs: usize,
    /// Number of letter-sized glyphs found in the region.
    pub glyphs: usize,
    /// Number of text lines found in the region.
    pub lines: usize,
    /// The case in which the known text appears on the cover.
    pub case_used: CaseUsed,
    /// Fraction of the letters of the known text that were found in the region
    /// (0 without labels; below 1 when the region shows only part of the text).
    pub text_coverage: f32,
    /// `true` when the best match is trustworthy enough to typeset with: glyphs were
    /// labelled, the best score is at least 0.8 and it leads every other *family* by a
    /// margin. `false` means "offer the list, or fall back to a house face".
    pub confident: bool,
    /// Score lead of the best match over the second.
    pub margin: f32,
    /// Score lead of the best match over the best font of a different family.
    pub family_margin: f32,
    /// `true` when the lettering is lighter than its background.
    pub light_on_dark: bool,
    /// Rotation of the text that was removed before comparing, in degrees.
    pub skew_degrees: f32,
}

impl Identification {
    pub(crate) fn empty() -> Self {
        Self {
            matches: Vec::new(),
            labelled_glyphs: 0,
            glyphs: 0,
            lines: 0,
            case_used: CaseUsed::Unknown,
            text_coverage: 0.0,
            confident: false,
            margin: 0.0,
            family_margin: 0.0,
            light_on_dark: false,
            skew_degrees: 0.0,
        }
    }

    /// Fills `margin`, `family_margin` and `confident` from the sorted matches.
    pub(crate) fn settle(&mut self) {
        let Some(first) = self.matches.first() else {
            self.confident = false;
            return;
        };
        let second = self.matches.get(1).map_or(0.0, |m| m.score);
        let other_family = self
            .matches
            .iter()
            .find(|m| m.entry.family != first.entry.family)
            .map_or(0.0, |m| m.score);
        self.margin = first.score - second;
        self.family_margin = first.score - other_family;
        self.confident = self.labelled_glyphs >= CONFIDENT_GLYPHS
            && first.score >= CONFIDENT_SCORE
            && self.family_margin >= CONFIDENT_FAMILY_MARGIN;
    }
}

/// Identifies the font of the text in `region` of `img`: the fonts of `index`, sorted by
/// decreasing similarity (the fast descriptor stage).
///
/// With `known_text` (the title as the catalogue has it, in any case), the glyphs are
/// matched with its characters and compared character by character; without it, or when
/// the glyph count does not fit the text, every glyph is compared with its most similar
/// characters, which is less reliable (scores are scaled down by 0.8).
///
/// Never fails: a region outside the image, an empty region or one without text yields
/// an empty list. See [`identify_detailed`] for the confidence signal.
#[must_use]
pub fn identify(
    img: &Pixmap,
    region: &TextRegion,
    known_text: Option<&str>,
    index: &FontIndex,
) -> Vec<Match> {
    identify_detailed(img, region, known_text, index).matches
}

/// Like [`identify`], with everything known about the match: how many glyphs were
/// labelled, the case used on the cover and whether the result is confident.
#[must_use]
pub fn identify_detailed(
    img: &Pixmap,
    region: &TextRegion,
    known_text: Option<&str>,
    index: &FontIndex,
) -> Identification {
    if index.is_empty() {
        return Identification::empty();
    }
    let Some(prepared) = Prepared::new(img, region, known_text, index.centroids()) else {
        return Identification::empty();
    };
    let distances = prepared.distances(index.entries().iter().map(|e| &*e.descriptor), index.centroids());
    let mut out = prepared.identification();
    out.matches = rank(index.entries(), &distances, prepared.labelling.is_some());
    out.settle();
    out
}

/// A region, analysed and (if possible) labelled.
pub(crate) struct Prepared {
    pub(crate) analysis: Analysis,
    pub(crate) obs: Observed,
    pub(crate) labelling: Option<Labelling>,
}

impl Prepared {
    pub(crate) fn new(
        img: &Pixmap,
        region: &TextRegion,
        known_text: Option<&str>,
        centroids: &FaceDescriptor,
    ) -> Option<Self> {
        let analysis = analyse(img, region)?;
        let obs = observe(&analysis, true);
        if obs.glyphs.is_empty() {
            return None;
        }
        let labelling = known_text.and_then(|text| label(&obs, text, centroids));
        Some(Self { analysis, obs, labelling })
    }

    /// An [`Identification`] with everything but the matches.
    pub(crate) fn identification(&self) -> Identification {
        let mut out = Identification::empty();
        out.glyphs = self.analysis.tall_count();
        out.lines = self.analysis.lines.len();
        out.light_on_dark = self.analysis.light_on_dark;
        out.skew_degrees = self.analysis.skew_degrees;
        if let Some(l) = &self.labelling {
            out.labelled_glyphs = l.labels.len();
            out.case_used = l.case;
            out.text_coverage = l.coverage;
        }
        out
    }

    /// The distance of every face to the glyphs (`None`: not comparable).
    pub(crate) fn distances<'a>(
        &self,
        faces: impl Iterator<Item = &'a FaceDescriptor>,
        centroids: &FaceDescriptor,
    ) -> Vec<Option<f32>> {
        match &self.labelling {
            Some(l) => faces.map(|f| labelled_distance(&self.obs, &l.labels, f)).collect(),
            None => {
                let guesses = guess_characters(&self.obs, centroids);
                faces.map(|f| unlabelled_distance(&self.obs, &guesses, f)).collect()
            }
        }
    }
}

/// Turns distances into sorted matches.
pub(crate) fn rank(entries: &[IndexEntry], distances: &[Option<f32>], labelled: bool) -> Vec<Match> {
    let factor = if labelled { 1.0 } else { UNLABELLED_FACTOR };
    let mut matches: Vec<Match> = entries
        .iter()
        .zip(distances)
        .map(|(entry, d)| Match {
            entry: entry.clone(),
            score: d.map_or(0.0, |d| factor * score_from_distance(d)),
        })
        .collect();
    sort_matches(&mut matches);
    matches
}

/// Sorts by decreasing score; ties keep the index order.
pub(crate) fn sort_matches(matches: &mut [Match]) {
    matches.sort_by(|a, b| b.score.total_cmp(&a.score));
}

/// Maps a distance (0 = identical) to a score in `0..=1`: `1 / (1 + (d / d_half)^3)`.
/// The cubic keeps small distances near 1 and separates the middle range, where the
/// decision between "same font" and "another family" falls.
#[inline]
pub(crate) fn score_from_distance(d: f32) -> f32 {
    if !d.is_finite() {
        return 0.0;
    }
    let r = (d.max(0.0) / HALF_SCORE_DISTANCE).powi(3);
    (1.0 / (1.0 + r)).clamp(0.0, 1.0)
}

/// Distance between the features of a glyph and a reference record.
#[inline]
fn glyph_distance(f: &Features, record: &[u8]) -> f32 {
    let hog = hog_distance(f.hog(), &record[..HOG_LEN]);
    let (s, r) = (f.scalars(), &record[HOG_LEN..FEATURE_LEN]);
    let d = |k: usize| f32::from(s[k].abs_diff(r[k])) / 255.0;
    hog + W_ASPECT * d(S_ASPECT)
        + W_DENSITY * d(S_DENSITY)
        + W_STROKE * d(S_STROKE)
        + W_CONTRAST * d(S_CONTRAST)
        + W_SLANT * d(S_SLANT)
}

/// Mean of the smallest `KEEP` share of the values (at least one).
fn trimmed_mean(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    let keep = ((values.len() as f32 * KEEP).ceil() as usize).clamp(1, values.len());
    values[..keep].iter().sum::<f32>() / keep as f32
}

/// Distance of a face to labelled glyphs: the trimmed mean of the per-glyph distances
/// plus a residual of the line geometry — once one scale (pixels per em) is fitted per
/// line, how far the glyph heights, widths and bottoms are from what the face predicts.
fn labelled_distance(obs: &Observed, labels: &[Label], face: &FaceDescriptor) -> Option<f32> {
    let mut distances: Vec<f32> = Vec::with_capacity(labels.len());
    // (line, height px, width px, bottom px, height em, width em, bottom em)
    let mut geometry: Vec<(usize, [f32; 6])> = Vec::with_capacity(labels.len());
    let mut compared = 0usize;
    for l in labels {
        let Some(o) = obs.glyphs.get(l.obs) else { continue };
        let (Some(f), Some(record)) = (o.features(l.whole), face.record(l.ch)) else {
            distances.push(MISSING_GLYPH);
            continue;
        };
        compared += 1;
        distances.push(glyph_distance(f, record));
        let b = o.bbox(l.whole);
        let (h_em, w_em, bottom_em) = record_geometry(record);
        if h_em > 0.02 && w_em > 0.02 {
            geometry.push((o.line, [b[3] - b[1], b[2] - b[0], b[3], h_em, w_em, bottom_em]));
        }
    }
    if compared == 0 {
        return None;
    }
    let shape = trimmed_mean(&mut distances);

    let (mut residual, mut weight) = (0.0f32, 0.0f32);
    let n_lines = obs.lines.len();
    for line in 0..n_lines {
        let members: Vec<&[f32; 6]> =
            geometry.iter().filter(|(l, _)| *l == line).map(|(_, g)| g).collect();
        if members.len() < 2 {
            continue;
        }
        let mut ratios: Vec<f32> = members.iter().map(|g| g[0] / g[3]).collect();
        let scale = median(&mut ratios).max(1e-3);
        let mut baselines: Vec<f32> = members.iter().map(|g| g[2] - scale * g[5]).collect();
        let baseline = median(&mut baselines);
        let mut sum = 0.0f32;
        for g in &members {
            let dh = (g[0] / (scale * g[3]) - 1.0).abs().min(0.5);
            let dw = (g[1] / (scale * g[4]) - 1.0).abs().min(0.5);
            let db = ((g[2] - scale * g[5] - baseline).abs() / (scale * 0.7)).min(0.5);
            sum += dh + 0.5 * dw + db;
        }
        residual += sum;
        weight += members.len() as f32;
    }
    let geometry_term = if weight > 0.0 { residual / weight } else { 0.0 };
    Some(shape + W_GEOMETRY * geometry_term)
}

/// For every glyph (a bounded sample), the three characters whose average shape over
/// the index is nearest.
fn guess_characters(obs: &Observed, centroids: &FaceDescriptor) -> Vec<(usize, [Option<char>; 3])> {
    let n = obs.glyphs.len();
    let step = n.div_ceil(MAX_UNLABELLED).max(1);
    let mut out = Vec::new();
    for (i, o) in obs.glyphs.iter().enumerate().step_by(step) {
        let Some(f) = o.whole.as_ref() else { continue };
        let mut best: [(f32, Option<char>); 3] = [(f32::INFINITY, None); 3];
        for &ch in centroids.chars() {
            let Some(record) = centroids.record(ch) else { continue };
            let d = glyph_distance(f, record);
            if d < best[2].0 {
                best[2] = (d, Some(ch));
                best.sort_by(|a, b| a.0.total_cmp(&b.0));
            }
        }
        out.push((i, [best[0].1, best[1].1, best[2].1]));
    }
    out
}

/// Distance of a face to unlabelled glyphs: per glyph the nearest of its guessed
/// characters, then the trimmed mean.
fn unlabelled_distance(
    obs: &Observed,
    guesses: &[(usize, [Option<char>; 3])],
    face: &FaceDescriptor,
) -> Option<f32> {
    let mut distances = Vec::with_capacity(guesses.len());
    let mut compared = 0usize;
    for (i, chars) in guesses {
        let Some(f) = obs.glyphs.get(*i).and_then(|o| o.whole.as_ref()) else { continue };
        let nearest = chars
            .iter()
            .flatten()
            .filter_map(|&ch| face.record(ch))
            .map(|record| glyph_distance(f, record))
            .fold(f32::INFINITY, f32::min);
        if nearest.is_finite() {
            compared += 1;
            distances.push(nearest);
        } else {
            distances.push(MISSING_GLYPH);
        }
    }
    (compared > 0).then(|| trimmed_mean(&mut distances))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_mapping_is_monotone_and_bounded() {
        assert_eq!(score_from_distance(0.0), 1.0);
        assert!((score_from_distance(HALF_SCORE_DISTANCE) - 0.5).abs() < 1e-6);
        assert_eq!(score_from_distance(f32::NAN), 0.0);
        assert_eq!(score_from_distance(f32::INFINITY), 0.0);
        assert_eq!(score_from_distance(-3.0), 1.0);
        let mut last = 1.0;
        for i in 1..200 {
            let s = score_from_distance(i as f32 * 0.02);
            assert!((0.0..=1.0).contains(&s) && s <= last);
            last = s;
        }
    }

    #[test]
    fn trimmed_mean_drops_the_worst_quarter() {
        assert_eq!(trimmed_mean(&mut []), 0.0);
        assert_eq!(trimmed_mean(&mut [0.4]), 0.4);
        let mut v = [0.1, 0.1, 0.1, 9.0];
        assert!((trimmed_mean(&mut v) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn nothing_to_identify_yields_an_empty_result() {
        let img = Pixmap::filled(50, 40, [200, 200, 200, 255]);
        let region = TextRegion { x: 0, y: 0, w: 50, h: 40, score: 1.0 };
        let r = identify_detailed(&img, &region, Some("Title"), &FontIndex::default());
        assert!(r.matches.is_empty() && !r.confident);
        assert_eq!(r.case_used, CaseUsed::Unknown);
    }
}
