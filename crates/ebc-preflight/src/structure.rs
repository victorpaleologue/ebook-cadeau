//! Document-structure checks: things a print file must not contain (encryption is handled
//! by the caller; here: JavaScript, actions, annotations, forms, attachments, layers).

use lopdf::{Dictionary, Document, Object};

use crate::codes;
use crate::pages::{PageBoxes, PageNode};
use crate::pdf::{RectPt, as_dict, get, get_array, get_dict, get_name, lossy, resolve};
use crate::sink::Sink;

/// Nesting depth followed inside one indirect object when scanning for forbidden content.
/// (`lopdf` itself refuses to parse objects nested deeper than 100 levels.)
const MAX_SCAN_DEPTH: usize = 128;
/// Annotations examined per page.
const MAX_ANNOTATIONS: usize = 10_000;

/// Catalog-level checks plus a scan of every object for JavaScript and embedded files.
pub(crate) fn check_document(doc: &Document, catalog: &Dictionary, sink: &mut Sink) {
    match catalog.get(b"OpenAction").ok().and_then(|action| resolve(doc, action)) {
        Some(Object::Array(_)) => sink.warning(
            codes::ACTION,
            None,
            "the catalog has an OpenAction destination; print files should not set a view on opening",
        ),
        Some(_) => sink.error(codes::ACTION, None, "the catalog has an OpenAction action"),
        None => {}
    }
    if get(doc, catalog, b"AA").is_some() {
        sink.error(codes::ACTION, None, "the catalog has additional actions (AA)");
    }
    if let Some(form) = get_dict(doc, catalog, b"AcroForm") {
        let fields = get_array(doc, form, b"Fields").map_or(0, <[Object]>::len);
        if fields > 0 || get(doc, form, b"XFA").is_some() {
            sink.error(
                codes::ACROFORM,
                None,
                format!("the document contains an interactive form ({fields} top-level field(s))"),
            );
        } else {
            sink.warning(
                codes::ACROFORM,
                None,
                "the document declares an (empty) interactive form",
            );
        }
    }
    if let Some(names) = get_dict(doc, catalog, b"Names") {
        if get_dict(doc, names, b"JavaScript").is_some_and(|tree| name_tree_has_entries(doc, tree))
        {
            sink.error(
                codes::JAVASCRIPT,
                None,
                "the document has document-level JavaScript (Names/JavaScript)",
            );
        }
        if get_dict(doc, names, b"EmbeddedFiles")
            .is_some_and(|tree| name_tree_has_entries(doc, tree))
        {
            sink.error(
                codes::EMBEDDED_FILE,
                None,
                "the document has file attachments (Names/EmbeddedFiles)",
            );
        }
    }
    scan_objects(doc, sink);
}

/// Whether the root of a name tree lists any name or has any child node.
fn name_tree_has_entries(doc: &Document, tree: &Dictionary) -> bool {
    [&b"Names"[..], b"Kids"]
        .iter()
        .any(|key| get_array(doc, tree, key).is_some_and(|entries| !entries.is_empty()))
}

/// Reports optional content (layers); the severity depends on the PDF/X level claimed.
pub(crate) fn check_optional_content(
    doc: &Document,
    catalog: &Dictionary,
    forbids_layers: bool,
    sink: &mut Sink,
) {
    let Some(properties) = get_dict(doc, catalog, b"OCProperties") else { return };
    let groups = get_array(doc, properties, b"OCGs").map_or(0, <[Object]>::len);
    let message = format!("the document uses optional content (layers): {groups} group(s)");
    if forbids_layers {
        sink.warning(
            codes::OPTIONAL_CONTENT,
            None,
            format!("{message}, which PDF/X-1a and PDF/X-3 do not allow"),
        );
    } else {
        sink.info(codes::OPTIONAL_CONTENT, None, message);
    }
}

/// Looks into every object of the file (reachable or not) for JavaScript actions and
/// embedded files. Iterative, so that deep nesting cannot overflow the stack.
fn scan_objects(doc: &Document, sink: &mut Sink) {
    let mut javascript = 0_usize;
    let mut embedded = 0_usize;
    let mut stack: Vec<(&Object, usize)> = Vec::new();
    for object in doc.objects.values() {
        stack.push((object, 0));
        while let Some((object, depth)) = stack.pop() {
            let dict = match object {
                Object::Dictionary(dict) => dict,
                Object::Stream(stream) => &stream.dict,
                Object::Array(items) => {
                    if depth < MAX_SCAN_DEPTH {
                        stack.extend(items.iter().map(|item| (item, depth + 1)));
                    }
                    continue;
                }
                _ => continue,
            };
            let is_name = |key: &[u8], value: &[u8]| matches!(dict.get(key), Ok(Object::Name(name)) if name == value);
            if dict.has(b"JS") || is_name(b"S", b"JavaScript") {
                javascript += 1;
            }
            if is_name(b"Type", b"EmbeddedFile") || dict.has(b"EF") {
                embedded += 1;
            }
            if depth < MAX_SCAN_DEPTH {
                stack.extend(dict.iter().map(|(_, value)| (value, depth + 1)));
            }
        }
    }
    if javascript > 0 {
        sink.error(
            codes::JAVASCRIPT,
            None,
            format!("the file contains JavaScript ({javascript} action(s) or script object(s))"),
        );
    }
    if embedded > 0 {
        sink.error(
            codes::EMBEDDED_FILE,
            None,
            format!(
                "the file contains embedded files ({embedded} file specification(s) or stream(s))"
            ),
        );
    }
}

/// Page-level checks: additional actions and annotations.
pub(crate) fn check_page(
    doc: &Document,
    node: &PageNode<'_>,
    boxes: &PageBoxes,
    page: u32,
    sink: &mut Sink,
) {
    if get(doc, node.dict, b"AA").is_some() {
        sink.error(codes::ACTION, Some(page), "the page has additional actions (AA)");
    }
    let Some(annotations) = get_array(doc, node.dict, b"Annots") else { return };
    let printed = boxes.printed_area();
    let mut inside: Vec<String> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    for annotation in annotations.iter().take(MAX_ANNOTATIONS) {
        let Some(dict) = as_dict(doc, annotation) else {
            inside.push("(malformed)".to_string());
            continue;
        };
        let subtype =
            get_name(doc, dict, b"Subtype").map_or_else(|| "(no subtype)".to_string(), lossy);
        // Printer's marks and trap networks are the annotations PDF/X expects.
        if subtype == "PrinterMark" || subtype == "TrapNet" {
            continue;
        }
        let rect = dict.get(b"Rect").ok().and_then(|rect| RectPt::from_object(doc, rect));
        match (rect, printed) {
            (Some(rect), Some(printed)) if !printed.overlaps(&rect, 0.0) => outside.push(subtype),
            _ => inside.push(subtype),
        }
    }
    if annotations.len() > MAX_ANNOTATIONS {
        inside.push("(more)".to_string());
    }
    if !inside.is_empty() {
        sink.error(
            codes::ANNOTATION,
            Some(page),
            format!(
                "the page has {} annotation(s) in the printed area: {}",
                inside.len(),
                summarise(&inside)
            ),
        );
    }
    if !outside.is_empty() {
        sink.warning(
            codes::ANNOTATION,
            Some(page),
            format!(
                "the page has {} annotation(s) outside the bleed box: {}",
                outside.len(),
                summarise(&outside)
            ),
        );
    }
}

/// `"Link ×3, Text"` — the distinct subtypes with their counts, at most a handful.
fn summarise(subtypes: &[String]) -> String {
    let mut counted: Vec<(&str, usize)> = Vec::new();
    for subtype in subtypes {
        match counted.iter_mut().find(|(name, _)| *name == subtype) {
            Some((_, count)) => *count += 1,
            None => counted.push((subtype, 1)),
        }
    }
    let mut parts: Vec<String> = counted
        .iter()
        .take(6)
        .map(
            |(name, count)| {
                if *count > 1 { format!("{name} ×{count}") } else { (*name).to_string() }
            },
        )
        .collect();
    if counted.len() > 6 {
        parts.push("…".to_string());
    }
    parts.join(", ")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use lopdf::{Stream, dictionary};

    use super::*;
    use crate::report::Severity;

    fn findings(doc: &Document, catalog: &Dictionary) -> Vec<(&'static str, Severity)> {
        let mut sink = Sink::default();
        check_document(doc, catalog, &mut sink);
        sink.into_sorted().into_iter().map(|f| (f.code, f.severity)).collect()
    }

    #[test]
    fn a_plain_catalog_is_clean() {
        let mut doc = Document::with_version("1.6");
        doc.add_object(dictionary! { "S" => "Transparency", "Type" => "Group" });
        doc.add_object(dictionary! { "Type" => "OutputIntent", "S" => "GTS_PDFX" });
        let catalog = dictionary! { "Type" => "Catalog", "Names" => dictionary! { "Dests" => dictionary! {} } };
        assert!(findings(&doc, &catalog).is_empty());
    }

    #[test]
    fn empty_name_trees_are_not_findings() {
        let doc = Document::with_version("1.6");
        let catalog = dictionary! { "Names" => dictionary! {
            "JavaScript" => dictionary! { "Names" => Vec::<Object>::new() },
            "EmbeddedFiles" => dictionary! { "Kids" => Vec::<Object>::new(), "Limits" => vec![Object::Null] },
        } };
        assert!(findings(&doc, &catalog).is_empty());
        // A tree that delegates to child nodes does have entries.
        let catalog = dictionary! { "Names" => dictionary! {
            "JavaScript" => dictionary! { "Kids" => vec![Object::Reference((5, 0))] },
        } };
        assert_eq!(findings(&doc, &catalog), vec![(codes::JAVASCRIPT, Severity::Error)]);
    }

    #[test]
    fn javascript_is_found_wherever_it_hides() {
        // As an OpenAction.
        let mut doc = Document::with_version("1.6");
        let action = doc.add_object(
            dictionary! { "S" => "JavaScript", "JS" => Object::string_literal("app.alert(1)") },
        );
        let catalog = dictionary! { "OpenAction" => action };
        assert_eq!(
            findings(&doc, &catalog),
            vec![(codes::ACTION, Severity::Error), (codes::JAVASCRIPT, Severity::Error)]
        );
        // Nested deep inside an unreferenced object, and in a name tree.
        let mut doc = Document::with_version("1.6");
        let mut nested = Object::Dictionary(dictionary! { "JS" => Object::string_literal("x") });
        for _ in 0..50 {
            nested = Object::Array(vec![Object::Dictionary(dictionary! { "Next" => nested })]);
        }
        doc.add_object(nested);
        let catalog = dictionary! {
            "Names" => dictionary! { "JavaScript" => dictionary! { "Names" => vec![Object::string_literal("a")] } },
        };
        let got = findings(&doc, &catalog);
        assert_eq!(
            got,
            vec![(codes::JAVASCRIPT, Severity::Error), (codes::JAVASCRIPT, Severity::Error)]
        );
    }

    #[test]
    fn actions_forms_and_attachments() {
        let mut doc = Document::with_version("1.6");
        let file =
            doc.add_object(Stream::new(dictionary! { "Type" => "EmbeddedFile" }, b"data".to_vec()));
        let spec = doc
            .add_object(dictionary! { "Type" => "Filespec", "EF" => dictionary! { "F" => file } });
        let field =
            doc.add_object(dictionary! { "FT" => "Tx", "T" => Object::string_literal("name") });
        let catalog = dictionary! {
            "OpenAction" => vec![Object::Reference((1, 0)), "Fit".into()],
            "AA" => dictionary! { "WC" => dictionary! { "S" => "Named", "N" => "Print" } },
            "AcroForm" => dictionary! { "Fields" => vec![field.into()] },
            "Names" => dictionary! { "EmbeddedFiles" => dictionary! { "Names" => vec![Object::string_literal("a"), spec.into()] } },
        };
        assert_eq!(
            findings(&doc, &catalog),
            vec![
                (codes::ACTION, Severity::Warning),
                (codes::ACTION, Severity::Error),
                (codes::ACROFORM, Severity::Error),
                (codes::EMBEDDED_FILE, Severity::Error),
                (codes::EMBEDDED_FILE, Severity::Error),
            ]
        );
        // An empty form is only a warning.
        let catalog =
            dictionary! { "AcroForm" => dictionary! { "Fields" => Vec::<Object>::new() } };
        assert_eq!(
            findings(&Document::with_version("1.6"), &catalog),
            vec![(codes::ACROFORM, Severity::Warning)]
        );
    }

    #[test]
    fn layers_are_graded_by_pdfx_level() {
        let doc = Document::with_version("1.6");
        let catalog = dictionary! { "OCProperties" => dictionary! { "OCGs" => vec![Object::Null, Object::Null] } };
        for (forbids, severity) in [(true, Severity::Warning), (false, Severity::Info)] {
            let mut sink = Sink::default();
            check_optional_content(&doc, &catalog, forbids, &mut sink);
            let got = sink.into_sorted();
            assert_eq!(got.len(), 1);
            assert_eq!((got[0].code, got[0].severity), (codes::OPTIONAL_CONTENT, severity));
            assert!(got[0].message.contains("2 group(s)"));
        }
    }

    #[test]
    fn annotation_counts_are_summarised() {
        let subtypes: Vec<String> =
            ["Link", "Text", "Link", "Link"].iter().map(|s| (*s).to_string()).collect();
        assert_eq!(summarise(&subtypes), "Link ×3, Text");
        let many: Vec<String> = (0..10).map(|i| format!("T{i}")).collect();
        assert_eq!(summarise(&many), "T0, T1, T2, T3, T4, T5, …");
    }
}
