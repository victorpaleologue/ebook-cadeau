//! Image XObjects: JPEG passthrough, Flate-compressed raw pixels, soft masks and ICC
//! colour spaces.

use std::borrow::Cow;
use std::collections::BTreeSet;

use ebc_scene::{ImageData, ImageId, ImageResource, PdfStandard, PixelFormat};
use pdf_writer::types::Predictor;
use pdf_writer::{Chunk, Filter, Finish, Ref};

use crate::error::PdfError;
use crate::writer::{RefAllocator, deflate, stream_len};
use crate::{icc, jpeg};

/// The sRGB profile used for RGB content without a profile of its own when the target
/// standard is colour-managed. ICC version 2, so it is valid in every PDF version.
pub(crate) static SRGB_PROFILE: &[u8] = include_bytes!("../../../assets/icc/sRGB-v2-micro.icc");

/// `true` when the standard allows soft masks (live transparency).
pub(crate) fn allows_transparency(standard: PdfStandard) -> bool {
    matches!(standard, PdfStandard::X4 | PdfStandard::Plain)
}

/// Highest ICC major version that may be embedded for the target PDF version
/// (PDF 1.3 predates ICC version 4).
pub(crate) fn max_icc_version(standard: PdfStandard) -> u8 {
    match standard {
        PdfStandard::X1a2001 | PdfStandard::X3_2002 => 2,
        PdfStandard::X4 | PdfStandard::Plain => 4,
    }
}

/// The distinct ICC profiles of the document, each written once.
#[derive(Default)]
pub(crate) struct IccPool<'a> {
    profiles: Vec<(&'a [u8], u8)>,
}

impl<'a> IccPool<'a> {
    /// Registers a profile with `components` channels and returns its index.
    pub(crate) fn intern(&mut self, profile: &'a [u8], components: u8) -> usize {
        if let Some(index) = self
            .profiles
            .iter()
            .position(|(p, n)| *n == components && (std::ptr::eq(*p, profile) || *p == profile))
        {
            return index;
        }
        self.profiles.push((profile, components));
        self.profiles.len() - 1
    }

    /// Writes the profiles whose index is in `used` as ICC streams and returns the
    /// references by index (`None` for profiles nothing refers to).
    pub(crate) fn write(
        &self,
        chunk: &mut Chunk,
        refs: &mut RefAllocator,
        compress: bool,
        used: &BTreeSet<usize>,
    ) -> Result<Vec<Option<Ref>>, PdfError> {
        let mut out = Vec::with_capacity(self.profiles.len());
        for (index, (profile, components)) in self.profiles.iter().enumerate() {
            if !used.contains(&index) {
                out.push(None);
                continue;
            }
            let id = refs.next()?;
            let data: Cow<'_, [u8]> =
                if compress { Cow::Owned(deflate(profile)) } else { Cow::Borrowed(profile) };
            stream_len(data.len(), "an ICC profile")?;
            let mut stream = chunk.icc_profile(id, &data);
            // `n` only accepts 1, 3 or 4, which `icc::inspect` guarantees.
            stream.n(i32::from(*components));
            if compress {
                stream.filter(Filter::FlateDecode);
            }
            stream.finish();
            out.push(Some(id));
        }
        Ok(out)
    }
}

/// The colour space of an image XObject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageSpace {
    DeviceGray,
    DeviceRgb,
    DeviceCmyk,
    /// `ICCBased`, with the index of the profile in the [`IccPool`].
    Icc(usize),
}

/// An image ready to be written.
#[derive(Debug)]
pub(crate) struct PreparedImage<'a> {
    id: ImageId,
    /// Name of the image in page resource dictionaries, e.g. `Im0`.
    pub(crate) resource_name: Vec<u8>,
    width: i32,
    height: i32,
    /// The stream data, already encoded with `filter`.
    data: Cow<'a, [u8]>,
    filter: Option<Filter>,
    /// Number of colour channels in `data` when it was PNG-predicted before deflating.
    predicted: Option<u8>,
    pub(crate) space: ImageSpace,
    /// Adobe-style CMYK JPEG: samples are stored inverted.
    pub(crate) inverted: bool,
    /// 8-bit alpha channel written as a soft mask.
    mask: Option<Encoded>,
}

/// Lossless sample data ready for a stream.
#[derive(Debug)]
struct Encoded {
    data: Vec<u8>,
    /// `true` when `data` is deflated.
    deflated: bool,
    /// `true` when the rows were PNG-predicted before deflating.
    predicted: bool,
}

impl Encoded {
    /// Encodes rows of `row_len` bytes with `channels` bytes per pixel: deflated, with
    /// PNG row prediction when that makes the stream smaller, or stored when `compress`
    /// is off.
    fn new(raw: &[u8], row_len: usize, channels: usize, compress: bool) -> Self {
        if !compress {
            return Self { data: raw.to_vec(), deflated: false, predicted: false };
        }
        let plain = deflate(raw);
        let predicted = deflate(&png_predict(raw, row_len, channels));
        if predicted.len() < plain.len() {
            Self { data: predicted, deflated: true, predicted: true }
        } else {
            Self { data: plain, deflated: true, predicted: false }
        }
    }
}

/// Applies PNG row filters (the `/Predictor 15` encoding of `FlateDecode`): every row is
/// prefixed with the filter type that minimises the sum of absolute residuals, the usual
/// heuristic of PNG encoders. Photographs and gradients deflate much better this way.
fn png_predict(raw: &[u8], row_len: usize, bpp: usize) -> Vec<u8> {
    if row_len == 0 || bpp == 0 {
        return raw.to_vec();
    }
    let rows = raw.len() / row_len;
    let mut out = Vec::with_capacity(raw.len() + rows);
    let zero_row = vec![0u8; row_len];
    let mut candidates: [Vec<u8>; 5] = std::array::from_fn(|_| vec![0u8; row_len]);
    for (index, row) in raw.chunks_exact(row_len).enumerate() {
        let above =
            if index == 0 { &zero_row[..] } else { &raw[(index - 1) * row_len..index * row_len] };
        for i in 0..row_len {
            let x = row[i];
            let a = if i >= bpp { row[i - bpp] } else { 0 };
            let b = above[i];
            let c = if i >= bpp { above[i - bpp] } else { 0 };
            candidates[0][i] = x;
            candidates[1][i] = x.wrapping_sub(a);
            candidates[2][i] = x.wrapping_sub(b);
            candidates[3][i] = x.wrapping_sub(((u16::from(a) + u16::from(b)) / 2) as u8);
            candidates[4][i] = x.wrapping_sub(paeth(a, b, c));
        }
        let cost = |bytes: &[u8]| -> u64 {
            bytes.iter().map(|&v| u64::from((v as i8).unsigned_abs())).sum()
        };
        let mut best = 0;
        let mut best_cost = cost(&candidates[0]);
        for (kind, candidate) in candidates.iter().enumerate().skip(1) {
            let c = cost(candidate);
            if c < best_cost {
                best = kind;
                best_cost = c;
            }
        }
        out.push(best as u8);
        out.extend_from_slice(&candidates[best]);
    }
    out
}

/// The Paeth predictor of the PNG specification.
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (i16::from(a), i16::from(b), i16::from(c));
    let p = ia + ib - ic;
    let (pa, pb, pc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

impl<'a> PreparedImage<'a> {
    /// Validates `resource` against the target `standard` and encodes its pixels.
    pub(crate) fn prepare(
        resource: &'a ImageResource,
        index: usize,
        standard: PdfStandard,
        compress: bool,
        pool: &mut IccPool<'a>,
    ) -> Result<Self, PdfError> {
        let id = resource.id;
        let invalid = |reason: String| PdfError::InvalidImage { image: id, reason };

        let (width_px, height_px) = resource.data.dimensions_px();
        let (Ok(width), Ok(height)) = (i32::try_from(width_px), i32::try_from(height_px)) else {
            return Err(invalid("the dimensions are too large".into()));
        };
        if width == 0 || height == 0 {
            return Err(invalid("the image has a zero dimension".into()));
        }
        let width_usize = usize::try_from(width_px).unwrap_or(usize::MAX);

        let channels: u8;
        let data: Cow<'a, [u8]>;
        let filter;
        let mut predicted = None;
        let mut inverted = false;
        let mut mask = None;

        match &resource.data {
            ImageData::Jpeg { bytes, components, .. } => {
                let info = jpeg::sniff(bytes).map_err(|e| invalid(e.into()))?;
                if (info.width, info.height) != (width_px, height_px) {
                    return Err(invalid(format!(
                        "the JPEG is {}x{} px but the resource says {width_px}x{height_px} px",
                        info.width, info.height
                    )));
                }
                if info.components != *components {
                    return Err(invalid(format!(
                        "the JPEG has {} components but the resource says {components}",
                        info.components
                    )));
                }
                if !matches!(components, 1 | 3 | 4) {
                    return Err(invalid(format!(
                        "JPEG images with {components} components are not supported"
                    )));
                }
                channels = *components;
                // Adobe applications store CMYK (and YCCK) samples inverted and flag the
                // file with an APP14 marker; PDF consumers do not undo that by themselves.
                inverted = channels == 4 && info.adobe_transform.is_some();
                data = Cow::Borrowed(&bytes[..]);
                filter = Some(Filter::DctDecode);
            }
            ImageData::Raw { pixels, format, .. } => {
                let expected = usize::try_from(width_px)
                    .ok()
                    .and_then(|w| w.checked_mul(usize::try_from(height_px).ok()?))
                    .and_then(|n| n.checked_mul(format.bytes_per_pixel()));
                if expected != Some(pixels.len()) {
                    return Err(invalid(format!(
                        "{} bytes of pixel data do not match {width_px}x{height_px} px {format:?}",
                        pixels.len()
                    )));
                }
                let raw: Cow<'a, [u8]> = match format {
                    PixelFormat::Gray8 => {
                        channels = 1;
                        Cow::Borrowed(&pixels[..])
                    }
                    PixelFormat::Rgb8 => {
                        channels = 3;
                        Cow::Borrowed(&pixels[..])
                    }
                    PixelFormat::Cmyk8 => {
                        channels = 4;
                        Cow::Borrowed(&pixels[..])
                    }
                    PixelFormat::Rgba8 => {
                        channels = 3;
                        let opaque = pixels.chunks_exact(4).all(|p| p[3] == 255);
                        if !opaque && !allows_transparency(standard) {
                            return Err(PdfError::TransparencyNotAllowed { standard, image: id });
                        }
                        let mut rgb = Vec::with_capacity(pixels.len() / 4 * 3);
                        for p in pixels.chunks_exact(4) {
                            rgb.extend_from_slice(&p[..3]);
                        }
                        if !opaque {
                            let alpha: Vec<u8> = pixels.chunks_exact(4).map(|p| p[3]).collect();
                            mask = Some(Encoded::new(&alpha, width_usize, 1, compress));
                        }
                        Cow::Owned(rgb)
                    }
                };
                if compress {
                    let row_len = width_usize * usize::from(channels);
                    let encoded = Encoded::new(&raw, row_len, usize::from(channels), true);
                    predicted = encoded.predicted.then_some(channels);
                    data = Cow::Owned(encoded.data);
                    filter = Some(Filter::FlateDecode);
                } else {
                    data = raw;
                    filter = None;
                }
            }
        }

        if channels == 3 && standard == PdfStandard::X1a2001 {
            return Err(PdfError::RgbNotAllowed { standard, location: format!("image {}", id.0) });
        }
        let space = match &resource.icc_profile {
            Some(profile) => {
                if standard == PdfStandard::X1a2001 {
                    return Err(PdfError::IccNotAllowed { standard, image: id });
                }
                let context = || format!("image {}", id.0);
                let info = icc::inspect(profile)
                    .map_err(|reason| PdfError::InvalidIccProfile { context: context(), reason })?;
                if info.components != channels {
                    return Err(PdfError::InvalidIccProfile {
                        context: context(),
                        reason: "the profile's colour space does not match the image's channels",
                    });
                }
                if info.version_major > max_icc_version(standard) {
                    return Err(PdfError::InvalidIccProfile {
                        context: context(),
                        reason: "the ICC version is newer than the target PDF version supports",
                    });
                }
                ImageSpace::Icc(pool.intern(profile, channels))
            }
            None => match (channels, standard) {
                (1, _) => ImageSpace::DeviceGray,
                (4, _) => ImageSpace::DeviceCmyk,
                (_, PdfStandard::Plain) => ImageSpace::DeviceRgb,
                // Untagged RGB is sRGB by contract; colour-managed standards need it said.
                _ => ImageSpace::Icc(pool.intern(SRGB_PROFILE, 3)),
            },
        };

        stream_len(data.len(), "an image")?;
        Ok(Self {
            id,
            resource_name: format!("Im{index}").into_bytes(),
            width,
            height,
            data,
            filter,
            predicted,
            space,
            inverted,
            mask,
        })
    }

    /// Writes the image XObject (and its soft mask) and returns the image's reference.
    pub(crate) fn write(
        &self,
        chunk: &mut Chunk,
        refs: &mut RefAllocator,
        icc_refs: &[Option<Ref>],
    ) -> Result<Ref, PdfError> {
        let image_ref = refs.next()?;
        let mask_ref = match &self.mask {
            Some(alpha) => {
                stream_len(alpha.data.len(), "an image mask")?;
                let mask_ref = refs.next()?;
                let mut mask = chunk.image_xobject(mask_ref, &alpha.data);
                mask.width(self.width).height(self.height).bits_per_component(8);
                mask.color_space().device_gray();
                if alpha.deflated {
                    mask.filter(Filter::FlateDecode);
                }
                if alpha.predicted {
                    mask.decode_parms()
                        .predictor(Predictor::PngOptimum)
                        .colors(1)
                        .bits_per_component(8)
                        .columns(self.width);
                }
                mask.finish();
                Some(mask_ref)
            }
            None => None,
        };

        let mut image = chunk.image_xobject(image_ref, &self.data);
        image.width(self.width).height(self.height).bits_per_component(8);
        match self.space {
            ImageSpace::DeviceGray => image.color_space().device_gray(),
            ImageSpace::DeviceRgb => image.color_space().device_rgb(),
            ImageSpace::DeviceCmyk => image.color_space().device_cmyk(),
            ImageSpace::Icc(index) => match icc_refs.get(index).copied().flatten() {
                Some(profile_ref) => image.color_space().icc_based(profile_ref),
                None => {
                    return Err(PdfError::InvalidImage {
                        image: self.id,
                        reason: "internal error: ICC profile was not written".into(),
                    });
                }
            },
        }
        if self.inverted {
            image.decode([1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
        }
        if let Some(mask_ref) = mask_ref {
            image.s_mask(mask_ref);
        }
        if let Some(filter) = self.filter {
            image.filter(filter);
        }
        if let Some(channels) = self.predicted {
            // `channels` is 1, 3 or 4 by construction.
            image
                .decode_parms()
                .predictor(Predictor::PngOptimum)
                .colors(i32::from(channels))
                .bits_per_component(8)
                .columns(self.width);
        }
        image.finish();
        Ok(image_ref)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn raw(format: PixelFormat, width: u32, height: u32, pixels: Vec<u8>) -> ImageResource {
        ImageResource {
            id: ImageId(4),
            data: ImageData::Raw {
                pixels: Arc::from(pixels),
                width_px: width,
                height_px: height,
                format,
            },
            icc_profile: None,
        }
    }

    fn prepare(
        resource: &ImageResource,
        standard: PdfStandard,
    ) -> Result<(ImageSpace, bool, usize), PdfError> {
        let mut pool = IccPool::default();
        let image = PreparedImage::prepare(resource, 0, standard, true, &mut pool)?;
        Ok((image.space, image.mask.is_some(), pool.profiles.len()))
    }

    #[test]
    fn colour_space_follows_the_standard() {
        let rgb = raw(PixelFormat::Rgb8, 2, 1, vec![0; 6]);
        assert_eq!(prepare(&rgb, PdfStandard::Plain), Ok((ImageSpace::DeviceRgb, false, 0)));
        assert_eq!(prepare(&rgb, PdfStandard::X4), Ok((ImageSpace::Icc(0), false, 1)));
        assert_eq!(prepare(&rgb, PdfStandard::X3_2002), Ok((ImageSpace::Icc(0), false, 1)));
        assert!(matches!(
            prepare(&rgb, PdfStandard::X1a2001),
            Err(PdfError::RgbNotAllowed { standard: PdfStandard::X1a2001, .. })
        ));

        let gray = raw(PixelFormat::Gray8, 2, 1, vec![0; 2]);
        let cmyk = raw(PixelFormat::Cmyk8, 2, 1, vec![0; 8]);
        for standard in
            [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4]
        {
            assert_eq!(prepare(&gray, standard), Ok((ImageSpace::DeviceGray, false, 0)));
            assert_eq!(prepare(&cmyk, standard), Ok((ImageSpace::DeviceCmyk, false, 0)));
        }
    }

    #[test]
    fn alpha_becomes_a_mask_only_where_allowed() {
        let translucent = raw(PixelFormat::Rgba8, 1, 2, vec![1, 2, 3, 255, 4, 5, 6, 128]);
        assert_eq!(prepare(&translucent, PdfStandard::X4), Ok((ImageSpace::Icc(0), true, 1)));
        assert_eq!(prepare(&translucent, PdfStandard::Plain), Ok((ImageSpace::DeviceRgb, true, 0)));
        assert_eq!(
            prepare(&translucent, PdfStandard::X3_2002),
            Err(PdfError::TransparencyNotAllowed {
                standard: PdfStandard::X3_2002,
                image: ImageId(4)
            })
        );
        // X-1a: the alpha channel is reported before the RGB data.
        assert!(matches!(
            prepare(&translucent, PdfStandard::X1a2001),
            Err(PdfError::TransparencyNotAllowed { .. })
        ));

        // A fully opaque alpha channel is simply dropped.
        let opaque = raw(PixelFormat::Rgba8, 1, 2, vec![1, 2, 3, 255, 4, 5, 6, 255]);
        assert_eq!(prepare(&opaque, PdfStandard::X3_2002), Ok((ImageSpace::Icc(0), false, 1)));
        assert_eq!(prepare(&opaque, PdfStandard::X4), Ok((ImageSpace::Icc(0), false, 1)));
        assert!(matches!(
            prepare(&opaque, PdfStandard::X1a2001),
            Err(PdfError::RgbNotAllowed { .. })
        ));
    }

    #[test]
    fn pixel_buffers_must_match_the_dimensions() {
        for (format, w, h, len) in [
            (PixelFormat::Rgb8, 2, 2, 11),
            (PixelFormat::Rgb8, 2, 2, 13),
            (PixelFormat::Gray8, 3, 3, 0),
            (PixelFormat::Cmyk8, 1, 1, 3),
            (PixelFormat::Rgba8, 1, 1, 3),
            (PixelFormat::Rgb8, 0, 5, 0),
            (PixelFormat::Rgb8, 5, 0, 0),
            (PixelFormat::Rgb8, u32::MAX, u32::MAX, 12),
            (PixelFormat::Gray8, 70_000, 70_000, 4),
        ] {
            let image = raw(format, w, h, vec![0; len]);
            assert!(
                matches!(
                    prepare(&image, PdfStandard::Plain),
                    Err(PdfError::InvalidImage { image: ImageId(4), .. })
                ),
                "{format:?} {w}x{h} with {len} bytes"
            );
        }
    }

    #[test]
    fn profiles_are_pooled_by_content() {
        let a = SRGB_PROFILE.to_vec();
        let b = SRGB_PROFILE.to_vec();
        let mut other = SRGB_PROFILE.to_vec();
        if let Some(last) = other.last_mut() {
            *last ^= 1;
        }
        let mut pool = IccPool::default();
        assert_eq!(pool.intern(&a, 3), 0);
        assert_eq!(pool.intern(&b, 3), 0);
        assert_eq!(pool.intern(&other, 3), 1);
        assert_eq!(pool.intern(SRGB_PROFILE, 3), 0);
        assert_eq!(pool.profiles.len(), 2);
    }
}
