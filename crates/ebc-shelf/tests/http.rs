//! The shelf over HTTP, in process, against a small mock of the core's partner API that
//! checks every signature: sign-in, own mode, form tokens, identity, the upload gate,
//! gifts, the §4.5 endpoint and its download links, notice and action, the
//! repeat-infringer policy, complaints and retention.

#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use ebc_shelf::{AppState, Config, ManualClock, MemoryMailer, MemoryStore, Mode, crypto};
use http_body_util::BodyExt;
use tower::ServiceExt;

const T0: i64 = 1_794_441_600;
const OPERATOR: &str = "moderation@etagere.test";

// ----- fixtures -------------------------------------------------------------------------

fn epub(title: &str, isbn: Option<&str>, extra: &[(&str, &[u8])]) -> Vec<u8> {
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::DEFAULT.compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::DEFAULT.compression_method(CompressionMethod::Deflated);
    zip.start_file("mimetype", stored).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();
    zip.start_file("META-INF/container.xml", deflated).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();
    let identifier =
        isbn.map_or(String::new(), |i| format!("<dc:identifier>urn:isbn:{i}</dc:identifier>"));
    let opf = format!(
        r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>{title}</dc:title>{identifier}<dc:language>fr</dc:language></metadata></package>"#
    );
    zip.start_file("OEBPS/package.opf", deflated).unwrap();
    zip.write_all(opf.as_bytes()).unwrap();
    zip.start_file("OEBPS/text.xhtml", deflated).unwrap();
    zip.write_all(format!("<html><body><p>{title}</p></body></html>").as_bytes()).unwrap();
    for (name, content) in extra {
        zip.start_file(*name, deflated).unwrap();
        zip.write_all(content).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

// ----- the mock core --------------------------------------------------------------------

#[derive(Clone, Default)]
struct Core {
    secret: String,
    calls: Arc<Mutex<Vec<String>>>,
    gifts: Arc<Mutex<Vec<(String, String)>>>, // (core id, status)
    down: Arc<Mutex<bool>>,
}

fn verify(core: &Core, headers: &HeaderMap, method: &str, path: &str, body: &[u8]) -> bool {
    let text = |name: &str| headers.get(name).and_then(|v| v.to_str().ok()).unwrap_or("");
    let ts: i64 = text("x-ebc-timestamp").parse().unwrap_or(0);
    text("x-ebc-partner") == "etagere"
        && text("x-ebc-signature")
            == crypto::request_signature(&core.secret, ts, method, path, body)
}

async fn core_create(
    State(core): State<Core>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    if *core.down.lock().unwrap() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    assert!(verify(&core, &headers, "POST", "/api/partner/v1/gifts", &body), "unsigned creation");
    let request: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(request["delivery"]["type"], "partner_file");
    assert_eq!(
        headers.get("idempotency-key").unwrap().to_str().unwrap(),
        request["order_ref"].as_str().unwrap()
    );
    let mut gifts = core.gifts.lock().unwrap();
    let id = format!("g_{}", gifts.len() + 1);
    gifts.push((id.clone(), "created".to_owned()));
    core.calls
        .lock()
        .unwrap()
        .push(format!("create {id} {}", request["order_ref"].as_str().unwrap()));
    let code = ebc_model::GiftCode::from_payload(1_000 + gifts.len() as u64).unwrap();
    let answer = serde_json::json!({"id": id, "code": code.compact(), "code_display": code.grouped(), "claim_url": format!("http://core.test/k/{}", code.compact()), "status": "created"});
    (StatusCode::CREATED, axum::Json(answer)).into_response()
}

async fn core_action(
    State(core): State<Core>,
    axum::extract::Path((id, action)): axum::extract::Path<(String, String)>,
    axum::extract::RawQuery(query): axum::extract::RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    let path = match &query {
        Some(q) => format!("/api/partner/v1/gifts/{id}/{action}?{q}"),
        None => format!("/api/partner/v1/gifts/{id}/{action}"),
    };
    assert!(verify(&core, &headers, "POST", &path, &body), "unsigned {action}");
    core.calls.lock().unwrap().push(format!("{action} {id}"));
    if action == "cancel" {
        for gift in core.gifts.lock().unwrap().iter_mut().filter(|g| g.0 == id) {
            gift.1 = "cancelled".to_owned();
        }
        return axum::Json(serde_json::json!({"id": id, "status": "cancelled"})).into_response();
    }
    let code = ebc_model::GiftCode::from_payload(999).unwrap();
    axum::Json(serde_json::json!({"code": code.compact(), "code_display": code.grouped(), "claim_url": format!("http://core.test/k/{}", code.compact())})).into_response()
}

async fn core_card(
    State(core): State<Core>,
    headers: HeaderMap,
    uri: axum::http::Uri,
) -> axum::response::Response {
    let path = uri.path_and_query().unwrap().as_str().to_owned();
    assert!(verify(&core, &headers, "GET", &path, b""), "unsigned card");
    (
        [("content-type", "application/pdf"), ("x-ebc-card-warnings", "cover_credit_missing")],
        b"%PDF-1.7 card".to_vec(),
    )
        .into_response()
}

async fn start_core(secret: &str) -> (String, Core) {
    let core = Core { secret: secret.to_owned(), ..Core::default() };
    let router = Router::new()
        .route("/api/partner/v1/gifts", post(core_create))
        .route("/api/partner/v1/gifts/{id}/card.pdf", get(core_card))
        .route("/api/partner/v1/gifts/{id}/{action}", post(core_action))
        .with_state(core.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    (base, core)
}

// ----- the shelf under test ---------------------------------------------------------------

struct Shelf {
    state: AppState,
    router: Router,
    mail: Arc<MemoryMailer>,
    clock: Arc<ManualClock>,
    store: Arc<MemoryStore>,
    core: Core,
}

async fn shelf_with(tweak: impl FnOnce(&mut Config)) -> Shelf {
    let mut config = Config::for_tests();
    let (base, core) = start_core(&config.partner_secret).await;
    config.core_url = base.clone();
    config.core_public_url = base;
    tweak(&mut config);
    let mail = Arc::new(MemoryMailer::new());
    let clock = Arc::new(ManualClock::new(T0));
    let store = Arc::new(MemoryStore::new());
    let state = AppState::builder(config)
        .mailer(mail.clone())
        .clock(clock.clone())
        .store(store.clone())
        .build()
        .unwrap();
    Shelf { router: ebc_shelf::app(state.clone()), state, mail, clock, store, core }
}

async fn shelf() -> Shelf {
    shelf_with(|_| {}).await
}

#[derive(Debug)]
struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Reply {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
    fn header(&self, name: &str) -> &str {
        self.headers.get(name).and_then(|v| v.to_str().ok()).unwrap_or("")
    }
    fn location(&self) -> &str {
        self.header("location")
    }
}

/// A browser: keeps the session cookie, finds the form token on pages.
struct Browser<'a> {
    shelf: &'a Shelf,
    cookie: Option<String>,
    csrf: Option<String>,
}

impl<'a> Browser<'a> {
    fn new(shelf: &'a Shelf) -> Self {
        Self { shelf, cookie: None, csrf: None }
    }

    async fn send(&mut self, request: Request<Body>) -> Reply {
        let response = self.shelf.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        for value in headers.get_all("set-cookie") {
            let value = value.to_str().unwrap();
            if let Some(rest) = value.strip_prefix("ebc_shelf=") {
                let token = rest.split(';').next().unwrap();
                self.cookie = (!token.is_empty()).then(|| token.to_owned());
            }
        }
        let body = response.into_body().collect().await.unwrap().to_bytes().to_vec();
        let reply = Reply { status, headers, body };
        if let Some(token) = find_csrf(&reply.text()) {
            self.csrf = Some(token);
        }
        reply
    }

    fn request(&self, method: &str, path: &str) -> axum::http::request::Builder {
        let peer: std::net::SocketAddr = "192.0.2.10:5000".parse().unwrap();
        let mut builder =
            Request::builder().method(method).uri(path).extension(axum::extract::ConnectInfo(peer));
        if let Some(cookie) = &self.cookie {
            builder = builder.header("cookie", format!("ebc_shelf={cookie}"));
        }
        builder
    }

    async fn get(&mut self, path: &str) -> Reply {
        let request = self.request("GET", path).body(Body::empty()).unwrap();
        self.send(request).await
    }

    async fn post(&mut self, path: &str, fields: &[(&str, &str)]) -> Reply {
        let mut all: Vec<(String, String)> =
            fields.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        if let Some(csrf) = &self.csrf
            && !fields.iter().any(|(k, _)| *k == "csrf")
        {
            all.push(("csrf".to_owned(), csrf.clone()));
        }
        let body = serde_urlencoded(&all);
        let request = self
            .request("POST", path)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();
        self.send(request).await
    }

    async fn upload(&mut self, fields: &[(&str, &str)], file: (&str, &[u8])) -> Reply {
        let boundary = "----shelf-test-boundary";
        let mut body = Vec::new();
        let csrf = self.csrf.clone().unwrap_or_default();
        let mut all: Vec<(&str, &str)> = vec![("csrf", csrf.as_str())];
        all.extend_from_slice(fields);
        for (name, value) in all {
            body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n").as_bytes());
        }
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n", file.0).as_bytes(),
        );
        body.extend_from_slice(file.1);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let request = self
            .request("POST", "/livres")
            .header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        self.send(request).await
    }
}

fn serde_urlencoded(fields: &[(String, String)]) -> String {
    let enc = |v: &str| {
        percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC).to_string()
    };
    fields.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect::<Vec<_>>().join("&")
}

fn find_csrf(html: &str) -> Option<String> {
    let at = html.find("name=\"csrf\" value=\"")?;
    let rest = &html[at + 19..];
    Some(rest[..rest.find('"')?].to_owned())
}

fn sign_in_link(shelf: &Shelf, email: &str) -> String {
    let mail = shelf.mail.sent_to(email).pop().expect("a sign-in mail");
    let line = mail
        .text
        .lines()
        .find(|l| l.contains("/connexion/"))
        .expect("a link in the mail")
        .trim()
        .to_owned();
    line.split_once("/connexion/").map(|(_, token)| format!("/connexion/{token}")).unwrap()
}

/// Signs in as `email` and fills the identification form.
async fn signed_in<'a>(shelf: &'a Shelf, email: &str) -> Browser<'a> {
    let mut browser = Browser::new(shelf);
    let asked = browser.post("/connexion", &[("email", email)]).await;
    assert_eq!(asked.status, StatusCode::OK, "{}", asked.text());
    assert!(asked.text().contains("id=\"link-sent\""));
    let link = sign_in_link(shelf, email);
    let confirm = browser.get(&link).await;
    assert!(confirm.text().contains("id=\"confirm-signin\""));
    let done = browser.post(&link, &[]).await;
    assert_eq!(done.status, StatusCode::SEE_OTHER);
    assert!(
        done.header("set-cookie").contains("HttpOnly")
            && done.header("set-cookie").contains("SameSite=Strict")
    );
    browser.get("/compte").await;
    let saved = browser
        .post(
            "/compte/identite",
            &[
                ("kind", "person"),
                ("name", "Jeanne Roy"),
                ("address", "12 rue des Lilas, 69003 Lyon"),
            ],
        )
        .await;
    assert_eq!(saved.status, StatusCode::SEE_OTHER, "{}", saved.text());
    browser.get("/livres/nouveau").await;
    browser
}

async fn upload_ok(browser: &mut Browser<'_>, title: &str, isbn: &str, bytes: &[u8]) -> String {
    browser.get("/livres/nouveau").await;
    let reply = browser
        .upload(
            &[
                ("title", title),
                ("author", "Jeanne Roy"),
                ("isbn", isbn),
                ("basis", "own_work"),
                ("declared", "1"),
            ],
            ("livre.epub", bytes),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::SEE_OTHER,
        "{} {}",
        reply.header("x-ebc-refusal"),
        reply.text()
    );
    reply.location().trim_start_matches("/livres/").to_owned()
}

async fn make_gift(browser: &mut Browser<'_>, book: &str) -> (String, String, String) {
    browser.get(&format!("/livres/{book}")).await;
    let reply = browser
        .post(
            &format!("/livres/{book}/cadeaux"),
            &[("count", "1"), ("recipient", "Sacha"), ("locale", "fr")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text());
    let shelf = browser.shelf;
    let gift = shelf.state.snapshot().gifts.into_iter().rfind(|g| g.book_id == book).unwrap();
    (gift.id, gift.core_id.unwrap(), gift.code.unwrap())
}

/// The core's signed link request for a gift.
async fn link_request(shelf: &Shelf, core_id: &str, order: &str, ts: i64, secret: &str) -> Reply {
    let sig = crypto::link_request_signature(secret, core_id, order, ts);
    let mut browser = Browser::new(shelf);
    browser
        .get(&format!(
            "/api/ebc/link?ebc_gift={core_id}&ebc_order={order}&ebc_ts={ts}&ebc_sig={sig}"
        ))
        .await
}

fn path_of(url: &str) -> String {
    let rest = url.split_once("://").unwrap().1;
    rest[rest.find('/').unwrap()..].to_owned()
}

// ----- tests ----------------------------------------------------------------------------

#[tokio::test]
async fn pages_work_without_scripts_and_carry_the_security_headers() {
    let shelf = shelf().await;
    let mut browser = Browser::new(&shelf);
    for path in ["/", "/connexion", "/signaler", "/conditions", "/mentions-legales", "/contact"] {
        let reply = browser.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        let csp = reply.header("content-security-policy");
        assert!(
            csp.contains("default-src 'none'") && csp.contains("form-action 'self'"),
            "{path}: {csp}"
        );
        assert_eq!(reply.header("x-content-type-options"), "nosniff");
        assert_eq!(reply.header("referrer-policy"), "no-referrer");
        assert_eq!(reply.header("x-robots-tag"), "noindex, nofollow");
        assert!(!reply.text().to_lowercase().contains("<script"), "{path}");
        assert!(reply.text().contains("lang=\"fr\""), "French by default: {path}");
    }
    let terms = browser.get("/conditions").await.text();
    assert!(
        terms.contains("id=\"draft\"")
            && terms.contains("id=\"repeat-policy\"")
            && terms.contains("id=\"retention\"")
    );
    let english = browser.get("/signaler?lang=en").await;
    assert!(english.text().contains("Report illegal content"));
    assert!(english.header("set-cookie").starts_with("ebc_lang=en"));
    assert_eq!(browser.get("/compte").await.location(), "/connexion");
    assert_eq!(browser.get("/nope").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sign_in_links_are_single_use_short_lived_and_hashed() {
    let shelf = shelf().await;
    let mut browser = Browser::new(&shelf);
    assert_eq!(
        browser.post("/connexion", &[("email", "not-an-address")]).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    browser.post("/connexion", &[("email", "Autrice@Exemple.FR")]).await;
    let link = sign_in_link(&shelf, "autrice@exemple.fr");
    let token = link.trim_start_matches("/connexion/").to_owned();
    let db = shelf.state.snapshot();
    assert!(!serde_json::to_string(&db).unwrap().contains(&token), "only the hash is stored");
    // A GET does not use the link up (mail scanners follow links).
    assert_eq!(browser.get(&link).await.status, StatusCode::OK);
    assert_eq!(browser.get(&link).await.status, StatusCode::OK);
    assert_eq!(browser.post(&link, &[]).await.status, StatusCode::SEE_OTHER);
    let mut other = Browser::new(&shelf);
    assert_eq!(other.post(&link, &[]).await.status, StatusCode::GONE, "single use");
    // Expired.
    let mut late = Browser::new(&shelf);
    late.post("/connexion", &[("email", "autre@exemple.fr")]).await;
    let link = sign_in_link(&shelf, "autre@exemple.fr");
    shelf.clock.advance(15 * 60 + 1);
    assert_eq!(late.post(&link, &[]).await.status, StatusCode::GONE);
    // Rate limit per address.
    for _ in 0..4 {
        late.post("/connexion", &[("email", "flood@exemple.fr")]).await;
    }
    assert_eq!(
        late.post("/connexion", &[("email", "flood@exemple.fr")]).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    // A forged session cookie is nobody.
    let mut forged = Browser::new(&shelf);
    forged.cookie = Some("ab".repeat(32));
    assert_eq!(forged.get("/compte").await.location(), "/connexion");
}

#[tokio::test]
async fn own_mode_refuses_a_second_account() {
    let shelf = shelf_with(|c| c.mode = Mode::Own).await;
    let mut stranger = Browser::new(&shelf);
    let reply = stranger.post("/connexion", &[("email", "autre@exemple.fr")]).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert!(reply.text().contains("n’ouvre pas de compte"));
    assert!(shelf.mail.sent_to("autre@exemple.fr").is_empty(), "no mail");
    let mut owner = signed_in(&shelf, OPERATOR).await;
    let page = owner.get("/compte").await.text();
    assert!(page.contains("id=\"upload-link\""));
    assert!(owner.get("/moderation").await.status == StatusCode::OK, "the owner moderates");
    assert_eq!(shelf.state.snapshot().accounts.len(), 1);
}

#[tokio::test]
async fn forms_need_the_session_token_and_the_same_origin() {
    let shelf = shelf().await;
    let mut browser = signed_in(&shelf, "autrice@exemple.fr").await;
    let reply = browser
        .post("/compte/identite", &[("csrf", "forged"), ("name", "X"), ("address", "Y")])
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let request = browser
        .request("POST", "/compte/identite")
        .header("origin", "https://evil.example")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("name=X"))
        .unwrap();
    assert_eq!(browser.send(request).await.status, StatusCode::FORBIDDEN);
    // Identity is required before uploading.
    let mut fresh = Browser::new(&shelf);
    fresh.post("/connexion", &[("email", "nouveau@exemple.fr")]).await;
    let link = sign_in_link(&shelf, "nouveau@exemple.fr");
    fresh.post(&link, &[]).await;
    let page = fresh.get("/compte").await.text();
    assert!(page.contains("id=\"identity-missing\"") && !page.contains("id=\"upload-link\""));
    assert_eq!(fresh.get("/livres/nouveau").await.location(), "/compte");
    let refused = fresh
        .post(
            "/compte/identite",
            &[("kind", "organisation"), ("name", "Éditions X"), ("address", "Paris")],
        )
        .await;
    assert_eq!(
        refused.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "an organisation names who acts for it"
    );
}

#[tokio::test]
async fn the_upload_gate_refuses_what_it_must_and_records_the_declaration() {
    let shelf = shelf_with(|c| c.max_file_bytes = 64 * 1024).await;
    let mut browser = signed_in(&shelf, "autrice@exemple.fr").await;
    let book = epub("Les Vagues du soir", None, &[]);
    let base = [
        ("title", "Les Vagues du soir"),
        ("author", "Jeanne Roy"),
        ("basis", "own_work"),
        ("declared", "1"),
    ];
    let refused = |fields: Vec<(&'static str, &'static str)>, file: Vec<u8>, code: &'static str| {
        (fields, file, code)
    };
    let without = |name: &str| base.iter().copied().filter(|(k, _)| *k != name).collect::<Vec<_>>();
    let drm = epub("x", None, &[("META-INF/rights.xml", b"<adept:rights/>")]);
    let lcp = epub("x", None, &[("META-INF/license.lcpl", b"{}")]);
    let big = {
        let mut b = epub("x", None, &[]);
        b.resize(70 * 1024, 0);
        b
    };
    let cases = vec![
        refused(without("declared"), book.clone(), "declaration_required"),
        refused(without("basis"), book.clone(), "declaration_required"),
        refused(without("title"), book.clone(), "title_required"),
        refused(base.to_vec(), drm, "drm_protected"),
        refused(base.to_vec(), lcp, "drm_protected"),
        refused(base.to_vec(), b"MZ\x90\x00 an executable".to_vec(), "unsupported_type"),
        refused(base.to_vec(), Vec::new(), "file_required"),
        refused(base.to_vec(), big, "file_too_large"),
        refused(
            [base.to_vec(), vec![("isbn", "978-2-07-036002-5")]].concat(),
            book.clone(),
            "invalid_isbn",
        ),
        refused(
            vec![
                ("title", "Le Horla"),
                ("author", "Guy de Maupassant"),
                ("basis", "public_domain"),
                ("death_year", "1990"),
                ("declared", "1"),
            ],
            book.clone(),
            "not_public_domain",
        ),
        refused(
            vec![
                ("title", "Libre"),
                ("author", "A"),
                ("basis", "open_licence"),
                ("licence", "gpl"),
                ("declared", "1"),
            ],
            book.clone(),
            "licence_required",
        ),
        refused(
            vec![
                ("title", "Diffusé"),
                ("author", "A"),
                ("basis", "licensed_distributor"),
                ("declared", "1"),
            ],
            book.clone(),
            "licensor_required",
        ),
        refused(
            vec![("title", "Acheté"), ("author", "A"), ("basis", "purchased"), ("declared", "1")],
            book.clone(),
            "declaration_required",
        ),
    ];
    for (fields, file, code) in cases {
        browser.get("/livres/nouveau").await;
        let reply = browser.upload(&fields, ("livre.epub", &file)).await;
        assert_eq!(reply.header("x-ebc-refusal"), code, "{fields:?}: {}", reply.text());
        assert!(reply.text().contains("id=\"error\""));
    }
    assert!(shelf.store.blob_names().is_empty(), "nothing refused was kept");
    assert!(shelf.state.snapshot().books.is_empty());

    // Accepted: public domain with the year of death, the file stored by its hash.
    browser.get("/livres/nouveau").await;
    let reply = browser
        .upload(
            &[
                ("title", "Le Horla"),
                ("author", "Guy de Maupassant"),
                ("basis", "public_domain"),
                ("death_year", "1893"),
                ("declared", "1"),
                ("isbn", "2-07-036002-4"),
            ],
            ("../../le-horla.epub", &book),
        )
        .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text());
    let db = shelf.state.snapshot();
    let stored = &db.books[0];
    assert_eq!(stored.sha256, crypto::sha256_hex(&book));
    assert_eq!(shelf.store.blob_names(), vec![stored.sha256.clone()]);
    assert_eq!(stored.file_name, "le-horla.epub");
    assert_eq!(stored.isbn13.as_deref(), Some("9782070360024"));
    assert_eq!(
        (stored.declaration.basis.as_str(), stored.declaration.details.as_str()),
        ("public_domain", "†1893")
    );
    assert_eq!(stored.declaration.declared_at, T0);
    assert_eq!(stored.declaration.text_version, ebc_shelf::DECLARATION_VERSION);
    assert!(
        stored.declaration.declarant.contains("Jeanne Roy")
            && stored.declaration.declarant.contains("autrice@exemple.fr")
    );
    assert_eq!(db.content_log[0].operation, "upload");
    let page = browser.get(reply.location()).await.text();
    assert!(page.contains("id=\"declaration\"") && page.contains("Le Horla"));

    // Quota per account: a second, different file no longer fits once full.
    let shelf = shelf_with(|c| c.account_quota_bytes = 3_000).await;
    let mut browser = signed_in(&shelf, "autrice@exemple.fr").await;
    let small = epub("Un", None, &[]);
    assert!(small.len() < 1_500);
    upload_ok(&mut browser, "Un", "", &small).await;
    let mut noise = Vec::new();
    let mut x: u32 = 12_345;
    for _ in 0..2_000 {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        noise.push((x >> 16) as u8);
    }
    browser.get("/livres/nouveau").await;
    let reply =
        browser.upload(&base, ("deux.epub", &epub("Deux", None, &[("noise.bin", &noise)]))).await;
    assert_eq!(reply.header("x-ebc-refusal"), "account_quota_exceeded");
}

#[tokio::test]
async fn gifts_are_created_at_the_core_and_the_link_endpoint_checks_its_signature() {
    let shelf = shelf().await;
    let mut browser = signed_in(&shelf, "autrice@exemple.fr").await;
    let bytes = epub("Les Vagues du soir", None, &[]);
    let book = upload_ok(&mut browser, "Les Vagues du soir", "", &bytes).await;
    let (gift, core_id, code) = make_gift(&mut browser, &book).await;
    let page = browser.get(&format!("/livres/{book}")).await.text();
    assert!(page.contains(&format!("id=\"code-{gift}\"")) && page.contains(&code));
    // The designer link keeps the code out of the query string.
    let designer = page
        .split(&format!("id=\"designer-{gift}\" href=\""))
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .to_owned();
    assert!(
        designer.contains("/app/#code=")
            && designer.contains("&amp;title=Les%20Vagues%20du%20soir"),
        "{designer}"
    );
    assert!(designer.contains(&code.replace('-', "%2D")) && !designer.contains('?'), "{designer}");

    // The card comes from the core's signed card.pdf.
    let card = browser.get(&format!("/cadeaux/{gift}/carte.pdf?profile=home-a4")).await;
    assert_eq!((card.status, card.header("content-type")), (StatusCode::OK, "application/pdf"));
    assert!(
        card.body.starts_with(b"%PDF-")
            && card.header("x-ebc-card-warnings") == "cover_credit_missing"
    );
    assert_eq!(
        browser.get(&format!("/cadeaux/{gift}/carte.pdf?profile=evil")).await.status,
        StatusCode::NOT_FOUND
    );

    // §4.5: forged, stale, foreign, malformed requests are refused; nothing is counted.
    let secret = shelf.state.config.partner_secret.clone();
    assert_eq!(
        link_request(&shelf, &core_id, &gift, T0, "wrong-secret-0123456789").await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        link_request(&shelf, &core_id, &gift, T0 - 901, &secret).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        link_request(&shelf, &core_id, &gift, T0 + 301, &secret).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        link_request(&shelf, &core_id, "c_other", T0, &secret).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        link_request(&shelf, "g_999", &gift, T0, &secret).await.status,
        StatusCode::NOT_FOUND
    );
    let mut raw = Browser::new(&shelf);
    assert_eq!(raw.get("/api/ebc/link?ebc_gift=g_1").await.status, StatusCode::BAD_REQUEST);
    assert_eq!(shelf.state.snapshot().gifts[0].deliveries, 0);

    // A good request: a link on the shelf's own host, short-lived.
    let answer = link_request(&shelf, &core_id, &gift, T0 - 30, &secret).await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.text());
    let json: serde_json::Value = serde_json::from_slice(&answer.body).unwrap();
    let url = json["download_url"].as_str().unwrap();
    assert!(url.starts_with("http://127.0.0.1:8104/f/"), "{url}");
    assert_eq!(json["media_type"], "application/epub+zip");
    assert_eq!(json["sha256"], crypto::sha256_hex(&bytes));
    assert_eq!(json["size_bytes"], bytes.len());
    assert_eq!(json["expires_at"], T0 + 300);
    assert_eq!(shelf.state.snapshot().gifts[0].deliveries, 1, "one claim counted");
    // The file: byte-identical, with Range and HEAD.
    let path = path_of(url);
    let mut device = Browser::new(&shelf);
    let full = device.get(&path).await;
    assert_eq!(full.status, StatusCode::OK);
    assert_eq!(full.body, bytes, "delivered unchanged");
    assert!(full.header("content-disposition").starts_with("attachment; filename=\"livre.epub\""));
    assert_eq!(full.header("accept-ranges"), "bytes");
    let request =
        Request::builder().uri(&path).header("range", "bytes=0-9").body(Body::empty()).unwrap();
    let part = device.send(request).await;
    assert_eq!(part.header("content-range"), format!("bytes 0-9/{}", bytes.len()));
    assert_eq!((part.status, part.body.as_slice()), (StatusCode::PARTIAL_CONTENT, &bytes[..10]));
    let head = device
        .send(Request::builder().method("HEAD").uri(&path).body(Body::empty()).unwrap())
        .await;
    assert_eq!((head.status, head.body.len()), (StatusCode::OK, 0));
    assert_eq!(head.header("content-length"), bytes.len().to_string());
    // Downloads are logged with a pseudonym, never an address.
    let deliveries = shelf.state.snapshot().deliveries;
    assert_eq!(deliveries.len(), 2, "full download and the range from byte 0");
    assert!(
        deliveries
            .iter()
            .all(|d| d.client.as_deref().is_some_and(|c| c.len() == 16 && !c.contains('.')))
    );
    // Wrong name, unknown token, expired link.
    assert_eq!(
        device.get(&path.replace("livre.epub", "autre.epub")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        device.get(&format!("/f/{}/livre.epub", "0".repeat(32))).await.status,
        StatusCode::NOT_FOUND
    );
    shelf.clock.advance(301);
    assert_eq!(device.get(&path).await.status, StatusCode::NOT_FOUND);

    // Reissue and cancel go through the core; a cancelled gift gets no link.
    assert_eq!(
        browser.post(&format!("/cadeaux/{gift}/nouveau-code"), &[]).await.status,
        StatusCode::SEE_OTHER
    );
    let reissued = ebc_model::GiftCode::from_payload(999).unwrap().grouped();
    assert_eq!(shelf.state.snapshot().gifts[0].code.as_deref(), Some(reissued.as_str()));
    assert_eq!(
        browser.post(&format!("/cadeaux/{gift}/annuler"), &[]).await.status,
        StatusCode::SEE_OTHER
    );
    let calls = shelf.core.calls.lock().unwrap().clone();
    assert!(
        calls.iter().any(|c| c == &format!("reissue {core_id}"))
            && calls.iter().any(|c| c == &format!("cancel {core_id}")),
        "{calls:?}"
    );
    let now = shelf.state.now();
    assert_eq!(link_request(&shelf, &core_id, &gift, now, &secret).await.status, StatusCode::GONE);
    let gift_record = &shelf.state.snapshot().gifts[0];
    assert!(
        gift_record.recipient_name.is_none() && gift_record.code.is_none(),
        "personal data dropped on cancellation"
    );
}

#[tokio::test]
async fn a_core_outage_leaves_gifts_pending_and_they_can_be_retried() {
    let shelf = shelf().await;
    let mut browser = signed_in(&shelf, "autrice@exemple.fr").await;
    let book = upload_ok(&mut browser, "Un", "", &epub("Un", None, &[])).await;
    *shelf.core.down.lock().unwrap() = true;
    browser.get(&format!("/livres/{book}")).await;
    let reply =
        browser.post(&format!("/livres/{book}/cadeaux"), &[("count", "2"), ("locale", "en")]).await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
    assert!(reply.text().contains("id=\"error\""));
    let gifts = shelf.state.snapshot().gifts;
    assert_eq!(gifts.len(), 2);
    assert!(gifts.iter().all(|g| !g.is_live()));
    *shelf.core.down.lock().unwrap() = false;
    for gift in &gifts {
        assert_eq!(
            browser.post(&format!("/cadeaux/{}/reessayer", gift.id), &[]).await.status,
            StatusCode::SEE_OTHER
        );
    }
    assert!(shelf.state.snapshot().gifts.iter().all(ebc_shelf::store::Gift::is_live));
    let refused = browser
        .post(&format!("/livres/{book}/cadeaux"), &[("count", "51"), ("locale", "fr")])
        .await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_upheld_notice_takes_the_book_down_and_its_hash_and_isbn_stay_out() {
    let shelf = shelf().await;
    let mut author = signed_in(&shelf, "autrice@exemple.fr").await;
    let bytes = epub("Contrefait", Some("9782070360024"), &[]);
    let book = upload_ok(&mut author, "Contrefait", "", &bytes).await;
    let (gift, core_id, code) = make_gift(&mut author, &book).await;
    assert_eq!(shelf.state.snapshot().books[0].embedded_isbns, vec!["9782070360024".to_owned()]);

    // A notice without an account; the code printed on a card is enough to locate it.
    let mut notifier = Browser::new(&shelf);
    let incomplete = notifier
        .post(
            "/signaler",
            &[("category", "copyright"), ("location", &code), ("explanation", "Mon roman.")],
        )
        .await;
    assert_eq!(
        incomplete.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "name, e-mail and good faith are required"
    );
    let reply = notifier
        .post(
            "/signaler",
            &[
                ("category", "copyright"),
                ("location", &format!("la carte porte le code {}", code.to_lowercase())),
                ("explanation", "C’est mon roman, publié chez mon éditeur."),
                ("name", "Ayant Droit"),
                ("email", "droits@editeur.example"),
                ("good_faith", "1"),
            ],
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let notice = shelf.state.snapshot().notices[0].clone();
    assert_eq!(notice.book_id.as_deref(), Some(book.as_str()), "located from the code");
    let ack = shelf.mail.sent_to("droits@editeur.example");
    assert!(ack[0].subject.contains(&notice.id), "acknowledged");

    // Only the operator reviews.
    assert_eq!(author.get("/moderation").await.status, StatusCode::FORBIDDEN);
    let mut operator = signed_in(&shelf, OPERATOR).await;
    let page = operator.get(&format!("/moderation/signalements/{}", notice.id)).await.text();
    assert!(
        page.contains("id=\"resolved\"") && page.contains("Jeanne Roy"),
        "the uploader is identified"
    );
    let reply = operator
        .post(
            &format!("/moderation/signalements/{}", notice.id),
            &[
                ("decision", "uphold"),
                ("statement", "Œuvre d’un tiers, sous contrat d’édition."),
                ("false_declaration", "1"),
            ],
        )
        .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text());

    let db = shelf.state.snapshot();
    assert!(!db.books[0].is_live());
    assert!(shelf.store.blob_names().is_empty(), "file deleted");
    assert!(
        shelf.core.calls.lock().unwrap().contains(&format!("cancel {core_id}")),
        "gift cancelled at the core"
    );
    assert_eq!(
        link_request(
            &shelf,
            &core_id,
            &gift,
            shelf.state.now(),
            &shelf.state.config.partner_secret
        )
        .await
        .status,
        StatusCode::GONE
    );
    assert_eq!(db.blocklist.len(), 2, "hash and ISBN");
    let account = db.accounts.iter().find(|a| a.email == "autrice@exemple.fr").unwrap();
    assert_eq!((account.upheld_notices, account.false_declarations), (1, 1));
    assert!(
        matches!(account.status, ebc_shelf::store::AccountStatus::Suspended { .. }),
        "one false declaration suspends"
    );
    // Statement of reasons, with the redress options; the notifier is told.
    let sor = shelf.mail.sent_to("autrice@exemple.fr").pop().unwrap();
    for needle in
        ["art. 17", "Contrefait", "Œuvre d’un tiers", "/compte/decisions/", "art. 21", "suspendu"]
    {
        assert!(sor.text.contains(needle), "{needle} missing from:\n{}", sor.text);
    }
    assert!(shelf.mail.sent_to("droits@editeur.example").pop().unwrap().text.contains("retiré"));

    // Re-upload refused by hash, then by ISBN (another file), for anybody.
    let mut other = signed_in(&shelf, "autre@exemple.fr").await;
    other.get("/livres/nouveau").await;
    let fields =
        [("title", "Contrefait"), ("author", "X"), ("basis", "own_work"), ("declared", "1")];
    assert_eq!(
        other.upload(&fields, ("a.epub", &bytes)).await.header("x-ebc-refusal"),
        "blocked_file"
    );
    let renamed = epub("Contrefait (bis)", Some("9782070360024"), &[]);
    assert_eq!(
        other.upload(&fields, ("b.epub", &renamed)).await.header("x-ebc-refusal"),
        "blocked_isbn"
    );
    let declared = [
        ("title", "Contrefait"),
        ("author", "X"),
        ("basis", "own_work"),
        ("declared", "1"),
        ("isbn", "978-2-07-036002-4"),
    ];
    assert_eq!(
        other
            .upload(&declared, ("c.epub", &epub("Autre", None, &[])))
            .await
            .header("x-ebc-refusal"),
        "blocked_isbn"
    );
    // The suspended author can no longer upload.
    let page = author.get("/compte").await.text();
    assert!(page.contains("id=\"suspended\"") && !page.contains("id=\"upload-link\""));

    // The internal complaint: granted, the strike and the blocklist entries go.
    let decision = author.get(&format!("/compte/decisions/{}", notice.id)).await.text();
    assert!(decision.contains("id=\"appeal-form\""));
    let filed = author
        .post(
            &format!("/compte/decisions/{}", notice.id),
            &[("text", "J’ai les droits : contrat joint.")],
        )
        .await;
    assert_eq!(filed.status, StatusCode::SEE_OTHER);
    let appeal = shelf.state.snapshot().appeals[0].id.clone();
    operator.get("/moderation").await;
    let reply = operator
        .post(
            &format!("/moderation/reclamations/{appeal}"),
            &[("grant", "1"), ("answer", "Contrat vérifié.")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
    let db = shelf.state.snapshot();
    assert!(db.blocklist.is_empty());
    let account = db.accounts.iter().find(|a| a.email == "autrice@exemple.fr").unwrap();
    assert_eq!(
        (account.upheld_notices, account.false_declarations, account.status.clone()),
        (0, 0, ebc_shelf::store::AccountStatus::Active)
    );
    assert!(shelf.mail.sent_to("autrice@exemple.fr").pop().unwrap().text.contains("acceptée"));
}

#[tokio::test]
async fn two_upheld_notices_suspend_and_a_third_bans() {
    let shelf = shelf().await;
    let mut author = signed_in(&shelf, "recidive@exemple.fr").await;
    let mut operator = signed_in(&shelf, OPERATOR).await;
    let mut takedown = async |n: usize, author: &mut Browser<'_>| {
        let book =
            upload_ok(author, &format!("Livre {n}"), "", &epub(&format!("Livre {n}"), None, &[]))
                .await;
        let mut notifier = Browser::new(&shelf);
        notifier
            .post(
                "/signaler",
                &[
                    ("category", "copyright"),
                    ("location", &format!("/livres/{book}")),
                    ("explanation", "x"),
                    ("name", "N"),
                    ("email", "n@exemple.fr"),
                    ("good_faith", "1"),
                ],
            )
            .await;
        let notice = shelf.state.snapshot().notices.last().unwrap().id.clone();
        operator.get("/moderation").await;
        let reply = operator
            .post(
                &format!("/moderation/signalements/{notice}"),
                &[("decision", "uphold"), ("statement", "Contrefaçon.")],
            )
            .await;
        assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text());
        book
    };
    let status = |shelf: &Shelf| {
        shelf
            .state
            .snapshot()
            .accounts
            .iter()
            .find(|a| a.email == "recidive@exemple.fr")
            .unwrap()
            .status
            .clone()
    };
    takedown(1, &mut author).await;
    assert_eq!(status(&shelf), ebc_shelf::store::AccountStatus::Active);
    let kept = upload_ok(&mut author, "Gardé", "", &epub("Gardé", None, &[])).await;
    takedown(2, &mut author).await;
    assert!(matches!(status(&shelf), ebc_shelf::store::AccountStatus::Suspended { .. }));
    // A suspended account's other books keep being delivered; a further upheld notice
    // (about the kept book) bans the account and removes everything.
    let mut notifier = Browser::new(&shelf);
    notifier
        .post(
            "/signaler",
            &[
                ("category", "other"),
                ("location", &kept),
                ("explanation", "x"),
                ("name", "N"),
                ("email", "n@exemple.fr"),
                ("good_faith", "1"),
            ],
        )
        .await;
    let notice = shelf.state.snapshot().notices.last().unwrap().id.clone();
    operator.get("/moderation").await;
    operator
        .post(
            &format!("/moderation/signalements/{notice}"),
            &[("decision", "uphold"), ("statement", "Encore.")],
        )
        .await;
    assert!(matches!(status(&shelf), ebc_shelf::store::AccountStatus::Banned { .. }));
    assert!(shelf.state.snapshot().books.iter().all(|b| !b.is_live()));
    assert_eq!(author.get("/compte").await.location(), "/connexion", "sessions ended");
    // A rejected notice changes nothing.
    let mut notifier = Browser::new(&shelf);
    notifier
        .post(
            "/signaler",
            &[
                ("category", "other"),
                ("location", "inconnu"),
                ("explanation", "x"),
                ("exception", "1"),
                ("good_faith", "1"),
            ],
        )
        .await;
    let notice = shelf.state.snapshot().notices.last().unwrap().clone();
    assert!(notice.exception && notice.notifier_email.is_empty(), "art. 16(2)(c) exception");
    operator.get("/moderation").await;
    let reply = operator
        .post(
            &format!("/moderation/signalements/{}", notice.id),
            &[("decision", "reject"), ("statement", "Aucun contenu identifié.")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn retention_purges_what_it_must_when_it_must() {
    let shelf = shelf().await;
    let mut author = signed_in(&shelf, "autrice@exemple.fr").await;
    let book = upload_ok(&mut author, "Un", "", &epub("Un", None, &[])).await;
    let (gift, _, _) = make_gift(&mut author, &book).await;
    author.get(&format!("/livres/{book}")).await;
    author.post(&format!("/cadeaux/{gift}/annuler"), &[]).await;
    let r = shelf.state.config.retention.clone();
    let day = 86_400;
    // The file of a book whose gifts were all cancelled goes after the idle period.
    shelf.clock.advance(r.idle_file_days * day + 1);
    shelf.state.maintain().await;
    let db = shelf.state.snapshot();
    assert!(!db.books[0].is_live() && shelf.store.blob_names().is_empty());
    // Connection data after a year; the declaration after the limitation period.
    assert!(
        db.content_log.iter().any(|e| e.connection.as_deref() == Some("192.0.2.10")),
        "{:?}",
        db.content_log
    );
    shelf.clock.advance(r.connection_days * day);
    shelf.state.purge();
    assert!(shelf.state.snapshot().content_log.iter().all(|e| e.connection.is_none()));
    shelf.clock.advance(r.declaration_days * day);
    shelf.state.purge();
    let db = shelf.state.snapshot();
    assert!(
        db.books.is_empty() && db.gifts.is_empty(),
        "declaration kept for the period, then deleted"
    );
    // Sessions expire (seven days); a closed account keeps its identity for the legal
    // period, cannot come back under the same address meanwhile, then is erased.
    assert_eq!(author.get("/compte").await.location(), "/connexion");
    let mut author = signed_in(&shelf, "autrice@exemple.fr").await;
    author.get("/compte").await;
    assert_eq!(author.post("/compte/fermer", &[("confirm", "1")]).await.status, StatusCode::OK);
    assert!(shelf.state.snapshot().accounts[0].identity.is_some(), "kept for the legal period");
    let mails = shelf.mail.sent_to("autrice@exemple.fr").len();
    let mut again = Browser::new(&shelf);
    assert_eq!(
        again.post("/connexion", &[("email", "autrice@exemple.fr")]).await.status,
        StatusCode::OK
    );
    assert_eq!(
        shelf.mail.sent_to("autrice@exemple.fr").len(),
        mails,
        "no link for a closed account"
    );
    shelf.clock.advance(r.identity_days * day + 1);
    shelf.state.purge();
    let account = shelf.state.snapshot().accounts[0].clone();
    assert!(account.identity.is_none() && account.email.is_empty() && account.erased_at.is_some());
}
