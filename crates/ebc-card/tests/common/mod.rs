//! Helpers shared by the integration tests: a demo request, encoders for own-made
//! covers, preflight expectations derived from a profile, and access to external tools.
//!
//! The book, the people and the address are invented.

#![allow(dead_code, unreachable_pub, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ebc_card::{CardRequest, CoverInput, FontSet, RightsBasis, bundled_fonts};
use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::{BookMeta, Contributor, GiftCode, Locale};
use ebc_preflight::{Expectations, Report, Severity};
use ebc_scene::{PdfStandard, Pixmap};

pub mod art;

/// Base URL of the invented redemption service.
pub const BASE_URL: &str = "https://ebk.example";
/// 2026-11-06 12:00:00 UTC.
pub const CREATED_UNIX: i64 = 1_794_000_000;

pub fn fonts() -> FontSet {
    bundled_fonts().unwrap()
}

pub fn code() -> GiftCode {
    GiftCode::from_payload(123_456_789_012).unwrap()
}

pub fn book() -> BookMeta {
    let mut book = BookMeta::titled("Les Vagues du soir");
    book.subtitle = Some("Roman".to_owned());
    book.contributors.push(Contributor::author("Jeanne Roy"));
    book.publisher = Some("Éditions du Lac (fictif)".to_owned());
    book.language = Some("fr".to_owned());
    book
}

/// A complete typographic request.
pub fn request(format: CardFormat, profile: PrintProfile, locale: Locale) -> CardRequest {
    let mut req = CardRequest::new(book(), code(), BASE_URL);
    req.format = format;
    req.profile = profile;
    req.locale = locale;
    req.created_unix = Some(CREATED_UNIX);
    req.giver = Some("Alice".to_owned());
    req.recipient = Some("Bob".to_owned());
    req.message = Some("Joyeux anniversaire ! J’ai adoré ce roman.".to_owned());
    req
}

pub fn poche(profile: PrintProfile) -> CardRequest {
    request(CardFormat::poche(), profile, Locale::Fr)
}

/// The own-made cover at print size for the poche (about 315 ppi letterboxed).
pub fn cover_pixmap() -> Pixmap {
    art::cover(620, 1004, false, true)
}

pub fn with_cover(mut req: CardRequest, rights: RightsBasis) -> CardRequest {
    req.cover = CoverInput::Pixels { pixmap: art::cover(1240, 2008, false, true), rights };
    req
}

/// Baseline (or progressive) JPEG bytes of a pixmap.
pub fn jpeg(pixmap: &Pixmap, progressive: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = jpeg_encoder::Encoder::new(&mut out, 88);
    encoder.set_progressive(progressive);
    encoder
        .encode(
            &art::rgb_bytes(pixmap),
            u16::try_from(pixmap.width()).unwrap(),
            u16::try_from(pixmap.height()).unwrap(),
            jpeg_encoder::ColorType::Rgb,
        )
        .unwrap();
    out
}

/// PNG bytes of a pixmap, alpha included.
pub fn png(pixmap: &Pixmap) -> Vec<u8> {
    use image::ImageEncoder;
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut out))
        .write_image(
            pixmap.data(),
            pixmap.width(),
            pixmap.height(),
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    out
}

/// What a printer's preflight would expect of a card composed for `req`'s profile and
/// format. `rgb_cover` says that a cover is embedded: PDF/X-4 then carries it as tagged
/// sRGB, which the expectations must allow; everything the composer paints itself is
/// CMYK or grey in a PDF/X file.
pub fn expectations(req: &CardRequest, rgb_cover: bool) -> Expectations {
    let profile = &req.profile;
    let spread = req.format.spread();
    let plain = profile.standard == PdfStandard::Plain;
    Expectations {
        bleed_mm: profile.bleed,
        min_ppi: profile.min_ppi,
        require_output_intent: !plain,
        allow_rgb: plain || rgb_cover,
        allow_transparency: false,
        max_total_ink_percent: Some(profile.max_total_ink_percent),
        trim_mm: Some((spread.w, spread.h)),
        spine_mm: Some(req.format.spine_width),
        min_qr_module_mm: Some(ebc_qr::MIN_PRINT_MODULE_MM),
    }
}

/// The error findings of a report, formatted.
pub fn errors(report: &Report) -> Vec<String> {
    report.errors().map(|f| format!("{} (page {:?}): {}", f.code, f.page, f.message)).collect()
}

/// The warning findings of a report, formatted.
pub fn warnings(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .map(|f| format!("{} (page {:?}): {}", f.code, f.page, f.message))
        .collect()
}

/// Whether an external program is on `PATH`.
pub fn tool(name: &str) -> bool {
    let found = std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()));
    if !found {
        eprintln!("note: `{name}` is not installed — this check is skipped");
    }
    found
}

/// Runs a program and returns its standard output; panics with its standard error when
/// it fails.
pub fn run(program: &str, args: &[&str]) -> String {
    let output = Command::new(program).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Where the sample PDFs go: `target/tmp/ebc-card-samples/`.
pub fn sample_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ebc-card-samples");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A scratch directory under `target/tmp/` for one test.
pub fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ebc-card-scratch").join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The text of every glyph run of a page, one string per run, with the run's size.
pub fn text_runs(doc: &ebc_scene::Document, page: usize) -> Vec<(String, f64)> {
    fn walk(item: &ebc_scene::Item, out: &mut Vec<(String, f64)>) {
        match item {
            ebc_scene::Item::Glyphs { glyph_text, size_pt, .. } => {
                out.push((glyph_text.concat(), *size_pt));
            }
            ebc_scene::Item::Group { items, .. } => items.iter().for_each(|i| walk(i, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    doc.pages[page].items.iter().for_each(|i| walk(i, &mut out));
    out
}

/// All the text of a page, runs joined by line feeds.
pub fn page_text(doc: &ebc_scene::Document, page: usize) -> String {
    text_runs(doc, page).into_iter().map(|(text, _)| text).collect::<Vec<_>>().join("\n")
}

/// Every image item of a page with its destination rectangle and, when it sits in a
/// clipped group, the clip's bounding box.
pub fn images(
    doc: &ebc_scene::Document,
    page: usize,
) -> Vec<(ebc_scene::ImageId, ebc_scene::Rect, Option<ebc_scene::Rect>)> {
    fn walk(
        item: &ebc_scene::Item,
        clip: Option<ebc_scene::Rect>,
        out: &mut Vec<(ebc_scene::ImageId, ebc_scene::Rect, Option<ebc_scene::Rect>)>,
    ) {
        match item {
            ebc_scene::Item::Image { image, rect } => out.push((*image, *rect, clip)),
            ebc_scene::Item::Group { items, clip: group_clip, transform, .. } => {
                assert_eq!(
                    *transform,
                    ebc_scene::Transform::IDENTITY,
                    "images are never transformed"
                );
                let clip = group_clip.as_ref().and_then(ebc_scene::Path::control_bounds).or(clip);
                items.iter().for_each(|i| walk(i, clip, out));
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    doc.pages[page].items.iter().for_each(|i| walk(i, None, &mut out));
    out
}
