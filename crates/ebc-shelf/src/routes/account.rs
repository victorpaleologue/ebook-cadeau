//! The account pages, the decisions about it, and the public information pages.

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;

use super::{Visitor, bad_form, forbidden, not_found, page, see_other, server_error};
use crate::clock::parse_iso_date;
use crate::pages;
use crate::service::{DecideError, clean};
use crate::state::AppState;
use crate::store::{Identity, NoticeDecision, PartyKind};
use crate::text::{S, s};

/// `GET /`.
pub(crate) async fn home(State(state): State<AppState>, visitor: Visitor) -> Response {
    page(StatusCode::OK, pages::home(visitor.chrome(&state)))
}

/// `GET /conditions`.
pub(crate) async fn terms(State(state): State<AppState>, visitor: Visitor) -> Response {
    page(StatusCode::OK, pages::terms(visitor.chrome(&state)))
}

/// `GET /mentions-legales`.
pub(crate) async fn legal(State(state): State<AppState>, visitor: Visitor) -> Response {
    page(StatusCode::OK, pages::legal(visitor.chrome(&state)))
}

/// `GET /contact`.
pub(crate) async fn contact(State(state): State<AppState>, visitor: Visitor) -> Response {
    page(StatusCode::OK, pages::contact(visitor.chrome(&state)))
}

/// `GET /compte`.
pub(crate) async fn account(State(state): State<AppState>, visitor: Visitor) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    let (books, decisions, used) = state.read(|db| {
        let mut books: Vec<_> = db.books.iter().filter(|b| b.account_id == account.id).cloned().collect();
        books.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        let decisions: Vec<_> = db
            .notices
            .iter()
            .filter(|n| matches!(&n.decision, NoticeDecision::Upheld { account_id, .. } if *account_id == account.id))
            .cloned()
            .collect();
        (books, decisions, db.used_bytes(Some(&account.id)))
    });
    page(StatusCode::OK, pages::account(visitor.chrome(&state), account, &books, &decisions, used))
}

/// `GET /compte/identite`.
pub(crate) async fn identity_page(State(state): State<AppState>, visitor: Visitor) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    page(
        StatusCode::OK,
        pages::identity_form(visitor.chrome(&state), account.identity.as_ref(), None),
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct IdentityForm {
    #[serde(default)]
    csrf: Option<String>,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    representative: String,
    #[serde(default)]
    address: String,
    #[serde(default)]
    phone: String,
    #[serde(default)]
    birth_date: String,
    #[serde(default)]
    birth_place: String,
}

fn identity_of(form: &IdentityForm, now: i64) -> Option<Identity> {
    let kind = match form.kind.as_str() {
        "organisation" => PartyKind::Organisation,
        _ => PartyKind::Person,
    };
    let opt = |text: &str, max: usize| -> Option<Option<String>> {
        if text.trim().is_empty() { Some(None) } else { clean(text, max, false).map(Some) }
    };
    let representative = opt(&form.representative, 200)?;
    if kind == PartyKind::Organisation && representative.is_none() {
        return None;
    }
    let phone = opt(&form.phone, 40)?;
    if phone
        .as_deref()
        .is_some_and(|p| !p.chars().all(|c| c.is_ascii_digit() || " +.-()".contains(c)))
    {
        return None;
    }
    let birth_date = opt(&form.birth_date, 10)?;
    if birth_date.as_deref().is_some_and(|d| parse_iso_date(d).is_none()) {
        return None;
    }
    Some(Identity {
        kind,
        name: clean(&form.name, 200, false)?,
        representative,
        postal_address: clean(&form.address, 500, true)?,
        phone,
        birth_date,
        birth_place: opt(&form.birth_place, 200)?,
        updated_at: now,
    })
}

/// `POST /compte/identite`.
pub(crate) async fn identity_save(
    State(state): State<AppState>,
    visitor: Visitor,
    Form(form): Form<IdentityForm>,
) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    if !visitor.csrf_ok(form.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    let Some(identity) = identity_of(&form, state.now()) else {
        let typed = Identity {
            kind: if form.kind == "organisation" {
                PartyKind::Organisation
            } else {
                PartyKind::Person
            },
            name: form.name.chars().take(200).collect(),
            representative: Some(form.representative.chars().take(200).collect()),
            postal_address: form.address.chars().take(500).collect(),
            phone: Some(form.phone.chars().take(40).collect()),
            birth_date: Some(form.birth_date.chars().take(10).collect()),
            birth_place: Some(form.birth_place.chars().take(200).collect()),
            updated_at: 0,
        };
        return page(
            StatusCode::UNPROCESSABLE_ENTITY,
            pages::identity_form(
                visitor.chrome(&state),
                Some(&typed),
                Some(s(visitor.locale, S::IdentityInvalid)),
            ),
        );
    };
    match state.save_identity(&account.id, identity) {
        Ok(()) => see_other("/compte"),
        Err(_) => server_error(&state, &visitor),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CloseForm {
    #[serde(default)]
    csrf: Option<String>,
    #[serde(default)]
    confirm: Option<String>,
}

/// `POST /compte/fermer`.
pub(crate) async fn close(
    State(state): State<AppState>,
    visitor: Visitor,
    Form(form): Form<CloseForm>,
) -> Response {
    let Some(account) = visitor.account.clone() else {
        return see_other("/connexion");
    };
    if !visitor.csrf_ok(form.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    if form.confirm.as_deref() != Some("1") {
        return see_other("/compte");
    }
    match state.close_account(&account).await {
        Ok(()) => {
            let mut anonymous = visitor.clone();
            anonymous.account = None;
            anonymous.csrf = None;
            let text = s(visitor.locale, S::AccountClosed);
            page(
                StatusCode::OK,
                pages::message(
                    anonymous.chrome(&state),
                    text,
                    &[],
                    Some(("/", &state.config.name)),
                ),
            )
        }
        Err(_) => server_error(&state, &visitor),
    }
}

/// `GET /compte/decisions/{id}`: statement of reasons and complaint form.
pub(crate) async fn decision_page(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    let found = state.read(|db| {
        let notice = db.notices.iter().find(|n| n.id == id)?.clone();
        let NoticeDecision::Upheld { at, book_id, account_id, .. } = &notice.decision else {
            return None;
        };
        if *account_id != account.id {
            return None;
        }
        let title = db.book(book_id).map(|b| b.title.clone()).unwrap_or_default();
        let appeal = db.appeals.iter().find(|a| a.notice_id == notice.id).cloned();
        let decided = *at;
        Some((notice, title, appeal, decided))
    });
    let Some((notice, title, appeal, decided)) = found else {
        return not_found(&state, &visitor);
    };
    let deadline = decided + state.config.appeal_days * 86_400;
    page(
        StatusCode::OK,
        pages::decision(
            visitor.chrome(&state),
            &notice,
            &title,
            appeal.as_ref(),
            deadline,
            state.now(),
        ),
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppealForm {
    #[serde(default)]
    csrf: Option<String>,
    #[serde(default)]
    text: String,
}

/// `POST /compte/decisions/{id}`: the internal complaint.
pub(crate) async fn appeal(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(form): Form<AppealForm>,
) -> Response {
    let Some(account) = visitor.account.as_ref() else {
        return see_other("/connexion");
    };
    if !visitor.csrf_ok(form.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    match state.file_appeal(account, &id, &form.text) {
        Ok(()) => see_other(&format!("/compte/decisions/{id}")),
        Err(DecideError::Invalid) => forbidden(&state, &visitor),
        Err(DecideError::Internal) => server_error(&state, &visitor),
    }
}
