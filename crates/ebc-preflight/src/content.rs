//! Interprets page content streams far enough to know what is painted, where, how large and
//! in which colour: graphics state stack, CTM, colours, images, form XObjects, patterns,
//! shadings, transparency and total ink.

use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

use crate::codes;
use crate::colour::{ColorClass, ColorSpace};
use crate::expect::Expectations;
use crate::lexer::{InlineColorSpace, InlineImage, Lexer, Operand, Token};
use crate::pdf::{
    Matrix, RectPt, as_dict, get, get_bool, get_dict, get_int, get_name, get_num, lossy, resolve,
    resolve_with_id,
};
use crate::qr;
use crate::report::{ImageSummary, QrCandidate, round1, round3};
use crate::sink::{Budget, Sink};

/// How deeply form XObjects and tiling patterns may nest.
const MAX_FORM_DEPTH: usize = 16;
/// How many graphics states `q` may stack; deeper saves are counted but not stored.
const MAX_GSTATE_DEPTH: usize = 256;
/// Image placements recorded per page.
const MAX_IMAGES_PER_PAGE: usize = 2_000;
/// Content streams read from a page's `Contents` array.
const MAX_CONTENT_STREAMS: usize = 4_096;
/// Overlap below this (in points, about 0.01 mm) does not count as touching the bleed box.
const TOUCH_TOLERANCE_PT: f64 = 0.03;

/// Identifies a font dictionary: by object id when it is an indirect object, by resource
/// name otherwise.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum FontKey {
    Id(ObjectId),
    Direct(Vec<u8>),
}

/// What the interpreter learned about one page.
#[derive(Debug, Default)]
pub(crate) struct PageFacts {
    pub(crate) images: Vec<ImageSummary>,
    pub(crate) color_spaces: BTreeSet<String>,
    pub(crate) used_fonts: BTreeSet<FontKey>,
    pub(crate) max_ink_percent: Option<f32>,
    pub(crate) transparency: bool,
    /// Whether an RGB-like colour space was used to paint (allowed or not).
    pub(crate) uses_rgb: bool,
    /// Bounding box, in default user space, of everything painted in (or reaching into) the
    /// printed area whose position is known.
    pub(crate) painted: Option<RectPt>,
    /// Whether something was painted whose extent is unknown but may be the whole page
    /// (a shading painted with `sh` fills the clip region).
    pub(crate) unbounded_paint: bool,
    pub(crate) qr_candidates: Vec<QrCandidate>,
}

/// A current colour: its space, the last components set and the pattern name, if any.
#[derive(Debug, Clone)]
struct Paint {
    space: Rc<ColorSpace>,
    components: [f64; 4],
    count: usize,
    pattern: Option<Rc<[u8]>>,
}

impl Paint {
    fn new(space: ColorSpace, components: &[f64]) -> Self {
        let mut stored = [0.0; 4];
        for (slot, value) in stored.iter_mut().zip(components) {
            *slot = *value;
        }
        Self { space: Rc::new(space), components: stored, count: components.len(), pattern: None }
    }

    /// The initial colour of both paints: black in `DeviceGray`.
    fn black() -> Self {
        Self::new(ColorSpace::DeviceGray, &[0.0])
    }
}

#[derive(Debug, Clone)]
struct GState {
    ctm: Matrix,
    fill: Paint,
    stroke: Paint,
    line_width: f64,
    text_render_mode: i64,
}

/// Where a paint operation lands.
#[derive(Debug, Clone, Copy)]
enum Geometry {
    /// Bounding box in default user space.
    Bbox(RectPt),
    /// Not determinable (text, shadings, pattern cells).
    Unknown,
}

/// The path under construction, in default user space.
#[derive(Debug, Default)]
struct PathState {
    bbox: Option<RectPt>,
    rects: Vec<qr::Rect>,
    /// A subpath that is not an axis-aligned rectangle was added.
    irregular: bool,
    subpath: Vec<(f64, f64)>,
    subpath_irregular: bool,
}

impl PathState {
    fn add_point(&mut self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            self.irregular = true;
            return;
        }
        self.bbox = Some(match self.bbox {
            None => RectPt { x0: x, y0: y, x1: x, y1: y },
            Some(b) => {
                RectPt { x0: b.x0.min(x), y0: b.y0.min(y), x1: b.x1.max(x), y1: b.y1.max(y) }
            }
        });
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.finish_subpath();
        self.add_point(x, y);
        self.subpath.push((x, y));
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.add_point(x, y);
        if self.subpath.len() < 6 {
            self.subpath.push((x, y));
        } else {
            self.subpath_irregular = true;
        }
    }

    fn curve_to(&mut self, points: &[(f64, f64)]) {
        for &(x, y) in points {
            self.add_point(x, y);
        }
        self.subpath_irregular = true;
    }

    fn rect(&mut self, corners: [(f64, f64); 4]) {
        self.finish_subpath();
        for (x, y) in corners {
            self.add_point(x, y);
        }
        self.subpath.extend_from_slice(&corners);
        self.finish_subpath();
    }

    fn finish_subpath(&mut self) {
        let points = std::mem::take(&mut self.subpath);
        let irregular = std::mem::take(&mut self.subpath_irregular);
        // A lone `m` paints nothing.
        if points.len() <= 1 && !irregular {
            return;
        }
        match qr::axis_aligned_rect(&points) {
            Some(rect) if !irregular && self.rects.len() < qr::MAX_RECTS => self.rects.push(rect),
            _ => self.irregular = true,
        }
    }

    fn is_empty(&self) -> bool {
        self.bbox.is_none()
    }

    fn clear(&mut self) {
        self.bbox = None;
        self.rects.clear();
        self.irregular = false;
        self.subpath.clear();
        self.subpath_irregular = false;
    }
}

/// Interprets the content of one page.
pub(crate) struct Interpreter<'a, 's> {
    doc: &'a Document,
    expect: &'a Expectations,
    page: u32,
    /// Millimetres per unit of default user space (`UserUnit` × 25.4 / 72).
    mm_per_unit: f64,
    /// The area that ends up on paper next to or inside the finished piece: the bleed box,
    /// else the trim box. `None` when the page has neither.
    printed: Option<RectPt>,
    sink: &'s mut Sink,
    budget: &'s mut Budget,
    facts: PageFacts,
    /// Form XObjects and tiling patterns currently being interpreted (cycle detection).
    active: Vec<ObjectId>,
    /// Tiling patterns already interpreted on this page.
    patterns_done: HashSet<ObjectId>,
    /// Greater than zero while inside a tiling pattern: geometry is then unknown.
    in_pattern: usize,
    /// Consecutive same-colour rectangle fills, candidates for a QR code.
    rect_run: Vec<qr::Rect>,
}

impl<'a, 's> Interpreter<'a, 's> {
    pub(crate) fn new(
        doc: &'a Document,
        expect: &'a Expectations,
        page: u32,
        user_unit: f64,
        printed: Option<RectPt>,
        sink: &'s mut Sink,
        budget: &'s mut Budget,
    ) -> Self {
        Self {
            doc,
            expect,
            page,
            mm_per_unit: user_unit * 25.4 / 72.0,
            printed,
            sink,
            budget,
            facts: PageFacts::default(),
            active: Vec::new(),
            patterns_done: HashSet::new(),
            in_pattern: 0,
            rect_run: Vec::new(),
        }
    }

    /// Interprets the page and returns what was found.
    pub(crate) fn run(
        mut self,
        page: &'a Dictionary,
        resources: Option<&'a Dictionary>,
    ) -> PageFacts {
        if self.is_transparency_group(page) {
            self.transparency("the page has a transparency group");
        }
        let data = self.page_content(page);
        let state = GState {
            ctm: Matrix::IDENTITY,
            fill: Paint::black(),
            stroke: Paint::black(),
            line_width: 1.0,
            text_render_mode: 0,
        };
        self.exec(&data, resources, state, 0, "page content");
        self.facts
    }

    fn is_transparency_group(&self, dict: &Dictionary) -> bool {
        get_dict(self.doc, dict, b"Group")
            .and_then(|group| get_name(self.doc, group, b"S"))
            .is_some_and(|s| s == b"Transparency")
    }

    fn transparency(&mut self, message: impl Into<String>) {
        self.facts.transparency = true;
        if !self.expect.allow_transparency {
            self.sink.error(codes::TRANSPARENCY, Some(self.page), message);
        }
    }

    // ---- streams -----------------------------------------------------------------------

    /// Concatenates the page's content streams (decoded), separated by newlines.
    fn page_content(&mut self, page: &'a Dictionary) -> Vec<u8> {
        let mut out = Vec::new();
        let Ok(contents) = page.get(b"Contents") else { return out };
        let mut streams: Vec<&Stream> = Vec::new();
        match resolve(self.doc, contents) {
            Some(Object::Stream(stream)) => streams.push(stream),
            Some(Object::Array(items)) => {
                if items.len() > MAX_CONTENT_STREAMS {
                    self.budget.report_exhausted(
                        self.sink,
                        Some(self.page),
                        "too many content streams",
                    );
                }
                for item in items.iter().take(MAX_CONTENT_STREAMS) {
                    match resolve(self.doc, item) {
                        Some(Object::Stream(stream)) => streams.push(stream),
                        _ => self.sink.error(
                            codes::CONTENT_INVALID,
                            Some(self.page),
                            "an entry of the page's Contents array is not a stream",
                        ),
                    }
                }
            }
            Some(_) => self.sink.error(
                codes::CONTENT_INVALID,
                Some(self.page),
                "the page's Contents is neither a stream nor an array of streams",
            ),
            // Absent, null or dangling: an empty page.
            None => {}
        }
        for stream in streams {
            if let Some(data) = self.decode(stream, "page content stream") {
                out.extend_from_slice(&data);
                out.push(b'\n');
            }
        }
        out
    }

    /// Decodes a stream within the document's byte budget, reporting failures.
    fn decode(&mut self, stream: &Stream, what: &str) -> Option<Vec<u8>> {
        let limit = self.budget.stream_limit();
        match crate::stream::decode(stream, limit) {
            Ok(data) => {
                self.budget.charge_bytes(data.len());
                Some(data)
            }
            Err(crate::stream::DecodeError::TooLarge) => {
                self.budget.report_exhausted(self.sink, Some(self.page), what);
                None
            }
            Err(crate::stream::DecodeError::Invalid(reason)) => {
                self.sink.error(
                    codes::CONTENT_INVALID,
                    Some(self.page),
                    format!("{what} cannot be decoded: {reason}"),
                );
                None
            }
        }
    }

    // ---- resources ---------------------------------------------------------------------

    /// The raw (unresolved) entry `resources[category][name]`.
    fn resource(
        &self,
        resources: Option<&'a Dictionary>,
        category: &[u8],
        name: &[u8],
    ) -> Option<&'a Object> {
        get_dict(self.doc, resources?, category)?.get(name).ok()
    }

    fn missing_resource(&mut self, category: &str, name: &[u8]) {
        self.sink.warning(
            codes::RESOURCE_MISSING,
            Some(self.page),
            format!("{category} resource /{} is used but not defined", lossy(name)),
        );
    }

    // ---- the interpreter loop ----------------------------------------------------------

    fn exec(
        &mut self,
        data: &[u8],
        resources: Option<&'a Dictionary>,
        initial: GState,
        depth: usize,
        what: &str,
    ) {
        let mut gs = initial;
        let mut stack: Vec<GState> = Vec::new();
        // `q` operators beyond MAX_GSTATE_DEPTH: counted so that their `Q` pops nothing.
        let mut overflow = 0_usize;
        let mut underflow = false;
        // Set when interpretation stops before the end of the stream.
        let mut aborted = false;
        let mut path = PathState::default();
        let mut lexer = Lexer::new(data);
        loop {
            if !self.budget.take_operation() {
                self.budget.report_exhausted(self.sink, Some(self.page), what);
                aborted = true;
                break;
            }
            let token = match lexer.next_token() {
                Ok(Some(token)) => token,
                Ok(None) => break,
                Err(error) => {
                    self.sink.error(
                        codes::CONTENT_INVALID,
                        Some(self.page),
                        format!("syntax error in {what} at byte {}: {}", error.offset, error.what),
                    );
                    aborted = true;
                    break;
                }
            };
            let operator = match token {
                Token::InlineImage(image) => {
                    self.flush_rect_run();
                    self.inline_image(&image, resources, &gs);
                    continue;
                }
                Token::Operator(operator) => operator,
            };
            match operator {
                // -- graphics state --
                b"q" => {
                    if stack.len() < MAX_GSTATE_DEPTH {
                        stack.push(gs.clone());
                    } else {
                        overflow += 1;
                    }
                }
                b"Q" => {
                    if overflow > 0 {
                        overflow -= 1;
                    } else if let Some(saved) = stack.pop() {
                        if !same_fill(&saved.fill, &gs.fill) {
                            self.flush_rect_run();
                        }
                        gs = saved;
                    } else {
                        underflow = true;
                    }
                }
                b"cm" => match lexer.numbers::<6>().and_then(|v| Matrix::from_slice(&v)) {
                    Some(matrix) => {
                        let ctm = matrix.then(&gs.ctm);
                        if ctm.is_sane() {
                            gs.ctm = ctm;
                        } else {
                            self.bad_operands(what, operator);
                        }
                    }
                    None => self.bad_operands(what, operator),
                },
                b"w" => match lexer.numbers::<1>() {
                    Some([width]) => gs.line_width = width.abs(),
                    None => self.bad_operands(what, operator),
                },
                b"gs" => match lexer.last_name() {
                    Some(name) => self.ext_gstate(name, resources, &mut gs),
                    None => self.bad_operands(what, operator),
                },
                // -- colour --
                b"g" | b"G" => match lexer.numbers::<1>() {
                    Some(v) => self.set_paint(
                        &mut gs,
                        operator == b"g",
                        Paint::new(ColorSpace::DeviceGray, &v),
                    ),
                    None => self.bad_operands(what, operator),
                },
                b"rg" | b"RG" => match lexer.numbers::<3>() {
                    Some(v) => self.set_paint(
                        &mut gs,
                        operator == b"rg",
                        Paint::new(ColorSpace::DeviceRgb, &v),
                    ),
                    None => self.bad_operands(what, operator),
                },
                b"k" | b"K" => match lexer.numbers::<4>() {
                    Some(v) => self.set_paint(
                        &mut gs,
                        operator == b"k",
                        Paint::new(ColorSpace::DeviceCmyk, &v),
                    ),
                    None => self.bad_operands(what, operator),
                },
                b"cs" | b"CS" => match lexer.last_name() {
                    Some(name) => {
                        let space = ColorSpace::from_name(self.doc, name, resources);
                        if let ColorSpace::Unknown(reason) = &space {
                            self.sink.warning(
                                codes::COLOR_UNKNOWN,
                                Some(self.page),
                                format!(
                                    "colour space /{} cannot be resolved ({reason})",
                                    lossy(name)
                                ),
                            );
                        }
                        // Initial colour: all components zero, except black for CMYK.
                        let initial =
                            if space.is_cmyk_ink() { [0.0, 0.0, 0.0, 1.0] } else { [0.0; 4] };
                        let mut paint = Paint::new(space, &initial);
                        paint.count = 0;
                        self.set_paint(&mut gs, operator == b"cs", paint);
                    }
                    None => self.bad_operands(what, operator),
                },
                b"sc" | b"scn" | b"SC" | b"SCN" => {
                    let fill = operator[0] == b's';
                    let mut components = [0.0; 4];
                    let mut count = 0;
                    for operand in lexer.operands() {
                        if let Operand::Number(value) = operand {
                            if count < 4 {
                                components[count] = *value;
                            }
                            count += 1;
                        }
                    }
                    let pattern = lexer.last_name().map(Rc::from);
                    if count == 0 && pattern.is_none() {
                        self.bad_operands(what, operator);
                    } else {
                        let current = if fill { &gs.fill } else { &gs.stroke };
                        let paint =
                            Paint { space: Rc::clone(&current.space), components, count, pattern };
                        self.set_paint(&mut gs, fill, paint);
                    }
                }
                // -- path construction --
                b"m" => match lexer.numbers::<2>() {
                    Some([x, y]) => {
                        let (x, y) = gs.ctm.apply(x, y);
                        path.move_to(x, y);
                    }
                    None => self.bad_operands(what, operator),
                },
                b"l" => match lexer.numbers::<2>() {
                    Some([x, y]) => {
                        let (x, y) = gs.ctm.apply(x, y);
                        path.line_to(x, y);
                    }
                    None => self.bad_operands(what, operator),
                },
                b"c" => match lexer.numbers::<6>() {
                    Some([x1, y1, x2, y2, x3, y3]) => path.curve_to(&[
                        gs.ctm.apply(x1, y1),
                        gs.ctm.apply(x2, y2),
                        gs.ctm.apply(x3, y3),
                    ]),
                    None => self.bad_operands(what, operator),
                },
                b"v" | b"y" => match lexer.numbers::<4>() {
                    Some([x1, y1, x2, y2]) => {
                        path.curve_to(&[gs.ctm.apply(x1, y1), gs.ctm.apply(x2, y2)]);
                    }
                    None => self.bad_operands(what, operator),
                },
                b"h" => path.finish_subpath(),
                b"re" => match lexer.numbers::<4>() {
                    Some([x, y, w, h]) => path.rect([
                        gs.ctm.apply(x, y),
                        gs.ctm.apply(x + w, y),
                        gs.ctm.apply(x + w, y + h),
                        gs.ctm.apply(x, y + h),
                    ]),
                    None => self.bad_operands(what, operator),
                },
                // -- path painting --
                b"f" | b"F" | b"f*" => {
                    self.paint_path(&mut path, &gs, resources, true, false, depth)
                }
                b"B" | b"B*" | b"b" | b"b*" => {
                    self.paint_path(&mut path, &gs, resources, true, true, depth)
                }
                b"S" | b"s" => self.paint_path(&mut path, &gs, resources, false, true, depth),
                b"n" => path.clear(),
                // -- text --
                b"Tf" => {
                    let name = lexer.operands().iter().rev().find_map(|operand| match operand {
                        Operand::Name(name) => Some(name.as_slice()),
                        _ => None,
                    });
                    match name {
                        Some(name) => self.select_font(name, resources),
                        None => self.bad_operands(what, operator),
                    }
                }
                b"Tr" => match lexer.numbers::<1>() {
                    // Whole and tiny by construction of the range check.
                    #[allow(clippy::cast_possible_truncation)]
                    Some([mode]) if (0.0..=7.0).contains(&mode) => {
                        gs.text_render_mode = mode as i64
                    }
                    _ => self.bad_operands(what, operator),
                },
                b"Tj" | b"TJ" | b"'" | b"\"" => {
                    self.flush_rect_run();
                    let mode = gs.text_render_mode;
                    let fill = matches!(mode, 0 | 2 | 4 | 6);
                    let stroke = matches!(mode, 1 | 2 | 5 | 6);
                    self.paint(&gs, resources, fill, stroke, Geometry::Unknown, depth);
                }
                // -- XObjects, shadings --
                b"Do" => match lexer.last_name() {
                    Some(name) => {
                        self.flush_rect_run();
                        self.do_xobject(name, resources, &gs, depth);
                    }
                    None => self.bad_operands(what, operator),
                },
                b"sh" => match lexer.last_name() {
                    Some(name) => {
                        self.flush_rect_run();
                        self.shading(name, resources);
                    }
                    None => self.bad_operands(what, operator),
                },
                // Clipping, text positioning, marked content, compatibility sections and the
                // remaining state operators do not affect what the checks look at.
                _ => {}
            }
        }
        self.flush_rect_run();
        if underflow {
            self.sink.warning(
                codes::CONTENT_UNBALANCED_Q,
                Some(self.page),
                format!("{what} restores a graphics state (Q) that was never saved (q)"),
            );
        }
        // A stream that was not read to its end says nothing about its last `Q`s.
        let unclosed = if aborted { 0 } else { stack.len() + overflow };
        if unclosed > 0 {
            self.sink.warning(
                codes::CONTENT_UNBALANCED_Q,
                Some(self.page),
                format!(
                    "{what} leaves {unclosed} saved graphics state(s) (q) without a matching Q"
                ),
            );
        }
    }

    fn bad_operands(&mut self, what: &str, operator: &[u8]) {
        self.sink.error(
            codes::CONTENT_INVALID,
            Some(self.page),
            format!("operator {} in {what} has missing or invalid operands", lossy(operator)),
        );
    }

    fn set_paint(&mut self, gs: &mut GState, fill: bool, paint: Paint) {
        if fill {
            if !same_fill(&gs.fill, &paint) {
                self.flush_rect_run();
            }
            gs.fill = paint;
        } else {
            gs.stroke = paint;
        }
    }

    // ---- painting ----------------------------------------------------------------------

    fn paint_path(
        &mut self,
        path: &mut PathState,
        gs: &GState,
        resources: Option<&'a Dictionary>,
        fill: bool,
        stroke: bool,
        depth: usize,
    ) {
        path.finish_subpath();
        if path.is_empty() {
            path.clear();
            return;
        }
        let rect_only =
            fill && !stroke && !path.irregular && !path.rects.is_empty() && self.in_pattern == 0;
        if rect_only {
            let room = qr::MAX_RECTS.saturating_sub(self.rect_run.len());
            self.rect_run.extend(path.rects.iter().take(room));
        } else {
            self.flush_rect_run();
        }
        let geometry = match path.bbox {
            Some(bbox) if self.in_pattern == 0 => {
                // A stroke reaches half a line width beyond the path.
                let grow = if stroke {
                    gs.line_width * 0.5 * gs.ctm.scale_x().max(gs.ctm.scale_y())
                } else {
                    0.0
                };
                let grow = if grow.is_finite() { grow } else { 0.0 };
                Geometry::Bbox(RectPt {
                    x0: bbox.x0 - grow,
                    y0: bbox.y0 - grow,
                    x1: bbox.x1 + grow,
                    y1: bbox.y1 + grow,
                })
            }
            _ => Geometry::Unknown,
        };
        path.clear();
        if stroke {
            self.check_stroke_width(gs, geometry);
        }
        self.paint(gs, resources, fill, stroke, geometry, depth);
    }

    /// Flags strokes too thin to print reliably.
    fn check_stroke_width(&mut self, gs: &GState, geometry: Geometry) {
        /// Printers ask for 0.25 pt at least; the margin absorbs rounding in transforms.
        const MIN_STROKE_PT: f64 = 0.24;
        if self.touches_printed_area(geometry) == Some(false) {
            return;
        }
        let scale = (gs.ctm.a * gs.ctm.d - gs.ctm.b * gs.ctm.c).abs().sqrt();
        let width_pt = gs.line_width * scale * self.mm_per_unit * 72.0 / 25.4;
        if !width_pt.is_finite() || width_pt >= MIN_STROKE_PT {
            return;
        }
        if width_pt == 0.0 {
            self.sink.error(
                codes::STROKE_HAIRLINE,
                Some(self.page),
                "a stroke has zero width, i.e. the thinnest line the device can draw: \
                 visible on screen, all but invisible on a press",
            );
        } else {
            self.sink.warning(
                codes::STROKE_HAIRLINE,
                Some(self.page),
                format!(
                    "a stroke is only {width_pt:.2} pt wide; lines thinner than 0.25 pt may \
                     break up or vanish in print"
                ),
            );
        }
    }

    fn paint(
        &mut self,
        gs: &GState,
        resources: Option<&'a Dictionary>,
        fill: bool,
        stroke: bool,
        geometry: Geometry,
        depth: usize,
    ) {
        if (fill || stroke)
            && let Geometry::Bbox(bbox) = geometry
        {
            self.note_painted(bbox);
        }
        if fill {
            self.use_paint(&gs.fill, resources, geometry, "fill", depth);
        }
        if stroke {
            self.use_paint(&gs.stroke, resources, geometry, "stroke", depth);
        }
    }

    /// Extends the bounding box of the artwork painted so far. Whatever lies entirely
    /// outside the printed area (crop marks, colour bars and notes in the slug) is not
    /// artwork: counting it would make every page with printer's marks look as if its
    /// artwork filled the bleed.
    fn note_painted(&mut self, bbox: RectPt) {
        if self.printed.is_some_and(|printed| !printed.overlaps(&bbox, TOUCH_TOLERANCE_PT)) {
            return;
        }
        self.facts.painted = Some(match self.facts.painted {
            None => bbox,
            Some(all) => RectPt {
                x0: all.x0.min(bbox.x0),
                y0: all.y0.min(bbox.y0),
                x1: all.x1.max(bbox.x1),
                y1: all.y1.max(bbox.y1),
            },
        });
    }

    /// Whether a paint operation touches the printed area. Unknown geometry counts as inside.
    fn touches_printed_area(&self, geometry: Geometry) -> Option<bool> {
        match (geometry, self.printed) {
            (Geometry::Bbox(bbox), Some(printed)) => {
                Some(printed.overlaps(&bbox, TOUCH_TOLERANCE_PT))
            }
            _ => None,
        }
    }

    fn use_paint(
        &mut self,
        paint: &Paint,
        resources: Option<&'a Dictionary>,
        geometry: Geometry,
        role: &str,
        depth: usize,
    ) {
        if let ColorSpace::Pattern(base) = &*paint.space {
            // After `cs /Pattern` without `scn` there is no pattern, and nothing to check.
            if let Some(name) = &paint.pattern {
                self.use_pattern(name, base.as_deref(), paint, resources, depth);
            }
            return;
        }
        self.note_space(&paint.space, &format!("a {role}"));
        self.check_colour_values(paint, geometry, role);
    }

    /// Checks registration colour and total ink for colour values in `paint.space`.
    fn check_colour_values(&mut self, paint: &Paint, geometry: Geometry, role: &str) {
        let space = match &*paint.space {
            ColorSpace::Pattern(Some(base)) => base.as_ref(),
            other => other,
        };
        match space.class() {
            ColorClass::Registration => {
                // A zero tint paints nothing on any plate.
                if paint.count == 0 || paint.components[0] > 0.0 {
                    let what =
                        format!("registration colour (Separation /All) is used for a {role}");
                    match self.touches_printed_area(geometry) {
                        Some(true) => self.sink.error(
                            codes::COLOR_REGISTRATION,
                            Some(self.page),
                            format!("{what} inside the bleed box"),
                        ),
                        Some(false) => {}
                        None => self.sink.warning(
                            codes::COLOR_REGISTRATION,
                            Some(self.page),
                            format!("{what} whose position cannot be determined"),
                        ),
                    }
                }
            }
            ColorClass::Cmyk if space.is_cmyk_ink() && paint.count >= 4 => {
                if self.touches_printed_area(geometry) == Some(false) {
                    return;
                }
                let total: f64 =
                    paint.components.iter().map(|c| c.clamp(0.0, 1.0)).sum::<f64>() * 100.0;
                // At most 400.
                #[allow(clippy::cast_possible_truncation)]
                let total = total as f32;
                self.facts.max_ink_percent =
                    Some(self.facts.max_ink_percent.map_or(total, |max| max.max(total)));
                if let Some(limit) = self.expect.max_total_ink_percent
                    && total > limit + 0.05
                {
                    let [c, m, y, k] =
                        paint.components.map(|v| (v.clamp(0.0, 1.0) * 100.0).round());
                    let message = format!(
                        "a {role} uses C{c} M{m} Y{y} K{k} = {total:.0} % total ink, \
                         above the limit of {limit:.0} %"
                    );
                    self.sink.error(codes::INK_TOTAL_TOO_HIGH, Some(self.page), message);
                }
            }
            _ => {}
        }
    }

    /// Records a colour space as used and flags what a CMYK job does not accept.
    fn note_space(&mut self, space: &ColorSpace, context: &str) {
        let description = space.describe();
        let class = space.class();
        self.facts.uses_rgb |= class == ColorClass::Rgb;
        match class {
            ColorClass::Rgb if !self.expect.allow_rgb => self.sink.error(
                codes::COLOR_RGB,
                Some(self.page),
                format!("{context} uses {description}"),
            ),
            ColorClass::Spot => self.sink.warning(
                codes::COLOR_SPOT,
                Some(self.page),
                format!("{context} uses the spot colour space {description}"),
            ),
            ColorClass::Unknown => self.sink.warning(
                codes::COLOR_UNKNOWN,
                Some(self.page),
                format!("{context} uses a colour space that cannot be resolved: {description}"),
            ),
            _ => {}
        }
        self.facts.color_spaces.insert(description);
    }

    fn use_pattern(
        &mut self,
        name: &[u8],
        base: Option<&ColorSpace>,
        paint: &Paint,
        resources: Option<&'a Dictionary>,
        depth: usize,
    ) {
        let Some(entry) = self.resource(resources, b"Pattern", name) else {
            self.missing_resource("Pattern", name);
            return;
        };
        let (id, Some(object)) = resolve_with_id(self.doc, entry) else {
            self.missing_resource("Pattern", name);
            return;
        };
        match object {
            // Shading pattern.
            Object::Dictionary(dict) => {
                if let Ok(shading) = dict.get(b"Shading") {
                    self.note_shading(shading, &format!("shading pattern /{}", lossy(name)));
                }
                if let Some(ext) = get_dict(self.doc, dict, b"ExtGState") {
                    self.check_ext_gstate(ext, name);
                }
            }
            // Tiling pattern.
            Object::Stream(stream) => {
                let uncoloured = get_int(self.doc, &stream.dict, b"PaintType") == Some(2);
                if uncoloured && let Some(base) = base {
                    self.note_space(base, &format!("uncoloured pattern /{}", lossy(name)));
                    self.check_colour_values(paint, Geometry::Unknown, "pattern fill");
                }
                if let Some(id) = id
                    && !self.patterns_done.insert(id)
                {
                    return;
                }
                self.run_nested(
                    id,
                    stream,
                    resources,
                    None,
                    depth,
                    &format!("tiling pattern /{}", lossy(name)),
                    true,
                );
            }
            _ => self.missing_resource("Pattern", name),
        }
    }

    fn shading(&mut self, name: &[u8], resources: Option<&'a Dictionary>) {
        self.facts.unbounded_paint = true;
        match self.resource(resources, b"Shading", name) {
            Some(shading) => self.note_shading(shading, &format!("shading /{}", lossy(name))),
            None => self.missing_resource("Shading", name),
        }
    }

    fn note_shading(&mut self, shading: &'a Object, context: &str) {
        let Some(dict) = as_dict(self.doc, shading) else { return };
        match dict.get(b"ColorSpace") {
            Ok(space) => {
                let space = ColorSpace::from_object(self.doc, space, None);
                self.note_space(&space, context);
            }
            Err(_) => self.sink.warning(
                codes::COLOR_UNKNOWN,
                Some(self.page),
                format!("{context} has no colour space"),
            ),
        }
    }

    // ---- extended graphics state -------------------------------------------------------

    fn ext_gstate(&mut self, name: &[u8], resources: Option<&'a Dictionary>, gs: &mut GState) {
        let dict =
            self.resource(resources, b"ExtGState", name).and_then(|entry| as_dict(self.doc, entry));
        let Some(dict) = dict else {
            self.missing_resource("ExtGState", name);
            return;
        };
        self.check_ext_gstate(dict, name);
        if let Some(width) = get_num(self.doc, dict, b"LW") {
            gs.line_width = width.abs();
        }
        // `Font [ref size]` selects a font like Tf does.
        if let Some(Object::Array(font)) = get(self.doc, dict, b"Font")
            && let Some((Some(id), Some(_))) = font.first().map(|f| resolve_with_id(self.doc, f))
        {
            self.facts.used_fonts.insert(FontKey::Id(id));
        }
    }

    fn check_ext_gstate(&mut self, dict: &Dictionary, name: &[u8]) {
        let name = lossy(name);
        if let Some(mask) = get(self.doc, dict, b"SMask")
            && !matches!(mask, Object::Name(none) if none == b"None")
        {
            self.transparency(format!("graphics state /{name} sets a soft mask (SMask)"));
        }
        for key in [&b"ca"[..], b"CA"] {
            if let Some(alpha) = get_num(self.doc, dict, key)
                && alpha < 1.0 - 1.0e-6
            {
                self.transparency(format!(
                    "graphics state /{name} sets opacity {} = {alpha:.3}",
                    lossy(key)
                ));
            }
        }
        let blend = match get(self.doc, dict, b"BM") {
            Some(Object::Name(mode)) => Some(mode.as_slice()),
            Some(Object::Array(modes)) => modes.iter().find_map(|m| match resolve(self.doc, m) {
                Some(Object::Name(mode)) => Some(mode.as_slice()),
                _ => None,
            }),
            _ => None,
        };
        if let Some(mode) = blend
            && mode != b"Normal"
            && mode != b"Compatible"
        {
            self.transparency(format!("graphics state /{name} sets blend mode {}", lossy(mode)));
        }
    }

    // ---- fonts -------------------------------------------------------------------------

    fn select_font(&mut self, name: &[u8], resources: Option<&'a Dictionary>) {
        match self.resource(resources, b"Font", name) {
            Some(entry) => {
                let key = match resolve_with_id(self.doc, entry) {
                    (Some(id), _) => FontKey::Id(id),
                    (None, _) => FontKey::Direct(name.to_vec()),
                };
                self.facts.used_fonts.insert(key);
            }
            None => self.missing_resource("Font", name),
        }
    }

    // ---- XObjects ----------------------------------------------------------------------

    fn do_xobject(
        &mut self,
        name: &[u8],
        resources: Option<&'a Dictionary>,
        gs: &GState,
        depth: usize,
    ) {
        let Some(entry) = self.resource(resources, b"XObject", name) else {
            self.missing_resource("XObject", name);
            return;
        };
        let (id, Some(Object::Stream(stream))) = resolve_with_id(self.doc, entry) else {
            self.missing_resource("XObject", name);
            return;
        };
        match get_name(self.doc, &stream.dict, b"Subtype") {
            Some(b"Image") => self.image_xobject(name, stream, resources, gs, depth),
            Some(b"Form") => {
                if self.is_transparency_group(&stream.dict) {
                    self.transparency(format!(
                        "form XObject /{} is a transparency group",
                        lossy(name)
                    ));
                }
                let what = format!("form XObject /{}", lossy(name));
                self.run_nested(id, stream, resources, Some(gs), depth, &what, false);
            }
            Some(b"PS") => self.sink.error(
                codes::POSTSCRIPT_XOBJECT,
                Some(self.page),
                format!("XObject /{} is a PostScript XObject", lossy(name)),
            ),
            _ => self.sink.error(
                codes::CONTENT_INVALID,
                Some(self.page),
                format!("XObject /{} has no valid Subtype", lossy(name)),
            ),
        }
    }

    /// Interprets a form XObject or tiling pattern cell.
    #[allow(clippy::too_many_arguments)]
    fn run_nested(
        &mut self,
        id: Option<ObjectId>,
        stream: &'a Stream,
        resources: Option<&'a Dictionary>,
        caller: Option<&GState>,
        depth: usize,
        what: &str,
        is_pattern: bool,
    ) {
        if depth >= MAX_FORM_DEPTH {
            self.budget.report_exhausted(
                self.sink,
                Some(self.page),
                "form XObjects nested too deeply",
            );
            return;
        }
        if let Some(id) = id
            && self.active.contains(&id)
        {
            self.sink.error(
                codes::CONTENT_INVALID,
                Some(self.page),
                format!("{what} draws itself (reference cycle)"),
            );
            return;
        }
        if !self.budget.take_nested_run() {
            self.budget.report_exhausted(
                self.sink,
                Some(self.page),
                "too many form XObject or pattern invocations",
            );
            return;
        }
        let Some(data) = self.decode(stream, what) else { return };
        let matrix = stream
            .dict
            .get(b"Matrix")
            .ok()
            .and_then(|m| Matrix::from_object(self.doc, m))
            .filter(Matrix::is_sane)
            .unwrap_or(Matrix::IDENTITY);
        // A form inherits the caller's graphics state; a pattern cell starts from defaults.
        let mut state = caller.cloned().unwrap_or(GState {
            ctm: Matrix::IDENTITY,
            fill: Paint::black(),
            stroke: Paint::black(),
            line_width: 1.0,
            text_render_mode: 0,
        });
        let ctm = matrix.then(&state.ctm);
        if ctm.is_sane() {
            state.ctm = ctm;
        }
        // Forms without their own resources fall back to the caller's (PDF 1.1 behaviour).
        let own = get_dict(self.doc, &stream.dict, b"Resources").or(resources);
        if let Some(id) = id {
            self.active.push(id);
        }
        if is_pattern {
            self.in_pattern += 1;
        }
        self.exec(&data, own, state, depth + 1, what);
        if is_pattern {
            self.in_pattern -= 1;
        }
        if id.is_some() {
            self.active.pop();
        }
    }

    fn image_xobject(
        &mut self,
        name: &[u8],
        stream: &'a Stream,
        resources: Option<&'a Dictionary>,
        gs: &GState,
        depth: usize,
    ) {
        let dict = &stream.dict;
        let display = format!("/{}", lossy(name));
        let filters = filter_names(self.doc, dict);
        let is_mask = get_bool(self.doc, dict, b"ImageMask") == Some(true);
        let color_space = if is_mask {
            self.paint(gs, resources, true, false, self.image_geometry(gs), depth);
            "ImageMask".to_string()
        } else {
            match dict.get(b"ColorSpace") {
                Ok(space) => {
                    let space = ColorSpace::from_object(self.doc, space, resources);
                    self.note_space(&space, &format!("image {display}"));
                    space.describe()
                }
                // JPEG 2000 data carries its own colour space.
                Err(_) if filters.iter().any(|f| f == "JPXDecode") => "JPX".to_string(),
                Err(_) => {
                    self.sink.warning(
                        codes::COLOR_UNKNOWN,
                        Some(self.page),
                        format!("image {display} has no colour space"),
                    );
                    "Unknown".to_string()
                }
            }
        };
        let soft_mask = matches!(get(self.doc, dict, b"SMask"), Some(Object::Stream(_)));
        if soft_mask {
            self.transparency(format!("image {display} has a soft mask (alpha channel)"));
        }
        if get_int(self.doc, dict, b"SMaskInData").is_some_and(|v| v > 0) {
            self.transparency(format!(
                "image {display} carries transparency in its JPEG 2000 data"
            ));
        }
        let bits = if is_mask { Some(1) } else { get_int(self.doc, dict, b"BitsPerComponent") };
        if !is_mask {
            self.check_image_ink(stream, &display, &filters, bits);
        }
        self.place_image(Placement {
            name: display,
            width: get_int(self.doc, dict, b"Width"),
            height: get_int(self.doc, dict, b"Height"),
            bits,
            color_space,
            filters,
            soft_mask,
            inline: false,
            ctm: gs.ctm,
        });
    }

    fn inline_image(
        &mut self,
        image: &InlineImage,
        resources: Option<&'a Dictionary>,
        gs: &GState,
    ) {
        let color_space = if image.image_mask {
            self.paint(gs, resources, true, false, self.image_geometry(gs), 0);
            "ImageMask".to_string()
        } else {
            let space = match &image.color_space {
                InlineColorSpace::Name(name) => ColorSpace::from_name(self.doc, name, resources),
                InlineColorSpace::Indexed(base) => {
                    ColorSpace::Indexed(Box::new(ColorSpace::from_name(self.doc, base, resources)))
                }
                InlineColorSpace::None => ColorSpace::Unknown("missing".to_string()),
                InlineColorSpace::Other => {
                    ColorSpace::Unknown("unsupported inline colour space".to_string())
                }
            };
            self.note_space(&space, "an inline image");
            space.describe()
        };
        self.place_image(Placement {
            name: "(inline)".to_string(),
            width: image.width,
            height: image.height,
            bits: if image.image_mask { Some(1) } else { image.bits_per_component },
            color_space,
            filters: image.filters.iter().map(|f| lossy(f)).collect(),
            soft_mask: false,
            inline: true,
            ctm: gs.ctm,
        });
    }

    /// The bounding box of the unit square an image is painted into.
    fn image_geometry(&self, gs: &GState) -> Geometry {
        self.unit_square_geometry(&gs.ctm)
    }

    /// The bounding box of the unit square under `ctm`.
    fn unit_square_geometry(&self, ctm: &Matrix) -> Geometry {
        if self.in_pattern > 0 {
            return Geometry::Unknown;
        }
        let corners =
            [ctm.apply(0.0, 0.0), ctm.apply(1.0, 0.0), ctm.apply(1.0, 1.0), ctm.apply(0.0, 1.0)];
        let mut bbox = RectPt {
            x0: f64::INFINITY,
            y0: f64::INFINITY,
            x1: f64::NEG_INFINITY,
            y1: f64::NEG_INFINITY,
        };
        for (x, y) in corners {
            if !(x.is_finite() && y.is_finite()) {
                return Geometry::Unknown;
            }
            bbox = RectPt {
                x0: bbox.x0.min(x),
                y0: bbox.y0.min(y),
                x1: bbox.x1.max(x),
                y1: bbox.y1.max(y),
            };
        }
        Geometry::Bbox(bbox)
    }

    fn place_image(&mut self, placement: Placement) {
        let Placement { name, width, height, bits, color_space, filters, soft_mask, inline, ctm } =
            placement;
        const MAX_SIDE: i64 = 1_000_000;
        let (Some(width_px), Some(height_px)) = (
            width.filter(|w| (1..=MAX_SIDE).contains(w)).and_then(|w| u32::try_from(w).ok()),
            height.filter(|h| (1..=MAX_SIDE).contains(h)).and_then(|h| u32::try_from(h).ok()),
        ) else {
            self.sink.warning(
                codes::IMAGE_INVALID,
                Some(self.page),
                format!("image {name} has no valid Width/Height"),
            );
            return;
        };
        let width_mm = ctm.scale_x() * self.mm_per_unit;
        let height_mm = ctm.scale_y() * self.mm_per_unit;
        if !(width_mm.is_finite() && height_mm.is_finite())
            || width_mm < 1.0e-4
            || height_mm < 1.0e-4
        {
            self.sink.warning(
                codes::IMAGE_INVALID,
                Some(self.page),
                format!("image {name} is placed with a degenerate transform (zero size)"),
            );
            return;
        }
        if let Geometry::Bbox(bbox) = self.unit_square_geometry(&ctm) {
            self.note_painted(bbox);
        }
        let ppi_x = f64::from(width_px) / (width_mm / 25.4);
        let ppi_y = f64::from(height_px) / (height_mm / 25.4);
        // Pictures (anything beyond a few pixels a side) must keep their proportions.
        let stretch = (ppi_x / ppi_y).max(ppi_y / ppi_x);
        if width_px > 2 && height_px > 2 && stretch > 1.01 {
            self.sink.warning(
                codes::IMAGE_DISTORTED,
                Some(self.page),
                format!(
                    "image {name} is scaled unevenly ({ppi_x:.0} × {ppi_y:.0} ppi): \
                     it is stretched by {:.0} % in one direction",
                    (stretch - 1.0) * 100.0
                ),
            );
        }
        // A single row or column of pixels has no resolution across it (gradient strips,
        // one-pixel colour swatches).
        let mut effective = f64::INFINITY;
        if width_px > 1 {
            effective = effective.min(ppi_x);
        }
        if height_px > 1 {
            effective = effective.min(ppi_y);
        }
        if effective < self.expect.min_ppi * 0.995 {
            self.sink.error(
                codes::IMAGE_LOW_RES,
                Some(self.page),
                format!(
                    "image {name} ({width_px}×{height_px} px) is printed at {width_mm:.1}×{height_mm:.1} mm, \
                     i.e. {effective:.0} ppi, below the minimum of {:.0} ppi",
                    self.expect.min_ppi
                ),
            );
        }
        if self.facts.images.len() >= MAX_IMAGES_PER_PAGE {
            self.sink.warning(
                codes::LIMIT_EXCEEDED,
                Some(self.page),
                format!(
                    "more than {MAX_IMAGES_PER_PAGE} image placements; only the first are listed"
                ),
            );
            return;
        }
        self.facts.images.push(ImageSummary {
            name,
            width_px,
            height_px,
            width_mm: round3(width_mm),
            height_mm: round3(height_mm),
            ppi_x: round1(ppi_x),
            ppi_y: round1(ppi_y),
            color_space,
            bits_per_component: bits.and_then(|b| u8::try_from(b).ok()).unwrap_or(0),
            filters,
            soft_mask,
            inline,
        });
    }

    /// For losslessly stored 8-bit CMYK images, measures the share of pixels above the
    /// total ink limit.
    fn check_image_ink(
        &mut self,
        stream: &Stream,
        display: &str,
        filters: &[String],
        bits: Option<i64>,
    ) {
        /// Share of pixels above the limit that triggers the warning.
        const MAX_SHARE: f64 = 0.005;
        /// Largest decoded image examined.
        const MAX_BYTES: usize = 32 * 1024 * 1024;
        let Some(limit) = self.expect.max_total_ink_percent else { return };
        if bits != Some(8) || filters.iter().any(|f| f != "FlateDecode") {
            return;
        }
        let is_cmyk = stream
            .dict
            .get(b"ColorSpace")
            .is_ok_and(|space| ColorSpace::from_object(self.doc, space, None).is_cmyk_ink());
        if !is_cmyk {
            return;
        }
        let (Some(width), Some(height)) = (
            get_int(self.doc, &stream.dict, b"Width").and_then(|w| usize::try_from(w).ok()),
            get_int(self.doc, &stream.dict, b"Height").and_then(|h| usize::try_from(h).ok()),
        ) else {
            return;
        };
        let Some(expected) = width.checked_mul(height).and_then(|n| n.checked_mul(4)) else {
            return;
        };
        if expected == 0 || expected > MAX_BYTES.min(self.budget.stream_limit()) {
            return;
        }
        // Predictors are handled by lopdf; a failure here is not a finding of its own.
        let Ok(data) = stream.get_plain_content_with_limit(expected) else { return };
        self.budget.charge_bytes(data.len());
        if data.len() != expected {
            return;
        }
        let limit_units = f64::from(limit) * 255.0 / 100.0;
        let over = data
            .chunks_exact(4)
            .filter(|px| {
                f64::from(px.iter().map(|&v| u32::from(v)).sum::<u32>()) > limit_units + 0.5
            })
            .count();
        // Pixel counts are far below 2^52.
        #[allow(clippy::cast_precision_loss)]
        let share = over as f64 / (expected / 4) as f64;
        if share > MAX_SHARE {
            self.sink.warning(
                codes::IMAGE_INK_TOTAL_TOO_HIGH,
                Some(self.page),
                format!(
                    "{:.0} % of the pixels of CMYK image {display} exceed {limit:.0} % total ink",
                    share * 100.0
                ),
            );
        }
    }

    // ---- QR candidates -----------------------------------------------------------------

    fn flush_rect_run(&mut self) {
        if self.rect_run.len() >= qr::MIN_RECTS
            && let Some(candidate) = qr::detect(&self.rect_run, self.mm_per_unit)
            && self.facts.qr_candidates.len() < 64
        {
            self.facts.qr_candidates.push(candidate);
        }
        self.rect_run.clear();
    }
}

/// Everything needed to record one image placement.
struct Placement {
    name: String,
    width: Option<i64>,
    height: Option<i64>,
    bits: Option<i64>,
    color_space: String,
    filters: Vec<String>,
    soft_mask: bool,
    inline: bool,
    ctm: Matrix,
}

/// Whether two fill colours are the same (space, components and pattern).
fn same_fill(a: &Paint, b: &Paint) -> bool {
    (Rc::ptr_eq(&a.space, &b.space) || a.space == b.space)
        && a.count == b.count
        && a.components == b.components
        && a.pattern == b.pattern
}

/// The names in a stream's `Filter` entry.
pub(crate) fn filter_names(doc: &Document, dict: &Dictionary) -> Vec<String> {
    match get(doc, dict, b"Filter") {
        Some(Object::Name(name)) => vec![lossy(name)],
        Some(Object::Array(items)) => items
            .iter()
            .take(16)
            .filter_map(|item| match resolve(doc, item) {
                Some(Object::Name(name)) => Some(lossy(name)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}
