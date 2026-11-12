//! Hostile and broken input: truncated files, random bytes, corrupted copies of a valid
//! file, reference cycles, recursion bombs, decompression bombs and floods.
//!
//! The contract under test: `check` returns `Err` or a report with findings — it never
//! panics (a contained panic would surface as `PreflightError::Internal`) and never hangs
//! (every call here runs under a watchdog).

#![allow(clippy::unwrap_used)]

mod common;

use std::sync::mpsc;
use std::time::Duration;

use common::{Fixture, Rng, card_expectations, codes, set};
use ebc_preflight::{Expectations, PreflightError, Report, Severity, check, codes as c};
use lopdf::{Object, Stream, dictionary};

/// Runs `check` on another thread and fails the test if it does not come back in time or
/// if a panic had to be contained.
fn bounded(pdf: &[u8], expect: &Expectations) -> Result<Report, PreflightError> {
    const WATCHDOG: Duration = Duration::from_secs(120);
    let (sender, receiver) = mpsc::channel();
    let data = pdf.to_vec();
    let expect = expect.clone();
    std::thread::spawn(move || {
        let _ = sender.send(check(&data, &expect));
    });
    let result = receiver.recv_timeout(WATCHDOG).expect("check() did not finish in time or crashed");
    if let Err(PreflightError::Internal(message)) = &result {
        panic!("check() panicked internally: {message}");
    }
    if let Ok(report) = &result {
        // Whatever was found must serialise to valid JSON.
        let json = report.to_json();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "invalid JSON: {json}");
    }
    result
}

/// Assembles a PDF from raw object bodies (object `n` is `bodies[n - 1]`), with a correct
/// cross-reference table and a trailer made of `trailer` (the inside of the dictionary).
fn raw_pdf(bodies: &[Vec<u8>], trailer: &str) -> Vec<u8> {
    let mut out = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(bodies.len());
    for (index, body) in bodies.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", bodies.len() + 1).as_bytes());
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {} {trailer} >>\nstartxref\n{xref}\n%%EOF\n", bodies.len() + 1).as_bytes(),
    );
    out
}

fn stream_body(dict: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< {dict} /Length {} >>\nstream\n", data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}

fn body(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

/// A one-page document whose page content and resources are given raw.
fn one_page(content: &[u8], resources: &str, extra_objects: &[Vec<u8>]) -> Vec<u8> {
    let mut bodies = vec![
        body("<< /Type /Catalog /Pages 2 0 R >>"),
        body("<< /Type /Pages /Kids [3 0 R] /Count 1 >>"),
        body(&format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /TrimBox [10 10 290 190] \
             /Contents 4 0 R /Resources << {resources} >> >>"
        )),
        stream_body("", content),
    ];
    bodies.extend_from_slice(extra_objects);
    raw_pdf(&bodies, "/Root 1 0 R")
}

fn home() -> Expectations {
    Expectations::home()
}

// ---- sanity of the helpers -----------------------------------------------------------------

#[test]
fn the_raw_builder_produces_readable_pdfs() {
    let pdf = one_page(b"0 0 0 1 k 20 20 50 50 re f", "", &[]);
    let report = bounded(&pdf, &home()).unwrap();
    assert_eq!(report.pages.len(), 1);
    assert_eq!(report.pages[0].color_spaces, vec!["DeviceCMYK".to_string()]);
    assert!(!report.has(c::XREF_DAMAGED), "{:#?}", report.findings);
    assert!(report.passed(), "{:#?}", report.findings);
}

// ---- not a PDF -----------------------------------------------------------------------------

#[test]
fn things_that_are_not_pdfs_are_errors() {
    for data in [&b""[..], b"%PDF", b"PK\x03\x04 zip", b"<html>%PDF</html>", b"\x00\x00\x00"] {
        assert_eq!(bounded(data, &home()), Err(PreflightError::NotPdf), "{data:?}");
    }
    for data in [&b"%PDF-"[..], b"%PDF-1.4", b"%PDF-1.7\n%%EOF\n", b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\n"] {
        match bounded(data, &home()) {
            Err(PreflightError::Parse(_) | PreflightError::NoCatalog) => {}
            other => panic!("{:?} should not be readable: {other:?}", String::from_utf8_lossy(data)),
        }
    }
    // The error says something useful.
    let error = bounded(b"%PDF-1.4\nnothing else", &home()).unwrap_err();
    assert!(error.to_string().starts_with("the PDF cannot be parsed"), "{error}");
}

#[test]
fn invalid_expectations_are_rejected_before_parsing() {
    let pdf = one_page(b"", "", &[]);
    let bad = Expectations { min_ppi: f64::NAN, ..Expectations::home() };
    assert_eq!(check(&pdf, &bad), Err(PreflightError::InvalidExpectations("min_ppi")));
    assert_eq!(check(b"not a pdf", &bad), Err(PreflightError::InvalidExpectations("min_ppi")));
}

// ---- truncation ----------------------------------------------------------------------------

fn assert_truncations_are_caught(bytes: &[u8]) {
    let intact = bounded(bytes, &card_expectations()).unwrap();
    assert!(intact.findings.is_empty(), "{:#?}", intact.findings);
    let startxref = bytes.windows(9).rposition(|w| w == b"startxref").unwrap();
    let step = (bytes.len() / 300).max(1);
    let mut cuts: Vec<usize> = (0..bytes.len()).step_by(step).collect();
    cuts.extend((bytes.len().saturating_sub(40)..bytes.len()).filter(|cut| cut % 3 == 0));
    let mut reports = 0;
    for cut in cuts {
        match bounded(&bytes[..cut], &card_expectations()) {
            Err(_) => {}
            Ok(report) => {
                reports += 1;
                if cut <= startxref {
                    assert!(
                        !report.passed(),
                        "a file cut at byte {cut} of {} passed: {:#?}",
                        bytes.len(),
                        report.findings
                    );
                    assert!(
                        report.has(c::XREF_DAMAGED),
                        "cut at {cut}: expected XREF_DAMAGED among {:?}",
                        report.codes()
                    );
                }
            }
        }
    }
    assert!(reports > 0, "some truncated files should be recoverable enough to report on");
}

#[test]
fn truncated_files_never_panic_and_never_pass() {
    let mut fixture = Fixture::reference();
    assert_truncations_are_caught(&fixture.bytes());
}

#[test]
fn truncated_files_with_object_streams_never_panic_and_never_pass() {
    let mut fixture = Fixture::reference();
    assert_truncations_are_caught(&fixture.bytes_modern());
}

// ---- random input --------------------------------------------------------------------------

#[test]
fn random_bytes_never_panic() {
    let fragments: [&[u8]; 16] = [
        b"%PDF-1.4\n",
        b" obj\n",
        b"\nendobj\n",
        b"stream\n",
        b"\nendstream",
        b"xref\n0 3\n",
        b"trailer\n<< /Root 1 0 R /Size 3 >>",
        b"startxref\n9\n%%EOF",
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 10 10] /Contents 4 0 R >>",
        b"1 0 obj",
        b"2 0 obj",
        b"3 0 obj",
        b"/Length 5 /Filter /FlateDecode",
        b"0000000015 00000 n \n",
    ];
    let mut rng = Rng::new(0xEBC0_0001);
    let mut parsed = 0;
    for round in 0..600 {
        let mut data = Vec::new();
        if round % 2 == 0 {
            data.extend_from_slice(b"%PDF-1.5\n");
        }
        let pieces = 1 + rng.below(12);
        for _ in 0..pieces {
            if rng.below(3) == 0 {
                let n = rng.below(200);
                data.extend_from_slice(&rng.bytes(n));
            } else {
                data.extend_from_slice(fragments[rng.below(fragments.len())]);
            }
        }
        if bounded(&data, &card_expectations()).is_ok() {
            parsed += 1;
        }
    }
    // Byte soup essentially never forms a document; the point is that nothing blew up.
    assert!(parsed < 600);
}

#[test]
fn corrupted_copies_of_a_valid_file_never_panic() {
    let mut fixture = Fixture::reference();
    let variants = [fixture.bytes(), fixture.bytes_modern()];
    let mut rng = Rng::new(0xEBC0_0002);
    let (mut errors, mut reports, mut clean) = (0, 0, 0);
    for round in 0..700 {
        let mut data = variants[round % 2].clone();
        match rng.below(5) {
            // Overwrite a few random bytes.
            0 | 1 => {
                for _ in 0..1 + rng.below(8) {
                    let at = rng.below(data.len());
                    data[at] = rng.next() as u8;
                }
            }
            // Delete a range.
            2 => {
                let start = rng.below(data.len());
                let end = (start + 1 + rng.below(200)).min(data.len());
                data.drain(start..end);
            }
            // Duplicate a range somewhere else.
            3 => {
                let start = rng.below(data.len());
                let end = (start + 1 + rng.below(200)).min(data.len());
                let chunk = data[start..end].to_vec();
                let at = rng.below(data.len());
                data.splice(at..at, chunk);
            }
            // Swap digits, which shifts offsets, lengths, object numbers and coordinates.
            _ => {
                for _ in 0..1 + rng.below(6) {
                    let at = rng.below(data.len());
                    if data[at].is_ascii_digit() {
                        data[at] = b'0' + (rng.below(10) as u8);
                    }
                }
            }
        }
        match bounded(&data, &card_expectations()) {
            Err(_) => errors += 1,
            Ok(report) if report.findings.is_empty() => clean += 1,
            Ok(_) => reports += 1,
        }
    }
    // Most corruptions must be noticed one way or the other. (A changed byte inside the
    // dummy image or font data, or a digit replaced by itself, legitimately goes unnoticed.)
    assert!(reports + errors > 350, "errors {errors}, reports with findings {reports}, clean {clean}");
}

// ---- broken structure ----------------------------------------------------------------------

#[test]
fn documents_without_usable_pages() {
    // No pages at all.
    let pdf = raw_pdf(
        &[body("<< /Type /Catalog /Pages 2 0 R >>"), body("<< /Type /Pages /Kids [] /Count 0 >>")],
        "/Root 1 0 R",
    );
    let report = bounded(&pdf, &home()).unwrap();
    assert!(report.has(c::NO_PAGES) && !report.passed(), "{:#?}", report.findings);
    assert!(report.pages.is_empty());

    // The catalog has no Pages; Pages is a number; Kids is a name.
    for bodies in [
        vec![body("<< /Type /Catalog >>")],
        vec![body("<< /Type /Catalog /Pages 2 0 R >>"), body("42")],
        vec![body("<< /Type /Catalog /Pages 2 0 R >>"), body("<< /Type /Pages /Kids /Oops >>")],
        vec![body("<< /Type /Catalog /Pages 2 0 R >>"), body("<< /Type /Pages /Kids [5 0 R 6 0 R] >>")],
    ] {
        let report = bounded(&raw_pdf(&bodies, "/Root 1 0 R"), &home()).unwrap();
        assert!(report.has(c::PAGE_TREE_INVALID), "{:#?}", report.findings);
        assert!(report.has(c::NO_PAGES));
    }

    // No catalog: the root is missing, dangling, or not a dictionary.
    assert!(bounded(&raw_pdf(&[body("<< >>")], ""), &home()).is_err());
    assert_eq!(bounded(&raw_pdf(&[body("<< >>")], "/Root 7 0 R"), &home()), Err(PreflightError::NoCatalog));
    assert_eq!(bounded(&raw_pdf(&[body("(catalog)")], "/Root 1 0 R"), &home()), Err(PreflightError::NoCatalog));
}

#[test]
fn page_tree_cycles_terminate() {
    let pdf = raw_pdf(
        &[
            body("<< /Type /Catalog /Pages 2 0 R >>"),
            body("<< /Type /Pages /Kids [2 0 R 3 0 R 2 0 R 4 0 R] /Count 2 >>"),
            body("<< /Type /Pages /Parent 2 0 R /Kids [2 0 R 3 0 R 4 0 R] >>"),
            body("<< /Type /Page /Parent 3 0 R /MediaBox [0 0 100 100] /TrimBox [0 0 100 100] >>"),
        ],
        "/Root 1 0 R",
    );
    let report = bounded(&pdf, &home()).unwrap();
    assert_eq!(report.pages.len(), 1, "the single real page is found exactly once");
    assert!(report.has(c::PAGE_TREE_INVALID));
    assert!(!report.passed());
}

#[test]
fn reference_cycles_in_every_corner_terminate() {
    let content = b"q /X Do /Y Do /C cs 1 scn 0 0 5 5 re f /D cs 1 scn 0 0 5 5 re f /G gs /H gs \
                    /Pattern cs /P scn 0 0 5 5 re f /S sh BT /F1 9 Tf (a) Tj /F2 9 Tf (b) Tj ET Q";
    let pdf = raw_pdf(
        &[
            // 1: catalog — every optional entry points at the self-referential object 9.
            body(
                "<< /Type /Catalog /Pages 2 0 R /OutputIntents 9 0 R /Metadata 9 0 R /Names 9 0 R \
                 /AcroForm 9 0 R /OpenAction 9 0 R /OCProperties 9 0 R /Version 9 0 R >>",
            ),
            // 2: page tree with itself among its kids and cyclic inherited attributes.
            body("<< /Type /Pages /Kids [3 0 R 2 0 R] /Count 2 /MediaBox 9 0 R /Resources 9 0 R /Rotate 9 0 R >>"),
            // 3: the page.
            body(
                "<< /Type /Page /Parent 2 0 R /Contents 10 0 R /Resources 4 0 R /MediaBox [0 0 9 0 R 100] \
                 /TrimBox 9 0 R /BleedBox [0 0 10 10] /Annots 9 0 R /Group 9 0 R /UserUnit 9 0 R /AA 9 0 R >>",
            ),
            // 4: resources shared by the page and by the form that draws itself.
            body(
                "<< /Font << /F1 5 0 R /F2 9 0 R >> /XObject << /X 6 0 R /Y 9 0 R >> \
                 /ColorSpace << /C 7 0 R /D 9 0 R >> /ExtGState << /G 9 0 R /H 8 0 R >> \
                 /Pattern << /P 9 0 R >> /Shading << /S 9 0 R >> >>",
            ),
            // 5: a composite font that is its own descendant and descriptor.
            body("<< /Type /Font /Subtype /Type0 /BaseFont /Loop /DescendantFonts [5 0 R] /FontDescriptor 5 0 R >>"),
            // 6: a form that draws itself.
            stream_body("/Type /XObject /Subtype /Form /BBox [0 0 1 1] /Resources 4 0 R", b"/X Do /Y Do"),
            // 7: an Indexed colour space based on itself.
            body("[/Indexed 7 0 R 255 <00>]"),
            // 8: a graphics state full of cycles.
            body("<< /SMask 8 0 R /ca 9 0 R /BM [9 0 R 8 0 R] /Font [5 0 R 10] /LW 9 0 R >>"),
            // 9: an object that is a reference to itself.
            body("9 0 R"),
            // 10: a Contents array that contains itself and the real stream.
            body("[10 0 R 11 0 R]"),
            stream_body("", content),
        ],
        "/Root 1 0 R /Info 9 0 R /ID 9 0 R",
    );
    let report = bounded(&pdf, &Expectations::professional()).unwrap();
    assert_eq!(report.pages.len(), 1);
    assert!(!report.passed());
    // The form recursion is named, the font without a program is caught, the rest is noise.
    let recursion = report.findings.iter().any(|f| f.code == c::CONTENT_INVALID && f.message.contains("draws itself"));
    assert!(recursion, "{:#?}", report.findings);
    assert!(report.has(c::FONT_NOT_EMBEDDED), "{:#?}", report.findings);
    assert!(report.has(c::PAGE_TREE_INVALID), "{:#?}", report.findings);
}

#[test]
fn form_recursion_is_detected() {
    // A draws B, B draws A.
    let a = stream_body("/Type /XObject /Subtype /Form /BBox [0 0 9 9] /Resources << /XObject << /B 6 0 R >> >>", b"/B Do");
    let b = stream_body("/Type /XObject /Subtype /Form /BBox [0 0 9 9] /Resources << /XObject << /A 5 0 R >> >>", b"/A Do");
    let pdf = one_page(b"/A Do 0 0 0 1 k 0 0 9 9 re f", "/XObject << /A 5 0 R >>", &[a, b]);
    let report = bounded(&pdf, &home()).unwrap();
    assert_eq!(codes(&report), set(&[c::CONTENT_INVALID, c::PDFX_NOT_CLAIMED, c::NO_TITLE, c::TRAPPED_NOT_SET]));
    let cycle = report.findings.iter().find(|f| f.code == c::CONTENT_INVALID).unwrap();
    assert!(cycle.message.contains("form XObject /A draws itself"), "{}", cycle.message);
    // The page content after the recursive form is still analysed.
    assert_eq!(report.pages[0].color_spaces, vec!["DeviceCMYK".to_string()]);

    // A tiling pattern whose cell paints with itself.
    let cell = stream_body(
        "/PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 4 4] /XStep 4 /YStep 4 \
         /Resources << /Pattern << /P 5 0 R >> >>",
        b"/Pattern cs /P scn 0 0 2 2 re f",
    );
    let pdf = one_page(b"/Pattern cs /P scn 0 0 50 50 re f", "/Pattern << /P 5 0 R >>", &[cell]);
    let report = bounded(&pdf, &home()).unwrap();
    assert_eq!(report.pages.len(), 1);
}

#[test]
fn form_bombs_exhaust_the_budget_not_the_machine() {
    // Fifteen forms, each drawing the next one ten times: 10^15 leaf invocations if followed.
    let mut extra = Vec::new();
    for level in 0..15 {
        let next = 5 + level + 1;
        let content = if level == 14 {
            b"0 0 1 1 re f ".repeat(10)
        } else {
            b"/N Do ".repeat(10)
        };
        extra.push(stream_body(
            &format!("/Type /XObject /Subtype /Form /BBox [0 0 9 9] /Resources << /XObject << /N {next} 0 R >> >>"),
            &content,
        ));
    }
    let pdf = one_page(b"/N Do", "/XObject << /N 5 0 R >>", &extra);
    let report = bounded(&pdf, &home()).unwrap();
    let limit = report.findings.iter().find(|f| f.code == c::LIMIT_EXCEEDED).expect("LIMIT_EXCEEDED");
    assert_eq!(limit.severity, Severity::Error);
    assert!(limit.message.contains("too complex"), "{}", limit.message);
    assert!(!report.passed(), "a partially analysed file must not pass");
    assert_eq!(report.findings.iter().filter(|f| f.code == c::LIMIT_EXCEEDED).count(), 1);
}

#[test]
fn the_operator_budget_bounds_the_work_per_document() {
    // 30 000 invocations of a form with 1 000 operators: 30 million operators in a file of
    // a few hundred kilobytes. The budget is 20 million.
    let form = stream_body("/Type /XObject /Subtype /Form /BBox [0 0 9 9]", &b"n ".repeat(1_000));
    let pdf = one_page(&b"/N Do ".repeat(30_000), "/XObject << /N 5 0 R >>", &[form]);
    assert!(pdf.len() < 400_000);
    let report = bounded(&pdf, &home()).unwrap();
    let limits: Vec<_> = report.findings.iter().filter(|f| f.code == c::LIMIT_EXCEEDED).collect();
    assert_eq!(limits.len(), 1, "{:#?}", report.findings);
    assert_eq!(limits[0].severity, Severity::Error);
    assert!(!report.passed());
}

#[test]
fn forms_nested_too_deeply_are_cut_off() {
    // A chain of 40 forms, each drawing the next once.
    let mut extra = Vec::new();
    for level in 0..40 {
        let next = 5 + level + 1;
        let (resources, content) = if level == 39 {
            (String::new(), &b"1 0 0 rg 0 0 1 1 re f"[..])
        } else {
            (format!("/Resources << /XObject << /N {next} 0 R >> >>"), &b"/N Do"[..])
        };
        extra.push(stream_body(&format!("/Type /XObject /Subtype /Form /BBox [0 0 9 9] {resources}"), content));
    }
    let pdf = one_page(b"/N Do", "/XObject << /N 5 0 R >>", &extra);
    let report = bounded(&pdf, &Expectations::professional()).unwrap();
    assert!(report.has(c::LIMIT_EXCEEDED), "{:#?}", report.findings);
    assert!(!report.has(c::COLOR_RGB), "the RGB fill at depth 40 is beyond the nesting limit");
    assert!(!report.passed());
}

// ---- bombs and floods ----------------------------------------------------------------------

#[test]
fn decompression_bombs_are_refused() {
    // 40 MiB of spaces deflate to a few dozen kilobytes; the per-stream limit is 32 MiB.
    let mut bomb = Stream::new(dictionary! {}, vec![b' '; 40 * 1024 * 1024]);
    bomb.compress().unwrap();
    assert!(bomb.content.len() < 200_000);
    let mut f = Fixture::reference();
    let id = f.doc.add_object(bomb.clone());
    f.dict(f.page1).set("Contents", id);
    let report = bounded(&f.bytes(), &card_expectations()).unwrap();
    assert_eq!(codes(&report), set(&[c::LIMIT_EXCEEDED]), "{:#?}", report.findings);
    assert!(!report.passed());

    // Doubly compressed.
    let mut twice = Stream::new(dictionary! {}, bomb.content.clone());
    twice.compress().unwrap();
    twice.dict.set("Filter", vec![Object::Name(b"FlateDecode".to_vec()), Object::Name(b"FlateDecode".to_vec())]);
    let mut f = Fixture::reference();
    let id = f.doc.add_object(twice);
    f.dict(f.page1).set("Contents", id);
    let report = bounded(&f.bytes(), &card_expectations()).unwrap();
    assert_eq!(codes(&report), set(&[c::LIMIT_EXCEEDED]), "{:#?}", report.findings);

    // A bomb as the output profile or as the XMP packet is simply unreadable metadata.
    let mut f = Fixture::reference();
    if let Object::Stream(stream) = f.doc.objects.get_mut(&f.icc).unwrap() {
        *stream = bomb.clone();
        stream.dict.set("N", 4);
    }
    let report = bounded(&f.bytes(), &card_expectations()).unwrap();
    assert_eq!(codes(&report), set(&[c::OUTPUT_INTENT_INVALID]), "{:#?}", report.findings);
}

#[test]
fn too_many_filters_and_unknown_filters_are_content_errors() {
    let filters: Vec<Object> = (0..40).map(|_| Object::Name(b"ASCIIHexDecode".to_vec())).collect();
    let mut f = Fixture::reference();
    let id = f.doc.add_object(Stream::new(dictionary! { "Filter" => filters }, b"71>".to_vec()));
    f.dict(f.page1).set("Contents", id);
    let report = bounded(&f.bytes(), &card_expectations()).unwrap();
    assert_eq!(codes(&report), set(&[c::CONTENT_INVALID]), "{:#?}", report.findings);
}

#[test]
fn operator_and_operand_floods_are_linear() {
    // Four megabytes of operands with no operator, then one.
    let mut content = b"1 ".repeat(2_000_000);
    content.extend_from_slice(b"0 0 0 1 k 0 0 9 9 re f");
    let report = bounded(&one_page(&content, "", &[]), &home()).unwrap();
    assert_eq!(report.pages[0].color_spaces, vec!["DeviceCMYK".to_string()]);

    // A million restores without saves, a million saves without restores.
    for flood in [&b"Q "[..], b"q "] {
        let report = bounded(&one_page(&flood.repeat(1_000_000), "", &[]), &home()).unwrap();
        assert!(report.has(c::CONTENT_UNBALANCED_Q), "{:#?}", report.findings);
    }

    // A megabyte-long name, string and comment.
    let mut content = b"/".to_vec();
    content.extend(std::iter::repeat_n(b'N', 1_000_000));
    content.extend_from_slice(b" gs (");
    content.extend(std::iter::repeat_n(b'(', 500_000));
    content.extend(std::iter::repeat_n(b')', 500_000));
    content.extend_from_slice(b") Tj %");
    content.extend(std::iter::repeat_n(b'%', 1_000_000));
    content.extend_from_slice(b"\n0 0 0 1 k 0 0 9 9 re f");
    let report = bounded(&one_page(&content, "", &[]), &home()).unwrap();
    // The text is shown in the default black (DeviceGray); the fill after the comment is CMYK.
    assert_eq!(report.pages[0].color_spaces, vec!["DeviceCMYK".to_string(), "DeviceGray".to_string()]);
    let missing = report.findings.iter().find(|f| f.code == c::RESOURCE_MISSING).unwrap();
    assert!(missing.message.len() < 400, "long names are abbreviated in messages");
}

#[test]
fn deeply_nested_syntax_in_content_and_in_objects() {
    // In a content stream: rejected by the lexer without recursion.
    let mut content = vec![b'['; 200_000];
    content.extend(std::iter::repeat_n(b']', 200_000));
    content.extend_from_slice(b" TJ");
    let report = bounded(&one_page(&content, "", &[]), &home()).unwrap();
    let invalid = report.findings.iter().find(|f| f.code == c::CONTENT_INVALID).unwrap();
    assert!(invalid.message.contains("nested too deeply"), "{}", invalid.message);

    // In the file body: nested arrays and nested dictionaries inside the page object.
    let mut arrays = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Deep ".to_vec();
    arrays.extend(std::iter::repeat_n(b'[', 100_000));
    arrays.extend(std::iter::repeat_n(b']', 100_000));
    arrays.extend_from_slice(b" >>");
    let mut dicts = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Deep ".to_vec();
    for _ in 0..100_000 {
        dicts.extend_from_slice(b"<</A ");
    }
    dicts.extend_from_slice(b"0");
    for _ in 0..100_000 {
        dicts.extend_from_slice(b">>");
    }
    dicts.extend_from_slice(b" >>");
    for page in [arrays, dicts] {
        let pdf = raw_pdf(
            &[body("<< /Type /Catalog /Pages 2 0 R >>"), body("<< /Type /Pages /Kids [3 0 R] /Count 1 >>"), page],
            "/Root 1 0 R",
        );
        // Either the parser gives up on the file, or the page that could not be read is
        // reported; both are fine, crashing is not.
        match bounded(&pdf, &home()) {
            Err(error) => assert!(matches!(error, PreflightError::Parse(_)), "{error}"),
            Ok(report) => assert!(!report.passed(), "{:#?}", report.findings),
        }
    }
}

#[test]
fn page_count_is_capped() {
    let mut bodies = vec![body("<< /Type /Catalog /Pages 2 0 R >>")];
    let kids: String = (0..2_500).map(|i| format!("{} 0 R ", i + 3)).collect();
    bodies.push(body(&format!("<< /Type /Pages /Kids [{kids}] /Count 2500 /MediaBox [0 0 100 100] >>")));
    for _ in 0..2_500 {
        bodies.push(body("<< /Type /Page /Parent 2 0 R /TrimBox [0 0 100 100] >>"));
    }
    let report = bounded(&raw_pdf(&bodies, "/Root 1 0 R"), &home()).unwrap();
    assert_eq!(report.pages.len(), 2_000);
    assert_eq!(report.document.page_count, 2_000);
    let limit = report.findings.iter().find(|f| f.code == c::LIMIT_EXCEEDED).unwrap();
    assert_eq!(limit.severity, Severity::Error);
}

#[test]
fn absurd_numbers_and_binary_names_do_not_break_the_report() {
    // Sixty digits in content streams (this crate's lexer reads them as huge reals); in
    // objects, the largest integer and a real that overflows to infinity when parsed.
    let nines = "9".repeat(60);
    let max = i64::MAX.to_string();
    let inf = format!("{nines}.5");
    let content = format!(
        "{nines} 0 0 {nines} 0 0 cm /I Do q -{nines} {nines} {nines} -{nines} re f Q \
         0.{nines} {nines} 1 1 k 0 0 1 1 re f {nines} w 0 0 m 1 1 l S"
    );
    let resources = "/XObject << /I 5 0 R >> /Font << /F#00#07#ff 6 0 R >>";
    let image = stream_body(
        &format!("/Type /XObject /Subtype /Image /Width {max} /Height -1 /ColorSpace /DeviceGray /BitsPerComponent {max}"),
        b"",
    );
    let font = body("<< /Type /Font /Subtype /Type1 /BaseFont /Bad#00#01#ffName >>");
    let bodies = vec![
        body("<< /Type /Catalog /Pages 2 0 R >>"),
        body("<< /Type /Pages /Kids [3 0 R] /Count 1 >>"),
        body(&format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {inf} 200] /TrimBox [-{max} 0 1 1] \
             /BleedBox [0 0 0.{nines} 1] /ArtBox [0 0 {max} {max}] /UserUnit {inf} /Rotate {max} \
             /Contents 4 0 R /Resources << {resources} >> >>"
        )),
        stream_body("", content.as_bytes()),
        image,
        font,
    ];
    let pdf = raw_pdf(&bodies, &format!("/Root 1 0 R /Info << /Title (\\000\\001\\377) /EBC.SpineWidthMm ({nines}) >>"));
    let report = bounded(&pdf, &Expectations::professional().with_spine(10.0)).unwrap();
    assert_eq!(report.pages.len(), 1, "{:#?}", report.findings);
    assert!(!report.passed());
    for code in [c::BOX_INVALID, c::BOX_MEDIA_MISSING, c::EBC_KEY_INVALID, c::FONT_NOT_EMBEDDED, c::CONTENT_INVALID, c::IMAGE_INVALID] {
        assert!(report.has(code), "{code} missing from {:#?}", report.findings);
    }
    // The control characters of the font name are replaced, and every number that made it
    // into the report is finite.
    assert_eq!(report.pages[0].fonts[0].name, "Bad\u{FFFD}\u{FFFD}ÿName");
    let json = report.to_json();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    fn all_numbers_finite(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Number(n) => n.as_f64().is_some_and(f64::is_finite),
            serde_json::Value::Null => true,
            serde_json::Value::Array(items) => items.iter().all(all_numbers_finite),
            serde_json::Value::Object(map) => map.values().all(all_numbers_finite),
            _ => true,
        }
    }
    assert!(all_numbers_finite(&value["pages"]), "{json}");
    // Non-finite floats would have been written as null; the image list must not hide any.
    assert!(!json.contains("\"ppi_x\":null") && !json.contains("\"width_mm\":null"), "{json}");
}

#[test]
fn wrong_stream_lengths_are_survivable() {
    for declared in ["0", "3", "99999", "-5", "(five)", "12 0 R"] {
        let mut content = format!("<< /Length {declared} >>\nstream\n").into_bytes();
        content.extend_from_slice(b"1 0 0 rg 0 0 9 9 re f\nendstream");
        let pdf = raw_pdf(
            &[
                body("<< /Type /Catalog /Pages 2 0 R >>"),
                body("<< /Type /Pages /Kids [3 0 R] /Count 1 >>"),
                body("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /TrimBox [0 0 300 200] /Contents 4 0 R >>"),
                content,
            ],
            "/Root 1 0 R",
        );
        // Readers disagree on how to recover from a wrong Length; whatever lopdf makes of
        // it, the result is an error or a report.
        let _ = bounded(&pdf, &Expectations::professional());
    }
}
