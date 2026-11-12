//! Printer's marks: crop marks, fold marks, registration targets, colour bar, scale ruler.
//!
//! Every mark lies inside [`PageGeometry::mark_area`](crate::PageGeometry::mark_area) and
//! outside the bleed box. The marks are the same for both faces of the card (the spread
//! is symmetric, so the creases are at the same x on the outside and the inside).
//!
//! Conventions used (crop-mark geometry is trade convention — no surveyed online printer
//! publishes one, see `docs/research/notes/print-requirements.prepress.md` §2):
//! solid strokes with butt caps; crop marks along the extension of each trim edge; fold
//! marks along the extension of each crease; nothing closer to the trim than the bleed.

use ebc_scene::{Color, Item, Path, Point, Rect, Size, Stroke, pt_to_mm};

use crate::format::CardFormat;
use crate::page::page_geometry;
use crate::profile::{MarkStyle, Marks, PrintProfile};

/// Name of the layer returned by [`marks`].
pub const MARKS_LAYER: &str = "marks";
/// Crop and fold marks that would end up shorter than this (because the slug or the
/// printable area is too small) are omitted rather than drawn as specks.
pub const MIN_MARK_LENGTH_MM: f64 = 1.0;
/// Dash pattern (on, off — millimetres) of the fold guide of [`MarkStyle::Guide`] marks.
pub const FOLD_GUIDE_DASH_MM: [f64; 2] = [1.0, 1.0];
/// Gap between a fold mark and its dashed extension, in millimetres.
const FOLD_GUIDE_GAP_MM: f64 = 1.0;
/// Longest dashed fold guide, in millimetres: on a sheet much larger than the card the
/// guide stops here instead of running all the way to the printable edge.
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
/// Width reserved after the ruler for its label (e.g. "100 mm" at 6–7 pt).
const RULER_LABEL_ROOM_MM: f64 = 14.0;

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
        // Never inside the bleed, whatever offset the profile asks for.
        let start = self.marks.offset.max(self.bleed);
        let length = self.marks.length.min(self.reach - start);
        (length >= MIN_MARK_LENGTH_MM).then_some((start, start + length))
    }

    fn half_stroke_mm(&self) -> f64 {
        pt_to_mm(self.marks.weight_pt) / 2.0
    }
}

/// Crop (trim) marks: at each of the four corners of the trim box, one horizontal and one
/// vertical stroke on the extensions of the two trim edges that meet there, starting
/// `max(marks.offset, bleed)` away from the corner and pointing outwards — so they never
/// enter the bleed. One stroked [`Item`] with eight sub-paths.
///
/// Returns nothing if the page cannot be built ([`new_page`](crate::new_page) would fail)
/// or there is less than [`MIN_MARK_LENGTH_MM`] of room. Ignores `marks.crop`.
#[must_use]
pub fn crop_marks(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let Some((a, b)) = cx.tick_span() else { return Vec::new() };
    let mut path = Path::new();
    for (x, dir_x) in [(0.0, -1.0), (cx.spread.w, 1.0)] {
        for (y, dir_y) in [(0.0, -1.0), (cx.spread.h, 1.0)] {
            // Along the horizontal trim edge, away from the corner.
            path.move_to(x + dir_x * a, y).line_to(x + dir_x * b, y);
            // Along the vertical trim edge, away from the corner.
            path.move_to(x, y + dir_y * a).line_to(x, y + dir_y * b);
        }
    }
    vec![Item::stroke(path, cx.marks.stroke())]
}

/// Fold marks: a short vertical stroke above and below the spread at each of the two
/// creases ([`CardFormat::fold_x`]), with the same offset and length as the crop marks.
/// With [`MarkStyle::Guide`] each stroke is continued, after a small gap, by a dashed
/// guide ([`FOLD_GUIDE_DASH_MM`]) running to the edge of the printable area (at most
/// [`FOLD_GUIDE_MAX_MM`] long), which tells fold marks apart from cut marks and gives a
/// ruler more to align with.
///
/// The first item holds the four solid strokes; the second, if present, the dashed
/// guides. Returns nothing if the page cannot be built or there is no room. Ignores
/// `marks.fold`.
#[must_use]
pub fn fold_marks(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let Some((a, b)) = cx.tick_span() else { return Vec::new() };
    let h = cx.spread.h;
    let mut ticks = Path::new();
    for x in cx.fold_x {
        ticks.move_to(x, -a).line_to(x, -b);
        ticks.move_to(x, h + a).line_to(x, h + b);
    }
    let mut items = vec![Item::stroke(ticks, cx.marks.stroke())];
    if cx.marks.style == MarkStyle::Guide {
        let start = b + FOLD_GUIDE_GAP_MM;
        let end = cx.reach.min(start + FOLD_GUIDE_MAX_MM);
        if end - start >= MIN_MARK_LENGTH_MM {
            let mut guide = Path::new();
            for x in cx.fold_x {
                guide.move_to(x, -start).line_to(x, -end);
                guide.move_to(x, h + start).line_to(x, h + end);
            }
            let dashed =
                Stroke::dashed(cx.marks.color(), cx.marks.weight_pt, FOLD_GUIDE_DASH_MM.to_vec());
            items.push(Item::stroke(guide, dashed));
        }
    }
    items
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
/// targets sit between the two fold marks: they shrink with the spine and are left out
/// when the spine is narrower than about 4.1 mm (the default 10 mm spine has full-size
/// targets on all four sides).
#[must_use]
pub fn registration_target_centers(format: &CardFormat, profile: &PrintProfile) -> Vec<Point> {
    Ctx::new(format, profile)
        .map(|cx| target_layout(&cx).into_iter().map(|t| t.center).collect())
        .unwrap_or_default()
}

/// Registration targets: a circle and a cross, centred on each side of the spread, in the
/// middle of the band between the bleed edge and the edge of the mark area (or of its
/// first [`FURNITURE_BAND_MAX_MM`] when the margin is wider than that). One stroked
/// [`Item`]. See [`registration_target_centers`] for the placement rule. Ignores
/// `marks.registration`.
#[must_use]
pub fn registration_targets(format: &CardFormat, profile: &PrintProfile) -> Vec<Item> {
    let Some(cx) = Ctx::new(format, profile) else { return Vec::new() };
    let targets = target_layout(&cx);
    if targets.is_empty() {
        return Vec::new();
    }
    let mut path = Path::new();
    for Target { center: c, half } in targets {
        path.extend(&Path::circle(c, half * TARGET_CIRCLE_RATIO));
        path.move_to(c.x - half, c.y).line_to(c.x + half, c.y);
        path.move_to(c.x, c.y - half).line_to(c.x, c.y + half);
    }
    vec![Item::stroke(path, cx.marks.stroke())]
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
    /// Baseline origin for a label such as "100 mm" (this crate sets no text); about
    /// 14 mm of free width follow it.
    pub label_origin: Point,
    /// The ruler itself: one stroked path (base line plus a tick every 10 mm).
    pub items: Vec<Item>,
}

/// The scale-check ruler: a horizontal line of known length above the left-hand panel,
/// between the top-left crop mark and the first fold mark, with ticks every 10 mm (taller
/// at the ends and, on the 100 mm ruler, at 50 mm). Ignores `marks.scale_ruler`.
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
    let available = cx.cover_w - 2.0 * FURNITURE_INSET_MM - RULER_LABEL_ROOM_MM;
    let length_mm = RULER_LENGTHS_MM.into_iter().find(|l| *l <= available)?;
    let x0 = FURNITURE_INSET_MM;
    // The base line is the edge nearest the card; ticks point away from it.
    let y = -(cx.bleed + (band - tick) / 2.0);
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
        label_origin: Point::new(x0 + length_mm + 1.5, y),
        items: vec![Item::stroke(path, cx.marks.stroke())],
    })
}

/// The two rectangles of the margin that no mark ever uses: above and below the
/// right-hand panel, between the second fold mark and the corner crop marks, as
/// `[above, below]`. A composer can set small print there — a file name, "print at
/// 100 %", "cut on the solid marks, fold on the dashed ones" — without checking for
/// collisions (this crate sets no text).
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
/// Contents, in order: [`crop_marks`], [`fold_marks`], [`registration_targets`],
/// [`color_bar`], [`scale_ruler`] — each only if its flag is set. Every point of every
/// path lies inside the page's media box (inside the printable area for sheet-based
/// profiles) and outside its bleed box.
///
/// The layer is empty when [`new_page`](crate::new_page) would fail for the same
/// arguments, and marks that do not fit the available margin are shortened or left out,
/// so the function never panics and never draws over the artwork.
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

    fn stroke_of(item: &Item) -> &Stroke {
        match item {
            Item::Path { stroke: Some(s), fill: None, .. } => s,
            other => panic!("not a stroked path: {other:?}"),
        }
    }

    #[test]
    fn professional_crop_marks_start_at_the_bleed_edge_and_are_5_mm_long() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        let items = crop_marks(&f, &p);
        assert_eq!(items.len(), 1);
        let s = stroke_of(&items[0]);
        assert_eq!(s.color, Color::Registration);
        assert_eq!(s.width_pt, 0.25);
        assert!(s.dash_mm.is_empty());
        let l = lines(&items[0]);
        assert_eq!(l.len(), 8, "two strokes per corner");
        // Top-left corner: horizontal then vertical.
        assert_eq!(l[0], (Point::new(-3.0, 0.0), Point::new(-8.0, 0.0)));
        assert_eq!(l[1], (Point::new(0.0, -3.0), Point::new(0.0, -8.0)));
        // Bottom-right corner.
        assert_eq!(l[6], (Point::new(233.0, 178.0), Point::new(238.0, 178.0)));
        assert_eq!(l[7], (Point::new(230.0, 181.0), Point::new(230.0, 186.0)));
    }

    #[test]
    fn professional_fold_marks_are_solid_strokes_at_both_creases() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        let items = fold_marks(&f, &p);
        assert_eq!(items.len(), 1, "no dashed guide on a press profile");
        assert!(stroke_of(&items[0]).dash_mm.is_empty());
        assert_eq!(
            lines(&items[0]),
            vec![
                (Point::new(110.0, -3.0), Point::new(110.0, -8.0)),
                (Point::new(110.0, 181.0), Point::new(110.0, 186.0)),
                (Point::new(120.0, -3.0), Point::new(120.0, -8.0)),
                (Point::new(120.0, 181.0), Point::new(120.0, 186.0)),
            ]
        );
    }

    #[test]
    fn home_fold_marks_get_a_dashed_grey_guide_to_the_printable_edge() {
        let (f, p) = (CardFormat::poche(), PrintProfile::home(Sheet::A4));
        let items = fold_marks(&f, &p);
        assert_eq!(items.len(), 2);
        let solid = stroke_of(&items[0]);
        let dashed = stroke_of(&items[1]);
        assert!(solid.dash_mm.is_empty());
        assert_eq!(dashed.dash_mm, vec![1.0, 1.0]);
        for s in [solid, dashed] {
            assert_eq!(s.color, Color::Gray(crate::GUIDE_GRAY_LEVEL));
            assert_eq!(s.width_pt, 0.5);
        }
        // Ticks 2..7 mm from the trim, guide 8..11 mm (11 = 16 mm margin - 5 mm unprintable).
        assert_eq!(lines(&items[0])[0], (Point::new(110.0, -2.0), Point::new(110.0, -7.0)));
        let g = lines(&items[1]);
        assert_eq!(g.len(), 4);
        assert_eq!(g[0], (Point::new(110.0, -8.0), Point::new(110.0, -11.0)));
        assert_eq!(g[3], (Point::new(120.0, 186.0), Point::new(120.0, 189.0)));
    }

    #[test]
    fn marks_never_start_inside_the_bleed_even_if_the_offset_says_so() {
        let mut p = PrintProfile::professional_eu();
        p.marks.offset = 0.0; // closer than the 3 mm bleed
        let l = lines(&crop_marks(&CardFormat::poche(), &p)[0]);
        assert_eq!(l[0], (Point::new(-3.0, 0.0), Point::new(-8.0, 0.0)));
        // A larger offset is honoured.
        p.marks.offset = 6.0;
        let l = lines(&crop_marks(&CardFormat::poche(), &p)[0]);
        assert_eq!(l[0], (Point::new(-6.0, 0.0), Point::new(-11.0, 0.0)));
    }

    #[test]
    fn marks_shrink_to_the_slug_and_vanish_when_there_is_none() {
        let f = CardFormat::poche();
        let mut p = PrintProfile::professional_eu();
        p.slug = 2.5; // room for 2.5 mm marks only
        let l = lines(&crop_marks(&f, &p)[0]);
        assert_eq!(l[0], (Point::new(-3.0, 0.0), Point::new(-5.5, 0.0)));
        p.slug = 0.9; // below MIN_MARK_LENGTH_MM
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
        assert_eq!(items.len(), 1);
        let Item::Path { path, .. } = &items[0] else { panic!() };
        // Four circles (4 cubics each) and four crosses (2 lines each).
        let cubics = path.segs.iter().filter(|s| matches!(s, PathSeg::CubicTo(..))).count();
        assert_eq!(cubics, 16);
        let l = lines(&items[0]);
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
        let l = lines(&registration_targets(&f, &p)[0]);
        let (a, b) = l[4];
        assert!((a.x - (110.0 + 0.5 + half_stroke)).abs() < 1e-9, "{a:?}");
        assert!((b.x - (116.0 - 0.5 - half_stroke)).abs() < 1e-9, "{b:?}");
        // The side targets keep their full size.
        assert_eq!(l[0], (Point::new(-11.5, 89.0), Point::new(-4.5, 89.0)));
        // 4 mm spine: no room for a legible target between the folds.
        let f = CardFormat::poche().with_spine(4.0).unwrap();
        let c = registration_target_centers(&f, &p);
        assert_eq!(c, vec![Point::new(-8.0, 89.0), Point::new(232.0, 89.0)]);
        assert_eq!(lines(&registration_targets(&f, &p)[0]).len(), 4);
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
        // 110 mm panel: 100 mm ruler + label does not fit, 50 mm does.
        assert_eq!(r.length_mm, 50.0);
        assert_eq!(r.start.x, 2.0);
        // Band is 2..11 mm above the trim, tick 2.5 mm: base line 5.25 mm above the trim.
        assert!((r.start.y + 5.25).abs() < 1e-9);
        let l = lines(&r.items[0]);
        assert_eq!(l[0], (r.start, Point::new(52.0, r.start.y)));
        assert_eq!(l.len(), 1 + 6, "base line + ticks at 0, 10, ... 50");
        let heights: Vec<f64> = l[1..].iter().map(|(a, b)| a.y - b.y).collect();
        for (got, want) in heights.iter().zip([2.5, 1.5, 1.5, 1.5, 1.5, 2.5]) {
            assert!((got - want).abs() < 1e-9, "{heights:?}");
        }
        for (i, (a, b)) in l[1..].iter().enumerate() {
            assert_eq!(a.x, 2.0 + 10.0 * i as f64);
            assert_eq!(a.x, b.x);
        }
        assert_eq!(r.label_origin, Point::new(53.5, r.start.y));
        assert_eq!(stroke_of(&r.items[0]).color, Color::Gray(crate::GUIDE_GRAY_LEVEL));

        // A5 on A3 has room for the 100 mm ruler, with a tall tick at 50 mm.
        let r = scale_ruler(&CardFormat::a5(), &PrintProfile::home(Sheet::A3)).unwrap();
        assert_eq!(r.length_mm, 100.0);
        let l = lines(&r.items[0]);
        assert_eq!(l.len(), 12);
        let tall: Vec<usize> =
            (0..11).filter(|i| (l[1 + i].0.y - l[1 + i].1.y - 2.5).abs() < 1e-9).collect();
        assert_eq!(tall, vec![0, 5, 10]);

        // The smallest cover gets the 20 mm ruler.
        let small = CardFormat::custom(Size::new(50.0, 80.0), 10.0).unwrap();
        assert_eq!(scale_ruler(&small, &home).unwrap().length_mm, 20.0);
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
        // So do the ruler, the colour bar and the caption areas.
        let r = scale_ruler(&f, &p).unwrap();
        assert!((r.start.y + 6.75).abs() < 1e-9, "{:?}", r.start);
        let Item::Path { path, .. } = &color_bar(&f, &p)[0] else { panic!() };
        assert_eq!(path.control_bounds().unwrap(), Rect::new(2.0, 153.5, 5.0, 5.0));
        let [above, below] = caption_areas(&f, &p).unwrap();
        assert_eq!(above, Rect::new(117.0, -13.5, 101.0, 11.0));
        assert_eq!(below, Rect::new(117.0, 150.5, 101.0, 11.0));
        // The dashed fold guides stop 20 mm after they start instead of running 60 mm.
        let g = lines(&fold_marks(&f, &p)[1]);
        assert_eq!(g[0], (Point::new(105.0, -8.0), Point::new(105.0, -28.0)));
        assert_eq!(g[1], (Point::new(105.0, 156.0), Point::new(105.0, 176.0)));
    }

    #[test]
    fn caption_areas_are_the_free_margin_beside_the_right_panel() {
        let f = CardFormat::poche();
        // Home on A4: the band runs from 2 to 11 mm outside the trim.
        let [above, below] = caption_areas(&f, &PrintProfile::home(Sheet::A4)).unwrap();
        assert_eq!(above, Rect::new(122.0, -10.5, 106.0, 8.0));
        assert_eq!(below, Rect::new(122.0, 180.5, 106.0, 8.0));
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
        assert_eq!(count(&pro), 3, "crop + fold + registration");
        let mut all = pro.clone();
        all.marks.color_bar = true;
        all.marks.scale_ruler = true;
        assert_eq!(count(&all), 3 + 11 + 1);
        assert_eq!(count(&pro.clone().without_marks()), 0);
        assert_eq!(count(&PrintProfile::upload_eu()), 0);
        let mut only_fold = pro.clone().without_marks();
        only_fold.marks.fold = true;
        only_fold.slug = 10.0;
        assert_eq!(count(&only_fold), 1);
        // Home: crop + fold ticks + fold guides + ruler.
        assert_eq!(count(&PrintProfile::home(Sheet::A4)), 4);
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
