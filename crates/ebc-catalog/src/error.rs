//! Errors a provider can report.

use thiserror::Error;

/// Why a response could not be turned into records.
///
/// "Not found" is not an error: it is an empty [`Parsed`](crate::Parsed).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CatalogError {
    /// HTTP 429, or a quota error reported in the body. Retry later.
    #[error("{provider}: rate limited (HTTP {status})")]
    RateLimited {
        /// Provider identifier.
        provider: &'static str,
        /// HTTP status.
        status: u16,
        /// Seconds to wait, from a numeric `Retry-After` header.
        retry_after_s: Option<u32>,
    },
    /// HTTP 5xx. The service is down or overloaded; retry later.
    #[error("{provider}: service unavailable (HTTP {status})")]
    Unavailable {
        /// Provider identifier.
        provider: &'static str,
        /// HTTP status.
        status: u16,
        /// Seconds to wait, from a numeric `Retry-After` header.
        retry_after_s: Option<u32>,
    },
    /// HTTP 401 or 403, or an invalid API key reported in the body.
    #[error("{provider}: access refused (HTTP {status}): {message}")]
    Unauthorized {
        /// Provider identifier.
        provider: &'static str,
        /// HTTP status.
        status: u16,
        /// What the server said, shortened.
        message: String,
    },
    /// Any other HTTP status that is neither success nor "not found".
    #[error("{provider}: unexpected HTTP status {status}")]
    Status {
        /// Provider identifier.
        provider: &'static str,
        /// HTTP status.
        status: u16,
    },
    /// The body is not in the expected format at all, e.g. an HTML error page where JSON
    /// was expected.
    #[error("{provider}: expected {expected} but received {found}")]
    UnexpectedContent {
        /// Provider identifier.
        provider: &'static str,
        /// Format the provider asked for (`"JSON"`, `"XML"`).
        expected: &'static str,
        /// What the body looks like (`"HTML"`, `"an empty body"`…).
        found: &'static str,
    },
    /// The body is in the expected format but broken or truncated, or lacks the structure
    /// the API documents.
    #[error("{provider}: malformed {format}: {detail}")]
    Malformed {
        /// Provider identifier.
        provider: &'static str,
        /// `"JSON"` or `"XML"`.
        format: &'static str,
        /// Parser message, shortened.
        detail: String,
    },
    /// The service answered with its own error report (SRU diagnostic, JSON `error`
    /// object).
    #[error("{provider}: the service reported an error: {message}")]
    Remote {
        /// Provider identifier.
        provider: &'static str,
        /// What the server said, shortened.
        message: String,
    },
    /// The body is larger than a catalogue answer can reasonably be; it was not parsed.
    #[error("{provider}: response of {bytes} bytes exceeds the {limit}-byte limit")]
    TooLarge {
        /// Provider identifier.
        provider: &'static str,
        /// Size of the body.
        bytes: usize,
        /// The limit.
        limit: usize,
    },
    /// `parse` was given a request this provider would never issue.
    #[error("{provider}: not a request of this provider: {url}")]
    UnknownRequest {
        /// Provider identifier.
        provider: &'static str,
        /// The URL of the request, shortened.
        url: String,
    },
    /// A follow-up request pointed outside the provider's own hosts and was not issued.
    #[error("{provider}: refused a follow-up request to {url}")]
    ForbiddenFollowUp {
        /// Provider identifier.
        provider: &'static str,
        /// The URL of the request, shortened.
        url: String,
    },
}

impl CatalogError {
    /// The identifier of the provider that reported the error.
    #[must_use]
    pub fn provider(&self) -> &'static str {
        match self {
            Self::RateLimited { provider, .. }
            | Self::Unavailable { provider, .. }
            | Self::Unauthorized { provider, .. }
            | Self::Status { provider, .. }
            | Self::UnexpectedContent { provider, .. }
            | Self::Malformed { provider, .. }
            | Self::Remote { provider, .. }
            | Self::TooLarge { provider, .. }
            | Self::UnknownRequest { provider, .. }
            | Self::ForbiddenFollowUp { provider, .. } => provider,
        }
    }

    /// `true` when trying again later can succeed (rate limit, outage).
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Unavailable { .. })
    }

    /// Seconds the server asked to wait before retrying, if it said.
    #[must_use]
    pub fn retry_after_s(&self) -> Option<u32> {
        match self {
            Self::RateLimited { retry_after_s, .. } | Self::Unavailable { retry_after_s, .. } => {
                *retry_after_s
            }
            _ => None,
        }
    }
}

/// Shortens server-supplied text for error messages: one line, at most 200 characters.
pub(crate) fn shorten(text: &str) -> String {
    let mut out = String::new();
    let mut last_space = true;
    for ch in text.chars() {
        let ch = if ch.is_whitespace() || ch.is_control() { ' ' } else { ch };
        if ch == ' ' && last_space {
            continue;
        }
        last_space = ch == ' ';
        out.push(ch);
        if out.chars().count() >= 200 {
            out.push('…');
            break;
        }
    }
    out.trim().to_owned()
}
