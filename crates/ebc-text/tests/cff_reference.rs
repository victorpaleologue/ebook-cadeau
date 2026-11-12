//! Opt-in cross-check of the bounded outline path against `ttf-parser`'s own outlines,
//! on real fonts installed on the machine (PostScript-flavoured ones are the point: their
//! charstrings are flattened by this crate before `ttf-parser` draws them).
//!
//! Not part of the offline suite — the fonts are not fixtures. Run with
//!
//! ```sh
//! EBC_TEXT_REFERENCE_FONTS=/path/a.otf:/path/b.otf \
//!     cargo test -p ebc-text --release --test cff_reference -- --ignored --nocapture
//! ```
#![allow(missing_docs)]

use ebc_scene::{Path, Point};
use ebc_text::{Font, ShapeOptions, ShapedGlyph, ShapedText, shape};

/// Builds a path from `ttf-parser` callbacks with the arithmetic of `to_outline`.
struct Reference {
    path: Path,
    scale: f64,
}

impl Reference {
    fn map(&self, x: f32, y: f32) -> (f64, f64) {
        (0.0 + f64::from(x) * self.scale, 0.0 - f64::from(y) * self.scale)
    }
}

impl ttf_parser::OutlineBuilder for Reference {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.path.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.path.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (cx, cy) = self.map(x1, y1);
        let (x, y) = self.map(x, y);
        self.path.quad_to(cx, cy, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1x, c1y) = self.map(x1, y1);
        let (c2x, c2y) = self.map(x2, y2);
        let (x, y) = self.map(x, y);
        self.path.cubic_to(c1x, c1y, c2x, c2y, x, y);
    }
    fn close(&mut self) {
        self.path.close();
    }
}

#[test]
#[ignore = "needs EBC_TEXT_REFERENCE_FONTS (a colon-separated list of font files)"]
fn every_glyph_of_the_reference_fonts_is_drawn_exactly_as_ttf_parser_draws_it() {
    let Ok(list) = std::env::var("EBC_TEXT_REFERENCE_FONTS") else {
        eprintln!("EBC_TEXT_REFERENCE_FONTS is not set: nothing checked");
        return;
    };
    for file in list.split(':').filter(|f| !f.is_empty()) {
        let bytes = std::fs::read(file).expect("readable font file");
        let faces = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
        for index in 0..faces {
            let font = Font::from_bytes(bytes.clone(), index).expect("parses");
            let face = ttf_parser::Face::parse(&bytes, index).expect("parses");
            let size_pt = 100.0;
            let scale = font.metrics().mm_per_unit(size_pt);
            let (mut drawn, mut different) = (0u32, 0u32);
            for id in 0..face.number_of_glyphs() {
                let mut reference = Reference { path: Path::new(), scale };
                if face.outline_glyph(ttf_parser::GlyphId(id), &mut reference).is_none() {
                    reference.path = Path::new();
                }
                let run = ShapedText {
                    glyphs: vec![ShapedGlyph {
                        id,
                        x: 0.0,
                        y: 0.0,
                        advance: 0.0,
                        text: String::new(),
                    }],
                    width: 0.0,
                    size_pt,
                };
                let ours = run.try_to_outline(&font, Point::new(0.0, 0.0)).expect("within budget");
                drawn += u32::from(!ours.is_empty());
                if ours != reference.path {
                    different += 1;
                    if different <= 5 {
                        eprintln!(
                            "{file} face {index} glyph {id}: {} segments, reference {}",
                            ours.segs.len(),
                            reference.path.segs.len()
                        );
                    }
                }
            }
            // Shaping uses a view of the font without its outline programs: the result
            // must be what the shaper produces on the complete font.
            let text = "Hamburgefonstiv ÀÉï fi 123 Œuvre e\u{301} AV. To";
            let ours = shape(&font, text, size_pt, &ShapeOptions::default());
            let full = rustybuzz::Face::from_slice(&bytes, index).expect("parses");
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(text);
            buffer.set_direction(rustybuzz::Direction::LeftToRight);
            let theirs = rustybuzz::shape(&full, &[], buffer);
            let ids: Vec<u32> = theirs.glyph_infos().iter().map(|i| i.glyph_id).collect();
            assert_eq!(ours.glyphs.iter().map(|g| u32::from(g.id)).collect::<Vec<_>>(), ids);
            let advance: i32 = theirs.glyph_positions().iter().map(|p| p.x_advance).sum();
            assert!((ours.width - f64::from(advance) * scale).abs() < 1e-9, "{file}");
            // Fallback mark positioning reads glyph extents. For fonts with a `COLR` table
            // the shaper takes them from the paint graph, which this crate withholds (it
            // can be made arbitrarily expensive): there the extents come from `glyf`.
            if face.tables().colr.is_none() {
                for (glyph, position) in ours.glyphs.iter().zip(theirs.glyph_positions()) {
                    let expected = -f64::from(position.y_offset) * scale;
                    assert!((glyph.y - expected).abs() < 1e-9, "{file}");
                }
            }

            eprintln!(
                "{file} face {index}: {} glyphs, {drawn} with an outline, {different} different \
                 ({:?}, oversized: {})",
                face.number_of_glyphs(),
                font.outline_format(),
                font.has_oversized_glyphs()
            );
            assert_eq!(different, 0, "{file} face {index}");
        }
    }
}
