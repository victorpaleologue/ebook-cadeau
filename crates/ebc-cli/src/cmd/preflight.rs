//! `ebc preflight`: check a PDF against a print profile.

use ebc_preflight::{Expectations, Severity, check};
use ebc_scene::PdfStandard;

use crate::args::PreflightArgs;
use crate::error::{CliError, Status};
use crate::util::{Streams, format_by_name, profile_by_name, read_input};

/// Runs `ebc preflight`.
pub(crate) fn run(args: &PreflightArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    let profile = profile_by_name(&args.profile)?;
    let plain = profile.standard == PdfStandard::Plain;
    // An upload portal wants exactly trim + bleed: no marks, no slug (check B4).
    let upload = profile.sheet.is_none() && !profile.marks.any();
    let mut expect = Expectations {
        bleed_mm: profile.bleed,
        min_ppi: profile.min_ppi,
        require_output_intent: !plain,
        allow_rgb: plain || profile.allow_rgb || args.allow_rgb,
        allow_transparency: false,
        max_total_ink_percent: Some(profile.max_total_ink_percent),
        trim_mm: None,
        spine_mm: None,
        min_qr_module_mm: None,
        marks_allowed: !upload,
        require_media_equals_bleed: upload,
    };
    if !args.generic {
        let format = format_by_name(&args.format)?;
        let spread = format.spread();
        expect.trim_mm = Some((spread.w, spread.h));
        expect.spine_mm = Some(format.spine_width);
        expect.min_qr_module_mm = Some(ebc_qr::MIN_PRINT_MODULE_MM);
    }

    let bytes = read_input(&args.pdf, ebc_preflight::MAX_PDF_BYTES as u64)?;
    let report = check(&bytes, &expect)
        .map_err(|e| CliError::failure(format!("{}: {e}", args.pdf.display())))?;

    if args.json {
        streams.line(report.to_json_pretty())?;
    } else {
        for finding in &report.findings {
            let page = finding.page.map_or_else(String::new, |n| format!(" (page {n})"));
            streams.line(format!(
                "{:<7} {}{page}: {}",
                finding.severity.as_str(),
                finding.code,
                finding.message
            ))?;
        }
        let errors = report.errors().count();
        let warnings = report.findings.iter().filter(|f| f.severity == Severity::Warning).count();
        streams.line(format!(
            "{}: {} — {errors} error(s), {warnings} warning(s), {} page(s), profile {}",
            args.pdf.display(),
            if report.passed() { "PASS" } else { "FAIL" },
            report.document.page_count,
            profile.name
        ))?;
        if !expect.allow_rgb
            && report.document.pdfx_version.is_some()
            && report.errors().any(|f| f.code == ebc_preflight::codes::COLOR_RGB)
        {
            streams.note(
                "hint: cards made by `ebc card` for a professional profile are CMYK; RGB here \
                 means the file was not converted to the output intent. Check again with \
                 --allow-rgb only if your printer converts RGB itself",
            );
        }
    }
    Ok(if report.passed() { Status::Success } else { Status::CheckFailed })
}
