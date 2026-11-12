//! HTTP handlers of the print provider API (`docs/api.md` §5).

use std::sync::Arc;

use axum::Router;
use axum::extract::multipart::MultipartRejection;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::json;

use crate::catalog::{self, Product, SHIPS_TO};
use crate::orders::{
    Address, CancelError, NewOrder, Order, PrinterState, Registration, sha256_hex,
};
use crate::preflight;

type Shared = Arc<PrinterState>;

/// Largest `order` JSON part accepted, in bytes.
const MAX_ORDER_JSON_BYTES: usize = 16 * 1024;
/// Largest quantity accepted in one order.
const MAX_QUANTITY: u32 = 10_000;

/// Builds the router around `state`.
pub(crate) fn router(state: Shared) -> Router {
    // The multipart body carries the PDF plus a little JSON and framing.
    let body_limit = state.config.max_pdf_bytes.saturating_add(64 * 1024);
    Router::new()
        .route("/", get(home))
        .route("/healthz", get(|| async { "ok" }))
        .route("/v1/products", get(products))
        .route("/v1/products/{id}", get(product))
        .route("/v1/orders", post(create_order).layer(DefaultBodyLimit::max(body_limit)))
        .route("/v1/orders/{id}", get(order))
        .route("/v1/orders/{id}/cancel", post(cancel_order))
        .route("/__test/orders", get(test_orders))
        .route("/__test/advance", post(test_advance))
        .fallback(|| async { error(StatusCode::NOT_FOUND, "not_found", "no such resource") })
        .with_state(state)
}

fn error(status: StatusCode, code: &str, message: &str) -> Response {
    (status, axum::Json(json!({ "error": { "code": code, "message": message } }))).into_response()
}

/// The public view of an order (`GET /v1/orders/{id}` and the creation answer). The postal
/// address is deliberately left out: order identifiers are the only credential of this demo.
fn order_body(order: &Order) -> serde_json::Value {
    json!({
        "order_id": order.order_id,
        "status": order.status,
        "tracking": order.tracking,
        "product": order.product,
        "quantity": order.quantity,
        "reference": order.reference,
        "price_cents": order.price_cents,
        "currency": order.currency,
        "created_at": order.created_at,
        "updated_at": order.updated_at,
        "history": order.history,
        "preflight": { "passed": true, "warnings": order.preflight_warnings },
    })
}

async fn home() -> Response {
    let mut rows = String::new();
    for p in catalog::products() {
        let from = p.price_cents.0.first().map_or(0, |(_, cents)| *cents);
        rows.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td><td>{} × {} mm</td><td>à partir de {},{:02} € \
             (dès {} ex.)</td></tr>\n",
            p.id,
            html_escape(p.name),
            p.trim_mm[0],
            p.trim_mm[1],
            from / 100,
            from % 100,
            p.min_quantity,
        ));
    }
    let page = format!(
        "<!DOCTYPE html>\n<html lang=\"fr\"><head><meta charset=\"utf-8\">\
         <meta name=\"robots\" content=\"noindex, nofollow\">\
         <title>Imprimerie du Dos Carré (démo)</title>\
         <style>body{{font-family:system-ui,sans-serif;max-width:50rem;margin:0 auto;padding:1.5em;line-height:1.5}}\
         #demo-banner{{background:#111;color:#fff;text-align:center;padding:.4em;font-weight:600}}\
         table{{border-collapse:collapse;width:100%}}td,th{{border-bottom:1px solid #ccc;padding:.4em;text-align:left}}</style>\
         </head><body>\
         <div id=\"demo-banner\" role=\"note\">Démo — imprimeur fictif, sans lien avec une entreprise réelle</div>\
         <h1>Imprimerie du Dos Carré (démo)</h1>\
         <p>Imprimeur à la demande fictif du projet eBook Cadeau. Il contrôle les PDF reçus \
         (fonds perdus, format, dos, résolution, profil de sortie) et simule l'impression et \
         l'expédition en accéléré. Aucun papier n'est imprimé ; les prix sont indicatifs.</p>\
         <table id=\"products\"><thead><tr><th>Produit</th><th>Description</th><th>Format à plat</th>\
         <th>Prix</th></tr></thead><tbody>\n{rows}</tbody></table>\
         <p>API : <code>GET /v1/products</code>, <code>POST /v1/orders</code>, \
         <code>GET /v1/orders/{{id}}</code> — voir <code>demo/README.md</code>.</p>\
         </body></html>\n"
    );
    ([(CONTENT_TYPE, "text/html; charset=utf-8")], page).into_response()
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

async fn products() -> Response {
    axum::Json(catalog::products()).into_response()
}

async fn product(Path(id): Path<String>) -> Response {
    match catalog::find(&id) {
        Some(product) => axum::Json(product).into_response(),
        None => error(StatusCode::NOT_FOUND, "unknown_product", "no such product"),
    }
}

/// The `order` part of a submission.
#[derive(Debug, Deserialize)]
struct OrderRequest {
    product: String,
    #[serde(default = "one")]
    quantity: u32,
    address: Address,
    reference: String,
    #[serde(default)]
    callback_url: Option<String>,
}

const fn one() -> u32 {
    1
}

/// A validated request, minus the file.
#[derive(Debug)]
struct ValidOrder {
    product: &'static Product,
    quantity: u32,
    reference: String,
    address: Address,
    callback_url: Option<String>,
    price_cents: u32,
}

fn clean_field(value: &str, max_chars: usize) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()
        && trimmed.chars().count() <= max_chars
        && !trimmed.chars().any(char::is_control))
    .then(|| trimmed.to_owned())
}

/// Validates the `order` part. On failure returns `(error code, message)`.
fn validate(request: OrderRequest) -> Result<ValidOrder, (&'static str, String)> {
    let product = catalog::find(&request.product)
        .ok_or(("unknown_product", "no such product; see GET /v1/products".to_owned()))?;
    if request.quantity == 0 || request.quantity > MAX_QUANTITY {
        return Err(("invalid_quantity", format!("quantity must be between 1 and {MAX_QUANTITY}")));
    }
    if request.quantity < product.min_quantity {
        return Err((
            "invalid_quantity",
            format!("{} is sold from {} copies", product.id, product.min_quantity),
        ));
    }
    let price_cents = product
        .price_for(request.quantity)
        .ok_or(("invalid_quantity", "no price for this quantity".to_owned()))?;
    let reference_ok = !request.reference.is_empty()
        && request.reference.len() <= 64
        && request
            .reference
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'));
    if !reference_ok {
        return Err((
            "invalid_reference",
            "reference must be 1 to 64 characters among A-Z a-z 0-9 - _ .".to_owned(),
        ));
    }
    let bad_address =
        |field: &str| ("invalid_address", format!("address.{field} is missing or too long"));
    let address = Address {
        name: clean_field(&request.address.name, 80).ok_or_else(|| bad_address("name"))?,
        line1: clean_field(&request.address.line1, 120).ok_or_else(|| bad_address("line1"))?,
        line2: match request.address.line2.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(line) => Some(clean_field(line, 120).ok_or_else(|| bad_address("line2"))?),
        },
        postal_code: clean_field(&request.address.postal_code, 12)
            .ok_or_else(|| bad_address("postal_code"))?,
        city: clean_field(&request.address.city, 80).ok_or_else(|| bad_address("city"))?,
        country: request.address.country.trim().to_ascii_uppercase(),
    };
    if !SHIPS_TO.contains(&address.country.as_str()) {
        return Err((
            "unsupported_country",
            format!("this printer posts to {} only", SHIPS_TO.join(", ")),
        ));
    }
    let callback_url = match request.callback_url.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(url) => {
            let parsed = reqwest::Url::parse(url).ok().filter(|u| {
                matches!(u.scheme(), "http" | "https") && u.host_str().is_some() && url.len() <= 500
            });
            if parsed.is_none() {
                return Err((
                    "invalid_callback_url",
                    "callback_url must be an http(s) URL of at most 500 characters".to_owned(),
                ));
            }
            Some(url.to_owned())
        }
    };
    Ok(ValidOrder {
        product,
        quantity: request.quantity,
        reference: request.reference,
        address,
        callback_url,
        price_cents,
    })
}

/// Reads the two parts of the multipart body: `(order JSON bytes, PDF bytes)`.
async fn read_parts(
    state: &PrinterState,
    mut multipart: Multipart,
) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>), Response> {
    let mut order = None;
    let mut file = None;
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(e) => return Err(multipart_error(e.status(), &e.body_text())),
        };
        let name = field.name().unwrap_or_default().to_owned();
        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => return Err(multipart_error(e.status(), &e.body_text())),
        };
        match name.as_str() {
            "order" => {
                if data.len() > MAX_ORDER_JSON_BYTES {
                    return Err(error(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "order_too_large",
                        "the order part is too large",
                    ));
                }
                order = Some(data.to_vec());
            }
            "file" => {
                if data.len() > state.config.max_pdf_bytes {
                    return Err(error(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "file_too_large",
                        "the PDF is too large",
                    ));
                }
                file = Some(data.to_vec());
            }
            // Unknown parts are ignored, as most upload APIs do.
            _ => {}
        }
    }
    Ok((order, file))
}

fn multipart_error(status: StatusCode, detail: &str) -> Response {
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        error(status, "file_too_large", "the upload is too large")
    } else {
        error(StatusCode::BAD_REQUEST, "invalid_multipart", detail)
    }
}

async fn create_order(
    State(state): State<Shared>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Response {
    let multipart = match multipart {
        Ok(multipart) => multipart,
        Err(rejection) => {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_multipart",
                &format!("expected multipart/form-data with parts `order` and `file`: {rejection}"),
            );
        }
    };
    let (order, file) = match read_parts(&state, multipart).await {
        Ok(parts) => parts,
        Err(response) => return response,
    };
    let (Some(order), Some(pdf)) = (order, file) else {
        return error(
            StatusCode::BAD_REQUEST,
            "missing_part",
            "both parts are required: `order` (JSON) and `file` (PDF)",
        );
    };
    let request: OrderRequest = match serde_json::from_slice(&order) {
        Ok(request) => request,
        Err(e) => {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_order",
                &format!("order is not valid: {e}"),
            );
        }
    };
    let valid = match validate(request) {
        Ok(valid) => valid,
        Err((code, message)) => return error(StatusCode::UNPROCESSABLE_ENTITY, code, &message),
    };

    // Preflight is CPU-bound: keep it off the async workers.
    let product = valid.product;
    let pdf = Arc::new(pdf);
    let verdict = {
        let pdf = pdf.clone();
        tokio::task::spawn_blocking(move || preflight::run(&pdf, product)).await
    };
    let Ok(verdict) = verdict else {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "preflight could not be run");
    };
    if !verdict.passed {
        let errors = verdict.findings.iter().filter(|f| f.severity == "error").count();
        let body = json!({ "error": {
            "code": "preflight_failed",
            "message": format!("the file cannot be printed as {}: {errors} error(s)", product.id),
            "findings": verdict.findings,
        }});
        return (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response();
    }

    let registration = state.register(NewOrder {
        product,
        quantity: valid.quantity,
        reference: valid.reference,
        address: valid.address,
        callback_url: valid.callback_url,
        price_cents: valid.price_cents,
        pdf_sha256: sha256_hex(&pdf),
        pdf_bytes: pdf.len(),
        preflight_warnings: verdict.warnings(),
    });
    match registration {
        Registration::Created(order) => {
            if let Some(tick) = state.config.tick {
                tokio::spawn(state.clone().run(order.order_id.clone(), tick));
            }
            (StatusCode::CREATED, axum::Json(order_body(&order))).into_response()
        }
        Registration::Replayed(order) => axum::Json(order_body(&order)).into_response(),
        Registration::Conflict => error(
            StatusCode::CONFLICT,
            "duplicate_reference",
            "this reference was already used for a different order",
        ),
        Registration::Full => error(
            StatusCode::SERVICE_UNAVAILABLE,
            "demo_full",
            "the demo printer has reached its order limit",
        ),
    }
}

async fn order(State(state): State<Shared>, Path(id): Path<String>) -> Response {
    match state.order(&id) {
        Some(order) => axum::Json(order_body(&order)).into_response(),
        None => error(StatusCode::NOT_FOUND, "unknown_order", "no such order"),
    }
}

async fn cancel_order(State(state): State<Shared>, Path(id): Path<String>) -> Response {
    match state.cancel(&id).await {
        Ok(order) => axum::Json(order_body(&order)).into_response(),
        Err(CancelError::Unknown) => error(StatusCode::NOT_FOUND, "unknown_order", "no such order"),
        Err(CancelError::TooLate) => error(
            StatusCode::CONFLICT,
            "too_late",
            "printing has started; the order cannot be cancelled",
        ),
    }
}

async fn test_orders(State(state): State<Shared>) -> Response {
    if !state.config.test_hooks {
        return error(StatusCode::NOT_FOUND, "not_found", "no such resource");
    }
    axum::Json(state.orders()).into_response()
}

async fn test_advance(State(state): State<Shared>) -> Response {
    if !state.config.test_hooks {
        return error(StatusCode::NOT_FOUND, "not_found", "no such resource");
    }
    let changed: Vec<serde_json::Value> =
        state.advance_all().await.iter().map(order_body).collect();
    axum::Json(changed).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(patch: impl FnOnce(&mut serde_json::Value)) -> OrderRequest {
        let mut value = json!({
            "product": "card-poche",
            "quantity": 1,
            "reference": "SEP-2026-000001",
            "address": {
                "name": "Sacha Martin", "line1": "12 rue des Lilas",
                "postal_code": "75011", "city": "Paris", "country": "fr"
            },
        });
        patch(&mut value);
        serde_json::from_value(value).expect("the patched request still deserialises")
    }

    fn code(result: Result<ValidOrder, (&'static str, String)>) -> &'static str {
        result.err().map_or("ok", |(code, _)| code)
    }

    #[test]
    fn a_plain_request_is_accepted_and_normalised() {
        let valid = validate(request(|_| {})).unwrap();
        assert_eq!(valid.product.id, "card-poche");
        assert_eq!(valid.price_cents, 590);
        assert_eq!(valid.address.country, "FR", "country codes are upper-cased");
        assert_eq!(valid.callback_url, None);
        // quantity defaults to 1 when absent.
        let valid = validate(request(|v| {
            v.as_object_mut().unwrap().remove("quantity");
        }))
        .unwrap();
        assert_eq!(valid.quantity, 1);
    }

    #[test]
    fn products_quantities_and_references_are_checked() {
        assert_eq!(code(validate(request(|v| v["product"] = json!("poster")))), "unknown_product");
        assert_eq!(code(validate(request(|v| v["quantity"] = json!(0)))), "invalid_quantity");
        assert_eq!(code(validate(request(|v| v["quantity"] = json!(10_001)))), "invalid_quantity");
        assert_eq!(
            code(validate(request(|v| v["product"] = json!("card-poche-offset")))),
            "invalid_quantity",
            "below the product's minimum"
        );
        for bad in ["", "a b", "réf", "../x", "x".repeat(65).as_str(), "a\nb"] {
            assert_eq!(
                code(validate(request(|v| v["reference"] = json!(bad)))),
                "invalid_reference",
                "{bad:?}"
            );
        }
    }

    #[test]
    fn addresses_and_callbacks_are_checked() {
        assert_eq!(
            code(validate(request(|v| v["address"]["name"] = json!("  ")))),
            "invalid_address"
        );
        assert_eq!(
            code(validate(request(|v| v["address"]["line1"] = json!("x".repeat(121))))),
            "invalid_address"
        );
        assert_eq!(
            code(validate(request(|v| v["address"]["city"] = json!("Pa\u{0}ris")))),
            "invalid_address"
        );
        assert_eq!(
            code(validate(request(|v| v["address"]["country"] = json!("US")))),
            "unsupported_country"
        );
        let with_line2 = validate(request(|v| v["address"]["line2"] = json!(" Bât. B "))).unwrap();
        assert_eq!(with_line2.address.line2.as_deref(), Some("Bât. B"));
        let blank_line2 = validate(request(|v| v["address"]["line2"] = json!(""))).unwrap();
        assert_eq!(blank_line2.address.line2, None);

        for bad in ["ftp://x/y", "javascript:alert(1)", "/relative", "http://"] {
            assert_eq!(
                code(validate(request(|v| v["callback_url"] = json!(bad)))),
                "invalid_callback_url",
                "{bad}"
            );
        }
        let ok = validate(request(|v| {
            v["callback_url"] = json!("http://127.0.0.1:8101/webhooks/printer?token=abc")
        }));
        assert_eq!(
            ok.unwrap().callback_url.as_deref(),
            Some("http://127.0.0.1:8101/webhooks/printer?token=abc")
        );
    }

    #[test]
    fn html_is_escaped() {
        assert_eq!(html_escape("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }
}
