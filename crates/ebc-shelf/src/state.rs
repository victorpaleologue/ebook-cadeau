//! The shared state of the service: configuration, records, the core client, the mailer,
//! the in-memory download links and the rate limits.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::sync::Semaphore;

use crate::clock::{Clock, Entropy, EntropyError, OsEntropy, SystemClock};
use crate::config::Config;
use crate::core::{CoreClient, CoreError};
use crate::crypto;
use crate::mail::{Mailer, MemoryMailer, NoMailer, SmtpMailer};
use crate::store::{Db, MemoryStore, Store, StoreError};

/// A problem that stops the service from starting.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ShelfError {
    /// The configuration is unusable.
    #[error("configuration: {0}")]
    Config(String),
    /// The data directory cannot be used.
    #[error("storage: {0}")]
    Storage(String),
    /// The core client cannot be built.
    #[error("core client: {0}")]
    Core(String),
    /// Listening failed.
    #[error("I/O: {0}")]
    Io(String),
}

impl From<std::io::Error> for ShelfError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// A download link handed to the core (§4.5), kept in memory only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) gift_id: String,
    pub(crate) book_id: String,
    pub(crate) sha256: String,
    pub(crate) file_name: String,
    pub(crate) media_type: &'static str,
    pub(crate) expires_at: i64,
}

/// Fixed-window counters keyed by client, address or account.
#[derive(Debug, Default)]
pub(crate) struct Counters {
    map: Mutex<HashMap<String, (i64, u32)>>,
}

impl Counters {
    fn map(&self) -> MutexGuard<'_, HashMap<String, (i64, u32)>> {
        self.map.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Counts one more event for `key` and says whether it is within `limit` per
    /// `window` seconds (`0` = unlimited). Returns the seconds to wait when it is not.
    pub(crate) fn hit(&self, key: &str, limit: u32, window: i64, now: i64) -> Result<(), i64> {
        if limit == 0 {
            return Ok(());
        }
        let mut map = self.map();
        if map.len() > 100_000 {
            map.retain(|_, (start, _)| now - *start < window);
        }
        let entry = map.entry(key.to_owned()).or_insert((now, 0));
        if now - entry.0 >= window {
            *entry = (now, 0);
        }
        if entry.1 >= limit {
            return Err((entry.0 + window - now).max(1));
        }
        entry.1 += 1;
        Ok(())
    }

    /// Whether `key` already spent its budget (without counting anything).
    pub(crate) fn spent(&self, key: &str, limit: u32, window: i64, now: i64) -> Option<i64> {
        if limit == 0 {
            return None;
        }
        let map = self.map();
        let (start, count) = map.get(key)?;
        (now - start < window && *count >= limit).then(|| (start + window - now).max(1))
    }
}

/// Everything a request needs.
pub struct Inner {
    /// The configuration.
    pub config: Config,
    pub(crate) store: Arc<dyn Store>,
    db: Mutex<Db>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) entropy: Arc<dyn Entropy>,
    pub(crate) mailer: Arc<dyn Mailer>,
    pub(crate) core: CoreClient,
    links: Mutex<HashMap<String, Link>>,
    pub(crate) counters: Counters,
    pub(crate) uploads: Semaphore,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner").field("config", &self.config).finish_non_exhaustive()
    }
}

/// The service state (cheap to clone).
#[derive(Debug, Clone)]
pub struct AppState(pub(crate) Arc<Inner>);

impl std::ops::Deref for AppState {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.0
    }
}

/// Builds an [`AppState`].
pub struct AppStateBuilder {
    config: Config,
    store: Option<Arc<dyn Store>>,
    clock: Option<Arc<dyn Clock>>,
    entropy: Option<Arc<dyn Entropy>>,
    mailer: Option<Arc<dyn Mailer>>,
}

impl std::fmt::Debug for AppStateBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateBuilder").field("config", &self.config).finish_non_exhaustive()
    }
}

impl AppStateBuilder {
    /// Where records and files live (default: in memory).
    #[must_use]
    pub fn store(mut self, store: Arc<dyn Store>) -> Self {
        self.store = Some(store);
        self
    }

    /// The clock (default: the system clock).
    #[must_use]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// The random source (default: the OS).
    #[must_use]
    pub fn entropy(mut self, entropy: Arc<dyn Entropy>) -> Self {
        self.entropy = Some(entropy);
        self
    }

    /// The mailer (default: SMTP to `EBC_SMTP_URL`, or none).
    #[must_use]
    pub fn mailer(mut self, mailer: Arc<dyn Mailer>) -> Self {
        self.mailer = Some(mailer);
        self
    }

    /// Loads the records and builds the state.
    ///
    /// # Errors
    /// [`ShelfError::Config`] when the secret is missing, [`ShelfError::Storage`] when the
    /// records cannot be read, [`ShelfError::Core`] when the HTTP client cannot be built.
    pub fn build(self) -> Result<AppState, ShelfError> {
        let config = self.config;
        if config.secret.chars().count() < 16 {
            return Err(ShelfError::Config("EBC_SHELF_SECRET: at least 16 characters".to_owned()));
        }
        let store = self.store.unwrap_or_else(|| Arc::new(MemoryStore::new()));
        let db = store.load().map_err(|e| ShelfError::Storage(e.to_string()))?;
        let timeout = Duration::from_millis(config.outbound_timeout_ms.max(1));
        let core =
            CoreClient::new(&config.core_url, &config.partner_id, &config.partner_secret, timeout)
                .map_err(|e| ShelfError::Core(e.to_string()))?;
        let mailer: Arc<dyn Mailer> = match (self.mailer, &config.smtp_url) {
            (Some(mailer), _) => mailer,
            (None, Some(url)) => Arc::new(SmtpMailer::new(url, &config.mail_from, timeout)),
            (None, None) => Arc::new(NoMailer),
        };
        let uploads =
            Semaphore::new(usize::try_from(config.max_concurrent_uploads.max(1)).unwrap_or(1));
        Ok(AppState(Arc::new(Inner {
            config,
            store,
            db: Mutex::new(db),
            clock: self.clock.unwrap_or_else(|| Arc::new(SystemClock)),
            entropy: self.entropy.unwrap_or_else(|| Arc::new(OsEntropy)),
            mailer,
            core,
            links: Mutex::new(HashMap::new()),
            counters: Counters::default(),
            uploads,
        })))
    }
}

/// Why a write failed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum WriteError<E> {
    /// The change was refused (nothing was written).
    #[error("refused")]
    Refused(E),
    /// The change could not be persisted (nothing was changed).
    #[error("storage: {0}")]
    Storage(StoreError),
}

impl AppState {
    /// A builder for `config`.
    #[must_use]
    pub fn builder(config: Config) -> AppStateBuilder {
        AppStateBuilder { config, store: None, clock: None, entropy: None, mailer: None }
    }

    /// An in-memory state with the test configuration and an in-memory outbox.
    ///
    /// # Panics
    /// Never with the test configuration (the builder cannot fail on it).
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn for_tests() -> (Self, Arc<MemoryMailer>) {
        let mailer = Arc::new(MemoryMailer::new());
        let state = Self::builder(Config::for_tests())
            .mailer(mailer.clone())
            .build()
            .expect("the test configuration is valid");
        (state, mailer)
    }

    /// The current time.
    #[must_use]
    pub fn now(&self) -> i64 {
        self.clock.now_unix()
    }

    /// `n` random bytes as hex.
    pub(crate) fn random_hex(&self, n: usize) -> Result<String, EntropyError> {
        let mut bytes = vec![0u8; n];
        self.entropy.fill(&mut bytes)?;
        Ok(crypto::hex(&bytes))
    }

    /// A new record id: `prefix` + 16 hex digits.
    pub(crate) fn new_id(&self, prefix: &str) -> Result<String, EntropyError> {
        Ok(format!("{prefix}{}", self.random_hex(8)?))
    }

    fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Reads the records.
    pub(crate) fn read<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(&self.db())
    }

    /// A copy of every record (tests and white-box checks).
    #[must_use]
    pub fn snapshot(&self) -> Db {
        self.db().clone()
    }

    /// Changes the records: `f` works on a copy, which is persisted and then replaces the
    /// records; if `f` refuses or the store fails, nothing changes.
    pub(crate) fn write<T, E>(
        &self,
        f: impl FnOnce(&mut Db) -> Result<T, E>,
    ) -> Result<T, WriteError<E>> {
        let mut db = self.db();
        let mut copy = db.clone();
        let value = f(&mut copy).map_err(WriteError::Refused)?;
        self.store.save(&copy).map_err(WriteError::Storage)?;
        *db = copy;
        Ok(value)
    }

    /// Records a download link (pruning expired ones).
    pub(crate) fn add_link(&self, token_hash: String, link: Link) {
        let now = self.now();
        let mut links = self.links.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        links.retain(|_, l| l.expires_at > now);
        links.insert(token_hash, link);
    }

    /// The live link behind `token_hash`.
    pub(crate) fn link(&self, token_hash: &str) -> Option<Link> {
        let now = self.now();
        let links = self.links.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        links.get(token_hash).filter(|l| l.expires_at > now).cloned()
    }

    /// Forgets every link to the files of these books (takedown, deletion).
    pub(crate) fn drop_links_of(&self, book_ids: &[String]) {
        let mut links = self.links.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        links.retain(|_, l| !book_ids.contains(&l.book_id));
    }

    /// Number of live download links (tests).
    #[must_use]
    pub fn live_links(&self) -> usize {
        let now = self.now();
        let links = self.links.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        links.values().filter(|l| l.expires_at > now).count()
    }

    /// The core client.
    pub(crate) fn core(&self) -> &CoreClient {
        &self.core
    }
}

impl From<CoreError> for ShelfError {
    fn from(error: CoreError) -> Self {
        Self::Core(error.to_string())
    }
}
