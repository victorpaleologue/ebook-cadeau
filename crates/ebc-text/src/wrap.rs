//! Greedy line breaking on Unicode line-break opportunities (UAX #14).

use std::borrow::Cow;
use std::ops::Range;

use unicode_linebreak::{BreakOpportunity, linebreaks};
use unicode_segmentation::UnicodeSegmentation;

use crate::font::Font;
use crate::guard::contain;
use crate::shape::{ShapeOptions, sane_size, shape_units};
use ebc_scene::MM_PER_PT;

/// Breaks `text` into lines at most `max_width` millimetres wide when set in `font` at
/// `size_pt`, and returns the lines ready to be shaped and drawn one below the other.
///
/// The algorithm is greedy (each line takes as many words as fit) and breaks only where
/// the Unicode line breaking algorithm allows: after spaces and hyphens, never at a
/// no-break space, never between a word and its closing punctuation.
///
/// * Line feeds (and the other mandatory breaks) always start a new line; an empty
///   line in the text stays an empty string in the result.
/// * Trailing white space is removed from every line (it "hangs" beyond the margin, so
///   it never causes a break); leading white space is kept.
/// * A word that is wider than `max_width` on its own is split between grapheme
///   clusters, as late as possible. A single cluster wider than the line still gets a
///   line: **no character is ever dropped**.
/// * A soft hyphen (U+00AD) at which a line is broken is returned as a visible `-`;
///   soft hyphens that were not used stay in the line (they are invisible when shaped).
/// * Every returned line fits: `shape(font, line, size_pt, opts).width <= max_width`,
///   unless the line is one unbreakable cluster.
/// * `max_width` of `f64::INFINITY` (or NaN) only breaks at mandatory breaks.
///
/// Returns an empty vector for an empty text. Use [`wrap_ranges`] to get byte ranges
/// into `text` instead of strings.
#[must_use]
pub fn wrap(
    font: &Font,
    text: &str,
    size_pt: f64,
    max_width: f64,
    opts: &ShapeOptions,
) -> Vec<String> {
    lines_of(font, text, size_pt, max_width, opts)
        .iter()
        .map(|line| display_line(&text[line.start..line.end], line.wrapped).into_owned())
        .collect()
}

/// Like [`wrap`], but returns the byte range of every line in `text`.
///
/// The ranges are contiguous and cover the text exactly (`ranges[0].start == 0`, each
/// range starts where the previous one ends, the last one ends at `text.len()`), so
/// each range still includes the white space or line feed at which the line was broken.
#[must_use]
pub fn wrap_ranges(
    font: &Font,
    text: &str,
    size_pt: f64,
    max_width: f64,
    opts: &ShapeOptions,
) -> Vec<Range<usize>> {
    lines_of(font, text, size_pt, max_width, opts).iter().map(|line| line.start..line.end).collect()
}

fn lines_of(
    font: &Font,
    text: &str,
    size_pt: f64,
    max_width: f64,
    opts: &ShapeOptions,
) -> Vec<Line> {
    let size_pt = sane_size(size_pt);
    let em = size_pt * MM_PER_PT;
    let spacing = f64::from(opts.letter_spacing_em);
    let estimate = Estimate {
        // Cross-word kerning is a tiny fraction of an em; half an em is a generous bound.
        margin: 0.5 * em,
        joint: if spacing.is_finite() { spacing * em } else { 0.0 },
    };
    contain(|| {
        let face = font.hb_face()?;
        let measure = |s: &str| shape_units(&face, s, opts).width_mm(size_pt);
        Some(break_lines(text, max_width, estimate, measure))
    })
    .flatten()
    // A font that cannot be measured: keep the text, break only where mandatory.
    .unwrap_or_else(|| break_lines(text, f64::INFINITY, estimate, |_| 0.0))
}

/// One line: a byte range of the text, and whether it ends at a break *opportunity*
/// (`wrapped`) rather than at a mandatory break, at the end of the text or at a forced
/// cut inside a word. Only wrapped lines turn a final soft hyphen into a visible one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Line {
    start: usize,
    end: usize,
    wrapped: bool,
}

/// The text of a line as it is drawn: without trailing white space, and — when the line
/// was wrapped at a soft hyphen — with a visible hyphen in its place.
fn display_line(slice: &str, wrapped: bool) -> Cow<'_, str> {
    let trimmed = slice.trim_end();
    match trimmed.strip_suffix('\u{ad}') {
        Some(stem) if wrapped => Cow::Owned(format!("{stem}-")),
        _ => Cow::Borrowed(trimmed),
    }
}

/// Segments longer than this many bytes are first measured on prefixes of doubling
/// length, so that a huge unbreakable "word" is never measured whole once per line:
/// finding out that a segment overflows costs about as much as measuring one line.
const PROBE_BYTES: usize = 16;

/// How the width of a line is estimated from the widths of its segments (the pieces
/// between two consecutive break opportunities), which are measured separately.
#[derive(Debug, Clone, Copy)]
struct Estimate {
    /// Upper bound of the error of such a sum against measuring the line in one piece
    /// (kerning or contextual shaping across the joints). Candidates whose estimate is
    /// within the margin of the limit are measured exactly.
    margin: f64,
    /// Width to add per joint: the letter spacing between two segments.
    joint: f64,
}

struct Breaker<'t, M: FnMut(&str) -> f64> {
    text: &'t str,
    max_width: f64,
    unlimited: bool,
    estimate: Estimate,
    measure: M,
}

impl<M: FnMut(&str) -> f64> Breaker<'_, M> {
    /// Does `text[start..end]`, drawn as one line, really fit? Measures the whole line.
    fn fits(&mut self, start: usize, end: usize, wrapped: bool) -> bool {
        if self.unlimited {
            return true;
        }
        let shown = display_line(&self.text[start..end], wrapped);
        // Nothing to draw: white space never forces a break.
        shown.is_empty() || (self.measure)(&shown) <= self.max_width
    }

    /// Width of the segment `text[start..end]` as drawn at the end of a line, or
    /// infinity as soon as a prefix of it is known to be wider than the line. (Advance
    /// widths grow with the text, so a prefix that overflows settles the question.)
    fn shown_width(&mut self, start: usize, end: usize, wrapped: bool) -> f64 {
        if self.unlimited {
            return 0.0;
        }
        let raw = &self.text[start..end];
        let mut probe = PROBE_BYTES;
        while probe < raw.len() {
            let mut cut = probe;
            while !raw.is_char_boundary(cut) {
                cut -= 1;
            }
            let prefix = raw[..cut].trim_end();
            if !prefix.is_empty() && (self.measure)(prefix) > self.max_width {
                return f64::INFINITY;
            }
            probe = probe.saturating_mul(2);
        }
        let shown = display_line(raw, wrapped);
        if shown.is_empty() { 0.0 } else { (self.measure)(&shown) }
    }

    /// Scans forward from the line start `start` (the next opportunity being
    /// `opps[first]`) and returns the index of the last opportunity at which the line
    /// could plausibly end, judging by the sum of the segment widths. Stops at the first
    /// mandatory break.
    fn scan(&mut self, opps: &[(usize, bool)], start: usize, first: usize) -> Option<usize> {
        let mut best = None;
        let mut width = 0.0; // estimated width of text[start..segment_start], spaces included
        let mut segment_start = start;
        let mut blank = true; // nothing but white space so far
        for (k, &(index, hard)) in opps.iter().enumerate().skip(first) {
            let raw = &self.text[segment_start..index];
            let shown = self.shown_width(segment_start, index, !hard);
            let at_line_start = segment_start == start;
            // The first segment is measured as it will be drawn: no estimation error.
            let (estimated, limit) = if at_line_start {
                (shown, self.max_width)
            } else {
                (width + self.estimate.joint + shown, self.max_width + self.estimate.margin)
            };
            blank = blank && raw.trim_end().is_empty();
            if !(self.unlimited || blank || estimated <= limit) {
                break;
            }
            best = Some(k);
            if hard {
                break;
            }
            let untouched = raw.trim_end().len() == raw.len() && !raw.ends_with('\u{ad}');
            let full = if self.unlimited {
                0.0
            } else if untouched {
                shown
            } else {
                (self.measure)(raw)
            };
            width = if at_line_start { full } else { width + self.estimate.joint + full };
            segment_start = index;
        }
        best
    }

    /// Where to cut an unbreakable `text[start..end]` that does not fit: after the
    /// last grapheme cluster that still fits, and after at least one cluster. White
    /// space and soft hyphens following the cut stay on the cut line (they are not
    /// drawn there: a forced cut is not a hyphenation point).
    fn emergency_cut(&mut self, start: usize, end: usize) -> usize {
        let text = self.text;
        let mut boundaries =
            text[start..end].grapheme_indices(true).map(|(i, g)| start + i + g.len());
        let Some(first) = boundaries.next() else { return end };

        let mut good = first;
        if self.fits(start, first, false) {
            // Gallop: try 1, 2, 4, … more clusters until one batch overflows, then
            // bisect inside that batch.
            let mut step = 1usize;
            loop {
                let batch: Vec<usize> = boundaries.by_ref().take(step).collect();
                let Some(&last) = batch.last() else { break };
                if self.fits(start, last, false) {
                    good = last;
                    step = step.saturating_mul(2);
                    continue;
                }
                let (mut lo, mut hi) = (0, batch.len() - 1); // batch[hi] overflows
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    if self.fits(start, batch[mid], false) {
                        good = batch[mid];
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                break;
            }
        }

        let hanging: usize = text[good..end]
            .chars()
            .take_while(|&c| c.is_whitespace() || c == '\u{ad}')
            .map(char::len_utf8)
            .sum();
        good + hanging
    }
}

/// The break opportunities of `text` as `(byte index, mandatory)`, strictly increasing
/// and ending with a mandatory break at the end of the text.
fn opportunities(text: &str) -> Vec<(usize, bool)> {
    let mut opps: Vec<(usize, bool)> = Vec::new();
    let mut last = 0;
    for (index, opportunity) in linebreaks(text) {
        // A conforming segmenter never trips this; never trust it blindly.
        if index <= last || index > text.len() || !text.is_char_boundary(index) {
            continue;
        }
        opps.push((index, opportunity == BreakOpportunity::Mandatory));
        last = index;
    }
    match opps.last_mut() {
        Some(end) if end.0 == text.len() => end.1 = true,
        _ if !text.is_empty() => opps.push((text.len(), true)),
        _ => {}
    }
    opps
}

/// The line breaker proper, independent of fonts: `measure` returns the width of a
/// string in the same unit as `max_width`.
///
/// Each line is found in two steps: a forward scan adds up the widths of the segments
/// (one measurement per segment, so the cost is linear in the text whatever the line
/// length), then the candidates are checked from the last one backwards by measuring
/// the whole line, so every line that is returned is known to fit exactly.
fn break_lines(
    text: &str,
    max_width: f64,
    estimate: Estimate,
    measure: impl FnMut(&str) -> f64,
) -> Vec<Line> {
    let unlimited = max_width.is_nan() || max_width == f64::INFINITY;
    let mut breaker = Breaker { text, max_width, unlimited, estimate, measure };
    let opps = opportunities(text);
    let mut lines = Vec::new();
    let mut start = 0usize;
    let mut next = 0usize; // invariant: opps[next] is the first opportunity after `start`

    while next < opps.len() {
        let mut chosen = None;
        if let Some(best) = breaker.scan(&opps, start, next) {
            let mut candidate = best;
            loop {
                let (index, hard) = opps[candidate];
                if breaker.fits(start, index, !hard) {
                    chosen = Some(candidate);
                    break;
                }
                if candidate == next {
                    break;
                }
                candidate -= 1;
            }
        }
        match chosen {
            Some(candidate) => {
                let (index, hard) = opps[candidate];
                lines.push(Line { start, end: index, wrapped: !hard });
                start = index;
                next = candidate + 1;
            }
            None => {
                // Not even the first segment fits: cut inside it. A lone cluster wider
                // than the line stays whole on a line of its own.
                let end = opps[next].0;
                let cut = match breaker.emergency_cut(start, end) {
                    cut if cut > start && cut <= end => cut,
                    _ => end, // unreachable; guarantees progress whatever happens
                };
                lines.push(Line { start, end: cut, wrapped: false });
                start = cut;
                if cut >= end {
                    next += 1;
                }
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXACT: Estimate = Estimate { margin: 0.0, joint: 0.0 };

    /// A fake monospaced font: every `char` is one unit wide, except the soft hyphen,
    /// which is invisible (as it is when shaped with a real font).
    fn mono(s: &str) -> f64 {
        s.chars().filter(|&c| c != '\u{ad}').count() as f64
    }

    /// Number of places where a space is followed by a visible character — where a
    /// font could kern across the joint between two segments.
    fn joints(s: &str) -> f64 {
        s.as_bytes().windows(2).filter(|w| w[0] == b' ' && w[1] != b' ').count() as f64
    }

    fn shown<'t>(text: &'t str, lines: &[Line]) -> Vec<Cow<'t, str>> {
        lines.iter().map(|l| display_line(&text[l.start..l.end], l.wrapped)).collect()
    }

    fn wrap_mono(text: &str, width: f64) -> Vec<String> {
        let lines = break_lines(text, width, EXACT, mono);
        shown(text, &lines).into_iter().map(Cow::into_owned).collect()
    }

    fn assert_partition(text: &str, lines: &[Line]) {
        let mut at = 0;
        for line in lines {
            assert_eq!(line.start, at, "lines must be contiguous");
            assert!(line.end > line.start, "lines must not be empty ranges");
            at = line.end;
        }
        assert_eq!(at, text.len(), "lines must cover the whole text");
    }

    #[test]
    fn greedy_fill() {
        assert_eq!(
            wrap_mono("the quick brown fox jumps over the lazy dog", 10.0),
            ["the quick", "brown fox", "jumps over", "the lazy", "dog"]
        );
        // Exactly fitting lines are not broken early: "jumps over" is 10 wide.
        assert_eq!(wrap_mono("jumps over", 10.0), ["jumps over"]);
        assert_eq!(wrap_mono("jumps over", 9.0), ["jumps", "over"]);
    }

    #[test]
    fn trailing_spaces_hang_and_never_cause_a_break() {
        assert_eq!(wrap_mono("ab      cd", 2.0), ["ab", "cd"]);
        assert_eq!(wrap_mono("ab   ", 2.0), ["ab"]);
        assert_eq!(wrap_mono("  ab", 4.0), ["  ab"], "leading white space is kept");
    }

    #[test]
    fn mandatory_breaks_and_blank_lines() {
        assert_eq!(wrap_mono("a\nb", 80.0), ["a", "b"]);
        assert_eq!(wrap_mono("a\n\nb\n", 80.0), ["a", "", "b"]);
        assert_eq!(wrap_mono("a\r\nb", 80.0), ["a", "b"]);
        assert_eq!(wrap_mono("\n", 80.0), [""]);
        assert_eq!(wrap_mono("a\u{2028}b\u{2029}c", 80.0), ["a", "b", "c"]);
        assert!(wrap_mono("", 80.0).is_empty());
        // A mandatory break in the middle of a line that would otherwise go on.
        assert_eq!(wrap_mono("aa bb\ncc dd ee", 8.0), ["aa bb", "cc dd ee"]);
        assert_eq!(wrap_mono("aa bb cc\ndd", 5.0), ["aa bb", "cc", "dd"]);
    }

    #[test]
    fn long_words_are_split_as_late_as_possible() {
        assert_eq!(wrap_mono("abcdefghij", 4.0), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap_mono("xx abcdefghij yy", 4.0), ["xx", "abcd", "efgh", "ij", "yy"]);
        // The tail of a split word shares its line with what follows.
        assert_eq!(wrap_mono("abcdef gh", 4.0), ["abcd", "ef", "gh"]);
        assert_eq!(wrap_mono("abcde f", 4.0), ["abcd", "e f"]);
    }

    #[test]
    fn graphemes_are_never_split() {
        // "e" + combining acute is one cluster of two chars (2 units wide here).
        let text = "e\u{301}e\u{301}e\u{301}";
        assert_eq!(wrap_mono(text, 2.0), ["e\u{301}", "e\u{301}", "e\u{301}"]);
        // Too narrow for even one cluster: one cluster per line, nothing lost.
        assert_eq!(wrap_mono(text, 1.0), ["e\u{301}", "e\u{301}", "e\u{301}"]);
        assert_eq!(wrap_mono("ab cd", 0.0), ["a", "b", "c", "d"]);
        assert_eq!(wrap_mono("ab cd", -5.0), ["a", "b", "c", "d"]);
    }

    #[test]
    fn no_break_spaces_and_french_punctuation_hold_together() {
        // NBSP before "!" and inside the guillemets.
        let text = "Bonjour\u{a0}! «\u{a0}Oui\u{a0}» dit-il";
        let lines = wrap_mono(text, 10.0);
        assert_eq!(lines, ["Bonjour\u{a0}!", "«\u{a0}Oui\u{a0}»", "dit-il"]);
        // A hyphen is a break opportunity, kept at the end of the line.
        assert_eq!(wrap_mono("dit-il", 4.0), ["dit-", "il"]);
    }

    #[test]
    fn soft_hyphens_show_only_where_the_line_breaks() {
        let text = "extra\u{ad}ordinaire";
        assert_eq!(wrap_mono(text, 40.0), [text], "unbroken: the soft hyphen stays as is");
        assert_eq!(wrap_mono(text, 10.0), ["extra-", "ordinaire"]);
        // "extra-" is 6 wide: with 5 units the hyphenated form does not fit, so the
        // word is cut without a hyphen (the soft hyphen stays, invisible) rather than
        // overflowing or leaving a lone "-" on a line.
        assert_eq!(wrap_mono(text, 5.0), ["extra\u{ad}", "ordin", "aire"]);
        // A soft hyphen right before a mandatory break is not a hyphenation point.
        assert_eq!(wrap_mono("ab\u{ad}\ncd", 40.0), ["ab\u{ad}", "cd"]);
        // Several hyphenation points: the last one that fits is used; the unused ones
        // stay in the line as (invisible) soft hyphens.
        let long = "in\u{ad}com\u{ad}pré\u{ad}hen\u{ad}sible";
        assert_eq!(wrap_mono(long, 9.0), ["in\u{ad}com\u{ad}pré-", "hen\u{ad}sible"]);
        assert_eq!(wrap_mono(long, 8.0), ["in\u{ad}com-", "pré\u{ad}hen-", "sible"]);
    }

    #[test]
    fn unlimited_widths_only_break_where_mandatory() {
        for width in [f64::INFINITY, f64::NAN] {
            assert_eq!(wrap_mono("a b c\nd e", width), ["a b c", "d e"]);
        }
    }

    #[test]
    fn opportunities_end_with_a_mandatory_break_at_the_end() {
        assert!(opportunities("").is_empty());
        assert_eq!(opportunities("a"), [(1, true)]);
        assert_eq!(opportunities("a b"), [(2, false), (3, true)]);
        assert_eq!(opportunities("a\nb\n"), [(2, true), (4, true)]);
        assert_eq!(opportunities("é-è"), [(3, false), (5, true)]);
    }

    #[test]
    fn ranges_partition_the_text_for_many_widths() {
        let text = "  L'Étranger — efficace œuvre\n\n\tUn livre\u{a0}; deux\u{ad}ième   essai. \
                    Anticonstitutionnellement 你好世界 e\u{301}\u{302} 👩‍👩‍👧 fin \n";
        for width in [-1.0, 0.0, 0.5, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 200.0, f64::NAN] {
            let lines = break_lines(text, width, EXACT, mono);
            assert_partition(text, &lines);
            for shown in shown(text, &lines) {
                let clusters = shown.graphemes(true).filter(|g| *g != "\u{ad}").count();
                assert!(
                    width.is_nan() || mono(&shown) <= width || clusters <= 1,
                    "{shown:?} overflows {width}"
                );
            }
        }
    }

    #[test]
    fn lines_fit_exactly_even_when_segment_sums_underestimate() {
        // A measure with strong positive "kerning" across word joints: every space
        // followed by a letter costs two extra units, which the per-segment estimate
        // cannot see. The exact verification must catch it and step back.
        let kerned = |s: &str| mono(s) + 2.0 * joints(s);
        let text = "aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp";
        for width in [4.0, 7.0, 10.0, 16.0, 25.0, 40.0] {
            for margin in [0.0, 1.0, 100.0] {
                let lines = break_lines(text, width, Estimate { margin, joint: 0.0 }, kerned);
                assert_partition(text, &lines);
                let shown = shown(text, &lines);
                for (k, line) in shown.iter().enumerate() {
                    assert!(kerned(line) <= width, "{line:?} overflows {width}");
                    // Still greedy: the next word would not have fitted.
                    if let Some(next) = shown.get(k + 1) {
                        let word = next.split(' ').next().unwrap_or_default();
                        assert!(kerned(&format!("{line} {word}")) > width, "{line:?} + {word:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_margin_recovers_greedy_lines_when_segment_sums_overestimate() {
        // Negative "kerning": lines are narrower than the sum of their words. Without a
        // margin the scan stops early (valid but not greedy); with a margin covering
        // the error, candidates near the limit are measured exactly.
        let tight = |s: &str| mono(s) - 0.5 * joints(s);
        let text = "aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp";
        for width in [6.0, 9.5, 13.0, 21.0] {
            for margin in [0.0, 8.0] {
                let lines = break_lines(text, width, Estimate { margin, joint: 0.0 }, tight);
                assert_partition(text, &lines);
                let shown = shown(text, &lines);
                for (k, line) in shown.iter().enumerate() {
                    assert!(tight(line) <= width, "{line:?} overflows {width}");
                    if let (true, Some(next)) = (margin > 0.0, shown.get(k + 1)) {
                        let word = next.split(' ').next().unwrap_or_default();
                        assert!(tight(&format!("{line} {word}")) > width, "{line:?} + {word:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn letter_spacing_joints_are_part_of_the_estimate() {
        // Every character is followed by one unit of spacing, except the last of the
        // measured string — as with letter spacing. Sums of segments miss one unit per
        // joint; `joint` puts it back, so no exact re-measuring is needed to be right.
        let spaced = |s: &str| (2.0 * mono(s) - 1.0).max(0.0);
        let text = "aa bb cc dd ee ff gg hh";
        let lines = break_lines(text, 15.0, Estimate { margin: 0.0, joint: 1.0 }, spaced);
        let shown = shown(text, &lines);
        assert_eq!(shown, ["aa bb cc", "dd ee ff", "gg hh"]);
        assert!(shown.iter().all(|l| spaced(l) <= 15.0));
    }

    #[test]
    fn huge_unbreakable_input_is_handled_in_linear_time() {
        // 200 000 letters without a single break opportunity. The measure function is
        // O(n), so a quadratic algorithm would need ~10^10 steps here.
        let text = "a".repeat(200_000);
        let mut measured = 0usize;
        let lines = break_lines(&text, 50.0, EXACT, |s| {
            measured += s.len();
            mono(s)
        });
        assert_partition(&text, &lines);
        assert_eq!(lines.len(), 4000);
        assert!(lines.iter().all(|l| l.end - l.start == 50));
        assert!(measured < 40 * text.len(), "measured {measured} bytes in total");
    }

    #[test]
    fn many_opportunities_on_one_endless_line_are_handled_in_linear_time() {
        // 50 000 words that all fit on one (absurdly wide) line: measuring the line
        // from its start at every opportunity would be quadratic.
        let text = "ab ".repeat(50_000);
        let mut measured = 0usize;
        let lines = break_lines(&text, 1e9, EXACT, |s| {
            measured += s.len();
            mono(s)
        });
        assert_eq!(lines, [Line { start: 0, end: text.len(), wrapped: false }]);
        assert!(measured < 10 * text.len(), "measured {measured} bytes in total");

        // Zero-width break opportunities only (U+200B): same thing with normal widths.
        let zero_width = "\u{200b}".repeat(30_000);
        let mut calls = 0usize;
        let lines = break_lines(&zero_width, 10.0, EXACT, |_| {
            calls += 1;
            0.0
        });
        assert_eq!(lines.len(), 1);
        assert!(calls < 3 * 30_000, "{calls} measurements");
    }

    #[test]
    fn a_column_too_narrow_for_anything_stays_linear() {
        // One cluster per line: the rest of the "word" must not be re-measured whole
        // for each of the 20 000 lines.
        let text = "a".repeat(20_000);
        let mut measured = 0usize;
        let lines = break_lines(&text, 0.0, EXACT, |s| {
            measured += s.len();
            mono(s)
        });
        assert_partition(&text, &lines);
        assert_eq!(lines.len(), 20_000);
        assert!(measured < 40 * text.len(), "measured {measured} bytes in total");
    }

    #[test]
    fn long_lines_use_prefix_probes_correctly() {
        // Segments longer than PROBE_BYTES that do fit must be accepted…
        let word = "x".repeat(1000);
        assert_eq!(wrap_mono(&word, 2000.0), [word.as_str()]);
        assert_eq!(wrap_mono(&word, 1000.0), [word.as_str()]);
        // …and cut at the right place when they do not.
        let lines = wrap_mono(&word, 600.0);
        assert_eq!(lines.iter().map(String::len).collect::<Vec<_>>(), [600, 400]);
        // Multi-byte text: probes must land on character boundaries.
        let accents = "é".repeat(700);
        let lines = wrap_mono(&accents, 300.0);
        assert_eq!(lines.iter().map(|l| l.chars().count()).collect::<Vec<_>>(), [300, 300, 100]);
        // Long lines made of many words.
        let words = "lorem ipsum dolor sit amet ".repeat(40); // 1080 bytes
        assert_eq!(wrap_mono(&words, 2000.0), [words.trim_end()]);
        let lines = wrap_mono(&words, 600.0);
        assert_eq!(lines.len(), 2);
        assert!(mono(&lines[0]) <= 600.0 && mono(&lines[0]) > 590.0, "{}", lines[0].len());
    }
}
