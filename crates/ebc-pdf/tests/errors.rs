//! Every constraint the writer enforces, and hostile input it must survive.

mod common;

use std::sync::Arc;

use common::*;
use ebc_pdf::{EmbeddingRestriction, PdfError, WriteOptions, write_pdf};
use ebc_scene::{
    Color, Document, FontId, Glyph, ImageData, ImageId, Item, OutputIntent, Page, Path, PathSeg,
    PdfStandard, PixelFormat, Point, Rect, Size, Stroke, Transform,
};

const X_LEVELS: [PdfStandard; 3] = [PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4];
const ALL: [PdfStandard; 4] =
    [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4];

/// A valid one-page document holding `items`.
fn doc_with(standard: PdfStandard, items: Vec<Item>) -> Document {
    let mut doc = base_doc(standard);
    let mut page = Page::new(Size::new(100.0, 80.0), 3.0, 0.0);
    page.items = items;
    doc.pages.push(page);
    doc
}

fn write(doc: &Document) -> Result<Vec<u8>, PdfError> {
    write_pdf(doc, &WriteOptions::default())
}

fn error(doc: &Document) -> PdfError {
    write(doc).expect_err("the document must be refused")
}

fn raw_image(format: PixelFormat, pixels: Vec<u8>) -> ImageData {
    ImageData::Raw { pixels: Arc::from(pixels), width_px: 1, height_px: 1, format }
}

fn image_item(image: ImageId) -> Item {
    Item::Image { image, rect: Rect::new(0.0, 0.0, 10.0, 10.0) }
}

// ----- document level ----------------------------------------------------------------

#[test]
fn a_document_needs_pages() {
    for standard in ALL {
        assert_eq!(error(&base_doc(standard)), PdfError::NoPages);
    }
    let mut huge = base_doc(PdfStandard::Plain);
    huge.pages = vec![Page::new(Size::new(10.0, 10.0), 0.0, 0.0); 8_001];
    assert_eq!(error(&huge), PdfError::TooLarge { what: "the number of pages" });
}

#[test]
fn pdfx_requires_an_output_intent() {
    for standard in X_LEVELS {
        let mut doc = doc_with(standard, vec![]);
        doc.output_intent = None;
        assert_eq!(error(&doc), PdfError::MissingOutputIntent { standard });
    }
    // Plain PDF does not.
    let mut doc = doc_with(PdfStandard::Plain, vec![]);
    doc.output_intent = None;
    assert!(write(&doc).is_ok());
}

#[test]
fn pdfx4_requires_the_profile_and_older_parts_a_profile_or_a_registry() {
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    doc.output_intent.as_mut().expect("intent").icc_profile = None;
    assert_eq!(error(&doc), PdfError::MissingOutputProfile { standard: PdfStandard::X4 });

    for standard in [PdfStandard::X1a2001, PdfStandard::X3_2002] {
        let mut doc = doc_with(standard, vec![]);
        let intent = doc.output_intent.as_mut().expect("intent");
        intent.icc_profile = None;
        assert!(write(&doc).is_ok(), "{standard:?}: a registered condition is enough");
        doc.output_intent.as_mut().expect("intent").registry_name = "  ".into();
        assert_eq!(error(&doc), PdfError::MissingOutputProfile { standard });
    }
}

#[test]
fn output_intent_must_be_usable() {
    for standard in ALL {
        let mut doc = doc_with(standard, vec![]);
        doc.output_intent.as_mut().expect("intent").condition_identifier = " ".into();
        assert!(matches!(error(&doc), PdfError::UnsupportedOutputIntent { .. }), "{standard:?}");
    }
    // PDF/X here means CMYK printing conditions.
    for standard in X_LEVELS {
        let mut doc = doc_with(standard, vec![]);
        doc.output_intent = Some(OutputIntent {
            condition_identifier: "sRGB".into(),
            info: "sRGB IEC61966-2.1".into(),
            registry_name: "http://www.color.org".into(),
            icc_profile: Some(srgb_v2_profile()),
            components: 3,
        });
        assert!(matches!(error(&doc), PdfError::UnsupportedOutputIntent { .. }), "{standard:?}");
    }
    // A condition that is in no registry must at least be described.
    for standard in X_LEVELS {
        let mut doc = doc_with(standard, vec![]);
        let intent = doc.output_intent.as_mut().expect("intent");
        intent.registry_name = String::new();
        intent.info = " ".into();
        assert!(matches!(error(&doc), PdfError::UnsupportedOutputIntent { .. }), "{standard:?}");
        doc.output_intent.as_mut().expect("intent").info = "House profile for press 3".into();
        assert!(write(&doc).is_ok(), "{standard:?}");
    }
    // Plain PDF may carry an RGB output intent.
    let mut doc = doc_with(PdfStandard::Plain, vec![]);
    doc.output_intent.as_mut().expect("intent").icc_profile = Some(srgb_v4_profile());
    doc.output_intent.as_mut().expect("intent").components = 3;
    assert!(write(&doc).is_ok());
}

#[test]
fn output_profile_must_be_a_matching_icc_profile() {
    let is_icc_error = |doc: &Document| matches!(error(doc), PdfError::InvalidIccProfile { context, .. } if context == "the output intent");
    for standard in ALL {
        // Not a profile at all.
        for garbage in [vec![], b"not an ICC profile".to_vec(), vec![0u8; 4096]] {
            let mut doc = doc_with(standard, vec![]);
            doc.output_intent.as_mut().expect("intent").icc_profile = Some(Arc::from(garbage));
            assert!(is_icc_error(&doc), "{standard:?}");
        }
        // Truncated profile.
        let mut doc = doc_with(standard, vec![]);
        let truncated = cmyk_profile()[..1000].to_vec();
        doc.output_intent.as_mut().expect("intent").icc_profile = Some(Arc::from(truncated));
        assert!(is_icc_error(&doc), "{standard:?}");
        // An RGB profile declared as CMYK.
        let mut doc = doc_with(standard, vec![]);
        doc.output_intent.as_mut().expect("intent").icc_profile = Some(srgb_v2_profile());
        assert!(is_icc_error(&doc), "{standard:?}");
    }
    // PDF 1.3 predates ICC version 4.
    let mut v4_cmyk = cmyk_profile().to_vec();
    v4_cmyk[8] = 4;
    for (standard, accepted) in [
        (PdfStandard::X1a2001, false),
        (PdfStandard::X3_2002, false),
        (PdfStandard::X4, true),
        (PdfStandard::Plain, true),
    ] {
        let mut doc = doc_with(standard, vec![]);
        doc.output_intent.as_mut().expect("intent").icc_profile = Some(Arc::from(v4_cmyk.clone()));
        assert_eq!(write(&doc).is_ok(), accepted, "{standard:?}");
        if !accepted {
            assert!(is_icc_error(&doc));
        }
    }
}

#[test]
fn pdfx_requires_a_title_and_a_creation_date() {
    for standard in X_LEVELS {
        for title in ["", "   ", "\u{0}\u{1}"] {
            let mut doc = doc_with(standard, vec![]);
            doc.info.title = title.into();
            assert_eq!(error(&doc), PdfError::MissingTitle { standard }, "{title:?}");
        }
        let mut doc = doc_with(standard, vec![]);
        doc.info.created_unix = None;
        assert_eq!(error(&doc), PdfError::MissingCreationDate { standard });
    }
    let mut doc = doc_with(PdfStandard::Plain, vec![]);
    doc.info.title = String::new();
    doc.info.created_unix = None;
    assert!(write(&doc).is_ok());
}

#[test]
fn creation_date_must_be_representable() {
    for standard in ALL {
        for seconds in [i64::MIN, -62_135_596_801, 253_402_300_800, i64::MAX] {
            let mut doc = doc_with(standard, vec![]);
            doc.info.created_unix = Some(seconds);
            assert_eq!(error(&doc), PdfError::InvalidDate(seconds));
        }
        for seconds in [-62_135_596_800, -1, 0, 253_402_300_799] {
            let mut doc = doc_with(standard, vec![]);
            doc.info.created_unix = Some(seconds);
            assert!(write(&doc).is_ok(), "{seconds}");
        }
    }
}

#[test]
fn custom_info_keys_are_validated() {
    let bad = [
        "",
        "has space",
        "tab\tkey",
        "naïve",
        "slash/key",
        "paren(key",
        "hash#key",
        "Title",
        "Producer",
        "CreationDate",
        "Trapped",
        "GTS_PDFXVersion",
        "GTS_PDFXConformance",
    ];
    for key in bad {
        let mut doc = doc_with(PdfStandard::X4, vec![]);
        doc.info.custom.push((key.into(), "value".into()));
        assert!(
            matches!(error(&doc), PdfError::InvalidCustomKey { key: k, .. } if k == key),
            "{key:?}"
        );
    }
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    doc.info.custom.push(("x".repeat(128), "value".into()));
    assert!(matches!(error(&doc), PdfError::InvalidCustomKey { .. }));

    let mut doc = doc_with(PdfStandard::X4, vec![]);
    doc.info.custom.push(("EBC.A".into(), "1".into()));
    doc.info.custom.push(("EBC.A".into(), "2".into()));
    assert!(
        matches!(error(&doc), PdfError::InvalidCustomKey { reason, .. } if reason.contains("more than once"))
    );

    let mut doc = doc_with(PdfStandard::X4, vec![]);
    doc.info.custom.push(("EBC.SpineWidthMm".into(), "10".into()));
    doc.info.custom.push(("ebc:fold-x_1".into(), String::new()));
    assert!(write(&doc).is_ok());
}

#[test]
fn page_geometry_must_be_sane() {
    let cases: [(Size, f64, f64); 12] = [
        (Size::new(0.0, 100.0), 0.0, 0.0),
        (Size::new(100.0, 0.0), 0.0, 0.0),
        (Size::new(-5.0, 100.0), 0.0, 0.0),
        (Size::new(f64::NAN, 100.0), 0.0, 0.0),
        (Size::new(100.0, f64::INFINITY), 0.0, 0.0),
        (Size::new(100.0, 100.0), -1.0, 0.0),
        (Size::new(100.0, 100.0), f64::NAN, 0.0),
        (Size::new(100.0, 100.0), 0.0, -0.5),
        (Size::new(100.0, 100.0), 0.0, f64::INFINITY),
        (Size::new(0.5, 100.0), 0.0, 0.0), // below the 3 pt minimum
        (Size::new(6000.0, 100.0), 0.0, 0.0), // beyond 200 inches
        (Size::new(5000.0, 100.0), 30.0, 15.0), // bleed and slug push it over
    ];
    for (trim, bleed, slug) in cases {
        let mut doc = base_doc(PdfStandard::X4);
        doc.pages.push(Page::new(Size::new(100.0, 100.0), 0.0, 0.0));
        doc.pages.push(Page::new(trim, bleed, slug));
        assert!(
            matches!(error(&doc), PdfError::InvalidPage { page: 2, .. }),
            "{trim:?} bleed {bleed} slug {slug}"
        );
    }
    // The limits themselves are fine.
    let mut doc = base_doc(PdfStandard::X4);
    doc.pages.push(Page::new(Size::new(5080.0, 1.1), 0.0, 0.0));
    assert!(write(&doc).is_ok());
}

// ----- colour and transparency -------------------------------------------------------

#[test]
fn rgb_paint_is_refused_in_pdfx1a_wherever_it_hides() {
    let rgb = Color::rgb8(200, 30, 30);
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let spectral = font("Spectral-Regular.ttf");
    let text = |font: FontId| text_item(font, &spectral, "rouge", 12.0, Point::new(5.0, 5.0), rgb);
    type MakeItem<'a> = Box<dyn Fn(FontId) -> Item + 'a>;
    let cases: Vec<(&str, MakeItem<'_>)> = vec![
        ("fill", Box::new(move |_| Item::filled_rect(rect, rgb))),
        ("stroke", Box::new(move |_| Item::stroke(Path::rect(rect), Stroke::solid(rgb, 1.0)))),
        ("text", Box::new(text)),
        (
            "nested",
            Box::new(move |_| {
                Item::layer("deep", vec![Item::clipped(rect, vec![Item::filled_rect(rect, rgb)])])
            }),
        ),
        ("black as rgb", Box::new(move |_| Item::filled_rect(rect, Color::rgb(0.0, 0.0, 0.0)))),
    ];
    for (name, make) in &cases {
        let mut doc = doc_with(PdfStandard::X1a2001, vec![]);
        let font = doc.add_font(spectral.clone(), 0);
        doc.pages[0].items.push(Item::filled_rect(rect, Color::BLACK));
        doc.pages[0].items.push(make(font));
        match error(&doc) {
            PdfError::RgbNotAllowed { standard: PdfStandard::X1a2001, location } => {
                assert!(location.contains("page 1"), "{name}: {location}");
            }
            other => panic!("{name}: unexpected {other:?}"),
        }
        // The very same content is fine everywhere else.
        for standard in [PdfStandard::X3_2002, PdfStandard::X4, PdfStandard::Plain] {
            doc.standard = standard;
            assert!(write(&doc).is_ok(), "{name} in {standard:?}");
        }
    }
}

#[test]
fn rgb_images_are_refused_in_pdfx1a() {
    let images = [
        jpeg_rgb(4, 4, |_, _| [9, 9, 9]),
        raw_image(PixelFormat::Rgb8, vec![1, 2, 3]),
        raw_image(PixelFormat::Rgba8, vec![1, 2, 3, 255]),
    ];
    for data in images {
        let mut doc = doc_with(PdfStandard::X1a2001, vec![]);
        let image = doc.add_image(data, None);
        doc.pages[0].items.push(image_item(image));
        match error(&doc) {
            PdfError::RgbNotAllowed { standard: PdfStandard::X1a2001, location } => {
                assert_eq!(location, "image 0");
            }
            other => panic!("unexpected {other:?}"),
        }
        for standard in [PdfStandard::X3_2002, PdfStandard::X4, PdfStandard::Plain] {
            doc.standard = standard;
            assert!(write(&doc).is_ok(), "{standard:?}");
        }
    }
    // Grey and CMYK images are welcome in PDF/X-1a.
    for data in [
        raw_image(PixelFormat::Gray8, vec![1]),
        raw_image(PixelFormat::Cmyk8, vec![1, 2, 3, 4]),
        jpeg_gray(4, 4, |_, _| 77),
        jpeg_cmyk(4, 4, [1, 2, 3, 4], true),
    ] {
        let mut doc = doc_with(PdfStandard::X1a2001, vec![]);
        let image = doc.add_image(data, None);
        doc.pages[0].items.push(image_item(image));
        assert!(write(&doc).is_ok());
    }
}

#[test]
fn transparency_is_refused_in_pdfx1a_and_pdfx3() {
    let translucent = raw_image(PixelFormat::Rgba8, vec![10, 20, 30, 254]);
    for standard in [PdfStandard::X1a2001, PdfStandard::X3_2002] {
        let mut doc = doc_with(standard, vec![]);
        let _other = doc.add_image(raw_image(PixelFormat::Gray8, vec![0]), None);
        let image = doc.add_image(translucent.clone(), None);
        doc.pages[0].items.push(image_item(image));
        assert_eq!(error(&doc), PdfError::TransparencyNotAllowed { standard, image: ImageId(1) });
    }
    for standard in [PdfStandard::X4, PdfStandard::Plain] {
        let mut doc = doc_with(standard, vec![]);
        let image = doc.add_image(translucent.clone(), None);
        doc.pages[0].items.push(image_item(image));
        assert!(write(&doc).is_ok(), "{standard:?}");
    }
}

#[test]
fn icc_tagged_images_are_refused_in_pdfx1a() {
    let mut doc = doc_with(PdfStandard::X1a2001, vec![]);
    let image =
        doc.add_image(raw_image(PixelFormat::Cmyk8, vec![0, 0, 0, 255]), Some(cmyk_profile()));
    doc.pages[0].items.push(image_item(image));
    assert_eq!(
        error(&doc),
        PdfError::IccNotAllowed { standard: PdfStandard::X1a2001, image: ImageId(0) }
    );
    for standard in [PdfStandard::X3_2002, PdfStandard::X4, PdfStandard::Plain] {
        doc.standard = standard;
        assert!(write(&doc).is_ok(), "{standard:?}");
    }
}

#[test]
fn image_profiles_must_match_the_image_and_the_pdf_version() {
    let is_icc_error = |doc: &Document| matches!(error(doc), PdfError::InvalidIccProfile { context, .. } if context == "image 0");
    // A CMYK profile on an RGB image, an RGB profile on a grey image, garbage.
    let cases = [
        (raw_image(PixelFormat::Rgb8, vec![1, 2, 3]), cmyk_profile()),
        (raw_image(PixelFormat::Gray8, vec![1]), srgb_v2_profile()),
        (raw_image(PixelFormat::Cmyk8, vec![1, 2, 3, 4]), srgb_v4_profile()),
        (raw_image(PixelFormat::Rgb8, vec![1, 2, 3]), Arc::from(b"junk".to_vec())),
        (jpeg_rgb(2, 2, |_, _| [1, 2, 3]), Arc::from(vec![0u8; 128])),
    ];
    for (data, profile) in cases {
        let mut doc = doc_with(PdfStandard::X4, vec![]);
        let image = doc.add_image(data, Some(profile));
        doc.pages[0].items.push(image_item(image));
        assert!(is_icc_error(&doc));
    }
    // An ICC v4 profile cannot go into a PDF 1.3 file.
    let mut doc = doc_with(PdfStandard::X3_2002, vec![]);
    let image = doc.add_image(raw_image(PixelFormat::Rgb8, vec![1, 2, 3]), Some(srgb_v4_profile()));
    doc.pages[0].items.push(image_item(image));
    assert!(is_icc_error(&doc));
    doc.images[0].icc_profile = Some(srgb_v2_profile());
    assert!(write(&doc).is_ok());
}

// ----- resources ---------------------------------------------------------------------

#[test]
fn unknown_resources_are_reported() {
    let mut doc = doc_with(PdfStandard::X4, vec![image_item(ImageId(3))]);
    assert_eq!(error(&doc), PdfError::UnknownImage(ImageId(3)));

    doc.pages[0].items = vec![Item::layer(
        "x",
        vec![Item::Glyphs {
            font: FontId(9),
            size_pt: 10.0,
            color: Color::BLACK,
            origin: Point::new(0.0, 0.0),
            glyphs: vec![Glyph { id: 1, x: 0.0, y: 0.0 }],
            glyph_text: vec![],
        }],
    )];
    assert_eq!(error(&doc), PdfError::UnknownFont(FontId(9)));
}

#[test]
fn malformed_images_are_reported_not_embedded() {
    let ImageData::Jpeg { bytes: good, .. } = jpeg_rgb(8, 6, |_, _| [1, 2, 3]) else {
        unreachable!()
    };
    let jpeg = |bytes: &[u8], width_px, height_px, components| ImageData::Jpeg {
        bytes: Arc::from(bytes.to_vec()),
        width_px,
        height_px,
        components,
    };
    let png_signature = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
    let cases = [
        jpeg(&good, 6, 8, 3), // swapped dimensions
        jpeg(&good, 8, 6, 1), // wrong component count
        jpeg(&good, 8, 6, 4),
        jpeg(&good[..40], 8, 6, 3), // truncated before the frame header
        jpeg(&[], 8, 6, 3),
        jpeg(png_signature, 8, 6, 3), // not a JPEG
        jpeg(&good, 0, 0, 3),
        ImageData::Raw {
            pixels: Arc::from(vec![0u8; 5]),
            width_px: 2,
            height_px: 1,
            format: PixelFormat::Rgb8,
        },
        ImageData::Raw {
            pixels: Arc::from(vec![]),
            width_px: 0,
            height_px: 0,
            format: PixelFormat::Gray8,
        },
        ImageData::Raw {
            pixels: Arc::from(vec![0u8; 16]),
            width_px: u32::MAX,
            height_px: u32::MAX,
            format: PixelFormat::Cmyk8,
        },
    ];
    for (i, data) in cases.into_iter().enumerate() {
        for standard in [PdfStandard::Plain, PdfStandard::X4] {
            let mut doc = doc_with(standard, vec![]);
            let image = doc.add_image(data.clone(), None);
            doc.pages[0].items.push(image_item(image));
            assert!(
                matches!(error(&doc), PdfError::InvalidImage { image: ImageId(0), .. }),
                "case {i} in {standard:?}"
            );
        }
    }
    // A broken image that no page draws is simply not embedded.
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    doc.add_image(jpeg(&[], 1, 1, 3), None);
    assert!(write(&doc).is_ok());
}

#[test]
fn fonts_that_forbid_embedding_are_refused() {
    let original = font("Spectral-Regular.ttf");
    let cases = [
        (0x0002, Some(EmbeddingRestriction::Restricted)),
        (0x0102, Some(EmbeddingRestriction::Restricted)),
        (0x0200, Some(EmbeddingRestriction::BitmapOnly)),
        (0x0204, Some(EmbeddingRestriction::BitmapOnly)),
        (0x0000, None),
        (0x0004, None), // preview & print
        (0x0008, None), // editable
        (0x0100, None), // no subsetting: embedded whole
    ];
    for (fs_type, expected) in cases {
        let data = with_fs_type(&original, fs_type);
        for standard in ALL {
            let mut doc = doc_with(standard, vec![]);
            let id = doc.add_font(data.clone(), 0);
            doc.pages[0].items.push(text_item(
                id,
                &data,
                "texte",
                10.0,
                Point::new(5.0, 5.0),
                Color::BLACK,
            ));
            match (write(&doc), expected) {
                (Ok(_), None) => {}
                (
                    Err(PdfError::FontEmbeddingForbidden { font, name, restriction }),
                    Some(wanted),
                ) => {
                    assert_eq!((font, restriction), (id, wanted), "fsType {fs_type:#06x}");
                    assert_eq!(name, "Spectral-Regular");
                }
                (other, _) => panic!("fsType {fs_type:#06x}: unexpected {other:?}"),
            }
        }
    }
    // A restricted font that is registered but never used does no harm.
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    doc.add_font(with_fs_type(&original, 0x0002), 0);
    assert!(write(&doc).is_ok());
}

#[test]
fn malformed_fonts_and_glyph_ids_are_reported() {
    let run = |font: FontId, glyph: u16| Item::Glyphs {
        font,
        size_pt: 10.0,
        color: Color::BLACK,
        origin: Point::new(5.0, 5.0),
        glyphs: vec![Glyph { id: glyph, x: 0.0, y: 0.0 }],
        glyph_text: vec!["x".into()],
    };
    let good = font("CourierPrime-Bold.ttf");
    let mut truncated = good[..good.len() / 3].to_vec();
    truncated.truncate(2000);
    for garbage in [vec![], b"<html>not a font</html>".to_vec(), vec![0u8; 100], truncated] {
        let mut doc = doc_with(PdfStandard::X4, vec![]);
        let id = doc.add_font(Arc::from(garbage), 0);
        doc.pages[0].items.push(run(id, 1));
        assert!(matches!(error(&doc), PdfError::InvalidFont { font, .. } if font == id));
    }
    // Wrong face index.
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    let id = doc.add_font(good.clone(), 3);
    doc.pages[0].items.push(run(id, 1));
    assert!(matches!(error(&doc), PdfError::InvalidFont { .. }));

    // Glyph ids beyond the font.
    let count = ttf_parser::Face::parse(&good, 0).expect("parses").number_of_glyphs();
    for glyph in [count, u16::MAX] {
        let mut doc = doc_with(PdfStandard::X4, vec![]);
        let id = doc.add_font(good.clone(), 0);
        doc.pages[0].items.push(run(id, glyph));
        assert_eq!(error(&doc), PdfError::InvalidGlyph { font: id, glyph, glyph_count: count });
    }
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    let id = doc.add_font(good, 0);
    doc.pages[0].items.push(run(id, count - 1));
    doc.pages[0].items.push(run(id, 0));
    assert!(write(&doc).is_ok());
}

// ----- content -----------------------------------------------------------------------

#[test]
fn numbers_must_be_finite_and_in_range() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let fill = |path: Path| Item::fill(path, Color::BLACK);
    let spectral = font("Spectral-Regular.ttf");
    let glyphs = |size_pt: f64, origin: Point, glyph: Glyph| Item::Glyphs {
        font: FontId(0),
        size_pt,
        color: Color::BLACK,
        origin,
        glyphs: vec![Glyph { id: 5, x: 0.0, y: 0.0 }, glyph],
        glyph_text: vec![],
    };
    let ok = Glyph { id: 6, x: 3.0, y: 0.0 };
    let bad: Vec<(&str, Item)> = vec![
        ("NaN point", fill(Path::line(Point::new(f64::NAN, 0.0), Point::new(1.0, 1.0)))),
        ("infinite point", fill(Path::line(Point::new(0.0, 0.0), Point::new(1.0, f64::INFINITY)))),
        ("huge point", fill(Path::line(Point::new(0.0, 0.0), Point::new(1.0e9, 0.0)))),
        (
            "NaN control",
            fill(Path {
                segs: vec![
                    PathSeg::MoveTo(Point::new(0.0, 0.0)),
                    PathSeg::QuadTo(Point::new(f64::NAN, 1.0), Point::new(2.0, 2.0)),
                ],
            }),
        ),
        ("NaN image", Item::Image { image: ImageId(0), rect: Rect::new(0.0, f64::NAN, 5.0, 5.0) }),
        (
            "infinite image",
            Item::Image { image: ImageId(0), rect: Rect::new(0.0, 0.0, f64::INFINITY, 5.0) },
        ),
        (
            "NaN transform",
            Item::group(
                Transform { a: f64::NAN, ..Transform::IDENTITY },
                vec![Item::filled_rect(rect, Color::BLACK)],
            ),
        ),
        (
            "huge translation",
            Item::group(
                Transform::translate(0.0, 1.0e12),
                vec![Item::filled_rect(rect, Color::BLACK)],
            ),
        ),
        (
            "NaN clip",
            Item::Group {
                transform: Transform::IDENTITY,
                clip: Some(Path::line(Point::new(0.0, 0.0), Point::new(f64::NAN, 0.0))),
                layer: None,
                items: vec![Item::filled_rect(rect, Color::BLACK)],
            },
        ),
        ("negative width", Item::stroke(Path::rect(rect), Stroke::solid(Color::BLACK, -0.1))),
        ("NaN width", Item::stroke(Path::rect(rect), Stroke::solid(Color::BLACK, f64::NAN))),
        (
            "negative dash",
            Item::stroke(Path::rect(rect), Stroke::dashed(Color::BLACK, 1.0, vec![1.0, -1.0])),
        ),
        (
            "all-zero dash",
            Item::stroke(Path::rect(rect), Stroke::dashed(Color::BLACK, 1.0, vec![0.0, 0.0])),
        ),
        (
            "NaN dash",
            Item::stroke(Path::rect(rect), Stroke::dashed(Color::BLACK, 1.0, vec![f64::NAN])),
        ),
        (
            "endless dash",
            Item::stroke(Path::rect(rect), Stroke::dashed(Color::BLACK, 1.0, vec![1.0; 9000])),
        ),
        ("zero font size", glyphs(0.0, Point::new(1.0, 1.0), ok)),
        ("negative font size", glyphs(-12.0, Point::new(1.0, 1.0), ok)),
        ("NaN font size", glyphs(f64::NAN, Point::new(1.0, 1.0), ok)),
        ("vanishing font size", glyphs(1.0e-60, Point::new(1.0, 1.0), ok)),
        ("NaN origin", glyphs(12.0, Point::new(f64::NAN, 1.0), ok)),
        ("NaN glyph x", glyphs(12.0, Point::new(1.0, 1.0), Glyph { id: 6, x: f64::NAN, y: 0.0 })),
        (
            "infinite glyph y",
            glyphs(12.0, Point::new(1.0, 1.0), Glyph { id: 6, x: 0.0, y: f64::NEG_INFINITY }),
        ),
    ];
    for (name, item) in bad {
        let mut doc = doc_with(PdfStandard::X4, vec![]);
        doc.add_font(spectral.clone(), 0);
        doc.add_image(raw_image(PixelFormat::Gray8, vec![0]), None);
        doc.pages.push(Page::new(Size::new(50.0, 50.0), 0.0, 0.0));
        doc.pages[1].items.push(item);
        assert!(
            matches!(error(&doc), PdfError::InvalidContent { page: 2, .. }),
            "{name}: {:?}",
            write(&doc).map(|pdf| pdf.len())
        );
    }
}

#[test]
fn paths_must_start_with_a_move() {
    let p = Point::new(1.0, 1.0);
    let bad = [
        vec![PathSeg::LineTo(p)],
        vec![PathSeg::Close],
        vec![PathSeg::QuadTo(p, p)],
        vec![PathSeg::CubicTo(p, p, p), PathSeg::MoveTo(p)],
    ];
    for segs in bad {
        let doc =
            doc_with(PdfStandard::X4, vec![Item::fill(Path { segs: segs.clone() }, Color::BLACK)]);
        assert!(matches!(error(&doc), PdfError::InvalidContent { page: 1, .. }), "{segs:?}");
        // The same goes for clip paths.
        let doc = doc_with(
            PdfStandard::X4,
            vec![Item::Group {
                transform: Transform::IDENTITY,
                clip: Some(Path { segs: segs.clone() }),
                layer: None,
                items: vec![Item::filled_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK)],
            }],
        );
        assert!(matches!(error(&doc), PdfError::InvalidContent { page: 1, .. }), "{segs:?}");
    }
    // Drawing on after a Close is fine: the current point is the sub-path's start.
    let ok = Path {
        segs: vec![
            PathSeg::MoveTo(p),
            PathSeg::LineTo(Point::new(5.0, 1.0)),
            PathSeg::Close,
            PathSeg::LineTo(Point::new(1.0, 5.0)),
        ],
    };
    assert!(
        write(&doc_with(PdfStandard::X4, vec![Item::stroke(ok, Stroke::solid(Color::BLACK, 1.0))]))
            .is_ok()
    );
}

fn nested(depth: usize) -> Item {
    let mut item = Item::filled_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
    for _ in 0..depth {
        item = Item::group(Transform::translate(0.1, 0.1), vec![item]);
    }
    item
}

#[test]
fn group_nesting_is_limited() {
    assert!(write(&doc_with(PdfStandard::X4, vec![nested(24)])).is_ok());
    for depth in [25, 26, 200, 5_000] {
        let doc = doc_with(
            PdfStandard::X4,
            vec![Item::filled_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK), nested(depth)],
        );
        assert_eq!(error(&doc), PdfError::NestingTooDeep { page: 1, max: 24 }, "depth {depth}");
    }
    // The deepest accepted nesting stays within the 28 levels PDF consumers must support,
    // even with an image (which needs one more level) at the bottom.
    let mut doc = doc_with(PdfStandard::X4, vec![]);
    let image = doc.add_image(raw_image(PixelFormat::Gray8, vec![0]), None);
    let mut item = image_item(image);
    for _ in 0..24 {
        item = Item::group(Transform::translate(0.1, 0.1), vec![item]);
    }
    doc.pages[0].items.push(item);
    let pdf =
        parse(&write_pdf(&doc, &WriteOptions { compress: false, keep_layers: false }).expect("ok"));
    let content = page_content(&pdf, 1);
    let mut depth = 0usize;
    let mut deepest = 0usize;
    for line in content.lines() {
        match line {
            "q" => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            "Q" => depth -= 1,
            _ => {}
        }
    }
    assert_eq!(depth, 0, "balanced");
    assert_eq!(deepest, 25);
    assert!(deepest <= 28);
}

#[test]
fn errors_are_readable() {
    let messages = [
        PdfError::NoPages.to_string(),
        PdfError::MissingOutputIntent { standard: PdfStandard::X4 }.to_string(),
        PdfError::RgbNotAllowed { standard: PdfStandard::X1a2001, location: "image 2".into() }
            .to_string(),
        PdfError::TransparencyNotAllowed { standard: PdfStandard::X3_2002, image: ImageId(5) }
            .to_string(),
        PdfError::FontEmbeddingForbidden {
            font: FontId(1),
            name: "Secret-Bold".into(),
            restriction: EmbeddingRestriction::Restricted,
        }
        .to_string(),
        PdfError::UnknownFont(FontId(4)).to_string(),
    ];
    assert_eq!(messages[0], "the document has no pages");
    assert_eq!(messages[1], "PDF/X-4 requires an output intent");
    assert_eq!(messages[2], "PDF/X-1a:2001 does not allow RGB content (image 2)");
    assert_eq!(
        messages[3],
        "PDF/X-3:2002 does not allow transparency (image 5 has a non-opaque alpha channel)"
    );
    assert_eq!(
        messages[4],
        "font 1 (Secret-Bold) must not be embedded: restricted-licence embedding (fsType bit 1)"
    );
    assert_eq!(messages[5], "unknown font id 4");
}
