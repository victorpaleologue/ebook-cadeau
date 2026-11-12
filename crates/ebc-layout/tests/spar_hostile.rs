//! Sparring tests: arbitrary bit patterns in every numeric field at once (NaN payloads,
//! subnormals, signed zeros, infinities, huge magnitudes), and the `EBC.*` Info values
//! read back the way `ebc-preflight` reads them.

#![allow(clippy::unwrap_used)]

use ebc_layout::{
    CardFormat, Fit, MarkStyle, PrintProfile, Sheet, caption_areas, color_bar, crop_marks,
    effective_ppi, fold_marks, guides, info_entries, marks, new_page, page_geometry, place_image,
    place_image_aligned, registration_target_centers, registration_targets, safe_panels, safe_rect,
    scale_ruler, spine_safe_rect,
};
use ebc_scene::{Item, PathSeg, Rect, Size};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Any `f64` at all, or — half the time — a plausible value, so that some fields are
    /// sane while their neighbours are not.
    fn wild(&mut self, plausible: f64) -> f64 {
        const SPECIAL: [f64; 10] = [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            5e-324,
            f64::MAX,
            f64::MIN,
            f64::EPSILON,
        ];
        match self.next() % 8 {
            0..=3 => plausible,
            4 => SPECIAL[(self.next() % 10) as usize],
            5 => -plausible,
            6 => plausible * 1e-12,
            _ => f64::from_bits(self.next()),
        }
    }
}

fn finite(r: &Rect) -> bool {
    r.x.is_finite() && r.y.is_finite() && r.w.is_finite() && r.h.is_finite()
}

fn all_points_finite(item: &Item) -> bool {
    match item {
        Item::Path { path, .. } => path.segs.iter().all(|s| match *s {
            PathSeg::MoveTo(p) | PathSeg::LineTo(p) => p.x.is_finite() && p.y.is_finite(),
            PathSeg::QuadTo(a, p) => [a, p].iter().all(|q| q.x.is_finite() && q.y.is_finite()),
            PathSeg::CubicTo(a, b, p) => {
                [a, b, p].iter().all(|q| q.x.is_finite() && q.y.is_finite())
            }
            PathSeg::Close => true,
        }),
        Item::Group { items, .. } => items.iter().all(all_points_finite),
        Item::Image { .. } | Item::Glyphs { .. } => false,
    }
}

#[test]
fn random_bit_patterns_never_panic_and_never_yield_a_broken_page() {
    let mut rng = Rng(0x0bad_f00d);
    let (mut ok, mut refused) = (0u32, 0u32);
    for n in 0..60_000 {
        let format = CardFormat {
            cover: Size::new(rng.wild(110.0), rng.wild(178.0)),
            spine_width: rng.wild(10.0),
        };
        let mut profile = if rng.next() & 1 == 0 {
            PrintProfile::professional_eu()
        } else {
            PrintProfile::home(Sheet::Custom(Size::new(rng.wild(297.0), rng.wild(210.0))))
        };
        profile.bleed = rng.wild(3.0);
        profile.slug = rng.wild(10.0);
        profile.safe_margin = rng.wild(4.0);
        profile.spine_safe_margin = rng.wild(1.5);
        profile.printable_margin = rng.wild(5.0);
        profile.marks.offset = rng.wild(3.0);
        profile.marks.length = rng.wild(5.0);
        profile.marks.weight_pt = rng.wild(0.25);
        profile.target_ppi = rng.wild(300.0);
        profile.min_ppi = rng.wild(150.0);
        profile.max_total_ink_percent = rng.wild(300.0) as f32;
        profile.marks.style = if rng.next() & 1 == 0 { MarkStyle::Press } else { MarkStyle::Guide };
        profile.marks.color_bar = true;
        profile.marks.scale_ruler = true;
        profile.marks.registration = true;
        let ctx = format!("#{n} {format:?} {profile:?}");

        // Everything public is called; nothing may panic, whatever comes back.
        let layer = marks(&format, &profile);
        let overlay = guides(&format, &profile);
        let _ = info_entries(&format, &profile);
        let _ = safe_panels(&format, &profile);
        let _ = spine_safe_rect(&format, &profile);
        let _ = format.outside().with_bleed(profile.bleed).bounds();
        let _ = format.inside().center_x();
        let _ = format.mirror_x(format.front());
        let _ = format.fold_lines();
        let _ = profile.target_px(format.spread());
        let _ = profile.min_px(format.spread());
        let _ = profile.judge_ppi(rng.wild(300.0));
        let hostile = Rect::new(rng.wild(0.0), rng.wild(0.0), rng.wild(110.0), rng.wild(178.0));
        let safe = safe_rect(hostile, &profile);
        // NaN in, NaN out is tolerated for a hostile panel; a negative size never is.
        let non_negative = |v: f64| v.is_nan() || v >= 0.0;
        assert!(non_negative(safe.w) && non_negative(safe.h), "{ctx}: safe area {safe:?}");
        let px = (rng.next() as u32, rng.next() as u32);
        for fit in [Fit::Cover, Fit::Contain] {
            let placed = place_image_aligned(px, hostile, fit, rng.wild(0.5), rng.wild(0.5));
            let ppi = effective_ppi(px, placed);
            assert!(non_negative(ppi), "{ctx}: negative resolution {ppi}");
        }

        match page_geometry(&format, &profile) {
            Ok(g) => {
                ok += 1;
                // Whatever survives validation is a real page with real marks.
                let page = new_page(&format, &profile).unwrap();
                assert!(page.bleed >= 0.0 && page.slug >= 0.0, "{ctx}");
                assert!(page.bleed.is_finite() && page.slug.is_finite(), "{ctx}");
                for r in [g.trim, g.bleed, g.media, g.mark_area] {
                    assert!(finite(&r) && r.w > 0.0 && r.h > 0.0, "{ctx}: {r:?}");
                }
                assert!(g.media.contains_rect(&g.bleed, 1e-9), "{ctx}");
                assert!(g.bleed.contains_rect(&g.trim, 1e-9), "{ctx}");
                assert!(all_points_finite(&layer) && all_points_finite(&overlay), "{ctx}");
            }
            Err(e) => {
                refused += 1;
                assert!(!e.to_string().is_empty());
                assert!(new_page(&format, &profile).is_err(), "{ctx}");
                assert_eq!(layer, Item::layer("marks", vec![]), "{ctx}");
                assert_eq!(overlay, Item::layer("guides", vec![]), "{ctx}");
                assert!(crop_marks(&format, &profile).is_empty(), "{ctx}");
                assert!(fold_marks(&format, &profile).is_empty(), "{ctx}");
                assert!(registration_targets(&format, &profile).is_empty(), "{ctx}");
                assert!(registration_target_centers(&format, &profile).is_empty(), "{ctx}");
                assert!(color_bar(&format, &profile).is_empty(), "{ctx}");
                assert!(scale_ruler(&format, &profile).is_none(), "{ctx}");
                assert!(caption_areas(&format, &profile).is_none(), "{ctx}");
            }
        }
    }
    // Both outcomes must occur, or the generator is not doing its job.
    assert!(ok > 20, "only {ok} valid pages");
    assert!(refused > 50_000, "only {refused} refusals");
}

/// For sane targets, any pixel size gives a finite, positive placement.
#[test]
fn any_pixel_size_gives_a_finite_placement_on_a_sane_target() {
    let mut rng = Rng(0x0001_cafe);
    for _ in 0..50_000 {
        let px = (
            1 + (rng.next() as u32 >> (rng.next() % 32)),
            1 + (rng.next() as u32 >> (rng.next() % 32)),
        );
        let unit = |r: &mut Rng| (r.next() >> 11) as f64 / (1u64 << 53) as f64;
        let target = Rect::new(
            unit(&mut rng) * 700.0 - 100.0,
            unit(&mut rng) * 700.0 - 100.0,
            0.01 + unit(&mut rng) * 650.0,
            0.01 + unit(&mut rng) * 650.0,
        );
        for fit in [Fit::Cover, Fit::Contain] {
            let r = place_image(px, target, fit);
            assert!(finite(&r) && r.w > 0.0 && r.h > 0.0, "{px:?} {fit:?} into {target:?}: {r:?}");
            match fit {
                Fit::Cover => {
                    assert!(r.contains_rect(&target, 1e-6), "{px:?} into {target:?}: {r:?}")
                }
                Fit::Contain => {
                    assert!(target.contains_rect(&r, 1e-6), "{px:?} into {target:?}: {r:?}")
                }
            }
            assert!(effective_ppi(px, r).is_finite());
        }
    }
}

/// `ebc-preflight` parses the `EBC.*` values as decimal millimetres in `0..=100000` and
/// compares them with the trim box with a 0.1 mm tolerance (`check_ebc_geometry`). The
/// same rules, applied to what `info_entries` writes for every valid format.
#[test]
fn info_entries_read_back_consistently() {
    let parse = |text: &str| -> f64 {
        let v: f64 = text.trim().parse().unwrap_or_else(|_| panic!("not a number: {text:?}"));
        assert!(v.is_finite() && (0.0..=100_000.0).contains(&v), "{text:?}");
        // Plain decimal notation only: no exponent, no sign.
        assert!(text.bytes().all(|b| b.is_ascii_digit() || b == b'.'), "{text:?}");
        v
    };
    let mut rng = Rng(0x0e_bc);
    let unit = |r: &mut Rng| (r.next() >> 11) as f64 / (1u64 << 53) as f64;
    for n in 0..20_000 {
        let format = if n < 4 {
            CardFormat::presets()[n].1
        } else {
            CardFormat::custom(
                Size::new(50.0 + 250.0 * unit(&mut rng), 50.0 + 250.0 * unit(&mut rng)),
                3.0 + 37.0 * unit(&mut rng),
            )
            .unwrap()
        };
        let profile = PrintProfile::professional_eu();
        let entries = info_entries(&format, &profile);
        let get = |key: &str| {
            let hits: Vec<&str> =
                entries.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect();
            assert_eq!(hits.len(), 1, "{key} must appear exactly once");
            hits[0]
        };
        for (k, _) in &entries {
            assert!(k.starts_with("EBC.") && k.bytes().all(|b| b.is_ascii_graphic()), "{k:?}");
        }
        let (trim_w, trim_h) = (format.spread().w, format.spread().h);
        let close = |a: f64, b: f64| (a - b).abs() <= 0.1;
        let cover_w = parse(get("EBC.CoverWidthMm"));
        let cover_h = parse(get("EBC.CoverHeightMm"));
        let spine = parse(get("EBC.SpineWidthMm"));
        let folds: Vec<f64> = get("EBC.FoldXMm").split(',').map(parse).collect();
        assert_eq!(folds.len(), 2, "{format:?}");
        assert!(close(cover_h, trim_h), "{format:?}");
        assert!(close(2.0 * cover_w + spine, trim_w), "{format:?}");
        assert!(folds[0] < folds[1] && folds[1] < trim_w, "{format:?}");
        assert!(close(folds[0] + folds[1], trim_w), "{format:?}: folds not centred");
        assert!(close(folds[1] - folds[0], spine), "{format:?}");
        assert!(close(folds[0], cover_w), "{format:?}");
        // And tighter than preflight asks: the file must not lie by more than its own
        // three decimals.
        assert!((folds[0] - format.fold_x()[0]).abs() <= 0.000_500_1, "{format:?}");
        assert!((folds[1] - format.fold_x()[1]).abs() <= 0.000_500_1, "{format:?}");
        assert!((parse(get("EBC.BleedMm")) - profile.bleed).abs() <= 0.000_500_1);
        assert_eq!(get("EBC.Profile"), "professional_eu");
    }
}
