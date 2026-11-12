//! (f) `reseller_licence`: neither demo store issues licence files, so this journey talks
//! to the partner API directly, as a third (in-test) partner whose tiny licence endpoint
//! issues **simulated** LCP licences (`ebc_demo_ereaders::lcp`: the shape of the flow, no
//! cryptography). The recipient presses `redeem`, the server fetches the licence from the
//! partner with a signed request and hands it over through a download session; the
//! reading system opens it, asks for the passphrase and fetches the publication.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use ebc_demo_ereaders::fixtures::EpubBuilder;
use ebc_demo_ereaders::{
    DeviceProfile, DrmScheme, Error, FileFormat, ImportSource, LcpError, LicenceStep, Outcome,
    SimulatedLcpReader, SimulatedLicenceBuilder,
};
use ebc_demo_integration::{T0, World};
use ebc_server::Partner;
use serde_json::json;

use crate::support::{assert_lint_clean, browser, sha256_hex};

const PARTNER_ID: &str = "librairie-lcp";
const PARTNER_SECRET: &str = "in-test-secret-librairie-lcp-0123456789";
const PASSPHRASE: &str = "horla-1887";

/// The in-test licence provider: `GET /licence` (signed like a redemption redirect,
/// `docs/api.md` §4.3) and `GET /pub/le-horla.epub` (the "protected" publication).
#[derive(Clone)]
struct Provider {
    base: String,
    publication: Arc<Vec<u8>>,
    licences_issued: Arc<AtomicUsize>,
}

async fn licence(
    State(provider): State<Provider>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let field = |name: &str| query.get(name).map(String::as_str).unwrap_or("");
    let (gift, order, signature) = (field("ebc_gift"), field("ebc_order"), field("ebc_sig"));
    let Ok(ts) = field("ebc_ts").parse::<i64>() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let genuine =
        signature == ebc_server::codes::redirect_signature(PARTNER_SECRET, gift, order, ts);
    // The test's clock never leaves the first hour after T0.
    if !genuine || !(T0..T0 + 3600).contains(&ts) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let n = provider.licences_issued.fetch_add(1, Ordering::SeqCst) + 1;
    let document = SimulatedLicenceBuilder::new(
        &format!("lic-{order}-{n}"),
        "https://librairie-lcp.example",
        &format!("{}/pub/le-horla.epub", provider.base),
        PASSPHRASE,
    )
    .text_hint("Le mot de passe indiqué dans votre carte")
    .describing(&provider.publication)
    .build();
    ([(header::CONTENT_TYPE, "application/vnd.readium.lcp.license.v1.0+json")], document)
        .into_response()
}

async fn publication(State(provider): State<Provider>) -> Response {
    ([(header::CONTENT_TYPE, "application/epub+zip")], provider.publication.as_ref().clone())
        .into_response()
}

async fn start_provider() -> Provider {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let provider = Provider {
        base: format!("http://{}", listener.local_addr().unwrap()),
        publication: Arc::new(
            EpubBuilder::new("Le Horla", "Guy de Maupassant").with_lcp_marker().build(),
        ),
        licences_issued: Arc::new(AtomicUsize::new(0)),
    };
    let router = Router::new()
        .route("/licence", get(licence))
        .route("/pub/le-horla.epub", get(publication))
        .with_state(provider.clone());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    provider
}

#[tokio::test]
async fn a_licence_gift_is_fetched_from_the_partner_and_opened_by_an_lcp_reader() {
    let provider = start_provider().await;
    let host = provider.base.trim_start_matches("http://").to_owned();
    let world = World::builder()
        .partner(Partner {
            id: PARTNER_ID.to_owned(),
            name: "Librairie LCP (démo)".to_owned(),
            secret: PARTNER_SECRET.to_owned(),
            webhook_url: None,
            redirect_hosts: vec![host],
        })
        .start()
        .await
        .unwrap();
    let api = world.partner_api_as(PARTNER_ID, PARTNER_SECRET);

    // --- the partner creates the gift (signed, idempotent) ---
    let request = json!({
        "order_ref": "LCP-2026-000001",
        "book": { "title": "Le Horla", "contributors": [{"name": "Guy de Maupassant"}], "language": "fr" },
        "giver_name": "Camille", "recipient_name": "Sacha", "message": "Frissons garantis.",
        "locale": "fr",
        "delivery": { "type": "reseller_licence", "format": "lcpl", "url": format!("{}/licence", provider.base) },
    });
    let created =
        api.post_json("/api/partner/v1/gifts", &request, Some("LCP-2026-000001")).await.unwrap();
    assert_eq!(created.status, 201, "{}", created.text());
    let gift = created.json();
    let claim_url = gift["claim_url"].as_str().unwrap().to_owned();
    let gift_id = gift["id"].as_str().unwrap().to_owned();
    // Replaying the creation returns the same gift and the same code.
    let replay =
        api.post_json("/api/partner/v1/gifts", &request, Some("LCP-2026-000001")).await.unwrap();
    assert_eq!((replay.status, &replay.json()["code"]), (201, &gift["code"]));
    // A licence URL on a host the partner did not declare is refused at creation.
    let mut elsewhere = request.clone();
    elsewhere["delivery"]["url"] = json!("http://127.0.0.1:9/licence");
    elsewhere["order_ref"] = json!("LCP-2026-000002");
    let refused = api.post_json("/api/partner/v1/gifts", &elsewhere, None).await.unwrap();
    assert_eq!(
        (refused.status, refused.error_code().as_deref()),
        (422, Some("redirect_host_not_allowed"))
    );

    // --- the recipient, in a reading application on a computer ---
    let mut app = browser(&world, DeviceProfile::thorium_desktop());
    assert!(app.open(&claim_url).await.unwrap().is_page());
    let page = app.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert_lint_clean(&app, "the licence gift page");
    assert_eq!(page.text_of("gift-title"), Some("Le Horla"));
    assert!(page.has("redeem") && !page.has("download"));
    let Outcome::Licence(file) = app.click("redeem").await.unwrap() else {
        panic!("pressing redeem did not hand a licence file to the reading system");
    };
    assert_eq!(file.format, FileFormat::Lcpl);
    assert!(file.file_name.ends_with(".lcpl"), "{}", file.file_name);
    assert!(file.url.path().starts_with("/d/"), "served through a download session");
    assert_eq!(provider.licences_issued.load(Ordering::SeqCst), 1);

    // --- the (simulated) LCP reading system ---
    let mut reader = SimulatedLcpReader::new(DeviceProfile::thorium_desktop()).unwrap();
    let step = reader.open_licence(&file.bytes, Some(&file.url)).await.unwrap();
    let LicenceStep::NeedsPassphrase(prompt) = step else {
        panic!("a first licence must ask for its passphrase: {step:?}");
    };
    assert_eq!(prompt.text_hint, "Le mot de passe indiqué dans votre carte");
    let wrong = reader.enter_passphrase("candide-1759").await.unwrap_err();
    assert!(matches!(wrong, Error::Lcp(LcpError::WrongPassphrase { .. })), "{wrong:?}");
    assert!(reader.library().is_empty());
    let entry = reader.enter_passphrase(PASSPHRASE).await.unwrap();
    assert_eq!((entry.title.as_str(), entry.format), ("Le Horla", FileFormat::Epub));
    assert_eq!(entry.sha256, sha256_hex(&provider.publication));
    assert_eq!(entry.drm, Some(DrmScheme::ReadiumLcp));
    assert!(matches!(entry.source, ImportSource::SimulatedLcp { .. }));

    // The same file in a plain browser download is refused by devices without LCP: the
    // licence path is what makes it readable.
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    assert!(kobo.open(&format!("{}/pub/le-horla.epub", provider.base)).await.is_err());
    assert!(SimulatedLcpReader::new(DeviceProfile::kobo_clara()).is_err());

    // --- the server's view ---
    let status = api.gift_status(&gift_id).await.unwrap().json();
    assert_eq!(
        (status["status"].as_str(), status["downloads"].as_u64()),
        (Some("redeemed"), Some(1))
    );
    // Licence gifts are not "fulfilled" by an account: that confirmation is refused.
    let confirm = api
        .post_json(
            &format!("/api/partner/v1/gifts/{gift_id}/fulfilled"),
            &json!({ "account_ref": "acct_1" }),
            None,
        )
        .await
        .unwrap();
    assert_eq!((confirm.status, confirm.error_code().as_deref()), (409, Some("wrong_delivery")));

    // Another partner cannot see or touch this gift.
    let septante = world.partner_api(world.septante.kind);
    assert_eq!(septante.gift_status(&gift_id).await.unwrap().status, 404);
}
