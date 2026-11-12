//! A `partner_file` gift of a partner that is not a store — standing in for the "shelf"
//! service of the two-service model (a partner hosting files for their rights holders;
//! `docs/architecture.md`). The core never holds the file: on each claim it asks the
//! partner's signed endpoint for a short-lived link and redirects the device to it.
//!
//! Covers the reveal date, the reissue of a lost card and the cancellation through the
//! partner API (what the creator API used to do), and a partner whose link points to a
//! host outside its `redirect_hosts`: refused, no claim counted, nothing downloaded.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use ebc_demo_ereaders::fixtures;
use ebc_demo_ereaders::{DeviceProfile, FileFormat};
use ebc_demo_integration::{T0, World};
use ebc_model::GiftCode;
use ebc_server::Partner;
use serde_json::json;

use crate::support::{assert_lint_clean, browser, http_post_form, sha256_hex};

const PARTNER_ID: &str = "etagere-demo";
const PARTNER_SECRET: &str = "in-test-secret-etagere-0123456789";

/// The stand-in shelf: `GET /ebc/file` (signed, `docs/api.md` §4.5) answers a JSON link
/// to `/dl/{token}/le-horla.epub`; `GET /ebc/elsewhere` answers a link on a host it never
/// declared; `GET /dl/…` serves the file.
#[derive(Clone)]
struct Shelf {
    base: String,
    epub: Arc<Vec<u8>>,
    links: Arc<AtomicUsize>,
}

async fn link(
    State(shelf): State<Shelf>,
    Path(kind): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let field = |name: &str| query.get(name).map(String::as_str).unwrap_or("");
    let Ok(ts) = field("ebc_ts").parse::<i64>() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let genuine = field("ebc_sig")
        == ebc_server::codes::redirect_signature(
            PARTNER_SECRET,
            field("ebc_gift"),
            field("ebc_order"),
            ts,
        );
    if !genuine {
        return StatusCode::FORBIDDEN.into_response();
    }
    let n = shelf.links.fetch_add(1, Ordering::SeqCst) + 1;
    let host = if kind == "elsewhere" {
        "http://files.elsewhere.example".to_owned()
    } else {
        shelf.base.clone()
    };
    let answer = json!({
        "download_url": format!("{host}/dl/{n}/le-horla.epub"),
        "file_name": "Le Horla.epub",
        "media_type": "application/epub+zip",
        "size_bytes": shelf.epub.len(),
        "sha256": sha256_hex(&shelf.epub),
        "expires_at": ts + 300,
    });
    ([(header::CONTENT_TYPE, "application/json")], answer.to_string()).into_response()
}

async fn file(State(shelf): State<Shelf>) -> Response {
    ([(header::CONTENT_TYPE, "application/epub+zip")], shelf.epub.as_ref().clone()).into_response()
}

async fn start_shelf() -> Shelf {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let shelf = Shelf {
        base: format!("http://{}", listener.local_addr().unwrap()),
        epub: Arc::new(fixtures::epub("Le Horla", "Guy de Maupassant")),
        links: Arc::new(AtomicUsize::new(0)),
    };
    let router = Router::new()
        .route("/ebc/{kind}", get(link))
        .route("/dl/{n}/le-horla.epub", get(file))
        .with_state(shelf.clone());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    shelf
}

async fn world_with(shelf: &Shelf) -> World {
    World::builder()
        .partner(Partner {
            id: PARTNER_ID.to_owned(),
            name: "Étagère (démo)".to_owned(),
            secret: PARTNER_SECRET.to_owned(),
            webhook_url: None,
            redirect_hosts: vec![shelf.base.trim_start_matches("http://").to_owned()],
        })
        .start()
        .await
        .unwrap()
}

#[tokio::test]
async fn a_shelf_gift_stays_wrapped_until_its_reveal_date_and_its_partner_manages_it() {
    let shelf = start_shelf().await;
    let world = world_with(&shelf).await;
    let api = world.partner_api_as(PARTNER_ID, PARTNER_SECRET);

    // --- the gift, to be revealed tomorrow, in English ---
    let reveal_at = T0 + 86_400;
    let created = api
        .post_json(
            "/api/partner/v1/gifts",
            &json!({
                "order_ref": "SHELF-0001",
                "book": { "title": "Le Horla", "contributors": [{"name": "Guy de Maupassant"}], "language": "fr" },
                "giver_name": "Camille", "recipient_name": "Sacha", "message": "Open it on your birthday!",
                "locale": "en", "reveal_at": reveal_at,
                "delivery": { "type": "partner_file", "format": "epub", "url": format!("{}/ebc/file", shelf.base) },
            }),
            Some("SHELF-0001"),
        )
        .await
        .unwrap();
    assert_eq!(created.status, 201, "{}", created.text());
    let gift = created.json();
    let id = gift["id"].as_str().unwrap().to_owned();
    let code = GiftCode::parse(gift["code"].as_str().unwrap()).unwrap();
    assert_eq!(gift["code_display"], code.grouped().as_str());
    let claim_url = gift["claim_url"].as_str().unwrap().to_owned();
    assert_eq!(claim_url, ebc_model::claim_url(&world.server.base, &code));
    assert!(gift.get("manage_token").is_none());

    // --- before the date: the page shows who it is from and when, and no action ---
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&claim_url).await.unwrap();
    let page = kobo.page().unwrap();
    assert_eq!(page.status(), 200);
    assert_eq!(page.lang(), Some("en"), "pages are served in the gift's locale");
    assert!(page.has("reveal-date") && page.text_of("gift-from").unwrap().contains("Camille"));
    assert!(!page.has("download") && !page.has("gift-title"), "the title stays a surprise");
    assert_lint_clean(&kobo, "the wrapped gift page");
    let ua = DeviceProfile::kobo_clara().user_agent;
    for action in ["download", "pair"] {
        let early = http_post_form(&format!("{claim_url}/{action}"), ua, &[]).await;
        assert_ne!(early.status, 303, "{action} before the reveal date");
        assert!(!early.text().contains("id=\"pair-code\""), "{action} before the reveal date");
    }
    assert_eq!(shelf.links.load(Ordering::SeqCst), 0, "the partner is asked nothing yet");

    // --- the day comes: the Kobo follows the core's session URL to the shelf's link ---
    world.advance(86_400 + 1);
    kobo.open(&claim_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().text_of("gift-title"), Some("Le Horla"));
    let entry = kobo.click("download").await.unwrap().download().unwrap().clone();
    assert_eq!((entry.title.as_str(), entry.format), ("Le Horla", FileFormat::Epub));
    assert_eq!(entry.sha256, sha256_hex(&shelf.epub), "the shelf's file, byte for byte");
    assert_eq!(shelf.links.load(Ordering::SeqCst), 1);
    let hops: Vec<String> = kobo.exchanges().iter().map(|x| x.url.to_string()).collect();
    assert!(hops.iter().any(|u| u.starts_with(&format!("{}/dl/", shelf.base))), "{hops:#?}");

    // --- the partner follows the gift (and nobody else can) ---
    let path = format!("/api/partner/v1/gifts/{id}");
    let status = api.gift_status(&id).await.unwrap();
    assert_eq!(
        (status.json()["status"].as_str(), status.json()["downloads"].as_u64()),
        (Some("redeemed"), Some(1))
    );
    let store = world.partner_api(ebc_demo_stores::StoreKind::Septante);
    assert_eq!(store.get(&path).await.unwrap().status, 404, "another partner's gift");

    // --- a lost card: a new code, the old one answers 410 ---
    let reissued = api.send("POST", &format!("{path}/reissue"), Vec::new(), &[]).await.unwrap();
    assert_eq!(reissued.status, 200, "{}", reissued.text());
    let new_code = GiftCode::parse(reissued.json()["code"].as_str().unwrap()).unwrap();
    assert_ne!(new_code, code);
    kobo.open(&claim_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 410);
    let new_url = reissued.json()["claim_url"].as_str().unwrap().to_owned();
    kobo.open(&new_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 200);
    assert!(kobo.page().unwrap().has("download"));

    // --- cancelled: the code stops working ---
    world.advance(1);
    let cancelled = api.send("POST", &format!("{path}/cancel"), Vec::new(), &[]).await.unwrap();
    assert_eq!(cancelled.status, 200, "{}", cancelled.text());
    kobo.open(&new_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 410);
    assert_eq!(api.gift_status(&id).await.unwrap().json()["status"], "cancelled");
}

#[tokio::test]
async fn a_download_link_on_an_undeclared_host_is_refused_and_costs_no_claim() {
    let shelf = start_shelf().await;
    let world = world_with(&shelf).await;
    let api = world.partner_api_as(PARTNER_ID, PARTNER_SECRET);
    let created = api
        .post_json(
            "/api/partner/v1/gifts",
            &json!({
                "order_ref": "SHELF-0002",
                "book": { "title": "Le Horla", "contributors": [{"name": "Guy de Maupassant"}] },
                "delivery": { "type": "partner_file", "format": "epub", "url": format!("{}/ebc/elsewhere", shelf.base) },
            }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(created.status, 201, "{}", created.text());
    let gift = created.json();
    let claim_url = gift["claim_url"].as_str().unwrap().to_owned();

    // The file endpoint itself must be on a declared host, at creation already.
    let undeclared = api
        .post_json(
            "/api/partner/v1/gifts",
            &json!({
                "order_ref": "SHELF-0003",
                "book": { "title": "Le Horla" },
                "delivery": { "type": "partner_file", "format": "epub", "url": "http://files.elsewhere.example/ebc/file" },
            }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        (undeclared.status, undeclared.error_code().as_deref()),
        (422, Some("redirect_host_not_allowed"))
    );

    // The Kobo presses "download": the partner answered, but with a link the core will
    // not send a device to. The recipient is told to try later; nothing is counted.
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&claim_url).await.unwrap();
    let outcome = kobo.click("download").await.unwrap();
    assert!(outcome.is_page(), "no download");
    let page = kobo.page().unwrap();
    assert_eq!(page.status(), 502, "{}", page.text());
    assert!(page.has("error") && page.has("download"), "the button stays for a retry");
    assert_lint_clean(&kobo, "the refused download");
    assert!(kobo.library().is_empty());
    assert!(
        kobo.exchanges().iter().all(|x| x.url.host_str() != Some("files.elsewhere.example")),
        "the device was never sent to the undeclared host"
    );
    assert_eq!(shelf.links.load(Ordering::SeqCst), 1, "the partner was asked once");
    let status = api.gift_status(gift["id"].as_str().unwrap()).await.unwrap().json();
    assert_eq!(
        (status["status"].as_str(), status["downloads"].as_u64()),
        (Some("created"), Some(0))
    );
}
