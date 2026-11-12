//! The HTTP surface: the router, the security headers every response carries, the
//! request extractor ([`Visitor`]) and helpers shared by the handlers.

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, DefaultBodyLimit, FromRequestParts, Request, State};
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use ebc_model::Locale;
use percent_encoding::AsciiSet;

use crate::crypto;
use crate::pages::{self, Chrome};
use crate::state::AppState;
use crate::store::Account;
use crate::text::{S, s, sf};

mod account;
mod auth;
mod books;
mod delivery;
mod notices;

/// Largest urlencoded form accepted.
const MAX_FORM_BYTES: usize = 64 * 1024;

/// Characters escaped in a file name used as one URL path segment.
pub(crate) const PATH_SEGMENT: &AsciiSet =
    &percent_encoding::NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'.');

const X_ROBOTS_TAG: HeaderName = HeaderName::from_static("x-robots-tag");

/// `Content-Security-Policy` of every response: no script, nothing from elsewhere, forms
/// only to this origin.
const CSP: &str = "default-src 'none'; style-src 'self'; img-src 'self'; form-action 'self'; \
    frame-ancestors 'none'; base-uri 'none'";

/// Builds the service. Mount it with connection information
/// (`into_make_service_with_connect_info::<SocketAddr>()`, or [`crate::serve`]) so that
/// clients are told apart; without it every client shares one rate-limit bucket.
pub fn app(state: AppState) -> Router {
    let upload_limit =
        usize::try_from(state.config.max_file_bytes.saturating_add(1 << 20)).unwrap_or(usize::MAX);
    let request_timeout = Duration::from_secs(state.config.request_timeout_secs.max(1));
    let body_idle_timeout = Duration::from_secs(state.config.body_idle_timeout_secs.max(1));
    let upload = Router::new()
        .route("/livres", post(books::upload))
        .layer(DefaultBodyLimit::max(upload_limit));
    Router::new()
        .route("/", get(account::home))
        .route("/healthz", get(|| async { "ok" }))
        .route("/static/shelf.css", get(style))
        .route("/connexion", get(auth::sign_in_page).post(auth::sign_in))
        .route("/connexion/{token}", get(auth::confirm_page).post(auth::confirm))
        .route("/deconnexion", post(auth::sign_out))
        .route("/compte", get(account::account))
        .route("/compte/identite", get(account::identity_page).post(account::identity_save))
        .route("/compte/fermer", post(account::close))
        .route("/compte/decisions/{id}", get(account::decision_page).post(account::appeal))
        .route("/livres/nouveau", get(books::upload_page))
        .route("/livres/{id}", get(books::book_page))
        .route("/livres/{id}/cadeaux", post(books::create_gifts))
        .route("/livres/{id}/supprimer", post(books::delete_book))
        .route("/cadeaux/{id}/annuler", post(books::cancel_gift))
        .route("/cadeaux/{id}/nouveau-code", post(books::reissue_gift))
        .route("/cadeaux/{id}/reessayer", post(books::retry_gift))
        .route("/cadeaux/{id}/carte.pdf", get(books::card))
        .route("/signaler", get(notices::notice_page).post(notices::notice))
        .route("/moderation", get(notices::moderation))
        .route(
            "/moderation/signalements/{id}",
            get(notices::moderation_notice).post(notices::decide),
        )
        .route("/moderation/reclamations/{id}", post(notices::decide_appeal))
        .route("/conditions", get(account::terms))
        .route("/mentions-legales", get(account::legal))
        .route("/contact", get(account::contact))
        .route("/api/ebc/link", get(delivery::link))
        .route("/f/{token}/{name}", get(delivery::file))
        .layer(DefaultBodyLimit::max(MAX_FORM_BYTES))
        .merge(upload)
        .fallback(fallback)
        .layer(tower_http::timeout::RequestBodyTimeoutLayer::new(body_idle_timeout))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        .layer(axum::middleware::from_fn_with_state(state.clone(), same_origin_posts))
        .layer(axum::middleware::from_fn(language_cookie))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::http::Request<Body>| {
                // Sign-in tokens and download tokens are secrets: they never reach the logs.
                // Neither the query string (the core's signed parameters) nor any header
                // (`Referer` may carry a gift code) is logged.
                let path = redact_path(request.uri().path());
                tracing::debug_span!("request", method = %request.method(), %path)
            },
        ))
        .with_state(state)
}

/// A request path fit for logs: the segment after `/connexion/` and `/f/` is replaced.
pub(crate) fn redact_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len().min(256));
    let mut secret_next = false;
    for (index, segment) in path.split('/').enumerate() {
        if index > 0 {
            out.push('/');
        }
        if secret_next && !segment.is_empty() {
            out.push('…');
        } else {
            out.extend(segment.chars().filter(|c| !c.is_control()).take(64));
        }
        secret_next =
            (index == 1 && matches!(segment, "connexion" | "f")) || (secret_next && index == 2);
    }
    out
}

async fn style() -> Response {
    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/css; charset=utf-8")),
            (header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400")),
        ],
        pages::STYLE,
    )
        .into_response()
}

/// Headers every response carries unless it set its own.
async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    let mut default = |name: HeaderName, value: &'static str| {
        headers.entry(name).or_insert(HeaderValue::from_static(value));
    };
    default(header::CONTENT_SECURITY_POLICY, CSP);
    default(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    default(header::REFERRER_POLICY, "no-referrer");
    default(header::X_FRAME_OPTIONS, "DENY");
    default(header::CACHE_CONTROL, "no-store");
    default(X_ROBOTS_TAG, "noindex, nofollow");
    response
}

/// `?lang=xx` on any page also remembers the choice in a cookie.
async fn language_cookie(request: Request, next: Next) -> Response {
    let chosen = request.uri().query().and_then(lang_param);
    let mut response = next.run(request).await;
    if let Some(locale) = chosen
        && let Ok(value) = HeaderValue::from_str(&format!(
            "ebc_lang={}; Path=/; Max-Age=31536000; SameSite=Lax",
            locale.code()
        ))
    {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

fn lang_param(query: &str) -> Option<Locale> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == "lang")
        .and_then(|(_, value)| Locale::from_tag(value))
        .filter(|l| Locale::ALL.contains(l))
}

/// A `POST` whose `Origin` names another site is refused (on top of the `SameSite=Strict`
/// session cookie and the form token).
async fn same_origin_posts(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() == Method::POST
        && let Some(origin) = request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok())
        && origin != "null"
        && origin.trim_end_matches('/') != state.config.public_url
    {
        return (StatusCode::FORBIDDEN, "cross-site request refused").into_response();
    }
    next.run(request).await
}

/// Anything that is not a route.
async fn fallback(State(state): State<AppState>, visitor: Visitor) -> Response {
    not_found(&state, &visitor)
}

/// Who is asking.
#[derive(Debug, Clone)]
pub(crate) struct Visitor {
    /// Language of the page.
    pub(crate) locale: Locale,
    /// Rate-limit key: the client's network (IPv4 /24, IPv6 /56).
    pub(crate) client: String,
    /// The client's address (kept with content creation, LCEN).
    pub(crate) ip: Option<String>,
    /// The session cookie value.
    pub(crate) session: Option<String>,
    /// The signed-in account.
    pub(crate) account: Option<Account>,
    /// The form token of the session.
    pub(crate) csrf: Option<String>,
    /// Path of the request (language links).
    pub(crate) path: String,
}

impl Visitor {
    /// The page context.
    pub(crate) fn chrome<'a>(&'a self, state: &'a AppState) -> Chrome<'a> {
        Chrome {
            locale: self.locale,
            config: &state.config,
            account: self.account.as_ref(),
            csrf: self.csrf.as_deref(),
            path: &self.path,
        }
    }

    /// Whether the form token matches this session.
    pub(crate) fn csrf_ok(&self, token: Option<&str>) -> bool {
        match (&self.csrf, token) {
            (Some(expected), Some(given)) => crypto::ct_eq(expected.as_bytes(), given.as_bytes()),
            _ => false,
        }
    }
}

/// The session cookie's name: `__Host-` prefixed over HTTPS.
pub(crate) fn session_cookie_name(state: &AppState) -> &'static str {
    if state.config.is_https() { "__Host-ebc_shelf" } else { "ebc_shelf" }
}

fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(n, _)| *n == name)
        .map(|(_, value)| value)
}

impl FromRequestParts<AppState> for Visitor {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        let peer = parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|info| info.0.ip());
        let address = client_address(state.config.trust_proxy, &parts.headers, peer);
        let session = cookie(&parts.headers, session_cookie_name(state))
            .filter(|v| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit()))
            .map(str::to_owned);
        let account = session.as_deref().and_then(|s| state.session_account(s));
        let session = session.filter(|_| account.is_some());
        let csrf = session.as_deref().map(|s| state.csrf_token(s));
        let locale = parts
            .uri
            .query()
            .and_then(lang_param)
            .or_else(|| cookie(&parts.headers, "ebc_lang").and_then(Locale::from_tag))
            .or_else(|| account.as_ref().and_then(|a| Locale::from_tag(&a.locale)))
            .or_else(|| {
                parts
                    .headers
                    .get(header::ACCEPT_LANGUAGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(Locale::from_accept_language)
            })
            .unwrap_or_default();
        Ok(Self {
            locale,
            client: network_key(address),
            ip: address.map(|a| a.to_string()),
            session,
            account,
            csrf,
            path: parts.uri.path().chars().take(200).collect(),
        })
    }
}

/// The client's address: the peer, or — only with `EBC_SHELF_TRUST_PROXY=1` — the last
/// `X-Forwarded-For` entry (the one the trusted proxy appended).
fn client_address(trust_proxy: bool, headers: &HeaderMap, peer: Option<IpAddr>) -> Option<IpAddr> {
    let forwarded = trust_proxy
        .then(|| {
            headers
                .get_all("x-forwarded-for")
                .iter()
                .next_back()
                .and_then(|v| v.to_str().ok())
                .and_then(|list| list.rsplit(',').next())
                .and_then(|entry| entry.trim().parse::<IpAddr>().ok())
        })
        .flatten();
    forwarded.or(peer).map(|ip| match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(IpAddr::V6(v6), IpAddr::V4),
        v4 @ IpAddr::V4(_) => v4,
    })
}

/// IPv4 /24 or IPv6 /56: one subscriber line is one client.
pub(crate) fn network_key(address: Option<IpAddr>) -> String {
    match address {
        Some(IpAddr::V4(v4)) => {
            let [a, b, c, _] = v4.octets();
            format!("{a}.{b}.{c}.0/24")
        }
        Some(IpAddr::V6(v6)) => {
            let seg = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/56", seg[0], seg[1], seg[2], seg[3] & 0xff00)
        }
        None => "unknown".to_owned(),
    }
}

/// An HTML page.
pub(crate) fn page(status: StatusCode, html: String) -> Response {
    (status, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

/// `303 See Other`.
pub(crate) fn see_other(location: &str) -> Response {
    let mut response = StatusCode::SEE_OTHER.into_response();
    if let Ok(value) = HeaderValue::from_str(location) {
        response.headers_mut().insert(header::LOCATION, value);
    }
    response
}

pub(crate) fn not_found(state: &AppState, visitor: &Visitor) -> Response {
    let text = s(visitor.locale, S::NotFound);
    page(
        StatusCode::NOT_FOUND,
        pages::message(visitor.chrome(state), text, &[], Some(("/", &state.config.name))),
    )
}

pub(crate) fn forbidden(state: &AppState, visitor: &Visitor) -> Response {
    let text = s(visitor.locale, S::Forbidden);
    page(
        StatusCode::FORBIDDEN,
        pages::message(visitor.chrome(state), text, &[], Some(("/", &state.config.name))),
    )
}

pub(crate) fn bad_form(state: &AppState, visitor: &Visitor) -> Response {
    let text = s(visitor.locale, S::BadForm);
    page(
        StatusCode::FORBIDDEN,
        pages::message(
            visitor.chrome(state),
            text,
            &[],
            Some(("/compte", s(visitor.locale, S::NavAccount))),
        ),
    )
}

pub(crate) fn server_error(state: &AppState, visitor: &Visitor) -> Response {
    let text = s(visitor.locale, S::ServerError);
    page(StatusCode::INTERNAL_SERVER_ERROR, pages::message(visitor.chrome(state), text, &[], None))
}

pub(crate) fn too_many(state: &AppState, visitor: &Visitor, wait: i64) -> Response {
    let text = sf(visitor.locale, S::TooMany, &[("minutes", &(wait / 60 + 1).to_string())]);
    let mut response = page(
        StatusCode::TOO_MANY_REQUESTS,
        pages::message(visitor.chrome(state), &text, &[], None),
    );
    if let Ok(value) = HeaderValue::from_str(&wait.max(1).to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_never_reach_the_logs() {
        let token = "ab".repeat(32);
        for (path, expected) in [
            (format!("/connexion/{token}"), "/connexion/…".to_owned()),
            (format!("/f/{token}/le-horla.epub"), "/f/…/…".to_owned()),
            ("/livres/b_0123".to_owned(), "/livres/b_0123".to_owned()),
            ("/connexion".to_owned(), "/connexion".to_owned()),
        ] {
            assert_eq!(redact_path(&path), expected);
        }
        assert!(redact_path(&format!("/{}\r\nfake", "a".repeat(500))).len() <= 70);
    }

    #[test]
    fn clients_are_grouped_by_network_and_forwarding_is_opt_in() {
        let peer = "203.0.113.77".parse::<IpAddr>().ok();
        assert_eq!(network_key(client_address(false, &HeaderMap::new(), peer)), "203.0.113.0/24");
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("1.2.3.4, 198.51.100.9"));
        assert_eq!(network_key(client_address(false, &headers, peer)), "203.0.113.0/24");
        assert_eq!(network_key(client_address(true, &headers, peer)), "198.51.100.0/24");
        let v6 = "2001:db8:aa:bbcc::1".parse::<IpAddr>().ok();
        assert_eq!(network_key(v6), "2001:db8:aa:bb00::/56");
        assert_eq!(network_key(None), "unknown");
    }

    #[test]
    fn language_parameters_are_checked() {
        assert_eq!(lang_param("lang=en"), Some(Locale::En));
        assert_eq!(lang_param("x=1&lang=de"), Some(Locale::De));
        assert_eq!(lang_param("lang=xx"), None);
        assert_eq!(lang_param("language=en"), None);
    }
}
