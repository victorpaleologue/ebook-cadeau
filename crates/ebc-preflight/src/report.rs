//! The result of a preflight run: findings plus a per-page and per-document summary.

use std::collections::BTreeMap;

use serde::Serialize;

/// How serious a [`Finding`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The file must not be sent to a printer as it is.
    Error,
    /// The file will print, but something deserves a human look.
    Warning,
    /// Informational only.
    Info,
}

impl Severity {
    /// Lower-case name, as used in the JSON report (`"error"`, `"warning"`, `"info"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One problem (or remark) found in the PDF.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// How serious it is.
    pub severity: Severity,
    /// Stable machine-readable code, one of the constants in [`crate::codes`].
    pub code: &'static str,
    /// 1-based page number, or `None` for document-level findings.
    pub page: Option<u32>,
    /// Human-readable explanation (English).
    pub message: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.page {
            Some(page) => {
                write!(f, "{} {} (page {}): {}", self.severity, self.code, page, self.message)
            }
            None => write!(f, "{} {}: {}", self.severity, self.code, self.message),
        }
    }
}

/// A page box in millimetres, in PDF default user space (origin bottom-left, y up).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BoxMm {
    /// Left edge.
    pub x0: f64,
    /// Bottom edge.
    pub y0: f64,
    /// Right edge.
    pub x1: f64,
    /// Top edge.
    pub y1: f64,
}

impl BoxMm {
    /// Width in millimetres.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    /// Height in millimetres.
    #[must_use]
    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    /// Whether `inner` lies inside `self`, allowing `tolerance` millimetres of overshoot.
    #[must_use]
    pub fn contains(&self, inner: &BoxMm, tolerance: f64) -> bool {
        inner.x0 >= self.x0 - tolerance
            && inner.y0 >= self.y0 - tolerance
            && inner.x1 <= self.x1 + tolerance
            && inner.y1 <= self.y1 + tolerance
    }

    /// Whether the two boxes overlap by more than `tolerance` millimetres in both directions.
    #[must_use]
    pub fn overlaps(&self, other: &BoxMm, tolerance: f64) -> bool {
        other.x1 > self.x0 + tolerance
            && other.x0 < self.x1 - tolerance
            && other.y1 > self.y0 + tolerance
            && other.y0 < self.y1 - tolerance
    }
}

/// Distance between the trim box and the bleed box on each side, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BleedMm {
    /// Left side.
    pub left: f64,
    /// Right side.
    pub right: f64,
    /// Top side.
    pub top: f64,
    /// Bottom side.
    pub bottom: f64,
}

impl BleedMm {
    /// The smallest of the four sides.
    #[must_use]
    pub fn min(&self) -> f64 {
        self.left.min(self.right).min(self.top).min(self.bottom)
    }
}

/// A font found in a page's resources (including those of nested form XObjects).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FontSummary {
    /// `BaseFont` name as written in the PDF (with its subset prefix, if any).
    pub name: String,
    /// Font `Subtype`: `Type0`, `Type1`, `MMType1`, `TrueType`, `Type3`, or `CIDFontType0`/
    /// `CIDFontType2` appended after a slash for composite fonts (e.g. `Type0/CIDFontType2`).
    pub subtype: String,
    /// Whether a font program (`FontFile`, `FontFile2` or `FontFile3`) is embedded.
    /// Type 3 fonts are reported as embedded: their glyphs are content streams.
    pub embedded: bool,
    /// Whether the name carries a six-letter subset prefix (`ABCDEF+Name`).
    pub subset: bool,
    /// The subset prefix without the `+`, when [`FontSummary::subset`] is true.
    pub subset_prefix: Option<String>,
    /// Whether a text-showing operator on the page actually uses the font.
    pub used: bool,
}

/// One placement of a raster image on a page.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageSummary {
    /// Resource name used by the `Do` operator, or `"(inline)"` for inline images.
    pub name: String,
    /// Width in pixels.
    pub width_px: u32,
    /// Height in pixels.
    pub height_px: u32,
    /// Printed width in millimetres (after all transforms).
    pub width_mm: f64,
    /// Printed height in millimetres (after all transforms).
    pub height_mm: f64,
    /// Effective horizontal resolution in pixels per inch.
    pub ppi_x: f64,
    /// Effective vertical resolution in pixels per inch.
    pub ppi_y: f64,
    /// Colour space, e.g. `DeviceCMYK`, `ICCBased(3)`, `Indexed(DeviceRGB)`, `ImageMask`.
    pub color_space: String,
    /// Bits per colour component.
    pub bits_per_component: u8,
    /// Stream filters, e.g. `DCTDecode`.
    pub filters: Vec<String>,
    /// Whether the image carries a soft mask (alpha channel).
    pub soft_mask: bool,
    /// Whether it is an inline image (`BI … ID … EI`).
    pub inline: bool,
}

impl ImageSummary {
    /// The lower of the horizontal and vertical effective resolutions.
    #[must_use]
    pub fn min_ppi(&self) -> f64 {
        self.ppi_x.min(self.ppi_y)
    }
}

/// A vector graphic that looks like a QR code: one fill made of many axis-aligned rectangles
/// on a common module grid with a square outline.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QrCandidate {
    /// Estimated module size in millimetres.
    pub module_mm: f64,
    /// Side of the symbol (without quiet zone) in millimetres.
    pub side_mm: f64,
    /// Estimated modules per side.
    pub modules: u32,
}

/// What was found on one page.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageSummary {
    /// 1-based page number.
    pub number: u32,
    /// `MediaBox` (inherited through the page tree), scaled by `UserUnit`.
    pub media_box: Option<BoxMm>,
    /// `CropBox`, when present.
    pub crop_box: Option<BoxMm>,
    /// `BleedBox`, when present.
    pub bleed_box: Option<BoxMm>,
    /// `TrimBox`, when present.
    pub trim_box: Option<BoxMm>,
    /// `ArtBox`, when present.
    pub art_box: Option<BoxMm>,
    /// Trim size `(width, height)` in millimetres, when a `TrimBox` is present.
    pub trim_mm: Option<(f64, f64)>,
    /// Bleed on each side: distance from the trim box to the bleed box (or, when the page
    /// has no `BleedBox`, to the box that takes its place: `CropBox`, else `MediaBox`).
    pub bleed_mm: Option<BleedMm>,
    /// Page `Rotate` value in degrees (0, 90, 180 or 270).
    pub rotate: i32,
    /// Fonts in the page's resources, sorted by name.
    pub fonts: Vec<FontSummary>,
    /// Every image placement, in painting order.
    pub images: Vec<ImageSummary>,
    /// Colour spaces actually used to paint (fills, strokes, text, images, shadings), sorted.
    pub color_spaces: Vec<String>,
    /// Highest total ink coverage (C+M+Y+K, in percent) of a CMYK fill or stroke that
    /// touches the bleed box, when the page paints with `DeviceCMYK` at all.
    pub max_total_ink_percent: Option<f32>,
    /// Whether the page uses any transparency feature.
    pub transparency: bool,
    /// Vector graphics that look like QR codes.
    pub qr_candidates: Vec<QrCandidate>,
}

/// The document's output intent, as far as it could be read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutputIntentSummary {
    /// The `S` subtype, normally `GTS_PDFX`.
    pub subtype: String,
    /// `OutputConditionIdentifier`, e.g. `FOGRA39`.
    pub identifier: Option<String>,
    /// `Info` / `OutputCondition` text.
    pub description: Option<String>,
    /// `RegistryName`.
    pub registry: Option<String>,
    /// Whether a `DestOutputProfile` is embedded.
    pub profile_embedded: bool,
    /// `N` of the embedded profile stream.
    pub profile_components: Option<u8>,
    /// Colour space signature from the ICC header (`CMYK`, `RGB`, `GRAY`, …).
    pub profile_color_space: Option<String>,
}

/// Document-level facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct DocumentSummary {
    /// Version from the file header, e.g. `1.6`.
    pub pdf_version: String,
    /// Number of pages.
    pub page_count: u32,
    /// Whether the file is encrypted.
    pub encrypted: bool,
    /// Info `Title`.
    pub title: Option<String>,
    /// Info `Trapped` (`True`, `False` or `Unknown`).
    pub trapped: Option<String>,
    /// The PDF/X version the file claims (Info `GTS_PDFXVersion` or XMP
    /// `pdfxid:GTS_PDFXVersion`), e.g. `PDF/X-4`.
    pub pdfx_version: Option<String>,
    /// The PDF/X output intent, if any.
    pub output_intent: Option<OutputIntentSummary>,
    /// The `EBC.*` keys of the Info dictionary, e.g. `EBC.SpineWidthMm` → `10`.
    pub ebc: BTreeMap<String, String>,
}

/// Everything a preflight run found.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    /// Problems and remarks, document-level first, then page by page.
    pub findings: Vec<Finding>,
    /// One summary per page, in page order.
    pub pages: Vec<PageSummary>,
    /// Document-level facts.
    pub document: DocumentSummary,
}

impl Report {
    /// `true` when there is no finding of [`Severity::Error`].
    #[must_use]
    pub fn passed(&self) -> bool {
        !self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    /// The findings of [`Severity::Error`].
    pub fn errors(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Error)
    }

    /// The findings of [`Severity::Warning`].
    pub fn warnings(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Warning)
    }

    /// Whether any finding carries `code`.
    #[must_use]
    pub fn has(&self, code: &str) -> bool {
        self.findings.iter().any(|f| f.code == code)
    }

    /// The distinct finding codes, sorted.
    #[must_use]
    pub fn codes(&self) -> Vec<&'static str> {
        let mut codes: Vec<&'static str> = self.findings.iter().map(|f| f.code).collect();
        codes.sort_unstable();
        codes.dedup();
        codes
    }

    /// Serialises the report as compact JSON, with an extra top-level `"passed"` flag.
    ///
    /// The output is deterministic: same PDF and expectations, same string. Non-finite
    /// numbers (which the checks avoid producing) would be written as `null`.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&JsonReport::from(self)).unwrap_or_else(|_| fallback_json())
    }

    /// Like [`Report::to_json`], indented for humans.
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&JsonReport::from(self)).unwrap_or_else(|_| fallback_json())
    }
}

/// The JSON shape: the report plus the derived `passed` flag.
#[derive(Serialize)]
struct JsonReport<'a> {
    passed: bool,
    findings: &'a [Finding],
    pages: &'a [PageSummary],
    document: &'a DocumentSummary,
}

impl<'a> From<&'a Report> for JsonReport<'a> {
    fn from(report: &'a Report) -> Self {
        Self {
            passed: report.passed(),
            findings: &report.findings,
            pages: &report.pages,
            document: &report.document,
        }
    }
}

/// Serialising these plain data types cannot fail; this keeps `to_json` total anyway.
fn fallback_json() -> String {
    String::from(r#"{"passed":false,"findings":[],"pages":[],"document":null}"#)
}

/// Rounds to three decimals: millimetre values in summaries are derived from PDF points and
/// would otherwise carry noise such as `178.00000000000003`.
pub(crate) fn round3(v: f64) -> f64 {
    if v.is_finite() { (v * 1000.0).round() / 1000.0 } else { 0.0 }
}

/// Rounds to one decimal (resolutions).
pub(crate) fn round1(v: f64) -> f64 {
    if v.is_finite() { (v * 10.0).round() / 10.0 } else { 0.0 }
}
