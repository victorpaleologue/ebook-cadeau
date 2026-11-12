//! `sniff::probe` against real files produced by encoders and hand-assembled headers.

mod common;

use common::*;
use ebc_covers::sniff::{Format, Orientation, WebpKind, probe, probe_details};
use image::GenericImageView;
use jpeg_encoder::ColorType;

fn dims(bytes: &[u8]) -> (Format, u32, u32) {
    let info = probe(bytes).expect("probe");
    (info.format, info.width, info.height)
}

#[test]
fn real_files_agree_with_a_full_decoder() {
    let files = [
        (Format::Png, png_rgb(37, 53)),
        (Format::Png, png_rgba(5, 300)),
        (Format::Gif, gif(20, 30)),
        (Format::Gif, gif_animated(64, 48)),
        (Format::WebP, webp_lossless(33, 47)),
        (Format::Jpeg, jpeg_baseline(64, 48)),
        (Format::Jpeg, jpeg_custom(123, 77, ColorType::Rgb, true, None, None, 0)),
        (Format::Jpeg, jpeg_custom(31, 17, ColorType::Luma, false, Some(2), None, 0)),
    ];
    for (format, bytes) in files {
        let info = probe(&bytes).expect("probe");
        let decoded = image::load_from_memory(&bytes).expect("decode");
        assert_eq!(info.format, format);
        assert_eq!((info.width, info.height), decoded.dimensions(), "{format:?}");
    }
}

#[test]
fn png_details() {
    let d = probe_details(&png_rgb(37, 53)).expect("png");
    assert_eq!(
        (d.bit_depth, d.channels, d.has_alpha, d.animated, d.progressive),
        (Some(8), Some(3), false, false, false)
    );
    let d = probe_details(&png_rgba(5, 300)).expect("png");
    assert_eq!((d.channels, d.has_alpha), (Some(4), true));
    // The IHDR alone is enough.
    assert_eq!(dims(&png_rgb(640, 480)[..24]), (Format::Png, 640, 480));
    assert!(probe(&png_rgb(640, 480)[..23]).is_none());
}

#[test]
fn gif_details() {
    assert_eq!(dims(&gif(20, 30)), (Format::Gif, 20, 30));
    assert!(!probe_details(&gif(20, 30)).expect("gif").animated);
    assert!(probe_details(&gif_animated(64, 48)).expect("gif").animated);
    assert_eq!(dims(b"GIF87a\x2C\x01\xC2\x01\x00\x00\x00"), (Format::Gif, 300, 450));
}

#[test]
fn jpeg_frame_types_and_components() {
    let baseline = probe_details(&jpeg_baseline(64, 48)).expect("jpeg");
    assert_eq!(
        (baseline.jpeg_sof, baseline.progressive, baseline.channels, baseline.bit_depth),
        (Some(0xC0), false, Some(3), Some(8))
    );
    let progressive =
        probe_details(&jpeg_custom(123, 77, ColorType::Rgb, true, None, None, 0)).expect("jpeg");
    assert_eq!((progressive.jpeg_sof, progressive.progressive), (Some(0xC2), true));
    assert_eq!((progressive.info.width, progressive.info.height), (123, 77));
    let grey = probe_details(&jpeg_custom(31, 17, ColorType::Luma, false, Some(2), None, 0))
        .expect("jpeg");
    assert_eq!(grey.channels, Some(1));
    let cmyk =
        probe_details(&jpeg_custom(40, 60, ColorType::Cmyk, false, None, None, 0)).expect("jpeg");
    assert_eq!((cmyk.channels, cmyk.info.width, cmyk.info.height), (Some(4), 40, 60));
}

#[test]
fn jpeg_every_sof_marker_by_hand() {
    for marker in [0xC0u8, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF] {
        let mut j = vec![0xFF, 0xD8];
        j.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
        j.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
        // A Huffman table (C4), a reserved JPG (C8) and an arithmetic-conditioning
        // segment (CC) that must not be read as frames, though their bytes look like one.
        for not_a_frame in [0xC4u8, 0xC8, 0xCC] {
            j.extend_from_slice(&[
                0xFF,
                not_a_frame,
                0x00,
                0x0B,
                8,
                0x11,
                0x11,
                0x22,
                0x22,
                3,
                0,
                0,
                0,
            ]);
        }
        // Fill bytes before the marker are legal.
        j.extend_from_slice(&[
            0xFF, 0xFF, 0xFF, 0xFF, marker, 0x00, 0x11, 12, 0x08, 0x36, 0x05, 0x13, 3,
        ]);
        j.extend_from_slice(&[1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
        j.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 1, 1, 0, 0, 0x3F, 0]);
        let d = probe_details(&j).unwrap_or_else(|| panic!("SOF {marker:#X}"));
        assert_eq!((d.info.width, d.info.height), (1299, 2102), "SOF {marker:#X}");
        assert_eq!(d.jpeg_sof, Some(marker));
        assert_eq!(d.bit_depth, Some(12));
        assert_eq!(d.progressive, matches!(marker, 0xC2 | 0xC6 | 0xCA | 0xCE), "SOF {marker:#X}");
    }
}

#[test]
fn jpeg_large_app_segments_and_restart_markers() {
    // 150 kB of APP7 data before the frame header: found when all bytes are supplied,
    // not found (and no panic) from the first 64 KiB only.
    let big = jpeg_custom(200, 300, ColorType::Rgb, false, Some(4), None, 150_000);
    assert_eq!(dims(&big), (Format::Jpeg, 200, 300));
    assert!(probe(&big[..ebc_covers::sniff::PROBE_BYTES_HINT]).is_none());
    // Restart markers live in the entropy-coded data, after the header.
    let restarts = jpeg_custom(64, 64, ColorType::Rgb, true, Some(1), None, 0);
    assert!(restarts.windows(2).any(|w| w == [0xFF, 0xD0]));
    assert_eq!(dims(&restarts), (Format::Jpeg, 64, 64));
}

#[test]
fn jpeg_exif_orientation() {
    for (value, expected) in [
        (1u16, Orientation::Normal),
        (3, Orientation::Rotate180),
        (6, Orientation::Rotate90),
        (8, Orientation::Rotate270),
        (5, Orientation::Transpose),
    ] {
        for little_endian in [true, false] {
            let exif = tiff_orientation(value, little_endian);
            let bytes = jpeg_custom(300, 200, ColorType::Rgb, false, None, Some(&exif), 0);
            let d = probe_details(&bytes).expect("jpeg");
            assert_eq!(d.orientation, expected, "{value} le={little_endian}");
            assert_eq!((d.info.width, d.info.height), (300, 200));
            let displayed = d.oriented();
            let swapped = value >= 5;
            assert_eq!(
                (displayed.width, displayed.height),
                if swapped { (200, 300) } else { (300, 200) }
            );
        }
    }
    // Garbage orientation values and a truncated EXIF block are ignored.
    let odd =
        jpeg_custom(30, 20, ColorType::Rgb, false, None, Some(&tiff_orientation(77, true)), 0);
    assert_eq!(probe_details(&odd).expect("jpeg").orientation, Orientation::Normal);
    let cut =
        jpeg_custom(30, 20, ColorType::Rgb, false, None, Some(&tiff_orientation(6, true)[..20]), 0);
    assert_eq!(probe_details(&cut).expect("jpeg").orientation, Orientation::Normal);
}

#[test]
fn webp_three_variants() {
    // Lossless, from a real encoder.
    let lossless = webp_lossless(33, 47);
    let d = probe_details(&lossless).expect("webp");
    assert_eq!(
        (d.info.format, d.info.width, d.info.height, d.webp),
        (Format::WebP, 33, 47, Some(WebpKind::Lossless))
    );

    // Lossy: 14-bit sizes; the two upper bits are the scaling factor and are masked.
    let lossy = riff_webp(&[(b"VP8 ", vp8_frame_header(550, 368, 0))]);
    let d = probe_details(&lossy).expect("webp");
    assert_eq!((d.info.width, d.info.height, d.webp), (550, 368, Some(WebpKind::Lossy)));
    let scaled = riff_webp(&[(b"VP8 ", vp8_frame_header(16383, 1, 3))]);
    assert_eq!(dims(&scaled), (Format::WebP, 16383, 1));
    // An inter frame (bit 0 of the frame tag set) has no size.
    let mut inter = vp8_frame_header(550, 368, 0);
    inter[0] |= 1;
    assert!(probe(&riff_webp(&[(b"VP8 ", inter)])).is_none());

    // Lossless by hand at the 14-bit maximum, with the alpha bit.
    let bits: u32 = (16384 - 1) | (16384 - 1) << 14 | 1 << 28;
    let mut vp8l = vec![0x2F];
    vp8l.extend_from_slice(&bits.to_le_bytes());
    let d = probe_details(&riff_webp(&[(b"VP8L", vp8l.clone())])).expect("webp");
    assert_eq!((d.info.width, d.info.height, d.has_alpha), (16384, 16384, true));
    vp8l[4] |= 0x20; // a version other than 0
    assert!(probe(&riff_webp(&[(b"VP8L", vp8l)])).is_none());

    // Extended: 24-bit canvas sizes, flags, EXIF chunk after the pixels.
    let mut exif = b"Exif\0\0".to_vec();
    exif.extend(tiff_orientation(6, false));
    let extended = riff_webp(&[
        (b"VP8X", vp8x_payload(0x10 | 0x08 | 0x02, 70_000, 3)),
        (b"ANIM", vec![0; 6]),
        (b"EXIF", exif),
    ]);
    let d = probe_details(&extended).expect("webp");
    assert_eq!((d.info.width, d.info.height, d.webp), (70_000, 3, Some(WebpKind::Extended)));
    assert!(d.has_alpha && d.animated);
    assert_eq!(d.orientation, Orientation::Rotate90);
    let max = riff_webp(&[(b"VP8X", vp8x_payload(0, 1 << 24, 1 << 24))]);
    assert_eq!(dims(&max), (Format::WebP, 1 << 24, 1 << 24));
    // Unknown first chunk.
    assert!(probe(&riff_webp(&[(b"JUNK", vec![0; 32])])).is_none());
}

#[test]
fn avif_and_heif_from_ispe() {
    let pixi = full(b"pixi", 0, 0, &[3, 10, 10, 10]);
    let props = vec![ispe(160, 240), pixi.clone(), ispe(1200, 1800)];
    // The primary item is associated with the third property (the large ispe).
    let avif = bmff(&[b"avif", b"mif1", b"miaf"], &props, &[3, 2]);
    let d = probe_details(&avif).expect("avif");
    assert_eq!((d.info.format, d.info.width, d.info.height), (Format::Avif, 1200, 1800));
    assert_eq!(d.bit_depth, Some(10));
    // … and with the first one when the association says so.
    assert_eq!(dims(&bmff(&[b"avif"], &props, &[1])), (Format::Avif, 160, 240));
    // No association: the largest extent wins.
    assert_eq!(dims(&bmff(&[b"avif"], &props, &[])), (Format::Avif, 1200, 1800));
    // HEIC brands.
    assert_eq!(dims(&bmff(&[b"heic", b"mif1"], &props, &[3])), (Format::Heif, 1200, 1800));
    assert_eq!(dims(&bmff(&[b"mif1"], &props, &[3])), (Format::Heif, 1200, 1800));
    // Rotation: irot counts anticlockwise quarter turns.
    let mut rotated = props.clone();
    rotated.push(bx(b"irot", &[1]));
    let d = probe_details(&bmff(&[b"avif"], &rotated, &[3, 4])).expect("avif");
    assert_eq!(d.orientation, Orientation::Rotate270);
    assert_eq!(d.display_size(), (1800, 1200));
    // Sequence brand.
    assert!(probe_details(&bmff(&[b"avis", b"avif"], &props, &[3])).expect("avif").animated);
    // Other ISO-BMFF files are not images.
    assert!(probe(&bmff(&[b"isom", b"mp42"], &props, &[3])).is_none());
    // No ispe at all, or a zero extent.
    assert!(probe(&bmff(&[b"avif"], &[pixi], &[1])).is_none());
    assert!(probe(&bmff(&[b"avif"], &[ispe(0, 10)], &[1])).is_none());
    // A 64-bit box size and a "to the end of the file" size on the way.
    let mut big = bx(b"ftyp", b"avif\0\0\0\0avif");
    let free_payload = [0u8; 24];
    big.extend_from_slice(&1u32.to_be_bytes());
    big.extend_from_slice(b"free");
    big.extend_from_slice(&((16 + free_payload.len()) as u64).to_be_bytes());
    big.extend_from_slice(&free_payload);
    let mut meta_body = vec![0u8; 4];
    meta_body.extend(bx(b"iprp", &bx(b"ipco", &ispe(321, 654))));
    big.extend_from_slice(&0u32.to_be_bytes()); // size 0: extends to the end
    big.extend_from_slice(b"meta");
    big.extend_from_slice(&meta_body);
    assert_eq!(dims(&big), (Format::Avif, 321, 654));
}

#[test]
fn not_images() {
    for bytes in [
        &b""[..],
        b"\xFF\xD8",
        b"<html><body>404</body></html>",
        b"{\"error\":\"not found\"}",
        b"%PDF-1.7",
        b"BM\x36\x00\x00\x00",
        b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00isommp42",
        b"RIFF\x24\x00\x00\x00WAVEfmt ",
    ] {
        assert!(probe(bytes).is_none(), "{:?}", String::from_utf8_lossy(bytes));
    }
}
