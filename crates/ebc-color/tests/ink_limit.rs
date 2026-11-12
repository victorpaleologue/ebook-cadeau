//! Ink limiting by grey-component replacement, through the bundled FOGRA39 profile.
//!
//! `tests/fixtures/lcms-a2b-fogra39.tsv` was written by Pillow's `ImageCms` (Little CMS)
//! with `tests/fixtures/lcms_a2b_reference.py`: the Lab of CMYK colours (a 5⁴ lattice and
//! the dark builds the limiter produces) through the profile's `A2B1` table, which is how
//! [`CmykProfile::limit_ink`] measures the colours it compares.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ebc_color::{CmykProfile, InkLimit, Lab, RenderingIntent, Rgb8, SrgbToCmyk, delta_e2000_lab};

fn profile() -> CmykProfile {
    let path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc");
    CmykProfile::parse(&std::fs::read(path).expect("bundled profile")).expect("parses")
}

fn total(cmyk: [f64; 4]) -> f64 {
    cmyk.iter().sum()
}

/// The dark colours the report quotes: the navy frame of the sample cover, black, and
/// dark red, green, brown and purple.
const DARK: [(&str, Rgb8); 6] = [
    ("navy", Rgb8::new(14, 22, 48)),
    ("black", Rgb8::new(0, 0, 0)),
    ("deep red", Rgb8::new(90, 10, 20)),
    ("forest green", Rgb8::new(20, 60, 30)),
    ("brown", Rgb8::new(70, 40, 20)),
    ("purple", Rgb8::new(50, 20, 70)),
];

#[test]
fn the_a2b_tables_read_like_little_cms() {
    let p = profile();
    let mut worst = 0.0_f64;
    let mut rows = 0;
    for line in include_str!("fixtures/lcms-a2b-fogra39.tsv").lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (cmyk, lab) = line.split_once('\t').unwrap();
        let cmyk: Vec<f64> = cmyk.split(' ').map(|v| v.parse::<f64>().unwrap() / 255.0).collect();
        let lab: Vec<f64> = lab.split(' ').map(|v| v.parse().unwrap()).collect();
        let ours = p.cmyk_to_lab(
            [cmyk[0], cmyk[1], cmyk[2], cmyk[3]],
            RenderingIntent::RelativeColorimetric,
        );
        let de =
            delta_e2000_lab(Lab::new(ours[0], ours[1], ours[2]), Lab::new(lab[0], lab[1], lab[2]));
        worst = worst.max(de);
        rows += 1;
    }
    assert!(rows > 600, "{rows}");
    // 8-bit Lab in the fixture: ±0.2 L*, ±0.5 a* and b*.
    assert!(worst < 1.0, "A2B1 differs from Little CMS by ΔE2000 {worst:.2}");
}

#[test]
fn dark_colours_are_rebuilt_under_the_limit_and_stay_close() {
    let p = profile();
    let link = SrgbToCmyk::relative_bpc(&p);
    for limit_percent in [240.0_f32, 300.0] {
        let limit = InkLimit::new(limit_percent).unwrap();
        for (name, colour) in DARK {
            let original = link.convert(colour);
            let limited = p.limit_ink(original, limit);
            let ink = total(limited.cmyk) * 100.0;
            eprintln!(
                "{limit_percent:>3} %  {name:<12} {:>5.1} % -> C{:.0} M{:.0} Y{:.0} K{:.0} = {ink:.1} %, ΔE2000 {:.2}",
                total(original) * 100.0,
                limited.cmyk[0] * 100.0,
                limited.cmyk[1] * 100.0,
                limited.cmyk[2] * 100.0,
                limited.cmyk[3] * 100.0,
                limited.delta_e,
            );
            assert!(ink <= f64::from(limit_percent) + 1e-6, "{name}: {ink}");
            assert!(limited.cmyk.iter().all(|v| (0.0..=1.0).contains(v)));
            // The reported ΔE is the real one.
            assert!((p.delta_e(original, limited.cmyk) - limited.delta_e).abs() < 1e-12);
            // Grey-component replacement keeps dark colours close to their appearance;
            // naive scaling of all four inks would not.
            assert!(limited.delta_e < 3.0, "{name} at {limit_percent} %: {}", limited.delta_e);
            let scaled = original.map(|v| v * f64::from(limit_percent) / 100.0 / total(original));
            if total(original) * 100.0 > f64::from(limit_percent) {
                assert!(p.delta_e(original, scaled) > limited.delta_e, "{name}");
            }
        }
    }
}

#[test]
fn colours_within_the_limit_are_untouched() {
    let p = profile();
    let limit = InkLimit::new(240.0).unwrap();
    for cmyk in [[0.6, 0.4, 0.4, 1.0], [0.0; 4], [1.0, 0.0, 1.0, 0.2], [0.1, 0.2, 0.3, 0.4]] {
        let limited = p.limit_ink(cmyk, limit);
        assert_eq!(limited.cmyk, cmyk);
        assert_eq!(limited.delta_e, 0.0);
    }
    // Hostile channels are cleaned, never trusted.
    let limited = p.limit_ink([f64::NAN, 7.0, -3.0, f64::INFINITY], limit);
    assert!(limited.cmyk.iter().all(|v| (0.0..=1.0).contains(v)));
    assert!(total(limited.cmyk) <= 2.4 + 1e-9);
    // A maximum black is respected, even below the colour's own black.
    let capped = p.limit_ink([0.7, 0.6, 0.6, 1.0], limit.with_max_black(90.0));
    assert!(capped.cmyk[3] <= 0.9 + 1e-12 && total(capped.cmyk) <= 2.4 + 1e-9);
    // A limit under the black alone reduces black too.
    let tiny = p.limit_ink([0.5, 0.5, 0.5, 1.0], InkLimit::new(80.0).unwrap());
    assert!(total(tiny.cmyk) <= 0.8 + 1e-9, "{tiny:?}");
}

#[test]
fn the_limited_transform_keeps_every_pixel_under_the_limit() {
    let p = profile();
    let plain = SrgbToCmyk::relative_bpc(&p);
    let link = plain.clone().with_ink_limit(&p, InkLimit::new(240.0).unwrap());
    assert_eq!(link.ink_limit().map(|l| l.total_percent()), Some(240.0));
    assert!(link.rebuilt_nodes() > 0);
    assert_eq!(plain.ink_limit(), None);
    let mut pixels = Vec::new();
    for r in (0..=255u16).step_by(5) {
        for g in (0..=255u16).step_by(5) {
            for b in (0..=255u16).step_by(5) {
                pixels.extend_from_slice(&[r as u8, g as u8, b as u8]);
                let c = Rgb8::new(r as u8, g as u8, b as u8);
                let limited = link.convert(c);
                assert!(total(limited) <= 2.4 + 1e-6, "{c}: {limited:?}");
                let unlimited = link.convert_unlimited(c);
                assert_eq!(unlimited, plain.convert(c));
                if total(unlimited) <= 2.4 {
                    // Within the limit, the profile's own conversion.
                    assert_eq!(limited, unlimited, "{c}");
                    assert_eq!(link.ink_limit_delta_e(c), 0.0);
                }
            }
        }
    }
    let cmyk = link.convert_pixels(&pixels);
    for px in cmyk.chunks_exact(4) {
        let sum: u32 = px.iter().map(|&v| u32::from(v)).sum();
        assert!(sum <= 612, "{px:?}");
    }
    // A 300 % limit on a 300 % profile changes nothing.
    let pro = plain.clone().with_ink_limit(&p, InkLimit::new(300.0).unwrap());
    assert_eq!(pro.rebuilt_nodes(), 0);
    assert_eq!(pro.convert_pixels(&pixels), plain.convert_pixels(&pixels));
}

#[test]
fn the_cost_of_the_limit_is_measured_per_colour() {
    let p = profile();
    let link = SrgbToCmyk::relative_bpc(&p).with_ink_limit(&p, InkLimit::new(240.0).unwrap());
    for (name, colour) in DARK {
        let de = link.ink_limit_delta_e(colour);
        eprintln!("grid  240 %  {name:<12} ΔE2000 {de:.2}");
        assert!(de < 4.0, "{name}: {de}");
    }
    assert_eq!(link.ink_limit_delta_e(Rgb8::new(232, 220, 194)), 0.0);
}
