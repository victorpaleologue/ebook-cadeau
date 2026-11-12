//! The flat ground of the outside page when a cover is printed and the request names no
//! ground: one colour **computed** from the cover, or paper white.
//!
//! # The rule
//!
//! Back, spine and front share one flat colour, so that the card reads as one object — a
//! jacket in the cover's own colour — and a crease that drifts by a millimetre shows
//! nothing (`docs/research/03-print-requirements.md`, §5 rule 2). The colour is a single
//! value measured on the cover, never pixels, textures or gradients
//! (`docs/research/07-legal-covers-and-reseller-rights.md`, rule 11):
//!
//! 1. **the frame** — the cover has a plain frame on all four sides
//!    ([`ebc_color::has_uniform_border`]): the frame's colour, so that the cover melts
//!    into the ground;
//! 2. **the binding edge** — otherwise, the strip along the cover's binding edge (its left
//!    4 %) is near-uniform: that strip's colour, so that the spine continues the cover;
//! 3. **the dominant colour** — otherwise, the largest swatch of the cover's five-colour
//!    palette ([`ebc_color::palette`]).
//!
//! **White is kept instead** when print argues against the computed colour:
//!
//! * *too faint* — written as process colour, none of its channels reaches
//!   [`GROUND_MIN_TINT_PERCENT`]: a tint that light does not print reliably, and paper
//!   white says the same thing;
//! * *too heavy* — it needs more than [`GROUND_MAX_INK_PERCENT`] of total ink (or more
//!   than the profile allows, if that is less). The ground covers the whole sheet and
//!   crosses both creases, where heavy coverage cracks (§4.3, "solids crossing a
//!   crease": prefer a mid or light ground). A near-black colour passes: it is written
//!   as K 100 alone, which is what that rule asks of a black ground;
//! * *no legible text* — the text the card sets on it (black or white, whichever
//!   contrasts more) stays under [`GROUND_MIN_CONTRAST`]:1, counting black as black ink
//!   on paper rather than as an ideal black. This rules out saturated mid-tones.
//!
//! [`CardRequest::ground`](crate::CardRequest::ground) overrides all of this: a ground
//! given by the caller is used as it is.
//!
//! # What is guaranteed
//!
//! In a PDF/X file the ground and the cover go through the same ICC transform to the
//! output intent (`ebc_color::SrgbToCmyk`, relative colorimetric with black-point
//! compensation), so the ground matches the cover's frame or edge colour on press to
//! within the transform's interpolation (the ground is also cleaned of tints under 5 %,
//! which can move a very light colour by a few per cent of one ink). The ink figures
//! judged here are those of that transform. In a home PDF both are sRGB and they match.
//! Without a readable output profile the naive device conversion is the fallback, and
//! then the ground is only *close to* the cover. The 240 % threshold is a house value,
//! not a measured one.

use ebc_color::{
    Edge, NEAR_BLACK_LSTAR, Rgb8, SrgbToCmyk, UNIFORM_EDGE_THRESHOLD, black_or_white,
    contrast_ratio, edge_palette, edge_uniformity, has_uniform_border, palette, to_cmyk_naive,
};
use ebc_scene::{Color, Pixmap};

/// Lowest WCAG 2 contrast ratio between a ground computed from the cover and the text
/// set on it; below it the ground is white.
pub const GROUND_MIN_CONTRAST: f64 = 4.5;
/// Highest total ink, in per cent, of a ground computed from the cover; above it (or
/// above the profile's own limit) the ground is white. A house value: well under the
/// common 300 % ceiling and under the strictest product limit surveyed (260 %), because
/// this solid covers the whole sheet and is folded twice.
pub const GROUND_MAX_INK_PERCENT: f32 = 240.0;
/// A ground computed from the cover whose every process channel is under this, in per
/// cent, is white: lighter tints do not print reliably.
pub const GROUND_MIN_TINT_PERCENT: f32 = 5.0;

/// Thickness of the strip examined along the binding edge, as a fraction of the cover's
/// width (the value `ebc-spine` uses for the same purpose).
const EDGE_STRIP_FRACTION: f32 = 0.04;
/// Number of swatches the cover is reduced to when its edges are busy.
const PALETTE_SIZE: usize = 5;
/// The binding edge of a front cover (left-to-right books).
const BINDING_EDGE: Edge = Edge::Left;
/// What 100 % black ink on paper looks like, as sRGB, for contrast estimates: the usual
/// screen stand-in for K 100 on coated stock. Text is never darker than this.
const PRINTED_BLACK: Rgb8 = Rgb8::new(35, 31, 32);

/// Why paper white was preferred to the colour computed from a cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Objection {
    /// No channel reaches [`GROUND_MIN_TINT_PERCENT`].
    TooFaint,
    /// More than [`GROUND_MAX_INK_PERCENT`] (or the profile's limit) of total ink.
    TooHeavy,
    /// Text on it stays under [`GROUND_MIN_CONTRAST`].
    NoLegibleText,
}

/// The one colour a cover stands for: its frame, else its plain binding edge, else its
/// dominant colour (see the module documentation). White for an image without pixels.
pub(crate) fn cover_colour(cover: &Pixmap) -> Rgb8 {
    if let Some(frame) = has_uniform_border(cover) {
        return frame;
    }
    if edge_uniformity(cover, BINDING_EDGE, EDGE_STRIP_FRACTION) >= UNIFORM_EDGE_THRESHOLD
        && let Some(edge) = edge_palette(cover, BINDING_EDGE, EDGE_STRIP_FRACTION, 1).first()
    {
        return edge.color;
    }
    palette(cover, PALETTE_SIZE).first().map_or(Rgb8::WHITE, |swatch| swatch.color)
}

/// Contrast between `ground` and the text the card sets on it: paper white, or black ink
/// ([`PRINTED_BLACK`]) — the choice of `Paint::text_on`.
pub(crate) fn text_contrast(ground: Rgb8) -> f64 {
    let text = if black_or_white(ground) == Rgb8::WHITE { Rgb8::WHITE } else { PRINTED_BLACK };
    contrast_ratio(text, ground)
}

/// What print has against `colour` as the ground of the whole outside, if anything.
/// `max_total_ink_percent` is the profile's limit; `link` the ICC transform the ground
/// will be written with (the naive device conversion is judged without one).
pub(crate) fn objection(
    colour: Rgb8,
    max_total_ink_percent: f32,
    link: Option<&SrgbToCmyk>,
) -> Option<Objection> {
    let limit = if max_total_ink_percent.is_finite() && max_total_ink_percent > 0.0 {
        max_total_ink_percent.min(GROUND_MAX_INK_PERCENT)
    } else {
        GROUND_MAX_INK_PERCENT
    };
    // The process build the ground would be written as, before any cap: a colour that
    // only fits once its ink has been cut is no longer the cover's colour.
    // Before any cleaning or cap: a near-black is K 100 alone, anything else the build
    // the transform (or, without one, the naive device formula) gives.
    let build = match link {
        _ if colour.to_lab().l < NEAR_BLACK_LSTAR => Color::BLACK,
        Some(link) => link.convert_color(colour),
        None => to_cmyk_naive(colour),
    };
    let (heaviest, total) = match build {
        Color::Cmyk { c, m, y, k } => (c.max(m).max(y).max(k) * 100.0, (c + m + y + k) * 100.0),
        Color::Gray(level) => ((1.0 - level) * 100.0, (1.0 - level) * 100.0),
        Color::Rgb { .. } | Color::Registration => (100.0, 400.0),
    };
    if heaviest < GROUND_MIN_TINT_PERCENT {
        Some(Objection::TooFaint)
    } else if total > limit + 1e-3 {
        Some(Objection::TooHeavy)
    } else if text_contrast(colour) < GROUND_MIN_CONTRAST {
        Some(Objection::NoLegibleText)
    } else {
        None
    }
}

/// The default ground of a card with a cover whose computed colour is `colour`: that
/// colour, or white when [`objection`] has something against it.
pub(crate) fn ground_from(
    colour: Rgb8,
    max_total_ink_percent: f32,
    link: Option<&SrgbToCmyk>,
) -> Rgb8 {
    match objection(colour, max_total_ink_percent, link) {
        None => colour,
        Some(_) => Rgb8::WHITE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `inner` framed by `frame` (10 % on every side).
    fn framed(frame: [u8; 3], inner: [u8; 3]) -> Pixmap {
        let (w, h) = (100u32, 160u32);
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let rim = x < 10 || x >= w - 10 || y < 10 || y >= h - 10;
                let [r, g, b] = if rim { frame } else { inner };
                data.extend_from_slice(&[r, g, b, 255]);
            }
        }
        Pixmap::from_rgba8(w, h, data).expect("consistent size")
    }

    /// A plain band along the left edge, then stripes of two other colours.
    fn banded(band: [u8; 3], a: [u8; 3], b: [u8; 3]) -> Pixmap {
        let (w, h) = (100u32, 160u32);
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let [r, g, bl] = if x < 12 {
                    band
                } else if y < 110 {
                    a
                } else {
                    b
                };
                data.extend_from_slice(&[r, g, bl, 255]);
            }
        }
        Pixmap::from_rgba8(w, h, data).expect("consistent size")
    }

    fn near(a: Rgb8, b: [u8; 3]) -> bool {
        let d = |x: u8, y: u8| x.abs_diff(y) <= 3;
        d(a.r, b[0]) && d(a.g, b[1]) && d(a.b, b[2])
    }

    #[test]
    fn the_colour_is_the_frame_then_the_binding_edge_then_the_dominant_swatch() {
        // A frame wins, whatever it encloses.
        let navy = [14, 22, 48];
        assert!(near(cover_colour(&framed(navy, [240, 120, 60])), navy));
        // No frame, but a plain binding edge.
        let cover = banded([120, 20, 30], [200, 210, 220], [30, 60, 90]);
        assert!(has_uniform_border(&cover).is_none());
        assert!(near(cover_colour(&cover), [120, 20, 30]));
        // Neither: the colour that covers most of the image (here the upper two thirds).
        let (w, h) = (100u32, 150u32);
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let stripe = (x / 5) % 2 == 0;
                let px = match (y < 100, stripe) {
                    (true, _) => [36, 84, 60],
                    (false, true) => [230, 200, 40],
                    (false, false) => [200, 60, 40],
                };
                data.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
        }
        let busy = Pixmap::from_rgba8(w, h, data).expect("consistent size");
        assert!(has_uniform_border(&busy).is_none());
        assert!(edge_uniformity(&busy, BINDING_EDGE, EDGE_STRIP_FRACTION) < UNIFORM_EDGE_THRESHOLD);
        assert!(near(cover_colour(&busy), [36, 84, 60]), "{}", cover_colour(&busy));
        // A flat image stands for its own colour; the same pixels always give the same.
        let flat = Pixmap::filled(40, 60, [31, 58, 95, 255]);
        assert_eq!(cover_colour(&flat), Rgb8::new(31, 58, 95));
        assert_eq!(cover_colour(&busy), cover_colour(&busy));
    }

    #[test]
    fn white_is_kept_when_ink_or_contrast_argue_for_it() {
        // Deep but not heavy: kept, with white text.
        for kept in [Rgb8::new(14, 22, 48), Rgb8::new(31, 58, 95), Rgb8::new(122, 31, 43)] {
            assert_eq!(objection(kept, 300.0, None), None, "{kept}");
            assert_eq!(ground_from(kept, 300.0, None), kept);
        }
        // Light but a real tint: kept, with black text.
        let cream = Rgb8::new(232, 220, 194);
        assert_eq!(ground_from(cream, 300.0, None), cream);
        // Near-black is written as K 100 alone: 100 % of ink, kept.
        assert_eq!(objection(Rgb8::new(8, 8, 10), 300.0, None), None);
        assert_eq!(objection(Rgb8::BLACK, 300.0, None), None);

        // Nearly white: no channel reaches 5 %.
        for faint in [Rgb8::WHITE, Rgb8::new(248, 246, 240), Rgb8::new(236, 240, 246)] {
            assert_eq!(objection(faint, 300.0, None), Some(Objection::TooFaint), "{faint}");
            assert_eq!(ground_from(faint, 300.0, None), Rgb8::WHITE);
        }
        // Dark and saturated: two full plates and most of a third.
        for heavy in [Rgb8::new(0, 0, 100), Rgb8::new(0, 60, 0), Rgb8::new(60, 10, 10)] {
            assert_eq!(objection(heavy, 300.0, None), Some(Objection::TooHeavy), "{heavy}");
            assert_eq!(ground_from(heavy, 300.0, None), Rgb8::WHITE);
        }
        // A profile with a lower limit lowers the bar; a nonsensical one changes nothing.
        let navy = Rgb8::new(14, 22, 48); // about 206 %
        assert_eq!(objection(navy, 200.0, None), Some(Objection::TooHeavy));
        assert_eq!(objection(navy, f32::NAN, None), None);
        assert_eq!(objection(navy, -1.0, None), None);
        assert_eq!(objection(navy, f32::INFINITY, None), None);
        // Saturated mid-tones: neither black ink nor paper white reads well on them.
        for mid in [Rgb8::new(60, 120, 200), Rgb8::new(220, 60, 40), Rgb8::new(40, 140, 60)] {
            assert_eq!(objection(mid, 300.0, None), Some(Objection::NoLegibleText), "{mid}");
            assert_eq!(ground_from(mid, 300.0, None), Rgb8::WHITE);
        }
    }

    #[test]
    fn every_ground_the_rule_can_produce_carries_legible_text_within_the_ink_limit() {
        let mut kept = 0;
        let mut white = 0;
        for r in (0..=255).step_by(15) {
            for g in (0..=255).step_by(15) {
                for b in (0..=255).step_by(15) {
                    let colour = Rgb8::new(r as u8, g as u8, b as u8);
                    let ground = ground_from(colour, 300.0, None);
                    // What `Paint::text_on` sets on it, against ideal and printed black.
                    let ideal = contrast_ratio(black_or_white(ground), ground);
                    assert!(ideal >= GROUND_MIN_CONTRAST, "{colour} -> {ground}: {ideal}");
                    assert!(text_contrast(ground) >= GROUND_MIN_CONTRAST, "{colour} -> {ground}");
                    let ink = ebc_color::to_cmyk_ink_safe(ground, 300.0).total_ink_percent();
                    assert!(
                        ink.is_some_and(|ink| ink <= GROUND_MAX_INK_PERCENT + 1e-3),
                        "{colour} -> {ground}: {ink:?}"
                    );
                    if ground == colour {
                        kept += 1;
                    } else {
                        assert_eq!(ground, Rgb8::WHITE);
                        white += 1;
                    }
                }
            }
        }
        // The rule is neither "always white" nor "anything goes".
        assert!(kept > 1500 && white > 500, "kept {kept}, white {white}");
    }

    #[cfg(feature = "bundled-fogra39")]
    #[test]
    fn grounds_judged_through_the_output_profile_stay_within_the_ink_limit() {
        let link = crate::OutputProfile::bundled_fogra39()
            .and_then(|p| {
                p.colour_link().map_err(|e| crate::CardError::InvalidOutputProfile(e.to_string()))
            })
            .expect("the bundled profile is readable");
        let paint = crate::paint::Paint::for_profile(
            &ebc_layout::PrintProfile::upload_eu(),
            Some(link.clone()),
        );
        let (mut kept, mut white) = (0, 0);
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let colour = Rgb8::new(r as u8, g as u8, b as u8);
                    let ground = ground_from(colour, 300.0, Some(&link));
                    if ground == colour {
                        kept += 1;
                        let ink = paint.flat(ground).total_ink_percent();
                        assert!(
                            ink.is_some_and(|ink| ink <= GROUND_MAX_INK_PERCENT + 1e-3),
                            "{colour}: {ink:?}"
                        );
                    } else {
                        white += 1;
                    }
                }
            }
        }
        assert!(kept > 300 && white > 300, "kept {kept}, white {white}");
        // Paper-like colours are too faint through the profile as well.
        assert_eq!(
            objection(Rgb8::new(253, 253, 252), 300.0, Some(&link)),
            Some(Objection::TooFaint)
        );
    }
}
