//! Error types: start-up errors ([`ServerError`]) and the JSON error envelope of the API
//! ([`ApiError`]).

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

/// Why the server could not be configured or started.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ServerError {
    /// An environment variable or the partners file is unusable.
    #[error("configuration: {0}")]
    Config(String),
    /// The data directory cannot be read or written.
    #[error("storage: {0}")]
    Storage(String),
    /// A lower-level I/O failure.
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),
    /// An outbound client (HTTP, SMTP) could not be built.
    #[error("client: {0}")]
    Client(String),
}

/// An API error: `{"error": {"code": "snake_case", "message": "…"}}` with a matching HTTP
/// status (`docs/api.md`, conventions).
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// HTTP status of the response.
    pub status: StatusCode,
    /// Stable machine-readable code.
    pub code: &'static str,
    /// Human-readable explanation (English; never contains secrets).
    pub message: String,
    /// Extra members merged into the `error` object (e.g. preflight `findings`).
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
    /// Value of the `Retry-After` header, in seconds.
    pub retry_after: Option<u64>,
}

impl ApiError {
    /// Builds an error.
    #[must_use]
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into(), extra: None, retry_after: None }
    }

    /// `400 Bad Request`.
    #[must_use]
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    /// `401 Unauthorized`.
    #[must_use]
    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message)
    }

    /// `404 Not Found` with the code `not_found`.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    /// `409 Conflict`.
    #[must_use]
    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    /// `422 Unprocessable Entity`.
    #[must_use]
    pub fn unprocessable(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    }

    /// `429 Too Many Requests` with a `Retry-After` header.
    #[must_use]
    pub fn too_many(message: impl Into<String>, retry_after_secs: u64) -> Self {
        let mut error = Self::new(StatusCode::TOO_MANY_REQUESTS, "rate_limited", message);
        error.retry_after = Some(retry_after_secs.max(1));
        error
    }

    /// `500 Internal Server Error`. The detail is logged, not sent.
    #[must_use]
    pub fn internal(detail: impl std::fmt::Display) -> Self {
        tracing::error!(%detail, "internal error");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", "internal server error")
    }

    /// Adds members to the `error` object.
    #[must_use]
    pub fn with_extra(mut self, key: &str, value: serde_json::Value) -> Self {
        self.extra.get_or_insert_with(serde_json::Map::new).insert(key.to_owned(), value);
        self
    }

    /// The JSON body of the error.
    #[must_use]
    pub fn body(&self) -> serde_json::Value {
        let mut error = self.extra.clone().unwrap_or_default();
        error.insert("code".to_owned(), self.code.into());
        error.insert("message".to_owned(), self.message.clone().into());
        serde_json::json!({ "error": error })
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.status.as_u16(), self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (self.status, axum::Json(self.body())).into_response();
        let headers = response.headers_mut();
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        if let Some(seconds) = self.retry_after {
            headers.insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_has_the_documented_envelope() {
        let error = ApiError::unprocessable("preflight_failed", "nope")
            .with_extra("findings", serde_json::json!([{"code": "X"}]));
        assert_eq!(
            error.body(),
            serde_json::json!({"error": {"code": "preflight_failed", "message": "nope", "findings": [{"code": "X"}]}})
        );
    }

    #[test]
    fn extra_members_cannot_replace_code_or_message() {
        let error = ApiError::bad_request("real", "msg").with_extra("code", "forged".into());
        assert_eq!(error.body()["error"]["code"], "real");
    }

    #[test]
    fn rate_limit_errors_carry_retry_after() {
        let response = ApiError::too_many("slow down", 0).into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "1");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}
