//! The redemption pages (`docs/api.md` §2): home, help, the gift page and its actions,
//! pairing. No JavaScript, no cookie; every answer is a page or a `303`.

use axum::extract::rejection::FormRejection;
use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use ebc_i18n::{Msg, t};
use ebc_model::{CodeError, Delivery, GiftCode, GiftStatus, Locale, PairCode};
use serde::Deserialize;

use super::{Visitor, html, see_other, with_retry_after};
use crate::device::DeviceClass;
use crate::mail;
use crate::names::ascii_file_name;
use crate::pages::{self, Chrome, GiftView, SendView};
use crate::rate::Verdict;
use crate::service::{ClaimError, CodeLookup};
use crate::state::AppState;
use crate::store::GiftRecord;
use crate::text::{S, s, sf};

/// Longest typed code echoed back into the form.
const MAX_ECHO_CHARS: usize = 64;

/// Query parameters of the pages.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct PageQuery {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    device: Option<String>,
}

/// Fields of the forms the pages post. All optional: a missing or unreadable body is
/// treated as an empty form.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ActionForm {
    #[serde(default)]
    device: Option<String>,
    #[serde(default)]
    opds: Option<String>,
    #[serde(default)]
    device_email: Option<String>,
    #[serde(default)]
    p: Option<String>,
}

type PostedForm = Result<Form<ActionForm>, FormRejection>;

fn form_of(posted: PostedForm) -> ActionForm {
    posted.map(|Form(form)| form).unwrap_or_default()
}

/// "Try again in n min." for a `Retry-After` in seconds.
fn wait_text(locale: Locale, retry_after_secs: u64) -> String {
    sf(locale, S::WaitMinutes, &[("minutes", &retry_after_secs.div_ceil(60).max(1).to_string())])
}

/// The `429` page.
fn too_many(chrome: Chrome<'_>, retry_after_secs: u64, pair: bool) -> Response {
    let errors = [
        t(chrome.locale, Msg::ErrorTooManyAttempts).to_owned(),
        wait_text(chrome.locale, retry_after_secs),
    ];
    let page =
        if pair { pages::pair_form(chrome, &errors) } else { pages::home(chrome, &errors, None) };
    with_retry_after(html(StatusCode::TOO_MANY_REQUESTS, page, None), retry_after_secs)
}

/// The lines of the `404` page for an unknown code, given the attempts left.
fn unknown_lines(locale: Locale, first: &str, remaining: Option<u32>) -> Vec<String> {
    let mut lines = vec![first.to_owned()];
    if let Some(left) = remaining {
        lines.push(sf(locale, S::AttemptsLeft, &[("count", &left.to_string())]));
        if left == 0 {
            lines.push(wait_text(locale, crate::rate::WINDOW_SECS.unsigned_abs()));
        }
    }
    lines
}

/// Parses what a recipient typed or pasted: a code, or a whole claim URL.
fn parse_code(raw: &str) -> Result<GiftCode, CodeError> {
    match GiftCode::parse(raw) {
        Err(CodeError::InvalidChar(_)) if raw.contains('/') => GiftCode::from_claim_url(raw),
        other => other,
    }
}

/// The `400` page for a mistyped code: not an attempt, says so.
fn typo(chrome: Chrome<'_>, error: &CodeError, raw: &str) -> Response {
    let locale = chrome.locale;
    let first = match error {
        CodeError::Check => t(locale, Msg::ErrorCodeTypo),
        _ => s(locale, S::CodeFormat),
    };
    let echo: String = raw.chars().filter(|c| !c.is_control()).take(MAX_ECHO_CHARS).collect();
    let errors = [first.to_owned(), s(locale, S::NotCounted).to_owned()];
    html(StatusCode::BAD_REQUEST, pages::home(chrome, &errors, Some(&echo)), None)
}

/// Finds the gift a typed code stands for, or the page that says why there is none:
/// `429` when the client is out of attempts (nothing is looked up), `400` for a typo (not
/// counted), `410` for a voided, cancelled or expired code, `404` for an unknown one
/// (counted, with the attempts left).
fn resolve(
    state: &AppState,
    visitor: &Visitor,
    raw: &str,
    device: Option<&str>,
) -> Result<(GiftRecord, GiftCode), Box<Response>> {
    let now = state.now();
    let chrome = visitor.chrome(state, visitor.locale(), device);
    if let Verdict::Blocked { retry_after_secs, .. } =
        state.code_limiter.check(&visitor.client, now)
    {
        return Err(Box::new(too_many(chrome, retry_after_secs, false)));
    }
    let code = parse_code(raw).map_err(|error| Box::new(typo(chrome, &error, raw)))?;
    let gone = |chrome: Chrome<'_>, text: &str| {
        let title = t(chrome.locale, Msg::RedeemTitle);
        Box::new(html(
            StatusCode::GONE,
            pages::message(chrome, title, &[text.to_owned()], None),
            None,
        ))
    };
    match state.lookup_code(&code) {
        CodeLookup::Gift(gift) => {
            let chrome = visitor.chrome(state, gift.locale, device);
            match gift.status_at(now) {
                GiftStatus::Cancelled => Err(gone(chrome, t(chrome.locale, Msg::ErrorGiftRevoked))),
                GiftStatus::Expired => Err(gone(chrome, t(chrome.locale, Msg::ErrorGiftExpired))),
                GiftStatus::Reissued => Err(gone(chrome, s(chrome.locale, S::Reissued))),
                _ => Ok((*gift, code)),
            }
        }
        CodeLookup::Voided => Err(gone(chrome, s(chrome.locale, S::Reissued))),
        CodeLookup::Unknown => match state.code_limiter.record_failure(&visitor.client, now) {
            Verdict::Blocked { retry_after_secs, .. } => {
                Err(Box::new(too_many(chrome, retry_after_secs, false)))
            }
            Verdict::Allowed { remaining } => {
                let lines = unknown_lines(
                    chrome.locale,
                    t(chrome.locale, Msg::ErrorCodeUnknown),
                    remaining,
                );
                Err(Box::new(html(StatusCode::NOT_FOUND, pages::home(chrome, &lines, None), None)))
            }
        },
    }
}

/// The origin (`scheme://host[:port]`) of a URL.
fn origin_of(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    Some(match parsed.port() {
        Some(port) => format!("{}://{host}:{port}", parsed.scheme()),
        None => format!("{}://{host}", parsed.scheme()),
    })
}

/// Renders the gift page with `status`.
fn gift_response(
    state: &AppState,
    visitor: &Visitor,
    gift: &GiftRecord,
    code: &GiftCode,
    device: Option<&str>,
    status: StatusCode,
    notices: &[String],
) -> Response {
    let now = state.now();
    let chrome = visitor.chrome(state, gift.locale, device);
    let partner = state.partner_of(gift);
    let store_origin = match &gift.delivery {
        Delivery::ResellerRedirect { url } => {
            partner.filter(|p| p.allows_url(url)).and_then(|_| origin_of(url))
        }
        _ => None,
    };
    let send =
        state.config.smtp_url.as_ref().filter(|_| !state.config.send_domains.is_empty()).map(
            |_| SendView {
                from: &state.config.send_from,
                domains: state
                    .config
                    .send_domains
                    .iter()
                    .map(|d| format!("@{d}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            },
        );
    let view = GiftView {
        chrome,
        gift,
        code,
        status: gift.status_at(now),
        revealed: gift.is_revealed_at(now),
        store_name: partner.map_or("", |p| p.display_name()),
        store_home: store_origin.as_ref().map(|origin| format!("{origin}/")),
        send,
        notices,
    };
    html(status, pages::gift(&view), store_origin.as_deref())
}

/// The page for a claim that could not go ahead.
fn claim_failed(
    state: &AppState,
    visitor: &Visitor,
    gift_id: &str,
    code: &GiftCode,
    device: Option<&str>,
    error: &ClaimError,
) -> Response {
    let Some(gift) = state.gift(gift_id) else {
        let chrome = visitor.chrome(state, visitor.locale(), device);
        let page = pages::message(
            chrome,
            t(chrome.locale, Msg::RedeemTitle),
            &[t(chrome.locale, Msg::ErrorGiftRevoked).to_owned()],
            None,
        );
        return html(StatusCode::GONE, page, None);
    };
    let locale = gift.locale;
    let chrome = visitor.chrome(state, locale, device);
    let notice = match error {
        ClaimError::Gone(status) => {
            let text = if *status == GiftStatus::Expired {
                Msg::ErrorGiftExpired
            } else {
                Msg::ErrorGiftRevoked
            };
            let page = pages::message(
                chrome,
                t(locale, Msg::RedeemTitle),
                &[t(locale, text).to_owned()],
                None,
            );
            return html(StatusCode::GONE, page, None);
        }
        // The page itself says why: the reveal date, or that the limit is reached.
        ClaimError::NotRevealed | ClaimError::Exhausted => None,
        ClaimError::NotAvailable => Some(s(locale, S::NotAvailable)),
        ClaimError::StoreUnavailable => Some(s(locale, S::StoreUnavailable)),
        ClaimError::SendLimit => Some(s(locale, S::SendLimit)),
        ClaimError::Internal => Some(t(locale, Msg::ErrorGeneric)),
    };
    let notices: Vec<String> = notice.into_iter().map(str::to_owned).collect();
    gift_response(state, visitor, &gift, code, device, error.status(), &notices)
}

/// `GET /`.
pub(crate) async fn home(
    State(state): State<AppState>,
    visitor: Visitor,
    Query(query): Query<PageQuery>,
) -> Response {
    let chrome = visitor.chrome(&state, visitor.locale(), query.device.as_deref());
    html(StatusCode::OK, pages::home(chrome, &[], None), None)
}

/// `GET /aide`.
pub(crate) async fn help(
    State(state): State<AppState>,
    visitor: Visitor,
    Query(query): Query<PageQuery>,
) -> Response {
    let chrome = visitor.chrome(&state, visitor.locale(), query.device.as_deref());
    html(StatusCode::OK, pages::help(chrome), None)
}

/// `GET /k?code=…`: normalises the typed code and redirects to its gift page.
pub(crate) async fn code_query(
    State(state): State<AppState>,
    visitor: Visitor,
    Query(query): Query<PageQuery>,
) -> Response {
    let chrome = visitor.chrome(&state, visitor.locale(), query.device.as_deref());
    let raw = query.code.as_deref().map(str::trim).unwrap_or("");
    if raw.is_empty() {
        return see_other(&chrome_link(chrome, "/"));
    }
    if let Verdict::Blocked { retry_after_secs, .. } =
        state.code_limiter.check(&visitor.client, state.now())
    {
        return too_many(chrome, retry_after_secs, false);
    }
    match parse_code(raw) {
        Ok(code) => see_other(&chrome_link(chrome, &format!("/k/{}", code.compact()))),
        Err(error) => typo(chrome, &error, raw),
    }
}

/// `path` with the explicitly chosen device, if any.
fn chrome_link(chrome: Chrome<'_>, path: &str) -> String {
    match chrome.chosen.and_then(DeviceClass::choice) {
        Some(choice) => format!("{path}?device={choice}"),
        None => path.to_owned(),
    }
}

/// `GET /k/{code}` (and `/K/{code}`): the gift page.
pub(crate) async fn gift_page(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(raw): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    let device = query.device.as_deref();
    match resolve(&state, &visitor, &raw, device) {
        Ok((gift, code)) => {
            gift_response(&state, &visitor, &gift, &code, device, StatusCode::OK, &[])
        }
        Err(page) => *page,
    }
}

/// `POST /k/{code}/download`: mints a download session and redirects to the file (or,
/// with `opds=1`, to the OPDS feed of the session).
pub(crate) async fn download(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(raw): Path<String>,
    posted: PostedForm,
) -> Response {
    let form = form_of(posted);
    let device = form.device.as_deref();
    let (gift, code) = match resolve(&state, &visitor, &raw, device) {
        Ok(found) => found,
        Err(page) => return *page,
    };
    if !matches!(gift.delivery, Delivery::HostedFile { .. }) {
        return claim_failed(&state, &visitor, &gift.id, &code, device, &ClaimError::NotAvailable);
    }
    let chrome = visitor.chrome(&state, gift.locale, device);
    match state.open_session(&gift.id, chrome.device).await {
        Ok(ticket) if form.opds.as_deref() == Some("1") => {
            see_other(&format!("/o/{}/catalog.xml", ticket.token))
        }
        Ok(ticket) => see_other(&ticket.download_path()),
        Err(error) => claim_failed(&state, &visitor, &gift.id, &code, device, &error),
    }
}

/// `POST /k/{code}/redeem`: to the store with signed parameters (`reseller_redirect`), or
/// to a download session serving the licence (`reseller_licence`).
pub(crate) async fn redeem(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(raw): Path<String>,
    posted: PostedForm,
) -> Response {
    let form = form_of(posted);
    let device = form.device.as_deref();
    let (gift, code) = match resolve(&state, &visitor, &raw, device) {
        Ok(found) => found,
        Err(page) => return *page,
    };
    let outcome = match &gift.delivery {
        Delivery::ResellerRedirect { .. } => {
            state.redeem_redirect(&gift).map(|url| url.to_string())
        }
        Delivery::ResellerLicence { .. } => {
            let chrome = visitor.chrome(&state, gift.locale, device);
            state.open_session(&gift.id, chrome.device).await.map(|ticket| ticket.download_path())
        }
        Delivery::HostedFile { .. } => Err(ClaimError::NotAvailable),
    };
    match outcome {
        Ok(location) => see_other(&location),
        Err(error) => claim_failed(&state, &visitor, &gift.id, &code, device, &error),
    }
}

/// `POST /k/{code}/pair`: creates the pair code and shows it.
pub(crate) async fn pair(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(raw): Path<String>,
    posted: PostedForm,
) -> Response {
    let form = form_of(posted);
    let device = form.device.as_deref();
    let (gift, code) = match resolve(&state, &visitor, &raw, device) {
        Ok(found) => found,
        Err(page) => return *page,
    };
    match state.create_pair(&gift) {
        Ok((pair, expires_at)) => {
            let chrome = visitor.chrome(&state, gift.locale, device);
            let page = pages::pair_created(
                chrome,
                &code,
                &pair.grouped(),
                expires_at,
                state.config.pair_ttl_secs,
            );
            html(StatusCode::OK, page, None)
        }
        Err(error) => claim_failed(&state, &visitor, &gift.id, &code, device, &error),
    }
}

/// `GET /e`: the pair-code form, on the e-reader.
pub(crate) async fn pair_form(
    State(state): State<AppState>,
    visitor: Visitor,
    Query(query): Query<PageQuery>,
) -> Response {
    let chrome = visitor.chrome(&state, visitor.locale(), query.device.as_deref());
    html(StatusCode::OK, pages::pair_form(chrome, &[]), None)
}

/// `POST /e`: validates the pair code and redirects to the file. Wrong codes count
/// against the per-client and the global failure budgets.
pub(crate) async fn pair_submit(
    State(state): State<AppState>,
    visitor: Visitor,
    posted: PostedForm,
) -> Response {
    let form = form_of(posted);
    let device = form.device.as_deref();
    let chrome = visitor.chrome(&state, visitor.locale(), device);
    let locale = chrome.locale;
    let now = state.now();
    if let Verdict::Blocked { retry_after_secs, .. } =
        state.code_limiter.check(&visitor.client, now)
    {
        return too_many(chrome, retry_after_secs, true);
    }
    let Ok(code) = PairCode::parse(form.p.as_deref().unwrap_or("")) else {
        let errors = [s(locale, S::PairFormat).to_owned(), s(locale, S::NotCounted).to_owned()];
        return html(StatusCode::BAD_REQUEST, pages::pair_form(chrome, &errors), None);
    };
    let gift_id = match state.take_pair(&code) {
        Ok(Some(gift_id)) => gift_id,
        Ok(None) => {
            return match state.code_limiter.record_failure(&visitor.client, now) {
                Verdict::Blocked { retry_after_secs, .. } => {
                    too_many(chrome, retry_after_secs, true)
                }
                Verdict::Allowed { remaining } => {
                    let lines = unknown_lines(locale, s(locale, S::PairUnknown), remaining);
                    html(StatusCode::NOT_FOUND, pages::pair_form(chrome, &lines), None)
                }
            };
        }
        Err(error) => {
            tracing::error!(%error, "pair code lookup failed");
            let errors = [t(locale, Msg::ErrorGeneric).to_owned()];
            return html(
                StatusCode::INTERNAL_SERVER_ERROR,
                pages::pair_form(chrome, &errors),
                None,
            );
        }
    };
    match state.open_session(&gift_id, chrome.device).await {
        Ok(ticket) => see_other(&ticket.download_path()),
        Err(error) => {
            let text = match error {
                ClaimError::StoreUnavailable => s(locale, S::StoreUnavailable),
                ClaimError::Exhausted => s(locale, S::Exhausted),
                ClaimError::Gone(GiftStatus::Expired) => t(locale, Msg::ErrorGiftExpired),
                ClaimError::Gone(_) => t(locale, Msg::ErrorGiftRevoked),
                _ => t(locale, Msg::ErrorGeneric),
            };
            html(error.status(), pages::pair_form(chrome, &[text.to_owned()]), None)
        }
    }
}

/// `POST /k/{code}/send`: e-mails the hosted file to a device address.
pub(crate) async fn send(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(raw): Path<String>,
    posted: PostedForm,
) -> Response {
    let form = form_of(posted);
    let device = form.device.as_deref();
    let (gift, code) = match resolve(&state, &visitor, &raw, device) {
        Ok(found) => found,
        Err(page) => return *page,
    };
    let chrome = visitor.chrome(&state, gift.locale, device);
    let locale = chrome.locale;
    let title = t(locale, Msg::SendToEreaderCta);
    let result = |status: StatusCode, text: &str, ok: bool| {
        html(status, pages::send_result(chrome, &code, title, text, ok), None)
    };

    let (Some(smtp_url), Delivery::HostedFile { sha256, media_type, file_name, .. }) =
        (&state.config.smtp_url, &gift.delivery)
    else {
        return claim_failed(&state, &visitor, &gift.id, &code, device, &ClaimError::NotAvailable);
    };
    let Some(to) = mail::parse_device_address(
        form.device_email.as_deref().unwrap_or(""),
        &state.config.send_domains,
    ) else {
        let domains = state
            .config
            .send_domains
            .iter()
            .map(|d| format!("@{d}"))
            .collect::<Vec<_>>()
            .join(", ");
        return result(
            StatusCode::BAD_REQUEST,
            &sf(locale, S::SendBadAddress, &[("domains", &domains)]),
            false,
        );
    };
    if let Err(error) = state.can_send(&gift) {
        return claim_failed(&state, &visitor, &gift.id, &code, device, &error);
    }
    let bytes = match state.read_blob(sha256).await {
        Ok(Some(bytes)) => bytes,
        other => {
            tracing::error!(gift = %gift.id, missing = other.is_ok(), "the hosted file cannot be read");
            return result(StatusCode::INTERNAL_SERVER_ERROR, t(locale, Msg::ErrorGeneric), false);
        }
    };
    let Ok(message_token) = crate::codes::random_hex(state.entropy.as_ref(), 16) else {
        return result(StatusCode::INTERNAL_SERVER_ERROR, t(locale, Msg::ErrorGeneric), false);
    };
    // Send-to-PocketBook only takes short ASCII names without commas; a Kindle shows the
    // title from the file itself. The URL name already has that shape.
    let subject =
        file_name.rsplit_once('.').map_or(file_name.as_str(), |(stem, _)| stem).to_owned();
    let outgoing = mail::Outgoing {
        smtp_url,
        from: &state.config.send_from,
        to,
        subject,
        text: sf(locale, S::MailBody, &[("title", &gift.book.title)]),
        attachment_name: ascii_file_name(
            &gift.book.title,
            file_name.rsplit_once('.').map_or("epub", |(_, ext)| ext),
        ),
        media_type,
        bytes: bytes.to_vec(),
        now_unix: state.now(),
        message_token,
    };
    let timeout = std::time::Duration::from_millis(state.config.outbound_timeout_ms.max(1));
    if let Err(error) = mail::send(outgoing, timeout).await {
        // Only the kind of failure is logged: SMTP error texts may quote the address.
        tracing::warn!(gift = %gift.id, kind = error.kind(), "send-to-device failed");
        return result(StatusCode::BAD_GATEWAY, s(locale, S::SendFailed), false);
    }
    if let Err(error) = state.record_send(&gift.id) {
        tracing::warn!(gift = %gift.id, ?error, "a sent file could not be counted");
    }
    result(StatusCode::OK, t(locale, Msg::SentConfirmation), true)
}
