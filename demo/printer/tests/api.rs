//! The print provider API over real HTTP, in-process on an ephemeral port. Offline: the
//! only network traffic is on the loopback interface.

#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use ebc_demo_printer::testkit::SampleCard;
use ebc_demo_printer::{PrinterConfig, PrinterState, app_with_state, catalog};
use reqwest::multipart::{Form, Part};
use serde_json::{Value, json};

// ----- harness ----------------------------------------------------------------------------

struct Printer {
    base: String,
    state: Arc<PrinterState>,
    http: reqwest::Client,
}

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

async fn printer(config: PrinterConfig) -> Printer {
    let state = PrinterState::new(config);
    let base = serve(app_with_state(state.clone())).await;
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    Printer { base, state, http }
}

fn poche_pdf() -> Vec<u8> {
    SampleCard::for_product(catalog::find("card-poche").unwrap()).to_pdf()
}

fn order_json(reference: &str) -> Value {
    json!({
        "product": "card-poche",
        "quantity": 1,
        "reference": reference,
        "address": {
            "name": "Sacha Martin", "line1": "12 rue des Lilas",
            "postal_code": "75011", "city": "Paris", "country": "FR"
        },
    })
}

impl Printer {
    async fn submit(&self, order: &Value, pdf: Vec<u8>) -> (StatusCode, Value) {
        let form = Form::new()
            .part("order", Part::text(order.to_string()).mime_str("application/json").unwrap())
            .part(
                "file",
                Part::bytes(pdf).file_name("card.pdf").mime_str("application/pdf").unwrap(),
            );
        self.submit_form(form).await
    }

    async fn submit_form(&self, form: Form) -> (StatusCode, Value) {
        let response = self
            .http
            .post(format!("{}/v1/orders", self.base))
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = response.status();
        (status, response.json().await.unwrap())
    }

    async fn get(&self, path: &str) -> (StatusCode, Value) {
        let response = self.http.get(format!("{}{path}", self.base)).send().await.unwrap();
        let status = response.status();
        (status, response.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, path: &str) -> (StatusCode, Value) {
        let response = self.http.post(format!("{}{path}", self.base)).send().await.unwrap();
        let status = response.status();
        (status, response.json().await.unwrap_or(Value::Null))
    }
}

fn error_code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap_or("<no error code>")
}

fn finding_codes(body: &Value) -> Vec<&str> {
    body["error"]["findings"]
        .as_array()
        .map(|findings| findings.iter().filter_map(|f| f["code"].as_str()).collect())
        .unwrap_or_default()
}

/// A callback receiver that records what it gets and answers with scripted statuses
/// (the last one repeats).
#[derive(Clone)]
struct Receiver {
    received: Arc<Mutex<Vec<Value>>>,
    hits: Arc<AtomicUsize>,
    script: Arc<Vec<u16>>,
}

async fn receiver(script: &[u16]) -> (String, Receiver) {
    let receiver = Receiver {
        received: Arc::default(),
        hits: Arc::default(),
        script: Arc::new(script.to_vec()),
    };
    async fn handle(State(r): State<Receiver>, body: axum::Json<Value>) -> StatusCode {
        let hit = r.hits.fetch_add(1, Ordering::SeqCst);
        let status = r.script.get(hit).or(r.script.last()).copied().unwrap_or(200);
        if status == 200 {
            r.received.lock().unwrap().push(body.0);
        }
        StatusCode::from_u16(status).unwrap()
    }
    let base = serve(Router::new().route("/hook", post(handle)).with_state(receiver.clone())).await;
    (format!("{base}/hook"), receiver)
}

// ----- catalogue --------------------------------------------------------------------------

#[tokio::test]
async fn catalogue_lists_the_products_with_geometry_and_prices() {
    let p = printer(PrinterConfig::manual()).await;
    let (status, body) = p.get("/v1/products").await;
    assert_eq!(status, StatusCode::OK);
    let products = body.as_array().unwrap();
    let ids: Vec<&str> = products.iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["card-poche", "card-a6", "card-digest", "card-a5", "card-poche-offset"]);
    let poche = &products[0];
    assert_eq!(poche["trim_mm"], json!([230.0, 178.0]));
    assert_eq!(poche["bleed_mm"], 3.0);
    assert_eq!(poche["spine_mm"], 10.0);
    assert_eq!(poche["price_cents"]["1"], 590);
    assert!(poche["paper"].as_str().unwrap().contains("350 g/m²"));
    assert_eq!(poche["cmyk_only"], false);
    assert_eq!(products[4]["cmyk_only"], true);

    let (status, one) = p.get("/v1/products/card-a5").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(one["trim_mm"], json!([306.0, 210.0]));
    let (status, missing) = p.get("/v1/products/poster").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&missing), "unknown_product");
}

#[tokio::test]
async fn home_page_says_it_is_a_demo() {
    let p = printer(PrinterConfig::manual()).await;
    let html = p.http.get(&p.base).send().await.unwrap().text().await.unwrap();
    assert!(html.contains("id=\"demo-banner\""));
    assert!(html.contains("Démo — imprimeur fictif, sans lien avec une entreprise réelle"));
    assert!(html.contains("card-poche"));
    let health = p.http.get(format!("{}/healthz", p.base)).send().await.unwrap();
    assert_eq!(health.text().await.unwrap(), "ok");
    let (status, body) = p.get("/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

// ----- orders -----------------------------------------------------------------------------

#[tokio::test]
async fn a_good_pdf_is_accepted_and_can_be_followed() {
    let p = printer(PrinterConfig::manual()).await;
    let pdf = poche_pdf();
    let (status, body) = p.submit(&order_json("SEP-2026-000001"), pdf.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["status"], "accepted");
    assert_eq!(body["price_cents"], 590);
    assert_eq!(body["currency"], "EUR");
    assert_eq!(body["reference"], "SEP-2026-000001");
    assert_eq!(body["preflight"]["passed"], true);
    assert_eq!(body["preflight"]["warnings"], json!([]));
    let id = body["order_id"].as_str().unwrap().to_owned();
    assert!(id.starts_with("po_"));

    let (status, fetched) = p.get(&format!("/v1/orders/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["status"], "accepted");
    assert_eq!(fetched["tracking"], Value::Null);
    assert!(fetched.get("address").is_none(), "the public view never shows the postal address");
    assert!(!fetched.to_string().contains("rue des Lilas"));

    // The test hook shows everything, including what was received.
    let (status, all) = p.get("/__test/orders").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all[0]["address"]["city"], "Paris");
    assert_eq!(all[0]["pdf_bytes"], pdf.len());
    assert_eq!(all[0]["pdf_sha256"].as_str().unwrap().len(), 64);

    let (status, missing) = p.get("/v1/orders/po_0000000000000000").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&missing), "unknown_order");
}

#[tokio::test]
async fn quantity_drives_the_price() {
    let p = printer(PrinterConfig::manual()).await;
    let mut order = order_json("LOT-25");
    order["quantity"] = json!(25);
    let (status, body) = p.submit(&order, poche_pdf()).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["price_cents"], 4500);
    assert_eq!(body["quantity"], 25);
}

#[tokio::test]
async fn bad_pdfs_are_refused_with_findings_and_nothing_is_stored() {
    let p = printer(PrinterConfig::manual()).await;
    let poche = catalog::find("card-poche").unwrap();
    let good = SampleCard::for_product(poche);
    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "wrong format",
            SampleCard::for_product(catalog::find("card-a6").unwrap()).to_pdf(),
            "BOX_TRIM_SIZE",
        ),
        ("no bleed", SampleCard { bleed_mm: 0.0, ..good.clone() }.to_pdf(), "BOX_BLEED_TOO_SMALL"),
        (
            "no output intent",
            SampleCard { output_intent: false, ..good.clone() }.to_pdf(),
            "NO_OUTPUT_INTENT",
        ),
        (
            "low resolution",
            SampleCard { image_ppi: Some(60.0), ..good.clone() }.to_pdf(),
            "IMAGE_LOW_RES",
        ),
        ("one page", SampleCard { pages: 1, ..good.clone() }.to_pdf(), "PAGE_COUNT"),
        (
            "wrong spine",
            SampleCard { spine_mm: 20.0, ..good.clone() }.to_pdf(),
            "EBC_SPINE_MISMATCH",
        ),
        ("not a pdf", b"<html>pas un PDF</html>".to_vec(), "NOT_A_PDF"),
        ("empty", Vec::new(), "NOT_A_PDF"),
    ];
    for (index, (label, pdf, expected)) in cases.into_iter().enumerate() {
        let (status, body) = p.submit(&order_json(&format!("BAD-{index}")), pdf).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{label}: {body}");
        assert_eq!(error_code(&body), "preflight_failed", "{label}");
        assert!(finding_codes(&body).contains(&expected), "{label}: {:?}", finding_codes(&body));
        // Each finding has the documented shape.
        for finding in body["error"]["findings"].as_array().unwrap() {
            assert!(["error", "warning", "info"].contains(&finding["severity"].as_str().unwrap()));
            assert!(finding["message"].as_str().is_some_and(|m| !m.is_empty()));
            assert!(finding.get("page").is_some());
        }
    }
    let truncated = poche_pdf();
    let (status, body) =
        p.submit(&order_json("BAD-CUT"), truncated[..truncated.len() / 2].to_vec()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&body), "preflight_failed");
    assert!(p.state.orders().is_empty(), "refused files leave no order behind");
}

#[tokio::test]
async fn rgb_is_refused_only_by_the_cmyk_only_product() {
    let p = printer(PrinterConfig::manual()).await;
    let rgb = SampleCard {
        rgb_fill: true,
        ..SampleCard::for_product(catalog::find("card-poche").unwrap())
    };
    let (status, _) = p.submit(&order_json("RGB-DIGITAL"), rgb.to_pdf()).await;
    assert_eq!(status, StatusCode::CREATED, "the digital product converts RGB itself");

    let mut offset = order_json("RGB-OFFSET");
    offset["product"] = json!("card-poche-offset");
    offset["quantity"] = json!(250);
    let (status, body) = p.submit(&offset, rgb.to_pdf()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(finding_codes(&body).contains(&"COLOR_RGB"), "{:?}", finding_codes(&body));

    let cmyk = SampleCard::for_product(catalog::find("card-poche-offset").unwrap());
    let (status, body) = p.submit(&offset, cmyk.to_pdf()).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["price_cents"], 12_900);
}

#[tokio::test]
async fn invalid_requests_get_precise_errors() {
    let p = printer(PrinterConfig::manual()).await;
    let pdf = poche_pdf();
    let patched = |patch: &dyn Fn(&mut Value)| {
        let mut order = order_json("REF-1");
        patch(&mut order);
        order
    };
    let cases: Vec<(Value, StatusCode, &str)> = vec![
        (
            patched(&|o| o["product"] = json!("poster")),
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_product",
        ),
        (
            patched(&|o| o["quantity"] = json!(0)),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_quantity",
        ),
        (
            patched(&|o| o["reference"] = json!("../../etc/passwd")),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_reference",
        ),
        (
            patched(&|o| o["reference"] = json!("<script>")),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_reference",
        ),
        (
            patched(&|o| o["address"]["country"] = json!("US")),
            StatusCode::UNPROCESSABLE_ENTITY,
            "unsupported_country",
        ),
        (
            patched(&|o| o["address"]["name"] = json!("")),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_address",
        ),
        (
            patched(&|o| o["callback_url"] = json!("file:///etc/passwd")),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_callback_url",
        ),
        (patched(&|o| o["quantity"] = json!(-3)), StatusCode::BAD_REQUEST, "invalid_order"),
        (patched(&|o| o["quantity"] = json!("many")), StatusCode::BAD_REQUEST, "invalid_order"),
        (
            patched(&|o| {
                o.as_object_mut().unwrap().remove("address");
            }),
            StatusCode::BAD_REQUEST,
            "invalid_order",
        ),
        (json!([1, 2, 3]), StatusCode::BAD_REQUEST, "invalid_order"),
    ];
    for (order, expected_status, expected_code) in cases {
        let (status, body) = p.submit(&order, pdf.clone()).await;
        assert_eq!((status, error_code(&body)), (expected_status, expected_code), "{order}");
    }

    // Missing parts.
    let only_file = Form::new().part("file", Part::bytes(pdf.clone()).file_name("card.pdf"));
    let (status, body) = p.submit_form(only_file).await;
    assert_eq!((status, error_code(&body)), (StatusCode::BAD_REQUEST, "missing_part"));
    let only_order = Form::new().part("order", Part::text(order_json("REF-2").to_string()));
    let (status, body) = p.submit_form(only_order).await;
    assert_eq!((status, error_code(&body)), (StatusCode::BAD_REQUEST, "missing_part"));

    // Not multipart at all.
    let response = p
        .http
        .post(format!("{}/v1/orders", p.base))
        .json(&order_json("REF-3"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response.json::<Value>().await.unwrap()), "invalid_multipart");

    // A multipart body that stops in the middle of a part.
    let response = p
        .http
        .post(format!("{}/v1/orders", p.base))
        .header("content-type", "multipart/form-data; boundary=XX")
        .body("--XX\r\nContent-Disposition: form-data; name=\"order\"\r\n\r\n{\"product\"")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response.json::<Value>().await.unwrap()), "invalid_multipart");

    // Unknown extra parts are ignored.
    let form = Form::new()
        .part("comment", Part::text("bonjour"))
        .part("order", Part::text(order_json("REF-4").to_string()))
        .part("file", Part::bytes(pdf).file_name("card.pdf"));
    let (status, _) = p.submit_form(form).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(p.state.orders().len(), 1);
}

#[tokio::test]
async fn oversized_uploads_are_refused() {
    let pdf = poche_pdf();
    let config = PrinterConfig { max_pdf_bytes: pdf.len() - 1, ..PrinterConfig::manual() };
    let p = printer(config).await;
    let (status, body) = p.submit(&order_json("BIG-1"), pdf.clone()).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(error_code(&body), "file_too_large");

    // Far beyond the limit: the body limit of the route answers before the part is read.
    let (status, body) = p.submit(&order_json("BIG-2"), vec![0u8; pdf.len() + 200_000]).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(error_code(&body), "file_too_large");

    // An order part stuffed with padding is refused too.
    let p = printer(PrinterConfig::manual()).await;
    let mut order = order_json("BIG-3");
    order["padding"] = json!("x".repeat(20_000));
    let (status, body) = p.submit(&order, pdf).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(error_code(&body), "order_too_large");
}

#[tokio::test]
async fn a_reference_is_an_idempotency_key() {
    let p = printer(PrinterConfig::manual()).await;
    let pdf = poche_pdf();
    let (status, first) = p.submit(&order_json("SEP-2026-000007"), pdf.clone()).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, replay) = p.submit(&order_json("SEP-2026-000007"), pdf.clone()).await;
    assert_eq!(status, StatusCode::OK, "a retry returns the existing order");
    assert_eq!(replay["order_id"], first["order_id"]);

    let other = SampleCard {
        title: Some("Autre carte".into()),
        ..SampleCard::for_product(catalog::find("card-poche").unwrap())
    };
    let (status, body) = p.submit(&order_json("SEP-2026-000007"), other.to_pdf()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "duplicate_reference");
    assert_eq!(p.state.orders().len(), 1);
}

// ----- life cycle and callbacks -----------------------------------------------------------

#[tokio::test]
async fn orders_advance_step_by_step_and_notify_the_callback() {
    let p = printer(PrinterConfig::manual()).await;
    let (hook, receiver) = receiver(&[200]).await;
    let mut order = order_json("SEP-2026-000010");
    order["callback_url"] = json!(format!("{hook}?token=s3cret"));
    let (_, created) = p.submit(&order, poche_pdf()).await;
    let id = created["order_id"].as_str().unwrap().to_owned();
    assert!(receiver.received.lock().unwrap().is_empty(), "acceptance is the HTTP answer itself");

    let (status, changed) = p.post("/__test/advance").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed[0]["status"], "printing");
    let (_, changed) = p.post("/__test/advance").await;
    assert_eq!(changed[0]["status"], "shipped");
    assert_eq!(changed[0]["tracking"], "DC00000001FR");
    let (_, changed) = p.post("/__test/advance").await;
    assert_eq!(changed, json!([]), "a shipped order does not move any more");

    let (_, fetched) = p.get(&format!("/v1/orders/{id}")).await;
    assert_eq!(fetched["status"], "shipped");
    assert_eq!(fetched["tracking"], "DC00000001FR");
    let history: Vec<&str> = fetched["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["status"].as_str().unwrap())
        .collect();
    assert_eq!(history, ["accepted", "printing", "shipped"]);

    let received = receiver.received.lock().unwrap().clone();
    assert_eq!(received.len(), 2);
    assert_eq!(received[0]["status"], "printing");
    assert_eq!(received[0]["order_id"], id.as_str());
    assert_eq!(received[0]["reference"], "SEP-2026-000010");
    assert_eq!(received[0]["tracking"], Value::Null);
    assert_eq!(received[1]["status"], "shipped");
    assert_eq!(received[1]["tracking"], "DC00000001FR");

    let log = p.state.order(&id).unwrap().callbacks;
    assert_eq!(log.len(), 2);
    assert!(log.iter().all(|attempt| attempt.delivered && attempt.http_status == Some(200)));
}

#[tokio::test]
async fn callbacks_are_retried_on_server_errors_but_not_on_client_errors() {
    let p = printer(PrinterConfig::manual()).await;

    // Two failures, then success: delivered on the third attempt.
    let (hook, flaky) = receiver(&[500, 503, 200]).await;
    let mut order = order_json("RETRY-1");
    order["callback_url"] = json!(hook);
    let (_, created) = p.submit(&order, poche_pdf()).await;
    let flaky_id = created["order_id"].as_str().unwrap().to_owned();

    // A receiver that says 404: one attempt only.
    let (hook, gone) = receiver(&[404]).await;
    let mut order = order_json("RETRY-2");
    order["callback_url"] = json!(hook);
    let (_, created) = p.submit(&order, poche_pdf()).await;
    let gone_id = created["order_id"].as_str().unwrap().to_owned();

    // A receiver that is down: the order still moves.
    let mut order = order_json("RETRY-3");
    order["callback_url"] = json!("http://127.0.0.1:9/hook");
    let (_, created) = p.submit(&order, poche_pdf()).await;
    let down_id = created["order_id"].as_str().unwrap().to_owned();

    let (_, changed) = p.post("/__test/advance").await;
    assert_eq!(changed.as_array().unwrap().len(), 3);

    assert_eq!(flaky.hits.load(Ordering::SeqCst), 3);
    assert_eq!(flaky.received.lock().unwrap().len(), 1);
    let log = p.state.order(&flaky_id).unwrap().callbacks;
    assert_eq!((log[0].delivered, log[0].http_status), (true, Some(200)));

    assert_eq!(gone.hits.load(Ordering::SeqCst), 1, "a 4xx is not retried");
    let log = p.state.order(&gone_id).unwrap().callbacks;
    assert_eq!((log[0].delivered, log[0].http_status), (false, Some(404)));

    let down = p.state.order(&down_id).unwrap();
    assert_eq!(down.status.as_str(), "printing");
    assert_eq!((down.callbacks[0].delivered, down.callbacks[0].http_status), (false, None));
}

#[tokio::test]
async fn automatic_mode_ships_without_being_pushed() {
    let config = PrinterConfig { tick: Some(Duration::from_millis(25)), ..PrinterConfig::manual() };
    let p = printer(config).await;
    let (hook, receiver) = receiver(&[200]).await;
    let mut order = order_json("AUTO-1");
    order["callback_url"] = json!(hook);
    let (status, created) = p.submit(&order, poche_pdf()).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["order_id"].as_str().unwrap().to_owned();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let (_, fetched) = p.get(&format!("/v1/orders/{id}")).await;
        if fetched["status"] == "shipped" && receiver.received.lock().unwrap().len() == 2 {
            assert_eq!(fetched["tracking"], "DC00000001FR");
            break;
        }
        assert!(tokio::time::Instant::now() < deadline, "the order never shipped: {fetched}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let statuses: Vec<String> = receiver
        .received
        .lock()
        .unwrap()
        .iter()
        .map(|event| event["status"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(statuses, ["printing", "shipped"]);
}

#[tokio::test]
async fn cancellation_is_possible_until_printing_starts() {
    let p = printer(PrinterConfig::manual()).await;
    let (hook, receiver) = receiver(&[200]).await;
    let mut order = order_json("CANCEL-1");
    order["callback_url"] = json!(hook);
    let (_, created) = p.submit(&order, poche_pdf()).await;
    let id = created["order_id"].as_str().unwrap().to_owned();

    let (status, body) = p.post(&format!("/v1/orders/{id}/cancel")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "cancelled");
    assert_eq!(receiver.received.lock().unwrap()[0]["status"], "cancelled");
    let (status, _) = p.post(&format!("/v1/orders/{id}/cancel")).await;
    assert_eq!(status, StatusCode::OK, "cancelling twice is harmless");
    let (_, changed) = p.post("/__test/advance").await;
    assert_eq!(changed, json!([]));

    let (_, created) = p.submit(&order_json("CANCEL-2"), poche_pdf()).await;
    let id = created["order_id"].as_str().unwrap().to_owned();
    p.post("/__test/advance").await;
    let (status, body) = p.post(&format!("/v1/orders/{id}/cancel")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "too_late");
    let (status, body) = p.post("/v1/orders/po_missing/cancel").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "unknown_order");
}

#[tokio::test]
async fn test_hooks_are_off_unless_enabled() {
    let config = PrinterConfig { test_hooks: false, ..PrinterConfig::manual() };
    let p = printer(config).await;
    let (status, _) = p.submit(&order_json("HOOK-1"), poche_pdf()).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = p.get("/__test/orders").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!body.to_string().contains("rue des Lilas"));
    let (status, _) = p.post("/__test/advance").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(p.state.orders()[0].status.as_str(), "accepted", "nothing moved");
}
