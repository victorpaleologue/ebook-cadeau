//! Writes sample spreads (back | spine | front) as PDF files for visual inspection:
//! `target/tmp/ebc-spine-samples/*.pdf`. Render them with
//! `pdftoppm -r 100 -png <file> <prefix>`. The test itself only checks that the files
//! are written; nothing is compared.

mod common;

use std::sync::Arc;

use common::*;
use ebc_color::Rgb8;
use ebc_i18n::SpineDirection::{self, BottomToTop, TopToBottom};
use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::BookMeta;
use ebc_scene::{Color, Document, ImageData, Item, Page, PdfStandard, PixelFormat, Pixmap, Rect};
use ebc_spine::{BackgroundStyle, SpineOptions, SpineRequest, SpineSource, design_with};
use ebc_text::Font;

fn add_cover(doc: &mut Document, cover: &Pixmap, rect: Rect) -> Item {
    let rgb: Vec<u8> = cover.data().chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
    let pixels: Arc<[u8]> = rgb.into();
    let image = doc.add_image(
        ImageData::Raw {
            pixels,
            width_px: cover.width(),
            height_px: cover.height(),
            format: PixelFormat::Rgb8,
        },
        None,
    );
    Item::Image { image, rect }
}

struct Sample<'a> {
    name: &'a str,
    book: BookMeta,
    cover: Pixmap,
    direction: SpineDirection,
    font: Font,
    options: SpineOptions,
    provided: Option<Pixmap>,
}

fn write(sample: &Sample<'_>) {
    let format = CardFormat::poche();
    let profile = PrintProfile::professional_eu();
    let panels = format.outside().with_bleed(profile.bleed);
    let mut doc = Document { standard: PdfStandard::Plain, ..Document::default() };
    doc.info.title = format!("ebc-spine sample: {}", sample.name);

    let source = match &sample.provided {
        Some(image) => SpineSource::Provided(image),
        None => SpineSource::Synthesize,
    };
    let regular = serif_regular();
    let req = SpineRequest {
        book: &sample.book,
        cover: Some(&sample.cover),
        source,
        rect: panels.spine,
        bleed: profile.bleed,
        title_font: &sample.font,
        author_font: &regular,
        direction: sample.direction,
        safe_margin: profile.spine_safe_margin,
    };
    let spine = design_with(&req, &sample.options, &mut doc).expect("design");

    let mut page = Page::new(format.spread(), profile.bleed, 0.0);
    let Rgb8 { r, g, b } = spine.background;
    page.items.push(Item::filled_rect(panels.left, Color::rgb8(r, g, b)));
    page.items.push(add_cover(&mut doc, &sample.cover, panels.right));
    page.items.extend(spine.items);
    doc.pages.push(page);

    let bytes = ebc_pdf::write_pdf(&doc, &ebc_pdf::WriteOptions::default()).expect("a PDF");
    let dir = format!("{}/../../target/tmp/ebc-spine-samples", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).expect("the sample directory");
    let path = format!("{dir}/{}.pdf", sample.name);
    std::fs::write(&path, &bytes).expect("the sample file");
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn sample_spreads_are_written() {
    let samples = [
        Sample {
            name: "busy-auto-bottom-to-top",
            book: book("Le Comte de Monte-Cristo", &["Alexandre Dumas"]),
            cover: busy_cover(7),
            direction: BottomToTop,
            font: serif(),
            options: SpineOptions::default(),
            provided: None,
        },
        Sample {
            name: "cream-top-to-bottom",
            book: book("The Picture of Dorian Gray", &["Oscar Wilde"]),
            cover: flat_cover([250, 245, 230]),
            direction: TopToBottom,
            font: sans(),
            options: SpineOptions::default(),
            provided: None,
        },
        Sample {
            name: "framed-long-title",
            book: book(
                "L'Extraordinaire Voyage du fakir qui était resté coincé dans une armoire Ikea",
                &["Romain Puértolas"],
            ),
            cover: framed_cover([255, 255, 255], 10, 3),
            direction: BottomToTop,
            font: serif(),
            options: SpineOptions::default(),
            provided: None,
        },
        Sample {
            name: "opt-in-edge-continuation",
            book: book("Le Horla", &["Guy de Maupassant"]),
            cover: busy_cover(4),
            direction: BottomToTop,
            font: caps(),
            options: SpineOptions {
                background: BackgroundStyle::EdgeContinuation,
                ..SpineOptions::default()
            },
            provided: None,
        },
        Sample {
            name: "opt-in-provided-35mm-cropped",
            book: book("Germinal", &["Émile Zola"]),
            cover: flat_cover([190, 40, 45]),
            direction: BottomToTop,
            font: serif(),
            options: SpineOptions::default(),
            provided: Some(spine_image(35.0, 178.0, [190, 40, 45], 5.0)),
        },
    ];
    for sample in &samples {
        write(sample);
    }
}
