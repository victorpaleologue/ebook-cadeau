//! Cover variants: formats, sizes, colour models, transparency, and bytes that are not
//! an image at all.

#![allow(clippy::unwrap_used)]

mod common;

use common::{art, errors, expectations, fonts, images, jpeg, png, poche};
use ebc_card::{
    CardError, CardRequest, CardWarning, CoverInput, MAX_COVER_SIDE_PX, RESAMPLE_PPI, RightsBasis,
    compose, render_pdf,
};
use ebc_layout::{PrintProfile, Sheet};
use ebc_preflight::check;
use ebc_scene::{ImageData, PixelFormat, Pixmap};

const RIGHTS: RightsBasis = RightsBasis::RightsHolderDeclared;

fn home() -> PrintProfile {
    PrintProfile::home(Sheet::A4)
}

fn encoded(bytes: Vec<u8>) -> CardRequest {
    let mut req = poche(home());
    req.cover = CoverInput::Encoded { bytes, rights: RIGHTS };
    req
}

fn low_resolution(warnings: &[CardWarning]) -> Option<(f64, bool)> {
    warnings.iter().find_map(|w| match w {
        CardWarning::LowResolution { ppi, below_minimum, .. } => Some((*ppi, *below_minimum)),
        _ => None,
    })
}

#[test]
fn a_baseline_or_progressive_rgb_jpeg_is_embedded_as_it_is() {
    let fonts = fonts();
    let pixmap = art::cover(1240, 2008, false, true);
    for progressive in [false, true] {
        let bytes = jpeg(&pixmap, progressive);
        let req = encoded(bytes.clone());
        let (doc, warnings) = compose(&req, &fonts).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        let ImageData::Jpeg { bytes: embedded, width_px, height_px, components } =
            &doc.images[0].data
        else {
            panic!("JPEG passthrough expected")
        };
        assert_eq!((*width_px, *height_px, *components), (1240, 2008, 3));
        assert!(embedded.as_ref() == bytes.as_slice(), "the file is embedded byte for byte");
        let (pdf, _) = render_pdf(&req, &fonts).unwrap();
        let report = check(&pdf, &expectations(&req, true)).unwrap();
        assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
        assert_eq!(report.pages[0].images[0].filters, ["DCTDecode"]);
    }
}

#[test]
fn a_grey_jpeg_stays_grey() {
    let fonts = fonts();
    let (w, h) = (620u16, 1004u16);
    let pixels: Vec<u8> = (0..u32::from(w) * u32::from(h)).map(|i| (i % 251) as u8).collect();
    let mut bytes = Vec::new();
    jpeg_encoder::Encoder::new(&mut bytes, 85)
        .encode(&pixels, w, h, jpeg_encoder::ColorType::Luma)
        .unwrap();
    let (doc, _) = compose(&encoded(bytes), &fonts).unwrap();
    assert!(matches!(doc.images[0].data, ImageData::Jpeg { components: 1, .. }));
}

#[test]
fn a_cmyk_jpeg_is_decoded_and_embedded_as_rgb() {
    let fonts = fonts();
    let (w, h) = (620u16, 1004u16);
    // C, M, Y, K per pixel: a cyan-to-magenta ramp. (Whether the decoder's CMYK-to-RGB
    // conversion gives the colours a printer would expect is not checked here: the
    // fixture comes from an encoder, not from a prepress tool.)
    let mut pixels = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let t = (u32::from(x) * 255 / u32::from(w)) as u8;
            pixels.extend_from_slice(&[255 - t, t, 0, (u32::from(y) * 60 / u32::from(h)) as u8]);
        }
    }
    let mut bytes = Vec::new();
    jpeg_encoder::Encoder::new(&mut bytes, 90)
        .encode(&pixels, w, h, jpeg_encoder::ColorType::Cmyk)
        .unwrap();
    let req = encoded(bytes);
    let (doc, _) = compose(&req, &fonts).unwrap();
    let ImageData::Raw { format, width_px, height_px, .. } = &doc.images[0].data else {
        panic!("a CMYK JPEG is never passed through")
    };
    assert_eq!((*format, *width_px, *height_px), (PixelFormat::Rgb8, 620, 1004));
    let (pdf, _) = render_pdf(&req, &fonts).unwrap();
    let report = check(&pdf, &expectations(&req, true)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
}

#[test]
fn png_transparency_is_composited_on_white() {
    let fonts = fonts();
    let mut pixmap = art::cover(620, 1004, false, false);
    // A fully transparent band at the top and a half-transparent black one below it.
    for y in 0..40 {
        for x in 0..620 {
            pixmap.set(x, y, [10, 200, 10, 0]);
            pixmap.set(x, y + 40, [0, 0, 0, 128]);
        }
    }
    for cover in [
        CoverInput::Encoded { bytes: png(&pixmap), rights: RIGHTS },
        CoverInput::Pixels { pixmap: pixmap.clone(), rights: RIGHTS },
    ] {
        let mut req = poche(PrintProfile::upload_eu());
        req.cover = cover;
        let (doc, _) = compose(&req, &fonts).unwrap();
        let ImageData::Raw { pixels, format: PixelFormat::Rgb8, width_px: 620, height_px: 1004 } =
            &doc.images[0].data
        else {
            panic!("raw RGB expected: {:?}", doc.images[0].data.dimensions_px())
        };
        assert_eq!(&pixels[..3], [255, 255, 255], "transparent becomes white, not green");
        let at = (50 * 620 + 10) * 3;
        assert_eq!(&pixels[at..at + 3], [127, 127, 127]);
        // No alpha channel is left, so no transparency reaches a PDF/X file.
        let (pdf, _) = render_pdf(&req, &fonts).unwrap();
        let report = check(&pdf, &expectations(&req, true)).unwrap();
        assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
        assert!(!report.pages[0].transparency);
    }
}

#[test]
fn a_webp_cover_is_decoded() {
    use image::ImageEncoder;
    let fonts = fonts();
    let pixmap = art::cover(310, 502, false, true);
    let mut bytes = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut bytes)
        .write_image(pixmap.data(), 310, 502, image::ExtendedColorType::Rgba8)
        .unwrap();
    let (doc, _) = compose(&encoded(bytes), &fonts).unwrap();
    assert_eq!(doc.images[0].data.dimensions_px(), (310, 502));
}

#[test]
fn a_landscape_cover_is_letterboxed_not_cropped() {
    let fonts = fonts();
    let req = encoded(jpeg(&art::cover(1200, 750, false, true), false));
    let (doc, _) = compose(&req, &fonts).unwrap();
    let (_, rect, clip) = images(&doc, 0)[0];
    assert_eq!(clip, None);
    assert!((rect.w - 100.0).abs() < 1e-9 && (rect.h - 62.5).abs() < 1e-9, "{rect:?}");
    assert_eq!(doc.images[0].data.dimensions_px(), (1200, 750));
}

#[test]
fn a_tiny_cover_is_never_upscaled_but_reported() {
    let fonts = fonts();
    let req = encoded(png(&art::cover(80, 120, false, false)));
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert_eq!(doc.images[0].data.dimensions_px(), (80, 120), "pixels are not invented");
    let (ppi, below_minimum) = low_resolution(&warnings).expect("a resolution warning");
    assert!(below_minimum && (ppi - 80.0 * 25.4 / 100.0).abs() < 0.01, "{ppi}");
    let text = warnings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    assert!(text.contains("20 ppi") && text.contains("100 ppi"), "the numbers are given: {text}");
    // The file is still written; the referee reports the same problem.
    let (pdf, _) = render_pdf(&req, &fonts).unwrap();
    let report = check(&pdf, &expectations(&req, true)).unwrap();
    assert!(report.has(ebc_preflight::codes::IMAGE_LOW_RES));

    // Between the warning level and the minimum (home: 150 and 100 ppi): a softer warning.
    let req = encoded(png(&art::cover(500, 810, false, false)));
    let (_, warnings) = compose(&req, &fonts).unwrap();
    let (ppi, below_minimum) = low_resolution(&warnings).unwrap();
    assert!(!below_minimum && ppi > 100.0 && ppi < 150.0, "{ppi}");
    // Enough pixels: no warning.
    let req = encoded(png(&art::cover(800, 1296, false, false)));
    assert!(low_resolution(&compose(&req, &fonts).unwrap().1).is_none());
}

#[test]
fn a_huge_cover_is_downsampled_to_print_resolution() {
    let fonts = fonts();
    // 6000 × 9000 px: 54 million pixels of a cheap pattern, encoded once.
    let (w, h) = (6000u16, 9000u16);
    let mut pixels = Vec::with_capacity(usize::from(w) * usize::from(h) * 3);
    for y in 0..u32::from(h) {
        for x in 0..u32::from(w) {
            pixels.extend_from_slice(&[(x / 24) as u8, (y / 36) as u8, ((x + y) / 60) as u8]);
        }
    }
    let mut bytes = Vec::new();
    jpeg_encoder::Encoder::new(&mut bytes, 70)
        .encode(&pixels, w, h, jpeg_encoder::ColorType::Rgb)
        .unwrap();
    drop(pixels);
    let req = encoded(bytes);
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    let ImageData::Raw { width_px, height_px, format: PixelFormat::Rgb8, .. } = &doc.images[0].data
    else {
        panic!("a resampled cover is raw RGB")
    };
    // Letterboxed at 100 × 150 mm → 300 ppi is 1181 × 1772 px.
    assert!((i64::from(*width_px) - 1181).abs() <= 1, "{width_px}");
    assert!((i64::from(*height_px) - 1772).abs() <= 1, "{height_px}");
    let downsampled = warnings.iter().find_map(|w| match w {
        CardWarning::CoverDownsampled { from_ppi, to_ppi } => Some((*from_ppi, *to_ppi)),
        _ => None,
    });
    let (from, to) = downsampled.expect("a note about the resampling");
    assert!((from - 1524.0).abs() < 1.0 && to == RESAMPLE_PPI, "{from} -> {to}");
    assert!(low_resolution(&warnings).is_none());
    let pdf = ebc_pdf::write_pdf(&doc, &ebc_pdf::WriteOptions::default()).unwrap();
    assert!(pdf.len() < 8_000_000, "{} bytes", pdf.len());
    let report = check(&pdf, &expectations(&req, true)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
    let image = &report.pages[0].images[0];
    assert!((image.min_ppi() - 300.0).abs() < 1.0, "{image:?}");
}

/// A PNG file that consists of a signature and an IHDR chunk only.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for byte in data {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
            }
        }
        !crc
    }
    let mut chunk = b"IHDR".to_vec();
    chunk.extend_from_slice(&width.to_be_bytes());
    chunk.extend_from_slice(&height.to_be_bytes());
    chunk.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    out.extend_from_slice(&13u32.to_be_bytes());
    out.extend_from_slice(&chunk);
    out.extend_from_slice(&crc32(&chunk).to_be_bytes());
    out
}

#[test]
fn bytes_that_are_not_a_usable_image_give_a_typed_error() {
    let fonts = fonts();
    let good = jpeg(&art::cover(310, 502, false, true), false);
    let mut gif = b"GIF89a".to_vec();
    gif.extend_from_slice(&[1, 0, 1, 0, 0x80, 0, 0, 0, 0, 0, 255, 255, 255, 0x3B]);
    let hostile: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("text", b"<html>not a cover</html>".to_vec()),
        ("zeros", vec![0u8; 4096]),
        ("gif", gif),
        ("jpeg header only", good[..20].to_vec()),
        ("jpeg with a broken header", {
            let mut b = good.clone();
            b[2..40].fill(0xFF);
            b
        }),
        ("png signature only", b"\x89PNG\r\n\x1a\n".to_vec()),
        ("png with a truncated body", png(&art::cover(64, 64, false, false))[..60].to_vec()),
        ("riff that is not webp", b"RIFF\x10\0\0\0WAVEfmt ".to_vec()),
    ];
    for (label, bytes) in hostile {
        match compose(&encoded(bytes), &fonts) {
            Err(CardError::CoverDecode(message)) => assert!(!message.is_empty(), "{label}"),
            Err(other) => panic!("{label}: unexpected error {other}"),
            Ok(_) => panic!("{label}: accepted"),
        }
    }
    // Truncated in the middle of the pixel data: a decoder may salvage it or refuse it,
    // but it must not panic and must not report anything but a decoding error.
    for cut in [good.len() / 3, good.len() / 2, good.len() - 2] {
        match compose(&encoded(good[..cut].to_vec()), &fonts) {
            Ok((doc, _)) => assert_eq!(doc.images.len(), 1),
            Err(CardError::CoverDecode(_)) => {}
            Err(other) => panic!("cut at {cut}: {other}"),
        }
    }
    // Random-looking bytes behind a valid JPEG start.
    let mut noise = good[..200].to_vec();
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..5_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        noise.push(state as u8);
    }
    assert!(matches!(compose(&encoded(noise), &fonts), Ok(_) | Err(CardError::CoverDecode(_))));
}

#[test]
fn oversized_covers_are_refused_before_any_pixel_is_decoded() {
    let fonts = fonts();
    for (w, h) in [(30_000, 30_000), (MAX_COVER_SIDE_PX + 1, 10), (12_000, 12_000)] {
        let result = compose(&encoded(png_header(w, h)), &fonts);
        let Err(CardError::CoverTooLarge { width, height, .. }) = result else {
            panic!("{w}x{h} must be refused as too large")
        };
        assert_eq!((width, height), (w, h));
    }
    // Within the limits, the same header-only file is simply not decodable.
    assert!(matches!(
        compose(&encoded(png_header(100, 100)), &fonts),
        Err(CardError::CoverDecode(_))
    ));
    // A one-pixel image is an image.
    let mut req = poche(home());
    req.cover = CoverInput::Pixels { pixmap: Pixmap::filled(1, 1, [9, 9, 9, 255]), rights: RIGHTS };
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert_eq!(doc.images.len(), 1);
    assert!(low_resolution(&warnings).is_some_and(|(_, below)| below));
}

#[test]
fn the_ground_is_the_colour_of_the_cover_or_paper_white() {
    use ebc_scene::{Color, Item};
    let fonts = fonts();
    let ground_of = |light: bool| {
        let mut req = poche(home());
        req.cover =
            CoverInput::Pixels { pixmap: art::cover(620, 1004, light, true), rights: RIGHTS };
        let (doc, _) = compose(&req, &fonts).unwrap();
        let Item::Path { fill: Some((color, _)), .. } = &doc.pages[0].items[0] else {
            panic!("ground")
        };
        *color
    };
    // The dark cover's navy frame becomes the ground of the whole outside: the cover
    // melts into a jacket of its own colour.
    assert_eq!(ground_of(false), Color::rgb8(14, 22, 48), "a dark cover on its frame colour");
    // The pale cover's frame is too faint a tint to print: paper white.
    assert_eq!(ground_of(true), Color::Gray(1.0), "a pale cover on paper white");
    // The caller's ground wins.
    let mut req = poche(PrintProfile::upload_eu());
    req.cover = CoverInput::Pixels { pixmap: art::cover(620, 1004, true, true), rights: RIGHTS };
    req.ground = Some(ebc_color::Rgb8::new(31, 58, 95));
    let (doc, _) = compose(&req, &fonts).unwrap();
    let Item::Path { fill: Some((Color::Cmyk { c, k, .. }, _)), .. } = &doc.pages[0].items[0]
    else {
        panic!("a CMYK ground in a PDF/X file")
    };
    assert!(*c > 0.5 && *k > 0.5);
}
