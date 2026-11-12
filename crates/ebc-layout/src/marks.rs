//! Printer's marks: crop marks, fold marks, registration targets, colour bar, scale ruler.
//!
//! Every mark lies inside [`PageGeometry::mark_area`](crate::PageGeometry::mark_area) and
//! outside the bleed box. The marks are the same for both faces of the card (the spread
//! is symmetric, so the creases are at the same x on the outside and the inside).
//!
//! Conventions used (`docs/research/03-print-requirements.md` §2.4; mark geometry is DTP
//! convention — no surveyed online printer publishes one, most of them refuse marks):
//! butt caps; crop marks **solid**, along the extension of each trim edge; fold marks
//! **dashed**, along the extension of each crease, so that a finisher cannot take a crease
//! for a cut; nothing closer to the trim than the bleed.
//!
//! # Marks and preflight
//!
//! Two rules keep the marks unambiguous for a preflight tool, which judges one painted
//! object at a time by its bounding box and cannot tell a butt cap from a projecting one:
//!
//! * **One object per mark.** Each crop mark, each fold mark and each registration target
//!   is its own [`Item`]. Eight crop marks in one path would have a bounding box as large
//!   as the page — "registration colour inside the bleed box" as far as such a tool can
//!   see.
//! * **A gap after the bleed.** Crop and fold marks start [`MARK_BLEED_GAP_MM`] beyond the
//!   bleed edge, not on it, so that no object in the registration colour touches the
//!   bleed box even when half a stroke width is added around its end points. The other
//!   furniture (targets, colour bar, ruler, caption areas) keeps the same distance.

use ebc_scene::{Color, Item, Path, Point, Rect, Size, Stroke, pt_to_mm};

use crate::error::LayoutError;
use crate::format::CardFormat;
use crate::page::page_geometry;
use crate::profile::{MarkStyle, Marks, PrintProfile};

/// Name of the layer returned by [`marks`].
pub const MARKS_LAYER: &str = "marks";
/// Clear space between the bleed edge and the start of crop and fold marks, in
/// millimetres.
///
/// Prepress convention keeps marks off the bleed: a mark that starts exactly on the bleed
/// edge is, for a preflight tool that adds half the stroke width around a path (it does
/// not know the cap is butt), an object in the registration colour that reaches into the
/// bleed box. Half a millimetre is more than half the heaviest stroke a profile may ask
/// for ([`MAX_MARK_WEIGHT_PT`](crate::MAX_MARK_WEIGHT_PT): 2 pt, i.e. 0.35 mm), costs the
/// finisher nothing, and is the distance all other furniture keeps from the bleed edge.
///
/// The effective start of a mark is `max(marks.offset, bleed + MARK_BLEED_GAP_MM)` from
/// the trim: a profile whose offset already clears the bleed by more is honoured as is.
pub const MARK_BLEED_GAP_MM: f64 = 0.5;
/// Crop and fold marks that would end up shorter than this (because the slug or the
/// printable area is too small) are omitted rather than drawn as specks
/// ([`check_marks_fit`] reports it).
pub const MIN_MARK_LENGTH_MM: f64 = 2.0;
/// Dash pattern (on, off — millimetres) of fold marks. A fold mark shorter than 5 mm uses
/// the same pattern scaled down so that it still shows three dashes.
pub const FOLD_MARK_DASH_MM: [f64; 2] = [1.0, 1.0];
/// Longest fold mark of [`MarkStyle::Guide`], measured from its start, in millimetres: on
/// a sheet much larger than the card the dashed guide stops here instead of running all
/// the way to the printable edge.
pub const FOLD_GUIDE_MAX_MM: f64 = 20.0;
/// Registration targets, the colour bar, the scale ruler and the caption areas use at most
/// this much of the margin, measured from the bleed edge, in millimetres — so that they
/// hug the card instead of floating in the middle of a large sheet.
pub const FURNITURE_BAND_MAX_MM: f64 = 12.0;

/// Half the width of a registration target's cross, in millimetres, when there is room.
const TARGET_HALF_MM: f64 = 3.5;
/// Registration targets that would have to be smaller than this are omitted.
const TARGET_MIN_HALF_MM: f64 = 1.5;
/// Radius of the target's circle as a fraction of the cross's half-width.
const TARGET_CIRCLE_RATIO: f64 = 0.57;
/// Clear space kept between a target and the bleed edge, the mark-area edge and the fold
/// marks, in millimetres.
const TARGET_CLEARANCE_MM: f64 = 0.5;

/// Process-colour patches of the colour bar as `[c, m, y, k]`, left to right: the four
/// solids, the three two-colour overprints (red, green, blue), then the four 50 % tints.
pub const COLOR_BAR_PATCHES: [[f32; 4]; 11] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 1.0, 0.0],
    [1.0, 0.0, 1.0, 0.0],
    [1.0, 1.0, 0.0, 0.0],
    [0.5, 0.0, 0.0, 0.0],
    [0.0, 0.5, 0.0, 0.0],
    [0.0, 0.0, 0.5, 0.0],
    [0.0, 0.0, 0.0, 0.5],
];
/// Preferred edge of a colour-bar patch, in millimetres (patches shrink to fit).
const COLOR_PATCH_MM: f64 = 5.0;
/// Colour bars whose patches would be lower than this are omitted.
const COLOR_PATCH_MIN_MM: f64 = 1.5;
/// Distance kept between slug furniture (colour bar, ruler) and the corner crop marks and
/// fold marks next to it, in millimetres.
const FURNITURE_INSET_MM: f64 = 2.0;
/// Clear space kept between slug furniture and the edges of the mark band.
const FURNITURE_CLEARANCE_MM: f64 = 0.5;

/// Lowest margin band [`caption_areas`] reports: room for one line of 5–6 pt type.
pub const CAPTION_MIN_HEIGHT_MM: f64 = 2.0;

/// Lengths a scale ruler can have, longest first, in millimetres.
const RULER_LENGTHS_MM: [f64; 3] = [100.0, 50.0, 20.0];
/// Height of the end ticks of the scale ruler, in millimetres, when there is room.
const RULER_TICK_MM: f64 = 2.5;
/// Scale rulers whose end ticks would be lower than this are omitted.
const RULER_MIN_TICK_MM: f64 = 1.5;
/// Width reserved for the ruler's label (e.g. "100 mm" at 6–7 pt).
const RULER_LABEL_ROOM_MM: f64 = 14.0;
/// Gap between the end of the ruler and a label set beside it, in millimetres.
const RULER_LABEL_SIDE_GAP_MM: f64 = 1.5;
/// Height reserved for a label set above the ruler: one line of 6 pt type.
const RULER_LABEL_HEIGHT_MM: f64 = 2.2;
/// Gap between the top of the ruler's ticks and a label set above them, in millimetres.
const RULER_LABEL_GAP_MM: f64 = 0.6;

/// The numbers every mark generator needs, or nothing if the page cannot be built.
struct Ctx {
    spread: Size,
    cover_w: f64,
    spine_w: f64,
    fold_x: [f64; 2],
    /// Distance from the trim edge to the bleed edge.
    bleed: f64,
    /// Distance from the trim edge to the limit of the mark area.
    reach: f64,
    marks: Marks,
}

impl Ctx {
    fn new(format: &CardFormat, profile: &PrintProfile) -> Option<Self> {
        let g = page_geometry(format, profile).ok()?;
        Some(Self {
            spread: format.spread(),
            cover_w: format.cover.w,
            spine_w: format.spine_width,
            fold_x: format.fold_x(),
            bleed: g.bleed_mm,
            reach: g.mark_reach_mm,
            marks: profile.marks,
        })
    }

    /// Width of the band marks may occupy, between the bleed edge and the mark-area edge.
    fn band(&self) -> f64 {
        self.reach - self.bleed
    }

    /// The part of the band, starting at the bleed edge, used by everything but crop and
    /// fold marks.
    fn furniture_band(&self) -> f64 {
        self.band().min(FURNITURE_BAND_MAX_MM)
    }

    /// Start and end of crop and fold marks, as distances from the trim edge, or `None`
    /// when there is no room for a mark worth drawing.
    fn tick_span(&self) -> Option<(f64, f64)> {
        let start = mark_start(&self.marks, self.bleed);
        let length = self.marks.length.min(self.reach - start);
        (length >= MIN_MARK_LENGTH_MM).then_some((start, start + length))
    }

    fn half_stroke_mm(&self) -> f64 {
        pt_to_mm(self.marks.weight_pt) / 2.0
    }
}

/// Distance from the trim edge at which crop and fold marks start: the profile's offset,
/// but never closer to the bleed edge than [`MARK_BLEED_GAP_MM`].
fn mark_start(marks: &Marks, bleed: f64) -> f64 {
    marks.offset.max(bleed + MARK_BLEED_GAP_MM)
}

/// Crop (trim) marks: at each of the four corners of the trim box, one horizontal and one
/// vertical stroke on the extensions of the two trim edges that meet there, starting
/// `max(marks.offset, bleed + `[`MARK_BLEED_GAP_MM`]`)` away from the corner and pointing
/// outwards — so they never touch the bleed.
///
/// Eight stroked [`Item`]s, one per mark ([`marks`] says why they are not one path): for
/// each corner in the order top-left, bottom-left, top-right, bottom-right, the
/// horizontal mark, then the vertical one. Each runs outwards from the card.
///
/// Returns nothing if the page cannot be built ([`new_page`](crate::new_page) would fail)
/// or there is less than [`MIN_MARK_LENGTH_MM`] of room. Ignores `marks.crop`.
#[must_use]
pub fn crop_marks(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let Some((a, b)) = cx.tick_span() else { return Vec::new() };
    let stroke = cx.marks.stroke();
    let mut items = Vec::with_capacity(8);
    let mut mark = |from: Point, to: Point| {
        items.push(Item::stroke(Path::line(from, to), stroke.clone()));
    };
    for (x, dir_x) in [(0.0, -1.0), (cx.spread.w, 1.0)] {
        for (y, dir_y) in [(0.0, -1.0), (cx.spread.h, 1.0)] {
            // Along the horizontal trim edge, away from the corner.
            mark(Point::new(x + dir_x * a, y), Point::new(x + dir_x * b, y));
            // Along the vertical trim edge, away from the corner.
            mark(Point::new(x, y + dir_y * a), Point::new(x, y + dir_y * b));
        }
    }
    items
}

/// The dash pattern of a fold mark `length` millimetres long: [`FOLD_MARK_DASH_MM`], or
/// the same pattern scaled down so that three dashes fit a shorter mark. A fold mark is
/// thus always recognisably dashed, and it begins and (up to 5 mm) ends with a dash.
fn fold_dash_mm(length: f64) -> Vec<f64> {
    let [on, off] = FOLD_MARK_DASH_MM;
    let scale = (length / (3.0 * on + 2.0 * off)).min(1.0);
    vec![on * scale, off * scale]
}

/// Fold marks: a **dashed** vertical stroke above and below the spread at each of the two
/// creases ([`CardFormat::fold_x`]), starting where the crop marks start. Crop marks are
/// solid, so the two cannot be confused — on the default card the top edge carries marks
/// at x = 0, 110, 120 and 230 mm, and read as cuts the middle two would have the spine
/// cut out.
///
/// * [`MarkStyle::Press`] and [`MarkStyle::Black`]: the same length as the crop marks.
/// * [`MarkStyle::Guide`]: the mark runs on to the edge of the printable area (at most
///   [`FOLD_GUIDE_MAX_MM`] from its start), which gives a ruler more to align with when
///   the crease is scored by hand.
///
/// The dash pattern is [`FOLD_MARK_DASH_MM`] (1 mm on, 1 mm off), measured from the end
/// nearest the card; marks shorter than 5 mm use a proportionally finer pattern so that
/// they still show three dashes.
///
/// Four stroked [`Item`]s, one per mark, each running outwards from the card: for each
/// crease, left to right, the mark above the spread, then the one below. Returns nothing
/// if the page cannot be built or there is less than [`MIN_MARK_LENGTH_MM`] of room.
/// Ignores `marks.fold`.
#[must_use]
pub fn fold_marks(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let Some((a, tick_end)) = cx.tick_span() else { return Vec::new() };
    let b = match cx.marks.style {
        MarkStyle::Press | MarkStyle::Black => tick_end,
        MarkStyle::Guide => cx.reach.min(a + FOLD_GUIDE_MAX_MM).max(tick_end),
    };
    let h = cx.spread.h;
    let dashed = Stroke::dashed(cx.marks.color(), cx.marks.weight_pt, fold_dash_mm(b - a));
    let mut items = Vec::with_capacity(4);
    for x in cx.fold_x {
        for (from, to) in [(-a, -b), (h + a, h + b)] {
            let path = Path::line(Point::new(x, from), Point::new(x, to));
            items.push(Item::stroke(path, dashed.clone()));
        }
    }
    items
}

/// Checks that the cut and fold marks the profile asks for can actually be drawn.
///
/// [`new_page`](crate::new_page) accepts any card whose bleed is printable, and
/// [`marks`] silently leaves out what does not fit. For a sheet printed at home that is
/// not good enough: without fold marks nobody can find the two creases. Call this before
/// composing a card and refuse (or offer another format, sheet or printer) on error.
///
/// # Errors
/// * whatever [`page_geometry`] reports;
/// * [`LayoutError::MarksDoNotFit`] — `marks.crop` or `marks.fold` is set but the marks
///   would be left out: fewer than [`MIN_MARK_LENGTH_MM`] lie between the start of the
///   marks, `max(marks.offset, bleed + `[`MARK_BLEED_GAP_MM`]`)`, and the edge of the mark
///   area (or `marks.length` itself is shorter than that).
///
/// Marks that are merely shortened, and registration targets, colour bar or scale ruler
/// that are dropped for lack of room, are not errors.
pub fn check_marks_fit(format: &CardFormat, profile: &PrintProfile) -> Result<(), LayoutError> {
    let g = page_geometry(format, profile)?;
    let wanted = &profile.marks;
    if !(wanted.crop || wanted.fold) {
        return Ok(());
    }
    let start = mark_start(wanted, g.bleed_mm);
    let available_mm = wanted.length.min(g.mark_reach_mm - start).max(0.0);
    if available_mm < MIN_MARK_LENGTH_MM {
        return Err(LayoutError::MarksDoNotFit { available_mm, needed_mm: MIN_MARK_LENGTH_MM });
    }
    Ok(())
}

/// One registration target: its centre and the half-width of its cross.
#[derive(Clone, Copy)]
struct Target {
    center: Point,
    half: f64,
}

/// The registration targets that fit: left and right always (if the band allows), top
/// and bottom only if the spine leaves room between the two fold marks.
fn target_layout(cx: &Ctx) -> Vec<Target> {
    let keep_clear = TARGET_CLEARANCE_MM + cx.half_stroke_mm();
    let band = cx.furniture_band();
    let side_half = TARGET_HALF_MM.min(band / 2.0 - keep_clear);
    if side_half < TARGET_MIN_HALF_MM {
        return Vec::new();
    }
    // Centre of the band between the bleed edge and the mark-area edge.
    let d = cx.bleed + band / 2.0;
    let (w, h) = (cx.spread.w, cx.spread.h);
    let mut targets = vec![
        Target { center: Point::new(-d, h / 2.0), half: side_half },
        Target { center: Point::new(w + d, h / 2.0), half: side_half },
    ];
    // The creases sit `spine / 2` either side of the middle of the top and bottom edges,
    // so a target centred there has to fit between their fold marks.
    let mid_half = side_half.min(cx.spine_w / 2.0 - keep_clear);
    if mid_half >= TARGET_MIN_HALF_MM {
        targets.push(Target { center: Point::new(w / 2.0, -d), half: mid_half });
        targets.push(Target { center: Point::new(w / 2.0, h + d), half: mid_half });
    }
    targets
}

/// Where [`registration_targets`] puts its targets: left, right, then — if present — top
/// and bottom. Empty if the page cannot be built or the mark band is too narrow.
///
/// Each target is centred on its side of the spread, so the set is symmetric and the
/// targets of the two faces coincide when the sheet is turned over. The top and bottom
/// targets sit between the two fold marks: they shrink with the spine (and would be left
/// out below about 4.1 mm, narrower than any valid spine; the default 10 mm spine has
/// full-size targets on all four sides).
#[must_use]
pub fn registration_target_centers(format: &CardFormat, profile: &PrintProfile) -> Vec<Point> {
    Ctx::new(format, profile)
        .map(|cx| target_layout(&cx).into_iter().map(|t| t.center).collect())
        .unwrap_or_default()
}

/// Registration targets: a circle and a cross, centred on each side of the spread, in the
/// middle of the band between the bleed edge and the edge of the mark area (or of its
/// first [`FURNITURE_BAND_MAX_MM`] when the margin is wider than that). One stroked
/// [`Item`] per target (circle, horizontal stroke, vertical stroke), in the order of
/// [`registration_target_centers`], which also documents the placement rule. Ignores
/// `marks.registration`.
#[must_use]
pub fn registration_targets(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let stroke = cx.marks.stroke();
    target_layout(&cx)
        .into_iter()
        .map(|Target { center: c, half }| {
            let mut path = Path::circle(c, half * TARGET_CIRCLE_RATIO);
            path.move_to(c.x - half, c.y).line_to(c.x + half, c.y);
            path.move_to(c.x, c.y - half).line_to(c.x, c.y + half);
            Item::stroke(path, stroke.clone())
        })
        .collect()
}

/// A colour control strip below the left-hand panel: one filled square per entry of
/// [`COLOR_BAR_PATCHES`], between the bottom-left crop mark and the first fold mark.
/// Patches shrink when the panel or the mark band is small. Ignores `marks.color_bar`.
///
/// The patches are plain process colours (at most 200 % ink) and lie outside the bleed
/// box, so they never count against a total-ink limit for the artwork.
#[must_use]
pub fn color_bar(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let band = cx.furniture_band();
    let patch_h = COLOR_PATCH_MM.min(band - 2.0 * FURNITURE_CLEARANCE_MM);
    if patch_h < COLOR_PATCH_MIN_MM {
        return Vec::new();
    }
    let available = cx.cover_w - 2.0 * FURNITURE_INSET_MM;
    let patch_w = COLOR_PATCH_MM.min(available / COLOR_BAR_PATCHES.len() as f64);
    let y = cx.spread.h + cx.bleed + (band - patch_h) / 2.0;
    let mut x = FURNITURE_INSET_MM;
    let mut items = Vec::with_capacity(COLOR_BAR_PATCHES.len());
    for [c, m, y_ink, k] in COLOR_BAR_PATCHES {
        items.push(Item::filled_rect(
            Rect::new(x, y, patch_w, patch_h),
            Color::cmyk(c, m, y_ink, k),
        ));
        x += patch_w;
    }
    items
}

/// A ruler printed in the margin so that a home user can check that the sheet came out at
/// 100 % (a "fit to page" print shrinks the card and misplaces the creases).
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleRuler {
    /// Left end of the ruler's base line.
    pub start: Point,
    /// Length of the ruler when printed at 100 %: 100, 50 or 20 mm, the longest that fits.
    pub length_mm: f64,
    /// Baseline origin for a label such as "100 mm" (this crate sets no text): the
    /// bottom-left corner of [`label_area`](Self::label_area).
    pub label_origin: Point,
    /// The free rectangle reserved for the label: beside the ruler when the panel is wide
    /// enough, otherwise above its ticks. No mark touches it and it lies inside the mark
    /// area. It is about 14 mm wide and at least 1.5 mm high (2.2 mm above the ruler):
    /// room for "100 mm" in 6 pt type.
    pub label_area: Rect,
    /// The ruler itself: one stroked path (base line plus a tick every 10 mm).
    pub items: Vec<Item>,
}

/// The scale-check ruler: a horizontal line of known length above the left-hand panel,
/// between the top-left crop mark and the first fold mark, with ticks every 10 mm (taller
/// at the ends and, on the 100 mm ruler, at 50 mm). Ignores `marks.scale_ruler`.
///
/// The ruler is the longest of 100, 50 and 20 mm that fits the panel together with its
/// label. The label goes beside the ruler when the panel is wide enough; otherwise it
/// goes above the ticks if the margin is high enough (about 6.5 mm) — which is how the
/// default 110 mm panel gets its 100 mm ruler on A4 — and failing that a shorter ruler is
/// used.
///
/// Returns `None` if the page cannot be built or the mark band is too narrow.
#[must_use]
pub fn scale_ruler(format: &CardFormat, profile: &PrintProfile) -> Option<ScaleRuler> {
    let cx = Ctx::new(format, profile)?;
    let keep_clear = FURNITURE_CLEARANCE_MM + cx.half_stroke_mm();
    let band = cx.furniture_band();
    let tick = RULER_TICK_MM.min(band - 2.0 * keep_clear);
    if tick < RULER_MIN_TICK_MM {
        return None;
    }
    let x0 = FURNITURE_INSET_MM;
    let panel_room = cx.cover_w - 2.0 * FURNITURE_INSET_MM;
    // Full-height ticks with a line of type above them.
    let stacked = RULER_TICK_MM + RULER_LABEL_GAP_MM + RULER_LABEL_HEIGHT_MM;
    let can_stack = band - 2.0 * keep_clear >= stacked;
    let (length_mm, label_above) = RULER_LENGTHS_MM.into_iter().find_map(|l| {
        if l + RULER_LABEL_ROOM_MM <= panel_room {
            Some((l, false))
        } else if l <= panel_room && can_stack {
            Some((l, true))
        } else {
            None
        }
    })?;
    // The base line is the edge nearest the card; ticks point away from it. Ruler (and
    // label, when it sits above) are centred in the band.
    let (tick, used) = if label_above { (RULER_TICK_MM, stacked) } else { (tick, tick) };
    let y = -(cx.bleed + (band - used) / 2.0);
    let label_area = if label_above {
        let top = y - stacked;
        Rect::new(x0, top, RULER_LABEL_ROOM_MM, RULER_LABEL_HEIGHT_MM)
    } else {
        let x = x0 + length_mm + RULER_LABEL_SIDE_GAP_MM;
        Rect::new(x, y - tick, RULER_LABEL_ROOM_MM - RULER_LABEL_SIDE_GAP_MM, tick)
    };
    let mut path = Path::new();
    path.move_to(x0, y).line_to(x0 + length_mm, y);
    let mut at = 0.0;
    while at <= length_mm + 1e-9 {
        let major = at == 0.0 || at >= length_mm - 1e-9 || (length_mm > 50.0 && at == 50.0);
        let height = if major { tick } else { tick * 0.6 };
        path.move_to(x0 + at, y).line_to(x0 + at, y - height);
        at += 10.0;
    }
    Some(ScaleRuler {
        start: Point::new(x0, y),
        length_mm,
        label_origin: Point::new(label_area.x, label_area.bottom()),
        label_area,
        items: vec![Item::stroke(path, cx.marks.stroke())],
    })
}

/// The two rectangles of the margin that no mark ever uses: above and below the
/// right-hand panel, between the second fold mark and the corner crop marks, as
/// `[above, below]`. A composer can set small print there — a job line, "print at 100 %,
/// both sides, flip on the short edge" (see
/// [`SheetPlacement::duplex_flip`](crate::SheetPlacement::duplex_flip)), "cut on the
/// solid marks, fold on the dashed ones" — without checking for collisions (this crate
/// sets no text).
///
/// The rectangles lie inside the mark area (hence inside the printable area of a home
/// sheet) and outside the bleed box. Returns `None` if the page cannot be built or the
/// band is lower than [`CAPTION_MIN_HEIGHT_MM`].
#[must_use]
pub fn caption_areas(format: &CardFormat, profile: &PrintProfile) -> Option<[Rect; 2]> {
    let cx = Ctx::new(format, profile)?;
    let height = cx.furniture_band() - 2.0 * FURNITURE_CLEARANCE_MM;
    if height < CAPTION_MIN_HEIGHT_MM {
        return None;
    }
    let x = cx.fold_x[1] + FURNITURE_INSET_MM;
    let width = cx.cover_w - 2.0 * FURNITURE_INSET_MM;
    let gap = cx.bleed + FURNITURE_CLEARANCE_MM;
    Some([
        Rect::new(x, -gap - height, width, height),
        Rect::new(x, cx.spread.h + gap, width, height),
    ])
}

/// All the marks enabled in `profile.marks`, as one layer named [`MARKS_LAYER`]
/// (`"marks"`), to be appended to both pages **after** the artwork.
///
/// Contents, in order: [`crop_marks`] (solid), [`fold_marks`] (dashed),
/// [`registration_targets`], [`color_bar`], [`scale_ruler`] — each only if its flag is
/// set. Every point of every path lies inside the page's media box (inside the printable
/// area for sheet-based profiles) and outside its bleed box.
///
/// # Marks and preflight
///
/// A preflight tool judges one painted object at a time by its bounding box, and cannot
/// tell a butt cap from a projecting one. Two rules keep the marks — drawn in the
/// registration colour for a press — unambiguous for it:
///
/// * **One object per mark.** Each crop mark, each fold mark and each registration target
///   is its own [`Item`]. Eight crop marks in one path would have a bounding box as large
///   as the page: "registration colour inside the bleed box" as far as such a tool can
///   see.
/// * **A gap after the bleed.** Crop and fold marks start [`MARK_BLEED_GAP_MM`] beyond the
///   bleed edge, not on it, so that no mark touches the bleed box even when half a
///   stroke width is added around its end points. The other furniture keeps the same
///   distance.
///
/// The layer is empty when [`new_page`](crate::new_page) would fail for the same
/// arguments, and marks that do not fit the available margin are shortened or left out,
/// so the function never panics and never draws over the artwork. Use
/// [`check_marks_fit`] to find out whether the cut and fold marks made it.
#[must_use]
pub fn marks(format: &CardFormat, profile: &PrintProfile) -> Item {
    let mut items = Vec::new();
    let wanted = &profile.marks;
    if wanted.crop {
        items.extend(crop_marks(format, profile));
    }
    if wanted.fold {
        items.extend(fold_marks(format, profile));
    }
    if wanted.registration {
        items.extend(registration_targets(format, profile));
    }
    if wanted.color_bar {
        items.extend(color_bar(format, profile));
    }
    if wanted.scale_ruler {
        items.extend(scale_ruler(format, profile).map(|r| r.items).unwrap_or_default());
    }
    Item::layer(MARKS_LAYER, items)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::profile::Sheet;
    use ebc_scene::PathSeg;

    /// The straight sub-paths of a stroked item, as (start, end) pairs.
    fn lines(item: &Item) -> Vec<(Point, Point)> {
        let Item::Path { path, .. } = item else { panic!("not a path: {item:?}") };
        let mut out = Vec::new();
        let mut cur = None;
        for seg in &path.segs {
            match *seg {
                PathSeg::MoveTo(p) => cur = Some(p),
                PathSeg::LineTo(p) => {
                    out.push((cur.unwrap(), p));
                    cur = Some(p);
                }
                _ => {}
            }
        }
        out
    }

    /// The straight sub-paths of all the items, in order.
    fn all_lines(items: &[Item]) -> Vec<(Point, Point)> {
        items.iter().flat_map(lines).collect()
    }

    fn stroke_of(item: &Item) -> &Stroke {
        match item {
            Item::Path { stroke: Some(s), fill: None, .. } => s,
            other => panic!("not a stroked path: {other:?}"),
        }
    }

    #[test]
    fn professional_crop_marks_start_clear_of_the_bleed_and_are_5_mm_long() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        let items = crop_marks(&f, &p);
        assert_eq!(items.len(), 8, "two marks per corner, one item each");
        for item in &items {
            let s = stroke_of(item);
            assert_eq!(s.color, Color::Registration);
            assert_eq!(s.width_pt, 0.25);
            assert!(s.dash_mm.is_empty());
            assert_eq!(lines(item).len(), 1, "one stroke per item");
        }
        let l = all_lines(&items);
        // Top-left corner: horizontal then vertical, from 0.5 mm beyond the 3 mm bleed.
        assert_eq!(MARK_BLEED_GAP_MM, 0.5);
        assert_eq!(l[0], (Point::new(-3.5, 0.0), Point::new(-8.5, 0.0)));
        assert_eq!(l[1], (Point::new(0.0, -3.5), Point::new(0.0, -8.5)));
        // Bottom-right corner.
        assert_eq!(l[6], (Point::new(233.5, 178.0), Point::new(238.5, 178.0)));
        assert_eq!(l[7], (Point::new(230.0, 181.5), Point::new(230.0, 186.5)));
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn point_close(a: Point, b: Point) -> bool {
        close(a.x, b.x) && close(a.y, b.y)
    }

    fn rect_close(a: Rect, b: Rect) -> bool {
        close(a.x, b.x) && close(a.y, b.y) && close(a.w, b.w) && close(a.h, b.h)
    }

    #[test]
    fn professional_fold_marks_are_dashed_strokes_at_both_creases() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        let items = fold_marks(&f, &p);
        assert_eq!(items.len(), 4, "above and below each crease, one item each");
        // Same colour and weight as the crop marks, but dashed 1 mm on, 1 mm off: a 5 mm
        // mark shows three dashes. A crease must never read as a cut.
        let crop_items = crop_marks(&f, &p);
        let crop = stroke_of(&crop_items[0]);
        assert!(crop.dash_mm.is_empty());
        for item in &items {
            let fold = stroke_of(item);
            assert_eq!(fold.dash_mm, vec![1.0, 1.0]);
            assert_eq!((fold.color, fold.width_pt), (crop.color, crop.width_pt));
            assert_eq!(fold.cap, ebc_scene::StrokeCap::Butt);
        }
        // Each mark runs outwards from the card, so its first dash is the one nearest it.
        assert_eq!(
            all_lines(&items),
            vec![
                (Point::new(110.0, -3.5), Point::new(110.0, -8.5)),
                (Point::new(110.0, 181.5), Point::new(110.0, 186.5)),
                (Point::new(120.0, -3.5), Point::new(120.0, -8.5)),
                (Point::new(120.0, 181.5), Point::new(120.0, 186.5)),
            ]
        );
    }

    #[test]
    fn home_fold_marks_are_dashed_grey_guides_running_to_the_printable_edge() {
        let (f, p) = (CardFormat::poche(), PrintProfile::home(Sheet::A4));
        let items = fold_marks(&f, &p);
        assert_eq!(items.len(), 4, "four dashed strokes, no solid part");
        for item in &items {
            let dashed = stroke_of(item);
            assert_eq!(dashed.dash_mm, vec![1.0, 1.0]);
            assert_eq!(dashed.color, Color::Gray(crate::GUIDE_GRAY_LEVEL));
            assert_eq!(dashed.width_pt, 0.5);
        }
        // From 0.5 mm beyond the bleed edge (2 mm) to the printable edge: 16 mm margin -
        // 6.35 mm unprintable = 9.65 mm from the trim. The solid cut marks stop at 7.5 mm.
        let g = all_lines(&items);
        assert_eq!(g.len(), 4);
        for (got, (x, from, to)) in g.iter().zip([
            (110.0, -2.5, -9.65),
            (110.0, 180.5, 187.65),
            (120.0, -2.5, -9.65),
            (120.0, 180.5, 187.65),
        ]) {
            assert_eq!((got.0.x, got.1.x), (x, x), "exactly on the crease");
            assert!(close(got.0.y, from) && close(got.1.y, to), "{got:?}");
        }
        let crop = all_lines(&crop_marks(&f, &p));
        assert_eq!(crop[1], (Point::new(0.0, -2.5), Point::new(0.0, -7.5)));
        assert!(crop_marks(&f, &p).iter().all(|item| stroke_of(item).dash_mm.is_empty()));
    }

    #[test]
    fn short_fold_marks_keep_three_dashes() {
        let f = CardFormat::poche();
        let dash_of = |slug: f64| {
            let mut p = PrintProfile::professional_eu();
            p.slug = slug;
            fold_marks(&f, &p).first().map(|i| stroke_of(i).dash_mm.clone())
        };
        // The marks start MARK_BLEED_GAP_MM into the slug. 5 mm or more of mark: the
        // nominal pattern.
        let gap = MARK_BLEED_GAP_MM;
        assert_eq!(dash_of(10.0), Some(vec![1.0, 1.0]));
        assert_eq!(dash_of(gap + 5.0), Some(vec![1.0, 1.0]));
        // A 2.5 mm mark is dashed 0.5 / 0.5 — on, off, on, off, on.
        assert_eq!(dash_of(gap + 2.5), Some(vec![0.5, 0.5]));
        // The shortest mark drawn (2 mm) still has three 0.4 mm dashes.
        let d = dash_of(gap + MIN_MARK_LENGTH_MM).unwrap();
        assert!(close(d[0], 0.4) && close(d[1], 0.4), "{d:?}");
        assert_eq!(dash_of(gap + 1.9), None);
        // A long guide never gets coarser than the nominal pattern.
        assert_eq!(fold_dash_mm(20.0), FOLD_MARK_DASH_MM.to_vec());
    }

    #[test]
    fn black_marks_for_digital_jobs_differ_only_in_colour() {
        let f = CardFormat::poche();
        let press = PrintProfile::professional_eu();
        let mut black = press.clone();
        black.marks.style = MarkStyle::Black;
        for (a, b) in [
            (crop_marks(&f, &press), crop_marks(&f, &black)),
            (fold_marks(&f, &press), fold_marks(&f, &black)),
            (registration_targets(&f, &press), registration_targets(&f, &black)),
        ] {
            assert_eq!(a.len(), b.len());
            assert_eq!(all_lines(&a), all_lines(&b));
            for (a, b) in a.iter().zip(&b) {
                assert_eq!(stroke_of(a).dash_mm, stroke_of(b).dash_mm);
                assert_eq!(stroke_of(a).color, Color::Registration);
                assert_eq!(stroke_of(b).color, Color::BLACK);
            }
        }
    }

    #[test]
    fn check_marks_fit_reports_marks_that_would_be_left_out() {
        let f = CardFormat::poche();
        // The reference cards have room.
        assert_eq!(check_marks_fit(&f, &PrintProfile::professional_eu()), Ok(()));
        assert_eq!(check_marks_fit(&f, &PrintProfile::home(Sheet::A4)), Ok(()));
        assert_eq!(check_marks_fit(&f, &PrintProfile::home(Sheet::Letter)), Ok(()));
        // No marks wanted: nothing to fit, even without a slug.
        assert_eq!(check_marks_fit(&f, &PrintProfile::upload_eu()), Ok(()));
        // A slug too small for the shortest mark: the marks start 0.5 mm into it.
        let mut p = PrintProfile::professional_eu();
        p.slug = 2.0;
        assert_eq!(
            check_marks_fit(&f, &p),
            Err(LayoutError::MarksDoNotFit { available_mm: 1.5, needed_mm: 2.0 })
        );
        assert!(crop_marks(&f, &p).is_empty() && fold_marks(&f, &p).is_empty());
        assert_eq!(
            check_marks_fit(&f, &p).unwrap_err().to_string(),
            "no room for cut and fold marks: they need at least 2.0 mm beyond the bleed \
             but only 1.5 mm is available"
        );
        // Shortened marks are fine.
        p.slug = 2.5;
        assert_eq!(check_marks_fit(&f, &p), Ok(()));
        assert_eq!(crop_marks(&f, &p).len(), 8);
        // Only fold marks wanted: still checked. Only a ruler wanted: not.
        p.slug = 0.0;
        p.marks = Marks { fold: true, ..Marks::none() };
        assert!(matches!(check_marks_fit(&f, &p), Err(LayoutError::MarksDoNotFit { .. })));
        p.marks = Marks { scale_ruler: true, ..Marks::none() };
        assert_eq!(check_marks_fit(&f, &p), Ok(()));
        // A 110 x 186 mm cover on A4: 12 mm above and below, 6.35 of it unprintable, 2 mm
        // bleed and the 0.5 mm gap leave 3.15 mm, enough for shortened marks.
        let tight = CardFormat::custom(Size::new(110.0, 186.0), 10.0).unwrap();
        let home = PrintProfile::home(Sheet::A4);
        assert_eq!(check_marks_fit(&tight, &home), Ok(()));
        let l = all_lines(&crop_marks(&tight, &home));
        assert!(point_close(l[1].0, Point::new(0.0, -2.5)), "{l:?}");
        assert!(point_close(l[1].1, Point::new(0.0, -5.65)), "{l:?}");
        // Four millimetres taller and the card still fits the sheet, but not its marks.
        let tighter = CardFormat::custom(Size::new(110.0, 190.0), 10.0).unwrap();
        assert!(crate::new_page(&tighter, &home).is_ok());
        assert!(matches!(
            check_marks_fit(&tighter, &home),
            Err(LayoutError::MarksDoNotFit { needed_mm: 2.0, .. })
        ));
        // A card that does not fit at all reports that, not the marks.
        assert!(matches!(
            check_marks_fit(&CardFormat::a5(), &home),
            Err(LayoutError::DoesNotFit { .. })
        ));
        let bad = CardFormat { cover: Size::new(f64::NAN, 1.0), spine_width: 10.0 };
        assert!(matches!(check_marks_fit(&bad, &home), Err(LayoutError::CoverOutOfRange { .. })));
    }

    #[test]
    fn marks_never_start_inside_the_bleed_even_if_the_offset_says_so() {
        let mut p = PrintProfile::professional_eu();
        p.marks.offset = 0.0; // closer than the 3 mm bleed
        let l = all_lines(&crop_marks(&CardFormat::poche(), &p));
        assert_eq!(l[0], (Point::new(-3.5, 0.0), Point::new(-8.5, 0.0)));
        // On the bleed edge, or less than the gap beyond it: still the gap.
        for offset in [3.0, 3.2, 3.5] {
            p.marks.offset = offset;
            let l = all_lines(&crop_marks(&CardFormat::poche(), &p));
            assert_eq!(l[0], (Point::new(-3.5, 0.0), Point::new(-8.5, 0.0)), "{offset}");
        }
        // A larger offset is honoured as it is.
        p.marks.offset = 6.0;
        let l = all_lines(&crop_marks(&CardFormat::poche(), &p));
        assert_eq!(l[0], (Point::new(-6.0, 0.0), Point::new(-11.0, 0.0)));
        let l = all_lines(&fold_marks(&CardFormat::poche(), &p));
        assert_eq!(l[0], (Point::new(110.0, -6.0), Point::new(110.0, -11.0)));
    }

    #[test]
    fn marks_shrink_to_the_slug_and_vanish_when_there_is_none() {
        let f = CardFormat::poche();
        let mut p = PrintProfile::professional_eu();
        p.slug = 3.0; // room for 2.5 mm marks only, after the 0.5 mm gap
        let l = all_lines(&crop_marks(&f, &p));
        assert_eq!(l[0], (Point::new(-3.5, 0.0), Point::new(-6.0, 0.0)));
        p.slug = 2.4; // 1.9 mm of mark: below MIN_MARK_LENGTH_MM
        assert!(crop_marks(&f, &p).is_empty());
        assert!(fold_marks(&f, &p).is_empty());
        p.slug = 0.9; // not even room for a registration target or a ruler
        assert!(crop_marks(&f, &p).is_empty());
        assert!(fold_marks(&f, &p).is_empty());
        assert!(registration_targets(&f, &p).is_empty());
        assert!(color_bar(&f, &p).is_empty());
        assert!(scale_ruler(&f, &p).is_none());
        // The layer is still there, just empty.
        assert_eq!(marks(&f, &p), Item::layer("marks", vec![]));
        // Zero-length marks are not drawn either.
        let mut p = PrintProfile::professional_eu();
        p.marks.length = 0.0;
        assert!(crop_marks(&f, &p).is_empty());
    }

    #[test]
    fn registration_targets_are_centred_on_each_side_for_the_default_spine() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        let c = registration_target_centers(&f, &p);
        // Band is 3..13 mm from the trim: centres 8 mm out.
        assert_eq!(
            c,
            vec![
                Point::new(-8.0, 89.0),
                Point::new(238.0, 89.0),
                Point::new(115.0, -8.0),
                Point::new(115.0, 186.0),
            ]
        );
        let items = registration_targets(&f, &p);
        assert_eq!(items.len(), 4, "one item per target");
        // Each a circle (4 cubics) and a cross (2 lines).
        for item in &items {
            let Item::Path { path, .. } = item else { panic!() };
            let cubics = path.segs.iter().filter(|s| matches!(s, PathSeg::CubicTo(..))).count();
            assert_eq!(cubics, 4);
            assert_eq!(lines(item).len(), 2);
            assert_eq!(stroke_of(item).color, Color::Registration);
        }
        let l = all_lines(&items);
        assert_eq!(l.len(), 8);
        assert_eq!(l[0], (Point::new(-11.5, 89.0), Point::new(-4.5, 89.0)));
        assert_eq!(l[1], (Point::new(-8.0, 85.5), Point::new(-8.0, 92.5)));
        // The top target is full size too: a 10 mm spine leaves room between the folds.
        assert_eq!(l[4], (Point::new(111.5, -8.0), Point::new(118.5, -8.0)));
        assert_eq!(stroke_of(&items[0]).color, Color::Registration);
    }

    #[test]
    fn narrow_spines_shrink_then_drop_the_top_and_bottom_targets() {
        let p = PrintProfile::professional_eu();
        let half_stroke = pt_to_mm(0.25) / 2.0;
        // 6 mm spine: the cross must end 0.5 mm (plus half a hairline) before each fold.
        let f = CardFormat::poche().with_spine(6.0).unwrap();
        let c = registration_target_centers(&f, &p);
        assert_eq!(c.len(), 4);
        assert_eq!(c[2], Point::new(113.0, -8.0));
        let l = all_lines(&registration_targets(&f, &p));
        let (a, b) = l[4];
        assert!((a.x - (110.0 + 0.5 + half_stroke)).abs() < 1e-9, "{a:?}");
        assert!((b.x - (116.0 - 0.5 - half_stroke)).abs() < 1e-9, "{b:?}");
        // The side targets keep their full size.
        assert_eq!(l[0], (Point::new(-11.5, 89.0), Point::new(-4.5, 89.0)));
        // 5 mm, the narrowest valid spine (two creases need 5 mm): still four targets.
        let f = CardFormat::poche().with_spine(5.0).unwrap();
        assert_eq!(registration_target_centers(&f, &p).len(), 4);
        // A 4 mm spine is no longer a valid format (it used to lose the top and bottom
        // targets); built directly, it gets no marks at all instead of a panic.
        let f = CardFormat { spine_width: 4.0, ..CardFormat::poche() };
        assert!(registration_target_centers(&f, &p).is_empty());
        assert!(registration_targets(&f, &p).is_empty());
    }

    #[test]
    fn color_bar_sits_below_the_left_panel_between_crop_and_fold_marks() {
        let (f, mut p) = (CardFormat::poche(), PrintProfile::professional_eu());
        p.marks.color_bar = true;
        let items = color_bar(&f, &p);
        assert_eq!(items.len(), COLOR_BAR_PATCHES.len());
        let mut x = 2.0;
        for (item, [c, m, y, k]) in items.iter().zip(COLOR_BAR_PATCHES) {
            let Item::Path { path, fill: Some((color, _)), stroke: None } = item else {
                panic!("patch must be a filled path")
            };
            assert_eq!(*color, Color::Cmyk { c, m, y, k });
            assert!(color.total_ink_percent().unwrap() <= 200.0);
            let b = path.control_bounds().unwrap();
            // 5 mm squares centred in the 10 mm slug: 2.5 mm below the bleed edge.
            assert!((b.x - x).abs() < 1e-9 && (b.y - 183.5).abs() < 1e-9);
            assert!((b.w - 5.0).abs() < 1e-9 && (b.h - 5.0).abs() < 1e-9);
            x += 5.0;
        }
        assert!(x <= f.fold_x()[0] - 2.0);
        // On the smallest cover the patches shrink to stay left of the fold mark.
        let small = CardFormat::custom(Size::new(50.0, 80.0), 10.0).unwrap();
        let items = color_bar(&small, &p);
        let Item::Path { path, .. } = items.last().unwrap() else { panic!() };
        let last = path.control_bounds().unwrap();
        assert!(last.right() <= 48.0 + 1e-9, "{last:?}");
        assert!(last.w > 4.0);
    }

    #[test]
    fn scale_ruler_lengths_and_ticks() {
        let home = PrintProfile::home(Sheet::A4);
        let r = scale_ruler(&CardFormat::poche(), &home).unwrap();
        // The home sheet of the default card carries the 100 mm ruler. A 110 mm panel
        // has no room for a label beside it, so the label goes above the ticks.
        assert_eq!(r.length_mm, 100.0);
        assert_eq!(r.start.x, 2.0);
        // Band is 2..9.65 mm above the trim; ticks (2.5) + gap (0.6) + label (2.2) are
        // centred in it: base line 3.175 mm above the trim.
        assert!(close(r.start.y, -3.175), "{:?}", r.start);
        let l = lines(&r.items[0]);
        assert_eq!(l[0], (r.start, Point::new(102.0, r.start.y)));
        assert_eq!(l.len(), 1 + 11, "base line + ticks at 0, 10, ... 100");
        let heights: Vec<f64> = l[1..].iter().map(|(a, b)| a.y - b.y).collect();
        for (i, got) in heights.iter().enumerate() {
            let want = if i % 5 == 0 { 2.5 } else { 1.5 };
            assert!(close(*got, want), "{heights:?}");
        }
        for (i, (a, b)) in l[1..].iter().enumerate() {
            assert_eq!(a.x, 2.0 + 10.0 * i as f64);
            assert_eq!(a.x, b.x);
        }
        assert!(rect_close(r.label_area, Rect::new(2.0, -8.475, 14.0, 2.2)), "{:?}", r.label_area);
        assert!(point_close(r.label_origin, Point::new(2.0, -6.275)));
        // The label clears the ticks below it and the printable edge above it.
        assert!(r.label_area.bottom() < r.start.y - 2.5);
        assert!(r.label_area.y > -9.65 + 0.5);
        assert_eq!(stroke_of(&r.items[0]).color, Color::Gray(crate::GUIDE_GRAY_LEVEL));

        // A5 on A3 has room for the 100 mm ruler with its label beside it, and a tall
        // tick at 50 mm.
        let r = scale_ruler(&CardFormat::a5(), &PrintProfile::home(Sheet::A3)).unwrap();
        assert_eq!(r.length_mm, 100.0);
        let l = lines(&r.items[0]);
        assert_eq!(l.len(), 12);
        let tall: Vec<usize> =
            (0..11).filter(|i| (l[1 + i].0.y - l[1 + i].1.y - 2.5).abs() < 1e-9).collect();
        assert_eq!(tall, vec![0, 5, 10]);
        assert_eq!(r.label_origin, Point::new(103.5, r.start.y));
        assert!(rect_close(r.label_area, Rect::new(103.5, r.start.y - 2.5, 12.5, 2.5)));

        // The smallest cover gets the 20 mm ruler, label beside it.
        let small = CardFormat::custom(Size::new(50.0, 80.0), 10.0).unwrap();
        let r = scale_ruler(&small, &home).unwrap();
        assert_eq!(r.length_mm, 20.0);
        assert_eq!(r.label_origin.x, 23.5);
    }

    #[test]
    fn a_low_margin_gets_a_shorter_ruler_with_its_label_beside_it() {
        // 110 x 184 mm covers on A4: 13 mm above the card, 4.65 mm of it for marks —
        // too low to stack a label above the ticks, so the 100 mm ruler does not fit.
        let f = CardFormat::custom(Size::new(110.0, 184.0), 10.0).unwrap();
        let r = scale_ruler(&f, &PrintProfile::home(Sheet::A4)).unwrap();
        assert_eq!(r.length_mm, 50.0);
        assert_eq!(r.label_origin.x, 53.5);
        assert!(close(r.label_area.bottom(), r.start.y));
        // Lower still (1.65 mm): no ruler at all.
        let f = CardFormat::custom(Size::new(110.0, 190.0), 10.0).unwrap();
        assert!(scale_ruler(&f, &PrintProfile::home(Sheet::A4)).is_none());
    }

    #[test]
    fn on_a_large_sheet_the_furniture_hugs_the_card() {
        // A6 on A3: 74.5 mm above and below the card, 69.5 mm of it printable.
        let (f, mut p) = (CardFormat::a6(), PrintProfile::home(Sheet::A3));
        p.marks.registration = true;
        p.marks.color_bar = true;
        // Targets sit in the middle of the first 12 mm beyond the bleed, not 35 mm out.
        let c = registration_target_centers(&f, &p);
        assert_eq!(c[0], Point::new(-8.0, 74.0));
        assert_eq!(c[2], Point::new(110.0, -8.0));
        // So do the ruler (100 mm, label above), the colour bar and the caption areas.
        let r = scale_ruler(&f, &p).unwrap();
        assert_eq!(r.length_mm, 100.0);
        assert!(close(r.start.y, -5.35), "{:?}", r.start);
        assert!(r.label_area.y > -14.0);
        let Item::Path { path, .. } = &color_bar(&f, &p)[0] else { panic!() };
        assert_eq!(path.control_bounds().unwrap(), Rect::new(2.0, 153.5, 5.0, 5.0));
        let [above, below] = caption_areas(&f, &p).unwrap();
        assert_eq!(above, Rect::new(117.0, -13.5, 101.0, 11.0));
        assert_eq!(below, Rect::new(117.0, 150.5, 101.0, 11.0));
        // The dashed fold guides stop 20 mm after they start instead of running 66 mm.
        let g = all_lines(&fold_marks(&f, &p));
        assert_eq!(g[0], (Point::new(105.0, -2.5), Point::new(105.0, -22.5)));
        assert_eq!(g[1], (Point::new(105.0, 150.5), Point::new(105.0, 170.5)));
    }

    #[test]
    fn caption_areas_are_the_free_margin_beside_the_right_panel() {
        let f = CardFormat::poche();
        // Home on A4: the band runs from 2 to 9.65 mm outside the trim.
        let [above, below] = caption_areas(&f, &PrintProfile::home(Sheet::A4)).unwrap();
        assert!(rect_close(above, Rect::new(122.0, -9.15, 106.0, 6.65)), "{above:?}");
        assert!(rect_close(below, Rect::new(122.0, 180.5, 106.0, 6.65)), "{below:?}");
        // Professional: 3 to 13 mm.
        let [above, below] = caption_areas(&f, &PrintProfile::professional_eu()).unwrap();
        assert_eq!(above, Rect::new(122.0, -12.5, 106.0, 9.0));
        assert_eq!(below, Rect::new(122.0, 181.5, 106.0, 9.0));
        // No slug, no captions; no page, no captions.
        assert!(caption_areas(&f, &PrintProfile::upload_eu()).is_none());
        assert!(caption_areas(&CardFormat::a5(), &PrintProfile::home(Sheet::A4)).is_none());
    }

    #[test]
    fn marks_layer_honours_the_flags() {
        let f = CardFormat::poche();
        let count = |p: &PrintProfile| match marks(&f, p) {
            Item::Group { layer, items, clip: None, transform } => {
                assert_eq!(layer.as_deref(), Some("marks"));
                assert_eq!(transform, ebc_scene::Transform::IDENTITY);
                items.len()
            }
            other => panic!("{other:?}"),
        };
        let pro = PrintProfile::professional_eu();
        assert_eq!(count(&pro), 8 + 4, "eight crop marks + four fold marks");
        let mut all = pro.clone();
        all.marks.registration = true;
        all.marks.color_bar = true;
        all.marks.scale_ruler = true;
        assert_eq!(count(&all), 8 + 4 + 4 + 11 + 1, "+ targets, patches and the ruler");
        assert_eq!(count(&pro.clone().without_marks()), 0);
        assert_eq!(count(&PrintProfile::upload_eu()), 0);
        let mut only_fold = pro.clone().without_marks();
        only_fold.marks.fold = true;
        only_fold.slug = 10.0;
        assert_eq!(count(&only_fold), 4);
        // Home: crop + fold + ruler.
        assert_eq!(count(&PrintProfile::home(Sheet::A4)), 8 + 4 + 1);
    }

    #[test]
    fn nothing_is_drawn_when_the_page_cannot_be_built() {
        let empty = Item::layer("marks", vec![]);
        // Does not fit.
        assert_eq!(marks(&CardFormat::a5(), &PrintProfile::home(Sheet::A4)), empty);
        // Invalid format.
        let bad = CardFormat { cover: Size::new(f64::NAN, f64::INFINITY), spine_width: -1.0 };
        assert_eq!(marks(&bad, &PrintProfile::professional_eu()), empty);
        assert!(registration_target_centers(&bad, &PrintProfile::professional_eu()).is_empty());
        // Invalid profile.
        let mut p = PrintProfile::professional_eu();
        p.marks.weight_pt = f64::NAN;
        assert_eq!(marks(&CardFormat::poche(), &p), empty);
    }
}
