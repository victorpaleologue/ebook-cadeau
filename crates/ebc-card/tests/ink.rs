//! Dark covers on professional cards: the jacket keeps its dark ground, built under the
//! printer's total ink limit (240 % upload, 300 % offset) by grey-component replacement,
//! the converted cover obeys the same limit pixel by pixel, and the card advises
//! lamination for the dark creases.

#![allow(clippy::unwrap_used)]

mod common;

use common::{errors, expectations, fonts, poche, with_cover};
use ebc_card::{CardWarning, RightsBasis, SpineChoice, compose, render_pdf};
use ebc_layout::{PrintProfile, Sheet};
use ebc_preflight::{check, codes};
use ebc_scene::{Color, Document, ImageData, Item, PixelFormat};

/// The colour of the ground: the fill that covers the bleed box of the outside.
fn ground(doc: &Document) -> Color {
    let bleed = doc.pages[0].bleed_box();
    doc.pages[0]
        .items
        .iter()
        .find_map(|item| match item {
            Item::Path { fill: Some((color, _)), path, .. }
                if path.control_bounds().is_some_and(|b| b.contains_rect(&bleed, 1e-6)) =>
            {
                Some(*color)
            }
            _ => None,
        })
        .expect("a ground over the bleed")
}

/// The largest total ink of any pixel of the document's CMYK images, in 8-bit units.
fn max_image_ink(doc: &Document) -> u32 {
    doc.images
        .iter()
        .filter_map(|image| match &image.data {
            ImageData::Raw { pixels, format: PixelFormat::Cmyk8, .. } => {
                pixels.chunks_exact(4).map(|px| px.iter().map(|&v| u32::from(v)).sum::<u32>()).max()
            }
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

fn advice(warnings: &[CardWarning]) -> Option<(f32, f32)> {
    warnings.iter().find_map(|w| match w {
        CardWarning::DarkAcrossCreases { max_ink_percent, min_lightness } => {
            Some((*max_ink_percent, *min_lightness))
        }
        _ => None,
    })
}

#[test]
fn a_dark_cover_keeps_its_dark_jacket_under_every_ink_limit() {
    let fonts = fonts();
    for (profile, limit) in
        [(PrintProfile::upload_eu(), 240.0_f32), (PrintProfile::professional_eu(), 300.0)]
    {
        let name = profile.name.clone();
        let req = with_cover(poche(profile), RightsBasis::RightsHolderDeclared);
        let (doc, warnings) = compose(&req, &fonts).unwrap();
        // The navy frame of the cover, not paper white.
        let Color::Cmyk { c, m, y, k } = ground(&doc) else { panic!("{name}: CMYK ground") };
        let ink = (c + m + y + k) * 100.0;
        assert!(ink <= limit + 1e-3, "{name}: ground {ink} %");
        assert!(
            ink > 230.0 && k > 0.6 && c > 0.5,
            "{name}: a dark navy build, got {c} {m} {y} {k}"
        );
        // Every pixel of the converted cover is under the limit too.
        let units = max_image_ink(&doc);
        assert!(f64::from(units) <= f64::from(limit) * 2.55, "{name}: {units}");
        // The dark creases are flagged, with lamination advice.
        let (max_ink, lightness) = advice(&warnings).expect("advice for dark creases");
        assert!(max_ink > 180.0 && max_ink <= limit + 0.1 && lightness < 35.0, "{name}");
        assert!(warnings.iter().any(|w| w.to_string().contains("lamination")));

        // The file passes the referee with the profile's own limit.
        let (pdf, _) = render_pdf(&req, &fonts).unwrap();
        let report = check(&pdf, &expectations(&req, false)).unwrap();
        assert!(errors(&report).is_empty(), "{name}: {:#?}", errors(&report));
        assert!(!report.has(codes::IMAGE_INK_TOTAL_TOO_HIGH), "{name}");
        assert!(!report.has(codes::INK_TOTAL_TOO_HIGH), "{name}");
        // The referee sees the heavy ink at the creases too (a warning).
        assert!(report.has(codes::INK_HEAVY_AT_CREASE), "{name}: {:?}", report.codes());
        if limit < 300.0 {
            let max = report.pages[0].max_total_ink_percent.unwrap();
            assert!(max <= limit + 0.5, "{name}: {max}");
        }
    }
}

#[test]
fn the_upload_limit_rebuilds_rather_than_lightens() {
    let fonts = fonts();
    let upload = with_cover(poche(PrintProfile::upload_eu()), RightsBasis::RightsHolderDeclared);
    let offset =
        with_cover(poche(PrintProfile::professional_eu()), RightsBasis::RightsHolderDeclared);
    let (a, _) = compose(&upload, &fonts).unwrap();
    let (b, _) = compose(&offset, &fonts).unwrap();
    let (Color::Cmyk { c: c1, m: m1, y: y1, k: k1 }, Color::Cmyk { c: c2, m: m2, y: y2, k: k2 }) =
        (ground(&a), ground(&b))
    else {
        panic!("CMYK grounds")
    };
    // Under 240 % the build has more black and less colour ink than under 300 %.
    assert!(k1 > k2, "K {k1} vs {k2}");
    assert!(c1 + m1 + y1 < c2 + m2 + y2);
}

#[test]
fn spine_images_and_custom_grounds_obey_the_limit() {
    let fonts = fonts();
    let mut req = with_cover(poche(PrintProfile::upload_eu()), RightsBasis::PrintLicence);
    req.spine = SpineChoice::EdgeContinuation;
    req.adaptation_authorised = true;
    let (doc, _) = compose(&req, &fonts).unwrap();
    assert!(doc.images.len() >= 2, "cover and spine image");
    assert!(max_image_ink(&doc) <= 612);
    // A caller's own ground is written under the limit as well.
    let mut req = poche(PrintProfile::upload_eu());
    req.ground = Some(ebc_color::Rgb8::new(10, 20, 60));
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    let ink = ground(&doc).total_ink_percent().unwrap();
    assert!(ink <= 240.0 + 1e-3 && ink > 200.0, "{ink}");
    assert!(advice(&warnings).is_some());
}

#[test]
fn light_grounds_and_home_cards_get_no_lamination_advice() {
    let fonts = fonts();
    let mut req = poche(PrintProfile::upload_eu());
    req.ground = Some(ebc_color::Rgb8::new(232, 220, 194));
    let (_, warnings) = compose(&req, &fonts).unwrap();
    assert_eq!(advice(&warnings), None, "{warnings:?}");
    // At home the same dark cover keeps its ground and says nothing about lamination.
    let home = with_cover(poche(PrintProfile::home(Sheet::A4)), RightsBasis::RightsHolderDeclared);
    let (doc, warnings) = compose(&home, &fonts).unwrap();
    assert!(matches!(ground(&doc), Color::Rgb { .. }));
    assert_eq!(advice(&warnings), None);
    // The preview is not a print file either.
    let pro = with_cover(poche(PrintProfile::upload_eu()), RightsBasis::RightsHolderDeclared);
    assert!(ebc_card::preview_svg(&pro, &fonts).is_ok());
}
