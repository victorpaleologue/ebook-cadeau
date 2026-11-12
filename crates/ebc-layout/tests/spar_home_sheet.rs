//! Sparring tests: which way round the home sheet is used.
//!
//! `page_geometry` uses the sheet in landscape whenever the card fits that way and only
//! then tries portrait. For a spread that is taller than it is wide this can pick the
//! orientation with (almost) no margin above and below — so no cut or fold guide fits —
//! although the same sheet turned by 90° has centimetres of room on every side.

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, MIN_MARK_LENGTH_MM, PrintProfile, Sheet, crop_marks, fold_marks, marks,
    page_geometry,
};
use ebc_scene::{Item, Size};

/// Margin between the bleed edge and the printable edge, on the tighter side, when
/// `spread` is centred on `sheet`; `None` if the card does not fit.
fn band(spread: Size, sheet: Size, profile: &PrintProfile) -> Option<f64> {
    let reserved = profile.bleed + profile.printable_margin;
    let tight = ((sheet.w - spread.w) / 2.0).min((sheet.h - spread.h) / 2.0);
    (tight >= reserved - 1e-9).then(|| tight - reserved)
}

/// A 50 x 195 mm cover with a 10 mm spine: spread 110 x 195 mm. On A4 portrait it has
/// 50 mm left and right and 51 mm above and below. Home printing needs its guides: without
/// fold ticks nobody can find the two creases.
#[test]
fn a_tall_card_on_a4_gets_its_cut_and_fold_guides() {
    let format = CardFormat::custom(Size::new(50.0, 195.0), 10.0).unwrap();
    let profile = PrintProfile::home(Sheet::A4);
    let g = page_geometry(&format, &profile).unwrap();
    let placement = g.sheet.unwrap();
    let Item::Group { items, .. } = marks(&format, &profile) else { panic!("not a layer") };
    assert!(
        !crop_marks(&format, &profile).is_empty()
            && !fold_marks(&format, &profile).is_empty()
            && !items.is_empty(),
        "no cut or fold guides at all: the sheet is used {} with a {:.1} mm mark band, \
         although A4 portrait leaves 50 mm around this 110 x 195 mm spread",
        if placement.landscape { "in landscape" } else { "in portrait" },
        g.mark_band_mm(),
    );
}

/// The same, as a property over a grid of formats and the four named sheets: when both
/// orientations fit, the one chosen must not be the one without room for marks if the
/// other one has room.
#[test]
fn the_orientation_with_room_for_marks_is_chosen_when_there_is_one() {
    let mut wrong = Vec::new();
    let mut both_fit = 0u32;
    for (sheet_name, sheet) in Sheet::NAMED {
        let profile = PrintProfile::home(sheet);
        for w in (50..=300).step_by(10) {
            for h in (50..=300).step_by(5) {
                let format =
                    CardFormat::custom(Size::new(f64::from(w), f64::from(h)), 10.0).unwrap();
                let spread = format.spread();
                let (Some(landscape), Some(portrait)) = (
                    band(spread, sheet.landscape(), &profile),
                    band(spread, sheet.portrait(), &profile),
                ) else {
                    continue;
                };
                both_fit += 1;
                let got = page_geometry(&format, &profile).unwrap().mark_band_mm();
                let best = landscape.max(portrait);
                if got < MIN_MARK_LENGTH_MM && best >= MIN_MARK_LENGTH_MM {
                    wrong.push(format!(
                        "{w} x {h} mm covers on {sheet_name}: mark band {got:.1} mm (no marks), \
                         the other orientation offers {best:.1} mm"
                    ));
                }
            }
        }
    }
    assert!(both_fit > 500, "the grid must exercise the choice ({both_fit})");
    assert!(
        wrong.is_empty(),
        "{} of {both_fit} cards lose all their marks to the orientation choice, e.g.\n{}",
        wrong.len(),
        wrong.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
    );
}
