//! Opt-in cross-check of the bounded outline path against the outlines `read-fonts`
//! draws on its own, on real fonts installed on the machine (PostScript-flavoured ones
//! are the point: their charstrings are run by this crate through a metered context), and
//! of shaping against `harfrust` on the complete, unrestricted font.
//!
//! The two paths agree in geometry, not command for command: this crate closes TrueType
//! contours with an explicit line, starts them at another point when the first one is
//! off the curve, keeps zero-length segments and fractional coordinates of charstrings
//! (the library drops the former and truncates the latter). Compared are therefore the
//! number of curves and the exact bounding box of every glyph, to within one font unit.
//!
//! Not part of the offline suite — the fonts are not fixtures. Run with
//!
//! ```sh
//! EBC_TEXT_REFERENCE_FONTS=/path/a.otf:/path/b.otf \
//!     cargo test -p ebc-text --release --test cff_reference -- --ignored --nocapture
//! ```
#![allow(missing_docs)]

use ebc_scene::PathSeg;
use ebc_scene::{Path, Point};
use ebc_text::{Font, ShapeOptions, ShapedGlyph, ShapedText, outline_bounds, shape};
use read_fonts::model::glyph::outline::{DrawMemory, DrawOptions, PathSink};
use read_fonts::model::glyph::{GlyphSource, GlyphSourceOrder};
use read_fonts::types::GlyphId;

/// Builds a path from the library's callbacks with the arithmetic of `to_outline`.
struct Reference {
    path: Path,
    scale: f64,
}

impl Reference {
    fn map(&self, x: f32, y: f32) -> (f64, f64) {
        (0.0 + f64::from(x) * self.scale, 0.0 - f64::from(y) * self.scale)
    }
}

impl PathSink for Reference {
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
fn every_glyph_of_the_reference_fonts_is_drawn_as_the_font_library_draws_it() {
    let Ok(list) = std::env::var("EBC_TEXT_REFERENCE_FONTS") else {
        eprintln!("EBC_TEXT_REFERENCE_FONTS is not set: nothing checked");
        return;
    };
    for file in list.split(':').filter(|f| !f.is_empty()) {
        let bytes = std::fs::read(file).expect("readable font file");
        let faces = read_fonts::CollectionRef::new(&bytes).map_or(1, |collection| collection.len());
        // The outline tables this crate draws, in its order of preference.
        let order =
            GlyphSourceOrder::new(&[GlyphSource::Glyf, GlyphSource::Cff, GlyphSource::Cff2]);
        for index in 0..faces {
            let font = Font::from_bytes(bytes.clone(), index).expect("parses");
            let face = read_fonts::model::Font::new(bytes.clone(), index).expect("parses");
            let outlines = face.outline_glyphs_with(order);
            let glyph_count = u16::try_from(face.num_glyphs()).expect("at most 65 535 glyphs");
            let size_pt = 100.0;
            let scale = font.metrics().mm_per_unit(size_pt);
            let (mut drawn, mut different) = (0u32, 0u32);
            for id in 0..glyph_count {
                let mut reference = Reference { path: Path::new(), scale };
                let drawn_by_the_library =
                    outlines.get(GlyphId::new(u32::from(id))).is_some_and(|glyph| {
                        glyph.draw(&DrawOptions::new(), DrawMemory::Scratch, &mut reference).is_ok()
                    });
                if !drawn_by_the_library {
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
                if !same_shape(&ours, &reference.path, scale) {
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
            let full = harfrust::ShaperFont::new(&face);
            let mut theirs = harfrust::Buffer::new();
            theirs.push_str(text);
            theirs.set_direction(harfrust::Direction::LeftToRight);
            theirs.guess_segment_properties();
            harfrust::shape(&full, &mut theirs, harfrust::ShapeOptions::new()).expect("shapes");
            let ids: Vec<u32> = theirs.glyph_infos().iter().map(|i| i.glyph_id).collect();
            assert_eq!(ours.glyphs.iter().map(|g| u32::from(g.id)).collect::<Vec<_>>(), ids);
            let advance: i32 = theirs.glyph_positions().iter().map(|p| p.x_advance).sum();
            assert!((ours.width - f64::from(advance) * scale).abs() < 1e-9, "{file}");
            // Fallback mark positioning reads glyph extents. For fonts with colour or
            // bitmap glyphs the shaper takes them from those tables, which this crate
            // withholds, and for PostScript fonts from the charstrings, which it withholds
            // from the shaper (they can be made arbitrarily expensive).
            if font.outline_format() == ebc_text::OutlineFormat::TrueType
                && [b"COLR", b"sbix", b"CBDT"].iter().all(|tag| !has_table(&bytes, tag))
            {
                for (glyph, position) in ours.glyphs.iter().zip(theirs.glyph_positions()) {
                    let expected = -f64::from(position.y_offset) * scale;
                    assert!((glyph.y - expected).abs() < 1e-9, "{file}");
                }
            }

            eprintln!(
                "{file} face {index}: {} glyphs, {drawn} with an outline, {different} different \
                 ({:?}, oversized: {})",
                glyph_count,
                font.outline_format(),
                font.has_oversized_glyphs()
            );
            assert_eq!(different, 0, "{file} face {index}");
        }
    }
}

/// `true` if the (single-font) file lists a table with this tag.
fn has_table(font: &[u8], tag: &[u8; 4]) -> bool {
    read_fonts::FontRef::new(font).is_ok_and(|font| {
        font.table_directory().table_records().iter().any(|record| record.tag() == *tag)
    })
}

/// `true` if two outlines have the same number of curves and the same exact bounding
/// box, to within one font unit (`scale` millimetres).
fn same_shape(a: &Path, b: &Path, scale: f64) -> bool {
    let curves = |path: &Path| {
        path.segs.iter().filter(|s| matches!(s, PathSeg::QuadTo(..) | PathSeg::CubicTo(..))).count()
    };
    let boxes_agree = match (outline_bounds(a), outline_bounds(b)) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            [a.x - b.x, a.y - b.y, a.right() - b.right(), a.bottom() - b.bottom()]
                .iter()
                .all(|d| d.abs() <= scale)
        }
        _ => false,
    };
    curves(a) == curves(b) && boxes_agree
}
