//! Document metadata checks: output intent, PDF/X markers, Info `Title` / `Trapped`, and the
//! `EBC.*` Info keys that carry the card geometry.

use std::collections::BTreeMap;

use lopdf::{Dictionary, Document, Object};

use crate::codes;
use crate::colour::parse_icc_header;
use crate::expect::{Expectations, TRIM_TOLERANCE_MM};
use crate::pdf::{as_dict, get, get_array, get_int, get_name, get_text, lossy, resolve};
use crate::report::{DocumentSummary, OutputIntentSummary, QrCandidate, Severity};
use crate::sink::Sink;
use crate::stream;

/// Largest ICC profile or XMP packet decoded.
const MAX_METADATA_BYTES: usize = 32 * 1024 * 1024;
/// `EBC.*` keys copied into the report.
const MAX_EBC_KEYS: usize = 64;

/// The PDF/X level a file claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PdfxLevel {
    X1a2001,
    X1a2003,
    X3v2002,
    X3v2003,
    X4,
    X5,
    X6,
    Other,
}

impl PdfxLevel {
    pub(crate) fn parse(marker: &str) -> Self {
        let marker = marker.trim();
        if marker.starts_with("PDF/X-1") {
            if marker.ends_with(":2003") { Self::X1a2003 } else { Self::X1a2001 }
        } else if marker.starts_with("PDF/X-3") {
            if marker.ends_with(":2003") { Self::X3v2003 } else { Self::X3v2002 }
        } else if marker.starts_with("PDF/X-4") {
            Self::X4
        } else if marker.starts_with("PDF/X-5") {
            Self::X5
        } else if marker.starts_with("PDF/X-6") {
            Self::X6
        } else {
            Self::Other
        }
    }

    /// The newest PDF version (major, minor) a conforming file may declare.
    fn max_pdf_version(self) -> Option<(u32, u32)> {
        match self {
            Self::X1a2001 | Self::X3v2002 => Some((1, 3)),
            Self::X1a2003 | Self::X3v2003 => Some((1, 4)),
            Self::X4 | Self::X5 => Some((1, 6)),
            Self::X6 => Some((2, 0)),
            Self::Other => None,
        }
    }

    /// Whether the level predates optional content (layers) and transparency.
    pub(crate) fn forbids_layers(self) -> bool {
        matches!(self, Self::X1a2001 | Self::X1a2003 | Self::X3v2002 | Self::X3v2003)
    }

    /// Whether the output profile must be embedded.
    fn requires_embedded_profile(self) -> bool {
        matches!(self, Self::X4 | Self::X6)
    }
}

/// Parses `"1.6"` into `(1, 6)`.
fn parse_version(text: &str) -> Option<(u32, u32)> {
    let (major, minor) = text.trim().split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// The Info dictionary, if the trailer has a usable one.
pub(crate) fn info_dict(doc: &Document) -> Option<&Dictionary> {
    as_dict(doc, doc.trailer.get(b"Info").ok()?)
}

/// Runs the metadata checks and fills the document summary. Returns the PDF/X level claimed.
pub(crate) fn check_metadata(
    doc: &Document,
    catalog: &Dictionary,
    expect: &Expectations,
    summary: &mut DocumentSummary,
    sink: &mut Sink,
) -> Option<PdfxLevel> {
    let info = info_dict(doc);
    let level = check_pdfx_markers(doc, catalog, info, expect, summary, sink);
    check_output_intent(doc, catalog, level, expect, summary, sink);
    check_info(doc, info, level, summary, sink);
    if let Some(info) = info {
        for (key, _) in info {
            if key.starts_with(b"EBC.") && summary.ebc.len() < MAX_EBC_KEYS {
                if let Some(value) = info_value(doc, info, key) {
                    summary.ebc.insert(lossy(key), value.chars().take(200).collect());
                }
            }
        }
    }
    level
}

/// An Info value as text: strings and names as they are, numbers in decimal notation
/// (a writer may store `EBC.SpineWidthMm` as a number rather than a string).
fn info_value(doc: &Document, info: &Dictionary, key: &[u8]) -> Option<String> {
    match get(doc, info, key)? {
        Object::Integer(value) => Some(value.to_string()),
        Object::Real(value) => Some(value.to_string()),
        _ => get_text(doc, info, key),
    }
}

// ---- PDF/X markers -------------------------------------------------------------------------

fn check_pdfx_markers(
    doc: &Document,
    catalog: &Dictionary,
    info: Option<&Dictionary>,
    expect: &Expectations,
    summary: &mut DocumentSummary,
    sink: &mut Sink,
) -> Option<PdfxLevel> {
    let from_info = info.and_then(|info| get_text(doc, info, b"GTS_PDFXVersion")).filter(|v| !v.trim().is_empty());
    let conformance =
        info.and_then(|info| get_text(doc, info, b"GTS_PDFXConformance")).filter(|v| !v.trim().is_empty());
    let from_xmp = xmp_packet(doc, catalog).and_then(|xmp| {
        xmp_property(&xmp, "pdfxid:GTS_PDFXVersion").or_else(|| xmp_property(&xmp, "pdfx:GTS_PDFXVersion"))
    });

    // PDF/X-1a:2001 files say "PDF/X-1:2001" in the version key and name the exact
    // conformance level in GTS_PDFXConformance.
    let info_claim = match (&from_info, &conformance) {
        (Some(version), Some(conformance)) if version.trim().starts_with("PDF/X-1:") => {
            Some(conformance.trim().to_string())
        }
        (Some(version), _) => Some(version.trim().to_string()),
        (None, _) => None,
    };
    let claim = from_xmp.clone().or_else(|| info_claim.clone());
    summary.pdfx_version.clone_from(&claim);

    let Some(claim) = claim else {
        let message = "the file does not claim PDF/X conformance (no GTS_PDFXVersion in Info or XMP)";
        if expect.require_output_intent {
            sink.warning(codes::PDFX_NOT_CLAIMED, None, message);
        } else {
            sink.info(codes::PDFX_NOT_CLAIMED, None, message);
        }
        return None;
    };
    let level = PdfxLevel::parse(&claim);
    if level == PdfxLevel::Other {
        sink.error(
            codes::PDFX_VERSION_MISMATCH,
            None,
            format!("unrecognised PDF/X version marker \"{}\"", lossy(claim.as_bytes())),
        );
    }
    if let (Some(xmp), Some(info_claim)) = (&from_xmp, &info_claim) {
        if PdfxLevel::parse(xmp) != PdfxLevel::parse(info_claim) {
            sink.error(
                codes::PDFX_VERSION_MISMATCH,
                None,
                format!(
                    "the Info dictionary claims {} but the XMP metadata claims {}",
                    lossy(info_claim.as_bytes()),
                    lossy(xmp.as_bytes())
                ),
            );
        }
    }
    // The catalog may raise the version declared in the header.
    let header = parse_version(&doc.version);
    let catalog_version =
        get_name(doc, catalog, b"Version").and_then(|v| std::str::from_utf8(v).ok()).and_then(parse_version);
    let effective = match (header, catalog_version) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    if let (Some(max), Some(actual)) = (level.max_pdf_version(), effective) {
        if actual > max {
            sink.error(
                codes::PDFX_VERSION_MISMATCH,
                None,
                format!(
                    "the file claims {} but is PDF {}.{}; that standard allows PDF {}.{} at most",
                    lossy(claim.as_bytes()),
                    actual.0,
                    actual.1,
                    max.0,
                    max.1
                ),
            );
        }
    }
    if doc.trailer.get(b"ID").ok().and_then(|id| resolve(doc, id)).is_none() {
        sink.warning(
            codes::NO_FILE_ID,
            None,
            "the file claims PDF/X but its trailer has no ID; PDF/X requires a file identifier",
        );
    }
    Some(level)
}

/// The catalog's XMP metadata packet as text.
fn xmp_packet(doc: &Document, catalog: &Dictionary) -> Option<String> {
    let Some(Object::Stream(metadata)) = get(doc, catalog, b"Metadata") else { return None };
    let bytes = stream::decode(metadata, MAX_METADATA_BYTES).ok()?;
    Some(decode_xml_text(&bytes))
}

/// Decodes XML bytes: UTF-16 with a byte-order mark, UTF-8 (lossily) otherwise.
fn decode_xml_text(bytes: &[u8]) -> String {
    let utf16 = |data: &[u8], big_endian: bool| {
        let units: Vec<u16> = data
            .chunks_exact(2)
            .map(|pair| {
                if big_endian { u16::from_be_bytes([pair[0], pair[1]]) } else { u16::from_le_bytes([pair[0], pair[1]]) }
            })
            .collect();
        String::from_utf16_lossy(&units)
    };
    match bytes {
        [0xFE, 0xFF, rest @ ..] => utf16(rest, true),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, false),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// Finds a simple XMP property, written either as an element
/// (`<ns:Name>value</ns:Name>`) or as an attribute (`ns:Name="value"`).
fn xmp_property(xmp: &str, name: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(found) = xmp.get(search_from..)?.find(name) {
        let start = search_from + found;
        let after = start + name.len();
        search_from = after;
        let before = xmp[..start].chars().next_back();
        let rest = &xmp[after..];
        let value = match before {
            // Element form.
            Some('<') => {
                let rest = rest.trim_start();
                let Some(rest) = rest.strip_prefix('>') else { continue };
                rest.find('<').map(|end| &rest[..end])
            }
            // Attribute form.
            Some(c) if c.is_whitespace() => {
                let rest = rest.trim_start();
                let Some(rest) = rest.strip_prefix('=') else { continue };
                let rest = rest.trim_start();
                let quote = rest.chars().next().filter(|q| *q == '"' || *q == '\'');
                quote.and_then(|q| {
                    let body = &rest[1..];
                    body.find(q).map(|end| &body[..end])
                })
            }
            _ => None,
        };
        if let Some(value) = value {
            let value = value.trim();
            if !value.is_empty() && value.len() <= 64 {
                return Some(value.to_string());
            }
        }
    }
    None
}

// ---- output intent -------------------------------------------------------------------------

fn check_output_intent(
    doc: &Document,
    catalog: &Dictionary,
    level: Option<PdfxLevel>,
    expect: &Expectations,
    summary: &mut DocumentSummary,
    sink: &mut Sink,
) {
    // Problems with the intent are errors when one is required, warnings otherwise.
    let grade = if expect.require_output_intent { Severity::Error } else { Severity::Warning };
    let intents: Vec<&Dictionary> = get_array(doc, catalog, b"OutputIntents")
        .unwrap_or(&[])
        .iter()
        .take(64)
        .filter_map(|intent| as_dict(doc, intent))
        .collect();
    if intents.is_empty() {
        if expect.require_output_intent {
            sink.error(
                codes::NO_OUTPUT_INTENT,
                None,
                "the document has no output intent: the printing condition is undefined",
            );
        }
        return;
    }
    let pdfx: Vec<&Dictionary> =
        intents.iter().copied().filter(|intent| get_name(doc, intent, b"S") == Some(&b"GTS_PDFX"[..])).collect();
    let Some(intent) = pdfx.first().copied() else {
        let other = intents.first().and_then(|i| get_name(doc, i, b"S")).map_or_else(|| "?".to_string(), lossy);
        sink.push(
            grade,
            codes::OUTPUT_INTENT_INVALID,
            None,
            format!("the document has an output intent of type {other} but none of type GTS_PDFX"),
        );
        summary.output_intent = intents.first().map(|intent| summarise_intent(doc, intent, other.clone()));
        return;
    };
    if pdfx.len() > 1 {
        sink.push(
            grade,
            codes::OUTPUT_INTENT_INVALID,
            None,
            format!("the document has {} GTS_PDFX output intents; exactly one is allowed", pdfx.len()),
        );
    }
    let mut intent_summary = summarise_intent(doc, intent, "GTS_PDFX".to_string());
    if intent_summary.identifier.is_none() {
        sink.push(
            grade,
            codes::OUTPUT_INTENT_INVALID,
            None,
            "the output intent has no OutputConditionIdentifier",
        );
    }
    match intent.get(b"DestOutputProfile").ok().and_then(|profile| resolve(doc, profile)) {
        Some(Object::Stream(profile)) => {
            intent_summary.profile_embedded = true;
            let declared = get_int(doc, &profile.dict, b"N").and_then(|n| u8::try_from(n).ok());
            intent_summary.profile_components = declared;
            match stream::decode(profile, MAX_METADATA_BYTES).ok().as_deref().and_then(parse_icc_header) {
                Some(header) => {
                    intent_summary.profile_color_space = Some(header.color_space.clone());
                    match (declared, header.components) {
                        (Some(n), Some(actual)) if n != actual => sink.error(
                            codes::OUTPUT_INTENT_PROFILE_MISMATCH,
                            None,
                            format!(
                                "the output profile stream says N = {n} but the ICC profile is {} ({actual} components)",
                                header.color_space
                            ),
                        ),
                        (None, _) => sink.push(
                            grade,
                            codes::OUTPUT_INTENT_INVALID,
                            None,
                            "the output profile stream has no valid N (number of components)",
                        ),
                        _ => {}
                    }
                    if header.class != "prtr" {
                        sink.warning(
                            codes::OUTPUT_INTENT_INVALID,
                            None,
                            format!(
                                "the output profile is of class '{}', not an output device profile ('prtr')",
                                header.class
                            ),
                        );
                    }
                }
                None => sink.push(
                    grade,
                    codes::OUTPUT_INTENT_INVALID,
                    None,
                    "the output profile is not a readable ICC profile",
                ),
            }
        }
        Some(_) => sink.push(
            grade,
            codes::OUTPUT_INTENT_INVALID,
            None,
            "the output intent's DestOutputProfile is not a stream",
        ),
        None => {
            if level.is_some_and(PdfxLevel::requires_embedded_profile) {
                sink.push(
                    grade,
                    codes::OUTPUT_INTENT_INVALID,
                    None,
                    "the output intent has no embedded profile (DestOutputProfile), which PDF/X-4 requires",
                );
            } else if intent_summary.registry.is_none() {
                sink.push(
                    grade,
                    codes::OUTPUT_INTENT_INVALID,
                    None,
                    "the output intent has neither an embedded profile nor a RegistryName for its identifier",
                );
            }
        }
    }
    summary.output_intent = Some(intent_summary);
}

fn summarise_intent(doc: &Document, intent: &Dictionary, subtype: String) -> OutputIntentSummary {
    let text = |key: &[u8]| get_text(doc, intent, key).map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
    OutputIntentSummary {
        subtype,
        identifier: text(b"OutputConditionIdentifier"),
        description: text(b"Info").or_else(|| text(b"OutputCondition")),
        registry: text(b"RegistryName"),
        profile_embedded: false,
        profile_components: None,
        profile_color_space: None,
    }
}

// ---- Info ----------------------------------------------------------------------------------

fn check_info(
    doc: &Document,
    info: Option<&Dictionary>,
    level: Option<PdfxLevel>,
    summary: &mut DocumentSummary,
    sink: &mut Sink,
) {
    // PDF/X makes Title and Trapped mandatory; elsewhere they are good practice.
    let grade = if level.is_some() { Severity::Error } else { Severity::Warning };
    summary.title =
        info.and_then(|info| get_text(doc, info, b"Title")).map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
    if summary.title.is_none() {
        sink.push(grade, codes::NO_TITLE, None, "the Info dictionary has no Title");
    }
    summary.trapped = info.and_then(|info| get_text(doc, info, b"Trapped")).map(|t| t.trim().to_string());
    match summary.trapped.as_deref() {
        Some("True" | "False") => {}
        Some(other) => sink.push(
            grade,
            codes::TRAPPED_NOT_SET,
            None,
            format!("Info Trapped is \"{}\"; it must be True or False", lossy(other.as_bytes())),
        ),
        None => sink.push(grade, codes::TRAPPED_NOT_SET, None, "the Info dictionary has no Trapped key"),
    }
}

// ---- EBC.* geometry ------------------------------------------------------------------------

/// Parses a decimal number as written in an `EBC.*` key.
fn parse_mm(text: &str) -> Option<f64> {
    let value: f64 = text.trim().parse().ok()?;
    (value.is_finite() && (0.0..=100_000.0).contains(&value)).then_some(value)
}

/// Checks the `EBC.*` Info keys against the trim size of the first page and against the
/// expectations, and the QR module size against the minimum.
pub(crate) fn check_ebc_geometry(
    ebc: &BTreeMap<String, String>,
    trim_mm: Option<(f64, f64)>,
    qr_candidates: &[(u32, QrCandidate)],
    expect: &Expectations,
    sink: &mut Sink,
) {
    let mut number = |key: &str| -> Option<f64> {
        let raw = ebc.get(key)?;
        let parsed = parse_mm(raw);
        if parsed.is_none() {
            sink.error(
                codes::EBC_KEY_INVALID,
                None,
                format!("Info key {key} is \"{raw}\", not a number of millimetres"),
            );
        }
        parsed
    };
    let spine = number("EBC.SpineWidthMm");
    let cover_w = number("EBC.CoverWidthMm");
    let cover_h = number("EBC.CoverHeightMm");
    let qr_module = number("EBC.QrModuleMm");
    let folds: Option<Vec<f64>> = ebc.get("EBC.FoldXMm").and_then(|raw| {
        let parsed: Option<Vec<f64>> = raw.split(',').take(16).map(parse_mm).collect();
        if parsed.is_none() {
            sink.error(
                codes::EBC_KEY_INVALID,
                None,
                format!("Info key EBC.FoldXMm is \"{raw}\", not a comma-separated list of millimetres"),
            );
        }
        parsed
    });
    let close = |a: f64, b: f64| (a - b).abs() <= TRIM_TOLERANCE_MM;
    let mut mismatch = |message: String| sink.error(codes::EBC_GEOMETRY_MISMATCH, None, message);

    if let Some((trim_w, trim_h)) = trim_mm {
        if let Some(height) = cover_h {
            if !close(height, trim_h) {
                mismatch(format!("EBC.CoverHeightMm is {height} but the trim box is {trim_h:.2} mm high"));
            }
        }
        if let (Some(width), Some(spine)) = (cover_w, spine) {
            let total = 2.0 * width + spine;
            if !close(total, trim_w) {
                mismatch(format!(
                    "two covers of {width} mm plus a {spine} mm spine make {total:.2} mm, but the trim box is {trim_w:.2} mm wide"
                ));
            }
        }
        if let Some(folds) = &folds {
            if folds.iter().any(|&x| x < -TRIM_TOLERANCE_MM || x > trim_w + TRIM_TOLERANCE_MM) {
                mismatch(format!("EBC.FoldXMm has a fold outside the trim box (0 to {trim_w:.2} mm)"));
            } else if let [first, second] = folds.as_slice() {
                // A card has two folds, symmetric about the middle of the spread.
                if !close(first + second, trim_w) {
                    mismatch(format!(
                        "the folds at {first} and {second} mm are not centred on the {trim_w:.2} mm wide trim box"
                    ));
                }
            }
        }
    }
    if let (Some(folds), Some(spine)) = (&folds, spine) {
        if let [first, second] = folds.as_slice() {
            if !close((second - first).abs(), spine) {
                mismatch(format!(
                    "the folds at {first} and {second} mm are {:.2} mm apart but EBC.SpineWidthMm is {spine}",
                    (second - first).abs()
                ));
            }
        }
    }
    if let (Some(folds), Some(width)) = (&folds, cover_w) {
        if let Some(first) = folds.iter().copied().reduce(f64::min) {
            if !close(first, width) {
                mismatch(format!("the first fold is at {first} mm but EBC.CoverWidthMm is {width}"));
            }
        }
    }

    if let Some(expected) = expect.spine_mm {
        // The declared spine, or the distance between the two folds.
        let declared = spine.or_else(|| match folds.as_deref() {
            Some([first, second]) => Some((second - first).abs()),
            _ => None,
        });
        match declared {
            Some(actual) if !close(actual, expected) => sink.error(
                codes::EBC_SPINE_MISMATCH,
                None,
                format!("the file declares a spine of {actual:.2} mm, expected {expected:.2} mm"),
            ),
            Some(_) => {}
            // An unparseable key has already been reported.
            None if ebc.contains_key("EBC.SpineWidthMm") => {}
            None => sink.error(
                codes::EBC_SPINE_MISSING,
                None,
                format!(
                    "a spine of {expected:.2} mm is expected but the file declares none (no EBC.SpineWidthMm or EBC.FoldXMm in Info)"
                ),
            ),
        }
    }

    if let Some(minimum) = expect.min_qr_module_mm {
        // A hundredth of a millimetre of slack for rounding to points.
        let too_small = |module: f64| module < minimum - 0.01;
        if let Some(module) = qr_module {
            if too_small(module) {
                sink.error(
                    codes::QR_MODULE_TOO_SMALL,
                    None,
                    format!("EBC.QrModuleMm declares {module} mm modules, below the minimum of {minimum} mm"),
                );
            }
        }
        for (page, candidate) in qr_candidates {
            if too_small(candidate.module_mm) {
                sink.error(
                    codes::QR_MODULE_TOO_SMALL,
                    Some(*page),
                    format!(
                        "a QR-like graphic ({0}×{0} modules, {1:.1} mm wide) has {2:.3} mm modules, below the minimum of {minimum} mm",
                        candidate.modules, candidate.side_mm, candidate.module_mm
                    ),
                );
            }
        }
        if qr_candidates.is_empty() && qr_module.is_none() {
            sink.info(
                codes::QR_NOT_FOUND,
                None,
                "no QR-like vector graphic was recognised, so the module size could not be verified",
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn pdfx_levels_and_versions() {
        assert_eq!(PdfxLevel::parse("PDF/X-1a:2001"), PdfxLevel::X1a2001);
        assert_eq!(PdfxLevel::parse("PDF/X-1:2001"), PdfxLevel::X1a2001);
        assert_eq!(PdfxLevel::parse(" PDF/X-1a:2003 "), PdfxLevel::X1a2003);
        assert_eq!(PdfxLevel::parse("PDF/X-3:2002"), PdfxLevel::X3v2002);
        assert_eq!(PdfxLevel::parse("PDF/X-3:2003"), PdfxLevel::X3v2003);
        assert_eq!(PdfxLevel::parse("PDF/X-4"), PdfxLevel::X4);
        assert_eq!(PdfxLevel::parse("PDF/X-4p"), PdfxLevel::X4);
        assert_eq!(PdfxLevel::parse("PDF/X-5g"), PdfxLevel::X5);
        assert_eq!(PdfxLevel::parse("PDF/X-6"), PdfxLevel::X6);
        assert_eq!(PdfxLevel::parse("PDF/A-1b"), PdfxLevel::Other);
        assert_eq!(PdfxLevel::X1a2001.max_pdf_version(), Some((1, 3)));
        assert_eq!(PdfxLevel::X4.max_pdf_version(), Some((1, 6)));
        assert!(PdfxLevel::X3v2002.forbids_layers() && !PdfxLevel::X4.forbids_layers());
        assert_eq!(parse_version("1.6"), Some((1, 6)));
        assert_eq!(parse_version("2.0"), Some((2, 0)));
        assert_eq!(parse_version("1"), None);
        assert_eq!(parse_version("x.y"), None);
        assert!((1, 7) > (1, 6) && (2, 0) > (1, 7));
    }

    #[test]
    fn xmp_properties_in_both_syntaxes() {
        let element = r#"<rdf:Description rdf:about="" xmlns:pdfxid="http://www.npes.org/pdfx/ns/id/">
            <pdfxid:GTS_PDFXVersion>PDF/X-4</pdfxid:GTS_PDFXVersion></rdf:Description>"#;
        assert_eq!(xmp_property(element, "pdfxid:GTS_PDFXVersion").as_deref(), Some("PDF/X-4"));
        let attribute = r#"<rdf:Description rdf:about=""
            pdfxid:GTS_PDFXVersion = 'PDF/X-4p' pdf:Trapped="False"/>"#;
        assert_eq!(xmp_property(attribute, "pdfxid:GTS_PDFXVersion").as_deref(), Some("PDF/X-4p"));
        assert_eq!(xmp_property(attribute, "pdf:Trapped").as_deref(), Some("False"));
        // The closing tag, a namespace declaration that merely contains the name, an empty
        // element and truncated packets are not values.
        assert_eq!(xmp_property("</pdfxid:GTS_PDFXVersion>", "pdfxid:GTS_PDFXVersion"), None);
        assert_eq!(xmp_property("<pdfxid:GTS_PDFXVersion></pdfxid:GTS_PDFXVersion>", "pdfxid:GTS_PDFXVersion"), None);
        assert_eq!(xmp_property("<pdfxid:GTS_PDFXVersion>PDF/X-4", "pdfxid:GTS_PDFXVersion"), None);
        assert_eq!(xmp_property(" pdfxid:GTS_PDFXVersion=\"PDF/X-4", "pdfxid:GTS_PDFXVersion"), None);
        assert_eq!(xmp_property("", "pdfxid:GTS_PDFXVersion"), None);
        // UTF-16 packets are decoded first.
        let mut utf16 = vec![0xFE, 0xFF];
        for unit in element.encode_utf16() {
            utf16.extend_from_slice(&unit.to_be_bytes());
        }
        let text = decode_xml_text(&utf16);
        assert_eq!(xmp_property(&text, "pdfxid:GTS_PDFXVersion").as_deref(), Some("PDF/X-4"));
        // Multi-byte text around the property must not break slicing.
        let unicode = "é<dc:title>Été</dc:title> pdfxid:GTS_PDFXVersion=\"PDF/X-4\" ü";
        assert_eq!(xmp_property(unicode, "pdfxid:GTS_PDFXVersion").as_deref(), Some("PDF/X-4"));
    }

    fn ebc(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect()
    }

    fn ebc_codes(
        keys: &BTreeMap<String, String>,
        trim: Option<(f64, f64)>,
        qr: &[(u32, QrCandidate)],
        expect: &Expectations,
    ) -> Vec<&'static str> {
        let mut sink = Sink::default();
        check_ebc_geometry(keys, trim, qr, expect, &mut sink);
        sink.into_sorted().into_iter().map(|f| f.code).collect()
    }

    const CARD: Option<(f64, f64)> = Some((230.0, 178.0));

    #[test]
    fn consistent_card_geometry_passes() {
        let keys = ebc(&[
            ("EBC.SpineWidthMm", "10"),
            ("EBC.FoldXMm", "110, 120"),
            ("EBC.CoverWidthMm", "110"),
            ("EBC.CoverHeightMm", "178.0"),
        ]);
        let expect = Expectations::professional().with_spine(10.0);
        assert!(ebc_codes(&keys, CARD, &[], &expect).is_empty());
        // No keys and no expectation: nothing to check.
        assert!(ebc_codes(&BTreeMap::new(), CARD, &[], &Expectations::professional()).is_empty());
    }

    #[test]
    fn geometry_contradictions_are_reported() {
        let expect = Expectations::professional();
        // Wrong height.
        let keys = ebc(&[("EBC.CoverHeightMm", "180")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_GEOMETRY_MISMATCH]);
        // Covers and spine do not add up to the trim width.
        let keys = ebc(&[("EBC.CoverWidthMm", "110"), ("EBC.SpineWidthMm", "12")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_GEOMETRY_MISMATCH]);
        // Folds not centred / not a spine apart / outside the page.
        let keys = ebc(&[("EBC.FoldXMm", "100,110")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_GEOMETRY_MISMATCH]);
        let keys = ebc(&[("EBC.FoldXMm", "108,122"), ("EBC.SpineWidthMm", "10")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_GEOMETRY_MISMATCH]);
        let keys = ebc(&[("EBC.FoldXMm", "110,240")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_GEOMETRY_MISMATCH]);
        // First fold must be one cover width from the left.
        let keys = ebc(&[("EBC.FoldXMm", "110,120"), ("EBC.CoverWidthMm", "105")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_GEOMETRY_MISMATCH]);
    }

    #[test]
    fn malformed_keys_are_reported_once() {
        let expect = Expectations::professional().with_spine(10.0);
        for bad in ["ten", "", "NaN", "inf", "-3", "1e9", "10,0"] {
            let keys = ebc(&[("EBC.SpineWidthMm", bad)]);
            assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_KEY_INVALID], "{bad:?}");
        }
        let keys = ebc(&[("EBC.FoldXMm", "110;120")]);
        assert_eq!(
            ebc_codes(&keys, CARD, &[], &Expectations::professional()),
            vec![codes::EBC_KEY_INVALID]
        );
    }

    #[test]
    fn expected_spine_is_verified() {
        let expect = Expectations::professional().with_spine(10.0);
        assert_eq!(ebc_codes(&BTreeMap::new(), CARD, &[], &expect), vec![codes::EBC_SPINE_MISSING]);
        let keys = ebc(&[("EBC.SpineWidthMm", "12"), ("EBC.CoverWidthMm", "109")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_SPINE_MISMATCH]);
        // Within a tenth of a millimetre is fine.
        let keys = ebc(&[("EBC.SpineWidthMm", "10.05")]);
        assert!(ebc_codes(&keys, CARD, &[], &expect).is_empty());
        // The folds alone are enough to derive the spine.
        let keys = ebc(&[("EBC.FoldXMm", "110,120")]);
        assert!(ebc_codes(&keys, CARD, &[], &expect).is_empty());
        let keys = ebc(&[("EBC.FoldXMm", "109,121")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::EBC_SPINE_MISMATCH]);
    }

    #[test]
    fn qr_module_size_is_verified() {
        let expect = Expectations::professional().with_min_qr_module(0.4);
        let small = QrCandidate { module_mm: 0.3, side_mm: 7.5, modules: 25 };
        let fine = QrCandidate { module_mm: 0.395, side_mm: 9.9, modules: 25 };
        assert_eq!(ebc_codes(&BTreeMap::new(), CARD, &[], &expect), vec![codes::QR_NOT_FOUND]);
        assert!(ebc_codes(&BTreeMap::new(), CARD, &[(2, fine.clone())], &expect).is_empty());
        assert_eq!(
            ebc_codes(&BTreeMap::new(), CARD, &[(2, fine), (2, small)], &expect),
            vec![codes::QR_MODULE_TOO_SMALL]
        );
        let keys = ebc(&[("EBC.QrModuleMm", "0.25")]);
        assert_eq!(ebc_codes(&keys, CARD, &[], &expect), vec![codes::QR_MODULE_TOO_SMALL]);
        let keys = ebc(&[("EBC.QrModuleMm", "0.5")]);
        assert!(ebc_codes(&keys, CARD, &[], &expect).is_empty());
        // Without the expectation nothing is checked.
        assert!(ebc_codes(&BTreeMap::new(), CARD, &[], &Expectations::professional()).is_empty());
    }
}
