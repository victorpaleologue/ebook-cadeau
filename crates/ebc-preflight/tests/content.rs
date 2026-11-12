//! The content stream interpreter: transforms, form XObjects, inline images, colour state,
//! resources and syntax errors.

#![allow(clippy::unwrap_used)]

mod common;

use common::{Fixture, card_expectations, codes, name, set};
use ebc_preflight::{Expectations, ImageSummary, Report, Severity, codes as c};
use lopdf::{Object, ObjectId, Stream, dictionary};

/// Adds a 300 × 600 px grey image as `/Im` to page 1 and replaces the page content.
fn page_with(content: &str) -> Fixture {
    let mut f = Fixture::reference();
    let image = grey_image(&mut f, 300, 600);
    f.add_resource(f.page1, "XObject", "Im", image);
    f.set_content(f.content1, content.as_bytes().to_vec());
    f
}

fn grey_image(f: &mut Fixture, width: i64, height: i64) -> ObjectId {
    f.doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => width, "Height" => height,
            "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8, "Filter" => "DCTDecode",
        },
        vec![0xFF, 0xD8, 0xFF, 0xD9],
    ))
}

/// Expectations that do not complain about resolution, so tests can look at the numbers.
fn lenient_ppi() -> Expectations {
    Expectations { min_ppi: 0.0, ..card_expectations() }
}

fn images(report: &Report) -> &[ImageSummary] {
    &report.pages[0].images
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 0.06
}

const INCH: f64 = 25.4;

#[test]
fn nested_transforms_accumulate_and_q_restores_them() {
    let mut f = page_with(
        "q 2 0 0 2 0 0 cm\n\
           q 1 0 0 1 10 10 cm\n\
             q 50 0 0 100 0 0 cm /Im Do Q\n\
           Q\n\
           q 36 0 0 36 0 0 cm /Im Do Q\n\
         Q\n\
         q 72 0 0 72 0 0 cm /Im Do Q\n\
         72 0 0 144 0 0 cm /Im Do\n",
    );
    let report = f.check_with(&lenient_ppi());
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    let got: Vec<(f64, f64, f64, f64)> =
        images(&report).iter().map(|i| (i.width_mm, i.height_mm, i.ppi_x, i.ppi_y)).collect();
    // 1: 50×100 units under two nested scales of 2 → 100×200 pt.
    // 2: the inner translate and scale were popped: 36 units × 2 → 72 pt = 1 in.
    // 3: everything popped: 72 pt = 1 in.
    // 4: outside any q: 1 in × 2 in.
    let expected = [
        (100.0 / 72.0 * INCH, 200.0 / 72.0 * INCH, 216.0, 216.0),
        (INCH, INCH, 300.0, 600.0),
        (INCH, INCH, 300.0, 600.0),
        (INCH, 2.0 * INCH, 300.0, 300.0),
    ];
    assert_eq!(got.len(), expected.len());
    for (g, e) in got.iter().zip(&expected) {
        assert!(
            close(g.0, e.0) && close(g.1, e.1) && close(g.2, e.2) && close(g.3, e.3),
            "{g:?} vs {e:?}"
        );
    }
}

#[test]
fn cm_premultiplies_the_ctm() {
    // scale(2,1), then a quarter turn, then the image matrix. With the correct order
    // (new CTM = M × CTM) the image is 72 pt wide and 144 pt high on the page; with the
    // wrong order it would be 144 × 72.
    let mut f = page_with("q 2 0 0 1 0 0 cm 0 1 -1 0 0 0 cm 72 0 0 72 0 0 cm /Im Do Q");
    let report = f.check_with(&lenient_ppi());
    let image = &images(&report)[0];
    assert!(close(image.width_mm, INCH) && close(image.height_mm, 2.0 * INCH), "{image:?}");
    assert!(close(image.ppi_x, 300.0) && close(image.ppi_y, 300.0), "{image:?}");
}

#[test]
fn rotation_and_shear_do_not_change_the_measured_size() {
    let (sin, cos) = (30.0_f64.to_radians().sin(), 30.0_f64.to_radians().cos());
    let content = format!(
        "q {cos:.6} {sin:.6} {:.6} {cos:.6} 200 200 cm 144 0 0 288 0 0 cm /Im Do Q\n\
         q 72 0 36 72 100 100 cm /Im Do Q\n",
        -sin
    );
    let mut f = page_with(&content);
    let report = f.check_with(&lenient_ppi());
    let rotated = &images(&report)[0];
    assert!(
        close(rotated.width_mm, 2.0 * INCH) && close(rotated.height_mm, 4.0 * INCH),
        "{rotated:?}"
    );
    assert!(close(rotated.ppi_x, 150.0) && close(rotated.ppi_y, 150.0), "{rotated:?}");
    // Sheared: the vertical edge is hypot(36, 72) pt long.
    let sheared = &images(&report)[1];
    let edge_mm = 36.0_f64.hypot(72.0) / 72.0 * INCH;
    assert!(close(sheared.width_mm, INCH) && close(sheared.height_mm, edge_mm), "{sheared:?}");
}

#[test]
fn form_xobjects_apply_their_matrix_and_their_own_resources() {
    let mut f = page_with("q 3 0 0 3 0 0 cm /Fm Do Q\nq /Fm Do Q\n");
    let inner_image = grey_image(&mut f, 300, 300);
    let form = f.doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form",
            "BBox" => vec![0.into(), 0.into(), 144.into(), 144.into()],
            "Matrix" => vec![Object::Real(0.5), 0.into(), 0.into(), Object::Real(0.5), 0.into(), 0.into()],
            // The form's /Im is a different image from the page's /Im.
            "Resources" => dictionary! { "XObject" => dictionary! { "Im" => inner_image } },
        },
        b"q 144 0 0 144 0 0 cm /Im Do Q".to_vec(),
    ));
    f.add_resource(f.page1, "XObject", "Fm", form);
    let report = f.check_with(&lenient_ppi());
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    let got = images(&report);
    assert_eq!(got.len(), 2);
    // 144 × 0.5 × 3 = 216 pt = 3 in → 100 ppi; then 144 × 0.5 = 72 pt = 1 in → 300 ppi.
    assert_eq!((got[0].width_px, got[0].height_px), (300, 300));
    assert!(close(got[0].width_mm, 3.0 * INCH) && close(got[0].ppi_x, 100.0), "{:?}", got[0]);
    assert!(close(got[1].width_mm, INCH) && close(got[1].ppi_x, 300.0), "{:?}", got[1]);

    // The same placements against a 150 ppi floor: only the enlarged one fails.
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::IMAGE_LOW_RES]));
    assert_eq!(report.findings.len(), 1);
    assert!(report.findings[0].message.contains("100 ppi"), "{}", report.findings[0].message);
}

#[test]
fn forms_without_resources_use_the_callers() {
    let mut f = page_with("q /Fm Do Q");
    let form = f.doc.add_object(Stream::new(
        dictionary! { "Type" => "XObject", "Subtype" => "Form", "BBox" => vec![0.into(), 0.into(), 72.into(), 72.into()] },
        b"q 72 0 0 144 0 0 cm /Im Do Q".to_vec(),
    ));
    f.add_resource(f.page1, "XObject", "Fm", form);
    let report = f.check_with(&lenient_ppi());
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert_eq!((images(&report)[0].width_px, images(&report)[0].height_px), (300, 600));
}

#[test]
fn forms_inherit_and_do_not_leak_graphics_state() {
    // The form paints with the caller's fill colour (RGB), then sets a CMYK colour and a
    // scale that must not survive the Do.
    let mut f = page_with("1 0 0 rg /Fm Do 72 0 0 72 0 0 cm /Im Do 0 0 10 10 re f");
    let form = f.doc.add_object(Stream::new(
        dictionary! { "Type" => "XObject", "Subtype" => "Form", "BBox" => vec![0.into(), 0.into(), 72.into(), 72.into()] },
        b"0 0 20 20 re f 0 0 0 1 k 10 0 0 10 0 0 cm q q q".to_vec(),
    ));
    f.add_resource(f.page1, "XObject", "Fm", form);
    let report = f.check_with(&lenient_ppi());
    assert_eq!(
        codes(&report),
        set(&[c::COLOR_RGB, c::CONTENT_UNBALANCED_Q]),
        "{:#?}",
        report.findings
    );
    let unbalanced = report.findings.iter().find(|f| f.code == c::CONTENT_UNBALANCED_Q).unwrap();
    assert!(unbalanced.message.contains("form XObject /Fm leaves 3"), "{}", unbalanced.message);
    // The image after the form is not scaled by the form's cm.
    assert!(close(images(&report)[0].width_mm, INCH), "{:?}", images(&report)[0]);
    // Only the RGB fills (in the form and after it) paint; the form's K fill colour is never used.
    assert_eq!(
        report.pages[0].color_spaces,
        vec!["DeviceGray".to_string(), "DeviceRGB".to_string()]
    );
}

#[test]
fn stray_q_inside_a_form_cannot_pop_the_callers_state() {
    let mut f = page_with("q 72 0 0 72 0 0 cm /Fm Do /Im Do Q");
    let form = f.doc.add_object(Stream::new(
        dictionary! { "Type" => "XObject", "Subtype" => "Form", "BBox" => vec![0.into(), 0.into(), 1.into(), 1.into()] },
        b"Q Q Q Q".to_vec(),
    ));
    f.add_resource(f.page1, "XObject", "Fm", form);
    let report = f.check_with(&lenient_ppi());
    assert_eq!(codes(&report), set(&[c::CONTENT_UNBALANCED_Q]));
    assert!(report.findings[0].message.contains("never saved"));
    assert!(close(images(&report)[0].width_mm, INCH), "the caller's CTM survived");
}

#[test]
fn the_same_image_placed_twice_is_reported_twice() {
    let mut f = page_with("q 72 0 0 144 0 0 cm /Im Do Q q 720 0 0 1440 0 0 cm /Im Do Q");
    let report = f.check();
    assert_eq!(images(&report).len(), 2);
    assert_eq!(report.findings.len(), 1, "only the enlarged placement is too coarse");
    assert_eq!(report.findings[0].code, c::IMAGE_LOW_RES);
    assert!(report.findings[0].message.contains("30 ppi"));
}

#[test]
fn user_unit_scales_image_placements() {
    let mut f = page_with("q 72 0 0 144 0 0 cm /Im Do Q");
    f.dict(f.page1).set("UserUnit", 2);
    let report = f.check_with(&lenient_ppi());
    let image = &images(&report)[0];
    assert!(close(image.width_mm, 2.0 * INCH) && close(image.ppi_x, 150.0), "{image:?}");
}

#[test]
fn single_pixel_rows_have_no_resolution_across_them() {
    // A 1×1 swatch and a 256×1 gradient strip, both stretched over 10 cm.
    let mut f =
        page_with("q 283.46 0 0 283.46 0 0 cm /Dot Do Q q 283.46 0 0 283.46 0 0 cm /Strip Do Q");
    let dot = grey_image(&mut f, 1, 1);
    let strip = grey_image(&mut f, 256, 1);
    f.add_resource(f.page1, "XObject", "Dot", dot);
    f.add_resource(f.page1, "XObject", "Strip", strip);
    let report = f.check();
    // The strip has 256 px over 100 mm = 65 ppi along its length: that axis does count.
    assert_eq!(codes(&report), set(&[c::IMAGE_LOW_RES]));
    assert_eq!(report.findings.len(), 1);
    assert!(report.findings[0].message.contains("/Strip"));
}

#[test]
fn degenerate_and_malformed_images() {
    let mut f = page_with("q 0 0 0 0 0 0 cm /Im Do Q q 72 0 0 72 0 0 cm /Bad Do /Huge Do Q");
    let bad = f.doc.add_object(Stream::new(
        dictionary! { "Subtype" => "Image", "Width" => -5, "Height" => name("tall"), "ColorSpace" => "DeviceGray" },
        Vec::new(),
    ));
    let huge = f.doc.add_object(Stream::new(
        dictionary! { "Subtype" => "Image", "Width" => 99_999_999_999_i64, "Height" => 1, "ColorSpace" => "DeviceGray" },
        Vec::new(),
    ));
    f.add_resource(f.page1, "XObject", "Bad", bad);
    f.add_resource(f.page1, "XObject", "Huge", huge);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::IMAGE_INVALID]), "{:#?}", report.findings);
    assert_eq!(report.findings.len(), 3);
    assert!(report.findings.iter().all(|f| f.severity == Severity::Warning));
    assert!(images(&report).is_empty());
}

#[test]
fn inline_images_are_measured_like_any_other() {
    let pixels = "\u{7f}".repeat(30 * 30);
    let content = format!(
        "q 72 0 0 72 100 100 cm BI /W 30 /H 30 /BPC 8 /CS /G ID {pixels} EI Q 0 0 0 1 k 0 0 5 5 re f"
    );
    let mut f = page_with(&content);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::IMAGE_LOW_RES]), "{:#?}", report.findings);
    let image = &images(&report)[0];
    assert!(image.inline);
    assert_eq!(image.name, "(inline)");
    assert_eq!((image.width_px, image.height_px, image.bits_per_component), (30, 30, 8));
    assert_eq!(image.color_space, "DeviceGray");
    assert!(close(image.ppi_x, 30.0), "{image:?}");
    // The operators after EI are still interpreted.
    assert_eq!(
        report.pages[0].color_spaces,
        vec!["DeviceCMYK".to_string(), "DeviceGray".to_string()]
    );

    // An RGB inline image, with the abbreviated names.
    let rgb = "\u{20}".repeat(4 * 4 * 3);
    let mut f = page_with(&format!("q 1 0 0 1 0 0 cm BI /W 4 /H 4 /BPC 8 /CS /RGB ID {rgb} EI Q"));
    let report = f.check_with(&lenient_ppi());
    assert_eq!(codes(&report), set(&[c::COLOR_RGB]));
    assert!(report.findings[0].message.contains("an inline image uses DeviceRGB"));
}

#[test]
fn image_masks_paint_with_the_fill_colour() {
    let mut f = page_with("1 0 0 rg q 72 0 0 72 0 0 cm /Stencil Do Q");
    let stencil = f.doc.add_object(Stream::new(
        dictionary! { "Subtype" => "Image", "Width" => 600, "Height" => 600, "ImageMask" => true, "BitsPerComponent" => 1 },
        vec![0; 45_000],
    ));
    f.add_resource(f.page1, "XObject", "Stencil", stencil);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::COLOR_RGB]), "{:#?}", report.findings);
    let image = &images(&report)[0];
    assert_eq!(image.color_space, "ImageMask");
    assert_eq!(image.bits_per_component, 1);
    assert!(close(image.ppi_x, 600.0));
}

#[test]
fn colour_spaces_from_resources() {
    let mut f = page_with(
        "/Cmyk cs 0.1 0.2 0.3 0.4 scn 0 0 10 10 re f\n\
         /Cal cs 1 0 0 scn 0 0 10 10 re f\n\
         /Lab CS 50 0 0 SCN 0 0 m 10 10 l S\n\
         /Process cs 0.5 0.5 scn 0 0 10 10 re f\n\
         /Nope cs 0.5 scn 0 0 10 10 re f\n",
    );
    let cmyk_profile = f
        .doc
        .add_object(Stream::new(dictionary! { "N" => 4 }, common::icc_profile(b"prtr", b"CMYK")));
    let white = vec![Object::Real(0.9505), 1.into(), Object::Real(1.089)];
    f.add_resource(f.page1, "ColorSpace", "Cmyk", vec![name("ICCBased"), cmyk_profile.into()]);
    f.add_resource(
        f.page1,
        "ColorSpace",
        "Cal",
        vec![name("CalRGB"), Object::Dictionary(dictionary! { "WhitePoint" => white.clone() })],
    );
    f.add_resource(
        f.page1,
        "ColorSpace",
        "Lab",
        vec![name("Lab"), Object::Dictionary(dictionary! { "WhitePoint" => white })],
    );
    f.add_resource(
        f.page1,
        "ColorSpace",
        "Process",
        vec![
            name("DeviceN"),
            Object::Array(vec![name("Cyan"), name("Black")]),
            name("DeviceCMYK"),
            Object::Null,
        ],
    );
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::COLOR_RGB, c::COLOR_UNKNOWN]), "{:#?}", report.findings);
    let rgb: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.code == c::COLOR_RGB)
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(rgb, vec!["a fill uses CalRGB", "a stroke uses Lab"]);
    assert_eq!(
        report.pages[0].color_spaces,
        vec!["CalRGB", "DeviceN(Cyan,Black)", "ICCBased(4)", "Lab", "Unknown(Nope)"]
    );
    // ICC-based CMYK counts towards the ink maximum: 10+20+30+40 = 100 %.
    assert_eq!(report.pages[0].max_total_ink_percent, Some(100.0));
}

#[test]
fn uncoloured_tiling_patterns_use_the_underlying_colour_space() {
    let mut f = page_with("/PCS cs 1 0 0 /Stripes scn 0 0 100 100 re f");
    let cell = f.doc.add_object(Stream::new(
        dictionary! {
            "PatternType" => 1, "PaintType" => 2, "TilingType" => 1,
            "BBox" => vec![0.into(), 0.into(), 4.into(), 4.into()], "XStep" => 4, "YStep" => 4,
            "Resources" => dictionary! {},
        },
        b"0 0 2 4 re f".to_vec(),
    ));
    f.add_resource(f.page1, "Pattern", "Stripes", cell);
    f.add_resource(f.page1, "ColorSpace", "PCS", vec![name("Pattern"), name("DeviceRGB")]);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::COLOR_RGB]), "{:#?}", report.findings);
    assert!(report.findings[0].message.contains("uncoloured pattern /Stripes uses DeviceRGB"));
}

#[test]
fn missing_resources_are_reported() {
    let mut f = page_with(
        "/NoImage Do /NoState gs /NoShade sh BT /NoFont 10 Tf (x) Tj ET /Pattern cs /NoPattern scn 0 0 5 5 re f",
    );
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::RESOURCE_MISSING]), "{:#?}", report.findings);
    assert_eq!(report.findings.len(), 5);
    assert!(report.passed(), "missing resources are warnings");
    let messages: Vec<&str> = report.findings.iter().map(|f| f.message.as_str()).collect();
    assert!(
        messages.contains(&"XObject resource /NoImage is used but not defined"),
        "{messages:?}"
    );
    assert!(messages.contains(&"Font resource /NoFont is used but not defined"), "{messages:?}");
}

#[test]
fn postscript_xobjects_are_errors() {
    let mut f = page_with("/Ps Do");
    let ps = f.doc.add_object(Stream::new(
        dictionary! { "Type" => "XObject", "Subtype" => "PS" },
        b"showpage".to_vec(),
    ));
    f.add_resource(f.page1, "XObject", "Ps", ps);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::POSTSCRIPT_XOBJECT]));
    assert!(!report.passed());
}

#[test]
fn unbalanced_graphics_state_operators() {
    let mut f = page_with("q q 0 0 0 1 k 0 0 10 10 re f Q");
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::CONTENT_UNBALANCED_Q]));
    assert_eq!(report.findings[0].severity, Severity::Warning);
    assert!(
        report.findings[0].message.contains("leaves 1 saved"),
        "{}",
        report.findings[0].message
    );

    let mut f = page_with("q 0 0 0 1 k 0 0 10 10 re f Q Q");
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::CONTENT_UNBALANCED_Q]));
    assert!(report.findings[0].message.contains("never saved"));

    // Thousands of unclosed saves: counted, not stored.
    let mut f = page_with(&"q ".repeat(100_000));
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::CONTENT_UNBALANCED_Q]));
    assert!(
        report.findings[0].message.contains("leaves 100000 saved"),
        "{}",
        report.findings[0].message
    );

    // Deep but balanced nesting (beyond the 256 states that are stored) restores the outer
    // state correctly: 1.01^300 ≈ 20 would show in the image size otherwise.
    let mut content = "q 1.01 0 0 1.01 0 0 cm ".repeat(300);
    content.push_str(&"Q ".repeat(300));
    content.push_str("72 0 0 144 0 0 cm /Im Do");
    let mut f = page_with(&content);
    let report = f.check_with(&lenient_ppi());
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    assert!(close(images(&report)[0].width_mm, INCH), "{:?}", images(&report)[0]);
}

#[test]
fn syntax_errors_stop_the_page_with_an_error_but_keep_what_was_seen() {
    // The open `q` is not reported as unbalanced: the stream was not read to its end.
    let content = "q 1 0 0 rg 0 0 10 10 re f (unterminated string 0 0 1 rg 20 20 5 5 re f";
    let mut f = page_with(content);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::COLOR_RGB, c::CONTENT_INVALID]), "{:#?}", report.findings);
    let invalid = report.findings.iter().find(|f| f.code == c::CONTENT_INVALID).unwrap();
    assert_eq!(invalid.severity, Severity::Error);
    assert!(invalid.message.contains("unterminated string"), "{}", invalid.message);
    let offset = content.find('(').unwrap();
    assert!(invalid.message.contains(&format!("at byte {offset}:")), "{}", invalid.message);

    for (content, reason) in [
        ("0 0 10 10 re f ]", "unexpected delimiter"),
        ("[1 2 3 TJ", "unterminated array or dictionary"),
        ("0 0 m 10 10", "operands without an operator"),
        ("BI /W 1 /H 1 /BPC 8 /CS /G ID x", "inline image without EI"),
    ] {
        let mut f = page_with(content);
        let report = f.check();
        assert_eq!(codes(&report), set(&[c::CONTENT_INVALID]), "{content}: {:#?}", report.findings);
        assert!(report.findings[0].message.contains(reason), "{}", report.findings[0].message);
    }
}

#[test]
fn operators_with_missing_operands_are_errors() {
    for content in [
        "1 0 0 cm",
        "0.5 0.5 rg",
        "(text) k",
        "/Im cm",
        "10 re",
        "gs",
        "Do",
        "sc",
        "9 Tr",
        "1 2 3 4 5 m l",
    ] {
        let mut f = page_with(content);
        let report = f.check();
        assert_eq!(codes(&report), set(&[c::CONTENT_INVALID]), "{content}: {:#?}", report.findings);
        assert!(report.findings[0].message.contains("missing or invalid operands"), "{content}");
    }
    // Absurd matrices are rejected instead of poisoning later arithmetic.
    let mut f = page_with(
        "99999999999999999999999999999999 0 0 99999999999999999999999999999999 0 0 cm \
         99999999999999999999999999999999 0 0 1 0 0 cm 72 0 0 72 0 0 cm /Im Do",
    );
    let report = f.check_with(&lenient_ppi());
    assert!(report.has(c::CONTENT_INVALID), "{:#?}", report.findings);
    let json = report.to_json();
    assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    assert!(!json.contains("null,\"ppi") && !json.contains("NaN") && !json.contains("inf"));
}

#[test]
fn content_split_across_several_streams_is_one_stream() {
    let mut f = page_with("");
    let part1 = f.doc.add_object(Stream::new(dictionary! {}, b"q 1 0 0".to_vec()));
    let mut compressed = Stream::new(dictionary! {}, b"rg 0 0 10 10 re".to_vec());
    compressed.compress().unwrap();
    let part2 = f.doc.add_object(compressed);
    let part3 = f.doc.add_object(Stream::new(dictionary! {}, b"f Q".to_vec()));
    f.dict(f.page1).set("Contents", vec![part1.into(), part2.into(), part3.into()]);
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::COLOR_RGB]), "{:#?}", report.findings);

    // Non-stream entries and wrong types are reported.
    f.dict(f.page1).set("Contents", vec![part1.into(), Object::Integer(4), part3.into()]);
    let report = f.check();
    assert!(report.has(c::CONTENT_INVALID));
    f.dict(f.page1).set("Contents", 12);
    assert_eq!(codes(&f.check()), set(&[c::CONTENT_INVALID]));
    // A page without content is simply empty.
    f.dict(f.page1).remove(b"Contents");
    assert!(f.check().findings.is_empty());
}

#[test]
fn compressed_content_is_decoded_and_corruption_is_an_error() {
    let mut f = page_with("");
    let text = b"q 1 0 0 rg 0 0 10 10 re f Q\n".repeat(50);
    let mut stream = Stream::new(dictionary! {}, text);
    stream.compress().unwrap();
    assert!(stream.dict.has(b"Filter"), "the stream should have been deflated");
    let good = stream.content.clone();
    let id = f.doc.add_object(stream);
    f.dict(f.page1).set("Contents", id);
    assert_eq!(codes(&f.check()), set(&[c::COLOR_RGB]));

    // Truncated in the middle: lenient readers would see half a page and stay silent.
    let truncated = good[..good.len() / 2].to_vec();
    if let Object::Stream(stream) = f.doc.objects.get_mut(&id).unwrap() {
        stream.set_content(truncated);
    }
    let report = f.check();
    assert_eq!(codes(&report), set(&[c::CONTENT_INVALID]), "{:#?}", report.findings);
    assert!(report.findings[0].message.contains("FlateDecode"), "{}", report.findings[0].message);

    // An unsupported filter on a content stream.
    if let Object::Stream(stream) = f.doc.objects.get_mut(&id).unwrap() {
        stream.dict.set("Filter", "JBIG2Decode");
    }
    assert_eq!(codes(&f.check()), set(&[c::CONTENT_INVALID]));
}

#[test]
fn image_placement_list_is_capped() {
    let mut f = page_with(&"q 72 0 0 144 0 0 cm /Im Do Q\n".repeat(2_500));
    let report = f.check();
    assert_eq!(images(&report).len(), 2_000);
    assert_eq!(codes(&report), set(&[c::LIMIT_EXCEEDED]));
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].severity, Severity::Warning);
    assert!(report.passed());
}
