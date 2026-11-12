//! Assigning the characters of the known text to the glyphs of an analysed region.
//!
//! The segmentation yields lines of letter-sized glyphs; the known title yields words of
//! letters. A dynamic programme distributes consecutive words over the lines so that
//! the letter counts agree (exactly, or nearly — a second dynamic programme then finds
//! the touching or broken glyphs). The cover may show the title in another case than
//! the catalogue: the case is the hypothesis whose letters look most like the glyphs.

use std::ops::Range;

use crate::descriptor::{Features, S_ASPECT, describe, hog_distance};
use crate::index::{FaceDescriptor, record_geometry};
use crate::segment::Analysis;

/// How the known text is cased on the cover, as inferred from the glyph shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CaseUsed {
    /// As the text was given.
    AsGiven,
    /// ALL CAPITALS.
    Upper,
    /// all lower case.
    Lower,
    /// First Letter Of Every Word Capitalised.
    Title,
    /// The glyphs could not be matched with the text (or no text was given).
    Unknown,
}

/// Largest number of letter-sized glyphs of a region that are described.
const MAX_OBSERVED: usize = 160;
/// Largest number of lines a labelled region may have.
const MAX_LINES: usize = 8;
/// Largest number of words of the known text that are considered.
const MAX_WORDS: usize = 48;
/// Cost of a letter of the text that no line accounts for.
const UNCOVERED_COST: f32 = 0.15;
/// A labelling whose letters resemble the glyphs less than this (mean distance to the
/// average shape of each letter over the index) is rejected. Measured on the bundled
/// fonts: right labels 0.35–0.75, wrong labels above 1.0.
const PLAUSIBLE_MAX: f32 = 0.92;

/// Hypotheses whose plausibility is worse than the best by more than this are dropped.
const PLAUSIBLE_SPREAD: f32 = 0.3;

/// One letter-sized glyph of a region, described.
#[derive(Debug, Clone)]
pub(crate) struct GlyphObs {
    /// Index of its line in the analysis.
    pub(crate) line: usize,
    /// Features of the whole glyph (base and detached marks).
    pub(crate) whole: Option<Features>,
    /// Features of the base alone (equal to `whole` for a one-part glyph).
    pub(crate) main: Option<Features>,
    /// Box of the whole glyph: x0, y0, x1, y1 in pixels of the analysed crop.
    pub(crate) whole_box: [f32; 4],
    /// Box of the base alone.
    pub(crate) main_box: [f32; 4],
}

impl GlyphObs {
    pub(crate) fn features(&self, whole: bool) -> Option<&Features> {
        if whole { self.whole.as_ref() } else { self.main.as_ref() }
    }

    pub(crate) fn bbox(&self, whole: bool) -> &[f32; 4] {
        if whole { &self.whole_box } else { &self.main_box }
    }
}

/// The letter-sized glyphs of a region, line by line, left to right.
#[derive(Debug, Clone, Default)]
pub(crate) struct Observed {
    pub(crate) glyphs: Vec<GlyphObs>,
    /// Per line of the analysis: its glyphs in `glyphs`.
    pub(crate) lines: Vec<Range<usize>>,
}

/// Collects the letter-sized glyphs of an analysed region, with their features when
/// `with_features` is set (boxes alone are enough to match letter counts).
pub(crate) fn observe(a: &Analysis, with_features: bool) -> Observed {
    let mut out = Observed::default();
    for (li, line) in a.lines.iter().enumerate() {
        let start = out.glyphs.len();
        for glyph in line.glyphs.iter().filter(|g| g.tall) {
            if out.glyphs.len() >= MAX_OBSERVED {
                break;
            }
            let Some(main) = a.comps.get(glyph.main as usize) else { continue };
            let whole = if with_features {
                a.crop_parts(&glyph.parts).and_then(|(crop, _)| describe(&crop))
            } else {
                None
            };
            let main_features = if with_features && glyph.parts.len() > 1 {
                a.crop_parts(&[glyph.main]).and_then(|(crop, _)| describe(&crop))
            } else {
                whole.clone()
            };
            out.glyphs.push(GlyphObs {
                line: li,
                whole,
                main: main_features,
                whole_box: [glyph.x0 as f32, glyph.y0 as f32, glyph.x1 as f32, glyph.y1 as f32],
                main_box: [main.x0 as f32, main.y0 as f32, main.x1 as f32, main.y1 as f32],
            });
        }
        out.lines.push(start..out.glyphs.len());
    }
    out
}

/// A glyph with its character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Label {
    /// Index into [`Observed::glyphs`].
    pub(crate) obs: usize,
    /// The reference character to compare with (accents folded away).
    pub(crate) ch: char,
    /// Compare the whole glyph (`true`) or its base without detached marks (`false`).
    pub(crate) whole: bool,
}

/// The result of matching the known text with the glyphs.
#[derive(Debug, Clone)]
pub(crate) struct Labelling {
    pub(crate) case: CaseUsed,
    pub(crate) labels: Vec<Label>,
    /// Per line of the analysis: the part of the text set on it, in the case used.
    pub(crate) line_texts: Vec<Option<String>>,
    /// Fraction of the letters of the text that lie on the matched lines.
    pub(crate) coverage: f32,
}

/// One letter-sized character of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ch {
    /// Reference character to compare with, if there is a comparable one.
    base: Option<char>,
    /// Compare the whole glyph or the base without its accent.
    whole: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Word {
    text: String,
    chars: Vec<Ch>,
}

/// Does `ch` draw (at least) one letter-sized shape?
fn is_tall(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '&' | '!' | '?' | '@' | '#' | '$' | '%' | '€' | '£' | '§')
}

/// The reference character a letter is compared with.
fn fold(ch: char) -> Ch {
    if ch.is_ascii_alphanumeric() || matches!(ch, '&' | '!' | '?') {
        return Ch { base: Some(ch), whole: true };
    }
    // Letters with a detached mark above: compared through their base, mark cut off.
    // A lower-case i or j loses its dot under an accent, so it has no comparable base.
    let base = match ch {
        'À'..='Å' | 'Ā' | 'Ă' => Some('A'),
        'È'..='Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ě' => Some('E'),
        'Ì'..='Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'İ' => Some('I'),
        'Ò'..='Ö' | 'Ō' | 'Ŏ' | 'Ő' => Some('O'),
        'Ù'..='Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' => Some('U'),
        'Ý' | 'Ÿ' | 'Ŷ' => Some('Y'),
        'Ñ' | 'Ń' | 'Ň' => Some('N'),
        'Ć' | 'Ĉ' | 'Ċ' | 'Č' => Some('C'),
        'Ś' | 'Ŝ' | 'Š' => Some('S'),
        'Ź' | 'Ż' | 'Ž' => Some('Z'),
        'Ŕ' | 'Ř' => Some('R'),
        'Ĝ' | 'Ğ' | 'Ġ' => Some('G'),
        'Ď' => Some('D'),
        'Ť' => Some('T'),
        'Ŵ' => Some('W'),
        'à'..='å' | 'ā' | 'ă' => Some('a'),
        'è'..='ë' | 'ē' | 'ĕ' | 'ė' | 'ě' => Some('e'),
        'ò'..='ö' | 'ō' | 'ŏ' | 'ő' => Some('o'),
        'ù'..='ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' => Some('u'),
        'ý' | 'ÿ' | 'ŷ' => Some('y'),
        'ñ' | 'ń' | 'ň' => Some('n'),
        'ć' | 'ĉ' | 'ċ' | 'č' => Some('c'),
        'ś' | 'ŝ' | 'š' => Some('s'),
        'ź' | 'ż' | 'ž' => Some('z'),
        'ŕ' | 'ř' => Some('r'),
        'ĝ' | 'ğ' | 'ġ' => Some('g'),
        'ŵ' => Some('w'),
        _ => None,
    };
    Ch { base, whole: false }
}

/// Does a token without letters or digits stick to the word before it? Closing
/// punctuation does — French sets a space before `! ? ; :` and inside `« »`, but a line
/// never starts with them — while a token such as `&` or `§` can start a line.
fn trails(token: &str) -> bool {
    token.chars().all(|c| matches!(c, '!' | '?' | ';' | ':' | '.' | ',' | '…' | '»' | ')' | ']'))
}

/// Splits the text into words: tokens with a letter or digit, plus the trailing
/// punctuation set apart by a space ("Où es-tu ?" → "Où", "es-tu ?"). Every
/// letter-sized character counts, wherever it sits: `!`, `?` and `&` draw shapes the
/// segmentation counts as letter-sized glyphs, so they must be counted here too. A
/// token of non-trailing symbols that draws letter-sized shapes (`&`) is a word of its
/// own; one that draws none (`—`, `-`, `«`) only adds to the text of its neighbour.
fn words(text: &str) -> Vec<Word> {
    let mut out: Vec<Word> = Vec::new();
    for token in text.split_whitespace() {
        let chars: Vec<Ch> = token.chars().filter(|&c| is_tall(c)).map(fold).collect();
        let has_letter = token.chars().any(char::is_alphanumeric);
        if !has_letter && (chars.is_empty() || trails(token)) {
            if let Some(last) = out.last_mut() {
                last.text.push(' ');
                last.text.push_str(token);
                last.chars.extend(chars);
                continue;
            }
            if chars.is_empty() {
                continue;
            }
        }
        if out.len() >= MAX_WORDS {
            break;
        }
        out.push(Word { text: token.to_string(), chars });
    }
    out
}

/// The text in the given case.
pub(crate) fn apply_case(text: &str, case: CaseUsed) -> String {
    match case {
        CaseUsed::Upper => text.to_uppercase(),
        CaseUsed::Lower => text.to_lowercase(),
        CaseUsed::Title => {
            let mut out = String::with_capacity(text.len());
            let mut start = true;
            for ch in text.chars() {
                if ch.is_alphanumeric() {
                    if start {
                        out.extend(ch.to_uppercase());
                    } else {
                        out.extend(ch.to_lowercase());
                    }
                    start = false;
                } else {
                    out.push(ch);
                    // An apostrophe does not start a new word ("L'Étranger" stays).
                    start = ch.is_whitespace() || ch == '-';
                }
            }
            out
        }
        CaseUsed::AsGiven | CaseUsed::Unknown => text.to_string(),
    }
}

#[inline]
fn width_mismatch(a: f32, b: f32) -> f32 {
    ((a - b).abs() / a.max(b).max(1e-3)).min(1.0)
}

/// Aligns the glyphs of one line with a run of letters when their counts differ a
/// little: a glyph may stand for two or three touching letters, two glyphs for one
/// broken letter. Returns the cost and the one-to-one pairs `(glyph, letter)`.
fn align_glyphs(widths: &[f32], gaps: &[f32], expected: &[f32]) -> Option<(f32, Vec<(usize, usize)>)> {
    let (g, n) = (widths.len(), expected.len());
    if g < 2 || n < 3 || g.abs_diff(n) > (n / 4).max(2) {
        return None;
    }
    const INF: f32 = f32::INFINITY;
    let stride = n + 1;
    let mut cost = vec![INF; (g + 1) * stride];
    let mut from = vec![0u8; (g + 1) * stride];
    cost[0] = 0.0;
    for i in 0..=g {
        for j in 0..=n {
            let here = cost[i * stride + j];
            if !here.is_finite() {
                continue;
            }
            let mut relax = |ni: usize, nj: usize, c: f32, mv: u8| {
                let slot = ni * stride + nj;
                if here + c < cost[slot] {
                    cost[slot] = here + c;
                    from[slot] = mv;
                }
            };
            if i < g && j < n {
                relax(i + 1, j + 1, 0.5 * width_mismatch(widths[i], expected[j]), 1);
            }
            if i < g && j + 1 < n {
                let e = expected[j] + expected[j + 1];
                relax(i + 1, j + 2, 0.6 + 0.5 * width_mismatch(widths[i], e), 2);
            }
            if i < g && j + 2 < n {
                let e = expected[j] + expected[j + 1] + expected[j + 2];
                relax(i + 1, j + 3, 1.3 + 0.5 * width_mismatch(widths[i], e), 3);
            }
            if i + 1 < g && j < n {
                let w = widths[i] + gaps[i] + widths[i + 1];
                relax(i + 2, j + 1, 0.8 + 0.5 * width_mismatch(w, expected[j]), 4);
            }
        }
    }
    let total = cost[g * stride + n];
    if !total.is_finite() {
        return None;
    }
    let mut pairs = Vec::new();
    let (mut i, mut j) = (g, n);
    while i > 0 || j > 0 {
        match from[i * stride + j] {
            1 => {
                i -= 1;
                j -= 1;
                pairs.push((i, j));
            }
            2 => {
                i -= 1;
                j -= 2;
            }
            3 => {
                i -= 1;
                j -= 3;
            }
            4 => {
                i -= 2;
                j -= 1;
            }
            _ => return None,
        }
    }
    pairs.reverse();
    Some((total, pairs))
}

/// The structural match of one case hypothesis.
struct Structure {
    cost: f32,
    labels: Vec<Label>,
    /// Per line: the range of words set on it.
    line_words: Vec<Option<Range<usize>>>,
    covered: usize,
    exact: bool,
}

/// Distributes consecutive words over the lines.
fn match_structure(obs: &Observed, words: &[Word], centroids: &FaceDescriptor) -> Option<Structure> {
    let n_lines = obs.lines.len();
    let n_words = words.len();
    if n_lines == 0 || n_lines > MAX_LINES || n_words == 0 {
        return None;
    }
    let mut prefix = vec![0usize; n_words + 1];
    for (i, w) in words.iter().enumerate() {
        prefix[i + 1] = prefix[i] + w.chars.len();
    }
    let total = prefix[n_words];
    let letters: Vec<Ch> = words.iter().flat_map(|w| w.chars.iter().copied()).collect();

    // Expected width of every letter relative to the capital height.
    let cap = centroids.record('H').map_or(0.7, |r| record_geometry(r).0).max(0.2);
    let expected: Vec<f32> = letters
        .iter()
        .map(|c| {
            c.base
                .and_then(|b| centroids.record(b))
                .map_or(0.62, |r| record_geometry(r).1 / cap)
        })
        .collect();

    // Cost and one-to-one pairs of setting words j..k on line l.
    let pair = |l: usize, j: usize, k: usize| -> Option<(f32, Vec<(usize, usize)>)> {
        let range = obs.lines[l].clone();
        let (g, n) = (range.len(), prefix[k] - prefix[j]);
        if g == 0 || n == 0 {
            return None;
        }
        if g == n {
            return Some((0.0, (0..g).map(|i| (i, prefix[j] + i)).collect()));
        }
        let glyphs = &obs.glyphs[range];
        let mut heights: Vec<f32> = glyphs.iter().map(|o| o.main_box[3] - o.main_box[1]).collect();
        heights.sort_by(f32::total_cmp);
        let href = heights[(heights.len() * 9 / 10).min(heights.len() - 1)].max(1.0);
        let widths: Vec<f32> =
            glyphs.iter().map(|o| (o.whole_box[2] - o.whole_box[0]) / href).collect();
        let gaps: Vec<f32> = glyphs
            .windows(2)
            .map(|p| ((p[1].whole_box[0] - p[0].whole_box[2]) / href).max(0.0))
            .chain(std::iter::once(0.0))
            .collect();
        let (cost, pairs) = align_glyphs(&widths, &gaps, &expected[prefix[j]..prefix[k]])?;
        Some((cost, pairs.into_iter().map(|(gi, ci)| (gi, prefix[j] + ci)).collect()))
    };

    // best[l][k]: cheapest way to account for lines 0..l with the words before cut k.
    const INF: f32 = f32::INFINITY;
    let stride = n_words + 1;
    let mut best = vec![INF; (n_lines + 1) * stride];
    // choice: usize::MAX = line skipped, else the start cut j.
    let mut choice = vec![usize::MAX; (n_lines + 1) * stride];
    for (k, slot) in best[..stride].iter_mut().enumerate() {
        *slot = UNCOVERED_COST * prefix[k] as f32;
    }
    for l in 0..n_lines {
        let skip = 1.0 + 0.25 * obs.lines[l].len() as f32;
        for k in 0..=n_words {
            let mut value = best[l * stride + k] + skip;
            let mut how = usize::MAX;
            for j in 0..k {
                let before = best[l * stride + j];
                if !before.is_finite() {
                    continue;
                }
                if let Some((c, _)) = pair(l, j, k) {
                    if before + c < value {
                        value = before + c;
                        how = j;
                    }
                }
            }
            best[(l + 1) * stride + k] = value;
            choice[(l + 1) * stride + k] = how;
        }
    }
    let (mut k, mut cost) = (0usize, INF);
    for cut in 0..=n_words {
        let c = best[n_lines * stride + cut] + UNCOVERED_COST * (total - prefix[cut]) as f32;
        if c < cost {
            cost = c;
            k = cut;
        }
    }
    if !cost.is_finite() {
        return None;
    }

    let mut labels = Vec::new();
    let mut line_words: Vec<Option<Range<usize>>> = vec![None; n_lines];
    let (mut covered, mut exact) = (0usize, true);
    for l in (0..n_lines).rev() {
        let j = choice[(l + 1) * stride + k];
        if j == usize::MAX {
            continue;
        }
        let (c, pairs) = pair(l, j, k)?;
        exact &= c == 0.0;
        covered += prefix[k] - prefix[j];
        for (gi, ci) in pairs {
            let ch = letters[ci];
            if let Some(base) = ch.base {
                labels.push(Label { obs: obs.lines[l].start + gi, ch: base, whole: ch.whole });
            }
        }
        line_words[l] = Some(j..k);
        k = j;
    }
    if covered == 0 {
        return None;
    }
    labels.sort_by_key(|l| l.obs);
    exact &= covered == total;
    Some(Structure { cost, labels, line_words, covered, exact })
}

/// Matches the letter counts of `cased_text` with the lines of glyphs, without looking
/// at shapes: `(share of the letters accounted for, counts fit exactly, lines matched)`.
pub(crate) fn match_text_structure(
    obs: &Observed,
    cased_text: &str,
    centroids: &FaceDescriptor,
) -> Option<(f32, bool, usize)> {
    let ws = words(cased_text);
    let structure = match_structure(obs, &ws, centroids)?;
    let total: usize = ws.iter().map(|w| w.chars.len()).sum();
    let matched = structure.line_words.iter().filter(|l| l.is_some()).count();
    Some((structure.covered as f32 / total.max(1) as f32, structure.exact, matched))
}

/// Mean distance between labelled glyphs and the average shape of their letters: the
/// orientation histogram, the aspect ratio and the height relative to the tallest
/// letter of the line. `None` when no label can be checked.
fn plausibility(obs: &Observed, labels: &[Label], centroids: &FaceDescriptor) -> Option<f32> {
    // Tallest glyph and tallest expected letter per line.
    let n_lines = obs.lines.len();
    let mut line_px = vec![0.0f32; n_lines];
    let mut line_em = vec![0.0f32; n_lines];
    for label in labels {
        let Some(o) = obs.glyphs.get(label.obs) else { continue };
        let Some(record) = centroids.record(label.ch) else { continue };
        let b = o.bbox(label.whole);
        line_px[o.line] = line_px[o.line].max(b[3] - b[1]);
        line_em[o.line] = line_em[o.line].max(record_geometry(record).0);
    }
    let (mut sum, mut n) = (0.0f32, 0usize);
    for label in labels {
        let Some(o) = obs.glyphs.get(label.obs) else { continue };
        let (Some(f), Some(record)) = (o.features(label.whole), centroids.record(label.ch)) else {
            continue;
        };
        let reference = &record[..f.0.len()];
        let hog = hog_distance(f.hog(), &reference[..f.hog().len()]);
        let aspect = f32::from(f.scalars()[S_ASPECT].abs_diff(reference[f.hog().len() + S_ASPECT]))
            / 255.0;
        let b = o.bbox(label.whole);
        let rel_px = (b[3] - b[1]) / line_px[o.line].max(1.0);
        let rel_em = record_geometry(record).0 / line_em[o.line].max(1e-3);
        sum += hog + aspect + 1.5 * (rel_px - rel_em).abs();
        n += 1;
    }
    (n > 0).then(|| sum / n as f32)
}

/// Matches `text` with the glyphs under every case hypothesis. Returns the hypotheses
/// whose letter counts fit the lines and whose letters look like the glyphs, most
/// plausible first (empty when none does).
///
/// More than one hypothesis can survive, and rightly so: a small-caps font draws the
/// lower-case letters of "Title" as capitals, a caps-only font draws everything so. The
/// caller scores every font under each hypothesis and keeps the best.
pub(crate) fn label(obs: &Observed, text: &str, centroids: &FaceDescriptor) -> Vec<Labelling> {
    let mut out: Vec<(f32, Labelling)> = Vec::new();
    if obs.glyphs.is_empty() || text.trim().is_empty() {
        return Vec::new();
    }
    let mut tried: Vec<String> = Vec::new();
    for case in [CaseUsed::AsGiven, CaseUsed::Upper, CaseUsed::Title, CaseUsed::Lower] {
        let cased = apply_case(text, case);
        if tried.contains(&cased) {
            continue;
        }
        let ws = words(&cased);
        tried.push(cased);
        let Some(structure) = match_structure(obs, &ws, centroids) else { continue };
        let total: usize = ws.iter().map(|w| w.chars.len()).sum();
        let Some(plaus) = plausibility(obs, &structure.labels, centroids) else { continue };
        if plaus > PLAUSIBLE_MAX || structure.labels.is_empty() {
            continue;
        }
        let line_texts = structure
            .line_words
            .iter()
            .map(|r| {
                r.clone()
                    .map(|r| ws[r].iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" "))
            })
            .collect();
        out.push((
            plaus + 0.03 * structure.cost,
            Labelling {
                case,
                labels: structure.labels,
                line_texts,
                coverage: structure.covered as f32 / total.max(1) as f32,
            },
        ));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Hypotheses far less plausible than the best only add noise.
    let limit = out.first().map_or(0.0, |(k, _)| k + PLAUSIBLE_SPREAD);
    out.into_iter().filter(|(k, _)| *k <= limit).map(|(_, l)| l).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_transforms() {
        assert_eq!(apply_case("l'étranger du nord-est", CaseUsed::Title), "L'étranger Du Nord-Est");
        assert_eq!(apply_case("Le Rouge", CaseUsed::Upper), "LE ROUGE");
        assert_eq!(apply_case("Le Rouge", CaseUsed::Lower), "le rouge");
        assert_eq!(apply_case("Le Rouge", CaseUsed::AsGiven), "Le Rouge");
        assert_eq!(apply_case("straße", CaseUsed::Upper), "STRASSE");
    }

    #[test]
    fn words_keep_punctuation_and_fold_accents() {
        let ws = words("  L'Été — 1984 !  ");
        assert_eq!(ws.len(), 2);
        assert_eq!(ws[0].text, "L'Été —");
        assert_eq!(
            ws[0].chars,
            vec![
                Ch { base: Some('L'), whole: true },
                Ch { base: Some('E'), whole: false },
                Ch { base: Some('t'), whole: true },
                Ch { base: Some('e'), whole: false },
            ]
        );
        assert_eq!(ws[1].text, "1984 !");
        // The exclamation mark is letter-sized (stem + dot): the segmenter counts it.
        assert_eq!(ws[1].chars.len(), 5);
        assert_eq!(ws[1].chars[4], Ch { base: Some('!'), whole: true });
        // "&" can start a line: a word of its own. "?" sticks to its word.
        let ws = words("Pride & Prejudice");
        assert_eq!(ws.iter().map(|w| w.chars.len()).collect::<Vec<_>>(), vec![5, 1, 9]);
        let ws = words("Où es-tu ?");
        assert_eq!(ws.iter().map(|w| w.text.as_str()).collect::<Vec<_>>(), vec!["Où", "es-tu ?"]);
        assert_eq!(ws[1].chars.len(), 5);
        // Letters without a comparable base still count.
        assert_eq!(fold('ç'), Ch { base: None, whole: false });
        assert_eq!(fold('ï'), Ch { base: None, whole: false });
        assert_eq!(fold('Ï'), Ch { base: Some('I'), whole: false });
        assert!(is_tall('ß') && is_tall('Ж') && !is_tall('-') && !is_tall('\''));
        assert!(words("... — !!").len() <= 1);
        assert!(words("").is_empty());
    }

    #[test]
    fn glyph_alignment_finds_a_touching_pair_and_a_broken_letter() {
        // Five letters of width 0.6; glyphs: the 2nd and 3rd touch.
        let expected = [0.6f32; 5];
        let widths = [0.6, 1.25, 0.6, 0.6];
        let gaps = [0.1f32; 4];
        let (cost, pairs) = align_glyphs(&widths, &gaps, &expected).expect("alignable");
        assert!(cost < 1.2, "{cost}");
        assert_eq!(pairs, vec![(0, 0), (2, 3), (3, 4)]);
        // Four letters, the 3rd broken in two halves.
        let expected = [0.6f32; 4];
        let widths = [0.6, 0.6, 0.25, 0.25, 0.6];
        let gaps = [0.1f32; 5];
        let (_, pairs) = align_glyphs(&widths, &gaps, &expected).expect("alignable");
        assert_eq!(pairs, vec![(0, 0), (1, 1), (4, 3)]);
        // Hopeless count differences are refused.
        assert!(align_glyphs(&[0.5; 3], &[0.1; 3], &[0.5; 9]).is_none());
        assert!(align_glyphs(&[], &[], &[0.5; 3]).is_none());
    }
}
