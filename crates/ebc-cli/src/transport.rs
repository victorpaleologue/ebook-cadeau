//! How HTTP requests leave the tool: over the network (cargo feature `net`) or from a
//! fixture directory (`--offline-fixtures`).

use std::path::Path;
use std::time::Duration;

use ebc_catalog::{LookupOptions, LookupReport, Provider, lookup_all};
use ebc_model::{HttpRequest, HttpResponse, Isbn13};

use crate::error::CliError;
use crate::fixtures::Fixtures;

/// Settings of the network transport.
#[derive(Debug, Clone)]
pub(crate) struct NetOptions {
    /// `User-Agent` of every request.
    pub(crate) user_agent: String,
    /// Time allowed per request.
    pub(crate) timeout: Duration,
    /// Honour `HTTPS_PROXY` and friends.
    pub(crate) use_proxy: bool,
    /// Replaces the per-provider minimum interval between two requests to one host
    /// (catalogue lookups only). `None` keeps each provider's own figure.
    pub(crate) min_interval: Option<Duration>,
    /// Largest response body accepted, in bytes.
    pub(crate) max_body_bytes: usize,
}

/// The network client and the runtime it runs on.
#[cfg(feature = "net")]
pub(crate) struct Net {
    runtime: tokio::runtime::Runtime,
    client: ebc_catalog::client::Client,
}

/// Where requests go.
pub(crate) enum Transport {
    /// Answers come from a fixture directory; nothing leaves the machine.
    Fixtures(Fixtures),
    /// Real HTTP.
    #[cfg(feature = "net")]
    Net(Net),
}

impl Transport {
    /// Opens the fixture directory when one is given, the network otherwise.
    ///
    /// # Errors
    /// An unreadable fixture index; a client that cannot be built; or no fixture
    /// directory in a build without the `net` feature.
    pub(crate) fn open(fixtures: Option<&Path>, options: NetOptions) -> Result<Self, CliError> {
        if let Some(dir) = fixtures {
            return Ok(Self::Fixtures(Fixtures::open(dir)?));
        }
        Self::open_net(options)
    }

    #[cfg(feature = "net")]
    fn open_net(options: NetOptions) -> Result<Self, CliError> {
        use ebc_catalog::client::{Client, ClientConfig};

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| CliError::failure(format!("cannot start the network runtime: {e}")))?;
        let client = Client::new(ClientConfig {
            user_agent: options.user_agent,
            timeout: options.timeout,
            max_body_bytes: options.max_body_bytes,
            min_interval: options.min_interval,
            use_system_proxy: options.use_proxy,
        })
        .map_err(|e| CliError::failure(format!("cannot build the HTTP client: {e}")))?;
        Ok(Self::Net(Net { runtime, client }))
    }

    #[cfg(not(feature = "net"))]
    fn open_net(_options: NetOptions) -> Result<Self, CliError> {
        Err(CliError::failure(
            "this build has no network support (cargo feature `net` is off): \
             pass --offline-fixtures DIR",
        ))
    }

    /// `true` when nothing leaves the machine.
    pub(crate) fn is_offline(&self) -> bool {
        matches!(self, Self::Fixtures(_))
    }

    /// Executes one request. Pacing is the caller's business.
    ///
    /// # Errors
    /// A transport error message; it never contains the query string of the URL.
    pub(crate) fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, String> {
        match self {
            Self::Fixtures(fixtures) => fixtures.execute(request),
            #[cfg(feature = "net")]
            Self::Net(net) => net
                .runtime
                .block_on(net.client.execute(request, Duration::ZERO))
                .map_err(|e| e.to_string()),
        }
    }

    /// Runs a catalogue cascade: the network client waits each provider's minimum
    /// interval between two requests to one host; fixtures answer at once.
    pub(crate) fn lookup_all(
        &self,
        providers: &[Box<dyn Provider>],
        isbn: &Isbn13,
        options: &LookupOptions,
    ) -> LookupReport {
        match self {
            Self::Fixtures(fixtures) => {
                lookup_all(providers, isbn, |request| fixtures.execute(request), options)
            }
            #[cfg(feature = "net")]
            Self::Net(net) => net.runtime.block_on(net.client.lookup_all(providers, isbn, options)),
        }
    }
}
