//! The recipient's side of a `reseller_redirect` gift (signed redirect §4.3, fulfilment
//! confirmation §4.4), and the webhooks the stores receive from eBook Cadeau.

#![allow(clippy::unwrap_used)]

mod common;

use axum::http::StatusCode;
use common::{
    GRANDE_SECRET, Store, T0, World, gift_home, plain_client, post_webhook, sign_request,
};
use ebc_demo_stores::StoreKind;
use serde_json::{Value, json};

/// A Grande Enseigne store with one gift of "Candide" bought by camille@example.org.
/// Returns the store, the buyer's order path and the gift id.
async fn store_with_gift(world: &World) -> (Store, String, String) {
    let store = world.store(StoreKind::GrandeEnseigne).await;
    let order = store
        .browser()
        .buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "Bonne lecture"))
        .await;
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("created"));
    (store, order.path.clone(), "g_0001".to_owned())
}

#[tokio::test]
async fn recipient_creates_an_account_and_finds_the_book_on_the_e_reader() {
    let world = World::new().await;
    let (store, order_path, gift_id) = store_with_gift(&world).await;
    let buyer = store.browser();
    buyer.login("camille@example.org", "motdepasse").await;

    // The recipient arrives from eBook Cadeau with the signed link, with no session.
    let recipient = store.browser();
    let landing = recipient.get(&world.ebc.redirect_url(&gift_id)).await;
    assert_eq!(landing.status, StatusCode::OK);
    assert_eq!(landing.hops, [(StatusCode::SEE_OTHER, "/cadeau/activer".to_owned())]);
    assert_eq!(landing.path, "/cadeau/activer", "the signed parameters leave the address bar");
    assert_eq!(landing.text("activation-book").as_deref(), Some("Candide"));
    assert_eq!(landing.text("activation-giver").as_deref(), Some("Camille"));
    assert!(landing.has("login-form") && landing.has("register-form") && !landing.has("activate"));
    assert_eq!(landing.count("input[name=\"next\"][value=\"/cadeau/activer\"]"), 2);

    // Creating an account brings them back to a confirmation step.
    let confirm = recipient
        .post(
            "/compte/inscription",
            &[
                ("email", "sacha@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;
    assert_eq!(confirm.path, "/cadeau/activer");
    assert_eq!(confirm.text("activation-account").as_deref(), Some("sacha@example.org"));
    assert!(confirm.has("activate") && !confirm.has("activation-already-owned"));
    assert!(
        store.state.library("sacha@example.org").is_empty(),
        "nothing happens before the click"
    );
    assert!(world.ebc.requests_to("POST", "/fulfilled").is_empty());

    let done = recipient.post("/cadeau/activer", &[]).await;
    assert_eq!(done.status, StatusCode::OK);
    assert_eq!(done.path, "/cadeau/merci");
    assert_eq!(done.text("activation-book").as_deref(), Some("Candide"));
    assert!(done.text("activation-sync").unwrap().contains("liseuse"));
    assert!(!done.has("activation-download"), "no file in the browser in this store");

    // The library lists the book, as a gift.
    let library = recipient.get("/bibliotheque").await;
    assert!(library.has("owned-candide"));
    assert!(library.text("library").unwrap().contains("Offert par Camille"));

    // eBook Cadeau was told, with an opaque account reference.
    let confirmations = world.ebc.requests_to("POST", "/g_0001/fulfilled");
    assert_eq!(confirmations.len(), 1);
    let account_ref = confirmations[0].json()["account_ref"].as_str().unwrap().to_owned();
    assert!(account_ref.starts_with("acct_") && !account_ref.contains("sacha"));
    assert_eq!(world.ebc.gift(&gift_id).fulfilled_by.as_deref(), Some(account_ref.as_str()));

    // The buyer sees the gift as fulfilled and can no longer cancel it.
    let order = buyer.get(&order_path).await;
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("fulfilled"));
    assert!(!order.has("cancel-gift"));
    let refused = buyer.post(&format!("{order_path}/annuler"), &[]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);

    // The recipient's e-reader syncs the book through the device API.
    let http = plain_client();
    let linked: Value = http
        .post(format!("{}/api/device/link", store.base))
        .json(&json!({"email": "sacha@example.org", "password": "motdepasse"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = linked["device_token"].as_str().unwrap();
    let books: Value = http
        .get(format!("{}/api/device/library", store.base))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(books[0]["id"], "candide");
    let epub = http
        .get(format!("{}{}", store.base, books[0]["download_url"].as_str().unwrap()))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    common::open_epub(&epub.bytes().await.unwrap());

    // The thank-you page is not a way to activate anything again.
    assert_eq!(recipient.post("/cadeau/activer", &[]).await.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_existing_customer_signs_in_and_is_warned_about_a_title_they_own() {
    let world = World::new().await;
    let (store, _, gift_id) = store_with_gift(&world).await;
    // Sacha already bought Candide here.
    store.browser().buy("sacha@example.org", &["candide"], &[]).await;

    let recipient = store.browser();
    recipient.get(&world.ebc.redirect_url(&gift_id)).await;
    let confirm = recipient
        .post(
            "/compte/connexion",
            &[
                ("email", "sacha@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;
    assert!(confirm.has("activate") && confirm.has("activation-already-owned"));
    let done = recipient.post("/cadeau/activer", &[]).await;
    assert!(done.text("activation-done").unwrap().contains("déjà"));
    assert_eq!(store.state.library("sacha@example.org").len(), 1);
    assert_eq!(world.ebc.gift(&gift_id).status, "fulfilled");

    // Someone already signed in when following the link goes straight to the confirmation.
    let order =
        store.browser().buy("lou@example.org", &["le-horla"], &gift_home("Lou", "Sacha", "")).await;
    assert!(order.has("gift-status"));
    let confirm = recipient.get(&world.ebc.redirect_url("g_0002")).await;
    assert_eq!(confirm.text("activation-book").as_deref(), Some("Le Horla"));
    assert_eq!(confirm.text("activation-giver").as_deref(), Some("Lou"));
    assert!(confirm.has("activate"));
    // "Not you?" signs out and comes back to the same gift.
    let switched = recipient.post("/compte/deconnexion", &[("next", "/cadeau/activer")]).await;
    assert!(switched.has("login-form") && switched.has("activation-book"));
}

#[tokio::test]
async fn the_redirect_signature_is_verified() {
    let world = World::new().await;
    let (store, _, gift_id) = store_with_gift(&world).await;
    let good = world.ebc.redirect_url(&gift_id);
    let problem = |page: &common::Page| page.attr("activation-error", "data-code");
    let visit = |url: String| {
        let browser = store.browser();
        async move { browser.get(&url).await }
    };

    // Any change to a signed value breaks the signature.
    let tampered = [
        good.replace("ebc_gift=g_0001", "ebc_gift=g_0002"),
        good.replace("ebc_order=GE-2026-000001", "ebc_order=GE-2026-000002"),
        good.replace(&format!("ebc_ts={T0}"), &format!("ebc_ts={}", T0 + 1)),
        format!("{}0", &good[..good.len() - 1]),
        format!("{}f", &good[..good.len() - 1]),
    ];
    for url in tampered.into_iter().filter(|url| *url != good) {
        let page = visit(url.clone()).await;
        assert_eq!(page.status, StatusCode::FORBIDDEN, "{url}");
        assert_eq!(problem(&page).as_deref(), Some("bad_signature"), "{url}");
        assert!(!page.has("login-form") && !page.has("activate"));
    }

    // A signature made with another partner's secret is worthless here.
    let foreign = {
        let sig = ebc_demo_stores::crypto::redirect_signature(
            common::SEPTANTE_SECRET,
            &gift_id,
            "GE-2026-000001",
            T0,
        );
        format!(
            "{}/cadeau/activer?ebc_gift={gift_id}&ebc_order=GE-2026-000001&ebc_ts={T0}&ebc_sig={sig}",
            store.base
        )
    };
    assert_eq!(visit(foreign).await.status, StatusCode::FORBIDDEN);

    // Missing or malformed parameters.
    let base = format!("{}/cadeau/activer", store.base);
    let incomplete = [
        format!("{base}?ebc_gift={gift_id}"),
        good.replace("&ebc_sig=", "&sig="),
        good.replace(&format!("ebc_ts={T0}"), "ebc_ts=hier"),
        good.replace(&format!("ebc_ts={T0}"), &format!("ebc_ts=+{T0}")),
        format!("{}zz", &good[..good.len() - 2]),
        good.replace("ebc_gift=g_0001", "ebc_gift="),
    ];
    for url in incomplete {
        let page = visit(url.clone()).await;
        assert_eq!(page.status, StatusCode::BAD_REQUEST, "{url}");
        assert_eq!(problem(&page).as_deref(), Some("malformed"), "{url}");
    }
    let bare = visit(base).await;
    assert_eq!(bare.status, StatusCode::BAD_REQUEST);
    assert_eq!(problem(&bare).as_deref(), Some("no_activation"));

    // A correctly signed link for a gift this store never sold.
    let sig =
        ebc_demo_stores::crypto::redirect_signature(GRANDE_SECRET, "g_9999", "GE-2026-000001", T0);
    let unknown = format!(
        "{}/cadeau/activer?ebc_gift=g_9999&ebc_order=GE-2026-000001&ebc_ts={T0}&ebc_sig={sig}",
        store.base
    );
    let page = visit(unknown).await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    assert_eq!(problem(&page).as_deref(), Some("unknown_gift"));

    // Nothing above created a pending activation or touched the gift.
    assert_eq!(world.ebc.gift(&gift_id).status, "created");
    assert_eq!(store.snapshot()["gifts"][0]["activated_by"], Value::Null);
    // And the genuine link still works.
    assert!(visit(good).await.has("register-form"));
}

#[tokio::test]
async fn stale_links_are_refused_and_fresh_ones_expire_in_the_session() {
    let world = World::new().await;
    let (store, _, gift_id) = store_with_gift(&world).await;
    let link = world.ebc.redirect_url(&gift_id);

    world.advance(900);
    let recipient = store.browser();
    let on_time = recipient.get(&link).await;
    assert!(on_time.has("register-form"), "exactly 900 s old is still accepted");

    world.advance(1);
    let late = store.browser().get(&link).await;
    assert_eq!(late.status, StatusCode::GONE);
    assert_eq!(late.attr("activation-error", "data-code").as_deref(), Some("stale_link"));

    // A link stamped in the future beyond the tolerated clock drift is refused too.
    let future = world.ebc.redirect_url_at(&gift_id, T0 + 901 + 301);
    assert_eq!(store.browser().get(&future).await.status, StatusCode::GONE);

    // The recipient who opened the link in time has 30 minutes to sign in…
    world.advance(29 * 60);
    let confirm = recipient
        .post(
            "/compte/inscription",
            &[
                ("email", "sacha@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;
    assert!(confirm.has("activate"));
    // …after which the verified link is forgotten.
    world.advance(2 * 60);
    let expired = recipient.post("/cadeau/activer", &[]).await;
    assert_eq!(expired.status, StatusCode::BAD_REQUEST);
    assert_eq!(expired.attr("activation-error", "data-code").as_deref(), Some("no_activation"));
    assert!(store.state.library("sacha@example.org").is_empty());

    // A new link (a new press on the button at eBook Cadeau) works.
    let fresh = recipient.get(&world.ebc.redirect_url(&gift_id)).await;
    assert!(fresh.has("activate"));
    assert_eq!(recipient.post("/cadeau/activer", &[]).await.path, "/cadeau/merci");
}

#[tokio::test]
async fn a_gift_goes_to_one_account_only() {
    let world = World::new().await;
    let (store, _, gift_id) = store_with_gift(&world).await;
    let link = world.ebc.redirect_url(&gift_id);

    // Two people open the same link before either confirms.
    let sacha = store.browser();
    sacha.get(&link).await;
    sacha
        .post(
            "/compte/inscription",
            &[
                ("email", "sacha@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;
    let intruder = store.browser();
    intruder.get(&link).await;
    let confirm = intruder
        .post(
            "/compte/inscription",
            &[
                ("email", "intrus@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;
    assert!(confirm.has("activate"));

    assert_eq!(sacha.post("/cadeau/activer", &[]).await.path, "/cadeau/merci");
    let refused = intruder.post("/cadeau/activer", &[]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert_eq!(refused.attr("activation-error", "data-code").as_deref(), Some("already_activated"));
    assert!(!refused.html().contains("sacha@example.org"), "the page does not say who has it");
    assert!(store.state.library("intrus@example.org").is_empty());

    // Opening the link again: the owner is told it is theirs, anyone else is refused.
    let again = sacha.get(&link).await;
    assert_eq!(again.status, StatusCode::OK);
    assert!(again.has("activation-done"));
    assert_eq!(intruder.get(&link).await.status, StatusCode::CONFLICT);
    assert_eq!(store.browser().get(&link).await.status, StatusCode::CONFLICT);
    assert_eq!(world.ebc.requests_to("POST", "/fulfilled").len(), 1);
    assert_eq!(store.state.library("sacha@example.org").len(), 1);
}

#[tokio::test]
async fn a_cancelled_gift_cannot_be_activated() {
    let world = World::new().await;
    let (store, order_path, gift_id) = store_with_gift(&world).await;
    let link = world.ebc.redirect_url(&gift_id);
    let recipient = store.browser();
    recipient.get(&link).await;
    recipient
        .post(
            "/compte/inscription",
            &[
                ("email", "sacha@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;

    let buyer = store.browser();
    buyer.login("camille@example.org", "motdepasse").await;
    let cancelled = buyer.post(&format!("{order_path}/annuler"), &[]).await;
    assert_eq!(cancelled.attr("gift-status", "data-status").as_deref(), Some("cancelled"));

    let refused = recipient.post("/cadeau/activer", &[]).await;
    assert_eq!(refused.status, StatusCode::GONE);
    assert_eq!(refused.attr("activation-error", "data-code").as_deref(), Some("gift_void"));
    assert_eq!(store.browser().get(&link).await.status, StatusCode::GONE);
    assert!(store.state.library("sacha@example.org").is_empty());
}

#[tokio::test]
async fn the_book_is_delivered_even_if_the_confirmation_must_be_retried() {
    let world = World::new().await;
    let (store, order_path, gift_id) = store_with_gift(&world).await;
    let recipient = store.browser();
    recipient.get(&world.ebc.redirect_url(&gift_id)).await;
    recipient
        .post(
            "/compte/inscription",
            &[
                ("email", "sacha@example.org"),
                ("password", "motdepasse"),
                ("next", "/cadeau/activer"),
            ],
        )
        .await;

    // eBook Cadeau is down at the moment of activation.
    world.ebc.fail_next("POST", "/fulfilled", 503, 1, false);
    let done = recipient.post("/cadeau/activer", &[]).await;
    assert_eq!(done.path, "/cadeau/merci", "the recipient is not punished for an outage");
    assert_eq!(store.state.library("sacha@example.org").len(), 1);
    assert_eq!(world.ebc.gift(&gift_id).status, "created");
    assert_eq!(store.snapshot()["gifts"][0]["fulfilment_confirmed"], false);

    // The next refresh of the buyer's order page retries the confirmation.
    let buyer = store.browser();
    buyer.login("camille@example.org", "motdepasse").await;
    let order = buyer.get(&order_path).await;
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("fulfilled"));
    assert_eq!(world.ebc.gift(&gift_id).status, "fulfilled");
    assert_eq!(store.snapshot()["gifts"][0]["fulfilment_confirmed"], true);
    assert_eq!(world.ebc.requests_to("POST", "/fulfilled").len(), 2);
    buyer.get(&order_path).await;
    assert_eq!(
        world.ebc.requests_to("POST", "/fulfilled").len(),
        2,
        "confirmed once, not on every load"
    );
}

#[tokio::test]
async fn webhooks_update_the_gift_when_their_signature_is_valid() {
    let world = World::new().await;
    let (store, order_path, gift_id) = store_with_gift(&world).await;
    let buyer = store.browser();
    buyer.login("camille@example.org", "motdepasse").await;
    let status_without_api = || async {
        // Make the status poll fail so that the page shows what the webhook recorded.
        world.ebc.fail_next("GET", "/gifts/g_0001", 503, 1, false);
        buyer.get(&order_path).await.attr("gift-status", "data-status").unwrap()
    };

    let event = json!({"event": "gift.redeemed", "gift_id": gift_id, "order_ref": "GE-2026-000001", "at": T0 + 5});
    let (status, body) = world.ebc.send_webhook(&store.base, "grande-enseigne", &event).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"received": true, "known": true}));
    assert_eq!(status_without_api().await, "redeemed");
    assert_eq!(store.snapshot()["gifts"][0]["redeemed_at"], T0 + 5);

    // Deliveries are retried by the sender: receiving the same event again changes nothing.
    let (status, _) = world.ebc.send_webhook(&store.base, "grande-enseigne", &event).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(store.snapshot()["webhooks"].as_array().unwrap().len(), 1);

    // Unknown gifts and unknown event names are acknowledged (so the sender stops retrying).
    let stranger =
        json!({"event": "gift.redeemed", "gift_id": "g_4242", "order_ref": "X", "at": T0});
    let (status, body) = world.ebc.send_webhook(&store.base, "grande-enseigne", &stranger).await;
    assert_eq!((status, body["known"].clone()), (StatusCode::OK, json!(false)));
    let novel = json!({"event": "gift.teleported", "gift_id": gift_id, "at": T0});
    let (status, _) = world.ebc.send_webhook(&store.base, "grande-enseigne", &novel).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(status_without_api().await, "redeemed");

    let cancelled = json!({"event": "gift.cancelled", "gift_id": gift_id, "order_ref": "GE-2026-000001", "at": T0 + 9});
    world.ebc.send_webhook(&store.base, "grande-enseigne", &cancelled).await;
    assert_eq!(status_without_api().await, "cancelled");
    // A late "downloaded" does not resurrect a cancelled gift.
    let late = json!({"event": "gift.downloaded", "gift_id": gift_id, "order_ref": "GE-2026-000001", "at": T0 + 3});
    world.ebc.send_webhook(&store.base, "grande-enseigne", &late).await;
    assert_eq!(status_without_api().await, "cancelled");
}

#[tokio::test]
async fn webhooks_with_a_bad_signature_are_rejected() {
    let world = World::new().await;
    let (store, _, gift_id) = store_with_gift(&world).await;
    let body =
        serde_json::to_vec(&json!({"event": "gift.cancelled", "gift_id": gift_id, "at": T0}))
            .unwrap();
    let ts = T0.to_string();
    let good = sign_request(GRANDE_SECRET, T0, "POST", "/webhooks/ebc", &body);
    let code = |(status, body): (StatusCode, Value)| {
        (status, body["error"]["code"].as_str().unwrap_or("").to_owned())
    };
    let unauthorized = |c: &str| (StatusCode::UNAUTHORIZED, c.to_owned());

    // Missing pieces.
    assert_eq!(
        code(post_webhook(&store.base, None, Some(&good), body.clone()).await),
        unauthorized("missing_signature")
    );
    assert_eq!(
        code(post_webhook(&store.base, Some(&ts), None, body.clone()).await),
        unauthorized("missing_signature")
    );
    assert_eq!(
        code(post_webhook(&store.base, Some(&ts), Some(&good[3..]), body.clone()).await),
        unauthorized("missing_signature"),
        "the v1= scheme prefix is required"
    );
    // Wrong secret, wrong method, wrong path, wrong body.
    for forged in [
        sign_request(common::SEPTANTE_SECRET, T0, "POST", "/webhooks/ebc", &body),
        sign_request(GRANDE_SECRET, T0, "GET", "/webhooks/ebc", &body),
        sign_request(GRANDE_SECRET, T0, "POST", "/webhooks/other", &body),
        sign_request(GRANDE_SECRET, T0, "POST", "/webhooks/ebc", b"{}"),
        sign_request(GRANDE_SECRET, T0 + 1, "POST", "/webhooks/ebc", &body),
    ] {
        assert_eq!(
            code(post_webhook(&store.base, Some(&ts), Some(&forged), body.clone()).await),
            unauthorized("bad_signature")
        );
    }
    // A replayed (old) delivery, even correctly signed.
    for skew in [-301, 301] {
        let old = (T0 + skew).to_string();
        let signature = sign_request(GRANDE_SECRET, T0 + skew, "POST", "/webhooks/ebc", &body);
        assert_eq!(
            code(post_webhook(&store.base, Some(&old), Some(&signature), body.clone()).await),
            unauthorized("stale_timestamp")
        );
    }
    // Nothing got through.
    assert_eq!(store.snapshot()["gifts"][0]["status"], "created");
    assert_eq!(store.snapshot()["webhooks"], json!([]));

    // Correctly signed but not an event: 400, still nothing recorded.
    for garbage in [b"pas du json".to_vec(), b"{}".to_vec(), b"[1,2]".to_vec()] {
        let signature = sign_request(GRANDE_SECRET, T0, "POST", "/webhooks/ebc", &garbage);
        let answer = post_webhook(&store.base, Some(&ts), Some(&signature), garbage).await;
        assert_eq!(code(answer), (StatusCode::BAD_REQUEST, "invalid_payload".to_owned()));
    }
    assert_eq!(store.snapshot()["webhooks"], json!([]));

    // The genuine one, at the edge of the time window, goes through.
    world.advance(300);
    let (status, _) = post_webhook(&store.base, Some(&ts), Some(&good), body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(store.snapshot()["gifts"][0]["status"], "cancelled");
}

#[tokio::test]
async fn webhooks_are_not_accepted_without_a_partner_secret() {
    let world = World::new().await;
    let store = world
        .store_with(StoreKind::Septante, |mut config| {
            config.partner = None;
            config
        })
        .await;
    let body = b"{}".to_vec();
    let signature = sign_request("", T0, "POST", "/webhooks/ebc", &body);
    let (status, _) =
        post_webhook(&store.base, Some(&T0.to_string()), Some(&signature), body).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "an empty secret must not validate anything");
}

#[tokio::test]
async fn both_stores_run_side_by_side_with_separate_state() {
    let world = World::new().await;
    let septante = world.store(StoreKind::Septante).await;
    let grande = world.store(StoreKind::GrandeEnseigne).await;
    let a = septante
        .browser()
        .buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", ""))
        .await;
    let b = grande
        .browser()
        .buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", ""))
        .await;
    assert_eq!(a.order_ref(), "SEP-2026-000001");
    assert_eq!(b.order_ref(), "GE-2026-000001");

    let gifts = world.ebc.gifts();
    assert_eq!(gifts.len(), 2);
    assert_eq!(
        (gifts[0].partner.as_str(), gifts[1].partner.as_str()),
        ("septante", "grande-enseigne")
    );
    assert_eq!(gifts[0].request["delivery"]["type"], "hosted_file");
    assert_eq!(gifts[1].request["delivery"]["type"], "reseller_redirect");

    // The redirect of the Grande Enseigne gift is meaningless to Septante.
    let link = world.ebc.redirect_url("g_0002").replace(&grande.base, &septante.base);
    assert_eq!(septante.browser().get(&link).await.status, StatusCode::FORBIDDEN);
    // And a webhook signed for one store is refused by the other.
    let event = json!({"event": "gift.redeemed", "gift_id": "g_0001", "at": T0});
    let (status, _) = world.ebc.send_webhook(&grande.base, "septante", &event).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, body) = world.ebc.send_webhook(&septante.base, "septante", &event).await;
    assert_eq!((status, body["known"].clone()), (StatusCode::OK, json!(true)));
    assert_eq!(grande.snapshot()["gifts"][0]["status"], "created");
    assert_eq!(septante.snapshot()["gifts"][0]["status"], "redeemed");
}
