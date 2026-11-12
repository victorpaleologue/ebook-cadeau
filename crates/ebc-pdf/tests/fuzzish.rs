//! Corrupted resources must produce an error or a file — never a panic.

mod common;

use std::sync::Arc;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Color, Document, Glyph, ImageData, Item, Page, PdfStandard, Point, Rect, Size};

/// A small deterministic pseudo-random generator (64-bit LCG, high bits).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 =
            self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn doc_with_font(data: Vec<u8>, glyphs: &[u16]) -> Document {
    let mut doc = base_doc(PdfStandard::X4);
    let id = doc.add_font(Arc::from(data), 0);
    let mut page = Page::new(Size::new(100.0, 50.0), 0.0, 0.0);
    page.items.push(Item::Glyphs {
        font: id,
        size_pt: 10.0,
        color: Color::BLACK,
        origin: Point::new(5.0, 20.0),
        glyphs: glyphs
            .iter()
            .enumerate()
            .map(|(i, g)| Glyph { id: *g, x: i as f64 * 3.0, y: 0.0 })
            .collect(),
        glyph_text: glyphs.iter().map(|_| "x".to_owned()).collect(),
    });
    doc.pages.push(page);
    doc
}

fn corrupt_fonts(original: &[u8], seed: u64, rounds: usize) -> (usize, usize) {
    let mut rng = Lcg(seed);
    let glyphs: Vec<u16> = (1..40).collect();
    let (mut ok, mut refused) = (0, 0);
    for round in 0..rounds {
        let mut data = original.to_vec();
        match round % 4 {
            // Flip a few bytes anywhere.
            0 => {
                for _ in 0..1 + rng.below(8) {
                    let i = rng.below(data.len());
                    data[i] ^= 1 << rng.below(8);
                }
            }
            // Trash the table directory.
            1 => {
                for _ in 0..1 + rng.below(4) {
                    let i = rng.below(12 + 16 * 20);
                    data[i] = rng.next() as u8;
                }
            }
            // Truncate.
            2 => data.truncate(rng.below(data.len())),
            // Overwrite a block with one value.
            _ => {
                let start = rng.below(data.len());
                let end = (start + 1 + rng.below(512)).min(data.len());
                let value = rng.next() as u8;
                data[start..end].fill(value);
            }
        }
        // The call must return; what it returns depends on what was hit.
        match write_pdf(&doc_with_font(data, &glyphs), &WriteOptions::default()) {
            Ok(pdf) => {
                assert!(pdf.starts_with(b"%PDF-1.6"));
                ok += 1;
            }
            Err(_) => refused += 1,
        }
    }
    (ok, refused)
}

#[test]
fn corrupted_truetype_fonts_never_panic() {
    let original = font("AtkinsonHyperlegible-Regular.ttf");
    let (ok, refused) = corrupt_fonts(&original, 0x5eed_0001, 600);
    eprintln!("corrupted TrueType fonts: {ok} written, {refused} refused");
    assert!(refused > 100, "corruption of the directory and truncation are detected");
}

#[test]
fn corrupted_cff_fonts_never_panic() {
    let original = fixture("SourceCodePro-Regular.otf");
    let (ok, refused) = corrupt_fonts(&original, 0x5eed_0002, 600);
    eprintln!("corrupted CFF fonts: {ok} written, {refused} refused");
    assert!(refused > 100, "corruption of the directory and truncation are detected");
}

#[test]
fn corrupted_jpegs_never_panic() {
    let ImageData::Jpeg { bytes: original, .. } =
        jpeg_rgb(24, 16, |x, y| [x as u8 * 10, y as u8 * 15, 77])
    else {
        unreachable!()
    };
    let mut rng = Lcg(0x5eed_0003);
    let (mut ok, mut refused) = (0, 0);
    for round in 0..2_000 {
        let mut data = original.to_vec();
        if round % 3 == 0 {
            data.truncate(rng.below(data.len()));
        } else {
            // Corrupt the header region, where the writer actually looks.
            for _ in 0..1 + rng.below(6) {
                let i = rng.below(data.len().min(700));
                data[i] = rng.next() as u8;
            }
        }
        let mut doc = base_doc(PdfStandard::X4);
        let image = doc.add_image(
            ImageData::Jpeg { bytes: Arc::from(data), width_px: 24, height_px: 16, components: 3 },
            None,
        );
        let mut page = Page::new(Size::new(100.0, 50.0), 0.0, 0.0);
        page.items.push(Item::Image { image, rect: Rect::new(0.0, 0.0, 24.0, 16.0) });
        doc.pages.push(page);
        match write_pdf(&doc, &WriteOptions::default()) {
            Ok(_) => ok += 1,
            Err(_) => refused += 1,
        }
    }
    eprintln!("corrupted JPEG files: {ok} written, {refused} refused");
    assert!(ok > 0 && refused > 500);
}

#[test]
fn corrupted_icc_profiles_never_panic() {
    let original = cmyk_profile();
    let mut rng = Lcg(0x5eed_0004);
    for round in 0..1_000 {
        let mut data = original.to_vec();
        if round % 2 == 0 {
            data.truncate(rng.below(data.len()));
        } else {
            for _ in 0..1 + rng.below(6) {
                let i = rng.below(132);
                data[i] = rng.next() as u8;
            }
        }
        let mut doc = base_doc(PdfStandard::X4);
        doc.output_intent.as_mut().expect("intent").icc_profile = Some(Arc::from(data));
        doc.pages.push(Page::new(Size::new(100.0, 50.0), 0.0, 0.0));
        let _ = write_pdf(&doc, &WriteOptions::default());
    }
}
