//! Text for the spine: what is set (title, author), in which font, at which size, and
//! where the outlines go.
//!
//! All measuring is done on the **ink** of the actual string (the exact bounds of its
//! outlines), not on font-wide metrics: an accented capital or a descender counts, and a
//! title without either may be set larger.

use ebc_i18n::SpineDirection;
use ebc_model::BookMeta;
use ebc_scene::{Path, Point, Rect, Transform, pt_to_mm};
use ebc_text::{Font, ShapeOptions, outline_bounds, shape};
use unicode_segmentation::UnicodeSegmentation;

use crate::SpineOptions;
use crate::geometry::Geometry;

/// Size at which text is measured; ink bounds are proportional to the size.
const REFERENCE_PT: f64 = 100.0;
/// Relative slack kept on every geometric limit so that floating-point rounding can
/// never push an outline over a margin.
const FIT_SLACK: f64 = 1.0e-9;

/// `true` for the no-break spaces that are kept as they are (French punctuation).
fn is_no_break_space(c: char) -> bool {
    matches!(c, '\u{00A0}' | '\u{202F}' | '\u{2007}')
}

/// `true` for invisible format characters that are removed before typesetting: soft
/// hyphen, zero-width space, directional marks, embeddings, overrides and isolates, word
/// joiner and invisible operators, byte-order mark. The zero-width joiner and non-joiner
/// are kept: shaping needs them.
fn is_ignorable(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'
            | '\u{200B}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FEFF}'
    )
}

/// Cleans a metadata string for a one-line setting: control characters and line breaks
/// become spaces, runs of ordinary white space collapse to one space, invisible format
/// characters are removed, and the ends are trimmed. No-break spaces are kept. Letters
/// and their case are never changed.
pub(crate) fn normalise(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for c in text.chars() {
        if is_ignorable(c) {
            continue;
        }
        let collapsible = c.is_control() || (c.is_whitespace() && !is_no_break_space(c));
        if collapsible {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(c);
    }
    out.trim_matches(|c: char| c == ' ' || is_no_break_space(c)).to_owned()
}

/// Characters that may separate a main title from what follows, or dangle at the end of
/// a shortened title.
fn is_trailing_punctuation(c: char) -> bool {
    c == ' '
        || is_no_break_space(c)
        || matches!(
            c,
            ',' | ';' | ':' | '.' | '-' | '–' | '—' | '(' | '[' | '/' | '«' | '“' | '"' | '\''
        )
}

/// The title to set: [`BookMeta::title`], cleaned. When the title field repeats the
/// record's own subtitle after a separator (`"Title: subtitle"`), the subtitle is cut
/// off. Nothing is ever added.
pub(crate) fn title_of(book: &BookMeta) -> String {
    let title = normalise(&book.title);
    let Some(subtitle) = book.subtitle.as_deref().map(normalise).filter(|s| !s.is_empty())
    else {
        return title;
    };
    let Some(head) = title.strip_suffix(subtitle.as_str()) else { return title };
    let separated = head.chars().next_back().is_some_and(is_trailing_punctuation);
    let head = head.trim_end_matches(is_trailing_punctuation);
    if separated && !head.is_empty() { head.to_owned() } else { title }
}

/// The author lines to try, most complete first: every author, then the first author
/// alone (when there are several).
pub(crate) fn author_candidates(book: &BookMeta) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let all = book.author_line().map(|line| normalise(&line)).unwrap_or_default();
    if !all.is_empty() {
        out.push(all);
    }
    if let Some(first) = book.authors().map(normalise).find(|name| !name.is_empty()) {
        if !out.contains(&first) {
            out.push(first);
        }
    }
    out
}

/// A string and the font that will draw it.
#[derive(Debug, Clone)]
pub(crate) struct Resolved {
    /// The font to use (the primary font, or the fallback).
    pub(crate) font: Font,
    /// The text with every character the font cannot draw removed.
    pub(crate) text: String,
    /// The characters that were removed, in order of first appearance.
    pub(crate) missing: Vec<char>,
    /// The characters the primary font lacks (empty when it covers the text).
    pub(crate) primary_missing: Vec<char>,
    /// `true` when `font` is the fallback font.
    pub(crate) fallback_used: bool,
}

/// Chooses the font for `text`: the primary font if it covers every character, else the
/// fallback if it covers more, and removes what the chosen font cannot draw (so that no
/// `.notdef` box is ever printed).
pub(crate) fn resolve(text: &str, primary: &Font, fallback: Option<&Font>) -> Resolved {
    let primary_missing = primary.missing_chars(text);
    if primary_missing.is_empty() {
        return Resolved {
            font: primary.clone(),
            text: text.to_owned(),
            missing: Vec::new(),
            primary_missing,
            fallback_used: false,
        };
    }
    let (font, missing, fallback_used) = match fallback {
        Some(other) => {
            let other_missing = other.missing_chars(text);
            if other_missing.len() < primary_missing.len() {
                (other.clone(), other_missing, true)
            } else {
                (primary.clone(), primary_missing.clone(), false)
            }
        }
        None => (primary.clone(), primary_missing.clone(), false),
    };
    let kept: String = text.chars().filter(|c| !missing.contains(c)).collect();
    Resolved { font, text: normalise(&kept), missing, primary_missing, fallback_used }
}

/// Formats a list of characters for a note: `'Ω' (U+03A9), 'я' (U+044F)`, at most ten.
pub(crate) fn describe_chars(chars: &[char]) -> String {
    let mut parts: Vec<String> =
        chars.iter().take(10).map(|&c| format!("'{c}' (U+{:04X})", c as u32)).collect();
    if chars.len() > 10 {
        parts.push(format!("and {} more", chars.len() - 10));
    }
    parts.join(", ")
}

/// The ink of a string per point of font size, in millimetres, relative to the start of
/// the baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Measure {
    /// Where the ink starts along the baseline.
    pub(crate) ink_start: f64,
    /// Where the ink ends along the baseline.
    pub(crate) ink_end: f64,
    /// How far the ink rises above the baseline (negative if it stays below).
    pub(crate) ink_top: f64,
    /// How far the ink drops below the baseline (negative if it stays above).
    pub(crate) ink_bottom: f64,
}

impl Measure {
    /// Length of the ink along the baseline, per point.
    pub(crate) fn length(&self) -> f64 {
        (self.ink_end - self.ink_start).max(0.0)
    }
}

/// Shaping options for a run.
pub(crate) fn shape_options(tracking: f32, language: Option<&str>) -> ShapeOptions {
    ShapeOptions {
        letter_spacing_em: tracking,
        language: language.map(str::to_owned),
        ..ShapeOptions::default()
    }
}

/// Measures the ink of `text`; `None` when nothing would be drawn.
pub(crate) fn measure(font: &Font, text: &str, opts: &ShapeOptions) -> Option<Measure> {
    let run = shape(font, text, REFERENCE_PT, opts);
    let ink = run.ink_bounds(font, Point::new(0.0, 0.0))?;
    let m = Measure {
        ink_start: ink.x / REFERENCE_PT,
        ink_end: ink.right() / REFERENCE_PT,
        ink_top: -ink.y / REFERENCE_PT,
        ink_bottom: ink.bottom() / REFERENCE_PT,
    };
    (m.ink_start.is_finite()
        && m.ink_end.is_finite()
        && m.ink_top.is_finite()
        && m.ink_bottom.is_finite())
    .then_some(m)
}

/// Capital height of `font` per point of size, in millimetres (0.7 em if the font
/// reports none).
pub(crate) fn cap_height_per_pt(font: &Font) -> f64 {
    let cap = font.metrics().cap_height_mm(1.0);
    if cap.is_finite() && cap > 0.0 { cap } else { 0.7 * pt_to_mm(1.0) }
}

/// The largest size at which a run measured as `m` fits: its ink no longer than `live`,
/// its capitals no taller than `band`, and — with the capital-height box centred on the
/// spine's centre line — all of its ink inside the band. May be below the minimum size.
fn best_size(m: &Measure, cap_pt: f64, band: f64, live: f64, max_pt: f64) -> f64 {
    let mut limit = f64::INFINITY;
    if m.length() > 0.0 {
        limit = limit.min(live / m.length());
    }
    limit = limit.min(band / cap_pt);
    let towards_tops = m.ink_top - cap_pt / 2.0;
    if towards_tops > 0.0 {
        limit = limit.min(band / 2.0 / towards_tops);
    }
    let towards_baseline = m.ink_bottom + cap_pt / 2.0;
    if towards_baseline > 0.0 {
        limit = limit.min(band / 2.0 / towards_baseline);
    }
    max_pt.min(limit * (1.0 - FIT_SLACK))
}

/// A run ready to be placed.
#[derive(Debug, Clone)]
pub(crate) struct SetRun {
    pub(crate) font: Font,
    pub(crate) text: String,
    pub(crate) size_pt: f64,
    pub(crate) shape: ShapeOptions,
    pub(crate) measure: Measure,
}

impl SetRun {
    /// Length of the ink along the spine, in millimetres.
    pub(crate) fn length(&self) -> f64 {
        self.measure.length() * self.size_pt
    }
}

/// How far down the fitting ladder the title had to go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TitleStep {
    /// The whole title, normal letter-spacing.
    Whole,
    /// The whole title with tightened letter-spacing.
    Tightened,
    /// Cut at the first subtitle separator.
    MainPart,
    /// Truncated at a word boundary, with an ellipsis.
    Truncated,
    /// Truncated inside its first word (no word fits), with an ellipsis.
    TruncatedInWord,
}

/// The fitted title.
#[derive(Debug, Clone)]
pub(crate) struct TitleFit {
    pub(crate) run: SetRun,
    pub(crate) step: TitleStep,
}

/// The part of a title before its first subtitle separator (`": "`, `" — "`, `" – "`,
/// `" - "`, `" ("`), if there is one and something of at least two letters precedes it.
fn main_part(text: &str) -> Option<&str> {
    let cut = [": ", ":\u{00A0}", " — ", " – ", " - ", " ("]
        .iter()
        .filter_map(|separator| text.find(separator))
        .min()?;
    let head = text.get(..cut)?.trim_end_matches(is_trailing_punctuation);
    (head.graphemes(true).count() >= 2 && head.len() < text.len()).then_some(head)
}

/// Fits the title on one line, going down the ladder only as far as needed:
///
/// 1. the whole title at the largest size in `min_size_pt..=max_size_pt` that fits;
/// 2. the same with letter-spacing tightened (half, then all of `max_tightening_em`);
/// 3. the title cut at its first subtitle separator;
/// 4. the title truncated at a word boundary, with an ellipsis, at the minimum size;
/// 5. the first word truncated at a grapheme boundary, with an ellipsis.
///
/// Returns `None` when not even one letter and an ellipsis fit at the minimum size (the
/// spine is too narrow or too short), or when the text draws nothing.
pub(crate) fn fit_title(
    font: &Font,
    text: &str,
    language: Option<&str>,
    geom: &Geometry,
    opts: &SpineOptions,
) -> Option<TitleFit> {
    let cap_pt = cap_height_per_pt(font);
    let (band, live) = (geom.band, geom.live_len());
    let attempt = |candidate: &str, tracking: f32| -> Option<SetRun> {
        let shape = shape_options(tracking, language);
        let m = measure(font, candidate, &shape)?;
        let size = best_size(&m, cap_pt, band, live, opts.max_size_pt);
        (size >= opts.min_size_pt).then(|| SetRun {
            font: font.clone(),
            text: candidate.to_owned(),
            size_pt: size,
            shape,
            measure: m,
        })
    };

    if let Some(run) = attempt(text, 0.0) {
        return Some(TitleFit { run, step: TitleStep::Whole });
    }
    if opts.max_tightening_em > 0.0 {
        for share in [0.5_f32, 1.0] {
            if let Some(run) = attempt(text, -opts.max_tightening_em * share) {
                return Some(TitleFit { run, step: TitleStep::Tightened });
            }
        }
    }
    if let Some(main) = main_part(text) {
        if let Some(run) = attempt(main, 0.0) {
            return Some(TitleFit { run, step: TitleStep::MainPart });
        }
    }

    let ellipsis = if font.missing_chars("…").is_empty() { "…" } else { "..." };
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    let with_words = |n: usize| -> String {
        let head = words.get(..n).unwrap_or(&[]).join(" ");
        format!("{}{ellipsis}", head.trim_end_matches(is_trailing_punctuation))
    };
    // Largest number of whole words that still fits (the predicate is monotonic: more
    // words are longer and can only add taller or deeper ink).
    let (mut lo, mut hi) = (0usize, words.len());
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if attempt(&with_words(mid), 0.0).is_some() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if lo >= 1 {
        let candidate = with_words(lo);
        if candidate != ellipsis {
            if let Some(run) = attempt(&candidate, 0.0) {
                return Some(TitleFit { run, step: TitleStep::Truncated });
            }
        }
    }

    // Not even the first word fits: cut it at a grapheme boundary.
    let first = words.first()?;
    let graphemes: Vec<&str> = first.graphemes(true).collect();
    let with_graphemes =
        |n: usize| -> String { format!("{}{ellipsis}", graphemes.get(..n).unwrap_or(&[]).concat()) };
    let (mut lo, mut hi) = (0usize, graphemes.len());
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if attempt(&with_graphemes(mid), 0.0).is_some() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if lo == 0 {
        return None;
    }
    attempt(&with_graphemes(lo), 0.0).map(|run| TitleFit { run, step: TitleStep::TruncatedInWord })
}

/// Fits an author line beside a title that has already been fitted. The title is never
/// made smaller for it: the author gets the largest size up to `author_scale` × the title
/// size that fits in the room left, and is dropped (`None`) if that is below
/// `author_min_size_pt`.
pub(crate) fn fit_author(
    font: &Font,
    text: &str,
    language: Option<&str>,
    title: &SetRun,
    geom: &Geometry,
    opts: &SpineOptions,
) -> Option<SetRun> {
    let shape = shape_options(0.0, language);
    let m = measure(font, text, &shape)?;
    if m.length() <= 0.0 {
        return None;
    }
    let (band, live) = (geom.band, geom.live_len());
    // The author shares the title's baseline, which sits half a title cap height off the
    // centre line.
    let half_cap = cap_height_per_pt(&title.font) * title.size_pt / 2.0;
    let room = live - title.length() - opts.author_gap;
    let mut limit = (room / m.length()).min(opts.author_max_share * live / m.length());
    if m.ink_top > 0.0 {
        limit = limit.min((band / 2.0 + half_cap) / m.ink_top);
    }
    if m.ink_bottom > 0.0 {
        limit = limit.min((band / 2.0 - half_cap) / m.ink_bottom);
    }
    let size = (opts.author_scale * title.size_pt).min(limit * (1.0 - FIT_SLACK));
    (size >= opts.author_min_size_pt).then(|| SetRun {
        font: font.clone(),
        text: text.to_owned(),
        size_pt: size,
        shape,
        measure: m,
    })
}

/// The x coordinate of the shared baseline: half a title cap height from the centre
/// line, on the back-cover (left) side for top-to-bottom text and on the front-cover
/// (right) side for bottom-to-top text, so that the capitals are centred on the spine.
pub(crate) fn baseline_x(title: &SetRun, direction: SpineDirection, geom: &Geometry) -> f64 {
    let half_cap = cap_height_per_pt(&title.font) * title.size_pt / 2.0;
    match direction {
        SpineDirection::TopToBottom => geom.center_x - half_cap,
        SpineDirection::BottomToTop => geom.center_x + half_cap,
    }
}

/// Turns a run into outlines on the page.
///
/// `along` is the distance, in the reading direction, from the start of the live length
/// (its top for top-to-bottom text, its bottom for bottom-to-top text) to the first bit
/// of ink of the run.
///
/// ```text
/// horizontal run (u →, v ↓, baseline v = 0)      on the page (x →, y ↓)
///
///   top-to-bottom:  (u, v) → (baseline_x − v, live_top    + origin + u)   rotate +90°
///   bottom-to-top:  (u, v) → (baseline_x + v, live_bottom − origin − u)   rotate −90°
/// ```
pub(crate) fn place(
    run: &SetRun,
    along: f64,
    baseline_x: f64,
    direction: SpineDirection,
    geom: &Geometry,
) -> Option<(Path, Rect)> {
    let shaped = shape(&run.font, &run.text, run.size_pt, &run.shape);
    let outline = shaped.to_outline(&run.font, Point::new(0.0, 0.0));
    let origin = along - run.measure.ink_start * run.size_pt;
    let transform = match direction {
        SpineDirection::TopToBottom => {
            Transform { a: 0.0, b: 1.0, c: -1.0, d: 0.0, e: baseline_x, f: geom.live_top + origin }
        }
        SpineDirection::BottomToTop => Transform {
            a: 0.0,
            b: -1.0,
            c: 1.0,
            d: 0.0,
            e: baseline_x,
            f: geom.live_bottom - origin,
        },
    };
    let path = outline.transformed(&transform);
    let bounds = outline_bounds(&path)?;
    Some((path, bounds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_model::Contributor;

    #[test]
    fn normalise_collapses_white_space_and_drops_invisible_characters() {
        assert_eq!(normalise("  Le\tComte \n de  Monte-Cristo  "), "Le Comte de Monte-Cristo");
        assert_eq!(normalise("a\u{202E}b\u{200B}c\u{FEFF}"), "abc");
        // No-break spaces survive inside, not at the ends.
        assert_eq!(normalise("\u{00A0}Qui\u{202F}? "), "Qui\u{202F}?");
        assert_eq!(normalise(" \u{0007}\r\n "), "");
        // Case, accents and ligatures are untouched.
        assert_eq!(normalise("CŒUR  d'Été"), "CŒUR d'Été");
    }

    #[test]
    fn title_drops_a_repeated_subtitle_only_after_a_separator() {
        let mut book = BookMeta::titled("Sapiens : Une brève histoire de l'humanité");
        assert_eq!(title_of(&book), "Sapiens : Une brève histoire de l'humanité");
        book.subtitle = Some("Une brève histoire de l'humanité".to_owned());
        assert_eq!(title_of(&book), "Sapiens");
        // A subtitle that is not in the title changes nothing.
        book.subtitle = Some("Autre chose".to_owned());
        assert_eq!(title_of(&book), "Sapiens : Une brève histoire de l'humanité");
        // The title must not be reduced to nothing, nor cut in the middle of a phrase.
        let mut same = BookMeta::titled("Germinal");
        same.subtitle = Some("Germinal".to_owned());
        assert_eq!(title_of(&same), "Germinal");
        let mut glued = BookMeta::titled("Le Rouge et le Noir");
        glued.subtitle = Some("le Noir".to_owned());
        assert_eq!(title_of(&glued), "Le Rouge et le Noir");
    }

    #[test]
    fn author_candidates_are_the_full_line_then_the_first_author() {
        let mut book = BookMeta::titled("T");
        assert!(author_candidates(&book).is_empty());
        book.contributors.push(Contributor::author("  Boileau "));
        assert_eq!(author_candidates(&book), ["Boileau"]);
        book.contributors.push(Contributor::author("Narcejac"));
        assert_eq!(author_candidates(&book), ["Boileau & Narcejac", "Boileau"]);
    }

    #[test]
    fn main_part_cuts_at_the_first_separator() {
        assert_eq!(main_part("Dune: Le Messie"), Some("Dune"));
        assert_eq!(main_part("Dune\u{00A0}: Le Messie — tome 2"), Some("Dune"));
        assert_eq!(main_part("Vingt mille lieues - édition illustrée"), Some("Vingt mille lieues"));
        assert_eq!(main_part("Candide (texte intégral)"), Some("Candide"));
        assert_eq!(main_part("Monte-Cristo"), None, "a hyphen inside a word is not a separator");
        assert_eq!(main_part("12:30"), None);
        assert_eq!(main_part("A: b"), None, "one letter is not a title");
    }

    #[test]
    fn describe_chars_is_bounded() {
        assert_eq!(describe_chars(&['Ω']), "'Ω' (U+03A9)");
        let many: Vec<char> = ('a'..='z').collect();
        let text = describe_chars(&many);
        assert!(text.ends_with("and 16 more"), "{text}");
    }
}
