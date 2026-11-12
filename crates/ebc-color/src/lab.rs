//! sRGB ↔ linear light ↔ CIE XYZ ↔ CIELAB (D65), with platform-independent arithmetic.
//!
//! The usual formulas need `x^2.4` and a cube root. `powf` and `cbrt` come from the
//! platform's maths library and may differ in the last bit between a native build and a
//! WebAssembly build, which could flip a nearest-cluster decision in [`crate::palette`].
//! The two roots are therefore computed here with Newton iterations built from `+ - * /`
//! only, which IEEE 754 defines exactly.

use std::sync::OnceLock;

use crate::Rgb8;

/// Linear sRGB → CIE XYZ (D65), IEC 61966-2-1 primaries.
const RGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.412_456_4, 0.357_576_1, 0.180_437_5],
    [0.212_672_9, 0.715_152_2, 0.072_175_0],
    [0.019_333_9, 0.119_192_0, 0.950_304_1],
];

/// CIE constant ε = (6/29)³.
const EPSILON: f64 = 216.0 / 24389.0;
/// CIE constant κ = (29/3)³.
const KAPPA: f64 = 24389.0 / 27.0;

/// A colour in CIELAB with the D65 reference white (the white of sRGB).
///
/// `l` is lightness (0 = black, 100 = white); `a` runs green (−) to red (+) and `b` blue (−)
/// to yellow (+). No chromatic adaptation to D50 is applied, so the values are comparable
/// with other "sRGB → Lab (D65)" calculators but not with ICC (D50) Lab values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Lab {
    /// Lightness L\*, nominally `0.0..=100.0`.
    pub l: f64,
    /// Green–red axis a\*.
    pub a: f64,
    /// Blue–yellow axis b\*.
    pub b: f64,
}

impl Lab {
    /// Creates a CIELAB colour from its coordinates.
    #[must_use]
    pub const fn new(l: f64, a: f64, b: f64) -> Self {
        Self { l, a, b }
    }

    /// Converts an sRGB colour to CIELAB (D65).
    ///
    /// Neutral greys map to exactly `a = b = 0`, black to `L = 0` and white to `L = 100`.
    #[must_use]
    pub fn from_rgb8(c: Rgb8) -> Self {
        let t = tables();
        let (r, g, b) = (t.linear[c.r as usize], t.linear[c.g as usize], t.linear[c.b as usize]);
        if c.r == c.g && c.g == c.b {
            // Achromatic: X/Xn = Y/Yn = Z/Zn = the linear value, so a = b = 0 exactly.
            return Self { l: lightness(r), a: 0.0, b: 0.0 };
        }
        let m = &RGB_TO_XYZ;
        let x = (m[0][0] * r + m[0][1] * g + m[0][2] * b) / t.white[0];
        let y = (m[1][0] * r + m[1][1] * g + m[1][2] * b) / t.white[1];
        let z = (m[2][0] * r + m[2][1] * g + m[2][2] * b) / t.white[2];
        let (fx, fy, fz) = (lab_f(x), lab_f(y), lab_f(z));
        Self { l: lightness(y), a: 500.0 * (fx - fy), b: 200.0 * (fy - fz) }
    }

    /// Converts back to 8-bit sRGB, clipping each channel to the sRGB gamut and rounding
    /// to the nearest code value. Non-finite coordinates produce an unspecified but valid
    /// colour (no panic).
    ///
    /// `Lab::from_rgb8(c).to_rgb8() == c` for every `c`.
    #[must_use]
    pub fn to_rgb8(self) -> Rgb8 {
        let t = tables();
        let fy = (self.l + 16.0) / 116.0;
        let fx = fy + self.a / 500.0;
        let fz = fy - self.b / 200.0;
        let x = lab_f_inv(fx) * t.white[0];
        let y = lab_f_inv(fy) * t.white[1];
        let z = lab_f_inv(fz) * t.white[2];
        let m = &t.xyz_to_rgb;
        let r = m[0][0] * x + m[0][1] * y + m[0][2] * z;
        let g = m[1][0] * x + m[1][1] * y + m[1][2] * z;
        let b = m[2][0] * x + m[2][1] * y + m[2][2] * z;
        Rgb8::new(encode(t, r), encode(t, g), encode(t, b))
    }

    /// Chroma C\*ab = √(a² + b²): 0 for greys, larger for more saturated colours.
    #[must_use]
    pub fn chroma(self) -> f64 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    /// The CIE 1976 colour difference ΔE\*ab: the Euclidean distance in CIELAB.
    ///
    /// Cheaper and more predictable than [`delta_e2000`](crate::delta_e2000); a difference of
    /// about 2.3 is commonly quoted as "just noticeable".
    #[must_use]
    pub fn delta_e76(self, other: Self) -> f64 {
        self.distance_squared(other).sqrt()
    }

    /// Squared Euclidean distance in CIELAB.
    #[must_use]
    pub(crate) fn distance_squared(self, other: Self) -> f64 {
        let (dl, da, db) = (self.l - other.l, self.a - other.a, self.b - other.b);
        dl * dl + da * da + db * db
    }
}

/// The linear-light value (0–1) of an 8-bit sRGB channel value.
#[must_use]
pub(crate) fn srgb_to_linear(v: u8) -> f64 {
    tables().linear[v as usize]
}

/// L\* from the relative luminance Y/Yn.
fn lightness(y: f64) -> f64 {
    if y > EPSILON { 116.0 * cbrt_newton(y) - 16.0 } else { KAPPA * y }
}

/// The CIELAB companding function f(t).
fn lab_f(t: f64) -> f64 {
    if t > EPSILON { cbrt_newton(t) } else { (KAPPA * t + 16.0) / 116.0 }
}

/// Inverse of [`lab_f`].
fn lab_f_inv(f: f64) -> f64 {
    let cube = f * f * f;
    if cube > EPSILON { cube } else { (116.0 * f - 16.0) / KAPPA }
}

/// Cube root of a positive number by Newton's method, using only basic operations so the
/// result is bit-identical on every IEEE-754 platform. Intended for `x` in `(0, ~2]`
/// (ratios to the reference white); returns 0 for `x <= 0` or NaN.
fn cbrt_newton(x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return 0.0;
    }
    // Starting at or above the root, Newton's iterates for y³ = x decrease monotonically.
    let mut y = x.max(1.0);
    for _ in 0..200 {
        let next = (2.0 * y + x / (y * y)) / 3.0;
        if next >= y {
            break;
        }
        y = next;
    }
    y
}

/// Fifth root of a positive number, same technique and domain as [`cbrt_newton`].
fn fifth_root_newton(x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return 0.0;
    }
    let mut y = x.max(1.0);
    for _ in 0..200 {
        let y2 = y * y;
        let next = (4.0 * y + x / (y2 * y2)) / 5.0;
        if next >= y {
            break;
        }
        y = next;
    }
    y
}

/// The sRGB electro-optical transfer function: encoded value (0–1) → linear light (0–1).
fn srgb_eotf(v: f64) -> f64 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        // t^2.4 = t² · (t^(1/5))²
        let t = (v + 0.055) / 1.055;
        let root = fifth_root_newton(t);
        t * t * root * root
    }
}

/// Lookup tables shared by every conversion.
struct Tables {
    /// Linear-light value of each 8-bit code.
    linear: [f64; 256],
    /// `thresholds[i]` is the linear value halfway (in the encoded domain) between code
    /// `i` and code `i + 1`: a linear value rounds to the number of thresholds it reaches.
    thresholds: [f64; 255],
    /// XYZ of the reference white = the RGB→XYZ matrix applied to (1, 1, 1).
    white: [f64; 3],
    /// Inverse of [`RGB_TO_XYZ`].
    xyz_to_rgb: [[f64; 3]; 3],
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut linear = [0.0; 256];
        for (i, slot) in linear.iter_mut().enumerate() {
            *slot = srgb_eotf(i as f64 / 255.0);
        }
        let mut thresholds = [0.0; 255];
        for (i, slot) in thresholds.iter_mut().enumerate() {
            *slot = srgb_eotf((i as f64 + 0.5) / 255.0);
        }
        let m = &RGB_TO_XYZ;
        let white = [
            m[0][0] + m[0][1] + m[0][2],
            m[1][0] + m[1][1] + m[1][2],
            m[2][0] + m[2][1] + m[2][2],
        ];
        Tables { linear, thresholds, white, xyz_to_rgb: invert3(m) }
    })
}

/// Inverse of a 3×3 matrix by cofactors. The sRGB matrix is well conditioned
/// (determinant ≈ 0.2), so no pivoting is needed.
fn invert3(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let cof = |r: usize, c: usize| {
        let (r1, r2) = ((r + 1) % 3, (r + 2) % 3);
        let (c1, c2) = ((c + 1) % 3, (c + 2) % 3);
        m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1]
    };
    let det = m[0][0] * cof(0, 0) + m[0][1] * cof(0, 1) + m[0][2] * cof(0, 2);
    let mut inv = [[0.0; 3]; 3];
    for (r, row) in inv.iter_mut().enumerate() {
        for (c, slot) in row.iter_mut().enumerate() {
            // Transposed cofactor matrix (cyclic indexing already carries the sign).
            *slot = cof(c, r) / det;
        }
    }
    inv
}

/// Linear light → nearest 8-bit sRGB code (values outside 0–1 clip; NaN maps to 0).
fn encode(t: &Tables, linear: f64) -> u8 {
    // At most 255 thresholds can be reached, so the count always fits a u8.
    t.thresholds.partition_point(|&threshold| threshold <= linear) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn newton_roots_agree_with_libm() {
        let mut x = 1e-6_f64;
        while x < 4.0 {
            let (c, f) = (cbrt_newton(x), fifth_root_newton(x));
            assert!(close(c, x.cbrt(), 4.0 * f64::EPSILON * c.max(1.0)), "cbrt({x}) = {c}");
            assert!(close(f, x.powf(0.2), 4.0 * f64::EPSILON * f.max(1.0)), "x^0.2({x}) = {f}");
            x *= 1.003;
        }
        assert_eq!(cbrt_newton(1.0), 1.0);
        assert_eq!(cbrt_newton(8.0), 2.0);
        assert_eq!(fifth_root_newton(32.0), 2.0);
    }

    #[test]
    fn newton_roots_reject_hostile_input_without_looping_or_panicking() {
        for bad in [0.0, -1.0, f64::NAN, f64::NEG_INFINITY] {
            assert_eq!(cbrt_newton(bad), 0.0);
            assert_eq!(fifth_root_newton(bad), 0.0);
        }
        // Outside the intended domain the iteration is capped; it must still terminate.
        assert!(cbrt_newton(f64::MAX).is_finite());
        assert!(cbrt_newton(f64::INFINITY).is_infinite() || cbrt_newton(f64::INFINITY).is_nan());
    }

    #[test]
    fn transfer_curve_matches_the_srgb_definition() {
        let t = tables();
        assert_eq!(t.linear[0], 0.0);
        assert!(close(t.linear[255], 1.0, 1e-15));
        for i in 0..=255usize {
            let v = i as f64 / 255.0;
            let expected =
                if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
            assert!(close(t.linear[i], expected, 1e-14), "code {i}");
            if i > 0 {
                assert!(t.linear[i] > t.linear[i - 1], "monotonic at {i}");
                assert!(t.thresholds[i - 1] > t.linear[i - 1] && t.thresholds[i - 1] < t.linear[i]);
            }
        }
        // Mid grey in linear light is the well-known 21.4 %.
        assert!(close(srgb_to_linear(128), 0.2158605, 1e-6));
    }

    #[test]
    fn inverse_matrix_is_an_inverse() {
        let inv = &tables().xyz_to_rgb;
        for r in 0..3 {
            for c in 0..3 {
                let v: f64 = (0..3).map(|k| inv[r][k] * RGB_TO_XYZ[k][c]).sum();
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!(close(v, expected, 1e-14), "({r},{c}) = {v}");
            }
        }
        // Published inverse (Lindbloom), to the precision the forward matrix allows.
        assert!(close(inv[0][0], 3.240_454_2, 1e-6));
        assert!(close(inv[1][1], 1.876_010_8, 1e-6));
        assert!(close(inv[2][2], 1.057_225_2, 1e-6));
    }

    #[test]
    fn anchors_are_exact() {
        assert_eq!(Lab::from_rgb8(Rgb8::BLACK), Lab::new(0.0, 0.0, 0.0));
        assert_eq!(Lab::from_rgb8(Rgb8::WHITE), Lab::new(100.0, 0.0, 0.0));
        for v in 0..=255u8 {
            let lab = Lab::from_rgb8(Rgb8::new(v, v, v));
            assert_eq!((lab.a, lab.b), (0.0, 0.0), "grey {v}");
            assert!((0.0..=100.0).contains(&lab.l));
        }
        // 18 % grey card: sRGB 119 is L* ≈ 50.
        assert!(close(Lab::from_rgb8(Rgb8::new(119, 119, 119)).l, 50.0, 0.1));
    }

    #[test]
    fn primaries_match_published_d65_values() {
        // Reference values for sRGB primaries/secondaries in CIELAB (D65, 2° observer).
        let cases = [
            (Rgb8::new(255, 0, 0), (53.2408, 80.0925, 67.2032)),
            (Rgb8::new(0, 255, 0), (87.7347, -86.1827, 83.1793)),
            (Rgb8::new(0, 0, 255), (32.2970, 79.1875, -107.8602)),
            (Rgb8::new(255, 255, 0), (97.1393, -21.5537, 94.4780)),
            (Rgb8::new(0, 255, 255), (91.1132, -48.0875, -14.1312)),
            (Rgb8::new(255, 0, 255), (60.3242, 98.2343, -60.8249)),
        ];
        for (rgb, (l, a, b)) in cases {
            let lab = Lab::from_rgb8(rgb);
            assert!(
                close(lab.l, l, 0.02) && close(lab.a, a, 0.02) && close(lab.b, b, 0.02),
                "{rgb}: {lab:?}"
            );
        }
    }

    #[test]
    fn round_trip_is_exact_on_a_lattice_and_a_pseudo_random_sample() {
        let steps: Vec<u8> = (0..=255u16).step_by(5).map(|v| v as u8).collect();
        for &r in &steps {
            for &g in &steps {
                for &b in &steps {
                    let c = Rgb8::new(r, g, b);
                    assert_eq!(Lab::from_rgb8(c).to_rgb8(), c);
                }
            }
        }
        let mut state = 0x1234_5678_9ABC_DEF0_u64;
        for _ in 0..20_000 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let c = Rgb8::from_key((state >> 40) as u32);
            assert_eq!(Lab::from_rgb8(c).to_rgb8(), c);
        }
    }

    #[test]
    fn out_of_gamut_and_non_finite_lab_values_clip_instead_of_panicking() {
        assert_eq!(Lab::new(150.0, 0.0, 0.0).to_rgb8(), Rgb8::WHITE);
        assert_eq!(Lab::new(-20.0, 0.0, 0.0).to_rgb8(), Rgb8::BLACK);
        // A red far more saturated than sRGB can show clips to pure red-ish.
        let c = Lab::new(50.0, 200.0, 150.0).to_rgb8();
        assert_eq!((c.r, c.b), (255, 0));
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let _ = Lab::new(bad, 0.0, 0.0).to_rgb8();
            let _ = Lab::new(50.0, bad, 0.0).to_rgb8();
            let _ = Lab::new(50.0, 0.0, bad).to_rgb8();
        }
    }

    #[test]
    fn chroma_and_delta_e76() {
        assert_eq!(Lab::new(50.0, 3.0, 4.0).chroma(), 5.0);
        assert_eq!(Lab::new(50.0, 0.0, 0.0).delta_e76(Lab::new(53.0, 4.0, 0.0)), 5.0);
        assert_eq!(Lab::from_rgb8(Rgb8::BLACK).delta_e76(Lab::from_rgb8(Rgb8::WHITE)), 100.0);
    }
}
