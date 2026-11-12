//! Cross-checks against independent tools, when they are installed: PDFs rewritten by
//! `qpdf` or produced by Ghostscript must be read the same way `pdffonts`, `pdfimages` and
//! `pdfinfo` (poppler) read them. Each test skips itself when a tool is missing.

#![allow(clippy::unwrap_used)]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{Fixture, card_expectations, codes, set};
use ebc_preflight::{Expectations, Report, check, codes as c};

fn tool(name: &str, version_flag: &str) -> bool {
    let found = Command::new(name)
        .arg(version_flag)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !found {
        eprintln!("skipping: `{name}` is not installed");
    }
    found
}

fn has_qpdf() -> bool {
    tool("qpdf", "--version")
}

fn has_gs() -> bool {
    tool("gs", "--version")
}

/// A scratch directory removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(test: &str) -> Self {
        Self(common::scratch_dir(test))
    }

    fn path(&self, file: &str) -> PathBuf {
        self.0.join(file)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(command: &mut Command) -> (bool, String) {
    let output = command.output().unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.success(), text)
}

fn write_reference(scratch: &Scratch) -> PathBuf {
    let path = scratch.path("reference.pdf");
    std::fs::write(&path, Fixture::reference().bytes()).unwrap();
    path
}

fn preflight(path: &Path, expect: &Expectations) -> Report {
    check(&std::fs::read(path).unwrap(), expect).unwrap()
}

// ---- qpdf ----------------------------------------------------------------------------------

#[test]
fn qpdf_accepts_the_reference_fixture() {
    if !has_qpdf() {
        return;
    }
    let scratch = Scratch::new("qpdf-check");
    let mut fixture = Fixture::reference();
    for (name, bytes) in [("classic.pdf", fixture.bytes()), ("modern.pdf", fixture.bytes_modern())] {
        let path = scratch.path(name);
        std::fs::write(&path, bytes).unwrap();
        let (ok, output) = run(Command::new("qpdf").arg("--check").arg(&path));
        assert!(ok, "qpdf --check {name} failed:\n{output}");
        assert!(output.contains("No syntax or stream encoding errors found"), "{output}");
    }
}

#[test]
fn files_rewritten_by_qpdf_give_the_same_report() {
    if !has_qpdf() {
        return;
    }
    let scratch = Scratch::new("qpdf-rewrite");
    let input = write_reference(&scratch);
    let original = preflight(&input, &card_expectations());
    assert!(original.findings.is_empty());
    let variants: [(&str, &[&str]); 5] = [
        ("objstm", &["--object-streams=generate"]),
        ("linearized", &["--linearize"]),
        ("recompressed", &["--compress-streams=y", "--recompress-flate", "--compression-level=9"]),
        ("qdf", &["--qdf", "--object-streams=disable"]),
        ("normalized", &["--normalize-content=y", "--stream-data=uncompress"]),
    ];
    for (name, arguments) in variants {
        let output = scratch.path(&format!("{name}.pdf"));
        let (ok, text) = run(Command::new("qpdf").args(arguments).arg(&input).arg(&output));
        assert!(ok, "qpdf {arguments:?} failed:\n{text}");
        let report = preflight(&output, &card_expectations());
        assert!(report.findings.is_empty(), "{name}: {:#?}", report.findings);
        assert_eq!(report.pages, original.pages, "{name}");
        assert_eq!(report.document.title, original.document.title, "{name}");
        assert_eq!(report.document.output_intent, original.document.output_intent, "{name}");
        assert_eq!(report.document.ebc, original.document.ebc, "{name}");
    }
}

#[test]
fn files_encrypted_by_qpdf_are_flagged() {
    if !has_qpdf() {
        return;
    }
    let scratch = Scratch::new("qpdf-encrypt");
    let input = write_reference(&scratch);
    // (label, user password, key length, extra flags)
    let variants: [(&str, &str, &str, &[&str]); 4] = [
        ("aes256-open", "", "256", &[]),
        ("aes256-locked", "secret", "256", &[]),
        ("aes128-open", "", "128", &["--use-aes=y"]),
        ("rc4-locked", "secret", "128", &["--use-aes=n"]),
    ];
    for (name, user_password, bits, extra) in variants {
        let output = scratch.path(&format!("{name}.pdf"));
        let (ok, text) = run(Command::new("qpdf")
            .arg("--allow-weak-crypto")
            .arg("--encrypt")
            .arg(user_password)
            .arg("owner")
            .arg(bits)
            .args(extra)
            .arg("--")
            .arg(&input)
            .arg(&output));
        assert!(ok, "qpdf --encrypt ({name}) failed:\n{text}");
        let report = preflight(&output, &card_expectations());
        assert!(report.has(c::ENCRYPTED), "{name}: {:#?}", report.findings);
        assert!(!report.passed(), "{name}");
        assert!(report.document.encrypted, "{name}");
        if user_password.is_empty() {
            // No password needed: the content was read, and nothing else is wrong with it —
            // except that 256-bit encryption makes qpdf raise the file to PDF 1.7, which a
            // PDF/X-4 file must not be.
            let expected: &[&'static str] =
                if bits == "256" { &[c::ENCRYPTED, c::PDFX_VERSION_MISMATCH] } else { &[c::ENCRYPTED] };
            assert_eq!(codes(&report), set(expected), "{name}: {:#?}", report.findings);
            assert_eq!(report.pages.len(), 2, "{name}");
            assert_eq!(report.pages[0].images.len(), 1, "{name}");
        } else {
            assert_eq!(codes(&report), set(&[c::ENCRYPTED]), "{name}: {:#?}", report.findings);
            assert!(report.pages.is_empty(), "{name}");
        }
    }
}

// ---- poppler -------------------------------------------------------------------------------

#[test]
fn page_boxes_agree_with_pdfinfo() {
    if !tool("pdfinfo", "-v") {
        return;
    }
    let scratch = Scratch::new("pdfinfo");
    let input = write_reference(&scratch);
    let (ok, text) = run(Command::new("pdfinfo").arg("-box").arg(&input));
    assert!(ok, "{text}");
    let report = preflight(&input, &card_expectations());
    let page = &report.pages[0];
    let poppler_box = |label: &str| -> [f64; 4] {
        let line = text.lines().find(|l| l.starts_with(label)).unwrap_or_else(|| panic!("no {label} in\n{text}"));
        let numbers: Vec<f64> = line[label.len()..].split_whitespace().map(|n| n.parse().unwrap()).collect();
        [numbers[0], numbers[1], numbers[2], numbers[3]]
    };
    let ours = |b: ebc_preflight::BoxMm| [b.x0, b.y0, b.x1, b.y1].map(|mm| mm * common::MM);
    for (label, mine) in [
        ("MediaBox:", page.media_box.unwrap()),
        ("BleedBox:", page.bleed_box.unwrap()),
        ("TrimBox:", page.trim_box.unwrap()),
    ] {
        let theirs = poppler_box(label);
        for (a, b) in ours(mine).iter().zip(theirs) {
            assert!((a - b).abs() < 0.011, "{label} ours {:?} vs pdfinfo {theirs:?}", ours(mine));
        }
    }
    assert!(text.contains("Pages:           2"), "{text}");
}

/// `(name, embedded, subset)` per font, as `pdffonts` reports them.
fn pdffonts(path: &Path) -> Vec<(String, bool, bool)> {
    let (ok, text) = run(Command::new("pdffonts").arg(path));
    assert!(ok, "{text}");
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            // name … emb sub uni object-number generation
            let n = tokens.len();
            (n >= 7).then(|| (tokens[0].to_string(), tokens[n - 5] == "yes", tokens[n - 4] == "yes"))
        })
        .collect()
}

/// `(width px, height px, x-ppi, y-ppi, inline)` per image, as `pdfimages -list` reports them.
fn pdfimages(path: &Path) -> Vec<(u32, u32, f64, f64, bool)> {
    let (ok, text) = run(Command::new("pdfimages").arg("-list").arg(path));
    assert!(ok, "{text}");
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let n = tokens.len();
            if n < 12 {
                return None;
            }
            // page num type width height … x-ppi y-ppi size ratio
            Some((
                tokens[3].parse().ok()?,
                tokens[4].parse().ok()?,
                tokens[n - 4].parse().ok()?,
                tokens[n - 3].parse().ok()?,
                line.contains("[inline]"),
            ))
        })
        .collect()
}

const POSTSCRIPT: &str = "%!PS
<< /PageSize [300 200] >> setpagedevice
/Helvetica findfont 12 scalefont setfont
1 0 0 setrgbcolor 20 150 moveto (Bonjour) show
/Times-Roman findfont 10 scalefont setfont
0 0 0 1 setcmykcolor 20 130 moveto (le monde) show
0.2 0.4 0.6 0.1 setcmykcolor 20 20 50 50 rectfill
% A 10 x 10 pixel image on one inch: 10 ppi (Ghostscript writes it inline).
gsave 100 20 translate 72 72 scale
10 10 8 [10 0 0 -10 0 10] { <IMAGE_SMALL> } image
grestore
% A 120 x 90 pixel image on half an inch by one inch, rotated: 240 x 90 ppi.
gsave 230 30 translate 20 rotate 36 72 scale
120 90 8 [120 0 0 -90 0 90] { <IMAGE_LARGE> } image
grestore
showpage
";

fn ghostscript(scratch: &Scratch, name: &str, embed_fonts: bool) -> PathBuf {
    let source = scratch.path("input.ps");
    let gradient = |n: usize| -> String { (0..n).map(|i| format!("{:02x}", (i * 7) % 256)).collect() };
    let postscript = POSTSCRIPT.replace("IMAGE_SMALL", &gradient(100)).replace("IMAGE_LARGE", &gradient(120 * 90));
    std::fs::write(&source, postscript).unwrap();
    let output = scratch.path(name);
    let mut command = Command::new("gs");
    command
        .args(["-q", "-dNOPAUSE", "-dBATCH", "-dSAFER", "-sDEVICE=pdfwrite", "-dCompatibilityLevel=1.4"])
        .args(["-dDownsampleGrayImages=false", "-dAutoFilterGrayImages=false", "-dGrayImageFilter=/FlateEncode"])
        .arg(format!("-sOutputFile={}", output.display()));
    if embed_fonts {
        command.args(["-dEmbedAllFonts=true", "-dSubsetFonts=true", "-c", "<< /NeverEmbed [ ] >> setdistillerparams"]);
    }
    let (ok, text) = run(command.arg("-f").arg(&source));
    assert!(ok, "gs failed:\n{text}");
    output
}

#[test]
fn ghostscript_output_fonts_agree_with_pdffonts() {
    if !has_gs() || !tool("pdffonts", "-v") {
        return;
    }
    let scratch = Scratch::new("gs-fonts");
    let mut saw_embedded = false;
    let mut saw_missing = false;
    for (name, embed) in [("embedded.pdf", true), ("not-embedded.pdf", false)] {
        let path = ghostscript(&scratch, name, embed);
        let report = preflight(&path, &Expectations::professional());
        let mut ours: Vec<(String, bool, bool)> =
            report.pages[0].fonts.iter().map(|f| (f.name.clone(), f.embedded, f.subset)).collect();
        let mut theirs = pdffonts(&path);
        ours.sort();
        theirs.sort();
        assert_eq!(ours, theirs, "{name}: our font list differs from pdffonts");
        assert_eq!(ours.len(), 2, "{name}");
        assert!(report.pages[0].fonts.iter().all(|f| f.used), "{name}");
        let all_embedded = ours.iter().all(|(_, embedded, _)| *embedded);
        assert_eq!(report.has(c::FONT_NOT_EMBEDDED), !all_embedded, "{name}: {:#?}", report.findings);
        saw_embedded |= all_embedded;
        saw_missing |= !all_embedded;
    }
    // The two Ghostscript invocations are meant to cover both outcomes.
    assert!(saw_embedded && saw_missing, "expected one embedded and one non-embedded variant");
}

#[test]
fn ghostscript_output_images_agree_with_pdfimages() {
    if !has_gs() || !tool("pdfimages", "-v") {
        return;
    }
    let scratch = Scratch::new("gs-images");
    let path = ghostscript(&scratch, "images.pdf", true);
    let report = preflight(&path, &Expectations::professional());
    let theirs = pdfimages(&path);
    let ours = &report.pages[0].images;
    assert_eq!(ours.len(), theirs.len(), "ours {ours:#?}\npdfimages {theirs:?}");
    assert_eq!(ours.len(), 2);
    for (mine, (width, height, x_ppi, y_ppi, inline)) in ours.iter().zip(&theirs) {
        assert_eq!((mine.width_px, mine.height_px), (*width, *height));
        assert_eq!(mine.inline, *inline, "{mine:?}");
        // pdfimages prints whole numbers.
        assert!((mine.ppi_x - x_ppi).abs() <= 1.0, "{mine:?} vs {x_ppi}");
        assert!((mine.ppi_y - y_ppi).abs() <= 1.0, "{mine:?} vs {y_ppi}");
        assert_eq!(mine.color_space, "DeviceGray");
    }
    // The numbers themselves: 10 px on one inch; 120 × 90 px on half an inch by one inch.
    assert!((ours[0].ppi_x - 10.0).abs() < 0.1 && (ours[0].ppi_y - 10.0).abs() < 0.1, "{:?}", ours[0]);
    assert!((ours[1].ppi_x - 240.0).abs() < 0.5 && (ours[1].ppi_y - 90.0).abs() < 0.5, "{:?}", ours[1]);
    assert!((ours[1].width_mm - 12.7).abs() < 0.05 && (ours[1].height_mm - 25.4).abs() < 0.05, "{:?}", ours[1]);

    // And the verdicts a printer would reach on this file.
    let low_res: Vec<_> = report.findings.iter().filter(|f| f.code == c::IMAGE_LOW_RES).collect();
    assert_eq!(low_res.len(), 2, "10 ppi and 90 ppi are both below 150: {:#?}", report.findings);
    assert!(report.has(c::COLOR_RGB), "the red text is DeviceRGB: {:#?}", report.findings);
    assert!(report.has(c::NO_OUTPUT_INTENT));
    assert!(!report.has(c::CONTENT_INVALID) && !report.has(c::XREF_DAMAGED), "{:#?}", report.findings);
    let spaces = &report.pages[0].color_spaces;
    for expected in ["DeviceCMYK", "DeviceGray", "DeviceRGB"] {
        assert!(spaces.iter().any(|s| s == expected), "{expected} missing from {spaces:?}");
    }
    // 20 + 40 + 60 + 10 = 130 % is the heaviest CMYK fill.
    assert_eq!(report.pages[0].max_total_ink_percent, Some(130.0));
    let raw = std::fs::read(&path).unwrap();
    let has_trim_box = raw.windows(8).any(|w| w == b"/TrimBox");
    assert_eq!(report.has(c::BOX_TRIM_MISSING), !has_trim_box);
}
