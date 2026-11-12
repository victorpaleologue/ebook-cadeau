//! Small sans-IO helpers shared by the providers: URL building, HTTP status
//! classification, body sniffing and decoding, tolerant JSON access.

use ebc_isbn::Isbn13;
use ebc_model::HttpResponse;
use serde_json::Value;

use crate::error::{CatalogError, shorten};
use crate::normalize::CP1252_HIGH;

/// Largest body a provider will parse (research implication 21: 2 MB).
pub(crate) const MAX_BODY: usize = 2 * 1024 * 1024;

/// Marker that introduces provider context in a URL fragment (`…#ebc=authors`).
const CONTEXT_MARKER: &str = "ebc=";

// ---------------------------------------------------------------------------------------
// URLs
// ---------------------------------------------------------------------------------------

/// Percent-encodes everything but RFC 3986 unreserved characters.
pub(crate) fn encode_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 0x0F)]));
        }
    }
    out
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Decodes `%XX` sequences and `+` (as a space); invalid sequences are kept as they are.
pub(crate) fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        let hex = |at: usize| bytes.get(at).and_then(|b| char::from(*b).to_digit(16));
        if byte == b'%'
            && let (Some(hi), Some(lo)) = (hex(i + 1), hex(i + 2))
        {
            out.push(u8::try_from(hi * 16 + lo).unwrap_or(b'?'));
            i += 3;
            continue;
        }
        out.push(if byte == b'+' { b' ' } else { byte });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Appends query parameters to `base`, which may already carry a query string. Names are
/// written as given (they are constants); values are percent-encoded.
pub(crate) fn url_with_query(base: &str, params: &[(&str, &str)]) -> String {
    let mut url = base.trim_end_matches(['?', '&']).to_owned();
    let mut separator = if url.contains('?') { '&' } else { '?' };
    for (name, value) in params {
        url.push(separator);
        separator = '&';
        url.push_str(name);
        url.push('=');
        url.push_str(&encode_component(value));
    }
    url
}

/// Joins a base URL and an absolute path, avoiding a double slash.
pub(crate) fn join(base: &str, path: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
}

/// The URL without its fragment.
pub(crate) fn without_fragment(url: &str) -> &str {
    url.split('#').next().unwrap_or(url)
}

/// Stores provider context in the URL fragment. HTTP clients never send fragments.
pub(crate) fn with_context(url: &str, context: &str) -> String {
    format!("{}#{CONTEXT_MARKER}{}", without_fragment(url), encode_component(context))
}

/// Reads back what [`with_context`] stored.
pub(crate) fn context(url: &str) -> Option<String> {
    let (_, fragment) = url.split_once('#')?;
    fragment.strip_prefix(CONTEXT_MARKER).map(percent_decode)
}

/// `scheme://host[:port]`, lower-cased, or `None` when the URL is not `http(s)`.
pub(crate) fn origin(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    // Refuse userinfo: "https://openlibrary.org@evil.example/" is not openlibrary.org.
    if authority.is_empty() || authority.contains('@') || authority.contains('\\') {
        return None;
    }
    Some(format!("{scheme}://{}", authority.to_ascii_lowercase()))
}

/// The host of an `http(s)` URL, lower-cased, without port.
pub(crate) fn host(url: &str) -> Option<String> {
    let origin = origin(url)?;
    let authority = origin.split_once("://")?.1;
    let host = if authority.starts_with('[') {
        authority.split(']').next().map(|h| format!("{h}]"))?
    } else {
        authority.split(':').next()?.to_owned()
    };
    Some(host)
}

/// The path of a URL (no query, no fragment); `"/"` when there is none.
pub(crate) fn path(url: &str) -> &str {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let start = after_scheme.find('/').unwrap_or(after_scheme.len());
    let tail = &after_scheme[start..];
    let end = tail.find(['?', '#']).unwrap_or(tail.len());
    if end == 0 { "/" } else { &tail[..end] }
}

/// The decoded value of the first query parameter called `name`.
pub(crate) fn query_param(url: &str, name: &str) -> Option<String> {
    let query = without_fragment(url).split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (percent_decode(key) == name).then(|| percent_decode(value))
    })
}

/// A URL fit for an error message or a log line: no fragment, API keys masked, shortened.
pub(crate) fn redact(url: &str) -> String {
    let url = without_fragment(url);
    let Some((base, query)) = url.split_once('?') else { return shorten(url) };
    let masked: Vec<String> = query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((key, _))
                if matches!(
                    key.to_ascii_lowercase().as_str(),
                    "key" | "apikey" | "api_key" | "token" | "access_token" | "wskey"
                ) =>
            {
                format!("{key}=…")
            }
            _ => pair.to_owned(),
        })
        .collect();
    shorten(&format!("{base}?{}", masked.join("&")))
}

// ---------------------------------------------------------------------------------------
// HTTP status
// ---------------------------------------------------------------------------------------

/// What the status line says about a response that is not an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Found {
    /// 2xx: read the body.
    Yes,
    /// 404 or 410: the source does not know the ISBN.
    No,
}

fn retry_after(response: &HttpResponse) -> Option<u32> {
    response.header("retry-after").and_then(|v| v.trim().parse::<u32>().ok())
}

/// Classifies the HTTP status and enforces the body size limit.
pub(crate) fn check_http(
    provider: &'static str,
    response: &HttpResponse,
) -> Result<Found, CatalogError> {
    if response.body.len() > MAX_BODY {
        return Err(CatalogError::TooLarge {
            provider,
            bytes: response.body.len(),
            limit: MAX_BODY,
        });
    }
    let status = response.status;
    match status {
        200..=299 => Ok(Found::Yes),
        404 | 410 => Ok(Found::No),
        429 => Err(CatalogError::RateLimited {
            provider,
            status,
            retry_after_s: retry_after(response),
        }),
        401 | 403 => Err(CatalogError::Unauthorized {
            provider,
            status,
            message: body_excerpt(&response.body),
        }),
        500..=599 => Err(CatalogError::Unavailable {
            provider,
            status,
            retry_after_s: retry_after(response),
        }),
        _ => Err(CatalogError::Status { provider, status }),
    }
}

/// The start of a body as one line of text, or nothing for HTML pages.
fn body_excerpt(body: &[u8]) -> String {
    if matches!(sniff(body), Sniff::Html | Sniff::Empty) {
        return String::new();
    }
    let head = body.get(..body.len().min(400)).unwrap_or(body);
    shorten(&String::from_utf8_lossy(head))
}

// ---------------------------------------------------------------------------------------
// Bodies
// ---------------------------------------------------------------------------------------

/// What a body looks like from its first bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sniff {
    Empty,
    Html,
    Xml,
    Json,
    Other,
}

impl Sniff {
    /// Wording for [`CatalogError::UnexpectedContent::found`].
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Self::Empty => "an empty body",
            Self::Html => "HTML",
            Self::Xml => "XML",
            Self::Json => "JSON",
            Self::Other => "something else",
        }
    }
}

/// Guesses the format of a body. UTF-16 bodies are recognised by their byte-order mark.
pub(crate) fn sniff(body: &[u8]) -> Sniff {
    let head: String = if body.starts_with(&[0xFF, 0xFE]) || body.starts_with(&[0xFE, 0xFF]) {
        decode_text(body.get(..body.len().min(1024)).unwrap_or(body), None)
    } else {
        String::from_utf8_lossy(body.get(..body.len().min(600)).unwrap_or(body)).into_owned()
    };
    let head = head.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    if head.is_empty() {
        return Sniff::Empty;
    }
    let lower = head.to_ascii_lowercase();
    if lower.starts_with("<!doctype html") || lower.starts_with("<html") || lower.contains("<html")
    {
        return Sniff::Html;
    }
    match lower.as_bytes().first() {
        Some(b'<') => Sniff::Xml,
        Some(b'{' | b'[') => Sniff::Json,
        _ => Sniff::Other,
    }
}

/// Decodes a body to text.
///
/// UTF-8 (with or without BOM) and UTF-16 (with BOM) are decoded as such. Bytes that are
/// not valid UTF-8 are read as Windows-1252 — a superset of ISO-8859-1 for printable
/// characters — unless `declared` names UTF-8, in which case invalid sequences become
/// U+FFFD.
pub(crate) fn decode_text(body: &[u8], declared: Option<&str>) -> String {
    if let Some(rest) = body.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    for (bom, big_endian) in [([0xFF_u8, 0xFE], false), ([0xFE, 0xFF], true)] {
        if let Some(rest) = body.strip_prefix(&bom) {
            let units = rest.chunks_exact(2).map(|pair| {
                let bytes = [pair[0], pair[1]];
                if big_endian { u16::from_be_bytes(bytes) } else { u16::from_le_bytes(bytes) }
            });
            return char::decode_utf16(units).map(|r| r.unwrap_or('\u{FFFD}')).collect();
        }
    }
    match std::str::from_utf8(body) {
        Ok(text) => text.to_owned(),
        Err(_) => {
            let utf8_declared = declared.is_some_and(|d| {
                let d = d.to_ascii_lowercase();
                d.contains("utf-8") || d.contains("utf8")
            });
            if utf8_declared {
                String::from_utf8_lossy(body).into_owned()
            } else {
                body.iter().map(|b| cp1252_char(*b)).collect()
            }
        }
    }
}

fn cp1252_char(byte: u8) -> char {
    match byte {
        0x80..=0x9F => match CP1252_HIGH[usize::from(byte - 0x80)] {
            '\0' => char::from(byte),
            ch => ch,
        },
        _ => char::from(byte),
    }
}

/// The `charset` parameter of the `Content-Type` header.
pub(crate) fn declared_charset(response: &HttpResponse) -> Option<String> {
    let header = response.header("content-type")?.to_ascii_lowercase();
    let (_, charset) = header.split_once("charset=")?;
    Some(charset.trim_matches(['"', '\'', ' ', ';']).to_owned())
}

/// Parses a JSON body, reporting HTML pages and empty bodies as such.
pub(crate) fn parse_json(
    provider: &'static str,
    response: &HttpResponse,
) -> Result<Value, CatalogError> {
    match sniff(&response.body) {
        Sniff::Json | Sniff::Other => {}
        other => {
            return Err(CatalogError::UnexpectedContent {
                provider,
                expected: "JSON",
                found: other.describe(),
            });
        }
    }
    let text = decode_text(&response.body, declared_charset(response).as_deref());
    serde_json::from_str::<Value>(text.trim_start_matches('\u{FEFF}')).map_err(|e| {
        CatalogError::Malformed { provider, format: "JSON", detail: shorten(&e.to_string()) }
    })
}

// ---------------------------------------------------------------------------------------
// Tolerant JSON access
// ---------------------------------------------------------------------------------------

/// A non-empty string member, trimmed.
pub(crate) fn jstr<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}

/// An array member; any other type gives an empty slice.
pub(crate) fn jarr<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value.get(key).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// A value as a list: an array gives its items, `null` nothing, anything else itself.
pub(crate) fn jlist(value: Option<&Value>) -> Vec<&Value> {
    match value {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items.iter().collect(),
        Some(other) => vec![other],
    }
}

/// The text of a scalar, or of the first of the given members of an object.
pub(crate) fn jtext(value: &Value, members: &[&str]) -> Option<String> {
    match value {
        Value::String(s) => Some(s.trim().to_owned()).filter(|s| !s.is_empty()),
        Value::Number(n) => Some(n.to_string()),
        Value::Object(_) => members.iter().find_map(|m| {
            value.get(*m).and_then(|inner| match inner {
                Value::String(_) | Value::Number(_) => jtext(inner, &[]),
                _ => None,
            })
        }),
        _ => None,
    }
}

/// Texts of a member that may be a scalar, an object or an array of either.
pub(crate) fn jtexts(value: &Value, key: &str, members: &[&str]) -> Vec<String> {
    jlist(value.get(key)).into_iter().filter_map(|item| jtext(item, members)).collect()
}

/// A positive integer written as a JSON number or as a string of digits.
pub(crate) fn jnum(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(n) => n.as_u64().and_then(|n| u32::try_from(n).ok()),
        Value::String(s) => s.trim().parse::<u32>().ok(),
        _ => None,
    }
    .filter(|n| *n > 0)
}

// ---------------------------------------------------------------------------------------
// ISBNs
// ---------------------------------------------------------------------------------------

/// Every valid ISBN written in `text`, as ISBN-13: `"2-07-036002-4 (br.)"`,
/// `"ISBN 9782070360024"`, `"urn:isbn:…"`.
pub(crate) fn isbns_in(text: &str) -> Vec<Isbn13> {
    let text = text.trim();
    if text.is_empty() || text.len() > 400 {
        return Vec::new();
    }
    if let Ok(isbn) = Isbn13::parse(text) {
        return vec![isbn];
    }
    let mut found = ebc_isbn::extract_all(text);
    if found.is_empty()
        && let Some(first) = text.split([' ', '(', ';', ',']).find(|t| !t.is_empty())
        && let Ok(isbn) = Isbn13::parse(first)
    {
        found.push(isbn);
    }
    found
}

/// Adds `isbn` to `list` unless it is already there.
pub(crate) fn push_isbn(list: &mut Vec<Isbn13>, isbn: Isbn13) {
    if !list.contains(&isbn) {
        list.push(isbn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_are_percent_encoded() {
        assert_eq!(encode_component("a b\"é&=/"), "a%20b%22%C3%A9%26%3D%2F");
        assert_eq!(percent_decode("a%20b%22%C3%A9%26%3D%2F+c%zz"), "a b\"é&=/ c%zz");
        assert_eq!(
            url_with_query("https://x.example/api?fixed=1", &[("q", "isbn:1 2")]),
            "https://x.example/api?fixed=1&q=isbn%3A1%202"
        );
    }

    #[test]
    fn context_travels_in_the_fragment() {
        let url = with_context("https://x.example/a?b=c#old", "authors|1");
        assert_eq!(url, "https://x.example/a?b=c#ebc=authors%7C1");
        assert_eq!(context(&url).as_deref(), Some("authors|1"));
        assert_eq!(context("https://x.example/a#other"), None);
        assert_eq!(without_fragment(&url), "https://x.example/a?b=c");
    }

    #[test]
    fn origins_hosts_and_paths() {
        assert_eq!(origin("HTTPS://OpenLibrary.org/a").as_deref(), Some("https://openlibrary.org"));
        assert_eq!(origin("http://127.0.0.1:8080").as_deref(), Some("http://127.0.0.1:8080"));
        assert_eq!(origin("https://openlibrary.org@evil.example/x"), None);
        assert_eq!(origin("ftp://x.example/"), None);
        assert_eq!(origin("javascript:alert(1)"), None);
        assert_eq!(host("http://127.0.0.1:8080/x").as_deref(), Some("127.0.0.1"));
        assert_eq!(path("https://x.example/a/b.json?c#d"), "/a/b.json");
        assert_eq!(path("https://x.example"), "/");
        assert_eq!(
            query_param("https://x.example/?a=1&q=isbn%3A9#f", "q").as_deref(),
            Some("isbn:9")
        );
    }

    #[test]
    fn keys_are_masked() {
        assert_eq!(
            redact("https://x.example/v?q=isbn:1&key=SECRET#ebc=x"),
            "https://x.example/v?q=isbn:1&key=…"
        );
    }

    #[test]
    fn bodies_are_sniffed_and_decoded() {
        assert_eq!(sniff(b""), Sniff::Empty);
        assert_eq!(sniff(b"  \n"), Sniff::Empty);
        assert_eq!(sniff(b"\xEF\xBB\xBF{\"a\":1}"), Sniff::Json);
        assert_eq!(sniff(b"<?xml version=\"1.0\"?><a/>"), Sniff::Xml);
        assert_eq!(sniff(b"<!DOCTYPE HTML><html>"), Sniff::Html);
        assert_eq!(sniff(b"<?xml version=\"1.0\"?>\n<html xmlns=\"x\">"), Sniff::Html);
        assert_eq!(sniff(b"Too many requests"), Sniff::Other);
        assert_eq!(decode_text(b"L'\xC9tranger", None), "L'Étranger");
        assert_eq!(decode_text(b"\x93x\x94", Some("windows-1252")), "“x”");
        assert_eq!(decode_text(b"L'\xC9", Some("UTF-8")), "L'\u{FFFD}");
        assert_eq!(decode_text(b"\xFF\xFE{\x00}\x00", None), "{}");
        assert_eq!(sniff(b"\xFF\xFE{\x00}\x00"), Sniff::Json);
    }

    #[test]
    fn isbns_are_found_in_catalogue_strings() {
        let camus = Isbn13::parse("9782070360024").expect("valid");
        assert_eq!(isbns_in("2-07-036002-4 (br.)"), vec![camus]);
        assert_eq!(isbns_in("ISBN 9782070360024"), vec![camus]);
        assert_eq!(isbns_in("978-2-07-036002-4"), vec![camus]);
        assert_eq!(isbns_in("http://catalogue.bnf.fr/ark:/12148/cb351139480"), vec![]);
        assert!(isbns_in("").is_empty());
    }
}
