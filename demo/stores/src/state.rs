//! In-memory state of one store: accounts, sessions, carts, orders, gifts and linked
//! devices. Demo-grade on purpose: nothing is persisted, and everything is bounded so that
//! a public demo cannot be made to grow without limit.
//!
//! The lock is never held across an `.await`: every method here is synchronous and short.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use crate::assets::Assets;
use crate::catalogue::{self, Book};
use crate::config::{GiftDelivery, StoreConfig};
use crate::crypto;
use crate::error::StoreError;
use crate::outbound::{PartnerClient, PostalAddress, PrinterClient, RemoteGiftStatus};

/// Most accounts a store keeps.
pub const MAX_ACCOUNTS: usize = 5_000;
/// Most sessions a store keeps (stale ones are evicted first).
pub const MAX_SESSIONS: usize = 20_000;
/// Most orders a store keeps.
pub const MAX_ORDERS: usize = 20_000;
/// Most linked devices a store keeps.
pub const MAX_DEVICES: usize = 20_000;
/// Idle time after which a session may be evicted, in seconds.
pub const SESSION_IDLE_SECS: i64 = 12 * 3600;
/// How long a verified activation link stays usable within a session, in seconds.
pub const PENDING_ACTIVATION_SECS: i64 = 30 * 60;
/// Most webhook events kept in the log.
const MAX_WEBHOOK_LOG: usize = 1_000;

/// Status of a gift, as the store knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftStatus {
    /// Not yet registered with eBook Cadeau (the pipeline has not completed).
    Pending,
    /// Registered; the card can be given.
    Created,
    /// The recipient used the code.
    Redeemed,
    /// The book is in the recipient's account (`reseller_redirect`).
    Fulfilled,
    /// Voided (refund).
    Cancelled,
    /// Past its expiry date.
    Expired,
}

impl GiftStatus {
    /// Parses the API name.
    #[must_use]
    pub fn from_api(name: &str) -> Option<Self> {
        Some(match name {
            "created" => Self::Created,
            "redeemed" => Self::Redeemed,
            "fulfilled" => Self::Fulfilled,
            "cancelled" => Self::Cancelled,
            "expired" => Self::Expired,
            _ => return None,
        })
    }

    /// The API name (`pending` for the store-only initial state).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Created => "created",
            Self::Redeemed => "redeemed",
            Self::Fulfilled => "fulfilled",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }

    /// Whether the gift can no longer change (voided or expired).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Expired)
    }

    /// Progress rank used to ignore out-of-order webhooks.
    const fn rank(self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Created => 1,
            Self::Redeemed => 2,
            Self::Fulfilled => 3,
            Self::Cancelled | Self::Expired => 4,
        }
    }
}

/// How the buyer gets the printed card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardDelivery {
    /// The buyer prints the PDF at home (free).
    Home,
    /// The printer prints the card and posts it.
    Post,
}

impl CardDelivery {
    /// Parses the form value (`home` / `post`).
    #[must_use]
    pub fn from_form(value: &str) -> Option<Self> {
        match value {
            "home" => Some(Self::Home),
            "post" => Some(Self::Post),
            _ => None,
        }
    }

    /// The form value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Post => "post",
        }
    }

    /// The `profile` query parameter of `card.pdf` for this choice.
    #[must_use]
    pub const fn card_profile(self) -> &'static str {
        match self {
            Self::Home => "home-a4",
            Self::Post => "pro",
        }
    }
}

/// The card PDF of a gift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardState {
    /// Not fetched yet.
    Pending,
    /// Fetched from eBook Cadeau.
    Ready(Arc<Vec<u8>>),
    /// eBook Cadeau answered `501`: no server-side card; the claim URL and code are shown
    /// instead.
    Unavailable,
}

impl Serialize for CardState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::Pending => map.serialize_entry("state", "pending")?,
            Self::Unavailable => map.serialize_entry("state", "unavailable")?,
            Self::Ready(pdf) => {
                map.serialize_entry("state", "ready")?;
                map.serialize_entry("bytes", &pdf.len())?;
                map.serialize_entry("sha256", &crypto::sha256_hex(pdf))?;
            }
        }
        map.end()
    }
}

/// The print order of a posted card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrintRecord {
    /// Printer-side order identifier, once accepted.
    pub order_id: Option<String>,
    /// `pending`, `accepted`, `printing`, `shipped`, `cancelled` or `rejected`.
    pub status: String,
    /// Tracking number, once shipped.
    pub tracking: Option<String>,
    /// Price charged by the printer, in cents.
    pub price_cents: Option<u32>,
    /// Preflight findings when the printer rejected the PDF.
    pub findings: Vec<String>,
    /// Secret quoted in the callback URL given to the printer.
    #[serde(skip)]
    pub callback_token: String,
}

/// One gift (one order line bought with the gift option).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GiftRecord {
    /// Reference sent to eBook Cadeau as `order_ref` and `Idempotency-Key`.
    pub line_ref: String,
    /// Upload identifier (`hosted_file`).
    pub upload_id: Option<String>,
    /// Gift identifier at eBook Cadeau.
    pub gift_id: Option<String>,
    /// Redemption code as printed.
    pub code_display: Option<String>,
    /// URL encoded in the QR code.
    pub claim_url: Option<String>,
    /// Current status.
    pub status: GiftStatus,
    /// When the status was last confirmed by the partner API.
    pub status_refreshed_at: Option<i64>,
    /// Whether the cover was sent.
    pub cover_sent: bool,
    /// The card PDF.
    pub card: CardState,
    /// The print order, for posted cards.
    pub print: Option<PrintRecord>,
    /// Last error of the pipeline (French, customer-facing).
    pub last_error: Option<String>,
    /// Account that activated the gift (`reseller_redirect`).
    pub activated_by: Option<String>,
    /// Whether eBook Cadeau acknowledged the fulfilment.
    pub fulfilment_confirmed: bool,
    /// Download sessions reported by eBook Cadeau.
    pub downloads: Option<u64>,
    /// When the recipient first used the code.
    pub redeemed_at: Option<i64>,
}

impl GiftRecord {
    fn new(line_ref: String) -> Self {
        Self {
            line_ref,
            upload_id: None,
            gift_id: None,
            code_display: None,
            claim_url: None,
            status: GiftStatus::Pending,
            status_refreshed_at: None,
            cover_sent: false,
            card: CardState::Pending,
            print: None,
            last_error: None,
            activated_by: None,
            fulfilment_confirmed: false,
            downloads: None,
            redeemed_at: None,
        }
    }
}

/// The gift choices made at checkout, common to every line of the order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GiftDetails {
    /// Buyer's name for the card.
    pub giver_name: String,
    /// Recipient's name.
    pub recipient_name: String,
    /// Personal message (may be empty).
    pub message: String,
    /// Print at home or printed and posted.
    pub card_delivery: CardDelivery,
    /// Postal address, for posted cards.
    pub address: Option<PostalAddress>,
    /// How the eBook reaches the recipient.
    pub delivery: GiftDelivery,
    /// Price of one posted card, in cents (0 for home printing).
    pub card_price_cents: u32,
}

/// One line of an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OrderLine {
    /// The book.
    pub book_id: &'static str,
    /// Price paid for the book, in cents.
    pub price_cents: u32,
    /// The gift, when the order was placed with the gift option.
    pub gift: Option<GiftRecord>,
}

/// Payment status of an order (payment is simulated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Paid.
    Paid,
    /// Gift cancelled and refunded.
    Refunded,
}

/// An order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Order {
    /// Order reference, e.g. `SEP-2026-000001`.
    pub reference: String,
    /// E-mail of the buyer's account.
    pub account: String,
    /// Creation time (Unix seconds).
    pub created_at: i64,
    /// The lines.
    pub lines: Vec<OrderLine>,
    /// Gift choices, when the gift option was ticked.
    pub gift: Option<GiftDetails>,
    /// Total paid, in cents (books + posted cards).
    pub total_cents: u32,
    /// Payment status.
    pub status: OrderStatus,
}

/// Where a library entry comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GiftProvenance {
    /// Gift identifier at eBook Cadeau.
    pub gift_id: String,
    /// Name of the buyer.
    pub giver_name: String,
    /// Name the buyer gave for the recipient.
    pub recipient_name: String,
}

/// One book owned by an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LibraryEntry {
    /// The book.
    pub book_id: &'static str,
    /// Order (line) reference it came from.
    pub order_ref: String,
    /// When it was added (Unix seconds).
    pub acquired_at: i64,
    /// Set when the book was received as a gift.
    pub gift: Option<GiftProvenance>,
}

/// A customer account. The password is kept as a salted PBKDF2 hash, in memory only.
#[derive(Debug, Clone, Serialize)]
pub struct Account {
    /// Lower-cased e-mail address (the login).
    pub email: String,
    #[serde(skip)]
    salt: [u8; 16],
    #[serde(skip)]
    password_hash: [u8; 32],
    /// Creation time (Unix seconds).
    pub created_at: i64,
    /// Owned books.
    pub library: Vec<LibraryEntry>,
}

/// A gift activation verified from a signed redirect, waiting for the recipient to sign in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingActivation {
    /// Gift identifier.
    pub gift_id: String,
    /// The gift's order (line) reference.
    pub line_ref: String,
    /// When the signature was verified (Unix seconds).
    pub verified_at: i64,
}

/// The last activation completed in a session (shown on the thank-you page).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedActivation {
    /// The book that was added.
    pub book_id: &'static str,
    /// Whether the account already owned the title.
    pub already_owned: bool,
}

#[derive(Debug, Default)]
struct Session {
    account: Option<String>,
    cart: Vec<&'static str>,
    pending_activation: Option<PendingActivation>,
    last_activation: Option<CompletedActivation>,
    last_seen: i64,
}

/// What a request handler needs to know about the caller's session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionView {
    /// Signed-in account, if any.
    pub account: Option<String>,
    /// Books in the cart, in the order they were added.
    pub cart: Vec<&'static str>,
    /// Verified activation waiting for sign-in / confirmation (already checked for expiry).
    pub pending_activation: Option<PendingActivation>,
    /// Last completed activation.
    pub last_activation: Option<CompletedActivation>,
}

/// A webhook event received from eBook Cadeau.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct WebhookEvent {
    /// `gift.redeemed`, `gift.downloaded` or `gift.cancelled`.
    pub event: String,
    /// Gift identifier.
    pub gift_id: String,
    /// The store's reference.
    #[serde(default)]
    pub order_ref: Option<String>,
    /// When it happened (Unix seconds).
    #[serde(default)]
    pub at: Option<i64>,
}

/// Why an account could not be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AccountError {
    /// The e-mail address is not usable.
    #[error("adresse e-mail invalide")]
    InvalidEmail,
    /// The password is too short or too long.
    #[error("le mot de passe doit compter entre 8 et 200 caractères")]
    InvalidPassword,
    /// An account already exists for this address.
    #[error("un compte existe déjà pour cette adresse")]
    Exists,
    /// The demo has reached its account limit.
    #[error("la démonstration a atteint sa limite de comptes")]
    Full,
    /// The random number generator failed.
    #[error("erreur interne")]
    Internal,
}

/// Why an order could not be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum OrderError {
    /// The cart is empty (possibly because the order was just placed by a double submit).
    #[error("le panier est vide")]
    EmptyCart,
    /// The demo has reached its order limit.
    #[error("la démonstration a atteint sa limite de commandes")]
    Full,
    /// The session does not exist any more.
    #[error("session expirée")]
    NoSession,
}

/// Result of looking at (or performing) a gift activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Activation {
    /// The gift can be (or has just been) added to the account.
    Ok {
        /// The gifted book.
        book: &'static Book,
        /// Buyer's name.
        giver_name: String,
        /// Whether the account already owns the title.
        already_owned: bool,
    },
    /// This account activated the gift earlier.
    AlreadyYours {
        /// The gifted book.
        book: &'static Book,
    },
    /// Another account activated the gift.
    TakenByOther,
    /// The gift was cancelled or has expired.
    Void,
    /// No such gift in this store.
    Unknown,
}

#[derive(Debug, Default)]
struct Inner {
    accounts: BTreeMap<String, Account>,
    sessions: HashMap<String, Session>,
    orders: BTreeMap<String, Order>,
    order_seq: u32,
    devices: HashMap<String, String>,
    webhook_log: Vec<WebhookEvent>,
    post_price: Option<(i64, Option<u32>)>,
}

/// Shared state of one storefront.
#[derive(Debug)]
pub struct AppState {
    /// The configuration the store was started with.
    pub config: StoreConfig,
    /// Generated covers and EPUBs, or why they could not be generated.
    pub assets: Result<&'static Assets, StoreError>,
    /// Client of the eBook Cadeau partner API, when configured.
    pub partner: Option<PartnerClient>,
    /// Client of the print provider, when configured.
    pub printer: Option<PrinterClient>,
    /// Problems found while building the collaborators' clients (shown on the home page).
    pub startup_warnings: Vec<String>,
    inner: Mutex<Inner>,
}

/// Normalises and validates an e-mail address used as a login. Deliberately simple: one
/// `@`, something on both sides, a dot-free check is left to real mail systems.
#[must_use]
pub fn normalise_email(input: &str) -> Option<String> {
    let email = input.trim().to_lowercase();
    if email.len() > 254 || email.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return None;
    }
    Some(email)
}

impl AppState {
    /// Builds the state: generates (or reuses) the static assets and creates the outgoing
    /// clients. Never fails: problems are kept in [`AppState::assets`] and
    /// [`AppState::startup_warnings`] and surface on the pages.
    #[must_use]
    pub fn new(config: StoreConfig) -> Arc<Self> {
        let mut startup_warnings = Vec::new();
        let partner = config.partner.as_ref().and_then(|p| {
            PartnerClient::new(p, config.clock.clone(), config.http_timeout)
                .map_err(|e| startup_warnings.push(e.to_string()))
                .ok()
        });
        let printer = config.printer_url.as_ref().and_then(|url| {
            PrinterClient::new(url, config.http_timeout)
                .map_err(|e| startup_warnings.push(e.to_string()))
                .ok()
        });
        Arc::new(Self {
            assets: Assets::get(),
            partner,
            printer,
            startup_warnings,
            config,
            inner: Mutex::new(Inner::default()),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A poisoned lock only means a handler panicked mid-update; the demo keeps serving.
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn now(&self) -> i64 {
        self.config.clock.now()
    }

    // ----- sessions ----------------------------------------------------------------------

    /// Reads the session `sid` (and refreshes its idle timer).
    #[must_use]
    pub fn session(&self, sid: Option<&str>) -> SessionView {
        let now = self.now();
        let mut inner = self.lock();
        let Some(session) = sid.and_then(|sid| inner.sessions.get_mut(sid)) else {
            return SessionView::default();
        };
        session.last_seen = now;
        if session
            .pending_activation
            .as_ref()
            .is_some_and(|p| now - p.verified_at > PENDING_ACTIVATION_SECS)
        {
            session.pending_activation = None;
        }
        SessionView {
            account: session.account.clone(),
            cart: session.cart.clone(),
            pending_activation: session.pending_activation.clone(),
            last_activation: session.last_activation.clone(),
        }
    }

    /// Returns the identifier of an existing session, or creates one. The boolean is `true`
    /// when a new session was created (the caller must then set the cookie).
    ///
    /// # Errors
    /// [`StoreError::Entropy`] when no identifier can be generated.
    pub fn ensure_session(&self, sid: Option<&str>) -> Result<(String, bool), StoreError> {
        let now = self.now();
        {
            let mut inner = self.lock();
            if let Some(sid) = sid
                && let Some(session) = inner.sessions.get_mut(sid)
            {
                session.last_seen = now;
                return Ok((sid.to_owned(), false));
            }
        }
        let new_sid = self.config.entropy.token(16)?;
        let mut inner = self.lock();
        Self::make_room(&mut inner, now);
        inner.sessions.insert(new_sid.clone(), Session { last_seen: now, ..Session::default() });
        Ok((new_sid, true))
    }

    /// Evicts stale sessions, then the least recently used ones, to stay under the cap.
    fn make_room(inner: &mut Inner, now: i64) {
        if inner.sessions.len() < MAX_SESSIONS {
            return;
        }
        inner.sessions.retain(|_, s| now - s.last_seen <= SESSION_IDLE_SECS);
        while inner.sessions.len() >= MAX_SESSIONS {
            let oldest =
                inner.sessions.iter().min_by_key(|(_, s)| s.last_seen).map(|(k, _)| k.clone());
            match oldest {
                Some(key) => inner.sessions.remove(&key),
                None => break,
            };
        }
    }

    fn with_session<R>(&self, sid: &str, f: impl FnOnce(&mut Session) -> R) -> Option<R> {
        let now = self.now();
        let mut inner = self.lock();
        let session = inner.sessions.get_mut(sid)?;
        session.last_seen = now;
        Some(f(session))
    }

    /// Adds a book to the cart (each title at most once: these are eBooks).
    pub fn cart_add(&self, sid: &str, book: &'static Book) {
        self.with_session(sid, |s| {
            if !s.cart.contains(&book.id) {
                s.cart.push(book.id);
            }
        });
    }

    /// Removes a book from the cart.
    pub fn cart_remove(&self, sid: &str, book_id: &str) {
        self.with_session(sid, |s| s.cart.retain(|id| *id != book_id));
    }

    /// Stores a verified activation in the session.
    pub fn set_pending_activation(&self, sid: &str, pending: PendingActivation) {
        self.with_session(sid, |s| s.pending_activation = Some(pending));
    }

    /// Signs the session out (the cart is kept).
    pub fn logout(&self, sid: &str) {
        self.with_session(sid, |s| s.account = None);
    }

    // ----- accounts ----------------------------------------------------------------------

    /// Creates an account.
    ///
    /// # Errors
    /// See [`AccountError`].
    pub fn register(&self, email: &str, password: &str) -> Result<String, AccountError> {
        let email = normalise_email(email).ok_or(AccountError::InvalidEmail)?;
        let length = password.chars().count();
        if !(8..=200).contains(&length) {
            return Err(AccountError::InvalidPassword);
        }
        if self.lock().accounts.contains_key(&email) {
            return Err(AccountError::Exists);
        }
        let mut salt = [0u8; 16];
        self.config.entropy.fill(&mut salt).map_err(|_| AccountError::Internal)?;
        // Hash outside the lock: it is the only slow step.
        let password_hash =
            crypto::pbkdf2_sha256(password.as_bytes(), &salt, self.config.password_iterations);
        let mut inner = self.lock();
        if inner.accounts.contains_key(&email) {
            return Err(AccountError::Exists);
        }
        if inner.accounts.len() >= MAX_ACCOUNTS {
            return Err(AccountError::Full);
        }
        let account = Account {
            email: email.clone(),
            salt,
            password_hash,
            created_at: self.now(),
            library: Vec::new(),
        };
        inner.accounts.insert(email.clone(), account);
        Ok(email)
    }

    /// Checks a login. Returns the normalised e-mail on success. Takes the same time
    /// whether or not the account exists.
    #[must_use]
    pub fn check_password(&self, email: &str, password: &str) -> Option<String> {
        let email = normalise_email(email)?;
        let stored = self.lock().accounts.get(&email).map(|a| (a.salt, a.password_hash));
        let (salt, expected) = stored.unwrap_or(([0u8; 16], [0u8; 32]));
        let computed =
            crypto::pbkdf2_sha256(password.as_bytes(), &salt, self.config.password_iterations);
        (crypto::ct_eq(&computed, &expected) && stored.is_some()).then_some(email)
    }

    /// Signs `email` in: the session identifier is rotated (session fixation) and the cart
    /// and pending activation are carried over. Returns the new session identifier.
    ///
    /// # Errors
    /// [`StoreError::Entropy`] when no identifier can be generated.
    pub fn sign_in(&self, old_sid: Option<&str>, email: &str) -> Result<String, StoreError> {
        let new_sid = self.config.entropy.token(16)?;
        let now = self.now();
        let mut inner = self.lock();
        let mut session = old_sid.and_then(|sid| inner.sessions.remove(sid)).unwrap_or_default();
        session.account = Some(email.to_owned());
        session.last_seen = now;
        Self::make_room(&mut inner, now);
        inner.sessions.insert(new_sid.clone(), session);
        Ok(new_sid)
    }

    /// The library of an account, oldest first.
    #[must_use]
    pub fn library(&self, email: &str) -> Vec<LibraryEntry> {
        self.lock().accounts.get(email).map(|a| a.library.clone()).unwrap_or_default()
    }

    // ----- devices (e-reader ecosystem API) ----------------------------------------------

    /// Links a device to an account and returns its bearer token (stored hashed).
    ///
    /// # Errors
    /// [`StoreError::Entropy`] when no token can be generated.
    pub fn link_device(&self, email: &str) -> Result<String, StoreError> {
        let token = self.config.entropy.token(32)?;
        let mut inner = self.lock();
        if inner.devices.len() >= MAX_DEVICES {
            // Drop an arbitrary old link rather than refusing new devices.
            if let Some(key) = inner.devices.keys().next().cloned() {
                inner.devices.remove(&key);
            }
        }
        inner.devices.insert(crypto::sha256_hex(token.as_bytes()), email.to_owned());
        Ok(token)
    }

    /// The account a device token belongs to.
    #[must_use]
    pub fn device_account(&self, token: &str) -> Option<String> {
        self.lock().devices.get(&crypto::sha256_hex(token.as_bytes())).cloned()
    }

    // ----- orders ------------------------------------------------------------------------

    /// Turns the cart of session `sid` into a paid order for `email`. Without the gift
    /// option the books go to the buyer's library; with it, each line gets a
    /// [`GiftRecord`] to be processed by the gift pipeline.
    ///
    /// # Errors
    /// See [`OrderError`].
    pub fn place_order(
        &self,
        sid: &str,
        email: &str,
        gift: Option<GiftDetails>,
    ) -> Result<Order, OrderError> {
        let now = self.now();
        let prefix = self.config.kind.order_prefix();
        let mut inner = self.lock();
        if inner.orders.len() >= MAX_ORDERS {
            return Err(OrderError::Full);
        }
        let session = inner.sessions.get_mut(sid).ok_or(OrderError::NoSession)?;
        let cart = std::mem::take(&mut session.cart);
        let books: Vec<&'static Book> = cart.iter().filter_map(|id| catalogue::find(id)).collect();
        if books.is_empty() {
            return Err(OrderError::EmptyCart);
        }
        inner.order_seq += 1;
        let reference = format!("{prefix}-{}-{:06}", crate::time::year(now), inner.order_seq);
        let single = books.len() == 1;
        let lines: Vec<OrderLine> = books
            .iter()
            .enumerate()
            .map(|(i, book)| OrderLine {
                book_id: book.id,
                price_cents: book.price_cents,
                gift: gift.as_ref().map(|_| {
                    GiftRecord::new(if single {
                        reference.clone()
                    } else {
                        format!("{reference}-{}", i + 1)
                    })
                }),
            })
            .collect();
        let books_total: u32 = lines.iter().map(|l| l.price_cents).sum();
        let cards_total = gift.as_ref().map_or(0, |g| {
            g.card_price_cents.saturating_mul(u32::try_from(lines.len()).unwrap_or(u32::MAX))
        });
        let order = Order {
            reference: reference.clone(),
            account: email.to_owned(),
            created_at: now,
            lines,
            gift,
            total_cents: books_total.saturating_add(cards_total),
            status: OrderStatus::Paid,
        };
        if order.gift.is_none()
            && let Some(account) = inner.accounts.get_mut(email)
        {
            for line in &order.lines {
                add_to_library(account, line.book_id, &reference, now, None);
            }
        }
        inner.orders.insert(reference, order.clone());
        Ok(order)
    }

    /// The order `reference` if it belongs to `email`.
    #[must_use]
    pub fn order_of(&self, reference: &str, email: &str) -> Option<Order> {
        self.lock().orders.get(reference).filter(|o| o.account == email).cloned()
    }

    /// The order `reference`, whoever owns it (internal use by the pipeline).
    #[must_use]
    pub fn order(&self, reference: &str) -> Option<Order> {
        self.lock().orders.get(reference).cloned()
    }

    /// Orders of an account, newest first.
    #[must_use]
    pub fn orders_of(&self, email: &str) -> Vec<Order> {
        let inner = self.lock();
        let mut orders: Vec<Order> =
            inner.orders.values().filter(|o| o.account == email).cloned().collect();
        orders.reverse();
        orders
    }

    /// Applies `f` to the gift whose line reference is `line_ref`.
    pub fn update_gift<R>(&self, line_ref: &str, f: impl FnOnce(&mut GiftRecord) -> R) -> Option<R> {
        let mut inner = self.lock();
        let gift = inner
            .orders
            .values_mut()
            .flat_map(|o| o.lines.iter_mut())
            .filter_map(|l| l.gift.as_mut())
            .find(|g| g.line_ref == line_ref)?;
        Some(f(gift))
    }

    /// Marks an order as refunded.
    pub fn mark_refunded(&self, reference: &str) {
        if let Some(order) = self.lock().orders.get_mut(reference) {
            order.status = OrderStatus::Refunded;
        }
    }

    /// Records the status reported by the partner API (authoritative).
    pub fn apply_remote_status(&self, line_ref: &str, remote: &RemoteGiftStatus) {
        let now = self.now();
        self.update_gift(line_ref, |gift| {
            if let Some(status) = GiftStatus::from_api(&remote.status) {
                gift.status = status;
                gift.status_refreshed_at = Some(now);
            }
            if remote.redeemed_at.is_some() {
                gift.redeemed_at = remote.redeemed_at;
            }
            if remote.downloads.is_some() {
                gift.downloads = remote.downloads;
            }
        });
    }

    /// Applies a (verified) webhook event. Idempotent, and tolerant of events arriving out
    /// of order: a gift never moves backwards. Returns whether the gift is known.
    pub fn apply_webhook(&self, event: &WebhookEvent) -> bool {
        let mut inner = self.lock();
        if !inner.webhook_log.contains(event) {
            if inner.webhook_log.len() >= MAX_WEBHOOK_LOG {
                inner.webhook_log.remove(0);
            }
            inner.webhook_log.push(event.clone());
        }
        let Some(gift) = inner
            .orders
            .values_mut()
            .flat_map(|o| o.lines.iter_mut())
            .filter_map(|l| l.gift.as_mut())
            .find(|g| g.gift_id.as_deref() == Some(event.gift_id.as_str()))
        else {
            return false;
        };
        let target = match event.event.as_str() {
            "gift.redeemed" | "gift.downloaded" => GiftStatus::Redeemed,
            "gift.cancelled" => GiftStatus::Cancelled,
            _ => return true,
        };
        if target.rank() > gift.status.rank() {
            gift.status = target;
        }
        if target == GiftStatus::Redeemed && gift.redeemed_at.is_none() {
            gift.redeemed_at = event.at;
        }
        true
    }

    /// Updates a print order from the printer's callback, matched by its secret token.
    /// Returns whether a print order matched.
    pub fn apply_printer_callback(
        &self,
        token: &str,
        order_id: Option<&str>,
        status: &str,
        tracking: Option<String>,
    ) -> bool {
        if token.is_empty() {
            return false;
        }
        let mut inner = self.lock();
        let Some(print) = inner
            .orders
            .values_mut()
            .flat_map(|o| o.lines.iter_mut())
            .filter_map(|l| l.gift.as_mut())
            .filter_map(|g| g.print.as_mut())
            .find(|p| crypto::ct_eq(p.callback_token.as_bytes(), token.as_bytes()))
        else {
            return false;
        };
        if order_id.is_some() && print.order_id.as_deref() != order_id {
            return false;
        }
        if matches!(status, "accepted" | "printing" | "shipped" | "cancelled") {
            status.clone_into(&mut print.status);
        }
        if tracking.is_some() {
            print.tracking = tracking;
        }
        true
    }

    // ----- activation (reseller_redirect) ------------------------------------------------

    /// Looks a gift up by its eBook Cadeau identifier **and** its order reference (both are
    /// covered by the redirect signature, and both must match one of this store's orders).
    fn find_gift<'a>(
        inner: &'a mut Inner,
        gift_id: &str,
        line_ref: &str,
    ) -> Option<(&'a GiftDetails, &'a mut OrderLine)> {
        inner.orders.values_mut().find_map(|order| {
            let details = order.gift.as_ref()?;
            let line = order.lines.iter_mut().find(|l| {
                l.gift.as_ref().is_some_and(|g| {
                    g.line_ref == line_ref && g.gift_id.as_deref() == Some(gift_id)
                })
            })?;
            Some((details, line))
        })
    }

    /// What would happen if `account` activated the gift now (nothing is changed).
    #[must_use]
    pub fn check_activation(
        &self,
        gift_id: &str,
        line_ref: &str,
        account: Option<&str>,
    ) -> Activation {
        let mut inner = self.lock();
        let owned: Vec<&'static str> = account
            .and_then(|email| inner.accounts.get(email))
            .map(|a| a.library.iter().map(|e| e.book_id).collect())
            .unwrap_or_default();
        let Some((details, line)) = Self::find_gift(&mut inner, gift_id, line_ref) else {
            return Activation::Unknown;
        };
        let (Some(gift), Some(book)) = (line.gift.as_ref(), catalogue::find(line.book_id)) else {
            return Activation::Unknown;
        };
        if gift.status.is_terminal() {
            return Activation::Void;
        }
        match (&gift.activated_by, account) {
            (Some(by), Some(me)) if by == me => Activation::AlreadyYours { book },
            (Some(_), _) => Activation::TakenByOther,
            (None, _) => Activation::Ok {
                book,
                giver_name: details.giver_name.clone(),
                already_owned: owned.contains(&book.id),
            },
        }
    }

    /// Adds the gifted book to `email`'s library and marks the gift as activated by that
    /// account. Atomic: two accounts racing for the same gift cannot both win.
    pub fn activate(&self, sid: &str, email: &str, gift_id: &str, line_ref: &str) -> Activation {
        let now = self.now();
        let mut inner = self.lock();
        let outcome = {
            let Some((details, line)) = Self::find_gift(&mut inner, gift_id, line_ref) else {
                return Activation::Unknown;
            };
            let giver_name = details.giver_name.clone();
            let recipient_name = details.recipient_name.clone();
            let book_id = line.book_id;
            let (Some(gift), Some(book)) = (line.gift.as_mut(), catalogue::find(book_id)) else {
                return Activation::Unknown;
            };
            if gift.status.is_terminal() {
                return Activation::Void;
            }
            match &gift.activated_by {
                Some(by) if by == email => return Activation::AlreadyYours { book },
                Some(_) => return Activation::TakenByOther,
                None => {}
            }
            gift.activated_by = Some(email.to_owned());
            gift.status = GiftStatus::Fulfilled;
            (book, giver_name, recipient_name)
        };
        let (book, giver_name, recipient_name) = outcome;
        let provenance = GiftProvenance {
            gift_id: gift_id.to_owned(),
            giver_name: giver_name.clone(),
            recipient_name,
        };
        let already_owned = inner
            .accounts
            .get_mut(email)
            .is_some_and(|account| !add_to_library(account, book.id, line_ref, now, Some(provenance)));
        if let Some(session) = inner.sessions.get_mut(sid) {
            session.pending_activation = None;
            session.last_activation =
                Some(CompletedActivation { book_id: book.id, already_owned });
        }
        Activation::Ok { book, giver_name, already_owned }
    }

    /// Gifts activated locally whose fulfilment eBook Cadeau has not acknowledged yet:
    /// `(line_ref, gift_id, account)`.
    #[must_use]
    pub fn unconfirmed_fulfilments(&self, order_ref: Option<&str>) -> Vec<(String, String, String)> {
        let inner = self.lock();
        inner
            .orders
            .values()
            .filter(|o| order_ref.is_none_or(|r| o.reference == r))
            .flat_map(|o| o.lines.iter())
            .filter_map(|l| l.gift.as_ref())
            .filter(|g| !g.fulfilment_confirmed)
            .filter_map(|g| {
                Some((g.line_ref.clone(), g.gift_id.clone()?, g.activated_by.clone()?))
            })
            .collect()
    }

    /// The opaque `account_ref` quoted to eBook Cadeau for `email`: stable per account,
    /// not reversible to the address.
    #[must_use]
    pub fn account_ref(&self, email: &str) -> String {
        let key = self.config.partner.as_ref().map_or("", |p| p.secret.as_str());
        let mac = crypto::hmac_sha256(key.as_bytes(), format!("account:{email}").as_bytes());
        format!("acct_{}", &crypto::hex(&mac)[..24])
    }

    // ----- cached printer price ----------------------------------------------------------

    /// The cached price of one posted card, if it was fetched less than `max_age` seconds
    /// ago. The inner `Option` is `None` when the printer had no usable price.
    #[must_use]
    pub fn cached_post_price(&self, max_age: i64) -> Option<Option<u32>> {
        let now = self.now();
        self.lock().post_price.filter(|(at, _)| now - at <= max_age).map(|(_, price)| price)
    }

    /// Stores the price of one posted card.
    pub fn cache_post_price(&self, price: Option<u32>) {
        let now = self.now();
        self.lock().post_price = Some((now, price));
    }

    // ----- test hook ---------------------------------------------------------------------

    /// Everything a test wants to inspect, as JSON: orders (with gifts), accounts (never
    /// their password hashes), linked device count and received webhooks.
    #[must_use]
    pub fn snapshot(&self) -> serde_json::Value {
        let inner = self.lock();
        let gifts: Vec<&GiftRecord> = inner
            .orders
            .values()
            .flat_map(|o| o.lines.iter())
            .filter_map(|l| l.gift.as_ref())
            .collect();
        serde_json::json!({
            "store": self.config.kind,
            "gift_delivery": self.config.gift_delivery,
            "orders": inner.orders.values().collect::<Vec<_>>(),
            "gifts": gifts,
            "accounts": inner.accounts.values().collect::<Vec<_>>(),
            "devices": inner.devices.len(),
            "sessions": inner.sessions.len(),
            "webhooks": inner.webhook_log,
        })
    }
}

/// Adds a book to a library unless the account already owns it. Returns whether it was
/// added.
fn add_to_library(
    account: &mut Account,
    book_id: &'static str,
    order_ref: &str,
    now: i64,
    gift: Option<GiftProvenance>,
) -> bool {
    if account.library.iter().any(|e| e.book_id == book_id) {
        return false;
    }
    account.library.push(LibraryEntry {
        book_id,
        order_ref: order_ref.to_owned(),
        acquired_at: now,
        gift,
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Clock, Entropy, StoreKind};

    fn state() -> Arc<AppState> {
        let mut config = StoreConfig::new(StoreKind::GrandeEnseigne)
            .with_clock(Clock::fixed(1_794_441_600))
            .with_entropy(Entropy::deterministic(1))
            .with_partner("http://127.0.0.1:9", "ge", "secret");
        config.password_iterations = 10;
        AppState::new(config)
    }

    fn gift_details() -> GiftDetails {
        GiftDetails {
            giver_name: "Camille".into(),
            recipient_name: "Sacha".into(),
            message: String::new(),
            card_delivery: CardDelivery::Home,
            address: None,
            delivery: GiftDelivery::ResellerRedirect,
            card_price_cents: 0,
        }
    }

    fn order_with(state: &AppState, email: &str, books: &[&str], gift: bool) -> Order {
        let (sid, _) = state.ensure_session(None).unwrap();
        for id in books {
            state.cart_add(&sid, catalogue::find(id).unwrap());
        }
        state.place_order(&sid, email, gift.then(gift_details)).unwrap()
    }

    #[test]
    fn emails_are_normalised_and_vetted() {
        assert_eq!(normalise_email("  Sacha@Example.ORG "), Some("sacha@example.org".into()));
        for bad in ["", "plain", "@x", "x@", "a@b@c", "a b@c", "a\n@b", &"x".repeat(300)] {
            assert_eq!(normalise_email(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn registration_and_login() {
        let s = state();
        assert_eq!(s.register("A@b.example", "motdepasse"), Ok("a@b.example".into()));
        assert_eq!(s.register("a@B.example", "motdepasse"), Err(AccountError::Exists));
        assert_eq!(s.register("nope", "motdepasse"), Err(AccountError::InvalidEmail));
        assert_eq!(s.register("c@d.example", "court"), Err(AccountError::InvalidPassword));
        assert_eq!(
            s.register("c@d.example", &"x".repeat(201)),
            Err(AccountError::InvalidPassword)
        );
        assert_eq!(s.check_password("a@b.example", "motdepasse"), Some("a@b.example".into()));
        assert_eq!(s.check_password(" A@B.EXAMPLE ", "motdepasse"), Some("a@b.example".into()));
        assert_eq!(s.check_password("a@b.example", "Motdepasse"), None);
        assert_eq!(s.check_password("ghost@b.example", "motdepasse"), None);
        // An unknown account must not be accepted with the all-zero fallback hash.
        assert_eq!(s.check_password("ghost@b.example", ""), None);
        // Password hashes never leave the state.
        let dump = s.snapshot().to_string();
        assert!(dump.contains("a@b.example"));
        assert!(!dump.contains("password") && !dump.contains("salt"));
    }

    #[test]
    fn sign_in_rotates_the_session_and_keeps_the_cart() {
        let s = state();
        let (sid, created) = s.ensure_session(None).unwrap();
        assert!(created);
        assert_eq!(s.ensure_session(Some(&sid)).unwrap(), (sid.clone(), false));
        // An unknown identifier supplied by the client is never adopted.
        let (other, created) = s.ensure_session(Some("attacker-chosen")).unwrap();
        assert!(created);
        assert_ne!(other, "attacker-chosen");

        let candide = catalogue::find("candide").unwrap();
        s.cart_add(&sid, candide);
        s.cart_add(&sid, candide);
        assert_eq!(s.session(Some(&sid)).cart, vec!["candide"]);
        s.register("a@b.example", "motdepasse").unwrap();
        let new_sid = s.sign_in(Some(&sid), "a@b.example").unwrap();
        assert_ne!(new_sid, sid);
        assert_eq!(s.session(Some(&sid)), SessionView::default(), "old identifier is dead");
        let view = s.session(Some(&new_sid));
        assert_eq!(view.account.as_deref(), Some("a@b.example"));
        assert_eq!(view.cart, vec!["candide"]);
        s.logout(&new_sid);
        assert_eq!(s.session(Some(&new_sid)).account, None);
        s.cart_remove(&new_sid, "candide");
        assert!(s.session(Some(&new_sid)).cart.is_empty());
    }

    #[test]
    fn plain_orders_fill_the_library_and_gift_orders_do_not() {
        let s = state();
        s.register("buyer@x.example", "motdepasse").unwrap();
        let plain = order_with(&s, "buyer@x.example", &["candide", "le-horla"], false);
        assert_eq!(plain.reference, "GE-2026-000001");
        assert_eq!(plain.total_cents, 299 + 99);
        assert!(plain.lines.iter().all(|l| l.gift.is_none()));
        let library = s.library("buyer@x.example");
        assert_eq!(library.iter().map(|e| e.book_id).collect::<Vec<_>>(), ["candide", "le-horla"]);

        let gift = order_with(&s, "buyer@x.example", &["madame-bovary"], true);
        assert_eq!(gift.reference, "GE-2026-000002");
        assert_eq!(gift.lines[0].gift.as_ref().unwrap().line_ref, "GE-2026-000002");
        assert_eq!(s.library("buyer@x.example").len(), 2, "a gift is not for the buyer");

        let multi = order_with(&s, "buyer@x.example", &["candide", "le-horla"], true);
        let refs: Vec<_> =
            multi.lines.iter().map(|l| l.gift.as_ref().unwrap().line_ref.clone()).collect();
        assert_eq!(refs, ["GE-2026-000003-1", "GE-2026-000003-2"]);

        // Orders are private to their account.
        assert!(s.order_of("GE-2026-000001", "buyer@x.example").is_some());
        assert!(s.order_of("GE-2026-000001", "other@x.example").is_none());
        assert_eq!(s.orders_of("buyer@x.example").len(), 3);
        assert_eq!(s.orders_of("buyer@x.example")[0].reference, "GE-2026-000003");
    }

    #[test]
    fn a_double_submit_finds_an_empty_cart() {
        let s = state();
        s.register("buyer@x.example", "motdepasse").unwrap();
        let (sid, _) = s.ensure_session(None).unwrap();
        s.cart_add(&sid, catalogue::find("candide").unwrap());
        assert!(s.place_order(&sid, "buyer@x.example", None).is_ok());
        assert_eq!(s.place_order(&sid, "buyer@x.example", None), Err(OrderError::EmptyCart));
        assert_eq!(s.place_order("gone", "buyer@x.example", None), Err(OrderError::NoSession));
    }

    fn created_gift(s: &AppState) -> (String, String) {
        s.register("buyer@x.example", "motdepasse").unwrap();
        let order = order_with(s, "buyer@x.example", &["candide"], true);
        let line_ref = order.reference.clone();
        s.update_gift(&line_ref, |g| {
            g.gift_id = Some("g_1".into());
            g.status = GiftStatus::Created;
        })
        .unwrap();
        ("g_1".to_owned(), line_ref)
    }

    #[test]
    fn activation_is_single_recipient() {
        let s = state();
        let (gift_id, line_ref) = created_gift(&s);
        s.register("sacha@x.example", "motdepasse").unwrap();
        s.register("intrus@x.example", "motdepasse").unwrap();
        let (sid, _) = s.ensure_session(None).unwrap();

        assert!(matches!(
            s.check_activation(&gift_id, &line_ref, None),
            Activation::Ok { already_owned: false, .. }
        ));
        // The pair must match: right gift with the wrong order reference is unknown.
        assert_eq!(s.check_activation(&gift_id, "GE-2026-999999", None), Activation::Unknown);
        assert_eq!(s.check_activation("g_2", &line_ref, None), Activation::Unknown);

        let done = s.activate(&sid, "sacha@x.example", &gift_id, &line_ref);
        assert!(matches!(done, Activation::Ok { already_owned: false, .. }));
        let library = s.library("sacha@x.example");
        assert_eq!(library.len(), 1);
        assert_eq!(library[0].gift.as_ref().unwrap().giver_name, "Camille");
        assert_eq!(
            s.session(Some(&sid)).last_activation,
            Some(CompletedActivation { book_id: "candide", already_owned: false })
        );

        // Same account again: idempotent. Another account: refused, library untouched.
        assert!(matches!(
            s.activate(&sid, "sacha@x.example", &gift_id, &line_ref),
            Activation::AlreadyYours { .. }
        ));
        assert_eq!(
            s.activate(&sid, "intrus@x.example", &gift_id, &line_ref),
            Activation::TakenByOther
        );
        assert!(s.library("intrus@x.example").is_empty());
        assert_eq!(s.library("sacha@x.example").len(), 1);
        assert_eq!(
            s.unconfirmed_fulfilments(None),
            vec![(line_ref.clone(), gift_id.clone(), "sacha@x.example".to_owned())]
        );
        s.update_gift(&line_ref, |g| g.fulfilment_confirmed = true);
        assert!(s.unconfirmed_fulfilments(None).is_empty());
    }

    #[test]
    fn cancelled_gifts_cannot_be_activated() {
        let s = state();
        let (gift_id, line_ref) = created_gift(&s);
        s.register("sacha@x.example", "motdepasse").unwrap();
        let (sid, _) = s.ensure_session(None).unwrap();
        s.update_gift(&line_ref, |g| g.status = GiftStatus::Cancelled);
        assert_eq!(s.check_activation(&gift_id, &line_ref, None), Activation::Void);
        assert_eq!(s.activate(&sid, "sacha@x.example", &gift_id, &line_ref), Activation::Void);
        assert!(s.library("sacha@x.example").is_empty());
    }

    #[test]
    fn activating_a_title_already_owned_is_flagged() {
        let s = state();
        let (gift_id, line_ref) = created_gift(&s);
        s.register("sacha@x.example", "motdepasse").unwrap();
        order_with(&s, "sacha@x.example", &["candide"], false);
        let (sid, _) = s.ensure_session(None).unwrap();
        assert!(matches!(
            s.check_activation(&gift_id, &line_ref, Some("sacha@x.example")),
            Activation::Ok { already_owned: true, .. }
        ));
        assert!(matches!(
            s.activate(&sid, "sacha@x.example", &gift_id, &line_ref),
            Activation::Ok { already_owned: true, .. }
        ));
        assert_eq!(s.library("sacha@x.example").len(), 1, "no duplicate entry");
    }

    #[test]
    fn webhooks_are_idempotent_and_never_move_a_gift_backwards() {
        let s = state();
        let (gift_id, line_ref) = created_gift(&s);
        let event = |name: &str, at: i64| WebhookEvent {
            event: name.into(),
            gift_id: gift_id.clone(),
            order_ref: Some(line_ref.clone()),
            at: Some(at),
        };
        let status = |s: &AppState| s.order(&line_ref).unwrap().lines[0].gift.clone().unwrap();

        assert!(s.apply_webhook(&event("gift.redeemed", 100)));
        assert!(s.apply_webhook(&event("gift.redeemed", 100)), "retries are harmless");
        assert_eq!(status(&s).status, GiftStatus::Redeemed);
        assert_eq!(status(&s).redeemed_at, Some(100));
        s.update_gift(&line_ref, |g| g.status = GiftStatus::Fulfilled);
        assert!(s.apply_webhook(&event("gift.downloaded", 200)));
        assert_eq!(status(&s).status, GiftStatus::Fulfilled, "a late event does not regress");
        assert_eq!(status(&s).redeemed_at, Some(100));
        assert!(s.apply_webhook(&event("gift.cancelled", 300)));
        assert_eq!(status(&s).status, GiftStatus::Cancelled);
        assert!(s.apply_webhook(&event("gift.redeemed", 400)));
        assert_eq!(status(&s).status, GiftStatus::Cancelled, "cancelled is final");
        assert!(s.apply_webhook(&event("gift.unheard-of", 500)), "unknown events are accepted");

        let unknown = WebhookEvent {
            event: "gift.redeemed".into(),
            gift_id: "g_nobody".into(),
            order_ref: None,
            at: None,
        };
        assert!(!s.apply_webhook(&unknown));
        assert_eq!(s.snapshot()["webhooks"].as_array().unwrap().len(), 6, "deduplicated log");
    }

    #[test]
    fn printer_callbacks_need_the_right_token() {
        let s = state();
        let (_, line_ref) = created_gift(&s);
        s.update_gift(&line_ref, |g| {
            g.print = Some(PrintRecord {
                order_id: Some("po_1".into()),
                status: "accepted".into(),
                tracking: None,
                price_cents: Some(590),
                findings: vec![],
                callback_token: "tok123".into(),
            });
        });
        assert!(!s.apply_printer_callback("", Some("po_1"), "shipped", None));
        assert!(!s.apply_printer_callback("wrong", Some("po_1"), "shipped", None));
        assert!(!s.apply_printer_callback("tok123", Some("po_other"), "shipped", None));
        assert!(s.apply_printer_callback("tok123", Some("po_1"), "printing", None));
        assert!(s.apply_printer_callback("tok123", None, "shipped", Some("DC-1".into())));
        // An unknown status word is ignored rather than displayed.
        assert!(s.apply_printer_callback("tok123", None, "<script>", None));
        let print = s.order(&line_ref).unwrap().lines[0].gift.clone().unwrap().print.unwrap();
        assert_eq!((print.status.as_str(), print.tracking.as_deref()), ("shipped", Some("DC-1")));
        assert!(!s.snapshot().to_string().contains("tok123"), "the token is not exposed");
    }

    #[test]
    fn devices_are_linked_by_token() {
        let s = state();
        let token = s.link_device("a@b.example").unwrap();
        assert_eq!(token.len(), 64);
        assert_eq!(s.device_account(&token).as_deref(), Some("a@b.example"));
        assert_eq!(s.device_account("nope"), None);
        assert_eq!(s.device_account(""), None);
    }

    #[test]
    fn account_refs_are_stable_opaque_and_distinct() {
        let s = state();
        let a = s.account_ref("a@b.example");
        assert_eq!(a, s.account_ref("a@b.example"));
        assert_ne!(a, s.account_ref("c@b.example"));
        assert!(a.starts_with("acct_") && a.len() == 29);
        assert!(!a.contains("a@b"));
    }

    #[test]
    fn pending_activations_expire() {
        let now = Arc::new(std::sync::atomic::AtomicI64::new(1_000_000));
        let clock = {
            let now = now.clone();
            Clock::from_fn(move || now.load(std::sync::atomic::Ordering::SeqCst))
        };
        let s = AppState::new(
            StoreConfig::new(StoreKind::Septante)
                .with_clock(clock)
                .with_entropy(Entropy::deterministic(2)),
        );
        let (sid, _) = s.ensure_session(None).unwrap();
        s.set_pending_activation(
            &sid,
            PendingActivation { gift_id: "g".into(), line_ref: "r".into(), verified_at: 1_000_000 },
        );
        now.store(1_000_000 + PENDING_ACTIVATION_SECS, std::sync::atomic::Ordering::SeqCst);
        assert!(s.session(Some(&sid)).pending_activation.is_some());
        now.store(1_000_000 + PENDING_ACTIVATION_SECS + 1, std::sync::atomic::Ordering::SeqCst);
        assert!(s.session(Some(&sid)).pending_activation.is_none());
    }

    #[test]
    fn post_price_cache_expires() {
        let s = state();
        assert_eq!(s.cached_post_price(60), None);
        s.cache_post_price(Some(590));
        assert_eq!(s.cached_post_price(60), Some(Some(590)));
        s.cache_post_price(None);
        assert_eq!(s.cached_post_price(60), Some(None));
        assert_eq!(s.cached_post_price(-1), None);
    }
}
