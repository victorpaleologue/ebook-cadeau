//! QR code generation tuned for print: compact payloads, vector output, size rules.
//!
//! The card's QR code carries the claim URL of a gifted eBook. On paper three things
//! matter, and this crate takes care of each:
//!
//! * **Few, large modules.** For a given printed size, a lower QR *version* means bigger
//!   modules and easier scanning. [`QrCode::encode`] picks the smallest version that holds
//!   the text, splitting it into the cheapest mix of numeric, alphanumeric and byte
//!   segments, and [`QrCode::encode_url`] additionally upper-cases the case-insensitive
//!   parts of a URL so that they fit the compact alphanumeric mode (see
//!   [`normalize_url`] for the exact rule).
//! * **Clean vector output.** [`QrCode::rects`] merges the dark modules into a small set of
//!   non-overlapping rectangles and [`QrCode::to_path`] turns them into one
//!   [`ebc_scene::Path`], to be filled with K-only black ([`ebc_scene::Color::BLACK`]).
//!   Neighbouring rectangles share bit-identical coordinates, so no hairlines appear
//!   between modules.
//! * **Size rules.** [`QUIET_ZONE_MODULES`], [`MIN_PRINT_MODULE_MM`] and the helpers
//!   [`QrCode::side_mm`], [`QrCode::module_mm_for_side`] and [`QrCode::fit_in`] let the
//!   layout check that the symbol is printable before it reaches a printer.
//!
//! Encoding itself is delegated to the MIT-licensed [`qrcodegen`] crate (Project Nayuki);
//! output is fully deterministic. The crate performs no I/O and compiles to
//! `wasm32-unknown-unknown`.
//!
//! # Example
//!
//! ```
//! use ebc_qr::{EcLevel, QrCode, QUIET_ZONE_MODULES, MIN_PRINT_MODULE_MM};
//! use ebc_scene::{Point, Rect};
//!
//! let qr = QrCode::encode_url("https://ebk.example/K/7K3M9QX2RD", EcLevel::M)?;
//! assert_eq!(qr.version(), 2); // 25 x 25 modules; the lower-case URL in byte mode needs version 3
//!
//! // Fit the symbol and its quiet zone into a 30 mm box on the page.
//! let placement = qr.fit_in(Rect::new(40.0, 60.0, 30.0, 30.0), QUIET_ZONE_MODULES);
//! assert!(placement.module_mm >= MIN_PRINT_MODULE_MM);
//! let path = qr.to_path(placement.origin, placement.module_mm);
//! assert!(!path.is_empty());
//! # Ok::<(), ebc_qr::QrError>(())
//! ```

#![forbid(unsafe_code)]

mod segment;
mod url;

use std::fmt;

use ebc_scene::{Path, PathSeg, Point, Rect};
use qrcodegen::{QrCodeEcc, QrSegment, Version};

use crate::segment::{Mode, VersionGroup};
pub use crate::url::normalize_url;

/// Width of the mandatory blank margin around a QR symbol, in modules (ISO/IEC 18004).
/// Phone cameras often cope with less; a printed card should not rely on it.
pub const QUIET_ZONE_MODULES: u32 = 4;

/// Smallest module size, in millimetres, that should be sent to print.
///
/// GS1 gives about 0.396 mm as the minimum X-dimension for codes scanned by phones;
/// below that, dot gain and camera resolution make decoding unreliable. Source:
/// `docs/research` prepress notes §7 (flagged there as not yet re-verified).
pub const MIN_PRINT_MODULE_MM: f64 = 0.4;

/// Comfortable module size, in millimetres, for a card held at reading distance
/// (same source and caveat as [`MIN_PRINT_MODULE_MM`]).
pub const RECOMMENDED_PRINT_MODULE_MM: f64 = 0.5;

/// Longest text, in bytes, that is even attempted: 7089 digits is the capacity of the
/// largest QR code (version 40, level L, numeric mode). Longer input is rejected up front.
pub const MAX_PAYLOAD_BYTES: usize = 7089;

/// Error-correction level: how much of the symbol may be damaged or obscured.
///
/// Higher levels need more modules for the same text. `M` is the usual choice for print;
/// `Q` if the card is likely to get scuffed; `H` only when a logo covers part of the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EcLevel {
    /// Low: about 7 % of the codewords can be restored.
    L,
    /// Medium: about 15 %.
    M,
    /// Quartile: about 25 %.
    Q,
    /// High: about 30 %.
    H,
}

impl EcLevel {
    /// All levels, weakest first.
    pub const ALL: [Self; 4] = [Self::L, Self::M, Self::Q, Self::H];

    /// Approximate share of codewords that can be restored, in percent (7, 15, 25 or 30).
    #[must_use]
    pub fn recovery_percent(self) -> u8 {
        match self {
            Self::L => 7,
            Self::M => 15,
            Self::Q => 25,
            Self::H => 30,
        }
    }

    fn to_backend(self) -> QrCodeEcc {
        match self {
            Self::L => QrCodeEcc::Low,
            Self::M => QrCodeEcc::Medium,
            Self::Q => QrCodeEcc::Quartile,
            Self::H => QrCodeEcc::High,
        }
    }

    fn from_backend(level: QrCodeEcc) -> Self {
        match level {
            QrCodeEcc::Low => Self::L,
            QrCodeEcc::Medium => Self::M,
            QrCodeEcc::Quartile => Self::Q,
            QrCodeEcc::High => Self::H,
        }
    }
}

/// Why a QR code could not be produced.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum QrError {
    /// The text to encode is empty. A blank code is legal but never what a card needs.
    #[error("nothing to encode: the payload is empty")]
    Empty,
    /// The text does not fit in the largest QR code (version 40) at the requested level.
    #[error(
        "the payload ({bytes} bytes) does not fit in a QR code at error-correction level {min_ec:?}"
    )]
    TooLong {
        /// Length of the rejected text in bytes.
        bytes: usize,
        /// The minimum error-correction level that was requested.
        min_ec: EcLevel,
    },
    /// The text passed to [`QrCode::encode_url`] is not a URL this crate can normalise.
    #[error("not an encodable URL: {0}")]
    InvalidUrl(&'static str),
}

/// Where to draw a symbol so that it fills a given box; see [`QrCode::fit_in`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Top-left corner of the symbol's first module (inside the quiet zone), in page
    /// millimetres — the `top_left` argument of [`QrCode::to_path`].
    pub origin: Point,
    /// Side of one module in millimetres — compare with [`MIN_PRINT_MODULE_MM`].
    pub module_mm: f64,
}

/// An immutable QR code symbol: a square matrix of dark and light modules.
#[derive(Clone, PartialEq, Eq)]
pub struct QrCode {
    size: u32,
    version: u8,
    ec_level: EcLevel,
    mask: u8,
    /// `size * size` modules in row-major order; `true` = dark.
    modules: Vec<bool>,
}

impl QrCode {
    /// Encodes `text` in the smallest QR version that can hold it with at least `min_ec`
    /// error correction.
    ///
    /// * The text is split into the cheapest mix of numeric, alphanumeric and byte
    ///   segments (bytes are the text's UTF-8 encoding, without an ECI header — what phone
    ///   scanners expect).
    /// * If the chosen version has room for a stronger error-correction level, the level
    ///   is raised for free; see [`ec_level`](Self::ec_level).
    /// * The mask pattern is chosen by the standard penalty rules. The same input always
    ///   produces the same symbol.
    ///
    /// # Errors
    ///
    /// [`QrError::Empty`] for an empty text, [`QrError::TooLong`] if it does not fit in a
    /// version 40 symbol at `min_ec` (at most 2953 bytes, 4296 alphanumeric characters or
    /// 7089 digits at level L).
    pub fn encode(text: &str, min_ec: EcLevel) -> Result<Self, QrError> {
        if text.is_empty() {
            return Err(QrError::Empty);
        }
        let too_long = QrError::TooLong { bytes: text.len(), min_ec };
        if text.len() > MAX_PAYLOAD_BYTES {
            return Err(too_long);
        }
        for group in VersionGroup::ALL {
            // The cheapest segmentation depends on the width of the length fields, which
            // is constant within a group of versions; trying the groups in ascending
            // order therefore finds the smallest possible version.
            let segments: Vec<QrSegment> = segment::optimal_runs(text, group)
                .into_iter()
                .map(|run| match run.mode {
                    Mode::Numeric => QrSegment::make_numeric(run.text),
                    Mode::Alphanumeric => QrSegment::make_alphanumeric(run.text),
                    Mode::Byte => QrSegment::make_bytes(run.text.as_bytes()),
                })
                .collect();
            let (lowest, highest) = group.versions();
            let encoded = qrcodegen::QrCode::encode_segments_advanced(
                &segments,
                min_ec.to_backend(),
                Version::new(lowest),
                Version::new(highest),
                None,
                true,
            );
            if let Ok(code) = encoded {
                return Ok(Self::from_backend(&code));
            }
        }
        Err(too_long)
    }

    /// Encodes a URL after rewriting it into its most compact equivalent form with
    /// [`normalize_url`]: the scheme and the host are upper-cased so that they (and an
    /// already upper-case path) use alphanumeric mode; a path, query or fragment
    /// containing lower-case letters is never changed and is stored in byte mode.
    ///
    /// The symbol decodes to the normalised URL, e.g. `HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD` —
    /// compare a scanned code with [`normalize_url`]`(url)`, not with `url` itself.
    ///
    /// # Errors
    ///
    /// Those of [`normalize_url`] and of [`encode`](Self::encode). A URL longer than
    /// [`MAX_PAYLOAD_BYTES`] is reported as [`QrError::TooLong`] without being parsed.
    pub fn encode_url(url: &str, min_ec: EcLevel) -> Result<Self, QrError> {
        if url.len() > MAX_PAYLOAD_BYTES {
            return Err(QrError::TooLong { bytes: url.len(), min_ec });
        }
        Self::encode(&normalize_url(url)?, min_ec)
    }

    fn from_backend(code: &qrcodegen::QrCode) -> Self {
        let side = code.size();
        let mut modules = Vec::with_capacity((side * side).max(0) as usize);
        for y in 0..side {
            for x in 0..side {
                modules.push(code.get_module(x, y));
            }
        }
        Self {
            size: side.max(0) as u32,
            version: code.version().value(),
            ec_level: EcLevel::from_backend(code.error_correction_level()),
            mask: code.mask().value(),
            modules,
        }
    }

    /// Number of modules per side, without the quiet zone: `17 + 4 × version` (21–177).
    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The QR version, 1–40.
    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }

    /// The error-correction level actually used: the requested minimum, or a stronger one
    /// if it fitted in the same version.
    #[must_use]
    pub fn ec_level(&self) -> EcLevel {
        self.ec_level
    }

    /// The mask pattern applied to the data modules, 0–7.
    #[must_use]
    pub fn mask(&self) -> u8 {
        self.mask
    }

    /// Whether the module in column `x`, row `y` is dark. `(0, 0)` is the top-left module;
    /// coordinates outside the symbol (the quiet zone and beyond) are light.
    #[must_use]
    pub fn module(&self, x: u32, y: u32) -> bool {
        if x >= self.size || y >= self.size {
            return false;
        }
        let index = y as usize * self.size as usize + x as usize;
        self.modules.get(index).copied().unwrap_or(false)
    }

    /// Number of dark modules.
    #[must_use]
    pub fn dark_count(&self) -> usize {
        self.modules.iter().filter(|&&dark| dark).count()
    }

    /// The dark modules as a minimal-ish set of rectangles `(x, y, w, h)` in module units.
    ///
    /// Each row is cut into maximal horizontal runs of dark modules; runs that have the
    /// same extent in consecutive rows are then stacked into one taller rectangle. The
    /// rectangles never overlap, together cover exactly the dark modules, and are sorted
    /// by `(y, x)`. A version 2 symbol needs roughly 120 rectangles instead of about 330
    /// single modules, which keeps the PDF small and free of seams.
    #[must_use]
    pub fn rects(&self) -> Vec<(u32, u32, u32, u32)> {
        let mut rects = Vec::new();
        // Runs of the previous row that may still grow downwards: (x, w, first row).
        let mut open: Vec<(u32, u32, u32)> = Vec::new();
        for y in 0..self.size {
            let mut still_open = Vec::with_capacity(open.len());
            let mut previous = open.iter().copied().peekable();
            let mut x = 0;
            while x < self.size {
                if !self.module(x, y) {
                    x += 1;
                    continue;
                }
                let start = x;
                while x < self.size && self.module(x, y) {
                    x += 1;
                }
                let width = x - start;
                // Close every open run that starts before this one.
                while let Some(&(px, pw, py)) = previous.peek() {
                    if px >= start {
                        break;
                    }
                    rects.push((px, py, pw, y - py));
                    previous.next();
                }
                match previous.peek() {
                    Some(&(px, pw, py)) if px == start && pw == width => {
                        still_open.push((px, pw, py));
                        previous.next();
                    }
                    _ => still_open.push((start, width, y)),
                }
            }
            for (px, pw, py) in previous {
                rects.push((px, py, pw, y - py));
            }
            open = still_open;
        }
        for (px, pw, py) in open {
            rects.push((px, py, pw, self.size - py));
        }
        rects.sort_unstable_by_key(|&(x, y, _, _)| (y, x));
        rects
    }

    /// The dark modules as one vector path of closed rectangles (see [`rects`](Self::rects)),
    /// ready to be filled with [`ebc_scene::Color::BLACK`] under either fill rule.
    ///
    /// `top_left` is the page position of the top-left corner of module `(0, 0)` — the
    /// symbol proper, *not* the quiet zone — and `module_mm` the side of one module. The
    /// path contains dark modules only: the caller must keep [`QUIET_ZONE_MODULES`] modules
    /// of white (or a very light tint) around it and is responsible for the background.
    ///
    /// Every coordinate is computed as `top_left + n × module_mm` for an integer `n`, so
    /// rectangles that touch share exactly the same coordinate values.
    ///
    /// A `module_mm` that is not a finite positive number, or a non-finite `top_left`,
    /// yields an empty path rather than a panic or garbage geometry.
    #[must_use]
    pub fn to_path(&self, top_left: Point, module_mm: f64) -> Path {
        let mut path = Path::new();
        let usable = module_mm.is_finite()
            && module_mm > 0.0
            && top_left.x.is_finite()
            && top_left.y.is_finite();
        if !usable {
            return path;
        }
        let at = |x: u32, y: u32| {
            Point::new(top_left.x + f64::from(x) * module_mm, top_left.y + f64::from(y) * module_mm)
        };
        let rects = self.rects();
        path.segs.reserve(rects.len() * 5);
        for (x, y, w, h) in rects {
            path.segs.extend([
                PathSeg::MoveTo(at(x, y)),
                PathSeg::LineTo(at(x + w, y)),
                PathSeg::LineTo(at(x + w, y + h)),
                PathSeg::LineTo(at(x, y + h)),
                PathSeg::Close,
            ]);
        }
        path
    }

    /// Side of the printed symbol in millimetres, including `quiet_modules` modules of
    /// quiet zone on every side: `(size + 2 × quiet_modules) × module_mm`.
    #[must_use]
    pub fn side_mm(&self, module_mm: f64, quiet_modules: u32) -> f64 {
        self.side_modules(quiet_modules) * module_mm
    }

    /// The module size, in millimetres, at which the symbol plus `quiet_modules` modules
    /// of quiet zone on every side is exactly `side_mm` wide — the inverse of
    /// [`side_mm`](Self::side_mm). Compare the result with [`MIN_PRINT_MODULE_MM`].
    #[must_use]
    pub fn module_mm_for_side(&self, side_mm: f64, quiet_modules: u32) -> f64 {
        side_mm / self.side_modules(quiet_modules)
    }

    /// Centres the symbol, surrounded by `quiet_modules` modules of quiet zone, in `outer`
    /// at the largest size that fits, and returns where and how large to draw it.
    ///
    /// The caller fills `outer` (or at least the centred square) with white, then fills
    /// `to_path(placement.origin, placement.module_mm)` with black.
    #[must_use]
    pub fn fit_in(&self, outer: Rect, quiet_modules: u32) -> Placement {
        let side = outer.w.min(outer.h);
        let module_mm = self.module_mm_for_side(side, quiet_modules);
        let quiet = f64::from(quiet_modules) * module_mm;
        Placement {
            origin: Point::new(
                outer.x + (outer.w - side) / 2.0 + quiet,
                outer.y + (outer.h - side) / 2.0 + quiet,
            ),
            module_mm,
        }
    }

    /// A standalone SVG image of the symbol: a white square including `quiet_modules`
    /// modules of quiet zone, with the dark modules as a single black path.
    ///
    /// The image is `(size + 2 × quiet_modules) × module_px` pixels wide; a `module_px`
    /// of 0 is treated as 1. Intended for on-screen previews — print output goes through
    /// [`to_path`](Self::to_path).
    #[must_use]
    pub fn to_svg(&self, module_px: u32, quiet_modules: u32) -> String {
        use fmt::Write as _;

        let quiet = u64::from(quiet_modules);
        let side = u64::from(self.size) + 2 * quiet;
        let side_px = u128::from(side) * u128::from(module_px.max(1));
        let mut data = String::new();
        for (x, y, w, h) in self.rects() {
            // Writing to a String cannot fail.
            let _ =
                write!(data, "M{},{}h{w}v{h}h-{w}z", u64::from(x) + quiet, u64::from(y) + quiet);
        }
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{side_px}\" height=\"{side_px}\" \
             viewBox=\"0 0 {side} {side}\" shape-rendering=\"crispEdges\">\
             <rect width=\"{side}\" height=\"{side}\" fill=\"#ffffff\"/>\
             <path d=\"{data}\" fill=\"#000000\"/></svg>"
        )
    }

    /// Modules per side including the quiet zone, as a float.
    fn side_modules(&self, quiet_modules: u32) -> f64 {
        f64::from(self.size) + 2.0 * f64::from(quiet_modules)
    }
}

impl fmt::Debug for QrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QrCode")
            .field("version", &self.version)
            .field("size", &self.size)
            .field("ec_level", &self.ec_level)
            .field("mask", &self.mask)
            .finish_non_exhaustive()
    }
}
