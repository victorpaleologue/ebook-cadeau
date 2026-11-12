//! HTTP handlers of a storefront.

use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{OriginalUri, Path, Query, State};
use axum::http::header::{
    AUTHORIZATION, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use maud::Markup;
use serde::Deserialize;

use crate::catalogue::{self, Book};
use crate::checkout;
use crate::config::StoreKind;
use crate::crypto::{self, SignatureError};
use crate::epub::{self, Watermark};
use crate::gift::{self, CancelError};
use crate::state::{
    AccountError, Activation, AppState, CardState, OrderError, PendingActivation, SessionView,
    WebhookEvent,
};
use crate::views::{self, CheckoutForm, CheckoutOffer, Chrome};

type Shared = Arc<AppState>;

/// Builds the router of a storefront around `state`.
pub(crate) fn router(state: Shared) -> Router {
    let mut router = Router::new()
        .route("/", get(home))
        .route("/healthz", get(|| async { "ok" }))
        .route("/livre/{id}", get(product))
        .route("/covers/{file}", get(cover))
        .route("/panier", get(cart))
        .route("/panier/ajouter", post(cart_add))
        .route("/panier/retirer", post(cart_remove))
        .route("/compte/connexion", get(login_page).post(login))
        .route("/compte/inscription", post(register))
        .route("/compte/deconnexion", post(logout))
        .route("/commande", get(checkout_page))
        .route("/commande/payer", post(pay))
        .route("/commandes", get(orders))
        .route("/commande/{reference}", get(order_page))
        .route("/commande/{reference}/relancer", post(order_retry))
        .route("/commande/{reference}/annuler", post(order_cancel))
        .route("/commande/{reference}/{file}", get(card_pdf))
        .route("/bibliotheque", get(library))
        .route("/bibliotheque/{file}", get(library_download))
        .route("/cadeau/activer", get(activation_page).post(activate))
        .route("/cadeau/merci", get(activation_done))
        .route("/webhooks/ebc", post(webhook_ebc))
        .route("/webhooks/printer", post(webhook_printer))
        .route("/__test/state", get(test_state));
    if state.config.kind == StoreKind::GrandeEnseigne {
        router = router
            .route("/api/device/link", post(device_link))
            .route("/api/device/library", get(device_library))
            .route("/api/device/books/{file}", get(device_book));
    }
    router
        .fallback(not_found)
        .layer(axum::middleware::map_response(security_headers))
        .with_state(state)
}

// ----- plumbing ---------------------------------------------------------------------------

/// Headers set on every response: pages are personalised (never cached), and a strict
/// Content-Security-Policy forbids any script, which backs up the templates' escaping.
async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'none'; img-src 'self'; style-src 'unsafe-inline'; form-action 'self'; \
             base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    headers.entry(CACHE_CONTROL).or_insert(HeaderValue::from_static("no-store"));
    response
}

/// The session identifier carried by the request's cookie, if it looks like one of ours.
fn session_id(state: &AppState, headers: &HeaderMap) -> Option<String> {
    let name = state.config.kind.cookie_name();
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_owned())
        .filter(|value| value.len() == 32 && value.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn session_cookie(state: &AppState, sid: &str) -> Option<HeaderValue> {
    let name = state.config.kind.cookie_name();
    HeaderValue::from_str(&format!("{name}={sid}; Path=/; HttpOnly; SameSite=Lax")).ok()
}

/// The caller's session identifier and view.
fn visitor(state: &AppState, headers: &HeaderMap) -> (Option<String>, SessionView) {
    let sid = session_id(state, headers);
    let view = state.session(sid.as_deref());
    (sid, view)
}

fn chrome<'a>(state: &AppState, view: &'a SessionView) -> Chrome<'a> {
    Chrome {
        store: state.config.kind,
        account: view.account.as_deref(),
        cart_count: view.cart.len(),
    }
}

fn html(status: StatusCode, markup: Markup) -> Response {
    (status, [(CONTENT_TYPE, "text/html; charset=utf-8")], markup.into_string()).into_response()
}

fn see_other(location: &str) -> Response {
    match HeaderValue::from_str(location) {
        Ok(value) => (StatusCode::SEE_OTHER, [(LOCATION, value)]).into_response(),
        Err(_) => {
            (StatusCode::SEE_OTHER, [(LOCATION, HeaderValue::from_static("/"))]).into_response()
        }
    }
}

fn with_cookie(mut response: Response, cookie: Option<HeaderValue>) -> Response {
    if let Some(cookie) = cookie {
        response.headers_mut().append(SET_COOKIE, cookie);
    }
    response
}

fn problem(
    state: &AppState,
    view: &SessionView,
    status: StatusCode,
    id: &str,
    code: &str,
    title: &str,
    message: &str,
) -> Response {
    html(status, views::problem(&chrome(state, view), id, code, title, message))
}

fn internal_error(state: &AppState, view: &SessionView) -> Response {
    problem(
        state,
        view,
        StatusCode::INTERNAL_SERVER_ERROR,
        "server-error",
        "internal",
        "Erreur interne",
        "Une erreur interne est survenue. Réessayez dans un instant.",
    )
}

fn json_error(status: StatusCode, code: &str, message: &str) -> Response {
    (status, axum::Json(serde_json::json!({ "error": { "code": code, "message": message } })))
        .into_response()
}

/// Only same-site absolute paths are accepted as a post-login destination.
fn safe_next(next: Option<&str>) -> &str {
    match next {
        Some(path)
            if path.starts_with('/')
                && !path.starts_with("//")
                && !path.contains('\\')
                && path.len() <= 200
                && !path.chars().any(char::is_control) =>
        {
            path
        }
        _ => "/",
    }
}

fn login_redirect(next: &str) -> Response {
    let encoded: String = reqwest::Url::parse("http://x/")
        .map(|mut url| {
            url.query_pairs_mut().append_pair("next", next);
            url.query().unwrap_or_default().to_owned()
        })
        .unwrap_or_default();
    see_other(&format!("/compte/connexion?{encoded}"))
}

fn cart_books(view: &SessionView) -> Vec<&'static Book> {
    view.cart.iter().filter_map(|id| catalogue::find(id)).collect()
}

async fn not_found(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    problem(
        &state,
        &view,
        StatusCode::NOT_FOUND,
        "not-found",
        "not_found",
        "Page introuvable",
        "Cette page n'existe pas.",
    )
}

// ----- catalogue --------------------------------------------------------------------------

async fn home(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    let mut warnings = state.startup_warnings.clone();
    if let Err(e) = &state.assets {
        warnings.push(e.to_string());
    }
    html(StatusCode::OK, views::home(&chrome(&state, &view), &warnings))
}

async fn product(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let Some(book) = catalogue::find(&id) else {
        return not_found(State(state), headers).await;
    };
    let owned = view
        .account
        .as_deref()
        .is_some_and(|email| state.library(email).iter().any(|e| e.book_id == book.id));
    let in_cart = view.cart.contains(&book.id);
    html(StatusCode::OK, views::product(&chrome(&state, &view), book, owned, in_cart))
}

async fn cover(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(file): Path<String>,
) -> Response {
    let found = file.rsplit_once('.').and_then(|(id, ext)| {
        let assets = state.assets.as_ref().ok()?.book(id)?;
        Some((assets, ext))
    });
    let cacheable = [(CACHE_CONTROL, "public, max-age=3600")];
    match found {
        Some((assets, "svg")) => {
            (cacheable, [(CONTENT_TYPE, "image/svg+xml")], assets.cover_svg.clone()).into_response()
        }
        Some((assets, "png")) => match assets.print_png() {
            Ok(png) => (cacheable, [(CONTENT_TYPE, "image/png")], png.to_vec()).into_response(),
            Err(_) => {
                let (_, view) = visitor(&state, &headers);
                internal_error(&state, &view)
            }
        },
        _ => not_found(State(state), headers).await,
    }
}

// ----- cart -------------------------------------------------------------------------------

#[derive(Deserialize)]
struct BookForm {
    #[serde(default)]
    book: String,
}

async fn cart(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    html(StatusCode::OK, views::cart(&chrome(&state, &view), &cart_books(&view)))
}

async fn cart_add(
    State(state): State<Shared>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<BookForm>,
) -> Response {
    let Some(book) = catalogue::find(&form.book) else {
        return not_found(State(state), headers).await;
    };
    let Ok((sid, created)) = state.ensure_session(session_id(&state, &headers).as_deref()) else {
        return internal_error(&state, &SessionView::default());
    };
    state.cart_add(&sid, book);
    let cookie = if created { session_cookie(&state, &sid) } else { None };
    with_cookie(see_other("/panier"), cookie)
}

async fn cart_remove(
    State(state): State<Shared>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<BookForm>,
) -> Response {
    if let Some(sid) = session_id(&state, &headers) {
        state.cart_remove(&sid, &form.book);
    }
    see_other("/panier")
}

// ----- accounts ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct NextQuery {
    #[serde(default)]
    next: Option<String>,
}

#[derive(Deserialize)]
struct Credentials {
    #[serde(default)]
    email: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    next: Option<String>,
}

/// The introduction shown above the sign-in forms when a verified gift is waiting.
fn pending_intro(state: &AppState, view: &SessionView) -> Option<Markup> {
    let pending = view.pending_activation.as_ref()?;
    match state.check_activation(&pending.gift_id, &pending.line_ref, None) {
        Activation::Ok { book, giver_name, .. } => Some(views::activation_intro(book, &giver_name)),
        _ => None,
    }
}

fn auth_page(
    state: &AppState,
    view: &SessionView,
    status: StatusCode,
    next: &str,
    error: Option<&str>,
    email: &str,
) -> Response {
    let intro = pending_intro(state, view);
    html(status, views::auth(&chrome(state, view), next, error, email, intro))
}

async fn login_page(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(query): Query<NextQuery>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let next = safe_next(query.next.as_deref());
    if view.account.is_some() {
        return see_other(next);
    }
    auth_page(&state, &view, StatusCode::OK, next, None, "")
}

async fn login(
    State(state): State<Shared>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<Credentials>,
) -> Response {
    let (sid, view) = visitor(&state, &headers);
    let next = safe_next(form.next.as_deref());
    let Some(email) = state.check_password(&form.email, &form.password) else {
        return auth_page(
            &state,
            &view,
            StatusCode::UNAUTHORIZED,
            next,
            Some("Adresse e-mail ou mot de passe incorrect."),
            &form.email,
        );
    };
    match state.sign_in(sid.as_deref(), &email) {
        Ok(new_sid) => with_cookie(see_other(next), session_cookie(&state, &new_sid)),
        Err(_) => internal_error(&state, &view),
    }
}

async fn register(
    State(state): State<Shared>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<Credentials>,
) -> Response {
    let (sid, view) = visitor(&state, &headers);
    let next = safe_next(form.next.as_deref());
    let email = match state.register(&form.email, &form.password) {
        Ok(email) => email,
        Err(error) => {
            let status = match error {
                AccountError::Exists => StatusCode::CONFLICT,
                AccountError::Full => StatusCode::SERVICE_UNAVAILABLE,
                AccountError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            let message = format!("Création du compte impossible : {error}.");
            return auth_page(&state, &view, status, next, Some(&message), &form.email);
        }
    };
    match state.sign_in(sid.as_deref(), &email) {
        Ok(new_sid) => with_cookie(see_other(next), session_cookie(&state, &new_sid)),
        Err(_) => internal_error(&state, &view),
    }
}

async fn logout(
    State(state): State<Shared>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<NextQuery>,
) -> Response {
    if let Some(sid) = session_id(&state, &headers) {
        state.logout(&sid);
    }
    see_other(safe_next(form.next.as_deref()))
}

// ----- checkout ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct PayForm {
    #[serde(default)]
    gift: Option<String>,
    #[serde(default, rename = "giver-name")]
    giver_name: String,
    #[serde(default, rename = "recipient-name")]
    recipient_name: String,
    #[serde(default, rename = "gift-message")]
    gift_message: String,
    #[serde(default, rename = "card-delivery")]
    card_delivery: String,
    #[serde(default, rename = "ship-name")]
    ship_name: String,
    #[serde(default, rename = "ship-street")]
    ship_street: String,
    #[serde(default, rename = "ship-postcode")]
    ship_postcode: String,
    #[serde(default, rename = "ship-city")]
    ship_city: String,
}

impl From<PayForm> for CheckoutForm {
    fn from(form: PayForm) -> Self {
        Self {
            gift: form.gift.is_some_and(|v| !v.is_empty()),
            giver_name: form.giver_name,
            recipient_name: form.recipient_name,
            message: form.gift_message,
            card_delivery: form.card_delivery,
            ship_name: form.ship_name,
            ship_street: form.ship_street,
            ship_postcode: form.ship_postcode,
            ship_city: form.ship_city,
        }
    }
}

async fn offer(state: &AppState) -> CheckoutOffer {
    let gift_available = state.partner.is_some() && state.assets.is_ok();
    CheckoutOffer {
        gift_available,
        post_price_cents: if gift_available { gift::post_card_price(state).await } else { None },
        delivery: state.config.gift_delivery,
    }
}

async fn checkout_page(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    if view.account.is_none() {
        return login_redirect("/commande");
    }
    let books = cart_books(&view);
    if books.is_empty() {
        return see_other("/panier");
    }
    let offer = offer(&state).await;
    let form = CheckoutForm { card_delivery: "home".to_owned(), ..CheckoutForm::default() };
    html(StatusCode::OK, views::checkout(&chrome(&state, &view), &books, &form, &[], &offer))
}

async fn pay(
    State(state): State<Shared>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<PayForm>,
) -> Response {
    let (sid, view) = visitor(&state, &headers);
    let (Some(sid), Some(email)) = (sid, view.account.clone()) else {
        return login_redirect("/commande");
    };
    let books = cart_books(&view);
    if books.is_empty() {
        return see_other("/panier");
    }
    let form = CheckoutForm::from(form);
    let offer = offer(&state).await;
    let details = match checkout::validate(&form, &offer) {
        Ok(details) => details,
        Err(errors) => {
            let page = views::checkout(&chrome(&state, &view), &books, &form, &errors, &offer);
            return html(StatusCode::UNPROCESSABLE_ENTITY, page);
        }
    };
    let order = match state.place_order(&sid, &email, details) {
        Ok(order) => order,
        Err(OrderError::EmptyCart | OrderError::NoSession) => return see_other("/panier"),
        Err(_) => {
            return problem(
                &state,
                &view,
                StatusCode::SERVICE_UNAVAILABLE,
                "order-error",
                "demo_full",
                "Commande impossible",
                "La démonstration a atteint sa limite de commandes.",
            );
        }
    };
    if order.gift.is_some() {
        gift::process_order(&state, &order.reference).await;
    }
    see_other(&format!("/commande/{}", order.reference))
}

async fn orders(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    let Some(email) = view.account.as_deref() else {
        return login_redirect("/commandes");
    };
    html(StatusCode::OK, views::orders(&chrome(&state, &view), &state.orders_of(email)))
}

/// Resolves the signed-in owner of an order, or produces the response to send instead
/// (login redirect, or a 404 that does not reveal whether the order exists).
fn owned_order(
    state: &AppState,
    view: &SessionView,
    reference: &str,
) -> Result<crate::state::Order, Box<Response>> {
    let Some(email) = view.account.as_deref() else {
        return Err(Box::new(login_redirect(&format!("/commande/{}", reference_slug(reference)))));
    };
    state.order_of(reference, email).ok_or_else(|| {
        Box::new(problem(
            state,
            view,
            StatusCode::NOT_FOUND,
            "not-found",
            "not_found",
            "Commande introuvable",
            "Cette commande n'existe pas ou n'est pas liée à votre compte.",
        ))
    })
}

/// Keeps only the characters an order reference can contain, for use in a redirect target.
fn reference_slug(reference: &str) -> String {
    reference.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').take(40).collect()
}

async fn order_page(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(reference): Path<String>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let order = match owned_order(&state, &view, &reference) {
        Ok(order) => order,
        Err(response) => return *response,
    };
    if order.gift.is_some() {
        gift::refresh_order(&state, &order.reference).await;
    }
    let order = state.order(&order.reference).unwrap_or(order);
    html(StatusCode::OK, views::order(&chrome(&state, &view), &order, None))
}

async fn order_retry(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(reference): Path<String>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let order = match owned_order(&state, &view, &reference) {
        Ok(order) => order,
        Err(response) => return *response,
    };
    gift::process_order(&state, &order.reference).await;
    see_other(&format!("/commande/{}", order.reference))
}

async fn order_cancel(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(reference): Path<String>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let order = match owned_order(&state, &view, &reference) {
        Ok(order) => order,
        Err(response) => return *response,
    };
    match gift::cancel_order(&state, &order.reference).await {
        Ok(()) => see_other(&format!("/commande/{}", order.reference)),
        Err(error) => {
            let status = match error {
                CancelError::Remote(_) => StatusCode::BAD_GATEWAY,
                _ => StatusCode::CONFLICT,
            };
            let order = state.order(&order.reference).unwrap_or(order);
            let notice = format!("Annulation impossible : {error}.");
            html(status, views::order(&chrome(&state, &view), &order, Some(&notice)))
        }
    }
}

async fn card_pdf(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path((reference, file)): Path<(String, String)>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let order = match owned_order(&state, &view, &reference) {
        Ok(order) => order,
        Err(response) => return *response,
    };
    let index = file
        .strip_prefix("carte-")
        .and_then(|rest| rest.strip_suffix(".pdf"))
        .and_then(|n| n.parse::<usize>().ok())
        .and_then(|n| n.checked_sub(1));
    let gift = index.and_then(|i| order.lines.iter().filter_map(|l| l.gift.as_ref()).nth(i));
    match gift.map(|g| (&g.card, &g.line_ref)) {
        Some((CardState::Ready(pdf), line_ref)) => {
            let disposition = format!("attachment; filename=\"carte-ebook-cadeau-{line_ref}.pdf\"");
            (
                [(CONTENT_TYPE, "application/pdf".to_owned()), (CONTENT_DISPOSITION, disposition)],
                pdf.as_ref().clone(),
            )
                .into_response()
        }
        _ => not_found(State(state), headers).await,
    }
}

// ----- library ----------------------------------------------------------------------------

async fn library(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    let Some(email) = view.account.as_deref() else {
        return login_redirect("/bibliotheque");
    };
    html(StatusCode::OK, views::library(&chrome(&state, &view), &state.library(email)))
}

fn epub_response(file_name: &str, bytes: Vec<u8>) -> Response {
    // File names come from the catalogue (ASCII identifiers), so no RFC 5987 form is needed.
    let disposition = format!("attachment; filename=\"{file_name}\"");
    ([(CONTENT_TYPE, epub::MEDIA_TYPE.to_owned()), (CONTENT_DISPOSITION, disposition)], bytes)
        .into_response()
}

async fn library_download(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(file): Path<String>,
) -> Response {
    let (_, view) = visitor(&state, &headers);
    let Some(email) = view.account.as_deref() else {
        return login_redirect("/bibliotheque");
    };
    if !state.config.kind.offers_downloads() {
        return problem(
            &state,
            &view,
            StatusCode::NOT_FOUND,
            "no-download",
            "no_download",
            "Téléchargement indisponible",
            "Vos Ebooks se lisent sur votre liseuse et dans l'application, pas depuis ce site.",
        );
    }
    let entry = file
        .strip_suffix(".epub")
        .and_then(|id| state.library(email).into_iter().find(|entry| entry.book_id == id));
    let assets = entry.as_ref().and_then(|e| state.assets.as_ref().ok()?.book(e.book_id));
    let (Some(entry), Some(assets)) = (entry, assets) else {
        return not_found(State(state), headers).await;
    };
    let licensee = match &entry.gift {
        Some(gift) => {
            format!("Offert à {} par {} — compte {email}", gift.recipient_name, gift.giver_name)
        }
        None => format!("Exemplaire de {email}"),
    };
    let watermark = Watermark {
        store_name: state.config.kind.display_name().to_owned(),
        order_ref: entry.order_ref.clone(),
        licensee,
        stamped_at: entry.acquired_at,
    };
    match assets.stamped_epub(&watermark) {
        Ok(bytes) => epub_response(&assets.book.file_name(), bytes),
        Err(_) => internal_error(&state, &view),
    }
}

// ----- gift activation (reseller_redirect, docs/api.md §4.3) ------------------------------

#[derive(Deserialize)]
struct ActivationQuery {
    #[serde(default)]
    ebc_gift: Option<String>,
    #[serde(default)]
    ebc_order: Option<String>,
    #[serde(default)]
    ebc_ts: Option<String>,
    #[serde(default)]
    ebc_sig: Option<String>,
}

fn activation_problem(
    state: &AppState,
    view: &SessionView,
    status: StatusCode,
    code: &str,
    message: &str,
) -> Response {
    problem(state, view, status, "activation-error", code, "Cadeau indisponible", message)
}

/// The response for an activation that cannot proceed.
fn activation_refusal(state: &AppState, view: &SessionView, outcome: &Activation) -> Response {
    match outcome {
        Activation::TakenByOther => activation_problem(
            state,
            view,
            StatusCode::CONFLICT,
            "already_activated",
            "Ce cadeau a déjà été ajouté à un autre compte.",
        ),
        Activation::Void => activation_problem(
            state,
            view,
            StatusCode::GONE,
            "gift_void",
            "Ce cadeau a été annulé ou a expiré.",
        ),
        _ => activation_problem(
            state,
            view,
            StatusCode::NOT_FOUND,
            "unknown_gift",
            "Ce cadeau est inconnu de cette boutique.",
        ),
    }
}

async fn activation_page(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(query): Query<ActivationQuery>,
) -> Response {
    let (sid, view) = visitor(&state, &headers);
    let signed = query.ebc_gift.is_some()
        || query.ebc_order.is_some()
        || query.ebc_ts.is_some()
        || query.ebc_sig.is_some();
    if signed {
        return verify_activation_link(&state, sid.as_deref(), &view, &query);
    }

    // No parameters: continue an activation verified earlier in this session.
    let Some(pending) = view.pending_activation.clone() else {
        return activation_problem(
            &state,
            &view,
            StatusCode::BAD_REQUEST,
            "no_activation",
            "Ce lien d'activation est incomplet ou a expiré. Reprenez depuis la page de votre cadeau.",
        );
    };
    let account = view.account.as_deref();
    match state.check_activation(&pending.gift_id, &pending.line_ref, account) {
        Activation::Ok { book, giver_name, already_owned } => {
            if account.is_some() {
                let page = views::activation_confirm(
                    &chrome(&state, &view),
                    book,
                    &giver_name,
                    already_owned,
                );
                html(StatusCode::OK, page)
            } else {
                auth_page(&state, &view, StatusCode::OK, "/cadeau/activer", None, "")
            }
        }
        Activation::AlreadyYours { book } => {
            html(StatusCode::OK, views::activation_done(&chrome(&state, &view), book, false))
        }
        other => activation_refusal(&state, &view, &other),
    }
}

/// Verifies the signed redirect parameters, remembers the gift in the session and
/// redirects to the parameter-free URL (so the signed link does not linger in the address
/// bar, history or `Referer`).
fn verify_activation_link(
    state: &AppState,
    sid: Option<&str>,
    view: &SessionView,
    query: &ActivationQuery,
) -> Response {
    let Some(partner) = state.config.partner.as_ref() else {
        return activation_refusal(state, view, &Activation::Unknown);
    };
    let (Some(gift_id), Some(order_ref), Some(ts), Some(sig)) = (
        query.ebc_gift.as_deref(),
        query.ebc_order.as_deref(),
        query.ebc_ts.as_deref(),
        query.ebc_sig.as_deref(),
    ) else {
        return activation_problem(
            state,
            view,
            StatusCode::BAD_REQUEST,
            "malformed",
            "Ce lien d'activation est incomplet.",
        );
    };
    let now = state.config.clock.now();
    match crypto::verify_redirect(&partner.secret, gift_id, order_ref, ts, sig, now) {
        Ok(()) => {}
        Err(SignatureError::Stale) => {
            return activation_problem(
                state,
                view,
                StatusCode::GONE,
                "stale_link",
                "Ce lien d'activation a expiré. Retournez sur la page de votre cadeau et appuyez de nouveau sur le bouton.",
            );
        }
        Err(SignatureError::Mismatch) => {
            return activation_problem(
                state,
                view,
                StatusCode::FORBIDDEN,
                "bad_signature",
                "Ce lien d'activation n'est pas valide.",
            );
        }
        Err(_) => {
            return activation_problem(
                state,
                view,
                StatusCode::BAD_REQUEST,
                "malformed",
                "Ce lien d'activation est incomplet.",
            );
        }
    }
    match state.check_activation(gift_id, order_ref, view.account.as_deref()) {
        Activation::Ok { .. } => {}
        Activation::AlreadyYours { book } => {
            return html(StatusCode::OK, views::activation_done(&chrome(state, view), book, false));
        }
        other => return activation_refusal(state, view, &other),
    }
    let Ok((sid, created)) = state.ensure_session(sid) else {
        return internal_error(state, view);
    };
    state.set_pending_activation(
        &sid,
        PendingActivation {
            gift_id: gift_id.to_owned(),
            line_ref: order_ref.to_owned(),
            verified_at: now,
        },
    );
    let cookie = if created { session_cookie(state, &sid) } else { None };
    with_cookie(see_other("/cadeau/activer"), cookie)
}

async fn activate(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (sid, view) = visitor(&state, &headers);
    let (Some(sid), Some(email)) = (sid, view.account.clone()) else {
        return login_redirect("/cadeau/activer");
    };
    let Some(pending) = view.pending_activation.clone() else {
        return activation_problem(
            &state,
            &view,
            StatusCode::BAD_REQUEST,
            "no_activation",
            "Ce lien d'activation est incomplet ou a expiré. Reprenez depuis la page de votre cadeau.",
        );
    };
    match state.activate(&sid, &email, &pending.gift_id, &pending.line_ref) {
        Activation::Ok { .. } => {
            // Tell eBook Cadeau (§4.4). A failure is retried on later page loads; the book
            // is in the library either way.
            gift::confirm_pending_fulfilments(&state, None).await;
            see_other("/cadeau/merci")
        }
        Activation::AlreadyYours { book } => {
            html(StatusCode::OK, views::activation_done(&chrome(&state, &view), book, false))
        }
        other => activation_refusal(&state, &view, &other),
    }
}

async fn activation_done(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let (_, view) = visitor(&state, &headers);
    let done = view.last_activation.as_ref().and_then(|a| Some((catalogue::find(a.book_id)?, a)));
    match done {
        Some((book, activation)) if view.account.is_some() => html(
            StatusCode::OK,
            views::activation_done(&chrome(&state, &view), book, activation.already_owned),
        ),
        _ => see_other("/bibliotheque"),
    }
}

// ----- webhooks ---------------------------------------------------------------------------

async fn webhook_ebc(
    State(state): State<Shared>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(partner) = state.config.partner.as_ref() else {
        return json_error(StatusCode::NOT_FOUND, "not_configured", "no partner API configured");
    };
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    let path = uri.path_and_query().map_or_else(|| uri.path(), |pq| pq.as_str());
    let verdict = crypto::verify_request_signature(
        &partner.secret,
        header("x-ebc-timestamp"),
        header("x-ebc-signature"),
        "POST",
        path,
        &body,
        state.config.clock.now(),
    );
    match verdict {
        Ok(()) => {}
        Err(SignatureError::Stale) => {
            return json_error(
                StatusCode::UNAUTHORIZED,
                "stale_timestamp",
                "timestamp too old or too new",
            );
        }
        Err(SignatureError::Mismatch) => {
            return json_error(StatusCode::UNAUTHORIZED, "bad_signature", "signature mismatch");
        }
        Err(_) => {
            return json_error(
                StatusCode::UNAUTHORIZED,
                "missing_signature",
                "X-EBC-Timestamp and X-EBC-Signature are required",
            );
        }
    }
    let Ok(event) = serde_json::from_slice::<WebhookEvent>(&body) else {
        return json_error(StatusCode::BAD_REQUEST, "invalid_payload", "expected a webhook event");
    };
    if event.event.len() > 64 || event.gift_id.len() > 128 {
        return json_error(StatusCode::BAD_REQUEST, "invalid_payload", "field too long");
    }
    let known = state.apply_webhook(&event);
    axum::Json(serde_json::json!({ "received": true, "known": known })).into_response()
}

#[derive(Deserialize)]
struct PrinterCallbackQuery {
    #[serde(default)]
    token: String,
}

#[derive(Deserialize)]
struct PrinterCallback {
    #[serde(default)]
    order_id: Option<String>,
    status: String,
    #[serde(default)]
    tracking: Option<serde_json::Value>,
}

async fn webhook_printer(
    State(state): State<Shared>,
    Query(query): Query<PrinterCallbackQuery>,
    body: Bytes,
) -> Response {
    let Ok(callback) = serde_json::from_slice::<PrinterCallback>(&body) else {
        return json_error(StatusCode::BAD_REQUEST, "invalid_payload", "expected a status change");
    };
    let tracking = match callback.tracking {
        Some(serde_json::Value::String(text)) if !text.is_empty() => {
            Some(text.chars().filter(|c| !c.is_control()).take(64).collect())
        }
        _ => None,
    };
    let matched = state.apply_printer_callback(
        &query.token,
        callback.order_id.as_deref(),
        &callback.status,
        tracking,
    );
    if matched {
        axum::Json(serde_json::json!({ "received": true })).into_response()
    } else {
        json_error(StatusCode::NOT_FOUND, "unknown_order", "no print order matches this callback")
    }
}

// ----- device API (the ecosystem's e-readers; Grande Enseigne only) -----------------------

#[derive(Deserialize)]
struct LinkRequest {
    email: String,
    password: String,
}

async fn device_link(State(state): State<Shared>, body: Bytes) -> Response {
    let Ok(request) = serde_json::from_slice::<LinkRequest>(&body) else {
        return json_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "expected {\"email\": …, \"password\": …}",
        );
    };
    let Some(email) = state.check_password(&request.email, &request.password) else {
        return json_error(
            StatusCode::UNAUTHORIZED,
            "invalid_credentials",
            "wrong e-mail or password",
        );
    };
    match state.link_device(&email) {
        Ok(token) => axum::Json(serde_json::json!({ "device_token": token })).into_response(),
        Err(_) => {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "token generation failed")
        }
    }
}

/// The account behind the request's `Authorization: Bearer` device token.
fn device_account(state: &AppState, headers: &HeaderMap) -> Result<String, Box<Response>> {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")))
        .and_then(|token| state.device_account(token.trim()))
        .ok_or_else(|| {
            let mut response = json_error(
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "unknown or missing device token",
            );
            response.headers_mut().insert("www-authenticate", HeaderValue::from_static("Bearer"));
            Box::new(response)
        })
}

async fn device_library(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let email = match device_account(&state, &headers) {
        Ok(email) => email,
        Err(response) => return *response,
    };
    let books: Vec<serde_json::Value> = state
        .library(&email)
        .iter()
        .filter_map(|entry| catalogue::find(entry.book_id))
        .map(|book| {
            serde_json::json!({
                "id": book.id,
                "title": book.full_title(),
                "author": book.author,
                "format": "epub",
                "download_url": format!("/api/device/books/{}.epub", book.id),
            })
        })
        .collect();
    axum::Json(books).into_response()
}

async fn device_book(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(file): Path<String>,
) -> Response {
    let email = match device_account(&state, &headers) {
        Ok(email) => email,
        Err(response) => return *response,
    };
    let owned = file
        .strip_suffix(".epub")
        .filter(|id| state.library(&email).iter().any(|entry| entry.book_id == *id))
        .and_then(|id| state.assets.as_ref().ok()?.book(id));
    match owned {
        Some(assets) => epub_response(&assets.book.file_name(), assets.epub.clone()),
        None => json_error(
            StatusCode::NOT_FOUND,
            "not_in_library",
            "this account does not own that book",
        ),
    }
}

// ----- test hook --------------------------------------------------------------------------

async fn test_state(State(state): State<Shared>, headers: HeaderMap) -> Response {
    if state.config.test_hooks {
        axum::Json(state.snapshot()).into_response()
    } else {
        not_found(State(state), headers).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_parameter_cannot_leave_the_site() {
        assert_eq!(safe_next(Some("/commande")), "/commande");
        assert_eq!(safe_next(Some("/cadeau/activer?x=1")), "/cadeau/activer?x=1");
        for bad in [
            "//evil.example",
            "https://evil.example",
            "/\\evil.example",
            "javascript:alert(1)",
            "",
            "relative",
            "/ok\r\nSet-Cookie: x=1",
        ] {
            assert_eq!(safe_next(Some(bad)), "/", "{bad:?}");
        }
        assert_eq!(safe_next(None), "/");
        assert_eq!(safe_next(Some(&format!("/{}", "a".repeat(300)))), "/");
    }

    #[test]
    fn login_redirect_encodes_the_destination() {
        let response = login_redirect("/commande/SEP-2026-000001?a=b&c");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers()[LOCATION],
            "/compte/connexion?next=%2Fcommande%2FSEP-2026-000001%3Fa%3Db%26c"
        );
    }

    #[test]
    fn order_references_are_sanitised_before_being_echoed() {
        assert_eq!(reference_slug("SEP-2026-000001"), "SEP-2026-000001");
        assert_eq!(reference_slug("../../x?y=<z>"), "xyz");
        assert_eq!(reference_slug(&"A".repeat(100)).len(), 40);
    }
}
