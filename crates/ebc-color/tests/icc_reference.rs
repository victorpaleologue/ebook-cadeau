//! The colour-management module against Little CMS.
//!
//! `tests/fixtures/lcms-fogra39.tsv` was written by Pillow's `ImageCms` (Little CMS 2.19)
//! from the bundled basICColor "ISO Coated v2 300%" profile (FOGRA39L): for a 9 × 9 × 9
//! sRGB lattice and a few card colours, the 8-bit CMYK of an sRGB → CMYK transform
//! (relative colorimetric with black-point compensation, and perceptual), and the Lab of
//! that CMYK through the profile's `A2B1` table (8-bit Lab, so ±0.4 L\* and ±0.5 a\*/b\*).
//! Regenerate it with `tests/fixtures/lcms_reference.py` (needs Python 3 and Pillow).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ebc_color::{CmykProfile, Lab, RenderingIntent, Rgb8, SrgbToCmyk, delta_e2000_lab};

fn profile() -> CmykProfile {
    let path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc");
    CmykProfile::parse(&std::fs::read(path).expect("bundled profile")).expect("parses")
}

struct Row {
    intent: String,
    rgb: Rgb8,
    cmyk: [u8; 4],
    lab: [f64; 3],
}

fn rows() -> Vec<Row> {
    let text = include_str!("fixtures/lcms-fogra39.tsv");
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            let nums = |s: &str| -> Vec<f64> { s.split(' ').map(|v| v.parse().unwrap()).collect() };
            let rgb = nums(cols[1]);
            let cmyk = nums(cols[2]);
            let lab = nums(cols[3]);
            Row {
                intent: cols[0].to_owned(),
                rgb: Rgb8::new(rgb[0] as u8, rgb[1] as u8, rgb[2] as u8),
                cmyk: [cmyk[0] as u8, cmyk[1] as u8, cmyk[2] as u8, cmyk[3] as u8],
                lab: [lab[0], lab[1], lab[2]],
            }
        })
        .collect()
}

fn lab(v: [f64; 3]) -> Lab {
    Lab::new(v[0], v[1], v[2])
}

fn unit(cmyk: [u8; 4]) -> [f64; 4] {
    cmyk.map(|v| f64::from(v) / 255.0)
}

#[test]
fn the_profile_tables_read_like_little_cms() {
    // A2B1 of the CMYK Little CMS produced gives the Lab Little CMS printed (8-bit Lab).
    let p = profile();
    let mut worst = 0.0_f64;
    for row in rows() {
        let ours = p.cmyk_to_lab(unit(row.cmyk), RenderingIntent::RelativeColorimetric);
        let de = delta_e2000_lab(lab(ours), lab(row.lab));
        worst = worst.max(de);
    }
    assert!(worst < 1.0, "A2B1 differs from Little CMS by ΔE2000 {worst:.2}");
}

fn check(intent_name: &str, link: &SrgbToCmyk) -> (f64, f64, f64) {
    let p = profile();
    let (mut worst_de, mut sum_de, mut n, mut worst_channel) = (0.0_f64, 0.0, 0.0, 0.0_f64);
    for row in rows().into_iter().filter(|r| r.intent == intent_name) {
        let ours = link.convert(row.rgb);
        let theirs = unit(row.cmyk);
        for (a, b) in ours.iter().zip(theirs) {
            worst_channel = worst_channel.max((a - b).abs());
        }
        let de = delta_e2000_lab(
            lab(p.cmyk_to_lab(ours, RenderingIntent::RelativeColorimetric)),
            lab(p.cmyk_to_lab(theirs, RenderingIntent::RelativeColorimetric)),
        );
        worst_de = worst_de.max(de);
        sum_de += de;
        n += 1.0;
    }
    (worst_de, sum_de / n, worst_channel)
}

#[test]
fn relative_colorimetric_with_bpc_matches_little_cms() {
    let link = SrgbToCmyk::relative_bpc(&profile());
    let (worst, mean, channel) = check("relative_bpc", &link);
    eprintln!("relative+BPC: ΔE2000 worst {worst:.3}, mean {mean:.3}, channel {channel:.4}");
    assert!(worst <= 2.0, "worst ΔE2000 {worst:.2}");
    assert!(mean <= 0.5, "mean ΔE2000 {mean:.2}");
}

#[test]
fn perceptual_matches_little_cms() {
    let link = SrgbToCmyk::new(&profile(), RenderingIntent::Perceptual, false);
    let (worst, mean, channel) = check("perceptual", &link);
    eprintln!("perceptual: ΔE2000 worst {worst:.3}, mean {mean:.3}, channel {channel:.4}");
    assert!(worst <= 2.0, "worst ΔE2000 {worst:.2}");
    assert!(mean <= 0.5, "mean ΔE2000 {mean:.2}");
}

#[test]
fn white_is_paper_and_black_stays_under_the_ink_limit() {
    let link = SrgbToCmyk::relative_bpc(&profile());
    // Paper white: no ink (to within the profile's 16-bit tables).
    assert!(link.convert(Rgb8::WHITE).iter().all(|v| *v < 1e-4));
    assert_eq!(link.convert_pixels(&[255, 255, 255]), [0, 0, 0, 0]);
    let black = link.convert(Rgb8::BLACK);
    let total: f64 = black.iter().sum();
    assert!(total <= 3.0 + 1e-6 && total > 2.5, "black {black:?}");
    // Pixels and flat colours go through the same grid.
    let px = link.convert_pixels(&[250, 240, 220, 0, 0, 0, 1, 2]);
    assert_eq!(px.len(), 8);
    let flat = link.convert(Rgb8::new(250, 240, 220));
    for (a, b) in px[..4].iter().zip(flat) {
        assert!((f64::from(*a) / 255.0 - b).abs() <= 0.5 / 255.0 + 1e-9);
    }
}

#[test]
fn the_conversion_is_deterministic() {
    let a = SrgbToCmyk::relative_bpc(&profile());
    let b = SrgbToCmyk::relative_bpc(&profile());
    let pixels: Vec<u8> = (0..=255u8).flat_map(|v| [v, 255 - v, v / 3]).collect();
    assert_eq!(a.convert_pixels(&pixels), b.convert_pixels(&pixels));
}
