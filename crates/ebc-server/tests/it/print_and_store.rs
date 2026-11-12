//! Print orders forwarded for partners (`docs/api.md` §4.6, §5), the file-backed store,
//! the in-flight body budget, retention, and the static web app (§6).

use std::sync::Arc;

use crate::common::{Harness, Receiver, T0, assert_page, start_shop};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use ebc_model::GiftCode;
use ebc_server::{FileStore, MemoryStore, Store};
use serde_json::json;

/// A cover upload the in-flight budget test sends without a valid signature.
const COVER: &str = "/api/partner/v1/gifts/g_00000000000000000000000000000000/cover";

fn error_code(reply: &crate::common::Reply) -> String {
    let body = reply.json();
    assert!(body["error"]["message"].is_string(), "error envelope: {body}");
    body["error"]["code"].as_str().unwrap().to_owned()
}

/// A shelf request one second after the previous one: signatures are single-use, and the
/// tests below send some identical requests on purpose.
async fn fresh(
    h: &Harness,
    method: Method,
    path: &str,
    body: impl Into<axum::body::Bytes>,
) -> crate::common::Reply {
    h.clock.advance(1);
    h.shelf(method, path, body).await
}

#[tokio::test]
async fn print_orders_forward_the_uploaded_card_to_the_print_provider() {
    let printer = Receiver::start(|request| {
        let text = String::from_utf8_lossy(&request.body);
        if text.contains("\"product\":\"card-a5\"") {
            let refusal = json!({ "error": { "code": "preflight_failed", "findings": [{ "code": "BOX_BLEED_TOO_SMALL", "page": 1 }] } });
            (StatusCode::UNPROCESSABLE_ENTITY, refusal.to_string().into_bytes())
        } else if text.contains("\"product\":\"card-a6\"") {
            let refusal = json!({ "error": { "code": "unsupported_country", "message": "this printer posts to FR only" } });
            (StatusCode::UNPROCESSABLE_ENTITY, refusal.to_string().into_bytes())
        } else if text.contains("\"product\":\"card-digest\"") {
            (StatusCode::BAD_REQUEST, b"<html>bad request</html>".to_vec())
        } else if text.contains("\"product\":\"card-gone\"") {
            (StatusCode::SERVICE_UNAVAILABLE, json!({ "error": { "code": "demo_full" } }).to_string().into_bytes())
        } else {
            (StatusCode::CREATED, json!({ "order_id": "po_17", "status": "accepted", "price_cents": 390 }).to_string().into_bytes())
        }
    })
    .await;
    let printer_url = printer.url("");
    let h = Harness::with(|c| c.printer_url = Some(printer_url));
    let gift = h.file_gift(json!({})).await;
    let id = gift["id"].as_str().unwrap();
    let card_path = format!("/api/partner/v1/gifts/{id}/card.pdf");
    let orders = format!("/api/partner/v1/gifts/{id}/print-orders");
    let address = json!({ "name": "Sacha Démo", "line1": "1 rue de l'Exemple", "postal_code": "75001", "city": "Paris", "country": "FR" });
    let pdf = b"%PDF-1.6\n% a card composed in the browser\n%%EOF\n".to_vec();
    let sha = ebc_server::codes::sha256_hex(&pdf);
    let order = |product: &str, sha: &str| {
        json!({ "product": product, "pdf_sha256": sha, "address": address }).to_string()
    };

    // No card yet; no signature; not a PDF.
    let early = fresh(&h, Method::POST, &orders, order("card-poche", &sha)).await;
    assert_eq!(
        (early.status, error_code(&early).as_str()),
        (StatusCode::CONFLICT, "card_pdf_missing")
    );
    assert_eq!(
        h.call(Method::PUT, &card_path, &[], pdf.clone()).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.call(Method::POST, &orders, &[], order("card-poche", &sha)).await.status,
        StatusCode::UNAUTHORIZED
    );
    // Another partner's gift is not found.
    let foreign = h.partner(Method::POST, &orders, order("card-poche", &sha), &[]).await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    let not_pdf = fresh(&h, Method::PUT, &card_path, "<html>").await;
    assert_eq!(not_pdf.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let put = fresh(&h, Method::PUT, &card_path, pdf.clone()).await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    assert_eq!(put.json()["sha256"], sha.as_str());

    // The order must name the PDF it is for.
    let mismatch = fresh(&h, Method::POST, &orders, order("card-poche", &"0".repeat(64))).await;
    assert_eq!(
        (mismatch.status, error_code(&mismatch).as_str()),
        (StatusCode::CONFLICT, "pdf_mismatch")
    );
    let bad = json!({ "product": "../x", "pdf_sha256": sha, "address": address });
    assert_eq!(
        fresh(&h, Method::POST, &orders, bad.to_string()).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    // A malformed address is a 400 naming the member (docs/api.md §4.6).
    for (bad_address, field) in [
        (json!("chez moi"), "address"),
        (json!({ "text": "1 rue de l'Exemple, 75001 Paris" }), "address.text"),
        (json!({ "name": "Sacha", "line1": "1 rue", "city": "Paris" }), "address.postal_code"),
        (
            json!({ "name": "Sacha", "line1": "1 rue", "postcode": "75001", "city": "Paris" }),
            "address.postcode",
        ),
        (
            json!({ "name": "Sacha", "line1": "1 rue", "postal_code": "75001", "city": "Paris", "country": "France" }),
            "address.country",
        ),
    ] {
        let body = json!({ "product": "card-poche", "pdf_sha256": sha, "address": bad_address });
        let reply = fresh(&h, Method::POST, &orders, body.to_string()).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::BAD_REQUEST, "invalid_address"),
            "{bad_address}"
        );
        assert_eq!(reply.json()["error"]["field"], field);
    }
    assert!(printer.requests().is_empty(), "nothing reaches the printer before the checks pass");

    let placed = fresh(&h, Method::POST, &orders, order("card-poche", &sha)).await;
    assert_eq!(placed.status, StatusCode::CREATED, "{}", placed.text());
    assert_eq!(
        placed.json(),
        json!({ "order_id": "po_17", "status": "accepted", "price_cents": 390, "currency": "EUR" })
    );

    // What the printer received: multipart/form-data with the parts `order` and `file` (§5).
    let request = printer.requests().pop().unwrap();
    assert_eq!((request.method.as_str(), request.path_and_query.as_str()), ("POST", "/v1/orders"));
    let content_type = request.headers["content-type"].to_str().unwrap();
    let boundary = content_type.strip_prefix("multipart/form-data; boundary=").unwrap();
    let body = String::from_utf8_lossy(&request.body).into_owned();
    let parts: Vec<&str> = body
        .split(&format!("--{boundary}"))
        .filter(|p| p.contains("Content-Disposition"))
        .collect();
    assert_eq!(parts.len(), 2, "{body}");
    let part =
        |name: &str| *parts.iter().find(|p| p.contains(&format!("name=\"{name}\""))).unwrap();
    let order_json: serde_json::Value =
        serde_json::from_str(part("order").split("\r\n\r\n").nth(1).unwrap().trim()).unwrap();
    assert_eq!(
        order_json,
        json!({ "product": "card-poche", "quantity": 1, "address": address, "reference": id })
    );
    assert!(
        part("file").contains("filename=\"card.pdf\"") && part("file").contains("application/pdf")
    );
    assert_eq!(
        part("file").split("\r\n\r\n").nth(1).unwrap().trim_end_matches("\r\n"),
        String::from_utf8(pdf).unwrap()
    );

    // The printer's preflight refuses: its findings are passed on.
    let refused = fresh(&h, Method::POST, &orders, order("card-a5", &sha)).await;
    assert_eq!(
        (refused.status, error_code(&refused).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "preflight_failed")
    );
    assert_eq!(refused.json()["error"]["findings"][0]["code"], "BOX_BLEED_TOO_SMALL");
    // Any other refusal of the printer is passed through with the printer's code.
    let rejected = fresh(&h, Method::POST, &orders, order("card-a6", &sha)).await;
    assert_eq!(
        (rejected.status, error_code(&rejected).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "printer_rejected")
    );
    let error = &rejected.json()["error"];
    assert_eq!(
        (&error["printer_code"], &error["printer_status"], &error["printer_message"]),
        (&json!("unsupported_country"), &json!(422), &json!("this printer posts to FR only"))
    );
    let rejected = fresh(&h, Method::POST, &orders, order("card-digest", &sha)).await;
    assert_eq!(
        (rejected.status, error_code(&rejected).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "printer_rejected")
    );
    assert_eq!(rejected.json()["error"]["printer_code"], "http_400");
    // A 5xx of the printer is still "unavailable".
    let full = fresh(&h, Method::POST, &orders, order("card-gone", &sha)).await;
    assert_eq!(
        (full.status, error_code(&full).as_str()),
        (StatusCode::BAD_GATEWAY, "printer_unavailable")
    );
    // The printer is down.
    printer.fail_first.store(1, std::sync::atomic::Ordering::SeqCst);
    let down = fresh(&h, Method::POST, &orders, order("card-poche", &sha)).await;
    assert_eq!(
        (down.status, error_code(&down).as_str()),
        (StatusCode::BAD_GATEWAY, "printer_unavailable")
    );

    // No print provider configured.
    let h = Harness::new();
    let gift = h.file_gift(json!({})).await;
    let orders = format!("/api/partner/v1/gifts/{}/print-orders", gift["id"].as_str().unwrap());
    let reply = fresh(&h, Method::POST, &orders, order("card-poche", &sha)).await;
    assert_eq!(
        (reply.status, error_code(&reply).as_str()),
        (StatusCode::SERVICE_UNAVAILABLE, "printing_not_configured")
    );
}

/// The whole path against the reference printer of `demo/printer` (docs/api.md §5): the
/// address shape the server documents is the one the printer takes, and its refusals keep
/// their codes.
#[tokio::test]
async fn print_orders_reach_the_reference_printer() {
    use ebc_demo_printer::testkit::SampleCard;
    use ebc_demo_printer::{PrinterConfig, PrinterState, catalog};

    let printer = PrinterState::new(PrinterConfig::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let printer_url = format!("http://{}", listener.local_addr().unwrap());
    let router = ebc_demo_printer::app_with_state(printer.clone());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let h = Harness::with(|c| c.printer_url = Some(printer_url));
    let gift = h.file_gift(json!({})).await;
    let id = gift["id"].as_str().unwrap();
    let orders = format!("/api/partner/v1/gifts/{id}/print-orders");
    let poche = catalog::find("card-poche").unwrap();
    let good = SampleCard::for_product(poche).to_pdf();
    let put =
        fresh(&h, Method::PUT, &format!("/api/partner/v1/gifts/{id}/card.pdf"), good.clone()).await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    let sha = ebc_server::codes::sha256_hex(&good);
    let order = |product: &str, address: serde_json::Value| {
        json!({ "product": product, "pdf_sha256": sha, "address": address }).to_string()
    };

    // What the web designer sends: no country (France by default), an empty second line.
    let address = json!({ "name": " Sacha Martin ", "line1": "12 rue des Lilas", "line2": "", "postal_code": "69003", "city": "Lyon" });
    let placed = fresh(&h, Method::POST, &orders, order("card-poche", address)).await;
    assert_eq!(placed.status, StatusCode::CREATED, "{}", placed.text());
    let receipt = placed.json();
    assert_eq!(receipt["status"], "accepted");
    assert!(receipt["price_cents"].as_u64().is_some_and(|p| p > 0), "{receipt}");
    let at_printer = printer.order(receipt["order_id"].as_str().unwrap()).unwrap();
    assert_eq!(at_printer.reference, id);
    assert_eq!(at_printer.pdf_sha256, sha);
    assert_eq!(
        (at_printer.address.name.as_str(), at_printer.address.line2.as_deref()),
        ("Sacha Martin", None)
    );
    assert_eq!(
        (at_printer.address.postal_code.as_str(), at_printer.address.city.as_str()),
        ("69003", "Lyon")
    );
    assert_eq!(at_printer.address.country, "FR");

    // A country the printer does not post to: its own code comes back, not a 502.
    let abroad = json!({ "name": "Sam", "line1": "1 Main Street", "postal_code": "10001", "city": "New York", "country": "us" });
    let refused = fresh(&h, Method::POST, &orders, order("card-poche", abroad)).await;
    assert_eq!(
        (refused.status, error_code(&refused).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "printer_rejected"),
        "{}",
        refused.text()
    );
    assert_eq!(refused.json()["error"]["printer_code"], "unsupported_country");

    // A product the printer does not sell.
    let address = json!({ "name": "Sacha", "line1": "1 rue", "postal_code": "75001", "city": "Paris", "country": "FR" });
    let unknown = fresh(&h, Method::POST, &orders, order("card-folio", address.clone())).await;
    assert_eq!(
        (unknown.status, unknown.json()["error"]["printer_code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("unknown_product"))
    );

    // The poche card ordered as A5: the printer's preflight findings are passed on.
    let wrong_format = fresh(&h, Method::POST, &orders, order("card-a5", address)).await;
    assert_eq!(
        (wrong_format.status, error_code(&wrong_format).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "preflight_failed"),
        "{}",
        wrong_format.text()
    );
    let findings = wrong_format.json()["error"]["findings"].as_array().cloned().unwrap();
    assert!(findings.iter().any(|f| f["severity"] == "error"), "{findings:#?}");
    assert_eq!(printer.orders().len(), 1, "only the good order was registered");
}

#[tokio::test]
async fn the_file_backed_store_survives_a_restart_and_holds_no_book() {
    let dir = tempfile::tempdir().unwrap();
    let shop = start_shop().await;
    let (code, session) = {
        let h = Harness::with_store(Arc::new(FileStore::open(dir.path()).unwrap()), |_| {});
        let gift = h.file_gift_at(&shop, "/ebc/file", json!({})).await;
        let code = gift["code"].as_str().unwrap().to_owned();
        let session = h.post_form(&format!("/k/{code}/download"), &[]).await.location();
        (code, session)
    };
    // On disk: one snapshot, no file at all (the partner serves the book), no clear code.
    let snapshot = std::fs::read_to_string(dir.path().join("db.json")).unwrap();
    assert!(
        !snapshot.contains(&code) && !snapshot.contains(&GiftCode::parse(&code).unwrap().grouped())
    );
    assert!(!snapshot.contains("/dl/"), "the partner's download link is not persisted");
    assert_eq!(std::fs::read_dir(dir.path().join("files")).unwrap().count(), 0);

    // A new process on the same directory: the gift, its claim count and the live
    // download session are all still there; the link is asked for again.
    let h = Harness::with_store(Arc::new(FileStore::open(dir.path()).unwrap()), |_| {});
    let page = h.get(&format!("/k/{code}")).await;
    assert_page(&page, StatusCode::OK, "fr", &["gift-title", "download", "claim-count"]);
    assert!(page.text_of("claim-count").contains("1 fois"));
    let file = h.get(&session).await;
    assert!(file.location().starts_with(&shop.url("/dl/")));
    let head = h.call(Method::HEAD, &session, &[], Body::empty()).await;
    assert_eq!(head.status, StatusCode::SEE_OTHER);
    assert_eq!(shop.requests().iter().filter(|r| r.path_and_query.starts_with("/ebc/")).count(), 2);
}

#[tokio::test]
async fn expired_gifts_are_anonymised_and_their_files_deleted_after_the_retention_period() {
    let store = Arc::new(MemoryStore::new());
    let h = Harness::with_store(store.clone(), |c| c.retention_secs = 30 * 86_400);
    let created = h.file_gift(json!({ "message": "Pour toi", "expires_at": T0 + 3_600 })).await;
    let (id, code) =
        (created["id"].as_str().unwrap().to_owned(), created["code"].as_str().unwrap().to_owned());
    let png = [&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a][..], &[7u8; 64][..]].concat();
    let sha256 = ebc_server::codes::sha256_hex(&png);
    let cover = fresh(&h, Method::PUT, &format!("/api/partner/v1/gifts/{id}/cover"), png).await;
    assert_eq!(cover.status, StatusCode::OK, "{}", cover.text());
    let gift = |h: &Harness| h.state.db.view(|db| db.gifts.get(&id).cloned()).unwrap();
    // Expired, within the retention period: kept as it is.
    h.clock.advance(3_601);
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);
    let _ = h.file_gift(json!({})).await; // a write runs the purge
    assert_eq!(gift(&h).giver_name.as_deref(), Some("Camille"));
    assert!(store.open_blob(&sha256).unwrap().is_some());
    // 30 days after the expiry: anonymised, cover gone, the code still 410.
    h.clock.advance(30 * 86_400);
    let _ = h.file_gift(json!({})).await;
    let record = gift(&h);
    assert!(record.anonymised_at.is_some());
    assert!(
        record.giver_name.is_none() && record.recipient_name.is_none() && record.message.is_none()
    );
    assert!(
        store.open_blob(&sha256).unwrap().is_none(),
        "the cover of the expired gift is deleted"
    );
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);
}

#[tokio::test]
async fn the_static_web_app_is_served_under_app_with_its_own_policy() {
    let root = tempfile::tempdir().unwrap();
    let web = root.path().join("web");
    std::fs::create_dir_all(web.join("pkg")).unwrap();
    std::fs::write(
        web.join("index.html"),
        "<!doctype html><title>eBook cadeau</title><script type=module src=main.js></script>",
    )
    .unwrap();
    std::fs::write(web.join("main.js"), "export {};").unwrap();
    std::fs::write(web.join("pkg/ebc_wasm_bg.wasm"), b"\0asm\x01\0\0\0").unwrap();
    std::fs::write(root.path().join("secret-outside.txt"), "secret").unwrap();
    let web_dir = web.clone();
    let h = Harness::with(|c| c.web_dir = Some(web_dir));

    let index = h.get("/app/").await;
    assert_eq!(index.status, StatusCode::OK);
    assert!(index.header("content-type").starts_with("text/html"));
    assert!(index.text().contains("<title>eBook cadeau</title>"));
    assert_eq!(h.get("/app/index.html").await.status, StatusCode::OK);
    let wasm = h.get("/app/pkg/ebc_wasm_bg.wasm").await;
    assert_eq!((wasm.status, wasm.header("content-type")), (StatusCode::OK, "application/wasm"));
    assert_eq!(wasm.body.as_ref(), b"\0asm\x01\0\0\0");
    assert!(h.get("/app/main.js").await.header("content-type").contains("javascript"));

    // The app may run its own scripts and WebAssembly; the recipient pages may not.
    let csp = index.header("content-security-policy");
    assert!(
        csp.contains("script-src 'self' 'wasm-unsafe-eval'")
            && csp.contains("frame-ancestors 'none'"),
        "{csp}"
    );
    assert_eq!(index.header("x-content-type-options"), "nosniff");
    assert!(!h.get("/").await.header("content-security-policy").contains("script-src"));

    // Nothing outside the directory.
    for path in [
        "/app/../secret-outside.txt",
        "/app/..%2Fsecret-outside.txt",
        "/app/%2e%2e/secret-outside.txt",
        "/app/pkg/../../secret-outside.txt",
        "/app/missing.js",
    ] {
        let reply = h.get(path).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
        assert!(!reply.text().contains("secret"), "{path}");
    }

    // Not configured: no /app/.
    assert_eq!(Harness::new().get("/app/").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn request_bodies_share_an_in_flight_budget_and_stalled_bodies_time_out() {
    use tokio::io::AsyncWriteExt;
    let h = Harness::with(|c| {
        c.max_cover_bytes = 100_000;
        c.max_card_pdf_bytes = 1_000;
        c.max_inflight_body_bytes = 0; // the floor: one largest body (100 000 bytes)
        c.body_idle_timeout_secs = 1;
    });
    // A client that sends 80 000 bytes, then stalls.
    let (mut tx, rx) = tokio::io::duplex(128 * 1024);
    tx.write_all(&vec![b'x'; 80_000]).await.unwrap();
    // Well-formed partner headers: the body is read before the MAC can be checked.
    let signature = format!("v1={}", "0".repeat(64));
    let t0 = T0.to_string();
    let signed = [
        ("x-ebc-partner", crate::common::PARTNER_ID),
        ("x-ebc-timestamp", t0.as_str()),
        ("x-ebc-signature", signature.as_str()),
    ];
    let mut stalled = axum::http::Request::builder()
        .method(Method::PUT)
        .uri(COVER)
        .header("x-forwarded-for", "192.0.2.1");
    for (name, value) in signed {
        stalled = stalled.header(name, value);
    }
    let stalled = stalled.body(Body::from_stream(tokio_util::io::ReaderStream::new(rx))).unwrap();
    let app = h.app.clone();
    let first = tokio::spawn(async move {
        use tower::ServiceExt;
        app.oneshot(stalled).await.unwrap()
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    // Its bytes are held: another body that does not fit is refused at once.
    let busy = h
        .call(
            Method::PUT,
            COVER,
            &[signed[0], signed[1], signed[2], ("x-forwarded-for", "192.0.2.2")],
            vec![b'y'; 30_000],
        )
        .await;
    assert_eq!(
        (busy.status, error_code(&busy).as_str()),
        (StatusCode::SERVICE_UNAVAILABLE, "server_busy")
    );
    assert!(busy.has_header("retry-after"));
    // The stalled body is cut off after the idle time-out, which frees the budget.
    let first = first.await.unwrap();
    assert_eq!(first.status(), StatusCode::REQUEST_TIMEOUT);
    drop(tx);
    let read = h
        .call(
            Method::PUT,
            COVER,
            &[signed[0], signed[1], signed[2], ("x-forwarded-for", "192.0.2.3")],
            vec![b'y'; 30_000],
        )
        .await;
    assert_eq!(
        (read.status, error_code(&read).as_str()),
        (StatusCode::UNAUTHORIZED, "bad_signature"),
        "read in full, then refused by its MAC"
    );
}
