//! Tolerant extraction of cover-image URLs, ISBNs and the title from a product page.
//!
//! [`extract`] reads the HTML of a publisher's (or bookseller-like) page **without
//! executing or fetching anything** and returns where the page says its image is:
//!
//! 1. JSON-LD (`<script type="application/ld+json">`): nodes of type `Book`, `Product` or
//!    `CreativeWork`, anywhere in the document tree (`@graph`, `mainEntity`,
//!    `workExample`, …), whose `image` is a string, an array, or an `ImageObject`;
//! 2. Open Graph: `og:image`, `og:image:url`, `og:image:secure_url`, with
//!    `og:image:width` / `og:image:height`;
//! 3. Twitter cards: `twitter:image`, `twitter:image:src`;
//! 4. `<link rel="image_src">`;
//! 5. microdata: `itemprop="image"` on `meta`, `img`, `link` or `a`.
//!
//! That is the extraction order of `docs/research/02-cover-sources.md` (§2.6,
//! implication 19), reflected in [`Found::confidence`]. Relative URLs are resolved
//! against `<base href>` and the page URL.
//!
//! The page is untrusted input, so:
//!
//! * every image URL is passed through [`crate::guard::check_url`]; a URL that is not
//!   plain `https` to a named public host, or whose host is on the denylist, is still
//!   listed but carries a [`Found::refusal`] — **do not request or link it**;
//! * the ISBNs found on the page are returned ([`PageInfo::isbns`]) so that the caller
//!   can confirm the page is about the requested book ([`PageInfo::confirms`]);
//! * work is bounded: only the first [`MAX_HTML_BYTES`] are read, JSON-LD blocks, JSON
//!   nodes, attributes, images and ISBNs are capped, and nothing recurses on the HTML.
//!
//! The result is URLs and declared dimensions only — facts, not pixels. In the hosted
//! service they are shown as plain links with the source name; they are never inlined
//! and never downloaded (research §7.3).
//!
//! The tokenizer is a small hand-written scanner, not an HTML5 parser: it understands
//! tags, quoted and unquoted attributes, comments, and the raw-text elements `script`,
//! `style` and `title`. One deliberate deviation: inside a JSON-LD block a `</script>`
//! that sits **inside a JSON string** does not end the block when the block parses as
//! JSON that way (browsers would cut it there; CMS output with an unescaped `</script>`
//! in a description is common enough to tolerate).

use ebc_isbn::Isbn13;
use ebc_model::BookMeta;
use serde_json::Value;
use url::Url;

use crate::guard::{self, Refusal};
use crate::tdm;

/// Only this many leading bytes of a page are read (2 MiB, the body cap of the fetcher
/// design); the rest is ignored and [`PageInfo::truncated`] is set.
pub const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;
/// At most this many images are returned.
pub const MAX_IMAGES: usize = 64;
/// At most this many ISBNs are returned.
pub const MAX_ISBNS: usize = 64;

const MAX_JSON_LD_BLOCKS: usize = 32;
const MAX_JSON_LD_BYTES: usize = 512 * 1024;
const MAX_JSON_NODES: usize = 20_000;
const MAX_JSON_DEPTH: usize = 64;
const MAX_RAW_IMAGES: usize = 512;
const MAX_ATTRS: usize = 64;
const MAX_ATTR_VALUE_BYTES: usize = 16 * 1024;
const MAX_TITLE_CHARS: usize = 300;
const MAX_ISBN_TEXT_BYTES: usize = 64 * 1024;

/// Which markup an image URL was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Via {
    /// A JSON-LD `Book`, `Product` or `CreativeWork` node.
    JsonLd,
    /// `og:image` and its companions.
    OpenGraph,
    /// `twitter:image`.
    TwitterCard,
    /// `<link rel="image_src">`.
    LinkImageSrc,
    /// `itemprop="image"`.
    Microdata,
}

/// One image URL found on a page.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    /// Absolute URL (resolved against `<base href>` and the page URL), without fragment.
    pub url: String,
    /// Width in pixels as **declared** by the page, if any. Unverified.
    pub width: Option<u32>,
    /// Height in pixels as declared by the page, if any. Unverified.
    pub height: Option<u32>,
    /// The markup the URL came from (the most trustworthy one when it appears twice).
    pub via: Via,
    /// 0–1, higher is more likely to be the book's cover: JSON-LD `Book` 0.95,
    /// `Product` 0.9, `CreativeWork` 0.8 (0.15 less for a node nested inside another
    /// typed node), Open Graph 0.7, Twitter 0.6, `image_src` 0.5, microdata 0.45.
    pub confidence: f32,
    /// ISBNs stated by the JSON-LD node the image belongs to (`isbn`, `gtin13`, `gtin`,
    /// `productID`); empty for the other kinds of markup.
    pub isbns: Vec<Isbn13>,
    /// `Some` when the URL must not be requested or linked (not `https`, an address or
    /// local name, a denied host, a known placeholder asset).
    pub refusal: Option<Refusal>,
}

/// How a page relates to the book that was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsbnMatch {
    /// The page states the book's own ISBN.
    Exact,
    /// The page states one of the book's related ISBNs (another edition or format): the
    /// cover may differ, ask the user to confirm.
    OtherEdition,
    /// Neither: flag the result as "ISBN not found on page".
    NotFound,
}

/// What [`extract`] found on a page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageInfo {
    /// Image URLs, most likely cover first, de-duplicated.
    pub images: Vec<Found>,
    /// Every valid ISBN on the page, as ISBN-13, de-duplicated: structured data first
    /// (JSON-LD fields, `isbn` meta tags), then JSON-LD text, then visible text.
    pub isbns: Vec<Isbn13>,
    /// The title: JSON-LD `name` of the best typed node, else `og:title`, else `<title>`.
    pub title: Option<String>,
    /// `<meta name="tdm-reservation">`: `Some(true)` reserved, `Some(false)` not
    /// reserved, `None` absent or invalid. Covers the HTML document only, not images.
    pub tdm_reservation: Option<bool>,
    /// A robots meta tag carries `noai` or `noimageai`.
    pub noai: bool,
    /// The input was longer than [`MAX_HTML_BYTES`] and was cut.
    pub truncated: bool,
}

impl PageInfo {
    /// `true` when the page states `isbn`.
    #[must_use]
    pub fn mentions(&self, isbn: &Isbn13) -> bool {
        self.isbns.contains(isbn)
    }

    /// Whether the page is about `book`, judged by the ISBNs it states.
    #[must_use]
    pub fn confirms(&self, book: &BookMeta) -> IsbnMatch {
        if book.isbn13.as_ref().is_some_and(|isbn| self.mentions(isbn)) {
            IsbnMatch::Exact
        } else if book.related_isbns.iter().any(|isbn| self.mentions(isbn)) {
            IsbnMatch::OtherEdition
        } else {
            IsbnMatch::NotFound
        }
    }

    /// The images that may be requested or linked ([`Found::refusal`] is `None`).
    pub fn usable(&self) -> impl Iterator<Item = &Found> {
        self.images.iter().filter(|image| image.refusal.is_none())
    }
}

// ------------------------------------------------------------------------------------
// Small helpers
// ------------------------------------------------------------------------------------

fn find_byte(bytes: &[u8], from: usize, needle: u8) -> Option<usize> {
    bytes.get(from..)?.iter().position(|b| *b == needle).map(|at| at + from)
}

/// Finds an ASCII `needle` starting with `<`, ignoring case, in `bytes[from..to]`.
fn find_tag_ci(bytes: &[u8], from: usize, to: usize, needle: &[u8]) -> Option<usize> {
    let to = to.min(bytes.len());
    let mut at = from;
    while let Some(lt) = find_byte(bytes, at, b'<').filter(|lt| *lt < to) {
        if bytes.get(lt..lt + needle.len()).is_some_and(|w| w.eq_ignore_ascii_case(needle)) {
            return Some(lt);
        }
        at = lt + 1;
    }
    None
}

/// Position just after the next `>` at or after `from`, or the end of the input.
fn after_gt(bytes: &[u8], from: usize) -> usize {
    find_byte(bytes, from, b'>').map_or(bytes.len(), |at| at + 1)
}

fn decode_entity(name: &str) -> Option<char> {
    if let Some(number) = name.strip_prefix('#') {
        let code = match number.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => number.parse::<u32>().ok()?,
        };
        return char::from_u32(code).filter(|c| *c != '\0');
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "eacute" => 'é',
        "egrave" => 'è',
        "ecirc" => 'ê',
        "euml" => 'ë',
        "agrave" => 'à',
        "acirc" => 'â',
        "auml" => 'ä',
        "ccedil" => 'ç',
        "icirc" => 'î',
        "iuml" => 'ï',
        "ocirc" => 'ô',
        "ouml" => 'ö',
        "ugrave" => 'ù',
        "ucirc" => 'û',
        "uuml" => 'ü',
        "ntilde" => 'ñ',
        "szlig" => 'ß',
        "oelig" => 'œ',
        "aelig" => 'æ',
        "Eacute" => 'É',
        "Egrave" => 'È',
        "Ecirc" => 'Ê',
        "Agrave" => 'À',
        "Ccedil" => 'Ç',
        "OElig" => 'Œ',
        "Auml" => 'Ä',
        "Ouml" => 'Ö',
        "Uuml" => 'Ü',
        "rsquo" => '\u{2019}',
        "lsquo" => '\u{2018}',
        "ldquo" => '\u{201c}',
        "rdquo" => '\u{201d}',
        "laquo" => '«',
        "raquo" => '»',
        "ndash" => '\u{2013}',
        "mdash" => '\u{2014}',
        "hellip" => '\u{2026}',
        "copy" => '©',
        "reg" => '®',
        "euro" => '€',
        _ => return None,
    })
}

/// Decodes numeric and the common named character references; unknown ones are kept.
fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp + 1..];
        let semicolon = tail.bytes().take(12).position(|b| b == b';');
        match semicolon.and_then(|end| decode_entity(&tail[..end]).map(|c| (c, end))) {
            Some((c, end)) => {
                out.push(c);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

fn collapse_whitespace(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for word in s.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
        if out.len() > max_chars * 4 {
            break;
        }
    }
    out.chars().take(max_chars).collect()
}

// ------------------------------------------------------------------------------------
// Tag scanner
// ------------------------------------------------------------------------------------

type Attrs = Vec<(String, String)>;

/// Parses the attributes of a tag whose name ends at `start`. Returns the attributes
/// (lower-cased names, entity-decoded values; only when `collect` is set) and the
/// position just after the closing `>` (or the end of the input).
fn parse_attrs(src: &str, start: usize, collect: bool) -> (Attrs, usize) {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut attrs = Attrs::new();
    let mut i = start;
    loop {
        while i < len && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        if i >= len {
            return (attrs, len);
        }
        if bytes[i] == b'>' {
            return (attrs, i + 1);
        }
        let name_start = i;
        while i < len && !bytes[i].is_ascii_whitespace() && !matches!(bytes[i], b'=' | b'>' | b'/') {
            i += 1;
        }
        if i == name_start {
            // A stray `=`: skip it so that the loop always advances.
            i += 1;
            continue;
        }
        let name = &src[name_start..i];
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = "";
        if i < len && bytes[i] == b'=' {
            i += 1;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < len && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                let value_start = i + 1;
                let value_end = find_byte(bytes, value_start, quote).unwrap_or(len);
                value = &src[value_start..value_end];
                i = (value_end + 1).min(len);
            } else {
                let value_start = i;
                while i < len && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                value = &src[value_start..i];
            }
        }
        if collect && attrs.len() < MAX_ATTRS && value.len() <= MAX_ATTR_VALUE_BYTES && name.len() <= 64 {
            attrs.push((name.to_ascii_lowercase(), decode_entities(value)));
        }
    }
}

fn attr<'a>(attrs: &'a Attrs, name: &str) -> Option<&'a str> {
    attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
}

fn has_token(value: Option<&str>, token: &str) -> bool {
    value.is_some_and(|v| v.split_ascii_whitespace().any(|t| t.eq_ignore_ascii_case(token)))
}

/// Reads a pixel dimension from an attribute or a JSON string: `"600"`, `"600px"`,
/// `"600.0"`.
fn parse_dimension(text: &str) -> Option<u32> {
    let digits: String = text.trim().chars().take_while(char::is_ascii_digit).take(7).collect();
    digits.parse::<u32>().ok().filter(|v| (1..=100_000).contains(v))
}

// ------------------------------------------------------------------------------------
// Collected state
// ------------------------------------------------------------------------------------

#[derive(Debug, Default)]
struct OgGroup {
    urls: Vec<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug)]
struct RawImage {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    via: Via,
    confidence: f32,
    isbns: Vec<Isbn13>,
}

#[derive(Debug, Default)]
struct Collector {
    images: Vec<RawImage>,
    og: Vec<OgGroup>,
    og_pending: (Option<u32>, Option<u32>),
    /// Text of structured ISBN fields (JSON-LD `isbn`, meta tags).
    isbn_fields: String,
    /// Raw text of the JSON-LD blocks.
    json_text: String,
    /// Visible text.
    text: String,
    base_href: Option<String>,
    html_title: Option<String>,
    og_title: Option<String>,
    json_title: Option<(f32, String)>,
    tdm_reservation: Option<bool>,
    noai: bool,
    json_blocks: usize,
    json_nodes: usize,
}

impl Collector {
    fn add_image(&mut self, url: &str, width: Option<u32>, height: Option<u32>, via: Via, confidence: f32, isbns: &[Isbn13]) {
        let url = url.trim();
        if url.is_empty() || url.len() > guard::MAX_URL_BYTES || self.images.len() >= MAX_RAW_IMAGES {
            return;
        }
        self.images.push(RawImage { url: url.to_owned(), width, height, via, confidence, isbns: isbns.to_vec() });
    }

    fn add_isbn_field(&mut self, text: &str) {
        if self.isbn_fields.len() + text.len() < MAX_ISBN_TEXT_BYTES {
            self.isbn_fields.push_str(text);
            self.isbn_fields.push('\n');
        }
    }

    fn og_image(&mut self, url: &str, starts_group: bool) {
        if self.og.len() >= MAX_IMAGES && starts_group {
            return;
        }
        if starts_group || self.og.is_empty() {
            let (width, height) = core::mem::take(&mut self.og_pending);
            self.og.push(OgGroup { urls: Vec::new(), width, height });
        }
        if let Some(group) = self.og.last_mut()
            && group.urls.len() < 4
        {
            group.urls.push(url.to_owned());
        }
    }

    fn og_dimension(&mut self, value: &str, is_width: bool) {
        let dimension = parse_dimension(value);
        let slot = match self.og.last_mut() {
            Some(group) if is_width => &mut group.width,
            Some(group) => &mut group.height,
            None if is_width => &mut self.og_pending.0,
            None => &mut self.og_pending.1,
        };
        if slot.is_none() {
            *slot = dimension;
        }
    }

    fn handle_meta(&mut self, attrs: &Attrs) {
        let content = attr(attrs, "content").unwrap_or_default();
        for key_attr in ["property", "name"] {
            let Some(key) = attr(attrs, key_attr) else { continue };
            match key.trim().to_ascii_lowercase().as_str() {
                "og:image" | "og:image:url" => self.og_image(content, true),
                "og:image:secure_url" => self.og_image(content, false),
                "og:image:width" => self.og_dimension(content, true),
                "og:image:height" => self.og_dimension(content, false),
                "twitter:image" | "twitter:image:src" => {
                    self.add_image(content, None, None, Via::TwitterCard, 0.6, &[]);
                }
                "og:title" => {
                    if self.og_title.is_none() && !content.trim().is_empty() {
                        self.og_title = Some(collapse_whitespace(content, MAX_TITLE_CHARS));
                    }
                }
                "book:isbn" | "books:isbn" | "og:isbn" | "isbn" | "citation_isbn" => {
                    self.add_isbn_field(content);
                }
                "tdm-reservation" => {
                    if self.tdm_reservation.is_none() {
                        self.tdm_reservation = tdm::parse_reservation(content);
                    }
                }
                "robots" => self.noai |= tdm::directives_say_noai(content),
                _ => {}
            }
        }
        self.handle_itemprop(attrs, content);
    }

    /// `itemprop="image"` (URL in `url`) and `itemprop="isbn"` on a `meta` tag.
    fn handle_itemprop(&mut self, attrs: &Attrs, url: &str) {
        let itemprop = attr(attrs, "itemprop");
        if has_token(itemprop, "image") {
            let width = attr(attrs, "width").and_then(parse_dimension);
            let height = attr(attrs, "height").and_then(parse_dimension);
            self.add_image(url, width, height, Via::Microdata, 0.45, &[]);
        }
        if (has_token(itemprop, "isbn") || has_token(itemprop, "gtin13") || has_token(itemprop, "gtin"))
            && let Some(content) = attr(attrs, "content")
        {
            self.add_isbn_field(content);
        }
    }

    fn handle_tag(&mut self, name: &str, attrs: &Attrs) {
        if name.eq_ignore_ascii_case("meta") {
            self.handle_meta(attrs);
        } else if name.eq_ignore_ascii_case("link") {
            let href = attr(attrs, "href").unwrap_or_default();
            if has_token(attr(attrs, "rel"), "image_src") {
                self.add_image(href, None, None, Via::LinkImageSrc, 0.5, &[]);
            }
            self.handle_itemprop(attrs, href);
        } else if name.eq_ignore_ascii_case("base") {
            if self.base_href.is_none() {
                self.base_href = attr(attrs, "href").map(|h| h.trim().to_owned()).filter(|h| !h.is_empty());
            }
        } else if name.eq_ignore_ascii_case("img") {
            self.handle_itemprop(attrs, attr(attrs, "src").unwrap_or_default());
        } else if name.eq_ignore_ascii_case("a") {
            self.handle_itemprop(attrs, attr(attrs, "href").unwrap_or_default());
        }
    }
}

// ------------------------------------------------------------------------------------
// JSON-LD
// ------------------------------------------------------------------------------------

/// The end of a JSON-LD script body: the first `</script` that is not inside a JSON
/// string, searched in `bytes[start..limit]`.
fn json_aware_script_end(bytes: &[u8], start: usize, limit: usize) -> Option<usize> {
    let limit = limit.min(bytes.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = start;
    while i < limit {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else if b == b'"' {
            in_string = true;
        } else if b == b'<' && bytes.get(i..i + 8).is_some_and(|w| w.eq_ignore_ascii_case(b"</script")) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Replaces raw control characters inside JSON strings by spaces (a frequent CMS
/// defect that strict parsers reject).
fn sanitise_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_string = false;
    let mut escaped = false;
    for c in s.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            } else if (c as u32) < 0x20 {
                out.push(' ');
                continue;
            }
        } else if c == '"' {
            in_string = true;
        }
        out.push(c);
    }
    out
}

/// Parses the body of a JSON-LD script, tolerating comment / CDATA wrappers, a trailing
/// semicolon, raw control characters in strings and entity-encoded quotes.
fn parse_json_ld(raw: &str) -> Option<Value> {
    let mut body = raw.trim();
    for (prefix, suffix) in [("<!--", "-->"), ("//<![CDATA[", "//]]>"), ("/*<![CDATA[*/", "/*]]>*/"), ("<![CDATA[", "]]>")] {
        if let Some(inner) = body.strip_prefix(prefix) {
            body = inner.strip_suffix(suffix).unwrap_or(inner).trim();
        }
    }
    let body = body.trim_end_matches(';').trim();
    if body.is_empty() {
        return None;
    }
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Some(value);
    }
    let cleaned = sanitise_json(body);
    if let Ok(value) = serde_json::from_str::<Value>(&cleaned) {
        return Some(value);
    }
    if body.contains("&quot;") {
        return serde_json::from_str::<Value>(&sanitise_json(&decode_entities(body))).ok();
    }
    None
}

/// The confidence of a JSON-LD node from its `@type`, or `None` when the type is not
/// one we read images from.
fn type_confidence(kind: Option<&Value>) -> Option<f32> {
    fn one(name: &str) -> Option<f32> {
        let name = name.trim();
        let name = ["https://schema.org/", "http://schema.org/", "schema:"]
            .iter()
            .find_map(|prefix| name.strip_prefix(prefix))
            .unwrap_or(name);
        if name.eq_ignore_ascii_case("Book") {
            Some(0.95)
        } else if name.eq_ignore_ascii_case("Product") {
            Some(0.9)
        } else if name.eq_ignore_ascii_case("CreativeWork") {
            Some(0.8)
        } else {
            None
        }
    }
    match kind? {
        Value::String(name) => one(name),
        Value::Array(names) => names
            .iter()
            .take(16)
            .filter_map(|n| n.as_str().and_then(one))
            .fold(None, |best: Option<f32>, c| Some(best.map_or(c, |b| b.max(c)))),
        _ => None,
    }
}

fn json_dimension(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(n) => {
            let v = n.as_f64()?;
            (v.is_finite() && (1.0..=100_000.0).contains(&v)).then(|| v.round() as u32)
        }
        Value::String(s) => parse_dimension(s),
        // QuantitativeValue { "value": 600 }
        Value::Object(o) => match o.get("value")? {
            Value::Number(n) => {
                let v = n.as_f64()?;
                (v.is_finite() && (1.0..=100_000.0).contains(&v)).then(|| v.round() as u32)
            }
            Value::String(s) => parse_dimension(s),
            _ => None,
        },
        _ => None,
    }
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.chars().take(256).collect()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

impl Collector {
    fn json_images(&mut self, image: &Value, confidence: f32, isbns: &[Isbn13], depth: usize) {
        match image {
            Value::String(url) => self.add_image(url, None, None, Via::JsonLd, confidence, isbns),
            Value::Array(items) if depth < 4 => {
                for item in items.iter().take(16) {
                    self.json_images(item, confidence, isbns, depth + 1);
                }
            }
            Value::Object(object) => {
                let url = ["url", "contentUrl", "@id"].iter().find_map(|k| object.get(*k)?.as_str());
                if let Some(url) = url {
                    let width = json_dimension(object.get("width"));
                    let height = json_dimension(object.get("height"));
                    self.add_image(url, width, height, Via::JsonLd, confidence, isbns);
                }
            }
            _ => {}
        }
    }

    fn walk_json(&mut self, value: &Value, depth: usize, inside_typed: bool) {
        if depth > MAX_JSON_DEPTH || self.json_nodes >= MAX_JSON_NODES {
            return;
        }
        self.json_nodes += 1;
        match value {
            Value::Array(items) => {
                for item in items {
                    self.walk_json(item, depth + 1, inside_typed);
                }
            }
            Value::Object(object) => {
                let confidence = type_confidence(object.get("@type").or_else(|| object.get("type")))
                    .map(|c| if inside_typed { c - 0.15 } else { c });
                if let Some(confidence) = confidence {
                    let mut field_text = String::new();
                    for key in ["isbn", "gtin13", "gtin", "productID"] {
                        match object.get(key) {
                            Some(Value::Array(items)) => {
                                for text in items.iter().take(8).filter_map(scalar_text) {
                                    field_text.push_str(&text);
                                    field_text.push('\n');
                                }
                            }
                            Some(other) => {
                                if let Some(text) = scalar_text(other) {
                                    field_text.push_str(&text);
                                    field_text.push('\n');
                                }
                            }
                            None => {}
                        }
                    }
                    let isbns = ebc_isbn::extract_all(&field_text);
                    self.add_isbn_field(&field_text);
                    if let Some(image) = object.get("image") {
                        self.json_images(image, confidence, &isbns, 0);
                    }
                    if let Some(name) = object.get("name").and_then(Value::as_str) {
                        let better = self.json_title.as_ref().is_none_or(|(c, _)| *c < confidence);
                        if better && !name.trim().is_empty() {
                            self.json_title = Some((confidence, collapse_whitespace(name, MAX_TITLE_CHARS)));
                        }
                    }
                }
                let typed = inside_typed || confidence.is_some();
                for (key, child) in object {
                    if key != "image" && (child.is_object() || child.is_array()) {
                        self.walk_json(child, depth + 1, typed);
                    }
                }
            }
            _ => {}
        }
    }

    /// Handles the body of a `<script type="application/ld+json">` that starts at
    /// `start`; returns the position where the body ends (at its `</script`, or the end
    /// of the input).
    fn handle_json_ld(&mut self, src: &str, start: usize) -> usize {
        let bytes = src.as_bytes();
        let html_end = find_tag_ci(bytes, start, bytes.len(), b"</script").unwrap_or(bytes.len());
        if self.json_blocks >= MAX_JSON_LD_BLOCKS {
            return html_end;
        }
        self.json_blocks += 1;
        let mut candidates = [None, None];
        let aware_end = json_aware_script_end(bytes, start, start.saturating_add(MAX_JSON_LD_BYTES));
        if let Some(end) = aware_end.filter(|end| *end != html_end) {
            candidates[0] = Some(end);
        }
        if html_end - start <= MAX_JSON_LD_BYTES {
            candidates[1] = Some(html_end);
        }
        for end in candidates.into_iter().flatten() {
            let raw = &src[start..end];
            if let Some(value) = parse_json_ld(raw) {
                if self.json_text.len() + raw.len() <= 2 * MAX_JSON_LD_BYTES {
                    self.json_text.push_str(raw);
                    self.json_text.push('\n');
                }
                self.walk_json(&value, 0, false);
                return end;
            }
        }
        html_end
    }
}

// ------------------------------------------------------------------------------------
// Entry point
// ------------------------------------------------------------------------------------

/// The scanner proper: walks the tags of `src` once and fills `collector`.
fn scan(src: &str, collector: &mut Collector) {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut pos = 0usize;
    while pos < len {
        let Some(lt) = find_byte(bytes, pos, b'<') else {
            collector.text.push_str(&src[pos..]);
            break;
        };
        collector.text.push_str(&src[pos..lt]);
        let rest = &bytes[lt..];
        if rest.starts_with(b"<!--") {
            pos = src[lt + 4..].find("-->").map_or(len, |at| lt + 4 + at + 3);
            continue;
        }
        if rest.starts_with(b"<!") || rest.starts_with(b"<?") {
            pos = after_gt(bytes, lt + 2);
            continue;
        }
        let closing = rest.get(1) == Some(&b'/');
        let name_start = lt + 1 + usize::from(closing);
        let mut name_end = name_start;
        while name_end < len && (bytes[name_end].is_ascii_alphanumeric() || matches!(bytes[name_end], b'-' | b':')) {
            name_end += 1;
        }
        if name_end == name_start || !bytes[name_start].is_ascii_alphabetic() {
            // A stray `<`: text.
            collector.text.push('<');
            pos = lt + 1;
            continue;
        }
        collector.text.push(' ');
        if closing {
            pos = after_gt(bytes, name_end);
            continue;
        }
        let name = &src[name_start..name_end];
        let wanted = ["meta", "link", "base", "img", "a", "script"].iter().any(|t| name.eq_ignore_ascii_case(t));
        let (attrs, after) = parse_attrs(src, name_end, wanted);
        pos = after;
        if name.eq_ignore_ascii_case("script") {
            let kind = attr(&attrs, "type").unwrap_or_default().trim().to_ascii_lowercase();
            let body_end = if kind.starts_with("application/ld+json") {
                collector.handle_json_ld(src, pos)
            } else {
                find_tag_ci(bytes, pos, len, b"</script").unwrap_or(len)
            };
            pos = after_gt(bytes, body_end);
        } else if name.eq_ignore_ascii_case("style") {
            let body_end = find_tag_ci(bytes, pos, len, b"</style").unwrap_or(len);
            pos = after_gt(bytes, body_end);
        } else if name.eq_ignore_ascii_case("title") {
            let body_end = find_tag_ci(bytes, pos, len, b"</title").unwrap_or(len);
            if collector.html_title.is_none() {
                let title = collapse_whitespace(&decode_entities(&src[pos..body_end]), MAX_TITLE_CHARS);
                if !title.is_empty() {
                    collector.html_title = Some(title);
                }
            }
            pos = after_gt(bytes, body_end);
        } else if wanted {
            collector.handle_tag(name, &attrs);
        }
    }
}

fn resolve(base: Option<&Url>, raw: &str) -> Option<Url> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > guard::MAX_URL_BYTES {
        return None;
    }
    let mut url = match base {
        Some(base) => base.join(raw).ok()?,
        None => Url::parse(raw).ok()?,
    };
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    url.set_fragment(None);
    Some(url)
}

/// Extracts image URLs, ISBNs and the title from the HTML of a product page.
///
/// `base_url` is the URL the page was served from (after redirects); when it is not an
/// absolute `http(s)` URL, only absolute URLs of the page can be resolved. Never
/// panics, whatever the input; see the module docs for the bounds.
///
/// ```
/// use ebc_covers::scrape::{extract, Via};
///
/// let html = r#"<html><head><title>L'&Eacute;tranger</title>
///   <meta property="og:image" content="/img/9782070360024.jpg">
///   <meta property="og:image:width" content="600"></head>
///   <body>ISBN 978-2-07-036002-4</body></html>"#;
/// let page = extract(html, "https://www.editeur.example.fr/livre/etranger");
/// assert_eq!(page.images[0].url, "https://www.editeur.example.fr/img/9782070360024.jpg");
/// assert_eq!((page.images[0].via, page.images[0].width), (Via::OpenGraph, Some(600)));
/// assert_eq!(page.isbns[0].to_string(), "9782070360024");
/// assert_eq!(page.title.as_deref(), Some("L'Étranger"));
/// ```
#[must_use]
pub fn extract(html: &str, base_url: &str) -> PageInfo {
    let truncated = html.len() > MAX_HTML_BYTES;
    let mut end = html.len().min(MAX_HTML_BYTES);
    while !html.is_char_boundary(end) {
        end -= 1;
    }
    let src = &html[..end];

    let mut collector = Collector::default();
    scan(src, &mut collector);

    // Open Graph groups become images now that their dimensions are known.
    for group in core::mem::take(&mut collector.og) {
        for url in &group.urls {
            collector.add_image(url, group.width, group.height, Via::OpenGraph, 0.7, &[]);
        }
    }

    // Base URL: <base href> resolved against the page URL.
    let page_url = Url::parse(base_url.trim()).ok().filter(|u| matches!(u.scheme(), "http" | "https"));
    let base = collector
        .base_href
        .as_deref()
        .and_then(|href| resolve(page_url.as_ref(), href))
        .or(page_url);

    // Resolve, de-duplicate (best confidence first), cap.
    let mut raw = core::mem::take(&mut collector.images);
    raw.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    let mut images: Vec<Found> = Vec::new();
    for image in raw {
        let Some(url) = resolve(base.as_ref(), &image.url) else { continue };
        let url = String::from(url);
        if let Some(existing) = images.iter_mut().find(|f| f.url == url) {
            existing.width = existing.width.or(image.width);
            existing.height = existing.height.or(image.height);
            for isbn in image.isbns {
                if !existing.isbns.contains(&isbn) {
                    existing.isbns.push(isbn);
                }
            }
            continue;
        }
        if images.len() >= MAX_IMAGES {
            continue;
        }
        let refusal = guard::check_url(&url).err();
        images.push(Found {
            url,
            width: image.width,
            height: image.height,
            via: image.via,
            confidence: image.confidence,
            isbns: image.isbns,
            refusal,
        });
    }

    // ISBNs: structured fields, then JSON-LD text, then visible text.
    let mut isbns: Vec<Isbn13> = Vec::new();
    let visible = decode_entities(&collector.text);
    for text in [collector.isbn_fields.as_str(), collector.json_text.as_str(), visible.as_str()] {
        for isbn in ebc_isbn::extract_all(text) {
            if isbns.len() < MAX_ISBNS && !isbns.contains(&isbn) {
                isbns.push(isbn);
            }
        }
    }

    let title = collector
        .json_title
        .map(|(_, title)| title)
        .or(collector.og_title)
        .or(collector.html_title)
        .filter(|t| !t.is_empty());

    PageInfo {
        images,
        isbns,
        title,
        tdm_reservation: collector.tdm_reservation,
        noai: collector.noai,
        truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities() {
        assert_eq!(decode_entities("a &amp; b &lt;c&gt; &quot;d&quot; &#233;&#xE9;&#XE9;"), "a & b <c> \"d\" ééé");
        assert_eq!(decode_entities("&unknown; & &; &#; &#xZZ; &#0; &#1114112; &amp"), "&unknown; & &; &#; &#xZZ; &#0; &#1114112; &amp");
        assert_eq!(decode_entities("L&rsquo;&Eacute;tranger &ndash; Folio"), "L\u{2019}Étranger \u{2013} Folio");
        assert_eq!(decode_entities("&&&&amp;&"), "&&&&&");
        assert_eq!(decode_entities("é&é;"), "é&é;");
    }

    #[test]
    fn attribute_parsing() {
        let (attrs, after) = parse_attrs(r#" a="1" B='two words' c=three d e = "x&amp;y" /> tail"#, 0, true);
        assert_eq!(
            attrs,
            vec![
                ("a".to_owned(), "1".to_owned()),
                ("b".to_owned(), "two words".to_owned()),
                ("c".to_owned(), "three".to_owned()),
                ("d".to_owned(), String::new()),
                ("e".to_owned(), "x&y".to_owned()),
            ]
        );
        assert_eq!(after, 47);
        // Unterminated quote and stray characters end at the input's end.
        let (attrs, after) = parse_attrs(r#" a="never closed"#, 0, true);
        assert_eq!((attrs.len(), after), (1, 16));
        let (attrs, after) = parse_attrs(" = == a=b>", 0, true);
        assert_eq!(attrs, vec![("a".to_owned(), "b".to_owned())]);
        assert_eq!(after, 10);
    }

    #[test]
    fn dimensions() {
        assert_eq!(parse_dimension("600"), Some(600));
        assert_eq!(parse_dimension(" 600px "), Some(600));
        assert_eq!(parse_dimension("600.5"), Some(600));
        assert_eq!(parse_dimension("0"), None);
        assert_eq!(parse_dimension("-5"), None);
        assert_eq!(parse_dimension("99999999999999999999"), None);
        assert_eq!(parse_dimension("auto"), None);
    }

    #[test]
    fn json_ld_wrappers_and_defects() {
        assert!(parse_json_ld(" <!-- {\"a\":1} --> ").is_some());
        assert!(parse_json_ld("//<![CDATA[\n{\"a\":1}\n//]]>").is_some());
        assert!(parse_json_ld("{\"a\":1};").is_some());
        assert!(parse_json_ld("{\"a\":\"line one\nline\ttwo\"}").is_some());
        assert!(parse_json_ld("{&quot;a&quot;:&quot;b&quot;}").is_some());
        assert!(parse_json_ld("").is_none());
        assert!(parse_json_ld("{\"a\":").is_none());
        assert!(parse_json_ld("not json").is_none());
    }
}
