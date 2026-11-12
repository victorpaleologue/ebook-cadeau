//! Download sessions: the file itself (`GET|HEAD /d/{session}/{file-name}`) and the
//! OPDS 1.2 / 2.0 feeds of the one publication (`/o/{session}/catalog.*`).
//!
//! A `partner_file` session answers `303 See Other` to a short-lived download link of the
//! partner (the service holds no book); a `reseller_licence` session serves the licence
//! stored for it, with `Range`.
//!
//! The session token is the only credential; it is looked up by its hash. Nothing in the
//! path is ever used as a file-system name: the file served is the blob recorded in the
//! session.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use ebc_i18n::{Msg, t};
use ebc_model::GiftStatus;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::{Visitor, html, see_other};
use crate::codes;
use crate::device::DeviceClass;
use crate::names::content_disposition;
use crate::pages;
use crate::service::ClaimError;
use crate::state::AppState;
use crate::store::{Blob, GiftRecord, SessionRecord};
use crate::text::{S, s};

/// `GET /static/eink.css`.
pub(crate) async fn style() -> Response {
    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/css; charset=utf-8")),
            (header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400")),
        ],
        pages::STYLE,
    )
        .into_response()
}

/// The live session behind a token, or the page saying the link is dead (`404` for a
/// token that never existed or was purged, `410` for one that has just expired).
fn session_of(
    state: &AppState,
    visitor: &Visitor,
    token: &str,
) -> Result<SessionRecord, Box<Response>> {
    let dead = |status: StatusCode| {
        let chrome = visitor.chrome(state, visitor.locale(), None);
        let locale = chrome.locale;
        let page = pages::message(
            chrome,
            t(locale, Msg::RedeemTitle),
            &[s(locale, S::SessionExpired).to_owned()],
            Some(("/", s(locale, S::EnterCode))),
        );
        Box::new(html(status, page, None))
    };
    // Tokens are 32 hexadecimal characters; anything else cannot be one.
    if token.len() != 32 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(dead(StatusCode::NOT_FOUND));
    }
    let key = codes::token_hash(token);
    match state.db.view(|db| db.sessions.get(&key).cloned()) {
        Some(session) if session.expires_at > state.now() => Ok(session),
        Some(_) => Err(dead(StatusCode::GONE)),
        None => Err(dead(StatusCode::NOT_FOUND)),
    }
}

/// What a `Range` header asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ByteRange {
    /// The whole file (no header, or one we do not honour: other units, several ranges,
    /// malformed values — RFC 9110 lets a server ignore `Range`).
    Full,
    /// Bytes `start..=end`.
    Partial(u64, u64),
    /// A syntactically valid range outside the file (`416`).
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
        // `-n`: the last n bytes.
        (true, false) => match number(last) {
            Some(0) => ByteRange::Unsatisfiable,
            Some(n) if len > 0 => ByteRange::Partial(len.saturating_sub(n), len - 1),
            Some(_) => ByteRange::Unsatisfiable,
            None => ByteRange::Full,
        },
        // `a-`: from a to the end.
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

/// Serves `blob` (or the part of it the request asks for) with download headers.
async fn serve_blob(
    blob: Blob,
    method: &Method,
    request: &HeaderMap,
    media_type: &str,
    etag: &str,
    disposition: Option<&str>,
) -> Response {
    let len = blob.len();
    let quoted_etag = format!("\"{etag}\"");
    let text = |name: header::HeaderName| request.get(name).and_then(|v| v.to_str().ok());
    // `If-Range` with another validator: the client's partial copy is of another file.
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
    let body = if *method == Method::HEAD || count == 0 {
        Body::empty()
    } else {
        match blob {
            Blob::Memory(bytes) => {
                let (from, to) = (
                    usize::try_from(start).unwrap_or(usize::MAX),
                    usize::try_from(end).unwrap_or(usize::MAX),
                );
                match bytes.get(from..=to) {
                    Some(_) => Body::from(bytes.slice(from..=to)),
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
    let mut response = (status, body).into_response();
    let headers = response.headers_mut();
    let mut set = |name: header::HeaderName, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(name, value);
        }
    };
    set(header::CONTENT_TYPE, media_type);
    set(header::CONTENT_LENGTH, &count.to_string());
    set(header::ACCEPT_RANGES, "bytes");
    set(header::ETAG, &quoted_etag);
    set(header::CACHE_CONTROL, "private, no-store");
    if status == StatusCode::PARTIAL_CONTENT {
        set(header::CONTENT_RANGE, &format!("bytes {start}-{end}/{len}"));
    }
    if let Some(disposition) = disposition {
        set(header::CONTENT_DISPOSITION, disposition);
    }
    response
}

/// `GET|HEAD /d/{session}/{file-name}`: the file of a download session.
///
/// A partner file: `303` to the partner's download link (the one obtained when the
/// session was opened while it is fresh, else a new one), `502` when the partner gives
/// none. The core never relays the bytes.
///
/// `Content-Disposition` always carries the ASCII `filename=`; the UTF-8 `filename*` is
/// added only for phones and computers, because old e-reader browsers mishandle it
/// (`docs/research/05-ereader-distribution-ux.md`, implication 34).
pub(crate) async fn download(
    State(state): State<AppState>,
    visitor: Visitor,
    method: Method,
    Path((token, name)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let session = match session_of(&state, &visitor, &token) {
        Ok(session) => session,
        Err(page) => return *page,
    };
    if name != session.file_name {
        return StatusCode::NOT_FOUND.into_response();
    }
    if session.partner_file {
        let key = codes::token_hash(&token);
        return match state.partner_download(&key, &session).await {
            Ok(url) => see_other(url.as_str()),
            Err(error) => {
                let chrome = visitor.chrome(&state, visitor.locale(), None);
                let locale = chrome.locale;
                let text = match error {
                    ClaimError::Gone(GiftStatus::Expired) => t(locale, Msg::ErrorGiftExpired),
                    ClaimError::Gone(_) => t(locale, Msg::ErrorGiftRevoked),
                    _ => s(locale, S::StoreUnavailable),
                };
                let page = pages::message(
                    chrome,
                    t(locale, Msg::RedeemTitle),
                    &[text.to_owned()],
                    Some(("/", s(locale, S::EnterCode))),
                );
                html(error.status(), page, None)
            }
        };
    }
    let blob = match state.db.store().open_blob(&session.sha256) {
        Ok(Some(blob)) => blob,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, "a stored file cannot be opened");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let chosen_eink =
        DeviceClass::ALL.iter().any(|class| class.is_eink() && class.as_str() == session.device);
    let display =
        (!chosen_eink && !visitor.detected.is_eink()).then_some(session.display_name.as_str());
    let disposition = content_disposition(&session.file_name, display);
    serve_blob(blob, &method, &headers, &session.media_type, &session.sha256, Some(&disposition))
        .await
}

/// `GET /o/{session}/cover`: the cover the partner supplied, for OPDS clients.
pub(crate) async fn opds_cover(
    State(state): State<AppState>,
    visitor: Visitor,
    method: Method,
    Path(token): Path<String>,
    headers: HeaderMap,
) -> Response {
    let session = match session_of(&state, &visitor, &token) {
        Ok(session) => session,
        Err(page) => return *page,
    };
    let Some(cover) = state.gift(&session.gift_id).and_then(|gift| gift.cover) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match state.db.store().open_blob(&cover.sha256) {
        Ok(Some(blob)) => {
            serve_blob(blob, &method, &headers, &cover.media_type, &cover.sha256, None).await
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Escapes text for XML content and attribute values; characters XML 1.0 forbids are
/// dropped.
pub(crate) fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) < 0x20 || matches!(c as u32, 0xFFFE | 0xFFFF) => {}
            c => out.push(c),
        }
    }
    out
}

/// RFC 3339 form of a Unix time, in UTC.
pub(crate) fn rfc3339(unix: i64) -> String {
    let (year, month, day) = ebc_i18n::date_from_unix(unix);
    let seconds = unix.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
}

/// The identifier of the publication: its ISBN as a URN, or one derived from the gift.
fn publication_id(gift: &GiftRecord) -> String {
    match &gift.book.isbn13 {
        Some(isbn) => format!("urn:isbn:{}", isbn.to_plain()),
        None => format!("urn:ebook-cadeau:gift:{}", gift.id),
    }
}

/// The gift a session belongs to, with the session; `404` when either is gone.
fn feed_source(
    state: &AppState,
    visitor: &Visitor,
    token: &str,
) -> Result<(SessionRecord, GiftRecord), Box<Response>> {
    let session = session_of(state, visitor, token)?;
    match state.gift(&session.gift_id) {
        Some(gift) => Ok((session, gift)),
        None => Err(Box::new(StatusCode::NOT_FOUND.into_response())),
    }
}

/// `GET /o/{session}/catalog.xml`: OPDS 1.2 acquisition feed with the one publication.
pub(crate) async fn opds_atom(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(token): Path<String>,
) -> Response {
    let (session, gift) = match feed_source(&state, &visitor, &token) {
        Ok(found) => found,
        Err(response) => return *response,
    };
    let updated = rfc3339(gift.created_at);
    let acquisition_type = "application/atom+xml;profile=opds-catalog;kind=acquisition";
    let mut xml = String::with_capacity(2048);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(
        "<feed xmlns=\"http://www.w3.org/2005/Atom\" xmlns:dc=\"http://purl.org/dc/terms/\" \
         xmlns:opds=\"http://opds-spec.org/2010/catalog\">\n",
    );
    xml.push_str(&format!("  <id>urn:ebook-cadeau:feed:{}</id>\n", xml_escape(&gift.id)));
    xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&gift.book.title)));
    xml.push_str(&format!("  <updated>{updated}</updated>\n"));
    xml.push_str("  <author><name>eBook Cadeau</name></author>\n");
    for rel in ["self", "start"] {
        xml.push_str(&format!(
            "  <link rel=\"{rel}\" href=\"/o/{token}/catalog.xml\" type=\"{acquisition_type}\"/>\n"
        ));
    }
    xml.push_str("  <entry>\n");
    xml.push_str(&format!("    <title>{}</title>\n", xml_escape(&gift.book.title)));
    xml.push_str(&format!("    <id>{}</id>\n", xml_escape(&publication_id(&gift))));
    xml.push_str(&format!("    <updated>{updated}</updated>\n"));
    for author in gift.book.authors() {
        xml.push_str(&format!("    <author><name>{}</name></author>\n", xml_escape(author)));
    }
    if let Some(isbn) = &gift.book.isbn13 {
        xml.push_str(&format!("    <dc:identifier>urn:isbn:{}</dc:identifier>\n", isbn.to_plain()));
    }
    if let Some(language) = &gift.book.language {
        xml.push_str(&format!("    <dc:language>{}</dc:language>\n", xml_escape(language)));
    }
    if let Some(publisher) = &gift.book.publisher {
        xml.push_str(&format!("    <dc:publisher>{}</dc:publisher>\n", xml_escape(publisher)));
    }
    if let Some(cover) = &gift.cover {
        xml.push_str(&format!(
            "    <link rel=\"http://opds-spec.org/image\" href=\"/o/{token}/cover\" type=\"{}\"/>\n",
            xml_escape(&cover.media_type)
        ));
    }
    // `length` only when known: a partner file may not announce its size.
    let length = match session.size_bytes {
        0 => String::new(),
        size => format!(" length=\"{size}\""),
    };
    xml.push_str(&format!(
        "    <link rel=\"http://opds-spec.org/acquisition\" href=\"/d/{token}/{}\" type=\"{}\"{length}/>\n",
        xml_escape(&session.file_name),
        xml_escape(&session.media_type),
    ));
    xml.push_str("  </entry>\n</feed>\n");
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/atom+xml;profile=opds-catalog;kind=acquisition"),
        )],
        xml,
    )
        .into_response()
}

/// `GET /o/{session}/catalog.json`: OPDS 2.0 feed with the one publication.
///
/// OPDS 2.0 asks every publication for an `images` collection; it is present only when
/// the partner supplied a cover (the server has no image of its own to offer).
pub(crate) async fn opds_json(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(token): Path<String>,
) -> Response {
    let (session, gift) = match feed_source(&state, &visitor, &token) {
        Ok(found) => found,
        Err(response) => return *response,
    };
    let mut metadata = serde_json::json!({
        "@type": "http://schema.org/Book",
        "title": gift.book.title,
        "identifier": publication_id(&gift),
        "modified": rfc3339(gift.created_at),
    });
    if let Some(object) = metadata.as_object_mut() {
        let authors: Vec<&str> = gift.book.authors().collect();
        if !authors.is_empty() {
            object.insert("author".to_owned(), authors.into());
        }
        if let Some(language) = &gift.book.language {
            object.insert("language".to_owned(), language.clone().into());
        }
        if let Some(publisher) = &gift.book.publisher {
            object.insert("publisher".to_owned(), publisher.clone().into());
        }
    }
    let mut publication = serde_json::json!({
        "metadata": metadata,
        "links": [{
            "rel": "http://opds-spec.org/acquisition",
            "href": format!("/d/{token}/{}", session.file_name),
            "type": session.media_type,
        }],
    });
    if let (Some(cover), Some(object)) = (&gift.cover, publication.as_object_mut()) {
        object.insert(
            "images".to_owned(),
            serde_json::json!([{ "href": format!("/o/{token}/cover"), "type": cover.media_type }]),
        );
    }
    let feed = serde_json::json!({
        "metadata": { "title": gift.book.title },
        "links": [{ "rel": "self", "href": format!("/o/{token}/catalog.json"), "type": "application/opds+json" }],
        "publications": [publication],
    });
    ([(header::CONTENT_TYPE, HeaderValue::from_static("application/opds+json"))], feed.to_string())
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_byte_ranges_are_resolved_against_the_file() {
        use ByteRange::{Full, Partial, Unsatisfiable};
        let cases: &[(Option<&str>, ByteRange)] = &[
            (None, Full),
            (Some("bytes=0-0"), Partial(0, 0)),
            (Some("bytes=0-99"), Partial(0, 99)),
            (Some("bytes=10-19"), Partial(10, 19)),
            (Some("bytes=90-500"), Partial(90, 99)),
            (Some("bytes=95-"), Partial(95, 99)),
            (Some("bytes=-5"), Partial(95, 99)),
            (Some("bytes=-500"), Partial(0, 99)),
            (Some(" bytes=1-2 "), Partial(1, 2)),
            (Some("bytes=100-"), Unsatisfiable),
            (Some("bytes=100-200"), Unsatisfiable),
            (Some("bytes=-0"), Unsatisfiable),
            // Ignored: other units, several ranges, nonsense.
            (Some("items=0-5"), Full),
            (Some("bytes=0-1,5-6"), Full),
            (Some("bytes=5-1"), Full),
            (Some("bytes=a-b"), Full),
            (Some("bytes=-"), Full),
            (Some("bytes="), Full),
            (Some("bytes=99999999999999999999999-"), Full),
            (Some("bytes=-18446744073709551615"), Partial(0, 99)),
        ];
        for (header, expected) in cases {
            assert_eq!(parse_range(*header, 100), *expected, "{header:?}");
        }
        assert_eq!(parse_range(Some("bytes=0-"), 0), Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=-1"), 0), Unsatisfiable);
    }

    #[test]
    fn xml_text_is_escaped_and_forbidden_characters_dropped() {
        assert_eq!(
            xml_escape("Tom & \"Jerry\" <b>'s</b>"),
            "Tom &amp; &quot;Jerry&quot; &lt;b&gt;&apos;s&lt;/b&gt;"
        );
        assert_eq!(xml_escape("a\u{0}b\u{1b}c\u{ffff}d\né"), "abcd\né");
    }

    #[test]
    fn timestamps_are_rfc_3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(1_794_441_600 + 3_723), "2026-11-12T01:02:03Z");
    }
}
