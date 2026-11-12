//! Preflight a PDF from the command line.
//!
//! ```sh
//! cargo run -p ebc-preflight --example preflight -- card.pdf
//! cargo run -p ebc-preflight --example preflight -- card.pdf --trim 230x178 --spine 10 --json
//! cargo run -p ebc-preflight --example preflight -- flyer.pdf --home
//! ```
//!
//! Exit status: 0 when the file passes, 1 when it has error-level findings, 2 when it cannot
//! be read or the arguments are wrong.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

use ebc_preflight::{Expectations, Report, Severity, check};

const USAGE: &str = "usage: preflight <file.pdf> [--home] [--bleed MM] [--min-ppi N] [--trim WxH] \
                     [--spine MM] [--qr-module MM] [--allow-rgb] [--allow-transparency] [--json]";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (path, expect, json) = match parse(&arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(2);
        }
    };
    let report = match check(&bytes, &expect) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::from(2);
        }
    };
    if json {
        println!("{}", report.to_json_pretty());
    } else {
        print_report(&path, &report);
    }
    if report.passed() { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

fn parse(arguments: &[String]) -> Result<(String, Expectations, bool), String> {
    let mut path = None;
    let mut expect = Expectations::professional();
    let mut json = false;
    let mut iter = arguments.iter();
    while let Some(argument) = iter.next() {
        let mut number = |name: &str| -> Result<f64, String> {
            let value = iter.next().ok_or_else(|| format!("{name} needs a value"))?;
            value.parse::<f64>().map_err(|_| format!("{name}: \"{value}\" is not a number"))
        };
        match argument.as_str() {
            "--home" => expect = Expectations::home(),
            "--json" => json = true,
            "--allow-rgb" => expect.allow_rgb = true,
            "--allow-transparency" => expect.allow_transparency = true,
            "--bleed" => expect.bleed_mm = number("--bleed")?,
            "--min-ppi" => expect.min_ppi = number("--min-ppi")?,
            "--spine" => expect.spine_mm = Some(number("--spine")?),
            "--qr-module" => expect.min_qr_module_mm = Some(number("--qr-module")?),
            "--trim" => {
                let value = iter.next().ok_or("--trim needs a value such as 230x178")?;
                let size = value
                    .split_once(['x', 'X'])
                    .and_then(|(w, h)| Some((w.parse::<f64>().ok()?, h.parse::<f64>().ok()?)))
                    .ok_or_else(|| {
                        format!("--trim: \"{value}\" is not WIDTHxHEIGHT in millimetres")
                    })?;
                expect.trim_mm = Some(size);
            }
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other => {
                if path.replace(other.to_string()).is_some() {
                    return Err("more than one file given".to_string());
                }
            }
        }
    }
    Ok((path.ok_or("no file given")?, expect, json))
}

fn print_report(path: &str, report: &Report) {
    let document = &report.document;
    println!("{path}: PDF {} with {} page(s)", document.pdf_version, document.page_count);
    if let Some(version) = &document.pdfx_version {
        println!("  claims {version}");
    }
    if let Some(intent) = &document.output_intent {
        println!("  output intent: {}", intent.identifier.as_deref().unwrap_or("(no identifier)"));
    }
    for page in &report.pages {
        let trim = page
            .trim_mm
            .map_or_else(|| "no trim box".to_string(), |(w, h)| format!("{w} × {h} mm"));
        let bleed = page.bleed_mm.map_or_else(String::new, |b| format!(", bleed ≥ {} mm", b.min()));
        println!("  page {}: {trim}{bleed}", page.number);
        for font in &page.fonts {
            let state = if font.embedded { "embedded" } else { "NOT embedded" };
            println!("    font  {} ({}, {state})", font.name, font.subtype);
        }
        for image in &page.images {
            println!(
                "    image {} {}×{} px at {:.0} ppi, {}",
                image.name,
                image.width_px,
                image.height_px,
                image.min_ppi(),
                image.color_space
            );
        }
        if !page.color_spaces.is_empty() {
            println!("    colour {}", page.color_spaces.join(", "));
        }
    }
    for finding in &report.findings {
        println!("{finding}");
    }
    let errors = report.errors().count();
    let warnings = report.findings.iter().filter(|f| f.severity == Severity::Warning).count();
    println!(
        "{}: {errors} error(s), {warnings} warning(s)",
        if report.passed() { "PASSED" } else { "FAILED" }
    );
}
