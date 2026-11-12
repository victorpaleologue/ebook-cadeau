//! Sparring tests: can a finisher tell a fold mark from a crop mark?
//!
//! On the press profile the four fold marks are drawn exactly like the eight crop marks:
//! same solid stroke, same colour, same weight, same distance from the trim, same length.
//! Along the top edge of the default card there are four identical vertical ticks at
//! x = 0, 110, 120 and 230 mm; two of them mean "cut here" and two mean "crease here".
//!
//! The trade convention — and both research notes this crate cites — is solid crop marks
//! and **dashed** fold marks:
//!
//! * `docs/research/notes/print-requirements.prepress.md` §2: "fold marks for the two
//!   spine folds drawn the same way, dashed";
//! * `docs/research/notes/print-requirements.folding.md` §2: "Dashed fold ticks vs solid
//!   crop marks is the DTP convention".
//!
//! The home profile already does this ("so they are not mistaken for cut marks",
//! `MarkStyle::Guide`); the press profile, where a guillotine does the cutting, does not.

#![allow(clippy::unwrap_used)]

use ebc_layout::{CardFormat, PrintProfile, Sheet, crop_marks, fold_marks};
use ebc_scene::{Item, PathSeg, Point, Stroke};

/// The stroke and the straight segments of each stroked path item.
fn strokes(items: &[Item]) -> Vec<(&Stroke, Vec<(Point, Point)>)> {
    items
        .iter()
        .map(|item| {
            let Item::Path { path, stroke: Some(stroke), fill: None } = item else {
                panic!("marks are stroked paths: {item:?}")
            };
            let mut segments = Vec::new();
            let mut cur = Point::default();
            for seg in &path.segs {
                match *seg {
                    PathSeg::MoveTo(p) => cur = p,
                    PathSeg::LineTo(p) => {
                        segments.push((cur, p));
                        cur = p;
                    }
                    _ => {}
                }
            }
            (stroke, segments)
        })
        .collect()
}

/// `true` if some fold-mark stroke differs from the crop-mark stroke in a way a person
/// can see on paper: a dash pattern, another weight or another colour.
fn distinguishable(format: &CardFormat, profile: &PrintProfile) -> bool {
    let crop = crop_marks(format, profile);
    let fold = fold_marks(format, profile);
    let crop = strokes(&crop);
    let fold = strokes(&fold);
    assert_eq!(crop.len(), 1, "one crop-mark item");
    assert!(!fold.is_empty(), "fold marks are drawn");
    let (crop_stroke, _) = crop[0];
    assert!(crop_stroke.dash_mm.is_empty(), "crop marks are solid");
    fold.iter().any(|(s, _)| {
        s.dash_mm != crop_stroke.dash_mm
            || s.width_pt != crop_stroke.width_pt
            || s.color != crop_stroke.color
    })
}

#[test]
fn home_fold_marks_are_told_apart_from_cut_marks() {
    for (name, format) in CardFormat::presets() {
        assert!(distinguishable(&format, &PrintProfile::home(Sheet::A3)), "{name}");
    }
}

#[test]
fn press_fold_marks_are_told_apart_from_crop_marks() {
    let format = CardFormat::poche();
    let profile = PrintProfile::professional_eu();
    // What the top edge looks like today: every vertical tick above the card, by x.
    let mut top_ticks: Vec<(f64, f64, f64, bool)> = Vec::new();
    for items in [crop_marks(&format, &profile), fold_marks(&format, &profile)] {
        for (stroke, segments) in strokes(&items) {
            for (a, b) in segments {
                if a.x == b.x && a.y < 0.0 {
                    top_ticks.push((a.x, a.y.max(b.y), a.y.min(b.y), stroke.dash_mm.is_empty()));
                }
            }
        }
    }
    top_ticks.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(
        distinguishable(&format, &profile),
        "fold marks and crop marks are drawn identically on {}; vertical ticks above the \
         card as (x, from y, to y, solid): {top_ticks:?}",
        profile.name
    );
}
