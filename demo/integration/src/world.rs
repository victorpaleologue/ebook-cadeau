//! [`World`]: the constellation, started in-process.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use ebc_demo_ereaders::{DeviceProfile, EinkBrowser, MailDevice};
use ebc_demo_printer::{PrinterConfig, PrinterState};
use ebc_demo_stores::{StoreConfig, StoreKind};
use ebc_server::{AppState, CardRenderer, Clock, Config, ManualClock, Partner, SystemClock};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::partner::PartnerApi;

/// The instant a manual-clock [`World`] starts at: 2026-11-12T00:00:00Z.
pub const T0: i64 = 1_794_441_600;

/// Why the constellation could not be started or used.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorldError {
    /// A listening socket could not be opened.
    #[error("cannot listen on {address}: {source}")]
    Bind {
        /// The address that was asked for.
        address: String,
        /// The operating system's reason.
        source: std::io::Error,
    },
    /// The eBook Cadeau server refused its configuration.
    #[error("the eBook Cadeau server could not start: {0}")]
    Server(String),
    /// The mail-to-device gateway could not start.
    #[error("the mail device could not start: {0}")]
    Mail(String),
    /// A simulated device failed (transport error, missing element, refused download…).
    #[error("simulated device: {0}")]
    Device(#[from] ebc_demo_ereaders::Error),
    /// An HTTP call made by a test helper failed before an answer was received.
    #[error("HTTP call failed: {0}")]
    Http(String),
    /// A page did not look like what the journey expected.
    #[error("unexpected page: {0}")]
    Journey(String),
}

/// TCP ports of the four HTTP services (`0` = any free port).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPorts {
    /// The eBook Cadeau server.
    pub server: u16,
    /// The store "Septante".
    pub septante: u16,
    /// The store "Grande Enseigne".
    pub grande_enseigne: u16,
    /// The printer.
    pub printer: u16,
}

impl WorldPorts {
    /// Free ports chosen by the operating system: what tests use.
    pub const EPHEMERAL: Self = Self { server: 0, septante: 0, grande_enseigne: 0, printer: 0 };
    /// The documented demo ports: server 8080, Septante 8101, Grande Enseigne 8102,
    /// printer 8103 (the defaults of the individual binaries and of
    /// `crates/ebc-server/partners.dev.toml`).
    pub const DEMO: Self =
        Self { server: 8080, septante: 8101, grande_enseigne: 8102, printer: 8103 };
}

/// The time every component of a [`World`] shares.
#[derive(Debug, Clone)]
pub enum WorldClock {
    /// A clock that only moves when a test says so.
    Manual(Arc<ManualClock>),
    /// The system clock (for the demo binary).
    System,
}

impl WorldClock {
    /// Seconds since the Unix epoch.
    #[must_use]
    pub fn now(&self) -> i64 {
        match self {
            Self::Manual(clock) => clock.now_unix(),
            Self::System => SystemClock.now_unix(),
        }
    }

    /// Moves a manual clock forward by `seconds`. Returns `false` (and does nothing) for
    /// the system clock.
    pub fn advance(&self, seconds: i64) -> bool {
        match self {
            Self::Manual(clock) => {
                clock.advance(seconds);
                true
            }
            Self::System => false,
        }
    }

    fn for_server(&self) -> Arc<dyn Clock> {
        match self {
            Self::Manual(clock) => Arc::clone(clock) as Arc<dyn Clock>,
            Self::System => Arc::new(SystemClock),
        }
    }

    fn for_store(&self) -> ebc_demo_stores::Clock {
        match self {
            Self::Manual(clock) => {
                let clock = Arc::clone(clock);
                ebc_demo_stores::Clock::from_fn(move || clock.now_unix())
            }
            Self::System => ebc_demo_stores::Clock::system(),
        }
    }

    fn for_printer(&self) -> ebc_demo_printer::Clock {
        match self {
            // The printer's clock only stamps its order history; a fixed one is enough.
            Self::Manual(clock) => ebc_demo_printer::Clock::fixed(clock.now_unix()),
            Self::System => ebc_demo_printer::Clock::system(),
        }
    }
}

/// The running eBook Cadeau server.
#[derive(Debug, Clone)]
pub struct ServerHandle {
    /// Base URL, e.g. `http://127.0.0.1:41873` (also its `EBC_PUBLIC_URL`).
    pub base: String,
    /// The in-process state, for white-box checks (pending webhooks, configuration).
    pub state: AppState,
}

impl ServerHandle {
    /// `base` + `path` (`path` starts with `/`).
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// `host:port`, as people type it on an e-reader.
    #[must_use]
    pub fn host(&self) -> &str {
        self.base.split_once("://").map_or(self.base.as_str(), |(_, host)| host)
    }
}

/// One running demo store.
#[derive(Debug, Clone)]
pub struct StoreHandle {
    /// Which store.
    pub kind: StoreKind,
    /// Base URL, e.g. `http://127.0.0.1:40111`.
    pub base: String,
    /// The in-process state ([`ebc_demo_stores::AppState::snapshot`] is the test hook).
    pub state: Arc<ebc_demo_stores::AppState>,
    /// Its `X-EBC-Partner` identifier at the server.
    pub partner_id: String,
    /// The secret it shares with the server (signs requests, redirects and webhooks).
    pub partner_secret: String,
}

impl StoreHandle {
    /// `base` + `path` (`path` starts with `/`).
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// `host:port` of the store, as listed in the server's `redirect_hosts`.
    #[must_use]
    pub fn host(&self) -> &str {
        self.base.split_once("://").map_or(self.base.as_str(), |(_, host)| host)
    }

    /// The store's state as JSON (orders, gifts, accounts, received webhooks).
    #[must_use]
    pub fn snapshot(&self) -> serde_json::Value {
        self.state.snapshot()
    }
}

/// The running demo printer.
#[derive(Debug, Clone)]
pub struct PrinterHandle {
    /// Base URL, e.g. `http://127.0.0.1:40222`.
    pub base: String,
    /// The in-process state (orders received, manual life-cycle stepping).
    pub state: Arc<PrinterState>,
}

type ServerTweak = Box<dyn FnOnce(&mut Config) + Send>;
type StoreTweak = Box<dyn Fn(StoreConfig) -> StoreConfig + Send>;

/// Configures and starts a [`World`]. Obtain one with [`World::builder`].
pub struct WorldBuilder {
    ports: WorldPorts,
    clock: WorldClock,
    mail: Option<(DeviceProfile, String)>,
    partners: Vec<Partner>,
    server_tweaks: Vec<ServerTweak>,
    store_tweak: Option<StoreTweak>,
    card_renderer: Option<Arc<dyn CardRenderer>>,
    printer_tick: Option<Duration>,
    fast_passwords: bool,
}

impl std::fmt::Debug for WorldBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldBuilder")
            .field("ports", &self.ports)
            .field("clock", &self.clock)
            .field("mail", &self.mail.as_ref().map(|(profile, address)| (profile.id, address)))
            .field("partners", &self.partners)
            .finish_non_exhaustive()
    }
}

impl WorldBuilder {
    fn new() -> Self {
        Self {
            ports: WorldPorts::EPHEMERAL,
            clock: WorldClock::Manual(Arc::new(ManualClock::new(T0))),
            mail: None,
            partners: Vec::new(),
            server_tweaks: Vec::new(),
            store_tweak: None,
            card_renderer: None,
            printer_tick: None,
            fast_passwords: true,
        }
    }

    /// Listens on these ports instead of ephemeral ones.
    #[must_use]
    pub fn ports(mut self, ports: WorldPorts) -> Self {
        self.ports = ports;
        self
    }

    /// Runs on the system clock (the default is a manual clock stopped at [`T0`]).
    #[must_use]
    pub fn system_clock(mut self) -> Self {
        self.clock = WorldClock::System;
        self
    }

    /// Also starts the mail-to-device gateway of `profile` (Send to Kindle,
    /// Send-to-PocketBook) for the device address `address`, and points the server's
    /// `EBC_SMTP_URL` at it.
    #[must_use]
    pub fn mail_device(mut self, profile: DeviceProfile, address: &str) -> Self {
        self.mail = Some((profile, address.to_owned()));
        self
    }

    /// Registers one more partner with the server, next to the two stores (for tests that
    /// call the partner API directly, e.g. with their own licence endpoint).
    #[must_use]
    pub fn partner(mut self, partner: Partner) -> Self {
        self.partners.push(partner);
        self
    }

    /// Adjusts the server configuration after the constellation's own wiring was applied
    /// (rate limits, time-outs, session limits…).
    #[must_use]
    pub fn server_config(mut self, tweak: impl FnOnce(&mut Config) + Send + 'static) -> Self {
        self.server_tweaks.push(Box::new(tweak));
        self
    }

    /// Adjusts the configuration of both stores after the constellation's own wiring.
    #[must_use]
    pub fn store_config(
        mut self,
        tweak: impl Fn(StoreConfig) -> StoreConfig + Send + 'static,
    ) -> Self {
        self.store_tweak = Some(Box::new(tweak));
        self
    }

    /// Replaces the server's card renderer (the default is the real card composer;
    /// `ebc_server::NoCardRenderer` makes `card.pdf` answer `501`).
    #[must_use]
    pub fn card_renderer(mut self, renderer: Arc<dyn CardRenderer>) -> Self {
        self.card_renderer = Some(renderer);
        self
    }

    /// Lets printer orders advance by themselves every `tick` (the default is manual
    /// stepping through [`PrinterState::advance_all`]).
    #[must_use]
    pub fn printer_tick(mut self, tick: Duration) -> Self {
        self.printer_tick = Some(tick);
        self
    }

    /// Keeps the stores' demo-grade password hashing at its normal cost (tests lower it,
    /// which is the default here).
    #[must_use]
    pub fn slow_passwords(mut self) -> Self {
        self.fast_passwords = false;
        self
    }

    /// Binds every socket, wires the components to each other and starts serving.
    ///
    /// # Errors
    /// [`WorldError::Bind`] when a port is taken, [`WorldError::Server`] /
    /// [`WorldError::Mail`] when a component refuses to start.
    pub async fn start(self) -> Result<World, WorldError> {
        // Bind first: every component needs to know the others' URLs before it starts.
        let (server_listener, server_base) = bind(self.ports.server).await?;
        let (septante_listener, septante_base) = bind(self.ports.septante).await?;
        let (grande_listener, grande_base) = bind(self.ports.grande_enseigne).await?;
        let (printer_listener, printer_base) = bind(self.ports.printer).await?;

        let mail = match &self.mail {
            Some((profile, address)) => Some(
                MailDevice::start(profile.clone(), address)
                    .await
                    .map_err(|e| WorldError::Mail(e.to_string()))?,
            ),
            None => None,
        };

        // --- the printer ---
        let printer_state = PrinterState::new(PrinterConfig {
            tick: self.printer_tick,
            clock: self.clock.for_printer(),
            ..PrinterConfig::manual()
        });
        let mut tasks = Vec::new();
        tasks
            .push(spawn(printer_listener, ebc_demo_printer::app_with_state(printer_state.clone())));

        // --- the server ---
        let store_partner = |kind: StoreKind, base: &str| Partner {
            id: kind.default_partner_id().to_owned(),
            name: format!("{} (démo)", kind.display_name()),
            secret: kind.default_partner_secret().to_owned(),
            webhook_url: Some(format!("{base}/webhooks/ebc")),
            redirect_hosts: vec![host_of(base).to_owned()],
        };
        let mut config = Config::for_tests();
        config.bind = host_of(&server_base).to_owned();
        config.public_url.clone_from(&server_base);
        config.partners = vec![
            store_partner(StoreKind::Septante, &septante_base),
            store_partner(StoreKind::GrandeEnseigne, &grande_base),
        ];
        config.partners.extend(self.partners);
        config.printer_url = Some(printer_base.clone());
        config.smtp_url = mail.as_ref().map(MailDevice::smtp_url);
        // Local components answer at once or not at all: retry quickly, give up quickly.
        config.webhook.base_delay_ms = 50;
        config.webhook.max_delay_ms = 400;
        config.webhook.timeout_ms = 5_000;
        config.outbound_timeout_ms = 10_000;
        for tweak in self.server_tweaks {
            tweak(&mut config);
        }
        let mut builder = AppState::builder(config).clock(self.clock.for_server());
        if let Some(renderer) = self.card_renderer {
            builder = builder.card_renderer(renderer);
        }
        let server_state = builder.build().map_err(|e| WorldError::Server(e.to_string()))?;
        let router = ebc_server::app(server_state.clone());
        tasks.push(tokio::spawn(async move {
            // Connection information lets the server tell clients apart by address, as
            // its own binary does.
            let service = router.into_make_service_with_connect_info::<SocketAddr>();
            let _ = axum::serve(server_listener, service).await;
        }));

        // --- the stores ---
        let fast_passwords = self.fast_passwords;
        let clock = self.clock.clone();
        let store_tweak = self.store_tweak;
        let mut start_store = |kind: StoreKind, listener: TcpListener, base: String| {
            let mut store_config = StoreConfig::new(kind)
                .with_public_url(&base)
                .with_partner(
                    &server_base,
                    kind.default_partner_id(),
                    kind.default_partner_secret(),
                )
                .with_printer(&printer_base)
                .with_clock(clock.for_store())
                .with_test_hooks(true);
            if fast_passwords {
                store_config.password_iterations = 3;
            }
            if let Some(tweak) = &store_tweak {
                store_config = tweak(store_config);
            }
            let state = ebc_demo_stores::AppState::new(store_config);
            tasks.push(spawn(listener, ebc_demo_stores::app_with_state(state.clone())));
            StoreHandle {
                kind,
                base,
                state,
                partner_id: kind.default_partner_id().to_owned(),
                partner_secret: kind.default_partner_secret().to_owned(),
            }
        };
        let septante = start_store(StoreKind::Septante, septante_listener, septante_base);
        let grande_enseigne = start_store(StoreKind::GrandeEnseigne, grande_listener, grande_base);

        Ok(World {
            clock: self.clock,
            server: ServerHandle { base: server_base, state: server_state },
            septante,
            grande_enseigne,
            printer: PrinterHandle { base: printer_base, state: printer_state },
            mail,
            tasks,
        })
    }
}

/// Binds `127.0.0.1:port` and returns the listener with its base URL.
async fn bind(port: u16) -> Result<(TcpListener, String), WorldError> {
    let address = format!("127.0.0.1:{port}");
    let bind_error = |source| WorldError::Bind { address: address.clone(), source };
    let listener = TcpListener::bind(&address).await.map_err(bind_error)?;
    let local = listener.local_addr().map_err(bind_error)?;
    Ok((listener, format!("http://{local}")))
}

fn spawn(listener: TcpListener, router: axum::Router) -> JoinHandle<()> {
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    })
}

fn host_of(base: &str) -> &str {
    base.split_once("://").map_or(base, |(_, host)| host)
}

/// The running constellation: base URLs, secrets, the shared clock and the in-process
/// states. Dropping it stops every component.
#[derive(Debug)]
pub struct World {
    /// The time every component shares.
    pub clock: WorldClock,
    /// The eBook Cadeau server.
    pub server: ServerHandle,
    /// "Septante": DRM-free store, gifts delivered as `partner_file` (it serves the file).
    pub septante: StoreHandle,
    /// "Grande Enseigne": ecosystem store, gifts delivered as `reseller_redirect`.
    pub grande_enseigne: StoreHandle,
    /// The print provider both the server and the stores talk to.
    pub printer: PrinterHandle,
    /// The mail-to-device gateway, when [`WorldBuilder::mail_device`] asked for one.
    pub mail: Option<MailDevice>,
    tasks: Vec<JoinHandle<()>>,
}

impl World {
    /// A builder with the test defaults: ephemeral ports, a manual clock stopped at
    /// [`T0`], manual printer stepping, cheap password hashing, no mail gateway.
    #[must_use]
    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    /// Starts the constellation with the test defaults.
    ///
    /// # Errors
    /// See [`WorldBuilder::start`].
    pub async fn start() -> Result<Self, WorldError> {
        Self::builder().start().await
    }

    /// The store of this kind.
    #[must_use]
    pub fn store(&self, kind: StoreKind) -> &StoreHandle {
        match kind {
            StoreKind::Septante => &self.septante,
            StoreKind::GrandeEnseigne => &self.grande_enseigne,
        }
    }

    /// Moves the shared manual clock forward (see [`WorldClock::advance`]).
    pub fn advance(&self, seconds: i64) -> bool {
        self.clock.advance(seconds)
    }

    /// A simulated browser of `profile` with default settings (no JavaScript, the
    /// profile's `User-Agent`, tap counting).
    ///
    /// # Errors
    /// [`WorldError::Device`] when the HTTP client cannot be built.
    pub fn browser(&self, profile: DeviceProfile) -> Result<EinkBrowser, WorldError> {
        Ok(EinkBrowser::new(profile)?)
    }

    /// A partner-API client signing as the store of this kind.
    #[must_use]
    pub fn partner_api(&self, kind: StoreKind) -> PartnerApi {
        let store = self.store(kind);
        self.partner_api_as(&store.partner_id, &store.partner_secret)
    }

    /// A partner-API client signing with any identifier and secret (for partners added
    /// with [`WorldBuilder::partner`], and for requests that must be refused).
    #[must_use]
    pub fn partner_api_as(&self, partner_id: &str, secret: &str) -> PartnerApi {
        PartnerApi::new(&self.server.base, partner_id, secret, self.clock.clone())
    }

    /// Waits until the server has delivered (or given up on) every queued webhook, at
    /// most `timeout`. Returns whether the outbox is empty.
    pub async fn webhooks_delivered(&self, timeout: Duration) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.server.state.pending_webhooks() == 0 {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// What runs where, for a printed table: `(name, URL, note)`.
    #[must_use]
    pub fn services(&self) -> Vec<(String, String, String)> {
        let mut rows = vec![
            (
                "eBook Cadeau (server)".to_owned(),
                self.server.base.clone(),
                "code form at /, help at /aide, pairing at /e".to_owned(),
            ),
            (
                "Septante (démo)".to_owned(),
                self.septante.base.clone(),
                "fictional DRM-free store — gifts as partner_file (it serves the file)".to_owned(),
            ),
            (
                "Grande Enseigne (démo)".to_owned(),
                self.grande_enseigne.base.clone(),
                "fictional ecosystem store — gifts as reseller_redirect".to_owned(),
            ),
            (
                "Imprimerie du Dos Carré (démo)".to_owned(),
                self.printer.base.clone(),
                "fictional printer — GET /v1/products, /__test/orders".to_owned(),
            ),
        ];
        if let Some(mail) = &self.mail {
            rows.push((
                "Mail-to-device gateway".to_owned(),
                mail.smtp_url(),
                format!("SMTP sink for {}", mail.address()),
            ));
        }
        rows
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
