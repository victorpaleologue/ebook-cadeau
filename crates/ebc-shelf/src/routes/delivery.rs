//! Delivery: the `partner_file` endpoint the core calls on each claim
//! (`GET /api/ebc/link`, `docs/api.md` §4.5) and the short-lived download link it hands
//! out (`GET|HEAD /f/{token}/{name}`).
//!
//! Neither the signed query nor the link nor any request header (`Referer` may carry the
//! gift page's URL, with its code) is ever logged.

use std::collections::HashMap;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::Visitor;
use crate::crypto;
use crate::service::LinkRefusal;
use crate::state::AppState;
use crate::store::Blob;

/// Failures counted per client network before `429`, over 10 minutes.
const FAIL_WINDOW: i64 = 600;

fn json_error(status: StatusCode, code: &str, message: &str) -> Response {
    let body = serde_json::json!({ "error": { "code": code, "message": message } });
    (status, Json(body)).into_response()
}

fn failure_budget(state: &AppState, visitor: &Visitor) -> Option<Response> {
    let key = format!("fail:{}", visitor.client);
    state.counters.spent(&key, state.config.rate_fail_per_10min, FAIL_WINDOW, state.now()).map(
        |wait| {
            let mut response =
                json_error(StatusCode::TOO_MANY_REQUESTS, "too_many_requests", "too many failures");
            if let Ok(value) = HeaderValue::from_str(&wait.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
            response
        },
    )
}

fn count_failure(state: &AppState, visitor: &Visitor) {
    let key = format!("fail:{}", visitor.client);
    let _ = state.counters.hit(&key, u32::MAX, FAIL_WINDOW, state.now());
}

/// `GET /api/ebc/link?ebc_gift=…&ebc_order=…&ebc_ts=…&ebc_sig=…` (§4.5).
pub(crate) async fn link(
    State(state): State<AppState>,
    visitor: Visitor,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if let Some(refused) = failure_budget(&state, &visitor) {
        return refused;
    }
    let field = |name: &str| query.get(name).map_or("", String::as_str);
    match state.issue_link(field("ebc_gift"), field("ebc_order"), field("ebc_ts"), field("ebc_sig"))
    {
        Ok(answer) => {
            tracing::info!(gift = field("ebc_order"), "download link issued");
            (StatusCode::OK, Json(answer)).into_response()
        }
        Err(refusal) => {
            if refusal != LinkRefusal::Internal {
                count_failure(&state, &visitor);
            }
            tracing::info!(?refusal, "link request refused");
            match refusal {
                LinkRefusal::Malformed => json_error(
                    StatusCode::BAD_REQUEST,
                    "malformed",
                    "signed parameters missing or malformed",
                ),
                LinkRefusal::Stale => json_error(
                    StatusCode::UNAUTHORIZED,
                    "stale_timestamp",
                    "the request is too old or too new",
                ),
                LinkRefusal::BadSignature => {
                    json_error(StatusCode::FORBIDDEN, "bad_signature", "the signature is wrong")
                }
                LinkRefusal::Unknown => {
                    json_error(StatusCode::NOT_FOUND, "unknown_gift", "no such gift here")
                }
                LinkRefusal::Gone => json_error(
                    StatusCode::GONE,
                    "gift_gone",
                    "the gift or its book is no longer available",
                ),
                LinkRefusal::Internal => {
                    json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "try again")
                }
            }
        }
    }
}

/// What a `Range` header asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ByteRange {
    /// The whole file (also for ranges we do not honour: other units, several ranges).
    Full,
    /// Bytes `start..=end`.
    Partial(u64, u64),
    /// Outside the file (`416`).
    Unsatisfiable,
}

/// Parses a single `bytes=` range against a file of `len` bytes.
pub(crate) fn parse_range(header: Option<&str>, len: u64) -> ByteRange {
    let Some(spec) = header.and_then(|h| h.trim().strip_prefix("bytes=")) else {
        return ByteRange::Full;
    };
    if spec.contains(',') {
        return ByteRange::Full;
    }
    let Some((first, last)) = spec.split_once('-') else { return ByteRange::Full };
    let number = |text: &str| text.trim().parse::<u64>().ok();
    match (first.trim().is_empty(), last.trim().is_empty()) {
        (true, false) => match number(last) {
            Some(0) => ByteRange::Unsatisfiable,
            Some(n) if len > 0 => ByteRange::Partial(len.saturating_sub(n), len - 1),
            Some(_) => ByteRange::Unsatisfiable,
            None => ByteRange::Full,
        },
        (false, true) => match number(first) {
            Some(start) if start < len => ByteRange::Partial(start, len - 1),
            Some(_) => ByteRange::Unsatisfiable,
            None => ByteRange::Full,
        },
        (false, false) => match (number(first), number(last)) {
            (Some(start), Some(end)) if start > end => ByteRange::Full,
            (Some(start), Some(end)) if start < len => ByteRange::Partial(start, end.min(len - 1)),
            (Some(_), Some(_)) => ByteRange::Unsatisfiable,
            _ => ByteRange::Full,
        },
        (true, true) => ByteRange::Full,
    }
}

/// RFC 8187 `attr-char`.
const ATTR_CHAR: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'#')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b'-')
    .remove(b'.')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'|')
    .remove(b'~');

/// `attachment; filename="ascii"; filename*=UTF-8''…` (RFC 6266).
pub(crate) fn content_disposition(name: &str) -> String {
    let ascii: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '_' })
        .collect();
    let mut value = format!("attachment; filename=\"{ascii}\"");
    if ascii != name {
        value.push_str("; filename*=UTF-8''");
        value.extend(utf8_percent_encode(name, ATTR_CHAR));
    }
    value
}

fn not_found_plain() -> Response {
    (StatusCode::NOT_FOUND, "this download link does not exist or has expired").into_response()
}

/// `GET|HEAD /f/{token}/{name}`: the file behind a live link, unchanged (byte-identical
/// to the upload), with `Content-Disposition`, `Content-Length`, `Accept-Ranges` and
/// single-range `Range` support. One delivery is recorded per download that starts at
/// byte 0.
pub(crate) async fn file(
    State(state): State<AppState>,
    visitor: Visitor,
    method: Method,
    Path((token, name)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if let Some(refused) = failure_budget(&state, &visitor) {
        return refused;
    }
    let link = (token.len() == 32 && token.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| state.link(&crypto::sha256_hex(token.to_ascii_lowercase().as_bytes())))
        .flatten()
        .filter(|link| link.file_name == name);
    let Some(link) = link else {
        count_failure(&state, &visitor);
        return not_found_plain();
    };
    let blob = match state.store.open_blob(&link.sha256) {
        Ok(Some(blob)) => blob,
        Ok(None) => return not_found_plain(),
        Err(error) => {
            tracing::error!(%error, "a stored file cannot be opened");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let len = blob.len();
    let quoted_etag = format!("\"{}\"", link.sha256);
    let text = |name: header::HeaderName| headers.get(name).and_then(|v| v.to_str().ok());
    let range_header = text(header::RANGE)
        .filter(|_| text(header::IF_RANGE).is_none_or(|v| v.trim() == quoted_etag));
    let (status, start, end) = match parse_range(range_header, len) {
        ByteRange::Full => (StatusCode::OK, 0, len.saturating_sub(1)),
        ByteRange::Partial(start, end) => (StatusCode::PARTIAL_CONTENT, start, end),
        ByteRange::Unsatisfiable => {
            let mut response = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
            if let Ok(value) = HeaderValue::from_str(&format!("bytes */{len}")) {
                response.headers_mut().insert(header::CONTENT_RANGE, value);
            }
            return response;
        }
    };
    let count = if len == 0 { 0 } else { end - start + 1 };
    let body = if method == Method::HEAD || count == 0 {
        Body::empty()
    } else {
        match blob {
            Blob::Memory(bytes) => {
                let (from, to) = (
                    usize::try_from(start).unwrap_or(usize::MAX),
                    usize::try_from(end).unwrap_or(usize::MAX),
                );
                match bytes.get(from..=to) {
                    Some(slice) => Body::from(slice.to_vec()),
                    None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            }
            Blob::File { path, .. } => {
                let opened = async {
                    let mut file = tokio::fs::File::open(&path).await?;
                    file.seek(std::io::SeekFrom::Start(start)).await?;
                    Ok::<_, std::io::Error>(file)
                };
                match opened.await {
                    Ok(file) => {
                        Body::from_stream(tokio_util::io::ReaderStream::new(file.take(count)))
                    }
                    Err(error) => {
                        tracing::error!(%error, "a stored file cannot be opened");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
            }
        }
    };
    if method == Method::GET && start == 0 {
        state.record_download(&link.gift_id, &visitor.client);
    }
    let mut response = (status, body).into_response();
    let out = response.headers_mut();
    let mut set = |name: header::HeaderName, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            out.insert(name, value);
        }
    };
    set(header::CONTENT_TYPE, link.media_type);
    set(header::CONTENT_LENGTH, &count.to_string());
    set(header::ACCEPT_RANGES, "bytes");
    set(header::ETAG, &quoted_etag);
    set(header::CACHE_CONTROL, "private, no-store");
    set(header::CONTENT_DISPOSITION, &content_disposition(&link.file_name));
    if status == StatusCode::PARTIAL_CONTENT {
        set(header::CONTENT_RANGE, &format!("bytes {start}-{end}/{len}"));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        assert_eq!(parse_range(None, 10), ByteRange::Full);
        assert_eq!(parse_range(Some("bytes=0-3"), 10), ByteRange::Partial(0, 3));
        assert_eq!(parse_range(Some("bytes=5-"), 10), ByteRange::Partial(5, 9));
        assert_eq!(parse_range(Some("bytes=-4"), 10), ByteRange::Partial(6, 9));
        assert_eq!(parse_range(Some("bytes=8-100"), 10), ByteRange::Partial(8, 9));
        assert_eq!(parse_range(Some("bytes=10-"), 10), ByteRange::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=0-1,4-5"), 10), ByteRange::Full);
        assert_eq!(parse_range(Some("items=0-1"), 10), ByteRange::Full);
        assert_eq!(parse_range(Some("bytes=x-y"), 10), ByteRange::Full);
    }

    #[test]
    fn content_disposition_is_ascii_with_a_utf8_variant() {
        assert_eq!(content_disposition("le-horla.epub"), "attachment; filename=\"le-horla.epub\"");
        assert_eq!(
            content_disposition("L'Étranger.epub"),
            "attachment; filename=\"L__tranger.epub\"; filename*=UTF-8''L%27%C3%89tranger.epub"
        );
        let hostile = content_disposition("a\"b\r\n.pdf");
        assert!(
            !hostile.contains('\r') && !hostile.contains('\n') && hostile.matches('"').count() == 2,
            "{hostile}"
        );
    }
}
