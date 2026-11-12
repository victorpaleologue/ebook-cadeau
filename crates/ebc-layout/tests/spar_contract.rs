//! Sparring tests: the crate against the letter of `docs/architecture.md` §5 (ebc-layout).
//!
//! The contract says: "For `home` profiles the PDF page **is** the sheet (MediaBox = A4
//! landscape, TrimBox = the spread centred in it) and marks must stay inside the printable
//! area."

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, Fit, LayoutError, Marks, Panels, PrintProfile, Sheet, marks, new_page,
    page_geometry, place_image, safe_rect,
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
/// be A4 landscape, 297 x 210 mm. `new_page` returns a 262 x 210 mm page instead.
///
/// Why it matters beyond the letter of the contract: the two faces only register if the
/// spread is centred on the *physical* sheet. With a 262 mm wide page that is left to the
/// PDF viewer's handling of a non-standard page size on A4 paper (centre, top-left, or
/// scale to fit) instead of being fixed by the file.
///
/// `ebc_scene::Page` has a single `slug` for all four sides, so this needs a per-axis slug
/// or an explicit media rectangle there (and in `ebc-pdf`) before `new_page` can comply.
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
    assert_eq!(
        profile,
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
