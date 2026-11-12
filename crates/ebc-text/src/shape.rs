//! Shaping: text → positioned glyphs in millimetres, glyph items and vector outlines.

use ebc_scene::{Color, FontId, Glyph, Item, Path, PathSeg, Point, Rect};
use ttf_parser::GlyphId;

use crate::font::{Font, mm_per_unit};
use crate::guard::contain;

/// Largest font size accepted by [`shape`], in points. Larger (or non-finite) sizes are
/// clamped so that every computed coordinate stays a finite number.
pub const MAX_SIZE_PT: f64 = 1.0e6;

/// Options for [`shape`] and everything built on it.
///
/// ```
/// use ebc_text::ShapeOptions;
/// let spaced_caps = ShapeOptions { letter_spacing_em: 0.08, ..ShapeOptions::default() };
/// # let _ = spaced_caps;
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShapeOptions {
    /// Extra space inserted **between** consecutive clusters (letters), as a fraction of
    /// the font size; negative values tighten. Nothing is added after the last letter.
    /// A non-zero value turns the standard ligatures (`liga`, `clig`) off, as browsers
    /// do, unless `features` sets them explicitly.
    pub letter_spacing_em: f32,
    /// BCP 47 language of the text (e.g. `"fr"`, `"tr"`), selecting language-specific
    /// glyph forms where the font has them.
    pub language: Option<String>,
    /// Lay the run out right-to-left. The whole run has one direction: there is no
    /// bidirectional reordering.
    pub rtl: bool,
    /// OpenType features as `(tag, value)`, e.g. `("smcp", 1)`, `("liga", 0)`.
    pub features: Vec<(String, u32)>,
}

/// One positioned glyph of a [`ShapedText`].
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedGlyph {
    /// Glyph index in the font (0 = `.notdef`: the font lacks the character).
    pub id: u16,
    /// Horizontal position of the glyph origin relative to the run origin, in mm.
    pub x: f64,
    /// Vertical position of the glyph origin relative to the baseline, in mm
    /// (positive = lower on the page).
    pub y: f64,
    /// How far the pen moves after this glyph, in mm (letter spacing included).
    pub advance: f64,
    /// The source text this glyph stands for. The first glyph of a cluster carries the
    /// text of the whole cluster (all letters of a ligature, a base letter with its
    /// combining marks); the other glyphs of the cluster carry an empty string.
    pub text: String,
}

/// A run of text shaped with one font at one size, on one baseline.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShapedText {
    /// The glyphs in visual (left-to-right) order.
    pub glyphs: Vec<ShapedGlyph>,
    /// Advance width of the run in mm: the sum of all glyph advances.
    pub width: f64,
    /// The font size the run was shaped at, in points.
    pub size_pt: f64,
}

/// Shapes `text` with `font` at `size_pt` points: applies the font's substitutions
/// (ligatures, localised forms), kerning and mark positioning, and returns the glyphs
/// positioned in millimetres relative to the start of the baseline.
///
/// * Characters the font lacks come out as glyph 0 (`.notdef`); check
///   [`Font::missing_chars`] first if that matters.
/// * Control characters (line feeds, tabs, …) and the Unicode line/paragraph separators
///   are set as spaces: a run is a single line. Use [`wrap`](crate::wrap) to break lines.
/// * `size_pt` is clamped to `0.0..=`[`MAX_SIZE_PT`]; NaN counts as 0.
/// * The concatenation of all [`ShapedGlyph::text`] values, taken in logical order, is
///   exactly `text`.
///
/// Shaping is resolution independent (no hinting), so positions and the width are
/// strictly proportional to `size_pt`.
#[must_use]
pub fn shape(font: &Font, text: &str, size_pt: f64, opts: &ShapeOptions) -> ShapedText {
    let size_pt = sane_size(size_pt);
    contain(|| Some(shape_units(&font.hb_face()?, text, opts).to_shaped(text, size_pt)))
        .flatten()
        .unwrap_or(ShapedText { glyphs: Vec::new(), width: 0.0, size_pt })
}

impl ShapedText {
    /// The run as an [`Item::Glyphs`] (real text set in an embedded font), with
    /// `glyph_text` filled for the PDF `ToUnicode` map.
    ///
    /// `font` must be the [`FontId`] under which the *same font file and face* was
    /// registered in the document. For an instance of a variable font use
    /// [`to_outline`](Self::to_outline) instead: the glyph ids refer to the font file,
    /// whose default instance a PDF viewer would draw.
    #[must_use]
    pub fn to_glyph_item(&self, font: FontId, color: Color, origin: Point) -> Item {
        Item::Glyphs {
            font,
            size_pt: self.size_pt,
            color,
            origin,
            glyphs: self.glyphs.iter().map(|g| Glyph { id: g.id, x: g.x, y: g.y }).collect(),
            glyph_text: self.glyphs.iter().map(|g| g.text.clone()).collect(),
        }
    }

    /// The run as filled vector outlines in page millimetres (y down), with the start of
    /// the baseline at `origin`. Fill it with the non-zero winding rule.
    ///
    /// Quadratic (TrueType) and cubic (CFF) curves are kept as they are in the font.
    /// `font` must be the font (and variation instance) the run was shaped with. Glyphs
    /// without a scalable outline (spaces, bitmap or colour-only glyphs) add nothing.
    #[must_use]
    pub fn to_outline(&self, font: &Font, origin: Point) -> Path {
        contain(|| {
            let mut path = Path::new();
            if let Some(face) = font.tt_face() {
                let scale = mm_per_unit(face.units_per_em(), self.size_pt);
                let placed = self.glyphs.iter().map(|g| (g.id, origin.x + g.x, origin.y + g.y));
                append_outlines(&face, placed, scale, &mut path);
            }
            path
        })
        .unwrap_or_default()
    }

    /// The exact bounding box of the ink of the run in page millimetres, for a baseline
    /// starting at `origin`; `None` if nothing is drawn (empty or blank text).
    ///
    /// Unlike [`width`](Self::width), this excludes side bearings and includes
    /// overhangs, so it is what to use for optical centring.
    #[must_use]
    pub fn ink_bounds(&self, font: &Font, origin: Point) -> Option<Rect> {
        outline_bounds(&self.to_outline(font, origin))
    }
}

/// The exact bounding box of a path: unlike [`Path::control_bounds`], Bézier control
/// points that lie outside the curve do not enlarge it. `None` for a path without
/// segments (or whose coordinates are not finite).
#[must_use]
pub fn outline_bounds(path: &Path) -> Option<Rect> {
    let mut bounds = Bounds::default();
    let mut start = Point::default();
    let mut current = Point::default();
    for seg in &path.segs {
        match *seg {
            PathSeg::MoveTo(p) => {
                bounds.add(p);
                start = p;
                current = p;
            }
            PathSeg::LineTo(p) => {
                bounds.add(p);
                current = p;
            }
            PathSeg::QuadTo(c, p) => {
                bounds.add(p);
                for t in [quad_extremum(current.x, c.x, p.x), quad_extremum(current.y, c.y, p.y)]
                    .into_iter()
                    .flatten()
                {
                    let u = 1.0 - t;
                    bounds.add(Point::new(
                        u * u * current.x + 2.0 * u * t * c.x + t * t * p.x,
                        u * u * current.y + 2.0 * u * t * c.y + t * t * p.y,
                    ));
                }
                current = p;
            }
            PathSeg::CubicTo(c1, c2, p) => {
                bounds.add(p);
                let roots = cubic_extrema(current.x, c1.x, c2.x, p.x)
                    .into_iter()
                    .chain(cubic_extrema(current.y, c1.y, c2.y, p.y));
                for t in roots.flatten() {
                    let u = 1.0 - t;
                    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    bounds.add(Point::new(
                        b0 * current.x + b1 * c1.x + b2 * c2.x + b3 * p.x,
                        b0 * current.y + b1 * c1.y + b2 * c2.y + b3 * p.y,
                    ));
                }
                current = p;
            }
            PathSeg::Close => current = start,
        }
    }
    bounds.rect()
}

#[derive(Default)]
struct Bounds {
    min_max: Option<(f64, f64, f64, f64)>,
    poisoned: bool,
}

impl Bounds {
    fn add(&mut self, p: Point) {
        if !(p.x.is_finite() && p.y.is_finite()) {
            self.poisoned = true;
            return;
        }
        self.min_max = Some(match self.min_max {
            None => (p.x, p.y, p.x, p.y),
            Some((x0, y0, x1, y1)) => (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y)),
        });
    }

    fn rect(&self) -> Option<Rect> {
        if self.poisoned {
            return None;
        }
        self.min_max.map(|(x0, y0, x1, y1)| Rect::new(x0, y0, x1 - x0, y1 - y0))
    }
}

/// Parameter of the interior extremum of one coordinate of a quadratic Bézier, if any.
fn quad_extremum(p0: f64, p1: f64, p2: f64) -> Option<f64> {
    let denom = p0 - 2.0 * p1 + p2;
    if denom.abs() < f64::EPSILON {
        return None;
    }
    let t = (p0 - p1) / denom;
    (t > 0.0 && t < 1.0).then_some(t)
}

/// Parameters of the interior extrema of one coordinate of a cubic Bézier (at most two).
fn cubic_extrema(p0: f64, p1: f64, p2: f64, p3: f64) -> [Option<f64>; 2] {
    // Derivative / 3 = a t^2 + b t + c.
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let c = p1 - p0;
    let inside = |t: f64| (t > 0.0 && t < 1.0).then_some(t);
    if a.abs() < 1e-12 {
        if b.abs() < 1e-12 {
            return [None, None];
        }
        return [inside(-c / b), None];
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return [None, None];
    }
    let root = disc.sqrt();
    [inside((-b + root) / (2.0 * a)), inside((-b - root) / (2.0 * a))]
}

/// Clamps a font size to the supported range (NaN and negative sizes become 0).
pub(crate) fn sane_size(size_pt: f64) -> f64 {
    if size_pt.is_nan() { 0.0 } else { size_pt.clamp(0.0, MAX_SIZE_PT) }
}

/// A glyph of a [`UnitRun`], in font units (y up).
#[derive(Debug, Clone, Copy)]
pub(crate) struct UnitGlyph {
    pub(crate) id: u16,
    /// Byte offset in the source text of the cluster this glyph belongs to.
    pub(crate) cluster: u32,
    /// Pen position before the glyph (letter spacing of earlier clusters included).
    pub(crate) pen: f64,
    /// Advance, including the letter spacing that follows the glyph's cluster.
    pub(crate) advance: f64,
    pub(crate) dx: f64,
    pub(crate) dy: f64,
}

/// The size-independent result of shaping: everything in font units.
#[derive(Debug, Clone)]
pub(crate) struct UnitRun {
    pub(crate) glyphs: Vec<UnitGlyph>,
    /// Total advance in font units.
    pub(crate) width: f64,
    pub(crate) units_per_em: u16,
}

impl UnitRun {
    /// Advance width in millimetres at `size_pt` — the very expression [`shape`] uses
    /// for [`ShapedText::width`], so that fitting and shaping always agree.
    pub(crate) fn width_mm(&self, size_pt: f64) -> f64 {
        self.width * mm_per_unit(self.units_per_em, size_pt)
    }

    fn to_shaped(&self, text: &str, size_pt: f64) -> ShapedText {
        let scale = mm_per_unit(self.units_per_em, size_pt);
        let clusters: Vec<u32> = self.glyphs.iter().map(|g| g.cluster).collect();
        let texts = cluster_texts(text, &clusters);
        let glyphs = self
            .glyphs
            .iter()
            .zip(texts)
            .map(|(g, text)| ShapedGlyph {
                id: g.id,
                x: (g.pen + g.dx) * scale,
                // Font y points up, page y points down. `+ 0.0` normalises -0.0.
                y: -g.dy * scale + 0.0,
                advance: g.advance * scale,
                text,
            })
            .collect();
        ShapedText { glyphs, width: self.width_mm(size_pt), size_pt }
    }
}

/// `true` for the characters that are set as a space: control characters and the
/// line/paragraph separators (a run is one line).
pub(crate) fn is_set_as_space(c: char) -> bool {
    c.is_control() || c == '\u{2028}' || c == '\u{2029}'
}

/// The character actually handed to the shaper.
fn shaping_char(c: char) -> char {
    if is_set_as_space(c) { ' ' } else { c }
}

fn features(opts: &ShapeOptions) -> Vec<rustybuzz::Feature> {
    let mut features: Vec<rustybuzz::Feature> = Vec::with_capacity(opts.features.len() + 2);
    if opts.letter_spacing_em != 0.0 && opts.letter_spacing_em.is_finite() {
        // Letter-spaced text must not fuse letters. User features come later and win.
        for tag in [b"liga", b"clig"] {
            features.push(rustybuzz::Feature::new(ttf_parser::Tag::from_bytes(tag), 0, ..));
        }
    }
    for (name, value) in &opts.features {
        let bytes = name.as_bytes();
        if bytes.is_empty() || bytes.len() > 4 || !bytes.iter().all(|b| (0x20..0x7F).contains(b)) {
            continue;
        }
        features.push(rustybuzz::Feature::new(
            ttf_parser::Tag::from_bytes_lossy(bytes),
            *value,
            ..,
        ));
    }
    features
}

fn language(opts: &ShapeOptions) -> Option<rustybuzz::Language> {
    let tag = opts.language.as_deref()?.trim();
    let well_formed = !tag.is_empty()
        && tag.len() <= 35
        && tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if !well_formed {
        return None;
    }
    tag.replace('_', "-").parse().ok()
}

/// Shapes `text` once, in font units. All measuring functions go through this.
pub(crate) fn shape_units(face: &rustybuzz::Face<'_>, text: &str, opts: &ShapeOptions) -> UnitRun {
    let units_per_em = face.as_ref().units_per_em();
    let mut run = UnitRun { glyphs: Vec::new(), width: 0.0, units_per_em };
    if text.is_empty() {
        return run;
    }

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    for (offset, c) in text.char_indices() {
        buffer.add(shaping_char(c), u32::try_from(offset).unwrap_or(u32::MAX));
    }
    buffer.set_direction(if opts.rtl {
        rustybuzz::Direction::RightToLeft
    } else {
        rustybuzz::Direction::LeftToRight
    });
    if let Some(language) = language(opts) {
        buffer.set_language(language);
    }
    buffer
        .set_flags(rustybuzz::BufferFlags::BEGINNING_OF_TEXT | rustybuzz::BufferFlags::END_OF_TEXT);

    let shaped = rustybuzz::shape(face, &features(opts), buffer);
    let infos = shaped.glyph_infos();
    let positions = shaped.glyph_positions();

    let spacing = if opts.letter_spacing_em.is_finite() {
        f64::from(opts.letter_spacing_em) * f64::from(units_per_em)
    } else {
        0.0
    };

    let count = infos.len().min(positions.len());
    run.glyphs.reserve(count);
    let mut pen = 0.0;
    for i in 0..count {
        let (info, pos) = (&infos[i], &positions[i]);
        let ends_cluster = i + 1 == count || infos[i + 1].cluster != info.cluster;
        let gap = if ends_cluster && i + 1 < count { spacing } else { 0.0 };
        let advance = f64::from(pos.x_advance) + gap;
        run.glyphs.push(UnitGlyph {
            id: u16::try_from(info.glyph_id).unwrap_or(0),
            cluster: info.cluster,
            pen,
            advance,
            dx: f64::from(pos.x_offset),
            dy: f64::from(pos.y_offset),
        });
        pen += advance;
    }
    run.width = pen;
    run
}

/// For every glyph, the slice of `text` it stands for (see [`ShapedGlyph::text`]).
///
/// `clusters[i]` is the byte offset at which glyph `i`'s cluster starts. Each cluster
/// owns the text up to the start of the next one; the first glyph met for a cluster gets
/// that text. The lowest cluster also owns any text before it, so nothing is ever lost.
fn cluster_texts(text: &str, clusters: &[u32]) -> Vec<String> {
    let mut starts: Vec<u32> = clusters.to_vec();
    starts.sort_unstable();
    starts.dedup();
    let mut emitted = vec![false; starts.len()];

    clusters
        .iter()
        .map(|cluster| {
            let Ok(index) = starts.binary_search(cluster) else { return String::new() };
            if std::mem::replace(&mut emitted[index], true) {
                return String::new();
            }
            let begin = if index == 0 { 0 } else { *cluster as usize };
            let end = starts.get(index + 1).map_or(text.len(), |&next| next as usize);
            text.get(begin..end).unwrap_or_default().to_owned()
        })
        .collect()
}

/// `true` if shaping `c` with this face draws something other than `.notdef`.
pub(crate) fn is_drawable(face: &rustybuzz::Face<'_>, c: char) -> bool {
    if is_set_as_space(c) {
        return true; // never reported as missing
    }
    if face.as_ref().glyph_index(c).is_some_and(|g| g.0 != 0) {
        return true;
    }
    // Not mapped directly: let the shaper try its fallbacks (decomposition, space and
    // hyphen variants, hiding of invisible format characters).
    let mut utf8 = [0u8; 4];
    let run = shape_units(face, c.encode_utf8(&mut utf8), &ShapeOptions::default());
    run.glyphs.iter().all(|g| g.id != 0)
}

/// Appends the outlines of `glyphs` — `(glyph id, x, y)` with the glyph origin already in
/// output coordinates (y down) — to `path`, scaling font units by `scale`.
pub(crate) fn append_outlines(
    face: &ttf_parser::Face<'_>,
    glyphs: impl Iterator<Item = (u16, f64, f64)>,
    scale: f64,
    path: &mut Path,
) {
    for (id, x, y) in glyphs {
        let keep = path.segs.len();
        let mut sink = OutlineSink { path, scale, x, y };
        if face.outline_glyph(GlyphId(id), &mut sink).is_none() {
            // No outline, or a malformed one: drop whatever was emitted for this glyph.
            path.segs.truncate(keep);
        }
    }
}

/// Receives a glyph outline in font units (y up) and appends it to a [`Path`] in output
/// units (y down).
struct OutlineSink<'p> {
    path: &'p mut Path,
    scale: f64,
    x: f64,
    y: f64,
}

impl OutlineSink<'_> {
    fn map(&self, x: f32, y: f32) -> (f64, f64) {
        (self.x + f64::from(x) * self.scale, self.y - f64::from(y) * self.scale)
    }
}

impl ttf_parser::OutlineBuilder for OutlineSink<'_> {
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

#[cfg(test)]
mod tests {
    use ttf_parser::OutlineBuilder;

    use super::*;

    #[test]
    fn cluster_texts_cover_the_whole_string() {
        // "ffi" ligature as one cluster of three letters, then a decomposed "é".
        let text = "ffie\u{301}!";
        let clusters = [0, 3, 3, 6];
        assert_eq!(cluster_texts(text, &clusters), ["ffi", "e\u{301}", "", "!"]);
    }

    #[test]
    fn cluster_texts_handle_right_to_left_order() {
        // Visual order of an RTL run: clusters descend.
        assert_eq!(cluster_texts("abc", &[2, 1, 0]), ["c", "b", "a"]);
        // A ligature of the first two letters, drawn last.
        assert_eq!(cluster_texts("abc", &[2, 0, 0]), ["c", "ab", ""]);
    }

    #[test]
    fn cluster_texts_never_lose_leading_text_or_panic_on_bad_offsets() {
        // The lowest cluster does not start at 0: the leading text is attached to it.
        assert_eq!(cluster_texts("xyz", &[1, 2]), ["xy", "z"]);
        // Offsets beyond the text or inside a multi-byte character yield empty strings.
        assert_eq!(cluster_texts("é", &[0, 1, 9]), ["", "", ""]);
        // Non-contiguous repeats of a cluster only emit its text once.
        assert_eq!(cluster_texts("ab", &[0, 1, 0]), ["a", "b", ""]);
        assert!(cluster_texts("", &[]).is_empty());
    }

    #[test]
    fn outline_sink_flips_y_scales_and_keeps_curve_types() {
        let mut path = Path::new();
        let mut sink = OutlineSink { path: &mut path, scale: 0.5, x: 10.0, y: 100.0 };
        sink.move_to(0.0, 0.0);
        sink.line_to(20.0, 0.0);
        sink.quad_to(40.0, 20.0, 20.0, 40.0);
        sink.curve_to(10.0, 60.0, -10.0, 60.0, 0.0, 0.0);
        sink.close();
        assert_eq!(
            path.segs,
            vec![
                PathSeg::MoveTo(Point::new(10.0, 100.0)),
                PathSeg::LineTo(Point::new(20.0, 100.0)),
                PathSeg::QuadTo(Point::new(30.0, 90.0), Point::new(20.0, 80.0)),
                PathSeg::CubicTo(
                    Point::new(15.0, 70.0),
                    Point::new(5.0, 70.0),
                    Point::new(10.0, 100.0)
                ),
                PathSeg::Close,
            ]
        );
    }

    #[test]
    fn exact_bounds_ignore_control_points_outside_the_curve() {
        // A quadratic arch from (0,0) to (10,0) with its control point at (5,-10):
        // the curve only reaches y = -5.
        let mut quad = Path::new();
        quad.move_to(0.0, 0.0).quad_to(5.0, -10.0, 10.0, 0.0).close();
        let b = outline_bounds(&quad).expect("bounds");
        assert!((b.y + 5.0).abs() < 1e-12 && (b.h - 5.0).abs() < 1e-12, "{b:?}");
        assert_eq!((b.x, b.w), (0.0, 10.0));
        assert_eq!(quad.control_bounds().map(|r| r.y), Some(-10.0));

        // The cubic approximation of a circle touches its bounding square exactly.
        let circle = Path::circle(Point::new(3.0, 4.0), 2.0);
        let b = outline_bounds(&circle).expect("bounds");
        assert!((b.x - 1.0).abs() < 1e-9 && (b.y - 2.0).abs() < 1e-9, "{b:?}");
        assert!((b.w - 4.0).abs() < 1e-9 && (b.h - 4.0).abs() < 1e-9, "{b:?}");

        // A cubic S-curve whose control points overshoot on both sides.
        let mut cubic = Path::new();
        cubic.move_to(0.0, 0.0).cubic_to(0.0, 30.0, 10.0, -30.0, 10.0, 0.0);
        let b = outline_bounds(&cubic).expect("bounds");
        assert!(b.y > -30.0 && b.bottom() < 30.0);
        assert!((b.y + b.bottom()).abs() < 1e-9, "symmetric curve: {b:?}");
        assert!((b.bottom() - 8.660_254_037_844_386).abs() < 1e-9, "{b:?}");
    }

    #[test]
    fn bounds_of_degenerate_paths() {
        assert_eq!(outline_bounds(&Path::new()), None);
        let mut nan = Path::new();
        nan.move_to(0.0, f64::NAN).line_to(1.0, 1.0);
        assert_eq!(outline_bounds(&nan), None);
        let dot = Path::line(Point::new(2.0, 3.0), Point::new(2.0, 3.0));
        assert_eq!(outline_bounds(&dot), Some(Rect::new(2.0, 3.0, 0.0, 0.0)));
    }

    #[test]
    fn sizes_are_sanitised() {
        assert_eq!(sane_size(f64::NAN), 0.0);
        assert_eq!(sane_size(-3.0), 0.0);
        assert_eq!(sane_size(f64::INFINITY), MAX_SIZE_PT);
        assert_eq!(sane_size(12.5), 12.5);
    }

    #[test]
    fn feature_and_language_options_are_validated() {
        let opts = ShapeOptions {
            letter_spacing_em: 0.1,
            language: Some("fr_FR".into()),
            rtl: false,
            features: vec![
                ("smcp".into(), 1),
                ("toolong".into(), 1),
                (String::new(), 1),
                ("é".into(), 1),
                ("ss1".into(), 2),
            ],
        };
        let f = features(&opts);
        let tags: Vec<String> = f.iter().map(|f| f.tag.to_string()).collect();
        assert_eq!(tags, ["liga", "clig", "smcp", "ss1 "]);
        assert_eq!(f[0].value, 0);
        assert_eq!(f[3].value, 2);
        assert_eq!(language(&opts).map(|l| l.as_str().to_owned()).as_deref(), Some("fr-fr"));

        let bad = ShapeOptions { language: Some("fr; DROP".into()), ..ShapeOptions::default() };
        assert!(language(&bad).is_none());
        assert!(features(&bad).is_empty());
        let blank = ShapeOptions { language: Some("  ".into()), ..ShapeOptions::default() };
        assert!(language(&blank).is_none());
    }
}
