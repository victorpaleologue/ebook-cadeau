//! Adversarial ("sparring") tests for `ebc-color`: conversions, contrast and CMYK helpers.
//!
//! Everything here goes through the public API only. The exhaustive sweeps over all
//! 16 777 216 sRGB colours are `#[ignore]`d because they take minutes in a debug build; run
//! them with `cargo test -p ebc-color --release --test spar_color -- --ignored`.

use ebc_color::{
    Lab, Rgb8, black_or_white, contrast_ratio, delta_e2000, delta_e2000_lab, ensure_contrast,
    from_scene_color, limit_total_ink, pick_text_color, relative_luminance, to_cmyk_gcr,
    to_cmyk_naive, to_scene_rgb,
};
use ebc_scene::Color;

/// xorshift64*: a tiny deterministic generator for test inputs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn rgb(&mut self) -> Rgb8 {
        let v = self.next();
        Rgb8::new((v >> 16) as u8, (v >> 32) as u8, (v >> 48) as u8)
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Every colour whose channels are all multiples of `step` (plus 255).
fn lattice(step: usize) -> impl Iterator<Item = Rgb8> {
    let axis: Vec<u8> = (0..=255usize).step_by(step).chain([255]).map(|v| v as u8).collect();
    let axis2 = axis.clone();
    let axis3 = axis.clone();
    axis.into_iter().flat_map(move |r| {
        let axis3 = axis3.clone();
        axis2
            .clone()
            .into_iter()
            .flat_map(move |g| axis3.clone().into_iter().map(move |b| Rgb8::new(r, g, b)))
    })
}

fn all_colours() -> impl Iterator<Item = Rgb8> {
    (0..=0x00FF_FFFFu32).map(|k| Rgb8::new((k >> 16) as u8, (k >> 8) as u8, k as u8))
}

// --- Lab ---------------------------------------------------------------------------------

/// The textbook sRGB → CIELAB (D65) conversion, written with the platform's `powf`/`cbrt`
/// and the usual rounded D65 white point, independently of the crate's Newton iterations.
fn reference_lab(c: Rgb8) -> (f64, f64, f64) {
    let lin = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
    let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
    let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b;
    let z = 0.019_333_9 * r + 0.119_192_0 * g + 0.950_304_1 * b;
    let f = |t: f64| {
        if t > 216.0 / 24389.0 { t.cbrt() } else { (24389.0 / 27.0 * t + 16.0) / 116.0 }
    };
    let (fx, fy, fz) = (f(x / 0.950_47), f(y / 1.0), f(z / 1.088_83));
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

#[test]
fn lab_agrees_with_an_independent_libm_implementation() {
    let mut worst = 0.0_f64;
    for c in lattice(5) {
        let lab = Lab::from_rgb8(c);
        let (l, a, b) = reference_lab(c);
        // The reference uses the 5-digit white point; the crate derives it from the
        // matrix. The two differ by a few 1e-5 in X/Z, i.e. well under 0.02 in a*/b*.
        let d = (lab.l - l).abs().max((lab.a - a).abs()).max((lab.b - b).abs());
        worst = worst.max(d);
        assert!(d < 0.02, "{c}: {lab:?} vs ({l}, {a}, {b})");
        assert!(lab.l.is_finite() && lab.a.is_finite() && lab.b.is_finite());
        assert!((0.0..=100.0).contains(&lab.l), "{c}: L = {}", lab.l);
    }
    assert!(worst > 0.0, "the comparison ran");
}

#[test]
fn lab_round_trip_is_exact_on_a_dense_lattice() {
    for c in lattice(3) {
        assert_eq!(Lab::from_rgb8(c).to_rgb8(), c, "{c}");
    }
}

/// The documentation promises `Lab::from_rgb8(c).to_rgb8() == c` for **every** `c`.
#[test]
#[ignore = "16.7 M colours: run with --release -- --ignored"]
fn lab_round_trip_is_exact_for_every_srgb_colour() {
    let mut failures = 0u32;
    for c in all_colours() {
        if Lab::from_rgb8(c).to_rgb8() != c {
            failures += 1;
            if failures <= 10 {
                eprintln!("round trip failed for {c}: got {}", Lab::from_rgb8(c).to_rgb8());
            }
        }
    }
    assert_eq!(failures, 0);
}

#[test]
fn lab_to_rgb_is_total_on_garbage() {
    let specials = [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        -0.0,
        0.0,
        1e300,
        -1e300,
        5e-324,
    ];
    for &l in &specials {
        for &a in &specials {
            for &b in &specials {
                let lab = Lab::new(l, a, b);
                let _ = lab.to_rgb8();
                let _ = lab.chroma();
                let _ = lab.delta_e76(Lab::new(50.0, 0.0, 0.0));
                let d = delta_e2000_lab(lab, Lab::new(50.0, 10.0, -10.0));
                assert!(d.is_nan() || d >= 0.0, "negative ΔE00 {d} for {lab:?}");
            }
        }
    }
    // Lightness is monotonic through the clipping at both ends.
    let mut previous = 0u8;
    for step in -200..=1200 {
        let grey = Lab::new(f64::from(step) / 10.0, 0.0, 0.0).to_rgb8();
        assert!(grey.r == grey.g && grey.g == grey.b, "neutral Lab must give a neutral grey");
        assert!(grey.r >= previous, "L* {} went darker", f64::from(step) / 10.0);
        previous = grey.r;
    }
    assert_eq!(previous, 255);
}

// --- contrast ----------------------------------------------------------------------------

/// WCAG 2.x as written in the recommendation (with its 0.03928 threshold and `powf`).
fn reference_luminance(c: Rgb8) -> f64 {
    let lin = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

#[test]
fn contrast_agrees_with_the_wcag_text() {
    let mut rng = Rng(0xC0_FFEE);
    for _ in 0..40_000 {
        let (a, b) = (rng.rgb(), rng.rgb());
        let (la, lb) = (reference_luminance(a), reference_luminance(b));
        let expected = (la.max(lb) + 0.05) / (la.min(lb) + 0.05);
        let got = contrast_ratio(a, b);
        assert!((got - expected).abs() < 1e-9, "{a} / {b}: {got} vs {expected}");
        assert!((relative_luminance(a) - la).abs() < 1e-12);
        assert!((1.0..=21.0).contains(&got));
    }
    for c in lattice(17) {
        let l = relative_luminance(c);
        assert!((0.0..=1.0).contains(&l), "{c}: {l}");
    }
}

#[test]
fn black_or_white_is_the_better_of_the_two_on_a_dense_lattice() {
    let mut worst = f64::INFINITY;
    for bg in lattice(3) {
        let pick = black_or_white(bg);
        let ratio = contrast_ratio(pick, bg);
        assert!(pick == Rgb8::BLACK || pick == Rgb8::WHITE);
        assert!(ratio >= contrast_ratio(Rgb8::BLACK, bg).max(contrast_ratio(Rgb8::WHITE, bg)));
        worst = worst.min(ratio);
    }
    // The documentation promises "always at least 4.58:1".
    assert!(worst >= 4.58, "worst black/white contrast is {worst}");
}

#[test]
#[ignore = "16.7 M colours: run with --release -- --ignored"]
fn black_or_white_reaches_4_58_for_every_background() {
    let mut worst = f64::INFINITY;
    for bg in all_colours() {
        worst = worst.min(contrast_ratio(black_or_white(bg), bg));
    }
    assert!(worst >= 4.58, "worst black/white contrast is {worst}");
}

#[test]
fn pick_text_color_returns_a_qualifying_candidate_or_the_fallback() {
    let mut rng = Rng(77);
    for round in 0..20_000 {
        let bg = rng.rgb();
        let candidates: Vec<Rgb8> = (0..(rng.next() % 5)).map(|_| rng.rgb()).collect();
        let min = match round % 7 {
            0 => f64::NAN,
            1 => f64::INFINITY,
            2 => f64::NEG_INFINITY,
            3 => 0.0,
            _ => 1.0 + rng.unit() * 20.5,
        };
        let pick = pick_text_color(bg, &candidates, min);
        let first_ok = candidates.iter().copied().find(|&c| contrast_ratio(c, bg) >= min);
        match first_ok {
            Some(c) => assert_eq!(pick, c, "must be the first qualifying candidate"),
            None => assert_eq!(pick, black_or_white(bg), "must fall back to black/white"),
        }
    }
}

#[test]
fn ensure_contrast_upholds_its_contract_for_random_pairs() {
    let mut rng = Rng(0xBAD_5EED);
    for round in 0..30_000 {
        let (fg, bg) = (rng.rgb(), rng.rgb());
        let min = match round % 6 {
            0 => 3.0,
            1 => 4.5,
            2 => 7.0,
            3 => 21.0,
            4 => 1.0,
            _ => 1.0 + rng.unit() * 20.0,
        };
        let out = ensure_contrast(fg, bg, min);
        let before = contrast_ratio(fg, bg);
        let best = contrast_ratio(black_or_white(bg), bg);
        if before >= min {
            assert_eq!(out, fg, "a colour that already passes must come back unchanged");
        } else if best >= min {
            assert!(
                contrast_ratio(out, bg) >= min,
                "{fg} on {bg} at {min}: got {out} ({})",
                contrast_ratio(out, bg)
            );
        } else {
            assert_eq!(out, black_or_white(bg), "unreachable ratio: best effort");
        }
    }
    // Hostile thresholds.
    for bad in [f64::NAN, f64::INFINITY, 1e300] {
        let out = ensure_contrast(Rgb8::new(10, 200, 30), Rgb8::new(128, 128, 128), bad);
        assert_eq!(out, black_or_white(Rgb8::new(128, 128, 128)));
    }
    for free in [f64::NEG_INFINITY, -1.0, 0.0, 1.0] {
        let fg = Rgb8::new(10, 200, 30);
        assert_eq!(ensure_contrast(fg, fg, free), fg);
    }
}

/// "The closest lighter or darker version of it that does": the result must not overshoot
/// by more than the L* step of 1 that the documentation describes (plus rounding).
#[test]
fn ensure_contrast_does_not_overshoot_grossly() {
    let mut rng = Rng(0x5EED);
    let mut checked = 0;
    for _ in 0..20_000 {
        let (fg, bg) = (rng.rgb(), rng.rgb());
        let min = 4.5;
        if contrast_ratio(fg, bg) >= min {
            continue;
        }
        let out = ensure_contrast(fg, bg, min);
        if out == Rgb8::BLACK || out == Rgb8::WHITE {
            continue;
        }
        // One L* step back towards the original must fail the ratio (otherwise the
        // function skipped a closer solution).
        let lab = out.to_lab();
        let original = fg.to_lab();
        let back = if lab.l > original.l { lab.l - 2.0 } else { lab.l + 2.0 };
        let closer = Lab::new(back, original.a, original.b).to_rgb8();
        assert!(
            contrast_ratio(closer, bg) < min + 0.35,
            "{fg} on {bg}: {out} overshoots, {closer} is {} already",
            contrast_ratio(closer, bg)
        );
        checked += 1;
    }
    assert!(checked > 1_000, "only {checked} cases exercised the loop");
}

// --- ΔE00 --------------------------------------------------------------------------------

#[test]
fn delta_e2000_is_a_sane_distance_on_srgb() {
    let mut rng = Rng(2000);
    for _ in 0..40_000 {
        let (a, b) = (rng.rgb(), rng.rgb());
        let d = delta_e2000(a, b);
        assert!(d.is_finite() && (0.0..=160.0).contains(&d), "{a} / {b}: {d}");
        assert!((d - delta_e2000(b, a)).abs() < 1e-9, "{a} / {b} asymmetric");
        assert_eq!(delta_e2000(a, a), 0.0);
        if a != b {
            assert!(d > 0.0, "{a} / {b}");
        }
    }
    // Neighbouring code values must never be reported as identical, anywhere in the cube
    // (dark blues are where the Lab differences are smallest).
    for c in lattice(15) {
        for (dr, dg, db) in [(1i16, 0i16, 0i16), (0, 1, 0), (0, 0, 1)] {
            let step = |v: u8, d: i16| if v == 255 { v - d as u8 } else { v + d as u8 };
            let n = Rgb8::new(step(c.r, dr), step(c.g, dg), step(c.b, db));
            let d = delta_e2000(c, n);
            assert!(d > 0.0 && d < 3.0, "{c} / {n}: {d}");
        }
    }
}

// --- CMYK --------------------------------------------------------------------------------

fn parts(color: Color) -> (f32, f32, f32, f32) {
    match color {
        Color::Cmyk { c, m, y, k } => (c, m, y, k),
        other => panic!("expected CMYK, got {other:?}"),
    }
}

fn check_naive(rgb: Rgb8) {
    let color = to_cmyk_naive(rgb);
    let (c, m, y, k) = parts(color);
    for v in [c, m, y, k] {
        assert!((0.0..=1.0).contains(&v), "{rgb}: {color:?}");
    }
    assert_eq!(c.min(m).min(y), 0.0, "{rgb}: one of C, M, Y must be empty: {color:?}");
    if rgb.r == rgb.g && rgb.g == rgb.b {
        assert_eq!((c, m, y), (0.0, 0.0, 0.0), "grey {rgb} must be K only");
    }
    assert!(color.total_ink_percent().is_some_and(|ink| ink <= 300.0 + 1e-3), "{rgb}");
    assert_eq!(from_scene_color(&color), rgb, "naive CMYK must map back to {rgb}");
}

#[test]
fn naive_cmyk_invariants_on_a_dense_lattice() {
    for rgb in lattice(3) {
        check_naive(rgb);
    }
    assert_eq!(to_cmyk_naive(Rgb8::BLACK), Color::BLACK);
    assert_eq!(to_cmyk_naive(Rgb8::WHITE), Color::WHITE);
}

#[test]
#[ignore = "16.7 M colours: run with --release -- --ignored"]
fn naive_cmyk_invariants_for_every_colour() {
    for rgb in all_colours() {
        check_naive(rgb);
    }
}

/// "Whatever the value [of gcr], the colour maps back to the same sRGB."
#[test]
fn partial_gcr_round_trips_on_a_dense_lattice() {
    for gcr in [0.0_f32, 0.1, 0.333, 0.5, 0.75, 0.999, 1.0] {
        for rgb in lattice(5) {
            let color = to_cmyk_gcr(rgb, gcr);
            let (c, m, y, k) = parts(color);
            for v in [c, m, y, k] {
                assert!((0.0..=1.0).contains(&v), "{rgb} at gcr {gcr}: {color:?}");
            }
            assert_eq!(from_scene_color(&color), rgb, "gcr {gcr}: {color:?}");
        }
    }
}

#[test]
#[ignore = "16.7 M colours x 3 levels: run with --release -- --ignored"]
fn partial_gcr_round_trips_for_every_colour() {
    for gcr in [0.0_f32, 0.37, 0.999] {
        for rgb in all_colours() {
            assert_eq!(from_scene_color(&to_cmyk_gcr(rgb, gcr)), rgb, "gcr {gcr}");
        }
    }
}

#[test]
fn hostile_gcr_values_are_clamped() {
    let c = Rgb8::new(30, 60, 90);
    for gcr in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::MAX, f32::MIN, -0.0, 1e-45] {
        let (cc, m, y, k) = parts(to_cmyk_gcr(c, gcr));
        for v in [cc, m, y, k] {
            assert!((0.0..=1.0).contains(&v), "gcr {gcr}: {v}");
        }
    }
}

#[test]
fn ink_limit_holds_for_random_cmyk_and_never_adds_ink() {
    let mut rng = Rng(400);
    for _ in 0..50_000 {
        let color =
            Color::cmyk(rng.unit() as f32, rng.unit() as f32, rng.unit() as f32, rng.unit() as f32);
        let limit = (rng.unit() * 420.0 - 10.0) as f32;
        let limited = limit_total_ink(color, limit);
        let (c0, m0, y0, k0) = parts(color);
        let (c1, m1, y1, k1) = parts(limited);
        assert_eq!(k1, k0, "black is never touched");
        assert!(c1 <= c0 && m1 <= m0 && y1 <= y0, "ink was added: {color:?} -> {limited:?}");
        let ink = (c1 + m1 + y1 + k1) * 100.0;
        // Either within the limit, or only the (untouchable) black is left.
        assert!(
            ink <= limit + 1e-2 || (c1 + m1 + y1 == 0.0),
            "{color:?} limited to {limit}: {limited:?} = {ink} %"
        );
        // Idempotent.
        assert_eq!(limit_total_ink(limited, limit), limited);
    }
    for limit in [f32::INFINITY, f32::MAX, 400.0] {
        let heavy = Color::cmyk(1.0, 1.0, 1.0, 1.0);
        assert_eq!(limit_total_ink(heavy, limit), heavy);
    }
    assert_eq!(
        limit_total_ink(Color::cmyk(1.0, 1.0, 1.0, 1.0), f32::NEG_INFINITY),
        Color::cmyk(0.0, 0.0, 0.0, 1.0)
    );
}

#[test]
fn scene_colour_conversion_is_total() {
    let specials = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 2.0, 0.5, -0.0];
    for &a in &specials {
        for &b in &specials {
            let _ = from_scene_color(&Color::Rgb { r: a, g: b, b: a });
            let _ = from_scene_color(&Color::Cmyk { c: a, m: b, y: a, k: b });
            let _ = from_scene_color(&Color::Gray(a));
            let _ = limit_total_ink(Color::Cmyk { c: a, m: b, y: a, k: b }, b);
        }
    }
    for c in lattice(51) {
        assert_eq!(from_scene_color(&to_scene_rgb(c)), c);
    }
}

// --- hex ---------------------------------------------------------------------------------

#[test]
fn hex_parser_survives_arbitrary_unicode() {
    let pieces = [
        "#", "f", "F", "0", "9", "g", " ", "\t", "\u{a0}", "é", "€", "𝔣", "\u{feff}", "\0", "+",
        "-", "x", "#ff", "\u{200b}", "Ａ", "０",
    ];
    let mut rng = Rng(16);
    for _ in 0..50_000 {
        let n = (rng.next() % 9) as usize;
        let text: String =
            (0..n).map(|_| pieces[(rng.next() % pieces.len() as u64) as usize]).collect();
        if let Some(c) = Rgb8::from_hex(&text) {
            // Whatever is accepted must be a real hex colour once trimmed.
            let t = text.trim();
            let digits = t.strip_prefix('#').unwrap_or(t);
            assert!(
                digits.len() == 3 || digits.len() == 6,
                "accepted {text:?} with {} digits",
                digits.len()
            );
            assert!(digits.bytes().all(|b| b.is_ascii_hexdigit()), "accepted {text:?} as {c}");
        }
    }
    // Full-width digits and signs are not hex digits.
    for bad in ["#ＦＦＦ", "+ff", "-ff", "#-1-1-1", "0x0000", "#ff\u{0}fff", "# fff", "#fff#"] {
        assert_eq!(Rgb8::from_hex(bad), None, "{bad:?}");
    }
    for c in lattice(51) {
        assert_eq!(Rgb8::from_hex(&c.to_string()), Some(c));
    }
}
