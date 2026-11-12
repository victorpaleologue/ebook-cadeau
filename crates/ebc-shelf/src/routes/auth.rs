//! Sign-in by e-mail link: `GET|POST /connexion`, `GET|POST /connexion/{token}`,
//! `POST /deconnexion`.

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{self, HeaderValue};
use axum::response::Response;
use serde::Deserialize;

use super::{Visitor, bad_form, page, see_other, server_error, session_cookie_name, too_many};
use crate::pages;
use crate::service::LoginRequest;
use crate::state::AppState;
use crate::text::{S, s};

#[derive(Debug, Deserialize)]
pub(crate) struct EmailForm {
    #[serde(default)]
    email: String,
}

/// `GET /connexion`.
pub(crate) async fn sign_in_page(State(state): State<AppState>, visitor: Visitor) -> Response {
    if visitor.account.is_some() {
        return see_other("/compte");
    }
    page(StatusCode::OK, pages::sign_in(visitor.chrome(&state), None, ""))
}

/// `POST /connexion`: sends a link (same answer whether or not the address has an
/// account).
pub(crate) async fn sign_in(
    State(state): State<AppState>,
    visitor: Visitor,
    Form(form): Form<EmailForm>,
) -> Response {
    let locale = visitor.locale;
    match state.request_login(&form.email, locale, &visitor.client).await {
        LoginRequest::Sent => page(StatusCode::OK, pages::link_sent(visitor.chrome(&state))),
        LoginRequest::Closed => page(
            StatusCode::FORBIDDEN,
            pages::sign_in(visitor.chrome(&state), Some(s(locale, S::SignupClosed)), ""),
        ),
        LoginRequest::Invalid => page(
            StatusCode::UNPROCESSABLE_ENTITY,
            pages::sign_in(visitor.chrome(&state), Some(s(locale, S::InvalidEmail)), &form.email),
        ),
        LoginRequest::MailFailed => page(
            StatusCode::SERVICE_UNAVAILABLE,
            pages::sign_in(visitor.chrome(&state), Some(s(locale, S::MailFailed)), &form.email),
        ),
        LoginRequest::TooMany(wait) => too_many(&state, &visitor, wait),
        LoginRequest::Internal => server_error(&state, &visitor),
    }
}

fn link_invalid(state: &AppState, visitor: &Visitor) -> Response {
    let locale = visitor.locale;
    page(
        StatusCode::GONE,
        pages::message(
            visitor.chrome(state),
            s(locale, S::SignInTitle),
            &[s(locale, S::LinkInvalid).to_owned()],
            Some(("/connexion", s(locale, S::NavSignIn))),
        ),
    )
}

fn plausible(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `GET /connexion/{token}`: a confirmation button (a GET never uses the link up, so mail
/// scanners that follow links cannot spend it).
pub(crate) async fn confirm_page(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(token): Path<String>,
) -> Response {
    if !plausible(&token) {
        return link_invalid(&state, &visitor);
    }
    match state.login_token(&token) {
        Some(login) => page(
            StatusCode::OK,
            pages::confirm_sign_in(visitor.chrome(&state), &token, &login.email),
        ),
        None => link_invalid(&state, &visitor),
    }
}

/// `POST /connexion/{token}`: uses the link, opens a session (cookie `HttpOnly`,
/// `SameSite=Strict`, `Secure` and `__Host-` over HTTPS).
pub(crate) async fn confirm(
    State(state): State<AppState>,
    visitor: Visitor,
    Path(token): Path<String>,
) -> Response {
    if !plausible(&token) {
        return link_invalid(&state, &visitor);
    }
    let session = match state.consume_login(&token) {
        Ok(Some(session)) => session,
        Ok(None) => return link_invalid(&state, &visitor),
        Err(error) => {
            tracing::error!(error = %error.0, "sign-in failed");
            return server_error(&state, &visitor);
        }
    };
    let secure = if state.config.is_https() { "; Secure" } else { "" };
    let cookie = format!(
        "{}={session}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{secure}",
        session_cookie_name(&state),
        state.config.session_ttl_secs
    );
    let mut response = see_other("/compte");
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

#[derive(Debug, Deserialize)]
pub(crate) struct CsrfForm {
    #[serde(default)]
    csrf: Option<String>,
}

/// `POST /deconnexion`.
pub(crate) async fn sign_out(
    State(state): State<AppState>,
    visitor: Visitor,
    Form(form): Form<CsrfForm>,
) -> Response {
    let Some(session) = &visitor.session else {
        return see_other("/");
    };
    if !visitor.csrf_ok(form.csrf.as_deref()) {
        return bad_form(&state, &visitor);
    }
    state.sign_out(session);
    let secure = if state.config.is_https() { "; Secure" } else { "" };
    let mut response = see_other("/");
    if let Ok(value) = HeaderValue::from_str(&format!(
        "{}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{secure}",
        session_cookie_name(&state)
    )) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}
