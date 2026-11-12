//! The font parser the tests check the writer against: `read-fonts` (the parser of the
//! fontations project), behind the handful of calls the tests need. The writer itself
//! reads fonts with its own table reader, so this is an independent opinion.

use read_fonts::model::Font;
use read_fonts::model::glyph::outline::{DrawMemory, DrawOptions};
use read_fonts::model::glyph::{GlyphSource, GlyphSourceOrder};
use read_fonts::ps::cff::CffFontRef;
use read_fonts::tables::post::Post;
use read_fonts::{FontData, FontRead, TableProvider};

/// Receives the outline of a glyph.
pub(crate) use read_fonts::model::glyph::outline::PathSink as OutlineBuilder;

/// A glyph index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlyphId(pub(crate) u16);

impl GlyphId {
    fn raw(self) -> read_fonts::types::GlyphId {
        read_fonts::types::GlyphId::new(u32::from(self.0))
    }
}

/// A box in font units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rect {
    pub(crate) x_min: i16,
    pub(crate) y_min: i16,
    pub(crate) x_max: i16,
    pub(crate) y_max: i16,
}

/// Forwards an outline and measures the box of its points and control points.
struct Measure<'s> {
    inner: &'s mut dyn OutlineBuilder,
    bounds: Option<(f32, f32, f32, f32)>,
}

impl Measure<'_> {
    fn see(&mut self, x: f32, y: f32) {
        self.bounds = Some(match self.bounds {
            None => (x, y, x, y),
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        });
    }

    fn rect(&self) -> Option<Rect> {
        let (x_min, y_min, x_max, y_max) = self.bounds?;
        Some(Rect {
            x_min: x_min as i16,
            y_min: y_min as i16,
            x_max: x_max as i16,
            y_max: y_max as i16,
        })
    }
}

impl OutlineBuilder for Measure<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.see(x, y);
        self.inner.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.see(x, y);
        self.inner.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.see(x1, y1);
        self.see(x, y);
        self.inner.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.see(x1, y1);
        self.see(x2, y2);
        self.see(x, y);
        self.inner.curve_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.inner.close();
    }
}

/// An outline nobody looks at.
struct Discard;

impl OutlineBuilder for Discard {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {}
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn close(&mut self) {}
}

/// A font face.
pub(crate) struct Face {
    font: Font,
}

impl Face {
    /// Parses face `index` of a font file; `None` if it is not a font.
    pub(crate) fn parse(data: &[u8], index: u32) -> Option<Self> {
        let font = Font::new(data.to_vec(), index)?;
        // A font has glyphs and an em square.
        (font.num_glyphs() > 0 && font.tables().head().is_ok()).then_some(Self { font })
    }

    pub(crate) fn number_of_glyphs(&self) -> u16 {
        u16::try_from(self.font.num_glyphs()).expect("at most 65 535 glyphs")
    }

    pub(crate) fn units_per_em(&self) -> u16 {
        self.font.units_per_em()
    }

    /// The glyph the font maps a character to.
    pub(crate) fn glyph_index(&self, c: char) -> Option<GlyphId> {
        let glyph = self.font.charmap().map_unicode(c)?;
        u16::try_from(glyph.to_u32()).ok().map(GlyphId)
    }

    /// The advance width of a glyph, in font units.
    pub(crate) fn glyph_hor_advance(&self, glyph: GlyphId) -> Option<u16> {
        (u32::from(glyph.0) < self.font.num_glyphs())
            .then(|| self.font.glyph_metrics().h_advance(glyph.raw()).round() as u16)
    }

    /// The box of a glyph's outline (control points included); `None` for a glyph
    /// without an outline.
    pub(crate) fn glyph_bounding_box(&self, glyph: GlyphId) -> Option<Rect> {
        let order =
            GlyphSourceOrder::new(&[GlyphSource::Glyf, GlyphSource::Cff, GlyphSource::Cff2]);
        let outline = self.font.outline_glyphs_with(order).get(glyph.raw())?;
        let mut measure = Measure { inner: &mut Discard, bounds: None };
        outline.draw(&DrawOptions::new(), DrawMemory::Scratch, &mut measure).ok()?;
        measure.rect()
    }

    /// The font bounding box of `head`.
    pub(crate) fn global_bounding_box(&self) -> Rect {
        let bounds = self.font.metrics().bounds;
        let unit = |v: read_fonts::types::F48Dot16| v.to_f64().round() as i16;
        Rect {
            x_min: unit(bounds.x_min),
            y_min: unit(bounds.y_min),
            x_max: unit(bounds.x_max),
            y_max: unit(bounds.y_max),
        }
    }

    /// The ascender of the horizontal line.
    pub(crate) fn ascender(&self) -> i16 {
        self.font.metrics().h_line().map_or(0, |line| line.ascender.to_f64().round() as i16)
    }

    /// The descender of the horizontal line (negative).
    pub(crate) fn descender(&self) -> i16 {
        self.font.metrics().h_line().map_or(0, |line| line.descender.to_f64().round() as i16)
    }

    /// The height of capital letters, if the font states it.
    pub(crate) fn capital_height(&self) -> Option<i16> {
        self.font.metrics().cap_height.map(|height| height.to_f64().round() as i16)
    }

    /// The italic angle in degrees (`post`).
    pub(crate) fn italic_angle(&self) -> f32 {
        self.font.tables().post().map_or(0.0, |post: Post<'_>| post.italic_angle().to_f32())
    }
}

/// A bare `CFF ` table.
pub(crate) mod cff {
    use super::*;

    pub(crate) struct Table<'a> {
        font: CffFontRef<'a>,
    }

    impl<'a> Table<'a> {
        /// Parses a `CFF ` table; `None` if it is not one.
        pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
            // The table must be readable on its own, down to its glyph programs.
            read_fonts::tables::cff::Cff::read(FontData::new(data)).ok()?;
            let font = CffFontRef::new_cff(data, 0, None).ok()?;
            (font.num_glyphs() > 0).then_some(Self { font })
        }

        pub(crate) fn number_of_glyphs(&self) -> u16 {
            u16::try_from(self.font.num_glyphs()).expect("at most 65 535 glyphs")
        }

        /// The CID of a glyph of a CID-keyed font.
        pub(crate) fn glyph_cid(&self, glyph: GlyphId) -> Option<u16> {
            if !self.font.is_cid() {
                return None;
            }
            self.font.charset()?.string_id(glyph.raw()).map(|cid| cid.to_u16())
        }

        /// Draws a glyph and returns the box of its outline.
        pub(crate) fn outline(
            &self,
            glyph: GlyphId,
            builder: &mut dyn OutlineBuilder,
        ) -> Result<Rect, ()> {
            let subfont_index = self.font.subfont_index(glyph.raw()).ok_or(())?;
            let subfont = self.font.subfont(subfont_index, &[]).map_err(|_| ())?;
            let mut measure = Measure { inner: builder, bounds: None };
            self.font.draw(&subfont, glyph.raw(), &[], None, &mut measure).map_err(|_| ())?;
            measure.rect().ok_or(())
        }
    }
}
