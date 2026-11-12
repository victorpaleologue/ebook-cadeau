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

/// `line` cut so that it fits `width` with an ellipsis at its end.
pub(crate) fn ellipsize(font: &Font, line: &str, size_pt: f64, width: f64) -> String {
    let ellipsis = if font.glyph_id('…').is_some() { "…" } else { "..." };
    let mut kept = line.trim_end().to_owned();
    loop {
        let trimmed = kept.trim_end_matches(|c: char| c.is_whitespace() || ",;:-–—(".contains(c));
        let candidate = format!("{trimmed}{ellipsis}");
        if trimmed.is_empty() || width_of(font, &candidate, size_pt) <= width {
            return candidate;
        }
        kept = trimmed.to_owned();
        kept.pop();
    }
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
    if text.trim().is_empty() || !(width > 0.0) || spec.max_lines == 0 {
        return Para::empty();
    }
    let min_pt = spec.min_pt.max(1.0);
    let mut size = spec.max_pt.max(min_pt);
    loop {
        let line_mm = pt_to_mm(size * spec.leading);
        let lines = wrap(font, text, size, width, &opts());
        let fits = lines.len() <= spec.max_lines && lines.len() as f64 * line_mm <= max_height + 1e-9;
        if fits {
            return Para { lines, size_pt: size, line_mm, clamped: false };
        }
        if size <= min_pt + 1e-9 {
            // Cut: as many lines as fit (at least one), the last one with an ellipsis.
            let by_height = (max_height / line_mm).floor();
            let room = if by_height.is_finite() && by_height >= 1.0 { by_height as usize } else { 1 };
            let keep = room.min(spec.max_lines).max(1);
            let mut lines: Vec<String> = lines.into_iter().take(keep).collect();
            if let Some(last) = lines.pop() {
                lines.push(ellipsize(font, &last, size, width));
            }
            return Para { lines, size_pt: size, line_mm, clamped: true };
        }
        size = (size - 0.5).max(min_pt);
    }
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
            items.extend(
                self.aligned(doc, font, line, para.size_pt, color, x, width, baseline, align)?,
            );
        }
        Ok(items)
    }
}
