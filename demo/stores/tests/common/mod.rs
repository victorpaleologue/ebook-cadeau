//! Test harness: a mock eBook Cadeau partner API that implements `docs/api.md` §4 strictly
//! (signatures, timestamps, idempotency), the real demo printer in manual mode, the stores
//! on ephemeral ports, and a tiny cookie-keeping "browser". Everything runs in-process on
//! the loopback interface; nothing leaves the machine.

#![allow(dead_code, clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use ebc_demo_printer::testkit::SampleCard;
use ebc_demo_printer::{PrinterConfig, PrinterState};
use ebc_demo_stores::{AppState, Clock, Entropy, StoreConfig, StoreKind};
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// The instant every test starts at: 2026-11-12T00:00:00Z.
pub const T0: i64 = 1_794_441_600;

pub async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// HMAC-SHA-256 written against the `hmac` crate directly, on purpose independent of the
/// helper the stores use, so that both sides of each signature are computed by different
/// code.
fn hmac_hex(secret: &str, message: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    hex(&mac.finalize().into_bytes())
}

// ----- mock eBook Cadeau partner API ------------------------------------------------------

/// One request the mock received, after (or despite) authentication.
#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    /// Status the mock answered with.
    pub status: u16,
}

impl Recorded {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
}

#[derive(Debug, Clone)]
pub struct MockGift {
    pub id: String,
    pub code: String,
    pub order_ref: String,
    pub partner: String,
    pub request: Value,
    pub status: String,
    pub redeemed_at: Option<i64>,
    pub downloads: u64,
    pub cover: Option<Vec<u8>>,
    pub fulfilled_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MockUpload {
    pub id: String,
    pub bytes: Vec<u8>,
    pub file_name: String,
    pub media_type: String,
}

/// How the mock answers `card.pdf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardMode {
    /// A sample card for the printer product with this id.
    Sample(&'static str),
    /// `501 Not Implemented`.
    NotImplemented,
}

/// A scripted failure: the next `remaining` requests whose path ends with `suffix` are
/// answered with `status`. With `after_processing`, the request is processed first and
/// only the answer is lost.
#[derive(Debug, Clone)]
struct FailRule {
    method: String,
    suffix: String,
    status: u16,
    after_processing: bool,
    remaining: u32,
}

#[derive(Default)]
struct EbcInner {
    partners: HashMap<String, String>,
    uploads: Vec<MockUpload>,
    gifts: Vec<MockGift>,
    idempotency: HashMap<(String, String), (String, u16, Value)>,
    log: Vec<Recorded>,
    card_mode: Option<CardMode>,
    fail: Vec<FailRule>,
}

#[derive(Clone)]
pub struct MockEbc {
    pub base: String,
    inner: Arc<Mutex<EbcInner>>,
    clock: Arc<AtomicI64>,
}

fn error(status: StatusCode, code: &str, message: &str) -> Response {
    (status, axum::Json(json!({"error": {"code": code, "message": message}}))).into_response()
}

impl MockEbc {
    pub async fn start(clock: Arc<AtomicI64>, partners: &[(&str, &str)]) -> Self {
        let inner = Arc::new(Mutex::new(EbcInner {
            partners: partners.iter().map(|(id, s)| ((*id).to_owned(), (*s).to_owned())).collect(),
            ..EbcInner::default()
        }));
        let mut mock = Self { base: String::new(), inner, clock };
        mock.base = serve(Router::new().fallback(handle).with_state(mock.clone())).await;
        mock
    }

    fn now(&self) -> i64 {
        self.clock.load(Ordering::SeqCst)
    }

    pub fn set_card_mode(&self, mode: CardMode) {
        self.inner.lock().unwrap().card_mode = Some(mode);
    }

    /// Scripts failures: see [`FailRule`].
    pub fn fail_next(
        &self,
        method: &str,
        suffix: &str,
        status: u16,
        times: u32,
        after_processing: bool,
    ) {
        self.inner.lock().unwrap().fail.push(FailRule {
            method: method.to_owned(),
            suffix: suffix.to_owned(),
            status,
            after_processing,
            remaining: times,
        });
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.inner.lock().unwrap().log.clone()
    }

    /// Requests whose method matches and whose path ends with `suffix`.
    pub fn requests_to(&self, method: &str, suffix: &str) -> Vec<Recorded> {
        self.requests()
            .into_iter()
            .filter(|r| r.method == method && r.path.ends_with(suffix))
            .collect()
    }

    pub fn gifts(&self) -> Vec<MockGift> {
        self.inner.lock().unwrap().gifts.clone()
    }

    pub fn gift(&self, id: &str) -> MockGift {
        self.gifts().into_iter().find(|g| g.id == id).unwrap()
    }

    pub fn uploads(&self) -> Vec<MockUpload> {
        self.inner.lock().unwrap().uploads.clone()
    }

    /// Simulates what happens on the eBook Cadeau side when the recipient uses the card.
    pub fn set_status(&self, gift_id: &str, status: &str, downloads: u64) {
        let now = self.now();
        let mut inner = self.inner.lock().unwrap();
        let gift = inner.gifts.iter_mut().find(|g| g.id == gift_id).unwrap();
        gift.status = status.to_owned();
        gift.downloads = downloads;
        if status != "created" && gift.redeemed_at.is_none() {
            gift.redeemed_at = Some(now);
        }
    }

    /// The URL the recipient's browser is sent to when pressing "redeem" (§4.3).
    pub fn redirect_url(&self, gift_id: &str) -> String {
        self.redirect_url_at(gift_id, self.now())
    }

    pub fn redirect_url_at(&self, gift_id: &str, ts: i64) -> String {
        let gift = self.gift(gift_id);
        let secret = self.inner.lock().unwrap().partners[&gift.partner].clone();
        let url = gift.request["delivery"]["url"].as_str().unwrap().to_owned();
        let sig = hmac_hex(&secret, &format!("{}.{}.{ts}", gift.id, gift.order_ref));
        format!("{url}?ebc_gift={}&ebc_order={}&ebc_ts={ts}&ebc_sig={sig}", gift.id, gift.order_ref)
    }

    /// Sends a signed webhook to a store (§4.4) and returns the store's answer.
    pub async fn send_webhook(
        &self,
        store_base: &str,
        partner: &str,
        event: &Value,
    ) -> (StatusCode, Value) {
        let secret = self.inner.lock().unwrap().partners[partner].clone();
        let body = serde_json::to_vec(event).unwrap();
        let ts = self.now();
        let signature = sign_request(&secret, ts, "POST", "/webhooks/ebc", &body);
        post_webhook(store_base, Some(&ts.to_string()), Some(&signature), body).await
    }
}

/// `X-EBC-Signature` per §4.1.
pub fn sign_request(
    secret: &str,
    ts: i64,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> String {
    let message = format!("{ts}.{method}.{path_and_query}.{}", hex(&Sha256::digest(body)));
    format!("v1={}", hmac_hex(secret, &message))
}

pub async fn post_webhook(
    store_base: &str,
    timestamp: Option<&str>,
    signature: Option<&str>,
    body: Vec<u8>,
) -> (StatusCode, Value) {
    let mut request = plain_client()
        .post(format!("{store_base}/webhooks/ebc"))
        .header("content-type", "application/json")
        .body(body);
    if let Some(timestamp) = timestamp {
        request = request.header("X-EBC-Timestamp", timestamp);
    }
    if let Some(signature) = signature {
        request = request.header("X-EBC-Signature", signature);
    }
    let response = request.send().await.unwrap();
    let status = response.status();
    (status, response.json().await.unwrap_or(Value::Null))
}

async fn handle(
    State(mock): State<MockEbc>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let response = respond(&mock, &method, &uri, &headers, &body);
    let recorded = Recorded {
        method: method.to_string(),
        path: uri.path().to_owned(),
        query: uri.query().map(str::to_owned),
        headers: headers
            .iter()
            .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap_or("").to_owned()))
            .collect(),
        body: body.to_vec(),
        status: response.status().as_u16(),
    };
    mock.inner.lock().unwrap().log.push(recorded);
    response
}

fn respond(
    mock: &MockEbc,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    body: &[u8],
) -> Response {
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    let mut inner = mock.inner.lock().unwrap();

    // --- §4.1 authentication, checked exactly as documented ---
    let Some(partner) = header("x-ebc-partner").filter(|p| inner.partners.contains_key(*p)) else {
        return error(StatusCode::UNAUTHORIZED, "unknown_partner", "unknown X-EBC-Partner");
    };
    let partner = partner.to_owned();
    let secret = inner.partners[&partner].clone();
    let Some(timestamp) = header("x-ebc-timestamp").and_then(|t| t.parse::<i64>().ok()) else {
        return error(StatusCode::UNAUTHORIZED, "bad_signature", "missing timestamp");
    };
    if (mock.now() - timestamp).abs() > 300 {
        return error(StatusCode::UNAUTHORIZED, "stale_timestamp", "timestamp outside ±300 s");
    }
    let path_and_query = uri.path_and_query().map_or_else(|| uri.path(), |pq| pq.as_str());
    let expected = sign_request(&secret, timestamp, method.as_str(), path_and_query, body);
    if header("x-ebc-signature") != Some(expected.as_str()) {
        return error(StatusCode::UNAUTHORIZED, "bad_signature", "signature mismatch");
    }

    // --- scripted failures ---
    let path = uri.path().to_owned();
    let rule = inner
        .fail
        .iter_mut()
        .find(|r| r.remaining > 0 && r.method == method.as_str() && path.ends_with(&r.suffix))
        .map(|r| {
            r.remaining -= 1;
            (r.status, r.after_processing)
        });
    if let Some((status, false)) = rule {
        return error(
            StatusCode::from_u16(status).unwrap(),
            "scripted_failure",
            "scripted failure",
        );
    }

    // --- idempotency (POST only) ---
    let body_hash = hex(&Sha256::digest(body));
    let idempotency_key =
        header("idempotency-key").filter(|_| *method == Method::POST).map(str::to_owned);
    if let Some(key) = &idempotency_key
        && let Some((hash, status, stored)) = inner.idempotency.get(&(partner.clone(), key.clone()))
    {
        return if *hash == body_hash {
            (StatusCode::from_u16(*status).unwrap(), axum::Json(stored.clone())).into_response()
        } else {
            error(StatusCode::CONFLICT, "idempotency_conflict", "same key, different body")
        };
    }

    let (status, value, raw) = route(mock, &mut inner, &partner, method, uri, headers, body);
    if let (Some(key), true, None) = (&idempotency_key, status.is_success(), &raw) {
        inner
            .idempotency
            .insert((partner, key.clone()), (body_hash, status.as_u16(), value.clone()));
    }
    if let Some((status, true)) = rule {
        return error(StatusCode::from_u16(status).unwrap(), "scripted_failure", "answer lost");
    }
    match raw {
        Some(bytes) => (status, [("content-type", "application/pdf")], bytes).into_response(),
        None => (status, axum::Json(value)).into_response(),
    }
}

fn fail(status: StatusCode, code: &str) -> (StatusCode, Value, Option<Vec<u8>>) {
    (status, json!({"error": {"code": code, "message": code}}), None)
}

/// The partner API proper. Returns `(status, JSON body, raw body)`.
fn route(
    mock: &MockEbc,
    inner: &mut EbcInner,
    partner: &str,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    body: &[u8],
) -> (StatusCode, Value, Option<Vec<u8>>) {
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    let segments: Vec<&str> = uri.path().trim_start_matches('/').split('/').collect();
    let Some(rest) = segments.strip_prefix(&["api", "partner", "v1"]) else {
        return fail(StatusCode::NOT_FOUND, "not_found");
    };
    match (method.as_str(), rest) {
        ("POST", ["uploads"]) => {
            if header("x-rights-basis") != Some("licensed_distributor") {
                return fail(StatusCode::BAD_REQUEST, "rights_basis_required");
            }
            let Some(file_name) = header("x-file-name").filter(|n| !n.is_empty()) else {
                return fail(StatusCode::BAD_REQUEST, "file_name_required");
            };
            // Sniffed, not trusted from the header: an EPUB is a ZIP whose first entry is
            // the stored `mimetype`.
            if body.len() < 58 || &body[30..58] != b"mimetypeapplication/epub+zip" {
                return fail(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
            }
            let id = format!("up_{:04}", inner.uploads.len() + 1);
            inner.uploads.push(MockUpload {
                id: id.clone(),
                bytes: body.to_vec(),
                file_name: file_name.to_owned(),
                media_type: header("content-type").unwrap_or("").to_owned(),
            });
            let receipt = json!({
                "upload_id": id, "sha256": hex(&Sha256::digest(body)),
                "size_bytes": body.len(), "media_type": "application/epub+zip",
            });
            (StatusCode::CREATED, receipt, None)
        }
        ("POST", ["gifts"]) => {
            let Ok(request) = serde_json::from_slice::<Value>(body) else {
                return fail(StatusCode::BAD_REQUEST, "invalid_json");
            };
            let text =
                |pointer: &str| request.pointer(pointer).and_then(Value::as_str).unwrap_or("");
            if text("/order_ref").is_empty() || text("/book/title").is_empty() {
                return fail(StatusCode::UNPROCESSABLE_ENTITY, "invalid_gift");
            }
            match text("/delivery/type") {
                "hosted_file" => {
                    let upload = text("/delivery/upload_id");
                    if !inner.uploads.iter().any(|u| u.id == upload) {
                        return fail(StatusCode::UNPROCESSABLE_ENTITY, "unknown_upload");
                    }
                }
                "reseller_redirect" => {
                    if !text("/delivery/url").starts_with("http") {
                        return fail(StatusCode::UNPROCESSABLE_ENTITY, "invalid_delivery");
                    }
                }
                _ => return fail(StatusCode::UNPROCESSABLE_ENTITY, "invalid_delivery"),
            }
            let n = inner.gifts.len() + 1;
            let id = format!("g_{n:04}");
            let code = format!("DEMO{n:05}X");
            inner.gifts.push(MockGift {
                id: id.clone(),
                code: code.clone(),
                order_ref: text("/order_ref").to_owned(),
                partner: partner.to_owned(),
                request: request.clone(),
                status: "created".to_owned(),
                redeemed_at: None,
                downloads: 0,
                cover: None,
                fulfilled_by: None,
            });
            let created = json!({
                "id": id, "code": code, "code_display": format!("{}-{}", &code[..5], &code[5..]),
                "claim_url": format!("{}/k/{code}", mock.base), "status": "created",
            });
            (StatusCode::CREATED, created, None)
        }
        (_, ["gifts", id, tail @ ..]) => {
            let Some(gift) = inner.gifts.iter_mut().find(|g| g.id == *id && g.partner == partner)
            else {
                return fail(StatusCode::NOT_FOUND, "unknown_gift");
            };
            match (method.as_str(), tail) {
                ("GET", []) => {
                    let status = json!({
                        "id": gift.id, "status": gift.status, "created_at": T0,
                        "redeemed_at": gift.redeemed_at, "downloads": gift.downloads,
                        "order_ref": gift.order_ref,
                    });
                    (StatusCode::OK, status, None)
                }
                ("PUT", ["cover"]) => {
                    if !body.starts_with(b"\x89PNG\r\n\x1a\n") && !body.starts_with(b"\xff\xd8") {
                        return fail(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
                    }
                    gift.cover = Some(body.to_vec());
                    (StatusCode::OK, json!({"ok": true}), None)
                }
                ("GET", ["card.pdf"]) => {
                    let title = format!("Carte {}", gift.order_ref);
                    match inner.card_mode.unwrap_or(CardMode::Sample("card-poche")) {
                        CardMode::NotImplemented => {
                            fail(StatusCode::NOT_IMPLEMENTED, "not_implemented")
                        }
                        CardMode::Sample(product) => {
                            let product = ebc_demo_printer::catalog::find(product).unwrap();
                            let card = SampleCard {
                                title: Some(title),
                                ..SampleCard::for_product(product)
                            };
                            (StatusCode::OK, Value::Null, Some(card.to_pdf()))
                        }
                    }
                }
                ("POST", ["cancel"]) => {
                    if gift.status == "redeemed" || gift.status == "fulfilled" {
                        return fail(StatusCode::CONFLICT, "already_redeemed");
                    }
                    gift.status = "cancelled".to_owned();
                    (StatusCode::OK, json!({"id": gift.id, "status": "cancelled"}), None)
                }
                ("POST", ["fulfilled"]) => {
                    let account = serde_json::from_slice::<Value>(body)
                        .ok()
                        .and_then(|v| v["account_ref"].as_str().map(str::to_owned));
                    let Some(account) = account.filter(|a| !a.is_empty()) else {
                        return fail(StatusCode::BAD_REQUEST, "account_ref_required");
                    };
                    match &gift.fulfilled_by {
                        Some(existing) if *existing != account => {
                            fail(StatusCode::CONFLICT, "already_fulfilled")
                        }
                        _ => {
                            gift.fulfilled_by = Some(account);
                            gift.status = "fulfilled".to_owned();
                            if gift.redeemed_at.is_none() {
                                gift.redeemed_at = Some(mock.now());
                            }
                            (StatusCode::OK, json!({"id": gift.id, "status": "fulfilled"}), None)
                        }
                    }
                }
                _ => fail(StatusCode::NOT_FOUND, "not_found"),
            }
        }
        _ => fail(StatusCode::NOT_FOUND, "not_found"),
    }
}

// ----- the world: clock + mock + printer + stores -----------------------------------------

pub const SEPTANTE_SECRET: &str = "secret-septante-for-tests";
pub const GRANDE_SECRET: &str = "secret-grande-enseigne-for-tests";

pub struct World {
    pub clock: Arc<AtomicI64>,
    pub ebc: MockEbc,
}

pub struct Printer {
    pub base: String,
    pub state: Arc<PrinterState>,
}

pub struct Store {
    pub base: String,
    pub state: Arc<AppState>,
    pub kind: StoreKind,
}

impl World {
    pub async fn new() -> Self {
        let clock = Arc::new(AtomicI64::new(T0));
        let ebc = MockEbc::start(
            clock.clone(),
            &[("septante", SEPTANTE_SECRET), ("grande-enseigne", GRANDE_SECRET)],
        )
        .await;
        Self { clock, ebc }
    }

    pub fn advance(&self, seconds: i64) {
        self.clock.fetch_add(seconds, Ordering::SeqCst);
    }

    pub fn store_clock(&self) -> Clock {
        let clock = self.clock.clone();
        Clock::from_fn(move || clock.load(Ordering::SeqCst))
    }

    /// The real demo printer, in manual mode.
    pub async fn printer(&self) -> Printer {
        let state = PrinterState::new(PrinterConfig::manual());
        let base = serve(ebc_demo_printer::app_with_state(state.clone())).await;
        Printer { base, state }
    }

    /// A store wired to the mock partner API, with the default delivery of its kind.
    pub async fn store(&self, kind: StoreKind) -> Store {
        self.store_with(kind, |config| config).await
    }

    /// Like [`World::store`], with a chance to adjust the configuration (printer URL,
    /// delivery, partner…).
    pub async fn store_with(
        &self,
        kind: StoreKind,
        adjust: impl FnOnce(StoreConfig) -> StoreConfig,
    ) -> Store {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let secret = match kind {
            StoreKind::Septante => SEPTANTE_SECRET,
            StoreKind::GrandeEnseigne => GRANDE_SECRET,
        };
        let mut config = StoreConfig::new(kind)
            .with_public_url(&base)
            .with_partner(&self.ebc.base, kind.slug(), secret)
            .with_clock(self.store_clock())
            .with_entropy(Entropy::deterministic(kind.default_port().into()))
            .with_test_hooks(true);
        config.password_iterations = 3;
        let state = AppState::new(adjust(config));
        let router = ebc_demo_stores::app_with_state(state.clone());
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Store { base, state, kind }
    }
}

impl Store {
    pub fn browser(&self) -> Browser {
        Browser::new(&self.base)
    }

    pub fn snapshot(&self) -> Value {
        self.state.snapshot()
    }
}

// ----- a small browser --------------------------------------------------------------------

pub fn plain_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Keeps cookies, follows redirects by hand (recording each hop), posts forms.
pub struct Browser {
    http: reqwest::Client,
    base: String,
}

pub struct Page {
    pub status: StatusCode,
    /// Path and query of the final URL.
    pub path: String,
    pub headers: reqwest::header::HeaderMap,
    pub body: Vec<u8>,
    /// `(status, Location)` of every redirect followed on the way.
    pub hops: Vec<(StatusCode, String)>,
}

impl Browser {
    pub fn new(base: &str) -> Self {
        let http = reqwest::Client::builder()
            .no_proxy()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        Self { http, base: base.to_owned() }
    }

    fn absolute(&self, path_or_url: &str) -> reqwest::Url {
        reqwest::Url::parse(&self.base).unwrap().join(path_or_url).unwrap()
    }

    pub async fn get(&self, path_or_url: &str) -> Page {
        let response = self.http.get(self.absolute(path_or_url)).send().await.unwrap();
        self.follow(response).await
    }

    pub async fn post(&self, path: &str, form: &[(&str, &str)]) -> Page {
        let body = reqwest::Url::parse_with_params("http://x/", form)
            .unwrap()
            .query()
            .unwrap_or("")
            .to_owned();
        let response = self
            .http
            .post(self.absolute(path))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .unwrap();
        self.follow(response).await
    }

    /// One request, redirects not followed.
    pub async fn get_once(&self, path_or_url: &str) -> Page {
        let response = self.http.get(self.absolute(path_or_url)).send().await.unwrap();
        Self::page(response, Vec::new()).await
    }

    async fn follow(&self, mut response: reqwest::Response) -> Page {
        let mut hops = Vec::new();
        while response.status().is_redirection() {
            assert!(hops.len() < 10, "redirect loop: {hops:?}");
            let location = response.headers()["location"].to_str().unwrap().to_owned();
            hops.push((response.status(), location.clone()));
            let next = response.url().join(&location).unwrap();
            assert_eq!(next.origin(), self.absolute("/").origin(), "redirected off-site to {next}");
            response = self.http.get(next).send().await.unwrap();
        }
        Self::page(response, hops).await
    }

    async fn page(response: reqwest::Response, hops: Vec<(StatusCode, String)>) -> Page {
        let url = response.url().clone();
        let path = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_owned(),
        };
        Page {
            status: response.status(),
            path,
            headers: response.headers().clone(),
            body: response.bytes().await.unwrap().to_vec(),
            hops,
        }
    }

    // --- common journeys ---

    pub async fn register(&self, email: &str, password: &str) -> Page {
        self.post("/compte/inscription", &[("email", email), ("password", password), ("next", "/")])
            .await
    }

    pub async fn login(&self, email: &str, password: &str) -> Page {
        self.post("/compte/connexion", &[("email", email), ("password", password), ("next", "/")])
            .await
    }

    pub async fn add_to_cart(&self, book: &str) -> Page {
        self.post("/panier/ajouter", &[("book", book)]).await
    }

    /// Registers, fills the cart with `books` and pays with the given checkout fields.
    /// Returns the page the browser lands on (the order confirmation when all goes well).
    pub async fn buy(&self, email: &str, books: &[&str], fields: &[(&str, &str)]) -> Page {
        let registered = self.register(email, "motdepasse").await;
        assert_eq!(registered.status, StatusCode::OK, "{}", registered.html());
        for book in books {
            self.add_to_cart(book).await;
        }
        self.post("/commande/payer", fields).await
    }
}

/// Checkout fields of a gift printed at home.
pub fn gift_home<'a>(
    giver: &'a str,
    recipient: &'a str,
    message: &'a str,
) -> Vec<(&'a str, &'a str)> {
    vec![
        ("gift", "1"),
        ("giver-name", giver),
        ("recipient-name", recipient),
        ("gift-message", message),
        ("card-delivery", "home"),
    ]
}

/// Checkout fields of a gift whose card is printed and posted.
pub fn gift_post<'a>(giver: &'a str, recipient: &'a str) -> Vec<(&'a str, &'a str)> {
    vec![
        ("gift", "1"),
        ("giver-name", giver),
        ("recipient-name", recipient),
        ("gift-message", ""),
        ("card-delivery", "post"),
        ("ship-name", "Sacha Martin"),
        ("ship-street", "12 rue des Lilas"),
        ("ship-postcode", "75011"),
        ("ship-city", "Paris"),
    ]
}

impl Page {
    pub fn html(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn doc(&self) -> scraper::Html {
        scraper::Html::parse_document(&self.html())
    }

    fn select_one<T>(
        &self,
        selector: &str,
        f: impl FnOnce(scraper::ElementRef<'_>) -> T,
    ) -> Option<T> {
        let doc = self.doc();
        let selector = scraper::Selector::parse(selector).unwrap();
        doc.select(&selector).next().map(f)
    }

    /// Whether an element with this id exists.
    pub fn has(&self, id: &str) -> bool {
        self.select_one(&format!("[id=\"{id}\"]"), |_| ()).is_some()
    }

    /// Text content of the element with this id, white space collapsed.
    pub fn text(&self, id: &str) -> Option<String> {
        self.select_one(&format!("[id=\"{id}\"]"), |e| {
            e.text().collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
        })
    }

    /// An attribute of the element with this id.
    pub fn attr(&self, id: &str, attribute: &str) -> Option<String> {
        self.select_one(&format!("[id=\"{id}\"]"), |e| e.value().attr(attribute).map(str::to_owned))
            .flatten()
    }

    /// Number of elements matching a CSS selector.
    pub fn count(&self, selector: &str) -> usize {
        let doc = self.doc();
        let selector = scraper::Selector::parse(selector).unwrap();
        doc.select(&selector).count()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// The order reference shown on a confirmation page.
    pub fn order_ref(&self) -> String {
        self.text("order-ref").unwrap_or_else(|| panic!("not an order page:\n{}", self.html()))
    }
}

/// Opens an EPUB and returns `(names in archive order, contents by name)`, checking on the
/// way that it is a well-formed OCF container.
pub fn open_epub(bytes: &[u8]) -> (Vec<String>, HashMap<String, Vec<u8>>) {
    use std::io::Read;
    assert_eq!(&bytes[..4], b"PK\x03\x04", "not a zip");
    assert_eq!(
        &bytes[30..58],
        b"mimetypeapplication/epub+zip",
        "mimetype must be first and stored"
    );
    assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), 0, "mimetype must not be compressed");
    assert_eq!(u16::from_le_bytes([bytes[28], bytes[29]]), 0, "mimetype must have no extra field");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut names = Vec::new();
    let mut files = HashMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let name = file.name().unwrap().into_owned();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        names.push(name.clone());
        files.insert(name, content);
    }
    assert_eq!(names[0], "mimetype");
    let container = String::from_utf8(files["META-INF/container.xml"].clone()).unwrap();
    assert!(container.contains("full-path=\"OEBPS/package.opf\""));
    for (name, content) in &files {
        if [".xml", ".opf", ".xhtml", ".ncx"].iter().any(|ext| name.ends_with(ext)) {
            let text = std::str::from_utf8(content).unwrap();
            let options =
                resvg::usvg::roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
            resvg::usvg::roxmltree::Document::parse_with_options(text, options)
                .unwrap_or_else(|e| panic!("{name} is not well-formed XML: {e}"));
        }
    }
    (names, files)
}

pub fn text_of(files: &HashMap<String, Vec<u8>>, name: &str) -> String {
    String::from_utf8(files.get(name).unwrap_or_else(|| panic!("{name} missing")).clone()).unwrap()
}
