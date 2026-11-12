//! The creator API (`docs/api.md` §3), the print-order forwarding (§5), the file-backed
//! store and the static web app (§6).

use std::sync::Arc;

use crate::common::{CONTAINER_XML, Harness, Receiver, T0, assert_page, epub, epub_with, zip_of};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use ebc_model::GiftCode;
use ebc_server::{FileStore, MemoryStore, Store};
use serde_json::json;

const UPLOADS: &str = "/api/v1/uploads";

fn error_code(reply: &crate::common::Reply) -> String {
    let body = reply.json();
    assert!(body["error"]["message"].is_string(), "error envelope: {body}");
    body["error"]["code"].as_str().unwrap().to_owned()
}

async fn upload(h: &Harness, bytes: Vec<u8>) -> crate::common::Reply {
    h.call(
        Method::POST,
        UPLOADS,
        &[("x-rights-basis", "own_work"), ("content-type", "application/epub+zip")],
        bytes,
    )
    .await
}

#[tokio::test]
async fn uploads_are_sniffed_and_drm_is_refused_whatever_the_headers_say() {
    let h = Harness::new();
    let book = epub();
    let ok = upload(&h, book.clone()).await;
    assert_eq!(ok.status, StatusCode::CREATED, "{}", ok.text());
    let receipt = ok.json();
    assert!(receipt["upload_id"].as_str().unwrap().starts_with("u_"));
    assert_eq!(receipt["sha256"], ebc_server::codes::sha256_hex(&book));
    assert_eq!(receipt["size_bytes"], book.len());
    assert_eq!(receipt["media_type"], "application/epub+zip");

    // A PDF is recognised from its bytes, even announced as something else.
    let pdf = h
        .call(
            Method::POST,
            UPLOADS,
            &[("x-rights-basis", "public_domain"), ("content-type", "text/plain")],
            &b"%PDF-1.7\n%%EOF\n"[..],
        )
        .await;
    assert_eq!(
        (pdf.status, pdf.json()["media_type"].as_str()),
        (StatusCode::CREATED, Some("application/pdf"))
    );

    // Anything else is refused, even announced as an EPUB.
    let not_books: Vec<Vec<u8>> = vec![
        b"just some text".to_vec(),
        b"<html><script>alert(1)</script></html>".to_vec(),
        zip_of(&[("hello.txt", b"hello", true)]),
        // An OCF-looking archive whose mimetype says something else.
        zip_of(&[
            ("mimetype", b"application/x-msdownload", false),
            ("META-INF/container.xml", CONTAINER_XML.as_bytes(), true),
        ]),
        // mimetype not first.
        zip_of(&[
            ("META-INF/container.xml", CONTAINER_XML.as_bytes(), true),
            ("mimetype", b"application/epub+zip", false),
        ]),
        b"PK\x03\x04".to_vec(),
        vec![0xff; 4096],
    ];
    for bytes in not_books {
        let reply = upload(&h, bytes).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type")
        );
    }
    let empty = upload(&h, Vec::new()).await;
    assert_eq!(
        (empty.status, error_code(&empty).as_str()),
        (StatusCode::BAD_REQUEST, "empty_body")
    );
    // A truncated EPUB is not accepted as a file we would hand to a recipient.
    let truncated = upload(&h, book[..book.len() / 2].to_vec()).await;
    assert_eq!(truncated.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    // DRM: Adobe rights.xml, an LCP licence, or real encryption of the content.
    let aes = br#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#">
      <enc:EncryptedData><enc:EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes256-cbc"/>
      <enc:CipherData><enc:CipherReference URI="OEBPS/chapter-1.xhtml"/></enc:CipherData></enc:EncryptedData></encryption>"#;
    let protected: [Vec<u8>; 4] = [
        epub_with(&[("META-INF/rights.xml", b"<rights/>")]),
        epub_with(&[("META-INF/license.lcpl", b"{}")]),
        epub_with(&[("META-INF/encryption.xml", aes)]),
        epub_with(&[("META-INF/encryption.xml", b"not xml at all")]),
    ];
    for bytes in protected {
        let reply = upload(&h, bytes).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::UNPROCESSABLE_ENTITY, "drm_protected")
        );
    }
    // Font obfuscation is not DRM.
    let fonts = br#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#">
      <enc:EncryptedData><enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/>
      <enc:CipherData><enc:CipherReference URI="OEBPS/font.otf"/></enc:CipherData></enc:EncryptedData></encryption>"#;
    assert_eq!(
        upload(&h, epub_with(&[("META-INF/encryption.xml", fonts)])).await.status,
        StatusCode::CREATED
    );

    // The rights declaration is mandatory and closed.
    let undeclared = h.call(Method::POST, UPLOADS, &[], epub()).await;
    assert_eq!(
        (undeclared.status, error_code(&undeclared).as_str()),
        (StatusCode::BAD_REQUEST, "missing_rights_basis")
    );
    for basis in ["bought_it", "OWN_WORK", "own_work, public_domain"] {
        let reply = h.call(Method::POST, UPLOADS, &[("x-rights-basis", basis)], epub()).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::BAD_REQUEST, "invalid_rights_basis"),
            "{basis}"
        );
    }
    for basis in ["own_work", "public_domain", "open_licence", "licensed_distributor"] {
        assert_eq!(
            h.call(Method::POST, UPLOADS, &[("x-rights-basis", basis)], epub()).await.status,
            StatusCode::CREATED,
            "{basis}"
        );
    }
}

#[tokio::test]
async fn oversized_bodies_are_413_and_open_endpoints_are_rate_limited() {
    let h = Harness::with(|c| {
        c.max_upload_bytes = 2_000;
        c.rate_create_per_10min = 4;
    });
    let from = |client: &'static str| [("x-rights-basis", "own_work"), ("x-forwarded-for", client)];
    // Too large, whether or not a Content-Length announces it.
    let big = vec![b'x'; 2_001];
    let announced = h
        .call(
            Method::POST,
            UPLOADS,
            &[
                ("x-rights-basis", "own_work"),
                ("x-forwarded-for", "192.0.2.1"),
                ("content-length", "2001"),
            ],
            big.clone(),
        )
        .await;
    assert_eq!(
        (announced.status, error_code(&announced).as_str()),
        (StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large")
    );
    let silent = h.call(Method::POST, UPLOADS, &from("192.0.2.1"), big).await;
    assert_eq!(silent.status, StatusCode::PAYLOAD_TOO_LARGE);
    let json_bomb = format!("{{\"book\":{{\"title\":\"{}\"}}}}", "x".repeat(70_000));
    let reply =
        h.call(Method::POST, "/api/v1/gifts", &[("x-forwarded-for", "192.0.2.1")], json_bomb).await;
    assert_eq!(reply.status, StatusCode::PAYLOAD_TOO_LARGE);

    // Four creations per client and ten minutes, uploads and gifts together.
    let mut statuses = Vec::new();
    for _ in 0..3 {
        statuses.push(h.call(Method::POST, UPLOADS, &from("192.0.2.1"), epub()).await.status);
    }
    assert_eq!(
        statuses,
        [StatusCode::CREATED, StatusCode::TOO_MANY_REQUESTS, StatusCode::TOO_MANY_REQUESTS],
        "three earlier requests counted"
    );
    let limited =
        h.call(Method::POST, "/api/v1/gifts", &[("x-forwarded-for", "192.0.2.1")], "{}").await;
    assert_eq!(
        (limited.status, error_code(&limited).as_str()),
        (StatusCode::TOO_MANY_REQUESTS, "rate_limited")
    );
    assert!((1..=600).contains(&limited.header("retry-after").parse::<u64>().unwrap()));
    assert_eq!(
        h.call(Method::POST, UPLOADS, &from("198.51.100.1"), epub()).await.status,
        StatusCode::CREATED
    );
    h.clock.advance(600);
    assert_eq!(
        h.call(Method::POST, UPLOADS, &from("192.0.2.1"), epub()).await.status,
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn gifts_are_created_validated_and_managed_with_the_manage_token() {
    let h = Harness::new();
    let gift = h.hosted_gift(json!({})).await;
    let id = gift["id"].as_str().unwrap();
    let code = gift["code"].as_str().unwrap();
    assert!(id.starts_with("g_"));
    let parsed = GiftCode::parse(code).unwrap();
    assert_eq!(code.len(), 12);
    assert_eq!(gift["code_display"], parsed.grouped());
    assert_eq!(gift["claim_url"], format!("https://ebk.example/k/{code}"));
    let token = gift["manage_token"].as_str().unwrap();
    assert!(token.len() >= 32);

    let path = format!("/api/v1/gifts/{id}");
    let bearer = format!("Bearer {token}");
    let doc =
        h.call(Method::GET, &path, &[("authorization", bearer.as_str())], Body::empty()).await;
    assert_eq!(doc.status, StatusCode::OK);
    assert_eq!(
        doc.json(),
        json!({ "id": id, "status": "created", "created_at": T0, "redeemed_at": null, "downloads": 0 })
    );

    // The token is the only way in.
    let missing = h.get(&path).await;
    assert_eq!(
        (missing.status, error_code(&missing).as_str()),
        (StatusCode::UNAUTHORIZED, "missing_token")
    );
    for wrong in [
        "Bearer nope",
        "Bearer ",
        &format!("Basic {token}"),
        &format!("Bearer {token}x"),
        &format!("Bearer {code}"),
    ] {
        for (method, target) in [
            (Method::GET, path.clone()),
            (Method::DELETE, path.clone()),
            (Method::POST, format!("{path}/reissue")),
        ] {
            let reply = h.call(method, &target, &[("authorization", wrong)], Body::empty()).await;
            assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{wrong}");
        }
    }
    let other = h.hosted_gift(json!({})).await;
    let cross = h
        .call(
            Method::GET,
            &format!("/api/v1/gifts/{}", other["id"].as_str().unwrap()),
            &[("authorization", bearer.as_str())],
            Body::empty(),
        )
        .await;
    assert_eq!(cross.status, StatusCode::UNAUTHORIZED, "a token opens its own gift only");
    let unknown = h
        .call(
            Method::GET,
            "/api/v1/gifts/g_00000000000000000000000000000000",
            &[("authorization", bearer.as_str())],
            Body::empty(),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(
        h.call(Method::GET, &path, &[("authorization", bearer.as_str())], Body::empty())
            .await
            .json()["status"],
        "created"
    );

    // Neither the code nor the token is stored in clear.
    let stored = h.state.db.view(|db| serde_json::to_string(db).unwrap());
    assert!(
        !stored.contains(code) && !stored.contains(&parsed.grouped()) && !stored.contains(token)
    );
    assert!(stored.contains("public_domain"), "the rights declaration is recorded with the gift");

    // Requests that cannot become a gift.
    let upload_id = h.upload(epub(), "x.epub").await;
    let delivery = json!({ "type": "hosted_file", "upload_id": upload_id });
    let post = |body: String| h.call(Method::POST, "/api/v1/gifts", &[], body);
    let cases: Vec<(String, StatusCode, &str)> = vec![
        ("{ not json".to_owned(), StatusCode::BAD_REQUEST, "invalid_json"),
        (String::new(), StatusCode::BAD_REQUEST, "invalid_json"),
        (json!({ "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_request"),
        (json!({ "book": { "title": "x", "isbn13": "9782070360025" }, "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_request"),
        (json!({ "book": { "title": "x" }, "delivery": { "type": "carrier_pigeon" } }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_request"),
        (json!({ "book": { "title": "   " }, "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_book"),
        (json!({ "book": { "title": "x".repeat(301) }, "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_book"),
        (json!({ "book": { "title": "x" }, "locale": "tlh", "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_locale"),
        (json!({ "book": { "title": "x" }, "message": "m".repeat(401), "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_gift"),
        (json!({ "book": { "title": "x" }, "giver_name": "a\u{7}b", "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_gift"),
        (json!({ "book": { "title": "x" }, "expires_at": T0 - 1, "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_gift"),
        (json!({ "book": { "title": "x" }, "reveal_at": T0 + 10, "expires_at": T0 + 5, "delivery": delivery }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "invalid_gift"),
        (json!({ "book": { "title": "x" }, "delivery": { "type": "hosted_file", "upload_id": "u_nope" } }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "unknown_upload"),
        (json!({ "book": { "title": "x" }, "delivery": { "type": "hosted_file", "upload_id": "../../etc/passwd" } }).to_string(), StatusCode::UNPROCESSABLE_ENTITY, "unknown_upload"),
    ];
    for (body, status, code) in cases {
        let reply = post(body.clone()).await;
        assert_eq!((reply.status, error_code(&reply).as_str()), (status, code), "{body}");
    }
    // An upload backs one gift.
    let body = json!({ "book": { "title": "x" }, "delivery": delivery }).to_string();
    assert_eq!(post(body.clone()).await.status, StatusCode::CREATED);
    let reused = post(body).await;
    assert_eq!(
        (reused.status, error_code(&reused).as_str()),
        (StatusCode::CONFLICT, "upload_already_used")
    );
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
    let gift = h.hosted_gift(json!({})).await;
    let id = gift["id"].as_str().unwrap();
    let bearer = format!("Bearer {}", gift["manage_token"].as_str().unwrap());
    let auth = [("authorization", bearer.as_str())];
    let card_path = format!("/api/v1/gifts/{id}/card.pdf");
    let orders = format!("/api/v1/gifts/{id}/print-orders");
    let address = json!({ "name": "Sacha Démo", "line1": "1 rue de l'Exemple", "postal_code": "75001", "city": "Paris", "country": "FR" });
    let pdf = b"%PDF-1.6\n% a card composed in the browser\n%%EOF\n".to_vec();
    let sha = ebc_server::codes::sha256_hex(&pdf);
    let order = |product: &str, sha: &str| {
        json!({ "product": product, "pdf_sha256": sha, "address": address }).to_string()
    };

    // No card yet; no token; not a PDF.
    let early = h.call(Method::POST, &orders, &auth, order("card-poche", &sha)).await;
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
    let not_pdf = h.call(Method::PUT, &card_path, &auth, "<html>").await;
    assert_eq!(not_pdf.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let put = h.call(Method::PUT, &card_path, &auth, pdf.clone()).await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    assert_eq!(put.json()["sha256"], sha.as_str());

    // The order must name the PDF it is for.
    let mismatch = h.call(Method::POST, &orders, &auth, order("card-poche", &"0".repeat(64))).await;
    assert_eq!(
        (mismatch.status, error_code(&mismatch).as_str()),
        (StatusCode::CONFLICT, "pdf_mismatch")
    );
    let bad = json!({ "product": "../x", "pdf_sha256": sha, "address": address });
    assert_eq!(
        h.call(Method::POST, &orders, &auth, bad.to_string()).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    // A malformed address is a 400 naming the member (docs/api.md §3).
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
        let reply = h.call(Method::POST, &orders, &auth, body.to_string()).await;
        assert_eq!(
            (reply.status, error_code(&reply).as_str()),
            (StatusCode::BAD_REQUEST, "invalid_address"),
            "{bad_address}"
        );
        assert_eq!(reply.json()["error"]["field"], field);
    }
    assert!(printer.requests().is_empty(), "nothing reaches the printer before the checks pass");

    let placed = h.call(Method::POST, &orders, &auth, order("card-poche", &sha)).await;
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
    let refused = h.call(Method::POST, &orders, &auth, order("card-a5", &sha)).await;
    assert_eq!(
        (refused.status, error_code(&refused).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "preflight_failed")
    );
    assert_eq!(refused.json()["error"]["findings"][0]["code"], "BOX_BLEED_TOO_SMALL");
    // Any other refusal of the printer is passed through with the printer's code.
    let rejected = h.call(Method::POST, &orders, &auth, order("card-a6", &sha)).await;
    assert_eq!(
        (rejected.status, error_code(&rejected).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "printer_rejected")
    );
    let error = &rejected.json()["error"];
    assert_eq!(
        (&error["printer_code"], &error["printer_status"], &error["printer_message"]),
        (&json!("unsupported_country"), &json!(422), &json!("this printer posts to FR only"))
    );
    let rejected = h.call(Method::POST, &orders, &auth, order("card-digest", &sha)).await;
    assert_eq!(
        (rejected.status, error_code(&rejected).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "printer_rejected")
    );
    assert_eq!(rejected.json()["error"]["printer_code"], "http_400");
    // A 5xx of the printer is still "unavailable".
    let full = h.call(Method::POST, &orders, &auth, order("card-gone", &sha)).await;
    assert_eq!(
        (full.status, error_code(&full).as_str()),
        (StatusCode::BAD_GATEWAY, "printer_unavailable")
    );
    // The printer is down.
    printer.fail_first.store(1, std::sync::atomic::Ordering::SeqCst);
    let down = h.call(Method::POST, &orders, &auth, order("card-poche", &sha)).await;
    assert_eq!(
        (down.status, error_code(&down).as_str()),
        (StatusCode::BAD_GATEWAY, "printer_unavailable")
    );

    // No print provider configured.
    let h = Harness::new();
    let gift = h.hosted_gift(json!({})).await;
    let bearer = format!("Bearer {}", gift["manage_token"].as_str().unwrap());
    let orders = format!("/api/v1/gifts/{}/print-orders", gift["id"].as_str().unwrap());
    let reply = h
        .call(
            Method::POST,
            &orders,
            &[("authorization", bearer.as_str())],
            order("card-poche", &sha),
        )
        .await;
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
    let gift = h.hosted_gift(json!({})).await;
    let id = gift["id"].as_str().unwrap();
    let bearer = format!("Bearer {}", gift["manage_token"].as_str().unwrap());
    let auth = [("authorization", bearer.as_str())];
    let orders = format!("/api/v1/gifts/{id}/print-orders");
    let poche = catalog::find("card-poche").unwrap();
    let good = SampleCard::for_product(poche).to_pdf();
    let put =
        h.call(Method::PUT, &format!("/api/v1/gifts/{id}/card.pdf"), &auth, good.clone()).await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    let sha = ebc_server::codes::sha256_hex(&good);
    let order = |product: &str, address: serde_json::Value| {
        json!({ "product": product, "pdf_sha256": sha, "address": address }).to_string()
    };

    // What the web designer sends: no country (France by default), an empty second line.
    let address = json!({ "name": " Sacha Martin ", "line1": "12 rue des Lilas", "line2": "", "postal_code": "69003", "city": "Lyon" });
    let placed = h.call(Method::POST, &orders, &auth, order("card-poche", address)).await;
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
    let refused = h.call(Method::POST, &orders, &auth, order("card-poche", abroad)).await;
    assert_eq!(
        (refused.status, error_code(&refused).as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, "printer_rejected"),
        "{}",
        refused.text()
    );
    assert_eq!(refused.json()["error"]["printer_code"], "unsupported_country");

    // A product the printer does not sell.
    let address = json!({ "name": "Sacha", "line1": "1 rue", "postal_code": "75001", "city": "Paris", "country": "FR" });
    let unknown = h.call(Method::POST, &orders, &auth, order("card-folio", address.clone())).await;
    assert_eq!(
        (unknown.status, unknown.json()["error"]["printer_code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("unknown_product"))
    );

    // The poche card ordered as A5: the printer's preflight findings are passed on.
    let wrong_format = h.call(Method::POST, &orders, &auth, order("card-a5", address)).await;
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
async fn the_file_backed_store_survives_a_restart_and_serves_ranges_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let book = epub();
    let (code, session) = {
        let h = Harness::with_store(Arc::new(FileStore::open(dir.path()).unwrap()), |_| {});
        let gift = h.hosted_gift(json!({})).await;
        let code = gift["code"].as_str().unwrap().to_owned();
        let session = h.post_form(&format!("/k/{code}/download"), &[]).await.location();
        (code, session)
    };
    // On disk: one snapshot, one file named by its hash; no clear code anywhere.
    let snapshot = std::fs::read_to_string(dir.path().join("db.json")).unwrap();
    assert!(
        !snapshot.contains(&code) && !snapshot.contains(&GiftCode::parse(&code).unwrap().grouped())
    );
    let files: Vec<String> = std::fs::read_dir(dir.path().join("files"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(files, [ebc_server::codes::sha256_hex(&book)]);

    // A new process on the same directory: the gift, its claim count and the live
    // download session are all still there.
    let h = Harness::with_store(Arc::new(FileStore::open(dir.path()).unwrap()), |_| {});
    let page = h.get(&format!("/k/{code}")).await;
    assert_page(&page, StatusCode::OK, "fr", &["gift-title", "download", "claim-count"]);
    assert!(page.text_of("claim-count").contains("1 fois"));
    let file = h.get(&session).await;
    assert_eq!((file.status, file.body.as_ref()), (StatusCode::OK, book.as_slice()));
    assert_eq!(file.header("content-length"), book.len().to_string());
    let part = h.call(Method::GET, &session, &[("range", "bytes=30-49")], Body::empty()).await;
    assert_eq!((part.status, part.body.as_ref()), (StatusCode::PARTIAL_CONTENT, &book[30..50]));
    assert_eq!(part.header("content-length"), "20");
    let head = h.call(Method::HEAD, &session, &[], Body::empty()).await;
    assert_eq!(
        (head.status, head.body.len(), head.header("content-length")),
        (StatusCode::OK, 0, book.len().to_string().as_str())
    );
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
        c.max_upload_bytes = 100_000;
        c.max_cover_bytes = 1_000;
        c.max_card_pdf_bytes = 1_000;
        c.max_inflight_body_bytes = 0; // the floor: one largest body (100 000 bytes)
        c.body_idle_timeout_secs = 1;
    });
    // A client that sends 80 000 bytes, then stalls.
    let (mut tx, rx) = tokio::io::duplex(128 * 1024);
    tx.write_all(&vec![b'x'; 80_000]).await.unwrap();
    let stalled = axum::http::Request::builder()
        .method(Method::POST)
        .uri(UPLOADS)
        .header("x-rights-basis", "own_work")
        .header("x-forwarded-for", "192.0.2.1")
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(rx)))
        .unwrap();
    let app = h.app.clone();
    let first = tokio::spawn(async move {
        use tower::ServiceExt;
        app.oneshot(stalled).await.unwrap()
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    // Its bytes are held: another body that does not fit is refused at once.
    let busy = h
        .call(
            Method::POST,
            UPLOADS,
            &[("x-rights-basis", "own_work"), ("x-forwarded-for", "192.0.2.2")],
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
    let ok = h
        .call(
            Method::POST,
            UPLOADS,
            &[("x-rights-basis", "own_work"), ("x-forwarded-for", "192.0.2.3")],
            epub(),
        )
        .await;
    assert_eq!(ok.status, StatusCode::CREATED, "{}", ok.text());
}

#[tokio::test]
async fn anonymous_uploads_have_a_storage_quota_and_a_short_life() {
    let size = epub().len() as u64;
    let h = Harness::with(|c| {
        c.anonymous_storage_quota_bytes = size * 2;
        c.anonymous_upload_orphan_ttl_secs = 3_600;
    });
    let up = |client: &'static str| {
        let h = &h;
        async move {
            h.call(
                Method::POST,
                UPLOADS,
                &[("x-rights-basis", "own_work"), ("x-forwarded-for", client)],
                epub(),
            )
            .await
        }
    };
    let first = up("192.0.2.1").await;
    assert_eq!(first.status, StatusCode::CREATED);
    assert_eq!(up("192.0.2.2").await.status, StatusCode::CREATED);
    let full = up("192.0.2.3").await;
    assert_eq!(
        (full.status, error_code(&full).as_str()),
        (StatusCode::INSUFFICIENT_STORAGE, "storage_quota_exceeded")
    );
    // Unattached anonymous uploads are dropped after an hour (partner ones after a day),
    // which frees the quota.
    h.clock.advance(3_601);
    assert_eq!(up("192.0.2.3").await.status, StatusCode::CREATED);
    let upload_id = first.json()["upload_id"].as_str().unwrap().to_owned();
    let late = json!({ "book": { "title": "x" }, "delivery": { "type": "hosted_file", "upload_id": upload_id } });
    let reply = h.call(Method::POST, "/api/v1/gifts", &[], late.to_string()).await;
    assert_eq!(error_code(&reply), "unknown_upload");
}

#[tokio::test]
async fn the_open_creation_api_has_a_global_budget() {
    let h = Harness::with(|c| {
        c.rate_create_per_10min = 100;
        c.rate_create_global_per_10min = 3;
    });
    let create = |client: &'static str| {
        let h = &h;
        async move { h.call(Method::POST, "/api/v1/gifts", &[("x-forwarded-for", client)], "{}").await }
    };
    for client in ["192.0.2.1", "192.0.2.2", "192.0.2.3"] {
        assert_ne!(create(client).await.status, StatusCode::TOO_MANY_REQUESTS);
    }
    let refused = create("198.51.100.1").await;
    assert_eq!(
        (refused.status, error_code(&refused).as_str()),
        (StatusCode::TOO_MANY_REQUESTS, "rate_limited")
    );
    h.clock.advance(600);
    assert_ne!(create("198.51.100.1").await.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn expired_gifts_are_anonymised_and_their_files_deleted_after_the_retention_period() {
    let store = Arc::new(MemoryStore::new());
    let h = Harness::with_store(store.clone(), |c| c.retention_secs = 30 * 86_400);
    let book = epub_with(&[("OEBPS/only-this-gift.xhtml", b"<html/>")]);
    let sha256 = ebc_server::codes::sha256_hex(&book);
    let upload_id = h.upload(book, "a.epub").await;
    let body = json!({
        "book": { "title": "Le Horla" }, "giver_name": "Camille", "recipient_name": "Sacha",
        "message": "Pour toi", "expires_at": T0 + 3_600,
        "delivery": { "type": "hosted_file", "upload_id": upload_id },
    });
    let created = h.call(Method::POST, "/api/v1/gifts", &[], body.to_string()).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let (id, code) = (
        created.json()["id"].as_str().unwrap().to_owned(),
        created.json()["code"].as_str().unwrap().to_owned(),
    );
    let gift = |h: &Harness| h.state.db.view(|db| db.gifts.get(&id).cloned()).unwrap();
    // Expired, within the retention period: kept as it is.
    h.clock.advance(3_601);
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);
    let _ = h.hosted_gift(json!({})).await;
    assert_eq!(gift(&h).giver_name.as_deref(), Some("Camille"));
    assert!(store.open_blob(&sha256).unwrap().is_some());
    // 30 days after the expiry: anonymised, file and upload record gone, the code still 410.
    h.clock.advance(30 * 86_400);
    let _ = h.hosted_gift(json!({})).await;
    let record = gift(&h);
    assert!(record.anonymised_at.is_some());
    assert!(
        record.giver_name.is_none() && record.recipient_name.is_none() && record.message.is_none()
    );
    assert!(store.open_blob(&sha256).unwrap().is_none(), "the file of the expired gift is deleted");
    assert!(h.state.db.view(|db| !db.uploads.contains_key(&upload_id)));
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);
}
