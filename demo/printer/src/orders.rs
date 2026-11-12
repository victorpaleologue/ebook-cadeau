//! Orders and their time-compressed life cycle: `accepted → printing → shipped`.
//!
//! In automatic mode each order advances one step per tick (2 s by default) in a background
//! task; in manual mode (no tick) orders only move when [`PrinterState::advance_all`] is
//! called, which makes tests deterministic. Status changes are POSTed to the order's
//! `callback_url`, when it has one.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::catalog::Product;
use crate::preflight::FindingBody;

/// Most orders the demo keeps in memory.
pub const MAX_ORDERS: usize = 10_000;
/// Most callback attempts recorded per order.
const MAX_CALLBACK_LOG: usize = 20;
/// Attempts made to deliver one status change.
const CALLBACK_ATTEMPTS: u32 = 3;

/// Injected wall clock (Unix seconds).
#[derive(Clone)]
pub struct Clock(Arc<dyn Fn() -> i64 + Send + Sync>);

impl Clock {
    /// The system clock.
    #[must_use]
    pub fn system() -> Self {
        Self(Arc::new(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        }))
    }

    /// A clock that always returns `unix_seconds`.
    #[must_use]
    pub fn fixed(unix_seconds: i64) -> Self {
        Self(Arc::new(move || unix_seconds))
    }

    /// Current time in Unix seconds.
    #[must_use]
    pub fn now(&self) -> i64 {
        (self.0)()
    }
}

impl fmt::Debug for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Clock(..)")
    }
}

/// Configuration of the fictional printer.
#[derive(Debug, Clone)]
pub struct PrinterConfig {
    /// Time between two steps of an order's life cycle. `None` = manual stepping only.
    pub tick: Option<Duration>,
    /// Enables `GET /__test/orders` and `POST /__test/advance`.
    pub test_hooks: bool,
    /// Wall clock for timestamps.
    pub clock: Clock,
    /// Mixed into order identifiers so that they cannot be guessed from the sequence.
    pub id_salt: String,
    /// Largest PDF accepted, in bytes.
    pub max_pdf_bytes: usize,
    /// Timeout of one callback delivery.
    pub callback_timeout: Duration,
    /// Pause between two delivery attempts of the same callback.
    pub callback_retry_delay: Duration,
}

impl Default for PrinterConfig {
    /// Automatic mode with a 2 s tick, no test hooks, the system clock.
    fn default() -> Self {
        Self {
            tick: Some(Duration::from_secs(2)),
            test_hooks: false,
            clock: Clock::system(),
            id_salt: "demo".to_owned(),
            max_pdf_bytes: 64 << 20,
            callback_timeout: Duration::from_secs(5),
            callback_retry_delay: Duration::from_millis(500),
        }
    }
}

impl PrinterConfig {
    /// Manual mode for tests: orders advance only through [`PrinterState::advance_all`] or
    /// `POST /__test/advance`; test hooks on; a fixed clock; no pause between retries.
    #[must_use]
    pub fn manual() -> Self {
        Self {
            tick: None,
            test_hooks: true,
            clock: Clock::fixed(1_794_441_600),
            id_salt: "test".to_owned(),
            callback_retry_delay: Duration::from_millis(1),
            callback_timeout: Duration::from_secs(2),
            ..Self::default()
        }
    }
}

/// Where an order is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// The PDF passed preflight; waiting for a press slot.
    Accepted,
    /// On press.
    Printing,
    /// Handed to the carrier.
    Shipped,
    /// Cancelled before printing.
    Cancelled,
}

impl OrderStatus {
    /// The API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Printing => "printing",
            Self::Shipped => "shipped",
            Self::Cancelled => "cancelled",
        }
    }

    /// The next step, if any.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Accepted => Some(Self::Printing),
            Self::Printing => Some(Self::Shipped),
            Self::Shipped | Self::Cancelled => None,
        }
    }
}

/// Postal address of an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    /// Addressee.
    pub name: String,
    /// First address line.
    pub line1: String,
    /// Second address line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line2: Option<String>,
    /// Postal code.
    pub postal_code: String,
    /// City.
    pub city: String,
    /// ISO 3166-1 alpha-2 country code.
    pub country: String,
}

/// One status change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryEntry {
    /// The status entered.
    pub status: OrderStatus,
    /// When (Unix seconds).
    pub at: i64,
}

/// One attempt to deliver a status change to the order's `callback_url`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CallbackAttempt {
    /// The status that was notified.
    pub status: OrderStatus,
    /// HTTP status of the receiver, or `None` when it could not be reached.
    pub http_status: Option<u16>,
    /// Whether the receiver acknowledged with a 2xx.
    pub delivered: bool,
}

/// An order as kept by the printer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Order {
    /// Identifier returned to the client (`po_…`).
    pub order_id: String,
    /// Product identifier.
    pub product: &'static str,
    /// Number of copies.
    pub quantity: u32,
    /// The client's own reference.
    pub reference: String,
    /// Where to post the cards.
    pub address: Address,
    /// Where status changes are POSTed.
    pub callback_url: Option<String>,
    /// Current status.
    pub status: OrderStatus,
    /// Total price in cents.
    pub price_cents: u32,
    /// Currency of the price.
    pub currency: &'static str,
    /// Tracking number, once shipped.
    pub tracking: Option<String>,
    /// Creation time (Unix seconds).
    pub created_at: i64,
    /// Time of the last status change (Unix seconds).
    pub updated_at: i64,
    /// Every status the order went through.
    pub history: Vec<HistoryEntry>,
    /// SHA-256 of the uploaded PDF (hex).
    pub pdf_sha256: String,
    /// Size of the uploaded PDF in bytes.
    pub pdf_bytes: usize,
    /// Preflight findings below error level.
    pub preflight_warnings: Vec<FindingBody>,
    /// Callback delivery log.
    pub callbacks: Vec<CallbackAttempt>,
    #[serde(skip)]
    sequence: u32,
}

/// What a client submits, after validation.
#[derive(Debug, Clone, PartialEq)]
pub struct NewOrder {
    /// The product.
    pub product: &'static Product,
    /// Number of copies.
    pub quantity: u32,
    /// The client's reference.
    pub reference: String,
    /// Postal address.
    pub address: Address,
    /// Callback URL.
    pub callback_url: Option<String>,
    /// Price for the quantity, in cents.
    pub price_cents: u32,
    /// SHA-256 of the PDF (hex).
    pub pdf_sha256: String,
    /// Size of the PDF.
    pub pdf_bytes: usize,
    /// Preflight findings below error level.
    pub preflight_warnings: Vec<FindingBody>,
}

/// Outcome of registering an order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Registration {
    /// A new order was created.
    Created(Order),
    /// The same reference was submitted before with the same file, product and quantity:
    /// the existing order is returned (idempotent replay).
    Replayed(Order),
    /// The reference is already used by a different order.
    Conflict,
    /// The demo has reached [`MAX_ORDERS`].
    Full,
}

/// Why an order could not be cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelError {
    /// No such order.
    Unknown,
    /// Printing has started (or the order is already shipped).
    TooLate,
}

#[derive(Debug, Default)]
struct Inner {
    orders: BTreeMap<String, Order>,
    by_reference: BTreeMap<String, String>,
    sequence: u32,
}

/// Shared state of the printer.
#[derive(Debug)]
pub struct PrinterState {
    /// The configuration the printer was started with.
    pub config: PrinterConfig,
    http: Option<reqwest::Client>,
    inner: Mutex<Inner>,
}

impl PrinterState {
    /// Creates the state. If the HTTP client used for callbacks cannot be built, callbacks
    /// are skipped (and logged as undelivered) rather than failing the printer.
    #[must_use]
    pub fn new(config: PrinterConfig) -> Arc<Self> {
        // reqwest is built without a bundled TLS provider; install ring once per process.
        // An error only means another component already installed one.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .timeout(config.callback_timeout)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .user_agent(concat!("ebc-demo-printer/", env!("CARGO_PKG_VERSION")))
            .build()
            .ok();
        Arc::new(Self { config, http, inner: Mutex::new(Inner::default()) })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A poisoned lock only means a handler panicked mid-update; keep serving.
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Registers an order that passed preflight.
    #[must_use]
    pub fn register(&self, new: NewOrder) -> Registration {
        let now = self.config.clock.now();
        let mut inner = self.lock();
        if let Some(existing) =
            inner.by_reference.get(&new.reference).and_then(|id| inner.orders.get(id))
        {
            let same = existing.pdf_sha256 == new.pdf_sha256
                && existing.product == new.product.id
                && existing.quantity == new.quantity;
            return if same { Registration::Replayed(existing.clone()) } else { Registration::Conflict };
        }
        if inner.orders.len() >= MAX_ORDERS {
            return Registration::Full;
        }
        inner.sequence += 1;
        let sequence = inner.sequence;
        let digest = Sha256::new()
            .chain_update(self.config.id_salt.as_bytes())
            .chain_update(sequence.to_be_bytes())
            .chain_update(new.reference.as_bytes())
            .chain_update(new.pdf_sha256.as_bytes())
            .finalize();
        let order_id = format!("po_{}", hex(&digest[..8]));
        let order = Order {
            order_id: order_id.clone(),
            product: new.product.id,
            quantity: new.quantity,
            reference: new.reference.clone(),
            address: new.address,
            callback_url: new.callback_url,
            status: OrderStatus::Accepted,
            price_cents: new.price_cents,
            currency: new.product.currency,
            tracking: None,
            created_at: now,
            updated_at: now,
            history: vec![HistoryEntry { status: OrderStatus::Accepted, at: now }],
            pdf_sha256: new.pdf_sha256,
            pdf_bytes: new.pdf_bytes,
            preflight_warnings: new.preflight_warnings,
            callbacks: Vec::new(),
            sequence,
        };
        inner.by_reference.insert(new.reference, order_id.clone());
        inner.orders.insert(order_id, order.clone());
        Registration::Created(order)
    }

    /// The order `order_id`.
    #[must_use]
    pub fn order(&self, order_id: &str) -> Option<Order> {
        self.lock().orders.get(order_id).cloned()
    }

    /// Every order, oldest first.
    #[must_use]
    pub fn orders(&self) -> Vec<Order> {
        let mut orders: Vec<Order> = self.lock().orders.values().cloned().collect();
        orders.sort_by_key(|o| o.sequence);
        orders
    }

    /// Moves an order to `status` and returns its new state, or `None` when the order does
    /// not exist or `expected` is not its current status any more.
    fn transition(&self, order_id: &str, expected: OrderStatus, status: OrderStatus) -> Option<Order> {
        let now = self.config.clock.now();
        let mut inner = self.lock();
        let order = inner.orders.get_mut(order_id)?;
        if order.status != expected {
            return None;
        }
        order.status = status;
        order.updated_at = now;
        order.history.push(HistoryEntry { status, at: now });
        if status == OrderStatus::Shipped {
            order.tracking = Some(format!("DC{:08}FR", order.sequence));
        }
        Some(order.clone())
    }

    /// Advances one order by one step. Returns the new state, or `None` when it cannot
    /// move (unknown, shipped or cancelled).
    fn step(&self, order_id: &str) -> Option<Order> {
        let current = self.order(order_id)?;
        self.transition(order_id, current.status, current.status.next()?)
    }

    /// Advances every order that can move by one step and delivers the callbacks. Returns
    /// the orders that changed. This is what `POST /__test/advance` calls.
    pub async fn advance_all(&self) -> Vec<Order> {
        let mut changed = Vec::new();
        for order in self.orders() {
            if let Some(updated) = self.step(&order.order_id) {
                self.notify(&updated).await;
                changed.push(self.order(&updated.order_id).unwrap_or(updated));
            }
        }
        changed
    }

    /// Cancels an order that is still `accepted`.
    ///
    /// # Errors
    /// See [`CancelError`].
    pub async fn cancel(&self, order_id: &str) -> Result<Order, CancelError> {
        let current = self.order(order_id).ok_or(CancelError::Unknown)?;
        if current.status == OrderStatus::Cancelled {
            return Ok(current);
        }
        let cancelled = self
            .transition(order_id, OrderStatus::Accepted, OrderStatus::Cancelled)
            .ok_or(CancelError::TooLate)?;
        self.notify(&cancelled).await;
        Ok(self.order(order_id).unwrap_or(cancelled))
    }

    /// Runs the life cycle of one order in automatic mode: one step per tick until it is
    /// shipped or cancelled.
    pub async fn run(self: Arc<Self>, order_id: String, tick: Duration) {
        loop {
            tokio::time::sleep(tick).await;
            match self.step(&order_id) {
                Some(updated) => {
                    self.notify(&updated).await;
                    if updated.status.next().is_none() {
                        return;
                    }
                }
                None => return,
            }
        }
    }

    /// Delivers the current status of `order` to its callback URL (a few attempts), and
    /// records the outcome in the order's callback log.
    async fn notify(&self, order: &Order) {
        let Some(url) = order.callback_url.as_deref() else { return };
        let body = serde_json::json!({
            "order_id": order.order_id,
            "reference": order.reference,
            "status": order.status,
            "tracking": order.tracking,
            "at": order.updated_at,
        });
        let mut outcome = CallbackAttempt { status: order.status, http_status: None, delivered: false };
        for attempt in 0..CALLBACK_ATTEMPTS {
            if attempt > 0 {
                tokio::time::sleep(self.config.callback_retry_delay).await;
            }
            let Some(http) = self.http.as_ref() else { break };
            if let Ok(response) = http.post(url).json(&body).send().await {
                outcome.http_status = Some(response.status().as_u16());
                outcome.delivered = response.status().is_success();
                // A 4xx will not get better by retrying.
                if outcome.delivered || response.status().is_client_error() {
                    break;
                }
            }
        }
        let mut inner = self.lock();
        if let Some(stored) = inner.orders.get_mut(&order.order_id)
            && stored.callbacks.len() < MAX_CALLBACK_LOG
        {
            stored.callbacks.push(outcome);
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// SHA-256 of `data`, as lower-case hex.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    fn new_order(reference: &str, sha: &str) -> NewOrder {
        NewOrder {
            product: catalog::find("card-poche").unwrap(),
            quantity: 1,
            reference: reference.to_owned(),
            address: Address {
                name: "Sacha Martin".into(),
                line1: "12 rue des Lilas".into(),
                line2: None,
                postal_code: "75011".into(),
                city: "Paris".into(),
                country: "FR".into(),
            },
            callback_url: None,
            price_cents: 590,
            pdf_sha256: sha.to_owned(),
            pdf_bytes: 1234,
            preflight_warnings: Vec::new(),
        }
    }

    fn created(registration: Registration) -> Order {
        match registration {
            Registration::Created(order) => order,
            other => panic!("expected a new order, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn life_cycle_advances_one_step_at_a_time() {
        let state = PrinterState::new(PrinterConfig::manual());
        let order = created(state.register(new_order("REF-1", "aa")));
        assert_eq!(order.status, OrderStatus::Accepted);
        assert!(order.order_id.starts_with("po_") && order.order_id.len() == 19);
        assert_eq!(order.tracking, None);

        let changed = state.advance_all().await;
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].status, OrderStatus::Printing);
        let changed = state.advance_all().await;
        assert_eq!(changed[0].status, OrderStatus::Shipped);
        assert_eq!(changed[0].tracking.as_deref(), Some("DC00000001FR"));
        assert!(state.advance_all().await.is_empty(), "shipped is final");
        let history: Vec<_> =
            state.order(&order.order_id).unwrap().history.iter().map(|h| h.status).collect();
        assert_eq!(history, [OrderStatus::Accepted, OrderStatus::Printing, OrderStatus::Shipped]);
    }

    #[tokio::test]
    async fn cancellation_is_only_possible_before_printing() {
        let state = PrinterState::new(PrinterConfig::manual());
        let a = created(state.register(new_order("REF-A", "aa")));
        let b = created(state.register(new_order("REF-B", "bb")));
        assert_eq!(state.cancel(&a.order_id).await.unwrap().status, OrderStatus::Cancelled);
        assert_eq!(
            state.cancel(&a.order_id).await.unwrap().status,
            OrderStatus::Cancelled,
            "cancelling twice is harmless"
        );
        let changed = state.advance_all().await;
        assert_eq!(changed.len(), 1, "a cancelled order does not move");
        assert_eq!(changed[0].order_id, b.order_id);
        assert_eq!(state.cancel(&b.order_id).await, Err(CancelError::TooLate));
        assert_eq!(state.cancel("po_missing").await, Err(CancelError::Unknown));
    }

    #[test]
    fn references_are_idempotent_keys() {
        let state = PrinterState::new(PrinterConfig::manual());
        let first = created(state.register(new_order("REF-1", "aa")));
        match state.register(new_order("REF-1", "aa")) {
            Registration::Replayed(order) => assert_eq!(order.order_id, first.order_id),
            other => panic!("expected a replay, got {other:?}"),
        }
        assert_eq!(state.register(new_order("REF-1", "bb")), Registration::Conflict);
        let mut more = new_order("REF-1", "aa");
        more.quantity = 2;
        assert_eq!(state.register(more), Registration::Conflict);
        assert_eq!(state.orders().len(), 1);
    }

    #[test]
    fn identifiers_are_unique_and_depend_on_the_salt() {
        let state = PrinterState::new(PrinterConfig::manual());
        let a = created(state.register(new_order("REF-1", "aa")));
        let b = created(state.register(new_order("REF-2", "aa")));
        assert_ne!(a.order_id, b.order_id);
        let other = PrinterState::new(PrinterConfig { id_salt: "other".into(), ..PrinterConfig::manual() });
        let c = created(other.register(new_order("REF-1", "aa")));
        assert_ne!(a.order_id, c.order_id);
        assert_eq!(state.orders().iter().map(|o| o.reference.as_str()).collect::<Vec<_>>(), ["REF-1", "REF-2"]);
    }

    #[tokio::test]
    async fn automatic_mode_reaches_shipped_on_its_own() {
        let config = PrinterConfig { tick: Some(Duration::from_millis(10)), ..PrinterConfig::manual() };
        let state = PrinterState::new(config);
        let order = created(state.register(new_order("REF-1", "aa")));
        let task = tokio::spawn(state.clone().run(order.order_id.clone(), Duration::from_millis(10)));
        tokio::time::timeout(Duration::from_secs(10), task).await.unwrap().unwrap();
        assert_eq!(state.order(&order.order_id).unwrap().status, OrderStatus::Shipped);
    }

    #[test]
    fn sha256_is_hex() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
