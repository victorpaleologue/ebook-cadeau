//! WebAssembly bindings exposing the eBook Cadeau pipeline to the browser.
//!
//! The crate has two layers:
//!
//! * [`api`] — all the logic, in plain Rust, unit-tested natively. Structured values
//!   cross the boundary as **JSON strings**, binary values as byte slices; errors are
//!   [`api::ApiError`]s with a stable code.
//! * the `#[wasm_bindgen]` shell in this file — one line per function: it forwards to
//!   [`api`] and turns an error into a JavaScript `Error` whose message is
//!   `"[code] text"`.
//!
//! Nothing here performs I/O, reads a clock or draws random numbers: the page does the
//! `fetch`, supplies `Date.now()` and `crypto.getRandomValues()`.
//!
//! # Exported functions
//!
//! | Function | Purpose |
//! |---|---|
//! | `version()` | crate version |
//! | `isbn_parse(input)` | validate an ISBN-13 or ISBN-10 |
//! | `catalog_providers()` | the catalogue sources and their terms |
//! | `catalog_lookup_requests(isbn, config)` | requests for the page to fetch |
//! | `catalog_parse(provider, isbn, config, request, response, body)` | read an answer |
//! | `catalog_merge(records)` | reconcile records |
//! | `cover_candidates(book, policy)` | where the cover can be *looked at* (never printable) |
//! | `cover_probe(bytes, format, profile)` | format, size, placeholder?, print quality |
//! | `scrape_extract(html, base_url)` | image URLs and ISBNs of a product page |
//! | `analyze_cover(bytes)` | palette, edges, a flat spine colour, text regions |
//! | `fontid_identify(cover, region, title, index)` | the closest open font |
//! | `gift_code_from_random(bytes)`, `gift_code_parse(input)`, `claim_urls(base, code)` | codes |
//! | `qr_svg(text)` | a QR code for the screen |
//! | `card_options()` | formats, profiles, rights declarations, fonts, locales |
//! | `render_card(request, cover, icc)` | the PDF and its warnings |
//! | `preview_card(request, cover)`, `preview_svg(request, cover, page)` | SVG previews |
//! | `i18n_catalog(locale)`, `i18n_t(locale, key)` | the page's strings |
//!
//! # Covers and rights
//!
//! A cover is printed only when the page hands over the bytes of a file the user picked
//! **and** the request names a rights basis other than `none`; see [`api::card`]. Covers
//! found by a lookup are for recognising the book and cannot reach a card: no function
//! takes a URL, and [`api::cover::cover_candidates`] returns no pixels.
//!
//! # Panics
//!
//! The release profile aborts on panic. [`start`] installs a hook that throws the panic
//! message as a JavaScript exception (`"panic: …"`) instead of an opaque `unreachable`
//! trap; the page must then discard the instance, whose memory may be inconsistent.
//!
//! # Features
//!
//! * `fontid` (default) — font identification through `ebc-fontid`'s bundled index.
//!   Without it `fontid_identify` answers `{"available": false}`.

#![forbid(unsafe_code)]

pub mod api;

use wasm_bindgen::prelude::*;

use api::ApiError;

/// Turns an API error into a JavaScript `Error` with the message `"[code] text"`.
fn js(error: ApiError) -> JsError {
    JsError::new(&error.to_string())
}

/// `None` for a missing or empty byte argument (`null`, `undefined`, an empty array).
fn non_empty(bytes: Option<Vec<u8>>) -> Option<Vec<u8>> {
    bytes.filter(|b| !b.is_empty())
}

/// Installs the panic hook. Runs when the module is instantiated.
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(|info| {
        wasm_bindgen::throw_str(&format!("panic: {info}"));
    }));
}

/// The version of this package.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    api::version()
}

/// See [`api::book::isbn_parse`].
///
/// # Errors
/// A JavaScript `Error` `"[invalid_isbn] …"`.
#[wasm_bindgen]
pub fn isbn_parse(input: &str) -> Result<String, JsError> {
    api::book::isbn_parse(input).map_err(js)
}

/// See [`api::book::catalog_providers`].
///
/// # Errors
/// None in practice.
#[wasm_bindgen]
pub fn catalog_providers() -> Result<String, JsError> {
    api::book::catalog_providers().map_err(js)
}

/// See [`api::book::catalog_lookup_requests`].
///
/// # Errors
/// `invalid_isbn`, `invalid_json`.
#[wasm_bindgen]
pub fn catalog_lookup_requests(isbn: &str, config_json: &str) -> Result<String, JsError> {
    api::book::catalog_lookup_requests(isbn, config_json).map_err(js)
}

/// See [`api::book::catalog_parse`].
///
/// # Errors
/// `invalid_isbn`, `invalid_json`, `unknown_provider`, `body_too_large`, `catalog_error`.
#[wasm_bindgen]
pub fn catalog_parse(
    provider: &str,
    isbn: &str,
    config_json: &str,
    request_json: &str,
    response_json: &str,
    body: &[u8],
) -> Result<String, JsError> {
    api::book::catalog_parse(provider, isbn, config_json, request_json, response_json, body)
        .map_err(js)
}

/// See [`api::book::catalog_merge`].
///
/// # Errors
/// `invalid_json`, `too_many_records`.
#[wasm_bindgen]
pub fn catalog_merge(records_json: &str) -> Result<String, JsError> {
    api::book::catalog_merge(records_json).map_err(js)
}

/// See [`api::cover::cover_candidates`].
///
/// # Errors
/// `invalid_json`.
#[wasm_bindgen]
pub fn cover_candidates(book_json: &str, policy_json: &str) -> Result<String, JsError> {
    api::cover::cover_candidates(book_json, policy_json).map_err(js)
}

/// See [`api::cover::cover_probe`].
///
/// # Errors
/// `not_an_image`, `unknown_format`, `unknown_profile`.
#[wasm_bindgen]
pub fn cover_probe(bytes: &[u8], format: &str, profile: &str) -> Result<String, JsError> {
    api::cover::cover_probe(bytes, format, profile).map_err(js)
}

/// See [`api::cover::scrape_extract`].
///
/// # Errors
/// None in practice.
#[wasm_bindgen]
pub fn scrape_extract(html: &str, base_url: &str) -> Result<String, JsError> {
    api::cover::scrape_extract(html, base_url).map_err(js)
}

/// See [`api::cover::analyze_cover`].
///
/// # Errors
/// `cover_decode`, `cover_too_large`.
#[wasm_bindgen]
pub fn analyze_cover(bytes: &[u8]) -> Result<String, JsError> {
    api::cover::analyze_cover(bytes).map_err(js)
}

/// See [`api::fontid::fontid_identify`]. `region_json` may be empty; `index` may be
/// `null` for the bundled index.
///
/// # Errors
/// `cover_decode`, `cover_too_large`, `invalid_json`, `invalid_font_index`.
#[wasm_bindgen]
pub fn fontid_identify(
    cover: &[u8],
    region_json: &str,
    title: &str,
    index: Option<Vec<u8>>,
) -> Result<String, JsError> {
    api::fontid::fontid_identify(cover, region_json, title, non_empty(index).as_deref())
        .map_err(js)
}

/// See [`api::book::gift_code_from_random`].
///
/// # Errors
/// `invalid_code`, `unfortunate_code`.
#[wasm_bindgen]
pub fn gift_code_from_random(bytes: &[u8]) -> Result<String, JsError> {
    api::book::gift_code_from_random(bytes).map_err(js)
}

/// See [`api::book::gift_code_parse`].
///
/// # Errors
/// `invalid_code`.
#[wasm_bindgen]
pub fn gift_code_parse(input: &str) -> Result<String, JsError> {
    api::book::gift_code_parse(input).map_err(js)
}

/// See [`api::book::claim_urls`].
///
/// # Errors
/// `invalid_code`, `invalid_base_url`.
#[wasm_bindgen]
pub fn claim_urls(base_url: &str, code: &str) -> Result<String, JsError> {
    api::book::claim_urls(base_url, code).map_err(js)
}

/// See [`api::book::qr_svg`].
///
/// # Errors
/// `qr_error`.
#[wasm_bindgen]
pub fn qr_svg(text: &str) -> Result<String, JsError> {
    api::book::qr_svg(text).map_err(js)
}

/// See [`api::card::card_options`].
///
/// # Errors
/// `internal`.
#[wasm_bindgen]
pub fn card_options() -> Result<String, JsError> {
    api::card::card_options().map_err(js)
}

/// A print file and its warnings, as returned by [`render_card`].
#[wasm_bindgen]
pub struct RenderedCard {
    inner: api::card::RenderedCard,
}

#[wasm_bindgen]
impl RenderedCard {
    /// The PDF, as a `Uint8Array` (a copy: the instance can be freed afterwards).
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn pdf(&self) -> Vec<u8> {
        self.inner.pdf.clone()
    }

    /// The warnings, as a JSON array of `{"code", "message", "params"}`.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn warnings(&self) -> String {
        self.inner.warnings_json.clone()
    }
}

/// See [`api::card::render_card`]. `cover` and `icc` may be `null`.
///
/// # Errors
/// See [`api::card::render_card`].
#[wasm_bindgen]
pub fn render_card(
    request_json: &str,
    cover: Option<Vec<u8>>,
    icc: Option<Vec<u8>>,
) -> Result<RenderedCard, JsError> {
    api::card::render_card(request_json, non_empty(cover), non_empty(icc).as_deref())
        .map(|inner| RenderedCard { inner })
        .map_err(js)
}

/// See [`api::card::preview_card`]. `cover` may be `null`.
///
/// # Errors
/// See [`api::card::render_card`].
#[wasm_bindgen]
pub fn preview_card(request_json: &str, cover: Option<Vec<u8>>) -> Result<String, JsError> {
    api::card::preview_card(request_json, non_empty(cover).as_deref()).map_err(js)
}

/// See [`api::card::preview_svg`]. `cover` may be `null`; `page` is 0 or 1.
///
/// # Errors
/// See [`api::card::render_card`]; `unknown_page`.
#[wasm_bindgen]
pub fn preview_svg(
    request_json: &str,
    cover: Option<Vec<u8>>,
    page: u32,
) -> Result<String, JsError> {
    api::card::preview_svg(request_json, non_empty(cover).as_deref(), page).map_err(js)
}

/// See [`api::ui::i18n_catalog`].
///
/// # Errors
/// `unknown_locale`.
#[wasm_bindgen]
pub fn i18n_catalog(locale: &str) -> Result<String, JsError> {
    api::ui::i18n_catalog(locale).map_err(js)
}

/// See [`api::ui::i18n_t`].
///
/// # Errors
/// `unknown_locale`, `unknown_key`.
#[wasm_bindgen]
pub fn i18n_t(locale: &str, key: &str) -> Result<String, JsError> {
    api::ui::i18n_t(locale, key).map_err(js)
}
