//! **Every outgoing HTTP call of both stores lives in this module**: the eBook Cadeau
//! partner API client (`docs/api.md` §4) and the print provider client (§5).
//!
//! Both stores share this code — they differ only in their [`PartnerConfig`]. When the real
//! server and the stores first meet, this is the one file to adjust: request paths, header
//! names, payload shapes and the leniency of the response parsing are all here.

use std::collections::BTreeMap;
use std::time::Duration;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::{Method, Response, StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::config::{Clock, PartnerConfig};
use crate::crypto;
use crate::error::StoreError;

/// Largest JSON response accepted from a collaborator.
const MAX_JSON_BYTES: usize = 1 << 20;
/// Largest card PDF accepted from the partner API.
const MAX_PDF_BYTES: usize = 64 << 20;

/// Characters escaped when an identifier is used as one URL path segment.
const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'\\')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Characters escaped in the `X-File-Name` header (percent-encoded UTF-8, RFC 3986
/// unreserved characters kept).
const HEADER_VALUE: &AsciiSet =
    &percent_encoding::NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

/// A failed outgoing call.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum OutboundError {
    /// The service could not be reached (refused, timed out, broken connection).
    #[error("{service} unreachable: {detail}")]
    Transport {
        /// Which collaborator.
        service: &'static str,
        /// Low-level description.
        detail: String,
    },
    /// The service answered with an error status.
    #[error("{service} answered {status} ({code}): {message}")]
    Api {
        /// Which collaborator.
        service: &'static str,
        /// HTTP status.
        status: u16,
        /// `error.code` of the JSON error body, or `"unknown"`.
        code: String,
        /// `error.message` of the JSON error body, or a short excerpt of the body.
        message: String,
    },
    /// The service answered 2xx with a body we could not understand.
    #[error("unexpected answer from {service}: {detail}")]
    Decode {
        /// Which collaborator.
        service: &'static str,
        /// What was wrong.
        detail: String,
    },
}

impl OutboundError {
    /// A short French sentence for the customer-facing pages.
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Transport { service, .. } => {
                format!("Le service {service} est momentanément injoignable.")
            }
            Self::Api { service, status, code, .. } => {
                format!("Le service {service} a refusé la demande (erreur {status}, {code}).")
            }
            Self::Decode { service, .. } => {
                format!("Le service {service} a renvoyé une réponse inattendue.")
            }
        }
    }

    /// The API error code, when the service returned one.
    #[must_use]
    pub fn api_code(&self) -> Option<&str> {
        match self {
            Self::Api { code, .. } => Some(code),
            _ => None,
        }
    }
}

/// Builds the shared HTTP client: rustls with the `ring` provider, no proxy, no redirects.
fn http_client(timeout: Duration) -> Result<reqwest::Client, StoreError> {
    // reqwest is built without a bundled TLS provider; install ring once per process.
    // An error only means another component already installed one.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(timeout.min(Duration::from_secs(5)))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .user_agent(concat!("ebc-demo-stores/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| StoreError::HttpClient(e.to_string()))
}

fn parse_base(name: &str, base: &str) -> Result<Url, StoreError> {
    match Url::parse(base) {
        Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => Ok(url),
        _ => Err(StoreError::Config(format!("{name} must be an http(s) URL, got {base:?}"))),
    }
}

/// Reads a response body, refusing anything larger than `cap` bytes.
async fn read_capped(
    service: &'static str,
    mut response: Response,
    cap: usize,
) -> Result<Vec<u8>, OutboundError> {
    if response.content_length().is_some_and(|len| len > cap as u64) {
        return Err(OutboundError::Decode { service, detail: "response too large".to_owned() });
    }
    let mut body = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if body.len() + chunk.len() > cap {
                    return Err(OutboundError::Decode {
                        service,
                        detail: "response too large".to_owned(),
                    });
                }
                body.extend_from_slice(&chunk);
            }
            Ok(None) => return Ok(body),
            Err(e) => return Err(OutboundError::Transport { service, detail: e.to_string() }),
        }
    }
}

/// Turns a non-2xx response into [`OutboundError::Api`], reading `{"error": {"code", "message"}}`
/// when the body has that shape.
async fn api_error(service: &'static str, response: Response) -> OutboundError {
    let status = response.status().as_u16();
    let body = read_capped(service, response, MAX_JSON_BYTES).await.unwrap_or_default();
    let (code, message) = error_fields(&body);
    OutboundError::Api { service, status, code, message }
}

fn error_fields(body: &[u8]) -> (String, String) {
    let parsed: Option<serde_json::Value> = serde_json::from_slice(body).ok();
    let field = |name: &str| {
        parsed
            .as_ref()
            .and_then(|v| v.get("error"))
            .and_then(|e| e.get(name))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    };
    let excerpt: String = String::from_utf8_lossy(body).chars().take(200).collect();
    (field("code").unwrap_or_else(|| "unknown".to_owned()), field("message").unwrap_or(excerpt))
}

fn decode_json<T: serde::de::DeserializeOwned>(
    service: &'static str,
    body: &[u8],
) -> Result<T, OutboundError> {
    serde_json::from_slice(body)
        .map_err(|e| OutboundError::Decode { service, detail: e.to_string() })
}

// ---------------------------------------------------------------------------------------
// eBook Cadeau partner API (docs/api.md §4)
// ---------------------------------------------------------------------------------------

const EBC: &str = "eBook Cadeau";

/// `book` object of a gift creation request.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BookPayload {
    /// Title (with the volume, if any).
    pub title: String,
    /// ISBN-13, digits only.
    pub isbn13: String,
    /// Authors.
    pub contributors: Vec<ContributorPayload>,
    /// Publisher name.
    pub publisher: String,
    /// BCP 47 language of the book.
    pub language: String,
}

/// One contributor of a [`BookPayload`].
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ContributorPayload {
    /// Display name.
    pub name: String,
}

/// `delivery` object of a gift creation request.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeliveryPayload {
    /// eBook Cadeau serves a file uploaded beforehand.
    HostedFile {
        /// Identifier returned by the upload.
        upload_id: String,
    },
    /// eBook Cadeau redirects the recipient to the store.
    ResellerRedirect {
        /// The store's activation URL.
        url: String,
    },
}

/// Body of `POST /api/partner/v1/gifts`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewGift {
    /// The store's reference for this gift (order line).
    pub order_ref: String,
    /// The book.
    pub book: BookPayload,
    /// Name of the buyer as it should appear on the card.
    pub giver_name: String,
    /// Name of the person receiving the gift.
    pub recipient_name: String,
    /// Personal message (may be empty).
    pub message: String,
    /// Locale of the card and redemption pages.
    pub locale: String,
    /// How the eBook reaches the recipient.
    pub delivery: DeliveryPayload,
}

/// Answer to a gift creation.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CreatedGift {
    /// Gift identifier (`g_…`).
    pub id: String,
    /// Compact redemption code.
    #[serde(default)]
    pub code: Option<String>,
    /// Redemption code as printed (`XXXXX-XXXXX`).
    #[serde(default)]
    pub code_display: Option<String>,
    /// URL encoded in the QR code.
    pub claim_url: String,
    /// Initial status (`created`).
    #[serde(default)]
    pub status: Option<String>,
}

/// Answer to an upload.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UploadReceipt {
    /// Identifier to quote in a `hosted_file` delivery.
    pub upload_id: String,
    /// SHA-256 of the stored bytes, when the server reports it.
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Answer to `GET /api/partner/v1/gifts/{id}`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RemoteGiftStatus {
    /// `created`, `redeemed`, `fulfilled`, `cancelled` or `expired`.
    pub status: String,
    /// When the gift was first claimed.
    #[serde(default)]
    pub redeemed_at: Option<i64>,
    /// Number of download sessions so far.
    #[serde(default)]
    pub downloads: Option<u64>,
    /// The order reference given at creation.
    #[serde(default)]
    pub order_ref: Option<String>,
}

/// Outcome of fetching the server-rendered card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardFetch {
    /// The PDF bytes.
    Ready(Vec<u8>),
    /// The server answered `501`: it does not render cards (yet).
    NotImplemented,
}

/// Outcome of a fulfilment confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fulfilment {
    /// Recorded (or already recorded for the same account).
    Confirmed,
    /// `409 already_fulfilled`: another account was confirmed earlier.
    AlreadyFulfilled,
}

/// Client of the eBook Cadeau partner API. Every request is signed as specified in §4.1.
#[derive(Debug, Clone)]
pub struct PartnerClient {
    http: reqwest::Client,
    base: Url,
    partner_id: String,
    secret: String,
    clock: Clock,
}

impl PartnerClient {
    /// Creates a client.
    ///
    /// # Errors
    /// [`StoreError::Config`] for an unusable base URL, [`StoreError::HttpClient`] when the
    /// HTTP client cannot be built.
    pub fn new(
        config: &PartnerConfig,
        clock: Clock,
        timeout: Duration,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            http: http_client(timeout)?,
            base: parse_base("the eBook Cadeau URL", &config.base_url)?,
            partner_id: config.partner_id.clone(),
            secret: config.secret.clone(),
            clock,
        })
    }

    /// Sends one signed request. `path_and_query` starts with `/`.
    async fn send(
        &self,
        method: Method,
        path_and_query: &str,
        content_type: Option<&str>,
        body: Vec<u8>,
        extra_headers: &[(&str, &str)],
    ) -> Result<Response, OutboundError> {
        let joined = format!("{}{path_and_query}", self.base.as_str().trim_end_matches('/'));
        let url = Url::parse(&joined)
            .map_err(|e| OutboundError::Decode { service: EBC, detail: format!("bad URL: {e}") })?;
        // Sign exactly what goes on the wire: the path and query of the parsed URL.
        let wire_path = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_owned(),
        };
        let timestamp = self.clock.now();
        let signature =
            crypto::request_signature(&self.secret, timestamp, method.as_str(), &wire_path, &body);
        let mut request = self
            .http
            .request(method, url)
            .header("X-EBC-Partner", &self.partner_id)
            .header("X-EBC-Timestamp", timestamp.to_string())
            .header("X-EBC-Signature", signature);
        if let Some(content_type) = content_type {
            request = request.header(reqwest::header::CONTENT_TYPE, content_type);
        }
        for (name, value) in extra_headers {
            request = request.header(*name, *value);
        }
        request
            .body(body)
            .send()
            .await
            .map_err(|e| OutboundError::Transport { service: EBC, detail: describe(&e) })
    }

    /// `POST /api/partner/v1/uploads` — stores a file for a `hosted_file` delivery, declared
    /// under the rights basis `licensed_distributor`.
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn upload(
        &self,
        file_name: &str,
        media_type: &str,
        bytes: Vec<u8>,
        idempotency_key: &str,
    ) -> Result<UploadReceipt, OutboundError> {
        let encoded_name = utf8_percent_encode(file_name, HEADER_VALUE).to_string();
        let response = self
            .send(
                Method::POST,
                "/api/partner/v1/uploads",
                Some(media_type),
                bytes,
                &[
                    ("X-File-Name", &encoded_name),
                    ("X-Rights-Basis", "licensed_distributor"),
                    ("Idempotency-Key", idempotency_key),
                ],
            )
            .await?;
        if !response.status().is_success() {
            return Err(api_error(EBC, response).await);
        }
        decode_json(EBC, &read_capped(EBC, response, MAX_JSON_BYTES).await?)
    }

    /// `POST /api/partner/v1/gifts` with `Idempotency-Key`.
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn create_gift(
        &self,
        gift: &NewGift,
        idempotency_key: &str,
    ) -> Result<CreatedGift, OutboundError> {
        let body = serde_json::to_vec(gift)
            .map_err(|e| OutboundError::Decode { service: EBC, detail: e.to_string() })?;
        let response = self
            .send(
                Method::POST,
                "/api/partner/v1/gifts",
                Some("application/json"),
                body,
                &[("Idempotency-Key", idempotency_key)],
            )
            .await?;
        if !response.status().is_success() {
            return Err(api_error(EBC, response).await);
        }
        let created: CreatedGift =
            decode_json(EBC, &read_capped(EBC, response, MAX_JSON_BYTES).await?)?;
        if !is_safe_id(&created.id) || created.claim_url.is_empty() {
            return Err(OutboundError::Decode {
                service: EBC,
                detail: "gift without a usable id or claim_url".to_owned(),
            });
        }
        Ok(created)
    }

    /// `PUT /api/partner/v1/gifts/{id}/cover` — the cover image the store is entitled to print.
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn put_cover(
        &self,
        gift_id: &str,
        media_type: &str,
        bytes: Vec<u8>,
    ) -> Result<(), OutboundError> {
        let path = format!("/api/partner/v1/gifts/{}/cover", segment(gift_id));
        let response = self.send(Method::PUT, &path, Some(media_type), bytes, &[]).await?;
        if response.status().is_success() { Ok(()) } else { Err(api_error(EBC, response).await) }
    }

    /// `GET /api/partner/v1/gifts/{id}/card.pdf?format=…&profile=…`.
    ///
    /// # Errors
    /// See [`OutboundError`]. A `501` is not an error: it yields [`CardFetch::NotImplemented`].
    pub async fn card_pdf(
        &self,
        gift_id: &str,
        format: &str,
        profile: &str,
    ) -> Result<CardFetch, OutboundError> {
        let path = format!(
            "/api/partner/v1/gifts/{}/card.pdf?format={}&profile={}",
            segment(gift_id),
            segment(format),
            segment(profile)
        );
        let response = self.send(Method::GET, &path, None, Vec::new(), &[]).await?;
        if response.status() == StatusCode::NOT_IMPLEMENTED {
            return Ok(CardFetch::NotImplemented);
        }
        if !response.status().is_success() {
            return Err(api_error(EBC, response).await);
        }
        let bytes = read_capped(EBC, response, MAX_PDF_BYTES).await?;
        if !bytes.starts_with(b"%PDF-") {
            return Err(OutboundError::Decode {
                service: EBC,
                detail: "card.pdf is not a PDF".to_owned(),
            });
        }
        Ok(CardFetch::Ready(bytes))
    }

    /// `GET /api/partner/v1/gifts/{id}`.
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn gift_status(&self, gift_id: &str) -> Result<RemoteGiftStatus, OutboundError> {
        let path = format!("/api/partner/v1/gifts/{}", segment(gift_id));
        let response = self.send(Method::GET, &path, None, Vec::new(), &[]).await?;
        if !response.status().is_success() {
            return Err(api_error(EBC, response).await);
        }
        decode_json(EBC, &read_capped(EBC, response, MAX_JSON_BYTES).await?)
    }

    /// `POST /api/partner/v1/gifts/{id}/cancel` — voids the code (refund path).
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn cancel_gift(&self, gift_id: &str) -> Result<(), OutboundError> {
        let path = format!("/api/partner/v1/gifts/{}/cancel", segment(gift_id));
        let response = self.send(Method::POST, &path, None, Vec::new(), &[]).await?;
        if response.status().is_success() { Ok(()) } else { Err(api_error(EBC, response).await) }
    }

    /// `POST /api/partner/v1/gifts/{id}/fulfilled` — tells eBook Cadeau the book is now in
    /// the recipient's account (§4.4).
    ///
    /// # Errors
    /// See [`OutboundError`]. `409 already_fulfilled` is reported as
    /// [`Fulfilment::AlreadyFulfilled`], not as an error.
    pub async fn confirm_fulfilled(
        &self,
        gift_id: &str,
        account_ref: &str,
    ) -> Result<Fulfilment, OutboundError> {
        let path = format!("/api/partner/v1/gifts/{}/fulfilled", segment(gift_id));
        let body = serde_json::to_vec(&serde_json::json!({ "account_ref": account_ref }))
            .map_err(|e| OutboundError::Decode { service: EBC, detail: e.to_string() })?;
        let response = self.send(Method::POST, &path, Some("application/json"), body, &[]).await?;
        if response.status().is_success() {
            return Ok(Fulfilment::Confirmed);
        }
        let error = api_error(EBC, response).await;
        match &error {
            OutboundError::Api { status: 409, code, .. } if code == "already_fulfilled" => {
                Ok(Fulfilment::AlreadyFulfilled)
            }
            _ => Err(error),
        }
    }
}

fn segment(value: &str) -> String {
    utf8_percent_encode(value, SEGMENT).to_string()
}

/// Whether an identifier received from a collaborator can be used as a URL path segment
/// without surprises: 1 to 128 characters among `A-Z a-z 0-9 _ - . ~`, and not a dot
/// segment (`.` / `..`), which URL normalisation would resolve away.
#[must_use]
pub fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id != "."
        && id != ".."
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'~'))
}

/// A compact description of a transport error (reqwest's `Display` hides the cause).
fn describe(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "timeout".to_owned()
    } else if error.is_connect() {
        "connection failed".to_owned()
    } else {
        error.to_string()
    }
}

// ---------------------------------------------------------------------------------------
// Print provider (docs/api.md §5)
// ---------------------------------------------------------------------------------------

const PRINTER: &str = "l'imprimeur";

/// One entry of the printer's catalogue (`GET /v1/products`). Unknown fields are ignored.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PrinterProduct {
    /// Product identifier, e.g. `card-poche`.
    pub id: String,
    /// Trim size of the flat spread, `[width, height]` in millimetres.
    #[serde(default)]
    pub trim_mm: Vec<f64>,
    /// Paper description.
    #[serde(default)]
    pub paper: Option<String>,
    /// Price ladder: quantity → total price in cents for that quantity.
    #[serde(default)]
    pub price_cents: BTreeMap<String, u32>,
}

impl PrinterProduct {
    /// Total price in cents for `quantity` copies: the unit price of the highest rung not
    /// above `quantity`, multiplied by `quantity`. `None` when the ladder has no usable
    /// rung (the quantity is below the smallest lot, or the ladder is empty).
    #[must_use]
    pub fn price_for(&self, quantity: u32) -> Option<u32> {
        let (rung, total) = self
            .price_cents
            .iter()
            .filter_map(|(k, v)| k.parse::<u32>().ok().filter(|q| *q > 0).map(|q| (q, *v)))
            .filter(|(q, _)| *q <= quantity)
            .max_by_key(|(q, _)| *q)?;
        let price = (u64::from(total) * u64::from(quantity)).div_ceil(u64::from(rung));
        u32::try_from(price).ok()
    }
}

/// Postal address of a print order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostalAddress {
    /// Addressee.
    pub name: String,
    /// Street line.
    pub line1: String,
    /// Postal code.
    pub postal_code: String,
    /// City.
    pub city: String,
    /// ISO 3166-1 alpha-2 country code.
    pub country: String,
}

/// The `order` part of `POST /v1/orders`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PrintOrderRequest {
    /// Product identifier.
    pub product: String,
    /// Number of copies.
    pub quantity: u32,
    /// Where to post the card.
    pub address: PostalAddress,
    /// The store's reference (order line).
    pub reference: String,
    /// Where the printer posts status changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

/// Outcome of submitting a print order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintSubmission {
    /// `201`: the PDF passed preflight.
    Accepted(PrintOrderStatus),
    /// `422 preflight_failed`: the printer refused the file.
    Rejected {
        /// One line per error finding: `CODE: message`.
        findings: Vec<String>,
    },
}

/// A print order as the printer reports it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PrintOrderStatus {
    /// Printer-side identifier. Absent from status polls of some printers.
    #[serde(default)]
    pub order_id: Option<String>,
    /// `accepted`, `printing`, `shipped` or `cancelled`.
    pub status: String,
    /// Price charged, in cents.
    #[serde(default)]
    pub price_cents: Option<u32>,
    /// Tracking information (a string, or an object with a `number`).
    #[serde(default)]
    pub tracking: Option<serde_json::Value>,
}

impl PrintOrderStatus {
    /// The tracking number as text, whatever shape the printer used.
    #[must_use]
    pub fn tracking_text(&self) -> Option<String> {
        match self.tracking.as_ref()? {
            serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
            serde_json::Value::Object(map) => {
                map.get("number").and_then(serde_json::Value::as_str).map(str::to_owned)
            }
            _ => None,
        }
    }
}

/// Client of the print provider API (§5). The demo printer has no authentication.
#[derive(Debug, Clone)]
pub struct PrinterClient {
    http: reqwest::Client,
    base: Url,
}

impl PrinterClient {
    /// Creates a client.
    ///
    /// # Errors
    /// [`StoreError::Config`] for an unusable base URL, [`StoreError::HttpClient`] when the
    /// HTTP client cannot be built.
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, StoreError> {
        Ok(Self { http: http_client(timeout)?, base: parse_base("EBC_PRINTER_URL", base_url)? })
    }

    fn url(&self, path: &str) -> Result<Url, OutboundError> {
        Url::parse(&format!("{}{path}", self.base.as_str().trim_end_matches('/'))).map_err(|e| {
            OutboundError::Decode { service: PRINTER, detail: format!("bad URL: {e}") }
        })
    }

    /// `GET /v1/products`.
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn products(&self) -> Result<Vec<PrinterProduct>, OutboundError> {
        let response = self
            .http
            .get(self.url("/v1/products")?)
            .send()
            .await
            .map_err(|e| OutboundError::Transport { service: PRINTER, detail: describe(&e) })?;
        if !response.status().is_success() {
            return Err(api_error(PRINTER, response).await);
        }
        decode_json(PRINTER, &read_capped(PRINTER, response, MAX_JSON_BYTES).await?)
    }

    /// `POST /v1/orders` (multipart: `order` JSON + `file` PDF).
    ///
    /// # Errors
    /// See [`OutboundError`]. A preflight refusal is [`PrintSubmission::Rejected`], not an
    /// error.
    pub async fn create_order(
        &self,
        order: &PrintOrderRequest,
        pdf: Vec<u8>,
    ) -> Result<PrintSubmission, OutboundError> {
        let decode = |e: String| OutboundError::Decode { service: PRINTER, detail: e };
        let order_json = serde_json::to_string(order).map_err(|e| decode(e.to_string()))?;
        let order_part = reqwest::multipart::Part::text(order_json)
            .mime_str("application/json")
            .map_err(|e| decode(e.to_string()))?;
        let file_part = reqwest::multipart::Part::bytes(pdf)
            .file_name("card.pdf")
            .mime_str("application/pdf")
            .map_err(|e| decode(e.to_string()))?;
        let form =
            reqwest::multipart::Form::new().part("order", order_part).part("file", file_part);
        let response = self
            .http
            .post(self.url("/v1/orders")?)
            .multipart(form)
            .send()
            .await
            .map_err(|e| OutboundError::Transport { service: PRINTER, detail: describe(&e) })?;
        let status = response.status();
        if status.is_success() {
            let body = read_capped(PRINTER, response, MAX_JSON_BYTES).await?;
            let accepted: PrintOrderStatus = decode_json(PRINTER, &body)?;
            if !accepted.order_id.as_deref().is_some_and(is_safe_id) {
                return Err(decode("order without a usable order_id".to_owned()));
            }
            return Ok(PrintSubmission::Accepted(accepted));
        }
        let body = read_capped(PRINTER, response, MAX_JSON_BYTES).await.unwrap_or_default();
        let (code, message) = error_fields(&body);
        if status == StatusCode::UNPROCESSABLE_ENTITY && code == "preflight_failed" {
            return Ok(PrintSubmission::Rejected { findings: preflight_findings(&body) });
        }
        Err(OutboundError::Api { service: PRINTER, status: status.as_u16(), code, message })
    }

    /// `GET /v1/orders/{id}`.
    ///
    /// # Errors
    /// See [`OutboundError`].
    pub async fn order(&self, order_id: &str) -> Result<PrintOrderStatus, OutboundError> {
        let response = self
            .http
            .get(self.url(&format!("/v1/orders/{}", segment(order_id)))?)
            .send()
            .await
            .map_err(|e| OutboundError::Transport { service: PRINTER, detail: describe(&e) })?;
        if !response.status().is_success() {
            return Err(api_error(PRINTER, response).await);
        }
        decode_json(PRINTER, &read_capped(PRINTER, response, MAX_JSON_BYTES).await?)
    }
}

/// Extracts the error-level findings of a `preflight_failed` body as `CODE: message` lines
/// (at most 20, each cut to 300 characters).
fn preflight_findings(body: &[u8]) -> Vec<String> {
    let parsed: Option<serde_json::Value> = serde_json::from_slice(body).ok();
    let findings = parsed
        .as_ref()
        .and_then(|v| v.get("error"))
        .and_then(|e| e.get("findings"))
        .and_then(serde_json::Value::as_array);
    let Some(findings) = findings else { return Vec::new() };
    findings
        .iter()
        .filter(|f| {
            f.get("severity").and_then(serde_json::Value::as_str).is_none_or(|s| s == "error")
        })
        .take(20)
        .map(|f| {
            let text = |name: &str| f.get(name).and_then(serde_json::Value::as_str).unwrap_or("?");
            let line = format!("{}: {}", text("code"), text("message"));
            line.chars().take(300).collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product(ladder: &[(&str, u32)]) -> PrinterProduct {
        PrinterProduct {
            id: "card-poche".into(),
            trim_mm: vec![230.0, 178.0],
            paper: None,
            price_cents: ladder.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
        }
    }

    #[test]
    fn price_ladder_uses_the_highest_rung_not_above_the_quantity() {
        let p = product(&[("1", 590), ("10", 2400), ("100", 8900)]);
        assert_eq!(p.price_for(1), Some(590));
        assert_eq!(p.price_for(2), Some(1180));
        assert_eq!(p.price_for(9), Some(5310));
        assert_eq!(p.price_for(10), Some(2400));
        assert_eq!(p.price_for(15), Some(3600));
        assert_eq!(p.price_for(100), Some(8900));
        assert_eq!(p.price_for(0), None);
        // A ladder that starts at 100 has no price for a single copy.
        assert_eq!(product(&[("100", 8900)]).price_for(1), None);
        assert_eq!(product(&[]).price_for(1), None);
        // Garbage rungs are ignored, and huge numbers do not overflow.
        assert_eq!(product(&[("x", 1), ("0", 5), ("1", 700)]).price_for(3), Some(2100));
        assert_eq!(product(&[("1", u32::MAX)]).price_for(u32::MAX), None);
    }

    #[test]
    fn delivery_and_gift_serialise_as_documented() {
        let gift = NewGift {
            order_ref: "SEP-2026-000001".into(),
            book: BookPayload {
                title: "Candide".into(),
                isbn13: "9786990000015".into(),
                contributors: vec![ContributorPayload { name: "Voltaire".into() }],
                publisher: "P".into(),
                language: "fr".into(),
            },
            giver_name: "Camille".into(),
            recipient_name: "Sacha".into(),
            message: String::new(),
            locale: "fr".into(),
            delivery: DeliveryPayload::ResellerRedirect { url: "https://s.example/cadeau".into() },
        };
        let value = serde_json::to_value(&gift).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "order_ref": "SEP-2026-000001",
                "book": {
                    "title": "Candide", "isbn13": "9786990000015",
                    "contributors": [{"name": "Voltaire"}], "publisher": "P", "language": "fr"
                },
                "giver_name": "Camille", "recipient_name": "Sacha", "message": "",
                "locale": "fr",
                "delivery": {"type": "reseller_redirect", "url": "https://s.example/cadeau"}
            })
        );
        assert_eq!(
            serde_json::to_value(DeliveryPayload::HostedFile { upload_id: "u_1".into() }).unwrap(),
            serde_json::json!({"type": "hosted_file", "upload_id": "u_1"})
        );
    }

    #[test]
    fn error_bodies_are_read_leniently() {
        assert_eq!(
            error_fields(br#"{"error":{"code":"bad_signature","message":"nope"}}"#),
            ("bad_signature".to_owned(), "nope".to_owned())
        );
        assert_eq!(
            error_fields(b"<html>502</html>"),
            ("unknown".into(), "<html>502</html>".into())
        );
        assert_eq!(error_fields(b""), ("unknown".into(), String::new()));
        assert_eq!(error_fields(br#"{"error":"flat"}"#).0, "unknown");
        let long = vec![b'x'; 5000];
        assert_eq!(error_fields(&long).1.len(), 200);
    }

    #[test]
    fn preflight_findings_keep_errors_only_and_are_bounded() {
        let body = serde_json::json!({"error": {"code": "preflight_failed", "findings": [
            {"severity": "error", "code": "BOX_TRIM_SIZE", "message": "trim is 210 x 148"},
            {"severity": "warning", "code": "FONT_TYPE3", "message": "w"},
            {"code": "NO_SEVERITY", "message": "kept"},
            "garbage"
        ]}});
        let lines = preflight_findings(&serde_json::to_vec(&body).unwrap());
        assert_eq!(lines, vec!["BOX_TRIM_SIZE: trim is 210 x 148", "NO_SEVERITY: kept", "?: ?"]);
        assert!(preflight_findings(b"not json").is_empty());
        let many: Vec<_> = (0..50)
            .map(|i| serde_json::json!({"severity": "error", "code": "C", "message": "m".repeat(i * 20)}))
            .collect();
        let body = serde_json::json!({"error": {"findings": many}});
        let lines = preflight_findings(&serde_json::to_vec(&body).unwrap());
        assert_eq!(lines.len(), 20);
        assert!(lines.iter().all(|l| l.chars().count() <= 300));
    }

    #[test]
    fn tracking_is_read_from_a_string_or_an_object() {
        let parse = |v: serde_json::Value| {
            serde_json::from_value::<PrintOrderStatus>(v).unwrap().tracking_text()
        };
        assert_eq!(
            parse(serde_json::json!({"status": "shipped", "tracking": "DC-1"})),
            Some("DC-1".into())
        );
        assert_eq!(
            parse(
                serde_json::json!({"status": "shipped", "tracking": {"number": "N2", "carrier": "x"}})
            ),
            Some("N2".into())
        );
        assert_eq!(parse(serde_json::json!({"status": "accepted", "tracking": null})), None);
        assert_eq!(parse(serde_json::json!({"status": "accepted"})), None);
        assert_eq!(parse(serde_json::json!({"status": "accepted", "tracking": 12})), None);
    }

    #[test]
    fn identifiers_are_escaped_as_single_path_segments() {
        assert_eq!(segment("g_abc-123"), "g_abc-123");
        assert_eq!(segment("../admin?x=1#y"), "..%2Fadmin%3Fx=1%23y");
        assert_eq!(segment("a b%c"), "a%20b%25c");
    }

    #[test]
    fn remote_identifiers_are_vetted() {
        for ok in ["g_abc-123", "po_0123456789abcdef", "a.b~c", "X"] {
            assert!(is_safe_id(ok), "{ok}");
        }
        let too_long = "a".repeat(129);
        for bad in ["", ".", "..", "a/b", "a b", "a?b", "a%2e", "é", too_long.as_str()] {
            assert!(!is_safe_id(bad), "{bad}");
        }
    }

    #[test]
    fn bad_base_urls_are_configuration_errors() {
        for bad in ["", "ftp://x", "/relative", "http://"] {
            assert!(matches!(
                PrinterClient::new(bad, Duration::from_secs(1)),
                Err(StoreError::Config(_))
            ));
        }
        assert!(PrinterClient::new("http://127.0.0.1:1", Duration::from_secs(1)).is_ok());
    }
}
