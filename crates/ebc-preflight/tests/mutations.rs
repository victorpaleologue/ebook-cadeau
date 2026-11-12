//! One mutation of the clean reference card per check: each must trigger exactly the
//! finding code that check owns, and nothing else.

#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::{
    BLEED, COVER_H, Fixture, TRIM_W, card_expectations, codes, icc_profile, name, rect_mm, set, string, xmp,
};
use ebc_preflight::{Expectations, PreflightError, Report, Severity, check, codes as c};
use lopdf::encryption::crypt_filters::{Aes128CryptFilter, CryptFilter};
use lopdf::{Dictionary, EncryptionState, EncryptionVersion, Object, Permissions, Stream, dictionary};

/// Asserts that the report carries exactly one finding code, at the given severity.
fn assert_only(report: &Report, code: &'static str, severity: Severity) {
    assert_eq!(codes(report), set(&[code]), "{:#?}", report.findings);
    for finding in &report.findings {
        assert_eq!(finding.severity, severity, "{finding}");
    }
    assert_eq!(report.passed(), severity != Severity::Error);
}

fn finding<'a>(report: &'a Report, code: &str) -> &'a ebc_preflight::Finding {
    report.findings.iter().find(|f| f.code == code).unwrap_or_else(|| panic!("no {code} in {:#?}", report.findings))
}

// ---- page boxes ----------------------------------------------------------------------------

#[test]
fn missing_trim_box() {
    let mut f = Fixture::reference();
    f.dict(f.page1).remove(b"TrimBox");
    let report = f.check();
    assert_only(&report, c::BOX_TRIM_MISSING, Severity::Error);
    assert_eq!(finding(&report, c::BOX_TRIM_MISSING).page, Some(1));
    assert_eq!(report.pages[0].trim_mm, None);
    assert_eq!(report.pages[1].trim_mm, Some((TRIM_W, COVER_H)));

    // An ArtBox is not a substitute.
    f.dict(f.page1).set("ArtBox", rect_mm(BLEED, BLEED, BLEED + TRIM_W, BLEED + COVER_H));
    let report = f.check();
    assert_only(&report, c::BOX_TRIM_MISSING, Severity::Error);
    assert!(finding(&report, c::BOX_TRIM_MISSING).message.contains("ArtBox"));
}

#[test]
fn bleed_of_one_millimetre() {
    let mut f = Fixture::reference();
    f.dict(f.page2).set("BleedBox", rect_mm(2.0, 2.0, BLEED + TRIM_W + 1.0, BLEED + COVER_H + 1.0));
    let report = f.check();
    assert_only(&report, c::BOX_BLEED_TOO_SMALL, Severity::Error);
    let found = finding(&report, c::BOX_BLEED_TOO_SMALL);
    assert_eq!(found.page, Some(2));
    assert!(found.message.contains("1.00 mm left"), "{}", found.message);
    let bleed = report.pages[1].bleed_mm.unwrap();
    assert_eq!((bleed.left, bleed.right, bleed.top, bleed.bottom), (1.0, 1.0, 1.0, 1.0));
    // The same file is fine for a printer that asks for 1 mm.
    let relaxed = f.check_with(&Expectations { bleed_mm: 1.0, ..card_expectations() });
    assert!(relaxed.findings.is_empty(), "{:#?}", relaxed.findings);
}

#[test]
fn bleed_missing_on_one_side_only() {
    let mut f = Fixture::reference();
    // Bleed everywhere except at the top edge.
    f.dict(f.page1).set("BleedBox", rect_mm(0.0, 0.0, BLEED + TRIM_W + BLEED, BLEED + COVER_H));
    let report = f.check();
    assert_only(&report, c::BOX_BLEED_TOO_SMALL, Severity::Error);
    assert!(finding(&report, c::BOX_BLEED_TOO_SMALL).message.contains("0.00 mm top"));
}

#[test]
fn bleed_box_absent_falls_back_to_the_media_box() {
    let mut f = Fixture::reference();
    f.dict(f.page1).remove(b"BleedBox");
    f.dict(f.page2).remove(b"BleedBox");
    let report = f.check();
    assert_only(&report, c::BOX_BLEED_MISSING, Severity::Warning);
    assert_eq!(report.findings.len(), 2);
    assert_eq!(report.pages[0].bleed_mm.unwrap().min(), 3.0);
}

#[test]
fn boxes_not_nested() {
    let mut f = Fixture::reference();
    // The trim box pokes out of the bleed (and media) box on the left.
    f.dict(f.page1).set("TrimBox", rect_mm(-1.0, BLEED, BLEED + TRIM_W - 4.0, BLEED + COVER_H));
    let report = f.check();
    assert_only(&report, c::BOX_NOT_NESTED, Severity::Error);

    let mut f = Fixture::reference();
    f.dict(f.page1).set("BleedBox", rect_mm(-5.0, -5.0, 241.0, 189.0));
    let report = f.check();
    assert_only(&report, c::BOX_NOT_NESTED, Severity::Error);
    assert!(finding(&report, c::BOX_NOT_NESTED).message.contains("MediaBox"));
}

#[test]
fn media_box_missing_or_malformed() {
    let mut f = Fixture::reference();
    f.dict(f.page1).remove(b"MediaBox");
    f.dict(f.page2).remove(b"MediaBox");
    let report = f.check();
    assert_only(&report, c::BOX_MEDIA_MISSING, Severity::Error);

    let mut f = Fixture::reference();
    f.dict(f.page1).set("MediaBox", vec![0.into(), 0.into(), name("wide"), 100.into()]);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::BOX_INVALID, c::BOX_MEDIA_MISSING]), "{:#?}", report.findings);
}

#[test]
fn media_box_is_inherited_from_the_page_tree() {
    let mut f = Fixture::reference();
    let media = f.dict(f.page1).remove(b"MediaBox").unwrap();
    f.dict(f.page2).remove(b"MediaBox");
    f.dict(f.pages).set("MediaBox", media);
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert!((report.pages[1].media_box.unwrap().width() - 236.0).abs() < 0.001);
}

#[test]
fn wrong_trim_size() {
    let mut f = Fixture::reference();
    let report = f.check_with(&card_expectations().with_trim(210.0, 148.0));
    assert_only(&report, c::BOX_TRIM_SIZE, Severity::Error);
    assert_eq!(report.findings.len(), 2, "one per page");
    assert!(report.findings[0].message.contains("230.00 × 178.00 mm, expected 210.00 × 148.00 mm"));
    // Within a tenth of a millimetre is accepted.
    let report = f.check_with(&card_expectations().with_trim(230.08, 177.93));
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
}

#[test]
fn pages_of_different_sizes() {
    let mut f = Fixture::reference();
    f.dict(f.page2).set("TrimBox", rect_mm(BLEED, BLEED, BLEED + TRIM_W - 20.0, BLEED + COVER_H));
    f.dict(f.page2).set("BleedBox", rect_mm(0.0, 0.0, TRIM_W - 20.0 + 2.0 * BLEED, COVER_H + 2.0 * BLEED));
    // No expected trim size here, to isolate the comparison between pages.
    let report = f.check_with(&Expectations { trim_mm: None, ..card_expectations() });
    assert_only(&report, c::PAGE_SIZE_MISMATCH, Severity::Error);
    assert_eq!(finding(&report, c::PAGE_SIZE_MISMATCH).page, Some(2));
}

#[test]
fn rotated_page_user_unit_crop_box_and_art_box_are_warnings() {
    let mut f = Fixture::reference();
    f.dict(f.page1).set("Rotate", 90);
    assert_only(&f.check(), c::PAGE_ROTATED, Severity::Warning);

    let mut f = Fixture::reference();
    f.dict(f.page1).set("CropBox", rect_mm(BLEED, BLEED, BLEED + TRIM_W, BLEED + COVER_H));
    assert_only(&f.check(), c::BOX_CROP_CLIPS_BLEED, Severity::Warning);

    let mut f = Fixture::reference();
    f.dict(f.page1).set("ArtBox", rect_mm(10.0, 10.0, 100.0, 100.0));
    assert_only(&f.check(), c::BOX_TRIM_AND_ART, Severity::Warning);

    // UserUnit 2 doubles every dimension: the trim size no longer matches, images halve
    // their resolution, and the unusual unit itself is flagged.
    let mut f = Fixture::reference();
    f.dict(f.page1).set("UserUnit", 2);
    let report = f.check();
    assert_eq!(
        codes(&report),
        set(&[c::PAGE_USER_UNIT, c::BOX_TRIM_SIZE, c::PAGE_SIZE_MISMATCH, c::EBC_GEOMETRY_MISMATCH]),
        "{:#?}",
        report.findings
    );
    assert_eq!(report.pages[0].trim_mm, Some((460.0, 356.0)));
    assert!((report.pages[0].images[0].ppi_x - 150.0).abs() < 0.5);
}

// ---- fonts ---------------------------------------------------------------------------------

#[test]
fn helvetica_not_embedded() {
    let mut f = Fixture::reference();
    let helvetica = f.doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
    });
    f.add_resource(f.page2, "Font", "F2", helvetica);
    f.append_content(f.content2, "BT /F2 10 Tf 0 0 0 1 k 300 300 Td (Bonne lecture) Tj ET");
    let report = f.check();
    assert_only(&report, c::FONT_NOT_EMBEDDED, Severity::Error);
    let found = finding(&report, c::FONT_NOT_EMBEDDED);
    assert_eq!(found.page, Some(2));
    assert!(found.message.contains("Helvetica (Type1)"), "{}", found.message);
    let fonts = &report.pages[1].fonts;
    assert_eq!(fonts.len(), 2);
    let helvetica = fonts.iter().find(|f| f.name == "Helvetica").unwrap();
    assert!(!helvetica.embedded && !helvetica.subset && helvetica.used);
    assert_eq!(report.pages[0].fonts.len(), 1, "page 1 does not list the font");
}

#[test]
fn font_listed_but_never_used_must_still_be_embedded() {
    let mut f = Fixture::reference();
    let times = f.doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Times-Roman" });
    f.add_resource(f.page1, "Font", "Unused", times);
    let report = f.check();
    assert_only(&report, c::FONT_NOT_EMBEDDED, Severity::Error);
    let font = report.pages[0].fonts.iter().find(|f| f.name == "Times-Roman").unwrap();
    assert!(!font.used);
}

#[test]
fn descendant_font_program_removed() {
    let mut f = Fixture::reference();
    f.dict(f.font_descriptor).remove(b"FontFile2");
    let report = f.check();
    assert_only(&report, c::FONT_NOT_EMBEDDED, Severity::Error);
    assert_eq!(report.findings.len(), 2, "the font is used on both pages");
    assert!(!report.pages[0].fonts[0].embedded);
}

#[test]
fn font_inside_a_form_xobject_is_checked() {
    let mut f = Fixture::reference();
    let courier = f.doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Courier" });
    let form = f.doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form",
            "BBox" => vec![0.into(), 0.into(), 200.into(), 50.into()],
            "Resources" => dictionary! { "Font" => dictionary! { "C" => courier } },
        },
        b"BT /C 8 Tf 0 0 0 1 k 0 0 Td (ABCDE-12345) Tj ET".to_vec(),
    ));
    f.add_resource(f.page2, "XObject", "Code", form);
    f.append_content(f.content2, "q 1 0 0 1 300 200 cm /Code Do Q");
    let report = f.check();
    assert_only(&report, c::FONT_NOT_EMBEDDED, Severity::Error);
    let courier = report.pages[1].fonts.iter().find(|f| f.name == "Courier").unwrap();
    assert!(courier.used, "the Tf inside the form marks the font as used");
}

#[test]
fn type3_and_fully_embedded_fonts() {
    let mut f = Fixture::reference();
    let glyph = f.doc.add_object(Stream::new(dictionary! {}, b"500 0 d0 0 0 400 400 re f".to_vec()));
    let type3 = f.doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type3", "Name" => "Logo",
        "FontBBox" => vec![0.into(), 0.into(), 500.into(), 500.into()],
        "FontMatrix" => vec![Object::Real(0.001), 0.into(), 0.into(), Object::Real(0.001), 0.into(), 0.into()],
        "CharProcs" => dictionary! { "a" => glyph },
        "Encoding" => dictionary! { "Type" => "Encoding", "Differences" => vec![97.into(), name("a")] },
        "FirstChar" => 97, "LastChar" => 97, "Widths" => vec![500.into()],
    });
    f.add_resource(f.page1, "Font", "T3", type3);
    let report = f.check();
    assert_only(&report, c::FONT_TYPE3, Severity::Warning);

    let mut f = Fixture::reference();
    for id in [f.font, f.cid_font] {
        f.dict(id).set("BaseFont", "AtkinsonHyperlegible-Regular");
    }
    let report = f.check();
    assert_only(&report, c::FONT_NOT_SUBSET, Severity::Info);
    assert!(report.pages[0].fonts[0].embedded && !report.pages[0].fonts[0].subset);
}

// ---- images --------------------------------------------------------------------------------

#[test]
fn image_at_50_ppi() {
    let mut f = Fixture::reference();
    f.dict(f.cover_image).set("Width", 222);
    f.dict(f.cover_image).set("Height", 362);
    let report = f.check();
    assert_only(&report, c::IMAGE_LOW_RES, Severity::Error);
    let found = finding(&report, c::IMAGE_LOW_RES);
    assert_eq!(found.page, Some(1));
    assert!(found.message.contains("222×362 px"), "{}", found.message);
    assert!(found.message.contains("50 ppi"), "{}", found.message);
    let image = &report.pages[0].images[0];
    assert!((image.ppi_x - 49.9).abs() < 0.2 && (image.ppi_y - 50.0).abs() < 0.2, "{image:?}");
    // A caller with a lower floor accepts it.
    let relaxed = f.check_with(&Expectations { min_ppi: 45.0, ..card_expectations() });
    assert!(relaxed.findings.is_empty());
}

#[test]
fn resolution_threshold_is_exact_enough() {
    // 113 mm wide: 667 px is 149.9 ppi (accepted, within half a percent), 660 px is 148.4 ppi.
    let mut f = Fixture::reference();
    f.dict(f.cover_image).set("Width", 667);
    assert!(f.check().findings.is_empty());
    f.dict(f.cover_image).set("Width", 660);
    assert_only(&f.check(), c::IMAGE_LOW_RES, Severity::Error);
    // Only one axis is too coarse: still an error.
    let mut f = Fixture::reference();
    f.dict(f.cover_image).set("Height", 500);
    assert_only(&f.check(), c::IMAGE_LOW_RES, Severity::Error);
}

#[test]
fn rgb_and_icc_rgb_images() {
    let mut f = Fixture::reference();
    f.dict(f.cover_image).set("ColorSpace", "DeviceRGB");
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert!(finding(&report, c::COLOR_RGB).message.contains("image /Im1 uses DeviceRGB"));
    assert_eq!(report.pages[0].images[0].color_space, "DeviceRGB");
    assert!(report.pages[0].color_spaces.contains(&"DeviceRGB".to_string()));

    // An sRGB-tagged image is still RGB for a CMYK job…
    let srgb = f.doc.add_object(Stream::new(dictionary! { "N" => 3 }, icc_profile(b"mntr", b"RGB ")));
    f.dict(f.cover_image).set("ColorSpace", vec![name("ICCBased"), srgb.into()]);
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert_eq!(report.pages[0].images[0].color_space, "ICCBased(3)");
    // …and fine for a PDF/X-4 workflow that accepts colour-managed RGB.
    let report = f.check_with(&Expectations { allow_rgb: true, ..card_expectations() });
    assert!(report.findings.is_empty(), "{:#?}", report.findings);

    // Indexed images inherit the verdict of their base space.
    f.dict(f.cover_image).set(
        "ColorSpace",
        vec![name("Indexed"), name("DeviceRGB"), 1.into(), Object::string_literal(vec![0_u8; 6])],
    );
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert_eq!(report.pages[0].images[0].color_space, "Indexed(DeviceRGB)");
}

#[test]
fn raw_cmyk_image_over_the_ink_limit() {
    let mut f = Fixture::reference();
    // 20 × 20 px, every pixel at C100 M100 Y100 K60 = 360 %.
    let pixels: Vec<u8> = [255, 255, 255, 153].repeat(400);
    let mut stream = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => 20, "Height" => 20,
            "ColorSpace" => "DeviceCMYK", "BitsPerComponent" => 8,
        },
        pixels,
    );
    stream.compress().unwrap();
    let image = f.doc.add_object(stream);
    f.add_resource(f.page2, "XObject", "Dark", image);
    f.append_content(f.content2, "q 4 0 0 4 300 300 cm /Dark Do Q");
    let report = f.check();
    assert_only(&report, c::IMAGE_INK_TOTAL_TOO_HIGH, Severity::Warning);
    assert!(finding(&report, c::IMAGE_INK_TOTAL_TOO_HIGH).message.starts_with("100 % of the pixels"));
    // With a 400 % limit (i.e. none in practice) the image is fine.
    let report = f.check_with(&Expectations { max_total_ink_percent: Some(400.0), ..card_expectations() });
    assert!(report.findings.is_empty());
}

// ---- colour --------------------------------------------------------------------------------

#[test]
fn device_rgb_fill() {
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 1 0 0 rg 100 100 50 50 re f Q");
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    let found = finding(&report, c::COLOR_RGB);
    assert_eq!(found.page, Some(1));
    assert!(found.message.contains("a fill uses DeviceRGB"), "{}", found.message);
    assert_eq!(report.pages[0].color_spaces, vec!["DeviceCMYK".to_string(), "DeviceRGB".to_string()]);
    assert_eq!(report.pages[1].color_spaces, vec!["DeviceCMYK".to_string()]);
    // Allowed when the caller says so.
    let report = f.check_with(&Expectations { allow_rgb: true, ..card_expectations() });
    assert!(report.findings.is_empty());
}

#[test]
fn rgb_that_is_set_but_never_painted_is_not_flagged() {
    let mut f = Fixture::reference();
    // Colours are set, a path is built and discarded with `n`, a clip is set: nothing is painted.
    f.append_content(f.content1, "q 1 0 0 rg 0 1 0 RG 10 10 50 50 re W n Q 0 0 1 rg");
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert_eq!(report.pages[0].color_spaces, vec!["DeviceCMYK".to_string()]);
}

#[test]
fn rgb_strokes_text_and_shadings() {
    // A stroke.
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 0 0 1 RG 2 w 100 100 m 200 200 l S Q");
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert!(finding(&report, c::COLOR_RGB).message.contains("a stroke uses DeviceRGB"));

    // Text: fill colour for render mode 0, stroke colour for mode 1, nothing for mode 3.
    let mut f = Fixture::reference();
    f.append_content(f.content1, "BT /F1 12 Tf 1 0 0 rg 100 100 Td <0001> Tj ET");
    assert_only(&f.check(), c::COLOR_RGB, Severity::Error);
    let mut f = Fixture::reference();
    f.append_content(f.content1, "BT /F1 12 Tf 1 0 0 RG 1 Tr 100 100 Td <0001> Tj ET");
    assert_only(&f.check(), c::COLOR_RGB, Severity::Error);
    let mut f = Fixture::reference();
    f.append_content(f.content1, "BT /F1 12 Tf 1 0 0 rg 1 0 0 RG 3 Tr 100 100 Td <0001> Tj ET");
    assert!(f.check().findings.is_empty(), "invisible text paints nothing");

    // A shading painted with `sh`, and one used as a pattern.
    let shading = dictionary! {
        "ShadingType" => 2, "ColorSpace" => "DeviceRGB",
        "Coords" => vec![0.into(), 0.into(), 100.into(), 0.into()],
        "Function" => dictionary! {
            "FunctionType" => 2, "Domain" => vec![0.into(), 1.into()], "N" => 1,
            "C0" => vec![1.into(), 0.into(), 0.into()], "C1" => vec![0.into(), 0.into(), 1.into()],
        },
    };
    let mut f = Fixture::reference();
    f.add_resource(f.page1, "Shading", "Sh1", shading.clone());
    f.append_content(f.content1, "q 100 100 50 50 re W n /Sh1 sh Q");
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert!(finding(&report, c::COLOR_RGB).message.contains("shading /Sh1"));

    let mut f = Fixture::reference();
    f.add_resource(f.page1, "Pattern", "P1", dictionary! { "PatternType" => 2, "Shading" => shading });
    f.append_content(f.content1, "q /Pattern cs /P1 scn 100 100 50 50 re f Q");
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert!(finding(&report, c::COLOR_RGB).message.contains("shading pattern /P1"));
}

#[test]
fn tiling_pattern_content_is_interpreted() {
    let mut f = Fixture::reference();
    let cell = f.doc.add_object(Stream::new(
        dictionary! {
            "Type" => "Pattern", "PatternType" => 1, "PaintType" => 1, "TilingType" => 1,
            "BBox" => vec![0.into(), 0.into(), 10.into(), 10.into()], "XStep" => 10, "YStep" => 10,
            "Resources" => dictionary! {},
        },
        b"0 1 0 rg 0 0 5 5 re f".to_vec(),
    ));
    f.add_resource(f.page1, "Pattern", "Tile", cell);
    f.append_content(f.content1, "q /Pattern cs /Tile scn 100 100 50 50 re f 200 100 50 50 re f Q");
    let report = f.check();
    assert_only(&report, c::COLOR_RGB, Severity::Error);
    assert_eq!(report.findings.len(), 1);
}

#[test]
fn spot_colour_is_a_warning() {
    let mut f = Fixture::reference();
    let tint = dictionary! {
        "FunctionType" => 2, "Domain" => vec![0.into(), 1.into()], "N" => 1,
        "C0" => vec![0.into(), 0.into(), 0.into(), 0.into()], "C1" => vec![0.into(), 1.into(), 1.into(), 0.into()],
    };
    f.add_resource(
        f.page1,
        "ColorSpace",
        "Red",
        vec![name("Separation"), name("PANTONE 485 C"), name("DeviceCMYK"), Object::Dictionary(tint)],
    );
    f.append_content(f.content1, "q /Red cs 1 scn 100 100 50 50 re f Q");
    let report = f.check();
    assert_only(&report, c::COLOR_SPOT, Severity::Warning);
    assert!(report.pages[0].color_spaces.contains(&"Separation(PANTONE 485 C)".to_string()));
}

/// Gives both pages a 10 mm slug around the bleed box and defines registration colour.
fn with_slug_and_registration() -> Fixture {
    let mut f = Fixture::reference();
    let tint = dictionary! {
        "FunctionType" => 2, "Domain" => vec![0.into(), 1.into()], "N" => 1,
        "C0" => vec![0.into(), 0.into(), 0.into(), 0.into()], "C1" => vec![1.into(), 1.into(), 1.into(), 1.into()],
    };
    for page in [f.page1, f.page2] {
        f.dict(page).set("MediaBox", rect_mm(-10.0, -10.0, 246.0, 194.0));
        f.add_resource(
            page,
            "ColorSpace",
            "Reg",
            vec![name("Separation"), name("All"), name("DeviceCMYK"), Object::Dictionary(tint.clone())],
        );
    }
    f
}

#[test]
fn registration_colour_belongs_in_the_slug() {
    // Crop marks in the slug, ending exactly at the bleed edge: fine.
    let mut f = with_slug_and_registration();
    let mm = common::MM;
    let marks = format!(
        "q /Reg CS 1 SCN 0.25 w {x0:.3} {y:.3} m {x1:.3} {y:.3} l S {vx:.3} {vy0:.3} m {vx:.3} {vy1:.3} l S Q",
        x0 = -8.0 * mm,
        x1 = -0.2 * mm,
        y = 3.0 * mm,
        vx = 3.0 * mm,
        vy0 = -8.0 * mm,
        vy1 = -0.2 * mm,
    );
    f.append_content(f.content1, &marks);
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert!(report.pages[0].color_spaces.contains(&"Separation(All)".to_string()));

    // A mark that runs into the bleed box prints on the product.
    let mut f = with_slug_and_registration();
    f.append_content(f.content1, &format!("q /Reg CS 1 SCN 0.25 w {:.3} 20 m {:.3} 20 l S Q", -8.0 * mm, 4.0 * mm));
    let report = f.check();
    assert_only(&report, c::COLOR_REGISTRATION, Severity::Error);

    // A registration fill in the middle of the cover: 400 % ink.
    let mut f = with_slug_and_registration();
    f.append_content(f.content1, "q /Reg cs 1 scn 100 100 20 20 re f Q");
    assert_only(&f.check(), c::COLOR_REGISTRATION, Severity::Error);

    // A zero tint paints nothing.
    let mut f = with_slug_and_registration();
    f.append_content(f.content1, "q /Reg cs 0 scn 100 100 20 20 re f Q");
    assert!(f.check().findings.is_empty());

    // Text in registration colour (a slug line): its extent is unknown, so only a warning.
    let mut f = with_slug_and_registration();
    f.append_content(f.content1, "BT /F1 6 Tf /Reg cs 1 scn -20 -20 Td <0001> Tj ET");
    assert_only(&f.check(), c::COLOR_REGISTRATION, Severity::Warning);
}

#[test]
fn ink_at_380_percent() {
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 1 0.9 0.9 1 k 100 100 50 50 re f Q");
    let report = f.check();
    assert_only(&report, c::INK_TOTAL_TOO_HIGH, Severity::Error);
    let found = finding(&report, c::INK_TOTAL_TOO_HIGH);
    assert_eq!(found.page, Some(1));
    assert!(found.message.contains("C100 M90 Y90 K100 = 380 %"), "{}", found.message);
    assert_eq!(report.pages[0].max_total_ink_percent, Some(380.0));
    // No limit, no finding (but the maximum is still reported).
    let report = f.check_with(&Expectations { max_total_ink_percent: None, ..card_expectations() });
    assert!(report.findings.is_empty());
    assert_eq!(report.pages[0].max_total_ink_percent, Some(380.0));
}

#[test]
fn ink_limit_boundaries_and_other_ways_to_set_cmyk() {
    // Exactly 300 % is allowed; 301 % is not.
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 1 1 0.5 0.5 k 100 100 50 50 re f Q");
    assert!(f.check().findings.is_empty());
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 1 1 0.51 0.5 k 100 100 50 50 re f Q");
    assert_only(&f.check(), c::INK_TOTAL_TOO_HIGH, Severity::Error);

    // A stroke set with K, a fill set with cs/scn, and text.
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 1 1 1 1 K 3 w 100 100 m 150 150 l S Q");
    let report = f.check();
    assert_only(&report, c::INK_TOTAL_TOO_HIGH, Severity::Error);
    assert!(finding(&report, c::INK_TOTAL_TOO_HIGH).message.contains("a stroke uses"));
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q /DeviceCMYK cs 0.9 0.9 0.9 0.9 scn 100 100 50 50 re f Q");
    assert_only(&f.check(), c::INK_TOTAL_TOO_HIGH, Severity::Error);
    let mut f = Fixture::reference();
    f.append_content(f.content1, "BT /F1 12 Tf 1 1 1 1 k 100 100 Td <0001> Tj ET");
    assert_only(&f.check(), c::INK_TOTAL_TOO_HIGH, Severity::Error);

    // Heavy ink that is set but not used, or used only in the slug, does not count.
    let mut f = Fixture::reference();
    f.append_content(f.content1, "q 1 1 1 1 k 1 1 1 1 K 100 100 50 50 re n Q");
    assert!(f.check().findings.is_empty());
    let mut f = with_slug_and_registration();
    f.append_content(f.content1, "q 1 1 1 1 k -20 -20 5 5 re f Q");
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert_eq!(report.pages[0].max_total_ink_percent, Some(90.0));
}

// ---- transparency --------------------------------------------------------------------------

#[test]
fn soft_mask_on_an_image() {
    let mut f = Fixture::reference();
    let mask = f.doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => 2, "Height" => 2,
            "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8,
        },
        vec![0, 128, 200, 255],
    ));
    f.dict(f.cover_image).set("SMask", mask);
    let report = f.check();
    assert_only(&report, c::TRANSPARENCY, Severity::Error);
    assert!(finding(&report, c::TRANSPARENCY).message.contains("image /Im1 has a soft mask"));
    assert!(report.pages[0].transparency && !report.pages[1].transparency);
    assert!(report.pages[0].images[0].soft_mask);
    assert_eq!(report.pages[0].images.len(), 1, "the mask is not a placement of its own");
    // PDF/X-4 workflows accept live transparency.
    let report = f.check_with(&Expectations { allow_transparency: true, ..card_expectations() });
    assert!(report.findings.is_empty());
    assert!(report.pages[0].transparency, "still reported in the summary");
}

#[test]
fn graphics_state_transparency() {
    let cases: Vec<(Dictionary, &str)> = vec![
        (dictionary! { "Type" => "ExtGState", "ca" => Object::Real(0.5) }, "opacity ca = 0.500"),
        (dictionary! { "CA" => Object::Real(0.25) }, "opacity CA = 0.250"),
        (dictionary! { "BM" => "Multiply" }, "blend mode Multiply"),
        (dictionary! { "BM" => vec![name("Screen"), name("Normal")] }, "blend mode Screen"),
        (
            dictionary! { "SMask" => dictionary! { "Type" => "Mask", "S" => "Luminosity" } },
            "sets a soft mask",
        ),
    ];
    for (state, expected) in cases {
        let mut f = Fixture::reference();
        f.add_resource(f.page2, "ExtGState", "GS1", state);
        f.append_content(f.content2, "q /GS1 gs 0 0 0 1 k 300 300 20 20 re f Q");
        let report = f.check();
        assert_only(&report, c::TRANSPARENCY, Severity::Error);
        let found = finding(&report, c::TRANSPARENCY);
        assert_eq!(found.page, Some(2));
        assert!(found.message.contains(expected), "{} / {expected}", found.message);
    }
    // Opaque settings are not transparency.
    let mut f = Fixture::reference();
    f.add_resource(
        f.page2,
        "ExtGState",
        "GS1",
        dictionary! { "ca" => 1, "CA" => Object::Real(1.0), "BM" => "Normal", "SMask" => "None", "OP" => true },
    );
    f.append_content(f.content2, "q /GS1 gs 0 0 0 1 k 300 300 20 20 re f Q");
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert!(!report.pages[1].transparency);
    // An ExtGState that is defined but never applied is not a finding either.
    let mut f = Fixture::reference();
    f.add_resource(f.page2, "ExtGState", "GS1", dictionary! { "ca" => Object::Real(0.1) });
    assert!(f.check().findings.is_empty());
}

#[test]
fn transparency_groups() {
    let group = dictionary! { "Type" => "Group", "S" => "Transparency", "CS" => "DeviceCMYK" };
    let mut f = Fixture::reference();
    f.dict(f.page1).set("Group", group.clone());
    let report = f.check();
    assert_only(&report, c::TRANSPARENCY, Severity::Error);
    assert!(finding(&report, c::TRANSPARENCY).message.contains("page has a transparency group"));

    let mut f = Fixture::reference();
    let form = f.doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form", "Group" => group,
            "BBox" => vec![0.into(), 0.into(), 10.into(), 10.into()],
        },
        b"0 0 0 1 k 0 0 10 10 re f".to_vec(),
    ));
    f.add_resource(f.page1, "XObject", "Fm1", form);
    // Defined but not drawn: no finding. Drawn: transparency.
    assert!(f.check().findings.is_empty());
    f.append_content(f.content1, "q 1 0 0 1 300 300 cm /Fm1 Do Q");
    let report = f.check();
    assert_only(&report, c::TRANSPARENCY, Severity::Error);
    assert!(finding(&report, c::TRANSPARENCY).message.contains("form XObject /Fm1"));
}

// ---- structure -----------------------------------------------------------------------------

#[test]
fn javascript_action() {
    let mut f = Fixture::reference();
    let script = f.doc.add_object(dictionary! { "S" => "JavaScript", "JS" => string("app.alert('Bonjour');") });
    f.dict(f.catalog).set(
        "Names",
        dictionary! { "JavaScript" => dictionary! { "Names" => vec![string("init"), script.into()] } },
    );
    let report = f.check();
    assert_only(&report, c::JAVASCRIPT, Severity::Error);
    assert!(report.findings.iter().all(|f| f.page.is_none()));

    // As an action run on opening, the action itself is reported too.
    let mut f = Fixture::reference();
    let script = f.doc.add_object(dictionary! { "S" => "JavaScript", "JS" => string("this.print();") });
    f.dict(f.catalog).set("OpenAction", script);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::ACTION, c::JAVASCRIPT]), "{:#?}", report.findings);
    assert!(!report.passed());

    // A script that nothing references is still in the file.
    let mut f = Fixture::reference();
    f.doc.add_object(dictionary! { "S" => "JavaScript", "JS" => string("orphan()") });
    assert_only(&f.check(), c::JAVASCRIPT, Severity::Error);
}

#[test]
fn open_action_and_page_actions() {
    let mut f = Fixture::reference();
    let destination = vec![Object::Reference(f.page1), name("Fit")];
    f.dict(f.catalog).set("OpenAction", destination);
    assert_only(&f.check(), c::ACTION, Severity::Warning);

    let mut f = Fixture::reference();
    f.dict(f.catalog).set("OpenAction", dictionary! { "S" => "Named", "N" => "Print" });
    assert_only(&f.check(), c::ACTION, Severity::Error);

    let mut f = Fixture::reference();
    f.dict(f.page2).set("AA", dictionary! { "O" => dictionary! { "S" => "GoTo", "D" => vec![Object::Null] } });
    let report = f.check();
    assert_only(&report, c::ACTION, Severity::Error);
    assert_eq!(finding(&report, c::ACTION).page, Some(2));
}

#[test]
fn annotations() {
    // A link on the back cover.
    let mut f = Fixture::reference();
    let link = f.doc.add_object(dictionary! {
        "Type" => "Annot", "Subtype" => "Link", "Rect" => rect_mm(20.0, 20.0, 80.0, 30.0),
        "Border" => vec![0.into(), 0.into(), 0.into()],
        "A" => dictionary! { "S" => "URI", "URI" => string("https://ebk.example/k/ABCDE12345") },
    });
    f.dict(f.page1).set("Annots", vec![link.into()]);
    let report = f.check();
    assert_only(&report, c::ANNOTATION, Severity::Error);
    assert_eq!(finding(&report, c::ANNOTATION).page, Some(1));
    assert!(finding(&report, c::ANNOTATION).message.contains("Link"));

    // Printer's marks are what PDF/X expects as annotations.
    let mut f = Fixture::reference();
    let mark = f.doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "PrinterMark", "Rect" => rect_mm(0.0, 0.0, 5.0, 5.0) });
    f.dict(f.page1).set("Annots", vec![mark.into()]);
    assert!(f.check().findings.is_empty());

    // A note parked in the slug, outside the bleed box, is tolerated with a warning.
    let mut f = with_slug_and_registration();
    let note = f.doc.add_object(dictionary! { "Type" => "Annot", "Subtype" => "Text", "Rect" => rect_mm(-9.0, -9.0, -4.0, -4.0) });
    f.dict(f.page1).set("Annots", vec![note.into()]);
    assert_only(&f.check(), c::ANNOTATION, Severity::Warning);

    // Garbage in the Annots array counts as an annotation in the printed area.
    let mut f = Fixture::reference();
    f.dict(f.page1).set("Annots", vec![Object::Integer(3), Object::Reference((9999, 0))]);
    assert_only(&f.check(), c::ANNOTATION, Severity::Error);
}

#[test]
fn interactive_form_and_embedded_file() {
    let mut f = Fixture::reference();
    let field = f.doc.add_object(dictionary! {
        "Type" => "Annot", "Subtype" => "Widget", "FT" => "Tx", "T" => string("code"),
        "Rect" => rect_mm(150.0, 40.0, 220.0, 50.0),
    });
    f.dict(f.page2).set("Annots", vec![field.into()]);
    f.dict(f.catalog).set("AcroForm", dictionary! { "Fields" => vec![field.into()] });
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::ACROFORM, c::ANNOTATION]), "{:#?}", report.findings);

    let mut f = Fixture::reference();
    let file = f.doc.add_object(Stream::new(dictionary! { "Type" => "EmbeddedFile" }, b"epub bytes".to_vec()));
    let spec = f.doc.add_object(dictionary! {
        "Type" => "Filespec", "F" => string("book.epub"), "EF" => dictionary! { "F" => file },
    });
    f.dict(f.catalog).set(
        "Names",
        dictionary! { "EmbeddedFiles" => dictionary! { "Names" => vec![string("book.epub"), spec.into()] } },
    );
    assert_only(&f.check(), c::EMBEDDED_FILE, Severity::Error);
}

#[test]
fn layers_are_reported() {
    let mut f = Fixture::reference();
    let layer = f.doc.add_object(dictionary! { "Type" => "OCG", "Name" => string("marks") });
    f.dict(f.catalog).set(
        "OCProperties",
        dictionary! { "OCGs" => vec![layer.into()], "D" => dictionary! { "Order" => vec![layer.into()] } },
    );
    // PDF/X-4 allows layers: informational.
    assert_only(&f.check(), c::OPTIONAL_CONTENT, Severity::Info);
}

#[test]
fn encrypted_with_an_empty_user_password() {
    let mut f = Fixture::reference();
    let state = EncryptionState::try_from(EncryptionVersion::V2 {
        document: &f.doc,
        owner_password: "owner",
        user_password: "",
        key_length: 128,
        permissions: Permissions::PRINTABLE | Permissions::PRINTABLE_IN_HIGH_QUALITY,
    })
    .unwrap();
    f.doc.encrypt(&state).unwrap();
    let bytes = f.bytes();
    assert!(bytes.windows(8).any(|w| w == b"/Encrypt"));
    let report = check(&bytes, &card_expectations()).unwrap();
    // The file can be read (no password needed), so every other check still ran — and passed.
    assert_only(&report, c::ENCRYPTED, Severity::Error);
    assert!(report.document.encrypted);
    assert_eq!(report.pages.len(), 2);
    assert_eq!(report.document.title.as_deref(), Some("Le Petit Prince — carte cadeau"));
    assert_eq!(report.pages[0].images.len(), 1);
}

#[test]
fn encrypted_with_a_user_password() {
    let mut f = Fixture::reference();
    let filter: Arc<dyn CryptFilter> = Arc::new(Aes128CryptFilter);
    let state = EncryptionState::try_from(EncryptionVersion::V4 {
        document: &f.doc,
        encrypt_metadata: true,
        crypt_filters: BTreeMap::from([(b"StdCF".to_vec(), filter)]),
        stream_filter: b"StdCF".to_vec(),
        string_filter: b"StdCF".to_vec(),
        owner_password: "owner",
        user_password: "secret",
        permissions: Permissions::PRINTABLE,
    })
    .unwrap();
    f.doc.encrypt(&state).unwrap();
    let report = check(&f.bytes(), &card_expectations()).unwrap();
    assert_only(&report, c::ENCRYPTED, Severity::Error);
    assert!(report.document.encrypted);
    assert!(report.pages.is_empty(), "nothing can be read without the password");
    assert!(report.findings[0].message.contains("password"));
}

#[test]
fn encrypt_entry_with_a_bogus_dictionary() {
    let mut f = Fixture::reference();
    let bogus = f.doc.add_object(dictionary! {
        "Filter" => "Standard", "V" => 1, "R" => 2, "P" => -44,
        "O" => Object::string_literal(vec![0x11_u8; 32]), "U" => Object::string_literal(vec![0x22_u8; 32]),
    });
    f.doc.trailer.set("Encrypt", bogus);
    let report = check(&f.bytes(), &card_expectations()).unwrap();
    assert_only(&report, c::ENCRYPTED, Severity::Error);

    // An Encrypt entry that is not even a security handler lopdf knows.
    let mut f = Fixture::reference();
    let weird = f.doc.add_object(dictionary! { "Filter" => "Acme.DRM", "V" => 9 });
    f.doc.trailer.set("Encrypt", weird);
    match check(&f.bytes(), &card_expectations()) {
        Ok(report) => assert_only(&report, c::ENCRYPTED, Severity::Error),
        Err(error) => assert!(matches!(error, PreflightError::Parse(_)), "{error}"),
    }
}

// ---- output intent and PDF/X ---------------------------------------------------------------

#[test]
fn missing_output_intent() {
    let mut f = Fixture::reference();
    f.dict(f.catalog).remove(b"OutputIntents");
    let report = f.check();
    assert_only(&report, c::NO_OUTPUT_INTENT, Severity::Error);
    assert!(report.document.output_intent.is_none());
    // Not required for home printing; an empty array is the same as none.
    f.dict(f.catalog).set("OutputIntents", Vec::<Object>::new());
    assert_only(&f.check(), c::NO_OUTPUT_INTENT, Severity::Error);
    let report = f.check_with(&Expectations { require_output_intent: false, ..card_expectations() });
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
}

#[test]
fn malformed_output_intents() {
    // Wrong subtype.
    let mut f = Fixture::reference();
    f.dict(f.output_intent).set("S", "GTS_PDFA1");
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Error);
    assert!(finding(&report, c::OUTPUT_INTENT_INVALID).message.contains("GTS_PDFA1"));

    // No identifier.
    let mut f = Fixture::reference();
    f.dict(f.output_intent).remove(b"OutputConditionIdentifier");
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Error);
    assert!(finding(&report, c::OUTPUT_INTENT_INVALID).message.contains("OutputConditionIdentifier"));

    // Two PDF/X intents.
    let mut f = Fixture::reference();
    let intent = Object::Reference(f.output_intent);
    f.dict(f.catalog).set("OutputIntents", vec![intent.clone(), intent]);
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Error);
    assert!(finding(&report, c::OUTPUT_INTENT_INVALID).message.contains("2 GTS_PDFX"));

    // PDF/X-4 needs the profile embedded.
    let mut f = Fixture::reference();
    f.dict(f.output_intent).remove(b"DestOutputProfile");
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Error);
    assert!(finding(&report, c::OUTPUT_INTENT_INVALID).message.contains("PDF/X-4 requires"));

    // The "profile" is not an ICC profile.
    let mut f = Fixture::reference();
    f.set_content(f.icc, b"definitely not an ICC profile".to_vec());
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Error);
    assert!(finding(&report, c::OUTPUT_INTENT_INVALID).message.contains("not a readable ICC profile"));

    // The same problems are only warnings when no output intent is required.
    let mut f = Fixture::reference();
    f.dict(f.output_intent).remove(b"OutputConditionIdentifier");
    let report = f.check_with(&Expectations { require_output_intent: false, ..card_expectations() });
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Warning);
}

#[test]
fn output_profile_component_count() {
    // N says 3 but the profile is CMYK.
    let mut f = Fixture::reference();
    f.dict(f.icc).set("N", 3);
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_PROFILE_MISMATCH, Severity::Error);
    let message = &finding(&report, c::OUTPUT_INTENT_PROFILE_MISMATCH).message;
    assert!(message.contains("N = 3") && message.contains("CMYK (4 components)"), "{message}");

    // N says 4 but the profile is RGB.
    let mut f = Fixture::reference();
    f.set_content(f.icc, icc_profile(b"prtr", b"RGB "));
    assert_only(&f.check(), c::OUTPUT_INTENT_PROFILE_MISMATCH, Severity::Error);

    // A monitor profile with the right component count is suspicious, not fatal.
    let mut f = Fixture::reference();
    f.set_content(f.icc, icc_profile(b"mntr", b"CMYK"));
    let report = f.check();
    assert_only(&report, c::OUTPUT_INTENT_INVALID, Severity::Warning);

    // A Flate-compressed profile is decoded before it is inspected.
    let mut f = Fixture::reference();
    if let Object::Stream(stream) = f.doc.objects.get_mut(&f.icc).unwrap() {
        stream.set_plain_content(icc_profile(b"prtr", b"CMYK").repeat(20));
        stream.compress().unwrap();
        assert!(stream.dict.has(b"Filter"));
    }
    assert!(f.check().findings.is_empty());
}

#[test]
fn pdfx_markers_must_agree_with_each_other_and_with_the_header() {
    // Info says X-1a, XMP says X-4.
    let mut f = Fixture::reference();
    f.set_info("GTS_PDFXVersion", "PDF/X-1:2001");
    f.set_info("GTS_PDFXConformance", "PDF/X-1a:2001");
    let report = f.check();
    assert_only(&report, c::PDFX_VERSION_MISMATCH, Severity::Error);
    assert!(finding(&report, c::PDFX_VERSION_MISMATCH).message.contains("PDF/X-1a:2001"));

    // A PDF 1.7 file cannot be PDF/X-4.
    let mut f = Fixture::reference();
    f.doc.version = "1.7".to_string();
    let report = f.check();
    assert_only(&report, c::PDFX_VERSION_MISMATCH, Severity::Error);
    assert!(finding(&report, c::PDFX_VERSION_MISMATCH).message.contains("PDF 1.7"));

    // The catalog can raise the version above the header.
    let mut f = Fixture::reference();
    f.dict(f.catalog).set("Version", "1.7");
    assert_only(&f.check(), c::PDFX_VERSION_MISMATCH, Severity::Error);

    // PDF/X-1a:2001 in a PDF 1.6 file (and with an X-4 style XMP removed).
    let mut f = Fixture::reference();
    f.set_info("GTS_PDFXVersion", "PDF/X-1:2001");
    f.set_info("GTS_PDFXConformance", "PDF/X-1a:2001");
    f.set_content(f.metadata, xmp("PDF/X-1a:2001"));
    let report = f.check();
    assert_only(&report, c::PDFX_VERSION_MISMATCH, Severity::Error);
    assert!(finding(&report, c::PDFX_VERSION_MISMATCH).message.contains("PDF 1.3 at most"));
    // The same markers in a PDF 1.3 file are consistent.
    f.doc.version = "1.3".to_string();
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert_eq!(report.document.pdfx_version.as_deref(), Some("PDF/X-1a:2001"));

    // Something that is not a PDF/X version at all.
    let mut f = Fixture::reference();
    f.set_content(f.metadata, xmp("PDF/A-1b"));
    f.set_info("GTS_PDFXVersion", "PDF/A-1b");
    assert_only(&f.check(), c::PDFX_VERSION_MISMATCH, Severity::Error);
}

#[test]
fn no_pdfx_claim() {
    let mut f = Fixture::reference();
    f.dict(f.info).remove(b"GTS_PDFXVersion");
    f.dict(f.catalog).remove(b"Metadata");
    let report = f.check();
    assert_only(&report, c::PDFX_NOT_CLAIMED, Severity::Warning);
    assert_eq!(report.document.pdfx_version, None);
    let report = f.check_with(&Expectations { require_output_intent: false, ..card_expectations() });
    assert_only(&report, c::PDFX_NOT_CLAIMED, Severity::Info);

    // The XMP marker alone is enough (that is where PDF/X-4 puts it).
    let mut f = Fixture::reference();
    f.dict(f.info).remove(b"GTS_PDFXVersion");
    assert!(f.check().findings.is_empty());
    // And so is the Info key alone.
    let mut f = Fixture::reference();
    f.dict(f.catalog).remove(b"Metadata");
    assert!(f.check().findings.is_empty());
}

#[test]
fn title_trapped_and_file_id() {
    let mut f = Fixture::reference();
    f.dict(f.info).remove(b"Title");
    assert_only(&f.check(), c::NO_TITLE, Severity::Error);
    f.set_info("Title", "   ");
    assert_only(&f.check(), c::NO_TITLE, Severity::Error);

    let mut f = Fixture::reference();
    f.dict(f.info).remove(b"Trapped");
    assert_only(&f.check(), c::TRAPPED_NOT_SET, Severity::Error);
    f.dict(f.info).set("Trapped", "Unknown");
    let report = f.check();
    assert_only(&report, c::TRAPPED_NOT_SET, Severity::Error);
    assert!(finding(&report, c::TRAPPED_NOT_SET).message.contains("Unknown"));
    f.dict(f.info).set("Trapped", "True");
    assert!(f.check().findings.is_empty());

    let mut f = Fixture::reference();
    f.doc.trailer.remove(b"ID");
    assert_only(&f.check(), c::NO_FILE_ID, Severity::Warning);

    // Without a PDF/X claim, a missing title is only a warning.
    let mut f = Fixture::reference();
    f.dict(f.info).remove(b"GTS_PDFXVersion");
    f.dict(f.catalog).remove(b"Metadata");
    f.dict(f.info).remove(b"Title");
    let report = f.check_with(&Expectations { require_output_intent: false, ..card_expectations() });
    assert_eq!(codes(&report), set(&[c::PDFX_NOT_CLAIMED, c::NO_TITLE]));
    assert!(report.passed());

    // No Info dictionary at all.
    let mut f = Fixture::reference();
    f.doc.trailer.remove(b"Info");
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::NO_TITLE, c::TRAPPED_NOT_SET, c::EBC_SPINE_MISSING]), "{:#?}", report.findings);
}

// ---- card geometry -------------------------------------------------------------------------

#[test]
fn wrong_spine_key() {
    // A 12 mm spine, consistently declared, where 10 mm is expected.
    let mut f = Fixture::reference();
    f.set_info("EBC.SpineWidthMm", "12");
    f.set_info("EBC.CoverWidthMm", "109");
    f.set_info("EBC.FoldXMm", "109,121");
    let report = f.check();
    assert_only(&report, c::EBC_SPINE_MISMATCH, Severity::Error);
    assert!(finding(&report, c::EBC_SPINE_MISMATCH).message.contains("12.00 mm, expected 10.00 mm"));
    // Without an expected spine, the consistent geometry is accepted.
    let report = f.check_with(&Expectations { spine_mm: None, ..card_expectations() });
    assert!(report.findings.is_empty());
}

#[test]
fn spine_key_contradicting_the_page_geometry() {
    let mut f = Fixture::reference();
    f.set_info("EBC.SpineWidthMm", "12");
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::EBC_GEOMETRY_MISMATCH, c::EBC_SPINE_MISMATCH]), "{:#?}", report.findings);
    // 2 × 110 + 12 ≠ 230, and the folds are 10 mm apart, not 12.
    assert_eq!(report.findings.iter().filter(|f| f.code == c::EBC_GEOMETRY_MISMATCH).count(), 2);
}

#[test]
fn spine_key_missing_malformed_or_numeric() {
    let mut f = Fixture::reference();
    f.dict(f.info).remove(b"EBC.SpineWidthMm");
    f.dict(f.info).remove(b"EBC.FoldXMm");
    assert_only(&f.check(), c::EBC_SPINE_MISSING, Severity::Error);

    let mut f = Fixture::reference();
    f.set_info("EBC.SpineWidthMm", "dix");
    assert_only(&f.check(), c::EBC_KEY_INVALID, Severity::Error);

    // Numbers written as PDF numbers rather than strings are read too.
    let mut f = Fixture::reference();
    f.dict(f.info).set("EBC.SpineWidthMm", 10);
    f.dict(f.info).set("EBC.CoverHeightMm", Object::Real(178.0));
    let report = f.check();
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert_eq!(report.document.ebc["EBC.SpineWidthMm"], "10");

    // The card height key must match the trim box.
    let mut f = Fixture::reference();
    f.set_info("EBC.CoverHeightMm", "200");
    assert_only(&f.check(), c::EBC_GEOMETRY_MISMATCH, Severity::Error);
}

#[test]
fn qr_modules_too_small() {
    let mut f = Fixture::reference();
    let content = String::from_utf8(f.content(f.content2)).unwrap();
    let original = common::qr_content(150.0, 80.0, common::QR_MODULE);
    assert!(content.contains(&original));
    f.set_content(f.content2, content.replace(&original, &common::qr_content(150.0, 80.0, 0.3)));
    let report = f.check();
    assert_only(&report, c::QR_MODULE_TOO_SMALL, Severity::Error);
    let found = finding(&report, c::QR_MODULE_TOO_SMALL);
    assert_eq!(found.page, Some(2));
    assert!(found.message.contains("25×25 modules"), "{}", found.message);
    assert!((report.pages[1].qr_candidates[0].module_mm - 0.3).abs() < 0.002);

    // No recognisable QR code at all: informational, since the check could not be made.
    let mut f = Fixture::reference();
    let content = String::from_utf8(f.content(f.content2)).unwrap();
    f.set_content(f.content2, content.replace(&original, ""));
    assert_only(&f.check(), c::QR_NOT_FOUND, Severity::Info);

    // The declared module size is checked as well.
    let mut f = Fixture::reference();
    f.set_info("EBC.QrModuleMm", "0.25");
    assert_only(&f.check(), c::QR_MODULE_TOO_SMALL, Severity::Error);
}

#[test]
fn qr_drawn_one_module_at_a_time_or_as_closed_paths_is_still_recognised() {
    let matrix = common::qr_matrix();
    let size = matrix.len();
    let module = 0.45 * common::MM;
    let (x0, y0) = (150.0 * common::MM, 80.0 * common::MM);
    // One `re f` per dark module.
    let mut per_module = String::from("q 0 0 0 1 k\n");
    // Each module as an explicit closed path, all filled at once.
    let mut closed_paths = String::from("q 0 0 0 1 k\n");
    for (row, cells) in matrix.iter().enumerate() {
        for (col, dark) in cells.iter().enumerate() {
            if !dark {
                continue;
            }
            let x = x0 + col as f64 * module;
            let y = y0 + (size - 1 - row) as f64 * module;
            per_module.push_str(&format!("{x:.4} {y:.4} {module:.4} {module:.4} re f\n"));
            closed_paths.push_str(&format!(
                "{x:.4} {y:.4} m {:.4} {y:.4} l {:.4} {:.4} l {x:.4} {:.4} l h\n",
                x + module,
                x + module,
                y + module,
                y + module
            ));
        }
    }
    per_module.push_str("Q\n");
    closed_paths.push_str("f Q\n");
    for drawing in [per_module, closed_paths] {
        let mut f = Fixture::reference();
        let content = String::from_utf8(f.content(f.content2)).unwrap();
        let original = common::qr_content(150.0, 80.0, common::QR_MODULE);
        f.set_content(f.content2, content.replace(&original, &drawing));
        let report = f.check();
        assert!(report.findings.is_empty(), "{:#?}", report.findings);
        let candidates = &report.pages[1].qr_candidates;
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].modules, 25);
        assert!((candidates[0].module_mm - 0.45).abs() < 0.002, "{candidates:?}");
    }
}
