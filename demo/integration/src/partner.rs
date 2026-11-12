//! A small client of the eBook Cadeau partner API (`docs/api.md` §4) for tests that talk
//! to the server directly: every request is signed as §4.1 specifies, with the clock of
//! the [`World`](crate::World). It can also send requests that are deliberately wrong
//! (stale timestamp, foreign signature) to check that the server refuses them.
//!
//! The stores do **not** use this client: they have their own
//! (`ebc_demo_stores::outbound`), which is what the journeys exercise.

use std::time::Duration;

use crate::world::{WorldClock, WorldError};

/// What the server answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiReply {
    /// HTTP status.
    pub status: u16,
    /// Response headers, names in lower case.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

impl ApiReply {
    /// The first header named `name` (lower case).
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n == name).map(|(_, value)| value.as_str())
    }

    /// The body as text (lossy).
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// The body as JSON; `Null` when it is not JSON.
    #[must_use]
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
    }

    /// `error.code` of a JSON error body.
    #[must_use]
    pub fn error_code(&self) -> Option<String> {
        self.json()["error"]["code"].as_str().map(str::to_owned)
    }
}

/// A plain HTTP client for loopback calls: no proxy, no redirects followed, bounded time.
///
/// # Errors
/// [`WorldError::Http`] when the client cannot be built.
pub fn plain_client() -> Result<reqwest::Client, WorldError> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| WorldError::Http(e.to_string()))
}

/// Reads a response into an [`ApiReply`].
///
/// # Errors
/// [`WorldError::Http`] when the body cannot be read.
pub async fn read_reply(response: reqwest::Response) -> Result<ApiReply, WorldError> {
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (name.as_str().to_owned(), String::from_utf8_lossy(value.as_bytes()).into_owned())
        })
        .collect();
    let body = response.bytes().await.map_err(|e| WorldError::Http(e.to_string()))?.to_vec();
    Ok(ApiReply { status, headers, body })
}

/// A signing client of the partner API.
#[derive(Debug, Clone)]
pub struct PartnerApi {
    base: String,
    partner_id: String,
    secret: String,
    clock: WorldClock,
}

impl PartnerApi {
    /// A client for the server at `base`, signing as `partner_id` with `secret`.
    #[must_use]
    pub fn new(base: &str, partner_id: &str, secret: &str, clock: WorldClock) -> Self {
        Self {
            base: base.trim_end_matches('/').to_owned(),
            partner_id: partner_id.to_owned(),
            secret: secret.to_owned(),
            clock,
        }
    }

    /// The `X-EBC-Signature` value of a request (§4.1).
    #[must_use]
    pub fn signature(
        &self,
        timestamp: i64,
        method: &str,
        path_and_query: &str,
        body: &[u8],
    ) -> String {
        ebc_server::codes::request_signature(&self.secret, timestamp, method, path_and_query, body)
    }

    /// Sends one request with explicit authentication headers — the way to send a wrong
    /// timestamp or a signature computed over something else.
    ///
    /// # Errors
    /// [`WorldError::Http`] when no answer is received.
    pub async fn send_raw(
        &self,
        method: &str,
        path_and_query: &str,
        timestamp: i64,
        signature: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
    ) -> Result<ApiReply, WorldError> {
        let http = plain_client()?;
        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| WorldError::Http(e.to_string()))?;
        let mut request = http
            .request(method, format!("{}{path_and_query}", self.base))
            .header("X-EBC-Partner", &self.partner_id)
            .header("X-EBC-Timestamp", timestamp.to_string())
            .header("X-EBC-Signature", signature);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response =
            request.body(body).send().await.map_err(|e| WorldError::Http(e.to_string()))?;
        read_reply(response).await
    }

    /// Sends one correctly signed request. `path_and_query` starts with `/`; `headers`
    /// are added as given (`Content-Type`, `Idempotency-Key`, `X-File-Name`…).
    ///
    /// # Errors
    /// [`WorldError::Http`] when no answer is received.
    pub async fn send(
        &self,
        method: &str,
        path_and_query: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
    ) -> Result<ApiReply, WorldError> {
        let timestamp = self.clock.now();
        let signature = self.signature(timestamp, method, path_and_query, &body);
        self.send_raw(method, path_and_query, timestamp, &signature, body, headers).await
    }

    /// A signed `GET`.
    ///
    /// # Errors
    /// See [`send`](Self::send).
    pub async fn get(&self, path_and_query: &str) -> Result<ApiReply, WorldError> {
        self.send("GET", path_and_query, Vec::new(), &[]).await
    }

    /// A signed `POST` of a JSON document, with an optional `Idempotency-Key`.
    ///
    /// # Errors
    /// See [`send`](Self::send).
    pub async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
        idempotency_key: Option<&str>,
    ) -> Result<ApiReply, WorldError> {
        let mut headers = vec![("Content-Type", "application/json")];
        if let Some(key) = idempotency_key {
            headers.push(("Idempotency-Key", key));
        }
        self.send("POST", path, body.to_string().into_bytes(), &headers).await
    }

    /// `GET /api/partner/v1/gifts/{id}`: the status document of a gift.
    ///
    /// # Errors
    /// See [`send`](Self::send).
    pub async fn gift_status(&self, gift_id: &str) -> Result<ApiReply, WorldError> {
        self.get(&format!("/api/partner/v1/gifts/{gift_id}")).await
    }
}
