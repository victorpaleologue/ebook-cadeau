//! Sparring tests: the mark generators under profiles and formats pushed to the limits of
//! what `validate` accepts (100 mm bleed, slug, offset and unprintable margin; 2 pt
//! strokes; 3 mm spines on 50 mm covers; sheets from barely large enough to 5080 mm).
//!
//! `tests/invariants.rs` samples the comfortable middle of the parameter space; this file
//! is an independent re-implementation of the "where is the ink" bookkeeping, run on the
//! corners.

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, LayoutError, MAX_MARGIN_MM, MAX_MARK_WEIGHT_PT, MAX_SHEET_MM, MarkStyle,
    PageGeometry, PrintProfile, Sheet, caption_areas, color_bar, crop_marks, fold_marks, guides,
    marks, new_page, page_geometry, registration_target_centers, registration_targets, scale_ruler,
};
use ebc_scene::{Item, PathSeg, Point, Rect, Size, pt_to_mm};

const EPS: f64 = 1e-9;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// A value in `lo..=hi`, biased towards the ends and towards "just inside the ends".
    fn edgy(&mut self, lo: f64, hi: f64) -> f64 {
        let span = hi - lo;
        match self.below(8) {
            0 => lo,
            1 => hi,
            2 => lo + span * 1e-9,
            3 => hi - span * 1e-9,
            4 => lo + span * self.unit() * 0.02,
            5 => hi - span * self.unit() * 0.02,
            _ => lo + span * self.unit(),
        }
    }

    fn format(&mut self) -> CardFormat {
        CardFormat::custom(
            Size::new(self.edgy(50.0, 300.0), self.edgy(50.0, 300.0)),
            self.edgy(3.0, 40.0),
        )
        .unwrap()
    }

    fn profile(&mut self, format: &CardFormat) -> PrintProfile {
        let mut p = PrintProfile::professional_eu();
        // Half the time small, realistic margins; half the time anything `validate` allows.
        let top = if self.below(2) == 0 { 15.0 } else { MAX_MARGIN_MM };
        p.bleed = self.edgy(0.0, top);
        p.slug = self.edgy(0.0, top);
        p.printable_margin = self.edgy(0.0, top);
        p.marks.offset = self.edgy(0.0, top);
        p.marks.length = self.edgy(0.0, top);
        p.marks.weight_pt = self.edgy(0.01, MAX_MARK_WEIGHT_PT);
        p.marks.style = if self.below(2) == 0 { MarkStyle::Press } else { MarkStyle::Guide };
        p.sheet = match self.below(4) {
            0 => None,
            1 => Some(Sheet::Custom(Size::new(
                self.edgy(1.0, MAX_SHEET_MM),
                self.edgy(1.0, MAX_SHEET_MM),
            ))),
            _ => {
                // A sheet that leaves between nothing and ~30 mm around the card, in a
                // random orientation, so that the fit decision and narrow bands are hit.
                let spread = format.spread();
                let need = 2.0 * (p.bleed + p.printable_margin);
                let w = spread.w + need + self.edgy(0.0, 30.0);
                let h = spread.h + need + self.edgy(0.0, 30.0);
                let (w, h) = if self.below(2) == 0 { (w, h) } else { (h, w) };
                Some(Sheet::Custom(Size::new(w.min(MAX_SHEET_MM), h.min(MAX_SHEET_MM))))
            }
        };
        p
    }
}

/// One inked rectangle and which generator drew it.
#[derive(Debug, Clone, Copy)]
struct Ink {
    kind: &'static str,
    rect: Rect,
}

/// The ink of a list of plain path items: per sub-path, the bounds of all its points,
/// widened by half the stroke width — except along a straight butt-capped line.
fn ink(kind: &'static str, items: &[Item]) -> Vec<Ink> {
    let mut out = Vec::new();
    for item in items {
        let Item::Path { path, fill, stroke } = item else {
            panic!("{kind}: not a path: {item:?}")
        };
        assert!(fill.is_some() != stroke.is_some(), "{kind}: filled xor stroked");
        let hw = stroke.as_ref().map_or(0.0, |s| {
            assert!(s.width_pt.is_finite() && s.width_pt > 0.0);
            pt_to_mm(s.width_pt) / 2.0
        });
        let mut subs: Vec<(Vec<Point>, bool)> = Vec::new();
        for seg in &path.segs {
            match *seg {
                PathSeg::MoveTo(p) => subs.push((vec![p], true)),
                PathSeg::LineTo(p) => subs.last_mut().unwrap().0.push(p),
                PathSeg::QuadTo(c, p) => {
                    let s = subs.last_mut().unwrap();
                    s.0.extend([c, p]);
                    s.1 = false;
                }
                PathSeg::CubicTo(a, b, p) => {
                    let s = subs.last_mut().unwrap();
                    s.0.extend([a, b, p]);
                    s.1 = false;
                }
                PathSeg::Close => subs.last_mut().unwrap().1 = false,
            }
        }
        for (points, open_polyline) in subs {
            let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
            for p in &points {
                assert!(p.x.is_finite() && p.y.is_finite(), "{kind}: non-finite point {p:?}");
                (x0, y0, x1, y1) = (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y));
            }
            let single_line = open_polyline && points.len() == 2;
            let (gx, gy) = match (single_line, x0 == x1, y0 == y1) {
                (true, true, false) => (hw, 0.0), // vertical line: no ink beyond its ends
                (true, false, true) => (0.0, hw), // horizontal line
                _ => (hw, hw),
            };
            out.push(Ink {
                kind,
                rect: Rect::new(x0 - gx, y0 - gy, x1 - x0 + 2.0 * gx, y1 - y0 + 2.0 * gy),
            });
        }
    }
    out
}

fn overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.right() - EPS
        && a.right() > b.x + EPS
        && a.y < b.bottom() - EPS
        && a.bottom() > b.y + EPS
}

fn all_ink(format: &CardFormat, profile: &PrintProfile) -> Vec<Ink> {
    let mut v = ink("crop", &crop_marks(format, profile));
    v.extend(ink("fold", &fold_marks(format, profile)));
    v.extend(ink("registration", &registration_targets(format, profile)));
    v.extend(ink("colour bar", &color_bar(format, profile)));
    v.extend(ink("ruler", &scale_ruler(format, profile).map(|r| r.items).unwrap_or_default()));
    v
}

/// Everything the mark generators promise, for one valid page.
fn check(format: &CardFormat, profile: &PrintProfile, g: &PageGeometry, ctx: &str) {
    // The page itself.
    let page = new_page(format, profile).unwrap();
    assert_eq!(page.trim_box(), g.trim, "{ctx}");
    assert_eq!(page.media_box(), g.media, "{ctx}");
    assert!(g.bleed.contains_rect(&g.trim, EPS), "{ctx}");
    assert!(g.media.contains_rect(&g.bleed, EPS), "{ctx}");
    assert!(g.media.contains_rect(&g.mark_area, EPS), "{ctx}");
    assert!(g.mark_area.contains_rect(&g.bleed, EPS), "{ctx}");
    if let Some(s) = &g.sheet {
        // The media box never claims more paper than there is, and what must print —
        // bleed and marks — is printable. (1e-6: the documented fit tolerance.)
        assert!(
            s.sheet.contains_rect(&g.media, 1e-6),
            "{ctx}: media {:?} sheet {:?}",
            g.media,
            s.sheet
        );
        assert!(s.printable.contains_rect(&g.bleed, 1e-6), "{ctx}: bleed not printable");
        assert!(s.printable.contains_rect(&g.mark_area, 1e-6), "{ctx}: mark area not printable");
    }

    let inks = all_ink(format, profile);
    for i in &inks {
        assert!(
            !overlap(&i.rect, &g.bleed),
            "{ctx}: {} ink {:?} enters the bleed box {:?}",
            i.kind,
            i.rect,
            g.bleed
        );
        assert!(
            g.mark_area.contains_rect(&i.rect, EPS),
            "{ctx}: {} ink {:?} leaves the mark area {:?}",
            i.kind,
            i.rect,
            g.mark_area
        );
        assert!(
            g.media.contains_rect(&i.rect, EPS),
            "{ctx}: {} ink {:?} leaves the media box",
            i.kind,
            i.rect
        );
        if let Some(s) = &g.sheet {
            assert!(
                s.printable.contains_rect(&i.rect, 1e-6),
                "{ctx}: {} ink {:?} is not printable",
                i.kind,
                i.rect
            );
        }
    }
    for (n, a) in inks.iter().enumerate() {
        for b in &inks[n + 1..] {
            assert!(
                a.kind == b.kind || !overlap(&a.rect, &b.rect),
                "{ctx}: {} {:?} collides with {} {:?}",
                a.kind,
                a.rect,
                b.kind,
                b.rect
            );
        }
    }
    if let Some(areas) = caption_areas(format, profile) {
        for area in areas {
            assert!(area.w > 0.0 && area.h > 0.0, "{ctx}");
            assert!(g.mark_area.contains_rect(&area, EPS), "{ctx}: caption {area:?}");
            assert!(!overlap(&area, &g.bleed), "{ctx}: caption {area:?} on the artwork");
            for i in &inks {
                assert!(
                    !overlap(&area, &i.rect),
                    "{ctx}: caption {area:?} hits {} {:?}",
                    i.kind,
                    i.rect
                );
            }
        }
    }
    // Targets are where `registration_target_centers` says, and nowhere near the card.
    for c in registration_target_centers(format, profile) {
        assert!(!g.bleed.contains(c), "{ctx}: target centre {c:?} inside the bleed box");
        assert!(g.mark_area.contains(c), "{ctx}: target centre {c:?}");
    }
    // The combined layer is made of the same paths.
    let Item::Group { items, .. } = marks(format, profile) else { panic!("{ctx}") };
    for i in ink("layer", &items) {
        assert!(!overlap(&i.rect, &g.bleed) && g.mark_area.contains_rect(&i.rect, EPS), "{ctx}");
    }
    // The proofing overlay stays on the card.
    let Item::Group { items, .. } = guides(format, profile) else { panic!("{ctx}") };
    for i in ink("guides", &items) {
        assert!(
            g.bleed.inflate(pt_to_mm(0.25)).contains_rect(&i.rect, EPS),
            "{ctx}: guide {:?}",
            i.rect
        );
    }
}

#[test]
fn marks_keep_their_promises_at_the_limits_of_validity() {
    let mut rng = Rng(0x5ba2_0001);
    let (mut built, mut refused, mut drew, mut on_sheet) = (0u32, 0u32, 0u32, 0u32);
    for n in 0..40_000 {
        let format = rng.format();
        let profile = rng.profile(&format);
        let ctx = format!("#{n} {format:?} {profile:?}");
        match page_geometry(&format, &profile) {
            Ok(g) => {
                built += 1;
                on_sheet += u32::from(g.sheet.is_some());
                drew += u32::from(!all_ink(&format, &profile).is_empty());
                check(&format, &profile, &g, &ctx);
            }
            Err(LayoutError::DoesNotFit { needed, .. }) => {
                refused += 1;
                // A refusal must be justified: the card really is too big both ways.
                let sheet = profile.sheet.expect("only a sheet can refuse a valid card");
                for s in [sheet.landscape(), sheet.portrait()] {
                    assert!(needed.w > s.w || needed.h > s.h, "{ctx}: {needed:?} fits {s:?}");
                }
                assert!(all_ink(&format, &profile).is_empty(), "{ctx}");
                assert!(caption_areas(&format, &profile).is_none(), "{ctx}");
            }
            Err(e) => panic!("{ctx}: valid input rejected with {e}"),
        }
    }
    // Not vacuous.
    assert!(built > 20_000, "built {built}");
    assert!(refused > 1_000, "refused {refused}");
    assert!(on_sheet > 8_000, "on a sheet {on_sheet}");
    assert!(drew > 10_000, "drew marks {drew}");
}

/// Directed corners that random sampling is unlikely to hit together.
#[test]
fn directed_extremes() {
    let smallest = CardFormat::custom(Size::new(50.0, 50.0), 3.0).unwrap();
    let largest = CardFormat::custom(Size::new(300.0, 300.0), 40.0).unwrap();
    let flat = CardFormat::custom(Size::new(300.0, 50.0), 3.0).unwrap();
    let tall = CardFormat::custom(Size::new(50.0, 300.0), 40.0).unwrap();
    let mut profiles = Vec::new();
    for style in [MarkStyle::Press, MarkStyle::Guide] {
        for weight in [0.01, 0.25, MAX_MARK_WEIGHT_PT] {
            for (bleed, slug, offset, length) in [
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 100.0, 0.0, 100.0),
                (100.0, 100.0, 0.0, 100.0),
                (100.0, 100.0, 100.0, 100.0),
                (0.0, 1.0, 0.0, 1.0),
                (0.0, 2.5, 0.0, 5.0),
                (3.0, 3.9, 3.0, 5.0),
                (3.0, 4.1, 3.0, 5.0),
                (3.0, 12.0, 14.0, 5.0),
                (3.0, 12.5, 3.0, 12.5),
            ] {
                let mut p = PrintProfile::professional_eu();
                p.marks.style = style;
                p.marks.weight_pt = weight;
                (p.bleed, p.slug, p.marks.offset, p.marks.length) = (bleed, slug, offset, length);
                p.marks.color_bar = true;
                p.marks.scale_ruler = true;
                profiles.push(p.clone());
                // The same margins provided by a sheet instead of a slug.
                for extra in [0.0, 1.0, 2.5, 7.0, 40.0] {
                    let mut q = p.clone();
                    q.printable_margin = 5.0;
                    q.sheet = Some(Sheet::Custom(Size::new(
                        640.0 + 2.0 * (bleed + 5.0 + extra),
                        300.0 + 2.0 * (bleed + 5.0 + extra),
                    )));
                    profiles.push(q);
                }
            }
        }
    }
    let mut checked = 0;
    for format in [smallest, largest, flat, tall, CardFormat::poche()] {
        for profile in &profiles {
            let ctx = format!("{format:?} {profile:?}");
            let g = page_geometry(&format, profile).unwrap_or_else(|e| panic!("{ctx}: {e}"));
            check(&format, profile, &g, &ctx);
            checked += 1;
        }
    }
    assert_eq!(checked, 5 * 2 * 3 * 10 * 6);
}

/// A sheet exactly as large as the card plus its margins must be accepted in whichever
/// orientation it is given, and the page must then be that sheet.
#[test]
fn a_sheet_that_fits_exactly_is_the_page() {
    for (name, format) in CardFormat::presets() {
        let mut p = PrintProfile::home(Sheet::A4);
        let spread = format.spread();
        // Equal margins on all four sides: bleed 2 + unprintable 5 + 9 mm for marks.
        let (w, h) = (spread.w + 32.0, spread.h + 32.0);
        for size in [Size::new(w, h), Size::new(h, w)] {
            p.sheet = Some(Sheet::Custom(size));
            let g = page_geometry(&format, &p).unwrap();
            let s = g.sheet.unwrap();
            assert!(s.media_is_sheet, "{name}");
            assert!(
                (g.media.w - w).abs() < EPS && (g.media.h - h).abs() < EPS,
                "{name}: {:?}",
                g.media
            );
            assert!((g.mark_reach_mm - 11.0).abs() < EPS, "{name}");
            // Landscape is preferred, and for these (wider than tall) spreads it is the
            // only orientation that fits.
            assert!(s.landscape, "{name}");
        }
    }
}
