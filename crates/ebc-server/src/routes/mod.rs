//! The HTTP surface: the router of `docs/api.md` §2–§4, the security headers every
//! response carries, and helpers shared by the handlers.

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use ebc_model::{GiftCode, Locale};

use crate::device::{self, DeviceClass};
use crate::error::ApiError;
use crate::pages::{self, Chrome};
use crate::state::AppState;
use crate::text::{S, s};

mod creator;
mod files;
mod partner;
mod recipient;

/// Largest JSON body accepted by the APIs.
pub(crate) const MAX_JSON_BYTES: usize = 64 * 1024;

const X_ROBOTS_TAG: HeaderName = HeaderName::from_static("x-robots-tag");

/// `Content-Security-Policy` of responses that are not pages: nothing may be loaded.
const CSP_NOTHING: &str = "default-src 'none'; frame-ancestors 'none'";
/// `Content-Security-Policy` of the static web app under `/app/` (scripts and WebAssembly
/// from this origin; covers and catalogue look-ups fetched by the browser over HTTPS).
const CSP_APP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; \
    style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; \
    connect-src 'self' https:; worker-src 'self' blob:; object-src 'none'; \
    base-uri 'self'; frame-ancestors 'none'";

/// Builds the service.
///
/// Mount it with `into_make_service_with_connect_info::<SocketAddr>()` so that clients
/// are told apart by their address; without connection information (in-process tests)
/// every client shares one rate-limit bucket unless `EBC_TRUST_PROXY` is set and
/// `X-Forwarded-For` is present.
pub fn app(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/", get(recipient::home))
        .route("/healthz", get(|| async { "ok" }))
        .route("/aide", get(recipient::help))
        .route("/static/eink.css", get(files::style))
        .route("/e", get(recipient::pair_form).post(recipient::pair_submit))
        .route("/E", get(recipient::pair_form).post(recipient::pair_submit))
        .route("/d/{session}/{name}", get(files::download))
        .route("/o/{session}/catalog.xml", get(files::opds_atom))
        .route("/o/{session}/catalog.json", get(files::opds_json))
        .route("/o/{session}/cover", get(files::opds_cover))
        .route("/api/v1/uploads", post(creator::upload))
        .route("/api/v1/gifts", post(creator::create))
        .route("/api/v1/gifts/{id}", get(creator::status).delete(creator::cancel))
        .route("/api/v1/gifts/{id}/reissue", post(creator::reissue))
        .route("/api/v1/gifts/{id}/card.pdf", put(creator::put_card_pdf))
        .route("/api/v1/gifts/{id}/print-orders", post(creator::print_order))
        .route("/api/partner/v1/uploads", post(partner::upload))
        .route("/api/partner/v1/gifts", post(partner::create))
        .route("/api/partner/v1/gifts/{id}", get(partner::status))
        .route("/api/partner/v1/gifts/{id}/cover", put(partner::put_cover))
        .route("/api/partner/v1/gifts/{id}/card.pdf", get(partner::card_pdf))
        .route("/api/partner/v1/gifts/{id}/cancel", post(partner::cancel))
        .route("/api/partner/v1/gifts/{id}/reissue", post(partner::reissue))
        .route("/api/partner/v1/gifts/{id}/fulfilled", post(partner::fulfilled));
    // The QR code carries the URL in upper case: `/K/` is `/k/`.
    for prefix in ["/k", "/K"] {
        router = router
            .route(prefix, get(recipient::code_query))
            .route(&format!("{prefix}/{{code}}"), get(recipient::gift_page))
            .route(&format!("{prefix}/{{code}}/download"), post(recipient::download))
            .route(&format!("{prefix}/{{code}}/pair"), post(recipient::pair))
            .route(&format!("{prefix}/{{code}}/send"), post(recipient::send))
            .route(&format!("{prefix}/{{code}}/redeem"), post(recipient::redeem));
    }
    if let Some(dir) = &state.config.web_dir {
        router = router.nest_service("/app", tower_http::services::ServeDir::new(dir));
    }
    let request_timeout = Duration::from_secs(state.config.request_timeout_secs.max(1));
    let body_idle_timeout = Duration::from_secs(state.config.body_idle_timeout_secs.max(1));
    router
        .fallback(fallback)
        // A request body that stalls, or a request that takes too long as a whole (body
        // upload included), is cut off: slow clients cannot hold a task for ever.
        .layer(tower_http::timeout::RequestBodyTimeoutLayer::new(body_idle_timeout))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::http::Request<Body>| {
                // Codes and download tokens are bearer secrets: they never reach the logs,
                // at any level. The query string is not logged at all.
                let path = redact_path(request.uri().path());
                tracing::debug_span!("request", method = %request.method(), %path)
            },
        ))
        .with_state(state)
}

/// A request path fit for logs: the segment after `/k/`, `/d/` and `/o/` (a gift code or
/// a download token) is replaced, and so is any segment that is itself a well-formed
/// gift code (`GET /{CODE}`) or looks like a token.
pub(crate) fn redact_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut after_secret_prefix = false;
    for (index, segment) in path.split('/').enumerate() {
        if index > 0 {
            out.push('/');
        }
        let token_like = segment.len() >= 24 && segment.bytes().all(|b| b.is_ascii_hexdigit());
        if !segment.is_empty()
            && (after_secret_prefix || token_like || GiftCode::parse(segment).is_ok())
        {
            out.push('…');
        } else {
            // Bound what a hostile path can write into a log line.
            out.extend(segment.chars().filter(|c| !c.is_control()).take(64));
        }
        after_secret_prefix = index == 1 && matches!(segment, "k" | "K" | "d" | "o");
    }
    out
}

/// Adds the headers of `docs/api.md` §2 to every response that does not set them itself.
async fn security_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let is_app = path == "/app" || path.starts_with("/app/");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    let mut default = |name: HeaderName, value: &'static str| {
        headers.entry(name).or_insert(HeaderValue::from_static(value));
    };
    default(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    default(header::REFERRER_POLICY, "no-referrer");
    default(header::X_FRAME_OPTIONS, "DENY");
    if is_app {
        default(header::CONTENT_SECURITY_POLICY, CSP_APP);
    } else {
        default(header::CACHE_CONTROL, "no-store");
        default(X_ROBOTS_TAG, "noindex, nofollow");
        default(header::CONTENT_SECURITY_POLICY, CSP_NOTHING);
    }
    response
}

/// Anything that is not a route: `GET /{CODE}` is a shortcut to the gift page; the rest
/// is `404` (JSON under `/api/`, a page elsewhere).
async fn fallback(
    State(state): State<AppState>,
    visitor: Visitor,
    method: Method,
    uri: Uri,
) -> Response {
    let path = uri.path();
    if path.starts_with("/api/") {
        return ApiError::not_found("no such endpoint").into_response();
    }
    let segment = path.trim_matches('/');
    if method == Method::GET
        && !segment.contains('/')
        && let Ok(code) = GiftCode::parse(segment)
    {
        return see_other(&format!("/k/{}", code.compact()));
    }
    let chrome = visitor.chrome(&state, visitor.locale(), None);
    let text = s(chrome.locale, S::NotFound);
    html(
        StatusCode::NOT_FOUND,
        pages::message(chrome, text, &[], Some(("/", s(chrome.locale, S::EnterCode)))),
        None,
    )
}

/// What the request says about who is asking.
#[derive(Debug, Clone)]
pub(crate) struct Visitor {
    /// Rate-limit key: the client's network (IPv4 /24, IPv6 /56), or `unknown`.
    pub(crate) client: String,
    /// Device class guessed from `User-Agent`.
    pub(crate) detected: DeviceClass,
    /// Best supported language of `Accept-Language`.
    pub(crate) accept: Option<Locale>,
}

impl Visitor {
    /// The language of pages that do not belong to a gift.
    pub(crate) fn locale(&self) -> Locale {
        self.accept.unwrap_or_default()
    }

    /// The page context for `locale` and an optional `?device=` value.
    pub(crate) fn chrome<'a>(
        &self,
        state: &'a AppState,
        locale: Locale,
        device: Option<&str>,
    ) -> Chrome<'a> {
        let chosen = device.and_then(DeviceClass::from_choice);
        Chrome {
            locale,
            device: chosen.unwrap_or(self.detected),
            chosen,
            host: state.config.public_host(),
        }
    }
}

impl FromRequestParts<AppState> for Visitor {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        let text = |name: HeaderName| parts.headers.get(name).and_then(|v| v.to_str().ok());
        let peer = parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|info| info.0.ip());
        Ok(Self {
            client: client_key(state.config.trust_proxy, &parts.headers, peer),
            detected: device::classify(text(header::USER_AGENT).unwrap_or("")),
            accept: text(header::ACCEPT_LANGUAGE).and_then(Locale::from_accept_language),
        })
    }
}

/// The rate-limit key of a client.
///
/// The peer address, or — only with `EBC_TRUST_PROXY=1` — the **last** entry of
/// `X-Forwarded-For`: the one appended by the trusted reverse proxy. Earlier entries are
/// whatever the client sent and would let it pick a fresh identity for every guess.
/// Addresses are grouped by network (IPv4 /24, IPv6 /56) so that one subscriber line is
/// one client.
pub(crate) fn client_key(trust_proxy: bool, headers: &HeaderMap, peer: Option<IpAddr>) -> String {
    let forwarded = trust_proxy
        .then(|| {
            headers
                .get_all("x-forwarded-for")
                .iter()
                .next_back()
                .and_then(|value| value.to_str().ok())
                .and_then(|list| list.rsplit(',').next())
                .and_then(|entry| entry.trim().parse::<IpAddr>().ok())
        })
        .flatten();
    let address = forwarded.or(peer).map(|ip| match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(IpAddr::V6(v6), IpAddr::V4),
        v4 @ IpAddr::V4(_) => v4,
    });
    match address {
        Some(IpAddr::V4(v4)) => {
            let [a, b, c, _] = v4.octets();
            format!("{a}.{b}.{c}.0/24")
        }
        Some(IpAddr::V6(v6)) => {
            let [a, b, c, d, ..] = v6.segments();
            format!("{a:x}:{b:x}:{c:x}:{:x}::/56", d & 0xff00)
        }
        None => "unknown".to_owned(),
    }
}

/// An HTML page with its `Content-Security-Policy`: no script, styles and images from
/// this origin only, forms posting to this origin — plus `form_action`, the store origin
/// a reseller gift redirects to after its form is posted.
pub(crate) fn html(status: StatusCode, body: String, form_action: Option<&str>) -> Response {
    let csp = format!(
        "default-src 'none'; style-src 'self'; img-src 'self'; form-action 'self'{}{}; \
         base-uri 'none'; frame-ancestors 'none'",
        if form_action.is_some() { " " } else { "" },
        form_action.unwrap_or(""),
    );
    let mut response = (status, body).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(value) = HeaderValue::from_str(&csp) {
        headers.insert(header::CONTENT_SECURITY_POLICY, value);
    }
    response
}

/// `303 See Other` to `location` (which must be ASCII; our URLs are).
pub(crate) fn see_other(location: &str) -> Response {
    match HeaderValue::from_str(location) {
        Ok(value) => (StatusCode::SEE_OTHER, [(header::LOCATION, value)]).into_response(),
        Err(_) => ApiError::internal("redirect target is not a valid header value").into_response(),
    }
}

/// Adds a `Retry-After` header.
pub(crate) fn with_retry_after(mut response: Response, seconds: u64) -> Response {
    response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from(seconds.max(1)));
    response
}

/// A JSON response.
pub(crate) fn json(status: StatusCode, value: &serde_json::Value) -> Response {
    let mut response = (status, value.to_string()).into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

/// A request body read into memory.
pub(crate) struct ReadBody {
    /// The bytes.
    pub(crate) bytes: Bytes,
    /// Their SHA-256, computed while they arrived.
    pub(crate) sha256: [u8; 32],
    /// The share of the in-flight body budget the bytes hold until this is dropped.
    _budget: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl ReadBody {
    /// The SHA-256 as lower-case hexadecimal.
    pub(crate) fn sha256_hex(&self) -> String {
        crate::codes::hex(&self.sha256)
    }
}

/// Reads a request body of at most `limit` bytes (`413 payload_too_large` beyond),
/// hashing it on the way.
///
/// Every byte held counts against the in-flight body budget shared by all requests
/// (`EBC_MAX_INFLIGHT_BODY_MB`): when it is spent the request is answered `503
/// server_busy` at once instead of being buffered. A body that stalls longer than
/// `EBC_BODY_IDLE_TIMEOUT_SECS` is answered `408`.
pub(crate) async fn read_body(
    state: &AppState,
    headers: &HeaderMap,
    mut body: Body,
    limit: usize,
) -> Result<ReadBody, ApiError> {
    use axum::body::HttpBody as _;
    use sha2::Digest as _;

    let too_large = || {
        ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            format!("the body may not exceed {limit} bytes"),
        )
    };
    let busy = || {
        let mut error = ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "server_busy",
            "too many request bodies are being received; try again shortly",
        );
        error.retry_after = Some(5);
        error
    };
    let declared = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    if declared.is_some_and(|length| length > limit as u64) {
        return Err(too_large());
    }
    let mut budget: Option<tokio::sync::OwnedSemaphorePermit> = None;
    let mut reserved = 0usize;
    let mut reserve = |total: usize| -> Result<(), ApiError> {
        let Some(needed) = total.checked_sub(reserved).filter(|n| *n > 0) else {
            return Ok(());
        };
        let permits = u32::try_from(needed).map_err(|_| too_large())?;
        let permit =
            Arc::clone(&state.body_budget).try_acquire_many_owned(permits).map_err(|_| busy())?;
        match &mut budget {
            Some(held) => held.merge(permit),
            None => budget = Some(permit),
        }
        reserved = total;
        Ok(())
    };
    // Room for the announced length is taken before anything is read.
    let announced = declared.and_then(|n| usize::try_from(n).ok()).unwrap_or(0);
    reserve(announced)?;
    let mut buffer = Vec::with_capacity(announced);
    let mut hasher = sha2::Sha256::new();
    while let Some(frame) = std::future::poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await {
        let frame = frame.map_err(|error| {
            let timed_out = error.into_inner().is::<tower_http::timeout::TimeoutError>();
            if timed_out {
                ApiError::new(
                    StatusCode::REQUEST_TIMEOUT,
                    "request_timeout",
                    "the request body stalled",
                )
            } else {
                ApiError::bad_request("body_unreadable", "the request body could not be read")
            }
        })?;
        let Ok(data) = frame.into_data() else { continue };
        let total = buffer.len().saturating_add(data.len());
        if total > limit {
            return Err(too_large());
        }
        reserve(total)?;
        hasher.update(&data);
        buffer.extend_from_slice(&data);
    }
    Ok(ReadBody { bytes: Bytes::from(buffer), sha256: hasher.finalize().into(), _budget: budget })
}

/// Parses a JSON body: `400 invalid_json` for text that is not JSON, `422
/// invalid_request` for JSON that does not fit (missing member, invalid ISBN…).
pub(crate) fn parse_json<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body).map_err(|error| match error.classify() {
        serde_json::error::Category::Data => {
            ApiError::unprocessable("invalid_request", error.to_string())
        }
        _ => ApiError::bad_request("invalid_json", error.to_string()),
    })
}

/// A header as text.
pub(crate) fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok()).map(str::trim).filter(|v| !v.is_empty())
}

/// The members of the answer to a gift creation or a reissue.
pub(crate) fn code_members(
    state: &AppState,
    code: &GiftCode,
) -> serde_json::Map<String, serde_json::Value> {
    let mut members = serde_json::Map::new();
    members.insert("code".to_owned(), code.compact().into());
    members.insert("code_display".to_owned(), code.grouped().into());
    members.insert(
        "claim_url".to_owned(),
        ebc_model::claim_url(&state.config.public_url, code).into(),
    );
    members
}

/// The status document of a gift (`docs/api.md` §3; `order_ref` for partner gifts).
pub(crate) fn status_json(state: &AppState, gift: &crate::store::GiftRecord) -> serde_json::Value {
    let mut value = serde_json::json!({
        "id": gift.id,
        "status": gift.status_at(state.now()).as_str(),
        "created_at": gift.created_at,
        "redeemed_at": gift.redeemed_at,
        "downloads": gift.claims,
    });
    if let (Some(order_ref), Some(object)) = (&gift.order_ref, value.as_object_mut()) {
        object.insert("order_ref".to_owned(), order_ref.clone().into());
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forwarded(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static(value));
        headers
    }

    #[test]
    fn codes_and_tokens_never_reach_the_logs() {
        let code = GiftCode::from_payload(123_456_789).map(|c| c.compact()).unwrap_or_default();
        let token = "0123456789abcdef0123456789abcdef";
        let cases = [
            (format!("/k/{code}"), "/k/…".to_owned()),
            (format!("/K/{}/download", code.to_lowercase()), "/K/…/download".to_owned()),
            ("/k/not-even-a-code/pair".to_owned(), "/k/…/pair".to_owned()),
            (format!("/{code}"), "/…".to_owned()),
            (format!("/d/{token}/Le-Horla.epub"), "/d/…/Le-Horla.epub".to_owned()),
            (format!("/o/{token}/catalog.xml"), "/o/…/catalog.xml".to_owned()),
            (format!("/x/{token}"), "/x/…".to_owned()),
            (
                "/api/v1/gifts/g_fedcba9876543210fedcba9876543210".to_owned(),
                "/api/v1/gifts/g_fedcba9876543210fedcba9876543210".to_owned(),
            ),
            ("/".to_owned(), "/".to_owned()),
            ("/aide".to_owned(), "/aide".to_owned()),
            ("/k".to_owned(), "/k".to_owned()),
        ];
        for (path, expected) in cases {
            let redacted = redact_path(&path);
            assert_eq!(redacted, expected, "{path}");
            assert!(!redacted.contains(&code) && !redacted.contains(token));
        }
        assert!(redact_path(&format!("/{}\r\nfake log line", "a".repeat(500))).len() <= 70);
    }

    #[test]
    fn clients_are_grouped_by_network() {
        let none = HeaderMap::new();
        let peer = |text: &str| text.parse::<IpAddr>().ok();
        assert_eq!(client_key(false, &none, peer("203.0.113.77")), "203.0.113.0/24");
        assert_eq!(client_key(false, &none, peer("203.0.113.1")), "203.0.113.0/24");
        assert_eq!(client_key(false, &none, peer("203.0.114.1")), "203.0.114.0/24");
        assert_eq!(
            client_key(false, &none, peer("2001:db8:aa:bbcc:1::1")),
            "2001:db8:aa:bb00::/56"
        );
        assert_eq!(
            client_key(false, &none, peer("2001:db8:aa:bbdd:ffff::9")),
            "2001:db8:aa:bb00::/56"
        );
        assert_eq!(client_key(false, &none, peer("::ffff:198.51.100.7")), "198.51.100.0/24");
        assert_eq!(client_key(false, &none, None), "unknown");
    }

    #[test]
    fn forwarded_for_is_ignored_unless_the_proxy_is_trusted_and_then_only_its_own_entry_counts() {
        let peer = "10.0.0.1".parse::<IpAddr>().ok();
        let spoofed = forwarded("1.2.3.4, 5.6.7.8, 198.51.100.9");
        // Not trusted: the header is the client's to forge.
        assert_eq!(client_key(false, &spoofed, peer), "10.0.0.0/24");
        // Trusted: the entry the proxy appended, never the ones the client prepended.
        assert_eq!(client_key(true, &spoofed, peer), "198.51.100.0/24");
        assert_eq!(client_key(true, &forwarded("198.51.100.9"), peer), "198.51.100.0/24");
        // Garbage or a missing header falls back to the peer.
        assert_eq!(client_key(true, &forwarded("not-an-address"), peer), "10.0.0.0/24");
        assert_eq!(client_key(true, &forwarded("1.2.3.4, <script>"), peer), "10.0.0.0/24");
        assert_eq!(client_key(true, &HeaderMap::new(), peer), "10.0.0.0/24");
    }
}
