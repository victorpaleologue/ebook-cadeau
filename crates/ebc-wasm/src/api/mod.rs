//! The whole browser-facing logic, as plain Rust.
//!
//! Every function takes and returns strings (JSON for structured values) and byte
//! slices, so that the `#[wasm_bindgen]` layer is a thin shell and everything can be
//! tested natively. Nothing here performs I/O, reads a clock or draws random numbers:
//! the page fetches, the page supplies the creation time and the random bytes.
//!
//! * [`book`] — ISBNs, the sans-IO catalogue lookup, gift codes, QR codes;
//! * [`cover`] — cover candidates (preview only), image probing, page scraping, colour
//!   analysis;
//! * [`card`] — the card request, the PDF and the SVG preview, with the rights gate;
//! * [`fontid`] — the closest open font for the lettering of a cover;
//! * [`ui`] — the strings of the web app, in seven languages.
//!
//! # Errors
//!
//! Every fallible function returns an [`ApiError`]: a stable machine-readable
//! [`code`](ApiError::code) the page can translate, and an English message.

pub mod book;
pub mod card;
pub mod cover;
pub mod fontid;
pub mod ui;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// An error of the browser API.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("[{code}] {message}")]
pub struct ApiError {
    /// Stable identifier in `snake_case`, e.g. `"invalid_isbn"`, `"rights_route_refused"`.
    pub code: &'static str,
    /// What went wrong, in English, for logs and as a fallback text.
    pub message: String,
}

impl ApiError {
    /// An error with `code` and a message.
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

/// Largest JSON argument accepted, in bytes. Requests are a few kilobytes; records to
/// merge a few hundred. The cap keeps a hostile page from making the parser allocate
/// without bound.
pub const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;

/// Parses a JSON argument called `what`.
pub(crate) fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: &str,
) -> Result<T, ApiError> {
    if json.len() > MAX_JSON_BYTES {
        return Err(ApiError::new(
            "invalid_json",
            format!(
                "{what}: {} bytes of JSON, more than the {MAX_JSON_BYTES} accepted",
                json.len()
            ),
        ));
    }
    serde_json::from_str(json).map_err(|e| ApiError::new("invalid_json", format!("{what}: {e}")))
}

/// Parses an optional JSON argument: an empty or blank string, or `null`, is the default.
pub(crate) fn from_json_or_default<T: DeserializeOwned + Default>(
    what: &'static str,
    json: &str,
) -> Result<T, ApiError> {
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(T::default());
    }
    from_json(what, json)
}

/// Serialises a result. The values serialised here are plain data (strings, numbers,
/// lists), for which `serde_json` cannot fail; the error path only keeps the function
/// total.
pub(crate) fn to_json<T: Serialize>(value: &T) -> Result<String, ApiError> {
    serde_json::to_string(value).map_err(|e| ApiError::new("internal", e.to_string()))
}

/// The version of this crate, e.g. `"0.1.0"`.
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_the_package_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert!(version().split('.').count() >= 3);
    }

    #[test]
    fn json_arguments_are_bounded_and_errors_are_typed() {
        let err = from_json::<Vec<u8>>("list", "{").expect_err("truncated JSON");
        assert_eq!(err.code, "invalid_json");
        assert!(err.to_string().starts_with("[invalid_json] list:"));
        let huge = format!("[{}0]", "0,".repeat(MAX_JSON_BYTES / 2 + 1));
        assert_eq!(
            from_json::<Vec<u8>>("list", &huge).expect_err("too large").code,
            "invalid_json"
        );
        assert_eq!(
            from_json_or_default::<Vec<u8>>("list", "  ").expect("default"),
            Vec::<u8>::new()
        );
        assert_eq!(
            from_json_or_default::<Vec<u8>>("list", "null").expect("default"),
            Vec::<u8>::new()
        );
        assert_eq!(from_json_or_default::<Vec<u8>>("list", "[1]").expect("parsed"), vec![1]);
    }
}
