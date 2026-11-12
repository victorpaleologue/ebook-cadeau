//! Conversions between [`Rgb8`] and the print pipeline's [`ebc_scene::Color`].

use ebc_scene::Color;

use crate::Rgb8;

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
/// * at least one of C, M, Y is always 0, so total ink never exceeds 300 %
///   (see [`limit_total_ink`] for stricter limits).
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
