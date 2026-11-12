//! WCAG 2.x contrast and text-colour selection.

use crate::Rgb8;
use crate::lab::{Lab, srgb_to_linear};

/// WCAG 2.x relative luminance of an sRGB colour: 0.0 for black, 1.0 for white.
///
/// `L = 0.2126·R + 0.7152·G + 0.0722·B` on linear-light channels. WCAG 2.0 printed the
/// linearisation threshold as 0.03928 where sRGB says 0.04045; no 8-bit value falls
/// between the two, so the result is the same either way.
#[must_use]
pub fn relative_luminance(c: Rgb8) -> f64 {
    let (r, g, b) = (srgb_to_linear(c.r), srgb_to_linear(c.g), srgb_to_linear(c.b));
    // Same polynomial with the green weight written as 1 − 0.2126 − 0.0722, so that a
    // neutral grey's luminance is exactly its linear value (white is exactly 1.0).
    0.2126 * (r - g) + 0.0722 * (b - g) + g
}

/// WCAG 2.x contrast ratio between two colours, from 1.0 (identical luminance) to 21.0
/// (black on white). The order of the arguments does not matter.
///
/// WCAG asks for at least 4.5:1 for body text (AA), 3:1 for large text and 7:1 for AAA.
///
/// ```
/// use ebc_color::{Rgb8, contrast_ratio};
/// assert_eq!(contrast_ratio(Rgb8::BLACK, Rgb8::WHITE), 21.0);
/// let grey = Rgb8::from_hex("#777").expect("valid hex");
/// assert!((contrast_ratio(grey, Rgb8::WHITE) - 4.48).abs() < 0.005);
/// ```
#[must_use]
pub fn contrast_ratio(a: Rgb8, b: Rgb8) -> f64 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    ((lighter + 0.05) / (darker + 0.05)).clamp(1.0, 21.0)
}

/// Black or white, whichever contrasts more with `background` (white on a tie).
///
/// The result always reaches at least 4.58:1, so it satisfies WCAG AA for any background.
#[must_use]
pub fn black_or_white(background: Rgb8) -> Rgb8 {
    if contrast_ratio(Rgb8::BLACK, background) > contrast_ratio(Rgb8::WHITE, background) {
        Rgb8::BLACK
    } else {
        Rgb8::WHITE
    }
}

/// Picks a text colour for `background`.
///
/// Returns the **first** entry of `candidates` whose [`contrast_ratio`] with `background`
/// is at least `min_ratio` — so list the candidates in order of preference (for instance
/// the cover's own palette, title colour first). When no candidate qualifies (or the list
/// is empty, or `min_ratio` is NaN or above 21) it falls back to [`black_or_white`], which
/// is the best contrast available but may itself be below `min_ratio` if that exceeds
/// about 4.58.
#[must_use]
pub fn pick_text_color(background: Rgb8, candidates: &[Rgb8], min_ratio: f64) -> Rgb8 {
    candidates
        .iter()
        .copied()
        .find(|&c| contrast_ratio(c, background) >= min_ratio)
        .unwrap_or_else(|| black_or_white(background))
}

/// Returns `foreground` if it already reaches `min_ratio` against `background`; otherwise
/// the closest lighter or darker version of it that does.
///
/// The colour keeps its CIELAB a\*/b\* (its hue, as far as the sRGB gamut allows) while
/// L\* moves in steps of 1 towards black or white — whichever of the two contrasts more
/// with the background. If even the extreme does not reach `min_ratio`, the result is
/// [`black_or_white`]. Use it to keep a cover's accent colour on the spine while
/// guaranteeing legibility.
#[must_use]
pub fn ensure_contrast(foreground: Rgb8, background: Rgb8, min_ratio: f64) -> Rgb8 {
    if contrast_ratio(foreground, background) >= min_ratio {
        return foreground;
    }
    let extreme = black_or_white(background);
    let lab = Lab::from_rgb8(foreground);
    let towards_white = extreme == Rgb8::WHITE;
    let mut l = lab.l.clamp(0.0, 100.0);
    for _ in 0..100 {
        l = if towards_white { (l + 1.0).min(100.0) } else { (l - 1.0).max(0.0) };
        let candidate = Lab::new(l, lab.a, lab.b).to_rgb8();
        if contrast_ratio(candidate, background) >= min_ratio {
            return candidate;
        }
        if l <= 0.0 || l >= 100.0 {
            break;
        }
    }
    extreme
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Rgb8 {
        Rgb8::from_hex(s).expect("valid hex literal in test")
    }

    #[test]
    fn known_wcag_ratios() {
        // (foreground, background, ratio, tolerance) — values published by WebAIM and the
        // WCAG techniques.
        let cases = [
            ("#000000", "#ffffff", 21.0, 1e-9),
            ("#777777", "#ffffff", 4.48, 0.005), // the classic "just fails AA" grey
            ("#767676", "#ffffff", 4.54, 0.005), // the lightest grey that passes AA on white
            ("#595959", "#ffffff", 7.0, 0.005),  // the lightest grey that passes AAA on white
            ("#949494", "#ffffff", 3.03, 0.005), // the lightest grey that passes 3:1
            ("#ff0000", "#ffffff", 3.998, 0.001), // pure red narrowly fails 4:1
            ("#0000ff", "#ffffff", 8.59, 0.005),
            ("#00ff00", "#ffffff", 1.37, 0.005),
            ("#ffff00", "#ffffff", 1.07, 0.005),
            ("#ffff00", "#000000", 19.56, 0.005),
            ("#0000ff", "#000000", 2.44, 0.005),
        ];
        for (fg, bg, expected, tol) in cases {
            let ratio = contrast_ratio(hex(fg), hex(bg));
            assert!((ratio - expected).abs() <= tol, "{fg} on {bg}: {ratio} vs {expected}");
        }
        assert_eq!(contrast_ratio(Rgb8::BLACK, Rgb8::WHITE), 21.0);
        assert!(contrast_ratio(hex("#777777"), Rgb8::WHITE) < 4.5);
        assert!(contrast_ratio(hex("#767676"), Rgb8::WHITE) >= 4.5);
        assert!(contrast_ratio(hex("#595959"), Rgb8::WHITE) >= 7.0);
        assert!(contrast_ratio(hex("#5a5a5a"), Rgb8::WHITE) < 7.0);
    }

    #[test]
    fn luminance_anchors() {
        assert_eq!(relative_luminance(Rgb8::BLACK), 0.0);
        assert!((relative_luminance(Rgb8::WHITE) - 1.0).abs() < 1e-12);
        assert!((relative_luminance(Rgb8::new(255, 0, 0)) - 0.2126).abs() < 1e-12);
        assert!((relative_luminance(Rgb8::new(0, 255, 0)) - 0.7152).abs() < 1e-12);
        assert!((relative_luminance(Rgb8::new(0, 0, 255)) - 0.0722).abs() < 1e-12);
        assert!((Rgb8::new(119, 119, 119).relative_luminance() - 0.18448).abs() < 1e-5);
    }

    #[test]
    fn ratio_is_symmetric_bounded_and_one_for_identical_colours() {
        let mut state = 7u64;
        for _ in 0..5_000 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695);
            let a = Rgb8::from_key((state >> 40) as u32);
            let b = Rgb8::from_key((state >> 16) as u32);
            let r = contrast_ratio(a, b);
            assert_eq!(r, contrast_ratio(b, a));
            assert!((1.0..=21.0).contains(&r));
            assert_eq!(contrast_ratio(a, a), 1.0);
        }
    }

    #[test]
    fn luminance_is_monotonic_in_every_channel() {
        for v in 1..=255u8 {
            assert!(
                Rgb8::new(v, 0, 0).relative_luminance()
                    > Rgb8::new(v - 1, 0, 0).relative_luminance()
            );
            assert!(
                Rgb8::new(9, v, 0).relative_luminance()
                    > Rgb8::new(9, v - 1, 0).relative_luminance()
            );
            assert!(
                Rgb8::new(9, 9, v).relative_luminance()
                    > Rgb8::new(9, 9, v - 1).relative_luminance()
            );
        }
    }

    #[test]
    fn black_or_white_always_passes_aa() {
        let mut worst = f64::INFINITY;
        for r in (0..=255u16).step_by(3) {
            for g in (0..=255u16).step_by(3) {
                for b in (0..=255u16).step_by(15) {
                    let bg = Rgb8::new(r as u8, g as u8, b as u8);
                    let pick = black_or_white(bg);
                    let ratio = contrast_ratio(pick, bg);
                    let other = if pick == Rgb8::BLACK { Rgb8::WHITE } else { Rgb8::BLACK };
                    assert!(ratio >= contrast_ratio(other, bg), "{bg}");
                    worst = worst.min(ratio);
                }
            }
        }
        // Theoretical minimum: sqrt(21) ≈ 4.583, where black and white tie.
        assert!((4.58..4.62).contains(&worst), "worst case {worst}");
        assert_eq!(black_or_white(Rgb8::WHITE), Rgb8::BLACK);
        assert_eq!(black_or_white(Rgb8::BLACK), Rgb8::WHITE);
        assert_eq!(black_or_white(hex("#ffff00")), Rgb8::BLACK);
        assert_eq!(black_or_white(hex("#0000ff")), Rgb8::WHITE);
    }

    #[test]
    fn pick_returns_the_first_qualifying_candidate_in_caller_order() {
        let bg = hex("#102040");
        let dim = hex("#203050"); // too close to the background
        let cream = hex("#f5ecd0");
        let yellow = hex("#ffd700");
        assert_eq!(pick_text_color(bg, &[dim, cream, yellow], 4.5), cream);
        assert_eq!(pick_text_color(bg, &[dim, yellow, cream], 4.5), yellow);
        // A lower bar lets an earlier candidate through.
        assert_eq!(pick_text_color(bg, &[dim, cream], 1.0), dim);
    }

    #[test]
    fn pick_falls_back_to_black_or_white() {
        let bg = hex("#f0f0f0");
        assert_eq!(pick_text_color(bg, &[], 4.5), Rgb8::BLACK);
        assert_eq!(pick_text_color(bg, &[hex("#eeeeee"), hex("#cccccc")], 4.5), Rgb8::BLACK);
        assert_eq!(pick_text_color(hex("#202020"), &[hex("#333333")], 4.5), Rgb8::WHITE);
        // Unreachable or nonsensical thresholds never panic and still give the best option.
        for bad in [22.0, f64::INFINITY, f64::NAN] {
            assert_eq!(pick_text_color(bg, &[Rgb8::BLACK, hex("#123456")], bad), Rgb8::BLACK);
        }
        // A negative threshold accepts anything, including the background itself.
        assert_eq!(pick_text_color(bg, &[bg], f64::NEG_INFINITY), bg);
    }

    #[test]
    fn ensure_contrast_keeps_good_colours_and_fixes_bad_ones() {
        let bg = hex("#7a1f1f"); // dark red cloth
        let gold = hex("#d4af37");
        assert_eq!(ensure_contrast(gold, bg, 3.0), gold, "already readable: unchanged");

        let maroon = hex("#8a3030"); // nearly the background
        let fixed = ensure_contrast(maroon, bg, 4.5);
        assert!(contrast_ratio(fixed, bg) >= 4.5);
        assert_ne!(fixed, Rgb8::WHITE, "a lighter red exists before reaching pure white");
        assert!(fixed.r > fixed.g && fixed.r > fixed.b, "still reddish: {fixed}");
        // Minimal change: one L* step less would not have been enough.
        let lab = fixed.to_lab();
        let less = Lab::new(lab.l - 1.5, lab.a, lab.b).to_rgb8();
        assert!(contrast_ratio(less, bg) < 4.5 + 0.2);
    }

    #[test]
    fn ensure_contrast_meets_the_ratio_whenever_black_or_white_can() {
        let mut state = 99u64;
        for _ in 0..600 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695);
            let fg = Rgb8::from_key((state >> 40) as u32);
            let bg = Rgb8::from_key((state >> 16) as u32);
            for min in [3.0, 4.5] {
                let out = ensure_contrast(fg, bg, min);
                assert!(contrast_ratio(out, bg) >= min, "{fg} on {bg} -> {out}");
            }
        }
        // Impossible request: best effort, no panic, no endless loop.
        let out = ensure_contrast(hex("#808080"), hex("#777777"), 15.0);
        assert_eq!(out, black_or_white(hex("#777777")));
        assert_eq!(ensure_contrast(hex("#808080"), hex("#777777"), f64::NAN), Rgb8::BLACK);
    }
}
