//! Hostile and degenerate input: nothing may panic, hang or allocate without bound.

mod common;

use common::{bundled_fonts, clean_spec, cover};
use ebc_fontid::synth::{CoverSpec, Degradation, Rng, degrade, render_cover};
use ebc_fontid::{
    FontIndex, IndexEntry, TextRegion, bundled_index, find_text_regions, find_title_regions,
    identify, identify_detailed, rerank,
};
use ebc_scene::Pixmap;
use ebc_text::Font;

fn region(x: u32, y: u32, w: u32, h: u32) -> TextRegion {
    TextRegion { x, y, w, h, score: 1.0 }
}

fn candidates(index: &FontIndex) -> Vec<(IndexEntry, Font)> {
    index.entries().iter().cloned().zip(bundled_fonts().into_iter().map(|(_, f)| f)).collect()
}

#[test]
fn flat_images_have_no_text() {
    let index = bundled_index().expect("embedded index");
    let cands = candidates(&index);
    for rgba in [[0, 0, 0, 255], [255, 255, 255, 255], [128, 64, 200, 255], [0, 0, 0, 0]] {
        for (w, h) in [(1, 1), (1, 300), (300, 1), (7, 7), (64, 64), (333, 517)] {
            let img = Pixmap::filled(w, h, rgba);
            assert!(find_text_regions(&img).is_empty(), "{w}x{h} {rgba:?}");
            assert!(find_title_regions(&img, "Any title").is_empty());
            let whole = region(0, 0, w, h);
            assert!(identify(&img, &whole, Some("Any title"), &index).is_empty());
            assert!(identify(&img, &whole, None, &index).is_empty());
            assert!(rerank(&img, &whole, "Any title", &cands).is_empty());
        }
    }
}

#[test]
fn degenerate_regions_yield_nothing() {
    let index = bundled_index().expect("embedded index");
    let fonts = bundled_fonts();
    let c = cover(&fonts, 0, &clean_spec(&["Germinal"], 40.0, 1));
    let m = u32::MAX;
    for r in [
        region(0, 0, 0, 0),
        region(10, 10, 0, 50),
        region(10, 10, 50, 0),
        region(420, 0, 10, 10),
        region(0, 630, 10, 10),
        region(m, m, m, m),
        region(m, 0, 1, 1),
        region(0, 0, 1, 1),
        region(419, 629, m, m),
    ] {
        let id = identify_detailed(&c.image, &r, Some("Germinal"), &index);
        assert!(id.matches.is_empty() && !id.confident, "{r:?}");
        assert!(rerank(&c.image, &r, "Germinal", &candidates(&index)).is_empty());
    }
    // A region far larger than the image is clamped to it and still works.
    let id = identify_detailed(&c.image, &region(0, 0, m, m), Some("Germinal"), &index);
    assert_eq!(id.matches.len(), 8);
    // A region with a NaN score is still a region.
    let mut odd = c.title_region;
    odd.score = f32::NAN;
    assert_eq!(identify(&c.image, &odd, Some("Germinal"), &index).len(), 8);
}

#[test]
fn noise_specks_and_stripes_do_not_panic_or_explode() {
    let index = bundled_index().expect("embedded index");
    let cands = candidates(&index);
    let mut rng = Rng::new(99);
    let (w, h) = (240u32, 320u32);
    // Uniform noise, salt-and-pepper specks, one-pixel stripes, a checkerboard.
    let mut images = Vec::new();
    let mut noise = Pixmap::filled(w, h, [0, 0, 0, 255]);
    let mut specks = Pixmap::filled(w, h, [255, 255, 255, 255]);
    let mut stripes = Pixmap::filled(w, h, [255, 255, 255, 255]);
    let mut checks = Pixmap::filled(w, h, [255, 255, 255, 255]);
    for y in 0..h {
        for x in 0..w {
            let v = (rng.next_u64() & 0xff) as u8;
            noise.set(x, y, [v, v.wrapping_mul(7), v.wrapping_add(90), 255]);
            if rng.chance(0.08) {
                specks.set(x, y, [0, 0, 0, 255]);
            }
            if x % 2 == 0 {
                stripes.set(x, y, [0, 0, 0, 255]);
            }
            if (x / 9 + y / 9) % 2 == 0 {
                checks.set(x, y, [10, 10, 10, 255]);
            }
        }
    }
    images.extend([noise, specks, stripes, checks]);
    for img in &images {
        let regions = find_text_regions(img);
        assert!(regions.len() <= 12);
        let _ = find_title_regions(img, "Some title of several words");
        let whole = region(0, 0, w, h);
        for text in [None, Some("Some title"), Some(""), Some("ÿ\u{0301}\u{202e}\u{0}é"), Some("A")] {
            let id = identify_detailed(img, &whole, text, &index);
            assert!(id.matches.len() <= 8);
            assert!(id.matches.iter().all(|m| m.score.is_finite() && (0.0..=1.0).contains(&m.score)));
            assert!(!id.confident, "nothing in noise is a confident match");
        }
        let rr = rerank(img, &whole, "Some title", &cands);
        assert!(rr.iter().all(|m| m.score.is_finite()));
        for r in regions.iter().take(3) {
            let _ = identify(img, r, Some("Some title"), &index);
        }
    }
}

#[test]
fn hostile_titles_are_handled() {
    let index = bundled_index().expect("embedded index");
    let fonts = bundled_fonts();
    let cands = candidates(&index);
    let c = cover(&fonts, 3, &clean_spec(&["Germinal"], 40.0, 1));
    let long = "mot ".repeat(5000);
    let titles = [
        "",
        " ",
        "\n\t",
        "........",
        "— – -",
        "🙂🙂🙂🙂🙂🙂🙂🙂",
        "ГЕРМИНАЛЬ",
        "ßßßßßßßß",
        "a\u{0}b\u{7f}c\u{feff}d",
        "\u{202e}lanimreG",
        "e\u{0301}\u{0301}\u{0301}\u{0301}",
        long.as_str(),
        "ǅǅǅǅ ﬁﬂ İıİı",
    ];
    for title in titles {
        let id = identify_detailed(&c.image, &c.title_region, Some(title), &index);
        assert_eq!(id.matches.len(), 8, "{:?}", &title[..title.len().min(20)]);
        assert_eq!(id.matches[0].entry.family, "Spectral", "the fallback still finds the family");
        let _ = find_title_regions(&c.image, title);
        let rr = rerank(&c.image, &c.title_region, title, &cands);
        assert_eq!(rr.len(), 8);
        assert!(rr.iter().all(|m| m.score.is_finite() && (0.0..=1.0).contains(&m.score)));
    }
}

#[test]
fn a_very_large_image_is_processed_at_reduced_size() {
    // 5000 x 7000 px (140 MB of RGBA): a small cover placed in a huge white page.
    let fonts = bundled_fonts();
    let index = bundled_index().expect("embedded index");
    let small = cover(&fonts, 6, &clean_spec(&["GERMINAL"], 60.0, 2));
    let scale = 10u32;
    let (w, h) = (5000u32, 7000u32);
    let mut data = vec![255u8; w as usize * h as usize * 4];
    for y in 0..630 * scale {
        for x in 0..420 * scale {
            let p = small.image.get(x / scale, y / scale).expect("inside");
            let i = ((y + 300) as usize * w as usize + (x + 400) as usize) * 4;
            data[i..i + 4].copy_from_slice(&p);
        }
    }
    let big = Pixmap::from_rgba8(w, h, data).expect("sized");
    let regions = find_title_regions(&big, "Germinal");
    assert!(!regions.is_empty());
    let truth = region(
        small.title_region.x * scale + 400,
        small.title_region.y * scale + 300,
        small.title_region.w * scale,
        small.title_region.h * scale,
    );
    assert!(ebc_fontid::eval::is_detected(&regions[0], &truth), "{:?} vs {truth:?}", regions[0]);
    let id = identify_detailed(&big, &regions[0], Some("Germinal"), &index);
    assert_eq!(id.labelled_glyphs, 8);
    assert_eq!(id.matches[0].entry.family, "Bebas Neue");
}

#[test]
fn the_synthesiser_refuses_what_it_cannot_draw() {
    let fonts = bundled_fonts();
    let (f, a) = (&fonts[0].1, &fonts[1].1);
    let ok = clean_spec(&["Emma"], 30.0, 1);
    assert!(render_cover(&ok, f, a).is_some());
    for bad in [
        CoverSpec { width: 0, ..ok.clone() },
        CoverSpec { height: 10, ..ok.clone() },
        CoverSpec { width: 100_000, ..ok.clone() },
        CoverSpec { title_lines: vec![], ..ok.clone() },
        CoverSpec { title_lines: vec!["   ".into()], ..ok.clone() },
        CoverSpec { cap_height_px: f32::NAN, ..ok.clone() },
    ] {
        assert!(render_cover(&bad, f, a).is_none(), "{bad:?}");
    }
    // Wild degradation parameters are ignored or clamped, never fatal.
    let img = Pixmap::filled(40, 30, [120, 130, 140, 255]);
    for d in [
        Degradation { blur_sigma: f32::NAN, noise_sigma: f32::INFINITY, rotation_degrees: f32::NAN, jpeg_quality: Some(0) },
        Degradation { blur_sigma: -3.0, noise_sigma: -1.0, rotation_degrees: 720.0, jpeg_quality: Some(255) },
        Degradation { blur_sigma: 1e9, noise_sigma: 1e9, rotation_degrees: 1e9, jpeg_quality: None },
    ] {
        let out = degrade(&img, &d, 5);
        assert_eq!((out.width(), out.height()), (40, 30));
    }
}
