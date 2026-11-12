//! The CIEDE2000 colour-difference formula.

use crate::Rgb8;
use crate::lab::Lab;

/// CIEDE2000 colour difference ΔE₀₀ between two sRGB colours (via CIELAB D65).
///
/// 0 means identical; about 1 is a just-noticeable difference; black versus white is 100.
/// The parametric factors k_L, k_C and k_H are 1 (reference conditions).
///
/// ```
/// use ebc_color::{Rgb8, delta_e2000};
/// assert_eq!(delta_e2000(Rgb8::WHITE, Rgb8::WHITE), 0.0);
/// assert!((delta_e2000(Rgb8::BLACK, Rgb8::WHITE) - 100.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn delta_e2000(a: Rgb8, b: Rgb8) -> f64 {
    delta_e2000_lab(Lab::from_rgb8(a), Lab::from_rgb8(b))
}

/// CIEDE2000 colour difference ΔE₀₀ between two CIELAB colours.
///
/// Implements the formulation of Sharma, Wu and Dalal, *"The CIEDE2000 Color-Difference
/// Formula: Implementation Notes, Supplementary Test Data, and Mathematical Observations"*
/// (Color Research & Application 30(1), 2005), including its handling of the hue
/// discontinuities, and is validated against all 34 test pairs of that paper. The function
/// is symmetric in its arguments.
#[must_use]
pub fn delta_e2000_lab(lab1: Lab, lab2: Lab) -> f64 {
    const POW25_7: f64 = 6_103_515_625.0; // 25^7

    // Step 1: C'ᵢ and h'ᵢ.
    let c1 = lab1.a.hypot(lab1.b);
    let c2 = lab2.a.hypot(lab2.b);
    let c_bar7 = ((c1 + c2) / 2.0).powi(7);
    let g = 0.5 * (1.0 - (c_bar7 / (c_bar7 + POW25_7)).sqrt());
    let a1p = (1.0 + g) * lab1.a;
    let a2p = (1.0 + g) * lab2.a;
    let c1p = a1p.hypot(lab1.b);
    let c2p = a2p.hypot(lab2.b);
    let h1p = hue_degrees(lab1.b, a1p);
    let h2p = hue_degrees(lab2.b, a2p);

    // Step 2: ΔL', ΔC', ΔH'.
    let dl = lab2.l - lab1.l;
    let dc = c2p - c1p;
    let achromatic = c1p * c2p == 0.0;
    let dh_deg = if achromatic {
        0.0
    } else {
        let d = h2p - h1p;
        if d > 180.0 {
            d - 360.0
        } else if d < -180.0 {
            d + 360.0
        } else {
            d
        }
    };
    let dh = 2.0 * (c1p * c2p).sqrt() * (dh_deg / 2.0).to_radians().sin();

    // Step 3: the weighted difference.
    let l_bar = (lab1.l + lab2.l) / 2.0;
    let c_bar = (c1p + c2p) / 2.0;
    let h_sum = h1p + h2p;
    let h_bar = if achromatic {
        h_sum
    } else if (h1p - h2p).abs() <= 180.0 {
        h_sum / 2.0
    } else if h_sum < 360.0 {
        (h_sum + 360.0) / 2.0
    } else {
        (h_sum - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * (h_bar - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar).to_radians().cos()
        + 0.32 * (3.0 * h_bar + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar - 63.0).to_radians().cos();
    let d_theta = 30.0 * (-((h_bar - 275.0) / 25.0).powi(2)).exp();
    let c_bar7 = c_bar.powi(7);
    let r_c = 2.0 * (c_bar7 / (c_bar7 + POW25_7)).sqrt();
    let l50 = (l_bar - 50.0).powi(2);
    let s_l = 1.0 + 0.015 * l50 / (20.0 + l50).sqrt();
    let s_c = 1.0 + 0.045 * c_bar;
    let s_h = 1.0 + 0.015 * c_bar * t;
    let r_t = -(2.0 * d_theta).to_radians().sin() * r_c;

    let (tl, tc, th) = (dl / s_l, dc / s_c, dh / s_h);
    // The radicand is non-negative in exact arithmetic; guard against −0.0…1 rounding.
    (tl * tl + tc * tc + th * th + r_t * tc * th).max(0.0).sqrt()
}

/// Hue angle in degrees, `0.0..360.0`; 0 when both components are zero.
fn hue_degrees(b: f64, a: f64) -> f64 {
    if a == 0.0 && b == 0.0 {
        return 0.0;
    }
    let h = b.atan2(a).to_degrees();
    if h < 0.0 { h + 360.0 } else { h }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 34 test pairs of Sharma, Wu & Dalal (2005), Table 1:
    /// `(L1, a1, b1, L2, a2, b2, ΔE00)`.
    const SHARMA: [(f64, f64, f64, f64, f64, f64, f64); 34] = [
        (50.0000, 2.6772, -79.7751, 50.0000, 0.0000, -82.7485, 2.0425),
        (50.0000, 3.1571, -77.2803, 50.0000, 0.0000, -82.7485, 2.8615),
        (50.0000, 2.8361, -74.0200, 50.0000, 0.0000, -82.7485, 3.4412),
        (50.0000, -1.3802, -84.2814, 50.0000, 0.0000, -82.7485, 1.0000),
        (50.0000, -1.1848, -84.8006, 50.0000, 0.0000, -82.7485, 1.0000),
        (50.0000, -0.9009, -85.5211, 50.0000, 0.0000, -82.7485, 1.0000),
        (50.0000, 0.0000, 0.0000, 50.0000, -1.0000, 2.0000, 2.3669),
        (50.0000, -1.0000, 2.0000, 50.0000, 0.0000, 0.0000, 2.3669),
        (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0009, 7.1792),
        (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0010, 7.1792),
        (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0011, 7.2195),
        (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0012, 7.2195),
        (50.0000, -0.0010, 2.4900, 50.0000, 0.0009, -2.4900, 4.8045),
        (50.0000, -0.0010, 2.4900, 50.0000, 0.0010, -2.4900, 4.8045),
        (50.0000, -0.0010, 2.4900, 50.0000, 0.0011, -2.4900, 4.7461),
        (50.0000, 2.5000, 0.0000, 50.0000, 0.0000, -2.5000, 4.3065),
        (50.0000, 2.5000, 0.0000, 73.0000, 25.0000, -18.0000, 27.1492),
        (50.0000, 2.5000, 0.0000, 61.0000, -5.0000, 29.0000, 22.8977),
        (50.0000, 2.5000, 0.0000, 56.0000, -27.0000, -3.0000, 31.9030),
        (50.0000, 2.5000, 0.0000, 58.0000, 24.0000, 15.0000, 19.4535),
        (50.0000, 2.5000, 0.0000, 50.0000, 3.1736, 0.5854, 1.0000),
        (50.0000, 2.5000, 0.0000, 50.0000, 3.2972, 0.0000, 1.0000),
        (50.0000, 2.5000, 0.0000, 50.0000, 1.8634, 0.5757, 1.0000),
        (50.0000, 2.5000, 0.0000, 50.0000, 3.2592, 0.3350, 1.0000),
        (60.2574, -34.0099, 36.2677, 60.4626, -34.1751, 39.4387, 1.2644),
        (63.0109, -31.0961, -5.8663, 62.8187, -29.7946, -4.0864, 1.2630),
        (61.2901, 3.7196, -5.3901, 61.4292, 2.2480, -4.9620, 1.8731),
        (35.0831, -44.1164, 3.7933, 35.0232, -40.0716, 1.5901, 1.8645),
        (22.7233, 20.0904, -46.6940, 23.0331, 14.9730, -42.5619, 2.0373),
        (36.4612, 47.8580, 18.3852, 36.2715, 50.5065, 21.2231, 1.4146),
        (90.8027, -2.0831, 1.4410, 91.1528, -1.6435, 0.0447, 1.4441),
        (90.9257, -0.5406, -0.9208, 88.6381, -0.8985, -0.7239, 1.5381),
        (6.7747, -0.2908, -2.4247, 5.8714, -0.0985, -2.2286, 0.6377),
        (2.0776, 0.0795, -1.1350, 0.9033, -0.0636, -0.5514, 0.9082),
    ];

    #[test]
    fn matches_all_34_sharma_reference_pairs() {
        for (i, &(l1, a1, b1, l2, a2, b2, expected)) in SHARMA.iter().enumerate() {
            let (x, y) = (Lab::new(l1, a1, b1), Lab::new(l2, a2, b2));
            let forward = delta_e2000_lab(x, y);
            let backward = delta_e2000_lab(y, x);
            // The paper tabulates four decimals.
            assert!(
                (forward - expected).abs() < 5e-5,
                "pair {}: got {forward:.6}, expected {expected:.4}",
                i + 1
            );
            assert!((forward - backward).abs() < 1e-10, "pair {} is not symmetric", i + 1);
        }
    }

    #[test]
    fn identity_and_achromatic_extremes() {
        assert_eq!(delta_e2000_lab(Lab::new(42.0, 10.0, -20.0), Lab::new(42.0, 10.0, -20.0)), 0.0);
        for v in [0u8, 1, 17, 128, 254, 255] {
            assert_eq!(delta_e2000(Rgb8::new(v, v, v), Rgb8::new(v, v, v)), 0.0);
        }
        // Two greys only differ in L*: ΔE00 = ΔL / S_L, and S_L = 1 when the mean L* is 50.
        assert!((delta_e2000(Rgb8::BLACK, Rgb8::WHITE) - 100.0).abs() < 1e-9);
        assert!(
            (delta_e2000_lab(Lab::new(40.0, 0.0, 0.0), Lab::new(60.0, 0.0, 0.0)) - 20.0).abs()
                < 1e-12
        );
    }

    #[test]
    fn behaves_like_a_perceptual_distance_on_srgb() {
        let mut state = 0xC0FFEE_u64;
        for _ in 0..3_000 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695);
            let a = Rgb8::from_key((state >> 40) as u32);
            let b = Rgb8::from_key((state >> 16) as u32);
            let d = delta_e2000(a, b);
            assert!(d.is_finite() && d >= 0.0);
            assert!((d - delta_e2000(b, a)).abs() < 1e-9, "{a} / {b}");
            if a != b {
                assert!(d > 0.0, "{a} / {b}");
            }
        }
        // Neighbouring 8-bit values are well below a just-noticeable difference…
        assert!(delta_e2000(Rgb8::new(120, 130, 140), Rgb8::new(121, 130, 140)) < 1.0);
        // …while complementary colours are far apart.
        assert!(delta_e2000(Rgb8::new(255, 0, 0), Rgb8::new(0, 255, 255)) > 50.0);
        // CIEDE2000 discounts chroma differences between saturated colours relative to
        // the Euclidean ΔE*ab.
        let (a, b) = (Rgb8::new(255, 0, 0), Rgb8::new(230, 20, 20));
        assert!(delta_e2000(a, b) < a.to_lab().delta_e76(b.to_lab()));
    }

    #[test]
    fn non_finite_input_does_not_panic() {
        let ok = Lab::new(50.0, 0.0, 0.0);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let _ = delta_e2000_lab(Lab::new(bad, 1.0, 1.0), ok);
            let _ = delta_e2000_lab(ok, Lab::new(50.0, bad, 1.0));
            let _ = delta_e2000_lab(Lab::new(50.0, 1.0, bad), Lab::new(50.0, bad, bad));
        }
    }
}
