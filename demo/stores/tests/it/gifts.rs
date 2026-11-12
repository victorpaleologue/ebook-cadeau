//! Checkout with the gift option, against a mock eBook Cadeau partner API that verifies
//! every signature (§4.1) and against the real demo printer.

use crate::common::{
    CardMode, World, code, code_display, gift_home, gift_post, open_epub, plain_client, sha256_hex,
    text_of,
};
use axum::http::StatusCode;
use ebc_demo_stores::{GiftDelivery, StoreKind};
use serde_json::{Value, json};

#[tokio::test]
async fn septante_gift_printed_at_home_is_a_partner_file_the_store_serves_itself() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    let order = browser
        .buy(
            "camille@example.org",
            &["candide"],
            &gift_home("Camille", "Sacha", "Joyeux anniversaire !"),
        )
        .await;
    assert_eq!(order.status, StatusCode::OK, "{}", order.html());
    let reference = order.order_ref();

    // --- what the buyer sees ---
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert_eq!(order.text("gift-recipient").as_deref(), Some("Sacha"));
    assert_eq!(order.text("gift-giver").as_deref(), Some("Camille"));
    assert_eq!(order.text("gift-message-text").as_deref(), Some("Joyeux anniversaire !"));
    assert_eq!(order.text("order-total").as_deref(), Some("2,99\u{a0}€"), "the home card is free");
    assert_eq!(order.text("claim-code").as_deref(), Some(code_display(1).as_str()));
    assert_eq!(order.text("claim-url"), Some(format!("{}/k/{}", world.ebc.base, code(1))));
    assert!(!order.has("print-status") && !order.has("retry") && !order.has("gift-error"));
    assert!(order.has("cancel-gift"));

    let card_href = order.attr("card-pdf", "href").unwrap();
    assert_eq!(card_href, format!("/commande/{reference}/carte-1.pdf"));
    let card = browser.get(&card_href).await;
    assert_eq!(card.status, StatusCode::OK);
    assert_eq!(card.header("content-type"), Some("application/pdf"));
    assert!(
        card.header("content-disposition").unwrap().starts_with("attachment; filename=\"carte-")
    );
    assert!(card.body.starts_with(b"%PDF-"));
    assert_eq!(
        browser.get(&format!("/commande/{reference}/carte-2.pdf")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        browser.get(&format!("/commande/{reference}/carte-0.pdf")).await.status,
        StatusCode::NOT_FOUND
    );

    // The gift is for someone else: nothing lands in the buyer's library.
    assert!(browser.get("/bibliotheque").await.has("library-empty"));

    // --- what the partner API received, in order, all correctly signed (the mock answers
    //     401 otherwise) ---
    let calls: Vec<(String, String)> =
        world.ebc.requests().iter().map(|r| (r.method.clone(), r.path.clone())).collect();
    let expected = [
        ("POST", "/api/partner/v1/gifts".to_owned()),
        ("PUT", "/api/partner/v1/gifts/g_0001/cover".to_owned()),
        ("GET", "/api/partner/v1/gifts/g_0001/card.pdf".to_owned()),
    ];
    for (index, (method, path)) in expected.iter().enumerate() {
        assert_eq!((calls[index].0.as_str(), &calls[index].1), (*method, path));
    }
    assert!(world.ebc.requests().iter().all(|r| r.status < 400), "{calls:?}");
    for request in world.ebc.requests() {
        assert_eq!(request.header("x-ebc-partner"), Some("septante"));
        assert!(request.header("x-ebc-signature").unwrap().starts_with("v1="));
    }

    let creation = &world.ebc.requests_to("POST", "/gifts")[0];
    assert_eq!(creation.header("idempotency-key"), Some(reference.as_str()));
    assert_eq!(creation.header("content-type"), Some("application/json"));
    assert_eq!(
        creation.json(),
        json!({
            "order_ref": reference,
            "book": {
                "title": "Candide",
                "isbn13": "9786990000015",
                "contributors": [{"name": "Voltaire"}],
                "publisher": "Éditions du Domaine public (démonstration)",
                "language": "fr",
            },
            "giver_name": "Camille",
            "recipient_name": "Sacha",
            "message": "Joyeux anniversaire !",
            "locale": "fr",
            "delivery": {"type": "partner_file", "format": "epub", "url": format!("{}/cadeau/fichier", store.base)},
        })
    );

    // --- the recipient claims the gift: eBook Cadeau asks the store for a link (signed),
    //     and the store stamps a copy for the recipient at that moment ---
    let (status, link) = world.ebc.ask_file_link("g_0001").await;
    assert_eq!(status, StatusCode::OK, "{link}");
    assert_eq!(link["file_name"], "candide.epub");
    assert_eq!(link["media_type"], "application/epub+zip");
    let download_url = link["download_url"].as_str().unwrap().to_owned();
    assert!(download_url.starts_with(&format!("{}/cadeau/telechargement/", store.base)));
    assert!(link["expires_at"].as_i64().unwrap() > world.ebc.now());
    let file = plain_client().get(&download_url).send().await.unwrap();
    assert_eq!(file.status(), StatusCode::OK);
    assert_eq!(file.headers()["content-type"], "application/epub+zip");
    let bytes = file.bytes().await.unwrap().to_vec();
    assert_eq!(link["size_bytes"].as_u64(), Some(bytes.len() as u64));
    assert_eq!(link["sha256"].as_str().unwrap(), sha256_hex(&bytes));
    let (_, files) = open_epub(&bytes);
    let watermark = text_of(&files, "OEBPS/watermark.xhtml");
    assert!(watermark.contains("Offert à Sacha par Camille"), "{watermark}");
    assert!(watermark.contains(&reference));
    assert!(
        text_of(&files, "OEBPS/package.opf")
            .contains("<meta property=\"demo:licensee\">Offert à Sacha par Camille</meta>")
    );
    assert_eq!(store.snapshot()["gifts"][0]["file_links"], 1);
    // Each claim gets a fresh link; a link dies after its five minutes.
    let (_, again) = world.ebc.ask_file_link("g_0001").await;
    assert_ne!(again["download_url"], link["download_url"]);
    world.clock.fetch_add(301, std::sync::atomic::Ordering::SeqCst);
    let expired = plain_client().get(&download_url).send().await.unwrap();
    assert_eq!(expired.status(), StatusCode::NOT_FOUND);
    // A forged or stale request gets nothing.
    let forged = world.ebc.redirect_url("g_0001").replace("ebc_sig=", "ebc_sig=zz");
    assert_eq!(plain_client().get(&forged).send().await.unwrap().status(), StatusCode::BAD_REQUEST);
    let tampered = world.ebc.redirect_url("g_0001").replace("ebc_gift=g_0001", "ebc_gift=g_0002");
    assert_eq!(plain_client().get(&tampered).send().await.unwrap().status(), StatusCode::FORBIDDEN);
    let stale = world.ebc.redirect_url_at("g_0001", world.ebc.now() - 901);
    assert_eq!(plain_client().get(&stale).send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    assert_eq!(store.snapshot()["gifts"][0]["file_links"], 2, "refused requests got no link");

    let cover_upload = &world.ebc.requests_to("PUT", "/g_0001/cover")[0];
    assert_eq!(
        cover_upload.query.as_deref(),
        Some("cover_rights=print_licence"),
        "the store declares its right to print the cover (in the signed path)"
    );
    let cover = world.ebc.gift("g_0001").cover.unwrap();
    let decoder = png::Decoder::new(std::io::Cursor::new(&cover));
    let reader = decoder.read_info().unwrap();
    assert!(reader.info().width >= 1300 && reader.info().height >= 2100);
    let cover_request = &world.ebc.requests_to("PUT", "/cover")[0];
    assert_eq!(cover_request.header("content-type"), Some("image/png"));

    let card_request = &world.ebc.requests_to("GET", "/card.pdf")[0];
    assert_eq!(card_request.query.as_deref(), Some("format=poche&profile=home-a4"));
}

#[tokio::test]
async fn gift_status_is_refreshed_from_the_partner_api_on_each_page_load() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    let order =
        browser.buy("camille@example.org", &["le-horla"], &gift_home("Camille", "Sacha", "")).await;
    let path = order.path.clone();
    assert!(!order.has("gift-message-text"), "no empty quote for an empty message");
    let before = world.ebc.requests_to("GET", "/gifts/g_0001").len();

    world.ebc.set_status("g_0001", "redeemed", 2);
    let page = browser.get(&path).await;
    assert_eq!(page.attr("gift-status", "data-status").as_deref(), Some("redeemed"));
    assert_eq!(page.text("gift-downloads").as_deref(), Some("2"));
    assert!(!page.has("cancel-gift"), "a used gift cannot be refunded");
    assert_eq!(world.ebc.requests_to("GET", "/gifts/g_0001").len(), before + 1);

    // If the partner API is down, the page still renders with the last known status.
    world.ebc.fail_next("GET", "/gifts/g_0001", 503, 1, false);
    world.ebc.set_status("g_0001", "expired", 2);
    let page = browser.get(&path).await;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(page.attr("gift-status", "data-status").as_deref(), Some("redeemed"));
    let page = browser.get(&path).await;
    assert_eq!(page.attr("gift-status", "data-status").as_deref(), Some("expired"));
}

#[tokio::test]
async fn posted_card_goes_through_the_printer() {
    let world = World::new().await;
    let printer = world.printer().await;
    let store = world.store_with(StoreKind::Septante, |c| c.with_printer(&printer.base)).await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("madame-bovary").await;

    // The checkout page quotes the printer's price for one card.
    let checkout = browser.get("/commande").await;
    assert_eq!(checkout.text("card-post-price").as_deref(), Some("5,90\u{a0}€"));
    assert_eq!(checkout.attr("card-delivery-post", "disabled"), None);
    assert!(
        checkout.attr("card-delivery-home", "checked").is_some(),
        "free home printing is the default"
    );
    assert!(
        checkout.has("ship-name") && checkout.has("ship-street") && checkout.has("ship-postcode")
    );

    let order = browser.post("/commande/payer", &gift_post("Camille", "Sacha")).await;
    assert_eq!(order.status, StatusCode::OK, "{}", order.html());
    let reference = order.order_ref();
    assert_eq!(order.text("order-total").as_deref(), Some("8,89\u{a0}€"), "2,99 + 5,90");
    assert!(order.text("order-card-line").unwrap().contains("5,90"));
    assert_eq!(order.attr("print-status", "data-status").as_deref(), Some("accepted"));
    assert!(order.has("card-pdf"), "the buyer can still download the card");

    // The card was requested with the professional profile and handed to the printer.
    let card_request = &world.ebc.requests_to("GET", "/card.pdf")[0];
    assert_eq!(card_request.query.as_deref(), Some("format=poche&profile=pro"));
    let printed = printer.state.orders();
    assert_eq!(printed.len(), 1);
    assert_eq!(printed[0].reference, reference);
    assert_eq!(printed[0].product, "card-poche");
    assert_eq!(printed[0].quantity, 1);
    assert_eq!(printed[0].price_cents, 590);
    assert_eq!(
        (printed[0].address.name.as_str(), printed[0].address.line1.as_str()),
        ("Sacha Martin", "12 rue des Lilas")
    );
    assert_eq!(
        (
            printed[0].address.postal_code.as_str(),
            printed[0].address.city.as_str(),
            printed[0].address.country.as_str()
        ),
        ("75011", "Paris", "FR")
    );
    let card = browser.get(&order.attr("card-pdf", "href").unwrap()).await;
    assert_eq!(printed[0].pdf_bytes, card.body.len(), "the printer got the same PDF");

    // The printer's callbacks reach the store (webhook with a secret token)…
    let callback = printed[0].callback_url.clone().unwrap();
    assert!(callback.starts_with(&format!("{}/webhooks/printer?token=", store.base)));
    printer.state.advance_all().await;
    assert!(printer.state.orders()[0].callbacks[0].delivered);
    assert_eq!(
        store.snapshot()["gifts"][0]["print"]["status"],
        "printing",
        "the callback alone updated the store"
    );
    // …and the order page also polls the printer.
    printer.state.advance_all().await;
    let page = browser.get(&order.path).await;
    assert_eq!(page.attr("print-status", "data-status").as_deref(), Some("shipped"));
    assert_eq!(page.text("print-tracking").as_deref(), Some("DC00000001FR"));
    assert!(!store.snapshot().to_string().contains("token="), "callback tokens stay private");
}

#[tokio::test]
async fn printer_callbacks_need_their_token() {
    let world = World::new().await;
    let printer = world.printer().await;
    let store = world.store_with(StoreKind::Septante, |c| c.with_printer(&printer.base)).await;
    let browser = store.browser();
    browser.buy("camille@example.org", &["candide"], &gift_post("Camille", "Sacha")).await;
    let order_id = printer.state.orders()[0].order_id.clone();
    let callback_url = printer.state.orders()[0].callback_url.clone().unwrap();

    let send = |url: String, body: Value| async move {
        let response = plain_client().post(url).json(&body).send().await.unwrap();
        response.status()
    };
    let forged = json!({"order_id": order_id, "status": "shipped", "tracking": "FAKE"});
    let base = format!("{}/webhooks/printer", store.base);
    assert_eq!(send(base.clone(), forged.clone()).await, StatusCode::NOT_FOUND);
    assert_eq!(send(format!("{base}?token=deadbeef"), forged.clone()).await, StatusCode::NOT_FOUND);
    assert_eq!(send(format!("{base}?token="), forged).await, StatusCode::NOT_FOUND);
    assert_eq!(store.snapshot()["gifts"][0]["print"]["status"], "accepted");

    // Right token, wrong order id; then garbage; then a status outside the vocabulary.
    let other = json!({"order_id": "po_other", "status": "shipped"});
    assert_eq!(send(callback_url.clone(), other).await, StatusCode::NOT_FOUND);
    let response = plain_client().post(&callback_url).body("pas du json").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let odd = json!({"order_id": order_id, "status": "<b>exploded</b>", "tracking": {"x": 1}});
    assert_eq!(send(callback_url.clone(), odd).await, StatusCode::OK);
    assert_eq!(store.snapshot()["gifts"][0]["print"]["status"], "accepted");

    let genuine = json!({"order_id": order_id, "status": "shipped", "tracking": "DC00000001FR"});
    assert_eq!(send(callback_url, genuine).await, StatusCode::OK);
    assert_eq!(store.snapshot()["gifts"][0]["print"]["status"], "shipped");
    assert_eq!(store.snapshot()["gifts"][0]["print"]["tracking"], "DC00000001FR");
}

#[tokio::test]
async fn a_card_the_printer_refuses_is_reported_with_its_findings() {
    let world = World::new().await;
    let printer = world.printer().await;
    // eBook Cadeau renders an A6 card although the store orders the "poche" product.
    world.ebc.set_card_mode(CardMode::Sample("card-a6"));
    let store = world.store_with(StoreKind::Septante, |c| c.with_printer(&printer.base)).await;
    let browser = store.browser();
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_post("Camille", "Sacha")).await;
    assert_eq!(order.status, StatusCode::OK);
    assert_eq!(order.attr("print-status", "data-status").as_deref(), Some("rejected"));
    assert!(order.text("print-findings").unwrap().contains("BOX_TRIM_SIZE"));
    assert!(order.has("gift-error"));
    assert_eq!(
        order.attr("gift-status", "data-status").as_deref(),
        Some("created"),
        "the gift itself is fine"
    );
    assert!(order.has("card-pdf"));
    assert!(printer.state.orders().is_empty());
    assert!(!order.has("retry"), "resending the same file would not help");
}

#[tokio::test]
async fn post_option_is_unavailable_without_a_reachable_printer() {
    let world = World::new().await;
    // No printer configured.
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("candide").await;
    let checkout = browser.get("/commande").await;
    assert!(checkout.attr("card-delivery-post", "disabled").is_some());
    assert!(checkout.has("card-post-unavailable") && !checkout.has("ship-fields"));
    // Forcing the choice is refused, and the form comes back filled in.
    let refused = browser.post("/commande/payer", &gift_post("Camille", "Sacha")).await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(refused.text("checkout-errors").unwrap().contains("envoi postal"));
    assert_eq!(refused.attr("giver-name", "value").as_deref(), Some("Camille"));
    assert_eq!(refused.attr("recipient-name", "value").as_deref(), Some("Sacha"));
    assert!(refused.attr("gift-option", "checked").is_some());
    assert_eq!(store.snapshot()["orders"], json!([]));
    assert!(browser.get("/panier").await.has("line-candide"), "the cart is kept");

    // A printer that is configured but down behaves the same.
    let store =
        world.store_with(StoreKind::Septante, |c| c.with_printer("http://127.0.0.1:9")).await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("candide").await;
    assert!(browser.get("/commande").await.has("card-post-unavailable"));
    let home = browser.post("/commande/payer", &gift_home("Camille", "Sacha", "")).await;
    assert_eq!(
        home.attr("gift-status", "data-status").as_deref(),
        Some("created"),
        "home printing still works"
    );
}

#[tokio::test]
async fn checkout_validates_the_gift_fields() {
    let world = World::new().await;
    let printer = world.printer().await;
    let store = world.store_with(StoreKind::Septante, |c| c.with_printer(&printer.base)).await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("candide").await;

    let missing = browser
        .post(
            "/commande/payer",
            &[
                ("gift", "1"),
                ("giver-name", " "),
                ("recipient-name", ""),
                ("card-delivery", "home"),
            ],
        )
        .await;
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing.count("#checkout-errors li"), 2);

    let long_name = "x".repeat(81);
    let too_long = browser.post("/commande/payer", &gift_home("Camille", &long_name, "")).await;
    assert_eq!(too_long.status, StatusCode::UNPROCESSABLE_ENTITY);

    let mut no_address = gift_post("Camille", "Sacha");
    no_address.retain(|(name, _)| !name.starts_with("ship-"));
    let page = browser.post("/commande/payer", &no_address).await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(page.count("#checkout-errors li"), 4);
    assert!(page.attr("card-delivery-post", "checked").is_some(), "the choice is remembered");

    let mut abroad = gift_post("Camille", "Sacha");
    abroad.retain(|(name, _)| *name != "ship-postcode");
    abroad.push(("ship-postcode", "SW1A 1AA"));
    assert_eq!(
        browser.post("/commande/payer", &abroad).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let unknown_choice = browser
        .post(
            "/commande/payer",
            &[
                ("gift", "1"),
                ("giver-name", "C"),
                ("recipient-name", "S"),
                ("card-delivery", "pigeon"),
            ],
        )
        .await;
    assert_eq!(unknown_choice.status, StatusCode::UNPROCESSABLE_ENTITY);

    assert_eq!(store.snapshot()["orders"], json!([]), "nothing was ordered");
    assert!(world.ebc.requests().is_empty() && printer.state.orders().is_empty());

    // Without the checkbox the gift fields are ignored altogether.
    let plain = browser
        .post(
            "/commande/payer",
            &[("giver-name", "Camille"), ("recipient-name", ""), ("card-delivery", "post")],
        )
        .await;
    assert_eq!(plain.status, StatusCode::OK);
    assert!(plain.has("purchase-done") && !plain.has("gift-status"));
}

#[tokio::test]
async fn gift_option_is_hidden_when_no_partner_api_is_configured() {
    let world = World::new().await;
    let store = world
        .store_with(StoreKind::Septante, |mut config| {
            config.partner = None;
            config
        })
        .await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("candide").await;
    let checkout = browser.get("/commande").await;
    assert!(checkout.has("gift-unavailable") && !checkout.has("gift-option"));
    let forced = browser.post("/commande/payer", &gift_home("Camille", "Sacha", "")).await;
    assert_eq!(forced.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(store.snapshot()["orders"], json!([]));
    // A plain purchase is unaffected.
    assert!(browser.post("/commande/payer", &[]).await.has("purchase-done"));
}

#[tokio::test]
async fn a_failed_step_can_be_retried_without_creating_anything_twice() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();

    // The gift creation fails once with a 503.
    world.ebc.fail_next("POST", "/gifts", 503, 1, false);
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    assert_eq!(
        order.status,
        StatusCode::OK,
        "a partner outage must not break the confirmation page"
    );
    let reference = order.order_ref();
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("pending"));
    assert!(order.text("gift-error").unwrap().contains("eBook Cadeau"));
    assert!(order.has("retry") && !order.has("card-pdf") && !order.has("claim-code"));
    assert_eq!(store.snapshot()["orders"][0]["status"], "paid");

    let retried = browser.post(&format!("/commande/{reference}/relancer"), &[]).await;
    assert_eq!(retried.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert!(retried.has("card-pdf") && !retried.has("retry") && !retried.has("gift-error"));

    let creations = world.ebc.requests_to("POST", "/gifts");
    assert_eq!(creations.len(), 2);
    assert_eq!(creations[0].header("idempotency-key"), creations[1].header("idempotency-key"));
    assert_eq!(creations[0].body, creations[1].body, "the retry sends the very same request");
    assert_eq!(world.ebc.gifts().len(), 1);
}

#[tokio::test]
async fn a_lost_answer_is_recovered_through_the_idempotency_key() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();

    // eBook Cadeau creates the gift but the answer never arrives (500 after processing).
    world.ebc.fail_next("POST", "/gifts", 500, 1, true);
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    let reference = order.order_ref();
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("pending"));
    assert_eq!(world.ebc.gifts().len(), 1, "the gift exists on the other side");

    let retried = browser.post(&format!("/commande/{reference}/relancer"), &[]).await;
    assert_eq!(retried.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert_eq!(retried.text("claim-code").as_deref(), Some(code_display(1).as_str()));
    assert_eq!(world.ebc.gifts().len(), 1, "replayed, not duplicated");
    assert_eq!(store.snapshot()["gifts"][0]["gift_id"], "g_0001");

    // Later steps are retried the same way: cover, then card.
    world.ebc.fail_next("PUT", "/cover", 502, 1, false);
    browser.add_to_cart("le-horla").await;
    let second = browser.post("/commande/payer", &gift_home("Camille", "Sacha", "")).await;
    assert!(second.has("retry") && second.has("claim-code") && !second.has("card-pdf"));
    world.ebc.fail_next("GET", "/card.pdf", 500, 1, false);
    let again = browser.post(&format!("/commande/{}/relancer", second.order_ref()), &[]).await;
    assert!(again.has("retry") && !again.has("card-pdf"));
    let done = browser.post(&format!("/commande/{}/relancer", second.order_ref()), &[]).await;
    assert!(done.has("card-pdf") && !done.has("retry"));
    assert_eq!(world.ebc.requests_to("PUT", "/g_0002/cover").len(), 2);
    assert_eq!(world.ebc.gifts().len(), 2);
}

#[tokio::test]
async fn an_unreachable_partner_api_leaves_a_paid_order_that_can_be_retried() {
    let world = World::new().await;
    let store = world
        .store_with(StoreKind::GrandeEnseigne, |c| {
            c.with_partner("http://127.0.0.1:9", "grande-enseigne", "x")
        })
        .await;
    let browser = store.browser();
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    assert_eq!(order.status, StatusCode::OK);
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("pending"));
    assert!(order.text("gift-error").unwrap().contains("injoignable"));
    assert!(order.has("retry") && order.has("cancel-gift"));
    // Cancelling a gift that never reached eBook Cadeau needs no remote call.
    let cancelled = browser.post(&format!("/commande/{}/annuler", order.order_ref()), &[]).await;
    assert_eq!(cancelled.attr("order-status", "data-status").as_deref(), Some("refunded"));
    assert_eq!(cancelled.attr("gift-status", "data-status").as_deref(), Some("cancelled"));
    assert!(!cancelled.has("retry") && !cancelled.has("cancel-gift"));
}

#[tokio::test]
async fn requests_with_a_skewed_clock_or_wrong_secret_are_refused_by_the_api() {
    let world = World::new().await;
    // A store whose clock is ten minutes behind: the API answers 401 stale_timestamp.
    let clock = world.clock.clone();
    let late = world
        .store_with(StoreKind::Septante, move |c| {
            c.with_clock(ebc_demo_stores::Clock::from_fn(move || {
                clock.load(std::sync::atomic::Ordering::SeqCst) - 601
            }))
        })
        .await;
    let order = late.browser().buy("a@example.org", &["candide"], &gift_home("A", "B", "")).await;
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("pending"));
    assert!(
        order.text("gift-error").unwrap().contains("stale_timestamp"),
        "{:?}",
        order.text("gift-error")
    );

    // A store with the wrong secret: 401 bad_signature.
    let wrong = world
        .store_with(StoreKind::Septante, |c| {
            c.with_partner(&world.ebc.base, "septante", "pas-le-bon-secret")
        })
        .await;
    let order = wrong.browser().buy("a@example.org", &["candide"], &gift_home("A", "B", "")).await;
    assert!(order.text("gift-error").unwrap().contains("bad_signature"));

    // An unknown partner id.
    let unknown = world
        .store_with(StoreKind::Septante, |c| {
            c.with_partner(&world.ebc.base, "inconnu", crate::common::SEPTANTE_SECRET)
        })
        .await;
    let order =
        unknown.browser().buy("a@example.org", &["candide"], &gift_home("A", "B", "")).await;
    assert!(order.text("gift-error").unwrap().contains("unknown_partner"));

    assert!(world.ebc.gifts().is_empty());
    assert!(world.ebc.requests().iter().all(|r| r.status == 401));
}

#[tokio::test]
async fn without_a_server_side_card_the_claim_link_and_code_are_shown() {
    let world = World::new().await;
    world.ebc.set_card_mode(CardMode::NotImplemented);
    let printer = world.printer().await;
    let store = world.store_with(StoreKind::Septante, |c| c.with_printer(&printer.base)).await;
    let browser = store.browser();
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert!(!order.has("card-pdf"));
    assert!(order.text("card-unavailable").unwrap().contains("n'est pas disponible"));
    assert_eq!(order.attr("claim-url", "href"), Some(format!("{}/k/{}", world.ebc.base, code(1))));
    assert_eq!(order.text("claim-code").as_deref(), Some(code_display(1).as_str()));
    assert!(!order.has("retry"), "a 501 is an answer, not a failure");
    assert_eq!(
        browser.get(&format!("/commande/{}/carte-1.pdf", order.order_ref())).await.status,
        StatusCode::NOT_FOUND
    );

    // With a posted card, the printer cannot be given anything: say so, order nothing.
    browser.add_to_cart("le-horla").await;
    let posted = browser.post("/commande/payer", &gift_post("Camille", "Sacha")).await;
    assert!(
        posted.has("card-unavailable") && posted.has("gift-error") && !posted.has("print-status")
    );
    assert!(printer.state.orders().is_empty());
}

#[tokio::test]
async fn several_books_make_several_gifts() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("candide").await;
    browser.add_to_cart("les-fleurs-du-mal").await;
    let checkout = browser.get("/commande").await;
    assert!(checkout.html().contains("Offrir ces ebooks (une carte par livre)"));
    let order =
        browser.post("/commande/payer", &gift_home("Camille", "Sacha", "Bonne lecture")).await;
    let reference = order.order_ref();

    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert_eq!(order.attr("gift-status-2", "data-status").as_deref(), Some("created"));
    assert_eq!(order.text("claim-code").as_deref(), Some(code_display(1).as_str()));
    assert_eq!(order.text("claim-code-2").as_deref(), Some(code_display(2).as_str()));
    let first = browser.get(&order.attr("card-pdf", "href").unwrap()).await;
    let second = browser.get(&order.attr("card-pdf-2", "href").unwrap()).await;
    assert!(first.body.starts_with(b"%PDF-") && second.body.starts_with(b"%PDF-"));
    assert_ne!(first.body, second.body, "one card per book");

    let gifts = world.ebc.gifts();
    assert_eq!(
        gifts.iter().map(|g| g.order_ref.clone()).collect::<Vec<_>>(),
        [format!("{reference}-1"), format!("{reference}-2")]
    );
    let titles: Vec<&str> =
        gifts.iter().map(|g| g.request["book"]["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Candide", "Les Fleurs du mal"]);
    let keys: Vec<String> = world
        .ebc
        .requests_to("POST", "/gifts")
        .iter()
        .map(|r| r.header("idempotency-key").unwrap().to_owned())
        .collect();
    assert_eq!(keys, [format!("{reference}-1"), format!("{reference}-2")]);
    // Each gift is a file the store serves; the store stamps the right book for each.
    for (gift_id, file_name) in [("g_0001", "candide.epub"), ("g_0002", "les-fleurs-du-mal.epub")] {
        assert_eq!(world.ebc.gift(gift_id).request["delivery"]["type"], "partner_file");
        let (status, link) = world.ebc.ask_file_link(gift_id).await;
        assert_eq!((status, link["file_name"].as_str()), (StatusCode::OK, Some(file_name)));
    }
}

#[tokio::test]
async fn cancelling_a_gift_voids_the_code_and_refunds_the_order() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    let reference = order.order_ref();

    // The partner API is down: nothing changes and the buyer is told.
    world.ebc.fail_next("POST", "/cancel", 503, 1, false);
    let failed = browser.post(&format!("/commande/{reference}/annuler"), &[]).await;
    assert_eq!(failed.status, StatusCode::BAD_GATEWAY);
    assert!(failed.text("order-notice").unwrap().contains("Annulation impossible"));
    assert_eq!(failed.attr("order-status", "data-status").as_deref(), Some("paid"));

    let cancelled = browser.post(&format!("/commande/{reference}/annuler"), &[]).await;
    assert_eq!(cancelled.status, StatusCode::OK);
    assert_eq!(cancelled.attr("order-status", "data-status").as_deref(), Some("refunded"));
    assert_eq!(cancelled.attr("gift-status", "data-status").as_deref(), Some("cancelled"));
    assert!(!cancelled.has("cancel-gift"));
    assert_eq!(world.ebc.gift("g_0001").status, "cancelled");

    // Cancelling again is refused, and so is cancelling a gift that was used meanwhile.
    let again = browser.post(&format!("/commande/{reference}/annuler"), &[]).await;
    assert_eq!(again.status, StatusCode::CONFLICT);

    browser.add_to_cart("le-horla").await;
    let used = browser.post("/commande/payer", &gift_home("Camille", "Sacha", "")).await;
    world.ebc.set_status("g_0002", "redeemed", 1);
    let refused = browser.post(&format!("/commande/{}/annuler", used.order_ref()), &[]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert!(refused.text("order-notice").unwrap().contains("déjà été utilisé"));
    assert_eq!(world.ebc.gift("g_0002").status, "redeemed");
    assert!(
        world.ebc.requests_to("POST", "/g_0002/cancel").is_empty(),
        "no cancel call for a used gift"
    );

    // A plain purchase has nothing to cancel.
    browser.add_to_cart("madame-bovary").await;
    let plain = browser.post("/commande/payer", &[]).await;
    let refused = browser.post(&format!("/commande/{}/annuler", plain.order_ref()), &[]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn hostile_names_and_messages_are_escaped_everywhere() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    let giver = "<script>alert('giver')</script>";
    let recipient = "\"><img src=x onerror=alert(1)> & Cie";
    let message = "</blockquote><script>alert(2)</script>\n<b>gras</b> &amp; {{7*7}}";
    let order = browser
        .buy("camille@example.org", &["candide"], &gift_home(giver, recipient, message))
        .await;
    assert_eq!(order.status, StatusCode::OK, "{}", order.html());

    // The page shows the text verbatim and contains no injected element.
    assert_eq!(order.text("gift-giver").as_deref(), Some(giver));
    assert_eq!(order.text("gift-recipient").as_deref(), Some(recipient));
    assert_eq!(
        order.text("gift-message-text").as_deref(),
        Some("</blockquote><script>alert(2)</script> <b>gras</b> &amp; {{7*7}}")
    );
    assert_eq!(order.count("script"), 0);
    assert_eq!(order.count("img"), 0);
    assert_eq!(order.count("#gift-message-text b"), 0);
    let html = order.html();
    assert!(!html.contains("<script>") && !html.contains("onerror=alert(1)>"));
    assert!(
        html.contains("&lt;script&gt;alert(&#39;giver&#39;)&lt;/script&gt;")
            || html.contains("&lt;script&gt;alert('giver')&lt;/script&gt;")
    );

    // The partner API receives the raw text, as JSON.
    let creation = world.ebc.requests_to("POST", "/gifts")[0].json();
    assert_eq!(creation["giver_name"], giver);
    assert_eq!(creation["recipient_name"], recipient);
    assert_eq!(creation["message"], message);

    // The stamped EPUB stays well-formed XML (checked by `open_epub`) and shows the text.
    let (_, link) = world.ebc.ask_file_link("g_0001").await;
    let bytes = plain_client()
        .get(link["download_url"].as_str().unwrap())
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let (_, files) = open_epub(&bytes);
    let watermark = text_of(&files, "OEBPS/watermark.xhtml");
    assert!(watermark.contains("&lt;script&gt;alert(&#39;giver&#39;)&lt;/script&gt;"));
    assert!(!watermark.contains("<script>") && !watermark.contains("<img"));

    // A failed validation re-renders the form with the hostile values safely in attributes.
    browser.add_to_cart("le-horla").await;
    let page = browser
        .post(
            "/commande/payer",
            &[
                ("gift", "1"),
                ("giver-name", giver),
                ("recipient-name", ""),
                ("gift-message", message),
                ("card-delivery", "home"),
            ],
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(page.attr("giver-name", "value").as_deref(), Some(giver));
    assert_eq!(
        page.text("gift-message").as_deref(),
        Some("</blockquote><script>alert(2)</script> <b>gras</b> &amp; {{7*7}}")
    );
    assert_eq!(page.count("script"), 0);

    // Control characters and bidi overrides are dropped, not stored.
    browser
        .post(
            "/commande/payer",
            &gift_home("Ca\u{0}mille\u{202e}", "Sa\r\ncha", "ligne 1\r\nligne 2\u{7}"),
        )
        .await;
    let gift = &world.ebc.gifts()[1].request;
    assert_eq!(gift["giver_name"], "Ca mille");
    assert_eq!(gift["recipient_name"], "Sa cha");
    assert_eq!(gift["message"], "ligne 1\nligne 2");
}

#[tokio::test]
async fn grande_enseigne_gifts_use_the_reseller_redirect() {
    let world = World::new().await;
    let store = world.store(StoreKind::GrandeEnseigne).await;
    let browser = store.browser();
    browser.register("camille@example.org", "motdepasse").await;
    browser.add_to_cart("monte-cristo-1").await;
    let checkout = browser.get("/commande").await;
    assert!(checkout.html().contains("Offrir cet Ebook"), "this store writes « Ebook »");
    assert!(checkout.text("gift-delivery-note").unwrap().contains("ajoute le livre à son compte"));
    let order = browser.post("/commande/payer", &gift_home("Camille", "Sacha", "")).await;
    let reference = order.order_ref();
    assert!(reference.starts_with("GE-2026-"));
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert!(order.has("card-pdf"));

    // This store serves no files: its books sync to the recipient's account.
    let endpoint =
        plain_client().get(format!("{}/cadeau/fichier", store.base)).send().await.unwrap();
    assert_eq!(endpoint.status(), StatusCode::NOT_FOUND);
    let creation = &world.ebc.requests_to("POST", "/gifts")[0];
    assert_eq!(creation.header("x-ebc-partner"), Some("grande-enseigne"));
    assert_eq!(
        creation.json()["delivery"],
        json!({"type": "reseller_redirect", "url": format!("{}/cadeau/activer", store.base)})
    );
    assert_eq!(creation.json()["book"]["title"], "Le Comte de Monte-Cristo — Tome 1");
    assert_eq!(store.snapshot()["gift_delivery"], "reseller_redirect");
}

#[tokio::test]
async fn septante_can_be_configured_for_the_reseller_redirect() {
    let world = World::new().await;
    let store = world
        .store_with(StoreKind::Septante, |c| c.with_gift_delivery(GiftDelivery::ResellerRedirect))
        .await;
    let buyer = store.browser();
    let order =
        buyer.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    assert_eq!(order.attr("gift-status", "data-status").as_deref(), Some("created"));
    assert_eq!(world.ebc.gift("g_0001").request["delivery"]["type"], "reseller_redirect");

    // The recipient activates the gift and downloads a copy stamped as a gift.
    let recipient = store.browser();
    let landing = recipient.get(&world.ebc.redirect_url("g_0001")).await;
    assert!(landing.has("register-form") && landing.has("activation-book"));
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
    let done = recipient.post("/cadeau/activer", &[]).await;
    assert_eq!(done.path, "/cadeau/merci");
    let href = done.attr("activation-download", "href").unwrap();
    let epub = recipient.get(&href).await;
    let (_, files) = open_epub(&epub.body);
    let watermark = text_of(&files, "OEBPS/watermark.xhtml");
    assert!(
        watermark.contains("Offert à Sacha par Camille — compte sacha@example.org"),
        "{watermark}"
    );
    assert_eq!(
        buyer.get(&order.path).await.attr("gift-status", "data-status").as_deref(),
        Some("fulfilled")
    );
}

#[tokio::test]
async fn a_lost_card_gets_a_new_code_and_the_old_one_is_voided() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    let order =
        browser.buy("camille@example.org", &["candide"], &gift_home("Camille", "Sacha", "")).await;
    let reference = order.order_ref();
    let reissue_path = format!("/commande/{reference}/nouveau-code");
    assert!(order.has("reissue-gift") && !order.has("code-reissued"));
    assert_eq!(order.text("claim-code").as_deref(), Some(code_display(1).as_str()));
    let first_card = store.snapshot()["gifts"][0]["card"]["sha256"].clone();

    // The answer of the first attempt is lost on the way back: the buyer is told, nothing
    // changes on the page — and the retry gets the *same* new code (one Idempotency-Key
    // per reissue), so no second code is burnt.
    world.ebc.fail_next("POST", "/reissue", 503, 1, true);
    let failed = browser.post(&reissue_path, &[("gift", "1")]).await;
    assert_eq!(failed.status, StatusCode::BAD_GATEWAY);
    assert!(failed.text("order-notice").unwrap().contains("Nouveau code impossible"));
    assert_eq!(failed.text("claim-code").as_deref(), Some(code_display(1).as_str()));

    let reissued = browser.post(&reissue_path, &[("gift", "1")]).await;
    assert_eq!(reissued.status, StatusCode::OK, "{}", reissued.html());
    assert_eq!(reissued.text("claim-code").as_deref(), Some(code_display(2).as_str()));
    assert_eq!(
        reissued.attr("claim-url", "href"),
        Some(format!("{}/k/{}", world.ebc.base, code(2)))
    );
    assert!(reissued.text("code-reissued").unwrap().contains("ne fonctionne plus"));
    let gift = world.ebc.gift("g_0001");
    assert_eq!((gift.code.as_str(), gift.voided.as_slice()), (code(2).as_str(), &[code(1)][..]));
    let calls = world.ebc.requests_to("POST", "/g_0001/reissue");
    assert_eq!(calls.len(), 2);
    let key = format!("{reference}-reissue-1");
    assert!(calls.iter().all(|c| c.header("idempotency-key") == Some(key.as_str())), "{calls:?}");
    // The card is fetched again: it must carry the new code.
    assert_eq!(world.ebc.requests_to("GET", "/g_0001/card.pdf").len(), 2);
    assert!(reissued.has("card-pdf"));
    assert_eq!(store.snapshot()["gifts"][0]["card"]["sha256"], first_card, "mock cards are equal");
    assert_eq!(store.snapshot()["gifts"][0]["reissues"], 1);

    // A second lost card: a new key, a third code. An empty form means the first gift.
    let again = browser.post(&reissue_path, &[]).await;
    assert_eq!(again.text("claim-code").as_deref(), Some(code_display(3).as_str()));
    let calls = world.ebc.requests_to("POST", "/g_0001/reissue");
    assert_eq!(calls[2].header("idempotency-key"), Some(format!("{reference}-reissue-2").as_str()));

    // No such gift in the order.
    for bad in ["2", "0", "x"] {
        let page = browser.post(&reissue_path, &[("gift", bad)]).await;
        assert_eq!(page.status, StatusCode::NOT_FOUND, "{bad}");
    }

    // Once the download limit is reached (v0.2 status `exhausted`) the gift counts as
    // used: no new code, no refund.
    world.ebc.set_status("g_0001", "exhausted", 10);
    let page = browser.get(&format!("/commande/{reference}")).await;
    assert_eq!(page.attr("gift-status", "data-status").as_deref(), Some("exhausted"));
    assert!(!page.has("reissue-gift") && !page.has("cancel-gift"));
    let refused = browser.post(&reissue_path, &[]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert!(refused.text("order-notice").unwrap().contains("ne peut plus recevoir"));
    let refused = browser.post(&format!("/commande/{reference}/annuler"), &[]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert_eq!(world.ebc.requests_to("POST", "/g_0001/reissue").len(), 3);

    // Someone else's order is not found, and nothing is reissued without signing in.
    let stranger = store.browser();
    stranger.register("mallory@example.org", "motdepasse").await;
    assert_eq!(stranger.post(&reissue_path, &[]).await.status, StatusCode::NOT_FOUND);
    let anonymous = store.browser().post(&reissue_path, &[]).await;
    assert_eq!(anonymous.path, format!("/compte/connexion?next=%2Fcommande%2F{reference}"));
}
