//! Realistic input: cards written by `ebc-pdf` (compressed streams, real subset fonts, real
//! images, printer's marks in the slug), preflighted and then mangled.
//!
//! `ebc-pdf` is a dev-dependency only. The fixtures of the other test files are deliberately
//! built with `lopdf` instead, so that the checks are never validated solely against the
//! writer they are meant to referee; this file is the cross-check that the two agree.

#![allow(clippy::unwrap_used)]

mod common;

use std::path::Path as FsPath;
use std::sync::{Arc, mpsc};
use std::time::Duration;

use common::{Rng, codes, set};
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_preflight::{Expectations, PreflightError, Report, Severity, check, codes as c};
use ebc_scene::{
    Color, Document, FontId, Glyph, ImageData, Item, OutputIntent, Page, Path, PdfStandard,
    PixelFormat, Point, Rect, Size, Stroke,
};

const COVER_W: f64 = 110.0;
const COVER_H: f64 = 178.0;
const SPINE: f64 = 10.0;
const TRIM: Size = Size::new(2.0 * COVER_W + SPINE, COVER_H);
const BLEED: f64 = 3.0;
const SLUG: f64 = 7.0;
const QR_MODULE: f64 = 0.5;
/// 2026-11-12T08:30:00Z.
const CREATED: i64 = 1_794_472_200;

/// "ISO Coated v2 300% (basICColor)", the printer-class (`prtr`) FOGRA39L profile that
/// `ebc-card` bundles (zlib licence).
fn fogra39() -> Arc<[u8]> {
    let path = FsPath::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc");
    Arc::from(std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
}

fn asset(relative: &str) -> Arc<[u8]> {
    let path = FsPath::new(env!("CARGO_MANIFEST_DIR")).join("../../assets").join(relative);
    Arc::from(std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
}

/// What can be varied about the card.
#[derive(Clone)]
struct Card {
    standard: PdfStandard,
    /// Cover image pixels; the image is placed on the front cover plus bleed (113 × 184 mm).
    cover_px: (u32, u32),
    background: Color,
    spine_key: &'static str,
    qr_module: f64,
    marks: bool,
    title: &'static str,
}

impl Card {
    fn good(standard: PdfStandard) -> Self {
        Self {
            standard,
            // 200 ppi.
            cover_px: (890, 1449),
            background: Color::cmyk(0.10, 0.05, 0.30, 0.0),
            spine_key: "10",
            qr_module: QR_MODULE,
            marks: true,
            title: "Le Petit Prince — carte cadeau",
        }
    }

    fn document(&self) -> Document {
        let mut doc = Document { standard: self.standard, ..Document::default() };
        doc.info.title = self.title.into();
        doc.info.creator = Some("ebc-preflight tests".into());
        doc.info.created_unix = Some(CREATED);
        doc.info.custom = vec![
            ("EBC.SpineWidthMm".into(), self.spine_key.into()),
            ("EBC.FoldXMm".into(), "110,120".into()),
            ("EBC.CoverWidthMm".into(), "110".into()),
            ("EBC.CoverHeightMm".into(), "178".into()),
        ];
        doc.output_intent = Some(OutputIntent {
            condition_identifier: "FOGRA39".into(),
            info: "ISO Coated v2 300% (basICColor)".into(),
            registry_name: "http://www.color.org".into(),
            icc_profile: Some(fogra39()),
            components: 4,
        });
        let font = doc.add_font(asset("fonts/AtkinsonHyperlegible-Regular.ttf"), 0);
        let (w, h) = self.cover_px;
        let mut pixels = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            for x in 0..w {
                // A soft gradient, 0 to 240 % ink.
                let c = u8::try_from(x * 153 / w).unwrap();
                let m = u8::try_from(y * 153 / h).unwrap();
                pixels.extend_from_slice(&[c, m, 153 - c / 2, 76]);
            }
        }
        let cover = doc.add_image(
            ImageData::Raw {
                pixels: Arc::from(pixels),
                width_px: w,
                height_px: h,
                format: PixelFormat::Cmyk8,
            },
            None,
        );

        let bleed_rect = Rect::new(-BLEED, -BLEED, TRIM.w + 2.0 * BLEED, TRIM.h + 2.0 * BLEED);
        let mut outside = Page::new(TRIM, BLEED, SLUG);
        outside.items.push(Item::filled_rect(bleed_rect, self.background));
        outside.items.push(Item::Image {
            image: cover,
            rect: Rect::new(COVER_W + SPINE, -BLEED, COVER_W + BLEED, COVER_H + 2.0 * BLEED),
        });
        outside.items.push(Item::filled_rect(
            Rect::new(COVER_W, -BLEED, SPINE, COVER_H + 2.0 * BLEED),
            Color::cmyk(0.0, 0.0, 0.0, 0.9),
        ));
        outside.items.push(text(font, 9.0, Point::new(20.0, 150.0), Color::BLACK, 12));
        if self.marks {
            outside.items.extend(crop_marks());
        }

        let mut inside = Page::new(TRIM, BLEED, SLUG);
        inside.items.push(Item::filled_rect(bleed_rect, Color::cmyk(0.0, 0.0, 0.0, 0.0)));
        inside
            .items
            .push(Item::fill(qr_path(Point::new(150.0, 60.0), self.qr_module), Color::BLACK));
        inside.items.push(text(font, 14.0, Point::new(150.0, 100.0), Color::BLACK, 10));
        inside.items.push(Item::stroke(
            Path::line(Point::new(140.0, 110.0), Point::new(220.0, 110.0)),
            Stroke::solid(Color::BLACK, 0.5),
        ));
        if self.marks {
            inside.items.extend(crop_marks());
        }
        doc.pages = vec![outside, inside];
        doc
    }

    fn pdf(&self) -> Vec<u8> {
        write_pdf(&self.document(), &WriteOptions { compress: true, keep_layers: false })
            .expect("the card is writable")
    }
}

/// A run of `count` glyphs. The glyph ids are arbitrary: only the font machinery matters.
fn text(font: FontId, size_pt: f64, origin: Point, color: Color, count: u16) -> Item {
    let glyphs = (0..count).map(|i| Glyph { id: 36 + i, x: f64::from(i) * 2.5, y: 0.0 }).collect();
    let glyph_text =
        (0..count).map(|i| char::from(b'A' + u8::try_from(i).unwrap()).to_string()).collect();
    Item::Glyphs { font, size_pt, color, origin, glyphs, glyph_text }
}

/// Crop marks in registration colour, entirely in the slug.
fn crop_marks() -> Vec<Item> {
    let mut items = Vec::new();
    let stroke = || Stroke::solid(Color::Registration, 0.25);
    for (x, dx) in [(0.0, -1.0), (TRIM.w, 1.0)] {
        for (y, dy) in [(0.0, -1.0), (TRIM.h, 1.0)] {
            let near = BLEED + 0.5;
            let far = BLEED + SLUG - 0.5;
            items.push(Item::stroke(
                Path::line(Point::new(x + dx * near, y), Point::new(x + dx * far, y)),
                stroke(),
            ));
            items.push(Item::stroke(
                Path::line(Point::new(x, y + dy * near), Point::new(x, y + dy * far)),
                stroke(),
            ));
        }
    }
    items
}

/// The fixture's QR-like matrix as one path of merged horizontal runs.
fn qr_path(origin: Point, module: f64) -> Path {
    let mut path = Path::new();
    for (row, cells) in common::qr_matrix().iter().enumerate() {
        let mut x = 0;
        while x < cells.len() {
            if !cells[x] {
                x += 1;
                continue;
            }
            let start = x;
            while x < cells.len() && cells[x] {
                x += 1;
            }
            path.push_rect(Rect::new(
                origin.x + start as f64 * module,
                origin.y + row as f64 * module,
                (x - start) as f64 * module,
                module,
            ));
        }
    }
    path
}

fn expectations() -> Expectations {
    Expectations::professional().with_trim(TRIM.w, TRIM.h).with_spine(SPINE).with_min_qr_module(0.4)
}

/// Runs `check` under a watchdog; a contained panic fails the test.
fn bounded(pdf: &[u8], expect: &Expectations) -> Result<Report, PreflightError> {
    let (sender, receiver) = mpsc::channel();
    let data = pdf.to_vec();
    let expect = expect.clone();
    std::thread::spawn(move || {
        let _ = sender.send(check(&data, &expect));
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(120))
        .expect("check() did not finish in time or crashed");
    if let Err(PreflightError::Internal(message)) = &result {
        panic!("check() panicked internally: {message}");
    }
    result
}

/// A good card carries no finding at all. (It used to carry one warning: the 8 kB CC0
/// profile of `assets/icc` is of class `scnr`, which is now an error for PDF/X output, so
/// the cards here use the printer-class FOGRA39 profile bundled with `ebc-card`.)
fn assert_clean(report: &Report, what: &str) {
    assert!(report.findings.is_empty(), "{what}: {:#?}", report.findings);
    assert!(report.passed(), "{what}");
}

#[test]
fn cards_written_by_ebc_pdf_pass() {
    for (standard, claim, version) in [
        (PdfStandard::X4, "PDF/X-4", "1.6"),
        (PdfStandard::X1a2001, "PDF/X-1a:2001", "1.3"),
        (PdfStandard::X3_2002, "PDF/X-3:2002", "1.3"),
    ] {
        let pdf = Card::good(standard).pdf();
        let report = check(&pdf, &expectations()).unwrap();
        assert_clean(&report, claim);

        assert_eq!(report.document.pdf_version, version);
        assert_eq!(report.document.pdfx_version.as_deref(), Some(claim));
        assert_eq!(report.document.title.as_deref(), Some("Le Petit Prince — carte cadeau"));
        assert_eq!(report.document.trapped.as_deref(), Some("False"));
        assert_eq!(report.document.ebc.get("EBC.FoldXMm").map(String::as_str), Some("110,120"));
        let intent = report.document.output_intent.as_ref().unwrap();
        assert_eq!(intent.identifier.as_deref(), Some("FOGRA39"));
        assert_eq!(intent.profile_components, Some(4));

        assert_eq!(report.pages.len(), 2);
        for page in &report.pages {
            assert_eq!(page.trim_mm, Some((230.0, 178.0)), "{claim}");
            let bleed = page.bleed_mm.unwrap();
            assert!((bleed.min() - 3.0).abs() < 0.01, "{claim}: {bleed:?}");
            let media = page.media_box.unwrap();
            assert!((media.x1 - media.x0 - 250.0).abs() < 0.01, "{claim}: {media:?}");
            assert_eq!(page.fonts.len(), 1, "{claim}");
            let font = &page.fonts[0];
            assert!(font.embedded && font.subset && font.used, "{claim}: {font:?}");
            assert!(font.name.ends_with("+AtkinsonHyperlegible-Regular"), "{}", font.name);
            assert!(!page.transparency);
            assert!(page.color_spaces.contains(&"DeviceCMYK".to_string()), "{claim}");
            // The crop marks are registration colour, and they stay out of the bleed box.
            assert!(page.color_spaces.contains(&"Separation(All)".to_string()), "{claim}");
        }
        let cover = &report.pages[0].images;
        assert_eq!(cover.len(), 1);
        assert_eq!((cover[0].width_px, cover[0].height_px), (890, 1449));
        assert!(
            (cover[0].width_mm - 113.0).abs() < 0.01 && (cover[0].height_mm - 184.0).abs() < 0.01
        );
        assert!((cover[0].ppi_x - 200.0).abs() < 0.5 && (cover[0].ppi_y - 200.0).abs() < 0.5);
        assert_eq!(cover[0].color_space, "DeviceCMYK");
        // Heaviest vector colour on the outside: the K-only text (100 %); the spine is 90 %.
        // (The image, a gradient of 210 % at most, is measured separately and is within
        // the limit: no IMAGE_INK_TOTAL_TOO_HIGH above.)
        let ink = report.pages[0].max_total_ink_percent.unwrap();
        assert!((ink - 100.0).abs() < 0.1, "{claim}: {ink}");
        // The vector QR code on the inside is recognised and measured.
        let qr = &report.pages[1].qr_candidates;
        assert_eq!(qr.len(), 1, "{claim}: {qr:?}");
        assert!((qr[0].module_mm - QR_MODULE).abs() < 0.01, "{claim}: {qr:?}");
    }
}

#[test]
fn defects_composed_through_the_writer_are_found() {
    let only = |card: &Card, expect: &Expectations| -> std::collections::BTreeSet<&'static str> {
        let report = check(&card.pdf(), expect).unwrap();
        let mut found = codes(&report);
        found.remove(c::OUTPUT_INTENT_INVALID);
        found
    };
    let good = Card::good(PdfStandard::X4);

    // A cover at 72 ppi.
    let card = Card { cover_px: (320, 521), ..good.clone() };
    let report = check(&card.pdf(), &expectations()).unwrap();
    assert!(!report.passed());
    let low = report.findings.iter().find(|f| f.code == c::IMAGE_LOW_RES).unwrap();
    assert_eq!((low.severity, low.page), (Severity::Error, Some(1)));
    assert!(low.message.contains("72 ppi"), "{low}");
    assert_eq!(only(&card, &expectations()), set(&[c::IMAGE_LOW_RES]));

    // A cover with the wrong proportions for its frame.
    let card = Card { cover_px: (890, 1000), ..good.clone() };
    assert_eq!(only(&card, &expectations()), set(&[c::IMAGE_DISTORTED, c::IMAGE_LOW_RES]));
    let card = Card { cover_px: (1100, 1449), ..good.clone() };
    assert_eq!(only(&card, &expectations()), set(&[c::IMAGE_DISTORTED]));

    // Rich black far over the ink limit.
    let card = Card { background: Color::cmyk(0.9, 0.9, 0.9, 1.0), ..good.clone() };
    assert_eq!(only(&card, &expectations()), set(&[c::INK_TOTAL_TOO_HIGH]));

    // The spine key contradicts both the expectation and the folds.
    let card = Card { spine_key: "12", ..good.clone() };
    assert_eq!(
        only(&card, &expectations()),
        set(&[c::EBC_GEOMETRY_MISMATCH, c::EBC_SPINE_MISMATCH])
    );

    // QR modules of 0.3 mm against a minimum of 0.4 mm.
    let card = Card { qr_module: 0.3, ..good.clone() };
    assert_eq!(only(&card, &expectations()), set(&[c::QR_MODULE_TOO_SMALL]));

    // Another product: the trim size and the spine no longer match.
    let other = Expectations::professional().with_trim(210.0, 148.0).with_spine(10.0);
    assert_eq!(only(&good, &other), set(&[c::BOX_TRIM_SIZE]));
    // More bleed than the file has.
    let greedy = Expectations { bleed_mm: 5.0, ..expectations() };
    assert_eq!(only(&good, &greedy), set(&[c::BOX_BLEED_TOO_SMALL]));

    // RGB artwork in a plain PDF: flagged for a press, fine at home.
    let card = Card {
        standard: PdfStandard::Plain,
        background: Color::rgb(0.9, 0.2, 0.2),
        ..good.clone()
    };
    assert_eq!(only(&card, &expectations()), set(&[c::COLOR_RGB, c::PDFX_NOT_CLAIMED]));
    let at_home = Expectations::home().with_trim(TRIM.w, TRIM.h).with_spine(SPINE);
    let report = check(&card.pdf(), &at_home).unwrap();
    assert!(report.passed(), "{:#?}", report.findings);
    assert!(!report.has(c::COLOR_RGB));
}

#[test]
fn truncated_and_corrupted_writer_output_never_panics_or_hangs() {
    let pdf = Card { cover_px: (300, 488), ..Card::good(PdfStandard::X4) }.pdf();
    let expect = Expectations { min_ppi: 60.0, ..expectations() };
    assert_clean(&bounded(&pdf, &expect).unwrap(), "intact");
    let startxref = pdf.windows(9).rposition(|w| w == b"startxref").unwrap();

    // Truncation: every cut either fails to parse or is reported as damaged; none passes.
    let step = (pdf.len() / 250).max(1);
    let mut cuts: Vec<usize> = (0..pdf.len()).step_by(step).collect();
    cuts.extend(pdf.len().saturating_sub(48)..pdf.len());
    let (mut errors, mut reports) = (0, 0);
    for cut in cuts {
        match bounded(&pdf[..cut], &expect) {
            Err(_) => errors += 1,
            Ok(report) => {
                reports += 1;
                if cut <= startxref {
                    assert!(!report.passed(), "cut at {cut} of {} passed", pdf.len());
                    assert!(report.has(c::XREF_DAMAGED), "cut at {cut}: {:?}", report.codes());
                }
            }
        }
    }
    assert!(errors > 0 && reports > 0, "{errors} errors, {reports} reports");

    // Corruption: overwritten bytes, zeroed blocks, a spliced-in copy of another part of the
    // file. Whatever comes back is acceptable as long as something does.
    let mut rng = Rng::new(0xEBC0_0BAD);
    for round in 0..300 {
        let mut data = pdf.clone();
        match round % 3 {
            0 => {
                for _ in 0..1 + rng.below(8) {
                    let at = rng.below(data.len());
                    data[at] = rng.bytes(1)[0];
                }
            }
            1 => {
                let at = rng.below(data.len());
                let len = (1 + rng.below(400)).min(data.len() - at);
                data[at..at + len].fill(0);
            }
            _ => {
                let from = rng.below(data.len());
                let len = (1 + rng.below(600)).min(data.len() - from);
                let to = rng.below(data.len() - len + 1);
                let chunk = data[from..from + len].to_vec();
                data[to..to + len].copy_from_slice(&chunk);
            }
        }
        let _ = bounded(&data, &expect);
    }

    // Garbage with a PDF header, and the file wrapped in garbage.
    for size in [0, 1, 64, 4096] {
        let mut data = b"%PDF-1.6\n".to_vec();
        data.extend(rng.bytes(size));
        assert!(bounded(&data, &expect).is_err(), "{size} random bytes parsed");
    }
    let mut wrapped = rng.bytes(200);
    wrapped.extend_from_slice(&pdf);
    wrapped.extend(rng.bytes(200));
    let _ = bounded(&wrapped, &expect);
}

/// Check B4 (`docs/research/03-print-requirements.md` §10.1): an upload is exactly trim +
/// bleed and carries no printer's marks. The writer's card has a 7 mm slug with crop marks
/// in the registration colour: both are errors for an upload, neither for a printer that
/// takes marks.
#[test]
fn an_upload_refuses_marks_and_a_slug() {
    let upload = Expectations { trim_mm: Some((TRIM.w, TRIM.h)), ..Expectations::upload() };
    let marked = Card::good(PdfStandard::X4).pdf();
    let report = check(&marked, &upload).unwrap();
    let found = codes(&report);
    assert!(found.contains(c::BOX_MEDIA_NOT_BLEED), "{found:?}");
    assert!(found.contains(c::MARKS_NOT_ALLOWED), "{found:?}");
    assert!(!report.passed());
    // The slug alone (no marks) is still refused; marks are allowed by default.
    let unmarked = Card { marks: false, ..Card::good(PdfStandard::X4) }.pdf();
    let found = codes(&check(&unmarked, &upload).unwrap());
    assert!(found.contains(c::BOX_MEDIA_NOT_BLEED) && !found.contains(c::MARKS_NOT_ALLOWED));
    assert!(check(&marked, &expectations()).unwrap().passed());
}

/// Check C6: a flat colour with a process channel above 0 and below 5 % is reported.
#[test]
fn light_tints_are_reported_for_professional_output() {
    let card =
        Card { background: Color::cmyk(0.10, 0.03, 0.30, 0.0), ..Card::good(PdfStandard::X4) };
    let report = check(&card.pdf(), &expectations()).unwrap();
    let tint = report.findings.iter().find(|f| f.code == c::INK_LIGHT_TINT).expect("a finding");
    assert_eq!((tint.severity, tint.page), (Severity::Warning, Some(1)));
    assert!(report.passed(), "a warning, not an error");
    let home = Expectations { require_output_intent: false, ..expectations() };
    assert!(!codes(&check(&card.pdf(), &home).unwrap()).contains(c::INK_LIGHT_TINT));
}
