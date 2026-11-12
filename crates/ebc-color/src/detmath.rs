//! The few transcendental functions CIEDE2000 needs, built from `+ − × ÷` only.
//!
//! `f64::sin`, `cos`, `atan2`, `exp` and `hypot` come from the platform's maths library
//! (glibc natively, a port of musl in WebAssembly) and may differ in the last bit between
//! the two. The ink limiter of [`crate::ink`] *chooses* a separation by comparing
//! ΔE2000 values, so a last-bit difference could, in principle, choose a different
//! separation in the browser than natively. These versions are polynomial evaluations
//! after an exact range reduction: IEEE 754 defines every step, so they give the same bits
//! everywhere. They are accurate to a few units in the last place over the ranges
//! CIEDE2000 uses (angles in degrees, `exp` of a non-positive argument), which the tests
//! check against the platform functions.

/// π / 180.
const RAD_PER_DEG: f64 = core::f64::consts::PI / 180.0;
/// ln 2, split so that `n · LN2_HI` is exact for |n| < 2^20 (Cody–Waite reduction).
const LN2_HI: f64 = 6.931_471_803_691_238e-1;
const LN2_LO: f64 = 1.908_214_929_270_587_7e-10;

/// sin(x) for |x| ≤ π/4 (Taylor series to x¹⁹: error below 1e-17).
fn sin_kernel(x: f64) -> f64 {
    let x2 = x * x;
    let mut term = x;
    let mut sum = x;
    for n in 1..10u32 {
        let k = f64::from(2 * n);
        term = -term * x2 / (k * (k + 1.0));
        sum += term;
    }
    sum
}

/// cos(x) for |x| ≤ π/4 (Taylor series to x²⁰).
fn cos_kernel(x: f64) -> f64 {
    let x2 = x * x;
    let mut term = 1.0;
    let mut sum = 1.0;
    for n in 1..11u32 {
        let k = f64::from(2 * n);
        term = -term * x2 / ((k - 1.0) * k);
        sum += term;
    }
    sum
}

/// The angle in degrees reduced to `[-45, 45]`, and the number of quarter turns taken off
/// (mod 4). Exact for the moderate angles CIEDE2000 produces (|deg| < 2^40).
fn quarter_turns(deg: f64) -> (f64, i64) {
    if !deg.is_finite() || deg.abs() > 1.0e12 {
        return (0.0, 0);
    }
    let q = (deg / 90.0).round();
    let rest = deg - q * 90.0;
    // `q` is a whole number far below 2^53: the conversion is exact.
    #[allow(clippy::cast_possible_truncation)]
    let turns = (q as i64).rem_euclid(4);
    (rest, turns)
}

/// sin of an angle in degrees.
pub(crate) fn sin_deg(deg: f64) -> f64 {
    let (rest, turns) = quarter_turns(deg);
    let x = rest * RAD_PER_DEG;
    match turns {
        0 => sin_kernel(x),
        1 => cos_kernel(x),
        2 => -sin_kernel(x),
        _ => -cos_kernel(x),
    }
}

/// cos of an angle in degrees.
pub(crate) fn cos_deg(deg: f64) -> f64 {
    let (rest, turns) = quarter_turns(deg);
    let x = rest * RAD_PER_DEG;
    match turns {
        0 => cos_kernel(x),
        1 => -sin_kernel(x),
        2 => -cos_kernel(x),
        _ => sin_kernel(x),
    }
}

/// atan(t) for |t| ≤ tan(π/8) ≈ 0.4142 (alternating series to t⁴⁷: error below 1e-18).
fn atan_kernel(t: f64) -> f64 {
    let t2 = t * t;
    let mut power = t;
    let mut sum = t;
    for n in 1..24u32 {
        power *= -t2;
        sum += power / f64::from(2 * n + 1);
    }
    sum
}

/// atan(t) for 0 ≤ t ≤ 1.
fn atan_unit(t: f64) -> f64 {
    const TAN_PI_8: f64 = 0.414_213_562_373_095_03;
    if t <= TAN_PI_8 {
        atan_kernel(t)
    } else {
        // atan t = π/4 + atan((t − 1) / (t + 1)), and |(t − 1)/(t + 1)| ≤ tan(π/8) here.
        core::f64::consts::FRAC_PI_4 + atan_kernel((t - 1.0) / (t + 1.0))
    }
}

/// atan2(y, x) in degrees, in `(-180, 180]`; 0 for (0, 0) and for non-finite input.
pub(crate) fn atan2_deg(y: f64, x: f64) -> f64 {
    if !(y.is_finite() && x.is_finite()) || (y == 0.0 && x == 0.0) {
        return 0.0;
    }
    let (ay, ax) = (y.abs(), x.abs());
    // The angle of (|x|, |y|) in the first quadrant.
    let first = if ay <= ax {
        atan_unit(ay / ax)
    } else {
        core::f64::consts::FRAC_PI_2 - atan_unit(ax / ay)
    };
    let angle = if x >= 0.0 { first } else { core::f64::consts::PI - first };
    let angle = if y < 0.0 { -angle } else { angle };
    angle / RAD_PER_DEG
}

/// e^x for x ≤ 0 (larger arguments are clamped to 0: CIEDE2000 only needs x ≤ 0).
/// Underflows to 0 below −700.
pub(crate) fn exp_neg(x: f64) -> f64 {
    if x.is_nan() {
        return 0.0;
    }
    let x = x.min(0.0);
    if x < -700.0 {
        return 0.0;
    }
    let n = (x / core::f64::consts::LN_2).round();
    let r = (x - n * LN2_HI) - n * LN2_LO; // |r| ≤ ln 2 / 2
    // Taylor series of e^r to r¹⁶ (|r| ≤ 0.35: error below 1e-17).
    let mut term = 1.0;
    let mut sum = 1.0;
    for k in 1..17u32 {
        term *= r / f64::from(k);
        sum += term;
    }
    // 2^n for −1010 ≤ n ≤ 0, built from its bits (exact).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let scale = f64::from_bits(((1023 + n as i64) as u64) << 52);
    sum * scale
}

/// √(a² + b²) (no overflow protection is needed for colour coordinates).
pub(crate) fn hypot(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrees_with_the_platform_functions() {
        let mut worst: f64 = 0.0;
        let mut deg = -1000.0;
        while deg <= 1000.0 {
            worst = worst.max((sin_deg(deg) - deg.to_radians().sin()).abs());
            worst = worst.max((cos_deg(deg) - deg.to_radians().cos()).abs());
            deg += 0.37;
        }
        assert!(worst < 1e-14, "trig {worst:e}");
        for (y, x) in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (0.0, -2.0)] {
            assert!((atan2_deg(y, x) - f64::atan2(y, x).to_degrees()).abs() < 1e-12);
        }
        let mut worst: f64 = 0.0;
        for i in -60..=60 {
            for j in -60..=60 {
                let (y, x) = (f64::from(i) * 1.7, f64::from(j) * 2.3);
                if x == 0.0 && y == 0.0 {
                    continue;
                }
                worst = worst.max((atan2_deg(y, x) - f64::atan2(y, x).to_degrees()).abs());
            }
        }
        assert!(worst < 1e-11, "atan2 {worst:e}");
        let mut x = 0.0;
        while x > -200.0 {
            let rel = (exp_neg(x) - x.exp()).abs() / x.exp();
            assert!(rel < 1e-14, "exp({x}) {rel:e}");
            x -= 0.731;
        }
        assert_eq!(exp_neg(0.0), 1.0);
        assert_eq!(exp_neg(-1000.0), 0.0);
        assert_eq!(exp_neg(5.0), 1.0);
        assert_eq!(atan2_deg(0.0, 0.0), 0.0);
        assert_eq!(atan2_deg(f64::NAN, 1.0), 0.0);
        assert_eq!(sin_deg(f64::NAN), 0.0);
        assert_eq!(cos_deg(f64::INFINITY), 1.0);
        assert!((hypot(3.0, 4.0) - 5.0).abs() < 1e-15);
    }
}
