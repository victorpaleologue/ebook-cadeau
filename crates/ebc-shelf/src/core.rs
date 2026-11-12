//! The client of the core's partner API (`docs/api.md` §4): every request is signed
//! (§4.1), creations carry an `Idempotency-Key`, nothing follows redirects, every answer
//! is read with a size cap. Signed URLs are never logged.

use std::time::Duration;

use reqwest::{Method, StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::crypto;

/// Largest JSON answer read from the core.
const MAX_JSON_BYTES: usize = 1 << 20;
/// Largest card PDF read from the core.
const MAX_PDF_BYTES: usize = 64 << 20;

/// A failed call to the core.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CoreError {
    /// The core could not be reached.
    #[error("the core is unreachable: {0}")]
    Transport(String),
    /// The core refused.
    #[error("the core answered {status} ({code})")]
    Api {
        /// HTTP status.
        status: u16,
        /// `error.code`, or `http_<status>`.
        code: String,
    },
    /// The core answered something unusable.
    #[error("unexpected answer from the core: {0}")]
    Decode(String),
}

/// `book` of a gift creation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BookPayload {
    /// Title.
    pub title: String,
    /// ISBN-13 (digits), when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<String>,
    /// The author.
    pub contributors: Vec<Contributor>,
    /// Language, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// One contributor.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Contributor {
    /// Display name.
    pub name: String,
}

/// `delivery` of a gift creation: always `partner_file` for a shelf.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Delivery {
    /// `partner_file`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// `epub` or `pdf`.
    pub format: &'static str,
    /// The shelf's §4.5 endpoint.
    pub url: String,
}

/// Body of `POST /api/partner/v1/gifts`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewGift {
    /// The shelf's gift id.
    pub order_ref: String,
    /// The book.
    pub book: BookPayload,
    /// Giver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub giver_name: Option<String>,
    /// Recipient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_name: Option<String>,
    /// Message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Locale.
    pub locale: String,
    /// Reveal date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reveal_at: Option<i64>,
    /// `partner_file`.
    pub delivery: Delivery,
}

/// The core's answer to a creation or a reissue.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct IssuedCode {
    /// Gift id (creation only).
    #[serde(default)]
    pub id: Option<String>,
    /// `XXXX-XXXX-XXXX`.
    pub code_display: String,
    /// Claim URL.
    pub claim_url: String,
}

/// `GET /api/partner/v1/gifts/{id}`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CoreGiftStatus {
    /// `created`, `redeemed`, `exhausted`, `cancelled`, `expired`…
    pub status: String,
    /// Claims so far.
    #[serde(default)]
    pub downloads: Option<u64>,
}

/// A card PDF and the composer's warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// The PDF.
    pub pdf: Vec<u8>,
    /// `X-EBC-Card-Warnings`, when present.
    pub warnings: Option<String>,
}

/// The partner-API client.
#[derive(Debug, Clone)]
pub struct CoreClient {
    http: reqwest::Client,
    base: Url,
    partner_id: String,
    secret: String,
}

impl CoreClient {
    /// A client of the core at `base` (e.g. `https://ebk.example`).
    ///
    /// # Errors
    /// [`CoreError::Decode`] for an unusable URL, [`CoreError::Transport`] when the HTTP
    /// client cannot be built.
    pub fn new(
        base: &str,
        partner_id: &str,
        secret: &str,
        timeout: Duration,
    ) -> Result<Self, CoreError> {
        // reqwest is built without a bundled TLS provider: install ring once per process.
        // An error only means another component already installed one.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout.min(Duration::from_secs(5)))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("ebc-shelf/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| CoreError::Transport(e.to_string()))?;
        let base = Url::parse(base).map_err(|e| CoreError::Decode(e.to_string()))?;
        Ok(Self { http, base, partner_id: partner_id.to_owned(), secret: secret.to_owned() })
    }

    async fn send(
        &self,
        method: Method,
        path_and_query: &str,
        body: Vec<u8>,
        idempotency_key: Option<&str>,
        now: i64,
    ) -> Result<reqwest::Response, CoreError> {
        let joined = format!("{}{path_and_query}", self.base.as_str().trim_end_matches('/'));
        let url = Url::parse(&joined).map_err(|e| CoreError::Decode(e.to_string()))?;
        // Sign exactly what goes on the wire.
        let wire = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_owned(),
        };
        let signature = crypto::request_signature(&self.secret, now, method.as_str(), &wire, &body);
        let mut request = self
            .http
            .request(method, url)
            .header("X-EBC-Partner", &self.partner_id)
            .header("X-EBC-Timestamp", now.to_string())
            .header("X-EBC-Signature", signature);
        if !body.is_empty() {
            request = request.header(reqwest::header::CONTENT_TYPE, "application/json");
        }
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        // The error's own text may contain the URL: keep only its kind.
        request.body(body).send().await.map_err(|e| CoreError::Transport(describe(&e)))
    }

    /// `POST /api/partner/v1/gifts`; `idempotency_key` is the shelf's gift id, so a
    /// retry after a lost answer gets the same gift.
    ///
    /// # Errors
    /// See [`CoreError`].
    pub async fn create_gift(&self, gift: &NewGift, now: i64) -> Result<IssuedCode, CoreError> {
        let body = serde_json::to_vec(gift).map_err(|e| CoreError::Decode(e.to_string()))?;
        let response = self
            .send(Method::POST, "/api/partner/v1/gifts", body, Some(&gift.order_ref), now)
            .await?;
        let issued: IssuedCode = json(response).await?;
        if !issued.id.as_deref().is_some_and(is_safe_id) || issued.claim_url.is_empty() {
            return Err(CoreError::Decode("gift without a usable id".to_owned()));
        }
        Ok(issued)
    }

    /// `POST /api/partner/v1/gifts/{id}/reissue` (a signed `nonce` keeps two reissues
    /// within one second apart).
    ///
    /// # Errors
    /// See [`CoreError`].
    pub async fn reissue(
        &self,
        core_id: &str,
        nonce: &str,
        now: i64,
    ) -> Result<IssuedCode, CoreError> {
        let path = format!("/api/partner/v1/gifts/{}/reissue?nonce={nonce}", segment(core_id)?);
        let response = self.send(Method::POST, &path, Vec::new(), Some(nonce), now).await?;
        json(response).await
    }

    /// `POST /api/partner/v1/gifts/{id}/cancel`. A gift the core already considers
    /// cancelled or expired (`409`, `410`) counts as cancelled.
    ///
    /// # Errors
    /// See [`CoreError`].
    pub async fn cancel(&self, core_id: &str, now: i64) -> Result<(), CoreError> {
        let path = format!("/api/partner/v1/gifts/{}/cancel", segment(core_id)?);
        let response = self.send(Method::POST, &path, Vec::new(), None, now).await?;
        let status = response.status();
        if status.is_success() || status == StatusCode::GONE {
            return Ok(());
        }
        let error = api_error(response).await;
        match &error {
            CoreError::Api { status: 409, code }
                if code.contains("cancel") || code.contains("gone") =>
            {
                Ok(())
            }
            _ => Err(error),
        }
    }

    /// `GET /api/partner/v1/gifts/{id}`.
    ///
    /// # Errors
    /// See [`CoreError`].
    pub async fn status(&self, core_id: &str, now: i64) -> Result<CoreGiftStatus, CoreError> {
        let path = format!("/api/partner/v1/gifts/{}", segment(core_id)?);
        json(self.send(Method::GET, &path, Vec::new(), None, now).await?).await
    }

    /// `GET /api/partner/v1/gifts/{id}/card.pdf?format=…&profile=…`.
    ///
    /// # Errors
    /// See [`CoreError`].
    pub async fn card(
        &self,
        core_id: &str,
        format: &str,
        profile: &str,
        now: i64,
    ) -> Result<Card, CoreError> {
        let path = format!(
            "/api/partner/v1/gifts/{}/card.pdf?format={format}&profile={profile}",
            segment(core_id)?
        );
        let response = self.send(Method::GET, &path, Vec::new(), None, now).await?;
        if !response.status().is_success() {
            return Err(api_error(response).await);
        }
        let warnings = response
            .headers()
            .get("x-ebc-card-warnings")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let pdf = read_capped(response, MAX_PDF_BYTES).await?;
        if !pdf.starts_with(b"%PDF-") {
            return Err(CoreError::Decode("card.pdf is not a PDF".to_owned()));
        }
        Ok(Card { pdf, warnings })
    }
}

async fn json<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T, CoreError> {
    if !response.status().is_success() {
        return Err(api_error(response).await);
    }
    let body = read_capped(response, MAX_JSON_BYTES).await?;
    serde_json::from_slice(&body).map_err(|e| CoreError::Decode(e.to_string()))
}

async fn api_error(response: reqwest::Response) -> CoreError {
    let status = response.status().as_u16();
    let body = read_capped(response, MAX_JSON_BYTES).await.unwrap_or_default();
    let code = serde_json::from_slice::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v["error"]["code"].as_str().map(str::to_owned))
        .filter(|c| c.len() <= 64 && c.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
        .unwrap_or_else(|| format!("http_{status}"));
    CoreError::Api { status, code }
}

async fn read_capped(mut response: reqwest::Response, cap: usize) -> Result<Vec<u8>, CoreError> {
    if response.content_length().is_some_and(|len| len > cap as u64) {
        return Err(CoreError::Decode("answer too large".to_owned()));
    }
    let mut body = Vec::new();
    while let Some(chunk) =
        response.chunk().await.map_err(|e| CoreError::Transport(describe(&e)))?
    {
        if body.len() + chunk.len() > cap {
            return Err(CoreError::Decode("answer too large".to_owned()));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn describe(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "timeout".to_owned()
    } else if error.is_connect() {
        "connection failed".to_owned()
    } else if error.is_body() || error.is_decode() {
        "broken answer".to_owned()
    } else {
        "request failed".to_owned()
    }
}

/// Whether an id from the core can be a URL path segment as is.
#[must_use]
pub fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id != "."
        && id != ".."
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'~'))
}

fn segment(id: &str) -> Result<&str, CoreError> {
    if is_safe_id(id) { Ok(id) } else { Err(CoreError::Decode("unsafe gift id".to_owned())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_from_the_core_are_checked_before_use_in_a_path() {
        assert!(is_safe_id("g_0123abcd"));
        for bad in ["", ".", "..", "a/b", "a?b", "a b", "é", &"a".repeat(129)] {
            assert!(!is_safe_id(bad), "{bad}");
        }
    }

    #[test]
    fn the_creation_body_omits_what_is_unknown() {
        let gift = NewGift {
            order_ref: "c_1".to_owned(),
            book: BookPayload {
                title: "Le Horla".to_owned(),
                isbn13: None,
                contributors: vec![Contributor { name: "Guy de Maupassant".to_owned() }],
                language: None,
            },
            giver_name: None,
            recipient_name: Some("Sacha".to_owned()),
            message: None,
            locale: "fr".to_owned(),
            reveal_at: None,
            delivery: Delivery {
                kind: "partner_file",
                format: "epub",
                url: "http://s/api/ebc/link".to_owned(),
            },
        };
        let json = serde_json::to_value(&gift).unwrap_or_default();
        assert_eq!(json["delivery"]["type"], "partner_file");
        assert!(json.get("giver_name").is_none() && json["book"].get("isbn13").is_none());
    }
}
