//! A fetched HTML page as an old e-reader browser would understand it: no script, forms, links.
//!
//! The page is parsed once (html5ever, with scripting **disabled** so `<noscript>` content is
//! real markup) and reduced to plain owned data: visible text, elements a test may look up by
//! id, forms with their controls and labels, and the facts the [lint](crate::lint) needs.
//! Parsing never fails and never panics; hostile markup at worst produces an empty page.

use std::ops::Range;

use ego_tree::iter::Edge;
use html5ever::driver::{self, ParseOpts};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use scraper::{Html, HtmlTreeSink, Node};
use url::Url;

use crate::css::{self, Rgb};

/// Most elements indexed for lookup on one page.
const MAX_INDEXED: usize = 20_000;
/// Most entries kept per kind of lint fact.
const MAX_FACTS: usize = 200;
/// Most text kept from one `<style>` block or attribute for the lint.
const MAX_STYLE_BYTES: usize = 64 * 1024;
/// Deepest element nesting the simulator builds a DOM for. Blink and modern WebKit cap the
/// parser's tree depth at 512; beyond it this simulator shows an empty page rather than spend
/// quadratic time in the HTML tree builder.
pub const MAX_NESTING: usize = 512;
/// Most attributes on one tag before the markup is treated as pathological.
pub const MAX_ATTRIBUTES: usize = 512;

/// Parses HTML the way a browser without JavaScript does.
pub(crate) fn parse_html(text: &str) -> Html {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts { scripting_enabled: false, ..Default::default() },
        ..Default::default()
    };
    driver::parse_document(HtmlTreeSink::new(Html::new_document()), opts).one(text)
}

/// Elements that never contain other elements, or whose end tag is optional (the tree builder
/// closes them implicitly, so counting them would overstate the depth of ordinary pages).
const NOT_NESTING: [&str; 33] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "p", "li", "dt", "dd", "tr", "td", "th", "thead", "tbody", "tfoot", "option",
    "optgroup", "colgroup", "rb", "rt", "rp", "html", "head", "body",
];

/// Elements whose content is text, not markup.
const RAW_TEXT: [&str; 7] = ["script", "style", "textarea", "title", "xmp", "noembed", "noframes"];

/// What a linear pre-scan of the markup found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shape {
    /// Estimated deepest nesting of elements.
    pub(crate) depth: usize,
    /// Largest number of attributes on a single tag (estimated).
    pub(crate) attributes: usize,
}

/// Estimates nesting depth and attribute counts without building a tree.
///
/// html5ever's tree builder is quadratic in nesting depth and in attributes per tag; a 1 MiB
/// page of `<div><div><div>…` would keep it busy for minutes. This scan is linear and only has
/// to be right about *pathological* input, so it is deliberately simple: it ignores comments
/// and raw-text elements, counts start tags of elements that really nest and subtracts their
/// end tags.
pub(crate) fn scan_shape(html: &str) -> Shape {
    let bytes = html.as_bytes();
    let mut shape = Shape { depth: 0, attributes: 0 };
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"<!--") {
            i = find_from(bytes, i + 4, b"-->").map_or(bytes.len(), |end| end + 3);
            continue;
        }
        let closing = bytes.get(i + 1) == Some(&b'/');
        let name_start = if closing { i + 2 } else { i + 1 };
        if !bytes.get(name_start).is_some_and(u8::is_ascii_alphabetic) {
            i += 1;
            continue;
        }
        let mut name_end = name_start;
        while name_end < bytes.len()
            && !bytes[name_end].is_ascii_whitespace()
            && !matches!(bytes[name_end], b'>' | b'/')
            && name_end - name_start < 32
        {
            name_end += 1;
        }
        let name = String::from_utf8_lossy(&bytes[name_start..name_end]).to_ascii_lowercase();
        // Walk to the end of the tag, honouring quoted attribute values.
        let mut j = name_end;
        let mut quote: Option<u8> = None;
        let mut attributes = 0usize;
        let mut in_space = false;
        while j < bytes.len() {
            let b = bytes[j];
            match quote {
                Some(q) if b == q => quote = None,
                Some(_) => {}
                None if b == b'"' || b == b'\'' => quote = Some(b),
                None if b == b'>' => break,
                None => {
                    let space = b.is_ascii_whitespace();
                    if in_space && !space && b != b'/' {
                        attributes += 1;
                    }
                    in_space = space;
                }
            }
            j += 1;
        }
        shape.attributes = shape.attributes.max(attributes);
        let self_closing = j > 0 && bytes.get(j.wrapping_sub(1)) == Some(&b'/');
        i = (j + 1).min(bytes.len());
        let name = name.as_str();
        if NOT_NESTING.contains(&name) {
            continue;
        }
        if closing {
            depth = depth.saturating_sub(1);
        } else if RAW_TEXT.contains(&name) {
            // Skip the raw text up to the matching end tag.
            let mut needle = Vec::with_capacity(name.len() + 2);
            needle.extend_from_slice(b"</");
            needle.extend_from_slice(name.as_bytes());
            i = find_ci(bytes, i, &needle).unwrap_or(bytes.len());
        } else if !self_closing {
            depth += 1;
            shape.depth = shape.depth.max(depth);
        }
    }
    shape
}

fn find_from(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    haystack.get(from..)?.windows(needle.len()).position(|w| w == needle).map(|p| p + from)
}

fn find_ci(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|w| w.eq_ignore_ascii_case(needle))
        .map(|p| p + from)
}

/// `method` of a form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormMethod {
    /// `GET` (also the default).
    Get,
    /// `POST`.
    Post,
    /// Anything else (`dialog`, typos). Browsers fall back to GET; the lint flags it.
    Other(String),
}

/// Kind of a form control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Single-line text of any flavour (`text`, `email`, `number`, unknown types…).
    Text,
    /// `type=password`.
    Password,
    /// `type=hidden`.
    Hidden,
    /// `type=checkbox`.
    Checkbox,
    /// `type=radio`.
    Radio,
    /// `<input type=submit>` or `<button>` / `<button type=submit>`.
    Submit,
    /// `<input type=image>`.
    Image,
    /// `type=reset`.
    Reset,
    /// `type=button`: does nothing without script.
    Button,
    /// `type=file`.
    File,
    /// `<select>`.
    Select,
    /// `<textarea>`.
    Textarea,
}

impl FieldKind {
    /// Whether a person can type text into it.
    pub fn is_typeable(self) -> bool {
        matches!(self, Self::Text | Self::Password | Self::Textarea)
    }

    /// Whether activating it submits the form.
    pub fn is_submit(self) -> bool {
        matches!(self, Self::Submit | Self::Image)
    }
}

/// An `<option>` of a `<select>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    /// Submitted value (`value` attribute, else the text).
    pub value: String,
    /// Visible text.
    pub text: String,
    /// `selected` attribute.
    pub selected: bool,
}

/// A form control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Kind.
    pub kind: FieldKind,
    /// Raw `type` attribute, lower-cased (empty for `<select>` / `<textarea>` / untyped).
    pub input_type: String,
    /// `id`.
    pub id: Option<String>,
    /// `name`.
    pub name: Option<String>,
    /// Initial value (`value` attribute, or the text of a `<textarea>`).
    pub value: String,
    /// The label a person reads next to the control: `<label for>`, a wrapping `<label>`,
    /// else `aria-label` / `title`.
    pub label: Option<String>,
    /// Visible text of a `<button>`, `value` of an `<input type=submit>`, `alt` of an image.
    pub text: String,
    /// Initially checked (checkbox / radio).
    pub checked: bool,
    /// `disabled`.
    pub disabled: bool,
    /// `readonly`.
    pub readonly: bool,
    /// `maxlength`, in characters.
    pub maxlength: Option<usize>,
    /// Options of a `<select>`.
    pub options: Vec<SelectOption>,
    /// Whether the control has an `onclick` handler.
    pub onclick: bool,
    /// Whether the control has a `placeholder`.
    pub placeholder: bool,
}

/// A `<form>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    /// `id`.
    pub id: Option<String>,
    /// `name`.
    pub name: Option<String>,
    /// `method`.
    pub method: FormMethod,
    /// Raw `action` attribute (`None` when absent).
    pub action: Option<String>,
    /// `enctype`, lower-cased; defaults to `application/x-www-form-urlencoded`.
    pub enctype: String,
    /// Whether the form has an `onsubmit` handler.
    pub onsubmit: bool,
    /// Controls, in document order.
    pub fields: Vec<Field>,
}

impl Form {
    /// The first control with this `name`.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name.as_deref() == Some(name))
    }

    /// Index of the default submit button (the first enabled one).
    pub fn default_submit(&self) -> Option<usize> {
        self.fields.iter().position(|f| f.kind.is_submit() && !f.disabled)
    }

    /// Short description for messages, e.g. `<form id="code-form">`.
    pub fn describe(&self) -> String {
        match (&self.id, &self.name) {
            (Some(id), _) => format!("<form id=\"{id}\">"),
            (None, Some(name)) => format!("<form name=\"{name}\">"),
            (None, None) => "<form>".to_owned(),
        }
    }
}

/// `<meta http-equiv="refresh">`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaRefresh {
    /// Delay in whole seconds.
    pub delay_secs: u32,
    /// Target (the page itself when the tag names no URL).
    pub url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Indexed {
    pub(crate) tag: String,
    pub(crate) id: Option<String>,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) text: Range<usize>,
    /// `(form index, field index)` when this element is a control inside a form.
    pub(crate) field: Option<(usize, usize)>,
    /// Whether the element sits inside a `<form>` at all.
    pub(crate) in_form: bool,
}

impl Indexed {
    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    pub(crate) fn describe(&self) -> String {
        describe(&self.tag, self.id.as_deref(), self.attr("name"))
    }
}

fn describe(tag: &str, id: Option<&str>, name: Option<&str>) -> String {
    let clip = |s: &str| s.chars().take(40).collect::<String>();
    match (id, name) {
        (Some(id), _) => format!("<{tag} id=\"{}\">", clip(id)),
        (None, Some(name)) => format!("<{tag} name=\"{}\">", clip(name)),
        (None, None) => format!("<{tag}>"),
    }
}

/// A `<script>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptFact {
    pub(crate) src: Option<String>,
    pub(crate) kind: String,
    pub(crate) inline_bytes: usize,
}

/// Text whose inline-declared colours contrast poorly.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContrastFact {
    pub(crate) element: String,
    pub(crate) fg: Rgb,
    pub(crate) bg: Rgb,
    pub(crate) ratio: f64,
}

/// Everything the lint needs, collected during the single parse pass.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct LintFacts {
    pub(crate) scripts: Vec<ScriptFact>,
    /// `(element description, raw URL)` of sub-resources.
    pub(crate) resources: Vec<(String, String)>,
    /// Elements with inline `on*` handlers.
    pub(crate) handlers: Vec<String>,
    /// Elements that only act through script, with the reason.
    pub(crate) needs_js: Vec<(String, String)>,
    /// Controls outside any `<form>`.
    pub(crate) orphan_controls: Vec<String>,
    /// Controls using the HTML5 `form=` attribute.
    pub(crate) form_attr_controls: Vec<String>,
    /// Inputs of types old browsers render as plain text.
    pub(crate) modern_inputs: Vec<(String, String)>,
    /// Links opening a new window.
    pub(crate) blank_targets: Vec<String>,
    pub(crate) contrast: Vec<ContrastFact>,
    /// `(element description, declared width in px)`.
    pub(crate) fixed_widths: Vec<(String, u32)>,
    /// Concatenated CSS (style blocks and attributes), lower-cased, bounded.
    pub(crate) css: String,
    pub(crate) noscript_text: Vec<String>,
    pub(crate) has_viewport: bool,
    pub(crate) has_doctype: bool,
    pub(crate) meta_charset: bool,
    pub(crate) truncated_index: bool,
    /// Set when the markup is too deeply nested (or otherwise pathological) to build a DOM:
    /// `(estimated depth, most attributes on one tag)`.
    pub(crate) pathological: Option<(usize, usize)>,
}

/// A page displayed by a simulated browser.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    url: Url,
    base: Url,
    status: u16,
    headers: Vec<(String, String)>,
    html: String,
    weight: usize,
    non_ascii: bool,
    title: Option<String>,
    lang: Option<String>,
    text: String,
    pub(crate) nodes: Vec<Indexed>,
    forms: Vec<Form>,
    meta_refresh: Option<MetaRefresh>,
    pub(crate) facts: LintFacts,
}

/// A looked-up element.
#[derive(Debug, Clone, Copy)]
pub struct Element<'a> {
    page: &'a Page,
    node: &'a Indexed,
}

impl<'a> Element<'a> {
    /// Tag name, lower-case.
    pub fn tag(&self) -> &'a str {
        &self.node.tag
    }

    /// `id` attribute.
    pub fn id(&self) -> Option<&'a str> {
        self.node.id.as_deref()
    }

    /// An attribute value.
    pub fn attr(&self, name: &str) -> Option<&'a str> {
        self.node.attr(name)
    }

    /// Visible text inside the element, whitespace-normalised.
    pub fn text(&self) -> &'a str {
        self.page.text_in(&self.node.text)
    }
}

impl Page {
    /// Builds a page from an HTTP response. `headers` use lower-case names.
    pub fn from_response(url: Url, status: u16, headers: Vec<(String, String)>, body: &[u8]) -> Self {
        let content_type = headers
            .iter()
            .find(|(name, _)| name == "content-type")
            .map(|(_, value)| value.to_ascii_lowercase())
            .unwrap_or_default();
        let html = decode(body, &content_type);
        let mut builder = Builder::new(&url);
        let shape = scan_shape(&html);
        if shape.depth > MAX_NESTING || shape.attributes > MAX_ATTRIBUTES {
            // Shown as an empty page: see `MAX_NESTING`.
            builder.facts.pathological = Some((shape.depth, shape.attributes));
        } else {
            builder.run(&parse_html(&html));
        }
        let mut page = builder.finish(url, status, headers, html, body.len());
        page.non_ascii = !body.is_ascii();
        page
    }

    /// Builds a page from markup alone (status 200, `text/html; charset=utf-8`), for linting
    /// templates without a server. An unparsable `url` falls back to `http://localhost/`.
    pub fn from_html(url: &str, html: &str) -> Self {
        let url = Url::parse(url).unwrap_or_else(|_| fallback_url());
        Self::from_response(
            url,
            200,
            vec![("content-type".to_owned(), "text/html; charset=utf-8".to_owned())],
            html.as_bytes(),
        )
    }

    /// URL the page was fetched from (after redirects).
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Base URL for relative links (`<base href>` or the page URL).
    pub fn base_url(&self) -> &Url {
        &self.base
    }

    /// HTTP status the page came with.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// A response header (lower-case name).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    /// All values of a response header (lower-case name).
    pub fn header_values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> {
        self.headers.iter().filter(move |(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    /// The markup as decoded.
    pub fn html(&self) -> &str {
        &self.html
    }

    /// Size of the HTML document as transferred, in bytes.
    pub fn weight_bytes(&self) -> usize {
        self.weight
    }

    /// `<title>`.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// `<html lang>`.
    pub fn lang(&self) -> Option<&str> {
        self.lang.as_deref()
    }

    /// All visible text, whitespace-normalised. `<noscript>` content counts as visible.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the visible text contains `needle` (case-sensitive).
    pub fn contains_text(&self, needle: &str) -> bool {
        self.text.contains(needle)
    }

    /// The first element with this `id`.
    pub fn element(&self, id: &str) -> Option<Element<'_>> {
        self.nodes
            .iter()
            .find(|n| n.id.as_deref() == Some(id))
            .map(|node| Element { page: self, node })
    }

    /// Whether an element with this `id` exists.
    pub fn has(&self, id: &str) -> bool {
        self.element(id).is_some()
    }

    /// Visible text of the element with this `id`.
    pub fn text_of(&self, id: &str) -> Option<&str> {
        self.element(id).map(|e| e.text())
    }

    /// All forms.
    pub fn forms(&self) -> &[Form] {
        &self.forms
    }

    /// A form by `id`, then by `name`.
    pub fn form(&self, key: &str) -> Option<&Form> {
        self.form_index(key).map(|i| &self.forms[i])
    }

    pub(crate) fn form_index(&self, key: &str) -> Option<usize> {
        self.forms
            .iter()
            .position(|f| f.id.as_deref() == Some(key))
            .or_else(|| self.forms.iter().position(|f| f.name.as_deref() == Some(key)))
    }

    /// Every `<a href>` on the page.
    pub fn links(&self) -> impl Iterator<Item = Element<'_>> {
        self.nodes
            .iter()
            .filter(|n| n.tag == "a" && n.attr("href").is_some())
            .map(move |node| Element { page: self, node })
    }

    /// `<meta http-equiv="refresh">`, if any.
    pub fn meta_refresh(&self) -> Option<&MetaRefresh> {
        self.meta_refresh.as_ref()
    }

    /// Text of every element matching a CSS selector (re-parses the markup; for tests).
    pub fn select_text(&self, selector: &str) -> Result<Vec<String>, crate::error::BrowserError> {
        let parsed = scraper::Selector::parse(selector)
            .map_err(|_| crate::error::BrowserError::BadSelector { selector: selector.to_owned() })?;
        if self.facts.pathological.is_some() {
            return Ok(Vec::new());
        }
        let html = parse_html(&self.html);
        Ok(html
            .select(&parsed)
            .take(MAX_INDEXED)
            .map(|el| el.text().flat_map(str::split_whitespace).collect::<Vec<_>>().join(" "))
            .collect())
    }

    /// Replaces the status and headers (used when the page is synthesised around a
    /// non-HTML response).
    pub(crate) fn set_response(&mut self, status: u16, headers: Vec<(String, String)>) {
        self.status = status;
        self.headers = headers;
    }

    pub(crate) fn text_in(&self, range: &Range<usize>) -> &str {
        self.text.get(range.clone()).unwrap_or("").trim()
    }

    pub(crate) fn has_non_ascii(&self) -> bool {
        self.non_ascii
    }

    pub(crate) fn charset_in_header(&self) -> bool {
        self.header("content-type").is_some_and(|ct| ct.to_ascii_lowercase().contains("charset="))
    }
}

fn fallback_url() -> Url {
    // A constant, well-formed URL: parsing it cannot fail.
    #[allow(clippy::expect_used)]
    Url::parse("http://localhost/").expect("constant URL parses")
}

/// Decodes a body as UTF-8 unless Latin-1 / Windows-1252 is declared.
fn decode(body: &[u8], content_type: &str) -> String {
    let prescan = String::from_utf8_lossy(&body[..body.len().min(1024)]).to_ascii_lowercase();
    let charset_of = |text: &str| {
        text.split("charset=").nth(1).map(|rest| {
            rest.trim_start_matches(['"', '\'', ' '])
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect::<String>()
        })
    };
    let charset = charset_of(content_type).or_else(|| charset_of(&prescan)).unwrap_or_default();
    match charset.as_str() {
        // Windows-1252's 0x80–0x9F block is approximated by Latin-1 (C1 controls).
        "iso-8859-1" | "latin1" | "windows-1252" | "cp1252" | "iso8859-1" => {
            body.iter().map(|b| char::from(*b)).collect()
        }
        _ => String::from_utf8_lossy(body).trim_start_matches('\u{feff}').to_owned(),
    }
}

const INLINE_TAGS: [&str; 28] = [
    "a", "abbr", "b", "bdo", "big", "cite", "code", "dfn", "em", "font", "i", "kbd", "label",
    "mark", "q", "s", "samp", "small", "span", "strike", "strong", "sub", "sup", "time", "tt",
    "u", "var", "wbr",
];

const MODERN_INPUT_TYPES: [&str; 8] =
    ["date", "datetime-local", "month", "week", "time", "color", "range", "datetime"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Closes {
    Nothing,
    Form,
    Label,
    Select,
    Option,
    Textarea,
    Title,
    Noscript,
    Style,
    Script,
}

struct Frame {
    node: Option<usize>,
    hides: bool,
    closes: Closes,
    block: bool,
    fg: Rgb,
    bg: Rgb,
    /// Colours come from an inline declaration somewhere up the tree.
    styled: bool,
    description: String,
}

struct LabelRec {
    for_id: Option<String>,
    text: Range<usize>,
}

struct Builder {
    page_url: Url,
    base: Option<Url>,
    text: String,
    pending_space: bool,
    nodes: Vec<Indexed>,
    forms: Vec<Form>,
    labels: Vec<LabelRec>,
    /// `(form, field, label)` — wrapping label of a control.
    wrapped: Vec<(usize, usize, usize)>,
    /// `(form, field, node)` — buttons whose text is their content.
    button_nodes: Vec<(usize, usize, usize)>,
    stack: Vec<Frame>,
    hidden_depth: usize,
    cur_form: Option<usize>,
    cur_label: Option<usize>,
    cur_select: Option<(usize, usize)>,
    /// `Some(value attribute)` while inside an `<option>`.
    cur_option: Option<Option<String>>,
    cur_textarea: Option<(usize, usize)>,
    title: Option<String>,
    in_title: bool,
    lang: Option<String>,
    meta_refresh: Option<MetaRefresh>,
    noscript_start: Option<usize>,
    raw_text: String,
    facts: LintFacts,
}

fn push_fact<T>(list: &mut Vec<T>, item: T) {
    if list.len() < MAX_FACTS {
        list.push(item);
    }
}

impl Builder {
    fn new(page_url: &Url) -> Self {
        Self {
            page_url: page_url.clone(),
            base: None,
            text: String::new(),
            pending_space: false,
            nodes: Vec::new(),
            forms: Vec::new(),
            labels: Vec::new(),
            wrapped: Vec::new(),
            button_nodes: Vec::new(),
            stack: Vec::new(),
            hidden_depth: 0,
            cur_form: None,
            cur_label: None,
            cur_select: None,
            cur_option: None,
            cur_textarea: None,
            title: None,
            in_title: false,
            lang: None,
            meta_refresh: None,
            noscript_start: None,
            raw_text: String::new(),
            facts: LintFacts::default(),
        }
    }

    fn base_url(&self) -> &Url {
        self.base.as_ref().unwrap_or(&self.page_url)
    }

    fn run(&mut self, html: &Html) {
        for edge in html.tree.root().traverse() {
            match edge {
                Edge::Open(node) => match node.value() {
                    Node::Element(element) => self.open(element),
                    Node::Text(text) => self.text_node(text),
                    Node::Doctype(_) => self.facts.has_doctype = true,
                    _ => {}
                },
                Edge::Close(node) => {
                    if node.value().is_element() {
                        self.close();
                    }
                }
            }
        }
    }

    fn append_visible(&mut self, s: &str) {
        for ch in s.chars() {
            if ch.is_whitespace() {
                self.pending_space = true;
            } else {
                if self.pending_space && !self.text.is_empty() {
                    self.text.push(' ');
                }
                self.pending_space = false;
                self.text.push(ch);
            }
        }
    }

    fn text_node(&mut self, text: &str) {
        if self.in_title {
            let title = self.title.get_or_insert_with(String::new);
            if title.len() < 1024 {
                title.push_str(text);
            }
        }
        match self.stack.last().map(|f| f.closes) {
            Some(Closes::Style) => {
                self.add_css(text);
                return;
            }
            Some(Closes::Script) => {
                if let Some(script) = self.facts.scripts.last_mut() {
                    script.inline_bytes += text.len();
                }
                return;
            }
            _ => {}
        }
        if let Some((form, field)) = self.cur_textarea {
            let value = &mut self.forms[form].fields[field].value;
            if value.len() < MAX_STYLE_BYTES {
                value.push_str(text);
            }
        }
        if self.cur_option.is_some() && self.raw_text.len() < 4096 {
            self.raw_text.push_str(text);
        }
        if self.hidden_depth > 0 {
            return;
        }
        if !text.trim().is_empty() {
            self.check_contrast();
        }
        self.append_visible(text);
    }

    fn check_contrast(&mut self) {
        let Some(frame) = self.stack.last() else { return };
        if !frame.styled {
            return;
        }
        let ratio = css::contrast_ratio(frame.fg, frame.bg);
        if ratio >= 7.0 {
            return;
        }
        let fact =
            ContrastFact { element: frame.description.clone(), fg: frame.fg, bg: frame.bg, ratio };
        if self.facts.contrast.last() != Some(&fact) {
            push_fact(&mut self.facts.contrast, fact);
        }
    }

    fn add_css(&mut self, css_text: &str) {
        let room = MAX_STYLE_BYTES.saturating_sub(self.facts.css.len());
        if room == 0 {
            return;
        }
        let mut end = css_text.len().min(room);
        while !css_text.is_char_boundary(end) {
            end -= 1;
        }
        self.facts.css.push_str(&css_text[..end].to_ascii_lowercase());
        self.facts.css.push('\n');
        for target in css::urls(&css_text[..end]) {
            push_fact(&mut self.facts.resources, ("css url()".to_owned(), target));
        }
    }

    fn open(&mut self, element: &scraper::node::Element) {
        let tag = element.name().to_ascii_lowercase();
        let attr = |name: &str| element.attr(name);
        let description = describe(&tag, element.id(), attr("name"));
        let block = !INLINE_TAGS.contains(&tag.as_str());
        if block {
            self.pending_space = true;
        }

        let parent = self.stack.last();
        let (mut fg, mut bg, mut styled) = parent
            .map(|f| (f.fg, f.bg, f.styled))
            .unwrap_or((css::BLACK, css::WHITE, false));
        let style = attr("style").unwrap_or("");
        let declarations = css::declarations(style);
        for (prop, value) in &declarations {
            match prop.as_str() {
                "color" => {
                    if let Some(c) = css::parse_color(value) {
                        fg = c;
                        styled = true;
                    }
                }
                "background" | "background-color" => {
                    if let Some(c) = css::background_color(value) {
                        bg = c;
                        styled = true;
                    }
                }
                "width" | "min-width" => {
                    if let Some(px) = css::px(value).filter(|px| *px > 320) {
                        push_fact(&mut self.facts.fixed_widths, (description.clone(), px));
                    }
                }
                _ => {}
            }
        }
        if !style.is_empty() {
            self.add_css(style);
        }
        if let Some(c) = attr("bgcolor").and_then(css::parse_color) {
            bg = c;
            styled = true;
        }
        if let Some(c) = attr("text").filter(|_| tag == "body").and_then(css::parse_color) {
            fg = c;
            styled = true;
        }
        if let Some(c) = attr("color").filter(|_| tag == "font").and_then(css::parse_color) {
            fg = c;
            styled = true;
        }
        if matches!(tag.as_str(), "table" | "img" | "td" | "th" | "div" | "iframe") {
            if let Some(px) = attr("width").and_then(css::px).filter(|px| *px > 320) {
                push_fact(&mut self.facts.fixed_widths, (description.clone(), px));
            }
        }

        // The content of <select> and <textarea> is captured as control state, not page text.
        let hides = matches!(tag.as_str(), "script" | "style" | "template" | "head" | "select" | "textarea")
            || attr("hidden").is_some()
            || css::hides(&declarations);
        if hides {
            self.hidden_depth += 1;
        }

        let has_handler = element.attrs().any(|(name, _)| name.starts_with("on"));
        if has_handler {
            push_fact(&mut self.facts.handlers, description.clone());
        }
        let onclick = attr("onclick").is_some();

        let mut closes = Closes::Nothing;
        let mut field: Option<(usize, usize)> = None;
        match tag.as_str() {
            "html" => {
                if self.lang.is_none() {
                    self.lang = attr("lang")
                        .or_else(|| attr("xml:lang"))
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .map(str::to_owned);
                }
            }
            "base" => {
                if self.base.is_none() {
                    self.base = attr("href")
                        .and_then(|href| self.page_url.join(href.trim()).ok())
                        .filter(|u| matches!(u.scheme(), "http" | "https"));
                }
            }
            "title" => {
                if self.title.is_none() {
                    self.in_title = true;
                    closes = Closes::Title;
                }
            }
            "meta" => self.meta(element),
            "script" => {
                closes = Closes::Script;
                let src = attr("src").map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned);
                if let Some(src) = &src {
                    push_fact(&mut self.facts.resources, (description.clone(), src.clone()));
                }
                push_fact(
                    &mut self.facts.scripts,
                    ScriptFact {
                        src,
                        kind: attr("type").unwrap_or("").trim().to_ascii_lowercase(),
                        inline_bytes: 0,
                    },
                );
            }
            "style" => closes = Closes::Style,
            "noscript" => {
                if self.noscript_start.is_none() {
                    self.noscript_start = Some(self.text.len());
                    closes = Closes::Noscript;
                }
            }
            "link" => {
                let rel = attr("rel").unwrap_or("").to_ascii_lowercase();
                let loads = rel
                    .split_whitespace()
                    .any(|r| matches!(r, "stylesheet" | "icon" | "preload" | "apple-touch-icon"));
                if let Some(href) = attr("href").filter(|_| loads) {
                    push_fact(&mut self.facts.resources, (description.clone(), href.to_owned()));
                }
            }
            "img" | "iframe" | "embed" | "video" | "audio" | "source" | "track" | "frame" => {
                if let Some(src) = attr("src").filter(|s| !s.trim().is_empty()) {
                    push_fact(&mut self.facts.resources, (description.clone(), src.to_owned()));
                }
            }
            "object" => {
                if let Some(data) = attr("data").filter(|s| !s.trim().is_empty()) {
                    push_fact(&mut self.facts.resources, (description.clone(), data.to_owned()));
                }
            }
            "form" => {
                if self.cur_form.is_none() {
                    let method = match attr("method").unwrap_or("get").trim().to_ascii_lowercase().as_str() {
                        "get" | "" => FormMethod::Get,
                        "post" => FormMethod::Post,
                        other => FormMethod::Other(other.to_owned()),
                    };
                    self.forms.push(Form {
                        id: element.id().map(str::to_owned),
                        name: attr("name").map(str::to_owned),
                        method,
                        action: attr("action").map(str::to_owned),
                        enctype: attr("enctype")
                            .map(|e| e.trim().to_ascii_lowercase())
                            .filter(|e| !e.is_empty())
                            .unwrap_or_else(|| "application/x-www-form-urlencoded".to_owned()),
                        onsubmit: attr("onsubmit").is_some(),
                        fields: Vec::new(),
                    });
                    self.cur_form = Some(self.forms.len() - 1);
                    closes = Closes::Form;
                }
            }
            "label" => {
                if self.cur_label.is_none() {
                    self.labels.push(LabelRec {
                        for_id: attr("for").map(str::to_owned),
                        text: self.text.len()..self.text.len(),
                    });
                    self.cur_label = Some(self.labels.len() - 1);
                    closes = Closes::Label;
                }
            }
            "a" => {
                let href = attr("href").map(str::trim);
                let lower = href.map(str::to_ascii_lowercase);
                let reason = match lower.as_deref() {
                    Some(h) if h.starts_with("javascript:") => Some("its href is a javascript: URL"),
                    Some(h) if (h.is_empty() || h.starts_with('#')) && onclick => {
                        Some("it is a placeholder link driven by onclick")
                    }
                    None if onclick => Some("it has onclick but no href"),
                    _ => None,
                };
                if let Some(reason) = reason {
                    push_fact(&mut self.facts.needs_js, (description.clone(), reason.to_owned()));
                }
                if attr("target").is_some_and(|t| t.eq_ignore_ascii_case("_blank")) {
                    push_fact(&mut self.facts.blank_targets, description.clone());
                }
            }
            "input" | "button" | "select" | "textarea" => {
                let (kind, input_type) = control_kind(&tag, attr("type"));
                if tag == "input" && MODERN_INPUT_TYPES.contains(&input_type.as_str()) {
                    push_fact(&mut self.facts.modern_inputs, (description.clone(), input_type.clone()));
                }
                if tag == "input" && input_type == "image" {
                    if let Some(src) = attr("src") {
                        push_fact(&mut self.facts.resources, (description.clone(), src.to_owned()));
                    }
                }
                if attr("form").is_some() {
                    push_fact(&mut self.facts.form_attr_controls, description.clone());
                }
                let control = Field {
                    kind,
                    input_type,
                    id: element.id().map(str::to_owned),
                    name: attr("name").map(str::to_owned).filter(|n| !n.is_empty()),
                    value: attr("value").unwrap_or("").to_owned(),
                    label: attr("aria-label")
                        .or_else(|| attr("title"))
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .map(str::to_owned),
                    text: match kind {
                        FieldKind::Image => attr("alt").unwrap_or("").to_owned(),
                        _ if tag == "input" => attr("value").unwrap_or("").to_owned(),
                        _ => String::new(),
                    },
                    checked: attr("checked").is_some(),
                    disabled: attr("disabled").is_some(),
                    readonly: attr("readonly").is_some(),
                    maxlength: attr("maxlength").and_then(|m| m.trim().parse::<usize>().ok()),
                    options: Vec::new(),
                    onclick,
                    placeholder: attr("placeholder").is_some_and(|p| !p.trim().is_empty()),
                };
                match self.cur_form {
                    Some(form) if self.forms[form].fields.len() < MAX_INDEXED => {
                        if kind == FieldKind::Button {
                            push_fact(
                                &mut self.facts.needs_js,
                                (description.clone(), "type=button does nothing without script".to_owned()),
                            );
                        }
                        self.forms[form].fields.push(control);
                        let index = self.forms[form].fields.len() - 1;
                        field = Some((form, index));
                        if let Some(label) = self.cur_label {
                            self.wrapped.push((form, index, label));
                        }
                        match tag.as_str() {
                            "select" if self.cur_select.is_none() => {
                                self.cur_select = Some((form, index));
                                closes = Closes::Select;
                            }
                            "textarea" if self.cur_textarea.is_none() => {
                                self.cur_textarea = Some((form, index));
                                closes = Closes::Textarea;
                            }
                            _ => {}
                        }
                    }
                    Some(_) => {}
                    None => {
                        if kind != FieldKind::Hidden {
                            push_fact(&mut self.facts.orphan_controls, description.clone());
                        }
                    }
                }
            }
            "option" => {
                if let Some((form, index)) = self.cur_select {
                    if self.cur_option.is_none() && self.forms[form].fields[index].options.len() < 1000 {
                        self.raw_text.clear();
                        self.cur_option = Some(attr("value").map(str::to_owned));
                        self.forms[form].fields[index].options.push(SelectOption {
                            value: String::new(),
                            text: String::new(),
                            selected: attr("selected").is_some(),
                        });
                        closes = Closes::Option;
                    }
                }
            }
            _ => {
                if onclick {
                    push_fact(
                        &mut self.facts.needs_js,
                        (description.clone(), format!("a <{tag}> made clickable with onclick")),
                    );
                }
            }
        }

        let indexed = element.id().is_some()
            || matches!(tag.as_str(), "a" | "button" | "input" | "select" | "textarea" | "form" | "label");
        let node = if indexed && self.nodes.len() < MAX_INDEXED {
            self.nodes.push(Indexed {
                tag: tag.clone(),
                id: element.id().map(str::to_owned),
                attrs: element
                    .attrs()
                    .take(64)
                    .map(|(name, value)| (name.to_ascii_lowercase(), value.to_owned()))
                    .collect(),
                text: self.text.len()..self.text.len(),
                field,
                in_form: self.cur_form.is_some(),
            });
            Some(self.nodes.len() - 1)
        } else {
            if indexed {
                self.facts.truncated_index = true;
            }
            None
        };
        if let (Some(node), Some((form, index)), "button") = (node, field, tag.as_str()) {
            self.button_nodes.push((form, index, node));
        }

        self.stack.push(Frame { node, hides, closes, block, fg, bg, styled, description });
    }

    fn meta(&mut self, element: &scraper::node::Element) {
        let attr = |name: &str| element.attr(name);
        if attr("charset").is_some() {
            self.facts.meta_charset = true;
        }
        if attr("name").is_some_and(|n| n.eq_ignore_ascii_case("viewport")) {
            self.facts.has_viewport = true;
        }
        let Some(equiv) = attr("http-equiv").map(str::to_ascii_lowercase) else { return };
        let content = attr("content").unwrap_or("");
        match equiv.as_str() {
            "content-type" if content.to_ascii_lowercase().contains("charset=") => {
                self.facts.meta_charset = true;
            }
            "refresh" if self.meta_refresh.is_none() => {
                self.meta_refresh = parse_meta_refresh(content, self.base_url());
            }
            _ => {}
        }
    }

    fn close(&mut self) {
        let Some(frame) = self.stack.pop() else { return };
        if let Some(node) = frame.node {
            self.nodes[node].text.end = self.text.len();
        }
        if frame.hides {
            self.hidden_depth = self.hidden_depth.saturating_sub(1);
        }
        if frame.block {
            self.pending_space = true;
        }
        match frame.closes {
            Closes::Nothing | Closes::Style | Closes::Script => {}
            Closes::Form => self.cur_form = None,
            Closes::Label => {
                if let Some(label) = self.cur_label.take() {
                    self.labels[label].text.end = self.text.len();
                }
            }
            Closes::Select => self.cur_select = None,
            Closes::Textarea => self.cur_textarea = None,
            Closes::Title => self.in_title = false,
            Closes::Option => {
                if let (Some(value), Some((form, index))) = (self.cur_option.take(), self.cur_select) {
                    let text = self.raw_text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if let Some(option) = self.forms[form].fields[index].options.last_mut() {
                        option.value = value.unwrap_or_else(|| text.clone());
                        option.text = text;
                    }
                    self.raw_text.clear();
                }
            }
            Closes::Noscript => {
                if let Some(start) = self.noscript_start.take() {
                    let text: String =
                        self.text.get(start..).unwrap_or("").trim().chars().take(500).collect();
                    if !text.is_empty() {
                        push_fact(&mut self.facts.noscript_text, text);
                    }
                }
            }
        }
    }

    fn finish(
        mut self,
        url: Url,
        status: u16,
        headers: Vec<(String, String)>,
        html: String,
        weight: usize,
    ) -> Page {
        let slice = |range: &Range<usize>| self.text.get(range.clone()).unwrap_or("").trim().to_owned();
        // Button text is the button's content.
        let button_texts: Vec<(usize, usize, String)> = self
            .button_nodes
            .iter()
            .map(|(form, index, node)| (*form, *index, slice(&self.nodes[*node].text)))
            .collect();
        for (form, index, text) in button_texts {
            self.forms[form].fields[index].text = text;
        }
        // Labels: wrapping label first, then an explicit `for` (which wins).
        let wrapped: Vec<(usize, usize, String)> = self
            .wrapped
            .iter()
            .map(|(form, index, label)| (*form, *index, slice(&self.labels[*label].text)))
            .collect();
        for (form, index, text) in wrapped {
            if !text.is_empty() {
                self.forms[form].fields[index].label = Some(text);
            }
        }
        let mut explicit: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for label in &self.labels {
            if let Some(id) = &label.for_id {
                let text = slice(&label.text);
                if !text.is_empty() {
                    explicit.entry(id.clone()).or_insert(text);
                }
            }
        }
        for form in &mut self.forms {
            for field in &mut form.fields {
                if let Some(text) = field.id.as_ref().and_then(|id| explicit.get(id)) {
                    field.label = Some(text.clone());
                }
            }
        }
        let title = self
            .title
            .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|t| !t.is_empty());
        let base = self.base.unwrap_or_else(|| url.clone());
        Page {
            url,
            base,
            status,
            headers,
            html,
            weight,
            non_ascii: false,
            title,
            lang: self.lang,
            text: self.text,
            nodes: self.nodes,
            forms: self.forms,
            meta_refresh: self.meta_refresh,
            facts: self.facts,
        }
    }
}

fn control_kind(tag: &str, type_attr: Option<&str>) -> (FieldKind, String) {
    let input_type = type_attr.unwrap_or("").trim().to_ascii_lowercase();
    let kind = match tag {
        "select" => return (FieldKind::Select, String::new()),
        "textarea" => return (FieldKind::Textarea, String::new()),
        "button" => match input_type.as_str() {
            "button" => FieldKind::Button,
            "reset" => FieldKind::Reset,
            // Missing or invalid type: submit.
            _ => FieldKind::Submit,
        },
        _ => match input_type.as_str() {
            "password" => FieldKind::Password,
            "hidden" => FieldKind::Hidden,
            "checkbox" => FieldKind::Checkbox,
            "radio" => FieldKind::Radio,
            "submit" => FieldKind::Submit,
            "image" => FieldKind::Image,
            "reset" => FieldKind::Reset,
            "button" => FieldKind::Button,
            "file" => FieldKind::File,
            // text, search, tel, url, email, number and anything unknown render as text.
            _ => FieldKind::Text,
        },
    };
    (kind, input_type)
}

fn parse_meta_refresh(content: &str, base: &Url) -> Option<MetaRefresh> {
    let (delay, rest) = match content.find([';', ',']) {
        Some(at) => (&content[..at], &content[at + 1..]),
        None => (content, ""),
    };
    let delay = delay.trim();
    let whole = delay.split('.').next().unwrap_or("");
    let delay_secs = if whole.is_empty() && delay.starts_with('.') {
        0
    } else {
        whole.parse::<u32>().ok()?
    };
    let rest = rest.trim();
    let target = if rest.len() >= 4 && rest.as_bytes()[..3].eq_ignore_ascii_case(b"url") {
        rest[3..].trim_start().strip_prefix('=').map(str::trim).unwrap_or(rest)
    } else {
        rest
    };
    let target = target.trim_matches(['"', '\'']).trim();
    let url = if target.is_empty() { base.clone() } else { base.join(target).ok()? };
    matches!(url.scheme(), "http" | "https").then_some(MetaRefresh { delay_secs, url })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const HOME: &str = r#"<!DOCTYPE html>
<html lang="fr"><head><meta charset="utf-8"><title> eBook   Cadeau </title>
<style>body{font-family:serif}</style></head>
<body>
<h1 id="headline">Votre livre vous attend</h1>
<form id="code-form" action="/k" method="get">
  <label for="code-input">Code du cadeau</label>
  <input id="code-input" name="code" maxlength="12" autocomplete="off">
  <input type="hidden" name="src" value="home">
  <button id="code-submit">Continuer</button>
</form>
<p>Besoin d'aide ? <a id="help" href="/aide">Lire <b>l'aide</b></a></p>
<noscript><p id="nojs">Tout fonctionne sans JavaScript.</p></noscript>
<script>document.title = "x < y";</script>
</body></html>"#;

    fn page(html: &str) -> Page {
        Page::from_html("http://ebk.example/start?x=1", html)
    }

    #[test]
    fn extracts_title_lang_text_and_elements() {
        let p = page(HOME);
        assert_eq!(p.title(), Some("eBook Cadeau"));
        assert_eq!(p.lang(), Some("fr"));
        assert_eq!(p.text_of("headline"), Some("Votre livre vous attend"));
        assert_eq!(p.text_of("help"), Some("Lire l'aide"));
        assert!(p.contains_text("Besoin d'aide ? Lire l'aide"));
        // Script and style content is not visible text; noscript content is.
        assert!(!p.text().contains("document.title"));
        assert!(!p.text().contains("font-family"));
        assert_eq!(p.text_of("nojs"), Some("Tout fonctionne sans JavaScript."));
        assert_eq!(p.element("help").unwrap().attr("href"), Some("/aide"));
        assert_eq!(p.element("help").unwrap().tag(), "a");
        assert!(p.element("missing").is_none());
        assert_eq!(p.links().count(), 1);
        assert_eq!(p.status(), 200);
        assert_eq!(p.weight_bytes(), HOME.len());
    }

    #[test]
    fn forms_expose_controls_labels_and_defaults() {
        let p = page(HOME);
        let form = p.form("code-form").unwrap();
        assert_eq!(form.method, FormMethod::Get);
        assert_eq!(form.action.as_deref(), Some("/k"));
        assert_eq!(form.enctype, "application/x-www-form-urlencoded");
        assert_eq!(form.fields.len(), 3);
        let code = form.field("code").unwrap();
        assert_eq!(code.kind, FieldKind::Text);
        assert_eq!(code.label.as_deref(), Some("Code du cadeau"));
        assert_eq!(code.maxlength, Some(12));
        assert_eq!(form.field("src").unwrap().kind, FieldKind::Hidden);
        let submit = &form.fields[form.default_submit().unwrap()];
        assert_eq!(submit.kind, FieldKind::Submit);
        assert_eq!(submit.text, "Continuer");
        assert_eq!(form.describe(), "<form id=\"code-form\">");
    }

    #[test]
    fn wrapping_labels_selects_textareas_and_radios() {
        let p = page(
            r#"<html lang="en"><body><form name="f" method="POST" action="/go" enctype="MULTIPART/FORM-DATA">
            <label>Your device <select name="device"><option value="kobo">Kobo</option>
              <option selected>Kindle</option></select></label>
            <label><input type="radio" name="fmt" value="epub" checked> EPUB</label>
            <label><input type="radio" name="fmt" value="pdf"> PDF</label>
            <textarea name="note">Hello
world</textarea>
            <input type="submit" value="Send it">
            <input type="date" name="when" aria-label="Delivery date">
            </form></body></html>"#,
        );
        let form = p.form("f").unwrap();
        assert_eq!(form.method, FormMethod::Post);
        assert_eq!(form.enctype, "multipart/form-data");
        let device = form.field("device").unwrap();
        assert_eq!(device.kind, FieldKind::Select);
        assert_eq!(device.label.as_deref(), Some("Your device"));
        assert_eq!(
            device.options,
            vec![
                SelectOption { value: "kobo".into(), text: "Kobo".into(), selected: false },
                SelectOption { value: "Kindle".into(), text: "Kindle".into(), selected: true },
            ]
        );
        let radios: Vec<_> = form.fields.iter().filter(|f| f.kind == FieldKind::Radio).collect();
        assert_eq!(radios.len(), 2);
        assert!(radios[0].checked && !radios[1].checked);
        assert_eq!(radios[1].label.as_deref(), Some("PDF"));
        assert_eq!(form.field("note").unwrap().value, "Hello\nworld");
        let submit = &form.fields[form.default_submit().unwrap()];
        assert_eq!(submit.text, "Send it");
        let when = form.field("when").unwrap();
        assert_eq!(when.kind, FieldKind::Text);
        assert_eq!(when.label.as_deref(), Some("Delivery date"));
        assert_eq!(p.facts.modern_inputs.len(), 1);
    }

    #[test]
    fn hidden_content_is_not_text_and_blocks_are_separated() {
        let p = page(
            r#"<html><body><p>one</p><p>two</p><span>three</span><span>four</span>
            <div hidden>secret</div><div style="display: none">gone</div>
            <template><p>tpl</p></template>a<br>b</body></html>"#,
        );
        assert_eq!(p.text(), "one two threefour a b");
    }

    #[test]
    fn base_href_and_meta_refresh_resolve_against_the_right_url() {
        let p = page(
            r#"<html><head><base href="/sub/"><meta http-equiv="Refresh" content="5; URL='next?a=1'"></head><body></body></html>"#,
        );
        assert_eq!(p.base_url().as_str(), "http://ebk.example/sub/");
        let refresh = p.meta_refresh().unwrap();
        assert_eq!(refresh.delay_secs, 5);
        assert_eq!(refresh.url.as_str(), "http://ebk.example/sub/next?a=1");

        let base = Url::parse("http://ebk.example/e").unwrap();
        assert_eq!(parse_meta_refresh("3", &base).unwrap().url, base);
        assert_eq!(parse_meta_refresh("0;url=/d/x", &base).unwrap().url.path(), "/d/x");
        assert_eq!(parse_meta_refresh("2.5, /y", &base).unwrap().delay_secs, 2);
        assert_eq!(parse_meta_refresh("", &base), None);
        assert_eq!(parse_meta_refresh("soon; url=/x", &base), None);
        assert_eq!(parse_meta_refresh("-1; url=/x", &base), None);
        assert_eq!(parse_meta_refresh("1; url=javascript:alert(1)", &base), None);
        assert_eq!(parse_meta_refresh("1; url", &base).unwrap().url.path(), "/url");
    }

    #[test]
    fn latin1_is_decoded_when_declared() {
        let body = b"<html><head><title>Caf\xe9</title></head><body>d\xe9j\xe0</body></html>";
        let latin = Page::from_response(
            Url::parse("http://ebk.example/").unwrap(),
            200,
            vec![("content-type".to_owned(), "text/html; charset=ISO-8859-1".to_owned())],
            body,
        );
        assert_eq!(latin.title(), Some("Café"));
        assert_eq!(latin.text(), "déjà");
        let undeclared = Page::from_response(
            Url::parse("http://ebk.example/").unwrap(),
            200,
            vec![("content-type".to_owned(), "text/html".to_owned())],
            body,
        );
        assert!(undeclared.text().contains('\u{fffd}'));
        assert!(undeclared.has_non_ascii());
    }

    #[test]
    fn hostile_markup_never_panics() {
        let cases: Vec<String> = vec![
            String::new(),
            "<".into(),
            "</form></form><form><form><input></form><input name=x></form>".into(),
            "<select><option><select><option>x</select>".into(),
            "<label><label><input></label></label><label for=''>".into(),
            "<textarea><form><input>".into(),
            "<meta http-equiv=refresh content=';;;'><base href='::::'>".into(),
            "<a href='%%%' onclick>x</a><button form=zz>".into(),
            "\u{0}\u{0}<p>\u{feff}\u{202e}</p>".into(),
            "<title>".to_owned() + &"t".repeat(5000),
            "<p style='color:".to_owned() + &"#".repeat(5000) + "'>x</p>",
            "<svg><foreignObject><form><input name=a></form></foreignObject></svg>".into(),
            "<table><form><tr><td><input name=q></td></tr></form></table>".into(),
        ];
        for case in cases {
            let p = page(&case);
            let _ = p.text();
            let _ = p.forms();
            let _ = p.lint(&crate::lint::LintConfig::default());
            let _ = p.select_text("p, input");
        }
        assert!(matches!(
            page("<p>x</p>").select_text("p[[["),
            Err(crate::error::BrowserError::BadSelector { .. })
        ));
    }

    #[test]
    fn pathological_markup_is_refused_in_linear_time() {
        let started = std::time::Instant::now();
        let mib = 1024 * 1024;
        // Deep nesting: 200 000 levels would take the tree builder minutes.
        let deep = "<div id=d>".repeat(mib / 10) + "x";
        let p = page(&deep);
        assert!(matches!(p.facts.pathological, Some((depth, _)) if depth > 100_000));
        assert_eq!(p.text(), "");
        assert!(p.forms().is_empty());
        assert!(p.select_text("div").unwrap().is_empty());
        let report = p.lint(&crate::lint::LintConfig { weight_budget_bytes: 2 * mib, ..Default::default() });
        assert!(report.errors().any(|f| f.code == "nesting-too-deep"), "{report}");
        // One tag with 100 000 attributes.
        let attrs = format!("<p {}>x</p>", (0..100_000).map(|i| format!("a{i}=1 ")).collect::<String>());
        assert!(matches!(page(&attrs).facts.pathological, Some((_, n)) if n > 50_000));
        // The cap is on real nesting: long flat documents and unclosed paragraphs are fine.
        let flat = "<p>paragraphe<li>item".repeat(20_000);
        let p = page(&flat);
        assert_eq!(p.facts.pathological, None);
        assert!(p.text().starts_with("paragraphe item paragraphe"));
        let nested_ok = "<div>".repeat(MAX_NESTING) + "ok" + &"</div>".repeat(MAX_NESTING);
        assert_eq!(page(&nested_ok).text(), "ok");
        assert!(page(&("<div>".repeat(MAX_NESTING + 1) + "no")).facts.pathological.is_some());
        // Many indexed elements: the index is capped.
        let wide = "<a href=/x id=l>y</a>".repeat(40_000);
        let p = page(&wide);
        assert_eq!(p.nodes.len(), MAX_INDEXED);
        assert!(p.facts.truncated_index);
        assert!(started.elapsed() < std::time::Duration::from_secs(20), "{:?}", started.elapsed());
    }

    #[test]
    fn shape_scan_ignores_comments_raw_text_and_quoted_markup() {
        let shape = |html: &str| scan_shape(html);
        assert_eq!(shape("").depth, 0);
        assert_eq!(shape("<div><span>x</span></div>").depth, 2);
        assert_eq!(shape("<!-- <div><div><div> --><b>x</b>").depth, 1);
        assert_eq!(shape("<script>if (a<b) { x = '<div><div><div>'; }</script><i>").depth, 1);
        assert_eq!(shape("<a title=\"<div><div>\" href='>'>x</a>").depth, 1);
        assert_eq!(shape("<br><br/><img src=x><p><p><p><li><li>").depth, 0);
        assert_eq!(shape("</div></div><div>").depth, 1);
        assert_eq!(shape("<svg/><svg/><svg/>").depth, 0);
        assert_eq!(shape("<div a=1 b=2 c>").attributes, 3);
        assert_eq!(shape("< div><3 <é> <").depth, 0);
        assert_eq!(shape("<textarea><div><div></textarea><em>").depth, 1);
        assert_eq!(shape("<script><div>").depth, 0);
        assert_eq!(shape("<!--").depth, 0);
        assert_eq!(shape("<div").depth, 1);
    }

}
