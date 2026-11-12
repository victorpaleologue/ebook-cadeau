//! Configuration of one storefront: which archetype it is, where its collaborators live,
//! and the injected sources of time and randomness.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::crypto;
use crate::error::StoreError;

/// Which of the two fictional stores an app instance is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreKind {
    /// "Septante": an independent French store selling DRM-free, watermarked eBooks with
    /// direct downloads.
    Septante,
    /// "Grande Enseigne": a large cultural retailer whose eBooks live in an e-reader
    /// ecosystem account; no file download in the browser.
    GrandeEnseigne,
}

impl StoreKind {
    /// Both stores, in port order.
    pub const ALL: [StoreKind; 2] = [StoreKind::Septante, StoreKind::GrandeEnseigne];

    /// URL- and env-friendly identifier: `septante` or `grande-enseigne`.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Septante => "septante",
            Self::GrandeEnseigne => "grande-enseigne",
        }
    }

    /// The fictional trading name shown in the UI.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Septante => "Septante",
            Self::GrandeEnseigne => "Grande Enseigne",
        }
    }

    /// Prefix of order references (`SEP-2026-000001`, `GE-2026-000001`).
    #[must_use]
    pub const fn order_prefix(self) -> &'static str {
        match self {
            Self::Septante => "SEP",
            Self::GrandeEnseigne => "GE",
        }
    }

    /// Name of the session cookie. The two stores usually run on the same host and cookies
    /// are not isolated by port, so each store needs its own name.
    #[must_use]
    pub const fn cookie_name(self) -> &'static str {
        match self {
            Self::Septante => "septante_sid",
            Self::GrandeEnseigne => "ge_sid",
        }
    }

    /// Default TCP port of the demo binary.
    #[must_use]
    pub const fn default_port(self) -> u16 {
        match self {
            Self::Septante => 8101,
            Self::GrandeEnseigne => 8102,
        }
    }

    /// Default delivery method for gifts.
    #[must_use]
    pub const fn default_gift_delivery(self) -> GiftDelivery {
        match self {
            Self::Septante => GiftDelivery::HostedFile,
            Self::GrandeEnseigne => GiftDelivery::ResellerRedirect,
        }
    }

    /// Whether customers download EPUB files from the browser (Septante) or only see them
    /// appear on their e-reader through the device API (Grande Enseigne).
    #[must_use]
    pub const fn offers_downloads(self) -> bool {
        matches!(self, Self::Septante)
    }

    /// Default partner identifier on the eBook Cadeau side.
    #[must_use]
    pub const fn default_partner_id(self) -> &'static str {
        self.slug()
    }

    /// Default (demo) partner secret. Matches `demo/stores/partners.demo.toml`.
    #[must_use]
    pub const fn default_partner_secret(self) -> &'static str {
        match self {
            Self::Septante => "demo-secret-septante",
            Self::GrandeEnseigne => "demo-secret-grande-enseigne",
        }
    }
}

/// How a gifted eBook reaches the recipient (`docs/api.md` §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftDelivery {
    /// The store uploads a watermarked EPUB to eBook Cadeau, which serves it to the
    /// recipient; no store account is needed.
    HostedFile,
    /// eBook Cadeau redirects the recipient to the store's activation page; the book is
    /// added to the recipient's store account.
    ResellerRedirect,
}

impl GiftDelivery {
    /// Parses `hosted_file` / `reseller_redirect` (as used in the API and in env vars).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "hosted_file" => Some(Self::HostedFile),
            "reseller_redirect" => Some(Self::ResellerRedirect),
            _ => None,
        }
    }

    /// The API name: `hosted_file` or `reseller_redirect`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HostedFile => "hosted_file",
            Self::ResellerRedirect => "reseller_redirect",
        }
    }
}

/// Credentials and location of the eBook Cadeau partner API.
#[derive(Clone)]
pub struct PartnerConfig {
    /// Base URL of the eBook Cadeau server, e.g. `http://127.0.0.1:8080`.
    pub base_url: String,
    /// Value of the `X-EBC-Partner` header.
    pub partner_id: String,
    /// Shared secret used for request signatures, the redemption redirect and webhooks.
    pub secret: String,
}

impl fmt::Debug for PartnerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PartnerConfig")
            .field("base_url", &self.base_url)
            .field("partner_id", &self.partner_id)
            .field("secret", &"<redacted>")
            .finish()
    }
}

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

    /// A clock backed by any function (tests use an atomic they can advance).
    #[must_use]
    pub fn from_fn(f: impl Fn() -> i64 + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
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

/// Injected source of random bytes (session identifiers, salts, device tokens).
#[derive(Clone)]
pub struct Entropy(Arc<dyn Fn(&mut [u8]) -> Result<(), StoreError> + Send + Sync>);

impl Entropy {
    /// The operating system's CSPRNG.
    #[must_use]
    pub fn os() -> Self {
        Self(Arc::new(|buf: &mut [u8]| getrandom::fill(buf).map_err(|_| StoreError::Entropy)))
    }

    /// A deterministic, **non-secret** generator for tests: SHA-256 of the seed and a
    /// counter. Never use it outside tests.
    #[must_use]
    pub fn deterministic(seed: u64) -> Self {
        let counter = AtomicU64::new(0);
        Self(Arc::new(move |buf: &mut [u8]| {
            for chunk in buf.chunks_mut(32) {
                let n = counter.fetch_add(1, Ordering::Relaxed);
                let mut input = [0u8; 16];
                input[..8].copy_from_slice(&seed.to_be_bytes());
                input[8..].copy_from_slice(&n.to_be_bytes());
                let block = crypto::sha256(&input);
                chunk.copy_from_slice(&block[..chunk.len()]);
            }
            Ok(())
        }))
    }

    /// Fills `buf` with random bytes.
    ///
    /// # Errors
    /// [`StoreError::Entropy`] when the underlying generator fails.
    pub fn fill(&self, buf: &mut [u8]) -> Result<(), StoreError> {
        (self.0)(buf)
    }

    /// A fresh random token of `bytes` bytes, hex-encoded.
    ///
    /// # Errors
    /// [`StoreError::Entropy`] when the underlying generator fails.
    pub fn token(&self, bytes: usize) -> Result<String, StoreError> {
        let mut buf = vec![0u8; bytes];
        self.fill(&mut buf)?;
        Ok(crypto::hex(&buf))
    }
}

impl fmt::Debug for Entropy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Entropy(..)")
    }
}

/// Everything one storefront needs to run.
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Which store this is.
    pub kind: StoreKind,
    /// Base URL of this store as browsers (and the printer's callbacks) reach it, without a
    /// trailing slash, e.g. `http://127.0.0.1:8101`. Used for the `reseller_redirect`
    /// delivery URL and callback URLs.
    pub public_url: String,
    /// The eBook Cadeau partner API. `None` disables the gift option at checkout.
    pub partner: Option<PartnerConfig>,
    /// Base URL of the print provider (`EBC_PRINTER_URL`). `None` disables the
    /// "printed and posted" card choice.
    pub printer_url: Option<String>,
    /// How gifts are delivered.
    pub gift_delivery: GiftDelivery,
    /// Enables `GET /__test/state` (`DEMO_TEST_HOOKS=1`).
    pub test_hooks: bool,
    /// Wall clock.
    pub clock: Clock,
    /// Random bytes.
    pub entropy: Entropy,
    /// Timeout of each outgoing HTTP call.
    pub http_timeout: Duration,
    /// PBKDF2 iterations for account passwords (demo-grade; lower it in tests).
    pub password_iterations: u32,
}

impl StoreConfig {
    /// A configuration with the defaults of `kind`: system clock, OS entropy, default
    /// delivery, no partner API and no printer.
    #[must_use]
    pub fn new(kind: StoreKind) -> Self {
        Self {
            kind,
            public_url: format!("http://127.0.0.1:{}", kind.default_port()),
            partner: None,
            printer_url: None,
            gift_delivery: kind.default_gift_delivery(),
            test_hooks: false,
            clock: Clock::system(),
            entropy: Entropy::os(),
            http_timeout: Duration::from_secs(15),
            password_iterations: 20_000,
        }
    }

    /// Sets the public base URL (a trailing slash is removed).
    #[must_use]
    pub fn with_public_url(mut self, url: impl Into<String>) -> Self {
        self.public_url = trim_base(&url.into());
        self
    }

    /// Sets the eBook Cadeau partner API credentials.
    #[must_use]
    pub fn with_partner(
        mut self,
        base_url: impl Into<String>,
        partner_id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        self.partner = Some(PartnerConfig {
            base_url: trim_base(&base_url.into()),
            partner_id: partner_id.into(),
            secret: secret.into(),
        });
        self
    }

    /// Sets the print provider base URL.
    #[must_use]
    pub fn with_printer(mut self, base_url: impl Into<String>) -> Self {
        self.printer_url = Some(trim_base(&base_url.into()));
        self
    }

    /// Sets the gift delivery method.
    #[must_use]
    pub fn with_gift_delivery(mut self, delivery: GiftDelivery) -> Self {
        self.gift_delivery = delivery;
        self
    }

    /// Enables or disables the test hooks.
    #[must_use]
    pub fn with_test_hooks(mut self, enabled: bool) -> Self {
        self.test_hooks = enabled;
        self
    }

    /// Replaces the clock.
    #[must_use]
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// Replaces the entropy source.
    #[must_use]
    pub fn with_entropy(mut self, entropy: Entropy) -> Self {
        self.entropy = entropy;
        self
    }

    /// Builds the configuration of `kind` from environment variables, read through
    /// `lookup` (the binary passes `std::env::var`; tests pass a map).
    ///
    /// | Variable | Default |
    /// |---|---|
    /// | `DEMO_{STORE}_PUBLIC_URL` | `http://127.0.0.1:{port}` |
    /// | `EBC_URL` | `http://127.0.0.1:8080` (set it to an empty string to disable gifts) |
    /// | `DEMO_{STORE}_PARTNER_ID` | `septante` / `grande-enseigne` |
    /// | `DEMO_{STORE}_PARTNER_SECRET` | `demo-secret-…` |
    /// | `DEMO_{STORE}_DELIVERY` | `hosted_file` (Septante) / `reseller_redirect` (Grande Enseigne) |
    /// | `EBC_PRINTER_URL` | unset: the "post" card choice is disabled |
    /// | `DEMO_TEST_HOOKS` | unset; `1` enables `/__test/state` |
    ///
    /// `{STORE}` is `SEPTANTE` or `GRANDE_ENSEIGNE`.
    ///
    /// # Errors
    /// [`StoreError::Config`] when a variable holds an unusable value.
    pub fn from_env(
        kind: StoreKind,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, StoreError> {
        let prefix = format!("DEMO_{}", kind.slug().to_ascii_uppercase().replace('-', "_"));
        let get = |name: &str| lookup(name).map(|v| v.trim().to_owned());
        let mut config = Self::new(kind);
        if let Some(url) = get(&format!("{prefix}_PUBLIC_URL")).filter(|v| !v.is_empty()) {
            check_http_url(&format!("{prefix}_PUBLIC_URL"), &url)?;
            config.public_url = trim_base(&url);
        }
        let ebc_url = get("EBC_URL").unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        if !ebc_url.is_empty() {
            check_http_url("EBC_URL", &ebc_url)?;
            let partner_id = get(&format!("{prefix}_PARTNER_ID"))
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| kind.default_partner_id().to_owned());
            let secret = get(&format!("{prefix}_PARTNER_SECRET"))
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| kind.default_partner_secret().to_owned());
            config = config.with_partner(ebc_url, partner_id, secret);
        }
        if let Some(url) = get("EBC_PRINTER_URL").filter(|v| !v.is_empty()) {
            check_http_url("EBC_PRINTER_URL", &url)?;
            config = config.with_printer(url);
        }
        if let Some(name) = get(&format!("{prefix}_DELIVERY")).filter(|v| !v.is_empty()) {
            config.gift_delivery = GiftDelivery::from_name(&name).ok_or_else(|| {
                StoreError::Config(format!(
                    "{prefix}_DELIVERY must be hosted_file or reseller_redirect, got {name:?}"
                ))
            })?;
        }
        config.test_hooks = get("DEMO_TEST_HOOKS").is_some_and(|v| v == "1");
        Ok(config)
    }
}

fn trim_base(url: &str) -> String {
    url.trim().trim_end_matches('/').to_owned()
}

fn check_http_url(name: &str, value: &str) -> Result<(), StoreError> {
    match reqwest::Url::parse(value) {
        Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => Ok(()),
        _ => Err(StoreError::Config(format!("{name} must be an http(s) URL, got {value:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let map: HashMap<String, String> =
            pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        move |name: &str| map.get(name).cloned()
    }

    #[test]
    fn defaults_per_store() {
        let s = StoreConfig::from_env(StoreKind::Septante, env(&[])).unwrap();
        assert_eq!(s.public_url, "http://127.0.0.1:8101");
        assert_eq!(s.gift_delivery, GiftDelivery::HostedFile);
        let partner = s.partner.unwrap();
        assert_eq!(partner.base_url, "http://127.0.0.1:8080");
        assert_eq!(partner.partner_id, "septante");
        assert!(s.printer_url.is_none());
        assert!(!s.test_hooks);

        let g = StoreConfig::from_env(StoreKind::GrandeEnseigne, env(&[])).unwrap();
        assert_eq!(g.public_url, "http://127.0.0.1:8102");
        assert_eq!(g.gift_delivery, GiftDelivery::ResellerRedirect);
        assert_eq!(g.partner.unwrap().partner_id, "grande-enseigne");
    }

    #[test]
    fn overrides_and_validation() {
        let s = StoreConfig::from_env(
            StoreKind::Septante,
            env(&[
                ("DEMO_SEPTANTE_PUBLIC_URL", "https://septante.example/"),
                ("EBC_URL", "http://ebc.test:9000/"),
                ("DEMO_SEPTANTE_PARTNER_ID", "p1"),
                ("DEMO_SEPTANTE_PARTNER_SECRET", "s1"),
                ("DEMO_SEPTANTE_DELIVERY", "reseller-redirect"),
                ("EBC_PRINTER_URL", "http://printer.test:8103"),
                ("DEMO_TEST_HOOKS", "1"),
            ]),
        )
        .unwrap();
        assert_eq!(s.public_url, "https://septante.example");
        let partner = s.partner.unwrap();
        assert_eq!(
            (partner.base_url.as_str(), partner.partner_id.as_str(), partner.secret.as_str()),
            ("http://ebc.test:9000", "p1", "s1")
        );
        assert_eq!(s.gift_delivery, GiftDelivery::ResellerRedirect);
        assert_eq!(s.printer_url.as_deref(), Some("http://printer.test:8103"));
        assert!(s.test_hooks);

        // An empty EBC_URL disables the gift option instead of failing.
        let off = StoreConfig::from_env(StoreKind::Septante, env(&[("EBC_URL", "")])).unwrap();
        assert!(off.partner.is_none());

        for (name, value) in [
            ("EBC_URL", "ftp://x"),
            ("EBC_URL", "not a url"),
            ("EBC_PRINTER_URL", "javascript:alert(1)"),
            ("DEMO_SEPTANTE_DELIVERY", "carrier_pigeon"),
            ("DEMO_SEPTANTE_PUBLIC_URL", "//no-scheme"),
        ] {
            let err = StoreConfig::from_env(StoreKind::Septante, env(&[(name, value)]));
            assert!(matches!(err, Err(StoreError::Config(_))), "{name}={value}");
        }
    }

    #[test]
    fn partner_debug_redacts_the_secret() {
        let c = StoreConfig::new(StoreKind::Septante).with_partner("http://e", "p", "TOPSECRET");
        assert!(!format!("{c:?}").contains("TOPSECRET"));
    }

    #[test]
    fn deterministic_entropy_is_reproducible_and_varies() {
        let a = Entropy::deterministic(7);
        let b = Entropy::deterministic(7);
        let (t1, t2) = (a.token(16).unwrap(), a.token(16).unwrap());
        assert_ne!(t1, t2);
        assert_eq!(t1, b.token(16).unwrap());
        assert_eq!(t1.len(), 32);
        assert_ne!(Entropy::deterministic(8).token(16).unwrap(), t1);
        // Buffers longer than one hash block are filled entirely.
        let mut big = [0u8; 100];
        a.fill(&mut big).unwrap();
        assert!(big[64..].iter().any(|b| *b != 0));
    }

    #[test]
    fn os_entropy_works() {
        let e = Entropy::os();
        assert_ne!(e.token(16).unwrap(), e.token(16).unwrap());
    }
}
