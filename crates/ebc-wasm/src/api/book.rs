//! ISBNs, the catalogue lookup (sans-IO: the page performs `fetch`), gift codes and QR
//! codes.

use std::collections::BTreeMap;

use ebc_catalog::{Config, Provider, ProviderInfo, is_denied_url, wire_url};
use ebc_model::{BookMeta, GiftCode, HttpRequest, HttpResponse, Isbn13, Method};
use ebc_qr::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};

use super::{ApiError, from_json, from_json_or_default, to_json};

// ---- ISBN ------------------------------------------------------------------------------

#[derive(Serialize)]
struct IsbnInfo {
    isbn13: String,
    hyphenated: String,
    isbn10: Option<String>,
    gs1_prefix: u16,
    group: Option<GroupInfo>,
    /// `"top_to_bottom"` or `"bottom_to_top"`, when the registration group tells.
    spine_direction: Option<&'static str>,
}

#[derive(Serialize)]
struct GroupInfo {
    prefix: &'static str,
    name: &'static str,
}

fn parse_isbn(input: &str) -> Result<Isbn13, ApiError> {
    // An ISBN is at most 17 characters with hyphens; anything much longer is not one,
    // and the bound keeps the error message short whatever was pasted.
    if input.len() > 64 {
        return Err(ApiError::new("invalid_isbn", "too long to be an ISBN"));
    }
    Isbn13::parse(input).map_err(|e| ApiError::new("invalid_isbn", e.to_string()))
}

/// Validates an ISBN-13 or ISBN-10 as typed (hyphens and spaces ignored).
///
/// Returns `{"isbn13", "hyphenated", "isbn10", "gs1_prefix", "group": {"prefix", "name"},
/// "spine_direction"}`; `isbn10` is `null` for 979 ISBNs, `group` and `spine_direction`
/// are `null` when the registration group is unknown.
///
/// # Errors
/// `invalid_isbn` with the reason (length, character, prefix, check digit).
pub fn isbn_parse(input: &str) -> Result<String, ApiError> {
    let isbn = parse_isbn(input)?;
    let mut probe = BookMeta::titled("");
    probe.isbn13 = Some(isbn);
    let group = isbn.registration_group();
    let info = IsbnInfo {
        isbn13: isbn.to_plain(),
        hyphenated: isbn.hyphenated(),
        isbn10: isbn.to_isbn10().ok().map(|i| i.to_plain()),
        gs1_prefix: isbn.gs1_prefix(),
        spine_direction: group
            .as_ref()
            .and_then(|g| direction_of_group(g.prefix, &probe))
            .map(ebc_i18n::SpineDirection::as_str),
        group: group.map(|g| GroupInfo { prefix: g.prefix, name: g.name }),
    };
    to_json(&info)
}

/// The spine direction the ISBN's registration group implies, if any: the two UI
/// locales with opposite conventions agree exactly when the ISBN decides.
fn direction_of_group(_prefix: &str, probe: &BookMeta) -> Option<ebc_i18n::SpineDirection> {
    let a = ebc_i18n::spine_direction_for_book(probe, ebc_i18n::Locale::Fr);
    let b = ebc_i18n::spine_direction_for_book(probe, ebc_i18n::Locale::En);
    (a == b).then_some(a)
}

// ---- Catalogue -------------------------------------------------------------------------

/// How the page wants the catalogue queried (all fields optional).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LookupConfig {
    /// Google Books API key; without one Google Books is not asked.
    google_books_key: Option<String>,
    /// Contact e-mail given to services that ask for one.
    contact_email: Option<String>,
    /// Base-URL overrides by provider id (tests, a relay of the deployment).
    base_urls: BTreeMap<String, String>,
    /// Providers to switch on although they are off by default.
    enable: Vec<String>,
    /// Providers to switch off.
    disable: Vec<String>,
    /// Also ask providers that are not known to allow cross-origin requests from a
    /// browser. Off by default: such requests fail in a page and only make sense when
    /// the caller is not a browser.
    include_non_cors: bool,
}

impl LookupConfig {
    fn to_config(&self) -> Config {
        let mut config = Config::new();
        if let Some(key) = &self.google_books_key {
            config = config.with_google_books_key(key.clone());
        }
        if let Some(email) = self.contact_email.as_ref().filter(|e| !e.trim().is_empty()) {
            config = config.with_contact_email(email.clone());
        }
        for (id, url) in &self.base_urls {
            if !url.trim().is_empty() {
                config = config.with_base_url(id.clone(), url.clone());
            }
        }
        for id in &self.enable {
            config = config.with_enabled(id.clone());
        }
        for id in &self.disable {
            config = config.with_disabled(id.clone());
        }
        config
    }

    /// A provider is asked from a page when it is known to send CORS headers, or when
    /// the deployment points it at its own server (a relay, a fixture server).
    fn usable_in_page(&self, info: &ProviderInfo) -> bool {
        self.include_non_cors
            || info.browser_cors
            || self.base_urls.get(info.id).is_some_and(|u| !u.trim().is_empty())
    }
}

#[derive(Serialize)]
struct ProviderDto {
    id: &'static str,
    name: &'static str,
    homepage: &'static str,
    data_licence: &'static str,
    terms_url: &'static str,
    requires_key: bool,
    browser_cors: bool,
    attribution: Option<&'static str>,
    min_interval_ms: u32,
    default_enabled: bool,
    hosts: &'static [&'static str],
    notes: &'static str,
}

impl ProviderDto {
    fn of(info: &ProviderInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            homepage: info.homepage,
            data_licence: info.data_licence,
            terms_url: info.terms_url,
            requires_key: info.requires_key,
            browser_cors: info.browser_cors,
            attribution: info.attribution,
            min_interval_ms: info.min_interval_ms,
            default_enabled: info.default_enabled,
            hosts: info.hosts,
            notes: info.notes,
        }
    }
}

/// Every catalogue provider with its licence and etiquette, as a JSON array of
/// `{"id", "name", "homepage", "data_licence", "terms_url", "requires_key",
/// "browser_cors", "attribution", "min_interval_ms", "default_enabled", "hosts",
/// "notes"}`.
///
/// # Errors
/// None in practice (see [`ApiError`]).
pub fn catalog_providers() -> Result<String, ApiError> {
    let list: Vec<ProviderDto> =
        ebc_catalog::provider_infos().iter().map(ProviderDto::of).collect();
    to_json(&list)
}

/// A request for the page to `fetch`.
#[derive(Serialize)]
struct RequestDto {
    /// The URL to fetch (without the private fragment).
    url: String,
    method: &'static str,
    /// Headers a page may set: `User-Agent` is left out (browsers refuse to set it).
    headers: Vec<(String, String)>,
    /// Hand this back, unchanged, to `catalog_parse` with the response.
    request: HttpRequest,
}

impl RequestDto {
    fn of(request: HttpRequest) -> Self {
        Self {
            url: wire_url(&request).to_owned(),
            method: request.method.as_str(),
            headers: request
                .headers
                .iter()
                .filter(|(name, _)| !name.eq_ignore_ascii_case("user-agent"))
                .cloned()
                .collect(),
            request,
        }
    }
}

#[derive(Serialize)]
struct ProviderRequests {
    provider: &'static str,
    name: &'static str,
    attribution: Option<&'static str>,
    min_interval_ms: u32,
    requests: Vec<RequestDto>,
}

/// The first requests to make for `isbn`, per provider, in priority order.
///
/// `config_json` is `{"google_books_key", "contact_email", "base_urls": {id: url},
/// "enable": [id], "disable": [id], "include_non_cors"}` (all optional; an empty string
/// is the default configuration). Only providers a page can call are returned: those
/// known to allow cross-origin requests, and those whose base URL is overridden.
///
/// Returns `[{"provider", "name", "attribution", "min_interval_ms", "requests": [{"url",
/// "method", "headers", "request"}]}]`. Fetch `url`, then pass `request` and the answer
/// to [`catalog_parse`].
///
/// # Errors
/// `invalid_isbn`, `invalid_json`.
pub fn catalog_lookup_requests(isbn: &str, config_json: &str) -> Result<String, ApiError> {
    let isbn = parse_isbn(isbn)?;
    let lookup: LookupConfig = from_json_or_default("config", config_json)?;
    let config = lookup.to_config();
    let list: Vec<ProviderRequests> = ebc_catalog::providers_for(&isbn, &config)
        .iter()
        .filter(|provider| lookup.usable_in_page(provider.info()))
        .map(|provider| {
            let info = provider.info();
            ProviderRequests {
                provider: info.id,
                name: info.name,
                attribution: info.attribution,
                min_interval_ms: info.min_interval_ms,
                requests: provider
                    .lookup(&isbn)
                    .into_iter()
                    .filter(|r| !is_denied_url(&r.url))
                    .map(RequestDto::of)
                    .collect(),
            }
        })
        .filter(|entry| !entry.requests.is_empty())
        .collect();
    to_json(&list)
}

/// What the page observed for a request: everything but the body.
#[derive(Debug, Deserialize)]
struct ResponseMeta {
    status: u16,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    final_url: String,
}

#[derive(Serialize)]
struct ParsedDto {
    records: Vec<BookMeta>,
    follow_up: Vec<RequestDto>,
    notes: Vec<String>,
}

/// `scheme://host[:port]` of a URL, lower-cased.
fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    (!authority.is_empty()).then(|| format!("{scheme}://{authority}").to_ascii_lowercase())
}

/// Follow-up requests may only go where the provider's first requests go (which covers
/// base-URL overrides) or to its declared hosts over HTTPS — never to a denied host.
fn follow_up_allowed(provider: &dyn Provider, isbn: &Isbn13, url: &str) -> bool {
    if is_denied_url(url) {
        return false;
    }
    let Some(origin) = origin_of(url) else { return false };
    let first: Vec<String> =
        provider.lookup(isbn).iter().filter_map(|r| origin_of(&r.url)).collect();
    if first.contains(&origin) {
        return true;
    }
    origin
        .strip_prefix("https://")
        .is_some_and(|host| provider.info().hosts.iter().any(|h| h.eq_ignore_ascii_case(host)))
}

/// Largest response body a provider is asked to read, in bytes.
pub const MAX_CATALOG_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Interprets the answer to a request issued by [`catalog_lookup_requests`] (or a
/// follow-up of an earlier call).
///
/// `request_json` is the `request` object handed out with the request; `response_json`
/// is `{"status", "headers": [[name, value]], "final_url"}`; `body` is the response body.
///
/// Returns `{"records": [BookMeta], "follow_up": [request…], "notes": [string]}`.
/// "Not found" is an empty result. Follow-up requests to hosts the provider does not
/// declare are dropped and mentioned in `notes`.
///
/// # Errors
/// `invalid_isbn`, `invalid_json`, `unknown_provider`, `body_too_large`, and
/// `catalog_error` when the response is an HTTP error or is not in the expected format.
pub fn catalog_parse(
    provider: &str,
    isbn: &str,
    config_json: &str,
    request_json: &str,
    response_json: &str,
    body: &[u8],
) -> Result<String, ApiError> {
    let isbn = parse_isbn(isbn)?;
    let lookup: LookupConfig = from_json_or_default("config", config_json)?;
    let request: HttpRequest = from_json("request", request_json)?;
    let meta: ResponseMeta = from_json("response", response_json)?;
    if body.len() > MAX_CATALOG_BODY_BYTES {
        return Err(ApiError::new(
            "body_too_large",
            format!("{} bytes, more than the {MAX_CATALOG_BODY_BYTES} accepted", body.len()),
        ));
    }
    let provider = ebc_catalog::provider_by_id(provider, &lookup.to_config())
        .ok_or_else(|| ApiError::new("unknown_provider", "no catalogue provider has this id"))?;
    let response = HttpResponse {
        status: meta.status,
        headers: meta.headers,
        body: body.to_vec(),
        final_url: if meta.final_url.is_empty() { request.url.clone() } else { meta.final_url },
    };
    let parsed = provider
        .parse(&isbn, &request, &response)
        .map_err(|e| ApiError::new("catalog_error", e.to_string()))?;
    let mut notes = parsed.notes;
    let mut follow_up = Vec::new();
    for next in parsed.follow_up {
        if next.method != Method::Post && follow_up_allowed(provider.as_ref(), &isbn, &next.url) {
            follow_up.push(RequestDto::of(next));
        } else {
            notes.push("a follow-up request to an undeclared host was dropped".to_owned());
        }
    }
    to_json(&ParsedDto { records: parsed.records, follow_up, notes })
}

/// Most records [`catalog_merge`] accepts.
pub const MAX_MERGE_RECORDS: usize = 256;

/// Reconciles records in priority order (the order the providers were asked) into one.
///
/// `records_json` is a JSON array of `BookMeta`; the result is the merged `BookMeta`, or
/// `null` when the list is empty or holds no title.
///
/// The description is dropped: catalogue descriptions are publishers' texts, and the
/// research asks that only facts be kept and printed.
///
/// # Errors
/// `invalid_json`, `too_many_records`.
pub fn catalog_merge(records_json: &str) -> Result<String, ApiError> {
    let records: Vec<BookMeta> = from_json("records", records_json)?;
    if records.len() > MAX_MERGE_RECORDS {
        return Err(ApiError::new("too_many_records", "more records than a lookup can produce"));
    }
    let merged = ebc_catalog::merge(&records).filter(|book| !book.title.trim().is_empty()).map(
        |mut book| {
            book.description = None;
            book
        },
    );
    to_json(&merged)
}

// ---- Gift codes ------------------------------------------------------------------------

#[derive(Serialize)]
struct CodeInfo {
    /// Twelve letters, e.g. `"BCDFGHJKLMNP"`.
    code: String,
    /// The printed form, `"BCDF-GHJK-LMNP"`.
    display: String,
}

impl CodeInfo {
    fn of(code: &GiftCode) -> Self {
        Self { code: code.compact(), display: code.grouped() }
    }
}

fn code_error(error: &ebc_model::CodeError) -> ApiError {
    ApiError::new("invalid_code", error.to_string())
}

/// Builds a gift code from random bytes the page drew with `crypto.getRandomValues`
/// (sixteen or more; at least eight). The same bytes give the same code.
///
/// A code that happens to spell an unfortunate abbreviation is refused, so that the
/// page draws again.
///
/// Returns `{"code", "display"}`.
///
/// # Errors
/// `invalid_code` (too few bytes), `unfortunate_code` (draw again).
pub fn gift_code_from_random(bytes: &[u8]) -> Result<String, ApiError> {
    let code = GiftCode::from_random_bytes(bytes).map_err(|e| code_error(&e))?;
    if code.has_unfortunate_letters() {
        return Err(ApiError::new("unfortunate_code", "draw new random bytes"));
    }
    to_json(&CodeInfo::of(&code))
}

/// Parses a code as typed or pasted — or a claim URL that carries one (`…/k/<code>`).
/// Case, hyphens, dots and spaces do not matter.
///
/// Returns `{"code", "display"}`. Success means *well-formed*, not *issued*.
///
/// # Errors
/// `invalid_code` with the reason (a character that is never part of a code, the
/// length, a wrong check letter).
pub fn gift_code_parse(input: &str) -> Result<String, ApiError> {
    if input.len() > 2048 {
        return Err(ApiError::new("invalid_code", "too long to be a code"));
    }
    let code = if input.contains("://") {
        GiftCode::from_claim_url(input.trim())
    } else {
        GiftCode::parse(input)
    }
    .map_err(|e| code_error(&e))?;
    to_json(&CodeInfo::of(&code))
}

#[derive(Serialize)]
struct ClaimInfo {
    code: String,
    display: String,
    /// Printed in clear: `https://ebk.example/k/BCDFGHJKLMNP`.
    claim_url: String,
    /// What the QR code carries (upper-cased when the base is a bare host).
    qr_payload: String,
    /// `true` when `qr_payload` is the upper-case form.
    qr_upper_case: bool,
}

/// The URLs of a gift: `{"code", "display", "claim_url", "qr_payload",
/// "qr_upper_case"}` for a base URL such as `"https://ebk.example"`.
///
/// # Errors
/// `invalid_code`; `invalid_base_url` when the base has no `http(s)` scheme or host.
pub fn claim_urls(base_url: &str, code: &str) -> Result<String, ApiError> {
    let code = GiftCode::parse(code).map_err(|e| code_error(&e))?;
    let claim_url = ebc_model::claim_url(base_url.trim(), &code);
    let (qr_payload, qr_upper_case) = ebc_card::qr_payload(&claim_url, &code)
        .map_err(|e| ApiError::new("invalid_base_url", e.to_string()))?;
    to_json(&ClaimInfo {
        code: code.compact(),
        display: code.grouped(),
        claim_url,
        qr_payload,
        qr_upper_case,
    })
}

// ---- QR --------------------------------------------------------------------------------

/// An SVG image of the QR code for `text`, at error-correction level M or better, with
/// the standard four-module quiet zone, for on-screen display (the card draws its own).
///
/// # Errors
/// `qr_error` when the text is empty or too long for a QR code.
pub fn qr_svg(text: &str) -> Result<String, ApiError> {
    let qr = QrCode::encode(text, EcLevel::M).map_err(|e| ApiError::new("qr_error", e.to_string()))?;
    Ok(qr.to_svg(8, ebc_qr::QUIET_ZONE_MODULES))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn value(json: &str) -> Value {
        serde_json::from_str(json).expect("the API returns JSON")
    }

    #[test]
    fn isbn_parse_accepts_both_forms_and_describes_the_group() {
        let v = value(&isbn_parse("978-2-07-036002-4").expect("valid"));
        assert_eq!(v["isbn13"], "9782070360024");
        assert_eq!(v["hyphenated"], "978-2-07-036002-4");
        assert_eq!(v["isbn10"], "2070360024");
        assert_eq!(v["gs1_prefix"], 978);
        assert_eq!(v["group"]["prefix"], "978-2");
        assert_eq!(v["spine_direction"], "bottom_to_top");

        let v = value(&isbn_parse("0-306-40615-2").expect("an ISBN-10"));
        assert_eq!(v["isbn13"], "9780306406157");
        assert_eq!(v["spine_direction"], "top_to_bottom");

        let v = value(&isbn_parse("979-10-90636-07-1").expect("a 979 ISBN"));
        assert_eq!(v["isbn10"], Value::Null);
    }

    #[test]
    fn isbn_parse_refuses_hostile_input() {
        for bad in [
            "",
            "9782070360025",
            "978-2-07-036002-4<script>",
            "９７８２０７０３６００２４",
            "\0\0\0",
            "1234567890123",
        ] {
            assert_eq!(isbn_parse(bad).expect_err(bad).code, "invalid_isbn", "{bad:?}");
        }
        let long = "9".repeat(100_000);
        let err = isbn_parse(&long).expect_err("too long");
        assert_eq!(err.code, "invalid_isbn");
        assert!(err.message.len() < 100);
    }

    #[test]
    fn providers_are_listed_with_their_terms() {
        let v = value(&catalog_providers().expect("list"));
        let list = v.as_array().expect("an array");
        assert_eq!(list.len(), ebc_catalog::PROVIDER_IDS.len());
        let ol = list.iter().find(|p| p["id"] == "openlibrary").expect("Open Library");
        assert_eq!(ol["browser_cors"], true);
        assert!(ol["terms_url"].as_str().expect("a URL").starts_with("https://"));
        let bnf = list.iter().find(|p| p["id"] == "bnf").expect("BnF");
        assert_eq!(bnf["browser_cors"], false);
    }

    #[test]
    fn only_providers_a_page_can_call_are_asked_by_default() {
        let v = value(&catalog_lookup_requests("9782070360024", "").expect("requests"));
        let ids: Vec<&str> =
            v.as_array().expect("array").iter().filter_map(|p| p["provider"].as_str()).collect();
        assert_eq!(ids, ["openlibrary"], "the BnF sends no CORS headers");
        let request = &v[0]["requests"][0];
        assert_eq!(request["method"], "GET");
        let url = request["url"].as_str().expect("url");
        assert!(url.starts_with("https://openlibrary.org/") && !url.contains('#'));
        // The page cannot set User-Agent; the request to hand back is intact.
        assert!(
            request["headers"]
                .as_array()
                .expect("headers")
                .iter()
                .all(|h| !h[0].as_str().unwrap_or("").eq_ignore_ascii_case("user-agent"))
        );
        assert!(request["request"]["url"].as_str().expect("url").starts_with(url));

        let all = value(
            &catalog_lookup_requests("9782070360024", r#"{"include_non_cors": true}"#)
                .expect("requests"),
        );
        assert_eq!(all[0]["provider"], "bnf");
    }

    #[test]
    fn base_url_overrides_enable_a_provider_and_move_its_requests() {
        let config = json!({"base_urls": {"bnf": "http://127.0.0.1:9000/sru", "openlibrary": "http://127.0.0.1:9000"}});
        let v = value(&catalog_lookup_requests("9782070360024", &config.to_string()).expect("ok"));
        let list = v.as_array().expect("array");
        assert_eq!(list[0]["provider"], "bnf");
        assert!(list[0]["requests"][0]["url"].as_str().expect("url").starts_with("http://127.0.0.1:9000/sru"));
        assert!(list[1]["requests"][0]["url"].as_str().expect("url").starts_with("http://127.0.0.1:9000/"));
    }

    #[test]
    fn a_lookup_round_trip_through_open_library() {
        let requests = value(&catalog_lookup_requests("9782070360024", "").expect("requests"));
        let request = requests[0]["requests"][0]["request"].to_string();
        let body = json!({
            "ISBN:9782070360024": {
                "details": {
                    "title": "L'Étranger",
                    "publishers": ["Gallimard"],
                    "authors": [{"key": "/authors/OL1A", "name": "Albert Camus"}],
                    "description": "A publisher's blurb."
                }
            }
        })
        .to_string();
        let response = json!({"status": 200, "headers": [["content-type", "application/json"]]});
        let parsed = value(
            &catalog_parse("openlibrary", "9782070360024", "", &request, &response.to_string(), body.as_bytes())
                .expect("parsed"),
        );
        let records = parsed["records"].as_array().expect("records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["isbn13"], "9782070360024");
        assert_eq!(records[0]["contributors"][0]["name"], "Albert Camus");

        let merged = value(&catalog_merge(&parsed["records"].to_string()).expect("merged"));
        assert_eq!(merged["publisher"], "Gallimard");
        assert_eq!(merged["description"], Value::Null, "descriptions are not kept");
        assert_eq!(merged["sources"][0]["provider"], "openlibrary");

        // Not found: an empty object.
        let none = value(
            &catalog_parse("openlibrary", "9782070360024", "", &request, &response.to_string(), b"{}")
                .expect("parsed"),
        );
        assert_eq!(none["records"].as_array().map(Vec::len), Some(0));
        assert_eq!(catalog_merge("[]").expect("merged"), "null");
    }

    #[test]
    fn catalog_parse_survives_hostile_answers() {
        let requests = value(&catalog_lookup_requests("9782070360024", "").expect("requests"));
        let request = requests[0]["requests"][0]["request"].to_string();
        let ok = r#"{"status": 200}"#;
        for body in [&b""[..], b"<html>", b"\xff\xfe\x00", b"[[[[[[[[", b"{\"ISBN:9782070360024\": 3}"] {
            // Whatever comes back is an error or an empty result — never a panic.
            if let Ok(json) = catalog_parse("openlibrary", "9782070360024", "", &request, ok, body) {
                assert!(value(&json)["records"].is_array());
            }
        }
        let err = catalog_parse("openlibrary", "9782070360024", "", &request, r#"{"status": 503}"#, b"")
            .expect_err("HTTP error");
        assert_eq!(err.code, "catalog_error");
        assert_eq!(
            catalog_parse("amazon", "9782070360024", "", &request, ok, b"{}").expect_err("unknown").code,
            "unknown_provider"
        );
        assert_eq!(
            catalog_parse("openlibrary", "9782070360024", "", "{", ok, b"{}").expect_err("bad").code,
            "invalid_json"
        );
        let big = vec![b' '; MAX_CATALOG_BODY_BYTES + 1];
        assert_eq!(
            catalog_parse("openlibrary", "9782070360024", "", &request, ok, &big).expect_err("big").code,
            "body_too_large"
        );
    }

    #[test]
    fn follow_ups_stay_on_the_providers_hosts() {
        let isbn = Isbn13::parse("9782070360024").expect("valid");
        let provider = ebc_catalog::provider_by_id("openlibrary", &Config::new()).expect("exists");
        let p = provider.as_ref();
        assert!(follow_up_allowed(p, &isbn, "https://openlibrary.org/authors/OL1A.json"));
        assert!(!follow_up_allowed(p, &isbn, "http://openlibrary.org.evil.example/x"));
        assert!(!follow_up_allowed(p, &isbn, "https://www.amazon.fr/dp/2070360024"));
        assert!(!follow_up_allowed(p, &isbn, "https://127.0.0.1/x"));
        assert!(!follow_up_allowed(p, &isbn, "javascript:alert(1)"));
        assert_eq!(origin_of("HTTPS://User@Example.org:8443/a?b#c").as_deref(), Some("https://example.org:8443"));
        assert_eq!(origin_of("no scheme"), None);
    }

    #[test]
    fn gift_codes_round_trip() {
        let v = value(&gift_code_from_random(&[7u8; 16]).expect("a code"));
        let code = v["code"].as_str().expect("code");
        let display = v["display"].as_str().expect("display");
        assert_eq!(code.len(), 12);
        assert!(code.chars().all(|c| GiftCode::ALPHABET.contains(c)));
        assert_eq!(display.replace('-', ""), code);
        // Deterministic, lenient on input, and found in a claim URL of either case.
        assert_eq!(value(&gift_code_from_random(&[7u8; 16]).expect("again"))["code"], code);
        assert_eq!(value(&gift_code_parse(&display.to_lowercase()).expect("typed"))["code"], code);
        let url = format!("HTTPS://EBK.EXAMPLE/K/{code}");
        assert_eq!(value(&gift_code_parse(&url).expect("scanned"))["display"], display);
    }

    #[test]
    fn gift_code_errors_are_typed() {
        assert_eq!(gift_code_from_random(&[1, 2, 3]).expect_err("too few").code, "invalid_code");
        for bad in ["", "BCDF-GHJK-LMN", "BCDF-GHJK-LMNA", "0000-0000-0000", "https://ebk.example/x", "\u{202e}"] {
            assert_eq!(gift_code_parse(bad).expect_err(bad).code, "invalid_code", "{bad:?}");
        }
        assert_eq!(gift_code_parse(&"B".repeat(5000)).expect_err("long").code, "invalid_code");
        // A single wrong letter is always caught by the check letter.
        let good = GiftCode::from_payload(123_456_789).expect("in range").compact();
        let typo = good.replacen(&good[0..1], if &good[0..1] == "B" { "C" } else { "B" }, 1);
        assert_eq!(gift_code_parse(&typo).expect_err("typo").code, "invalid_code");
    }

    #[test]
    fn unfortunate_codes_are_redrawn() {
        // Find random bytes that give a flagged code: about 1.4 % of codes are.
        let mut flagged = 0;
        for seed in 0u32..2000 {
            let mut bytes = [0u8; 16];
            for (i, b) in bytes.iter_mut().enumerate() {
                *b = (seed.wrapping_mul(2_654_435_761).rotate_left(i as u32 * 5) >> 8) as u8;
            }
            match gift_code_from_random(&bytes) {
                Ok(json) => {
                    let code = value(&json)["code"].as_str().expect("code").to_owned();
                    assert!(!GiftCode::parse(&code).expect("valid").has_unfortunate_letters());
                }
                Err(e) => {
                    assert_eq!(e.code, "unfortunate_code");
                    flagged += 1;
                }
            }
        }
        assert!(flagged > 0, "the sample should contain flagged codes");
    }

    #[test]
    fn claim_urls_upper_case_the_qr_payload_for_a_bare_host() {
        let code = GiftCode::from_payload(42).expect("in range").compact();
        let v = value(&claim_urls("https://ebk.example/", &code).expect("urls"));
        assert_eq!(v["claim_url"], format!("https://ebk.example/k/{code}"));
        assert_eq!(v["qr_payload"], format!("HTTPS://EBK.EXAMPLE/K/{code}"));
        assert_eq!(v["qr_upper_case"], true);
        assert_eq!(claim_urls("ftp://ebk.example", &code).expect_err("scheme").code, "invalid_base_url");
        assert_eq!(claim_urls("", &code).expect_err("empty").code, "invalid_base_url");
        assert_eq!(claim_urls("https://ebk.example", "nope").expect_err("code").code, "invalid_code");
    }

    #[test]
    fn qr_svg_is_a_standalone_image() {
        let svg = qr_svg("HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNP").expect("svg");
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("<path d=\"M"));
        assert!(!svg.contains("<script"));
        assert_eq!(qr_svg("").expect_err("empty").code, "qr_error");
        assert_eq!(qr_svg(&"x".repeat(10_000)).expect_err("too long").code, "qr_error");
    }
}
