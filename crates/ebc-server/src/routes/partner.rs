//! Partner API (`docs/api.md` §4): HMAC-signed requests, idempotency keys, gifts, cover,
//! card rendering, print orders, cancellation, reissue and fulfilment. Every gift is a
//! partner's — a store, or a shelf service hosting files for their rights holders.
//!
//! # Authentication (§4.1)
//!
//! `X-EBC-Signature: v1=hex(HMAC-SHA-256(secret, "{ts}.{METHOD}.{path?query}.{sha256(body)}"))`
//! is verified in constant time over the request **as received** (method, original path
//! and query, raw body). The timestamp must be within 300 s of the server clock. An
//! unknown partner gets the same `401 bad_signature` as a wrong signature.
//!
//! Partner ids are not secret, so nothing expensive happens before the signature is
//! known to be possibly right: a malformed signature, an unknown partner or a stale
//! timestamp is refused before the body is read; the body is then hashed while it
//! arrives, inside the in-flight body budget shared by all requests, and kept only once
//! the MAC matches. Failed authentications are budgeted per client network
//! (`EBC_RATE_PARTNER_FAIL_PER_10MIN`): beyond, `429` before anything is read.
//!
//! # Replays
//!
//! A captured request can be replayed until its timestamp is stale. Every verified
//! signature of a `POST` or `PUT` is therefore kept as a **nonce** (its decoded MAC, so
//! that the hex case or any header outside the MAC changes nothing) until the timestamp
//! has left the window. The same signature again is answered with the response it got
//! the first time — success or error — and never acts twice: a replayed *reissue* must
//! not void the code it just issued, whatever `Idempotency-Key` the replay carries. The
//! `Idempotency-Key` is an *additional* key that lets a partner retry with a fresh
//! signature. `GET` requests only read and may be repeated.

use std::future::Future;
use std::net::SocketAddr;

use axum::body::Bytes;
use axum::extract::{ConnectInfo, Path, Query, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use ebc_model::{Delivery, GiftStatus};
use serde::Deserialize;

use super::{
    MAX_JSON_BYTES, ReadBody, client_key, code_members, header_text, json, parse_json, read_body,
    status_json,
};
use crate::card::{CardFormat, CardProfile, CardRenderError, CardRequest};
use crate::codes;
use crate::config::Partner;
use crate::error::ApiError;
use crate::names::{ascii_file_name, content_disposition};
use crate::rate::Verdict;
use crate::service::GiftRequest;
use crate::state::AppState;
use crate::store::{BlobRef, GiftRecord, IdempotencyRecord, IdempotentOutcome};

/// An authenticated partner request.
struct Authed {
    partner: Partner,
    parts: Parts,
    body: Bytes,
    /// SHA-256 of the body, hexadecimal.
    body_sha256: String,
    /// The verified MAC in canonical form (lower-case hex of the decoded bytes).
    mac: String,
    /// Keeps the body's share of the in-flight body budget until the request ends.
    _read: ReadBody,
}

fn bad_signature() -> ApiError {
    ApiError::unauthorized("bad_signature", "the request signature is missing or wrong")
}

fn stale() -> ApiError {
    ApiError::unauthorized(
        "stale_timestamp",
        "X-EBC-Timestamp must be within 300 s of the server clock",
    )
}

/// Verifies the signature headers and reads the body (at most `limit` bytes).
///
/// Everything that can be refused without the body is refused first; failures count
/// against the client's partner budget.
async fn authenticate(
    state: &AppState,
    request: Request,
    limit: usize,
) -> Result<Authed, ApiError> {
    let (parts, body) = request.into_parts();
    let peer = parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|info| info.0.ip());
    let client = client_key(state.config.trust_proxy, &parts.headers, peer);
    let now = state.now();
    if let Verdict::Blocked { retry_after_secs, .. } = state.partner_limiter.check(&client, now) {
        return Err(ApiError::too_many(
            "too many failed partner authentications from this network",
            retry_after_secs,
        ));
    }
    let failed = |error: ApiError| {
        let _ = state.partner_limiter.record_failure(&client, now);
        error
    };
    let partner_id =
        header_text(&parts.headers, "x-ebc-partner").ok_or_else(|| failed(bad_signature()))?;
    let mac = header_text(&parts.headers, "x-ebc-signature")
        .and_then(codes::parse_request_signature)
        .ok_or_else(|| failed(bad_signature()))?;
    let Some(partner) = state.config.partner(partner_id).cloned() else {
        return Err(failed(bad_signature()));
    };
    // The window is checked before the body is read: stale requests cost nothing. The
    // difference is computed without overflow: any timestamp is untrusted input.
    let timestamp = header_text(&parts.headers, "x-ebc-timestamp")
        .and_then(|t| t.parse::<i64>().ok())
        .ok_or_else(|| failed(stale()))?;
    let skew = now.checked_sub(timestamp).and_then(i64::checked_abs);
    if skew.is_none_or(|skew| skew > state.config.partner_clock_skew_secs) {
        return Err(failed(stale()));
    }
    let read = read_body(state, &parts.headers, body, limit).await?;
    let body_sha256 = read.sha256_hex();
    let path_and_query =
        parts.uri.path_and_query().map_or_else(|| parts.uri.path(), |pq| pq.as_str());
    let genuine = codes::verify_request_mac(
        &partner.secret,
        timestamp,
        parts.method.as_str(),
        path_and_query,
        &body_sha256,
        &mac,
    );
    if !genuine {
        return Err(failed(bad_signature()));
    }
    Ok(Authed {
        partner,
        body: read.bytes.clone(),
        body_sha256,
        mac: codes::hex(&mac),
        parts,
        _read: read,
    })
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

/// Removes idempotency keys from the in-flight set when the request ends.
struct InFlight<'a>(&'a AppState, Vec<String>);

impl Drop for InFlight<'_> {
    fn drop(&mut self) {
        let mut in_flight =
            self.0.in_flight.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for key in &self.1 {
            in_flight.remove(key);
        }
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

/// Runs `work` at most once per signature and per idempotency key (§4.1).
///
/// * The verified signature (canonical MAC) is a nonce: the same signature again gets
///   the first answer, whatever it was, until the timestamp leaves the window.
/// * The `Idempotency-Key`, when present, is an additional key: a request signed anew
///   with the same key and body gets the first *successful* answer, with a different
///   body `409 idempotency_conflict`. Failed requests are not remembered under it, so
///   they can be retried (signed again) once the cause is fixed.
async fn idempotent<Fut>(state: &AppState, authed: &Authed, work: impl FnOnce() -> Fut) -> Response
where
    Fut: Future<Output = Result<Outcome, ApiError>>,
{
    let headers = &authed.parts.headers;
    let named = match header_text(headers, "idempotency-key") {
        Some(key) if key.len() <= 255 => Some(key),
        Some(_) => {
            return ApiError::bad_request(
                "invalid_idempotency_key",
                "Idempotency-Key: at most 255 characters",
            )
            .into_response();
        }
        None => None,
    };
    let scope =
        format!("{}\n{}\n{}", authed.partner.id, authed.parts.method, authed.parts.uri.path());
    let signature_key = format!("{scope}\nsig:{}", authed.mac);
    let named_key = named.map(|key| format!("{scope}\nkey:{key}"));
    let body_sha256 = authed.body_sha256.clone();
    let now = state.now();
    let ttl = state.config.idempotency_ttl_secs;

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
    let remembered = || {
        let live = |key: &str| {
            state
                .db
                .view(|db| db.idempotency.get(key).cloned())
                .filter(|record| record.is_live(now, ttl))
        };
        live(&signature_key).or_else(|| named_key.as_deref().and_then(live))
    };

    if let Some(record) = remembered() {
        return replay(&record);
    }
    let keys: Vec<String> =
        std::iter::once(signature_key.clone()).chain(named_key.clone()).collect();
    {
        let mut in_flight =
            state.in_flight.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if keys.iter().any(|key| in_flight.contains(key)) {
            return ApiError::conflict(
                "idempotency_in_progress",
                "a request with this signature or Idempotency-Key is being processed",
            )
            .into_response();
        }
        in_flight.extend(keys.iter().cloned());
    }
    let _guard = InFlight(state, keys);
    // A request with the same key may have finished between the two checks.
    if let Some(record) = remembered() {
        return replay(&record);
    }

    let result = work().await;
    let stored = match &result {
        Ok(Outcome { status, code_of: Some((gift_id, reissue)), .. }) => {
            IdempotentOutcome::GiftCode {
                status: status.as_u16(),
                gift_id: gift_id.clone(),
                reissue: *reissue,
            }
        }
        Ok(Outcome { status, body, code_of: None }) => {
            IdempotentOutcome::Response { status: status.as_u16(), body: body.to_string() }
        }
        Err(error) => IdempotentOutcome::Response {
            status: error.status.as_u16(),
            body: error.body().to_string(),
        },
    };
    // The signature is spent until its timestamp can no longer pass the window check.
    let skew = state.config.partner_clock_skew_secs.max(0);
    let nonce_until = now.saturating_add(skew.saturating_mul(2)).saturating_add(1);
    let record = |expires_at| IdempotencyRecord {
        body_sha256: body_sha256.clone(),
        created_at: now,
        outcome: stored.clone(),
        expires_at,
    };
    let remember_named = result.is_ok();
    let saved = state.db.update(|db| {
        db.idempotency.insert(signature_key.clone(), record(Some(nonce_until)));
        if let (true, Some(key)) = (remember_named, &named_key) {
            db.idempotency.insert(key.clone(), record(None));
        }
    });
    if let Err(error) = saved {
        // The records stay in memory (see `Database::update`): replays are still refused.
        tracing::error!(%error, "could not persist an idempotency record");
    }
    match result {
        Ok(outcome) => json(outcome.status, &outcome.body),
        Err(error) => error.into_response(),
    }
}

/// The gift `id` if it belongs to `partner`; `404` otherwise (other partners' gifts are
/// indistinguishable from missing ones).
fn own_gift(state: &AppState, partner: &Partner, id: &str) -> Result<GiftRecord, ApiError> {
    state
        .gift(id)
        .filter(|gift| gift.partner.as_deref() == Some(partner.id.as_str()))
        .ok_or_else(|| ApiError::not_found("no such gift"))
}

/// `POST /api/partner/v1/gifts`.
pub(crate) async fn create(State(state): State<AppState>, request: Request) -> Response {
    let authed = match authenticate(&state, request, MAX_JSON_BYTES).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        let request = GiftRequest::parse(&authed.body)?;
        let created = state.create_gift(request, &authed.partner)?;
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

/// Query of the cover upload.
#[derive(Debug, Deserialize)]
struct CoverQuery {
    #[serde(default)]
    cover_rights: Option<String>,
}

/// `PUT /api/partner/v1/gifts/{id}/cover[?cover_rights=…]`: the cover image the partner
/// may print. The optional `cover_rights` (part of the signed path) declares the rights
/// basis under which it may be printed; without it the basis declared at creation — or
/// `none` — stays, and a cover without a basis is never printed.
pub(crate) async fn put_cover(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let authed = match authenticate(&state, request, state.config.max_cover_bytes).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    // Through the nonce store too: a captured cover upload cannot be replayed later to
    // put back a cover the partner has replaced since.
    idempotent(&state, &authed, || async {
        let gift = own_gift(&state, &authed.partner, &id)?;
        if gift.status == GiftStatus::Cancelled {
            return Err(ApiError::conflict("gift_cancelled", "the gift is cancelled"));
        }
        let Query(query) = Query::<CoverQuery>::try_from_uri(&authed.parts.uri)
            .map_err(|e| ApiError::bad_request("invalid_parameter", e.to_string()))?;
        let declared = crate::service::parse_cover_rights(query.cover_rights.as_deref())?;
        let media_type = crate::card::sniff_image(&authed.body).ok_or_else(|| {
            ApiError::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                "the cover must be a JPEG, PNG or WebP image",
            )
        })?;
        let blob = BlobRef {
            sha256: authed.body_sha256.clone(),
            size_bytes: authed.body.len() as u64,
            media_type: media_type.to_owned(),
        };
        let receipt = serde_json::json!({
            "sha256": blob.sha256, "size_bytes": blob.size_bytes, "media_type": blob.media_type,
            "cover_rights": declared.unwrap_or(gift.cover_rights).as_str(),
        });
        state.attach_blob(&id, blob, authed.body.clone(), true, declared).await?;
        Ok(Outcome::ok(receipt))
    })
    .await
}

/// `PUT /api/partner/v1/gifts/{id}/card.pdf`: a card PDF the partner composed itself
/// (e.g. in the card designer), kept for a print order.
pub(crate) async fn put_card_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let authed = match authenticate(&state, request, state.config.max_card_pdf_bytes).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        let gift = own_gift(&state, &authed.partner, &id)?;
        if gift.status_at(state.now()).is_gone() {
            return Err(ApiError::conflict("gift_gone", "the gift is cancelled or expired"));
        }
        if !authed.body.starts_with(b"%PDF-") {
            return Err(ApiError::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                "the body must be a PDF",
            ));
        }
        let blob = BlobRef {
            sha256: authed.body_sha256.clone(),
            size_bytes: authed.body.len() as u64,
            media_type: "application/pdf".to_owned(),
        };
        let receipt = serde_json::json!({ "sha256": blob.sha256, "size_bytes": blob.size_bytes });
        state.attach_blob(&id, blob, authed.body.clone(), false, None).await?;
        Ok(Outcome::ok(receipt))
    })
    .await
}

/// `POST /api/partner/v1/gifts/{id}/print-orders`: the uploaded card to the print
/// provider ([`super::print::place_print_order`]).
pub(crate) async fn print_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let authed = match authenticate(&state, request, MAX_JSON_BYTES).await {
        Ok(authed) => authed,
        Err(error) => return error.into_response(),
    };
    idempotent(&state, &authed, || async {
        let gift = own_gift(&state, &authed.partner, &id)?;
        let receipt = super::print::place_print_order(&state, &gift, &authed.body).await?;
        Ok(Outcome { status: StatusCode::CREATED, body: receipt, code_of: None })
    })
    .await
}

/// Response header of `card.pdf` listing the composer's warning codes.
pub(crate) const CARD_WARNINGS_HEADER: &str = "x-ebc-card-warnings";

/// Query of `card.pdf`.
#[derive(Debug, Deserialize)]
struct CardQuery {
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    profile: Option<String>,
}

/// `GET /api/partner/v1/gifts/{id}/card.pdf?format=…&profile=…`: the print-ready card,
/// through the [`CardRenderer`](crate::card::CardRenderer) (the card composer with the
/// cargo feature `card`; `501` without a renderer).
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
        public_url: state.config.public_url.clone(),
        claim_url: ebc_model::claim_url(&state.config.public_url, &code),
        claim_url_qr: ebc_model::claim_url_qr(&state.config.public_url, &code),
        delivery_type: gift.delivery.type_name(),
        store_name: (!matches!(gift.delivery, Delivery::PartnerFile { .. }))
            .then(|| authed.partner.display_name().to_owned()),
        cover,
        cover_rights: gift.cover_rights,
        expires_unix: gift.expires_at,
        format,
        profile,
        created_unix: gift.created_at,
    };
    let renderer = std::sync::Arc::clone(&state.card_renderer);
    let rendered = tokio::task::spawn_blocking(move || renderer.render_with_warnings(&card))
        .await
        .map_err(ApiError::internal)?;
    let rendered = rendered.map_err(|error| match error {
        CardRenderError::NotImplemented => {
            ApiError::new(StatusCode::NOT_IMPLEMENTED, "not_implemented", error.to_string())
        }
        CardRenderError::Unprocessable(_) => {
            ApiError::unprocessable("card_unprocessable", error.to_string())
        }
        CardRenderError::RightsRefused(_) => {
            ApiError::unprocessable("cover_rights_refused", error.to_string())
        }
        other => ApiError::internal(other),
    })?;
    let file_name = ascii_file_name(&format!("carte {}", gift.book.title), "pdf");
    let mut response = (StatusCode::OK, rendered.pdf).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
    if let Ok(value) = HeaderValue::from_str(&content_disposition(&file_name, None)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    // The composer's warnings as stable codes, comma-separated (`docs/api.md` §4.2).
    // Only plain `snake_case` codes are sent; anything else a renderer reports is left out.
    let codes: Vec<&str> = rendered
        .warnings
        .iter()
        .map(String::as_str)
        .filter(|code| {
            !code.is_empty()
                && code.len() <= 64
                && code.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
        .take(32)
        .collect();
    if !codes.is_empty()
        && let Ok(value) = HeaderValue::from_str(&codes.join(", "))
    {
        headers.insert(HeaderName::from_static(CARD_WARNINGS_HEADER), value);
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
