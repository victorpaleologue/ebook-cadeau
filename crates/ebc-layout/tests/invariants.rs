//! Geometric invariants of the layout, checked over every preset, every built-in profile
//! and a few thousand pseudo-random formats and profiles (fixed seeds: deterministic).

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, FOLD_GUIDE_MAX_MM, FURNITURE_BAND_MAX_MM, Face, Fit, LayoutError,
    MARK_BLEED_GAP_MM, MIN_MARK_LENGTH_MM, MarkStyle, PageGeometry, PrintProfile, Resolution,
    Sheet, caption_areas, check_marks_fit, color_bar, crop_marks, effective_ppi, fold_marks,
    guides, info_entries, marks, new_page, page_geometry, place_image, place_image_aligned,
    place_image_bleed, registration_target_centers, registration_targets, safe_panels, safe_rect,
    scale_ruler, spine_safe_rect,
};
use ebc_scene::{Color, Item, Path, PathSeg, Point, Rect, Size, Stroke, Transform, pt_to_mm};

const EPS: f64 = 1e-9;

// ---------------------------------------------------------------------------------------
// A tiny deterministic generator (SplitMix64) — no dependency, same sequence everywhere.
// ---------------------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    fn flip(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    fn format(&mut self) -> CardFormat {
        // One time in eight, snap to whole millimetres or to the limits.
        let snap = self.below(8) == 0;
        let mut v = |lo: f64, hi: f64| {
            let x = self.range(lo, hi);
            if snap { x.round().clamp(lo, hi) } else { x }
        };
        let (w, h, s) = (v(50.0, 300.0), v(50.0, 300.0), v(3.0, 40.0));
        CardFormat::custom(Size::new(w, h), s).unwrap()
    }

    fn sheet(&mut self) -> Sheet {
        match self.below(6) {
            0 => Sheet::A4,
            1 => Sheet::Letter,
            2 => Sheet::A3,
            3 => Sheet::Sra3,
            _ => Sheet::Custom(Size::new(self.range(80.0, 900.0), self.range(80.0, 900.0))),
        }
    }

    fn profile(&mut self) -> PrintProfile {
        let mut p = if self.flip() {
            PrintProfile::professional_eu()
        } else {
            PrintProfile::home(self.sheet())
        };
        if self.below(4) == 0 {
            // A press profile imposed on a sheet, or a home profile without one.
            p.sheet = if p.sheet.is_some() { None } else { Some(self.sheet()) };
        }
        p.bleed = if self.below(6) == 0 { 0.0 } else { self.range(0.0, 6.0) };
        p.slug = if self.below(6) == 0 { 0.0 } else { self.range(0.0, 25.0) };
        p.safe_margin = self.range(0.0, 12.0);
        p.spine_safe_margin = self.range(0.0, 5.0);
        p.printable_margin = if self.below(6) == 0 { 0.0 } else { self.range(0.0, 9.0) };
        p.marks.offset = self.range(0.0, 9.0);
        p.marks.length = self.range(0.0, 12.0);
        p.marks.weight_pt = self.range(0.05, 2.0);
        p.marks.style = match self.below(4) {
            0 | 1 => MarkStyle::Press,
            2 => MarkStyle::Guide,
            _ => MarkStyle::Black,
        };
        p.marks.crop = self.flip();
        p.marks.fold = self.flip();
        p.marks.registration = self.flip();
        p.marks.color_bar = self.flip();
        p.marks.scale_ruler = self.flip();
        p
    }
}

// ---------------------------------------------------------------------------------------
// Walking what a layer draws.
// ---------------------------------------------------------------------------------------

/// One sub-path: every point it mentions (control points included), its straight
/// segments (the closing one included), and whether it has curves.
#[derive(Debug, Default)]
struct SubPath {
    points: Vec<Point>,
    segments: Vec<(Point, Point)>,
    curved: bool,
}

fn subpaths(path: &Path) -> Vec<SubPath> {
    let mut out: Vec<SubPath> = Vec::new();
    let mut start = Point::default();
    let mut cur = Point::default();
    for seg in &path.segs {
        match *seg {
            PathSeg::MoveTo(p) => {
                out.push(SubPath { points: vec![p], ..SubPath::default() });
                start = p;
                cur = p;
            }
            PathSeg::LineTo(p) => {
                let sp = out.last_mut().expect("LineTo before MoveTo");
                sp.points.push(p);
                sp.segments.push((cur, p));
                cur = p;
            }
            PathSeg::QuadTo(c, p) => {
                let sp = out.last_mut().expect("QuadTo before MoveTo");
                sp.points.extend([c, p]);
                sp.curved = true;
                cur = p;
            }
            PathSeg::CubicTo(c1, c2, p) => {
                let sp = out.last_mut().expect("CubicTo before MoveTo");
                sp.points.extend([c1, c2, p]);
                sp.curved = true;
                cur = p;
            }
            PathSeg::Close => {
                let sp = out.last_mut().expect("Close before MoveTo");
                if cur != start {
                    sp.segments.push((cur, start));
                }
                cur = start;
            }
        }
    }
    out
}

struct Drawn<'a> {
    path: &'a Path,
    stroke: Option<&'a Stroke>,
    fill: Option<Color>,
}

/// Everything a marks/guides layer draws. Such a layer may only contain plain paths.
fn drawn(items: &[Item]) -> Vec<Drawn<'_>> {
    items
        .iter()
        .map(|item| match item {
            Item::Path { path, fill, stroke } => {
                assert!(fill.is_some() != stroke.is_some(), "either filled or stroked: {item:?}");
                assert!(!path.is_empty());
                Drawn { path, stroke: stroke.as_ref(), fill: fill.map(|(c, _)| c) }
            }
            other => panic!("unexpected item in a marks layer: {other:?}"),
        })
        .collect()
}

fn layer_items<'a>(layer: &'a Item, name: &str) -> &'a [Item] {
    match layer {
        Item::Group { transform, clip: None, layer: Some(l), items } => {
            assert_eq!(l, name);
            assert_eq!(*transform, Transform::IDENTITY);
            items
        }
        other => panic!("not a plain named layer: {other:?}"),
    }
}

fn bounds(points: &[Point]) -> Rect {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in points {
        assert!(p.x.is_finite() && p.y.is_finite(), "non-finite point {p:?}");
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

/// The area a sub-path inks: its bounds, widened by half the stroke width — except along
/// the direction of a straight butt-capped line, which ends exactly at its end points.
fn ink(sp: &SubPath, stroke: Option<&Stroke>) -> Rect {
    let b = bounds(&sp.points);
    let Some(stroke) = stroke else { return b };
    let hw = pt_to_mm(stroke.width_pt) / 2.0;
    let straight = !sp.curved && sp.segments.len() == 1;
    if straight && b.h == 0.0 {
        Rect::new(b.x, b.y - hw, b.w, 2.0 * hw)
    } else if straight && b.w == 0.0 {
        Rect::new(b.x - hw, b.y, 2.0 * hw, b.h)
    } else {
        Rect::new(b.x - hw, b.y - hw, b.w + 2.0 * hw, b.h + 2.0 * hw)
    }
}

/// `true` if the two rectangles share more than a boundary.
fn overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.right() - EPS
        && a.right() > b.x + EPS
        && a.y < b.bottom() - EPS
        && a.bottom() > b.y + EPS
}

fn strictly_inside(r: &Rect, p: Point) -> bool {
    p.x > r.x + EPS && p.x < r.right() - EPS && p.y > r.y + EPS && p.y < r.bottom() - EPS
}

fn within(r: &Rect, p: Point) -> bool {
    p.x >= r.x - EPS && p.x <= r.right() + EPS && p.y >= r.y - EPS && p.y <= r.bottom() + EPS
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPS
}

fn rect_close(a: &Rect, b: &Rect) -> bool {
    close(a.x, b.x) && close(a.y, b.y) && close(a.w, b.w) && close(a.h, b.h)
}

// ---------------------------------------------------------------------------------------
// The checks.
// ---------------------------------------------------------------------------------------

/// Every point of every path is inside the media box and the mark area and outside the
/// bleed box; the same holds for the ink once stroke widths are taken into account.
///
/// What a preflight tool sees is coarser — one box per painted object, around all of its
/// sub-paths, grown by half the stroke width in every direction because the cap style is
/// unknown to it. That box must stay clear of the bleed box as well, or marks in the
/// registration colour are reported as lying in the printed area.
fn check_layer_stays_in_the_margin(items: &[Item], g: &PageGeometry, ctx: &str) {
    for d in drawn(items) {
        if let Some(s) = d.stroke {
            assert!(s.width_pt > 0.0 && s.width_pt.is_finite(), "{ctx}");
            assert!(s.dash_mm.iter().all(|v| *v > 0.0 && v.is_finite()), "{ctx}");
        }
        let all: Vec<Point> = subpaths(d.path).into_iter().flat_map(|sp| sp.points).collect();
        let object = bounds(&all).inflate(d.stroke.map_or(0.0, |s| pt_to_mm(s.width_pt) / 2.0));
        assert!(
            !overlap(&object, &g.bleed),
            "{ctx}: the box of one object, {object:?}, enters the bleed {:?}",
            g.bleed
        );
        for sp in subpaths(d.path) {
            for p in &sp.points {
                assert!(within(&g.media, *p), "{ctx}: {p:?} outside media {:?}", g.media);
                assert!(within(&g.mark_area, *p), "{ctx}: {p:?} outside {:?}", g.mark_area);
                assert!(!strictly_inside(&g.bleed, *p), "{ctx}: {p:?} inside bleed {:?}", g.bleed);
            }
            // A straight segment could cross the bleed box with both ends outside it; its
            // bounds would then overlap the box. Same test covers curves and patches.
            let ink = ink(&sp, d.stroke);
            assert!(!overlap(&ink, &g.bleed), "{ctx}: ink {ink:?} enters bleed {:?}", g.bleed);
            assert!(g.media.contains_rect(&ink, EPS), "{ctx}: ink {ink:?} leaves the media box");
            assert!(g.mark_area.contains_rect(&ink, EPS), "{ctx}: ink {ink:?} leaves mark area");
            if let Some(s) = &g.sheet {
                assert!(s.printable.contains_rect(&ink, EPS), "{ctx}: ink {ink:?} not printable");
            }
        }
    }
}

fn check_page(format: &CardFormat, profile: &PrintProfile, g: &PageGeometry, ctx: &str) {
    let page = new_page(format, profile).unwrap();
    let spread = format.spread();
    assert_eq!(page.trim, spread, "{ctx}");
    assert_eq!(page.bleed, profile.bleed, "{ctx}");
    assert!(page.items.is_empty() && page.label.is_none());
    assert_eq!(page.trim_box(), g.trim, "{ctx}");
    assert_eq!(page.bleed_box(), g.bleed, "{ctx}");
    assert_eq!(page.media_box(), g.media, "{ctx}");
    assert_eq!(g.trim, Rect::new(0.0, 0.0, spread.w, spread.h));
    // Trim within bleed within media, with the right amounts.
    assert!(g.bleed.contains_rect(&g.trim, EPS), "{ctx}");
    assert!(g.media.contains_rect(&g.bleed, EPS), "{ctx}");
    assert!(rect_close(&g.bleed, &g.trim.inflate(profile.bleed)), "{ctx}");
    assert!(page.slug >= 0.0 && page.slug.is_finite(), "{ctx}");
    // Marks may use the band between bleed and mark area, nothing else.
    assert!(g.media.contains_rect(&g.mark_area, EPS), "{ctx}");
    assert!(g.mark_area.contains_rect(&g.bleed, EPS), "{ctx}");
    assert!(g.mark_band_mm() >= 0.0);

    match (&profile.sheet, &g.sheet) {
        (None, None) => {
            assert_eq!(page.slug, profile.slug, "{ctx}");
            assert_eq!(page.slug_v, None, "{ctx}: a press page has one slug");
            assert_eq!((g.slug_mm, g.slug_v_mm), (profile.slug, profile.slug), "{ctx}");
            assert!(rect_close(&g.media, &g.trim.inflate(profile.bleed + profile.slug)), "{ctx}");
            assert!(rect_close(&g.mark_area, &g.media), "{ctx}");
        }
        (Some(sheet), Some(s)) => {
            let size = if s.landscape { sheet.landscape() } else { sheet.portrait() };
            assert!(close(s.sheet.w, size.w) && close(s.sheet.h, size.h), "{ctx}");
            // Centred: same centre as the spread, hence both faces register.
            assert!(close(s.sheet.center().x, g.trim.center().x), "{ctx}");
            assert!(close(s.sheet.center().y, g.trim.center().y), "{ctx}");
            assert!(close(g.media.center().x, s.sheet.center().x), "{ctx}");
            assert!(close(g.media.center().y, s.sheet.center().y), "{ctx}");
            // The page is the sheet: a slug per direction makes up the sheet's margins.
            assert!(rect_close(&g.media, &s.sheet), "{ctx}: {:?} vs {:?}", g.media, s.sheet);
            assert!(s.media_is_sheet, "{ctx}");
            assert_eq!(page.slug, g.slug_mm, "{ctx}");
            assert_eq!(page.slug_v, Some(g.slug_v_mm), "{ctx}");
            assert!(g.slug_v_mm >= 0.0 && g.slug_v_mm.is_finite(), "{ctx}");
            assert!(close(profile.bleed + g.slug_mm, (size.w - spread.w) / 2.0), "{ctx}");
            assert!(close(profile.bleed + g.slug_v_mm, (size.h - spread.h) / 2.0), "{ctx}");
            // Bleed and marks are printable.
            assert!(rect_close(&s.printable, &s.sheet.inflate(-profile.printable_margin)));
            assert!(s.printable.contains_rect(&g.bleed, 1e-6), "{ctx}");
            assert!(s.printable.contains_rect(&g.mark_area, 1e-6), "{ctx}");
            // The sheet takes the orientation of the spread (a square spread counts as
            // landscape), so page, sheet and spread are all the same way round...
            assert_eq!(s.landscape, spread.w >= spread.h, "{ctx}");
            if s.landscape {
                assert!(g.media.w >= g.media.h - EPS, "{ctx}");
            } else {
                assert!(g.media.w <= g.media.h + EPS, "{ctx}");
            }
            assert_eq!(s.duplex_flip(), g.duplex_flip(), "{ctx}");
            // ...and that is the orientation with the most room on its tighter side: the
            // other one never offers a wider band for marks.
            let other = if s.landscape { sheet.portrait() } else { sheet.landscape() };
            let tight = |size: Size| ((size.w - spread.w) / 2.0).min((size.h - spread.h) / 2.0);
            assert!(tight(size) >= tight(other) - EPS, "{ctx}: the other orientation is roomier");
            assert!(
                close(g.mark_reach_mm, (tight(size) - profile.printable_margin).max(profile.bleed)),
                "{ctx}"
            );
        }
        other => panic!("{ctx}: sheet mismatch {other:?}"),
    }
}

fn check_crop_marks(format: &CardFormat, profile: &PrintProfile, g: &PageGeometry, ctx: &str) {
    let items = crop_marks(format, profile);
    // Never on the bleed edge: at least MARK_BLEED_GAP_MM beyond it.
    let start = profile.marks.offset.max(profile.bleed + MARK_BLEED_GAP_MM);
    let room = g.mark_reach_mm - start;
    let expected_len = profile.marks.length.min(room);
    if expected_len < MIN_MARK_LENGTH_MM {
        assert!(items.is_empty(), "{ctx}: marks drawn without room");
        return;
    }
    assert_eq!(items.len(), 8, "{ctx}: two marks per corner, each its own item");
    let (w, h) = (format.spread().w, format.spread().h);
    // corner index: 0 = top-left, 1 = top-right, 2 = bottom-left, 3 = bottom-right
    let mut horizontal = [0u8; 4];
    let mut vertical = [0u8; 4];
    for d in &drawn(&items) {
        let stroke = d.stroke.expect("crop marks are stroked");
        assert!(stroke.dash_mm.is_empty(), "{ctx}: crop marks are solid");
        assert_eq!(stroke.color, profile.marks.color());
        assert_eq!(stroke.width_pt, profile.marks.weight_pt);
        let sps = subpaths(d.path);
        assert_eq!(sps.len(), 1, "{ctx}: one stroke per item");
        let sp = &sps[0];
        assert!(!sp.curved && sp.segments.len() == 1, "{ctx}");
        let (a, b) = sp.segments[0];
        let len = if a.y == b.y {
            // Along the extension of the top or bottom trim edge — exactly.
            assert!(a.y == 0.0 || a.y == h, "{ctx}: horizontal crop mark at y = {}", a.y);
            let left = a.x < 0.0 && b.x < 0.0;
            let right = a.x > w && b.x > w;
            assert!(left ^ right, "{ctx}: {a:?} {b:?}");
            let near = if left { -a.x.max(b.x) } else { a.x.min(b.x) - w };
            assert!(close(near, start), "{ctx}: starts {near} from the trim, wanted {start}");
            horizontal[usize::from(a.y == h) * 2 + usize::from(right)] += 1;
            (a.x - b.x).abs()
        } else {
            assert_eq!(a.x, b.x, "{ctx}: crop marks are axis-aligned");
            assert!(a.x == 0.0 || a.x == w, "{ctx}: vertical crop mark at x = {}", a.x);
            let top = a.y < 0.0 && b.y < 0.0;
            let bottom = a.y > h && b.y > h;
            assert!(top ^ bottom, "{ctx}: {a:?} {b:?}");
            let near = if top { -a.y.max(b.y) } else { a.y.min(b.y) - h };
            assert!(close(near, start), "{ctx}: starts {near} from the trim, wanted {start}");
            vertical[usize::from(bottom) * 2 + usize::from(a.x == w)] += 1;
            (a.y - b.y).abs()
        };
        assert!(close(len, expected_len), "{ctx}: length {len}, wanted {expected_len}");
    }
    assert_eq!(horizontal, [1; 4], "{ctx}");
    assert_eq!(vertical, [1; 4], "{ctx}");
}

fn check_fold_marks(format: &CardFormat, profile: &PrintProfile, g: &PageGeometry, ctx: &str) {
    let items = fold_marks(format, profile);
    let start = profile.marks.offset.max(profile.bleed + MARK_BLEED_GAP_MM);
    let expected_len = profile.marks.length.min(g.mark_reach_mm - start);
    if expected_len < MIN_MARK_LENGTH_MM {
        assert!(items.is_empty(), "{ctx}: fold marks drawn without room");
        return;
    }
    // A fold mark is as long as a crop mark, except in the guide style, where it runs on
    // to the limit of the mark area or for 20 mm, whichever comes first.
    let expected_len = match profile.marks.style {
        MarkStyle::Press | MarkStyle::Black => expected_len,
        MarkStyle::Guide => (g.mark_reach_mm - start).min(FOLD_GUIDE_MAX_MM).max(expected_len),
    };
    assert_eq!(items.len(), 4, "{ctx}: above and below, at both creases; no solid part");
    let [f0, f1] = format.fold_x();
    let h = format.spread().h;
    let mut seen = [[false; 2]; 2];
    for d in &drawn(&items) {
        let stroke = d.stroke.expect("fold marks are stroked");
        assert_eq!(stroke.color, profile.marks.color());
        assert_eq!(stroke.width_pt, profile.marks.weight_pt);
        // Dashed in every style — a crease must never read as a cut — with the nominal
        // 1 mm on / 1 mm off pattern, or a finer one on a mark too short for three dashes.
        let [on, off] = stroke.dash_mm[..] else {
            panic!("{ctx}: fold marks are dashed on/off, got {:?}", stroke.dash_mm)
        };
        assert!(on > 0.0 && on <= 1.0 + EPS && close(on, off), "{ctx}: dash {on}/{off}");
        assert!(3.0 * on + 2.0 * off <= expected_len + EPS, "{ctx}: fewer than three dashes");
        assert!(close(on, 1.0) || close(5.0 * on, expected_len), "{ctx}: dash {on}/{off}");
        let sps = subpaths(d.path);
        assert_eq!(sps.len(), 1, "{ctx}: one stroke per item");
        for sp in &sps {
            assert!(!sp.curved && sp.segments.len() == 1, "{ctx}");
            let (a, b) = sp.segments[0];
            assert_eq!(a.x, b.x, "{ctx}: fold marks are vertical");
            // Exactly on a crease: bit-for-bit the value of fold_x().
            let fold = if a.x == f0 {
                0
            } else if a.x == f1 {
                1
            } else {
                panic!("{ctx}: fold mark at x = {} but creases are at {f0} and {f1}", a.x)
            };
            let above = a.y < 0.0 && b.y < 0.0;
            let below = a.y > h && b.y > h;
            assert!(above ^ below, "{ctx}: {a:?} {b:?}");
            let (near, far) = if above {
                (-a.y.max(b.y), -a.y.min(b.y))
            } else {
                (a.y.min(b.y) - h, a.y.max(b.y) - h)
            };
            assert!(
                near >= profile.bleed + MARK_BLEED_GAP_MM - EPS,
                "{ctx}: fold mark on or inside the bleed edge"
            );
            assert!(far <= g.mark_reach_mm + EPS, "{ctx}: fold mark beyond the mark area");
            assert!(close(near, start) && close(far - near, expected_len), "{ctx}");
            // Drawn outwards, so that the dash pattern starts at the end nearest the card.
            let (from, to) = if above { (-a.y, -b.y) } else { (a.y - h, b.y - h) };
            assert!(from < to, "{ctx}: fold mark drawn towards the card");
            assert!(!seen[fold][usize::from(below)], "{ctx}: duplicate fold mark");
            seen[fold][usize::from(below)] = true;
        }
    }
    assert_eq!(seen, [[true; 2]; 2], "{ctx}");
}

/// Ink rectangles of every sub-path of a list of items.
fn ink_rects(items: &[Item]) -> Vec<Rect> {
    drawn(items)
        .iter()
        .flat_map(|d| subpaths(d.path).iter().map(|sp| ink(sp, d.stroke)).collect::<Vec<_>>())
        .collect()
}

/// Marks of different kinds never run into each other.
fn check_marks_do_not_collide(format: &CardFormat, profile: &PrintProfile, ctx: &str) {
    let kinds: [(&str, Vec<Rect>); 5] = [
        ("crop", ink_rects(&crop_marks(format, profile))),
        ("fold", ink_rects(&fold_marks(format, profile))),
        ("registration", ink_rects(&registration_targets(format, profile))),
        ("colour bar", ink_rects(&color_bar(format, profile))),
        ("ruler", ink_rects(&scale_ruler(format, profile).map(|r| r.items).unwrap_or_default())),
    ];
    for (i, (name_a, a)) in kinds.iter().enumerate() {
        for (name_b, b) in &kinds[i + 1..] {
            for ra in a {
                for rb in b {
                    assert!(!overlap(ra, rb), "{ctx}: {name_a} {ra:?} overlaps {name_b} {rb:?}");
                }
            }
        }
    }
}

/// Crop marks, fold marks and registration targets look the same after the sheet is
/// turned over (mirror about the vertical centre line) or upside down, so the marks of
/// the two faces coincide.
fn check_marks_are_symmetric(format: &CardFormat, profile: &PrintProfile, ctx: &str) {
    let (w, h) = (format.spread().w, format.spread().h);
    let mut segs: Vec<(Point, Point)> = Vec::new();
    for items in [crop_marks(format, profile), fold_marks(format, profile)] {
        for d in drawn(&items) {
            segs.extend(subpaths(d.path).into_iter().flat_map(|sp| sp.segments));
        }
    }
    let same = |a: Point, b: Point| close(a.x, b.x) && close(a.y, b.y);
    let has = |a: Point, b: Point| {
        segs.iter().any(|(c, d)| (same(a, *c) && same(b, *d)) || (same(a, *d) && same(b, *c)))
    };
    for (a, b) in &segs {
        let flip_x = |p: &Point| Point::new(w - p.x, p.y);
        let flip_y = |p: &Point| Point::new(p.x, h - p.y);
        assert!(has(flip_x(a), flip_x(b)), "{ctx}: no mirror image of {a:?}-{b:?}");
        assert!(has(flip_y(a), flip_y(b)), "{ctx}: no upside-down image of {a:?}-{b:?}");
    }
    let centres = registration_target_centers(format, profile);
    for c in &centres {
        assert!(centres.iter().any(|d| same(Point::new(w - c.x, c.y), *d)), "{ctx}: {c:?}");
        assert!(centres.iter().any(|d| same(Point::new(c.x, h - c.y), *d)), "{ctx}: {c:?}");
    }
}

fn check_colors(layer_items: &[Item], profile: &PrintProfile, ctx: &str) {
    for d in drawn(layer_items) {
        if let Some(stroke) = d.stroke {
            assert_eq!(stroke.color, profile.marks.color(), "{ctx}");
            match (profile.marks.style, stroke.color) {
                (MarkStyle::Press, Color::Registration) => {}
                // Digital jobs: black only.
                (MarkStyle::Black, c) => assert_eq!(c, Color::BLACK, "{ctx}"),
                // Home guides: light grey on the black plate only.
                (MarkStyle::Guide, Color::Gray(v)) => assert!((0.3..0.9).contains(&v), "{ctx}"),
                other => panic!("{ctx}: wrong mark colour {other:?}"),
            }
        }
        if let Some(fill) = d.fill {
            // Only the colour bar is filled: plain process colours, never RGB or
            // registration, never above 200 % ink.
            let Color::Cmyk { .. } = fill else { panic!("{ctx}: fill {fill:?}") };
            assert!(fill.total_ink_percent().unwrap() <= 200.0, "{ctx}");
        }
    }
}

/// Runs every invariant on one combination. Returns `true` if the page could be built.
fn check_combination(format: &CardFormat, profile: &PrintProfile, ctx: &str) -> bool {
    let layer = marks(format, profile);
    let items = layer_items(&layer, "marks");
    let g = match page_geometry(format, profile) {
        Ok(g) => g,
        Err(e) => {
            // Formats and profiles reaching this function are valid, so the only possible
            // refusal is a sheet that is too small — in BOTH orientations. The error
            // reports the sheet the way round it was tried: that of the spread.
            let LayoutError::DoesNotFit { needed, sheet: reported } = &e else {
                panic!("{ctx}: {e}")
            };
            let sheet = profile.sheet.expect("only a sheet can make a page impossible");
            let margin = 2.0 * (profile.bleed + profile.printable_margin);
            let spread = format.spread();
            assert!(close(needed.w, spread.w + margin) && close(needed.h, spread.h + margin));
            let tried = if spread.w >= spread.h { sheet.landscape() } else { sheet.portrait() };
            assert_eq!(*reported, tried, "{ctx}");
            assert_eq!(check_marks_fit(format, profile), Err(e.clone()), "{ctx}");
            for s in [sheet.landscape(), sheet.portrait()] {
                assert!(needed.w > s.w || needed.h > s.h, "{ctx}: {needed:?} fits {s:?}");
            }
            assert_eq!(new_page(format, profile), Err(e), "{ctx}");
            // Nothing is drawn for a page that does not exist.
            assert!(items.is_empty(), "{ctx}");
            assert!(crop_marks(format, profile).is_empty());
            assert!(fold_marks(format, profile).is_empty());
            assert!(registration_targets(format, profile).is_empty());
            assert!(registration_target_centers(format, profile).is_empty());
            assert!(color_bar(format, profile).is_empty());
            assert!(scale_ruler(format, profile).is_none());
            assert!(caption_areas(format, profile).is_none());
            assert!(layer_items(&guides(format, profile), "guides").is_empty());
            return false;
        }
    };
    check_page(format, profile, &g, ctx);

    // The layer as the profile asks for it...
    check_layer_stays_in_the_margin(items, &g, ctx);
    check_colors(items, profile, ctx);
    // ...and every generator on its own, whatever the flags say.
    let ruler = scale_ruler(format, profile);
    let all: Vec<Item> = [
        crop_marks(format, profile),
        fold_marks(format, profile),
        registration_targets(format, profile),
        color_bar(format, profile),
        ruler.as_ref().map(|r| r.items.clone()).unwrap_or_default(),
    ]
    .concat();
    check_layer_stays_in_the_margin(&all, &g, ctx);
    check_colors(&all, profile, ctx);
    check_crop_marks(format, profile, &g, ctx);
    check_fold_marks(format, profile, &g, ctx);
    // `check_marks_fit` is an error exactly when wanted cut or fold marks are left out.
    let wanted = profile.marks.crop || profile.marks.fold;
    let left_out = crop_marks(format, profile).is_empty();
    assert_eq!(left_out, fold_marks(format, profile).is_empty(), "{ctx}");
    match check_marks_fit(format, profile) {
        Ok(()) => assert!(!(wanted && left_out), "{ctx}: marks left out unnoticed"),
        Err(LayoutError::MarksDoNotFit { available_mm, needed_mm }) => {
            assert!(wanted && left_out, "{ctx}: marks reported missing but drawn or unwanted");
            assert!(available_mm >= 0.0 && available_mm < needed_mm, "{ctx}");
            assert_eq!(needed_mm, MIN_MARK_LENGTH_MM, "{ctx}");
        }
        Err(other) => panic!("{ctx}: {other}"),
    }
    check_marks_do_not_collide(format, profile, ctx);
    check_marks_are_symmetric(format, profile, ctx);

    // The flags select exactly the generators' output, in order.
    let m = &profile.marks;
    let mut expected: Vec<Item> = Vec::new();
    for (on, part) in [
        (m.crop, crop_marks(format, profile)),
        (m.fold, fold_marks(format, profile)),
        (m.registration, registration_targets(format, profile)),
        (m.color_bar, color_bar(format, profile)),
        (m.scale_ruler, ruler.as_ref().map(|r| r.items.clone()).unwrap_or_default()),
    ] {
        if on {
            expected.extend(part);
        }
    }
    assert_eq!(items, expected.as_slice(), "{ctx}");

    if let Some(r) = &ruler {
        // The ruler really has the length it claims, and its label fits before the fold.
        let sps = subpaths(drawn(&r.items)[0].path);
        let (a, b) = sps[0].segments[0];
        assert_eq!(a, r.start);
        assert!(close(b.x - a.x, r.length_mm) && a.y == b.y, "{ctx}");
        assert!([100.0, 50.0, 20.0].contains(&r.length_mm));
        assert_eq!(sps.len() as f64, 1.0 + r.length_mm / 10.0 + 1.0, "{ctx}: one tick per cm");
        assert!(within(&g.mark_area, r.label_origin), "{ctx}");
        assert!(r.label_origin.x + 10.0 < format.fold_x()[0], "{ctx}");
    }

    // The caption areas are free: no mark of any kind touches them, they are printable
    // and they stay off the artwork.
    if let Some(areas) = caption_areas(format, profile) {
        for area in areas {
            assert!(area.w >= 46.0 && area.h >= 2.0 - EPS, "{ctx}: {area:?}");
            assert!(g.mark_area.contains_rect(&area, EPS), "{ctx}: {area:?}");
            assert!(!overlap(&area, &g.bleed), "{ctx}: {area:?}");
            for ink in ink_rects(&all) {
                assert!(!overlap(&area, &ink), "{ctx}: caption area {area:?} hits {ink:?}");
            }
        }
    }

    // Registration targets: centred on their side, in the middle of the mark band.
    let centres = registration_target_centers(format, profile);
    assert!(matches!(centres.len(), 0 | 2 | 4), "{ctx}");
    let (w, h) = (format.spread().w, format.spread().h);
    let d = profile.bleed + g.mark_band_mm().min(FURNITURE_BAND_MAX_MM) / 2.0;
    for (c, want) in centres.iter().zip([
        Point::new(-d, h / 2.0),
        Point::new(w + d, h / 2.0),
        Point::new(w / 2.0, -d),
        Point::new(w / 2.0, h + d),
    ]) {
        assert!(close(c.x, want.x) && close(c.y, want.y), "{ctx}: {c:?} vs {want:?}");
    }

    // The guides overlay stays on the page.
    for d in drawn(layer_items(&guides(format, profile), "guides")) {
        for sp in subpaths(d.path) {
            assert!(sp.points.iter().all(|p| within(&g.bleed, *p)), "{ctx}");
        }
    }
    true
}

fn built_in_profiles() -> Vec<PrintProfile> {
    let mut v = vec![PrintProfile::professional_eu(), PrintProfile::upload_eu()];
    v.extend(Sheet::NAMED.map(|(_, sheet)| PrintProfile::home(sheet)));
    v.push(PrintProfile::home(Sheet::Custom(Size::new(330.0, 487.0))));
    // Everything switched on, in both styles, with and without a sheet.
    let mut all_on = Vec::new();
    for base in &v {
        let mut p = base.clone();
        p.marks.crop = true;
        p.marks.fold = true;
        p.marks.registration = true;
        p.marks.color_bar = true;
        p.marks.scale_ruler = true;
        if p.sheet.is_none() {
            p.slug = 10.0;
        }
        p.name.push_str("+all");
        all_on.push(p);
    }
    v.extend(all_on);
    // A press profile imposed one-up on SRA3.
    let mut imposed = PrintProfile::professional_eu();
    imposed.sheet = Some(Sheet::Sra3);
    imposed.name.push_str("@sra3");
    v.push(imposed);
    v
}

// ---------------------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------------------

fn presets_and_random_formats(rng: &mut Rng, n: usize) -> Vec<CardFormat> {
    let mut v: Vec<CardFormat> = CardFormat::presets().into_iter().map(|(_, f)| f).collect();
    v.extend((0..n).map(|_| rng.format()));
    v
}

#[test]
fn panels_tile_the_spread_exactly() {
    let mut rng = Rng(0x5eed_0001);
    for f in presets_and_random_formats(&mut rng, 2000) {
        let spread = f.spread();
        for face in [Face::Outside, Face::Inside] {
            let p = f.panels(face);
            // No gap, no overlap — exact equality, not "close".
            assert_eq!(p.left.x, 0.0, "{f:?}");
            assert_eq!(p.left.right(), p.spine.x, "{f:?}");
            assert_eq!(p.spine.right(), p.right.x, "{f:?}");
            assert_eq!(p.right.right(), spread.w, "{f:?}");
            for r in p.to_array() {
                assert_eq!((r.y, r.h), (0.0, spread.h), "{f:?}");
                assert!(r.w > 0.0);
            }
            assert_eq!(p.left.w, f.cover.w);
            assert_eq!(p.right.w, f.cover.w);
            assert_eq!(p.spine.w, f.spine_width);
            assert_eq!(p.bounds(), f.trim_rect(), "{f:?}");
            assert_eq!(f.fold_x(), [p.spine.x, p.right.x], "{f:?}");
            let area: f64 = p.to_array().iter().map(|r| r.w * r.h).sum();
            assert!((area - spread.w * spread.h).abs() <= 1e-9 * area, "{f:?}");
        }
        assert!(close(spread.w, 2.0 * f.cover.w + f.spine_width));
        assert_eq!(spread.h, f.cover.h);
        // Both creases are strictly inside the spread and `spine_width` apart.
        let [a, b] = f.fold_x();
        assert!(0.0 < a && a < b && b < spread.w);
        assert!(close(b - a, f.spine_width));
        // The panels grown into the bleed tile the bleed box.
        let bleed = rng.range(0.0, 6.0);
        let grown = f.outside().with_bleed(bleed);
        // The creases do not move (exactly); the left panel's right edge is recomputed
        // from a new origin and width, hence only "close".
        assert_eq!([grown.spine.x, grown.right.x], f.fold_x());
        assert_eq!(grown.spine.right(), grown.right.x);
        assert!(close(grown.left.right(), grown.spine.x));
        assert!(close(grown.left.x, -bleed) && close(grown.right.right(), spread.w + bleed));
        assert!(rect_close(&grown.bounds(), &f.trim_rect().inflate(bleed)), "{f:?}");
        for r in grown.to_array() {
            assert!(close(r.y, -bleed) && close(r.h, spread.h + 2.0 * bleed));
        }
    }
}

#[test]
fn inside_is_the_mirror_of_outside() {
    let mut rng = Rng(0x5eed_0002);
    for f in presets_and_random_formats(&mut rng, 2000) {
        let (o, i) = (f.outside(), f.inside());
        // Turn the sheet over: what was on the right is now on the left.
        assert!(rect_close(&i.left, &f.mirror_x(o.right)), "{f:?}");
        assert!(rect_close(&i.spine, &f.mirror_x(o.spine)), "{f:?}");
        assert!(rect_close(&i.right, &f.mirror_x(o.left)), "{f:?}");
        // Named accessors: the inside-front is printed on the back of the front cover.
        assert_eq!(f.back(), o.left);
        assert_eq!(f.spine(), o.spine);
        assert_eq!(f.front(), o.right);
        assert_eq!(f.inside_front(), i.left);
        assert_eq!(f.inside_spine(), i.spine);
        assert_eq!(f.inside_back(), i.right);
        assert!(rect_close(&f.inside_front(), &f.mirror_x(f.front())));
        assert!(rect_close(&f.inside_back(), &f.mirror_x(f.back())));
        // The creases are in the same place on both faces.
        let [a, b] = f.fold_x();
        assert!(close(f.spread().w - b, a) && close(f.spread().w - a, b), "{f:?}");
        // Mirroring twice is the identity.
        let r = Rect::new(rng.range(-10.0, 300.0), rng.range(-10.0, 300.0), 12.0, 7.0);
        assert!(rect_close(&f.mirror_x(f.mirror_x(r)), &r));
    }
}

#[test]
fn every_preset_with_every_built_in_profile() {
    let mut built = 0;
    let mut rejected = Vec::new();
    for (fname, format) in CardFormat::presets() {
        for profile in built_in_profiles() {
            let ctx = format!("{fname} / {}", profile.name);
            if check_combination(&format, &profile, &ctx) {
                built += 1;
            } else {
                rejected.push(ctx);
            }
        }
    }
    // Exactly the combinations that cannot work are rejected: digest and A5 on A4 and on
    // Letter (plain and "+all" variants).
    let mut expected: Vec<String> = Vec::new();
    for f in ["digest", "a5"] {
        for p in ["home_a4", "home_letter", "home_a4+all", "home_letter+all"] {
            expected.push(format!("{f} / {p}"));
        }
    }
    rejected.sort();
    expected.sort();
    assert_eq!(rejected, expected);
    assert_eq!(built, 4 * built_in_profiles().len() - 8);
}

#[test]
fn the_default_professional_card_has_all_its_marks() {
    // Guard against the invariants passing vacuously: on the reference card nothing is
    // shortened or dropped.
    let (f, mut p) = (CardFormat::poche(), PrintProfile::professional_eu());
    // By default: solid crop marks and dashed fold marks, nothing else.
    let layer = marks(&f, &p);
    let sub: Vec<usize> =
        drawn(layer_items(&layer, "marks")).iter().map(|d| subpaths(d.path).len()).collect();
    assert_eq!(sub, vec![1; 8 + 4], "eight crop marks and four fold marks, one item each");
    assert_eq!(check_marks_fit(&f, &p), Ok(()));
    // With everything switched on.
    p.marks.registration = true;
    p.marks.color_bar = true;
    p.marks.scale_ruler = true;
    let layer = marks(&f, &p);
    let items = layer_items(&layer, "marks");
    let sub: Vec<usize> = drawn(items).iter().map(|d| subpaths(d.path).len()).collect();
    // crop (8 strokes), fold (4), four targets (circle + 2 strokes each), 11 patches,
    // ruler (base line + 11 ticks: 100 mm, its label stacked above the ticks because a
    // 110 mm panel has no room for it beside the ruler).
    let mut want = vec![1; 8 + 4];
    want.extend([3; 4]);
    want.extend([1; 11]);
    want.push(12);
    assert_eq!(sub, want);
    assert_eq!(registration_target_centers(&f, &p).len(), 4);
    assert_eq!(scale_ruler(&f, &p).map(|r| r.length_mm), Some(100.0));
    // And the home card on A4 and on Letter: solid cut marks, dashed fold guides (no
    // solid part that could be taken for a cut), the 100 mm ruler.
    for sheet in [Sheet::A4, Sheet::Letter] {
        let h = PrintProfile::home(sheet);
        let layer = marks(&f, &h);
        let sub: Vec<usize> =
            drawn(layer_items(&layer, "marks")).iter().map(|d| subpaths(d.path).len()).collect();
        let mut want = vec![1; 8 + 4];
        want.push(12);
        assert_eq!(sub, want, "{sheet:?}");
        assert_eq!(check_marks_fit(&f, &h), Ok(()), "{sheet:?}");
        // Nothing is shortened: full 5 mm cut marks inside the printable area.
        let g = page_geometry(&f, &h).unwrap();
        assert!(g.mark_band_mm() >= h.marks.length, "{sheet:?}: {}", g.mark_band_mm());
    }
}

#[test]
fn random_formats_and_profiles_keep_every_invariant() {
    let mut rng = Rng(0x5eed_0003);
    let (mut built, mut rejected, mut with_sheet, mut drew_marks) = (0, 0, 0, 0);
    for n in 0..6000 {
        let format = rng.format();
        let profile = rng.profile();
        let ctx = format!("#{n} {format:?} {profile:?}");
        if check_combination(&format, &profile, &ctx) {
            built += 1;
            with_sheet += usize::from(profile.sheet.is_some());
            if let Item::Group { items, .. } = marks(&format, &profile) {
                drew_marks += usize::from(!items.is_empty());
            }
        } else {
            assert!(profile.sheet.is_some(), "{ctx}: only a sheet can make a page impossible");
            rejected += 1;
        }
    }
    // The loop must exercise every branch a meaningful number of times.
    assert!(built > 3000, "built {built}");
    assert!(rejected > 500, "rejected {rejected}");
    assert!(with_sheet > 500, "with sheet {with_sheet}");
    assert!(drew_marks > 2000, "drew marks {drew_marks}");
}

#[test]
fn safe_areas_stay_inside_their_panels() {
    let mut rng = Rng(0x5eed_0004);
    for _ in 0..3000 {
        let (f, p) = (rng.format(), rng.profile());
        let panels = f.outside();
        let safe = safe_panels(&f, &p);
        for (s, panel) in safe.to_array().iter().zip(panels.to_array()) {
            assert!(s.w >= 0.0 && s.h >= 0.0, "{s:?}");
            assert!(panel.contains_rect(s, EPS), "{s:?} outside {panel:?}");
            // Centred in its panel: equal margins on opposite sides.
            assert!(close(s.center().x, panel.center().x) && close(s.center().y, panel.center().y));
        }
        assert_eq!(safe.left, safe_rect(panels.left, &p));
        assert_eq!(safe.right, safe_rect(panels.right, &p));
        assert_eq!(safe.spine, spine_safe_rect(&f, &p));
        // When the margins fit, they are exactly what the profile asks for.
        if 2.0 * p.safe_margin < f.cover.w.min(f.cover.h) {
            assert!(close(safe.left.x, p.safe_margin) && close(safe.left.y, p.safe_margin));
            assert!(close(safe.right.right(), f.spread().w - p.safe_margin));
            assert!(close(safe.spine.y, p.safe_margin));
        }
        if 2.0 * p.spine_safe_margin < f.spine_width {
            assert!(close(safe.spine.x - f.fold_x()[0], p.spine_safe_margin));
            assert!(close(f.fold_x()[1] - safe.spine.right(), p.spine_safe_margin));
        }
    }
}

#[test]
fn place_image_preserves_aspect_and_covers_or_fits() {
    let mut rng = Rng(0x5eed_0005);
    for n in 0..20_000 {
        let px = (1 + rng.below(12_000) as u32, 1 + rng.below(12_000) as u32);
        let target = Rect::new(
            rng.range(-50.0, 400.0),
            rng.range(-50.0, 400.0),
            rng.range(0.5, 600.0),
            rng.range(0.5, 600.0),
        );
        let aspect = f64::from(px.0) / f64::from(px.1);
        for fit in [Fit::Cover, Fit::Contain] {
            let (ax, ay) = (rng.unit(), rng.unit());
            for r in [place_image(px, target, fit), place_image_aligned(px, target, fit, ax, ay)] {
                let ctx = format!("#{n} {px:?} into {target:?} ({fit:?}) -> {r:?}");
                assert!(r.w > 0.0 && r.h > 0.0, "{ctx}");
                // Never distorted.
                assert!((r.w / r.h - aspect).abs() <= 1e-9 * aspect, "{ctx}");
                // One dimension matches the target exactly, the other over- or undershoots.
                assert!(r.w == target.w || r.h == target.h, "{ctx}");
                match fit {
                    Fit::Cover => {
                        assert!(r.contains_rect(&target, EPS), "{ctx}");
                        assert!(r.w >= target.w && r.h >= target.h, "{ctx}");
                    }
                    Fit::Contain => {
                        assert!(target.contains_rect(&r, EPS), "{ctx}");
                        assert!(r.w <= target.w && r.h <= target.h, "{ctx}");
                    }
                }
                // Same resolution in both directions (that is what "no distortion" means).
                let ppi_x = f64::from(px.0) / (r.w / 25.4);
                let ppi_y = f64::from(px.1) / (r.h / 25.4);
                assert!((ppi_x - ppi_y).abs() <= 1e-6 * ppi_x, "{ctx}");
                assert!((effective_ppi(px, r) - ppi_x.min(ppi_y)).abs() <= 1e-9 * ppi_x, "{ctx}");
            }
            // The default placement is centred.
            let r = place_image(px, target, fit);
            assert!((r.center().x - target.center().x).abs() <= 1e-6, "#{n}");
            assert!((r.center().y - target.center().y).abs() <= 1e-6, "#{n}");
        }
        // Contain is never larger than Cover; they coincide only for equal aspect ratios.
        let cover = place_image(px, target, Fit::Cover);
        let contain = place_image(px, target, Fit::Contain);
        assert!(contain.w <= cover.w && contain.h <= cover.h);
    }
}

#[test]
fn cover_image_on_each_preset_fills_the_front_panel_with_bleed() {
    for (name, f) in CardFormat::presets() {
        for profile in [PrintProfile::professional_eu(), PrintProfile::home(Sheet::A3)] {
            let panel = f.front();
            let clip = f.outside().with_bleed(profile.bleed).right;
            // The front panel plus bleed on its three outer edges, flush with the crease.
            assert_eq!(clip.x, f.fold_x()[1], "{name}");
            assert!(close(clip.right(), f.spread().w + profile.bleed));
            assert!(close(clip.y, -profile.bleed));
            // A cover scan with exactly the pixels the profile asks for over the area it
            // has to cover: the panel plus bleed all round.
            let (pw, ph) = profile.target_px(panel.inflate(profile.bleed).size());
            let placed = place_image_bleed((pw, ph), panel, profile.bleed);
            assert!(placed.contains_rect(&clip, EPS), "{name}");
            // Centred on the panel: the fold hides what the trim removes.
            assert!(close(placed.center().x, panel.center().x), "{name}");
            assert!(close(placed.center().y, panel.center().y), "{name}");
            // The pixels asked for are enough, with no slack: the image keeps its aspect
            // ratio, so both axes have the resolution of the one that decides the scale,
            // and that one has at least the pixels `target_px` computed for it.
            let ppi = effective_ppi((pw, ph), placed);
            assert_eq!(profile.judge_ppi(ppi), Resolution::Good, "{name}: {ppi}");
            assert!(ppi >= profile.target_ppi * (1.0 - 1e-9), "{name}: {ppi}");
            // Filling the target exactly (no aspect mismatch): exactly on target.
            let exact = effective_ppi((pw, ph), panel.inflate(profile.bleed));
            assert_eq!(profile.judge_ppi(exact), Resolution::Good, "{name}: {exact}");
        }
    }
}

#[test]
fn hostile_numbers_never_panic_and_never_draw() {
    let nasty = [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        5e-324,
        -0.0,
        0.0,
        -1.0,
        1e-9,
        1e9,
        1e300,
    ];
    type SetFormat = fn(&mut CardFormat, f64);
    type SetProfile = fn(&mut PrintProfile, f64);
    let format_fields: [SetFormat; 3] =
        [|f, v| f.cover.w = v, |f, v| f.cover.h = v, |f, v| f.spine_width = v];
    let profile_fields: [SetProfile; 13] = [
        |p, v| p.bleed = v,
        |p, v| p.safe_margin = v,
        |p, v| p.spine_safe_margin = v,
        |p, v| p.slug = v,
        |p, v| p.printable_margin = v,
        |p, v| p.marks.offset = v,
        |p, v| p.marks.length = v,
        |p, v| p.marks.weight_pt = v,
        |p, v| p.target_ppi = v,
        |p, v| p.min_ppi = v,
        |p, v| p.max_total_ink_percent = v as f32,
        |p, v| p.sheet = Some(Sheet::Custom(Size::new(v, 300.0))),
        |p, v| p.sheet = Some(Sheet::Custom(Size::new(300.0, v))),
    ];
    let bases = [PrintProfile::professional_eu(), PrintProfile::home(Sheet::A4)];
    let mut errors = 0;
    let mut run = |format: &CardFormat, profile: &PrintProfile| {
        // Everything public gets called; nothing may panic.
        let page = new_page(format, profile);
        let layer = marks(format, profile);
        let _ = guides(format, profile);
        let _ = info_entries(format, profile);
        let _ = safe_panels(format, profile);
        let _ = spine_safe_rect(format, profile);
        let _ = format.outside().with_bleed(profile.bleed);
        let _ = profile.target_px(format.spread());
        let _ = profile.judge_ppi(profile.min_ppi);
        match page {
            Ok(_) => {
                // Still a valid combination (e.g. a zero margin): the full invariants hold.
                assert!(check_combination(format, profile, "hostile but valid"));
            }
            Err(e) => {
                errors += 1;
                assert!(!e.to_string().is_empty());
                assert!(layer_items(&layer, "marks").is_empty(), "{e}");
                assert!(crop_marks(format, profile).is_empty());
                assert!(fold_marks(format, profile).is_empty());
                assert!(registration_targets(format, profile).is_empty());
                assert!(color_bar(format, profile).is_empty());
                assert!(scale_ruler(format, profile).is_none());
            }
        }
    };
    for v in nasty {
        for set in format_fields {
            for base in &bases {
                let mut f = CardFormat::poche();
                set(&mut f, v);
                assert!(f.validate().is_err(), "{v} must not be a valid dimension");
                run(&f, base);
            }
        }
        for set in profile_fields {
            for base in &bases {
                let mut p = base.clone();
                set(&mut p, v);
                run(&CardFormat::poche(), &p);
            }
        }
        // Everything hostile at once.
        let f = CardFormat { cover: Size::new(v, v), spine_width: v };
        let mut p = PrintProfile::home(Sheet::Custom(Size::new(v, v)));
        for set in profile_fields {
            set(&mut p, v);
        }
        run(&f, &p);
        // Images.
        for fit in [Fit::Cover, Fit::Contain] {
            let r = place_image_aligned((3, 2), Rect::new(v, v, v, v), fit, v, v);
            let _ = effective_ppi((3, 2), r);
            let r = place_image_aligned((3, 2), Rect::new(0.0, 0.0, 30.0, 20.0), fit, v, v);
            assert!(r.x.is_finite() && r.y.is_finite() && r.w == 30.0 && r.h == 20.0);
        }
        let _ = safe_rect(Rect::new(v, v, v, v), &bases[0]);
    }
    assert!(errors > 200, "only {errors} hostile combinations were rejected");
}

#[test]
fn errors_are_reported_in_a_stable_order() {
    // Format problems come before profile problems, which come before fit problems.
    let bad_format = CardFormat { cover: Size::new(10.0, 178.0), spine_width: 10.0 };
    let mut bad_profile = PrintProfile::home(Sheet::Custom(Size::new(20.0, 20.0)));
    bad_profile.bleed = -1.0;
    assert!(matches!(
        new_page(&bad_format, &bad_profile),
        Err(LayoutError::CoverOutOfRange { dimension: "width", .. })
    ));
    assert!(matches!(
        new_page(&CardFormat::poche(), &bad_profile),
        Err(LayoutError::InvalidProfile { field: "bleed", .. })
    ));
    bad_profile.bleed = 2.0;
    assert!(matches!(
        new_page(&CardFormat::poche(), &bad_profile),
        Err(LayoutError::DoesNotFit { .. })
    ));
}
