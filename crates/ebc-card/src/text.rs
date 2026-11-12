//! Small typesetting helpers on top of `ebc-text`: cleaning untrusted strings, fitting
//! paragraphs into boxes, and turning lines into scene items.

use ebc_scene::{Color, Document, FontId, Item, Point, pt_to_mm};
use ebc_text::{Font, ShapeOptions, fit_size, shape, wrap};

use crate::error::{CardError, CardWarning, TextField};

/// Longest text, in characters, that is considered at all; the rest is dropped.
pub(crate) const MAX_TEXT_CHARS: usize = 4_000;
/// Smallest type size the composer ever sets, in points (printers ask for 6 pt or more).
pub(crate) const MIN_TYPE_PT: f64 = 6.0;
/// Smallest size of white ("reversed") text on a coloured ground, in points.
pub(crate) const MIN_REVERSED_PT: f64 = 8.0;

/// Horizontal alignment of a line in its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Align {
    Left,
    Center,
    Right,
}

/// A paragraph broken into lines at one size.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Para {
    pub(crate) lines: Vec<String>,
    pub(crate) size_pt: f64,
    /// Distance between baselines, in millimetres.
    pub(crate) line_mm: f64,
    /// `true` when text was cut to make the paragraph fit.
    pub(crate) clamped: bool,
}

impl Para {
    pub(crate) fn empty() -> Self {
        Self { lines: Vec::new(), size_pt: MIN_TYPE_PT, line_mm: 0.0, clamped: false }
    }

    pub(crate) fn height(&self) -> f64 {
        self.lines.len() as f64 * self.line_mm
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// How a paragraph may be fitted.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParaSpec {
    /// Preferred size, in points.
    pub(crate) max_pt: f64,
    /// Smallest size tried before text is cut, in points.
    pub(crate) min_pt: f64,
    /// Line height as a multiple of the size.
    pub(crate) leading: f64,
    /// Most lines allowed.
    pub(crate) max_lines: usize,
}

impl ParaSpec {
    pub(crate) const fn new(max_pt: f64, min_pt: f64, leading: f64, max_lines: usize) -> Self {
        Self { max_pt, min_pt, leading, max_lines }
    }

    /// One size only.
    pub(crate) const fn fixed(size_pt: f64, leading: f64, max_lines: usize) -> Self {
        Self::new(size_pt, size_pt, leading, max_lines)
    }
}

fn opts() -> ShapeOptions {
    ShapeOptions::default()
}

/// Advance width of `text` on one line, in millimetres.
pub(crate) fn width_of(font: &Font, text: &str, size_pt: f64) -> f64 {
    shape(font, text, size_pt, &opts()).width
}

/// Describes characters for a warning: `'語' (U+8A9E)`, at most twelve of them.
fn describe(chars: &[char]) -> String {
    let mut out: Vec<String> =
        chars.iter().take(12).map(|c| format!("'{c}' (U+{:04X})", u32::from(*c))).collect();
    if chars.len() > 12 {
        out.push("…".to_owned());
    }
    out.join(", ")
}

/// Makes an untrusted string safe to set in `font`: control characters become spaces
/// (line feeds are kept when `multiline`), runs of spaces collapse, over-long input is
/// cut, and characters the font cannot draw are removed. Problems are reported as
/// warnings for `field`.
pub(crate) fn clean(
    font: &Font,
    text: &str,
    field: TextField,
    multiline: bool,
    warnings: &mut Vec<CardWarning>,
) -> String {
    let mut truncated = false;
    let mut normalised = String::with_capacity(text.len().min(MAX_TEXT_CHARS * 4));
    for (count, c) in text.chars().enumerate() {
        if count >= MAX_TEXT_CHARS {
            truncated = true;
            break;
        }
        let c = match c {
            '\n' | '\u{2028}' | '\u{2029}' if multiline => '\n',
            // Invisible format characters (joiners, direction marks, BOM, soft hyphen)
            // are kept for the shaper; every other control character is a space.
            c if c.is_control() => ' ',
            c => c,
        };
        normalised.push(c);
    }
    let missing = font.missing_chars(&normalised);
    let mut out = String::with_capacity(normalised.len());
    for line in normalised.split('\n') {
        let mut previous_space = true;
        let start = out.len();
        for c in line.chars().filter(|c| !missing.contains(c)) {
            let space = c == ' ' || c == '\t';
            if space && previous_space {
                continue;
            }
            out.push(if space { ' ' } else { c });
            previous_space = space;
        }
        let kept = out[start..].trim_end().len();
        out.truncate(start + kept);
        out.push('\n');
    }
    let out = out.trim_matches('\n').to_owned();
    if !missing.is_empty() {
        let warning = CardWarning::MissingGlyphs { field, characters: describe(&missing) };
        if !warnings.contains(&warning) {
            warnings.push(warning);
        }
    }
    if truncated {
        note_clamped(field, warnings);
    }
    out
}

/// Records that `field` was cut (once).
pub(crate) fn note_clamped(field: TextField, warnings: &mut Vec<CardWarning>) {
    let warning = CardWarning::TextClamped { field };
    if !warnings.contains(&warning) {
        warnings.push(warning);
    }
}

/// `line` cut so that it fits `width` with an ellipsis at its end: the longest beginning
/// of the line that fits, found by bisection (so that a 2 000-character name costs a
/// dozen measurements, not two thousand).
pub(crate) fn ellipsize(font: &Font, line: &str, size_pt: f64, width: f64) -> String {
    let ellipsis = if font.glyph_id('…').is_some() { "…" } else { "..." };
    let candidate = |chars: usize| {
        let kept: String = line.chars().take(chars).collect();
        let kept = kept.trim_end_matches(|c: char| c.is_whitespace() || ",;:-–—(".contains(c));
        format!("{kept}{ellipsis}")
    };
    let fits = |chars: usize| width_of(font, &candidate(chars), size_pt) <= width;
    let total = line.trim_end().chars().count();
    if total == 0 || fits(total) {
        return candidate(total);
    }
    // Invariant: `low` characters fit (or `low` is 0), `high` characters do not.
    let (mut low, mut high) = (0, total);
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if fits(middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    candidate(low)
}

/// The word joiner: an invisible character at which a line is never broken.
const WORD_JOINER: char = '\u{2060}';

/// Keeps one-letter prefixes with their word ("e-reader", "e-book", "t-shirt"), so that
/// no line ends in "e-": a word joiner is put after the hyphen for the line breaker.
/// [`layout`] removes it again from the lines it returns.
fn protect_prefix_hyphens(text: &str) -> String {
    if !text.contains('-') {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 6);
    for (i, &c) in chars.iter().enumerate() {
        out.push(c);
        let one_letter_prefix = c == '-'
            && i >= 1
            && chars[i - 1].is_alphabetic()
            && (i == 1 || !chars[i - 2].is_alphanumeric())
            && chars.get(i + 1).is_some_and(|next| next.is_alphabetic());
        if one_letter_prefix {
            out.push(WORD_JOINER);
        }
    }
    out
}

/// Breaks `text` into lines no wider than `width` and no higher in total than
/// `max_height`, at the largest size the spec allows; when it does not fit even at the
/// smallest size, the paragraph is cut and ends with an ellipsis.
pub(crate) fn layout(
    font: &Font,
    text: &str,
    width: f64,
    max_height: f64,
    spec: &ParaSpec,
) -> Para {
    let usable = width.is_finite() && width > 0.0;
    if text.trim().is_empty() || !usable || spec.max_lines == 0 {
        return Para::empty();
    }
    let protected = protect_prefix_hyphens(text);
    let text = protected.as_str();
    let min_pt = spec.min_pt.max(1.0);
    let max_pt = spec.max_pt.max(min_pt);
    let attempt = |size: f64| -> (Vec<String>, f64, bool) {
        let line_mm = pt_to_mm(size * spec.leading);
        let lines: Vec<String> = wrap(font, text, size, width, &opts())
            .into_iter()
            .map(|line| line.replace(WORD_JOINER, ""))
            .collect();
        let fits =
            lines.len() <= spec.max_lines && lines.len() as f64 * line_mm <= max_height + 1e-9;
        (lines, line_mm, fits)
    };

    // The preferred size, in the usual case.
    let (lines, line_mm, fits) = attempt(max_pt);
    if fits {
        return Para { lines, size_pt: max_pt, line_mm, clamped: false };
    }
    // The smallest size: if even that does not fit, cut — as many lines as fit (at least
    // one), the last one with an ellipsis.
    let (lines, line_mm, fits) = attempt(min_pt);
    if !fits {
        let by_height = (max_height / line_mm).floor();
        // The cast saturates: an unlimited height keeps every line `max_lines` allows.
        let room = if by_height >= 1.0 { by_height as usize } else { 1 };
        let keep = room.min(spec.max_lines).max(1);
        let mut lines: Vec<String> = lines.into_iter().take(keep).collect();
        if let Some(last) = lines.pop() {
            lines.push(ellipsize(font, &last, min_pt, width));
        }
        return Para { lines, size_pt: min_pt, line_mm, clamped: true };
    }
    // Somewhere in between: the largest half-point step that fits, by bisection.
    let steps = ((max_pt - min_pt) / 0.5).ceil().max(1.0) as usize;
    let size_at = |step: usize| (min_pt + 0.5 * step as f64).min(max_pt);
    let mut best = Para { lines, size_pt: min_pt, line_mm, clamped: false };
    let (mut low, mut high) = (0usize, steps);
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        let size = size_at(middle);
        let (lines, line_mm, fits) = attempt(size);
        if fits {
            low = middle;
            best = Para { lines, size_pt: size, line_mm, clamped: false };
        } else {
            high = middle;
        }
    }
    best
}

/// One line at the largest size in `min_pt..=max_pt` at which it is at most `width`
/// wide; cut with an ellipsis when it does not fit at `min_pt`. Returns the text to set,
/// its size and whether it was cut.
pub(crate) fn fit_line(
    font: &Font,
    text: &str,
    width: f64,
    max_pt: f64,
    min_pt: f64,
) -> (String, f64, bool) {
    let min_pt = min_pt.min(max_pt);
    match fit_size(font, text, width.max(0.0), f64::INFINITY, min_pt, max_pt, &opts()) {
        Some(size) => (text.to_owned(), size, false),
        None => (ellipsize(font, text, min_pt, width), min_pt, true),
    }
}

/// Where the first baseline of a paragraph lies below the top of its box: the capitals
/// are centred in the line.
pub(crate) fn first_baseline(font: &Font, size_pt: f64, line_mm: f64) -> f64 {
    (line_mm + font.metrics().cap_height_mm(size_pt)) / 2.0
}

/// Registers fonts in the document on first use and turns lines into items.
#[derive(Debug, Default)]
pub(crate) struct Setter {
    fonts: Vec<(Font, Option<FontId>)>,
}

impl Setter {
    /// How `font` is drawn: `Some(id)` as embedded text, `None` as outlines (variable
    /// fonts, whose default instance a viewer would draw, and fonts that forbid subset
    /// embedding but allow outlines).
    fn mode(&mut self, doc: &mut Document, font: &Font) -> Result<Option<FontId>, CardError> {
        if let Some((_, mode)) = self.fonts.iter().find(|(f, _)| f == font) {
            return Ok(*mode);
        }
        let mode = if !font.is_variable() && font.allows_subset_embedding() {
            Some(doc.add_font(font.data().clone(), font.face_index()))
        } else if font.allows_outlines() {
            None
        } else {
            return Err(CardError::FontNotUsable { name: font.full_name() });
        };
        self.fonts.push((font.clone(), mode));
        Ok(mode)
    }

    /// One line of text with its baseline starting at `origin`. Returns nothing for text
    /// that draws nothing.
    pub(crate) fn line(
        &mut self,
        doc: &mut Document,
        font: &Font,
        text: &str,
        size_pt: f64,
        color: Color,
        origin: Point,
    ) -> Result<Option<Item>, CardError> {
        let mut run = shape(font, text, size_pt, &opts());
        // `clean` removed what the font cannot draw; whatever still maps to `.notdef`
        // (a format character the shaper did not hide) is dropped rather than printed as
        // an empty box.
        run.glyphs.retain(|g| g.id != 0);
        if run.glyphs.is_empty() {
            return Ok(None);
        }
        Ok(Some(match self.mode(doc, font)? {
            Some(id) => run.to_glyph_item(id, color, origin),
            None => Item::fill(run.to_outline(font, origin), color),
        }))
    }

    /// One line aligned in the horizontal span `x..x + width`, on `baseline`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn aligned(
        &mut self,
        doc: &mut Document,
        font: &Font,
        text: &str,
        size_pt: f64,
        color: Color,
        x: f64,
        width: f64,
        baseline: f64,
        align: Align,
    ) -> Result<Option<Item>, CardError> {
        let w = width_of(font, text, size_pt);
        let x = match align {
            Align::Left => x,
            Align::Center => x + (width - w) / 2.0,
            Align::Right => x + width - w,
        };
        self.line(doc, font, text, size_pt, color, Point::new(x, baseline))
    }

    /// A paragraph whose box starts at `(x, top)` and is `width` wide.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn para(
        &mut self,
        doc: &mut Document,
        font: &Font,
        para: &Para,
        color: Color,
        x: f64,
        top: f64,
        width: f64,
        align: Align,
    ) -> Result<Vec<Item>, CardError> {
        let mut items = Vec::with_capacity(para.lines.len());
        let first = top + first_baseline(font, para.size_pt, para.line_mm);
        for (i, line) in para.lines.iter().enumerate() {
            let baseline = first + i as f64 * para.line_mm;
            items.extend(self.aligned(
                doc,
                font,
                line,
                para.size_pt,
                color,
                x,
                width,
                baseline,
                align,
            )?);
        }
        Ok(items)
    }
}

#[cfg(all(test, feature = "bundled-fonts"))]
mod tests {
    use super::*;

    fn fonts() -> crate::FontSet {
        crate::bundled_fonts().expect("bundled fonts")
    }

    #[test]
    fn cleaning_removes_what_cannot_be_drawn_and_reports_it() {
        let font = fonts().body;
        let mut warnings = Vec::new();
        let out = clean(
            &font,
            "  Bonjour \u{1F600}  le\tmonde\r\n語 ! ",
            TextField::Message,
            false,
            &mut warnings,
        );
        assert_eq!(out, "Bonjour le monde !");
        assert_eq!(warnings.len(), 1);
        let CardWarning::MissingGlyphs { field: TextField::Message, characters } = &warnings[0]
        else {
            panic!("{warnings:?}")
        };
        assert!(characters.contains("U+1F600") && characters.contains("U+8A9E"), "{characters}");

        // Line feeds survive in multi-line mode; over-long input is cut and reported.
        let mut warnings = Vec::new();
        assert_eq!(clean(&font, "a\n\n b \n", TextField::Message, true, &mut warnings), "a\n\nb");
        assert!(warnings.is_empty());
        let long = "x".repeat(MAX_TEXT_CHARS + 50);
        let out = clean(&font, &long, TextField::Title, false, &mut warnings);
        assert_eq!(out.chars().count(), MAX_TEXT_CHARS);
        assert_eq!(warnings, [CardWarning::TextClamped { field: TextField::Title }]);
        // Nothing drawable at all: an empty string, not a panic.
        let mut warnings = Vec::new();
        assert_eq!(clean(&font, "語語語", TextField::Title, false, &mut warnings), "");
        assert_eq!(clean(&font, "", TextField::Title, false, &mut warnings), "");
    }

    #[test]
    fn paragraphs_shrink_then_get_cut_with_an_ellipsis() {
        let font = fonts().body;
        let text =
            "Un texte assez long pour ne pas tenir sur une seule ligne de cinquante millimètres.";
        let roomy = layout(&font, text, 50.0, 100.0, &ParaSpec::new(12.0, 8.0, 1.3, 20));
        assert!(!roomy.clamped && roomy.size_pt == 12.0 && roomy.lines.len() > 2);
        let tight =
            layout(&font, text, 50.0, roomy.height() * 0.7, &ParaSpec::new(12.0, 8.0, 1.3, 20));
        assert!(!tight.clamped && tight.size_pt < 12.0);
        assert!(tight.height() <= roomy.height() * 0.7 + 1e-9);
        let cut = layout(&font, text, 50.0, 6.0, &ParaSpec::new(12.0, 8.0, 1.3, 20));
        assert!(cut.clamped && cut.size_pt == 8.0);
        assert!(cut.height() <= 6.0 + 1e-9);
        let last = cut.lines.last().expect("one line at least");
        assert!(last.ends_with('…'), "{last}");
        assert!(width_of(&font, last, 8.0) <= 50.0);
        for line in &roomy.lines {
            assert!(width_of(&font, line, roomy.size_pt) <= 50.0 + 1e-9);
        }
        // Degenerate boxes give nothing rather than looping.
        assert!(layout(&font, text, 0.0, 10.0, &ParaSpec::fixed(10.0, 1.2, 3)).is_empty());
        assert!(layout(&font, text, f64::NAN, 10.0, &ParaSpec::fixed(10.0, 1.2, 3)).is_empty());
        assert!(layout(&font, "   ", 50.0, 10.0, &ParaSpec::fixed(10.0, 1.2, 3)).is_empty());
        let one = layout(&font, text, 50.0, 0.0, &ParaSpec::fixed(10.0, 1.2, 3));
        assert_eq!(one.lines.len(), 1);
        // No height limit: the line limit alone decides.
        let three = layout(&font, text, 30.0, f64::INFINITY, &ParaSpec::fixed(10.0, 1.2, 3));
        assert!(three.clamped && three.lines.len() == 3, "{three:?}");
    }

    #[test]
    fn one_letter_prefixes_stay_with_their_word() {
        let font = fonts().body;
        let text = "Open this card to receive it on an e-reader, tablet or phone.";
        for width in [40.0, 55.0, 62.0, 70.0, 80.0, 90.0] {
            let para = layout(&font, text, width, 200.0, &ParaSpec::fixed(11.0, 1.3, 20));
            for line in &para.lines {
                assert!(!line.trim_end().ends_with("e-"), "{width}: {line:?}");
                assert!(!line.contains(WORD_JOINER));
            }
        }
        // Ordinary hyphens still break.
        assert_eq!(
            protect_prefix_hyphens("Jean-Pierre et l'e-book"),
            "Jean-Pierre et l'e-\u{2060}book"
        );
    }

    #[test]
    fn single_lines_are_fitted_or_cut() {
        let font = fonts().body_bold;
        let (text, size, cut) = fit_line(&font, "ebk.example", 60.0, 12.0, 7.0);
        assert_eq!((text.as_str(), size, cut), ("ebk.example", 12.0, false));
        let long = "un-nom-de-domaine-vraiment-beaucoup-trop-long.example";
        let (text, size, cut) = fit_line(&font, long, 30.0, 12.0, 7.0);
        assert!(cut && size == 7.0 && text.ends_with('…'));
        assert!(width_of(&font, &text, size) <= 30.0);
        assert_eq!(ellipsize(&font, "", 8.0, 10.0), "…");
    }
}
