//! Notice and action: the public notice form (DSA art. 16) and the operator's review
//! pages (decisions, complaints, blocklist).

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;

use super::{Visitor, bad_form, forbidden, not_found, page, see_other, server_error, too_many};
use crate::pages::{self, ModerationView, NoticeValues};
use crate::service::{DecideError, Decision, NoticeError, NoticeForm};
use crate::state::AppState;
use crate::store::{Account, AccountStatus, NoticeDecision, Role};
use crate::text::{S, s};

/// `GET /signaler`.
pub(crate) async fn notice_page(State(state): State<AppState>, visitor: Visitor) -> Response {
    page(StatusCode::OK, pages::notice_form(visitor.chrome(&state), &NoticeValues::default(), None))
}

#[derive(Debug, Deserialize)]
pub(crate) struct NoticeFields {
    #[serde(default)]
    category: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    explanation: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    exception: Option<String>,
    #[serde(default)]
    good_faith: Option<String>,
}

/// `POST /signaler`: no account needed; rate-limited per client network.
pub(crate) async fn notice(
    State(state): State<AppState>,
    visitor: Visitor,
    Form(fields): Form<NoticeFields>,
) -> Response {
    let values = NoticeValues {
        category: fields.category.clone(),
        location: fields.location.chars().take(2_000).collect(),
        explanation: fields.explanation.chars().take(5_000).collect(),
        name: fields.name.chars().take(200).collect(),
        email: fields.email.chars().take(254).collect(),
    };
    let form = NoticeForm {
        category: fields.category,
        location: fields.location,
        explanation: fields.explanation,
        name: fields.name,
        email: fields.email,
        exception: fields.exception.as_deref() == Some("1"),
        good_faith: fields.good_faith.as_deref() == Some("1"),
    };
    match state.file_notice(form, visitor.locale, &visitor.client).await {
        Ok(id) => page(StatusCode::OK, pages::notice_received(visitor.chrome(&state), &id)),
        Err(NoticeError::Invalid) => page(
            StatusCode::UNPROCESSABLE_ENTITY,
            pages::notice_form(
                visitor.chrome(&state),
                &values,
                Some(s(visitor.locale, S::NoticeErr)),
            ),
        ),
        Err(NoticeError::TooMany(wait)) => too_many(&state, &visitor, wait),
        Err(NoticeError::Internal) => server_error(&state, &visitor),
    }
}

fn operator(visitor: &Visitor) -> Option<&Account> {
    visitor.account.as_ref().filter(|a| {
        a.role == Role::Operator
            && matches!(a.status, AccountStatus::Active | AccountStatus::Suspended { .. })
    })
}

/// `GET /moderation`.
pub(crate) async fn moderation(State(state): State<AppState>, visitor: Visitor) -> Response {
    if visitor.account.is_none() {
        return see_other("/connexion");
    }
    if operator(&visitor).is_none() {
        return forbidden(&state, &visitor);
    }
    let db = state.snapshot();
    let mut open: Vec<_> =
        db.notices.iter().filter(|n| matches!(n.decision, NoticeDecision::Open)).collect();
    open.sort_by_key(|n| n.received_at);
    let mut decided: Vec<_> =
        db.notices.iter().filter(|n| !matches!(n.decision, NoticeDecision::Open)).collect();
    decided.sort_by_key(|n| std::cmp::Reverse(n.received_at));
    decided.truncate(200);
    let appeals = db
        .appeals
        .iter()
        .filter(|a| a.granted.is_none())
        .map(|a| (a, db.notices.iter().find(|n| n.id == a.notice_id)))
        .collect();
    let suspended = db
        .accounts
        .iter()
        .filter(|a| !matches!(a.status, AccountStatus::Active | AccountStatus::Closed { .. }))
        .collect();
    let view = ModerationView { open, decided, appeals, blocklist: &db.blocklist, suspended };
    page(StatusCode::OK, pages::moderation(visitor.chrome(&state), &view))
}

fn render_notice(
    state: &AppState,
    visitor: &Visitor,
    id: &str,
    error: Option<&str>,
    status: StatusCode,
) -> Response {
    let db = state.snapshot();
    let Some(notice) = db.notices.iter().find(|n| n.id == id) else {
        return not_found(state, visitor);
    };
    let book_id = match &notice.decision {
        NoticeDecision::Upheld { book_id, .. } => Some(book_id.as_str()),
        _ => notice.book_id.as_deref(),
    };
    let book = book_id.and_then(|b| db.book(b));
    let uploader = book.and_then(|b| db.account(&b.account_id));
    page(status, pages::moderation_notice(visitor.chrome(state), notice, book, uploader, error))
}

/// `GET /moderation/signalements/{id}`.
pub(crate) async fn moderation_notice(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
) -> Response {
    if visitor.account.is_none() {
        return see_other("/connexion");
    }
    if operator(&visitor).is_none() {
        return forbidden(&state, &visitor);
    }
    render_notice(&state, &visitor, &id, None, StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionFields {
    #[serde(default)]
    csrf: Option<String>,
    #[serde(default)]
    decision: String,
    #[serde(default)]
    book_id: String,
    #[serde(default)]
    statement: String,
    #[serde(default)]
    false_declaration: Option<String>,
}

/// `POST /moderation/signalements/{id}`.
pub(crate) async fn decide(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(fields): Form<DecisionFields>,
) -> Response {
    if visitor.account.is_none() {
        return see_other("/connexion");
    }
    if operator(&visitor).is_none() {
        return forbidden(&state, &visitor);
    }
    if !visitor.csrf_ok(fields.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    let decision = match fields.decision.as_str() {
        "uphold" => {
            let book_id = if fields.book_id.trim().is_empty() {
                state
                    .read(|db| {
                        db.notices.iter().find(|n| n.id == id).and_then(|n| n.book_id.clone())
                    })
                    .unwrap_or_default()
            } else {
                fields.book_id.trim().to_owned()
            };
            Decision::Uphold {
                book_id,
                statement: fields.statement,
                false_declaration: fields.false_declaration.as_deref() == Some("1"),
            }
        }
        "reject" => Decision::Reject { reason: fields.statement },
        _ => {
            return render_notice(
                &state,
                &visitor,
                &id,
                Some(s(visitor.locale, S::ErrDecision)),
                StatusCode::UNPROCESSABLE_ENTITY,
            );
        }
    };
    match state.decide_notice(&id, decision).await {
        Ok(()) => see_other(&format!("/moderation/signalements/{id}")),
        Err(DecideError::Invalid) => render_notice(
            &state,
            &visitor,
            &id,
            Some(s(visitor.locale, S::ErrDecision)),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        Err(DecideError::Internal) => server_error(&state, &visitor),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppealDecision {
    #[serde(default)]
    csrf: Option<String>,
    #[serde(default)]
    grant: String,
    #[serde(default)]
    answer: String,
}

/// `POST /moderation/reclamations/{id}`.
pub(crate) async fn decide_appeal(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(id): Path<String>,
    Form(fields): Form<AppealDecision>,
) -> Response {
    if visitor.account.is_none() {
        return see_other("/connexion");
    }
    if operator(&visitor).is_none() {
        return forbidden(&state, &visitor);
    }
    if !visitor.csrf_ok(fields.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    match state.decide_appeal(&id, fields.grant == "1", &fields.answer).await {
        Ok(()) => see_other("/moderation"),
        Err(DecideError::Invalid) => not_found(&state, &visitor),
        Err(DecideError::Internal) => server_error(&state, &visitor),
    }
}
