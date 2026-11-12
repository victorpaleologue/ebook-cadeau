//! Sparring tests: the crate against the letter of `docs/architecture.md` §5 (ebc-layout).
//!
//! The contract says: "For `home` profiles the PDF page **is** the sheet (MediaBox = A4
//! landscape, TrimBox = the spread centred in it) and marks must stay inside the printable
//! area."

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, Face, Fit, LayoutError, Marks, Panels, PrintProfile, Sheet, marks, new_page,
    new_page_for, page_geometry, place_image, safe_rect,
};
use ebc_scene::{Item, Page, PdfStandard, Rect, Size};

fn rect_close(a: Rect, b: Rect) -> bool {
    let close = |x: f64, y: f64| (x - y).abs() < 1e-9;
    close(a.x, b.x) && close(a.y, b.y) && close(a.w, b.w) && close(a.h, b.h)
}

/// The sheet in landscape with `spread` centred on it, in page coordinates.
fn sheet_around(spread: Size, sheet: Sheet) -> Rect {
    let s = sheet.landscape();
    Rect::new(-(s.w - spread.w) / 2.0, -(s.h - spread.h) / 2.0, s.w, s.h)
}

/// The contract's own example: the default card printed at home on A4. The PDF page must
/// be A4 landscape, 297 x 210 mm. (When this test was written `new_page` returned a
/// 262 x 210 mm page; see the note at the end of this comment.)
///
/// Why it matters beyond the letter of the contract: the two faces only register if the
/// spread is centred on the *physical* sheet. With a 262 mm wide page that is left to the
/// PDF viewer's handling of a non-standard page size on A4 paper (centre, top-left, or
/// scale to fit) instead of being fixed by the file.
///
/// `ebc_scene::Page` had a single `slug` for all four sides, so this needed a per-axis
/// slug there (and in `ebc-pdf`) before `new_page` could comply. `Page::slug_v` is that
/// change; this test and the next are its acceptance tests.
#[test]
fn home_a4_page_is_a4_landscape() {
    let format = CardFormat::poche();
    let page = new_page(&format, &PrintProfile::home(Sheet::A4)).unwrap();
    let media = page.media_box();
    assert_eq!(page.trim_box(), Rect::new(0.0, 0.0, 230.0, 178.0), "TrimBox = the spread");
    assert!(
        rect_close(media, Rect::new(-33.5, -16.0, 297.0, 210.0)),
        "MediaBox must be A4 landscape (297 x 210 mm) around the centred spread, got {media:?}"
    );
}

/// Same for every preset on every named sheet it fits on.
#[test]
fn home_media_box_is_the_sheet_for_every_preset_that_fits() {
    let mut wrong = Vec::new();
    let mut fitted = 0;
    for (sheet_name, sheet) in Sheet::NAMED {
        for (format_name, format) in CardFormat::presets() {
            let profile = PrintProfile::home(sheet);
            let Ok(page) = new_page(&format, &profile) else { continue };
            fitted += 1;
            let want = sheet_around(format.spread(), sheet);
            let got = page.media_box();
            if !rect_close(got, want) {
                wrong.push(format!(
                    "{format_name} on {sheet_name}: MediaBox {} x {} mm, sheet is {} x {} mm",
                    got.w, got.h, want.w, want.h
                ));
            }
            // The geometry report must not contradict the page either.
            let g = page_geometry(&format, &profile).unwrap();
            assert!(rect_close(g.sheet.unwrap().sheet, want), "{format_name} on {sheet_name}");
        }
    }
    // poche and a6 on A4 and Letter, all four presets on A3 and SRA3.
    assert_eq!(fitted, 12);
    assert!(
        wrong.is_empty(),
        "{} of {fitted} home pages are not the sheet:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// The rest of the contract sentence, for the same twelve combinations: the `TrimBox` is
/// the spread, centred in the sheet; the `BleedBox` is the spread plus the bleed; the
/// marks stay inside the printable area. And why a vertical slug is needed at all: with
/// one slug for all four sides no page is the sheet.
#[test]
fn home_page_is_the_sheet_with_the_spread_centred_and_printable_marks() {
    let mut fitted = 0;
    for (sheet_name, sheet) in Sheet::NAMED {
        for (format_name, format) in CardFormat::presets() {
            let ctx = format!("{format_name} on {sheet_name}");
            let profile = PrintProfile::home(sheet);
            let Ok(page) = new_page(&format, &profile) else { continue };
            fitted += 1;
            let spread = format.spread();
            let want = sheet_around(spread, sheet);
            let (mx, my) = (-want.x, -want.y);

            // The sheet's margins around the spread differ (by 4.25 mm at least — A6 on
            // Letter — and by 17.5 mm for poche on A4), so no single slug describes it...
            assert!((mx - my).abs() > 4.0, "{ctx}: margins {mx} and {my}");
            for slug in [0.0, mx - profile.bleed, my - profile.bleed] {
                let media = Page::new(spread, profile.bleed, slug).media_box();
                assert!(!rect_close(media, want), "{ctx}: slug {slug}");
            }
            // ...and the page carries one per direction.
            let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
            assert!(close(page.slug, mx - profile.bleed), "{ctx}: slug {}", page.slug);
            let slug_v = page.slug_v.expect("a home page has a vertical slug");
            assert!(close(slug_v, my - profile.bleed), "{ctx}: vertical slug {slug_v}");

            let media = page.media_box();
            assert!(rect_close(media, want), "{ctx}: {media:?}");
            assert_eq!(page.trim_box(), Rect::new(0.0, 0.0, spread.w, spread.h), "{ctx}");
            assert_eq!(page.bleed_box(), page.trim_box().inflate(profile.bleed), "{ctx}");
            assert!(close(media.center().x, page.trim_box().center().x), "{ctx}");
            assert!(close(media.center().y, page.trim_box().center().y), "{ctx}");

            let g = page_geometry(&format, &profile).unwrap();
            let placement = g.sheet.unwrap();
            assert_eq!(g.media, media, "{ctx}: the report and the page agree");
            assert!(placement.landscape && placement.media_is_sheet, "{ctx}");
            // Marks: inside the page and inside the printable area of the sheet.
            assert!(media.contains_rect(&g.mark_area, 1e-9), "{ctx}");
            assert!(placement.printable.contains_rect(&g.mark_area, 1e-9), "{ctx}");
            assert!(
                rect_close(placement.printable, want.inflate(-profile.printable_margin)),
                "{ctx}"
            );
            // Both faces get the same page.
            for face in Face::BOTH {
                let labelled = new_page_for(face, &format, &profile).unwrap();
                assert_eq!(labelled.media_box(), media, "{ctx}");
            }
        }
    }
    assert_eq!(fitted, 12);

    // A sheet whose margins around the spread are equal needs no second slug value, but
    // gets the same treatment.
    let format = CardFormat::poche();
    let sheet = Sheet::Custom(Size::new(230.0 + 40.0, 178.0 + 40.0));
    let profile = PrintProfile::home(sheet);
    let page = new_page(&format, &profile).unwrap();
    assert!(rect_close(page.media_box(), Rect::new(-20.0, -20.0, 270.0, 218.0)));
    assert!(page_geometry(&format, &profile).unwrap().sheet.unwrap().media_is_sheet);
    // Press pages are untouched: one slug, no vertical slug.
    let press = new_page(&format, &PrintProfile::professional_eu()).unwrap();
    assert_eq!(press.slug_v, None);
    assert_eq!(press.media_box(), Rect::new(-13.0, -13.0, 256.0, 204.0));
}

/// Every name the contract lists, used the way the contract writes it. This is a compile
/// check as much as a run-time one: other crates are written against these names.
#[test]
fn the_contract_surface_exists_and_behaves() {
    assert_eq!(CardFormat::SPINE_DEFAULT_MM, 10.0);
    let formats: [CardFormat; 4] =
        [CardFormat::poche(), CardFormat::a6(), CardFormat::digest(), CardFormat::a5()];
    assert_eq!(formats[0].cover, Size::new(110.0, 178.0));
    assert_eq!(formats[2].cover, Size::new(130.0, 200.0));
    let custom: Result<CardFormat, LayoutError> = CardFormat::custom(Size::new(120.0, 190.0), 8.0);
    let custom = custom.unwrap();
    assert_eq!(custom.spread(), Size::new(248.0, 190.0));
    let outside: Panels = custom.outside();
    let inside: Panels = custom.inside();
    assert_eq!((outside.left.w, outside.spine.w, outside.right.w), (120.0, 8.0, 120.0));
    assert_eq!(inside.spine, outside.spine);
    let folds: [f64; 2] = custom.fold_x();
    assert_eq!(folds, [120.0, 128.0]);

    // The contract's `PrintProfile` is a plain struct with exactly these fields.
    let profile = PrintProfile {
        name: "house".to_owned(),
        bleed: 3.0,
        safe_margin: 4.0,
        spine_safe_margin: 1.5,
        slug: 10.0,
        marks: Marks::press(),
        standard: PdfStandard::X4,
        target_ppi: 300.0,
        min_ppi: 150.0,
        max_total_ink_percent: 300.0,
        sheet: None,
        printable_margin: 0.0,
    };
    // `professional_eu()` is that profile but for the two safety margins: when this test
    // was written they were 4 mm and 1.5 mm; docs/research/03-print-requirements.md (§1.4
    // rows 4 and 5, implication 2), which did not exist yet, sets 5 mm and 2 mm.
    assert_eq!(
        PrintProfile { safe_margin: 5.0, spine_safe_margin: 2.0, ..profile.clone() },
        PrintProfile { name: "house".to_owned(), ..PrintProfile::professional_eu() }
    );
    let m = profile.marks;
    let _: (bool, bool, bool, bool, f64, f64, f64) =
        (m.crop, m.fold, m.registration, m.color_bar, m.offset, m.length, m.weight_pt);
    for sheet in
        [Sheet::A4, Sheet::Letter, Sheet::A3, Sheet::Sra3, Sheet::Custom(Size::new(1.0, 1.0))]
    {
        let home: PrintProfile = PrintProfile::home(sheet);
        assert_eq!(home.sheet, Some(sheet));
    }

    let page: Result<Page, LayoutError> = new_page(&custom, &profile);
    let page = page.unwrap();
    assert_eq!(page.trim, custom.spread());
    let layer: Item = marks(&custom, &profile);
    assert!(matches!(&layer, Item::Group { layer: Some(name), .. } if name == "marks"));
    let safe: Rect = safe_rect(outside.right, &profile);
    assert_eq!(safe, Rect::new(132.0, 4.0, 112.0, 182.0));
    let placed: Rect = place_image((600, 900), outside.right, Fit::Cover);
    assert!(placed.contains_rect(&outside.right, 1e-9));
    let placed: Rect = place_image((600, 900), outside.right, Fit::Contain);
    assert!(outside.right.contains_rect(&placed, 1e-9));
    // "errors if the spread cannot fit the sheet"
    assert!(matches!(
        new_page(&CardFormat::a5(), &PrintProfile::home(Sheet::A4)),
        Err(LayoutError::DoesNotFit { .. })
    ));
}
