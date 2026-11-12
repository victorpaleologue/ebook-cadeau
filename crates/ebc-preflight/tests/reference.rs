//! The clean reference card must pass without a single finding, and the report must
//! describe it accurately.

#![allow(clippy::unwrap_used)]

mod common;

use common::{COVER_H, COVER_PX, Fixture, QR_MODULE, TRIM_W, card_expectations};
use ebc_preflight::{Expectations, check};

#[test]
fn reference_card_passes_with_no_findings_at_all() {
    let report = Fixture::reference().check();
    assert!(report.findings.is_empty(), "unexpected findings: {:#?}", report.findings);
    assert!(report.passed());
}

#[test]
fn report_describes_the_document() {
    let report = Fixture::reference().check();
    let doc = &report.document;
    assert_eq!(doc.pdf_version, "1.6");
    assert_eq!(doc.page_count, 2);
    assert!(!doc.encrypted);
    assert_eq!(doc.title.as_deref(), Some("Le Petit Prince — carte cadeau"));
    assert_eq!(doc.trapped.as_deref(), Some("False"));
    assert_eq!(doc.pdfx_version.as_deref(), Some("PDF/X-4"));
    let intent = doc.output_intent.as_ref().unwrap();
    assert_eq!(intent.subtype, "GTS_PDFX");
    assert_eq!(intent.identifier.as_deref(), Some("FOGRA39"));
    assert_eq!(intent.registry.as_deref(), Some("http://www.color.org"));
    assert!(intent.profile_embedded);
    assert_eq!(intent.profile_components, Some(4));
    assert_eq!(intent.profile_color_space.as_deref(), Some("CMYK"));
    let keys: Vec<(&str, &str)> = doc.ebc.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    assert_eq!(
        keys,
        vec![
            ("EBC.CoverHeightMm", "178"),
            ("EBC.CoverWidthMm", "110"),
            ("EBC.FoldXMm", "110,120"),
            ("EBC.SpineWidthMm", "10"),
        ]
    );
}

#[test]
fn report_describes_the_pages() {
    let report = Fixture::reference().check();
    assert_eq!(report.pages.len(), 2);
    for (index, page) in report.pages.iter().enumerate() {
        assert_eq!(page.number as usize, index + 1);
        assert_eq!(page.trim_mm, Some((TRIM_W, COVER_H)));
        let bleed = page.bleed_mm.unwrap();
        assert_eq!((bleed.left, bleed.right, bleed.top, bleed.bottom), (3.0, 3.0, 3.0, 3.0));
        let media = page.media_box.unwrap();
        assert!((media.width() - 236.0).abs() < 0.001 && (media.height() - 184.0).abs() < 0.001);
        let trim = page.trim_box.unwrap();
        assert!((trim.x0 - 3.0).abs() < 0.001 && (trim.y1 - 181.0).abs() < 0.001);
        assert!(page.crop_box.is_none() && page.art_box.is_none());
        assert_eq!(page.rotate, 0);
        assert!(!page.transparency);
        assert_eq!(page.color_spaces, vec!["DeviceCMYK".to_string()]);
        assert_eq!(page.fonts.len(), 1);
        let font = &page.fonts[0];
        assert_eq!(font.name, "EBCAAA+AtkinsonHyperlegible-Regular");
        assert_eq!(font.subtype, "Type0/CIDFontType2");
        assert!(font.embedded && font.subset && font.used);
        assert_eq!(font.subset_prefix.as_deref(), Some("EBCAAA"));
    }

    // Page 1: the cover image at 300 ppi, the heaviest fill is the 90 % black spine.
    let outside = &report.pages[0];
    assert_eq!(outside.images.len(), 1);
    let image = &outside.images[0];
    assert_eq!(image.name, "/Im1");
    assert_eq!((i64::from(image.width_px), i64::from(image.height_px)), COVER_PX);
    assert!((image.width_mm - 113.0).abs() < 0.01, "{image:?}");
    assert!((image.height_mm - 184.0).abs() < 0.01, "{image:?}");
    assert!((image.ppi_x - 300.0).abs() < 0.5 && (image.ppi_y - 300.0).abs() < 0.5, "{image:?}");
    assert!((image.min_ppi() - 300.0).abs() < 0.5);
    assert_eq!(image.color_space, "DeviceCMYK");
    assert_eq!(image.bits_per_component, 8);
    assert_eq!(image.filters, vec!["DCTDecode".to_string()]);
    assert!(!image.soft_mask && !image.inline);
    assert_eq!(outside.max_total_ink_percent, Some(90.0));
    assert!(outside.qr_candidates.is_empty());

    // Page 2: the QR code is recognised and measured.
    let inside = &report.pages[1];
    assert!(inside.images.is_empty());
    assert_eq!(inside.max_total_ink_percent, Some(100.0));
    assert_eq!(inside.qr_candidates.len(), 1, "{:?}", inside.qr_candidates);
    let qr = &inside.qr_candidates[0];
    assert_eq!(qr.modules, 25);
    assert!((qr.module_mm - QR_MODULE).abs() < 0.002, "{qr:?}");
    assert!((qr.side_mm - 12.5).abs() < 0.01, "{qr:?}");
}

#[test]
fn json_report_is_valid_stable_and_complete() {
    let mut fixture = Fixture::reference();
    let bytes = fixture.bytes();
    let report = check(&bytes, &card_expectations()).unwrap();
    let json = report.to_json();
    // Deterministic: same input, same output, also across a fresh serialisation of the PDF.
    assert_eq!(json, check(&bytes, &card_expectations()).unwrap().to_json());
    assert_eq!(json, Fixture::reference().check().to_json());

    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["passed"], true);
    assert_eq!(value["findings"].as_array().unwrap().len(), 0);
    assert_eq!(value["document"]["pdfx_version"], "PDF/X-4");
    assert_eq!(value["document"]["output_intent"]["identifier"], "FOGRA39");
    assert_eq!(value["document"]["ebc"]["EBC.SpineWidthMm"], "10");
    assert_eq!(value["pages"].as_array().unwrap().len(), 2);
    assert_eq!(value["pages"][0]["trim_mm"], serde_json::json!([230.0, 178.0]));
    assert_eq!(value["pages"][0]["bleed_mm"]["left"], 3.0);
    assert_eq!(value["pages"][0]["images"][0]["color_space"], "DeviceCMYK");
    assert_eq!(value["pages"][0]["fonts"][0]["embedded"], true);
    assert_eq!(value["pages"][1]["qr_candidates"][0]["modules"], 25);
    // The pretty form carries the same data.
    let pretty: serde_json::Value = serde_json::from_str(&report.to_json_pretty()).unwrap();
    assert_eq!(pretty, value);

    // A failing report serialises its findings with lower-case severities and stable codes.
    let failing = check(&bytes, &Expectations { bleed_mm: 5.0, ..card_expectations() }).unwrap();
    let value: serde_json::Value = serde_json::from_str(&failing.to_json()).unwrap();
    assert_eq!(value["passed"], false);
    let finding = &value["findings"][0];
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["code"], "BOX_BLEED_TOO_SMALL");
    assert_eq!(finding["page"], 1);
    assert!(finding["message"].as_str().unwrap().contains("3.00 mm"));
}

#[test]
fn object_streams_and_xref_streams_give_the_same_report() {
    let mut fixture = Fixture::reference();
    let classic = fixture.check();
    let modern_bytes = fixture.bytes_modern();
    assert!(
        modern_bytes.windows(7).any(|w| w == b"/ObjStm"),
        "the modern serialisation should use object streams"
    );
    let modern = check(&modern_bytes, &card_expectations()).unwrap();
    assert!(modern.findings.is_empty(), "{:#?}", modern.findings);
    assert_eq!(modern.pages, classic.pages);
    assert_eq!(modern.document.ebc, classic.document.ebc);
    assert_eq!(modern.document.output_intent, classic.document.output_intent);
}

#[test]
fn leading_junk_before_the_header_is_tolerated_but_only_a_little() {
    let mut fixture = Fixture::reference();
    let bytes = fixture.bytes();
    // More than a kilobyte of junk: not a PDF any more, as far as printers are concerned.
    let mut far = vec![b'\n'; 2000];
    far.extend_from_slice(&bytes);
    assert_eq!(check(&far, &card_expectations()), Err(ebc_preflight::PreflightError::NotPdf));
}

#[test]
fn every_emitted_code_is_listed_in_codes_all() {
    use ebc_preflight::codes;
    let mut sorted = codes::ALL.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted, codes::ALL, "codes::ALL must be sorted and free of duplicates");
    for code in codes::ALL {
        assert!(
            code.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'),
            "{code} is not SCREAMING_SNAKE_CASE"
        );
    }
}
