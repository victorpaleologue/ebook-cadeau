//! Shared test harness: the service mounted in-process with a manual clock, signed
//! partner calls, EPUB fixtures, HTML checks and small in-test servers (SMTP sink, HTTP
//! receiver). Everything is offline: only loopback sockets are used.

use std::io::{Cursor, Write};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use ebc_server::{AppState, Config, ManualClock, MemoryStore, Partner, Store, app};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// 2026-11-12T00:00:00Z.
pub const T0: i64 = 1_794_441_600;
pub const PARTNER_ID: &str = "septante";
pub const PARTNER_SECRET: &str = "septante-demo-secret-0123456789";

pub const UA_KOBO: &str = "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (Kobo Touch 0390/4.38.23038)";
pub const UA_KINDLE: &str = "Mozilla/5.0 (X11; U; Linux armv7l like Android; en-us) AppleWebKit/531.2+ (KHTML, like Gecko) Version/5.0 Safari/533.2+ Kindle/3.0+";
pub const UA_PHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1";
pub const UA_DESKTOP: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0";

/// What the service answered.
pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl Reply {
    pub fn text(&self) -> String {
        String::from_utf8(self.body.to_vec()).unwrap()
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|e| panic!("not JSON ({e}): {}", self.text()))
    }

    pub fn header(&self, name: &str) -> &str {
        self.headers.get(name).unwrap_or_else(|| panic!("no {name} header")).to_str().unwrap()
    }

    pub fn has_header(&self, name: &str) -> bool {
        self.headers.contains_key(name)
    }

    pub fn location(&self) -> String {
        assert_eq!(self.status, StatusCode::SEE_OTHER, "{}", self.text());
        self.header("location").to_owned()
    }

    pub fn html(&self) -> scraper::Html {
        scraper::Html::parse_document(&self.text())
    }

    /// Text of the element with this id; panics when it is missing or duplicated.
    pub fn text_of(&self, id: &str) -> String {
        let html = self.html();
        let selector = scraper::Selector::parse(&format!("[id=\"{id}\"]")).unwrap();
        let found: Vec<_> = html.select(&selector).collect();
        assert_eq!(found.len(), 1, "expected exactly one #{id} in:\n{}", self.text());
        found[0].text().collect::<String>().trim().to_owned()
    }

    pub fn has_id(&self, id: &str) -> bool {
        let selector = scraper::Selector::parse(&format!("[id=\"{id}\"]")).unwrap();
        self.html().select(&selector).next().is_some()
    }

    /// Value of `attribute` on the element with this id.
    pub fn attr_of(&self, id: &str, attribute: &str) -> Option<String> {
        let selector = scraper::Selector::parse(&format!("[id=\"{id}\"]")).unwrap();
        self.html()
            .select(&selector)
            .next()
            .and_then(|e| e.value().attr(attribute).map(str::to_owned))
    }
}

/// The service in-process, with a clock the test controls.
pub struct Harness {
    pub state: AppState,
    pub app: Router,
    pub clock: Arc<ManualClock>,
}

/// The configuration tests start from: readable public URL, trusted proxy (so that a
/// test picks its client with `X-Forwarded-For`), one partner.
pub fn test_config() -> Config {
    let mut config = Config::for_tests();
    config.public_url = "https://ebk.example".to_owned();
    config.trust_proxy = true;
    config.partners = vec![partner(&[], None)];
    config
}

pub fn partner(redirect_hosts: &[&str], webhook_url: Option<String>) -> Partner {
    Partner {
        id: PARTNER_ID.to_owned(),
        name: "Septante (démo)".to_owned(),
        secret: PARTNER_SECRET.to_owned(),
        webhook_url,
        redirect_hosts: redirect_hosts.iter().map(|h| (*h).to_owned()).collect(),
    }
}

impl Harness {
    pub fn new() -> Self {
        Self::with(|_| {})
    }

    pub fn with(configure: impl FnOnce(&mut Config)) -> Self {
        Self::with_store(Arc::new(MemoryStore::new()), configure)
    }

    pub fn with_store(store: Arc<dyn Store>, configure: impl FnOnce(&mut Config)) -> Self {
        let mut config = test_config();
        configure(&mut config);
        let clock = Arc::new(ManualClock::new(T0));
        let state = AppState::builder(config).store(store).clock(clock.clone()).build().unwrap();
        Self { app: app(state.clone()), state, clock }
    }

    pub async fn send(&self, request: Request<Body>) -> Reply {
        let response = self.app.clone().oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        Reply { status: parts.status, headers: parts.headers, body }
    }

    pub async fn call(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, &str)],
        body: impl Into<Body>,
    ) -> Reply {
        let mut builder = Request::builder().method(method).uri(path);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        self.send(builder.body(body.into()).unwrap()).await
    }

    pub async fn get(&self, path: &str) -> Reply {
        self.call(Method::GET, path, &[], Body::empty()).await
    }

    pub async fn get_as(&self, path: &str, user_agent: &str) -> Reply {
        self.call(Method::GET, path, &[("user-agent", user_agent)], Body::empty()).await
    }

    /// `GET` from a given client address (the harness trusts `X-Forwarded-For`).
    pub async fn get_from(&self, path: &str, client: &str) -> Reply {
        self.call(Method::GET, path, &[("x-forwarded-for", client)], Body::empty()).await
    }

    pub async fn post_form(&self, path: &str, fields: &[(&str, &str)]) -> Reply {
        self.post_form_with(path, fields, &[]).await
    }

    pub async fn post_form_with(
        &self,
        path: &str,
        fields: &[(&str, &str)],
        headers: &[(&str, &str)],
    ) -> Reply {
        let body = fields
            .iter()
            .map(|(name, value)| {
                format!(
                    "{name}={}",
                    percent_encoding::utf8_percent_encode(
                        value,
                        percent_encoding::NON_ALPHANUMERIC
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        let mut all = vec![("content-type", "application/x-www-form-urlencoded")];
        all.extend_from_slice(headers);
        self.call(Method::POST, path, &all, body).await
    }

    /// A signed partner request (`docs/api.md` §4.1), timestamped with the test clock.
    pub async fn partner(
        &self,
        method: Method,
        path: &str,
        body: impl Into<Bytes>,
        headers: &[(&str, &str)],
    ) -> Reply {
        let body: Bytes = body.into();
        let timestamp = self.state.now();
        self.partner_signed(method, path, body, headers, timestamp, PARTNER_SECRET).await
    }

    pub async fn partner_signed(
        &self,
        method: Method,
        path: &str,
        body: Bytes,
        headers: &[(&str, &str)],
        timestamp: i64,
        secret: &str,
    ) -> Reply {
        let signature =
            ebc_server::codes::request_signature(secret, timestamp, method.as_str(), path, &body);
        let timestamp = timestamp.to_string();
        let mut all = vec![
            ("x-ebc-partner", PARTNER_ID),
            ("x-ebc-timestamp", timestamp.as_str()),
            ("x-ebc-signature", signature.as_str()),
        ];
        all.extend_from_slice(headers);
        self.call(method, path, &all, Body::from(body)).await
    }

    pub async fn partner_json(
        &self,
        method: Method,
        path: &str,
        body: &serde_json::Value,
        headers: &[(&str, &str)],
    ) -> Reply {
        self.partner(method, path, body.to_string(), headers).await
    }

    /// Uploads `bytes` through the creator API; returns the upload id.
    pub async fn upload(&self, bytes: Vec<u8>, file_name: &str) -> String {
        let reply = self
            .call(
                Method::POST,
                "/api/v1/uploads",
                &[
                    ("x-rights-basis", "public_domain"),
                    ("x-file-name", file_name),
                    ("content-type", "application/epub+zip"),
                ],
                bytes,
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        reply.json()["upload_id"].as_str().unwrap().to_owned()
    }

    /// Creates a standalone hosted-file gift; returns the creation response.
    pub async fn hosted_gift(&self, overrides: serde_json::Value) -> serde_json::Value {
        let upload_id = self.upload(epub(), "le-horla.epub").await;
        let mut body = serde_json::json!({
            "book": { "title": "Le Horla", "isbn13": "9782070360024", "contributors": [{"name": "Guy de Maupassant"}], "language": "fr" },
            "giver_name": "Camille", "recipient_name": "Sacha", "message": "Joyeux anniversaire !",
            "locale": "fr",
            "delivery": { "type": "hosted_file", "upload_id": upload_id },
        });
        merge(&mut body, overrides);
        let reply = self
            .call(
                Method::POST,
                "/api/v1/gifts",
                &[("content-type", "application/json")],
                body.to_string(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        reply.json()
    }

    /// Creates a partner gift with the given delivery; returns the creation response.
    pub async fn partner_gift(
        &self,
        delivery: serde_json::Value,
        overrides: serde_json::Value,
    ) -> serde_json::Value {
        let mut body = serde_json::json!({
            "order_ref": "ORD-2026-000123",
            "book": { "title": "Madame Bovary", "contributors": [{"name": "Gustave Flaubert"}], "publisher": "Éditions Démo", "language": "fr" },
            "giver_name": "Camille", "recipient_name": "Sacha", "message": "Bonne lecture", "locale": "fr",
            "delivery": delivery,
        });
        merge(&mut body, overrides);
        let reply = self.partner_json(Method::POST, "/api/partner/v1/gifts", &body, &[]).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        reply.json()
    }
}

/// Shallow-merges the members of `overrides` into `target`.
pub fn merge(target: &mut serde_json::Value, overrides: serde_json::Value) {
    if let (Some(target), serde_json::Value::Object(overrides)) =
        (target.as_object_mut(), overrides)
    {
        for (key, value) in overrides {
            target.insert(key, value);
        }
    }
}

pub const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#;

/// Builds a ZIP from `(name, content, deflate)` entries, in order.
pub fn zip_of(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
    use zip::write::SimpleFileOptions;
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content, deflate) in entries {
        let method = if *deflate {
            zip::CompressionMethod::Deflated
        } else {
            zip::CompressionMethod::Stored
        };
        zip.start_file(*name, SimpleFileOptions::DEFAULT.compression_method(method)).unwrap();
        zip.write_all(content).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// A minimal valid EPUB with extra entries.
pub fn epub_with(extra: &[(&str, &[u8])]) -> Vec<u8> {
    let mut entries: Vec<(&str, &[u8], bool)> = vec![
        ("mimetype", b"application/epub+zip", false),
        ("META-INF/container.xml", CONTAINER_XML.as_bytes(), true),
        ("OEBPS/package.opf", b"<package/>", true),
        (
            "OEBPS/chapter-1.xhtml",
            b"<html><body><p>Le 8 mai. Quelle journee admirable !</p></body></html>",
            true,
        ),
    ];
    for (name, content) in extra {
        entries.push((name, content, true));
    }
    zip_of(&entries)
}

pub fn epub() -> Vec<u8> {
    epub_with(&[])
}

/// Checks what every recipient page must satisfy (`docs/api.md` §2): status, language,
/// no script of any kind, nothing loaded from elsewhere, weight budget, no cookie, the
/// security headers, and the presence (exactly once) of `ids`.
pub fn assert_page(reply: &Reply, status: StatusCode, lang: &str, ids: &[&str]) {
    let text = reply.text();
    assert_eq!(reply.status, status, "{text}");
    assert!(reply.header("content-type").starts_with("text/html; charset=utf-8"));
    assert!(text.len() < 30_000, "page weighs {} bytes", text.len());
    assert!(text.starts_with("<!DOCTYPE html>"), "{}", &text[..40.min(text.len())]);

    assert_eq!(reply.header("cache-control"), "no-store");
    assert_eq!(reply.header("referrer-policy"), "no-referrer");
    assert_eq!(reply.header("x-robots-tag"), "noindex, nofollow");
    assert_eq!(reply.header("x-content-type-options"), "nosniff");
    let csp = reply.header("content-security-policy");
    assert!(csp.contains("default-src 'none'"), "{csp}");
    assert!(!csp.contains("script-src") && !csp.contains("unsafe-inline"), "{csp}");
    assert!(!reply.has_header("set-cookie"));

    let html = reply.html();
    let select = |css: &str| scraper::Selector::parse(css).unwrap();
    let root = html.select(&select("html")).next().unwrap();
    assert_eq!(root.value().attr("lang"), Some(lang), "{text}");
    assert_eq!(html.select(&select("script")).count(), 0, "{text}");
    assert_eq!(html.select(&select("style")).count(), 0, "inline style is blocked by the CSP");
    assert_eq!(html.select(&select("meta[charset]")).count(), 1);
    assert_eq!(html.select(&select("meta[name=viewport]")).count(), 1);
    for element in html.select(&select("*")) {
        for (name, value) in element.value().attrs() {
            assert!(!name.starts_with("on"), "event handler {name} in {text}");
            assert!(!value.trim_start().to_ascii_lowercase().starts_with("javascript:"), "{text}");
            assert!(name != "style", "inline style attribute in {text}");
        }
    }
    // Nothing is loaded from another origin; forms post to this origin.
    for (css, attribute) in [("link", "href"), ("img", "src"), ("form", "action")] {
        for element in html.select(&select(css)) {
            let target = element.value().attr(attribute).unwrap_or("");
            assert!(
                target.starts_with('/') && !target.starts_with("//"),
                "{css} {attribute}={target:?}"
            );
        }
    }
    // Every visible input has a label; every form a submit button.
    for input in html.select(&select("input:not([type=hidden])")) {
        let id = input.value().attr("id").unwrap();
        assert_eq!(
            html.select(&select(&format!("label[for=\"{id}\"]"))).count(),
            1,
            "label for {id}"
        );
    }
    for form in html.select(&select("form")) {
        assert_eq!(form.select(&select("button[type=submit]")).count(), 1, "{}", form.html());
    }
    for id in ids {
        let found = html.select(&select(&format!("[id=\"{id}\"]"))).count();
        assert_eq!(found, 1, "expected exactly one #{id} in:\n{text}");
    }
}

/// One HTTP request received by an in-test server.
#[derive(Debug, Clone)]
pub struct Received {
    pub method: String,
    pub path_and_query: String,
    pub headers: HeaderMap,
    pub body: Bytes,
}

/// An in-test HTTP server on a loopback port that records what it receives and answers
/// with a programmable function.
pub struct Receiver {
    pub addr: SocketAddr,
    pub received: Arc<Mutex<Vec<Received>>>,
    /// Number of requests to answer with `500` before answering normally.
    pub fail_first: Arc<AtomicUsize>,
}

type Responder = Arc<dyn Fn(&Received) -> (StatusCode, Vec<u8>) + Send + Sync>;

impl Receiver {
    /// A server answering every request with `respond`.
    pub async fn start(
        respond: impl Fn(&Received) -> (StatusCode, Vec<u8>) + Send + Sync + 'static,
    ) -> Self {
        let received = Arc::new(Mutex::new(Vec::new()));
        let fail_first = Arc::new(AtomicUsize::new(0));
        let respond: Responder = Arc::new(respond);
        let (log, failures) = (received.clone(), fail_first.clone());
        let router = Router::new().fallback(move |request: Request<Body>| {
            let (log, failures, respond) = (log.clone(), failures.clone(), respond.clone());
            async move {
                let (parts, body) = request.into_parts();
                let body = body.collect().await.unwrap().to_bytes();
                let entry = Received {
                    method: parts.method.to_string(),
                    path_and_query: parts
                        .uri
                        .path_and_query()
                        .map(|pq| pq.to_string())
                        .unwrap_or_default(),
                    headers: parts.headers,
                    body,
                };
                log.lock().unwrap().push(entry.clone());
                let failing = failures
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
                    .is_ok();
                if failing {
                    (StatusCode::INTERNAL_SERVER_ERROR, b"try again".to_vec())
                } else {
                    respond(&entry)
                }
            }
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Self { addr, received, fail_first }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    pub fn host(&self) -> String {
        self.addr.to_string()
    }

    pub fn requests(&self) -> Vec<Received> {
        self.received.lock().unwrap().clone()
    }

    /// Waits (up to 5 s of real time) until `count` requests have arrived.
    pub async fn wait_for(&self, count: usize) -> Vec<Received> {
        for _ in 0..500 {
            let requests = self.requests();
            if requests.len() >= count {
                return requests;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("expected {count} requests, got {}", self.requests().len());
    }
}

/// One message accepted by the SMTP sink.
#[derive(Debug, Clone)]
pub struct Mail {
    pub mail_from: String,
    pub rcpt_to: Vec<String>,
    pub data: Vec<u8>,
}

/// A tiny SMTP server on a loopback port: speaks just enough of the protocol for one
/// client at a time and keeps the messages.
pub struct SmtpSink {
    pub addr: SocketAddr,
    pub mails: Arc<Mutex<Vec<Mail>>>,
}

impl SmtpSink {
    pub async fn start() -> Self {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mails = Arc::new(Mutex::new(Vec::new()));
        let sink = mails.clone();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else { return };
                let sink = sink.clone();
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.into_split();
                    let mut lines = BufReader::new(reader);
                    let mut mail =
                        Mail { mail_from: String::new(), rcpt_to: Vec::new(), data: Vec::new() };
                    writer.write_all(b"220 sink.test ESMTP\r\n").await.unwrap();
                    let mut line = Vec::new();
                    loop {
                        line.clear();
                        if lines.read_until(b'\n', &mut line).await.unwrap_or(0) == 0 {
                            return;
                        }
                        let command = String::from_utf8_lossy(&line).trim_end().to_owned();
                        let upper = command.to_ascii_uppercase();
                        let answer: &[u8] =
                            if upper.starts_with("EHLO") || upper.starts_with("HELO") {
                                b"250 sink.test\r\n"
                            } else if upper.starts_with("MAIL FROM:") {
                                mail.mail_from = command[10..].trim().to_owned();
                                b"250 OK\r\n"
                            } else if upper.starts_with("RCPT TO:") {
                                mail.rcpt_to.push(command[8..].trim().to_owned());
                                b"250 OK\r\n"
                            } else if upper == "DATA" {
                                writer.write_all(b"354 end with <CRLF>.<CRLF>\r\n").await.unwrap();
                                loop {
                                    line.clear();
                                    if lines.read_until(b'\n', &mut line).await.unwrap_or(0) == 0 {
                                        return;
                                    }
                                    if line == b".\r\n" {
                                        break;
                                    }
                                    // SMTP transparency: a leading dot was doubled by the client.
                                    let start = usize::from(line.starts_with(b".."));
                                    mail.data.extend_from_slice(&line[start..]);
                                }
                                sink.lock().unwrap().push(mail.clone());
                                mail = Mail {
                                    mail_from: String::new(),
                                    rcpt_to: Vec::new(),
                                    data: Vec::new(),
                                };
                                b"250 queued\r\n"
                            } else if upper == "QUIT" {
                                writer.write_all(b"221 bye\r\n").await.unwrap();
                                return;
                            } else {
                                b"250 OK\r\n"
                            };
                        writer.write_all(answer).await.unwrap();
                    }
                });
            }
        });
        Self { addr, mails }
    }

    pub fn url(&self) -> String {
        format!("smtp://{}", self.addr)
    }

    pub fn messages(&self) -> Vec<Mail> {
        self.mails.lock().unwrap().clone()
    }
}
