//! The finished PDF seen through independent tools: poppler renders it, zbar reads the
//! QR code back, pdftotext extracts the text, qpdf and Ghostscript check the structure.
//! Each check is skipped with a note when its tool is not installed.

#![allow(clippy::unwrap_used)]

mod common;

use std::path::{Path, PathBuf};

use common::{BASE_URL, code, fonts, poche, run, scratch, tool, with_cover};
use ebc_card::{CardRequest, RightsBasis, SpineChoice, render_pdf};
use ebc_layout::{CardFormat, PrintProfile, Sheet, page_geometry};
use ebc_model::claim_url_qr;

fn write_pdf(dir: &Path, name: &str, req: &CardRequest) -> PathBuf {
    let (pdf, _) = render_pdf(req, &fonts()).unwrap();
    let path = dir.join(format!("{name}.pdf"));
    std::fs::write(&path, pdf).unwrap();
    path
}

/// Renders one page (1-based) to a PNG at `dpi` and returns its path.
fn rasterize(pdf: &Path, page: u32, dpi: u32) -> PathBuf {
    let stem = pdf.with_extension("").display().to_string();
    let prefix = format!("{stem}-p{page}-{dpi}");
    let page = page.to_string();
    run(
        "pdftoppm",
        &[
            "-f",
            &page,
            "-l",
            &page,
            "-r",
            &dpi.to_string(),
            "-png",
            "-singlefile",
            pdf.to_str().unwrap(),
            &prefix,
        ],
    );
    PathBuf::from(format!("{prefix}.png"))
}

fn profiles() -> Vec<PrintProfile> {
    vec![PrintProfile::upload_eu(), PrintProfile::professional_eu(), PrintProfile::home(Sheet::A4)]
}

#[test]
fn the_qr_code_read_from_the_rendered_page_is_the_upper_case_claim_url() {
    if !(tool("pdftoppm") && tool("zbarimg")) {
        return;
    }
    let dir = scratch("qr");
    let expected = claim_url_qr(BASE_URL, &code());
    assert_eq!(expected, format!("HTTPS://EBK.EXAMPLE/K/{}", code().compact()));
    for profile in profiles() {
        let pdf = write_pdf(&dir, &format!("qr-{}", profile.name), &poche(profile.clone()));
        // 300 dpi, a coarse 150 dpi, and a print scaled to 97 % ("fit to page").
        for dpi in [300, 150, 291] {
            let png = rasterize(&pdf, 2, dpi);
            let decoded = run("zbarimg", &["--raw", "-q", png.to_str().unwrap()]);
            assert_eq!(decoded.trim(), expected, "{} at {dpi} dpi", profile.name);
        }
        // Nothing on the outside page scans as a code (no barcode, no stray QR).
        let png = rasterize(&pdf, 1, 150);
        let output = std::process::Command::new("zbarimg")
            .args(["--raw", "-q", png.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty(), "{}", profile.name);
    }
    // With a cover and a store, in another language: the same payload.
    let mut req = with_cover(poche(PrintProfile::home(Sheet::Letter)), RightsBasis::PublicDomain);
    req.locale = ebc_model::Locale::De;
    req.store_name = Some("Septante (Demo)".to_owned());
    let pdf = write_pdf(&dir, "qr-cover-de", &req);
    let png = rasterize(&pdf, 2, 300);
    assert_eq!(run("zbarimg", &["--raw", "-q", png.to_str().unwrap()]).trim(), expected);
}

#[test]
fn the_text_can_be_extracted_and_sits_on_the_right_panels() {
    if !tool("pdftotext") {
        return;
    }
    let dir = scratch("text");
    let grouped = code().grouped();
    for profile in profiles() {
        let req = poche(profile.clone());
        let pdf = write_pdf(&dir, &format!("text-{}", profile.name), &req);
        let pdf_path = pdf.to_str().unwrap();
        // Line breaks and column gaps of the layout mode are noise here.
        let flat = |text: String| text.split_whitespace().collect::<Vec<_>>().join(" ");
        let inside = flat(run("pdftotext", &["-f", "2", "-l", "2", "-layout", pdf_path, "-"]));
        assert!(inside.contains(&grouped), "{}: {inside}", profile.name);
        assert!(inside.contains("ebk.example"), "{}: {inside}", profile.name);
        assert!(inside.contains("Ce livre est un cadeau") && inside.contains("Pour le lire"));
        assert!(inside.contains("Scanner ce code avec l’appareil photo"), "{inside}");
        assert!(inside.contains("Sans téléphone"), "the no-phone line: {inside}");
        let outside = flat(run("pdftotext", &["-f", "1", "-l", "1", "-layout", pdf_path, "-"]));
        assert!(outside.contains("Jeanne Roy") && outside.contains("Éditions du Lac"), "{outside}");
        assert!(!outside.contains(&grouped), "the code is inside the card only");

        // Left half of page 2 = inside-front (the gift); right half = inside-back (how
        // to get the book). pdftotext crops in pixels at the resolution given.
        let geometry = page_geometry(&req.format, &req.profile).unwrap();
        let px = |mm: f64| ((mm / 25.4) * 72.0).round().to_string();
        let middle = req.format.spread().w / 2.0 - geometry.media.x;
        let (half, height) = (px(middle), px(geometry.media.h));
        let left = run(
            "pdftotext",
            &[
                "-f", "2", "-l", "2", "-r", "72", "-x", "0", "-y", "0", "-W", &half, "-H", &height,
                pdf_path, "-",
            ],
        );
        let right = run(
            "pdftotext",
            &[
                "-f", "2", "-l", "2", "-r", "72", "-x", &half, "-y", "0", "-W", &half, "-H",
                &height, pdf_path, "-",
            ],
        );
        let (left, right) = (flat(left), flat(right));
        assert!(left.contains("Ce livre est un cadeau") && left.contains("Pour Bob"), "{left}");
        assert!(right.contains("Sans téléphone ? Sur la liseuse ou un ordinateur"), "{right}");
        assert!(right.contains("Usage unique. Sans date limite."), "the legal lines: {right}");
        assert!(!left.contains(&grouped));
        assert!(right.contains(&grouped) && right.contains("Pour le lire"), "{right}");
        // Outside: the front cover (title, publisher) is the right-hand panel.
        let front = run(
            "pdftotext",
            &[
                "-f", "1", "-l", "1", "-r", "72", "-x", &half, "-y", "0", "-W", &half, "-H",
                &height, pdf_path, "-",
            ],
        );
        assert!(front.contains("Éditions du Lac") && front.contains("Roman"), "{front}");
    }
}

/// The pixel of a rendered page at a position given in page millimetres.
fn pixel(
    image: &image::RgbImage,
    geometry: &ebc_layout::PageGeometry,
    dpi: f64,
    x: f64,
    y: f64,
) -> [u8; 3] {
    let px = ((x - geometry.media.x) / 25.4 * dpi) as u32;
    let py = ((y - geometry.media.y) / 25.4 * dpi) as u32;
    image.get_pixel(px.min(image.width() - 1), py.min(image.height() - 1)).0
}

fn close(a: [u8; 3], b: [u8; 3], tolerance: i32) -> bool {
    a.iter().zip(b).all(|(x, y)| (i32::from(*x) - i32::from(y)).abs() <= tolerance)
}

#[test]
fn the_rendered_outside_shows_the_cover_on_the_right_and_the_spine_colour_in_the_middle() {
    if !tool("pdftoppm") {
        return;
    }
    let dir = scratch("raster");
    let format = CardFormat::poche();
    let dpi = 60;
    // Home profile: RGB all the way, so colours can be compared closely.
    let profile = PrintProfile::home(Sheet::A4);
    let geometry = page_geometry(&format, &profile).unwrap();
    let front = format.front();
    let spine = format.spine();
    let back = format.back();

    // Typographic card: one flat ground over back, spine and front.
    let pdf = write_pdf(&dir, "typographic", &poche(profile.clone()));
    let page = image::open(rasterize(&pdf, 1, dpi)).unwrap().to_rgb8();
    let ground = pixel(&page, &geometry, f64::from(dpi), back.center().x, back.center().y);
    assert!(!close(ground, [255, 255, 255], 40), "a coloured ground: {ground:?}");
    for (x, y) in [
        (spine.center().x, 4.0),
        (spine.center().x, 174.0),
        (front.x + 3.0, 100.0),
        (front.right() - 2.0, 3.0),
        (-1.0, -1.0), // in the bleed
    ] {
        let p = pixel(&page, &geometry, f64::from(dpi), x, y);
        assert!(close(p, ground, 6), "({x}, {y}): {p:?} vs ground {ground:?}");
    }
    // Outside the bleed the sheet is white.
    let margin = pixel(&page, &geometry, f64::from(dpi), -10.0, 80.0);
    assert!(close(margin, [255, 255, 255], 2), "{margin:?}");
    // The inside is paper white.
    let inside = image::open(rasterize(&pdf, 2, dpi)).unwrap().to_rgb8();
    for (x, y) in [(3.0, 3.0), (spine.center().x, 90.0), (227.0, 175.0)] {
        assert!(close(pixel(&inside, &geometry, f64::from(dpi), x, y), [255, 255, 255], 2));
    }

    // With the own-made cover: its sun in the middle of the front panel, its dark frame
    // near the panel's safe edge, and the frame's colour as the one ground around it, on
    // the spine and on the back — a jacket in the cover's own colour.
    let mut req = with_cover(poche(profile.clone()), RightsBasis::RightsHolderDeclared);
    req.spine = SpineChoice::CoverColour;
    let pdf = write_pdf(&dir, "cover", &req);
    let page = image::open(rasterize(&pdf, 1, dpi)).unwrap().to_rgb8();
    let at = |x: f64, y: f64| pixel(&page, &geometry, f64::from(dpi), x, y);
    let sun = at(front.center().x, 5.0 + 0.40 * 168.0 * (100.0 / 103.75));
    assert!(sun[0] > 220 && sun[1] > 180 && sun[2] < 180, "the sun: {sun:?}");
    let frame = at(front.x + 7.0, 60.0);
    assert!(close(frame, [14, 22, 48], 12), "the cover's frame: {frame:?}");
    let around = at(front.x + 1.5, 60.0);
    assert!(close(around, [14, 22, 48], 3), "the ground is the frame's colour: {around:?}");
    let spine_colour = at(spine.center().x, 3.0);
    assert!(
        close(spine_colour, [14, 22, 48], 12),
        "the spine takes the frame's colour: {spine_colour:?}"
    );
    assert!(close(at(back.center().x, 90.0), spine_colour, 3), "and so does the back");
    // The back panel (left) shows no part of the cover image.
    assert!(!close(at(back.center().x, 76.0), sun, 60));

    // A press file renders too (colours go through poppler's CMYK conversion, so only
    // coarse checks): coloured ground, white inside.
    let pro = PrintProfile::professional_eu();
    let geometry = page_geometry(&format, &pro).unwrap();
    let pdf = write_pdf(&dir, "press", &poche(pro));
    let page = image::open(rasterize(&pdf, 1, dpi)).unwrap().to_rgb8();
    let ground = pixel(&page, &geometry, f64::from(dpi), back.center().x, back.center().y);
    assert!(!close(ground, [255, 255, 255], 40), "{ground:?}");
    assert!(close(pixel(&page, &geometry, f64::from(dpi), spine.center().x, 4.0), ground, 8));
}

#[test]
fn structure_boxes_and_fonts_according_to_qpdf_poppler_and_ghostscript() {
    let dir = scratch("structure");
    let mut cases = Vec::new();
    for profile in profiles() {
        cases.push((format!("typo-{}", profile.name), poche(profile.clone())));
        cases.push((
            format!("cover-{}", profile.name),
            with_cover(poche(profile), RightsBasis::RightsHolderDeclared),
        ));
    }
    for (name, req) in cases {
        let pdf = write_pdf(&dir, &name, &req);
        let path = pdf.to_str().unwrap();
        if tool("qpdf") {
            let out = run("qpdf", &["--check", path]);
            assert!(out.contains("No syntax or stream encoding errors found"), "{name}: {out}");
        }
        if tool("pdfinfo") {
            let info = run("pdfinfo", &["-box", path]);
            let pages = info.lines().find(|l| l.starts_with("Pages:")).unwrap();
            assert!(pages.trim_end().ends_with(" 2"), "{name}: {pages}");
            let geometry = page_geometry(&req.format, &req.profile).unwrap();
            let pt = |mm: f64| mm / 25.4 * 72.0;
            let page_size = info.lines().find(|l| l.starts_with("Page size:")).unwrap();
            let numbers: Vec<f64> =
                page_size.split_whitespace().filter_map(|token| token.parse().ok()).collect();
            assert!((numbers[0] - pt(geometry.media.w)).abs() < 0.01, "{name}: {page_size}");
            assert!((numbers[1] - pt(geometry.media.h)).abs() < 0.01, "{name}: {page_size}");
            for key in ["MediaBox:", "BleedBox:", "TrimBox:"] {
                assert!(info.contains(key), "{name}: {info}");
            }
            let trim: Vec<f64> = info
                .lines()
                .find(|l| l.starts_with("TrimBox:"))
                .unwrap()
                .split_whitespace()
                .filter_map(|token| token.parse().ok())
                .collect();
            assert!(
                (trim[2] - trim[0] - pt(req.format.spread().w)).abs() < 0.01,
                "{name}: {trim:?}"
            );
            assert!(
                (trim[3] - trim[1] - pt(req.format.spread().h)).abs() < 0.01,
                "{name}: {trim:?}"
            );
        }
        if tool("pdffonts") {
            let fonts = run("pdffonts", &[path]);
            let rows: Vec<&str> = fonts.lines().skip(2).collect();
            assert!(rows.len() >= 3, "{name}: {fonts}");
            for row in rows {
                // name, type, encoding, emb, sub, uni: embedded subsets with ToUnicode.
                let cells: Vec<&str> = row.split_whitespace().collect();
                let flags = &cells[cells.len() - 5..cells.len() - 2];
                assert_eq!(flags, ["yes", "yes", "yes"], "{name}: {row}");
            }
        }
        if tool("gs") {
            let output = std::process::Command::new("gs")
                .args(["-q", "-dNOPAUSE", "-dBATCH", "-dPDFSTOPONERROR", "-sDEVICE=nullpage", path])
                .output()
                .unwrap();
            let noise = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.status.success() && noise.trim().is_empty(), "{name}: gs says {noise}");
        }
    }
}
