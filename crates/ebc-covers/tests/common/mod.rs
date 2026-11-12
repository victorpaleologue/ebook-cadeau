//! Helpers shared by the integration tests: real image bytes produced with the `image`
//! and `jpeg-encoder` crates, hand-assembled container headers, a tiny PRNG.
#![allow(dead_code)]

use std::io::Cursor;

use ebc_scene::Pixmap;
use image::{DynamicImage, ImageFormat, RgbImage, RgbaImage};

/// xorshift64*: deterministic pseudo-random numbers for the robustness loops.
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    pub(crate) fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| (self.next() >> 32) as u8).collect()
    }
}

/// A busy, colourful test picture (nothing like a flat placeholder tile).
pub(crate) fn busy_rgb(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        let v = x.wrapping_mul(2_654_435_761).wrapping_add(y.wrapping_mul(40_503)) >> 7;
        image::Rgb([(x * 7 + y * 13) as u8, (v >> 3) as u8, (x * 3 ^ y * 5) as u8])
    })
}

pub(crate) fn encode(img: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, format).expect("encode");
    out.into_inner()
}

pub(crate) fn png_rgb(w: u32, h: u32) -> Vec<u8> {
    encode(&DynamicImage::ImageRgb8(busy_rgb(w, h)), ImageFormat::Png)
}

pub(crate) fn png_rgba(w: u32, h: u32) -> Vec<u8> {
    let rgba = RgbaImage::from_fn(w, h, |x, y| image::Rgba([x as u8, y as u8, 99, (x + y) as u8]));
    encode(&DynamicImage::ImageRgba8(rgba), ImageFormat::Png)
}

pub(crate) fn png_flat(w: u32, h: u32, grey: u8) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, image::Rgb([grey, grey, grey]));
    encode(&DynamicImage::ImageRgb8(img), ImageFormat::Png)
}

pub(crate) fn jpeg_flat(w: u32, h: u32, grey: u8) -> Vec<u8> {
    let img = RgbImage::from_pixel(w, h, image::Rgb([grey, grey, grey]));
    encode(&DynamicImage::ImageRgb8(img), ImageFormat::Jpeg)
}

pub(crate) fn gif(w: u32, h: u32) -> Vec<u8> {
    encode(&DynamicImage::ImageRgb8(busy_rgb(w, h)), ImageFormat::Gif)
}

pub(crate) fn gif_animated(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut out);
        let frames = (0..2u8).map(|i| {
            let rgba = RgbaImage::from_pixel(w, h, image::Rgba([i * 200, 40, 90, 255]));
            image::Frame::new(rgba)
        });
        encoder.encode_frames(frames).expect("gif frames");
    }
    out
}

pub(crate) fn webp_lossless(w: u32, h: u32) -> Vec<u8> {
    encode(&DynamicImage::ImageRgb8(busy_rgb(w, h)), ImageFormat::WebP)
}

pub(crate) fn jpeg_baseline(w: u32, h: u32) -> Vec<u8> {
    encode(&DynamicImage::ImageRgb8(busy_rgb(w, h)), ImageFormat::Jpeg)
}

/// JPEG through `jpeg-encoder`, which can do progressive frames, restart markers,
/// grey and CMYK, and extra APP segments.
pub(crate) fn jpeg_custom(
    w: u16,
    h: u16,
    color: jpeg_encoder::ColorType,
    progressive: bool,
    restart_interval: Option<u16>,
    exif: Option<&[u8]>,
    app_padding: usize,
) -> Vec<u8> {
    let channels = match color {
        jpeg_encoder::ColorType::Luma => 1,
        jpeg_encoder::ColorType::Cmyk => 4,
        _ => 3,
    };
    let data: Vec<u8> = (0..usize::from(w) * usize::from(h) * channels).map(|i| (i * 31 % 251) as u8).collect();
    let mut out = Vec::new();
    {
        let mut encoder = jpeg_encoder::Encoder::new(&mut out, 80);
        encoder.set_progressive(progressive);
        if let Some(interval) = restart_interval {
            encoder.set_restart_interval(interval);
        }
        if let Some(exif) = exif {
            encoder.add_exif_metadata(exif).expect("exif");
        }
        let mut left = app_padding;
        while left > 0 {
            let n = left.min(60_000);
            encoder.add_app_segment(7, vec![0xAB; n]).expect("app segment");
            left -= n;
        }
        encoder.encode(&data, w, h, color).expect("jpeg");
    }
    out
}

/// A TIFF block holding only the orientation tag, little- or big-endian.
pub(crate) fn tiff_orientation(value: u16, little_endian: bool) -> Vec<u8> {
    let mut t = Vec::new();
    let w16 = |t: &mut Vec<u8>, v: u16| t.extend_from_slice(&if little_endian { v.to_le_bytes() } else { v.to_be_bytes() });
    let w32 = |t: &mut Vec<u8>, v: u32| t.extend_from_slice(&if little_endian { v.to_le_bytes() } else { v.to_be_bytes() });
    t.extend_from_slice(if little_endian { b"II" } else { b"MM" });
    w16(&mut t, 42);
    w32(&mut t, 8); // offset of the first IFD
    w16(&mut t, 2); // two entries
    // 0x010F Make, ASCII, 4 bytes inline.
    w16(&mut t, 0x010F);
    w16(&mut t, 2);
    w32(&mut t, 4);
    t.extend_from_slice(b"ebc\0");
    // 0x0112 Orientation, SHORT, 1 value.
    w16(&mut t, 0x0112);
    w16(&mut t, 3);
    w32(&mut t, 1);
    w16(&mut t, value);
    w16(&mut t, 0);
    w32(&mut t, 0); // no next IFD
    t
}

/// RIFF container around the given chunks.
pub(crate) fn riff_webp(chunks: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
    let mut body = b"WEBP".to_vec();
    for (kind, data) in chunks {
        body.extend_from_slice(*kind);
        body.extend_from_slice(&(data.len() as u32).to_le_bytes());
        body.extend_from_slice(data);
        if data.len() % 2 == 1 {
            body.push(0);
        }
    }
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    out
}

/// The first bytes of a lossy `VP8 ` key frame with the given size and scale bits.
pub(crate) fn vp8_frame_header(width: u16, height: u16, scale_bits: u16) -> Vec<u8> {
    let mut d = vec![0x10, 0x02, 0x00, 0x9D, 0x01, 0x2A]; // frame tag (key frame), start code
    d.extend_from_slice(&(width | scale_bits << 14).to_le_bytes());
    d.extend_from_slice(&(height | scale_bits << 14).to_le_bytes());
    d.extend_from_slice(&[0u8; 24]);
    d
}

/// The payload of a `VP8X` chunk.
pub(crate) fn vp8x_payload(flags: u8, width: u32, height: u32) -> Vec<u8> {
    let mut d = vec![flags, 0, 0, 0];
    d.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
    d.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
    d
}

/// One ISO-BMFF box.
pub(crate) fn bx(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
    out.extend_from_slice(kind);
    out.extend_from_slice(payload);
    out
}

/// A FullBox: version, 24-bit flags, payload.
pub(crate) fn full(kind: &[u8; 4], version: u8, flags: u32, payload: &[u8]) -> Vec<u8> {
    let mut p = vec![version];
    p.extend_from_slice(&flags.to_be_bytes()[1..]);
    p.extend_from_slice(payload);
    bx(kind, &p)
}

pub(crate) fn ispe(w: u32, h: u32) -> Vec<u8> {
    full(b"ispe", 0, 0, &[w.to_be_bytes(), h.to_be_bytes()].concat())
}

/// A minimal AVIF/HEIF header: `ftyp`, then `meta` with `pitm`, `iprp { ipco, ipma }`.
/// `properties` are the boxes of `ipco`; `primary_props` the 1-based indices
/// associated with the primary item (empty: no `ipma` at all).
pub(crate) fn bmff(brands: &[&[u8; 4]], properties: &[Vec<u8>], primary_props: &[u8]) -> Vec<u8> {
    let mut ftyp = brands[0].to_vec();
    ftyp.extend_from_slice(&0u32.to_be_bytes());
    for brand in brands {
        ftyp.extend_from_slice(*brand);
    }
    let mut meta = full(b"hdlr", 0, 0, &[&[0u8; 4][..], b"pict", &[0u8; 13][..]].concat());
    meta.extend(full(b"pitm", 0, 0, &1u16.to_be_bytes()));
    let mut iprp = bx(b"ipco", &properties.concat());
    if !primary_props.is_empty() {
        let mut ipma = 1u32.to_be_bytes().to_vec(); // one entry
        ipma.extend_from_slice(&1u16.to_be_bytes()); // item 1
        ipma.push(primary_props.len() as u8);
        ipma.extend_from_slice(primary_props);
        iprp.extend(full(b"ipma", 0, 0, &ipma));
    }
    meta.extend(bx(b"iprp", &iprp));
    let mut out = bx(b"ftyp", &ftyp);
    out.extend(full(b"meta", 0, 0, &meta));
    out.extend(bx(b"mdat", &[0u8; 32]));
    out
}

/// Decodes encoded bytes to a `Pixmap` through the `image` crate.
pub(crate) fn decode(bytes: &[u8]) -> Pixmap {
    let rgba = image::load_from_memory(bytes).expect("decodes").to_rgba8();
    Pixmap::from_rgba8(rgba.width(), rgba.height(), rgba.into_raw()).expect("pixmap")
}

pub(crate) fn fixture(path: &str) -> String {
    let full = format!("{}/tests/fixtures/{path}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{full}: {e}"))
}
