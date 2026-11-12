//! Glyph outlines with a bound on the work a font can ask for.
//!
//! Font files are hostile input, and two outline formats let a tiny file describe an
//! astronomically large drawing:
//!
//! * **TrueType composites** (`glyf`): a glyph may be made of other glyphs, which may be
//!   made of other glyphs… A 1 kB font with 18 levels of two components each stands for
//!   2^18 copies of its last glyph; with empty leaves nothing is ever drawn, yet every
//!   component is visited.
//! * **Type 2 subroutines** (`CFF `, `CFF2`): the same with nested subroutine calls.
//!
//! `read-fonts`, which draws the glyphs, has limits of its own for one glyph (2 048
//! component references that lead to a non-empty glyph, 64 levels, 65 536 points, a
//! million charstring operators). They do not count references to empty glyphs, they are
//! per glyph — a long text multiplies them — and they do not tell how much a glyph did
//! cost. This module therefore stands between the crate and the library:
//!
//! * for `glyf` fonts the *expanded size* of a glyph — one unit per component reference
//!   and per point, summed over the whole component tree — is computed first, exactly and
//!   in linear time (each glyph is looked at once), from the same parsed glyph records
//!   the library walks; a glyph above [`MAX_GLYPH_COST`] is not drawn;
//! * for `CFF `/`CFF2` fonts the library's charstring interpreter gets its subroutines
//!   through a metered context (see [`crate::cff`]) and cannot run longer than the
//!   budget allows;
//! * every [`Outliner`] — one per `to_outline`, `rasterize` or font load — has an overall
//!   budget of [`MAX_RUN_COST`], so that a huge text cannot multiply a merely expensive
//!   glyph into minutes of work or gigabytes of path data.
//!
//! A `glyf` glyph within [`MAX_GLYPH_COST`] but beyond the library's own limits (more
//! than 2 048 component references, or nested deeper than 64 levels) is still drawn: its
//! component tree is expanded here, down to sub-trees the library accepts. Only hostile
//! or synthetic fonts have such glyphs; on that path the component offsets are those of
//! the default instance (no `gvar` deltas for the expanded levels), anchors given as
//! point numbers count as a zero offset and `SCALED_COMPONENT_OFFSET` is ignored.
//!
//! The analysis and the library read the font through the same table collection
//! ([`crate::sfnt::Tables`]): there is no way for them to look at different bytes.

use std::collections::BTreeMap;
use std::num::NonZeroU16;

use read_fonts::TableProvider as _;
use read_fonts::limits::{MAX_COMPOSITE_EDGES, MAX_RECURSION_DEPTH};
use read_fonts::model::glyph::outline::{
    DrawMemory, DrawOptions, NullSink, OutlineGlyphs, PathContourStart, PathSink,
};
use read_fonts::model::glyph::{GlyphSource, GlyphSourceOrder};
use read_fonts::model::{Error, Font as Face};
use read_fonts::tables::glyf::{Anchor, Component, Glyf, Glyph};
use read_fonts::tables::loca::{Loca, LocaGlyph};
use read_fonts::types::GlyphId;

use crate::cff::{CffOutlines, MAX_CHARSTRING_STEPS, Stop};
use crate::sfnt::Tables;

/// Largest expanded size of one TrueType glyph: component references plus points over
/// its whole component tree. A glyph cannot legitimately have more than 65 535 points
/// (point numbers are 16-bit).
pub(crate) const MAX_GLYPH_COST: u32 = 1 << 16;

/// Largest glyph cost at which glyph outlines stay available to the *shaper* of a
/// variable font without `HVAR`/`VVAR` tables — the one situation in which shaping
/// itself looks at glyph outlines (to derive advances), once per glyph of the text.
pub(crate) const MAX_SHAPING_GLYPH_COST: u32 = 1 << 12;

/// Overall budget of one [`Outliner`], in the units of [`MAX_GLYPH_COST`] (points and
/// component references) or charstring bytes: about 20 000 average letters, or 3 000
/// complex ideographs, per call — and at most some 70 MB of path data.
pub(crate) const MAX_RUN_COST: u64 = 1 << 20;

/// Most path segments one glyph may contribute (a contour of `n` points yields at most
/// `n + 2` segments, so this is beyond any real glyph).
pub(crate) const MAX_GLYPH_SEGMENTS: usize = 2 * 65_536;

/// The only outline source the library is asked to draw: `glyf`. (Charstrings are run
/// by [`crate::cff`]; `VARC` composites are not supported.)
const GLYF_ONLY: GlyphSourceOrder = GlyphSourceOrder::new(&[GlyphSource::Glyf]);

/// Why a glyph was left out of an outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// The glyph on its own is beyond [`MAX_GLYPH_COST`] / the charstring budget.
    GlyphTooComplex(u16),
    /// The glyph would have been fine, but the run's overall budget is used up.
    BudgetExhausted(u16),
}

/// Whether the shaper must not see the `glyf` outlines of a font whose most expensive
/// glyph costs `max_glyph_cost`.
pub(crate) fn glyf_must_be_hidden(tables: &Tables, max_glyph_cost: u32) -> bool {
    let outlines_while_shaping =
        tables.has(b"fvar") && !tables.has(b"HVAR") && !tables.has(b"VVAR");
    max_glyph_cost > MAX_GLYPH_COST
        || (outlines_while_shaping && max_glyph_cost > MAX_SHAPING_GLYPH_COST)
}

/// The control box of a glyph outline in font units, as far as this crate needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlyphBox {
    /// The highest coordinate of any point or control point.
    pub(crate) y_max: i16,
}

// ---------------------------------------------------------------------------------------
// TrueType: the expanded size of composite glyphs
// ---------------------------------------------------------------------------------------

/// What drawing a `glyf` glyph involves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphInfo {
    /// The expanded size: 1 for the glyph itself, plus its points, plus one per
    /// component reference, plus the cost of each component. [`GlyfCosts::IN_PROGRESS`]
    /// while the components are being added up; `u32::MAX` (saturated) for a glyph that
    /// contains itself.
    cost: u32,
    /// Component references in the expanded tree that lead to a non-empty glyph — what
    /// the library counts against its limit of 2 048.
    edges: u32,
    /// Levels of non-empty glyphs below this one (0 for a simple glyph).
    height: u32,
    /// The glyph has no outline data (a space).
    empty: bool,
    /// The glyph, or one of its components, cannot be read: the library draws nothing.
    broken: bool,
}

impl GlyphInfo {
    /// The library draws this glyph in one go (see [`read_fonts::limits`]).
    fn within_library_limits(&self) -> bool {
        self.edges as usize <= MAX_COMPOSITE_EDGES && self.height as usize <= MAX_RECURSION_DEPTH
    }
}

/// Computes glyph costs from the `glyf` and `loca` tables, memoised.
pub(crate) struct GlyfCosts<'a> {
    glyf: Glyf<'a>,
    loca: Loca<'a>,
    memo: BTreeMap<u32, GlyphInfo>,
}

/// A glyph whose components are being added up.
struct Frame {
    glyph: u32,
    components: Vec<u32>,
    next: usize,
    info: GlyphInfo,
}

impl<'a> GlyfCosts<'a> {
    /// Marks a glyph on the expansion stack. Real costs are at least 1.
    const IN_PROGRESS: u32 = 0;

    /// `None` for a font without usable `glyf`/`loca` tables.
    pub(crate) fn new(face: &'a Face) -> Option<Self> {
        // The very tables the library draws from.
        let (glyf, loca) = face.tables().selected_glyf_loca(None).ok()?;
        Some(Self { glyf, loca, memo: BTreeMap::new() })
    }

    /// The largest cost of any of the font's first `glyph_count` glyphs.
    pub(crate) fn max_cost(&mut self, glyph_count: NonZeroU16) -> u32 {
        (0..glyph_count.get()).map(|glyph| self.cost(glyph)).max().unwrap_or(1)
    }

    /// The expanded size of a glyph: 1 for the glyph itself, plus its points, plus the
    /// cost of each of its components — an upper bound of the work it takes to outline
    /// it. `u32::MAX` (saturated) for a glyph that contains itself.
    pub(crate) fn cost(&mut self, glyph: u16) -> u32 {
        self.info(u32::from(glyph)).cost
    }

    fn info(&mut self, glyph: u32) -> GlyphInfo {
        if let Some(&known) = self.memo.get(&glyph) {
            let cost = if known.cost == Self::IN_PROGRESS { u32::MAX } else { known.cost };
            return GlyphInfo { cost, ..known };
        }
        // Depth-first over the component graph with an explicit stack: a chain of
        // components can be as long as the font has glyphs.
        let mut stack = vec![self.enter(glyph)];
        let mut result = None;
        while let Some(top) = stack.last_mut() {
            if let Some(&component) = top.components.get(top.next) {
                top.next += 1;
                match self.memo.get(&component) {
                    // A cycle: the library gives up on such glyphs at its depth limit.
                    Some(known) if known.cost == Self::IN_PROGRESS => top.info.cost = u32::MAX,
                    Some(&known) => top.info.add_component(known),
                    None => {
                        let frame = self.enter(component);
                        stack.push(frame);
                    }
                }
                continue;
            }
            let (finished, info) = (top.glyph, top.info);
            stack.pop();
            self.memo.insert(finished, info);
            match stack.last_mut() {
                Some(parent) => parent.info.add_component(info),
                None => result = Some(info),
            }
        }
        result.unwrap_or(GlyphInfo {
            cost: u32::MAX,
            edges: u32::MAX,
            height: u32::MAX,
            empty: false,
            broken: true,
        })
    }

    /// The glyph record, read exactly as the library reads it.
    fn read(&self, glyph: u32) -> Option<LocaGlyph<'a>> {
        self.loca.get(GlyphId::new(glyph), &self.glyf)
    }

    /// Reads a glyph's header: its own cost and the glyphs it is composed of.
    fn enter(&mut self, glyph: u32) -> Frame {
        let mut info =
            GlyphInfo { cost: Self::IN_PROGRESS, edges: 0, height: 0, empty: false, broken: false };
        self.memo.insert(glyph, info);
        info.cost = 1;
        let mut components = Vec::new();
        match self.read(glyph) {
            None => info.broken = true,
            Some(LocaGlyph::Empty) => info.empty = true,
            Some(LocaGlyph::Glyph(Glyph::Simple(simple))) => {
                let points = u32::try_from(simple.num_points()).unwrap_or(u32::MAX);
                info.cost = info.cost.saturating_add(points);
            }
            Some(LocaGlyph::Glyph(Glyph::Composite(composite))) => {
                // The iterator the library uses to walk the component tree.
                components.extend(
                    composite.component_glyphs_and_flags().map(|(glyph, _)| glyph.to_u32()),
                );
                let references = u32::try_from(components.len()).unwrap_or(u32::MAX);
                info.cost = info.cost.saturating_add(references);
            }
        }
        Frame { glyph, components, next: 0, info }
    }
}

impl GlyphInfo {
    /// Accounts for one reference to a component (the reference itself is part of the
    /// composite's own cost).
    fn add_component(&mut self, component: GlyphInfo) {
        self.cost = self.cost.saturating_add(component.cost);
        if component.broken {
            self.broken = true;
        } else if !component.empty {
            self.edges = self.edges.saturating_add(component.edges).saturating_add(1);
            self.height = self.height.max(component.height.saturating_add(1));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Sinks
// ---------------------------------------------------------------------------------------

/// The last command of the contour being drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Last {
    /// A move or a line: the contour stands at one of its on-curve points.
    Straight,
    /// A curve.
    Curve,
}

/// Forwards an outline, measures its control box and — for TrueType outlines — closes
/// every contour with an explicit line back to its first point.
///
/// The library ends a contour whose last point is on the curve with a bare "close"; the
/// paths of this crate have always carried the closing line itself (a consumer that
/// strokes or offsets them need not know that "close" draws one). A contour that ends
/// with a curve arriving at its first point needs no line.
struct Measured<'s> {
    inner: &'s mut dyn PathSink,
    /// `(x_min, y_min, x_max, y_max)` of every coordinate seen so far.
    bounds: Option<(f32, f32, f32, f32)>,
    closing_lines: bool,
    /// The contour being drawn.
    contour: Option<Contour>,
}

/// An open contour: its first point, where it stands and how it got there.
#[derive(Debug, Clone, Copy)]
struct Contour {
    first: (f32, f32),
    current: (f32, f32),
    last: Last,
}

impl<'s> Measured<'s> {
    fn new(inner: &'s mut dyn PathSink, closing_lines: bool) -> Self {
        Self { inner, bounds: None, closing_lines, contour: None }
    }

    fn see(&mut self, x: f32, y: f32) {
        self.bounds = Some(match self.bounds {
            None => (x, y, x, y),
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        });
    }

    fn arrive(&mut self, x: f32, y: f32, last: Last) {
        self.see(x, y);
        if let Some(contour) = &mut self.contour {
            contour.current = (x, y);
            contour.last = last;
        }
    }

    /// The box of what was drawn; `None` if nothing was drawn or if the outline does
    /// not fit 16-bit coordinates (no glyph of a real font is that large).
    fn finish(&self) -> Option<GlyphBox> {
        let (x_min, y_min, x_max, y_max) = self.bounds?;
        let fits = |v: f32| (f32::from(i16::MIN)..=f32::from(i16::MAX)).contains(&v);
        (fits(x_min) && fits(y_min) && fits(x_max) && fits(y_max))
            .then_some(GlyphBox { y_max: y_max as i16 })
    }
}

impl PathSink for Measured<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.see(x, y);
        self.contour = Some(Contour { first: (x, y), current: (x, y), last: Last::Straight });
        self.inner.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.arrive(x, y, Last::Straight);
        self.inner.line_to(x, y);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.see(cx0, cy0);
        self.arrive(x, y, Last::Curve);
        self.inner.quad_to(cx0, cy0, x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.see(cx0, cy0);
        self.see(cx1, cy1);
        self.arrive(x, y, Last::Curve);
        self.inner.curve_to(cx0, cy0, cx1, cy1, x, y);
    }

    fn close(&mut self) {
        let closing_line = self.contour.take().filter(|contour| {
            self.closing_lines
                && (contour.last == Last::Straight || contour.current != contour.first)
        });
        if let Some(contour) = closing_line {
            self.inner.line_to(contour.first.0, contour.first.1);
        }
        self.inner.close();
    }
}

/// The placement of a component: `x' = xx·x + xy·y + dx`, `y' = yx·x + yy·y + dy`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Placement {
    xx: f32,
    xy: f32,
    yx: f32,
    yy: f32,
    dx: f32,
    dy: f32,
}

impl Placement {
    const IDENTITY: Self = Self { xx: 1.0, xy: 0.0, yx: 0.0, yy: 1.0, dx: 0.0, dy: 0.0 };

    /// The placement of a component record inside its composite glyph.
    fn of(component: &Component) -> Self {
        let (dx, dy) = match component.anchor {
            Anchor::Offset { x, y } => (f32::from(x), f32::from(y)),
            // Point-matched anchors are not resolved on this path (see the module docs).
            Anchor::Point { .. } => (0.0, 0.0),
        };
        let t = component.transform;
        Self { xx: t.xx.to_f32(), xy: t.xy.to_f32(), yx: t.yx.to_f32(), yy: t.yy.to_f32(), dx, dy }
    }

    fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (self.xx * x + self.xy * y + self.dx, self.yx * x + self.yy * y + self.dy)
    }

    /// The placement of something placed by `inner` inside a glyph placed by `self`.
    fn then(&self, inner: &Self) -> Self {
        let (dx, dy) = self.apply(inner.dx, inner.dy);
        Self {
            xx: self.xx * inner.xx + self.xy * inner.yx,
            xy: self.xx * inner.xy + self.xy * inner.yy,
            yx: self.yx * inner.xx + self.yy * inner.yx,
            yy: self.yx * inner.xy + self.yy * inner.yy,
            dx,
            dy,
        }
    }
}

/// Forwards an outline through a [`Placement`].
struct Placed<'s> {
    inner: &'s mut dyn PathSink,
    placement: Placement,
}

impl PathSink for Placed<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.placement.apply(x, y);
        self.inner.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.placement.apply(x, y);
        self.inner.line_to(x, y);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let (cx0, cy0) = self.placement.apply(cx0, cy0);
        let (x, y) = self.placement.apply(x, y);
        self.inner.quad_to(cx0, cy0, x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let (cx0, cy0) = self.placement.apply(cx0, cy0);
        let (cx1, cy1) = self.placement.apply(cx1, cy1);
        let (x, y) = self.placement.apply(x, y);
        self.inner.curve_to(cx0, cy0, cx1, cy1, x, y);
    }

    fn close(&mut self) {
        self.inner.close();
    }
}

// ---------------------------------------------------------------------------------------
// The outliner
// ---------------------------------------------------------------------------------------

enum Outlines<'a> {
    TrueType(GlyfCosts<'a>, OutlineGlyphs<'a>),
    PostScript(Box<CffOutlines<'a>>, bool),
    None,
}

/// Draws glyph outlines of one face under a budget; see the module documentation.
pub(crate) struct Outliner<'a> {
    /// The face (at its variation instance) the outlines are taken from.
    face: &'a Face,
    outlines: Outlines<'a>,
    remaining: u64,
    refusal: Option<Refusal>,
}

/// How a `glyf` glyph went.
enum Drawn {
    Yes,
    /// Malformed, or missing.
    No,
    /// The library refused it for being too large.
    TooComplex,
}

impl<'a> Outliner<'a> {
    /// An outliner for `face`, which must be a view of `tables`.
    pub(crate) fn new(face: &'a Face, tables: &'a Tables) -> Self {
        let units_per_em = tables.units_per_em();
        // The precedence when a font has several kinds of outlines: glyf, CFF, CFF2.
        let outlines = if let Some(costs) = GlyfCosts::new(face) {
            Outlines::TrueType(costs, face.outline_glyphs_with(GLYF_ONLY))
        } else if let Some(font) =
            tables.get(b"CFF ").and_then(|table| CffOutlines::parse_cff(table, units_per_em))
        {
            Outlines::PostScript(Box::new(font), false)
        } else if let Some(font) =
            tables.get(b"CFF2").and_then(|table| CffOutlines::parse_cff2(table, units_per_em))
        {
            Outlines::PostScript(Box::new(font), true)
        } else {
            Outlines::None
        };
        Self { face, outlines, remaining: MAX_RUN_COST, refusal: None }
    }

    /// The kind of outlines that are drawn.
    pub(crate) fn format(&self) -> crate::font::OutlineFormat {
        use crate::font::OutlineFormat;
        match self.outlines {
            Outlines::TrueType(..) => OutlineFormat::TrueType,
            Outlines::PostScript(_, false) => OutlineFormat::Cff,
            Outlines::PostScript(_, true) => OutlineFormat::Cff2,
            Outlines::None => OutlineFormat::None,
        }
    }

    /// The first glyph that was refused, if any, and why.
    pub(crate) fn refusal(&self) -> Option<Refusal> {
        self.refusal
    }

    /// Sends the outline of `glyph`, in font units, to `sink` and returns its control
    /// box. `None` if the glyph has no outline, is malformed, or was refused (see
    /// [`refusal`](Self::refusal)); `sink` may then have received part of an outline.
    pub(crate) fn outline(&mut self, glyph: u16, sink: &mut dyn PathSink) -> Option<GlyphBox> {
        let truetype = matches!(self.outlines, Outlines::TrueType(..));
        let mut measured = Measured::new(sink, truetype);
        match &mut self.outlines {
            Outlines::None => None,
            Outlines::TrueType(costs, glyphs) => {
                let info = costs.info(u32::from(glyph));
                if info.cost > MAX_GLYPH_COST {
                    self.refusal.get_or_insert(Refusal::GlyphTooComplex(glyph));
                    return None;
                }
                if u64::from(info.cost) > self.remaining {
                    self.refusal.get_or_insert(Refusal::BudgetExhausted(glyph));
                    return None;
                }
                self.remaining -= u64::from(info.cost);
                match draw_glyf(costs, glyphs, u32::from(glyph), &mut measured) {
                    Drawn::Yes => measured.finish(),
                    Drawn::No => None,
                    Drawn::TooComplex => {
                        self.refusal.get_or_insert(Refusal::GlyphTooComplex(glyph));
                        None
                    }
                }
            }
            Outlines::PostScript(font, _) => {
                let allowed =
                    u32::try_from(self.remaining).unwrap_or(u32::MAX).min(MAX_CHARSTRING_STEPS);
                let mut steps = allowed;
                let coords = self.face.normalized_coords();
                let outcome = font.outline(glyph, coords, &mut steps, &mut measured);
                // One unit per glyph on top of the steps, so that glyphs without any
                // operator are not free.
                let used = u64::from(allowed - steps) + 1;
                self.remaining = self.remaining.saturating_sub(used);
                match outcome {
                    Ok(()) => measured.finish(),
                    Err(Stop::Invalid) => None,
                    Err(Stop::TooComplex) => {
                        let refusal = if allowed < MAX_CHARSTRING_STEPS {
                            Refusal::BudgetExhausted(glyph)
                        } else {
                            Refusal::GlyphTooComplex(glyph)
                        };
                        self.refusal.get_or_insert(refusal);
                        None
                    }
                }
            }
        }
    }

    /// The control box of a glyph's outline in font units (the outline is computed and
    /// discarded).
    pub(crate) fn bounding_box(&mut self, glyph: u16) -> Option<GlyphBox> {
        self.outline(glyph, &mut NullSink)
    }
}

/// Draws a `glyf` glyph whose cost has been accepted (hence: a finite tree of at most
/// [`MAX_GLYPH_COST`] references and points).
fn draw_glyf(
    costs: &mut GlyfCosts<'_>,
    glyphs: &OutlineGlyphs<'_>,
    root: u32,
    sink: &mut dyn PathSink,
) -> Drawn {
    // A contour whose first point is off the curve starts at its second point (or half
    // way to it), as it always has in this crate; the library's default starts at the
    // last point.
    //
    // Drawn "at a size" of one pixel per font unit: the coordinates are still font
    // units, but the library then works in 1/64 of a unit instead of whole units, which
    // matters where it computes (scaled or rotated components, variation deltas).
    let options = DrawOptions::new()
        .with_contour_start(PathContourStart::ScanForward)
        .with_ppem(Some(f32::from(glyphs.font().units_per_em())));
    // Glyphs still to draw, last first. All real glyphs are drawn by the library in one
    // go; a glyph beyond its limits is replaced by its components, each popped once.
    let mut pending = vec![(root, Placement::IDENTITY)];
    while let Some((glyph, placement)) = pending.pop() {
        let info = costs.info(glyph);
        if info.broken {
            return Drawn::No;
        }
        if info.empty {
            continue;
        }
        if info.within_library_limits() {
            let Some(outline) = glyphs.get(GlyphId::new(glyph)) else { return Drawn::No };
            let result = if placement == Placement::IDENTITY {
                outline.draw(&options, DrawMemory::Scratch, sink)
            } else {
                let mut placed = Placed { inner: &mut *sink, placement };
                outline.draw(&options, DrawMemory::Scratch, &mut placed)
            };
            match result {
                Ok(_) => {}
                Err(Error::LimitExceeded) => return Drawn::TooComplex,
                Err(_) => return Drawn::No,
            }
        } else {
            let Some(LocaGlyph::Glyph(Glyph::Composite(composite))) = costs.read(glyph) else {
                return Drawn::No;
            };
            let first = pending.len();
            pending.extend(composite.components().map(|component| {
                (component.glyph.to_u32(), placement.then(&Placement::of(&component)))
            }));
            if let Some(added) = pending.get_mut(first..) {
                added.reverse(); // the first component is drawn first
            }
        }
    }
    Drawn::Yes
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    /// `loca` (long format) and `glyf` for the given glyph programs.
    fn glyf_tables(glyphs: &[Vec<u8>]) -> (Vec<u8>, Vec<u8>) {
        let (mut loca, mut glyf) = (Vec::new(), Vec::new());
        for glyph in glyphs {
            loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
            glyf.extend_from_slice(glyph); // unpadded: the long format allows any offset
        }
        loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
        (loca, glyf)
    }

    fn composite(components: &[u16]) -> Vec<u8> {
        let mut glyph = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0];
        for (i, component) in components.iter().enumerate() {
            let more = if i + 1 < components.len() { 0x20 } else { 0 };
            glyph.extend_from_slice(&[0, 0x02 | more]);
            glyph.extend_from_slice(&component.to_be_bytes());
            glyph.extend_from_slice(&[0, 0]);
        }
        glyph
    }

    /// A simple glyph header claiming `points` points in one contour (no point data:
    /// the cost only reads the header, which ends with the instruction count).
    fn simple(points: u16) -> Vec<u8> {
        let mut glyph = vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        glyph.extend_from_slice(&(points - 1).to_be_bytes());
        glyph.extend_from_slice(&[0, 0]);
        glyph
    }

    /// A face with the given glyph programs.
    fn face_of(glyphs: &[Vec<u8>]) -> (Arc<Tables>, Face) {
        let (loca, glyf) = glyf_tables(glyphs);
        let mut head = vec![0u8; 54];
        head[18..20].copy_from_slice(&1000u16.to_be_bytes());
        head[50..52].copy_from_slice(&1u16.to_be_bytes()); // long loca
        let mut maxp = vec![0, 0, 0x50, 0];
        maxp.extend_from_slice(&(glyphs.len() as u16).to_be_bytes());
        let tables: [(&[u8; 4], Vec<u8>); 5] = [
            (b"glyf", glyf),
            (b"head", head),
            (b"hhea", vec![0; 36]),
            (b"loca", loca),
            (b"maxp", maxp),
        ];
        let mut font = vec![0, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0];
        let mut offset = 12 + 16 * tables.len();
        for (tag, data) in &tables {
            font.extend_from_slice(*tag);
            font.extend_from_slice(&[0; 4]);
            font.extend_from_slice(&(offset as u32).to_be_bytes());
            font.extend_from_slice(&(data.len() as u32).to_be_bytes());
            offset += data.len();
        }
        for (_, data) in &tables {
            font.extend_from_slice(data);
        }
        let tables = Arc::new(Tables::collect(&Arc::from(font), 0).expect("a usable font"));
        let face = tables.face().expect("a face");
        (tables, face)
    }

    fn costs_of(glyphs: &[Vec<u8>]) -> Vec<u32> {
        let (_tables, face) = face_of(glyphs);
        let mut costs = GlyfCosts::new(&face).expect("usable tables");
        (0..glyphs.len() as u16).map(|glyph| costs.cost(glyph)).collect()
    }

    #[test]
    fn costs_add_up_over_the_component_tree() {
        let costs = costs_of(&[
            Vec::new(),            // 0: empty
            simple(4),             // 1: 1 + 4 points
            composite(&[1, 1, 0]), // 2: 1 + 3 references + 5 + 5 + 1
            composite(&[2, 2]),    // 3: 1 + 2 + 15 + 15
            composite(&[3, 9, 1]), // 4: 1 + 3 + 33 + 1 (no such glyph) + 5
        ]);
        assert_eq!(costs, [1, 5, 15, 33, 43]);
    }

    #[test]
    fn edges_and_heights_are_what_the_library_counts() {
        let glyphs = [
            Vec::new(),            // 0: empty
            simple(4),             // 1
            composite(&[1, 1, 0]), // 2: two references to a non-empty glyph
            composite(&[2, 2]),    // 3: 2 + 2 x 2
            composite(&[3, 9, 1]), // 4: refers to a glyph that does not exist
            composite(&[0, 0]),    // 5: only empty components
        ];
        let (_tables, face) = face_of(&glyphs);
        let mut costs = GlyfCosts::new(&face).expect("usable tables");
        let seen: Vec<(u32, u32, bool, bool)> = (0..6)
            .map(|glyph| costs.info(glyph))
            .map(|info| (info.edges, info.height, info.empty, info.broken))
            .collect();
        assert_eq!(
            seen,
            [
                (0, 0, true, false),
                (0, 0, false, false),
                (2, 1, false, false),
                (6, 2, false, false),
                (8, 3, false, true),
                (0, 0, false, false),
            ]
        );
        assert!(costs.info(3).within_library_limits());
    }

    #[test]
    fn exponential_trees_cost_linear_time_and_saturate() {
        // 40 levels of fan-out 4 over an empty leaf: 4^40 references.
        let mut glyphs = vec![Vec::new()];
        for level in 1..=40u16 {
            glyphs.push(composite(&[level - 1; 4]));
        }
        let costs = costs_of(&glyphs);
        assert_eq!(costs[1], 1 + 4 + 4);
        assert_eq!(costs[2], 1 + 4 + 4 * 9);
        assert_eq!(costs[40], u32::MAX);
        assert!(costs[8] > MAX_GLYPH_COST);
    }

    #[test]
    fn cycles_and_very_long_chains_are_handled_without_recursion() {
        // 0 -> 1 -> 2 -> 0, and 3 -> 0.
        let costs = costs_of(&[composite(&[1]), composite(&[2]), composite(&[0]), composite(&[0])]);
        assert_eq!(costs, [u32::MAX; 4]);
        // A chain through 60 000 glyphs is fine (and cheap).
        let mut glyphs = vec![simple(3)];
        for glyph in 1..60_000u16 {
            glyphs.push(composite(&[glyph - 1]));
        }
        let costs = costs_of(&glyphs);
        assert_eq!(costs[0], 4);
        assert_eq!(costs[59_999], 4 + 2 * 59_999);
    }

    #[test]
    fn truncated_and_odd_component_records_do_not_panic() {
        let mut odd = composite(&[1, 1]);
        odd.truncate(odd.len() - 1); // the last record lacks a byte of its offsets
        let words = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03, 0, 1, 0, 0, 0, 0];
        let no_offsets = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x20, 0, 1, 0, 0x00, 0, 1];
        let costs =
            costs_of(&[Vec::new(), simple(2), odd, words, no_offsets, vec![0xFF], vec![0, 5]]);
        // The library's tree walk reads flags and glyph id of a record and skips the
        // rest unchecked: a record cut short after its glyph id still names a component.
        assert_eq!(costs[2], 1 + 2 + 3 + 3, "both records name a component");
        assert_eq!(costs[3], 1 + 1 + 3);
        // Arguments that are point numbers are skipped like offsets; what follows them
        // here is the start of a record without a glyph id.
        assert_eq!(costs[4], 1 + 1 + 3);
        assert_eq!((costs[5], costs[6]), (1, 1));
    }

    /// The outline of a glyph as a list of commands.
    fn drawn(outliner: &mut Outliner<'_>, glyph: u16) -> Option<Vec<String>> {
        #[derive(Default)]
        struct Trace(Vec<String>);
        impl PathSink for Trace {
            fn move_to(&mut self, x: f32, y: f32) {
                self.0.push(format!("M {x} {y}"));
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.0.push(format!("L {x} {y}"));
            }
            fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
                self.0.push(format!("Q {cx0} {cy0} {x} {y}"));
            }
            fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
                self.0.push("C".into());
            }
            fn close(&mut self) {
                self.0.push("Z".into());
            }
        }
        let mut trace = Trace::default();
        outliner.outline(glyph, &mut trace).map(|_| trace.0)
    }

    /// A triangle (0,0) (0,10) (10,0) as a complete simple glyph.
    fn triangle() -> Vec<u8> {
        let mut glyph = vec![0, 1, 0, 0, 0, 0, 0, 10, 0, 10];
        glyph.extend_from_slice(&[0, 2, 0, 0]); // last point, no instructions
        // On-curve points with one-byte coordinates; the last y delta is negative.
        glyph.extend_from_slice(&[0x37, 0x37, 0x17]);
        glyph.extend_from_slice(&[0, 0, 10]); // x deltas
        glyph.extend_from_slice(&[0, 10, 10]); // y deltas (magnitudes)
        glyph
    }

    /// A composite of the given `(glyph, dx, dy)` components, with a scale of 0.5 on the
    /// components marked so.
    fn placed(components: &[(u16, i8, i8, bool)]) -> Vec<u8> {
        let mut glyph = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0];
        for (i, &(component, dx, dy, half)) in components.iter().enumerate() {
            let more = if i + 1 < components.len() { 0x20 } else { 0 };
            let scale = if half { 0x08 } else { 0 };
            glyph.extend_from_slice(&[0, 0x02 | more | scale]);
            glyph.extend_from_slice(&component.to_be_bytes());
            glyph.extend_from_slice(&[dx as u8, dy as u8]);
            if half {
                glyph.extend_from_slice(&[0x20, 0x00]); // F2Dot14 0.5
            }
        }
        glyph
    }

    #[test]
    fn glyphs_beyond_the_library_limits_are_expanded_here_and_drawn_the_same() {
        // Glyph 2 places two triangles; glyph 3 nests glyph 2, moved and scaled.
        let small = [
            Vec::new(),
            triangle(),
            placed(&[(1, 0, 0, false), (1, 20, 6, false)]),
            placed(&[(2, 100, 50, true), (1, -7, 3, false)]),
        ];
        let (tables, face) = face_of(&small);
        let mut outliner = Outliner::new(&face, &tables);
        let reference = drawn(&mut outliner, 3).expect("drawn by the library");
        assert_eq!(reference.len(), 3 * 5, "three triangles: move, two lines, line home, close");
        assert_eq!(&reference[..5], ["M 100 50", "L 100 55", "L 105 50", "L 100 50", "Z"]);
        assert_eq!(&reference[5..10], ["M 110 53", "L 110 58", "L 115 53", "L 110 53", "Z"]);
        assert_eq!(&reference[10..], ["M -7 3", "L -7 13", "L 3 3", "L -7 3", "Z"]);

        // The same tree, expanded by `draw_glyf` because every level counts as "too
        // large": compare against the library's own rendering of glyph 3.
        let GlyfCostsAndGlyphs(mut costs, glyphs) = costs_and_glyphs(&face);
        for glyph in 0..4 {
            let _ = costs.info(glyph);
        }
        for info in costs.memo.values_mut() {
            if info.edges > 0 {
                info.edges = u32::MAX; // forces the expansion of every composite
            }
        }
        let mut expanded = Vec::new();
        struct Collect<'v>(&'v mut Vec<String>);
        impl PathSink for Collect<'_> {
            fn move_to(&mut self, x: f32, y: f32) {
                self.0.push(format!("M {x} {y}"));
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.0.push(format!("L {x} {y}"));
            }
            fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
                self.0.push(format!("Q {cx0} {cy0} {x} {y}"));
            }
            fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
                self.0.push("C".into());
            }
            fn close(&mut self) {
                self.0.push("Z".into());
            }
        }
        let mut collect = Collect(&mut expanded);
        let mut measured = Measured::new(&mut collect, true);
        assert!(matches!(draw_glyf(&mut costs, &glyphs, 3, &mut measured), Drawn::Yes));
        assert_eq!(expanded, reference);
    }

    struct GlyfCostsAndGlyphs<'a>(GlyfCosts<'a>, OutlineGlyphs<'a>);

    fn costs_and_glyphs(face: &Face) -> GlyfCostsAndGlyphs<'_> {
        GlyfCostsAndGlyphs(
            GlyfCosts::new(face).expect("usable tables"),
            face.outline_glyphs_with(GLYF_ONLY),
        )
    }

    #[test]
    fn a_glyph_with_more_references_than_the_library_follows_is_still_drawn_in_full() {
        // Glyph 2 is 50 triangles, glyph 3 is 50 x glyph 2: 2 550 references to
        // non-empty glyphs, beyond the library's 2 048, within our 65 536.
        let fifty = |glyph: u16| placed(&vec![(glyph, 1, 0, false); 50]);
        let glyphs = [Vec::new(), triangle(), fifty(1), fifty(2), fifty(3), fifty(4)];
        let (tables, face) = face_of(&glyphs);
        let mut outliner = Outliner::new(&face, &tables);
        let one = drawn(&mut outliner, 1).expect("a triangle").len();
        assert_eq!(drawn(&mut outliner, 2).expect("drawn").len(), 50 * one);
        let big = drawn(&mut outliner, 3).expect("expanded and drawn");
        assert_eq!(big.len(), 2_500 * one);
        assert_eq!(outliner.refusal(), None);
        // 125 000 triangles are 375 000 points: refused, and reported.
        assert_eq!(drawn(&mut outliner, 4), None);
        assert_eq!(outliner.refusal(), Some(Refusal::GlyphTooComplex(4)));
        // The budget of the outliner went down by the cost of what was drawn.
        let spent = MAX_RUN_COST - outliner.remaining;
        assert_eq!(spent, 4 + (1 + 50 + 50 * 4) + (1 + 50 + 50 * 251));
    }
}
