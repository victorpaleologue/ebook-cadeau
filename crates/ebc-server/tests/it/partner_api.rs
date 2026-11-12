//! The partner API (`docs/api.md` §4): request signatures, idempotency, the redemption
//! redirect, fulfilment, webhooks, uploads, cover and card rendering.

use std::sync::{Arc, Mutex};

use crate::common::{
    Harness, PARTNER_ID, PARTNER_SECRET, Receiver, T0, UA_KOBO, UA_PHONE, assert_page, epub,
    epub_with, partner,
};
use axum::body::{Body, Bytes};
use axum::http::{Method, StatusCode};
use ebc_server::codes::{redirect_signature, request_signature, verify_request_signature};
use ebc_server::{
    AppState, CardRenderError, CardRenderer, CardRequest, CoverRights, ManualClock, NoCardRenderer,
    app,
};
use serde_json::json;

const STORE_URL: &str = "https://store.example/cadeau/activer";

fn redirect_harness() -> Harness {
    Harness::with(|c| c.partners = vec![partner(&["store.example"], None)])
}

fn redirect_delivery() -> serde_json::Value {
    json!({ "type": "reseller_redirect", "url": STORE_URL })
}

/// What a store sends, absent values included (empty strings, as the demo stores do).
fn gift_body(order_ref: &str) -> serde_json::Value {
    json!({
        "order_ref": order_ref,
        "book": { "title": "Madame Bovary", "isbn13": "", "contributors": [{"name": "Gustave Flaubert"}], "publisher": "", "language": "fr" },
        "giver_name": "Camille", "recipient_name": "Sacha", "message": "", "locale": "fr",
        "delivery": redirect_delivery(),
    })
}

fn error_code(reply: &crate::common::Reply) -> String {
    let body = reply.json();
    assert!(body["error"]["message"].is_string(), "error envelope: {body}");
    body["error"]["code"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn requests_must_be_signed_by_a_known_partner_within_the_time_window() {
    let h = redirect_harness();
    let path = "/api/partner/v1/gifts";
    let body = Bytes::from(gift_body("ORD-1").to_string());
    let send = |timestamp: i64, secret: &'static str, sent: Bytes| {
        let (h, signed) = (&h, body.clone());
        async move {
            // The signature covers `signed`; `sent` is what actually travels.
            let signature = request_signature(secret, timestamp, "POST", path, &signed);
            let timestamp = timestamp.to_string();
            let headers = [
                ("x-ebc-partner", PARTNER_ID),
                ("x-ebc-timestamp", timestamp.as_str()),
                ("x-ebc-signature", signature.as_str()),
            ];
            h.call(Method::POST, path, &headers, Body::from(sent)).await
        }
    };

    // Good.
    let created = send(T0, PARTNER_SECRET, body.clone()).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let gift = created.json();
    assert_eq!(gift["status"], "created");
    assert!(gift["id"].as_str().unwrap().starts_with("g_"));
    let code = gift["code"].as_str().unwrap();
    assert_eq!(gift["claim_url"], format!("https://ebk.example/k/{code}"));
    assert_eq!(gift["code_display"].as_str().unwrap().replace('-', ""), code);
    assert!(gift.get("manage_token").is_none(), "partners authenticate with their signature");

    // Wrong secret, tampered body.
    let wrong = send(T0, "another-secret-0123456789", body.clone()).await;
    assert_eq!(
        (wrong.status, error_code(&wrong).as_str()),
        (StatusCode::UNAUTHORIZED, "bad_signature")
    );
    let tampered = send(T0, PARTNER_SECRET, Bytes::from(gift_body("ORD-2").to_string())).await;
    assert_eq!(
        (tampered.status, error_code(&tampered).as_str()),
        (StatusCode::UNAUTHORIZED, "bad_signature")
    );

    // Stale and future timestamps, however well signed.
    for (skew, expected) in [(-301, "stale_timestamp"), (301, "stale_timestamp")] {
        let reply = send(T0 + skew, PARTNER_SECRET, body.clone()).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNAUTHORIZED, expected),
            "{skew}"
        );
    }
    for skew in [-300, 300] {
        assert_eq!(
            send(T0 + skew, PARTNER_SECRET, body.clone()).await.status,
            StatusCode::CREATED,
            "{skew}"
        );
    }

    // Missing or malformed headers, unknown partner: all the same 401.
    let signature = request_signature(PARTNER_SECRET, T0, "POST", path, &body);
    let now = T0.to_string();
    let cases: [Vec<(&str, &str)>; 6] = [
        vec![],
        vec![("x-ebc-partner", PARTNER_ID), ("x-ebc-timestamp", now.as_str())],
        vec![("x-ebc-timestamp", now.as_str()), ("x-ebc-signature", signature.as_str())],
        vec![
            ("x-ebc-partner", "nobody"),
            ("x-ebc-timestamp", now.as_str()),
            ("x-ebc-signature", signature.as_str()),
        ],
        vec![
            ("x-ebc-partner", PARTNER_ID),
            ("x-ebc-timestamp", now.as_str()),
            ("x-ebc-signature", "v1=zz"),
        ],
        vec![
            ("x-ebc-partner", PARTNER_ID),
            ("x-ebc-timestamp", now.as_str()),
            ("x-ebc-signature", &signature[3..]),
        ],
    ];
    for headers in cases {
        let reply = h.call(Method::POST, path, &headers, Body::from(body.clone())).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNAUTHORIZED, "bad_signature"),
            "{headers:?}"
        );
    }
    let no_time = h
        .call(
            Method::POST,
            path,
            &[
                ("x-ebc-partner", PARTNER_ID),
                ("x-ebc-timestamp", "soon"),
                ("x-ebc-signature", signature.as_str()),
            ],
            Body::from(body.clone()),
        )
        .await;
    assert_eq!(error_code(&no_time), "stale_timestamp");

    // The signature binds the method, the path and the query string.
    let id = gift["id"].as_str().unwrap();
    let status_path = format!("/api/partner/v1/gifts/{id}");
    assert_eq!(
        h.partner(Method::GET, &status_path, Bytes::new(), &[]).await.status,
        StatusCode::OK
    );
    let for_get = request_signature(PARTNER_SECRET, T0, "GET", &status_path, b"");
    let headers = [
        ("x-ebc-partner", PARTNER_ID),
        ("x-ebc-timestamp", now.as_str()),
        ("x-ebc-signature", for_get.as_str()),
    ];
    for (method, target) in [
        (Method::POST, format!("{status_path}/cancel")),
        (Method::GET, format!("{status_path}?x=1")),
        (Method::GET, format!("{status_path}/card.pdf")),
    ] {
        let reply = h.call(method, &target, &headers, Body::empty()).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNAUTHORIZED, "bad_signature"),
            "{target}"
        );
    }
    // The creator API does not open partner gifts.
    let sneaky = h
        .call(
            Method::GET,
            &format!("/api/v1/gifts/{id}"),
            &[("authorization", "Bearer x")],
            Body::empty(),
        )
        .await;
    assert_eq!(sneaky.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn idempotency_keys_replay_the_first_answer_and_refuse_a_different_body() {
    let h = redirect_harness();
    let path = "/api/partner/v1/gifts";
    let key = [("idempotency-key", "ORD-2026-000123/1")];
    let gifts = || h.state.db.view(|db| db.gifts.len());

    let first = h.partner_json(Method::POST, path, &gift_body("ORD-A"), &key).await;
    assert_eq!(first.status, StatusCode::CREATED);
    assert!(!first.has_header("idempotent-replayed"));
    // Same key, same body, later (new timestamp, new signature): the original answer.
    h.clock.advance(120);
    let replay = h.partner_json(Method::POST, path, &gift_body("ORD-A"), &key).await;
    assert_eq!(replay.status, StatusCode::CREATED);
    assert_eq!(replay.json(), first.json());
    assert_eq!(replay.header("idempotent-replayed"), "true");
    assert_eq!(gifts(), 1);
    // Same key, another body.
    let conflict = h.partner_json(Method::POST, path, &gift_body("ORD-B"), &key).await;
    assert_eq!(
        (conflict.status, error_code(&conflict).as_str()),
        (StatusCode::CONFLICT, "idempotency_conflict")
    );
    assert_eq!(gifts(), 1);
    // Another key: another gift, another code.
    let other = h
        .partner_json(
            Method::POST,
            path,
            &gift_body("ORD-A"),
            &[("idempotency-key", "ORD-2026-000123/2")],
        )
        .await;
    assert_ne!(other.json()["code"], first.json()["code"]);
    assert_eq!(gifts(), 2);
    // The same key on another route is another operation, not a conflict.
    let id = first.json()["id"].as_str().unwrap().to_owned();
    let cancelled = h
        .partner(Method::POST, &format!("/api/partner/v1/gifts/{id}/cancel"), Bytes::new(), &key)
        .await;
    assert_eq!(cancelled.status, StatusCode::OK, "{}", cancelled.text());

    // Without a key, a byte-identical replay of a captured request (same timestamp, same
    // signature) is answered from memory too: it cannot act twice.
    let body = Bytes::from(gift_body("ORD-C").to_string());
    let once =
        h.partner_signed(Method::POST, path, body.clone(), &[], T0 + 100, PARTNER_SECRET).await;
    let twice =
        h.partner_signed(Method::POST, path, body.clone(), &[], T0 + 100, PARTNER_SECRET).await;
    assert_eq!((once.status, twice.status), (StatusCode::CREATED, StatusCode::CREATED));
    assert_eq!(once.json(), twice.json());
    assert_eq!(gifts(), 3);
    // A genuinely new request (the next second) is a new gift.
    let later = h.partner_signed(Method::POST, path, body, &[], T0 + 101, PARTNER_SECRET).await;
    assert_ne!(later.json()["id"], once.json()["id"]);

    // A failed request is not remembered under its key: signed anew, it can be retried
    // once the cause is fixed. Its *signature* is spent, though: the very same request
    // again (a replay) gets the first answer, whatever headers it adds.
    let retry_key = [("idempotency-key", "retry")];
    let mut bad = gift_body("ORD-D");
    bad["delivery"] = json!({ "type": "hosted_file", "upload_id": "u_missing" });
    let failed = h.partner_json(Method::POST, path, &bad, &retry_key).await;
    assert_eq!(
        (failed.status, error_code(&failed).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "unknown_upload")
    );
    let replayed = h.partner_json(Method::POST, path, &bad, &[("idempotency-key", "other")]).await;
    assert_eq!(replayed.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(replayed.header("idempotent-replayed"), "true");
    h.clock.advance(1);
    let again = h.partner_json(Method::POST, path, &bad, &retry_key).await;
    assert_eq!(again.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(!again.has_header("idempotent-replayed"));

    // Neither the database nor the remembered answers hold a clear code.
    let stored = h.state.db.view(|db| serde_json::to_string(db).unwrap());
    for reply in [&first, &other, &once] {
        let gift = reply.json();
        let (code, display) =
            (gift["code"].as_str().unwrap(), gift["code_display"].as_str().unwrap());
        assert!(!stored.contains(code) && !stored.contains(display), "a clear code was stored");
    }
    // Keys expire after a day.
    h.clock.advance(86_400);
    let expired = h
        .partner_json(
            Method::POST,
            path,
            &gift_body("ORD-A"),
            &[("idempotency-key", "ORD-2026-000123/2")],
        )
        .await;
    assert_ne!(expired.json()["id"], other.json()["id"]);
}

#[tokio::test]
async fn reseller_redirect_signs_the_hand_over_and_fulfilment_is_single_recipient() {
    let h = redirect_harness();
    let gift = h.partner_gift(redirect_delivery(), json!({})).await;
    let (id, code) =
        (gift["id"].as_str().unwrap().to_owned(), gift["code"].as_str().unwrap().to_owned());
    let status = || async {
        h.partner(Method::GET, &format!("/api/partner/v1/gifts/{id}"), Bytes::new(), &[])
            .await
            .json()
    };
    let doc = status().await;
    assert_eq!(
        (doc["status"].as_str(), doc["order_ref"].as_str(), doc["downloads"].as_u64()),
        (Some("created"), Some("ORD-2026-000123"), Some(0))
    );
    assert_eq!(doc["created_at"], T0);
    assert!(doc["redeemed_at"].is_null());

    // Every device class gets the same primary action: add to the store library.
    for agent in [UA_PHONE, UA_KOBO] {
        let page = h.get_as(&format!("/k/{code}"), agent).await;
        assert_page(
            &page,
            StatusCode::OK,
            "fr",
            &[
                "gift-title",
                "primary-action",
                "redeem",
                "other-ways",
                "device-chooser",
                "claim-count",
            ],
        );
        assert!(page.text_of("redeem").contains("Septante (démo)"));
        assert!(
            !page.has_id("download") && !page.has_id("pair-start") && !page.has_id("send-form")
        );
        // The browser may follow the redirect of this form to the store, and only there.
        assert!(
            page.header("content-security-policy")
                .contains("form-action 'self' https://store.example;")
        );
    }

    h.clock.advance(60);
    let location = h.post_form(&format!("/k/{code}/redeem"), &[]).await.location();
    let url = reqwest::Url::parse(&location).unwrap();
    assert_eq!(format!("{}://{}{}", url.scheme(), url.host_str().unwrap(), url.path()), STORE_URL);
    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();
    assert_eq!(query.len(), 4);
    assert_eq!(
        (query["ebc_gift"].as_str(), query["ebc_order"].as_str()),
        (id.as_str(), "ORD-2026-000123")
    );
    let ts: i64 = query["ebc_ts"].parse().unwrap();
    assert_eq!(ts, T0 + 60);
    assert_eq!(query["ebc_sig"], redirect_signature(PARTNER_SECRET, &id, "ORD-2026-000123", ts));
    assert!(!location.contains(&code), "the code never travels to the store");
    let doc = status().await;
    assert_eq!(
        (doc["status"].as_str(), doc["redeemed_at"].as_i64()),
        (Some("redeemed"), Some(T0 + 60))
    );

    // File routes do not exist for this gift.
    for action in ["download", "pair", "send"] {
        assert_eq!(
            h.post_form(&format!("/k/{code}/{action}"), &[]).await.status,
            StatusCode::CONFLICT,
            "{action}"
        );
    }

    // The store confirms: once, idempotently; a second account is a conflict.
    let fulfilled = format!("/api/partner/v1/gifts/{id}/fulfilled");
    let invalid =
        h.partner_json(Method::POST, &fulfilled, &json!({ "account_ref": "" }), &[]).await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    let confirmed =
        h.partner_json(Method::POST, &fulfilled, &json!({ "account_ref": "acct-42" }), &[]).await;
    assert_eq!(confirmed.status, StatusCode::OK, "{}", confirmed.text());
    assert_eq!(confirmed.json()["status"], "fulfilled");
    h.clock.advance(5);
    let again =
        h.partner_json(Method::POST, &fulfilled, &json!({ "account_ref": "acct-42" }), &[]).await;
    assert_eq!(
        (again.status, again.json()["status"].as_str()),
        (StatusCode::OK, Some("fulfilled"))
    );
    let other =
        h.partner_json(Method::POST, &fulfilled, &json!({ "account_ref": "acct-43" }), &[]).await;
    assert_eq!(
        (other.status, error_code(&other).as_str()),
        (StatusCode::CONFLICT, "already_fulfilled")
    );
    let stored = h.state.db.view(|db| serde_json::to_string(db).unwrap());
    assert!(!stored.contains("acct-42"), "the account reference is kept only as a hash");

    // The gift page now only says the book is in an account, and links to the store.
    let page = h.get(&format!("/k/{code}")).await;
    assert_page(
        &page,
        StatusCode::OK,
        "fr",
        &["gift-title", "primary-action", "store-link", "claim-count"],
    );
    assert!(!page.has_id("redeem"));
    assert!(page.text_of("primary-action").contains("déjà dans une bibliothèque"));
    assert_eq!(page.attr_of("store-link", "href").as_deref(), Some("https://store.example/"));
    assert_eq!(page.attr_of("store-link", "rel").as_deref(), Some("noreferrer"));
    let late = h.post_form(&format!("/k/{code}/redeem"), &[]).await;
    assert_eq!(late.status, StatusCode::CONFLICT);
    assert!(!late.has_header("location"));
}

#[tokio::test]
async fn redirect_targets_come_from_the_allow_list_only() {
    let h = Harness::with(|c| {
        c.partners = vec![partner(&["store.example", "shop.example:8443"], None)]
    });
    let create = |url: &'static str, kind: &'static str| {
        let h = &h;
        async move {
            let mut body = gift_body("ORD-X");
            body["delivery"] = json!({ "type": kind, "url": url, "format": "lcpl" });
            h.partner_json(Method::POST, "/api/partner/v1/gifts", &body, &[]).await
        }
    };
    for url in [
        "https://store.example/a?b=c",
        "http://store.example:8080/",
        "https://shop.example:8443/x",
        "https://STORE.example/",
    ] {
        assert_eq!(create(url, "reseller_redirect").await.status, StatusCode::CREATED, "{url}");
    }
    let refused = [
        "https://evil.example/cadeau",
        "https://store.example.evil.example/",
        "https://evil.example/?next=https://store.example/",
        "https://store.example@evil.example/",
        "https://user:pass@store.example/",
        "https://shop.example/x",
        "https://shop.example:9000/x",
        "//store.example/x",
        "javascript:alert(1)",
        "ftp://store.example/x",
        "/cadeau/activer",
        "",
    ];
    for url in refused {
        for kind in ["reseller_redirect", "reseller_licence"] {
            let reply = create(url, kind).await;
            assert_eq!(
                reply.status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "{kind} {url:?}: {}",
                reply.text()
            );
            assert_eq!(error_code(&reply), "redirect_host_not_allowed", "{url:?}");
        }
    }

    // Standalone creators cannot create store deliveries at all.
    let body = gift_body("ORD-Y").to_string();
    let reply = h.call(Method::POST, "/api/v1/gifts", &[], body).await;
    assert_eq!(
        (reply.status, error_code(&reply).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "unsupported_delivery")
    );

    // If the partner later drops the host from its allow-list, existing gifts stop
    // redirecting there: the target is checked again when the button is pressed.
    let gift = h
        .partner_gift(
            json!({ "type": "reseller_redirect", "url": "https://shop.example:8443/x" }),
            json!({}),
        )
        .await;
    let mut config = h.state.config.clone();
    config.partners = vec![partner(&["store.example"], None)];
    let clock = Arc::new(ManualClock::new(T0));
    let narrowed = AppState::builder(config).clock(clock).build().unwrap();
    narrowed.db.update(|db| *db = h.state.db.view(Clone::clone)).unwrap();
    let narrowed = Harness { app: app(narrowed.clone()), state: narrowed, clock: h.clock.clone() };
    let reply =
        narrowed.post_form(&format!("/k/{}/redeem", gift["code"].as_str().unwrap()), &[]).await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
    assert!(!reply.has_header("location"));
}

#[tokio::test]
async fn webhooks_are_signed_retried_with_backoff_and_eventually_given_up() {
    let hooks = Receiver::start(|_| (StatusCode::NO_CONTENT, Vec::new())).await;
    let webhook_url = hooks.url("/hooks/ebc?shop=1");
    let h = Harness::with(|c| {
        c.partners = vec![partner(&["nowhere.example"], Some(webhook_url))];
        c.webhook.base_delay_ms = 10;
        c.webhook.max_delay_ms = 40;
        c.webhook.max_attempts = 4;
    });
    // A partner-hosted file.
    let upload = h
        .partner(
            Method::POST,
            "/api/partner/v1/uploads",
            epub(),
            &[
                ("x-rights-basis", "licensed_distributor"),
                ("x-file-name", "bovary.epub"),
                ("idempotency-key", "up-1"),
            ],
        )
        .await;
    assert_eq!(upload.status, StatusCode::CREATED, "{}", upload.text());
    let upload_id = upload.json()["upload_id"].as_str().unwrap().to_owned();
    let gift =
        h.partner_gift(json!({ "type": "hosted_file", "upload_id": upload_id }), json!({})).await;
    let (id, code) =
        (gift["id"].as_str().unwrap().to_owned(), gift["code"].as_str().unwrap().to_owned());
    assert!(hooks.requests().is_empty(), "creation is not an event");

    // The receiver fails twice: each event is retried until it gets through.
    hooks.fail_first.store(2, std::sync::atomic::Ordering::SeqCst);
    h.clock.advance(30);
    let file = h.post_form(&format!("/k/{code}/download"), &[]).await.location();
    assert_eq!(h.get(&file).await.status, StatusCode::OK);
    let deliveries = hooks.wait_for(4).await;
    for _ in 0..200 {
        if h.state.pending_webhooks() == 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(h.state.pending_webhooks(), 0, "delivered events leave the outbox");
    assert_eq!(hooks.requests().len(), 4, "two events, two failed attempts in total");

    let mut events = std::collections::BTreeSet::new();
    for delivery in &deliveries {
        assert_eq!(delivery.method, "POST");
        assert_eq!(delivery.path_and_query, "/hooks/ebc?shop=1");
        assert_eq!(delivery.headers["x-ebc-partner"], PARTNER_ID);
        let timestamp: i64 = delivery.headers["x-ebc-timestamp"].to_str().unwrap().parse().unwrap();
        assert_eq!(timestamp, T0 + 30);
        let signature = delivery.headers["x-ebc-signature"].to_str().unwrap();
        assert!(
            verify_request_signature(
                PARTNER_SECRET,
                timestamp,
                "POST",
                "/hooks/ebc?shop=1",
                &delivery.body,
                signature
            ),
            "webhook signature (§4.1)"
        );
        let event: serde_json::Value = serde_json::from_slice(&delivery.body).unwrap();
        assert_eq!(
            (event["gift_id"].as_str(), event["order_ref"].as_str(), event["at"].as_i64()),
            (Some(id.as_str()), Some("ORD-2026-000123"), Some(T0 + 30))
        );
        assert!(event["id"].as_str().unwrap().starts_with("evt_"));
        assert!(
            !delivery.body.windows(code.len()).any(|w| w == code.as_bytes()),
            "events never carry the code"
        );
        events.insert(event["event"].as_str().unwrap().to_owned());
    }
    assert_eq!(events.into_iter().collect::<Vec<_>>(), ["gift.downloaded", "gift.redeemed"]);

    // A second download is "downloaded" only; cancelling is "cancelled", once.
    let before = hooks.requests().len();
    h.post_form(&format!("/k/{code}/download"), &[]).await.location();
    let cancel = format!("/api/partner/v1/gifts/{id}/cancel");
    assert_eq!(
        h.partner(Method::POST, &cancel, Bytes::new(), &[]).await.json()["status"],
        "cancelled"
    );
    h.clock.advance(1);
    assert_eq!(
        h.partner(Method::POST, &cancel, Bytes::new(), &[]).await.json()["status"],
        "cancelled"
    );
    let deliveries = hooks.wait_for(before + 2).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let names: Vec<String> = hooks.requests()[before..]
        .iter()
        .map(|d| {
            serde_json::from_slice::<serde_json::Value>(&d.body).unwrap()["event"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(deliveries.len(), before + 2);
    assert!(
        names.contains(&"gift.downloaded".to_owned())
            && names.contains(&"gift.cancelled".to_owned()),
        "{names:?}"
    );
    assert_eq!(names.len(), 2, "{names:?}");
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);

    // A receiver that never recovers: four attempts, then the event is dropped.
    let gift = h
        .partner_gift(
            json!({ "type": "reseller_redirect", "url": "https://nowhere.example/" }),
            json!({}),
        )
        .await;
    let before = hooks.requests().len();
    hooks.fail_first.store(1_000, std::sync::atomic::Ordering::SeqCst);
    let cancel = format!("/api/partner/v1/gifts/{}/cancel", gift["id"].as_str().unwrap());
    assert_eq!(h.partner(Method::POST, &cancel, Bytes::new(), &[]).await.status, StatusCode::OK);
    hooks.wait_for(before + 4).await;
    for _ in 0..200 {
        if h.state.pending_webhooks() == 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!((hooks.requests().len(), h.state.pending_webhooks()), (before + 4, 0));
}

/// A renderer that records what it was asked and returns a recognisable PDF.
#[derive(Default)]
struct RecordingRenderer(Mutex<Vec<CardRequest>>);

impl CardRenderer for RecordingRenderer {
    fn render(&self, request: &CardRequest) -> Result<Vec<u8>, CardRenderError> {
        self.0.lock().unwrap().push(request.clone());
        if request.cover.is_none() {
            return Err(CardRenderError::Unprocessable("a cover is needed".to_owned()));
        }
        Ok(b"%PDF-1.6 fake card".to_vec())
    }
}

#[tokio::test]
async fn uploads_cover_and_card_rendering_through_the_renderer_seam() {
    let renderer = Arc::new(RecordingRenderer::default());
    let mut config = crate::common::test_config();
    config.partners = vec![partner(&["store.example"], None), {
        let mut other = partner(&[], None);
        other.id = "autre".to_owned();
        other
    }];
    let clock = Arc::new(ManualClock::new(T0));
    let state = AppState::builder(config)
        .clock(clock.clone())
        .card_renderer(renderer.clone())
        .build()
        .unwrap();
    let h = Harness { app: app(state.clone()), state, clock };

    // Uploads follow the rules of the creator API.
    let uploads = "/api/partner/v1/uploads";
    let no_basis = h.partner(Method::POST, uploads, epub(), &[]).await;
    assert_eq!(
        (no_basis.status, error_code(&no_basis).as_str()),
        (StatusCode::BAD_REQUEST, "missing_rights_basis")
    );
    let basis = [("x-rights-basis", "licensed_distributor")];
    let drm = h
        .partner(Method::POST, uploads, epub_with(&[("META-INF/rights.xml", b"<rights/>")]), &basis)
        .await;
    assert_eq!(
        (drm.status, error_code(&drm).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "drm_protected")
    );
    let junk = h.partner(Method::POST, uploads, &b"MZ\x90\x00 not a book"[..], &basis).await;
    assert_eq!(junk.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    // The first attempt's signature is spent (X-Rights-Basis is not covered by it): the
    // corrected upload is signed anew, a second later.
    let replay = h.partner(Method::POST, uploads, epub(), &basis).await;
    assert_eq!(
        (replay.status, replay.header("idempotent-replayed")),
        (StatusCode::BAD_REQUEST, "true")
    );
    h.clock.advance(1);
    let ok = h.partner(Method::POST, uploads, epub(), &basis).await;
    assert_eq!(ok.status, StatusCode::CREATED);
    assert_eq!(ok.json()["media_type"], "application/epub+zip");
    // A creator cannot use a partner's upload, nor another partner.
    let upload_id = ok.json()["upload_id"].as_str().unwrap().to_owned();
    let stolen = json!({ "book": { "title": "x" }, "delivery": { "type": "hosted_file", "upload_id": upload_id } }).to_string();
    let reply = h.call(Method::POST, "/api/v1/gifts", &[], stolen).await;
    assert_eq!(error_code(&reply), "unknown_upload");

    let gift =
        h.partner_gift(json!({ "type": "reseller_redirect", "url": STORE_URL }), json!({})).await;
    let (id, code) = (gift["id"].as_str().unwrap(), gift["code"].as_str().unwrap());
    let card = format!("/api/partner/v1/gifts/{id}/card.pdf?format=poche&profile=home-a4");

    // Parameters are validated before anything is rendered.
    for bad in ["format=a4&profile=pro", "format=poche&profile=home", "format=../x"] {
        let reply = h
            .partner(
                Method::GET,
                &format!("/api/partner/v1/gifts/{id}/card.pdf?{bad}"),
                Bytes::new(),
                &[],
            )
            .await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::BAD_REQUEST, "invalid_parameter"),
            "{bad}"
        );
    }
    // Without a cover this renderer cannot compose the card.
    let no_cover = h.partner(Method::GET, &card, Bytes::new(), &[]).await;
    assert_eq!(
        (no_cover.status, error_code(&no_cover).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "card_unprocessable")
    );

    // The cover: an image the partner uploads (never a URL the server would fetch).
    let cover_path = format!("/api/partner/v1/gifts/{id}/cover");
    let png = [&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a][..], &[0u8; 64][..]].concat();
    let not_image = h
        .partner(
            Method::PUT,
            &cover_path,
            "<svg onload=alert(1)/>",
            &[("content-type", "image/png")],
        )
        .await;
    assert_eq!(not_image.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let put = h
        .partner(
            Method::PUT,
            &cover_path,
            png.clone(),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    assert_eq!(
        (put.json()["media_type"].as_str(), put.json()["size_bytes"].as_u64()),
        (Some("image/png"), Some(72))
    );

    let pdf = h.partner(Method::GET, &card, Bytes::new(), &[]).await;
    assert_eq!(pdf.status, StatusCode::OK, "{}", pdf.text());
    assert_eq!(pdf.header("content-type"), "application/pdf");
    assert_eq!(
        pdf.header("content-disposition"),
        "attachment; filename=\"carte-Madame-Bovary.pdf\""
    );
    assert_eq!(pdf.body.as_ref(), b"%PDF-1.6 fake card");
    let request = renderer.0.lock().unwrap().last().cloned().unwrap();
    assert_eq!(request.code.compact(), code, "the card carries the code given at creation");
    assert_eq!(request.claim_url, format!("https://ebk.example/k/{code}"));
    assert_eq!(request.claim_url_qr, format!("HTTPS://EBK.EXAMPLE/K/{code}"));
    assert_eq!((request.format.as_str(), request.profile.as_str()), ("poche", "home-a4"));
    assert_eq!(
        (request.delivery_type, request.store_name.as_deref()),
        ("reseller_redirect", Some("Septante (démo)"))
    );
    assert_eq!(request.cover, Some((png.clone(), "image/png".to_owned())));
    assert_eq!(request.public_url, "https://ebk.example");
    // Nothing was declared about the cover: the renderer is told so (and must not print it).
    assert_eq!(request.cover_rights, CoverRights::None);
    assert_eq!(put.json()["cover_rights"], "none");

    // The rights basis is declared on the upload (in the signed path)…
    let declared = h
        .partner(Method::PUT, &format!("{cover_path}?cover_rights=print_licence"), png.clone(), &[])
        .await;
    assert_eq!(declared.status, StatusCode::OK, "{}", declared.text());
    assert_eq!(declared.json()["cover_rights"], "print_licence");
    h.partner(Method::GET, &card, Bytes::new(), &[]).await;
    let request = renderer.0.lock().unwrap().last().cloned().unwrap();
    assert_eq!(request.cover_rights, CoverRights::PrintLicence);
    // …it stays when the image is replaced without a new declaration…
    h.partner(Method::PUT, &cover_path, png.clone(), &[]).await;
    h.partner(Method::GET, &card, Bytes::new(), &[]).await;
    let request = renderer.0.lock().unwrap().last().cloned().unwrap();
    assert_eq!(request.cover_rights, CoverRights::PrintLicence);
    // …can be withdrawn, and anything else is refused without touching the gift.
    for bad in ["user_private_copy", "PRINT_LICENCE", "yes"] {
        let reply = h
            .partner(Method::PUT, &format!("{cover_path}?cover_rights={bad}"), png.clone(), &[])
            .await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNPROCESSABLE_ENTITY, "invalid_cover_rights"),
            "{bad}"
        );
    }
    let withdrawn =
        h.partner(Method::PUT, &format!("{cover_path}?cover_rights=none"), png.clone(), &[]).await;
    assert_eq!(withdrawn.json()["cover_rights"], "none");
    // …or declared when the gift is created.
    let at_creation = h
        .partner_gift(
            json!({ "type": "reseller_redirect", "url": STORE_URL }),
            json!({ "order_ref": "ORD-RIGHTS", "cover_rights": "public_domain" }),
        )
        .await;
    let other_id = at_creation["id"].as_str().unwrap();
    h.partner(Method::PUT, &format!("/api/partner/v1/gifts/{other_id}/cover"), png, &[]).await;
    let other_card = format!("/api/partner/v1/gifts/{other_id}/card.pdf");
    h.partner(Method::GET, &other_card, Bytes::new(), &[]).await;
    let request = renderer.0.lock().unwrap().last().cloned().unwrap();
    assert_eq!(request.cover_rights, CoverRights::PublicDomain);
    let mut bad_body = gift_body("ORD-BAD-RIGHTS");
    bad_body["cover_rights"] = json!("whatever");
    let reply = h.partner_json(Method::POST, "/api/partner/v1/gifts", &bad_body, &[]).await;
    assert_eq!(
        (reply.status, error_code(&reply).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "invalid_cover_rights")
    );
    h.partner(Method::GET, &card, Bytes::new(), &[]).await;
    let request = renderer.0.lock().unwrap().last().cloned().unwrap();
    assert_eq!(
        (request.book.title.as_str(), request.giver_name.as_deref(), request.locale.code()),
        ("Madame Bovary", Some("Camille"), "fr")
    );

    // After a reissue the card carries the new code.
    let reissued = h
        .partner(Method::POST, &format!("/api/partner/v1/gifts/{id}/reissue"), Bytes::new(), &[])
        .await;
    assert_eq!(reissued.status, StatusCode::OK, "{}", reissued.text());
    let new_code = reissued.json()["code"].as_str().unwrap().to_owned();
    assert_ne!(new_code, code);
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);
    assert_eq!(h.get(&format!("/k/{new_code}")).await.status, StatusCode::OK);
    // A replay of the very same reissue request returns the same code instead of voiding it.
    let replay = h
        .partner(Method::POST, &format!("/api/partner/v1/gifts/{id}/reissue"), Bytes::new(), &[])
        .await;
    assert_eq!(replay.json()["code"], new_code.as_str());
    assert_eq!(h.get(&format!("/k/{new_code}")).await.status, StatusCode::OK);
    assert_eq!(h.partner(Method::GET, &card, Bytes::new(), &[]).await.status, StatusCode::OK);
    assert_eq!(renderer.0.lock().unwrap().last().unwrap().code.compact(), new_code);

    // Another partner sees none of this.
    for (method, path) in [
        (Method::GET, format!("/api/partner/v1/gifts/{id}")),
        (Method::GET, card.clone()),
        (Method::PUT, cover_path.clone()),
        (Method::POST, format!("/api/partner/v1/gifts/{id}/cancel")),
        (Method::POST, format!("/api/partner/v1/gifts/{id}/reissue")),
        (Method::POST, format!("/api/partner/v1/gifts/{id}/fulfilled")),
    ] {
        let body = if path.ends_with("fulfilled") {
            Bytes::from_static(b"{\"account_ref\":\"a\"}")
        } else {
            Bytes::new()
        };
        let signature = request_signature(PARTNER_SECRET, T0, method.as_str(), &path, &body);
        let now = T0.to_string();
        let headers = [
            ("x-ebc-partner", "autre"),
            ("x-ebc-timestamp", now.as_str()),
            ("x-ebc-signature", signature.as_str()),
        ];
        let reply = h.call(method, &path, &headers, Body::from(body)).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}: {}", reply.text());
    }
    assert_eq!(h.get(&format!("/k/{new_code}")).await.status, StatusCode::OK);
}

/// A valid PNG: a 200 × 320 px portrait "cover" in two flat colours.
#[cfg(feature = "card")]
fn cover_png() -> Vec<u8> {
    let (width, height) = (200u32, 320u32);
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for _ in 0..width {
            pixels.extend_from_slice(if y < 110 { &[178, 34, 52] } else { &[244, 236, 220] });
        }
    }
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }
    out
}

/// With the cargo feature `card` (the default) `card.pdf` is the real card, composed by
/// `ebc-card`: typographic unless the partner declared a right to print its cover.
#[cfg(feature = "card")]
#[tokio::test]
async fn card_pdf_is_composed_by_ebc_card_and_prints_a_cover_only_with_declared_rights() {
    let h = redirect_harness();
    let gift = h.partner_gift(redirect_delivery(), json!({})).await;
    let id = gift["id"].as_str().unwrap();
    let card = |query: &str| format!("/api/partner/v1/gifts/{id}/card.pdf{query}");

    // Every documented format for the printer; the defaults are poche + pro.
    let default = h.partner(Method::GET, &card(""), Bytes::new(), &[]).await;
    assert_eq!(default.status, StatusCode::OK, "{}", default.text());
    assert_eq!(default.header("content-type"), "application/pdf");
    assert_eq!(
        default.header("content-disposition"),
        "attachment; filename=\"carte-Madame-Bovary.pdf\""
    );
    assert!(default.body.starts_with(b"%PDF-"));
    let poche_pro =
        h.partner(Method::GET, &card("?format=poche&profile=pro"), Bytes::new(), &[]).await;
    assert_eq!(poche_pro.body, default.body, "deterministic, and the defaults are poche + pro");
    for query in [
        "?format=a6&profile=pro",
        "?format=digest&profile=pro",
        "?format=a5&profile=pro",
        "?format=poche&profile=home-a4",
        "?format=a6&profile=home-letter",
    ] {
        let reply = h.partner(Method::GET, &card(query), Bytes::new(), &[]).await;
        assert_eq!(reply.status, StatusCode::OK, "{query}: {}", reply.text());
        assert!(reply.body.starts_with(b"%PDF-") && reply.body != default.body, "{query}");
    }
    // An A5 card does not fit an A4 sheet: the request is the problem, not the server.
    let too_big =
        h.partner(Method::GET, &card("?format=a5&profile=home-a4"), Bytes::new(), &[]).await;
    assert_eq!(
        (too_big.status, error_code(&too_big).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "card_unprocessable")
    );

    // A cover without a declared basis is kept but not printed: the same typographic card.
    let cover_path = format!("/api/partner/v1/gifts/{id}/cover");
    let put = h.partner(Method::PUT, &cover_path, cover_png(), &[]).await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    let undeclared = h.partner(Method::GET, &card(""), Bytes::new(), &[]).await;
    assert_eq!(undeclared.body, default.body, "an undeclared cover never enters the print file");

    // Declared as licensed for print: the cover is on the card (the file embeds an image).
    let put = h
        .partner(Method::PUT, &format!("{cover_path}?cover_rights=print_licence"), cover_png(), &[])
        .await;
    assert_eq!(put.json()["cover_rights"], "print_licence");
    let with_cover = h.partner(Method::GET, &card(""), Bytes::new(), &[]).await;
    assert_eq!(with_cover.status, StatusCode::OK, "{}", with_cover.text());
    assert_ne!(with_cover.body, default.body);
    let has_image =
        |pdf: &[u8]| pdf.windows(14).any(|w| w == b"/Subtype /Image" || w == b"/Subtype/Image");
    assert!(has_image(&with_cover.body) && !has_image(&default.body));

    // The rights gate: a promotion-only licence prints (reduced) at home, never at a printer.
    h.partner(Method::PUT, &format!("{cover_path}?cover_rights=promo_all_media"), cover_png(), &[])
        .await;
    let refused = h.partner(Method::GET, &card("?profile=pro"), Bytes::new(), &[]).await;
    assert_eq!(
        (refused.status, error_code(&refused).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "cover_rights_refused"),
        "{}",
        refused.text()
    );
    let at_home = h.partner(Method::GET, &card("?profile=home-a4"), Bytes::new(), &[]).await;
    assert_eq!(at_home.status, StatusCode::OK, "{}", at_home.text());

    // A declared cover that is not a decodable image is the partner's problem (422).
    let truncated =
        [&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a][..], &[0u8; 64][..]].concat();
    h.partner(Method::PUT, &format!("{cover_path}?cover_rights=print_licence"), truncated, &[])
        .await;
    let broken = h.partner(Method::GET, &card(""), Bytes::new(), &[]).await;
    assert_eq!(
        (broken.status, error_code(&broken).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "card_unprocessable")
    );
}

#[tokio::test]
async fn card_pdf_answers_501_without_a_renderer() {
    let mut config = crate::common::test_config();
    config.partners = vec![partner(&["store.example"], None)];
    let clock = Arc::new(ManualClock::new(T0));
    let state = AppState::builder(config)
        .clock(clock.clone())
        .card_renderer(Arc::new(NoCardRenderer))
        .build()
        .unwrap();
    let h = Harness { app: app(state.clone()), state, clock };
    let gift = h.partner_gift(redirect_delivery(), json!({})).await;
    let path = format!(
        "/api/partner/v1/gifts/{}/card.pdf?format=a6&profile=pro",
        gift["id"].as_str().unwrap()
    );
    let reply = h.partner(Method::GET, &path, Bytes::new(), &[]).await;
    assert_eq!(
        (reply.status, error_code(&reply).as_str()),
        (StatusCode::NOT_IMPLEMENTED, "not_implemented")
    );
    let missing =
        h.partner(Method::GET, "/api/partner/v1/gifts/g_nope/card.pdf", Bytes::new(), &[]).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn failed_authentications_are_budgeted_per_network_before_anything_is_read() {
    let h = Harness::with(|c| c.rate_partner_fail_per_10min = 3);
    let ts = T0.to_string();
    let wrong_mac = format!("v1={}", "ab".repeat(32));
    let forged = |client: &'static str| {
        let (ts, wrong_mac) = (ts.clone(), wrong_mac.clone());
        let h = &h;
        async move {
            let headers = [
                ("x-ebc-partner", PARTNER_ID),
                ("x-ebc-timestamp", ts.as_str()),
                ("x-ebc-signature", wrong_mac.as_str()),
                ("x-rights-basis", "licensed_distributor"),
                ("x-forwarded-for", client),
            ];
            h.call(Method::POST, "/api/partner/v1/uploads", &headers, epub()).await
        }
    };
    // A well-formed but wrong signature: the body is hashed, the MAC compared, 401.
    for _ in 0..3 {
        let reply = forged("198.51.100.7").await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNAUTHORIZED, "bad_signature")
        );
    }
    // The network's budget is spent: 429 before anything is looked at.
    let blocked = forged("198.51.100.8").await;
    assert_eq!(
        (blocked.status, error_code(&blocked).as_str()),
        (StatusCode::TOO_MANY_REQUESTS, "rate_limited")
    );
    assert!(blocked.has_header("retry-after"));
    // Other networks — the genuine partner — are not affected.
    assert_eq!(forged("203.0.113.9").await.status, StatusCode::UNAUTHORIZED);
    let genuine = h
        .partner(
            Method::GET,
            "/api/partner/v1/gifts/g_00000000000000000000000000000000",
            Bytes::new(),
            &[],
        )
        .await;
    assert_eq!(genuine.status, StatusCode::NOT_FOUND);
    // And the window slides.
    h.clock.advance(600);
    let later = h
        .partner_signed(
            Method::GET,
            "/api/partner/v1/gifts/g_1",
            Bytes::new(),
            &[("x-forwarded-for", "198.51.100.7")],
            T0 + 600,
            PARTNER_SECRET,
        )
        .await;
    assert_eq!(later.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_signature_is_a_nonce_for_every_state_changing_request() {
    let h = redirect_harness();
    let gift = h.partner_gift(redirect_delivery(), json!({})).await;
    let id = gift["id"].as_str().unwrap().to_owned();
    let cancel = format!("/api/partner/v1/gifts/{id}/cancel");
    let signature = request_signature(PARTNER_SECRET, T0, "POST", &cancel, b"");
    let ts = T0.to_string();
    let send = |key: Option<&'static str>, signature: String| {
        let (ts, cancel) = (ts.clone(), cancel.clone());
        let h = &h;
        async move {
            let mut headers = vec![
                ("x-ebc-partner", PARTNER_ID),
                ("x-ebc-timestamp", ts.as_str()),
                ("x-ebc-signature", signature.as_str()),
            ];
            if let Some(key) = key {
                headers.push(("idempotency-key", key));
            }
            h.call(Method::POST, &cancel, &headers, Body::empty()).await
        }
    };
    let first = send(Some("k-1"), signature.clone()).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert!(!first.has_header("idempotent-replayed"));
    // Whatever unsigned header or hex case the copy carries, it is the same signature.
    for (key, signature) in [
        (None, signature.clone()),
        (Some("k-2"), signature.clone()),
        (Some("k-1"), format!("v1={}", signature[3..].to_ascii_uppercase())),
    ] {
        let replay = send(key, signature).await;
        assert_eq!(replay.status, StatusCode::OK);
        assert_eq!(replay.header("idempotent-replayed"), "true");
        assert_eq!(replay.json(), first.json());
    }
    // Nonces are kept only while the timestamp could still pass the window check.
    let nonces = |h: &Harness| {
        h.state.db.view(|db| db.idempotency.keys().filter(|k| k.contains("\nsig:")).count())
    };
    assert_eq!(nonces(&h), 2, "the creation and the cancellation");
    h.clock.advance(601);
    let _ = h.partner_gift(redirect_delivery(), json!({ "order_ref": "ORD-2" })).await;
    let _ = h.hosted_gift(json!({})).await; // a write that purges
    assert_eq!(nonces(&h), 1, "only the newest signature is still remembered");
}

#[tokio::test]
async fn deliberate_repeats_within_one_second_are_told_apart_by_a_signed_nonce_parameter() {
    let h = redirect_harness();
    let gift = h.partner_gift(redirect_delivery(), json!({})).await;
    let id = gift["id"].as_str().unwrap().to_owned();
    // Two reissues in the same second: the same signature unless the signed query differs.
    let first = h
        .partner(
            Method::POST,
            &format!("/api/partner/v1/gifts/{id}/reissue?nonce=1"),
            Bytes::new(),
            &[],
        )
        .await;
    let second = h
        .partner(
            Method::POST,
            &format!("/api/partner/v1/gifts/{id}/reissue?nonce=2"),
            Bytes::new(),
            &[],
        )
        .await;
    assert_eq!((first.status, second.status), (StatusCode::OK, StatusCode::OK));
    assert!(!second.has_header("idempotent-replayed"));
    let (old, new) = (
        first.json()["code"].as_str().unwrap().to_owned(),
        second.json()["code"].as_str().unwrap().to_owned(),
    );
    assert_ne!(old, new);
    assert_eq!(h.get(&format!("/k/{old}")).await.status, StatusCode::GONE);
    assert_eq!(h.get(&format!("/k/{new}")).await.status, StatusCode::OK);
}
