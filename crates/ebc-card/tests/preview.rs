//! The SVG preview of composed cards.

#![allow(clippy::unwrap_used)]

mod common;

use common::{fonts, poche, with_cover};
use ebc_card::{RightsBasis, compose, preview_svg, svg};
use ebc_layout::{PrintProfile, Sheet};

#[test]
fn previews_show_what_the_pdf_shows() {
    let fonts = fonts();
    let req = with_cover(poche(PrintProfile::home(Sheet::A4)), RightsBasis::RightsHolderDeclared);
    let svgs = preview_svg(&req, &fonts).unwrap();
    assert_eq!(svgs.len(), 2);
    let (outside, inside) = (&svgs[0], &svgs[1]);
    for page in &svgs {
        // The sheet itself: A4 landscape, in millimetres, the trim box at the origin.
        assert!(
            page.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" width="297mm" height="210mm" viewBox="-33.5 -16 297 210">"#),
            "{}",
            &page[..160]
        );
        assert!(page.ends_with("</svg>"));
        assert_eq!(page.matches("<g").count(), page.matches("</g>").count());
        assert!(page.contains(r#"data-layer="marks""#) && page.contains(r#"data-layer="notes""#));
        // Text is drawn as outlines: no font is needed to look at the preview.
        assert!(!page.contains("<text"));
    }
    assert!(outside.contains("<title>outside</title>") && inside.contains("<title>inside</title>"));
    // The cover: one raster image, drawn where the document places it.
    assert_eq!(outside.matches("<image").count(), 1);
    assert!(outside.contains("href=\"data:image/png;base64,"));
    assert_eq!(inside.matches("<image").count(), 0);
    let (doc, _) = compose(&req, &fonts).unwrap();
    let (_, rect, _) = common::images(&doc, 0)[0];
    assert!((rect.x - 125.0).abs() < 1e-9 && (rect.y - 8.0323).abs() < 1e-3, "{rect:?}");
    assert!(outside.contains(r#"<image x="125" y="8.03"#), "the image is drawn at {rect:?}");
    // Many glyph runs and the QR code as paths.
    assert!(inside.matches("<path").count() > 40, "{}", inside.matches("<path").count());
    // Dashed fold marks.
    assert!(outside.contains("stroke-dasharray="));
    // The same request gives the same preview, and the pages equal `svg::render_page`.
    assert_eq!(svgs, preview_svg(&req, &fonts).unwrap());
    assert_eq!(*outside, svg::render_page(&doc, 0));
    assert_eq!(*inside, svg::render_page(&doc, 1));
}

#[test]
fn a_jpeg_cover_is_previewed_as_the_jpeg_and_a_full_bleed_cover_is_clipped() {
    let fonts = fonts();
    let mut req = poche(PrintProfile::upload_eu());
    let bytes = common::jpeg(&common::art::cover(620, 1004, false, true), false);
    req.cover = ebc_card::CoverInput::Encoded { bytes, rights: RightsBasis::PrintLicence };
    req.trim_authorised = true;
    let svgs = preview_svg(&req, &fonts).unwrap();
    assert!(svgs[0].contains("href=\"data:image/jpeg;base64,/9j/"));
    assert!(svgs[0].contains("clip-path=\"url(#clip1)\""));
    assert!(svgs[0].contains(r#"viewBox="-3 -3 236 184""#), "{}", &svgs[0][..200]);
}
