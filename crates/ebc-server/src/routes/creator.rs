//! Creator API, standalone mode (`docs/api.md` §3): uploads, gifts managed with a bearer
//! token, and print orders forwarded to the print provider (§5).
//!
//! Uploads and gift creation are open (no account), so they are rate limited per client
//! (`EBC_RATE_CREATE_PER_10MIN`); everything else needs the gift's manage token.

use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::{
    MAX_JSON_BYTES, Visitor, code_members, header_text, json, parse_json, read_body, status_json,
};
use crate::codes;
use crate::error::ApiError;
use crate::rate::Verdict;
use crate::service::GiftRequest;
use crate::state::AppState;
use crate::store::{BlobRef, GiftRecord, PrintOrderRecord};
use crate::upload::RightsBasis;

/// Reads the mandatory `X-Rights-Basis` declaration.
pub(crate) fn rights_basis(headers: &HeaderMap) -> Result<RightsBasis, ApiError> {
    let value = header_text(headers, "x-rights-basis").ok_or_else(|| {
        ApiError::bad_request(
            "missing_rights_basis",
            "X-Rights-Basis is mandatory: own_work, public_domain, open_licence or licensed_distributor",
        )
    })?;
    RightsBasis::parse(value).ok_or_else(|| {
        ApiError::bad_request(
            "invalid_rights_basis",
            "X-Rights-Basis must be own_work, public_domain, open_licence or licensed_distributor",
        )
    })
}

/// The JSON receipt of an upload.
pub(crate) fn upload_receipt(upload: &crate::store::UploadRecord) -> serde_json::Value {
    serde_json::json!({
        "upload_id": upload.id,
        "sha256": upload.sha256,
        "size_bytes": upload.size_bytes,
        "media_type": upload.media_type,
    })
}

/// Counts one request against the client's creation budget.
fn throttle(state: &AppState, visitor: &Visitor) -> Result<(), ApiError> {
    let now = state.now();
    if let Verdict::Blocked { retry_after_secs, .. } =
        state.create_limiter.check(&visitor.client, now)
    {
        return Err(ApiError::too_many(
            "too many uploads or gifts from this client",
            retry_after_secs,
        ));
    }
    let _ = state.create_limiter.record_failure(&visitor.client, now);
    Ok(())
}

/// The gift `id` if the request carries its manage token.
fn authorise(state: &AppState, headers: &HeaderMap, id: &str) -> Result<GiftRecord, ApiError> {
    let gift = state.gift(id).ok_or_else(|| ApiError::not_found("no such gift"))?;
    // Partner gifts have no manage token: they are not reachable through this API.
    let expected =
        gift.manage_token_hash.as_deref().ok_or_else(|| ApiError::not_found("no such gift"))?;
    let token = header_text(headers, header::AUTHORIZATION.as_str())
        .and_then(|value| value.strip_prefix("Bearer ").or_else(|| value.strip_prefix("bearer ")))
        .map(str::trim)
        .ok_or_else(|| {
            ApiError::unauthorized(
                "missing_token",
                "Authorization: Bearer {manage_token} is required",
            )
        })?;
    if codes::ct_eq(codes::token_hash(token).as_bytes(), expected.as_bytes()) {
        Ok(gift)
    } else {
        Err(ApiError::unauthorized("invalid_token", "the manage token does not match this gift"))
    }
}

/// `POST /api/v1/uploads`.
pub(crate) async fn upload(
    State(state): State<AppState>,
    visitor: Visitor,
    request: Request,
) -> Result<Response, ApiError> {
    let (parts, body) = request.into_parts();
    let basis = rights_basis(&parts.headers)?;
    throttle(&state, &visitor)?;
    let bytes = read_body(&parts.headers, body, state.config.max_upload_bytes).await?;
    let record =
        state.store_upload(bytes, header_text(&parts.headers, "x-file-name"), basis, None).await?;
    Ok(json(StatusCode::CREATED, &upload_receipt(&record)))
}

/// `POST /api/v1/gifts`.
pub(crate) async fn create(
    State(state): State<AppState>,
    visitor: Visitor,
    request: Request,
) -> Result<Response, ApiError> {
    let (parts, body) = request.into_parts();
    throttle(&state, &visitor)?;
    let body = read_body(&parts.headers, body, MAX_JSON_BYTES).await?;
    let request = GiftRequest::parse(&body)?;
    let created = state.create_gift(request, None)?;
    let mut members = code_members(&state, &created.code);
    members.insert("id".to_owned(), created.record.id.clone().into());
    members.insert("manage_token".to_owned(), created.manage_token.into());
    Ok(json(StatusCode::CREATED, &serde_json::Value::Object(members)))
}

/// `GET /api/v1/gifts/{id}`.
pub(crate) async fn status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let gift = authorise(&state, &headers, &id)?;
    Ok(json(StatusCode::OK, &status_json(&state, &gift)))
}

/// `DELETE /api/v1/gifts/{id}`: the code stops working and the hosted file is deleted.
pub(crate) async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    authorise(&state, &headers, &id)?;
    let gift = state.cancel_gift(&id)?;
    Ok(json(StatusCode::OK, &status_json(&state, &gift)))
}

/// `POST /api/v1/gifts/{id}/reissue`: voids the code and returns a new one.
pub(crate) async fn reissue(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    authorise(&state, &headers, &id)?;
    let (_, code) = state.reissue_gift(&id)?;
    Ok(json(StatusCode::OK, &serde_json::Value::Object(code_members(&state, &code))))
}

/// `PUT /api/v1/gifts/{id}/card.pdf`: the card composed in the browser, kept for a print
/// order.
pub(crate) async fn put_card_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    let (parts, body) = request.into_parts();
    let gift = authorise(&state, &parts.headers, &id)?;
    if gift.status_at(state.now()).is_gone() {
        return Err(ApiError::conflict("gift_gone", "the gift is cancelled or expired"));
    }
    let bytes = read_body(&parts.headers, body, state.config.max_card_pdf_bytes).await?;
    if !bytes.starts_with(b"%PDF-") {
        return Err(ApiError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "the body must be a PDF",
        ));
    }
    let blob = BlobRef {
        sha256: codes::sha256_hex(&bytes),
        size_bytes: bytes.len() as u64,
        media_type: "application/pdf".to_owned(),
    };
    let receipt = serde_json::json!({ "sha256": blob.sha256, "size_bytes": blob.size_bytes });
    state.attach_blob(&id, blob, bytes, false).await?;
    Ok(json(StatusCode::OK, &receipt))
}

/// Body of `POST /api/v1/gifts/{id}/print-orders`.
#[derive(Debug, Deserialize)]
struct PrintOrderRequest {
    product: String,
    pdf_sha256: String,
    address: serde_json::Value,
    #[serde(default)]
    quantity: Option<u32>,
}

/// `POST /api/v1/gifts/{id}/print-orders`: forwards the uploaded card to the print
/// provider (`EBC_PRINTER_URL`, `docs/api.md` §5) as `multipart/form-data`.
pub(crate) async fn print_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    let (parts, body) = request.into_parts();
    let gift = authorise(&state, &parts.headers, &id)?;
    let body = read_body(&parts.headers, body, MAX_JSON_BYTES).await?;
    let order: PrintOrderRequest = parse_json(&body)?;
    let product_ok = (1..=64).contains(&order.product.len())
        && order
            .product
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'));
    if !product_ok || !order.address.is_object() {
        return Err(ApiError::unprocessable(
            "invalid_request",
            "product must be a product id and address an object",
        ));
    }
    let quantity = order.quantity.unwrap_or(1);
    if !(1..=1000).contains(&quantity) {
        return Err(ApiError::unprocessable(
            "invalid_request",
            "quantity must be between 1 and 1000",
        ));
    }
    let printer = state.config.printer_url.as_deref().ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "printing_not_configured",
            "no print provider is configured",
        )
    })?;
    let card = gift.card_pdf.as_ref().ok_or_else(|| {
        ApiError::conflict("card_pdf_missing", "upload the card with PUT …/card.pdf first")
    })?;
    if !order.pdf_sha256.eq_ignore_ascii_case(&card.sha256) {
        return Err(ApiError::conflict(
            "pdf_mismatch",
            "pdf_sha256 does not match the uploaded card",
        ));
    }
    let pdf = state
        .read_blob(&card.sha256)
        .await?
        .ok_or_else(|| ApiError::internal("the card PDF is missing"))?;

    let unavailable =
        |detail: String| ApiError::new(StatusCode::BAD_GATEWAY, "printer_unavailable", detail);
    let order_json = serde_json::json!({
        "product": order.product,
        "quantity": quantity,
        "address": order.address,
        "reference": gift.id,
    });
    let file = reqwest::multipart::Part::bytes(pdf.to_vec())
        .file_name("card.pdf")
        .mime_str("application/pdf")
        .map_err(ApiError::internal)?;
    let order_part = reqwest::multipart::Part::text(order_json.to_string())
        .mime_str("application/json")
        .map_err(ApiError::internal)?;
    let form = reqwest::multipart::Form::new().part("order", order_part).part("file", file);
    let response =
        state.http.post(format!("{printer}/v1/orders")).multipart(form).send().await.map_err(
            |e| unavailable(format!("the print provider did not answer: {}", e.without_url())),
        )?;
    let printer_status = response.status();
    let answer: serde_json::Value = match response.bytes().await {
        Ok(bytes) if bytes.len() <= MAX_JSON_BYTES * 4 => {
            serde_json::from_slice(&bytes).unwrap_or_default()
        }
        _ => serde_json::Value::Null,
    };
    if printer_status == StatusCode::UNPROCESSABLE_ENTITY {
        // The printer's preflight refused the PDF: pass its findings on.
        let findings =
            answer.pointer("/error/findings").cloned().unwrap_or_else(|| serde_json::json!([]));
        return Err(ApiError::unprocessable(
            "preflight_failed",
            "the print provider refused the PDF",
        )
        .with_extra("findings", findings));
    }
    let order_id = answer
        .get("order_id")
        .and_then(serde_json::Value::as_str)
        .filter(|_| printer_status.is_success());
    let Some(order_id) = order_id else {
        return Err(unavailable(format!("the print provider answered {printer_status}")));
    };
    let record = PrintOrderRecord {
        order_id: order_id.to_owned(),
        product: order.product,
        created_at: state.now(),
    };
    state.db.update(|db| {
        if let Some(gift) = db.gifts.get_mut(&id) {
            gift.print_orders.push(record);
        }
    })?;
    let receipt = serde_json::json!({
        "order_id": order_id,
        "status": answer.get("status").cloned().unwrap_or_else(|| "accepted".into()),
        "price_cents": answer.get("price_cents").cloned().unwrap_or_default(),
        "currency": answer.get("currency").cloned().unwrap_or_else(|| "EUR".into()),
    });
    Ok(json(StatusCode::CREATED, &receipt).into_response())
}
