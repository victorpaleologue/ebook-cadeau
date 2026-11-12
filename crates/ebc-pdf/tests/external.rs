//! Checks with external tools — poppler (`pdfinfo`, `pdffonts`, `pdftotext`, `pdftoppm`),
//! `qpdf` and Ghostscript — on small documents built for one purpose each. Every check is
//! skipped with a note when its tool is not installed.

mod common;

use std::sync::Arc;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{
    Color, Document, FillRule, ImageData, Item, MM_PER_PT, Page, Path, PdfStandard, PixelFormat,
    Point, Rect, Size, Stroke, StrokeCap, Transform,
};
use image::RgbImage;

const ALL: [PdfStandard; 4] =
    [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4];

fn name_of(standard: PdfStandard) -> &'static str {
    match standard {
        PdfStandard::Plain => "plain",
        PdfStandard::X1a2001 => "x1a",
        PdfStandard::X3_2002 => "x3",
        PdfStandard::X4 => "x4",
    }
}

fn raw(format: PixelFormat, width: u32, height: u32, pixels: Vec<u8>) -> ImageData {
    ImageData::Raw { pixels: Arc::from(pixels), width_px: width, height_px: height, format }
}

/// Writes `doc` into the scratch directory of `test` and renders its first page at 72 dpi
/// (1 px = 1 pt). Returns `None` when poppler is missing.
fn rasterize(test: &str, name: &str, doc: &Document) -> Option<RgbImage> {
    let pdf = write_pdf(doc, &WriteOptions::default()).expect("writable");
    let path = save(test, &format!("{name}.pdf"), &pdf);
    render(test, &path, 1, 72)
}

fn is_red(p: [u8; 3]) -> bool {
    p[0] > 180 && p[1] < 100 && p[2] < 100
}
fn is_green(p: [u8; 3]) -> bool {
    p[1] > 120 && p[0] < 110 && p[2] < 140 && p[1] > p[0] + 40
}
fn is_blue(p: [u8; 3]) -> bool {
    // Pure RGB blue, or the darker blue that 100 % cyan + magenta ink gives.
    p[2] > 120 && p[0] < 90 && p[1] < 90 && p[2] > p[0] + 60
}
fn is_white(p: [u8; 3]) -> bool {
    p.iter().all(|c| *c > 240)
}
fn is_black(p: [u8; 3]) -> bool {
    p.iter().all(|c| *c < 60)
}

/// The items of the geometry test: a red rectangle, a blue image clipped to its centre
/// and a green bar turned a quarter turn about (30, 50).
fn geometry_items(doc: &mut Document) -> Vec<Item> {
    let blue = if doc.standard == PdfStandard::X1a2001 {
        doc.add_image(jpeg_cmyk(32, 32, [255, 255, 0, 0], false), None)
    } else {
        doc.add_image(jpeg_rgb(32, 32, |_, _| [0, 0, 255]), None)
    };
    vec![
        Item::filled_rect(Rect::new(10.0, 10.0, 30.0, 20.0), Color::cmyk(0.0, 1.0, 1.0, 0.0)),
        Item::clipped(
            Rect::new(60.0, 20.0, 20.0, 20.0),
            vec![Item::Image { image: blue, rect: Rect::new(50.0, 10.0, 40.0, 40.0) }],
        ),
        Item::group(
            Transform::rotate_about(90.0, Point::new(30.0, 50.0)),
            vec![Item::filled_rect(
                Rect::new(30.0, 48.0, 20.0, 4.0),
                Color::cmyk(1.0, 0.0, 1.0, 0.0),
            )],
        ),
    ]
}

#[test]
fn rectangle_clipped_image_and_rotated_content_land_where_expected() {
    for standard in ALL {
        // Once without and once with bleed and slug: the picture only shifts.
        for (bleed, slug) in [(0.0, 0.0), (3.0, 7.0)] {
            let mut doc = base_doc(standard);
            let mut page = Page::new(Size::new(100.0, 80.0), bleed, slug);
            page.items = geometry_items(&mut doc);
            doc.pages.push(page);
            let name = format!("geometry-{}-{bleed}", name_of(standard));
            let Some(img) = rasterize("external-geometry", &name, &doc) else { return };
            let margin = bleed + slug;
            let at = |x: f64, y: f64| pixel_at(&img, 72, margin, x, y);
            let size = |mm: f64| ((mm + 2.0 * margin) * 72.0 / 25.4).round() as u32;
            assert!(
                img.width().abs_diff(size(100.0)) <= 1 && img.height().abs_diff(size(80.0)) <= 1
            );

            // Red rectangle: (10, 10) .. (40, 30), measured from the top-left of the trim.
            for (x, y) in [(25.0, 20.0), (11.0, 11.0), (39.0, 11.0), (11.0, 29.0), (39.0, 29.0)] {
                assert!(is_red(at(x, y)), "{name}: red at ({x}, {y}): {:?}", at(x, y));
            }
            for (x, y) in [(9.0, 20.0), (41.0, 20.0), (25.0, 9.0), (25.0, 31.0), (25.0, 60.0)] {
                assert!(is_white(at(x, y)), "{name}: white at ({x}, {y}): {:?}", at(x, y));
            }

            // Image drawn at (50, 10) .. (90, 50) but clipped to (60, 20) .. (80, 40).
            for (x, y) in [(70.0, 30.0), (61.0, 21.0), (79.0, 39.0), (61.0, 39.0), (79.0, 21.0)] {
                assert!(is_blue(at(x, y)), "{name}: blue at ({x}, {y}): {:?}", at(x, y));
            }
            for (x, y) in
                [(55.0, 15.0), (85.0, 45.0), (70.0, 15.0), (70.0, 45.0), (59.0, 30.0), (81.0, 30.0)]
            {
                assert!(is_white(at(x, y)), "{name}: clipped away at ({x}, {y}): {:?}", at(x, y));
            }

            // The bar (30, 48) .. (50, 52), turned clockwise about (30, 50), now runs
            // downwards: (28, 50) .. (32, 70).
            for (x, y) in [(30.0, 60.0), (29.0, 51.0), (31.0, 69.0)] {
                assert!(is_green(at(x, y)), "{name}: green at ({x}, {y}): {:?}", at(x, y));
            }
            for (x, y) in
                [(40.0, 50.0), (48.0, 50.0), (30.0, 46.0), (26.5, 60.0), (33.5, 60.0), (30.0, 72.0)]
            {
                assert!(is_white(at(x, y)), "{name}: white at ({x}, {y}): {:?}", at(x, y));
            }
        }
    }
}

/// A page whose slug differs between the horizontal and the vertical direction (a trim
/// centred on a sheet of paper): poppler must report the boxes of that sheet and the
/// content must sit where the trim box is, not where a uniform slug would put it.
#[test]
fn a_vertical_slug_moves_the_boxes_and_the_content_together() {
    for standard in [PdfStandard::Plain, PdfStandard::X4] {
        // (bleed, slug, vertical slug): taller margins, wider margins, vertical only.
        for (bleed, slug, slug_v) in [(3.0, 7.0, 20.0), (2.0, 25.0, 4.0), (0.0, 0.0, 12.0)] {
            let mut doc = base_doc(standard);
            let mut page = Page::new(Size::new(100.0, 80.0), bleed, slug);
            page.slug_v = Some(slug_v);
            page.items = geometry_items(&mut doc);
            doc.pages.push(page);
            let name = format!("vslug-{}-{slug}-{slug_v}", name_of(standard));
            let (mx, my) = (bleed + slug, bleed + slug_v);
            let (w, h) = (100.0 + 2.0 * mx, 80.0 + 2.0 * my);

            let pdf = write_pdf(&doc, &WriteOptions::default()).expect("writable");
            let path = save("external-vslug", &format!("{name}.pdf"), &pdf);
            if let Some((ok, stdout, stderr)) = run("pdfinfo", &["-box", &path]) {
                assert!(ok, "pdfinfo failed: {stderr}");
                let boxes = |key: &str| -> Vec<f64> {
                    let prefix = format!("{key}:");
                    let line = stdout
                        .lines()
                        .find(|l| l.starts_with(&prefix))
                        .unwrap_or_else(|| panic!("no `{prefix}` in\n{stdout}"));
                    line[prefix.len()..]
                        .split_whitespace()
                        .map(|n| n.parse().expect("number"))
                        .collect()
                };
                assert_close(&boxes("MediaBox"), &[0.0, 0.0, pt(w), pt(h)], 0.01);
                assert_close(&boxes("CropBox"), &[0.0, 0.0, pt(w), pt(h)], 0.01);
                assert_close(
                    &boxes("BleedBox"),
                    &[pt(slug), pt(slug_v), pt(w - slug), pt(h - slug_v)],
                    0.01,
                );
                assert_close(
                    &boxes("TrimBox"),
                    &[pt(mx), pt(my), pt(mx + 100.0), pt(my + 80.0)],
                    0.01,
                );
            }

            let Some(img) = render("external-vslug", &path, 1, 72) else { return };
            let px = |mm: f64| (mm * 72.0 / 25.4).round() as u32;
            assert!(
                img.width().abs_diff(px(w)) <= 1 && img.height().abs_diff(px(h)) <= 1,
                "{name}: {} x {} px",
                img.width(),
                img.height()
            );
            // Page millimetres (origin: top-left of the trim) to pixels, per axis.
            let at = |x: f64, y: f64| {
                let scale = 72.0 / 25.4;
                let (col, row) =
                    (((x + mx) * scale).floor() as u32, ((y + my) * scale).floor() as u32);
                img.get_pixel(col.min(img.width() - 1), row.min(img.height() - 1)).0
            };
            // Red rectangle (10, 10) .. (40, 30): every corner and just outside each edge.
            for (x, y) in [(25.0, 20.0), (11.0, 11.0), (39.0, 11.0), (11.0, 29.0), (39.0, 29.0)] {
                assert!(is_red(at(x, y)), "{name}: red at ({x}, {y}): {:?}", at(x, y));
            }
            for (x, y) in [(9.0, 20.0), (41.0, 20.0), (25.0, 9.0), (25.0, 31.0), (25.0, 60.0)] {
                assert!(is_white(at(x, y)), "{name}: white at ({x}, {y}): {:?}", at(x, y));
            }
            // Clipped image: visible in (60, 20) .. (80, 40) only.
            for (x, y) in [(70.0, 30.0), (61.0, 21.0), (79.0, 39.0)] {
                assert!(is_blue(at(x, y)), "{name}: blue at ({x}, {y}): {:?}", at(x, y));
            }
            for (x, y) in [(70.0, 19.0), (70.0, 41.0), (59.0, 30.0), (81.0, 30.0)] {
                assert!(is_white(at(x, y)), "{name}: clipped away at ({x}, {y}): {:?}", at(x, y));
            }
            // Rotated bar: (28, 50) .. (32, 70).
            for (x, y) in [(30.0, 60.0), (29.0, 51.0), (31.0, 69.0)] {
                assert!(is_green(at(x, y)), "{name}: green at ({x}, {y}): {:?}", at(x, y));
            }
            for (x, y) in [(30.0, 48.5), (30.0, 71.5), (26.5, 60.0), (33.5, 60.0)] {
                assert!(is_white(at(x, y)), "{name}: white at ({x}, {y}): {:?}", at(x, y));
            }
        }
    }
}

#[test]
fn images_are_upright_and_keep_their_colours() {
    // 2 x 2 quadrants: red | green over blue | white.
    let quadrants = |x: u32, y: u32, half: u32| -> [u8; 3] {
        match (x < half, y < half) {
            (true, true) => [255, 0, 0],
            (false, true) => [0, 255, 0],
            (true, false) => [0, 0, 255],
            (false, false) => [255, 255, 255],
        }
    };
    let mut raw_pixels = Vec::new();
    for y in 0..2 {
        for x in 0..2 {
            raw_pixels.extend_from_slice(&quadrants(x, y, 1));
        }
    }
    let rgba: Vec<u8> = raw_pixels.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect();
    let sources: [(&str, ImageData); 3] = [
        ("raw", raw(PixelFormat::Rgb8, 2, 2, raw_pixels)),
        ("rgba", raw(PixelFormat::Rgba8, 2, 2, rgba)),
        ("jpeg", jpeg_rgb(64, 64, |x, y| quadrants(x, y, 32))),
    ];
    for standard in [PdfStandard::Plain, PdfStandard::X3_2002, PdfStandard::X4] {
        for (kind, data) in &sources {
            let mut doc = base_doc(standard);
            let image = doc.add_image(data.clone(), None);
            let mut page = Page::new(Size::new(60.0, 60.0), 0.0, 0.0);
            // Deliberately not square: 40 x 30 mm.
            page.items.push(Item::Image { image, rect: Rect::new(10.0, 20.0, 40.0, 30.0) });
            doc.pages.push(page);
            let name = format!("quadrants-{}-{kind}", name_of(standard));
            let Some(img) = rasterize("external-images", &name, &doc) else { return };
            let at = |x: f64, y: f64| pixel_at(&img, 72, 0.0, x, y);
            let checks = [
                ((20.0, 27.5), [255u8, 0, 0], "top-left"),
                ((40.0, 27.5), [0, 255, 0], "top-right"),
                ((20.0, 42.5), [0, 0, 255], "bottom-left"),
                ((40.0, 42.5), [255, 255, 255], "bottom-right"),
                ((5.0, 35.0), [255, 255, 255], "left of the image"),
                ((30.0, 15.0), [255, 255, 255], "above the image"),
            ];
            for ((x, y), expected, what) in checks {
                assert!(close(at(x, y), expected, 12), "{name}: {what}: {:?}", at(x, y));
            }
        }
    }
}

#[test]
fn predicted_gradients_decode_to_the_right_pixels() {
    // Smooth raw images are PNG-predicted before compression; renderers must get the
    // original samples back. 64 x 64 px on 64 x 64 pt: one pixel per rendered pixel.
    let (w, h) = (64u32, 64u32);
    let pixel = |x: u32, y: u32| -> [u8; 3] { [(x * 4) as u8, (y * 4) as u8, ((x + y) * 2) as u8] };
    let mut rgb = Vec::new();
    let mut rgba = Vec::new();
    for y in 0..h {
        for x in 0..w {
            rgb.extend_from_slice(&pixel(x, y));
            rgba.extend_from_slice(&pixel(x, y));
            // Opaque except for a transparent top-left corner.
            rgba.push(if x < 16 && y < 16 { 0 } else { 255 });
        }
    }
    let side = 64.0 * MM_PER_PT;
    for standard in [PdfStandard::Plain, PdfStandard::X4] {
        let mut doc = base_doc(standard);
        let opaque = doc.add_image(raw(PixelFormat::Rgb8, w, h, rgb.clone()), None);
        let masked = doc.add_image(raw(PixelFormat::Rgba8, w, h, rgba.clone()), None);
        let mut page = Page::new(Size::new(3.0 * side, side), 0.0, 0.0);
        page.items.push(Item::Image { image: opaque, rect: Rect::new(0.0, 0.0, side, side) });
        page.items
            .push(Item::Image { image: masked, rect: Rect::new(2.0 * side, 0.0, side, side) });
        doc.pages.push(page);
        let name = format!("gradient-{}", name_of(standard));
        let Some(img) = rasterize("external-images", &name, &doc) else { return };
        assert_eq!((img.width(), img.height()), (192, 64));
        for (x, y) in [(2, 2), (10, 50), (33, 7), (40, 40), (61, 61), (5, 30), (60, 3)] {
            let expected = pixel(x, y);
            let got = img.get_pixel(x, y).0;
            assert!(close(got, expected, 6), "{name}: pixel ({x}, {y}): {got:?} vs {expected:?}");
            let got = img.get_pixel(128 + x, y).0;
            let expected = if x < 16 && y < 16 { [255, 255, 255] } else { expected };
            assert!(
                close(got, expected, 6),
                "{name}: masked pixel ({x}, {y}): {got:?} vs {expected:?}"
            );
        }
    }
}

#[test]
fn grey_and_cmyk_images_are_not_inverted() {
    for standard in ALL {
        let mut doc = base_doc(standard);
        let grey = doc.add_image(raw(PixelFormat::Gray8, 2, 1, vec![0, 255]), None);
        let grey_jpeg = doc.add_image(jpeg_gray(16, 16, |x, _| if x < 8 { 0 } else { 255 }), None);
        let cyan_raw = doc.add_image(raw(PixelFormat::Cmyk8, 1, 1, vec![255, 0, 0, 0]), None);
        let cyan_cmyk = doc.add_image(jpeg_cmyk(16, 16, [255, 0, 0, 0], false), None);
        let cyan_ycck = doc.add_image(jpeg_cmyk(16, 16, [255, 0, 0, 0], true), None);
        let black_ycck = doc.add_image(jpeg_cmyk(16, 16, [0, 0, 0, 255], true), None);
        let mut page = Page::new(Size::new(140.0, 40.0), 0.0, 0.0);
        for (i, image) in
            [grey, grey_jpeg, cyan_raw, cyan_cmyk, cyan_ycck, black_ycck].into_iter().enumerate()
        {
            page.items.push(Item::Image {
                image,
                rect: Rect::new(10.0 + 20.0 * i as f64, 10.0, 16.0, 16.0),
            });
        }
        doc.pages.push(page);
        let name = format!("inks-{}", name_of(standard));
        let Some(img) = rasterize("external-images", &name, &doc) else { return };
        let at = |x: f64, y: f64| pixel_at(&img, 72, 0.0, x, y);

        for x0 in [10.0, 30.0] {
            assert!(
                is_black(at(x0 + 4.0, 18.0)),
                "{name}: grey 0 is black: {:?}",
                at(x0 + 4.0, 18.0)
            );
            assert!(
                is_white(at(x0 + 12.0, 18.0)),
                "{name}: grey 255 is white: {:?}",
                at(x0 + 12.0, 18.0)
            );
        }
        // 100 % cyan ink: no red light, plenty of green and blue. An inverted image would
        // come out dark red or black.
        for (x0, what) in [(50.0, "raw"), (70.0, "CMYK JPEG"), (90.0, "YCCK JPEG")] {
            let p = at(x0 + 8.0, 18.0);
            assert!(p[0] < 90 && p[1] > 130 && p[2] > 180, "{name}: {what} cyan: {p:?}");
        }
        // 100 % black ink.
        assert!(is_black(at(118.0, 18.0)), "{name}: K only: {:?}", at(118.0, 18.0));
    }
}

#[test]
fn soft_masks_blend_with_the_background() {
    for standard in [PdfStandard::X4, PdfStandard::Plain] {
        let mut doc = base_doc(standard);
        // Left half opaque yellow, right half fully transparent, over a black bar.
        let mut rgba = Vec::new();
        for _y in 0..4 {
            for x in 0..4 {
                rgba.extend_from_slice(&[255, 255, 0, if x < 2 { 255 } else { 0 }]);
            }
        }
        let image = doc.add_image(raw(PixelFormat::Rgba8, 4, 4, rgba), None);
        let mut page = Page::new(Size::new(60.0, 40.0), 0.0, 0.0);
        page.items.push(Item::filled_rect(Rect::new(0.0, 15.0, 60.0, 10.0), Color::BLACK));
        page.items.push(Item::Image { image, rect: Rect::new(10.0, 10.0, 40.0, 20.0) });
        doc.pages.push(page);
        let name = format!("mask-{}", name_of(standard));
        let Some(img) = rasterize("external-images", &name, &doc) else { return };
        let at = |x: f64, y: f64| pixel_at(&img, 72, 0.0, x, y);
        let yellow = |p: [u8; 3]| p[0] > 220 && p[1] > 220 && p[2] < 60;
        assert!(yellow(at(15.0, 20.0)), "{name}: opaque part covers the bar: {:?}", at(15.0, 20.0));
        assert!(yellow(at(15.0, 12.0)), "{name}: {:?}", at(15.0, 12.0));
        assert!(is_black(at(45.0, 20.0)), "{name}: bar shows through: {:?}", at(45.0, 20.0));
        assert!(is_white(at(45.0, 12.0)), "{name}: paper shows through: {:?}", at(45.0, 12.0));
    }
}

#[test]
fn strokes_fill_rules_and_registration_colour_render_correctly() {
    for standard in ALL {
        let mut doc = base_doc(standard);
        let mut page = Page::new(Size::new(100.0, 60.0), 0.0, 0.0);
        // An 8 pt wide horizontal line at y = 10 mm (8 px at 72 dpi), butt caps.
        page.items.push(Item::stroke(
            Path::line(Point::new(10.0, 10.0), Point::new(60.0, 10.0)),
            Stroke::solid(Color::BLACK, 8.0),
        ));
        // A dashed line: 4 mm on, 4 mm off.
        page.items.push(Item::stroke(
            Path::line(Point::new(10.0, 20.0), Point::new(60.0, 20.0)),
            Stroke::dashed(Color::BLACK, 4.0, vec![4.0, 4.0]),
        ));
        // Round caps extend the line by half its width.
        page.items.push(Item::stroke(
            Path::line(Point::new(70.0, 10.0), Point::new(90.0, 10.0)),
            Stroke { cap: StrokeCap::Round, ..Stroke::solid(Color::BLACK, 8.0) },
        ));
        // Two nested squares drawn in the same direction: non-zero fills the hole,
        // even-odd leaves it open.
        let mut nested = Path::new();
        nested.push_rect(Rect::new(0.0, 0.0, 16.0, 16.0)).push_rect(Rect::new(5.0, 5.0, 6.0, 6.0));
        for (x, rule) in [(10.0, FillRule::NonZero), (35.0, FillRule::EvenOdd)] {
            page.items.push(Item::group(
                Transform::translate(x, 30.0),
                vec![Item::Path {
                    path: nested.clone(),
                    fill: Some((Color::BLACK, rule)),
                    stroke: None,
                }],
            ));
        }
        // Registration colour prints on every plate: black on screen.
        page.items.push(Item::filled_rect(Rect::new(70.0, 30.0, 16.0, 16.0), Color::Registration));
        doc.pages.push(page);
        let name = format!("strokes-{}", name_of(standard));
        let Some(img) = rasterize("external-strokes", &name, &doc) else { return };
        let at = |x: f64, y: f64| pixel_at(&img, 72, 0.0, x, y);
        let half_width = 4.0 * MM_PER_PT;

        assert!(is_black(at(35.0, 10.0)), "{name}");
        assert!(
            is_black(at(35.0, 10.0 - half_width + 0.5))
                && is_black(at(35.0, 10.0 + half_width - 0.5)),
            "{name}: the line is 8 pt wide"
        );
        assert!(
            is_white(at(35.0, 10.0 - half_width - 0.6))
                && is_white(at(35.0, 10.0 + half_width + 0.6)),
            "{name}: and not wider"
        );
        assert!(
            is_white(at(9.0, 10.0)) && is_white(at(61.0, 10.0)),
            "{name}: butt caps end at the end points"
        );

        // Dashes: on at 10..14, off at 14..18, on at 18..22, ...
        for (x, on) in
            [(12.0, true), (16.0, false), (20.0, true), (24.0, false), (28.0, true), (56.0, false)]
        {
            let p = at(x, 20.0);
            assert_eq!(is_black(p), on, "{name}: dash at x = {x}: {p:?}");
            assert_eq!(is_white(p), !on, "{name}: gap at x = {x}: {p:?}");
        }

        assert!(
            is_black(at(70.0 - half_width + 0.6, 10.0)),
            "{name}: round cap before the start point"
        );
        assert!(
            is_black(at(90.0 + half_width - 0.6, 10.0)),
            "{name}: round cap after the end point"
        );
        assert!(is_white(at(70.0 - half_width - 0.6, 10.0)), "{name}");

        assert!(
            is_black(at(18.0, 38.0)),
            "{name}: non-zero fills the inner square: {:?}",
            at(18.0, 38.0)
        );
        assert!(is_white(at(43.0, 38.0)), "{name}: even-odd leaves a hole: {:?}", at(43.0, 38.0));
        assert!(is_black(at(37.0, 32.0)) && is_black(at(49.0, 44.0)), "{name}: the ring is filled");

        assert!(is_black(at(78.0, 38.0)), "{name}: registration colour: {:?}", at(78.0, 38.0));
    }
}

#[test]
fn text_is_upright_and_sits_on_its_baseline() {
    let data = font("AtkinsonHyperlegible-Bold.ttf");
    let face = reference::Face::parse(&data, 0).expect("parses");
    // A capital I at 72 pt: a plain vertical bar from the baseline up to the cap height.
    let glyph = face.glyph_index('I').expect("glyph");
    let bbox = face.glyph_bounding_box(glyph).expect("outline");
    let size_pt = 72.0;
    let unit = size_pt * MM_PER_PT / f64::from(face.units_per_em());
    let (x_min, x_max) = (f64::from(bbox.x_min) * unit, f64::from(bbox.x_max) * unit);
    let (bottom, top) = (f64::from(bbox.y_min) * unit, f64::from(bbox.y_max) * unit);
    assert!(top > 15.0 && bottom.abs() < 0.5 && x_max - x_min > 2.0, "test glyph is a tall bar");

    for standard in ALL {
        let mut doc = base_doc(standard);
        let id = doc.add_font(data.clone(), 0);
        let mut page = Page::new(Size::new(100.0, 60.0), 0.0, 0.0);
        let origin = Point::new(20.0, 40.0);
        page.items.push(text_item(id, &data, "I", size_pt, origin, Color::BLACK));
        // The same glyph in a group turned a quarter turn clockwise about (70, 20):
        // the bar now points to the right of its origin.
        page.items.push(Item::group(
            Transform::translate(70.0, 20.0).then_inner(Transform::rotate(90.0)),
            vec![text_item(id, &data, "I", size_pt, Point::new(0.0, 0.0), Color::BLACK)],
        ));
        doc.pages.push(page);
        let name = format!("text-{}", name_of(standard));
        let Some(img) = rasterize("external-text", &name, &doc) else { return };
        let at = |x: f64, y: f64| pixel_at(&img, 72, 0.0, x, y);
        let centre_x = origin.x + (x_min + x_max) / 2.0;

        // Upright: ink from the baseline (y = 40) up to the cap height, none below.
        assert!(is_black(at(centre_x, origin.y - top / 2.0)), "{name}: middle of the stem");
        assert!(is_black(at(centre_x, origin.y - top + 1.0)), "{name}: top of the stem");
        assert!(is_black(at(centre_x, origin.y - 1.0)), "{name}: foot of the stem");
        assert!(is_white(at(centre_x, origin.y + 1.5)), "{name}: nothing below the baseline");
        assert!(
            is_white(at(centre_x, origin.y - top - 1.5)),
            "{name}: nothing above the cap height"
        );
        assert!(
            is_white(at(origin.x + x_min - 1.5, origin.y - top / 2.0)),
            "{name}: left of the stem"
        );
        assert!(
            is_white(at(origin.x + x_max + 1.5, origin.y - top / 2.0)),
            "{name}: right of the stem"
        );

        // Rotated: local (x, y) -> page (70 - y, 20 + x); "up" (negative y) is to the right.
        let centre_y = 20.0 + (x_min + x_max) / 2.0;
        assert!(is_black(at(70.0 + top / 2.0, centre_y)), "{name}: rotated stem");
        assert!(is_black(at(70.0 + top - 1.0, centre_y)), "{name}: rotated top");
        assert!(is_white(at(70.0 - 1.5, centre_y)), "{name}: nothing left of the rotated baseline");
        assert!(is_white(at(70.0 + top + 1.5, centre_y)), "{name}");
    }
}

/// A document that uses every feature at once.
fn everything(standard: PdfStandard) -> Document {
    let mut doc = base_doc(standard);
    let serif = font("Spectral-Regular.ttf");
    let italic = font("Spectral-Italic.ttf");
    let cff = fixture("SourceCodePro-Regular.otf");
    let whole = with_fs_type(&font("CourierPrime-Bold.ttf"), 0x0100);
    let (f_serif, f_italic, f_cff, f_whole) = (
        doc.add_font(serif.clone(), 0),
        doc.add_font(italic.clone(), 0),
        doc.add_font(cff.clone(), 0),
        doc.add_font(whole.clone(), 0),
    );
    let mut page = Page::new(Size::new(150.0, 100.0), 3.0, 5.0);
    page.label = Some("recto".into());
    page.items = geometry_items(&mut doc);
    let lines: [(_, &Arc<[u8]>, &str); 4] = [
        (f_serif, &serif, EXTRACT_LINES[0]),
        (f_italic, &italic, EXTRACT_LINES[1]),
        (f_cff, &cff, EXTRACT_LINES[2]),
        (f_whole, &whole, EXTRACT_LINES[3]),
    ];
    for (i, (id, data, line)) in lines.into_iter().enumerate() {
        page.items.push(text_item(
            id,
            data,
            line,
            11.0,
            Point::new(10.0, 70.0 + 7.0 * i as f64),
            Color::BLACK,
        ));
    }
    page.items.push(Item::layer(
        "marks",
        vec![Item::stroke(
            Path::line(Point::new(-7.0, 0.0), Point::new(-4.0, 0.0)),
            Stroke::dashed(Color::Registration, 0.25, vec![0.5, 0.5]),
        )],
    ));
    page.items.push(Item::fill(Path::circle(Point::new(120.0, 30.0), 10.0), Color::gray(0.5)));
    if standard != PdfStandard::X1a2001 {
        page.items.push(Item::fill(
            Path::circle(Point::new(120.0, 60.0), 10.0),
            Color::rgb8(250, 120, 10),
        ));
        let tagged =
            doc.add_image(raw(PixelFormat::Rgb8, 1, 1, vec![9, 99, 199]), Some(srgb_v2_profile()));
        page.items.push(Item::Image { image: tagged, rect: Rect::new(100.0, 80.0, 10.0, 10.0) });
    }
    if matches!(standard, PdfStandard::X4 | PdfStandard::Plain) {
        let translucent =
            doc.add_image(raw(PixelFormat::Rgba8, 2, 1, vec![255, 0, 0, 128, 0, 0, 255, 64]), None);
        page.items
            .push(Item::Image { image: translucent, rect: Rect::new(115.0, 80.0, 20.0, 10.0) });
    }
    doc.pages.push(page.clone());
    page.label = None;
    page.trim = Size::new(210.0, 297.0);
    doc.pages.push(page);
    doc
}

const EXTRACT_LINES: [&str; 4] = [
    "Élève, où était l'œuvre de Noël ? Ça, à Æsope !",
    "Zoë préfère « cœur » à “coeur” — naïveté…",
    "fn main() { println!(\"déjà vu: {}\", 0xCAFE); }",
    "CODE: K7Q2M-9XTRD / ÉTÉ 2026",
];

#[test]
fn every_variant_passes_qpdf_and_ghostscript() {
    for standard in ALL {
        let doc = everything(standard);
        for compress in [true, false] {
            for keep_layers in [true, false] {
                let pdf =
                    write_pdf(&doc, &WriteOptions { compress, keep_layers }).expect("writable");
                let name = format!("everything-{}-{compress}-{keep_layers}.pdf", name_of(standard));
                let path = save("external-validators", &name, &pdf);

                if let Some((ok, stdout, stderr)) = run("qpdf", &["--check", &path]) {
                    assert!(ok, "{name}: qpdf --check failed:\n{stdout}\n{stderr}");
                    assert!(
                        stdout.contains("No syntax or stream encoding errors found"),
                        "{name}: {stdout}"
                    );
                    assert!(stderr.trim().is_empty(), "{name}: {stderr}");
                }
                if let Some((ok, stdout, stderr)) = run(
                    "gs",
                    &[
                        "-q",
                        "-dNOPAUSE",
                        "-dBATCH",
                        "-dPDFSTOPONERROR",
                        "-dPDFSTOPONWARNING",
                        "-sDEVICE=nullpage",
                        &path,
                    ],
                ) {
                    let output = format!("{stdout}{stderr}").to_lowercase();
                    assert!(ok, "{name}: ghostscript failed:\n{output}");
                    assert!(
                        !output.contains("error") && !output.contains("warning"),
                        "{name}: {output}"
                    );
                }
            }
        }
    }
}

#[test]
fn pdfinfo_reports_the_boxes_of_every_page() {
    let doc = everything(PdfStandard::X4);
    let pdf = write_pdf(&doc, &WriteOptions::default()).expect("writable");
    let path = save("external-pdfinfo", "boxes.pdf", &pdf);
    let Some((ok, stdout, stderr)) = run("pdfinfo", &["-box", "-f", "1", "-l", "2", &path]) else {
        return;
    };
    assert!(ok, "pdfinfo failed: {stderr}");
    let boxes = |page: u32, name: &str| -> Vec<f64> {
        let prefix = format!("Page{page:>5} {name}:");
        let line = stdout
            .lines()
            .find(|l| l.starts_with(&prefix))
            .unwrap_or_else(|| panic!("no `{prefix}` in\n{stdout}"));
        line[prefix.len()..].split_whitespace().map(|n| n.parse().expect("number")).collect()
    };
    // Page 1: 150 x 100 mm trim, 3 mm bleed, 5 mm slug.
    assert_close(&boxes(1, "MediaBox"), &[0.0, 0.0, pt(166.0), pt(116.0)], 0.01);
    assert_close(&boxes(1, "CropBox"), &[0.0, 0.0, pt(166.0), pt(116.0)], 0.01);
    assert_close(&boxes(1, "BleedBox"), &[pt(5.0), pt(5.0), pt(161.0), pt(111.0)], 0.01);
    assert_close(&boxes(1, "TrimBox"), &[pt(8.0), pt(8.0), pt(158.0), pt(108.0)], 0.01);
    // Page 2: A4 trim with the same bleed and slug.
    assert_close(&boxes(2, "MediaBox"), &[0.0, 0.0, pt(226.0), pt(313.0)], 0.01);
    assert_close(&boxes(2, "BleedBox"), &[pt(5.0), pt(5.0), pt(221.0), pt(308.0)], 0.01);
    assert_close(&boxes(2, "TrimBox"), &[pt(8.0), pt(8.0), pt(218.0), pt(305.0)], 0.01);
    assert!(stdout.contains("Page    1 size:"), "{stdout}");
}

#[test]
fn pdffonts_sees_embedded_subsets_and_pdftotext_recovers_the_text() {
    for standard in ALL {
        let doc = everything(standard);
        let pdf = write_pdf(&doc, &WriteOptions::default()).expect("writable");
        let path = save("external-text", &format!("fonts-{}.pdf", name_of(standard)), &pdf);

        if let Some((ok, stdout, stderr)) = run("pdffonts", &[&path]) {
            assert!(ok, "pdffonts failed: {stderr}");
            let rows: Vec<Vec<&str>> =
                stdout.lines().skip(2).map(|l| l.split_whitespace().collect()).collect();
            assert_eq!(rows.len(), 4, "{stdout}");
            let row = |needle: &str| -> &Vec<&str> {
                rows.iter()
                    .find(|r| r[0].contains(needle))
                    .unwrap_or_else(|| panic!("no {needle} in\n{stdout}"))
            };
            let flags = |r: &Vec<&str>| -> (String, String, String) {
                let n = r.len();
                (r[n - 5].to_owned(), r[n - 4].to_owned(), r[n - 3].to_owned())
            };
            let yes = |a: &str, b: &str, c: &str| (a.to_owned(), b.to_owned(), c.to_owned());
            // emb, sub, uni
            assert_eq!(flags(row("+Spectral-Regular")), yes("yes", "yes", "yes"), "{stdout}");
            assert_eq!(flags(row("+Spectral-Italic")), yes("yes", "yes", "yes"), "{stdout}");
            assert_eq!(flags(row("+SourceCodePro-Regular")), yes("yes", "yes", "yes"), "{stdout}");
            // The font that forbids subsetting is embedded whole.
            let whole = row("CourierPrime-Bold");
            assert!(!whole[0].contains('+'), "{stdout}");
            assert_eq!(flags(whole), yes("yes", "no", "yes"), "{stdout}");
            assert!(row("+SourceCodePro-Regular").join(" ").contains("CID Type 0C"), "{stdout}");
            assert!(row("+Spectral-Regular").join(" ").contains("CID TrueType"), "{stdout}");
        }

        if let Some((ok, stdout, stderr)) = run("pdftotext", &["-enc", "UTF-8", &path, "-"]) {
            assert!(ok, "pdftotext failed: {stderr}");
            for line in EXTRACT_LINES {
                assert_eq!(
                    stdout.matches(line).count(),
                    2,
                    "{standard:?}: {line:?} on both pages:\n{stdout}"
                );
            }
        }
    }
}
