//! Preflight checks for print PDFs: page boxes, fonts, image resolution, colour,
//! transparency, ink coverage and PDF/X markers.
//!
//! This crate is the independent referee of the eBook Cadeau print pipeline. It does not
//! depend on the crate that writes the PDFs (`ebc-pdf`): it parses the bytes back with
//! [`lopdf`], walks every page's content stream and reports what a printer's own preflight
//! would complain about. It is also what the demo print-on-demand API runs on uploads, so it
//! treats its input as hostile: [`check`] never panics and never hangs, whatever the bytes.
//!
//! # Example
//!
//! ```
//! use ebc_preflight::{check, codes, Expectations, PreflightError, Severity};
//!
//! // Not a PDF at all: an error, not a report.
//! assert_eq!(check(b"hello", &Expectations::professional()), Err(PreflightError::NotPdf));
//!
//! // A minimal one-page PDF, built here with lopdf: A6 landscape, no trim box,
//! // text in a font that is not embedded, painted in RGB.
//! use lopdf::{dictionary, Document, Object, Stream};
//! let mut doc = Document::with_version("1.4");
//! let pages = doc.new_object_id();
//! let font = doc.add_object(dictionary! {
//!     "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
//! });
//! let content = doc.add_object(Stream::new(
//!     dictionary! {},
//!     b"BT /F1 12 Tf 1 0 0 rg 20 20 Td (Bonne lecture) Tj ET".to_vec(),
//! ));
//! let page = doc.add_object(dictionary! {
//!     "Type" => "Page", "Parent" => pages, "Contents" => content,
//!     "MediaBox" => vec![0.into(), 0.into(), 420.into(), 298.into()],
//!     "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
//! });
//! doc.objects.insert(pages, Object::Dictionary(dictionary! {
//!     "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1,
//! }));
//! let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages });
//! doc.trailer.set("Root", catalog);
//! let mut pdf = Vec::new();
//! doc.save_to(&mut pdf).unwrap();
//!
//! let report = check(&pdf, &Expectations::professional()).unwrap();
//! assert!(!report.passed());
//! for code in [codes::BOX_TRIM_MISSING, codes::FONT_NOT_EMBEDDED, codes::COLOR_RGB, codes::NO_OUTPUT_INTENT] {
//!     assert!(report.has(code), "{code} expected in {:?}", report.codes());
//! }
//! let font = &report.pages[0].fonts[0];
//! assert_eq!((font.name.as_str(), font.embedded, font.used), ("Helvetica", false, true));
//! assert!(report.to_json().starts_with(r#"{"passed":false,"#));
//!
//! // Printing at home is more forgiving, but an unembedded font is still an error.
//! let relaxed = check(&pdf, &Expectations::home()).unwrap();
//! let errors: Vec<_> = relaxed.errors().map(|f| f.code).collect();
//! assert_eq!(errors, [codes::BOX_TRIM_MISSING, codes::FONT_NOT_EMBEDDED]);
//! assert!(relaxed.findings.iter().all(|f| f.severity != Severity::Error || f.page == Some(1)));
//! ```
//!
//! # What is checked
//!
//! * **Structure** — the file parses with an intact cross-reference table, is not encrypted,
//!   and contains no JavaScript, actions, annotations, interactive forms or embedded files.
//! * **Page boxes** — `MediaBox` and `TrimBox` present, `TrimBox ⊆ BleedBox ⊆ MediaBox`,
//!   bleed at least the expected width on all four sides, expected trim size, all pages the
//!   same size.
//! * **Fonts** — every font in the page's resource tree (forms and patterns included) has an
//!   embedded font program; subset prefixes are reported; Type 3 fonts are flagged.
//! * **Images** — every placement of every image is located by interpreting the content
//!   stream (graphics state stack, `cm`, nested form XObjects), which gives its printed size
//!   and thus its effective resolution.
//! * **Colour** — the colour spaces actually used to paint; RGB-like spaces, spot colours and
//!   registration colour inside the bleed box are flagged.
//! * **Transparency** — soft masks, opacity below 1, blend modes, transparency groups.
//! * **Ink** — total ink of CMYK fills and strokes, and of losslessly stored CMYK images.
//! * **Output intent and PDF/X** — a `GTS_PDFX` output intent with identifier and a profile
//!   whose component count matches; version markers consistent with the file header;
//!   Info `Title` and `Trapped`.
//! * **Card geometry** — the `EBC.*` Info keys written by the card composer
//!   (`EBC.SpineWidthMm`, `EBC.FoldXMm` as a comma-separated list of x positions measured
//!   from the left edge of the trim box, `EBC.CoverWidthMm`, `EBC.CoverHeightMm`, and the
//!   optional `EBC.QrModuleMm`) agree with the trim box and with the expected spine.
//!
//! # Limits
//!
//! Analysis is bounded so that hostile files cost bounded time and memory: see
//! [`MAX_PDF_BYTES`] for the input, and the report carries a [`codes::LIMIT_EXCEEDED`] error
//! when a document exceeds an internal budget (20 million content operators, 32 MiB per
//! decoded content stream and 128 MiB in total, 2 000 pages, 50 000 form XObject or pattern
//! invocations, nested 16 deep at most).

#![forbid(unsafe_code)]

pub mod codes;
mod colour;
mod content;
mod expect;
mod fonts;
mod lexer;
mod meta;
mod pages;
mod pdf;
mod qr;
mod report;
mod sink;
mod stream;
mod structure;

use std::panic::{AssertUnwindSafe, catch_unwind};

use lopdf::{Document, LoadOptions};

pub use expect::{
    BLEED_TOLERANCE_MM, Expectations, NESTING_TOLERANCE_MM, PreflightError, TRIM_TOLERANCE_MM,
};
pub use report::{
    BleedMm, BoxMm, DocumentSummary, Finding, FontSummary, ImageSummary, OutputIntentSummary,
    PageSummary, QrCandidate, Report, Severity,
};

use crate::content::Interpreter;
use crate::report::round3;
use crate::sink::{Budget, Sink};

/// Largest input accepted by [`check`], in bytes (512 MiB). A card PDF is a few megabytes.
pub const MAX_PDF_BYTES: usize = 512 * 1024 * 1024;

/// Limit for object streams and cross-reference streams while loading the file.
const MAX_OBJECT_STREAM_BYTES: usize = 64 * 1024 * 1024;

/// Preflights a PDF against `expect`.
///
/// Returns `Err` when the bytes cannot be read as a PDF at all (see [`PreflightError`]);
/// otherwise a [`Report`] whose [`Report::passed`] tells whether any error-level finding was
/// raised. The function is deterministic and never panics on malformed input.
///
/// # Errors
///
/// [`PreflightError::NotPdf`], [`PreflightError::TooLarge`], [`PreflightError::Parse`],
/// [`PreflightError::NoCatalog`] for unreadable input, and
/// [`PreflightError::InvalidExpectations`] for nonsensical expectations.
pub fn check(pdf: &[u8], expect: &Expectations) -> Result<Report, PreflightError> {
    expect.validate()?;
    if pdf.len() > MAX_PDF_BYTES {
        return Err(PreflightError::TooLarge { size: pdf.len(), limit: MAX_PDF_BYTES });
    }
    let header_window = &pdf[..pdf.len().min(1024)];
    if !header_window.windows(5).any(|w| w == b"%PDF-") {
        return Err(PreflightError::NotPdf);
    }
    // The checks are written not to panic, and `lopdf` is hardened against hostile input;
    // this net turns any bug that slips through into an error instead of taking the caller
    // (a server, a browser tab) down. It cannot catch stack overflows, which is why every
    // traversal in this crate is iterative or depth-limited.
    match catch_unwind(AssertUnwindSafe(|| check_parsed(pdf, expect))) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panic".to_string());
            Err(PreflightError::Internal(message))
        }
    }
}

fn check_parsed(pdf: &[u8], expect: &Expectations) -> Result<Report, PreflightError> {
    let options =
        LoadOptions { max_decompressed_size: Some(MAX_OBJECT_STREAM_BYTES), ..LoadOptions::default() };
    let doc = match Document::load_mem_with_options(pdf, options) {
        Ok(doc) => doc,
        Err(
            lopdf::Error::Decryption(_)
            | lopdf::Error::InvalidPassword
            | lopdf::Error::UnsupportedSecurityHandler(_),
        ) => return Ok(encrypted_report("the file is encrypted and cannot be read without a password")),
        Err(error) => return Err(PreflightError::Parse(stream::describe(&error))),
    };

    let mut sink = Sink::default();
    let mut summary = DocumentSummary { pdf_version: doc.version.clone(), ..DocumentSummary::default() };

    let encrypted = doc.trailer.has(b"Encrypt") || doc.encryption_state.is_some();
    let catalog = doc.trailer.get(b"Root").ok().and_then(|root| pdf::as_dict(&doc, root));
    if encrypted {
        summary.encrypted = true;
        if catalog.is_none() {
            return Ok(encrypted_report("the file is encrypted and cannot be read without a password"));
        }
        sink.error(
            codes::ENCRYPTED,
            None,
            "the file is encrypted (it opens without a password, but printers reject encrypted files)",
        );
    }
    let Some(catalog) = catalog else { return Err(PreflightError::NoCatalog) };

    if let Some(recovery) = doc.xref_recovery {
        sink.error(
            codes::XREF_DAMAGED,
            None,
            format!(
                "the cross-reference table is damaged and had to be repaired to read the file ({recovery:?}); \
                 the file may be truncated or corrupt"
            ),
        );
    }

    structure::check_document(&doc, catalog, &mut sink);
    let level = meta::check_metadata(&doc, catalog, expect, &mut summary, &mut sink);
    structure::check_optional_content(&doc, catalog, level.is_some_and(meta::PdfxLevel::forbids_layers), &mut sink);

    let nodes = pages::collect(&doc, catalog, &mut sink);
    summary.page_count = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
    if nodes.is_empty() {
        sink.error(codes::NO_PAGES, None, "the document has no pages");
    }

    let mut budget = Budget::new();
    let mut page_summaries = Vec::with_capacity(nodes.len());
    let mut all_boxes = Vec::with_capacity(nodes.len());
    let mut qr_candidates: Vec<(u32, QrCandidate)> = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let number = u32::try_from(index + 1).unwrap_or(u32::MAX);
        let boxes = pages::read_boxes(&doc, node, number, &mut sink);
        let measures = pages::check_boxes(&boxes, expect, number, &mut sink);
        structure::check_page(&doc, node, &boxes, number, &mut sink);

        let interpreter = Interpreter::new(
            &doc,
            expect,
            number,
            boxes.user_unit,
            boxes.printed_area(),
            &mut sink,
            &mut budget,
        );
        let facts = interpreter.run(node.dict, node.resources);
        let fonts = fonts::check_fonts(&doc, node.resources, &facts.used_fonts, number, &mut sink);
        qr_candidates.extend(facts.qr_candidates.iter().cloned().map(|candidate| (number, candidate)));

        let to_mm = |rect: Option<pdf::RectPt>| {
            rect.map(|rect| {
                let mm = boxes.to_mm(rect);
                BoxMm { x0: round3(mm.x0), y0: round3(mm.y0), x1: round3(mm.x1), y1: round3(mm.y1) }
            })
        };
        page_summaries.push(PageSummary {
            number,
            media_box: to_mm(boxes.media),
            crop_box: to_mm(boxes.crop),
            bleed_box: to_mm(boxes.bleed),
            trim_box: to_mm(boxes.trim),
            art_box: to_mm(boxes.art),
            trim_mm: measures.trim_mm,
            bleed_mm: measures.bleed_mm,
            rotate: boxes.rotate,
            fonts,
            images: facts.images,
            color_spaces: facts.color_spaces.into_iter().collect(),
            max_total_ink_percent: facts.max_ink_percent.map(|v| (v * 10.0).round() / 10.0),
            transparency: facts.transparency,
            qr_candidates: facts.qr_candidates,
        });
        all_boxes.push(boxes);
    }
    pages::check_same_size(&all_boxes, &mut sink);
    meta::check_ebc_geometry(
        &summary.ebc,
        all_boxes.first().and_then(pages::PageBoxes::trim_size_mm),
        &qr_candidates,
        expect,
        &mut sink,
    );

    Ok(Report { findings: sink.into_sorted(), pages: page_summaries, document: summary })
}

/// The report for a file that cannot be read because it is encrypted.
fn encrypted_report(message: &str) -> Report {
    Report {
        findings: vec![Finding {
            severity: Severity::Error,
            code: codes::ENCRYPTED,
            page: None,
            message: message.to_string(),
        }],
        pages: Vec::new(),
        document: DocumentSummary { encrypted: true, ..DocumentSummary::default() },
    }
}
