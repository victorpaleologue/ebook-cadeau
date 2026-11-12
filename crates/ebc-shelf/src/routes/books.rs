//! Books and their gifts: upload, book page, gift creation, cancellation, reissue, card
//! PDFs (fetched from the core's partner API) and deletion.

use std::collections::HashMap;

use axum::Form;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::{self, HeaderValue};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::{Visitor, bad_form, forbidden, not_found, page, see_other, server_error};
use crate::pages::{self, UploadValues};
use crate::service::{GiftError, GiftForm, UploadForm, UploadRefusal, may_publish};
use crate::state::AppState;
use crate::store::{Account, Role};
use crate::text::{S, sf};

/// Longest text field of the upload form, in bytes.
const MAX_FIELD_BYTES: usize = 10 * 1024;

/// `GET /livres/nouveau`.
pub(crate) async fn upload_page(State(state): State<AppState>, visitor: Visitor) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    if !may_publish(account) {
        return see_other("/compte");
    }
    page(StatusCode::OK, pages::upload_form(visitor.chrome(&state), &UploadValues::default(), None))
}

/// Reads the multipart upload form; the file is read up to the size limit plus one byte
/// (so that a larger file is recognised as such without being kept).
async fn read_upload(
    mut multipart: Multipart,
    max_file: u64,
) -> Result<(UploadForm, Option<String>, bool), ()> {
    let mut form = UploadForm::default();
    let mut csrf = None;
    let mut too_large = false;
    let cap = usize::try_from(max_file.saturating_add(1)).unwrap_or(usize::MAX);
    while let Some(mut field) = multipart.next_field().await.map_err(|_| ())? {
        let name = field.name().unwrap_or("").to_owned();
        if name == "file" {
            form.file_name = field.file_name().unwrap_or("").chars().take(255).collect();
            let mut bytes = Vec::new();
            while let Some(chunk) = field.chunk().await.map_err(|_| ())? {
                if bytes.len() + chunk.len() > cap {
                    too_large = true;
                    break;
                }
                bytes.extend_from_slice(&chunk);
            }
            if too_large {
                break;
            }
            form.bytes = bytes;
            continue;
        }
        let mut value = Vec::new();
        while let Some(chunk) = field.chunk().await.map_err(|_| ())? {
            if value.len() + chunk.len() > MAX_FIELD_BYTES {
                return Err(());
            }
            value.extend_from_slice(&chunk);
        }
        let value = String::from_utf8(value).map_err(|_| ())?;
        match name.as_str() {
            "csrf" => csrf = Some(value),
            "title" => form.title = value,
            "author" => form.author = value,
            "isbn" => form.isbn = value,
            "language" => form.language = value,
            "basis" => form.basis = value,
            "death_year" => form.death_year = value,
            "licence" => form.licence = value,
            "licensor" => form.licensor = value,
            "declared" => form.declared = value == "1",
            _ => {}
        }
    }
    Ok((form, csrf, too_large))
}

/// `POST /livres` (multipart): the upload gate, then the book page.
pub(crate) async fn upload(
    State(state): State<AppState>,
    visitor: Visitor,
    multipart: Multipart,
) -> Response {
    let Some(account) = visitor.account.clone() else {
        return see_other("/connexion");
    };
    // Each upload is held in memory while it is checked: bound how many run at once.
    let Ok(_permit) = state.uploads.try_acquire() else {
        let message = crate::text::s(visitor.locale, S::ErrBusy);
        return page(
            StatusCode::SERVICE_UNAVAILABLE,
            pages::upload_form(visitor.chrome(&state), &UploadValues::default(), Some(message)),
        );
    };
    let Ok((form, csrf, too_large)) = read_upload(multipart, state.config.max_file_bytes).await
    else {
        return bad_form(&state, &visitor);
    };
    if !visitor.csrf_ok(csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    let values = UploadValues {
        title: form.title.clone(),
        author: form.author.clone(),
        isbn: form.isbn.clone(),
        language: form.language.clone(),
        basis: form.basis.clone(),
        death_year: form.death_year.clone(),
        licence: form.licence.clone(),
        licensor: form.licensor.clone(),
    };
    let result = if too_large {
        Err(UploadRefusal::TooLarge)
    } else {
        state.upload(&account, form, visitor.ip.clone())
    };
    match result {
        Ok(book) => {
            tracing::info!(book = %book.id, account = %account.id, "book uploaded");
            see_other(&format!("/livres/{}", book.id))
        }
        Err(refusal) => {
            tracing::info!(account = %account.id, refusal = refusal.code(), "upload refused");
            let status = match refusal {
                UploadRefusal::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
                UploadRefusal::TooMany(_) => StatusCode::TOO_MANY_REQUESTS,
                UploadRefusal::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                UploadRefusal::CannotUpload => StatusCode::FORBIDDEN,
                UploadRefusal::Type | UploadRefusal::Drm | UploadRefusal::Damaged => {
                    StatusCode::UNSUPPORTED_MEDIA_TYPE
                }
                UploadRefusal::BlockedFile | UploadRefusal::BlockedIsbn(_) => StatusCode::FORBIDDEN,
                UploadRefusal::QuotaAccount | UploadRefusal::QuotaTotal => {
                    StatusCode::INSUFFICIENT_STORAGE
                }
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            let message = refusal.message(&state, visitor.locale, &account.id);
            let mut response =
                page(status, pages::upload_form(visitor.chrome(&state), &values, Some(&message)));
            if let Ok(value) = HeaderValue::from_str(refusal.code()) {
                response.headers_mut().insert("x-ebc-refusal", value);
            }
            response
        }
    }
}

/// The book if this account may see it (its owner, or the operator).
fn visible_book(state: &AppState, account: &Account, id: &str) -> Option<crate::store::Book> {
    state.read(|db| {
        db.book(id)
            .filter(|b| b.account_id == account.id || account.role == Role::Operator)
            .cloned()
    })
}

fn render_book(
    state: &AppState,
    visitor: &Visitor,
    account: &Account,
    id: &str,
    error: Option<&str>,
    status: StatusCode,
) -> Response {
    let Some(book) = visible_book(state, account, id) else {
        return not_found(state, visitor);
    };
    let mut gifts = state
        .read(|db| db.gifts.iter().filter(|g| g.book_id == book.id).cloned().collect::<Vec<_>>());
    gifts.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.id.cmp(&b.id)));
    let owner = book.account_id == account.id;
    page(
        status,
        pages::book(visitor.chrome(state), &book, &gifts, owner, may_publish(account), error),
    )
}

/// `GET /livres/{id}`.
pub(crate) async fn book_page(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    render_book(&state, &visitor, account, &id, None, StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub(crate) struct GiftFields {
    #[serde(default)]
    csrf: Option<String>,
    #[serde(default)]
    count: String,
    #[serde(default)]
    giver: String,
    #[serde(default)]
    recipient: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    locale: String,
    #[serde(default)]
    reveal: String,
}

fn core_error_text(visitor: &Visitor, detail: &str) -> String {
    sf(visitor.locale, S::ErrCore, &[("detail", detail)])
}

/// `POST /livres/{id}/cadeaux`.
pub(crate) async fn create_gifts(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(fields): Form<GiftFields>,
) -> Response {
    let Some(account) = visitor.account.clone() else {
        return see_other("/connexion");
    };
    if !visitor.csrf_ok(fields.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    let max = state.config.max_gifts_per_request;
    let Some(form) = GiftForm::parse(
        &fields.count,
        &fields.giver,
        &fields.recipient,
        &fields.message,
        &fields.locale,
        &fields.reveal,
        max,
    ) else {
        let error = sf(visitor.locale, S::ErrGiftForm, &[("max", &max.to_string())]);
        return render_book(
            &state,
            &visitor,
            &account,
            &id,
            Some(&error),
            StatusCode::UNPROCESSABLE_ENTITY,
        );
    };
    match state.create_gifts(&account, &id, &form, visitor.ip.clone()).await {
        Ok(()) => see_other(&format!("/livres/{id}")),
        Err(GiftError::Forbidden) => forbidden(&state, &visitor),
        Err(GiftError::Core(detail)) => render_book(
            &state,
            &visitor,
            &account,
            &id,
            Some(&core_error_text(&visitor, &detail)),
            StatusCode::BAD_GATEWAY,
        ),
        Err(GiftError::Internal) => server_error(&state, &visitor),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CsrfOnly {
    #[serde(default)]
    csrf: Option<String>,
}

async fn gift_action(
    state: AppState,
    visitor: Visitor,
    id: String,
    form: CsrfOnly,
    action: &str,
) -> Response {
    let Some(account) = visitor.account.clone() else {
        return see_other("/connexion");
    };
    if !visitor.csrf_ok(form.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    let Some(book_id) = state
        .read(|db| db.gift(&id).filter(|g| g.account_id == account.id).map(|g| g.book_id.clone()))
    else {
        return not_found(&state, &visitor);
    };
    let result = match action {
        "cancel" => state.cancel_gift(&account, &id).await,
        "reissue" => state.reissue_gift(&account, &id).await,
        _ => state.push_gift(&id).await,
    };
    match result {
        Ok(()) => see_other(&format!("/livres/{book_id}")),
        Err(GiftError::Forbidden) => forbidden(&state, &visitor),
        Err(GiftError::Core(detail)) => render_book(
            &state,
            &visitor,
            &account,
            &book_id,
            Some(&core_error_text(&visitor, &detail)),
            StatusCode::BAD_GATEWAY,
        ),
        Err(GiftError::Internal) => server_error(&state, &visitor),
    }
}

/// `POST /cadeaux/{id}/annuler`.
pub(crate) async fn cancel_gift(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    gift_action(state, visitor, id, form, "cancel").await
}

/// `POST /cadeaux/{id}/nouveau-code`.
pub(crate) async fn reissue_gift(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    gift_action(state, visitor, id, form, "reissue").await
}

/// `POST /cadeaux/{id}/reessayer`.
pub(crate) async fn retry_gift(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    gift_action(state, visitor, id, form, "retry").await
}

/// `POST /livres/{id}/supprimer`.
pub(crate) async fn delete_book(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    let Some(account) = visitor.account.clone() else {
        return see_other("/connexion");
    };
    if !visitor.csrf_ok(form.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    match state.delete_book(&account, &id, visitor.ip.clone()).await {
        Ok(()) => see_other(&format!("/livres/{id}")),
        Err(GiftError::Forbidden) => not_found(&state, &visitor),
        Err(_) => server_error(&state, &visitor),
    }
}

/// `GET /cadeaux/{id}/carte.pdf?profile=home-a4|home-letter|pro&format=poche|a6|digest|a5`:
/// the card rendered by the core (signed partner request), handed to the owner.
pub(crate) async fn card(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let Some(account) = visitor.account.clone() else {
        return see_other("/connexion");
    };
    let profile = query.get("profile").map_or("home-a4", String::as_str);
    let format = query.get("format").map_or("poche", String::as_str);
    if !matches!(profile, "home-a4" | "home-letter" | "pro")
        || !matches!(format, "poche" | "a6" | "digest" | "a5")
    {
        return not_found(&state, &visitor);
    }
    let Some(core_id) = state.read(|db| {
        db.gift(&id)
            .filter(|g| g.account_id == account.id && g.is_live())
            .and_then(|g| g.core_id.clone())
    }) else {
        return not_found(&state, &visitor);
    };
    match state.core().card(&core_id, format, profile, state.now()).await {
        Ok(card) => {
            let mut response = (StatusCode::OK, card.pdf).into_response();
            let headers = response.headers_mut();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
            if let Ok(value) =
                HeaderValue::from_str(&format!("attachment; filename=\"carte-{id}-{profile}.pdf\""))
            {
                headers.insert(header::CONTENT_DISPOSITION, value);
            }
            if let Some(warnings) = card.warnings.and_then(|w| HeaderValue::from_str(&w).ok()) {
                headers.insert("x-ebc-card-warnings", warnings);
            }
            response
        }
        Err(error) => {
            tracing::warn!(gift = %id, %error, "card not obtained from the core");
            let book_id =
                state.read(|db| db.gift(&id).map(|g| g.book_id.clone())).unwrap_or_default();
            render_book(
                &state,
                &visitor,
                &account,
                &book_id,
                Some(&core_error_text(&visitor, "card.pdf")),
                StatusCode::BAD_GATEWAY,
            )
        }
    }
}
