//! Preflight of an uploaded PDF against a product, delegated to the `ebc-preflight` crate.

use ebc_preflight::{Expectations, PreflightError, Severity};
use serde::Serialize;

use crate::catalog::Product;

/// Smallest QR module the printer guarantees to reproduce cleanly, in millimetres.
pub const MIN_QR_MODULE_MM: f64 = 0.4;

/// One preflight finding, as returned in API responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FindingBody {
    /// `error`, `warning` or `info`.
    pub severity: &'static str,
    /// Stable machine code (see `ebc_preflight::codes`, plus `UNREADABLE` and `PAGE_COUNT`
    /// raised by the printer itself).
    pub code: String,
    /// 1-based page number, when the finding is about one page.
    pub page: Option<u32>,
    /// Human-readable explanation (English).
    pub message: String,
}

/// The verdict on an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// Whether the file can be printed as it is.
    pub passed: bool,
    /// Every finding, most severe first as reported.
    pub findings: Vec<FindingBody>,
}

impl Verdict {
    /// The findings below error level (kept on accepted orders).
    #[must_use]
    pub fn warnings(&self) -> Vec<FindingBody> {
        self.findings.iter().filter(|f| f.severity != "error").cloned().collect()
    }
}

/// What a PDF must satisfy to be printed as `product`.
#[must_use]
pub fn expectations(product: &Product) -> Expectations {
    Expectations {
        bleed_mm: product.bleed_mm,
        min_ppi: product.min_ppi,
        require_output_intent: product.output_intent_required,
        // RGB (and transparency, which a PDF/X-1a offset workflow cannot carry) is refused
        // only by the products that say they are CMYK-only.
        allow_rgb: !product.cmyk_only,
        allow_transparency: !product.cmyk_only,
        max_total_ink_percent: Some(product.max_total_ink_percent),
        trim_mm: Some((product.trim_mm[0], product.trim_mm[1])),
        spine_mm: Some(product.spine_mm),
        min_qr_module_mm: Some(MIN_QR_MODULE_MM),
    }
}

/// Preflights `pdf` for `product`. Never panics, whatever the bytes.
#[must_use]
pub fn run(pdf: &[u8], product: &Product) -> Verdict {
    let report = match ebc_preflight::check(pdf, &expectations(product)) {
        Ok(report) => report,
        Err(error) => {
            let code = match error {
                PreflightError::NotPdf => "NOT_A_PDF",
                PreflightError::TooLarge { .. } => "TOO_LARGE",
                _ => "UNREADABLE",
            };
            return Verdict {
                passed: false,
                findings: vec![FindingBody {
                    severity: "error",
                    code: code.to_owned(),
                    page: None,
                    message: error.to_string(),
                }],
            };
        }
    };
    let mut findings: Vec<FindingBody> = report
        .findings
        .iter()
        .map(|f| FindingBody {
            severity: match f.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            },
            code: f.code.to_owned(),
            page: f.page,
            message: f.message.clone(),
        })
        .collect();
    // The printer's own rule: one page per side of the sheet.
    let pages = u32::try_from(report.pages.len()).unwrap_or(u32::MAX);
    if pages != product.pages {
        findings.push(FindingBody {
            severity: "error",
            code: "PAGE_COUNT".to_owned(),
            page: None,
            message: format!(
                "the file has {pages} page(s); this product needs exactly {} (outside spread, inside spread)",
                product.pages
            ),
        });
    }
    let passed = !findings.iter().any(|f| f.severity == "error");
    Verdict { passed, findings }
}
