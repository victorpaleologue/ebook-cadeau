//! Sans-IO HTTP messages.
//!
//! Catalogue and cover providers never perform network I/O themselves: they *describe* the
//! request to make ([`HttpRequest`]) and *interpret* the response ([`HttpResponse`]). The
//! caller executes the request with whatever transport it has (`reqwest` on a server,
//! `fetch` in a browser, a fixture file in tests).

use serde::{Deserialize, Serialize};

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    /// `GET`
    #[default]
    Get,
    /// `HEAD`
    Head,
    /// `POST`
    Post,
}

impl Method {
    /// The method name as sent on the wire.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
        }
    }
}

/// A request for the caller to execute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Method.
    pub method: Method,
    /// Absolute URL.
    pub url: String,
    /// Header name/value pairs. Names are case-insensitive.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// Optional request body.
    #[serde(default)]
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// A `GET` request without headers.
    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self { method: Method::Get, url: url.into(), headers: Vec::new(), body: None }
    }

    /// Adds a header and returns the request.
    #[must_use]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

/// The response to an [`HttpRequest`], as observed by the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpResponse {
    /// Status code.
    pub status: u16,
    /// Header name/value pairs. Names are case-insensitive.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// Response body.
    #[serde(default)]
    pub body: Vec<u8>,
    /// The URL the body was finally served from (after redirects).
    pub final_url: String,
}

impl HttpResponse {
    /// Returns `true` for 2xx statuses.
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The first header with the given name (case-insensitive).
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    /// The media type from `Content-Type`, lower-cased, without parameters.
    #[must_use]
    pub fn content_type(&self) -> Option<String> {
        self.header("content-type")
            .map(|v| v.split(';').next().unwrap_or("").trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
    }

    /// The body decoded as UTF-8, replacing invalid sequences.
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_helpers() {
        let r = HttpResponse {
            status: 200,
            headers: vec![("Content-Type".into(), "Application/JSON; charset=utf-8".into())],
            body: b"{}".to_vec(),
            final_url: "https://example.org/x".into(),
        };
        assert!(r.is_success());
        assert_eq!(r.header("content-type"), Some("Application/JSON; charset=utf-8"));
        assert_eq!(r.content_type().as_deref(), Some("application/json"));
        assert_eq!(r.text(), "{}");
    }

    #[test]
    fn request_builder() {
        let r = HttpRequest::get("https://example.org").with_header("Accept", "application/json");
        assert_eq!(r.method.as_str(), "GET");
        assert_eq!(r.headers.len(), 1);
    }
}
