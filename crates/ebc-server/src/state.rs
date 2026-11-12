//! Shared state of the service: configuration, storage, clock, random source, rate
//! limiters, outbound clients and the card renderer.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::card::CardRenderer;
use crate::clock::{Clock, Entropy, OsEntropy, SystemClock};
use crate::config::Config;
use crate::error::ServerError;
use crate::rate::RateLimiter;
use crate::store::{Database, MemoryStore, Store};

/// What every request handler shares. Cheap to clone.
#[derive(Clone)]
pub struct AppState(Arc<Inner>);

/// The fields behind an [`AppState`].
pub struct Inner {
    /// Configuration.
    pub config: Config,
    /// The data set and the stored files.
    pub db: Database,
    /// Time source.
    pub clock: Arc<dyn Clock>,
    /// Random source.
    pub entropy: Arc<dyn Entropy>,
    /// Budgets for unknown gift codes and pair codes (per client and global).
    pub code_limiter: RateLimiter,
    /// Budget for uploads and gift creations on the open creator API (per client).
    pub create_limiter: RateLimiter,
    /// Outbound HTTP client (webhooks, licences, print provider). Never follows redirects.
    pub http: reqwest::Client,
    /// Renders `card.pdf` for partners.
    pub card_renderer: Arc<dyn CardRenderer>,
    /// Idempotency keys of requests being processed right now.
    pub(crate) in_flight: Mutex<HashSet<String>>,
}

impl std::ops::Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").field("config", &self.config).finish_non_exhaustive()
    }
}

/// Builds an [`AppState`]; everything but the configuration has a default.
pub struct AppStateBuilder {
    config: Config,
    store: Option<Arc<dyn Store>>,
    clock: Arc<dyn Clock>,
    entropy: Arc<dyn Entropy>,
    card_renderer: Arc<dyn CardRenderer>,
}

impl AppStateBuilder {
    /// Where data is persisted (default: [`MemoryStore`]).
    #[must_use]
    pub fn store(mut self, store: Arc<dyn Store>) -> Self {
        self.store = Some(store);
        self
    }

    /// The time source (default: the system clock).
    #[must_use]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// The random source (default: the operating system's).
    #[must_use]
    pub fn entropy(mut self, entropy: Arc<dyn Entropy>) -> Self {
        self.entropy = entropy;
        self
    }

    /// The card renderer (default: [`crate::card::default_renderer`] — the card composer
    /// with the cargo feature `card`, `501` without it).
    #[must_use]
    pub fn card_renderer(mut self, renderer: Arc<dyn CardRenderer>) -> Self {
        self.card_renderer = renderer;
        self
    }

    /// Loads the data set and builds the outbound clients.
    ///
    /// # Errors
    /// [`ServerError::Storage`] when the store cannot be read, [`ServerError::Client`]
    /// when the HTTP client cannot be built.
    pub fn build(self) -> Result<AppState, ServerError> {
        let store = self.store.unwrap_or_else(|| Arc::new(MemoryStore::new()));
        let db = Database::open(store).map_err(|e| ServerError::Storage(e.to_string()))?;
        // reqwest is built without a bundled TLS provider: install ring's once. An error
        // only means a provider is already installed, which is what we want.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_millis(self.config.outbound_timeout_ms.max(1)))
            .user_agent(concat!("ebook-cadeau/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| ServerError::Client(e.to_string()))?;
        Ok(AppState(Arc::new(Inner {
            code_limiter: RateLimiter::new(
                self.config.rate_unknown_per_10min,
                self.config.rate_unknown_global_per_10min,
            ),
            create_limiter: RateLimiter::new(self.config.rate_create_per_10min, 0),
            config: self.config,
            db,
            clock: self.clock,
            entropy: self.entropy,
            http,
            card_renderer: self.card_renderer,
            in_flight: Mutex::new(HashSet::new()),
        })))
    }
}

impl AppState {
    /// Starts building a state around `config`.
    #[must_use]
    pub fn builder(config: Config) -> AppStateBuilder {
        AppStateBuilder {
            config,
            store: None,
            clock: Arc::new(SystemClock),
            entropy: Arc::new(OsEntropy),
            card_renderer: crate::card::default_renderer(),
        }
    }

    /// A state with the given configuration and store, the system clock and the system
    /// random source.
    ///
    /// # Errors
    /// See [`AppStateBuilder::build`].
    pub fn new(config: Config, store: Arc<dyn Store>) -> Result<Self, ServerError> {
        Self::builder(config).store(store).build()
    }

    /// An in-memory state with [`Config::for_tests`] (public development secret, no
    /// partners, no SMTP): for tests and in-process demos, **never for production**.
    ///
    /// # Panics
    /// Only if the HTTP client cannot be initialised, which does not depend on input.
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn for_tests() -> Self {
        Self::builder(Config::for_tests()).build().expect("in-memory state")
    }

    /// The current time.
    #[must_use]
    pub fn now(&self) -> i64 {
        self.clock.now_unix()
    }
}
