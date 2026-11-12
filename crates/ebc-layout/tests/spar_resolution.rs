//! Sparring tests: pixel requirements and resolution verdicts must agree with each other
//! and with plain arithmetic on inch-based sizes (the sizes cover scans are prepared at).
//!
//! `PrintProfile::target_px` / `min_px` compute `ceil(mm / 25.4 * ppi)` and
//! `effective_ppi` computes `px / (mm / 25.4)`. `mm / 25.4` is not exact in binary
//! floating point: `152.4 / 25.4` is `6.000000000000001`, so a 6 inch panel "needs"
//! 1801 px at 300 ppi and an 1800 px scan of it is reported as 299.99999999999994 ppi,
//! i.e. below target.

#![allow(clippy::unwrap_used)]

use ebc_layout::{PrintProfile, Resolution, Sheet, effective_ppi};
use ebc_scene::{Rect, Size};

fn at(size: Size) -> Rect {
    Rect::new(0.0, 0.0, size.w, size.h)
}

/// 6 x 9 in (152.4 x 228.6 mm) is the most common US trade paperback trim. At 300 ppi it
/// is 1800 x 2700 px — not 1801.
#[test]
fn target_px_of_a_6_by_9_inch_cover_is_1800_by_2700() {
    let p = PrintProfile::professional_eu();
    assert_eq!(p.target_ppi, 300.0);
    assert_eq!(p.target_px(Size::new(152.4, 228.6)), (1800, 2700));
}

/// Whole and half inches, written the way a caller writes them (millimetre literals).
#[test]
fn pixel_requirements_are_exact_on_inch_based_sizes() {
    // (millimetres, inches)
    let lengths = [
        (25.4, 1.0),
        (50.8, 2.0),
        (76.2, 3.0),
        (101.6, 4.0),
        (127.0, 5.0),
        (139.7, 5.5),
        (152.4, 6.0),
        (177.8, 7.0),
        (203.2, 8.0),
        (215.9, 8.5),
        (228.6, 9.0),
        (254.0, 10.0),
        (279.4, 11.0),
    ];
    let mut wrong = Vec::new();
    for profile in [PrintProfile::professional_eu(), PrintProfile::home(Sheet::A3)] {
        for (mm, inches) in lengths {
            let size = Size::new(mm, mm);
            for (label, ppi, got) in [
                ("target_px", profile.target_ppi, profile.target_px(size).0),
                ("min_px", profile.min_ppi, profile.min_px(size).0),
            ] {
                // inches * ppi is a whole number for every pair above.
                let want = (inches * ppi).round() as u32;
                if got != want {
                    wrong.push(format!(
                        "{}: {label}({mm} mm = {inches} in at {ppi} ppi) = {got}, expected {want}",
                        profile.name
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{} wrong pixel requirements:\n{}", wrong.len(), wrong.join("\n"));
}

/// A scan made at exactly the target resolution must be judged `Good`, not `Low`.
#[test]
fn a_scan_at_exactly_300_ppi_is_good() {
    let p = PrintProfile::professional_eu();
    // (pixels, millimetres): 6 x 9 in, 3 x 7 in, at 300 ppi.
    for (px, size) in
        [((1800, 2700), Size::new(152.4, 228.6)), ((900, 2100), Size::new(76.2, 177.8))]
    {
        let ppi = effective_ppi(px, at(size));
        assert_eq!(
            p.judge_ppi(ppi),
            Resolution::Good,
            "{px:?} px on {size:?} is exactly 300 ppi but effective_ppi says {ppi:?}"
        );
    }
}

/// The crate's own advice must be self-consistent: an image with exactly the pixels
/// `target_px` asks for is `Good`, one with exactly `min_px` is not `TooLow`, and the
/// requirement is tight (one pixel fewer is not enough).
#[test]
fn an_image_with_the_required_pixels_passes_the_verdict() {
    let mut not_good = Vec::new();
    let mut too_low = Vec::new();
    let mut not_tight = Vec::new();
    let mut checked = 0u32;
    for profile in [PrintProfile::professional_eu(), PrintProfile::home(Sheet::A3)] {
        // Every cover dimension from 50 to 300 mm in steps of 0.01 mm.
        for hundredths in 5_000u32..=30_000 {
            let mm = f64::from(hundredths) / 100.0;
            let size = Size::new(mm, mm);
            checked += 1;

            let (tw, th) = profile.target_px(size);
            let verdict = profile.judge_ppi(effective_ppi((tw, th), at(size)));
            if verdict != Resolution::Good {
                not_good.push(format!(
                    "{}: target_px({mm}) = {tw} px, but {tw} px on {mm} mm is judged {verdict:?}",
                    profile.name
                ));
            }
            let (mw, mh) = profile.min_px(size);
            let verdict = profile.judge_ppi(effective_ppi((mw, mh), at(size)));
            if verdict == Resolution::TooLow {
                too_low.push(format!(
                    "{}: min_px({mm}) = {mw} px, but {mw} px on {mm} mm is judged TooLow",
                    profile.name
                ));
            }
            // Tightness, with exact integer arithmetic: (tw - 1) px must be below target,
            // i.e. (tw - 1) * 25.4 < mm * ppi  <=>  (tw - 1) * 2540 < hundredths * ppi.
            let ppi = profile.target_ppi as u64;
            if tw > 0 && u64::from(tw - 1) * 2540 >= u64::from(hundredths) * ppi {
                not_tight.push(format!(
                    "{}: target_px({mm}) = {tw} px, but {} px already reach {ppi} ppi",
                    profile.name,
                    tw - 1
                ));
            }
        }
    }
    let show = |v: &[String]| v.iter().take(5).cloned().collect::<Vec<_>>().join("\n  ");
    assert!(
        not_good.is_empty() && too_low.is_empty() && not_tight.is_empty(),
        "of {checked} sizes:\n\
         {} where the target pixel count is not judged Good, e.g.\n  {}\n\
         {} where the minimum pixel count is judged TooLow, e.g.\n  {}\n\
         {} where target_px asks for one pixel too many, e.g.\n  {}",
        not_good.len(),
        show(&not_good),
        too_low.len(),
        show(&too_low),
        not_tight.len(),
        show(&not_tight),
    );
}
