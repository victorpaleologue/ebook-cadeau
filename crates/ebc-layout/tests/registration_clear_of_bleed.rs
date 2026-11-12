//! What a printer's preflight sees of the marks: nothing in the registration colour may
//! touch the bleed box.
//!
//! A preflight tool looks at one painted object at a time and judges it by its bounding
//! box — the box around **all** of its sub-paths, grown by half the stroke width in every
//! direction, since it does not know that the caps are butt. Two things used to make
//! `ebc-preflight` report `COLOR_REGISTRATION` on `professional_eu` pages:
//!
//! * the eight crop marks were one path, whose box is the whole page;
//! * the marks started exactly on the bleed edge, so that even a single mark's box
//!   reached half a hairline into the bleed box.
//!
//! Both are checked here for every preset format and every named profile, as the
//! profile asks for its marks, with every kind of mark switched on, and with the marks
//! forced into the registration colour.

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, MARK_BLEED_GAP_MM, MarkStyle, PrintProfile, marks, new_page, page_geometry,
};
use ebc_scene::{Color, Item, Rect, pt_to_mm};

/// The box of every object painted in the registration colour, as a preflight tool
/// computes it, with a description for failure messages.
fn registration_boxes(item: &Item, out: &mut Vec<(Rect, String)>) {
    match item {
        Item::Path { path, fill, stroke } => {
            let filled = fill.as_ref().is_some_and(|(c, _)| *c == Color::Registration);
            let stroked = stroke.as_ref().filter(|s| s.color == Color::Registration);
            if !filled && stroked.is_none() {
                return;
            }
            let half = stroked.map_or(0.0, |s| pt_to_mm(s.width_pt) / 2.0);
            let bounds = path.control_bounds().expect("a mark without geometry");
            out.push((bounds.inflate(half), format!("{} segment(s)", path.segs.len())));
        }
        Item::Group { transform, items, .. } => {
            assert_eq!(*transform, ebc_scene::Transform::IDENTITY, "marks are not transformed");
            for item in items {
                registration_boxes(item, out);
            }
        }
        Item::Glyphs { color, .. } => {
            assert_ne!(*color, Color::Registration, "no text in the registration colour");
        }
        Item::Image { .. } => {}
    }
}

/// `true` if the two rectangles overlap or merely touch.
fn touches(a: &Rect, b: &Rect) -> bool {
    a.x <= b.right() && a.right() >= b.x && a.y <= b.bottom() && a.bottom() >= b.y
}

/// The profile as it is, with every kind of mark switched on, and with those marks in the
/// registration colour (a slug is added to the profiles that have none).
fn variants(base: &PrintProfile) -> Vec<(String, PrintProfile)> {
    let mut all_on = base.clone();
    all_on.marks.crop = true;
    all_on.marks.fold = true;
    all_on.marks.registration = true;
    all_on.marks.color_bar = true;
    all_on.marks.scale_ruler = true;
    if all_on.sheet.is_none() {
        all_on.slug = all_on.slug.max(10.0);
    }
    let mut press = all_on.clone();
    press.marks.style = MarkStyle::Press;
    let mut heavy = press.clone();
    heavy.marks.weight_pt = ebc_layout::MAX_MARK_WEIGHT_PT;
    vec![
        (base.name.clone(), base.clone()),
        (format!("{} + every mark", base.name), all_on),
        (format!("{} + every mark, press style", base.name), press),
        (format!("{} + every mark, press style, heaviest stroke", base.name), heavy),
    ]
}

#[test]
fn no_registration_coloured_object_touches_the_bleed_box_on_any_preset() {
    let mut pages = 0;
    let mut objects = 0;
    for (format_name, format) in CardFormat::presets() {
        for profile_name in PrintProfile::NAMES {
            let base = PrintProfile::by_name(profile_name).unwrap();
            for (label, profile) in variants(&base) {
                let ctx = format!("{format_name} / {label}");
                let Ok(mut page) = new_page(&format, &profile) else {
                    // The digest and A5 cards do not fit an A4 or Letter sheet.
                    assert!(profile.sheet.is_some(), "{ctx}");
                    continue;
                };
                page.items.push(marks(&format, &profile));
                let bleed = page.bleed_box();
                assert_eq!(bleed, page_geometry(&format, &profile).unwrap().bleed, "{ctx}");

                let mut boxes = Vec::new();
                for item in &page.items {
                    registration_boxes(item, &mut boxes);
                }
                for (object, what) in &boxes {
                    assert!(
                        !touches(object, &bleed),
                        "{ctx}: a registration-coloured object ({what}) with box {object:?} \
                         touches the bleed box {bleed:?}"
                    );
                    assert!(object.intersect(&bleed).is_none(), "{ctx}");
                }
                pages += 1;
                objects += boxes.len();

                // Not vacuous: press-style marks are there, in the registration colour.
                if profile.marks.style == MarkStyle::Press && profile.marks.any() {
                    assert!(boxes.len() >= 8 + 4, "{ctx}: {} objects", boxes.len());
                    // And this is why they are separate objects: together their box is
                    // the page, bleed box included.
                    let union = boxes.iter().map(|(r, _)| *r).reduce(|a, b| a.union(&b)).unwrap();
                    assert!(union.contains_rect(&bleed, 0.0), "{ctx}");
                }
            }
        }
    }
    assert!(pages >= 4 * 4 * 3, "only {pages} pages were checked");
    assert!(objects > 500, "only {objects} registration-coloured objects were checked");
}

#[test]
fn the_professional_profile_keeps_its_marks_a_documented_gap_from_the_bleed() {
    let (format, profile) = (CardFormat::poche(), PrintProfile::professional_eu());
    assert_eq!(profile.marks.color(), Color::Registration);
    let mut boxes = Vec::new();
    registration_boxes(&marks(&format, &profile), &mut boxes);
    assert_eq!(boxes.len(), 8 + 4, "eight crop marks and four fold marks");
    let bleed = page_geometry(&format, &profile).unwrap().bleed;
    let half = pt_to_mm(profile.marks.weight_pt) / 2.0;
    for (object, _) in &boxes {
        // Distance from the bleed box, along whichever axis separates the two.
        let dx = (bleed.x - object.right()).max(object.x - bleed.right());
        let dy = (bleed.y - object.bottom()).max(object.y - bleed.bottom());
        let distance = dx.max(dy);
        assert!(
            (distance - (MARK_BLEED_GAP_MM - half)).abs() < 1e-9,
            "{object:?} is {distance} mm from the bleed box"
        );
    }
}
