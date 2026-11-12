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
//!   was not saved incrementally, and contains no JavaScript, actions, annotations,
//!   interactive forms or embedded files.
//! * **Page boxes** — `MediaBox` and `TrimBox` present, `TrimBox ⊆ BleedBox ⊆ MediaBox`,
//!   bleed at least the expected width on all four sides, expected trim size, all pages the
//!   same size; artwork that reaches the trim edge must continue into the bleed.
//! * **Fonts** — every font in the page's resource tree (forms and patterns included) has an
//!   embedded font program; subset prefixes are reported; Type 3 fonts are flagged.
//! * **Images** — every placement of every image is located by interpreting the content
//!   stream (graphics state stack, `cm`, nested form XObjects), which gives its printed size
//!   and thus its effective resolution; stretched images are flagged.
//! * **Colour** — the colour spaces actually used to paint; RGB-like spaces, spot colours and
//!   registration colour inside the bleed box are flagged.
//! * **Transparency** — soft masks, opacity below 1, blend modes, transparency groups.
//! * **Ink** — total ink of CMYK fills and strokes, and of losslessly stored CMYK images
//!   (the decoded pixels; more than 0.5 % of them above the limit is an error in a
//!   professional file); heavy ink (above [`HEAVY_INK_AT_CREASE_PERCENT`]) within
//!   [`CREASE_BAND_MM`] of a crease declared in `EBC.FoldXMm`, a warning that advises
//!   lamination; hairline strokes.
//! * **Output intent and PDF/X** — a `GTS_PDFX` output intent with identifier and a profile
//!   whose component count matches; version markers consistent with each other, with the
//!   file header and with the features used; `Title`, `Trapped`, dates and file identifier.
//! * **Card geometry** — the `EBC.*` Info keys written by the card composer
//!   (`EBC.SpineWidthMm`, `EBC.FoldXMm` as a comma-separated list of x positions measured
//!   from the left edge of the trim box, `EBC.CoverWidthMm`, `EBC.CoverHeightMm`, and the
//!   optional `EBC.BleedMm` and `EBC.QrModuleMm`) agree with the page boxes and with the
//!   expected spine; a vector QR code is recognised and its module size measured.
//!
//! # What is not checked
//!
//! Text size and safety margins (they need glyph metrics), overprint settings, the ink
//! coverage of JPEG-compressed images (this crate has no JPEG decoder, so a CMYK JPEG is
//! neither checked against the ink limit nor at the creases), whether a heavy fill on a
//! crease is hidden by something painted over it (the crease check does not follow
//! occlusion), the content of
//! Type 3 glyph procedures and annotation appearances, and whether the QR code actually
//! decodes (that needs a renderer — the end-to-end tests do it). Clipping paths are ignored
//! when deciding where something is painted.
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

pub use content::{CREASE_BAND_MM, HEAVY_INK_AT_CREASE_PERCENT};
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

/// Largest input accepted by [`check`], in bytes (256 MiB). A card PDF is a few megabytes.
pub const MAX_PDF_BYTES: usize = 256 * 1024 * 1024;

/// Image placements and fonts listed in the page summaries of one report. Everything is
/// still checked beyond these numbers; only the listing stops.
const MAX_LISTED_ITEMS: usize = 20_000;

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
    let options = LoadOptions {
        max_decompressed_size: Some(MAX_OBJECT_STREAM_BYTES),
        ..LoadOptions::default()
    };
    let doc = match Document::load_mem_with_options(pdf, options) {
        Ok(doc) => doc,
        Err(
            lopdf::Error::Decryption(_)
            | lopdf::Error::InvalidPassword
            | lopdf::Error::UnsupportedSecurityHandler(_),
        ) => {
            return Ok(encrypted_report(
                "the file is encrypted and cannot be read without a password",
            ));
        }
        Err(error) => return Err(PreflightError::Parse(stream::describe(&error))),
    };

    let mut sink = Sink::default();
    let mut summary =
        DocumentSummary { pdf_version: doc.version.clone(), ..DocumentSummary::default() };

    let encrypted = doc.trailer.has(b"Encrypt") || doc.encryption_state.is_some();
    let catalog = doc.trailer.get(b"Root").ok().and_then(|root| pdf::as_dict(&doc, root));
    if encrypted {
        summary.encrypted = true;
        if catalog.is_none() {
            return Ok(encrypted_report(
                "the file is encrypted and cannot be read without a password",
            ));
        }
        sink.error(
            codes::ENCRYPTED,
            None,
            "the file is encrypted (it opens without a password, but printers reject encrypted files)",
        );
    }
    let Some(catalog) = catalog else { return Err(PreflightError::NoCatalog) };

    match doc.xref_recovery {
        None => {}
        // The table itself is intact; only the pointer to it was off (viewers cope).
        Some(lopdf::xref::XrefRecovery::OffsetCorrected) => sink.warning(
            codes::XREF_DAMAGED,
            None,
            "a cross-reference offset is wrong; the table was found nearby, but strict \
             readers may reject the file",
        ),
        // Rebuilt by scanning for objects (or any repair lopdf may add later).
        Some(_) => sink.error(
            codes::XREF_DAMAGED,
            None,
            "the cross-reference table is damaged and had to be rebuilt by scanning the file; \
             the file is probably truncated or corrupt",
        ),
    }

    let revisions = doc.revision_count();
    if revisions > 1 {
        sink.warning(
            codes::INCREMENTAL_UPDATE,
            None,
            format!(
                "the file was saved incrementally and holds {revisions} revisions; \
                 earlier versions of its content are still inside it"
            ),
        );
    }

    structure::check_document(&doc, catalog, &mut sink);
    let level = meta::check_metadata(&doc, catalog, expect, &mut summary, &mut sink);
    structure::check_optional_content(
        &doc,
        catalog,
        level.is_some_and(meta::PdfxLevel::forbids_layers),
        expect.require_output_intent,
        &mut sink,
    );

    let nodes = pages::collect(&doc, catalog, &mut sink);
    summary.page_count = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
    if nodes.is_empty() {
        sink.error(codes::NO_PAGES, None, "the document has no pages");
    }

    let folds = meta::fold_positions(&summary.ebc);
    let mut budget = Budget::new();
    let mut page_summaries = Vec::with_capacity(nodes.len());
    let mut all_boxes = Vec::with_capacity(nodes.len());
    let mut qr_candidates: Vec<(u32, QrCandidate)> = Vec::new();
    let mut listed = 0_usize;
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
        )
        .with_creases(crease_bands(&folds, &boxes));
        let facts = interpreter.run(node.dict, node.resources);
        if !facts.unbounded_paint {
            pages::check_bleed_painted(&boxes, facts.painted, expect, number, &mut sink);
        }
        let mut fonts =
            fonts::check_fonts(&doc, node.resources, &facts.used_fonts, number, &mut sink);
        let mut images = facts.images;
        // Many pages sharing many fonts or images must not blow the report up.
        let room = MAX_LISTED_ITEMS.saturating_sub(listed);
        if fonts.len() + images.len() > room {
            images.truncate(room);
            fonts.truncate(room - images.len());
            sink.warning(
                codes::LIMIT_EXCEEDED,
                None,
                format!(
                    "the report lists the first {MAX_LISTED_ITEMS} fonts and image placements only \
                     (everything was still checked)"
                ),
            );
        }
        listed += fonts.len() + images.len();
        // What the expectations let through may still contradict the file's own PDF/X claim.
        if let Some(level) = level {
            let claim = summary.pdfx_version.as_deref().unwrap_or("PDF/X");
            if level.forbids_rgb() && facts.uses_rgb && expect.allow_rgb {
                sink.error(
                    codes::PDFX_CONTENT_VIOLATION,
                    Some(number),
                    format!("the file claims {claim}, which allows CMYK, grey and spot colours only, but the page uses RGB colour"),
                );
            }
            if level.forbids_layers() && facts.transparency && expect.allow_transparency {
                sink.error(
                    codes::PDFX_CONTENT_VIOLATION,
                    Some(number),
                    format!("the file claims {claim}, which does not allow transparency, but the page uses it"),
                );
            }
        }
        qr_candidates
            .extend(facts.qr_candidates.iter().cloned().map(|candidate| (number, candidate)));

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
            images,
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
        page_summaries.first().and_then(|page| page.bleed_mm).map(|bleed| bleed.min()),
        &qr_candidates,
        expect,
        &mut sink,
    );

    Ok(Report { findings: sink.into_sorted(), pages: page_summaries, document: summary })
}

/// The bands of ±[`CREASE_BAND_MM`] around the folds (millimetres from the left edge of
/// the trim box), over the height of the trim box, in default user space.
fn crease_bands(folds: &[f64], boxes: &pages::PageBoxes) -> Vec<pdf::RectPt> {
    let Some(trim) = boxes.trim else { return Vec::new() };
    let units_per_mm = 72.0 / 25.4 / boxes.user_unit;
    if !units_per_mm.is_finite() || units_per_mm <= 0.0 {
        return Vec::new();
    }
    folds
        .iter()
        .map(|&x| {
            let centre = trim.x0 + x * units_per_mm;
            let half = CREASE_BAND_MM * units_per_mm;
            pdf::RectPt { x0: centre - half, y0: trim.y0, x1: centre + half, y1: trim.y1 }
        })
        .collect()
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
