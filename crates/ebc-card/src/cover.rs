//! Decoding the cover and preparing it for the print file.
//!
//! * JPEG, PNG and WebP are decoded with the `image` crate; the bytes are hostile input,
//!   so dimensions are checked before any pixel is decoded and a panic inside a decoder
//!   is contained (where panics can be caught at all).
//! * A baseline or progressive 8-bit RGB or grey JPEG whose pixels need no change is
//!   embedded as it is; everything else (PNG, WebP, CMYK or rotated JPEG, raw pixels)
//!   becomes raw sRGB with any transparency composited on white.
//! * A cover with more than about 350 ppi at its printed size is resampled to 300 ppi; a
//!   cover is never upscaled, low resolution is reported instead.
//!
//! Not done: embedded ICC profiles are ignored (the pixels are treated as sRGB, with a
//! warning), and nothing is converted to the output-intent colour space.

use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use ebc_layout::{PrintProfile, Resolution, effective_ppi};
use ebc_scene::{ImageData, PixelFormat, Pixmap, Rect, Size};
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage};

use crate::error::{CardError, CardWarning};
use crate::request::CoverInput;

/// Largest cover side accepted, in pixels.
pub const MAX_COVER_SIDE_PX: u32 = 16_384;
/// Largest cover accepted, in pixels (a 6000 × 9000 px scan is 54 million).
pub const MAX_COVER_PIXELS: u64 = 100_000_000;
/// Largest encoded cover file accepted, in bytes (64 MiB).
pub const MAX_COVER_BYTES: usize = 64 * 1024 * 1024;
/// Above this effective resolution a cover is resampled before it is embedded.
pub const MAX_EMBED_PPI: f64 = 350.0;
/// The resolution a resampled cover gets.
pub const RESAMPLE_PPI: f64 = 300.0;
/// Longest side of the copy used to compute colours.
const ANALYSIS_SIDE_PX: u32 = 256;

/// A decoded cover.
pub(crate) struct DecodedCover {
    /// The pixels as displayed (orientation applied), transparency composited on white.
    rgb: RgbImage,
    /// The original file and its component count, when it can be embedded untouched.
    jpeg: Option<(Arc<[u8]>, u8)>,
    /// The file carried an ICC profile (which is ignored).
    has_icc_profile: bool,
}

impl DecodedCover {
    /// Decodes the cover of a request; `None` for [`CoverInput::None`].
    pub(crate) fn decode(input: &CoverInput) -> Result<Option<Self>, CardError> {
        match input {
            CoverInput::None => Ok(None),
            CoverInput::Pixels { pixmap, .. } => Ok(Some(Self::from_pixmap(pixmap)?)),
            CoverInput::Encoded { bytes, .. } => {
                // The decoders are fuzzed, but they are third-party code fed with
                // untrusted bytes: a panic must not take the caller down.
                catch_unwind(AssertUnwindSafe(|| Self::from_bytes(bytes)))
                    .unwrap_or_else(|_| {
                        Err(CardError::CoverDecode("the image decoder failed".to_owned()))
                    })
                    .map(Some)
            }
        }
    }

    fn from_pixmap(pixmap: &Pixmap) -> Result<Self, CardError> {
        let (width, height) = (pixmap.width(), pixmap.height());
        check_size(width, height)?;
        let mut rgb = Vec::with_capacity(pixmap.data().len() / 4 * 3);
        for px in pixmap.data().chunks_exact(4) {
            rgb.extend_from_slice(&on_white([px[0], px[1], px[2], px[3]]));
        }
        let rgb = RgbImage::from_raw(width, height, rgb)
            .ok_or_else(|| CardError::CoverDecode("inconsistent pixel buffer".to_owned()))?;
        Ok(Self { rgb, jpeg: None, has_icc_profile: false })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, CardError> {
        let failed = |e: image::ImageError| CardError::CoverDecode(e.to_string());
        if bytes.len() > MAX_COVER_BYTES {
            return Err(CardError::CoverDecode(format!(
                "the file has {} bytes, more than the {MAX_COVER_BYTES} accepted",
                bytes.len()
            )));
        }
        let format = image::guess_format(bytes).map_err(failed)?;
        if !matches!(format, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
            return Err(CardError::CoverDecode(format!(
                "{format:?} images are not supported (JPEG, PNG and WebP are)"
            )));
        }
        // The size the header announces is checked before a decoder sees the file …
        if let Some((width, height)) = announced_size(bytes, format) {
            check_size(width, height)?;
        }
        // … and again as the decoder reads it (the only check for WebP).
        let (width, height) = ImageReader::with_format(Cursor::new(bytes), format)
            .into_dimensions()
            .map_err(failed)?;
        check_size(width, height)?;

        let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_COVER_SIDE_PX);
        limits.max_image_height = Some(MAX_COVER_SIDE_PX);
        limits.max_alloc = Some(MAX_COVER_PIXELS * 8);
        reader.limits(limits);
        let mut decoder = reader.into_decoder().map_err(failed)?;
        let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
        let has_icc_profile = decoder.icc_profile().ok().flatten().is_some();
        let mut image = DynamicImage::from_decoder(decoder).map_err(failed)?;
        image.apply_orientation(orientation);
        check_size(image.width(), image.height())?;

        let untouched = format == ImageFormat::Jpeg && orientation == Orientation::NoTransforms;
        let jpeg = untouched
            .then(|| embeddable_jpeg_components(bytes))
            .flatten()
            .map(|components| (Arc::<[u8]>::from(bytes), components));
        Ok(Self { rgb: flatten(image), jpeg, has_icc_profile })
    }

    /// Pixel dimensions as displayed.
    pub(crate) fn size_px(&self) -> (u32, u32) {
        self.rgb.dimensions()
    }

    /// A small opaque copy for colour computations (palette, edge colours).
    pub(crate) fn analysis(&self) -> Pixmap {
        self.scaled_pixmap(ANALYSIS_SIDE_PX)
    }

    /// An opaque copy whose longest side is at most `max_side` pixels (never enlarged).
    pub(crate) fn scaled_pixmap(&self, max_side: u32) -> Pixmap {
        let (w, h) = self.rgb.dimensions();
        let longest = w.max(h);
        let small;
        let source = if longest > max_side.max(1) {
            let scale = f64::from(max_side.max(1)) / f64::from(longest);
            let nw = ((f64::from(w) * scale).round() as u32).max(1);
            let nh = ((f64::from(h) * scale).round() as u32).max(1);
            small = image::imageops::thumbnail(&self.rgb, nw, nh);
            &small
        } else {
            &self.rgb
        };
        let mut rgba = Vec::with_capacity(source.as_raw().len() / 3 * 4);
        for px in source.as_raw().chunks_exact(3) {
            rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
        Pixmap::from_rgba8(source.width(), source.height(), rgba)
            .unwrap_or_else(|| Pixmap::filled(1, 1, [255, 255, 255, 255]))
    }

    /// The image data to embed for a cover printed at `placed` millimetres, with the
    /// resolution warnings that go with it.
    pub(crate) fn into_image_data(
        self,
        placed: Size,
        profile: &PrintProfile,
        warnings: &mut Vec<CardWarning>,
    ) -> ImageData {
        let (width, height) = self.rgb.dimensions();
        let ppi = effective_ppi((width, height), Rect::new(0.0, 0.0, placed.w, placed.h));
        match profile.judge_ppi(ppi) {
            Resolution::TooLow => warnings.push(CardWarning::LowResolution {
                ppi,
                threshold_ppi: profile.min_ppi,
                below_minimum: true,
            }),
            Resolution::Low => warnings.push(CardWarning::LowResolution {
                ppi,
                threshold_ppi: profile.warn_ppi(),
                below_minimum: false,
            }),
            Resolution::Good | Resolution::Acceptable => {}
        }
        if self.has_icc_profile {
            warnings.push(CardWarning::CoverProfileIgnored);
        }

        if ppi > MAX_EMBED_PPI {
            let scale = RESAMPLE_PPI / ppi;
            let nw = ((f64::from(width) * scale).round() as u32).max(1);
            let nh = ((f64::from(height) * scale).round() as u32).max(1);
            let resized =
                image::imageops::resize(&self.rgb, nw, nh, image::imageops::FilterType::CatmullRom);
            warnings.push(CardWarning::CoverDownsampled { from_ppi: ppi, to_ppi: RESAMPLE_PPI });
            return raw_rgb(resized);
        }
        match self.jpeg {
            Some((bytes, components)) => {
                ImageData::Jpeg { bytes, width_px: width, height_px: height, components }
            }
            None => raw_rgb(self.rgb),
        }
    }
}

fn raw_rgb(image: RgbImage) -> ImageData {
    let (width_px, height_px) = image.dimensions();
    ImageData::Raw {
        pixels: Arc::from(image.into_raw()),
        width_px,
        height_px,
        format: PixelFormat::Rgb8,
    }
}

fn check_size(width: u32, height: u32) -> Result<(), CardError> {
    if width == 0 || height == 0 {
        return Err(CardError::CoverDecode("the image has a zero dimension".to_owned()));
    }
    let too_large = width > MAX_COVER_SIDE_PX
        || height > MAX_COVER_SIDE_PX
        || u64::from(width) * u64::from(height) > MAX_COVER_PIXELS;
    if too_large {
        return Err(CardError::CoverTooLarge {
            width,
            height,
            max_side: MAX_COVER_SIDE_PX,
            max_pixels: MAX_COVER_PIXELS,
        });
    }
    Ok(())
}

/// One non-premultiplied RGBA pixel composited on white.
fn on_white([r, g, b, a]: [u8; 4]) -> [u8; 3] {
    let blend = |c: u8| {
        let (c, a) = (u32::from(c), u32::from(a));
        // 0..=255 by construction.
        ((c * a + 255 * (255 - a) + 127) / 255) as u8
    };
    if a == 255 { [r, g, b] } else { [blend(r), blend(g), blend(b)] }
}

/// The image as 8-bit sRGB, transparency composited on white.
fn flatten(image: DynamicImage) -> RgbImage {
    if !image.color().has_alpha() {
        return image.into_rgb8();
    }
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut rgb = Vec::with_capacity(rgba.as_raw().len() / 4 * 3);
    for px in rgba.as_raw().chunks_exact(4) {
        rgb.extend_from_slice(&on_white([px[0], px[1], px[2], px[3]]));
    }
    // The buffer has exactly width × height × 3 bytes; the fallback is unreachable.
    RgbImage::from_raw(width, height, rgb).unwrap_or_else(|| RgbImage::new(width, height))
}

/// The pixel size a PNG or JPEG header announces, read without any decoder: the IHDR
/// chunk of a PNG, the first frame header of a JPEG. `None` when it cannot be found
/// (the decoder then decides) and for other formats.
fn announced_size(data: &[u8], format: ImageFormat) -> Option<(u32, u32)> {
    match format {
        ImageFormat::Png => {
            let ihdr = data.get(8..24)?;
            if &ihdr[4..8] != b"IHDR" {
                return None;
            }
            let be = |at: usize| {
                u32::from_be_bytes([ihdr[at], ihdr[at + 1], ihdr[at + 2], ihdr[at + 3]])
            };
            Some((be(8), be(12)))
        }
        ImageFormat::Jpeg => jpeg_frame(data).map(|frame| (frame.width, frame.height)),
        _ => None,
    }
}

/// The first frame header of a JPEG file.
struct JpegFrame {
    /// The `SOFn` marker byte (0xC0 = baseline, 0xC2 = progressive, …).
    marker: u8,
    precision: u8,
    width: u32,
    height: u32,
    components: u8,
}

/// Walks the marker segments of a JPEG file up to its first frame header.
fn jpeg_frame(data: &[u8]) -> Option<JpegFrame> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut pos = 2usize;
    loop {
        if *data.get(pos)? != 0xFF {
            return None;
        }
        while *data.get(pos)? == 0xFF {
            pos += 1;
        }
        let marker = *data.get(pos)?;
        pos += 1;
        match marker {
            0x01 | 0xD0..=0xD7 => continue,
            0xD8..=0xDA => return None,
            _ => {}
        }
        let length = usize::from(u16::from_be_bytes([*data.get(pos)?, *data.get(pos + 1)?]));
        if length < 2 {
            return None;
        }
        let segment = data.get(pos + 2..pos + length)?;
        pos += length;
        // SOF0–SOF15, except DHT (C4), JPG (C8) and DAC (CC), which are not frames.
        if matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            let be = |at: usize| {
                Some(u32::from(u16::from_be_bytes([*segment.get(at)?, *segment.get(at + 1)?])))
            };
            return Some(JpegFrame {
                marker,
                precision: *segment.first()?,
                height: be(1)?,
                width: be(3)?,
                components: *segment.get(5)?,
            });
        }
    }
}

/// The number of components (1 or 3) of a JPEG file that PDF can embed untouched and
/// that needs no colour change: baseline, extended-sequential or progressive Huffman
/// coding, 8-bit samples, grey or three components. `None` for anything else (CMYK,
/// lossless, arithmetic coding, 12-bit, a corrupt marker stream).
fn embeddable_jpeg_components(data: &[u8]) -> Option<u8> {
    let frame = jpeg_frame(data)?;
    (matches!(frame.marker, 0xC0..=0xC2)
        && frame.precision == 8
        && matches!(frame.components, 1 | 3))
    .then_some(frame.components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparency_is_composited_on_white() {
        assert_eq!(on_white([10, 20, 30, 255]), [10, 20, 30]);
        assert_eq!(on_white([10, 20, 30, 0]), [255, 255, 255]);
        assert_eq!(on_white([0, 0, 0, 128]), [127, 127, 127]);
    }

    #[test]
    fn size_limits() {
        assert!(check_size(6000, 9000).is_ok());
        assert!(matches!(check_size(0, 10), Err(CardError::CoverDecode(_))));
        assert!(matches!(check_size(20_000, 10), Err(CardError::CoverTooLarge { .. })));
        assert!(matches!(check_size(12_000, 12_000), Err(CardError::CoverTooLarge { .. })));
    }

    #[test]
    fn jpeg_headers() {
        // SOI, SOF0 with 3 components, 8 bit.
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 8, 0, 4, 0, 4, 3];
        jpeg.extend_from_slice(&[0; 9]);
        assert_eq!(embeddable_jpeg_components(&jpeg), Some(3));
        let mut cmyk = jpeg.clone();
        cmyk[11] = 4;
        assert_eq!(embeddable_jpeg_components(&cmyk), None);
        let mut twelve_bit = jpeg.clone();
        twelve_bit[6] = 12;
        assert_eq!(embeddable_jpeg_components(&twelve_bit), None);
        let mut lossless = jpeg.clone();
        lossless[3] = 0xC3;
        assert_eq!(embeddable_jpeg_components(&lossless), None);
        assert_eq!(embeddable_jpeg_components(&jpeg[..7]), None);
        assert_eq!(embeddable_jpeg_components(b"not a jpeg"), None);
        assert_eq!(embeddable_jpeg_components(&[]), None);
        // The announced size is read from the same header: 4 × 4 px here.
        assert_eq!(announced_size(&jpeg, ImageFormat::Jpeg), Some((4, 4)));
        assert_eq!(announced_size(&lossless, ImageFormat::Jpeg), Some((4, 4)));
        assert_eq!(announced_size(&jpeg[..7], ImageFormat::Jpeg), None);
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend_from_slice(&[0, 0, 0x75, 0x30, 0, 0, 0, 7]);
        assert_eq!(announced_size(&png, ImageFormat::Png), Some((30_000, 7)));
        assert_eq!(announced_size(&png[..20], ImageFormat::Png), None);
        assert_eq!(announced_size(&png, ImageFormat::WebP), None);
    }

    #[test]
    fn pixmaps_are_scaled_down_but_never_up() {
        let big = Pixmap::filled(600, 900, [200, 10, 10, 255]);
        let cover = DecodedCover::from_pixmap(&big).expect("valid");
        let small = cover.analysis();
        assert_eq!((small.width(), small.height()), (171, 256));
        assert_eq!(small.get(5, 5), Some([200, 10, 10, 255]));
        let tiny =
            DecodedCover::from_pixmap(&Pixmap::filled(8, 12, [0, 0, 0, 255])).expect("valid");
        assert_eq!((tiny.analysis().width(), tiny.analysis().height()), (8, 12));
    }
}
