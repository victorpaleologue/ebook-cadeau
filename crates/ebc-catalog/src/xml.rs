//! XML read into a small owned tree, plus the two envelopes catalogues use: SRU
//! (`searchRetrieveResponse`) and MARC-XML / MarcXchange records.
//!
//! The parser never resolves external entities or loads a DTD (`quick-xml` does neither),
//! and it bounds the nesting depth.

use ebc_model::HttpResponse;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::error::{CatalogError, shorten};
use crate::normalize::resolve_entity;
use crate::util::{Sniff, declared_charset, decode_text, sniff};

/// Deepest element nesting accepted. Catalogue answers use fewer than twenty levels.
const MAX_DEPTH: usize = 96;

/// An XML element: local name (namespace prefix removed), attributes, the text directly
/// inside it and its child elements.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) text: String,
    pub(crate) children: Vec<Node>,
}

impl Node {
    /// The value of an attribute, by local name.
    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    /// The first child element called `name`.
    pub(crate) fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    /// All child elements called `name`.
    pub(crate) fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// Trimmed text of the first child called `name`, when not empty.
    pub(crate) fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.trim()).filter(|t| !t.is_empty())
    }

    /// Every element called `name` in this subtree (this node included), in document order.
    pub(crate) fn find_all<'a>(&'a self, name: &str, out: &mut Vec<&'a Node>) {
        if self.name == name {
            out.push(self);
        }
        // Depth is bounded by MAX_DEPTH at parse time, so recursion is safe.
        for child in &self.children {
            child.find_all(name, out);
        }
    }
}

fn local_name(qualified: &str) -> String {
    qualified.rsplit(':').next().unwrap_or(qualified).to_owned()
}

/// Decodes character and entity references in an attribute value.
fn unescape(raw: &str) -> String {
    if !raw.contains('&') {
        return raw.to_owned();
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp + 1..];
        match tail.find(';').filter(|end| *end <= 12) {
            Some(end) => match resolve_entity(&tail[..end]) {
                Some(text) => {
                    out.push_str(&text);
                    rest = &tail[end + 1..];
                }
                None => {
                    out.push('&');
                    rest = tail;
                }
            },
            None => {
                out.push('&');
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

fn element(start: &BytesStart<'_>) -> Node {
    let mut node = Node { name: local_name(start.name().as_ref()), ..Node::default() };
    // Malformed attributes are skipped rather than failing the whole document.
    for attribute in start.attributes().flatten() {
        let key = attribute.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        node.attrs.push((local_name(key), unescape(&attribute.value)));
    }
    node
}

fn malformed(provider: &'static str, detail: &str) -> CatalogError {
    CatalogError::Malformed { provider, format: "XML", detail: shorten(detail) }
}

/// The encoding named in the XML declaration, lower-cased.
fn declared_in_prolog(body: &[u8]) -> Option<String> {
    let head = String::from_utf8_lossy(body.get(..body.len().min(200))?).to_ascii_lowercase();
    let declaration = head.split("?>").next()?;
    let (_, after) = declaration.split_once("encoding")?;
    let value = after.trim_start().strip_prefix('=')?.trim_start();
    let quote = value.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    value[1..].split(quote).next().map(str::to_owned)
}

/// Parses an XML body into a tree.
///
/// # Errors
/// [`CatalogError::UnexpectedContent`] for HTML, JSON or empty bodies,
/// [`CatalogError::Malformed`] for broken or truncated XML.
pub(crate) fn parse_xml(
    provider: &'static str,
    response: &HttpResponse,
) -> Result<Node, CatalogError> {
    match sniff(&response.body) {
        Sniff::Xml => {}
        other => {
            return Err(CatalogError::UnexpectedContent {
                provider,
                expected: "XML",
                found: other.describe(),
            });
        }
    }
    let declared = declared_in_prolog(&response.body).or_else(|| declared_charset(response));
    let text = decode_text(&response.body, declared.as_deref());
    parse_xml_text(provider, &text)
}

pub(crate) fn parse_xml_text(provider: &'static str, text: &str) -> Result<Node, CatalogError> {
    let mut reader = Reader::from_str(text.trim_start_matches('\u{FEFF}'));
    let mut stack: Vec<Node> = Vec::new();
    let mut root: Option<Node> = None;
    let attach = |stack: &mut Vec<Node>, root: &mut Option<Node>, node: Node| match stack.last_mut()
    {
        Some(parent) => parent.children.push(node),
        None => {
            if root.is_none() {
                *root = Some(node);
            }
        }
    };
    loop {
        let event = reader.read_event().map_err(|e| malformed(provider, &e.to_string()))?;
        match event {
            Event::Start(start) => {
                if stack.len() >= MAX_DEPTH {
                    return Err(malformed(provider, "elements are nested too deeply"));
                }
                stack.push(element(&start));
            }
            Event::Empty(start) => {
                let node = element(&start);
                attach(&mut stack, &mut root, node);
            }
            Event::End(_) => match stack.pop() {
                Some(node) => attach(&mut stack, &mut root, node),
                None => return Err(malformed(provider, "closing tag without an opening tag")),
            },
            Event::Text(content) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&content);
                }
            }
            Event::CData(content) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&content);
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(top) = stack.last_mut() {
                    match resolve_entity(&reference) {
                        Some(resolved) => top.text.push_str(&resolved),
                        None => {
                            top.text.push('&');
                            top.text.push_str(&reference);
                            top.text.push(';');
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if let Some(open) = stack.last() {
        return Err(malformed(
            provider,
            &format!("the document ends inside <{}> (truncated?)", open.name),
        ));
    }
    root.ok_or_else(|| malformed(provider, "no root element"))
}

// ---------------------------------------------------------------------------------------
// SRU
// ---------------------------------------------------------------------------------------

/// The `recordData` elements of an SRU `searchRetrieveResponse`.
///
/// # Errors
/// [`CatalogError::Remote`] when the server sent diagnostics and no record,
/// [`CatalogError::Malformed`] when the document is not an SRU response.
pub(crate) fn sru_records<'a>(
    provider: &'static str,
    root: &'a Node,
) -> Result<Vec<&'a Node>, CatalogError> {
    let diagnostic = |node: &Node| -> CatalogError {
        let mut found = Vec::new();
        node.find_all("diagnostic", &mut found);
        let message = found
            .first()
            .map(|d| {
                ["message", "details", "uri"]
                    .iter()
                    .filter_map(|part| d.child_text(part))
                    .collect::<Vec<_>>()
                    .join(" — ")
            })
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "SRU diagnostic without a message".to_owned());
        CatalogError::Remote { provider, message: shorten(&message) }
    };
    match root.name.as_str() {
        "searchRetrieveResponse" => {
            let records: Vec<&Node> = root
                .children_named("records")
                .flat_map(|list| list.children_named("record"))
                .filter_map(|record| record.child("recordData"))
                .collect();
            if records.is_empty()
                && let Some(diagnostics) = root.child("diagnostics")
            {
                return Err(diagnostic(diagnostics));
            }
            Ok(records)
        }
        "diagnostics" | "diagnostic" => Err(diagnostic(root)),
        other => Err(malformed(
            provider,
            &format!("root element <{other}> is not an SRU searchRetrieveResponse"),
        )),
    }
}

// ---------------------------------------------------------------------------------------
// MARC
// ---------------------------------------------------------------------------------------

/// A MARC data field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarcField {
    pub(crate) tag: String,
    pub(crate) ind1: char,
    pub(crate) ind2: char,
    pub(crate) subfields: Vec<(char, String)>,
}

impl MarcField {
    /// First non-empty subfield with this code.
    pub(crate) fn sub(&self, code: char) -> Option<&str> {
        self.subs(code).next()
    }

    /// All non-empty subfields with this code.
    pub(crate) fn subs(&self, code: char) -> impl Iterator<Item = &str> {
        self.subfields
            .iter()
            .filter(move |(c, v)| *c == code && !v.trim().is_empty())
            .map(|(_, v)| v.trim())
    }
}

/// A MARC record (MARC 21 or UNIMARC) read from MARC-XML or MarcXchange.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MarcRecord {
    /// `id` attribute of the record element (BnF: the ARK).
    pub(crate) id: Option<String>,
    pub(crate) controls: Vec<(String, String)>,
    pub(crate) fields: Vec<MarcField>,
}

impl MarcRecord {
    /// A control field (`001`, `003`, `008`…).
    pub(crate) fn control(&self, tag: &str) -> Option<&str> {
        self.controls
            .iter()
            .find(|(t, _)| t == tag)
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.trim().is_empty())
    }

    /// Data fields with this tag.
    pub(crate) fn fields<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a MarcField> {
        self.fields.iter().filter(move |f| f.tag == tag)
    }

    /// First occurrence of subfield `code` in the fields tagged `tag`.
    pub(crate) fn first(&self, tag: &str, code: char) -> Option<&str> {
        self.fields.iter().filter(|f| f.tag == tag).find_map(|f| f.sub(code))
    }
}

fn first_char(value: Option<&str>) -> char {
    value.and_then(|v| v.chars().next()).unwrap_or(' ')
}

/// Every MARC record in the subtree: elements called `record` that hold `datafield` or
/// `controlfield` children, whatever the namespace prefix or envelope.
pub(crate) fn marc_records(node: &Node) -> Vec<MarcRecord> {
    let mut candidates = Vec::new();
    node.find_all("record", &mut candidates);
    candidates
        .into_iter()
        .filter(|n| n.children.iter().any(|c| c.name == "datafield" || c.name == "controlfield"))
        .map(|n| MarcRecord {
            id: n.attr("id").map(str::to_owned),
            controls: n
                .children_named("controlfield")
                .filter_map(|c| Some((c.attr("tag")?.trim().to_owned(), c.text.clone())))
                .collect(),
            fields: n
                .children_named("datafield")
                .filter_map(|f| {
                    Some(MarcField {
                        tag: f.attr("tag")?.trim().to_owned(),
                        ind1: first_char(f.attr("ind1")),
                        ind2: first_char(f.attr("ind2")),
                        subfields: f
                            .children_named("subfield")
                            .map(|s| (first_char(s.attr("code")), s.text.clone()))
                            .collect(),
                    })
                })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Node, CatalogError> {
        parse_xml_text("test", text)
    }

    #[test]
    fn builds_a_tree_without_prefixes() {
        let root = parse(
            r#"<?xml version="1.0"?><!DOCTYPE a><a:root xmlns:a="urn:x" a:id="1&amp;2">
               <a:item>Caf&#233; &amp; <![CDATA[<th&>]]> &eacute;t&unknown;</a:item><b/></a:root>"#,
        )
        .expect("parses");
        assert_eq!(root.name, "root");
        assert_eq!(root.attr("id"), Some("1&2"));
        assert_eq!(root.child_text("item"), Some("Café & <th&> ét&unknown;"));
        assert!(root.child("b").is_some());
    }

    #[test]
    fn truncated_and_mismatched_documents_are_errors() {
        for broken in ["<a><b>text", "<a></b>", "</a>", "<a", "   ", "<a><b></a></b>"] {
            assert!(parse(broken).is_err(), "{broken:?} must not parse");
        }
        let deep = "<a>".repeat(500);
        assert!(matches!(parse(&deep), Err(CatalogError::Malformed { .. })));
    }

    #[test]
    fn external_entities_are_not_expanded() {
        let root = parse(r#"<!DOCTYPE a [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><a>&xxe;</a>"#)
            .expect("parses");
        assert_eq!(root.text, "&xxe;");
    }

    #[test]
    fn latin1_documents_are_decoded_from_their_declaration() {
        let body = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><a>L'\xC9tranger</a>".to_vec();
        let response =
            HttpResponse { status: 200, headers: vec![], body, final_url: String::new() };
        assert_eq!(parse_xml("test", &response).expect("parses").text, "L'Étranger");
    }

    #[test]
    fn sru_and_marc_envelopes() {
        let root = parse(
            r#"<srw:searchRetrieveResponse xmlns:srw="http://www.loc.gov/zing/srw/">
              <srw:numberOfRecords>1</srw:numberOfRecords><srw:records><srw:record>
              <srw:recordData><mxc:record xmlns:mxc="info:lc/xmlns/marcxchange-v2" id="ark:/1/x">
              <mxc:leader>x</mxc:leader><mxc:controlfield tag="001">A</mxc:controlfield>
              <mxc:datafield tag="200" ind1="1" ind2=" "><mxc:subfield code="a">T</mxc:subfield>
              <mxc:subfield code="e"> </mxc:subfield></mxc:datafield></mxc:record>
              </srw:recordData></srw:record></srw:records></srw:searchRetrieveResponse>"#,
        )
        .expect("parses");
        let data = sru_records("test", &root).expect("is SRU");
        assert_eq!(data.len(), 1);
        let records = marc_records(data[0]);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id.as_deref(), Some("ark:/1/x"));
        assert_eq!(records[0].control("001"), Some("A"));
        assert_eq!(records[0].first("200", 'a'), Some("T"));
        assert_eq!(records[0].first("200", 'e'), None);
        assert_eq!(records[0].fields[0].ind1, '1');
    }

    #[test]
    fn sru_diagnostics_are_remote_errors() {
        let root = parse(
            r#"<searchRetrieveResponse><numberOfRecords>0</numberOfRecords><diagnostics>
               <diagnostic><uri>info:srw/diagnostic/1/10</uri><message>Query syntax error</message>
               </diagnostic></diagnostics></searchRetrieveResponse>"#,
        )
        .expect("parses");
        let error = sru_records("test", &root).expect_err("diagnostic");
        assert!(
            matches!(&error, CatalogError::Remote { message, .. } if message.contains("syntax"))
        );
        let other = parse("<explainResponse/>").expect("parses");
        assert!(matches!(sru_records("test", &other), Err(CatalogError::Malformed { .. })));
    }
}
