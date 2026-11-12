//! What the caller expects of the PDF, and the errors [`crate::check`] can return.

use serde::{Deserialize, Serialize};

/// Tolerance when comparing the trim size (and the `EBC.*` geometry) with expected values,
/// in millimetres. PDF boxes are written in points with limited precision.
pub const TRIM_TOLERANCE_MM: f64 = 0.1;

/// Tolerance when comparing the bleed with the expected bleed, in millimetres
/// (3 mm is 8.504 pt; a writer rounding to two decimals yields 2.9986 mm).
pub const BLEED_TOLERANCE_MM: f64 = 0.05;

/// Tolerance for the nesting of page boxes, in millimetres.
pub const NESTING_TOLERANCE_MM: f64 = 0.01;

/// What the PDF is expected to satisfy. Build one from a print profile, or start from
/// [`Expectations::professional`] / [`Expectations::home`] and adjust.
///
/// ```
/// use ebc_preflight::Expectations;
///
/// let expect = Expectations::professional().with_trim(230.0, 178.0).with_spine(10.0);
/// assert_eq!(expect.bleed_mm, 3.0);
/// assert_eq!(expect.trim_mm, Some((230.0, 178.0)));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Expectations {
    /// Minimum bleed on every side of the trim box, in millimetres. `0` disables the check.
    pub bleed_mm: f64,
    /// Minimum effective resolution of placed images, in pixels per inch. `0` disables the
    /// check.
    pub min_ppi: f64,
    /// Whether a well-formed PDF/X output intent is required.
    pub require_output_intent: bool,
    /// Whether RGB-like colour (`DeviceRGB`, `CalRGB`, `Lab`, three-component ICC) is accepted.
    pub allow_rgb: bool,
    /// Whether transparency (soft masks, opacity, blend modes, transparency groups) is accepted.
    pub allow_transparency: bool,
    /// Total ink limit (C+M+Y+K) in percent for CMYK fills and strokes; `None` disables the
    /// check.
    pub max_total_ink_percent: Option<f32>,
    /// Expected trim size `(width, height)` in millimetres (the whole spread for a card).
    pub trim_mm: Option<(f64, f64)>,
    /// Expected spine width in millimetres, verified against the `EBC.SpineWidthMm` /
    /// `EBC.FoldXMm` Info keys.
    pub spine_mm: Option<f64>,
    /// Minimum QR module size in millimetres, verified against QR-like vector graphics and
    /// the optional `EBC.QrModuleMm` Info key.
    pub min_qr_module_mm: Option<f64>,
}

impl Default for Expectations {
    /// Same as [`Expectations::professional`].
    fn default() -> Self {
        Self::professional()
    }
}

impl Expectations {
    /// What European online printers commonly ask for: 3 mm bleed, images at 150 ppi or
    /// more (300 is the target; below 150 is visibly soft), a PDF/X output intent, CMYK or
    /// grey only, no transparency, at most 300 % total ink.
    #[must_use]
    pub fn professional() -> Self {
        Self {
            bleed_mm: 3.0,
            min_ppi: 150.0,
            require_output_intent: true,
            allow_rgb: false,
            allow_transparency: false,
            max_total_ink_percent: Some(300.0),
            trim_mm: None,
            spine_mm: None,
            min_qr_module_mm: None,
        }
    }

    /// For printing at home or at a copy shop: no bleed requirement, 100 ppi, RGB and
    /// transparency accepted, no output intent or ink limit.
    #[must_use]
    pub fn home() -> Self {
        Self {
            bleed_mm: 0.0,
            min_ppi: 100.0,
            require_output_intent: false,
            allow_rgb: true,
            allow_transparency: true,
            max_total_ink_percent: None,
            trim_mm: None,
            spine_mm: None,
            min_qr_module_mm: None,
        }
    }

    /// Sets the expected trim size in millimetres.
    #[must_use]
    pub fn with_trim(mut self, width_mm: f64, height_mm: f64) -> Self {
        self.trim_mm = Some((width_mm, height_mm));
        self
    }

    /// Sets the expected spine width in millimetres.
    #[must_use]
    pub fn with_spine(mut self, spine_mm: f64) -> Self {
        self.spine_mm = Some(spine_mm);
        self
    }

    /// Sets the minimum QR module size in millimetres.
    #[must_use]
    pub fn with_min_qr_module(mut self, module_mm: f64) -> Self {
        self.min_qr_module_mm = Some(module_mm);
        self
    }

    /// Checks that every number is finite and non-negative.
    pub(crate) fn validate(&self) -> Result<(), PreflightError> {
        let ok = |v: f64| v.is_finite() && v >= 0.0;
        let bad = |field: &'static str| Err(PreflightError::InvalidExpectations(field));
        if !ok(self.bleed_mm) {
            return bad("bleed_mm");
        }
        if !ok(self.min_ppi) {
            return bad("min_ppi");
        }
        if self.max_total_ink_percent.is_some_and(|v| !ok(f64::from(v))) {
            return bad("max_total_ink_percent");
        }
        if self.trim_mm.is_some_and(|(w, h)| !(ok(w) && ok(h) && w > 0.0 && h > 0.0)) {
            return bad("trim_mm");
        }
        if self.spine_mm.is_some_and(|v| !ok(v)) {
            return bad("spine_mm");
        }
        if self.min_qr_module_mm.is_some_and(|v| !ok(v)) {
            return bad("min_qr_module_mm");
        }
        Ok(())
    }
}

/// Why a file could not be preflighted at all. Problems *in* a readable PDF are reported as
/// [`crate::Finding`]s instead.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PreflightError {
    /// The data does not start with a `%PDF-` header (within the first 1024 bytes).
    #[error("not a PDF file: no %PDF- header")]
    NotPdf,
    /// The file is larger than [`crate::MAX_PDF_BYTES`].
    #[error("the file is too large to preflight: {size} bytes (limit {limit})")]
    TooLarge {
        /// Size of the input in bytes.
        size: usize,
        /// The limit in bytes.
        limit: usize,
    },
    /// The PDF structure (cross-reference table, trailer, objects) could not be parsed.
    #[error("the PDF cannot be parsed: {0}")]
    Parse(String),
    /// The file parses but has no document catalog.
    #[error("the PDF has no document catalog")]
    NoCatalog,
    /// A value in [`Expectations`] is negative, NaN or infinite.
    #[error("invalid expectation: {0} must be a finite, non-negative number")]
    InvalidExpectations(&'static str),
    /// A bug: the checker (or the PDF parser underneath) panicked on this input. The panic
    /// was contained; please report the file.
    #[error("internal error while preflighting: {0}")]
    Internal(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn presets_and_builders() {
        let pro = Expectations::default();
        assert_eq!(pro, Expectations::professional());
        assert!(pro.require_output_intent && !pro.allow_rgb && !pro.allow_transparency);
        assert_eq!(pro.max_total_ink_percent, Some(300.0));
        let home = Expectations::home().with_trim(230.0, 178.0).with_spine(10.0).with_min_qr_module(0.4);
        assert!(home.allow_rgb && home.allow_transparency && !home.require_output_intent);
        assert_eq!(home.trim_mm, Some((230.0, 178.0)));
        assert_eq!(home.spine_mm, Some(10.0));
        assert_eq!(home.min_qr_module_mm, Some(0.4));
        assert!(home.validate().is_ok());
    }

    #[test]
    fn rejects_nonsense_numbers() {
        let cases: Vec<(Expectations, &str)> = vec![
            (Expectations { bleed_mm: -1.0, ..Expectations::default() }, "bleed_mm"),
            (Expectations { bleed_mm: f64::NAN, ..Expectations::default() }, "bleed_mm"),
            (Expectations { min_ppi: f64::INFINITY, ..Expectations::default() }, "min_ppi"),
            (
                Expectations { max_total_ink_percent: Some(f32::NAN), ..Expectations::default() },
                "max_total_ink_percent",
            ),
            (Expectations::default().with_trim(0.0, 100.0), "trim_mm"),
            (Expectations::default().with_trim(100.0, f64::NAN), "trim_mm"),
            (Expectations::default().with_spine(-0.1), "spine_mm"),
            (Expectations::default().with_min_qr_module(f64::NEG_INFINITY), "min_qr_module_mm"),
        ];
        for (expect, field) in cases {
            assert_eq!(expect.validate(), Err(PreflightError::InvalidExpectations(field)));
        }
    }

    #[test]
    fn json_round_trip_with_defaults() {
        let expect: Expectations =
            serde_json::from_str(r#"{"bleed_mm": 2, "trim_mm": [230, 178], "allow_rgb": true}"#).unwrap();
        assert_eq!(expect.bleed_mm, 2.0);
        assert_eq!(expect.trim_mm, Some((230.0, 178.0)));
        assert!(expect.allow_rgb);
        // Unspecified fields take the professional defaults.
        assert_eq!(expect.min_ppi, 150.0);
        assert!(expect.require_output_intent);
        let json = serde_json::to_string(&expect).unwrap();
        assert_eq!(serde_json::from_str::<Expectations>(&json).unwrap(), expect);
    }
}
