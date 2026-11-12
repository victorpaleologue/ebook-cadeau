//! Header-only image sniffing: container format and pixel size without decoding.
//!
//! [`probe`] looks at the first bytes of a file and answers "what is this and how big is
//! it?" for JPEG, PNG, GIF, WebP and AVIF/HEIF. Nothing is decoded and nothing is
//! allocated in proportion to the input; every loop is bounded, every read is checked, so
//! the functions never panic, whatever the bytes are.
//!
//! The format is decided from **magic bytes**, never from a `Content-Type` header or a
//! file extension: cover servers routinely answer `200 image/jpeg` with an HTML error page
//! or a GIF placeholder (see `docs/research/02-cover-sources.md`, §4).
//!
//! ```
//! use ebc_covers::sniff::{probe, Format};
//!
//! // A 1 x 1 GIF, the classic "no cover" answer.
//! let gif = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\xff\xff\xff\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00;";
//! let info = probe(gif).expect("a GIF header");
//! assert_eq!((info.format, info.width, info.height), (Format::Gif, 1, 1));
//! assert!(probe(b"<html>not an image</html>").is_none());
//! ```

/// How many leading bytes are normally enough for [`probe`] to succeed.
///
/// JPEG files may carry large `APPn` segments (EXIF, ICC profiles, thumbnails) before the
/// frame header; 64 KiB covers nearly all files met in practice. Callers that fetched less
/// simply get `None` and may fetch more.
pub const PROBE_BYTES_HINT: usize = 64 * 1024;

/// Upper bound on JPEG segments walked before giving up.
const MAX_JPEG_SEGMENTS: usize = 1024;
/// Upper bound on PNG chunks walked while looking for `acTL`, `tRNS` and `eXIf`.
const MAX_PNG_CHUNKS: usize = 256;
/// Upper bound on GIF blocks and sub-blocks walked while counting frames.
const MAX_GIF_HOPS: usize = 1 << 16;
/// Upper bound on RIFF chunks walked in a WebP file.
const MAX_RIFF_CHUNKS: usize = 256;
/// Upper bound on boxes walked at each level of an ISO-BMFF (AVIF/HEIF) file.
const MAX_BMFF_BOXES: usize = 256;
/// Upper bound on item-property associations read from an `ipma` box.
const MAX_BMFF_ASSOCIATIONS: usize = 4096;
/// Upper bound on IFD entries read from an EXIF block.
const MAX_IFD_ENTRIES: usize = 512;

/// Image container formats recognised by [`probe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// JPEG (JFIF/EXIF), any frame type.
    Jpeg,
    /// PNG (including APNG).
    Png,
    /// GIF87a / GIF89a.
    Gif,
    /// WebP (lossy `VP8`, lossless `VP8L` or extended `VP8X`).
    WebP,
    /// AVIF (AV1 in ISO-BMFF).
    Avif,
    /// HEIF/HEIC (HEVC in ISO-BMFF).
    Heif,
}

impl Format {
    /// The IANA media type.
    #[must_use]
    pub const fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Avif => "image/avif",
            Self::Heif => "image/heif",
        }
    }

    /// The usual file extension, without a dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Avif => "avif",
            Self::Heif => "heic",
        }
    }
}

/// Format and **stored** pixel size of an image.
///
/// `width` and `height` describe the pixel grid as encoded. A JPEG shot on a phone may
/// carry an EXIF orientation that swaps the two on display; [`probe_details`] reports it
/// and [`Details::oriented`] gives the size as displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageInfo {
    /// Container format, from magic bytes.
    pub format: Format,
    /// Width in pixels (never zero).
    pub width: u32,
    /// Height in pixels (never zero).
    pub height: u32,
}

impl ImageInfo {
    /// Builds an `ImageInfo` from parts.
    #[must_use]
    pub const fn new(format: Format, width: u32, height: u32) -> Self {
        Self { format, width, height }
    }

    /// `width / height`; `0.0` when the height is zero.
    #[must_use]
    pub fn aspect(&self) -> f64 {
        if self.height == 0 { 0.0 } else { f64::from(self.width) / f64::from(self.height) }
    }

    /// Number of pixels.
    #[must_use]
    pub fn pixels(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

/// EXIF orientation (TIFF tag `0x0112`): how the stored grid must be transformed for
/// display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// 1 — display as stored.
    #[default]
    Normal,
    /// 2 — mirrored left-right.
    FlipHorizontal,
    /// 3 — rotated 180°.
    Rotate180,
    /// 4 — mirrored top-bottom.
    FlipVertical,
    /// 5 — mirrored along the top-left / bottom-right diagonal.
    Transpose,
    /// 6 — must be rotated 90° clockwise for display.
    Rotate90,
    /// 7 — mirrored along the top-right / bottom-left diagonal.
    Transverse,
    /// 8 — must be rotated 270° clockwise (90° anticlockwise) for display.
    Rotate270,
}

impl Orientation {
    /// Maps an EXIF orientation value (1–8); anything else is `None`.
    #[must_use]
    pub const fn from_exif(value: u32) -> Option<Self> {
        Some(match value {
            1 => Self::Normal,
            2 => Self::FlipHorizontal,
            3 => Self::Rotate180,
            4 => Self::FlipVertical,
            5 => Self::Transpose,
            6 => Self::Rotate90,
            7 => Self::Transverse,
            8 => Self::Rotate270,
            _ => return None,
        })
    }

    /// The EXIF value (1–8).
    #[must_use]
    pub const fn exif_value(self) -> u8 {
        match self {
            Self::Normal => 1,
            Self::FlipHorizontal => 2,
            Self::Rotate180 => 3,
            Self::FlipVertical => 4,
            Self::Transpose => 5,
            Self::Rotate90 => 6,
            Self::Transverse => 7,
            Self::Rotate270 => 8,
        }
    }

    /// `true` when width and height trade places on display (values 5–8).
    #[must_use]
    pub const fn swaps_axes(self) -> bool {
        matches!(self, Self::Transpose | Self::Rotate90 | Self::Transverse | Self::Rotate270)
    }
}

/// Which bitstream a WebP container holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebpKind {
    /// Simple lossy file (`VP8 ` chunk).
    Lossy,
    /// Simple lossless file (`VP8L` chunk).
    Lossless,
    /// Extended file (`VP8X` chunk): may hold alpha, animation, EXIF, ICC.
    Extended,
}

/// Everything [`probe_details`] can tell from the headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Details {
    /// Format and stored pixel size.
    pub info: ImageInfo,
    /// Orientation to apply for display. [`Orientation::Normal`] when the file says
    /// nothing. Read from: JPEG `APP1` EXIF, PNG `eXIf`, WebP `EXIF` chunk (only when the
    /// whole file is supplied — the chunk sits after the pixels), AVIF/HEIF `irot`
    /// (rotation only; an `imir` mirror is ignored).
    pub orientation: Orientation,
    /// Bits per sample, when the header states it (JPEG, PNG, AVIF/HEIF `pixi`).
    pub bit_depth: Option<u8>,
    /// Number of colour components: JPEG 1 = grey, 3 = YCbCr/RGB, 4 = CMYK/YCCK; PNG
    /// channels per pixel as stored (palette = 1).
    pub channels: Option<u8>,
    /// The file declares transparency (PNG colour types 4/6 or a `tRNS` chunk before the
    /// pixels, WebP alpha flag, AVIF/HEIF auxiliary alpha image).
    pub has_alpha: bool,
    /// The file declares more than one frame (GIF with two image blocks, APNG `acTL`,
    /// WebP animation flag, AVIF/HEIF sequence brand).
    pub animated: bool,
    /// JPEG: progressive frame (SOF2, SOF6, SOF10, SOF14). PNG: Adam7 interlacing.
    pub progressive: bool,
    /// JPEG only: the start-of-frame marker byte (`0xC0`–`0xCF`).
    pub jpeg_sof: Option<u8>,
    /// WebP only: which bitstream chunk was found.
    pub webp: Option<WebpKind>,
}

impl Details {
    fn new(format: Format, width: u32, height: u32) -> Self {
        Self {
            info: ImageInfo { format, width, height },
            orientation: Orientation::Normal,
            bit_depth: None,
            channels: None,
            has_alpha: false,
            animated: false,
            progressive: false,
            jpeg_sof: None,
            webp: None,
        }
    }

    /// `(width, height)` as displayed, i.e. after applying [`Details::orientation`].
    #[must_use]
    pub const fn display_size(&self) -> (u32, u32) {
        if self.orientation.swaps_axes() {
            (self.info.height, self.info.width)
        } else {
            (self.info.width, self.info.height)
        }
    }

    /// The image size as displayed, as an [`ImageInfo`] — what print-quality and aspect
    /// checks should be fed.
    #[must_use]
    pub const fn oriented(&self) -> ImageInfo {
        let (width, height) = self.display_size();
        ImageInfo { format: self.info.format, width, height }
    }
}

/// Reads format and stored pixel size from the first bytes of an image.
///
/// Returns `None` when the bytes are not a recognised image, when the header is
/// truncated before the size, or when a dimension is zero. Never panics.
#[must_use]
pub fn probe(bytes: &[u8]) -> Option<ImageInfo> {
    probe_details(bytes).map(|d| d.info)
}

/// Like [`probe`], with orientation, bit depth, alpha, animation and frame type.
#[must_use]
pub fn probe_details(bytes: &[u8]) -> Option<Details> {
    let details = if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        probe_jpeg(bytes)
    } else if bytes.starts_with(&PNG_SIGNATURE) {
        probe_png(bytes)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        probe_gif(bytes)
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP".as_slice()) {
        probe_webp(bytes)
    } else if bytes.get(4..8) == Some(b"ftyp".as_slice()) {
        probe_bmff(bytes)
    } else {
        None
    }?;
    (details.info.width > 0 && details.info.height > 0).then_some(details)
}

// ------------------------------------------------------------------------------------
// Checked readers
// ------------------------------------------------------------------------------------

fn slice(b: &[u8], at: usize, len: usize) -> Option<&[u8]> {
    b.get(at..at.checked_add(len)?)
}

fn u8_at(b: &[u8], at: usize) -> Option<u8> {
    b.get(at).copied()
}

fn be16(b: &[u8], at: usize) -> Option<u16> {
    slice(b, at, 2).map(|s| u16::from_be_bytes([s[0], s[1]]))
}

fn le16(b: &[u8], at: usize) -> Option<u16> {
    slice(b, at, 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    slice(b, at, 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    slice(b, at, 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn le24(b: &[u8], at: usize) -> Option<u32> {
    slice(b, at, 3).map(|s| u32::from_le_bytes([s[0], s[1], s[2], 0]))
}

fn be64(b: &[u8], at: usize) -> Option<u64> {
    slice(b, at, 8).map(|s| u64::from_be_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]))
}

// ------------------------------------------------------------------------------------
// EXIF (TIFF) orientation
// ------------------------------------------------------------------------------------

/// Reads the orientation tag from a TIFF block (`II*\0…` or `MM\0*…`).
fn tiff_orientation(tiff: &[u8]) -> Option<Orientation> {
    let little = match slice(tiff, 0, 2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let r16 = |at: usize| if little { le16(tiff, at) } else { be16(tiff, at) };
    let r32 = |at: usize| if little { le32(tiff, at) } else { be32(tiff, at) };
    if r16(2)? != 42 {
        return None;
    }
    let ifd = usize::try_from(r32(4)?).ok()?;
    let count = usize::from(r16(ifd)?).min(MAX_IFD_ENTRIES);
    for i in 0..count {
        let entry = ifd.checked_add(2)?.checked_add(i.checked_mul(12)?)?;
        if r16(entry)? != 0x0112 {
            continue;
        }
        let kind = r16(entry.checked_add(2)?)?;
        let value_at = entry.checked_add(8)?;
        let value = match kind {
            3 => u32::from(r16(value_at)?),
            4 => r32(value_at)?,
            1 => u32::from(u8_at(tiff, value_at)?),
            _ => return None,
        };
        return Orientation::from_exif(value);
    }
    None
}

/// Reads the orientation from an EXIF payload that may start with `Exif\0\0`.
fn exif_orientation(payload: &[u8]) -> Option<Orientation> {
    let tiff = payload.strip_prefix(b"Exif\0\0").unwrap_or(payload);
    tiff_orientation(tiff)
}

// ------------------------------------------------------------------------------------
// JPEG
// ------------------------------------------------------------------------------------

/// `true` for the thirteen start-of-frame markers: `C0`–`CF` except `C4` (DHT), `C8`
/// (reserved JPG) and `CC` (DAC).
const fn is_sof(marker: u8) -> bool {
    matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC)
}

fn probe_jpeg(b: &[u8]) -> Option<Details> {
    let mut pos = 2usize; // after SOI
    let mut frame: Option<Details> = None;
    let mut orientation: Option<Orientation> = None;
    for _ in 0..MAX_JPEG_SEGMENTS {
        // A marker is 0xFF followed by a non-0xFF, non-zero byte; 0xFF bytes may pad.
        if u8_at(b, pos) != Some(0xFF) {
            break;
        }
        let mut at = pos + 1;
        while u8_at(b, at) == Some(0xFF) {
            at += 1;
        }
        let Some(marker) = u8_at(b, at) else { break };
        pos = at + 1;
        match marker {
            // Standalone markers carry no length.
            0x01 | 0xD0..=0xD8 => continue,
            // Entropy-coded data, end of image or a stuffed byte: the header is over.
            0x00 | 0xD9 | 0xDA => break,
            _ => {}
        }
        let Some(len) = be16(b, pos).map(usize::from) else { break };
        if len < 2 {
            break;
        }
        let end = pos.saturating_add(len).min(b.len());
        let payload = b.get(pos + 2..end).unwrap_or_default();
        if is_sof(marker) && frame.is_none() {
            let (Some(precision), Some(height), Some(width), Some(components)) =
                (u8_at(payload, 0), be16(payload, 1), be16(payload, 3), u8_at(payload, 5))
            else {
                break;
            };
            let mut d = Details::new(Format::Jpeg, u32::from(width), u32::from(height));
            d.bit_depth = Some(precision);
            d.channels = Some(components);
            d.progressive = matches!(marker, 0xC2 | 0xC6 | 0xCA | 0xCE);
            d.jpeg_sof = Some(marker);
            frame = Some(d);
        } else if marker == 0xE1 && orientation.is_none() && payload.starts_with(b"Exif\0\0") {
            orientation = exif_orientation(payload);
        }
        if frame.is_some() && orientation.is_some() {
            break;
        }
        pos = pos.saturating_add(len);
    }
    let mut details = frame?;
    details.orientation = orientation.unwrap_or_default();
    Some(details)
}

// ------------------------------------------------------------------------------------
// PNG
// ------------------------------------------------------------------------------------

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn probe_png(b: &[u8]) -> Option<Details> {
    if slice(b, 12, 4)? != b"IHDR" {
        return None;
    }
    let width = be32(b, 16)?;
    let height = be32(b, 20)?;
    // The PNG specification limits dimensions to 2^31 - 1.
    if width > 0x7FFF_FFFF || height > 0x7FFF_FFFF {
        return None;
    }
    let mut d = Details::new(Format::Png, width, height);
    d.bit_depth = u8_at(b, 24);
    if let Some(colour_type) = u8_at(b, 25) {
        d.channels = match colour_type {
            0 | 3 => Some(1),
            2 => Some(3),
            4 => Some(2),
            6 => Some(4),
            _ => None,
        };
        d.has_alpha = matches!(colour_type, 4 | 6);
    }
    d.progressive = u8_at(b, 28) == Some(1);

    // Walk the chunks before the pixels for animation, palette transparency and EXIF.
    let mut pos = 8usize;
    for _ in 0..MAX_PNG_CHUNKS {
        let (Some(len), Some(kind)) = (be32(b, pos), slice(b, pos.saturating_add(4), 4)) else {
            break;
        };
        let Ok(len) = usize::try_from(len) else { break };
        match kind {
            b"IDAT" | b"IEND" => break,
            b"acTL" => d.animated = true,
            b"tRNS" => d.has_alpha = true,
            b"eXIf" => {
                if let Some(o) = slice(b, pos.saturating_add(8), len).and_then(exif_orientation) {
                    d.orientation = o;
                }
            }
            _ => {}
        }
        let Some(next) = pos.checked_add(12).and_then(|p| p.checked_add(len)) else { break };
        pos = next;
    }
    Some(d)
}

// ------------------------------------------------------------------------------------
// GIF
// ------------------------------------------------------------------------------------

fn probe_gif(b: &[u8]) -> Option<Details> {
    let width = u32::from(le16(b, 6)?);
    let height = u32::from(le16(b, 8)?);
    let mut d = Details::new(Format::Gif, width, height);
    d.bit_depth = u8_at(b, 10).map(|packed| ((packed >> 4) & 7) + 1);
    d.animated = gif_frame_count_up_to_two(b) >= 2;
    Some(d)
}

/// Size in bytes of a GIF colour table announced by a packed field, or 0.
const fn gif_table_len(packed: u8) -> usize {
    if packed & 0x80 == 0 { 0 } else { 3 * (1usize << ((packed & 7) + 1)) }
}

/// Counts image descriptors, stopping at two or at the first sign of truncation.
fn gif_frame_count_up_to_two(b: &[u8]) -> u32 {
    let Some(packed) = u8_at(b, 10) else { return 0 };
    let mut pos = 13 + gif_table_len(packed);
    let mut frames = 0u32;
    let mut hops = 0usize;
    // Skips a chain of data sub-blocks; `None` when the data ends early.
    let skip_sub_blocks = |mut at: usize, hops: &mut usize| -> Option<usize> {
        loop {
            *hops += 1;
            if *hops > MAX_GIF_HOPS {
                return None;
            }
            let n = usize::from(u8_at(b, at)?);
            at = at.checked_add(1 + n)?;
            if n == 0 {
                return Some(at);
            }
        }
    };
    while hops < MAX_GIF_HOPS {
        hops += 1;
        match u8_at(b, pos) {
            Some(0x21) => {
                // Extension: introducer, label, sub-blocks.
                let Some(next) = skip_sub_blocks(pos + 2, &mut hops) else { break };
                pos = next;
            }
            Some(0x2C) => {
                frames += 1;
                if frames >= 2 {
                    break;
                }
                let Some(local) = u8_at(b, pos + 9) else { break };
                // Descriptor (10 bytes), local colour table, LZW code size, sub-blocks.
                let data = pos + 10 + gif_table_len(local) + 1;
                let Some(next) = skip_sub_blocks(data, &mut hops) else { break };
                pos = next;
            }
            // Trailer, garbage or end of data.
            _ => break,
        }
    }
    frames
}

// ------------------------------------------------------------------------------------
// WebP
// ------------------------------------------------------------------------------------

fn probe_webp(b: &[u8]) -> Option<Details> {
    match slice(b, 12, 4)? {
        b"VP8 " => {
            // Frame tag (3 bytes, bit 0 clear on a key frame), start code, 14-bit sizes.
            let tag = u8_at(b, 20)?;
            if tag & 1 != 0 || slice(b, 23, 3)? != [0x9D, 0x01, 0x2A] {
                return None;
            }
            let width = u32::from(le16(b, 26)? & 0x3FFF);
            let height = u32::from(le16(b, 28)? & 0x3FFF);
            let mut d = Details::new(Format::WebP, width, height);
            d.webp = Some(WebpKind::Lossy);
            d.bit_depth = Some(8);
            Some(d)
        }
        b"VP8L" => {
            if u8_at(b, 20)? != 0x2F {
                return None;
            }
            let bits = le32(b, 21)?;
            if bits >> 29 != 0 {
                return None; // unknown bitstream version
            }
            let mut d =
                Details::new(Format::WebP, (bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1);
            d.webp = Some(WebpKind::Lossless);
            d.has_alpha = (bits >> 28) & 1 == 1;
            d.bit_depth = Some(8);
            Some(d)
        }
        b"VP8X" => {
            let flags = u8_at(b, 20)?;
            let mut d = Details::new(Format::WebP, le24(b, 24)? + 1, le24(b, 27)? + 1);
            d.webp = Some(WebpKind::Extended);
            d.has_alpha = flags & 0x10 != 0;
            d.animated = flags & 0x02 != 0;
            d.bit_depth = Some(8);
            if flags & 0x08 != 0 {
                d.orientation = webp_exif_orientation(b).unwrap_or_default();
            }
            Some(d)
        }
        _ => None,
    }
}

/// Finds the `EXIF` chunk of an extended WebP file, if the bytes reach that far.
fn webp_exif_orientation(b: &[u8]) -> Option<Orientation> {
    let mut pos = 12usize;
    for _ in 0..MAX_RIFF_CHUNKS {
        let kind = slice(b, pos, 4)?;
        let len = usize::try_from(le32(b, pos + 4)?).ok()?;
        if kind == b"EXIF" {
            return exif_orientation(slice(b, pos + 8, len)?);
        }
        // Chunks are padded to an even size.
        pos = pos.checked_add(8)?.checked_add(len)?.checked_add(len & 1)?;
    }
    None
}

// ------------------------------------------------------------------------------------
// ISO-BMFF: AVIF and HEIF
// ------------------------------------------------------------------------------------

/// One box header: where the payload sits (clamped to the available bytes) and where the
/// next sibling starts.
#[derive(Debug, Clone, Copy)]
struct BoxRef {
    kind: [u8; 4],
    start: usize,
    end: usize,
    next: usize,
}

fn read_box(b: &[u8], pos: usize, limit: usize) -> Option<BoxRef> {
    let limit = limit.min(b.len());
    let size32 = be32(b, pos)?;
    let kind: [u8; 4] = slice(b, pos + 4, 4)?.try_into().ok()?;
    let (header, size) = match size32 {
        0 => (8usize, u64::try_from(limit.checked_sub(pos)?).ok()?),
        1 => (16usize, be64(b, pos + 8)?),
        n => (8usize, u64::from(n)),
    };
    if size < header as u64 {
        return None;
    }
    let next = pos.saturating_add(usize::try_from(size).unwrap_or(usize::MAX));
    let start = pos.checked_add(header)?;
    if start > limit {
        return None;
    }
    Some(BoxRef { kind, start, end: next.min(limit), next })
}

/// Calls `visit` for each child box in `b[start..end]`, at most [`MAX_BMFF_BOXES`] times.
fn for_each_box(b: &[u8], start: usize, end: usize, mut visit: impl FnMut(usize, BoxRef)) {
    let mut pos = start;
    for index in 0..MAX_BMFF_BOXES {
        if pos >= end {
            break;
        }
        let Some(bx) = read_box(b, pos, end) else { break };
        visit(index, bx);
        if bx.next <= pos {
            break;
        }
        pos = bx.next;
    }
}

/// What we keep of one item property in `ipco`.
#[derive(Debug, Clone, Copy)]
enum Property {
    Size(u32, u32),
    Rotation(u8),
    Depth(u8),
    Alpha,
    Other,
}

fn probe_bmff(b: &[u8]) -> Option<Details> {
    let ftyp = read_box(b, 0, b.len())?;
    if &ftyp.kind != b"ftyp" {
        return None;
    }
    let brands = b.get(ftyp.start..ftyp.end)?;
    let mut is_avif = false;
    let mut is_heif = false;
    let mut is_sequence = false;
    // Major brand at 0..4, minor version at 4..8, compatible brands after.
    for (i, brand) in brands.chunks_exact(4).take(64).enumerate() {
        if i == 1 {
            continue;
        }
        match brand {
            b"avif" => is_avif = true,
            b"avis" => {
                is_avif = true;
                is_sequence = true;
            }
            b"heic" | b"heix" | b"heim" | b"heis" | b"mif1" => is_heif = true,
            b"hevc" | b"hevx" | b"hevm" | b"hevs" | b"msf1" => {
                is_heif = true;
                is_sequence = true;
            }
            _ => {}
        }
    }
    let format = match (is_avif, is_heif) {
        (true, _) => Format::Avif,
        (false, true) => Format::Heif,
        (false, false) => return None,
    };

    // Locate `meta` among the top-level boxes.
    let mut meta: Option<BoxRef> = None;
    for_each_box(b, ftyp.next, b.len(), |_, bx| {
        if &bx.kind == b"meta" && meta.is_none() {
            meta = Some(bx);
        }
    });
    let meta = meta?;
    // A `meta` box cut short would make the answer depend on how many bytes were
    // fetched (a thumbnail's extent may come before the primary image's): wait for all
    // of it.
    if meta.next > b.len() {
        return None;
    }

    let mut primary: Option<u32> = None;
    let mut properties: Vec<Property> = Vec::new();
    let mut primary_props: Vec<usize> = Vec::new();
    let mut ipma: Option<BoxRef> = None;
    // `meta` is a FullBox: four bytes of version and flags precede its children.
    for_each_box(b, meta.start.saturating_add(4), meta.end, |_, bx| match &bx.kind {
        b"pitm" => {
            primary = match u8_at(b, bx.start) {
                Some(0) => be16(b, bx.start + 4).map(u32::from),
                Some(_) => be32(b, bx.start + 4),
                None => None,
            };
        }
        b"iprp" => for_each_box(b, bx.start, bx.end, |_, inner| match &inner.kind {
            b"ipco" => for_each_box(b, inner.start, inner.end, |_, p| {
                properties.push(match &p.kind {
                    b"ispe" => match (be32(b, p.start + 4), be32(b, p.start + 8)) {
                        (Some(w), Some(h)) if p.start + 12 <= p.end => Property::Size(w, h),
                        _ => Property::Other,
                    },
                    b"irot" => {
                        u8_at(b, p.start).map_or(Property::Other, |v| Property::Rotation(v & 3))
                    }
                    b"pixi" => u8_at(b, p.start + 5).map_or(Property::Other, Property::Depth),
                    b"auxC" => {
                        let urn = b.get(p.start + 4..p.end).unwrap_or_default();
                        if urn.windows(5).any(|w| w == b"alpha") {
                            Property::Alpha
                        } else {
                            Property::Other
                        }
                    }
                    _ => Property::Other,
                });
            }),
            b"ipma" => ipma = Some(inner),
            _ => {}
        }),
        _ => {}
    });

    if let (Some(primary), Some(ipma)) = (primary, ipma) {
        primary_props = bmff_primary_properties(b, ipma, primary).unwrap_or_default();
    }

    let mut size: Option<(u32, u32)> = None;
    let mut rotation = 0u8;
    let mut depth = None;
    for &index in &primary_props {
        // Property indices are 1-based.
        match index.checked_sub(1).and_then(|i| properties.get(i)) {
            Some(&Property::Size(w, h)) if size.is_none() => size = Some((w, h)),
            Some(&Property::Rotation(r)) => rotation = r,
            Some(&Property::Depth(d)) => depth = Some(d),
            _ => {}
        }
    }
    // Without a usable association, fall back to the largest declared extent: the
    // primary image is normally the largest item (thumbnails and tiles are smaller).
    let (width, height) = size.or_else(|| {
        properties
            .iter()
            .filter_map(|p| match *p {
                Property::Size(w, h) => Some((w, h)),
                _ => None,
            })
            .max_by_key(|&(w, h)| u64::from(w) * u64::from(h))
    })?;

    let mut d = Details::new(format, width, height);
    d.bit_depth = depth;
    d.animated = is_sequence;
    d.has_alpha = properties.iter().any(|p| matches!(p, Property::Alpha));
    // `irot` counts anticlockwise quarter turns.
    d.orientation = match rotation {
        1 => Orientation::Rotate270,
        2 => Orientation::Rotate180,
        3 => Orientation::Rotate90,
        _ => Orientation::Normal,
    };
    Some(d)
}

/// Property indices associated with item `primary` in an `ipma` box.
fn bmff_primary_properties(b: &[u8], ipma: BoxRef, primary: u32) -> Option<Vec<usize>> {
    let data = b.get(ipma.start..ipma.end)?;
    let version = u8_at(data, 0)?;
    let wide = u8_at(data, 3)? & 1 == 1;
    let entries = be32(data, 4)?;
    let mut pos = 8usize;
    let mut budget = MAX_BMFF_ASSOCIATIONS;
    for _ in 0..entries {
        let item = if version < 1 {
            let v = u32::from(be16(data, pos)?);
            pos += 2;
            v
        } else {
            let v = be32(data, pos)?;
            pos += 4;
            v
        };
        let count = usize::from(u8_at(data, pos)?);
        pos += 1;
        let mut found = Vec::new();
        for _ in 0..count {
            budget = budget.checked_sub(1)?;
            let index = if wide {
                let v = usize::from(be16(data, pos)? & 0x7FFF);
                pos += 2;
                v
            } else {
                let v = usize::from(u8_at(data, pos)? & 0x7F);
                pos += 1;
                v
            };
            found.push(index);
        }
        if item == primary {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_images_and_short_input() {
        assert!(probe(b"").is_none());
        assert!(probe(b"\xFF").is_none());
        assert!(probe(b"\xFF\xD8\xFF").is_none());
        assert!(probe(b"<!doctype html><html>").is_none());
        assert!(probe(b"GIF89a").is_none());
        assert!(probe(&PNG_SIGNATURE).is_none());
        assert!(probe(b"RIFF\0\0\0\0WEBP").is_none());
    }

    #[test]
    fn orientation_round_trip_and_axes() {
        for v in 1..=8u32 {
            let o = Orientation::from_exif(v).expect("valid");
            assert_eq!(u32::from(o.exif_value()), v);
            assert_eq!(o.swaps_axes(), v >= 5);
        }
        assert!(Orientation::from_exif(0).is_none());
        assert!(Orientation::from_exif(9).is_none());
    }

    #[test]
    fn sof_marker_set_is_exactly_thirteen() {
        let sof: Vec<u8> = (0u8..=255).filter(|m| is_sof(*m)).collect();
        assert_eq!(
            sof,
            [0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF]
        );
    }

    #[test]
    fn zero_dimensions_are_not_images() {
        // GIF with a 0 x 5 logical screen.
        assert!(probe(b"GIF89a\x00\x00\x05\x00\x00\x00\x00").is_none());
        // PNG IHDR with zero height.
        let mut png = PNG_SIGNATURE.to_vec();
        png.extend_from_slice(&[0, 0, 0, 13]);
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&[0, 0, 0, 7, 0, 0, 0, 0, 8, 2, 0, 0, 0]);
        assert!(probe(&png).is_none());
    }

    #[test]
    fn format_strings() {
        assert_eq!(Format::Jpeg.mime(), "image/jpeg");
        assert_eq!(Format::Heif.extension(), "heic");
        assert_eq!(ImageInfo::new(Format::Png, 200, 400).aspect(), 0.5);
        assert_eq!(ImageInfo::new(Format::Png, 200, 400).pixels(), 80_000);
    }
}
