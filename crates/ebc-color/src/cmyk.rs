//! Conversions between [`Rgb8`] and the print pipeline's [`ebc_scene::Color`].

use ebc_scene::Color;

use crate::Rgb8;
use crate::lab::Lab;

/// Converts an sRGB colour to device CMYK with **full grey-component replacement**.
///
/// The grey component — the amount of ink common to cyan, magenta and yellow — is moved
/// entirely onto the black plate:
///
/// ```text
/// c' = 1 − r      m' = 1 − g      y' = 1 − b          (r, g, b in 0..=1)
/// k  = min(c', m', y')
/// c  = (c' − k) / (1 − k)   (likewise m, y;  all 0 when k = 1)
/// ```
///
/// Consequences that matter for print:
///
/// * black is K 100 only ([`Color::BLACK`]) and every neutral grey is a K-only tint, so text
///   and rules never depend on plate registration;
/// * at least one of C, M, Y is always 0, so total ink never exceeds 300 %. **It gets
///   very close, though**: the maximum over all 16.7 million colours is 299.6 %, at
///   `#000001` (C 100 M 100 K 99.6) — see below. Pass the result through
///   [`limit_total_ink`] with the print profile's limit (260–280 % is common on uncoated
///   stock), or use [`to_cmyk_ink_safe`], which does that for you.
///
/// **Near-black colours become heavy three-plate builds.** The division by `1 − k`
/// amplifies tiny channel differences as `k` approaches 1: `#0a0000`, which no eye tells
/// from black (L\* ≈ 0.6), converts to M 100 Y 100 K 96 — 296 % on three plates — while
/// `#000000` is K 100 alone. Colours taken from a dark cover's palette hit this routinely,
/// and it defeats the "text in K only" rule printers ask for. The formula is kept because
/// it is the exact inverse of [`Color::to_srgb_approx`]; [`to_cmyk_ink_safe`] is the
/// conversion to use for text, rules and QR codes.
///
/// **This is not a colour-managed conversion.** No ICC profile, dot gain or ink behaviour
/// is modelled; the printed colour of chromatic values will differ from the screen colour
/// (saturated blues and greens most of all). Use it for text, rules and flat tints, not for
/// photographs.
///
/// ```
/// use ebc_color::{Rgb8, to_cmyk_naive};
/// use ebc_scene::Color;
/// assert_eq!(to_cmyk_naive(Rgb8::BLACK), Color::BLACK);
/// assert_eq!(to_cmyk_naive(Rgb8::new(255, 0, 0)), Color::Cmyk { c: 0.0, m: 1.0, y: 1.0, k: 0.0 });
/// ```
#[must_use]
pub fn to_cmyk_naive(c: Rgb8) -> Color {
    to_cmyk_gcr(c, 1.0)
}

/// Like [`to_cmyk_naive`] but moves only the fraction `gcr` (0.0–1.0) of the grey component
/// onto the black plate; the rest stays as equal parts of C, M and Y.
///
/// `gcr = 1.0` is [`to_cmyk_naive`]; `gcr = 0.0` yields a CMY-only colour (K = 0). Values
/// outside the range are clamped and NaN is treated as 1.0. Whatever the value, the colour
/// maps back to the same sRGB through [`Color::to_srgb_approx`]. Equally uncalibrated.
#[must_use]
pub fn to_cmyk_gcr(c: Rgb8, gcr: f32) -> Color {
    let gcr = if gcr.is_nan() { 1.0 } else { gcr.clamp(0.0, 1.0) };
    let c0 = 1.0 - f32::from(c.r) / 255.0;
    let m0 = 1.0 - f32::from(c.g) / 255.0;
    let y0 = 1.0 - f32::from(c.b) / 255.0;
    let k = c0.min(m0).min(y0) * gcr;
    if k >= 1.0 {
        return Color::BLACK;
    }
    let rest = 1.0 - k;
    Color::cmyk((c0 - k) / rest, (m0 - k) / rest, (y0 - k) / rest, k)
}

/// CIELAB lightness below which [`to_cmyk_ink_safe`] prints a colour as plain black.
///
/// L\* 5 is darker than `#111111`; next to real black ink on paper such colours are not
/// told apart, whereas their naive CMYK builds reach 250–300 % ink on three plates.
pub const NEAR_BLACK_LSTAR: f64 = 5.0;

/// [`to_cmyk_naive`] made safe for small print elements (text, rules, marks):
///
/// * a colour darker than [`NEAR_BLACK_LSTAR`] becomes [`Color::BLACK`] (K 100 only), so
///   "almost black" never turns into a three-plate build that needs perfect registration;
/// * every other colour is converted with [`to_cmyk_naive`], then capped at
///   `max_ink_percent` and cleaned of tints under 5 % by [`finish_flat`] (a NaN limit
///   means "no cap").
///
/// The result is therefore no longer an exact inverse of [`Color::to_srgb_approx`] for the
/// colours it changes. Equally uncalibrated (no ICC profile).
///
/// ```
/// use ebc_color::{Rgb8, to_cmyk_ink_safe};
/// use ebc_scene::Color;
/// assert_eq!(to_cmyk_ink_safe(Rgb8::new(10, 0, 0), 300.0), Color::BLACK);
/// let navy = to_cmyk_ink_safe(Rgb8::new(0, 0, 60), 240.0);
/// assert!(navy.total_ink_percent().is_some_and(|ink| ink <= 240.001));
/// ```
#[must_use]
pub fn to_cmyk_ink_safe(c: Rgb8, max_ink_percent: f32) -> Color {
    if Lab::from_rgb8(c).l < NEAR_BLACK_LSTAR {
        return Color::BLACK;
    }
    finish_flat(to_cmyk_naive(c), max_ink_percent)
}

/// A process channel below this fraction (2.5 %) of a flat colour is dropped by
/// [`clean_tints`].
pub const TINT_DROP_BELOW: f32 = 0.025;
/// The lightest tint a flat colour may carry on a process channel: 5 %
/// (Pixartprinting: "tints never below 5 %"; `docs/research/03-print-requirements.md`
/// §3.4 and §4.3). [`clean_tints`] raises channels from 2.5 % up to it.
pub const MIN_TINT: f32 = 0.05;

/// Cleans the light tints of a flat CMYK colour (house rule of
/// `docs/research/03-print-requirements.md` §3.4, implication 24): every channel below
/// [`TINT_DROP_BELOW`] (2.5 %) becomes 0, every channel from 2.5 % up to [`MIN_TINT`]
/// (5 %) becomes 5 %. Tints that light print unreliably or not at all. Other colours
/// are returned unchanged.
///
/// ```
/// use ebc_color::clean_tints;
/// use ebc_scene::Color;
/// assert_eq!(clean_tints(Color::cmyk(0.02, 0.04, 0.2, 0.0)), Color::cmyk(0.0, 0.05, 0.2, 0.0));
/// ```
#[must_use]
pub fn clean_tints(color: Color) -> Color {
    let Color::Cmyk { c, m, y, k } = color else {
        return color;
    };
    let clean = |v: f32| {
        if v.is_nan() || v < TINT_DROP_BELOW {
            0.0
        } else if v < MIN_TINT {
            MIN_TINT
        } else {
            v
        }
    };
    Color::cmyk(clean(c), clean(m), clean(y), clean(k))
}

/// The last step for every flat process colour (grounds, spine fills): the total ink is
/// capped with [`limit_total_ink`], the tints are cleaned with [`clean_tints`], and, if
/// raising a tint to 5 % pushed the total back over the limit, the excess is taken off
/// the heaviest of C, M and Y. The result has no channel strictly between 0 and 5 % and
/// respects the limit (a NaN limit means "no cap").
///
/// ```
/// use ebc_color::finish_flat;
/// use ebc_scene::Color;
/// let c = finish_flat(Color::cmyk(0.9, 0.9, 0.9, 0.9), 300.0);
/// assert!(c.total_ink_percent().is_some_and(|ink| ink <= 300.001));
/// ```
#[must_use]
pub fn finish_flat(color: Color, max_ink_percent: f32) -> Color {
    let cleaned = clean_tints(limit_total_ink(color, max_ink_percent));
    let Color::Cmyk { c, m, y, k } = cleaned else {
        return cleaned;
    };
    let limit = max_ink_percent / 100.0;
    let excess = c + m + y + k - limit;
    if limit.is_nan() || excess <= 1e-6 {
        return cleaned;
    }
    let mut channels = [c, m, y];
    let heaviest = (0..3).fold(0, |best, i| if channels[i] > channels[best] { i } else { best });
    let reduced = channels[heaviest] - excess;
    channels[heaviest] = if reduced >= MIN_TINT { reduced } else { 0.0 };
    Color::cmyk(channels[0], channels[1], channels[2], k)
}

/// The colour as an sRGB [`Color::Rgb`], left to the printer's colour management.
#[must_use]
pub fn to_scene_rgb(c: Rgb8) -> Color {
    Color::rgb8(c.r, c.g, c.b)
}

/// The nearest 8-bit sRGB colour to a scene colour, through [`Color::to_srgb_approx`]
/// (exact for RGB and grey, the naive device formula for CMYK, black for registration).
/// For contrast estimates and previews only.
#[must_use]
pub fn from_scene_color(color: &Color) -> Rgb8 {
    let [r, g, b] = color.to_srgb_approx();
    let quantise = |v: f32| {
        // `as u8` saturates and maps NaN to 0, so out-of-range components cannot panic.
        (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    };
    Rgb8::new(quantise(r), quantise(g), quantise(b))
}

/// Reduces a CMYK colour's total ink coverage to at most `max_percent` (e.g. 300 for coated
/// stock, 260–280 for uncoated — see the print profile).
///
/// Black is kept and C, M and Y are scaled down by a common factor, which darkens nothing
/// and keeps the hue roughly stable. Colours already within the limit, non-CMYK colours and
/// a NaN limit are returned unchanged. A limit below the black component alone removes all
/// of C, M and Y but never touches K.
#[must_use]
pub fn limit_total_ink(color: Color, max_percent: f32) -> Color {
    let Color::Cmyk { c, m, y, k } = color else {
        return color;
    };
    let limit = max_percent / 100.0;
    let cmy = c + m + y;
    // The small slack keeps a colour sitting exactly on the limit from being rescaled by
    // a rounding error.
    if limit.is_nan() || cmy + k <= limit + 1e-6 || cmy <= 0.0 {
        return color;
    }
    let scale = ((limit - k) / cmy).clamp(0.0, 1.0);
    Color::cmyk(c * scale, m * scale, y * scale, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(color: Color) -> (f32, f32, f32, f32) {
        match color {
            Color::Cmyk { c, m, y, k } => (c, m, y, k),
            other => panic!("expected CMYK, got {other:?}"),
        }
    }

    #[test]
    fn anchors() {
        assert_eq!(to_cmyk_naive(Rgb8::BLACK), Color::BLACK);
        assert_eq!(to_cmyk_naive(Rgb8::WHITE), Color::WHITE);
        assert_eq!(parts(to_cmyk_naive(Rgb8::new(255, 0, 0))), (0.0, 1.0, 1.0, 0.0));
        assert_eq!(parts(to_cmyk_naive(Rgb8::new(0, 255, 0))), (1.0, 0.0, 1.0, 0.0));
        assert_eq!(parts(to_cmyk_naive(Rgb8::new(0, 0, 255))), (1.0, 1.0, 0.0, 0.0));
        assert_eq!(parts(to_cmyk_naive(Rgb8::new(0, 255, 255))), (1.0, 0.0, 0.0, 0.0));
        assert_eq!(parts(to_cmyk_naive(Rgb8::new(255, 0, 255))), (0.0, 1.0, 0.0, 0.0));
        assert_eq!(parts(to_cmyk_naive(Rgb8::new(255, 255, 0))), (0.0, 0.0, 1.0, 0.0));
        // Dark red: the grey component (here 1 − 0.5) goes to K.
        let (c, m, y, k) = parts(to_cmyk_naive(Rgb8::new(128, 0, 0)));
        assert_eq!((c, m, y), (0.0, 1.0, 1.0));
        assert!((k - (1.0 - 128.0 / 255.0)).abs() < 1e-6);
    }

    #[test]
    fn greys_print_on_the_black_plate_only() {
        for v in 0..=255u8 {
            let (c, m, y, k) = parts(to_cmyk_naive(Rgb8::new(v, v, v)));
            assert_eq!((c, m, y), (0.0, 0.0, 0.0), "grey {v}");
            assert!((k - (1.0 - f32::from(v) / 255.0)).abs() < 1e-6);
        }
    }

    #[test]
    fn full_gcr_always_leaves_one_plate_empty_and_stays_under_300_percent() {
        for r in (0..=255u16).step_by(5) {
            for g in (0..=255u16).step_by(5) {
                for b in (0..=255u16).step_by(5) {
                    let rgb = Rgb8::new(r as u8, g as u8, b as u8);
                    let color = to_cmyk_naive(rgb);
                    let (c, m, y, k) = parts(color);
                    for v in [c, m, y, k] {
                        assert!((0.0..=1.0).contains(&v), "{rgb}: {color:?}");
                    }
                    assert_eq!(c.min(m).min(y), 0.0, "{rgb}: {color:?}");
                    let ink = color.total_ink_percent().expect("CMYK has an ink total");
                    assert!(ink <= 300.0 + 1e-3, "{rgb}: {ink}");
                }
            }
        }
    }

    /// Audit finding: the implementer's report said "can reach 280 %"; the true maximum
    /// is 299.6 %, and it is reached next to black. All colours with any channel ≤ 1 are
    /// scanned (the maximum needs k = 254/255 or one empty channel pair), plus a lattice.
    #[test]
    fn documented_ink_maximum_and_near_black_behaviour() {
        let ink = |rgb: Rgb8| to_cmyk_naive(rgb).total_ink_percent().expect("CMYK");
        let mut worst = (0.0_f32, Rgb8::BLACK);
        let mut visit = |rgb: Rgb8| {
            let v = ink(rgb);
            if v > worst.0 {
                worst = (v, rgb);
            }
        };
        for a in 0..=255u8 {
            for b in 0..=255u8 {
                for low in 0..=1u8 {
                    visit(Rgb8::new(low, a, b));
                    visit(Rgb8::new(a, low, b));
                    visit(Rgb8::new(a, b, low));
                }
            }
        }
        for r in (0..=255u16).step_by(5) {
            for g in (0..=255u16).step_by(5) {
                for b in (0..=255u16).step_by(5) {
                    visit(Rgb8::new(r as u8, g as u8, b as u8));
                }
            }
        }
        assert!((worst.0 - 299.608).abs() < 0.01, "maximum {} at {}", worst.0, worst.1);
        // The documented examples.
        assert!((ink(Rgb8::new(0, 0, 1)) - 299.608).abs() < 0.01);
        let (c, m, y, k) = parts(to_cmyk_naive(Rgb8::new(10, 0, 0)));
        assert_eq!((c, m, y), (0.0, 1.0, 1.0));
        assert!((k - 245.0 / 255.0).abs() < 1e-6);
        assert!(Rgb8::new(10, 0, 0).to_lab().l < 1.0, "visually black");
    }

    #[test]
    fn ink_safe_conversion_snaps_near_blacks_and_caps_ink() {
        // Near-black tints print on the black plate alone.
        for rgb in [Rgb8::BLACK, Rgb8::new(0, 0, 1), Rgb8::new(10, 0, 0), Rgb8::new(12, 12, 16)] {
            assert_eq!(to_cmyk_ink_safe(rgb, 300.0), Color::BLACK, "{rgb}");
        }
        // Everything else is the naive conversion under the cap, for every limit.
        for limit in [200.0_f32, 240.0, 260.0, 300.0, f32::NAN] {
            for r in (0..=255u16).step_by(17) {
                for g in (0..=255u16).step_by(17) {
                    for b in (0..=255u16).step_by(17) {
                        let rgb = Rgb8::new(r as u8, g as u8, b as u8);
                        let safe = to_cmyk_ink_safe(rgb, limit);
                        let ink = safe.total_ink_percent().expect("CMYK");
                        if !limit.is_nan() {
                            assert!(ink <= limit + 1e-3, "{rgb}: {ink} > {limit}");
                        }
                        if rgb.to_lab().l >= NEAR_BLACK_LSTAR {
                            assert_eq!(safe, finish_flat(to_cmyk_naive(rgb), limit));
                        } else {
                            assert_eq!(safe, Color::BLACK);
                        }
                    }
                }
            }
        }
        // Colours that are clearly not black are left alone under a generous limit.
        let red = Rgb8::new(200, 30, 30);
        assert_eq!(to_cmyk_ink_safe(red, 300.0), to_cmyk_naive(red));
        assert_eq!(to_cmyk_ink_safe(Rgb8::WHITE, 0.0), Color::WHITE);
    }

    #[test]
    fn every_gcr_level_round_trips_through_the_scene_approximation() {
        for gcr in [0.0, 0.25, 0.5, 0.8, 1.0] {
            for r in (0..=255u16).step_by(15) {
                for g in (0..=255u16).step_by(15) {
                    for b in (0..=255u16).step_by(15) {
                        let rgb = Rgb8::new(r as u8, g as u8, b as u8);
                        assert_eq!(from_scene_color(&to_cmyk_gcr(rgb, gcr)), rgb, "gcr {gcr}");
                    }
                }
            }
        }
    }

    #[test]
    fn gcr_level_controls_the_black_plate() {
        let brown = Rgb8::new(120, 80, 40);
        let (_, _, _, k_full) = parts(to_cmyk_gcr(brown, 1.0));
        let (c_half, m_half, y_half, k_half) = parts(to_cmyk_gcr(brown, 0.5));
        let (c_none, m_none, y_none, k_none) = parts(to_cmyk_gcr(brown, 0.0));
        assert!((k_half - k_full / 2.0).abs() < 1e-6);
        assert_eq!(k_none, 0.0);
        assert!(c_none > c_half && m_none > m_half && y_none > y_half);
        assert!((c_none - (1.0 - 120.0 / 255.0)).abs() < 1e-6);
        // Hostile parameters: clamped, NaN means "full".
        assert_eq!(to_cmyk_gcr(brown, 7.0), to_cmyk_gcr(brown, 1.0));
        assert_eq!(to_cmyk_gcr(brown, -3.0), to_cmyk_gcr(brown, 0.0));
        assert_eq!(to_cmyk_gcr(brown, f32::NAN), to_cmyk_naive(brown));
        assert_eq!(to_cmyk_gcr(Rgb8::BLACK, 0.0), Color::cmyk(1.0, 1.0, 1.0, 0.0));
    }

    #[test]
    fn scene_rgb_conversion_round_trips() {
        assert_eq!(to_scene_rgb(Rgb8::new(255, 0, 51)), Color::Rgb { r: 1.0, g: 0.0, b: 0.2 });
        for v in 0..=255u8 {
            let rgb = Rgb8::new(v, 255 - v, v / 2);
            assert_eq!(from_scene_color(&to_scene_rgb(rgb)), rgb);
        }
        assert_eq!(from_scene_color(&Color::Registration), Rgb8::BLACK);
        assert_eq!(from_scene_color(&Color::gray(1.0)), Rgb8::WHITE);
        assert_eq!(from_scene_color(&Color::BLACK), Rgb8::BLACK);
        // Hand-built out-of-range components (bypassing the clamping constructors).
        let wild = Color::Rgb { r: f32::NAN, g: 9.0, b: -4.0 };
        assert_eq!(from_scene_color(&wild), Rgb8::new(0, 255, 0));
    }

    #[test]
    fn flat_colours_carry_no_tint_under_five_percent() {
        assert_eq!(
            clean_tints(Color::cmyk(0.024, 0.025, 0.049, 0.05)),
            Color::cmyk(0.0, 0.05, 0.05, 0.05)
        );
        assert_eq!(
            clean_tints(Color::cmyk(f32::NAN, 0.3, 0.0, 1.0)),
            Color::cmyk(0.0, 0.3, 0.0, 1.0)
        );
        assert_eq!(clean_tints(Color::gray(0.99)), Color::gray(0.99));
        for limit in [200.0_f32, 260.0, 300.0, f32::NAN] {
            for r in (0..=255u16).step_by(15) {
                for g in (0..=255u16).step_by(15) {
                    for b in (0..=255u16).step_by(15) {
                        let rgb = Rgb8::new(r as u8, g as u8, b as u8);
                        let (c, m, y, k) = parts(to_cmyk_ink_safe(rgb, limit));
                        for v in [c, m, y, k] {
                            assert!(v == 0.0 || v >= MIN_TINT, "{rgb}: {v}");
                        }
                        if !limit.is_nan() {
                            assert!((c + m + y + k) * 100.0 <= limit + 1e-3, "{rgb}");
                        }
                    }
                }
            }
        }
        // Raising a tint to 5 % at the limit takes the excess off the heaviest plate.
        let (c, m, y, k) = parts(finish_flat(Color::cmyk(0.03, 0.9, 0.9, 0.9), 270.0));
        assert_eq!((c, k), (MIN_TINT, 0.9));
        assert!(((c + m + y + k) - 2.7).abs() < 1e-5 && m.min(y) >= MIN_TINT);
    }

    #[test]
    fn ink_limit_scales_cmy_and_keeps_black() {
        let heavy = Color::cmyk(1.0, 1.0, 0.0, 0.8); // 280 %
        let (c, m, y, k) = parts(limit_total_ink(heavy, 240.0));
        assert_eq!((y, k), (0.0, 0.8));
        assert!((c - 0.8).abs() < 1e-6 && (m - 0.8).abs() < 1e-6);
        let total = limit_total_ink(heavy, 240.0).total_ink_percent().expect("CMYK");
        assert!((total - 240.0).abs() < 1e-3);

        assert_eq!(limit_total_ink(heavy, 300.0), heavy, "already within the limit");
        assert_eq!(limit_total_ink(heavy, 280.0), heavy, "exactly at the limit");
        assert_eq!(limit_total_ink(heavy, 50.0), Color::cmyk(0.0, 0.0, 0.0, 0.8));
        assert_eq!(limit_total_ink(heavy, -10.0), Color::cmyk(0.0, 0.0, 0.0, 0.8));
        assert_eq!(limit_total_ink(heavy, f32::NAN), heavy);
        assert_eq!(limit_total_ink(Color::BLACK, 0.0), Color::BLACK);
        for other in [Color::Registration, Color::gray(0.2), Color::rgb8(1, 2, 3)] {
            assert_eq!(limit_total_ink(other, 10.0), other);
        }
    }

    #[test]
    fn ink_limit_is_respected_for_every_naive_conversion() {
        for limit in [200.0_f32, 240.0, 260.0] {
            for r in (0..=255u16).step_by(17) {
                for g in (0..=255u16).step_by(17) {
                    for b in (0..=255u16).step_by(17) {
                        let rgb = Rgb8::new(r as u8, g as u8, b as u8);
                        let limited = limit_total_ink(to_cmyk_naive(rgb), limit);
                        let ink = limited.total_ink_percent().expect("CMYK");
                        assert!(ink <= limit + 1e-3, "{rgb}: {ink} > {limit}");
                    }
                }
            }
        }
    }
}
