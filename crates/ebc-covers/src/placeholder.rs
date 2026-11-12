//! Detecting "no cover available" answers and images that cannot be a book front.
//!
//! Cover services rarely answer "not found" honestly: they send a 1 × 1 GIF, a grey
//! "image not available" tile, or an HTML page with an image media type. The checks here
//! follow `docs/research/02-cover-sources.md`, §4 and implications 36–37. They come in
//! three layers:
//!
//! * **bytes** — [`assess`] / [`is_placeholder`]: tiny bodies, 1 × 1 and thumbnail-sized
//!   images, known digests. Needs only the header facts from [`crate::sniff::probe`];
//! * **source** — [`assess_for_source`]: adds what is known about one provider (the two
//!   byte lengths of the BnF placeholder, the Google "image not available" digests);
//! * **pixels** — [`assess_pixels`]: a near-uniform or nearly colourless image, or one
//!   whose perceptual hash matches a gallery of known placeholders. Optional, because it
//!   needs decoded pixels; the hosted service never reads the pixels of a third-party
//!   preview, so this layer is for files the user supplied and for operator tools.
//!
//! An implausible aspect ratio is reported as a [`Suspicion`], not as a rejection: square
//! audiobook covers and landscape picture books are real covers.
//!
//! The thresholds are proposals to tune on a measured sample (research open question 2).
//! Only hashes and byte lengths of placeholders are kept here, never the images.

use ebc_scene::Pixmap;
use md5::{Digest, Md5};

use crate::sniff::{self, ImageInfo};

/// A body shorter than this cannot be a cover (1 × 1 GIFs are 35–43 bytes).
pub const MIN_BODY_BYTES: usize = 100;
/// An image with a side shorter than this is a thumbnail or an icon, not a cover.
pub const MIN_SIDE_PX: u32 = 100;
/// Plausible `width / height` range for a book front; outside it the image is flagged
/// as suspicious.
pub const ASPECT_RANGE: (f64, f64) = (0.55, 0.85);
/// Below this many bits per pixel a complete file is flagged as suspiciously empty
/// (flat colour compresses to almost nothing in every format).
pub const MIN_BITS_PER_PIXEL: f64 = 0.12;

/// MD5 digests of the two Google Books "image not available" pictures (the digests
/// calibre rejects; see research §2.2).
pub const GOOGLE_PLACEHOLDER_MD5: [&str; 2] =
    ["0de4383ebad0adad5eeb8975cd796657", "a64fa89d7ebc97075c1d363fc5fea71f"];

/// `Content-Length` values of the two BnF catalogue placeholder bodies (research §2.3).
pub const BNF_PLACEHOLDER_LENGTHS: [u64; 2] = [4566, 4658];

/// Substrings of image URLs known to be "no image" assets: Amazon's `no-img-sm.gif` icon
/// and the placeholder image id filtered by Open Library's importer (research §4).
/// They are listed so that a URL met in third-party metadata is recognised, not because
/// any request is ever sent to those hosts.
pub const PLACEHOLDER_URL_MARKERS: [&str; 2] =
    ["/images/G/01/x-site/icons/no-img-sm.gif", "/01RmK+J4pJL."];

/// Why an image is considered a placeholder (a definite rejection).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reason {
    /// The bytes are not an image in a recognised format (HTML error page, truncated
    /// body, …).
    NotAnImage,
    /// The body is shorter than [`MIN_BODY_BYTES`].
    TinyBody {
        /// Body length in bytes.
        bytes: usize,
    },
    /// A 1 × 1 image.
    OnePixel,
    /// A side is shorter than [`MIN_SIDE_PX`].
    TooSmall {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
    /// The MD5 of the body is a known placeholder of the named source.
    KnownDigest {
        /// Source id, e.g. `"googlebooks"`.
        source: &'static str,
    },
    /// The body length is a known placeholder length of the named source.
    KnownLength {
        /// Source id, e.g. `"bnf"`.
        source: &'static str,
        /// Body length in bytes.
        bytes: u64,
    },
    /// The luminance barely varies: a flat tile.
    NearUniform {
        /// Standard deviation of the luminance on a 0–255 scale.
        luma_stddev: f32,
    },
    /// Too few distinct colours once quantised to 4 bits per channel.
    FewColours {
        /// Number of distinct quantised colours.
        count: u32,
    },
    /// The perceptual hash is within the allowed distance of a gallery entry.
    KnownHash {
        /// Hamming distance to the closest gallery entry.
        distance: u32,
    },
}

/// Something odd that deserves a second look but does not disqualify the image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Suspicion {
    /// `width / height` is outside [`ASPECT_RANGE`].
    AspectRatio {
        /// The ratio found.
        ratio: f64,
    },
    /// The file holds several frames; covers are still images.
    Animated,
    /// The file is very small for its pixel count: probably a flat tile.
    LowBitsPerPixel {
        /// Bits of file per pixel.
        bits: f64,
    },
    /// The body has the length of a placeholder known for some source, but the source
    /// of these bytes was not given (see [`assess_for_source`]).
    PlaceholderLength {
        /// Body length in bytes.
        bytes: u64,
    },
}

/// The outcome of a placeholder check.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Assessment {
    /// Definite reasons to reject the image. Empty when it passed.
    pub reasons: Vec<Reason>,
    /// Oddities to show the user ("is this the right cover?").
    pub suspicions: Vec<Suspicion>,
}

impl Assessment {
    /// `true` when at least one definite reason was found.
    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        !self.reasons.is_empty()
    }

    /// `true` when the image is not rejected but something is odd.
    #[must_use]
    pub fn is_suspicious(&self) -> bool {
        !self.suspicions.is_empty()
    }

    /// `true` when nothing at all was found.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.reasons.is_empty() && self.suspicions.is_empty()
    }

    /// Merges another assessment into this one (e.g. byte-level plus pixel-level).
    pub fn merge(&mut self, other: Self) {
        self.reasons.extend(other.reasons);
        self.suspicions.extend(other.suspicions);
    }
}

/// Lower-case hexadecimal MD5 of `bytes`.
///
/// MD5 is used only because the published placeholder digests are MD5; it has no
/// security role here.
#[must_use]
pub fn md5_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Md5::digest(bytes);
    let mut out = String::with_capacity(32);
    for byte in digest.iter() {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0F)]));
    }
    out
}

/// `true` when the URL points at a known "no image" asset
/// ([`PLACEHOLDER_URL_MARKERS`]).
#[must_use]
pub fn is_placeholder_url(url: &str) -> bool {
    PLACEHOLDER_URL_MARKERS.iter().any(|marker| url.contains(marker))
}

/// `true` when `content_length` is one of the BnF placeholder lengths.
#[must_use]
pub fn is_bnf_placeholder_length(content_length: u64) -> bool {
    BNF_PLACEHOLDER_LENGTHS.contains(&content_length)
}

/// Checks header facts and the **complete** body of an image.
///
/// `info` normally comes from [`crate::sniff::probe`] on the same bytes. The check is
/// source-agnostic; use [`assess_for_source`] when the provider is known.
#[must_use]
pub fn assess(info: &ImageInfo, bytes: &[u8]) -> Assessment {
    let mut out = Assessment::default();
    if bytes.len() < MIN_BODY_BYTES {
        out.reasons.push(Reason::TinyBody { bytes: bytes.len() });
    }
    if info.width <= 1 && info.height <= 1 {
        out.reasons.push(Reason::OnePixel);
    } else if info.width < MIN_SIDE_PX || info.height < MIN_SIDE_PX {
        out.reasons.push(Reason::TooSmall { width: info.width, height: info.height });
    }
    if !bytes.is_empty() {
        let digest = md5_hex(bytes);
        if GOOGLE_PLACEHOLDER_MD5.contains(&digest.as_str()) {
            out.reasons.push(Reason::KnownDigest { source: "googlebooks" });
        }
    }
    if out.reasons.is_empty() {
        let ratio = info.aspect();
        if ratio < ASPECT_RANGE.0 || ratio > ASPECT_RANGE.1 {
            out.suspicions.push(Suspicion::AspectRatio { ratio });
        }
        let pixels = info.pixels();
        if pixels > 0 {
            let bits = (bytes.len() as f64) * 8.0 / (pixels as f64);
            if bits < MIN_BITS_PER_PIXEL {
                out.suspicions.push(Suspicion::LowBitsPerPixel { bits });
            }
        }
        let len = bytes.len() as u64;
        if is_bnf_placeholder_length(len) {
            out.suspicions.push(Suspicion::PlaceholderLength { bytes: len });
        }
        if sniff::probe_details(bytes).is_some_and(|d| d.animated) {
            out.suspicions.push(Suspicion::Animated);
        }
    }
    out
}

/// Like [`assess`], but sniffs the bytes itself: a body that is not a recognised image
/// is rejected with [`Reason::NotAnImage`].
#[must_use]
pub fn assess_bytes(bytes: &[u8]) -> Assessment {
    match sniff::probe(bytes) {
        Some(info) => assess(&info, bytes),
        None => {
            let mut reasons = vec![Reason::NotAnImage];
            if bytes.len() < MIN_BODY_BYTES {
                reasons.push(Reason::TinyBody { bytes: bytes.len() });
            }
            Assessment { reasons, suspicions: Vec::new() }
        }
    }
}

/// Like [`assess`], plus the signatures known for one source id (`"bnf"`: the two
/// placeholder body lengths become a definite reason).
#[must_use]
pub fn assess_for_source(source: &str, info: &ImageInfo, bytes: &[u8]) -> Assessment {
    let mut out = assess(info, bytes);
    let len = bytes.len() as u64;
    if source == "bnf" && is_bnf_placeholder_length(len) {
        out.suspicions.retain(|s| !matches!(s, Suspicion::PlaceholderLength { .. }));
        out.reasons.push(Reason::KnownLength { source: "bnf", bytes: len });
    }
    out
}

/// `true` when the image is a placeholder according to [`assess`]: a body under 100
/// bytes, a 1 × 1 image, a side under 100 px, or a known "no image" digest.
///
/// An implausible aspect ratio does **not** make this return `true`; it is reported by
/// [`assess`] as a [`Suspicion`].
///
/// ```
/// use ebc_covers::is_placeholder;
/// use ebc_covers::sniff::probe;
///
/// let gif = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\xff\xff\xff\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00;";
/// let info = probe(gif).expect("gif header");
/// assert!(is_placeholder(&info, gif));
/// ```
#[must_use]
pub fn is_placeholder(info: &ImageInfo, bytes: &[u8]) -> bool {
    assess(info, bytes).is_placeholder()
}

// ------------------------------------------------------------------------------------
// Pixel-level checks
// ------------------------------------------------------------------------------------

/// Thresholds of the pixel-level check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelThresholds {
    /// A luminance standard deviation below this (0–255 scale) means "flat tile".
    pub min_luma_stddev: f32,
    /// Fewer distinct colours than this, after quantisation to 4 bits per channel,
    /// means "placeholder graphic".
    pub min_distinct_colours: u32,
    /// Maximum Hamming distance (out of 64 bits) for a perceptual-hash match.
    pub max_hash_distance: u32,
}

impl Default for PixelThresholds {
    /// Standard deviation 2, 16 colours, distance 6 (research implication 36).
    fn default() -> Self {
        Self { min_luma_stddev: 2.0, min_distinct_colours: 16, max_hash_distance: 6 }
    }
}

/// At most about this many pixels are sampled for the statistics.
const MAX_SAMPLES: u64 = 1 << 20;

/// Integer luminance (Rec. 601 weights) of a pixel composited over white.
fn luma(px: &[u8]) -> u32 {
    let a = u32::from(px[3]);
    let over_white = |c: u8| (u32::from(c) * a + 255 * (255 - a) + 127) / 255;
    (299 * over_white(px[0]) + 587 * over_white(px[1]) + 114 * over_white(px[2]) + 500) / 1000
}

/// Sampling step so that at most [`MAX_SAMPLES`] pixels are visited.
fn sample_step(image: &Pixmap) -> usize {
    let pixels = u64::from(image.width()) * u64::from(image.height());
    let mut step = 1usize;
    while pixels / (step as u64 * step as u64) > MAX_SAMPLES {
        step += 1;
    }
    step
}

/// Calls `visit` on every sampled pixel (RGBA slice).
fn for_each_sample(image: &Pixmap, mut visit: impl FnMut(&[u8])) {
    let step = sample_step(image);
    let width = image.width() as usize;
    let data = image.data();
    for row in data.chunks_exact(width * 4).step_by(step) {
        for px in row.chunks_exact(4).step_by(step) {
            visit(px);
        }
    }
}

/// Standard deviation of the luminance (0–255 scale), transparent pixels counted as
/// white. Large images are sampled on a regular grid.
#[must_use]
pub fn luma_stddev(image: &Pixmap) -> f32 {
    let (mut n, mut sum, mut sum_sq) = (0u64, 0u64, 0u64);
    for_each_sample(image, |px| {
        let l = u64::from(luma(px));
        n += 1;
        sum += l;
        sum_sq += l * l;
    });
    if n == 0 {
        return 0.0;
    }
    let mean = sum as f64 / n as f64;
    let variance = (sum_sq as f64 / n as f64 - mean * mean).max(0.0);
    variance.sqrt() as f32
}

/// Number of distinct colours after quantisation to 4 bits per channel (at most 4096),
/// transparent pixels counted as white. Large images are sampled on a regular grid.
#[must_use]
pub fn distinct_colours_4bit(image: &Pixmap) -> u32 {
    let mut seen = [0u64; 64];
    for_each_sample(image, |px| {
        let a = u32::from(px[3]);
        let q = |c: u8| ((u32::from(c) * a + 255 * (255 - a) + 127) / 255) >> 4;
        let index = (q(px[0]) << 8 | q(px[1]) << 4 | q(px[2])) as usize;
        seen[index >> 6] |= 1u64 << (index & 63);
    });
    seen.iter().map(|word| word.count_ones()).sum()
}

/// A 64-bit difference hash (dHash): the image is reduced to a 9 × 8 grid of average
/// luminances and each bit says whether a cell is brighter than its right neighbour.
/// Visually similar images give hashes at a small [`hamming`] distance.
#[must_use]
pub fn dhash(image: &Pixmap) -> u64 {
    const COLS: usize = 9;
    const ROWS: usize = 8;
    let (w, h) = (image.width() as usize, image.height() as usize);
    let data = image.data();
    let mut cells = [[0u64; COLS]; ROWS];
    for (cy, row) in cells.iter_mut().enumerate() {
        let y0 = (cy * h / ROWS).min(h - 1);
        let y1 = ((cy + 1) * h / ROWS).clamp(y0 + 1, h);
        for (cx, cell) in row.iter_mut().enumerate() {
            let x0 = (cx * w / COLS).min(w - 1);
            let x1 = ((cx + 1) * w / COLS).clamp(x0 + 1, w);
            let mut sum = 0u64;
            for y in y0..y1 {
                let line = &data[(y * w + x0) * 4..(y * w + x1) * 4];
                sum += line.chunks_exact(4).map(|px| u64::from(luma(px))).sum::<u64>();
            }
            // Scaled average, so that small differences survive the integer division.
            *cell = sum * 16 / ((y1 - y0) as u64 * (x1 - x0) as u64);
        }
    }
    let mut hash = 0u64;
    for row in &cells {
        for pair in row.windows(2) {
            hash = hash << 1 | u64::from(pair[0] > pair[1]);
        }
    }
    hash
}

/// Number of differing bits between two hashes.
#[must_use]
pub const fn hamming(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Pixel-level placeholder check on a decoded image.
///
/// `gallery` holds [`dhash`] values of placeholders collected per provider; the crate
/// ships none (no placeholder image was collected by the research), so pass `&[]` or
/// your own list.
#[must_use]
pub fn assess_pixels(image: &Pixmap, thresholds: &PixelThresholds, gallery: &[u64]) -> Assessment {
    let mut out = Assessment::default();
    let (w, h) = (image.width(), image.height());
    if w <= 1 && h <= 1 {
        out.reasons.push(Reason::OnePixel);
        return out;
    }
    if w < MIN_SIDE_PX || h < MIN_SIDE_PX {
        out.reasons.push(Reason::TooSmall { width: w, height: h });
    }
    let stddev = luma_stddev(image);
    if stddev < thresholds.min_luma_stddev {
        out.reasons.push(Reason::NearUniform { luma_stddev: stddev });
    }
    let colours = distinct_colours_4bit(image);
    if colours < thresholds.min_distinct_colours {
        out.reasons.push(Reason::FewColours { count: colours });
    }
    if !gallery.is_empty() {
        let hash = dhash(image);
        if let Some(distance) = gallery.iter().map(|known| hamming(hash, *known)).min() {
            if distance <= thresholds.max_hash_distance {
                out.reasons.push(Reason::KnownHash { distance });
            }
        }
    }
    if out.reasons.is_empty() {
        let ratio = f64::from(w) / f64::from(h);
        if ratio < ASPECT_RANGE.0 || ratio > ASPECT_RANGE.1 {
            out.suspicions.push(Suspicion::AspectRatio { ratio });
        }
    }
    out
}

/// `true` when [`assess_pixels`] with the default thresholds and an empty gallery
/// rejects the image (near-uniform, nearly colourless or tiny).
#[must_use]
pub fn is_placeholder_pixels(image: &Pixmap) -> bool {
    assess_pixels(image, &PixelThresholds::default(), &[]).is_placeholder()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sniff::Format;

    #[test]
    fn md5_matches_rfc_1321_vectors() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(md5_hex(b"message digest"), "f96b697d7cb7938d525a2f31aaf161d0");
    }

    #[test]
    fn byte_level_rules() {
        let body = vec![0u8; 20_000];
        let info = |w, h| ImageInfo::new(Format::Jpeg, w, h);
        assert!(assess(&info(400, 600), &body).is_clean());
        assert!(!is_placeholder(&info(400, 600), &body));

        // Tiny body.
        assert_eq!(assess(&info(400, 600), &body[..99]).reasons, [Reason::TinyBody { bytes: 99 }]);
        assert!(!is_placeholder(&info(400, 600), &body[..100]));
        // 1 x 1 and thumbnails.
        assert_eq!(assess(&info(1, 1), &body).reasons, [Reason::OnePixel]);
        assert_eq!(
            assess(&info(99, 150), &body).reasons,
            [Reason::TooSmall { width: 99, height: 150 }]
        );
        assert_eq!(assess(&info(1, 600), &body).reasons, [Reason::TooSmall { width: 1, height: 600 }]);
        assert!(!is_placeholder(&info(100, 150), &body));
    }

    #[test]
    fn aspect_ratio_is_a_suspicion_not_a_rejection() {
        let body = vec![0u8; 50_000];
        for (w, h) in [(600u32, 600u32), (800, 500), (200, 900)] {
            let a = assess(&ImageInfo::new(Format::Png, w, h), &body);
            assert!(!a.is_placeholder(), "{w}x{h}");
            assert!(
                a.suspicions.iter().any(|s| matches!(s, Suspicion::AspectRatio { .. })),
                "{w}x{h}"
            );
        }
        for (w, h) in [(550u32, 1000u32), (850, 1000), (400, 600), (500, 762)] {
            assert!(assess(&ImageInfo::new(Format::Png, w, h), &body).is_clean(), "{w}x{h}");
        }
    }

    #[test]
    fn low_bits_per_pixel_is_flagged() {
        // 500 x 750 px in 2 kB: about 0.04 bits per pixel.
        let a = assess(&ImageInfo::new(Format::Png, 500, 750), &vec![7u8; 2000]);
        assert!(!a.is_placeholder());
        assert!(a.suspicions.iter().any(|s| matches!(s, Suspicion::LowBitsPerPixel { .. })));
    }

    #[test]
    fn bnf_lengths_depend_on_the_source() {
        let info = ImageInfo::new(Format::Jpeg, 120, 180);
        for len in BNF_PLACEHOLDER_LENGTHS {
            let body = vec![1u8; len as usize];
            let generic = assess(&info, &body);
            assert!(!generic.is_placeholder());
            assert!(generic.suspicions.contains(&Suspicion::PlaceholderLength { bytes: len }));
            let bnf = assess_for_source("bnf", &info, &body);
            assert_eq!(bnf.reasons, [Reason::KnownLength { source: "bnf", bytes: len }]);
            assert!(!bnf.suspicions.iter().any(|s| matches!(s, Suspicion::PlaceholderLength { .. })));
            assert!(!assess_for_source("openlibrary", &info, &body).is_placeholder());
        }
        assert!(!assess_for_source("bnf", &info, &vec![1u8; 4567]).is_placeholder());
        assert!(is_bnf_placeholder_length(4658) && !is_bnf_placeholder_length(4657));
    }

    #[test]
    fn not_an_image_is_rejected() {
        let a = assess_bytes(b"<html><body>404 Not Found</body></html>");
        assert!(a.reasons.contains(&Reason::NotAnImage));
        assert!(a.reasons.contains(&Reason::TinyBody { bytes: 39 }));
        assert!(assess_bytes(&[]).is_placeholder());
    }

    #[test]
    fn placeholder_urls() {
        assert!(is_placeholder_url(
            "https://images-na.example/images/G/01/x-site/icons/no-img-sm.gif"
        ));
        assert!(is_placeholder_url("https://m.media.example/images/I/01RmK+J4pJL.jpg"));
        assert!(!is_placeholder_url("https://covers.openlibrary.org/b/id/123-M.jpg"));
    }

    fn checker(w: u32, h: u32) -> Pixmap {
        let mut p = Pixmap::filled(w, h, [255, 255, 255, 255]);
        for y in 0..h {
            for x in 0..w {
                // A deterministic, busy pattern with many colours.
                let v = (x * 7 + y * 13) % 256;
                p.set(x, y, [v as u8, ((x * 3) % 256) as u8, ((y * 5) % 256) as u8, 255]);
            }
        }
        p
    }

    #[test]
    fn pixel_level_rejects_flat_and_colourless_tiles() {
        let flat = Pixmap::filled(300, 450, [200, 200, 200, 255]);
        let a = assess_pixels(&flat, &PixelThresholds::default(), &[]);
        assert!(a.reasons.iter().any(|r| matches!(r, Reason::NearUniform { .. })));
        assert!(a.reasons.contains(&Reason::FewColours { count: 1 }));
        assert!(is_placeholder_pixels(&flat));

        // Fully transparent pixels count as white: still flat.
        assert!(is_placeholder_pixels(&Pixmap::filled(300, 450, [9, 80, 200, 0])));

        // Grey tile with a darker caption band: varied luminance, but 2 colours.
        let mut tile = Pixmap::filled(300, 450, [220, 220, 220, 255]);
        for y in 200..250 {
            for x in 40..260 {
                tile.set(x, y, [90, 90, 90, 255]);
            }
        }
        let a = assess_pixels(&tile, &PixelThresholds::default(), &[]);
        assert_eq!(a.reasons, [Reason::FewColours { count: 2 }]);

        let busy = checker(300, 450);
        assert!(luma_stddev(&busy) > 20.0);
        assert!(distinct_colours_4bit(&busy) > 100);
        assert!(!is_placeholder_pixels(&busy));
        assert!(assess_pixels(&busy, &PixelThresholds::default(), &[]).is_clean());
    }

    #[test]
    fn pixel_level_small_and_odd_shapes() {
        assert_eq!(
            assess_pixels(&Pixmap::filled(1, 1, [0, 0, 0, 255]), &PixelThresholds::default(), &[])
                .reasons,
            [Reason::OnePixel]
        );
        assert!(is_placeholder_pixels(&checker(60, 90)));
        let square = assess_pixels(&checker(300, 300), &PixelThresholds::default(), &[]);
        assert!(!square.is_placeholder());
        assert!(square.suspicions.iter().any(|s| matches!(s, Suspicion::AspectRatio { .. })));
        // Degenerate shapes must not panic in the hash.
        let _ = dhash(&checker(1, 1));
        let _ = dhash(&checker(3, 2));
        let _ = dhash(&checker(2, 500));
    }

    #[test]
    fn dhash_is_stable_under_small_changes_and_differs_between_images() {
        let a = checker(360, 540);
        // The same picture with mild brightness noise.
        let mut b = a.clone();
        for (i, px) in b.data_mut().chunks_exact_mut(4).enumerate() {
            px[0] = px[0].saturating_add((i % 3) as u8);
        }
        let (ha, hb) = (dhash(&a), dhash(&b));
        assert!(hamming(ha, hb) <= 4, "distance {}", hamming(ha, hb));

        // A left-to-right gradient against its mirror image: every bit flips.
        let mut left = Pixmap::filled(180, 160, [0, 0, 0, 255]);
        let mut right = left.clone();
        for y in 0..160 {
            for x in 0..180u32 {
                let v = (x * 255 / 179) as u8;
                left.set(x, y, [v, v, v, 255]);
                right.set(179 - x, y, [v, v, v, 255]);
            }
        }
        assert_eq!(dhash(&left), 0);
        assert_eq!(dhash(&right), u64::MAX);
        assert_eq!(hamming(dhash(&left), dhash(&right)), 64);

        // Gallery match.
        let thresholds = PixelThresholds::default();
        let hit = assess_pixels(&b, &thresholds, &[0, ha]);
        assert!(hit.reasons.iter().any(|r| matches!(r, Reason::KnownHash { .. })));
        assert!(!assess_pixels(&b, &thresholds, &[!ha]).is_placeholder());
    }

    #[test]
    fn sampling_keeps_large_images_bounded() {
        let big = Pixmap::filled(3000, 2000, [10, 20, 30, 255]);
        assert!(sample_step(&big) >= 2);
        assert_eq!(luma_stddev(&big), 0.0);
        assert_eq!(distinct_colours_4bit(&big), 1);
    }
}
