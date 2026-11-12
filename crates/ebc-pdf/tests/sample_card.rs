//! The full sample card: written to `target/tmp/ebc-pdf/sample-card/` for manual
//! inspection, parsed back, pushed through every external validator that is installed,
//! and rendered to check that things land where the layout put them.

mod common;

use common::card::{self, BLEED, MARGIN, SLUG, TRIM};
use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::PdfStandard;

const STANDARDS: [(PdfStandard, &str); 4] = [
    (PdfStandard::X4, "card-x4"),
    (PdfStandard::X3_2002, "card-x3"),
    (PdfStandard::X1a2001, "card-x1a"),
    (PdfStandard::Plain, "card-plain"),
];
/// Writes the card for `standard` into the scratch directory of `test` (each test has
/// its own, so that parallel tests never read a file another one is rewriting).
fn write_card(test: &str, standard: PdfStandard, name: &str) -> (Vec<u8>, String) {
    let doc = card::document(standard);
    let pdf = write_pdf(&doc, &WriteOptions { compress: true, keep_layers: true })
        .expect("card is writable");
    let path = save(test, &format!("{name}.pdf"), &pdf);
    (pdf, path)
}

#[test]
fn sample_card_structure() {
    for (standard, name) in STANDARDS {
        let (bytes, path) = write_card("sample-card", standard, name);
        // A human-readable variant next to it, for inspection in a text editor.
        let readable = write_pdf(
            &card::document(standard),
            &WriteOptions { compress: false, keep_layers: true },
        )
        .expect("card is writable");
        save("sample-card", &format!("{name}-uncompressed.pdf"), &readable);
        eprintln!("sample card written to {path} ({} bytes)", bytes.len());
        assert!(
            bytes.len() < 200_000,
            "{name}: {} bytes — subsetting keeps the file small",
            bytes.len()
        );
        let pdf = parse(&bytes);
        assert_eq!(pdf.get_pages().len(), 2);

        for n in 1..=2 {
            let p = page(&pdf, n);
            let media = [0.0, 0.0, pt(TRIM.w + 2.0 * MARGIN), pt(TRIM.h + 2.0 * MARGIN)];
            assert_close(&get_numbers(&pdf, p, "MediaBox"), &media, 1e-3);
            assert_close(&get_numbers(&pdf, p, "CropBox"), &media, 1e-3);
            assert_close(
                &get_numbers(&pdf, p, "BleedBox"),
                &[
                    pt(SLUG),
                    pt(SLUG),
                    pt(SLUG + TRIM.w + 2.0 * BLEED),
                    pt(SLUG + TRIM.h + 2.0 * BLEED),
                ],
                1e-3,
            );
            assert_close(
                &get_numbers(&pdf, p, "TrimBox"),
                &[pt(MARGIN), pt(MARGIN), pt(MARGIN + TRIM.w), pt(MARGIN + TRIM.h)],
                1e-3,
            );
        }

        // Page 1 uses four fonts, page 2 five (one of them CFF); six in total, all subset.
        let font_names = |n: u32| -> Vec<String> {
            let fonts = resources(&pdf, n, "Font").expect("fonts");
            fonts
                .iter()
                .map(|(k, _)| font_info(&pdf, n, &String::from_utf8_lossy(k)).base_font)
                .collect()
        };
        let strip = |names: Vec<String>| -> Vec<String> {
            names
                .into_iter()
                .map(|n| {
                    let (tag, name) = n.split_once('+').expect("every font is a subset");
                    assert_eq!(tag.len(), 6);
                    name.to_owned()
                })
                .collect()
        };
        assert_eq!(
            strip(font_names(1)),
            [
                "AtkinsonHyperlegible-Regular",
                "Spectral-SemiBold",
                "Spectral-Italic",
                "BebasNeue-Regular"
            ]
        );
        assert_eq!(
            strip(font_names(2)),
            [
                "AtkinsonHyperlegible-Regular",
                "Spectral-SemiBold",
                "Spectral-Italic",
                "CourierPrime-Bold",
                "SourceCodePro-Regular"
            ]
        );
        assert_eq!(font_info(&pdf, 2, "F5").cid_subtype, "CIDFontType0");
        assert_eq!(font_info(&pdf, 2, "F4").cid_subtype, "CIDFontType2");

        // Layers survive only where the standard has them.
        let layered = matches!(standard, PdfStandard::X4 | PdfStandard::Plain);
        assert_eq!(catalog(&pdf).has(b"OCProperties"), layered, "{name}");
        // Transparency (the badge's soft mask) likewise.
        let has_mask =
            pdf.objects.values().any(|o| o.as_stream().is_ok_and(|s| s.dict.has(b"SMask")));
        assert_eq!(has_mask, layered, "{name}");
        // Geometry travels in the Info dictionary for the preflight tool.
        let info = info(&pdf);
        assert_eq!(get_text(&pdf, info, "EBC.SpineWidthMm"), "10");
        assert_eq!(get_text(&pdf, info, "EBC.FoldXMm"), "110,120");
    }
}

#[test]
fn sample_card_passes_external_validators() {
    for (standard, name) in STANDARDS {
        let (_, path) = write_card("sample-card-validators", standard, name);

        if let Some((ok, stdout, stderr)) = run("qpdf", &["--check", &path]) {
            assert!(ok, "{name}: qpdf --check failed:\n{stdout}\n{stderr}");
            assert!(stdout.contains("No syntax or stream encoding errors found"), "{stdout}");
            assert!(!stderr.contains("WARNING"), "{name}: {stderr}");
        }

        if let Some((ok, stdout, stderr)) = run(
            "gs",
            &[
                "-q",
                "-dNOPAUSE",
                "-dBATCH",
                "-dPDFSTOPONERROR",
                "-dPDFSTOPONWARNING",
                "-sDEVICE=nullpage",
                &path,
            ],
        ) {
            assert!(ok, "{name}: ghostscript failed:\n{stdout}\n{stderr}");
            let output = format!("{stdout}{stderr}");
            assert!(!output.to_lowercase().contains("error"), "{name}: {output}");
            assert!(!output.to_lowercase().contains("warning"), "{name}: {output}");
        }

        if let Some((ok, stdout, stderr)) = run("pdffonts", &[&path]) {
            assert!(ok, "{name}: pdffonts failed: {stderr}");
            let rows: Vec<&str> = stdout.lines().skip(2).collect();
            assert_eq!(rows.len(), 6, "{stdout}");
            for row in &rows {
                let columns: Vec<&str> = row.split_whitespace().collect();
                // name, type (2-3 words), encoding, emb, sub, uni, object id (2 numbers)
                let n = columns.len();
                assert_eq!(&columns[n - 5..n - 2], ["yes", "yes", "yes"], "emb sub uni: {row}");
                assert_eq!(columns[n - 6], "Identity-H", "{row}");
            }
            assert_eq!(rows.iter().filter(|r| r.contains("CID TrueType")).count(), 5, "{stdout}");
            assert_eq!(rows.iter().filter(|r| r.contains("CID Type 0C")).count(), 1, "{stdout}");
        }

        if let Some((ok, stdout, stderr)) = run("pdfinfo", &["-box", &path]) {
            assert!(ok, "{name}: pdfinfo failed: {stderr}");
            let field = |key: &str| -> String {
                stdout
                    .lines()
                    .find_map(|l| l.strip_prefix(key).map(|v| v.trim().to_owned()))
                    .unwrap_or_else(|| panic!("pdfinfo prints no `{key}`:\n{stdout}"))
            };
            assert_eq!(field("Title:"), "Carte cadeau — L'Étranger");
            assert_eq!(field("Author:"), "Albert Camus");
            assert_eq!(field("Pages:"), "2");
            assert_eq!(field("Encrypted:"), "no");
            assert_eq!(field("JavaScript:"), "no");
            let expected_version = match standard {
                PdfStandard::X1a2001 | PdfStandard::X3_2002 => "1.3",
                PdfStandard::X4 => "1.6",
                PdfStandard::Plain => "1.7",
            };
            assert_eq!(field("PDF version:"), expected_version);
            let subtype =
                stdout.lines().find_map(|l| l.strip_prefix("PDF subtype:")).map(str::trim);
            let expected_subtype = match standard {
                PdfStandard::X1a2001 => Some("PDF/X-1:2001"),
                PdfStandard::X3_2002 => Some("PDF/X-3:2002"),
                PdfStandard::X4 => Some("PDF/X-4"),
                PdfStandard::Plain => None,
            };
            assert_eq!(subtype, expected_subtype, "{name}");
            let numbers = |key: &str| -> Vec<f64> {
                field(key).split_whitespace().map(|n| n.parse().expect("number")).collect()
            };
            assert_close(&numbers("MediaBox:"), &[0.0, 0.0, 708.66, 561.26], 0.01);
            assert_close(&numbers("CropBox:"), &[0.0, 0.0, 708.66, 561.26], 0.01);
            assert_close(&numbers("BleedBox:"), &[19.84, 19.84, 688.82, 541.42], 0.01);
            assert_close(&numbers("TrimBox:"), &[28.35, 28.35, 680.31, 532.91], 0.01);
        }

        for (page_number, expected) in
            [(1, card::OUTSIDE_TEXT.as_slice()), (2, card::INSIDE_TEXT.as_slice())]
        {
            let n = page_number.to_string();
            if let Some((ok, stdout, stderr)) = run("pdftotext", &["-f", &n, "-l", &n, &path, "-"])
            {
                assert!(ok, "{name}: pdftotext failed: {stderr}");
                for line in expected {
                    assert!(
                        stdout.contains(line),
                        "{name} page {page_number}: missing {line:?} in\n{stdout}"
                    );
                }
                if page_number == 1 {
                    // The rotated spine text is extracted as well.
                    assert!(stdout.contains(card::SPINE_TITLE), "{name}:\n{stdout}");
                    assert!(stdout.contains(card::SPINE_AUTHOR), "{name}:\n{stdout}");
                }
            }
        }

        if let Some((ok, _, stderr)) = run("mutool", &["draw", "-o", "/dev/null", &path]) {
            assert!(ok, "{name}: mutool draw failed: {stderr}");
            assert!(!stderr.contains("error"), "{name}: {stderr}");
        }
    }
}

fn is_reddish(p: [u8; 3]) -> bool {
    p[0] > 180 && p[1] < 100 && p[2] < 100
}

fn is_whitish(p: [u8; 3]) -> bool {
    p.iter().all(|c| *c > 225)
}

fn is_dark(p: [u8; 3]) -> bool {
    p.iter().all(|c| *c < 90)
}

#[test]
fn sample_card_renders_where_the_layout_says() {
    for (standard, name) in STANDARDS {
        const TEST: &str = "sample-card-rendering";
        let (_, path) = write_card(TEST, standard, name);
        let Some(outside) = render(TEST, &path, 1, 72) else { return };
        let at = |x: f64, y: f64| pixel_at(&outside, 72, MARGIN, x, y);

        // 250 x 198 mm at 72 dpi.
        assert!(outside.width().abs_diff(709) <= 1 && outside.height().abs_diff(561) <= 1);

        // Slug area is bare paper, the bleed carries the background tint.
        assert!(is_whitish(at(-6.0, 50.0)), "{name}: slug {:?}", at(-6.0, 50.0));
        let paper = at(50.0, 150.0);
        assert!(paper[0] > 235 && paper[2] > 200 && paper[2] < 240, "{name}: paper tint {paper:?}");
        assert!(close(at(-2.0, 50.0), paper, 6), "{name}: background reaches into the bleed");
        assert!(
            close(at(50.0, 180.0), paper, 6),
            "{name}: background reaches into the bottom bleed"
        );

        // The spine band is dark blue from bleed edge to bleed edge ...
        let spine = at(111.0, 100.0);
        assert!(spine[2] > spine[0] + 40 && spine[0] < 90, "{name}: spine colour {spine:?}");
        assert!(close(at(119.0, -2.0), spine, 8) && close(at(111.0, 180.0), spine, 8), "{name}");
        // ... and the cover image, which starts 12.5 mm further left, is clipped at the fold.
        assert!(
            close(at(111.0, 66.0), spine, 8),
            "{name}: image leaks into the spine: {:?}",
            at(111.0, 66.0)
        );
        assert!(close(at(118.5, 20.0), spine, 8), "{name}");

        // Cover: pale disc at pixel (150, 150) of the image, teal above, orange below.
        let disc = at(176.5, 66.0);
        assert!(disc.iter().all(|c| *c > 200), "{name}: disc {disc:?}");
        let top = at(225.0, 2.0);
        assert!(top[2] > top[0] + 40, "{name}: top of the cover is teal: {top:?}");
        let bottom = at(225.0, 176.0);
        assert!(bottom[0] > bottom[2] + 90, "{name}: bottom of the cover is orange: {bottom:?}");
        // The image bleeds off the right and bottom edges.
        assert!(!is_whitish(at(232.0, 100.0)) && !is_whitish(at(200.0, 180.0)), "{name}");
        assert!(is_whitish(at(235.0, 100.0)), "{name}: nothing beyond the bleed box");

        // Spine title: light glyphs inside the band, running downwards from y = 12 mm,
        // and none where there is no text.
        let light_pixels = |y0: f64, y1: f64| -> usize {
            let mut count = 0;
            let mut y = y0;
            while y < y1 {
                let mut x = 112.0;
                while x < 118.0 {
                    if is_whitish(at(x, y)) {
                        count += 1;
                    }
                    x += 0.2;
                }
                y += 0.2;
            }
            count
        };
        assert!(
            light_pixels(12.0, 45.0) > 150,
            "{name}: spine title missing ({})",
            light_pixels(12.0, 45.0)
        );
        assert_eq!(light_pixels(60.0, 120.0), 0, "{name}: stray marks on the spine");
        assert_eq!(light_pixels(0.0, 10.0), 0, "{name}: the title starts 12 mm from the top");

        // Back panel headline: dark glyph pixels on the paper.
        let mut dark = 0;
        for i in 0..400 {
            let (x, y) = (12.0 + f64::from(i % 80), 26.0 + f64::from(i / 80));
            if is_dark(at(x, y)) {
                dark += 1;
            }
        }
        assert!(dark > 10, "{name}: headline missing");

        // Inside, at a higher resolution for the code symbol.
        let Some(inside) = render(TEST, &path, 2, 144) else { return };
        let at = |x: f64, y: f64| pixel_at(&inside, 144, MARGIN, x, y);
        let stamp = card::STAMP;
        assert!(is_reddish(at(stamp.x + 15.0, stamp.y + 7.5)), "{name}: {:?}", at(35.0, 127.5));
        for (x, y) in [(1.0, 1.0), (29.0, 1.0), (1.0, 14.0), (29.0, 14.0)] {
            assert!(is_reddish(at(stamp.x + x, stamp.y + y)), "{name}: stamp corner ({x}, {y})");
        }
        for (x, y) in [(-1.0, 7.5), (31.0, 7.5), (15.0, -1.0), (15.0, 16.0)] {
            assert!(
                is_whitish(at(stamp.x + x, stamp.y + y)),
                "{name}: outside the stamp ({x}, {y})"
            );
        }
        let disc = at(card::DISC.0.x, card::DISC.0.y);
        assert!(disc[2] > 150 && disc[0] < 80, "{name}: blue disc {disc:?}");

        // Every module of the code symbol is where the pattern says.
        for my in 0..card::QR_SIZE {
            for mx in 0..card::QR_SIZE {
                let x = card::QR_ORIGIN.x + (f64::from(mx) + 0.5) * card::QR_MODULE;
                let y = card::QR_ORIGIN.y + (f64::from(my) + 0.5) * card::QR_MODULE;
                let pixel = at(x, y);
                if card::qr_module(mx, my) {
                    assert!(
                        is_dark(pixel),
                        "{name}: module ({mx}, {my}) should be dark: {pixel:?}"
                    );
                } else {
                    assert!(
                        is_whitish(pixel),
                        "{name}: module ({mx}, {my}) should be light: {pixel:?}"
                    );
                }
            }
        }
        // Quiet zone all around.
        for offset in [-2.0, f64::from(card::QR_SIZE) * card::QR_MODULE + 2.0] {
            assert!(is_whitish(at(card::QR_ORIGIN.x + offset, card::QR_ORIGIN.y + 10.0)), "{name}");
            assert!(is_whitish(at(card::QR_ORIGIN.x + 10.0, card::QR_ORIGIN.y + offset)), "{name}");
        }
    }
}
