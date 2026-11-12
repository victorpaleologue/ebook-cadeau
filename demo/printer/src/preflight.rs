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
    // Start from the crate's professional defaults and set every field the product
    // decides, so that fields added to `Expectations` later keep sensible values.
    let mut expect = Expectations::professional()
        .with_trim(product.trim_mm[0], product.trim_mm[1])
        .with_spine(product.spine_mm)
        .with_min_qr_module(MIN_QR_MODULE_MM);
    expect.bleed_mm = product.bleed_mm;
    expect.min_ppi = product.min_ppi;
    expect.require_output_intent = product.output_intent_required;
    // RGB (and transparency, which a PDF/X-1a offset workflow cannot carry) is refused only
    // by the products that say they are CMYK-only.
    expect.allow_rgb = !product.cmyk_only;
    expect.allow_transparency = !product.cmyk_only;
    expect.max_total_ink_percent = Some(product.max_total_ink_percent);
    expect
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::testkit::SampleCard;

    fn codes(verdict: &Verdict, severity: &str) -> Vec<String> {
        let mut codes: Vec<String> = verdict
            .findings
            .iter()
            .filter(|f| f.severity == severity)
            .map(|f| f.code.clone())
            .collect();
        codes.sort();
        codes.dedup();
        codes
    }

    fn poche() -> &'static catalog::Product {
        catalog::find("card-poche").unwrap()
    }

    fn offset() -> &'static catalog::Product {
        catalog::find("card-poche-offset").unwrap()
    }

    /// Asserts that `card` is refused for `product` with (at least) the error `code`.
    fn assert_refused(card: &SampleCard, product: &catalog::Product, code: &str) {
        let verdict = run(&card.to_pdf(), product);
        assert!(!verdict.passed, "{code}: the file should have been refused");
        assert!(
            codes(&verdict, "error").iter().any(|c| c == code),
            "expected {code} among {:?}",
            codes(&verdict, "error")
        );
    }

    #[test]
    fn the_sample_card_of_every_product_passes_its_own_preflight() {
        for product in catalog::products() {
            let verdict = run(&SampleCard::for_product(product).to_pdf(), product);
            assert!(verdict.passed, "{}: {:#?}", product.id, verdict.findings);
            assert_eq!(codes(&verdict, "error"), Vec::<String>::new(), "{}", product.id);
            assert_eq!(codes(&verdict, "warning"), Vec::<String>::new(), "{}", product.id);
            assert_eq!(codes(&verdict, "info"), Vec::<String>::new(), "{}", product.id);
        }
    }

    #[test]
    fn a_card_without_a_qr_code_is_accepted_with_a_remark() {
        let card = SampleCard { qr_module_mm: None, ..SampleCard::for_product(poche()) };
        let verdict = run(&card.to_pdf(), poche());
        assert!(verdict.passed);
        assert_eq!(codes(&verdict, "info"), ["QR_NOT_FOUND"]);
        assert_eq!(verdict.warnings().len(), 1, "remarks travel with the accepted order");
    }

    #[test]
    fn expectations_are_derived_from_the_product() {
        let digital = expectations(poche());
        assert_eq!(digital.trim_mm, Some((230.0, 178.0)));
        assert_eq!(digital.spine_mm, Some(10.0));
        assert_eq!((digital.bleed_mm, digital.min_ppi), (3.0, 150.0));
        assert!(digital.require_output_intent);
        assert!(digital.allow_rgb && digital.allow_transparency, "digital products accept RGB");
        assert_eq!(digital.max_total_ink_percent, Some(300.0));
        assert_eq!(digital.min_qr_module_mm, Some(MIN_QR_MODULE_MM));
        let press = expectations(offset());
        assert!(!press.allow_rgb && !press.allow_transparency, "CMYK-only products refuse RGB");
        assert_eq!(expectations(catalog::find("card-a5").unwrap()).trim_mm, Some((306.0, 210.0)));
    }

    #[test]
    fn a_card_of_another_format_is_refused() {
        let a6 = SampleCard::for_product(catalog::find("card-a6").unwrap());
        assert_refused(&a6, poche(), "BOX_TRIM_SIZE");
        // …and the other way round.
        let poche_card = SampleCard::for_product(poche());
        assert_refused(&poche_card, catalog::find("card-a6").unwrap(), "BOX_TRIM_SIZE");
    }

    #[test]
    fn each_broken_rule_is_reported_with_its_code() {
        let good = SampleCard::for_product(poche());
        assert_refused(&SampleCard { bleed_mm: 1.0, ..good.clone() }, poche(), "BOX_BLEED_TOO_SMALL");
        assert_refused(&SampleCard { bleed_mm: 0.0, ..good.clone() }, poche(), "BOX_BLEED_TOO_SMALL");
        assert_refused(&SampleCard { output_intent: false, ..good.clone() }, poche(), "NO_OUTPUT_INTENT");
        assert_refused(&SampleCard { image_ppi: Some(72.0), ..good.clone() }, poche(), "IMAGE_LOW_RES");
        assert_refused(&SampleCard { heavy_ink: true, ..good.clone() }, poche(), "INK_TOTAL_TOO_HIGH");
        assert_refused(&SampleCard { geometry_keys: false, ..good.clone() }, poche(), "EBC_SPINE_MISSING");
        assert_refused(&SampleCard { spine_mm: 14.0, ..good.clone() }, poche(), "EBC_SPINE_MISMATCH");
        assert_refused(&SampleCard { title: None, ..good.clone() }, poche(), "NO_TITLE");
        assert_refused(&SampleCard { qr_module_mm: Some(0.25), ..good.clone() }, poche(), "QR_MODULE_TOO_SMALL");
        assert_refused(&SampleCard { pages: 1, ..good.clone() }, poche(), "PAGE_COUNT");
        assert_refused(&SampleCard { pages: 3, ..good.clone() }, poche(), "PAGE_COUNT");
        // An image just above the minimum is fine.
        assert!(run(&SampleCard { image_ppi: Some(151.0), ..good }.to_pdf(), poche()).passed);
    }

    #[test]
    fn rgb_and_transparency_are_refused_only_by_cmyk_only_products() {
        let rgb = SampleCard { rgb_fill: true, ..SampleCard::for_product(poche()) };
        let translucent = SampleCard { transparency: true, ..SampleCard::for_product(poche()) };
        assert!(run(&rgb.to_pdf(), poche()).passed, "the digital product accepts RGB");
        assert!(run(&translucent.to_pdf(), poche()).passed);
        assert_refused(&rgb, offset(), "COLOR_RGB");
        assert_refused(&translucent, offset(), "TRANSPARENCY");
    }

    #[test]
    fn unreadable_files_are_refused_without_panicking() {
        let good = SampleCard::for_product(poche()).to_pdf();
        let not_pdf = run(b"GIF89a not a pdf", poche());
        assert!(!not_pdf.passed);
        assert_eq!(codes(&not_pdf, "error"), ["NOT_A_PDF"]);
        assert!(!run(b"", poche()).passed);
        assert!(!run(b"%PDF-1.6\n", poche()).passed);
        // Truncations at many points: never a pass, never a panic.
        for cut in (1..good.len()).step_by(good.len() / 60 + 1) {
            let verdict = run(&good[..cut], poche());
            assert!(!verdict.passed, "a file cut at byte {cut} must not be printable");
            assert!(!verdict.findings.is_empty());
        }
        // Bit flips in the structure: any verdict is acceptable, but no panic.
        for position in (0..good.len()).step_by(good.len() / 200 + 1) {
            let mut damaged = good.clone();
            damaged[position] ^= 0x55;
            let _ = run(&damaged, poche());
        }
    }
}
