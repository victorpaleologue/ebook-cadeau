//! Rasterising shaped text to coverage masks.
#![cfg(feature = "raster")]

mod common;

use common::close;
use ebc_scene::{MM_PER_PT, Point};
use ebc_text::{
    Font, FontError, MAX_MASK_PIXELS, MAX_MASK_SIDE, Mask, RasterizedText, ShapeOptions, rasterize,
    shape, try_rasterize,
};

const FRENCH: &str = "L'Étranger — efficace œuvre";

fn opts() -> ShapeOptions {
    ShapeOptions::default()
}

fn synthetic() -> Font {
    Font::from_bytes(common::SyntheticFont::new().build(), 0).expect("synthetic font")
}

fn raster(font: &Font, text: &str, px_per_em: f32, opts: &ShapeOptions) -> RasterizedText {
    try_rasterize(font, text, px_per_em, opts).expect("rasterises")
}

fn column_has_ink(mask: &Mask, x: u32) -> bool {
    (0..mask.height).any(|y| mask.get(x, y) != 0)
}

fn row_has_ink(mask: &Mask, y: u32) -> bool {
    mask.row(y).iter().any(|&v| v != 0)
}

/// Sum of all coverage values, in pixels of ink.
fn ink_area(mask: &Mask) -> f64 {
    mask.data.iter().map(|&v| f64::from(v)).sum::<f64>() / 255.0
}

#[test]
fn mask_is_not_empty_tight_and_within_the_expected_pixel_bounds() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let m = font.metrics();
        let ppem = 48.0f32;
        let px = |units: i16| f64::from(units) / f64::from(m.units_per_em) * f64::from(ppem);
        let r = raster(&font, FRENCH, ppem, &opts());
        let mask = &r.mask;

        assert!(!mask.is_empty(), "{label}");
        assert_eq!(mask.data.len(), mask.width as usize * mask.height as usize, "{label}");
        assert!(mask.data.contains(&255), "{label}: some pixels are fully covered");

        // Cropped to the ink: the outermost rows and columns all have ink.
        assert!(row_has_ink(mask, 0) && row_has_ink(mask, mask.height - 1), "{label}");
        assert!(column_has_ink(mask, 0) && column_has_ink(mask, mask.width - 1), "{label}");

        // Horizontally: the ink spans the advance width, give or take a side bearing.
        let advance = f64::from(r.advance);
        let em = f64::from(ppem);
        assert!(close(f64::from(mask.width), advance, 0.25 * em), "{label}: {} px", mask.width);
        assert!(r.left.abs() <= (0.15 * em) as i32, "{label}: left {}", r.left);
        // Vertically: capitals above the baseline, a "g" below (except in Bebas Neue),
        // everything inside the line box (accents may exceed the ascender a little).
        let (top, bottom) = (f64::from(r.top), f64::from(r.top) + f64::from(mask.height));
        assert!(top <= -px(m.cap_height) + 1.0, "{label}: top {top}");
        assert!(top >= -px(m.ascender) - 0.1 * em - 1.0, "{label}: top {top}");
        assert!(bottom <= -px(m.descender) + 1.0, "{label}: bottom {bottom}");
        assert!(bottom >= 0.0, "{label}: bottom {bottom}");

        // The mask is the pixel-aligned bounding box of the vector outline.
        let size_pt = 10.0;
        let px_per_mm = em / (size_pt * MM_PER_PT);
        let ink = shape(&font, FRENCH, size_pt, &opts())
            .ink_bounds(&font, Point::new(0.0, 0.0))
            .expect("ink");
        let expected = [
            (ink.x * px_per_mm, f64::from(r.left)),
            (ink.y * px_per_mm, top),
            (ink.right() * px_per_mm, f64::from(r.left) + f64::from(mask.width)),
            (ink.bottom() * px_per_mm, bottom),
        ];
        for (vector, pixel) in expected {
            assert!(close(vector, pixel, 1.0), "{label}: outline {vector} vs mask {pixel}");
        }
    }
}

#[test]
fn a_rectangular_glyph_is_rendered_exactly() {
    // The synthetic "A" is the rectangle 100..500 x 0..700 of a 1000-unit em, with a
    // 600-unit advance. At 100 px per em that is 40 x 70 fully covered pixels.
    let r = raster(&synthetic(), "A", 100.0, &opts());
    assert_eq!((r.mask.width, r.mask.height), (40, 70));
    assert_eq!((r.left, r.top), (10, -70));
    assert!(close(f64::from(r.advance), 60.0, 1e-4));
    assert!(r.mask.data.iter().all(|&v| v == 255), "every pixel is fully inside the glyph");

    // At 25 px per em the rectangle is 10 x 17.5 px starting at x = 2.5: the edge
    // columns and the top row are half covered, and the total coverage is the area.
    let r = raster(&synthetic(), "A", 25.0, &opts());
    assert_eq!((r.mask.width, r.mask.height), (11, 18));
    assert_eq!((r.left, r.top), (2, -18));
    assert!(close(ink_area(&r.mask), 175.0, 1.0), "{}", ink_area(&r.mask));
    let half = |v: u8| (126..=129).contains(&v);
    assert!(half(r.mask.get(0, 9)) && half(r.mask.get(10, 9)), "half-covered side columns");
    assert!(half(r.mask.get(5, 0)), "half-covered top row");
    assert_eq!(r.mask.get(5, 9), 255);
    assert_eq!(r.mask.get(5, 17), 255, "the bottom row sits on the baseline");
}

#[test]
fn coverage_equals_the_area_of_the_outline() {
    // For real glyphs: the sum of the coverage is the area enclosed by the outline.
    // Bebas Neue's "I" is a plain rectangle, so its area is width x height of the ink.
    let font = common::bebas();
    let ppem = 200.0f32;
    let r = raster(&font, "I", ppem, &opts());
    let size_pt = 10.0;
    let px_per_mm = f64::from(ppem) / (size_pt * MM_PER_PT);
    let ink =
        shape(&font, "I", size_pt, &opts()).ink_bounds(&font, Point::new(0.0, 0.0)).expect("ink");
    let area = ink.w * ink.h * px_per_mm * px_per_mm;
    assert!(close(ink_area(&r.mask), area, 0.01 * area), "{} vs {area}", ink_area(&r.mask));
    assert!(close(f64::from(r.mask.height), 0.7 * f64::from(ppem), 1.0), "cap height is 0.7 em");
    // The centre of the stem is solid ink.
    assert_eq!(r.mask.get(r.mask.width / 2, r.mask.height / 2), 255);
}

#[test]
fn glyphs_are_drawn_where_shaping_puts_them() {
    let font = synthetic();
    // "AA": rectangles at 10..50 and 70..110 px, with a 20 px gap between them.
    let plain = raster(&font, "AA", 100.0, &opts());
    assert_eq!((plain.mask.width, plain.left), (100, 10));
    for x in 0..100 {
        assert_eq!(column_has_ink(&plain.mask, x), !(40..60).contains(&x), "column {x}");
    }
    assert!(close(f64::from(plain.advance), 120.0, 1e-4));

    // Letter spacing of half an em moves the second letter 50 px to the right.
    let spaced_opts = ShapeOptions { letter_spacing_em: 0.5, ..opts() };
    let spaced = raster(&font, "AA", 100.0, &spaced_opts);
    assert_eq!(spaced.mask.width, 150);
    for x in 0..150 {
        assert_eq!(column_has_ink(&spaced.mask, x), !(40..110).contains(&x), "column {x}");
    }
    assert!(close(f64::from(spaced.advance), 170.0, 1e-4));

    // The space glyph draws nothing but still moves the pen (advance 300 units).
    let with_space = raster(&font, "A A", 100.0, &opts());
    assert_eq!(with_space.mask.width, 130);

    // Kerning in a real font: "AV" is tighter than with kerning switched off.
    let serif = common::spectral();
    let no_kerning = ShapeOptions { features: vec![("kern".into(), 0)], ..opts() };
    let kerned = raster(&serif, "AV", 200.0, &opts());
    let loose = raster(&serif, "AV", 200.0, &no_kerning);
    let shift = f64::from(loose.advance - kerned.advance);
    assert!(shift > 5.0, "kerning of {shift} px at 200 px per em");
    assert!(
        close(f64::from(loose.mask.width) - f64::from(kerned.mask.width), shift, 1.5),
        "the mask shrinks by the kerning"
    );
}

#[test]
fn overlapping_glyphs_are_filled_solid() {
    // Negative letter spacing makes the two rectangles overlap by 10 px (40..50).
    // With the non-zero rule the overlap is ink, not a hole and not double coverage.
    let tight = ShapeOptions { letter_spacing_em: -0.3, ..opts() };
    let r = raster(&synthetic(), "AA", 100.0, &tight);
    assert_eq!((r.mask.width, r.mask.height, r.left), (70, 70, 10));
    assert!(r.mask.data.iter().all(|&v| v == 255));
}

#[test]
fn right_to_left_runs_are_drawn_mirrored_in_order() {
    let font = common::courier();
    let rtl = ShapeOptions { rtl: true, ..opts() };
    let reversed = rasterize(&font, "abc", 64.0, &rtl);
    assert_eq!(reversed, rasterize(&font, "cba", 64.0, &opts()));
    assert_ne!(reversed, rasterize(&font, "abc", 64.0, &opts()));
}

#[test]
fn nothing_to_draw_gives_an_empty_mask() {
    let font = common::atkinson();
    for text in ["", " ", "   \t\n", "\u{200b}\u{feff}"] {
        assert_eq!(rasterize(&font, text, 32.0, &opts()), Mask::default(), "{text:?}");
        let r = raster(&font, text, 32.0, &opts());
        assert!(r.mask.is_empty());
        assert_eq!((r.left, r.top), (0, 0));
    }
    // Spaces still advance.
    assert!(raster(&font, "   ", 32.0, &opts()).advance > 10.0);
    assert_eq!(raster(&font, "", 32.0, &opts()).advance, 0.0);
    // A character without an outline in the font: the synthetic .notdef is empty.
    assert!(rasterize(&synthetic(), "B", 32.0, &opts()).is_empty());
    // A real font draws its .notdef box for a missing character.
    assert!(!rasterize(&font, "你", 32.0, &opts()).is_empty());
}

#[test]
fn invalid_sizes_are_errors_not_panics() {
    let font = common::atkinson();
    for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(
            matches!(
                try_rasterize(&font, "abc", bad, &opts()),
                Err(FontError::InvalidRasterSize(_))
            ),
            "{bad}"
        );
        assert_eq!(rasterize(&font, "abc", bad, &opts()), Mask::default(), "{bad}");
    }
    // Tiny but valid sizes work: the whole word covers a pixel or two.
    let tiny = raster(&font, "abc", 0.5, &opts());
    assert!(tiny.mask.width <= 2 && tiny.mask.height <= 1 || tiny.mask.is_empty());
    let _ = rasterize(&font, "abc", f32::MIN_POSITIVE, &opts());
}

#[test]
fn oversized_requests_are_refused() {
    let font = common::atkinson();
    // One letter, 20 000 px per em: about 13 000 x 13 000 px, far above the budget.
    match try_rasterize(&font, "M", 20_000.0, &opts()) {
        Err(FontError::RasterTooLarge { width, height }) => {
            assert!(width * height > MAX_MASK_PIXELS, "{width} x {height}");
        }
        other => panic!("expected RasterTooLarge, got {other:?}"),
    }
    // A long line at a modest size: too wide.
    let long = "M".repeat(2_000);
    match try_rasterize(&font, &long, 64.0, &opts()) {
        Err(FontError::RasterTooLarge { width, .. }) => assert!(width > u64::from(MAX_MASK_SIDE)),
        other => panic!("expected RasterTooLarge, got {other:?}"),
    }
    assert_eq!(rasterize(&font, &long, 64.0, &opts()), Mask::default());
    for huge in [1e9f32, 1e30, f32::MAX] {
        assert!(try_rasterize(&font, "M", huge, &opts()).is_err(), "{huge}");
    }
    // Absurd letter spacing pushes the second letter far away.
    let apart = ShapeOptions { letter_spacing_em: 1e6, ..opts() };
    assert!(matches!(
        try_rasterize(&font, "MM", 64.0, &apart),
        Err(FontError::RasterTooLarge { .. })
    ));
    // The largest allowed masks do work: 4000 px tall capitals.
    let big = raster(&font, "I", 6_000.0, &opts());
    assert!(
        big.mask.height > 3_900
            && u64::from(big.mask.width) * u64::from(big.mask.height) <= MAX_MASK_PIXELS
    );
}

#[test]
fn rasterising_is_deterministic_and_scales() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let first = raster(&font, FRENCH, 40.0, &opts());
        assert_eq!(raster(&common::font(bytes), FRENCH, 40.0, &opts()), first, "{label}");

        let double = raster(&font, FRENCH, 80.0, &opts());
        assert!(close(f64::from(double.mask.width), 2.0 * f64::from(first.mask.width), 2.0));
        assert!(close(f64::from(double.mask.height), 2.0 * f64::from(first.mask.height), 2.0));
        assert!(close(f64::from(double.advance), 2.0 * f64::from(first.advance), 1e-3));
        assert!(close(
            ink_area(&double.mask),
            4.0 * ink_area(&first.mask),
            0.02 * ink_area(&double.mask)
        ));
    }
}

#[test]
fn variation_instances_are_rasterised() {
    let font = Font::from_bytes(
        common::SyntheticFont { variable: true, ..common::SyntheticFont::new() }.build(),
        0,
    )
    .expect("font");
    let regular = raster(&font, "A", 100.0, &opts());
    let black = raster(&font.with_variations(&[("wght", 900.0)]), "A", 100.0, &opts());
    let thin = raster(&font.with_variations(&[("wght", 100.0)]), "A", 100.0, &opts());
    assert_eq!((regular.mask.width, black.mask.width, thin.mask.width), (40, 50, 35));
    assert_eq!((regular.mask.height, black.mask.height, thin.mask.height), (70, 70, 70));
    assert!(close(f64::from(black.advance), 70.0, 1e-3));
}
