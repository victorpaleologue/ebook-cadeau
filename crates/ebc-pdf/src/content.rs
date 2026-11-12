//! Page content streams: scene items → PDF graphics operators.
//!
//! Every page starts with one `cm` that maps the scene's coordinate system (millimetres,
//! origin at the top-left corner of the trim box, y down) onto PDF default user space
//! (points, origin at the bottom-left corner of the media box, y up). Everything after it
//! is written in page millimetres, exactly as in the scene, so group transforms can be
//! emitted unchanged. Stroke widths (given in points) are converted to millimetres, text
//! is flipped back upright by its text matrix, and images by their placement matrix.

use std::collections::{BTreeMap, BTreeSet};

use ebc_scene::{
    Color, FillRule, FontId, Glyph, ImageId, Item, MM_PER_PT, PT_PER_MM, Page, Path, PathSeg,
    PdfStandard, Point, Rect, Stroke, StrokeCap, StrokeJoin, Transform,
};
use pdf_writer::types::{LineCapStyle, LineJoinStyle};
use pdf_writer::{Content, Name, Settings, Str};

use crate::error::PdfError;
use crate::font::EmbeddedFont;

/// Deepest allowed nesting of [`Item::Group`]s. PDF consumers are only required to
/// support 28 nested `q` operators; this leaves room for the writer's own.
pub(crate) const MAX_GROUP_DEPTH: usize = 24;

/// Largest absolute coordinate (in millimetres) the writer accepts: thirty metres, far
/// beyond any sheet, and inside the range of real numbers that PDF 1.3 consumers have
/// to support (about ±32767).
const MAX_COORDINATE: f64 = 30_000.0;

/// Longest array the writer puts into a content stream. PDF 1.3 consumers only have to
/// support 8191 elements.
const MAX_ARRAY_LEN: usize = 8_000;

/// Smallest glyph displacement worth a `TJ` adjustment, in thousandths of an em:
/// 1/20000 em is 0.2 µm at 12 pt, far below what any device resolves, and large enough
/// to absorb floating-point noise so that naturally spaced text stays one string.
const MIN_ADJUSTMENT: f64 = 0.05;

/// Longest string the writer puts into a content stream, in bytes.
const MAX_STRING_LEN: usize = 16_000;

/// Resource name of the ICC-based sRGB colour space.
pub(crate) const SRGB_SPACE: &[u8] = b"sRGB";
/// Resource name of the `/Separation /All` (registration) colour space.
pub(crate) const REGISTRATION_SPACE: &[u8] = b"Registration";

/// What a page's content stream needs from the page's resource dictionary.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct PageUse {
    /// Fonts used on the page, by [`FontId`] value.
    pub(crate) fonts: BTreeSet<u32>,
    /// Images used on the page, by [`ImageId`] value.
    pub(crate) images: BTreeSet<u32>,
    /// Optional-content groups used on the page, by layer index.
    pub(crate) layers: BTreeSet<usize>,
    /// The ICC-based sRGB colour space is used.
    pub(crate) srgb: bool,
    /// The registration colour space is used.
    pub(crate) registration: bool,
}

/// Document-wide lookup tables shared by all pages.
pub(crate) struct ContentContext<'a> {
    /// The conformance level, which decides how colours are written.
    pub(crate) standard: PdfStandard,
    /// Pretty-print the operators (one per line).
    pub(crate) pretty: bool,
    /// Embedded fonts by [`FontId`] value.
    pub(crate) fonts: &'a BTreeMap<u32, EmbeddedFont>,
    /// Resource names of the images by [`ImageId`] value.
    pub(crate) images: &'a BTreeMap<u32, Vec<u8>>,
    /// Layer name → index, or `None` when layers are flattened.
    pub(crate) layers: Option<&'a BTreeMap<String, usize>>,
}

/// Resource name of the optional-content group with the given index.
pub(crate) fn layer_resource_name(index: usize) -> Vec<u8> {
    format!("OC{index}").into_bytes()
}

/// A colour resolved to the operators that set it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Paint {
    Gray(f32),
    Cmyk([f32; 4]),
    DeviceRgb([f32; 3]),
    IccRgb([f32; 3]),
    Registration,
}

/// The part of the graphics state the writer tracks to avoid redundant operators.
/// `None` means "unknown", which forces the next use to set the parameter.
#[derive(Debug, Clone, PartialEq, Default)]
struct GraphicsState {
    fill: Option<Paint>,
    stroke: Option<Paint>,
    line_width: Option<f32>,
    cap: Option<StrokeCap>,
    join: Option<StrokeJoin>,
    dash: Option<Vec<f32>>,
}

/// One element of a `TJ` array.
enum TextItem {
    Show(Vec<u8>),
    Adjust(f32),
}

/// Builds the content stream of one page.
pub(crate) struct PageContent<'a> {
    ctx: &'a ContentContext<'a>,
    page_number: usize,
    content: Content,
    used: PageUse,
    state: GraphicsState,
    saved: Vec<GraphicsState>,
}

impl<'a> PageContent<'a> {
    /// Renders `page` and returns the content stream bytes and the resources it uses.
    pub(crate) fn render(
        ctx: &'a ContentContext<'a>,
        page: &Page,
        page_number: usize,
    ) -> Result<(Vec<u8>, PageUse), PdfError> {
        let mut this = Self {
            ctx,
            page_number,
            content: Content::with_settings(Settings { pretty: ctx.pretty }),
            used: PageUse::default(),
            state: GraphicsState::default(),
            saved: Vec::new(),
        };
        let margin = page.bleed + page.slug;
        // (x, y) mm, y down from the trim's top-left  →  points, y up from the media box's
        // bottom-left corner.
        this.content.transform([
            PT_PER_MM as f32,
            0.0,
            0.0,
            -(PT_PER_MM as f32),
            round_pt(margin * PT_PER_MM),
            round_pt((page.trim.h + margin) * PT_PER_MM),
        ]);
        for item in &page.items {
            this.item(item, 0)?;
        }
        Ok((this.content.finish().into_vec(), this.used))
    }

    fn invalid(&self, reason: &'static str) -> PdfError {
        PdfError::InvalidContent { page: self.page_number, reason }
    }

    /// A length or coordinate in millimetres, checked and rounded to 10 nm.
    fn coord(&self, value: f64) -> Result<f32, PdfError> {
        if !value.is_finite() {
            return Err(self.invalid("a coordinate is not a finite number"));
        }
        if value.abs() > MAX_COORDINATE {
            return Err(self.invalid("a coordinate is out of range"));
        }
        Ok(((value * 1.0e5).round() / 1.0e5) as f32)
    }

    /// A dimensionless matrix coefficient, checked and rounded so that values like
    /// `cos(90°) = 6e-17` become an exact zero.
    fn coefficient(&self, value: f64) -> Result<f32, PdfError> {
        if !value.is_finite() {
            return Err(self.invalid("a transform is not finite"));
        }
        if value.abs() > MAX_COORDINATE {
            return Err(self.invalid("a transform is out of range"));
        }
        Ok(((value * 1.0e8).round() / 1.0e8) as f32)
    }

    fn save(&mut self) {
        self.content.save_state();
        self.saved.push(self.state.clone());
    }

    fn restore(&mut self) {
        self.content.restore_state();
        if let Some(state) = self.saved.pop() {
            self.state = state;
        }
    }

    fn item(&mut self, item: &Item, depth: usize) -> Result<(), PdfError> {
        match item {
            Item::Path { path, fill, stroke } => self.path(path, fill.as_ref(), stroke.as_ref()),
            Item::Image { image, rect } => self.image(*image, *rect),
            Item::Glyphs { font, size_pt, color, origin, glyphs, .. } => {
                self.glyphs(*font, *size_pt, *color, *origin, glyphs)
            }
            Item::Group { transform, clip, layer, items } => {
                self.group(transform, clip.as_ref(), layer.as_deref(), items, depth)
            }
        }
    }

    // ----- colour ---------------------------------------------------------------------

    fn resolve(&mut self, color: Color) -> Result<Paint, PdfError> {
        let unit = |v: f32| {
            let v = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
            (v * 1.0e4).round() / 1.0e4
        };
        Ok(match color {
            Color::Cmyk { c, m, y, k } => Paint::Cmyk([unit(c), unit(m), unit(y), unit(k)]),
            Color::Gray(v) => Paint::Gray(unit(v)),
            Color::Registration => {
                self.used.registration = true;
                Paint::Registration
            }
            Color::Rgb { r, g, b } => {
                let rgb = [unit(r), unit(g), unit(b)];
                match self.ctx.standard {
                    PdfStandard::Plain => Paint::DeviceRgb(rgb),
                    PdfStandard::X3_2002 | PdfStandard::X4 => {
                        self.used.srgb = true;
                        Paint::IccRgb(rgb)
                    }
                    PdfStandard::X1a2001 => {
                        return Err(PdfError::RgbNotAllowed {
                            standard: self.ctx.standard,
                            location: format!("a paint colour on page {}", self.page_number),
                        });
                    }
                }
            }
        })
    }

    fn set_fill(&mut self, color: Color) -> Result<(), PdfError> {
        let paint = self.resolve(color)?;
        if self.state.fill == Some(paint) {
            return Ok(());
        }
        match paint {
            Paint::Gray(v) => {
                self.content.set_fill_gray(v);
            }
            Paint::Cmyk([c, m, y, k]) => {
                self.content.set_fill_cmyk(c, m, y, k);
            }
            Paint::DeviceRgb([r, g, b]) => {
                self.content.set_fill_rgb(r, g, b);
            }
            Paint::IccRgb(rgb) => {
                self.content.set_fill_color_space(Name(SRGB_SPACE));
                self.content.set_fill_color(rgb);
            }
            Paint::Registration => {
                self.content.set_fill_color_space(Name(REGISTRATION_SPACE));
                self.content.set_fill_color([1.0]);
            }
        }
        self.state.fill = Some(paint);
        Ok(())
    }

    fn set_stroke(&mut self, stroke: &Stroke) -> Result<(), PdfError> {
        if !stroke.width_pt.is_finite() || stroke.width_pt < 0.0 {
            return Err(self.invalid("a stroke width is negative or not finite"));
        }
        if stroke.dash_mm.iter().any(|d| !d.is_finite() || *d < 0.0) {
            return Err(self.invalid("a dash length is negative or not finite"));
        }
        if !stroke.dash_mm.is_empty() && stroke.dash_mm.iter().all(|d| *d == 0.0) {
            return Err(self.invalid("a dash pattern has only zero lengths"));
        }
        if stroke.dash_mm.len() > MAX_ARRAY_LEN {
            return Err(self.invalid("a dash pattern has too many entries"));
        }

        let paint = self.resolve(stroke.color)?;
        if self.state.stroke != Some(paint) {
            match paint {
                Paint::Gray(v) => {
                    self.content.set_stroke_gray(v);
                }
                Paint::Cmyk([c, m, y, k]) => {
                    self.content.set_stroke_cmyk(c, m, y, k);
                }
                Paint::DeviceRgb([r, g, b]) => {
                    self.content.set_stroke_rgb(r, g, b);
                }
                Paint::IccRgb(rgb) => {
                    self.content.set_stroke_color_space(Name(SRGB_SPACE));
                    self.content.set_stroke_color(rgb);
                }
                Paint::Registration => {
                    self.content.set_stroke_color_space(Name(REGISTRATION_SPACE));
                    self.content.set_stroke_color([1.0]);
                }
            }
            self.state.stroke = Some(paint);
        }

        // The page is drawn in millimetres, the width is given in points.
        let width = self.coord(stroke.width_pt * MM_PER_PT)?;
        if self.state.line_width != Some(width) {
            self.content.set_line_width(width);
            self.state.line_width = Some(width);
        }
        if self.state.cap != Some(stroke.cap) {
            self.content.set_line_cap(match stroke.cap {
                StrokeCap::Butt => LineCapStyle::ButtCap,
                StrokeCap::Round => LineCapStyle::RoundCap,
                StrokeCap::Square => LineCapStyle::ProjectingSquareCap,
            });
            self.state.cap = Some(stroke.cap);
        }
        if self.state.join != Some(stroke.join) {
            self.content.set_line_join(match stroke.join {
                StrokeJoin::Miter => LineJoinStyle::MiterJoin,
                StrokeJoin::Round => LineJoinStyle::RoundJoin,
                StrokeJoin::Bevel => LineJoinStyle::BevelJoin,
            });
            self.state.join = Some(stroke.join);
        }
        let mut dash = Vec::with_capacity(stroke.dash_mm.len());
        for d in &stroke.dash_mm {
            dash.push(self.coord(*d)?);
        }
        if self.state.dash.as_ref() != Some(&dash) {
            self.content.set_dash_pattern(dash.iter().copied(), 0.0);
            self.state.dash = Some(dash);
        }
        Ok(())
    }

    // ----- paths ----------------------------------------------------------------------

    /// Writes the path construction operators. Returns `false` (and writes nothing) for
    /// a path without segments.
    fn construct_path(&mut self, path: &Path) -> Result<bool, PdfError> {
        if path.segs.is_empty() {
            return Ok(false);
        }
        // Validate and convert first so that nothing is written for an invalid path.
        enum Op {
            Move(f32, f32),
            Line(f32, f32),
            Cubic([f32; 6]),
            Close,
        }
        let mut ops = Vec::with_capacity(path.segs.len());
        let mut current: Option<Point> = None;
        let mut start = Point::default();
        let no_start = "a path draws before its first MoveTo";
        for seg in &path.segs {
            match *seg {
                PathSeg::MoveTo(p) => {
                    ops.push(Op::Move(self.coord(p.x)?, self.coord(p.y)?));
                    current = Some(p);
                    start = p;
                }
                PathSeg::LineTo(p) => {
                    if current.is_none() {
                        return Err(self.invalid(no_start));
                    }
                    ops.push(Op::Line(self.coord(p.x)?, self.coord(p.y)?));
                    current = Some(p);
                }
                PathSeg::QuadTo(c, p) => {
                    let Some(p0) = current else { return Err(self.invalid(no_start)) };
                    // Degree elevation: a quadratic is the cubic with these controls.
                    let c1 = Point::new(
                        p0.x + (c.x - p0.x) * 2.0 / 3.0,
                        p0.y + (c.y - p0.y) * 2.0 / 3.0,
                    );
                    let c2 =
                        Point::new(p.x + (c.x - p.x) * 2.0 / 3.0, p.y + (c.y - p.y) * 2.0 / 3.0);
                    ops.push(Op::Cubic([
                        self.coord(c1.x)?,
                        self.coord(c1.y)?,
                        self.coord(c2.x)?,
                        self.coord(c2.y)?,
                        self.coord(p.x)?,
                        self.coord(p.y)?,
                    ]));
                    current = Some(p);
                }
                PathSeg::CubicTo(c1, c2, p) => {
                    if current.is_none() {
                        return Err(self.invalid(no_start));
                    }
                    ops.push(Op::Cubic([
                        self.coord(c1.x)?,
                        self.coord(c1.y)?,
                        self.coord(c2.x)?,
                        self.coord(c2.y)?,
                        self.coord(p.x)?,
                        self.coord(p.y)?,
                    ]));
                    current = Some(p);
                }
                PathSeg::Close => {
                    if current.is_none() {
                        return Err(self.invalid(no_start));
                    }
                    ops.push(Op::Close);
                    current = Some(start);
                }
            }
        }
        for op in ops {
            match op {
                Op::Move(x, y) => self.content.move_to(x, y),
                Op::Line(x, y) => self.content.line_to(x, y),
                Op::Cubic([x1, y1, x2, y2, x3, y3]) => {
                    self.content.cubic_to(x1, y1, x2, y2, x3, y3)
                }
                Op::Close => self.content.close_path(),
            };
        }
        Ok(true)
    }

    fn path(
        &mut self,
        path: &Path,
        fill: Option<&(Color, FillRule)>,
        stroke: Option<&Stroke>,
    ) -> Result<(), PdfError> {
        if path.segs.is_empty() || (fill.is_none() && stroke.is_none()) {
            return Ok(());
        }
        // Graphics state operators must precede path construction.
        if let Some((color, _)) = fill {
            self.set_fill(*color)?;
        }
        if let Some(stroke) = stroke {
            self.set_stroke(stroke)?;
        }
        if !self.construct_path(path)? {
            return Ok(());
        }
        match (fill.map(|(_, rule)| *rule), stroke.is_some()) {
            (Some(FillRule::NonZero), false) => self.content.fill_nonzero(),
            (Some(FillRule::EvenOdd), false) => self.content.fill_even_odd(),
            (Some(FillRule::NonZero), true) => self.content.fill_nonzero_and_stroke(),
            (Some(FillRule::EvenOdd), true) => self.content.fill_even_odd_and_stroke(),
            (None, _) => self.content.stroke(),
        };
        Ok(())
    }

    // ----- images ---------------------------------------------------------------------

    fn image(&mut self, image: ImageId, rect: Rect) -> Result<(), PdfError> {
        let Some(name) = self.ctx.images.get(&image.0) else {
            return Err(PdfError::UnknownImage(image));
        };
        let x = self.coord(rect.x)?;
        let (w, h) = (self.coord(rect.w)?, self.coord(rect.h)?);
        let bottom = self.coord(rect.y + rect.h)?;
        if w <= 0.0 || h <= 0.0 {
            // Nothing to paint; a degenerate matrix would only upset PDF consumers.
            return Ok(());
        }
        self.used.images.insert(image.0);
        self.save();
        // Image space is the unit square with the first sample row at the top (y = 1);
        // the page's y axis points down, hence the negative height.
        self.content.transform([w, 0.0, 0.0, -h, x, bottom]);
        self.content.x_object(Name(name));
        self.restore();
        Ok(())
    }

    // ----- text -----------------------------------------------------------------------

    fn glyphs(
        &mut self,
        font_id: FontId,
        size_pt: f64,
        color: Color,
        origin: Point,
        glyphs: &[Glyph],
    ) -> Result<(), PdfError> {
        if glyphs.is_empty() {
            return Ok(());
        }
        let ctx = self.ctx;
        let Some(font) = ctx.fonts.get(&font_id.0) else {
            return Err(PdfError::UnknownFont(font_id));
        };
        if !size_pt.is_finite() || size_pt <= 0.0 {
            return Err(self.invalid("a font size is not a positive finite number"));
        }
        // The page is drawn in millimetres, so is the font size. It is deliberately not
        // rounded: advances are multiples of it, and a rounding error would add up along
        // a line.
        self.coord(size_pt * MM_PER_PT)?;
        let size = (size_pt * MM_PER_PT) as f32;
        if size <= 0.0 {
            return Err(self.invalid("a font size is too small"));
        }
        let size_mm = f64::from(size);

        // Split the run into segments of glyphs on one baseline. Within a segment glyphs
        // are positioned with TJ adjustments relative to the advance widths declared in
        // the PDF, so every glyph lands on its requested x (to 1/20000 em, see
        // `MIN_ADJUSTMENT`) and errors never accumulate.
        let mut segments: Vec<(f32, f32, Vec<TextItem>)> = Vec::new();
        let mut index = 0;
        while index < glyphs.len() {
            let first = glyphs[index];
            let start_x = self.coord(origin.x + first.x)?;
            let start_y = self.coord(origin.y + first.y)?;
            let mut pen = f64::from(start_x);
            let mut items = Vec::new();
            let mut pending: Vec<u8> = Vec::new();

            // Long runs are split so that no array or string exceeds what old consumers
            // are required to read.
            while index < glyphs.len() && items.len() < MAX_ARRAY_LEN {
                let glyph = glyphs[index];
                if self.coord(origin.y + glyph.y)? != start_y {
                    break;
                }
                // Checked for range, but kept at full precision: adjustments are relative,
                // so they are not limited by the precision of absolute coordinates.
                self.coord(origin.x + glyph.x)?;
                let wanted = origin.x + glyph.x;
                // A positive TJ number moves the next glyph to the left.
                let adjust = ((pen - wanted) / size_mm * 1000.0 * 1.0e3).round() / 1.0e3;
                if adjust.abs() >= MIN_ADJUSTMENT {
                    if adjust.abs() > 20_000.0 {
                        // A jump of more than 20 em: start a new segment instead of
                        // writing a huge adjustment. (Never the segment's first glyph,
                        // whose adjustment is zero by construction.)
                        break;
                    }
                    let adjust = adjust as f32;
                    if !pending.is_empty() {
                        items.push(TextItem::Show(std::mem::take(&mut pending)));
                    }
                    items.push(TextItem::Adjust(adjust));
                    pen -= f64::from(adjust) / 1000.0 * size_mm;
                }
                let Some(cid) = font.cid(glyph.id) else {
                    return Err(self.invalid("internal error: a glyph is missing from its font"));
                };
                pending.extend_from_slice(&cid.to_be_bytes());
                if pending.len() >= MAX_STRING_LEN {
                    items.push(TextItem::Show(std::mem::take(&mut pending)));
                }
                pen += f64::from(font.width(cid)) / 1000.0 * size_mm;
                index += 1;
            }
            if !pending.is_empty() {
                items.push(TextItem::Show(pending));
            }
            segments.push((start_x, start_y, items));
        }

        self.used.fonts.insert(font_id.0);
        self.set_fill(color)?;
        self.content.begin_text();
        self.content.set_font(font.resource_name(), size);
        for (x, y, items) in &segments {
            // Text space has y up; the page has y down.
            self.content.set_text_matrix([1.0, 0.0, 0.0, -1.0, *x, *y]);
            let mut show = self.content.show_positioned();
            let mut array = show.items();
            for item in items {
                match item {
                    TextItem::Show(bytes) => array.show(Str(bytes)),
                    TextItem::Adjust(amount) => array.adjust(*amount),
                };
            }
        }
        self.content.end_text();
        Ok(())
    }

    // ----- groups ---------------------------------------------------------------------

    fn group(
        &mut self,
        transform: &Transform,
        clip: Option<&Path>,
        layer: Option<&str>,
        items: &[Item],
        depth: usize,
    ) -> Result<(), PdfError> {
        if depth >= MAX_GROUP_DEPTH {
            return Err(PdfError::NestingTooDeep { page: self.page_number, max: MAX_GROUP_DEPTH });
        }
        let matrix = [
            self.coefficient(transform.a)?,
            self.coefficient(transform.b)?,
            self.coefficient(transform.c)?,
            self.coefficient(transform.d)?,
            self.coord(transform.e)?,
            self.coord(transform.f)?,
        ];
        let determinant = f64::from(matrix[0]) * f64::from(matrix[3])
            - f64::from(matrix[1]) * f64::from(matrix[2]);
        // A collapsed coordinate system, an empty clip or no children: nothing is visible.
        if items.is_empty()
            || determinant.abs() < 1.0e-12
            || clip.is_some_and(|c| c.segs.is_empty())
        {
            return Ok(());
        }

        let layer_index = match (layer, self.ctx.layers) {
            (Some(name), Some(layers)) => layers.get(name).copied(),
            _ => None,
        };
        if let Some(index) = layer_index {
            self.used.layers.insert(index);
            self.content
                .begin_marked_content_with_properties(Name(b"OC"))
                .properties_named(Name(&layer_resource_name(index)));
        }

        self.save();
        if matrix != [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] {
            self.content.transform(matrix);
        }
        if let Some(clip) = clip
            && self.construct_path(clip)?
        {
            self.content.clip_nonzero();
            self.content.end_path();
        }
        for item in items {
            self.item(item, depth + 1)?;
        }
        self.restore();

        if layer_index.is_some() {
            self.content.end_marked_content();
        }
        Ok(())
    }
}

/// Rounds a length in points to 1/10000 pt.
pub(crate) fn round_pt(value: f64) -> f32 {
    ((value * 1.0e4).round() / 1.0e4) as f32
}
