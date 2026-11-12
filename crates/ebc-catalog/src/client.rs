//! An HTTP executor for servers and command-line tools: `reqwest` with `rustls`.
//!
//! Only built with the `client` cargo feature, and never for `wasm32` (a browser host
//! drives [`Session`] with `fetch` instead).
//!
//! What it adds to a bare HTTP client, following the research's "network etiquette":
//!
//! * one `User-Agent` on every request ([`DEFAULT_USER_AGENT`]
//!   unless configured);
//! * a minimum interval between two requests to the same host, taken from
//!   [`ProviderInfo::min_interval_ms`](crate::ProviderInfo::min_interval_ms);
//! * a timeout per request and a cap on the body size;
//! * a refusal to contact — or to be redirected to — the hosts of
//!   [`DENIED_HOSTS`](crate::DENIED_HOSTS);
//! * a [`Cache`] hook, which is told which provider a response belongs to so that it can
//!   honour [`ProviderInfo::permanent_storage`](crate::ProviderInfo::permanent_storage)
//!   (Google Books responses must not be written to disk).
//!
//! Retries and back-off are left to the caller: [`CatalogError::is_transient`] and
//! [`CatalogError::retry_after_s`] say when one is worth it.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, HttpRequest, HttpResponse, Method};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::driver::{LookupOptions, LookupReport, Outcome, ProviderOutcome, Session, is_complete};
use crate::error::{CatalogError, shorten};
use crate::merge::merge;
use crate::provider::{Provider, ProviderInfo};
use crate::util::{MAX_BODY, host, redact, without_fragment};
use crate::{DEFAULT_USER_AGENT, is_denied_host};

/// Why a request could not be executed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ClientError {
    /// The HTTP client could not be built (TLS set-up, invalid `User-Agent`).
    #[error("cannot build the HTTP client: {0}")]
    Build(String),
    /// The URL targets a host on the deny-list, or is not an `http(s)` URL.
    #[error("refused to contact {0}")]
    Denied(String),
    /// The request timed out.
    #[error("timed out after {0:?}")]
    Timeout(Duration),
    /// The body is larger than the configured limit; it was not read to the end.
    #[error("response larger than {0} bytes")]
    TooLarge(usize),
    /// Any other failure: DNS, connection, TLS, protocol.
    #[error("{0}")]
    Http(String),
}

/// A response store consulted before the network and fed after it.
///
/// The crate ships no implementation: what may be kept, where and for how long depends on
/// the source's terms (research implication 24: open sources 30 days, misses 24 hours,
/// Google Books in memory for the session only).
pub trait Cache: Send + Sync {
    /// A stored response for `request`, if one is still fresh.
    fn get(&self, provider: &ProviderInfo, request: &HttpRequest) -> Option<HttpResponse>;
    /// Offers a response for storage. Only called for responses with a status below 500
    /// other than 429.
    fn put(&self, provider: &ProviderInfo, request: &HttpRequest, response: &HttpResponse);
}

/// Settings of a [`Client`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientConfig {
    /// `User-Agent` header value.
    pub user_agent: String,
    /// Time allowed for one request, connection included.
    pub timeout: Duration,
    /// Largest response body accepted, in bytes.
    pub max_body_bytes: usize,
    /// Replaces every provider's minimum interval between two requests to one host.
    /// `None` uses [`ProviderInfo::min_interval_ms`]. Tests against a local fixture
    /// server set it to zero.
    pub min_interval: Option<Duration>,
    /// Honour the proxy settings of the environment (`HTTPS_PROXY`…). Switch off to reach
    /// a fixture server on the loopback interface.
    pub use_system_proxy: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            // Research implication 14 proposes 4 s per provider; SRU services are slower
            // than that often enough to allow a little more per request.
            timeout: Duration::from_secs(8),
            max_body_bytes: MAX_BODY,
            min_interval: None,
            use_system_proxy: true,
        }
    }
}

/// The `reqwest`-based executor. Cheap to clone; clones share the rate limiter.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    config: ClientConfig,
    /// Per host: the earliest instant at which the next request may leave.
    next_slot: Arc<Mutex<HashMap<String, Instant>>>,
    cache: Option<Arc<dyn Cache>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .field("cache", &self.cache.is_some())
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Builds a client.
    ///
    /// # Errors
    /// [`ClientError::Build`] when `reqwest` refuses the configuration.
    pub fn new(config: ClientConfig) -> Result<Self, ClientError> {
        // reqwest is built without a TLS crypto provider of its own; install ring once.
        // An error only means another part of the program installed one already.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let redirects = reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("too many redirects")
            } else if attempt.url().host_str().is_none_or(is_denied_host) {
                attempt.error("redirected to a host on the deny-list")
            } else {
                attempt.follow()
            }
        });
        let mut builder = reqwest::Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(config.timeout)
            .connect_timeout(config.timeout)
            .redirect(redirects);
        if !config.use_system_proxy {
            builder = builder.no_proxy();
        }
        let http = builder.build().map_err(|e| ClientError::Build(shorten(&e.to_string())))?;
        Ok(Self { http, config, next_slot: Arc::new(Mutex::new(HashMap::new())), cache: None })
    }

    /// Attaches a response cache.
    #[must_use]
    pub fn with_cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// The settings this client was built with.
    #[must_use]
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Waits until a request to `host` may leave, and books the slot after it.
    async fn wait_for_slot(&self, host: &str, interval: Duration) {
        let wait = {
            let mut slots = self.next_slot.lock().await;
            let now = Instant::now();
            let start = slots.get(host).copied().filter(|slot| *slot > now).unwrap_or(now);
            slots.insert(host.to_owned(), start + interval);
            start.saturating_duration_since(now)
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
    }

    /// Executes one request, after waiting for `min_interval` to have passed since the
    /// previous request to the same host. The URL fragment (provider context) is not sent.
    ///
    /// # Errors
    /// A [`ClientError`]; error messages never contain the request URL's query string.
    pub async fn execute(
        &self,
        request: &HttpRequest,
        min_interval: Duration,
    ) -> Result<HttpResponse, ClientError> {
        let url = without_fragment(&request.url);
        let Some(target) = host(url).filter(|h| !is_denied_host(h)) else {
            return Err(ClientError::Denied(redact(url)));
        };
        self.wait_for_slot(&target, self.config.min_interval.unwrap_or(min_interval)).await;

        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Head => reqwest::Method::HEAD,
            Method::Post => reqwest::Method::POST,
        };
        let mut builder = self.http.request(method, url);
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }
        let failure = |error: reqwest::Error| {
            if error.is_timeout() {
                ClientError::Timeout(self.config.timeout)
            } else {
                // The URL may carry an API key: keep it out of the message.
                ClientError::Http(shorten(&error.without_url().to_string()))
            }
        };
        let mut response = builder.send().await.map_err(failure)?;
        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (name.as_str().to_owned(), String::from_utf8_lossy(value.as_bytes()).into_owned())
            })
            .collect();
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(failure)? {
            if body.len() + chunk.len() > self.config.max_body_bytes {
                return Err(ClientError::TooLarge(self.config.max_body_bytes));
            }
            body.extend_from_slice(&chunk);
        }
        Ok(HttpResponse { status, headers, body, final_url })
    }

    /// Runs one provider for `isbn`, one request after the other.
    ///
    /// # Errors
    /// As [`Session::finish`]: the first error met when no record could be read.
    pub async fn lookup<P: Provider + ?Sized>(
        &self,
        provider: &P,
        isbn: &Isbn13,
        max_requests: usize,
    ) -> Result<Outcome, CatalogError> {
        let info = provider.info();
        let interval = Duration::from_millis(u64::from(info.min_interval_ms));
        let mut session = Session::new(provider, isbn, max_requests);
        while let Some(request) = session.next_request() {
            if let Some(cached) = self.cache.as_ref().and_then(|cache| cache.get(info, &request)) {
                session.feed(&request, &cached);
                continue;
            }
            match self.execute(&request, interval).await {
                Ok(response) => {
                    if let Some(cache) = &self.cache
                        && response.status < 500
                        && response.status != 429
                    {
                        cache.put(info, &request, &response);
                    }
                    session.feed(&request, &response);
                }
                Err(error) => session.fail(&request, &error.to_string()),
            }
        }
        session.finish()
    }

    /// Queries `providers` in order and merges what they return, with the same rules as
    /// [`lookup_all`](crate::lookup_all). Providers are queried one after the other.
    pub async fn lookup_all(
        &self,
        providers: &[Box<dyn Provider>],
        isbn: &Isbn13,
        options: &LookupOptions,
    ) -> LookupReport {
        let mut report = LookupReport::default();
        let mut records: Vec<BookMeta> = Vec::new();
        for (index, provider) in providers.iter().enumerate() {
            let id = provider.info().id;
            if options.stop_when_complete
                && index >= options.always_query
                && merge(&records).as_ref().is_some_and(is_complete)
            {
                report.skipped.push(id);
                continue;
            }
            let result = self.lookup(provider, isbn, options.max_requests_per_provider).await;
            if let Ok(Outcome { record: Some(record), .. }) = &result {
                records.push(record.clone());
            }
            report.providers.push(ProviderOutcome { provider: id, result });
        }
        report.merged = merge(&records);
        report
    }
}
