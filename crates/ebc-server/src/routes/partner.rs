//! Partner API, reseller-integrated mode (`docs/api.md` §4): HMAC-signed requests,
//! idempotency keys, gifts, cover, card rendering, cancellation, reissue and fulfilment.
//!
//! # Authentication (§4.1)
//!
//! `X-EBC-Signature: v1=hex(HMAC-SHA-256(secret, "{ts}.{METHOD}.{path?query}.{sha256(body)}"))`
//! is verified in constant time over the request **as received** (method, original path
//! and query, raw body). The timestamp must be within 300 s of the server clock. An
//! unknown partner gets the same `401 bad_signature` as a wrong signature.
//!
//! # Replays
//!
//! A captured request can be replayed until its timestamp is stale. Every `POST` is
//! therefore idempotent: under its `Idempotency-Key`, or — when the header is absent —
//! under its own signature, so a byte-identical replay returns the first answer instead
//! of acting twice (a replayed *reissue* must not void the code it just issued). `GET`
//! and `PUT` requests are naturally idempotent.

use std::future::Future;

use axum::body::Bytes;
use axum::extract::{Path, Query, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use ebc_model::{Delivery, GiftStatus};
use serde::Deserialize;

use super::creator::{rights_basis, upload_receipt};
use super::{MAX_JSON_BYTES, code_members, header_text, json, parse_json, read_body, status_json};
use crate::card::{CardFormat, CardProfile, CardRenderError, CardRequest};
use crate::codes;
use crate::config::Partner;
use crate::error::ApiError;
use crate::names::{ascii_file_name, content_disposition};
use crate::service::GiftRequest;
use crate::state::AppState;
use crate::store::{BlobRef, GiftRecord, IdempotencyRecord, IdempotentOutcome};
use crate::upload;

/// An authenticated partner request.
struct Authed {
    partner: Partner,
    parts: Parts,
    body: Bytes,
}

fn bad_signature() -> ApiError {
    ApiError::unauthorized("bad_signature", "the request signature is missing or wrong")
}

/// Verifies the signature headers and reads the body (at most `limit` bytes).
async fn authenticate(
    state: &AppState,
    request: Request,
    limit: usize,
) -> Result<Authed, ApiError> {
    let (parts, body) = request.into_parts();
    let partner_id = header_text(&parts.headers, "x-ebc-partner").ok_or_else(bad_signature)?;
    let signature =
        header_text(&parts.headers, "x-ebc-signature").ok_or_else(bad_signature)?.to_owned();
    let timestamp =
        header_text(&parts.headers, "x-ebc-timestamp").and_then(|t| t.parse::<i64>().ok());
    let Some(partner) = state.config.partner(partner_id).cloned() else {
        return Err(bad_signature());
    };
    // The window is checked before the body is read: stale requests cost nothing.
    let stale = || {
        ApiError::unauthorized(
            "stale_timestamp",
            "X-EBC-Timestamp must be within 300 s of the server clock",
        )
    };
    let timestamp = timestamp.ok_or_else(stale)?;
    if (state.now() - timestamp).abs() > state.config.partner_clock_skew_secs {
        return Err(stale());
    }
    let body = read_body(&parts.headers, body, limit).await?;
    let path_and_query =
        parts.uri.path_and_query().map_or_else(|| parts.uri.path(), |pq| pq.as_str());
    let genuine = codes::verify_request_signature(
        &partner.secret,
        timestamp,
        parts.method.as_str(),
        path_and_query,
        &body,
        &signature,
    );
    if !genuine {
        return Err(bad_signature());
    }
    Ok(Authed { partner, parts, body })
}

/// What an idempotent handler produced.
struct Outcome {
    status: StatusCode,
    body: serde_json::Value,
    /// Set when the body carries the gift's code: `(gift id, is a reissue)`. The body is
    /// then rebuilt on replay instead of being stored.
    code_of: Option<(String, bool)>,
}

impl Outcome {
    fn ok(body: serde_json::Value) -> Self {
        Self { status: StatusCode::OK, body, code_of: None }
    }
}

/// Removes an idempotency key from the in-flight set when the request ends.
struct InFlight<'a>(&'a AppState, String);

impl Drop for InFlight<'_> {
    fn drop(&mut self) {
        self.0.in_flight.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&self.1);
    }
}

/// The body of a creation (`reissue == false`) or reissue answer for `gift`.
fn code_body(state: &AppState, gift: &GiftRecord, reissue: bool) -> Option<serde_json::Value> {
    let code = state.derived_code(gift)?;
    let mut members = code_members(state, &code);
    if !reissue {
        members.insert("id".to_owned(), gift.id.clone().into());
        members.insert("status".to_owned(), gift.status_at(state.now()).as_str().into());
    }
    Some(serde_json::Value::Object(members))
}

/// Runs `work` at most once per idempotency key (§4.1): a replay with the same body gets
/// the first answer, with a different body `409 idempotency_conflict`. Only successful
/// answers are remembered, so a failed request can simply be retried.
async fn idempotent<Fut>(state: &AppState, authed: &Authed, work: impl FnOnce() -> Fut) -> Response
where
    Fut: Future<Output = Result<Outcome, ApiError>>,
{
    let headers = &authed.parts.headers;
    let key = match header_text(headers, "idempotency-key") {
        Some(key) if key.len() <= 255 => format!("key:{key}"),
        Some(_) => {
            return ApiError::bad_request(
                "invalid_idempotency_key",
                "Idempotency-Key: at most 255 characters",
            )
            .into_response();
        }
        None => format!("sig:{}", header_text(headers, "x-ebc-signature").unwrap_or("")),
    };
    let storage_key = format!(
        "{}\n{}\n{}\n{key}",
        authed.partner.id,
        authed.parts.method,
        authed.parts.uri.path()
    );
    let body_sha256 = codes::sha256_hex(&authed.body);
    let now = state.now();

    let replay = |record: &IdempotencyRecord| -> Response {
        if record.body_sha256 != body_sha256 {
            return ApiError::conflict(
                "idempotency_conflict",
                "this Idempotency-Key was used with a different body",
            )
            .into_response();
        }
        let (status, body) = match &record.outcome {
            IdempotentOutcome::Response { status, body } => (*status, body.clone()),
            IdempotentOutcome::GiftCode { status, gift_id, reissue } => {
                match state.gift(gift_id).and_then(|gift| code_body(state, &gift, *reissue)) {
                    Some(body) => (*status, body.to_string()),
                    None => {
                        return ApiError::conflict(
                            "idempotency_conflict",
                            "the gift of the original request no longer exists",
                        )
                        .into_response();
                    }
                }
            }
        };
        let mut response =
            (StatusCode::from_u16(status).unwrap_or(StatusCode::OK), body).into_response();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        response.headers_mut().insert("idempotent-replayed", HeaderValue::from_static("true"));
        response
    };
    let remembered = |state: &AppState| {
        state
            .db
            .view(|db| db.idempotency.get(&storage_key).cloned())
            .filter(|r| now - r.created_at < state.config.idempotency_ttl_secs)
    };

    if let Some(record) = remembered(state) {
        return replay(&record);
    }
    let fresh = state
        .in_flight
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(storage_key.clone());
    if !fresh {
        return ApiError::conflict(
            "idempotency_in_progress",
            "a request with this Idempotency-Key is being processed",
        )
        .into_response();
    }
    let _guard = InFlight(state, storage_key.clone());
    // A request with the same key may have finished between the two checks.
    if let Some(record) = remembered(state) {
        return replay(&record);
    }

    let outcome = match work().await {
        Ok(outcome) => outcome,
        Err(error) => return error.into_response(),
    };
    let stored = match &outcome.code_of {
        Some((gift_id, reissue)) => IdempotentOutcome::GiftCode {
            status: outcome.status.as_u16(),
            gift_id: gift_id.clone(),
            reissue: *reissue,
        },
        None => IdempotentOutcome::Response {
            status: outcome.status.as_u16(),
            body: outcome.body.to_string(),
        },
    };
    let record = IdempotencyRecord { body_sha256, created_at: now, outcome: stored };
    if let Err(error) = state.db.update(|db| db.idempotency.insert(storage_key.clone(), record)) {
        tracing::error!(%error, "could not remember an idempotency key");
    }
    json(outcome.status, &outcome.body)
}

/// The gift `id` if it belongs to `partner`; `404` otherwise (other partners' gifts are
/// indistinguishable from missing ones).
fn own_gift(state: &AppState, partner: &Partner, id: &str) -> Result<GiftRecord, ApiError> {
    state
        .gift(id)
        .filter(|gift| gift.partner.as_deref() == Some(partner.id.as_str()))
        .ok_or_else(|| ApiError::not_found("no such gift"))
}

/// `POST /api/partner/v1/uploads`.
pub(crate) async fn upload(State(state): State<AppState>, request: Request) -> Response {
    let authed = match authenticate(&state, request, state.config.max_upload_bytes).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        let basis = rights_basis(&authed.parts.headers)?;
        let file_name = header_text(&authed.parts.headers, "x-file-name");
        let record = state
            .store_upload(authed.body.clone(), file_name, basis, Some(&authed.partner.id))
            .await?;
        Ok(Outcome { status: StatusCode::CREATED, body: upload_receipt(&record), code_of: None })
    })
    .await
}

/// `POST /api/partner/v1/gifts`.
pub(crate) async fn create(State(state): State<AppState>, request: Request) -> Response {
    let authed = match authenticate(&state, request, MAX_JSON_BYTES).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        let request: GiftRequest = parse_json(&authed.body)?;
        let created = state.create_gift(request, Some(&authed.partner))?;
        let mut members = code_members(&state, &created.code);
        members.insert("id".to_owned(), created.record.id.clone().into());
        members.insert("status".to_owned(), "created".into());
        Ok(Outcome {
            status: StatusCode::CREATED,
            body: serde_json::Value::Object(members),
            code_of: Some((created.record.id, false)),
        })
    })
    .await
}

/// `GET /api/partner/v1/gifts/{id}`.
pub(crate) async fn status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    let authed = authenticate(&state, request, MAX_JSON_BYTES).await?;
    let gift = own_gift(&state, &authed.partner, &id)?;
    Ok(json(StatusCode::OK, &status_json(&state, &gift)))
}

/// `PUT /api/partner/v1/gifts/{id}/cover`: the cover image the partner may print.
pub(crate) async fn put_cover(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    let authed = authenticate(&state, request, state.config.max_cover_bytes).await?;
    let gift = own_gift(&state, &authed.partner, &id)?;
    if gift.status == GiftStatus::Cancelled {
        return Err(ApiError::conflict("gift_cancelled", "the gift is cancelled"));
    }
    let media_type = upload::sniff_image(&authed.body).ok_or_else(|| {
        ApiError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "the cover must be a JPEG, PNG or WebP image",
        )
    })?;
    let blob = BlobRef {
        sha256: codes::sha256_hex(&authed.body),
        size_bytes: authed.body.len() as u64,
        media_type: media_type.to_owned(),
    };
    let receipt = serde_json::json!({ "sha256": blob.sha256, "size_bytes": blob.size_bytes, "media_type": blob.media_type });
    state.attach_blob(&id, blob, authed.body, true).await?;
    Ok(json(StatusCode::OK, &receipt))
}

/// Query of `card.pdf`.
#[derive(Debug, Deserialize)]
struct CardQuery {
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    profile: Option<String>,
}

/// `GET /api/partner/v1/gifts/{id}/card.pdf?format=…&profile=…`: the print-ready card,
/// through the [`CardRenderer`](crate::card::CardRenderer) (`501` until one is plugged in).
pub(crate) async fn card_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, ApiError> {
    let authed = authenticate(&state, request, MAX_JSON_BYTES).await?;
    let gift = own_gift(&state, &authed.partner, &id)?;
    let Query(query) = Query::<CardQuery>::try_from_uri(&authed.parts.uri)
        .map_err(|e| ApiError::bad_request("invalid_parameter", e.to_string()))?;
    let format =
        CardFormat::parse(query.format.as_deref().unwrap_or("poche")).ok_or_else(|| {
            ApiError::bad_request("invalid_parameter", "format must be poche, a6, digest or a5")
        })?;
    let profile =
        CardProfile::parse(query.profile.as_deref().unwrap_or("pro")).ok_or_else(|| {
            ApiError::bad_request(
                "invalid_parameter",
                "profile must be pro, home-a4 or home-letter",
            )
        })?;
    if gift.status == GiftStatus::Cancelled {
        return Err(ApiError::conflict("gift_cancelled", "the gift is cancelled"));
    }
    let code = state.derived_code(&gift).ok_or_else(|| {
        ApiError::conflict(
            "code_unavailable",
            "the code of this gift cannot be reproduced; reissue it",
        )
    })?;
    let cover = match &gift.cover {
        Some(cover) => state
            .read_blob(&cover.sha256)
            .await?
            .map(|bytes| (bytes.to_vec(), cover.media_type.clone())),
        None => None,
    };
    let card = CardRequest {
        gift_id: gift.id.clone(),
        book: gift.book.clone(),
        giver_name: gift.giver_name.clone(),
        recipient_name: gift.recipient_name.clone(),
        message: gift.message.clone(),
        locale: gift.locale,
        code,
        claim_url: ebc_model::claim_url(&state.config.public_url, &code),
        claim_url_qr: ebc_model::claim_url_qr(&state.config.public_url, &code),
        delivery_type: gift.delivery.type_name(),
        store_name: (!matches!(gift.delivery, Delivery::HostedFile { .. }))
            .then(|| authed.partner.display_name().to_owned()),
        cover,
        format,
        profile,
        created_unix: gift.created_at,
    };
    let renderer = std::sync::Arc::clone(&state.card_renderer);
    let rendered = tokio::task::spawn_blocking(move || renderer.render(&card))
        .await
        .map_err(ApiError::internal)?;
    let pdf = rendered.map_err(|error| match error {
        CardRenderError::NotImplemented => {
            ApiError::new(StatusCode::NOT_IMPLEMENTED, "not_implemented", error.to_string())
        }
        CardRenderError::Unprocessable(_) => {
            ApiError::unprocessable("card_unprocessable", error.to_string())
        }
        other => ApiError::internal(other),
    })?;
    let file_name = ascii_file_name(&format!("carte {}", gift.book.title), "pdf");
    let mut response = (StatusCode::OK, pdf).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
    if let Ok(value) = HeaderValue::from_str(&content_disposition(&file_name, None)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

/// `POST /api/partner/v1/gifts/{id}/cancel`: voids the code (refund path).
pub(crate) async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let authed = match authenticate(&state, request, MAX_JSON_BYTES).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        own_gift(&state, &authed.partner, &id)?;
        let gift = state.cancel_gift(&id)?;
        Ok(Outcome::ok(status_json(&state, &gift)))
    })
    .await
}

/// `POST /api/partner/v1/gifts/{id}/reissue`: a new code; the previous one answers `410`.
pub(crate) async fn reissue(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let authed = match authenticate(&state, request, MAX_JSON_BYTES).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        own_gift(&state, &authed.partner, &id)?;
        let (gift, code) = state.reissue_gift(&id)?;
        Ok(Outcome {
            status: StatusCode::OK,
            body: serde_json::Value::Object(code_members(&state, &code)),
            code_of: Some((gift.id, true)),
        })
    })
    .await
}

/// Body of the fulfilment confirmation.
#[derive(Debug, Deserialize)]
struct FulfilledRequest {
    account_ref: String,
}

/// `POST /api/partner/v1/gifts/{id}/fulfilled`: the store confirms the book is in an
/// account (§4.4). Confirming again with the same `account_ref` is fine; with another
/// one it is `409 already_fulfilled`. Only a hash of the reference is kept.
pub(crate) async fn fulfilled(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let authed = match authenticate(&state, request, MAX_JSON_BYTES).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        let confirmation: FulfilledRequest = parse_json(&authed.body)?;
        let account_ref = confirmation.account_ref.trim();
        if account_ref.is_empty() || account_ref.len() > 256 {
            return Err(ApiError::unprocessable(
                "invalid_request",
                "account_ref must be 1 to 256 characters",
            ));
        }
        let gift = own_gift(&state, &authed.partner, &id)?;
        if !matches!(gift.delivery, Delivery::ResellerRedirect { .. }) {
            return Err(ApiError::conflict(
                "wrong_delivery",
                "only reseller_redirect gifts are confirmed as fulfilled",
            ));
        }
        let account_hash = codes::sha256_hex(
            format!("ebc:account-ref:v1:{}:{account_ref}", authed.partner.id).as_bytes(),
        );
        let now = state.now();
        let updated = state.db.update(|db| -> Result<GiftRecord, ApiError> {
            let gift = db.gifts.get_mut(&id).ok_or_else(|| ApiError::not_found("no such gift"))?;
            match (gift.status_at(now), gift.account_ref_hash.as_deref()) {
                (GiftStatus::Fulfilled, Some(existing)) if existing == account_hash => {}
                (GiftStatus::Fulfilled, _) => {
                    return Err(ApiError::conflict(
                        "already_fulfilled",
                        "this gift is already in another account",
                    ));
                }
                (GiftStatus::Created | GiftStatus::Redeemed, _) => {
                    gift.status = GiftStatus::Fulfilled;
                    gift.account_ref_hash = Some(account_hash);
                    gift.redeemed_at.get_or_insert(now);
                    gift.last_claim_at.get_or_insert(now);
                }
                (status, _) => {
                    return Err(ApiError::conflict(
                        "not_claimable",
                        format!("the gift is {status}"),
                    ));
                }
            }
            Ok(gift.clone())
        })??;
        Ok(Outcome::ok(status_json(&state, &updated)))
    })
    .await
}
