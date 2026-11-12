//! Server configuration: the environment variables of `docs/api.md` §6 and the partners file.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::ServerError;

/// One eBook reseller integrating the "eBook cadeau" option (`docs/api.md` §4).
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct Partner {
    /// Value of the `X-EBC-Partner` header.
    pub id: String,
    /// Trading name shown to recipients ("add to my {name} library").
    #[serde(default)]
    pub name: String,
    /// Shared secret: request signatures, redirect signatures, webhook signatures.
    pub secret: String,
    /// Where signed events are POSTed; `None` disables webhooks for this partner.
    #[serde(default)]
    pub webhook_url: Option<String>,
    /// Hosts (`host` or `host:port`) the partner's delivery URLs may point to.
    #[serde(default)]
    pub redirect_hosts: Vec<String>,
}

impl std::fmt::Debug for Partner {
    /// Redacts the secret.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Partner")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("secret", &"***")
            .field("webhook_url", &self.webhook_url)
            .field("redirect_hosts", &self.redirect_hosts)
            .finish()
    }
}

impl Partner {
    /// The name to show to people: `name`, or the id when no name is configured.
    #[must_use]
    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() { &self.id } else { &self.name }
    }

    /// Whether `url` is an absolute `http(s)` URL whose host is one of
    /// [`redirect_hosts`](Self::redirect_hosts). An entry with a port only matches that
    /// port; an entry without one matches any port. URLs with credentials are refused.
    #[must_use]
    pub fn allows_url(&self, url: &str) -> bool {
        let Ok(parsed) = reqwest::Url::parse(url) else { return false };
        if !matches!(parsed.scheme(), "http" | "https")
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            return false;
        }
        let Some(host) = parsed.host_str().map(str::to_ascii_lowercase) else { return false };
        let port = parsed.port_or_known_default();
        self.redirect_hosts.iter().any(|entry| {
            let entry = entry.trim().to_ascii_lowercase();
            match entry.rsplit_once(':') {
                // `[::1]:8443` or `host:8443`; a bare IPv6 literal has no numeric tail.
                Some((entry_host, entry_port)) if entry_port.parse::<u16>().is_ok() => {
                    entry_host == host && entry_port.parse::<u16>().ok() == port
                }
                _ => entry == host,
            }
        })
    }
}

/// The partners file (`EBC_PARTNERS`): a TOML document with one `[[partners]]` table per
/// partner.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PartnersFile {
    /// The partners.
    #[serde(default, alias = "partner")]
    pub partners: Vec<Partner>,
}

impl PartnersFile {
    /// Parses and validates the TOML text.
    ///
    /// # Errors
    /// [`ServerError::Config`] for invalid TOML, duplicate or unusable ids, short secrets
    /// or malformed webhook URLs.
    pub fn parse(text: &str) -> Result<Self, ServerError> {
        let file: Self =
            toml::from_str(text).map_err(|e| ServerError::Config(format!("partners file: {e}")))?;
        let mut seen = std::collections::BTreeSet::new();
        for partner in &file.partners {
            let id_ok = !partner.id.is_empty()
                && partner.id.len() <= 64
                && partner.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
            if !id_ok {
                return Err(ServerError::Config(format!(
                    "partner id {:?} must be 1-64 characters from A-Z a-z 0-9 - _",
                    partner.id
                )));
            }
            if !seen.insert(partner.id.clone()) {
                return Err(ServerError::Config(format!("duplicate partner id {:?}", partner.id)));
            }
            if partner.secret.len() < 16 {
                return Err(ServerError::Config(format!(
                    "partner {:?}: the secret must be at least 16 characters",
                    partner.id
                )));
            }
            if let Some(url) = &partner.webhook_url {
                let ok = reqwest::Url::parse(url)
                    .is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.has_host());
                if !ok {
                    return Err(ServerError::Config(format!(
                        "partner {:?}: webhook_url must be an absolute http(s) URL",
                        partner.id
                    )));
                }
            }
        }
        Ok(file)
    }
}

/// Retry policy of webhook deliveries (`docs/api.md` §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebhookPolicy {
    /// Deliveries attempted before an event is given up (at least 1).
    pub max_attempts: u32,
    /// Delay before the second attempt; doubled after each failure.
    pub base_delay_ms: u64,
    /// Upper bound of the delay between two attempts.
    pub max_delay_ms: u64,
    /// Time allowed for one delivery.
    pub timeout_ms: u64,
}

impl Default for WebhookPolicy {
    /// 8 attempts over about 10 minutes: 5 s, 10 s, 20 s … capped at 5 minutes.
    fn default() -> Self {
        Self { max_attempts: 8, base_delay_ms: 5_000, max_delay_ms: 300_000, timeout_ms: 10_000 }
    }
}

impl WebhookPolicy {
    /// Delay to wait after the `failed_attempts`-th failure (1-based): exponential
    /// backoff, capped.
    #[must_use]
    pub fn delay_after(&self, failed_attempts: u32) -> std::time::Duration {
        let shift = failed_attempts.saturating_sub(1).min(32);
        let delay = self.base_delay_ms.saturating_mul(1u64 << shift).min(self.max_delay_ms);
        std::time::Duration::from_millis(delay)
    }
}

/// Everything the server needs to know. Built from the environment with
/// [`Config::from_env`], or by hand for tests and demos ([`Config::for_tests`]).
#[derive(Clone)]
pub struct Config {
    /// `EBC_BIND`: listen address.
    pub bind: String,
    /// `EBC_PUBLIC_URL`: base of claim URLs, without trailing slash.
    pub public_url: String,
    /// `EBC_DATA_DIR`: database and uploaded files.
    pub data_dir: PathBuf,
    /// `EBC_CODE_SECRET`: HMAC key for code hashing.
    pub code_secret: Vec<u8>,
    /// Partners loaded from `EBC_PARTNERS`.
    pub partners: Vec<Partner>,
    /// `EBC_SMTP_URL`: `smtp://host:port` for send-to-device; `None` disables the feature.
    pub smtp_url: Option<String>,
    /// `EBC_SEND_FROM`: sender address recipients must approve (Send to Kindle).
    pub send_from: String,
    /// `EBC_SEND_DOMAINS`: allowed device e-mail domains, lower case.
    pub send_domains: Vec<String>,
    /// `EBC_PRINTER_URL`: print provider base URL.
    pub printer_url: Option<String>,
    /// `EBC_RATE_UNKNOWN_PER_10MIN`: unknown-code attempts per client before `429`
    /// (0 disables the per-client limit).
    pub rate_unknown_per_10min: u32,
    /// `EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN`: unknown codes from all clients before every
    /// further wrong guess gets `429` (0 disables the global budget).
    pub rate_unknown_global_per_10min: u32,
    /// `EBC_RATE_CREATE_PER_10MIN` (addition to the contract): uploads and gift creations
    /// per client on the open creator API (0 disables the limit).
    pub rate_create_per_10min: u32,
    /// `EBC_WEB_DIR`: directory of the static web app served at `/app/`.
    pub web_dir: Option<PathBuf>,
    /// `EBC_TRUST_PROXY`: honour `X-Forwarded-For` for client identification.
    pub trust_proxy: bool,

    /// Largest accepted eBook upload, in bytes (50 MB).
    pub max_upload_bytes: usize,
    /// Largest accepted cover image, in bytes (20 MB).
    pub max_cover_bytes: usize,
    /// Largest accepted card PDF, in bytes.
    pub max_card_pdf_bytes: usize,
    /// Lifetime of a download session, in seconds.
    pub session_ttl_secs: i64,
    /// Lifetime of a pair code, in seconds (15 minutes).
    pub pair_ttl_secs: i64,
    /// Download sessions allowed per gift before it is exhausted.
    pub max_download_sessions: u32,
    /// Accepted clock difference on signed partner requests, in seconds.
    pub partner_clock_skew_secs: i64,
    /// How long an `Idempotency-Key` is remembered, in seconds.
    pub idempotency_ttl_secs: i64,
    /// How long an upload may stay unattached to a gift before it is deleted, in seconds.
    pub upload_orphan_ttl_secs: i64,
    /// Webhook retry policy.
    pub webhook: WebhookPolicy,
    /// Time allowed for a call to the print provider or a reseller licence endpoint.
    pub outbound_timeout_ms: u64,
}

impl std::fmt::Debug for Config {
    /// Redacts the code secret (partners redact theirs).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("bind", &self.bind)
            .field("public_url", &self.public_url)
            .field("data_dir", &self.data_dir)
            .field("code_secret", &"***")
            .field("partners", &self.partners)
            .field("smtp_url", &self.smtp_url.as_ref().map(|_| "***"))
            .field("send_from", &self.send_from)
            .field("send_domains", &self.send_domains)
            .field("printer_url", &self.printer_url)
            .field("rate_unknown_per_10min", &self.rate_unknown_per_10min)
            .field("rate_unknown_global_per_10min", &self.rate_unknown_global_per_10min)
            .field("web_dir", &self.web_dir)
            .field("trust_proxy", &self.trust_proxy)
            .finish_non_exhaustive()
    }
}

/// Name of the file under the data directory where a development code secret is kept.
const DEV_SECRET_FILE: &str = "code-secret";

impl Config {
    /// The documented defaults, with a fixed development secret. **Not for production**:
    /// the code secret is public.
    #[must_use]
    pub fn for_tests() -> Self {
        Self {
            bind: "127.0.0.1:8080".to_owned(),
            public_url: "http://127.0.0.1:8080".to_owned(),
            data_dir: PathBuf::from("./var"),
            code_secret: b"ebc-test-code-secret-not-for-production".to_vec(),
            partners: Vec::new(),
            smtp_url: None,
            send_from: "livraison@localhost".to_owned(),
            send_domains: vec!["kindle.com".to_owned(), "pbsync.com".to_owned()],
            printer_url: None,
            rate_unknown_per_10min: 10,
            rate_unknown_global_per_10min: 500,
            rate_create_per_10min: 120,
            web_dir: None,
            trust_proxy: false,
            max_upload_bytes: 50 * 1024 * 1024,
            max_cover_bytes: 20 * 1024 * 1024,
            max_card_pdf_bytes: 64 * 1024 * 1024,
            session_ttl_secs: 15 * 60,
            pair_ttl_secs: 15 * 60,
            max_download_sessions: 10,
            partner_clock_skew_secs: 300,
            idempotency_ttl_secs: 24 * 3600,
            upload_orphan_ttl_secs: 24 * 3600,
            webhook: WebhookPolicy::default(),
            outbound_timeout_ms: 15_000,
        }
    }

    /// Reads the configuration from the process environment (`docs/api.md` §6).
    ///
    /// # Errors
    /// [`ServerError::Config`] for unusable values, a missing `EBC_CODE_SECRET` outside
    /// development, or an unreadable partners file.
    pub fn from_env() -> Result<Self, ServerError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Like [`from_env`](Self::from_env), reading variables through `lookup` (testable).
    ///
    /// # Errors
    /// See [`from_env`](Self::from_env).
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ServerError> {
        let get = |name: &str| lookup(name).map(|v| v.trim().to_owned()).filter(|v| !v.is_empty());
        let mut config = Self::for_tests();

        if let Some(bind) = get("EBC_BIND") {
            config.bind = bind;
        }
        if let Some(url) = get("EBC_PUBLIC_URL") {
            config.public_url = url;
        }
        config.public_url = normalise_public_url(&config.public_url)?;
        if let Some(dir) = get("EBC_DATA_DIR") {
            config.data_dir = PathBuf::from(dir);
        }
        config.code_secret = match get("EBC_CODE_SECRET") {
            Some(secret) if secret.len() >= 16 => secret.into_bytes(),
            Some(_) => {
                return Err(ServerError::Config(
                    "EBC_CODE_SECRET must be at least 16 characters".to_owned(),
                ));
            }
            None if is_development_url(&config.public_url) => dev_secret(&config.data_dir)?,
            None => {
                return Err(ServerError::Config(
                    "EBC_CODE_SECRET is required when EBC_PUBLIC_URL is not a loopback address"
                        .to_owned(),
                ));
            }
        };
        if let Some(path) = get("EBC_PARTNERS") {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| ServerError::Config(format!("EBC_PARTNERS ({path}): {e}")))?;
            config.partners = PartnersFile::parse(&text)?.partners;
        }
        config.smtp_url = get("EBC_SMTP_URL");
        if let Some(url) = &config.smtp_url {
            let ok = reqwest::Url::parse(url)
                .is_ok_and(|u| matches!(u.scheme(), "smtp" | "smtps") && u.has_host());
            if !ok {
                return Err(ServerError::Config(
                    "EBC_SMTP_URL must look like smtp://host:port".to_owned(),
                ));
            }
        }
        if let Some(from) = get("EBC_SEND_FROM") {
            if from.parse::<lettre::message::Mailbox>().is_err() {
                return Err(ServerError::Config(
                    "EBC_SEND_FROM must be an e-mail address".to_owned(),
                ));
            }
            config.send_from = from;
        }
        if let Some(domains) = get("EBC_SEND_DOMAINS") {
            config.send_domains = parse_domains(&domains)?;
        }
        config.printer_url = match get("EBC_PRINTER_URL") {
            Some(url) => {
                let ok = reqwest::Url::parse(&url)
                    .is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.has_host());
                if !ok {
                    return Err(ServerError::Config(
                        "EBC_PRINTER_URL must be an absolute http(s) URL".to_owned(),
                    ));
                }
                Some(url.trim_end_matches('/').to_owned())
            }
            None => None,
        };
        let number = |name: &str, default: u32| -> Result<u32, ServerError> {
            match get(name) {
                Some(text) => text.parse::<u32>().map_err(|_| {
                    ServerError::Config(format!("{name} must be a non-negative integer"))
                }),
                None => Ok(default),
            }
        };
        config.rate_unknown_per_10min =
            number("EBC_RATE_UNKNOWN_PER_10MIN", config.rate_unknown_per_10min)?;
        config.rate_unknown_global_per_10min =
            number("EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN", config.rate_unknown_global_per_10min)?;
        config.rate_create_per_10min =
            number("EBC_RATE_CREATE_PER_10MIN", config.rate_create_per_10min)?;
        config.web_dir = get("EBC_WEB_DIR").map(PathBuf::from);
        config.trust_proxy = match get("EBC_TRUST_PROXY").as_deref() {
            None | Some("0" | "false" | "no" | "off") => false,
            Some("1" | "true" | "yes" | "on") => true,
            Some(other) => {
                return Err(ServerError::Config(format!(
                    "EBC_TRUST_PROXY must be 0 or 1, got {other:?}"
                )));
            }
        };
        Ok(config)
    }

    /// What makes the public URL a poor fit for a printed card, as human-readable
    /// warnings (none for loopback addresses). The card prints the host in lower case and
    /// its QR code encodes the URL in upper case, so the host should be short and made of
    /// letters and dots only (`docs/research/05-ereader-distribution-ux.md`, implication 23).
    #[must_use]
    pub fn public_url_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if is_development_url(&self.public_url) {
            return warnings;
        }
        let host = self.public_host();
        if !self.public_url.starts_with("https://") {
            warnings.push("EBC_PUBLIC_URL is not https: codes travel in clear".to_owned());
        }
        if !host.bytes().all(|b| b.is_ascii_lowercase() || b == b'.') {
            warnings.push(format!(
                "EBC_PUBLIC_URL host {host:?} is not made of lower-case letters and dots only: \
                 it is typed by hand on e-readers and upper-cased in the QR code"
            ));
        }
        if host.len() > 12 {
            warnings.push(format!("EBC_PUBLIC_URL host {host:?} is long to type on an e-reader"));
        }
        warnings
    }

    /// The partner with this id, if configured.
    #[must_use]
    pub fn partner(&self, id: &str) -> Option<&Partner> {
        self.partners.iter().find(|p| p.id == id)
    }

    /// The host (and port, if any) of the public URL, as people type it: `ebk.example`.
    #[must_use]
    pub fn public_host(&self) -> &str {
        let rest = self.public_url.split_once("://").map_or(self.public_url.as_str(), |(_, r)| r);
        rest.split('/').next().unwrap_or(rest)
    }
}

/// Validates `EBC_PUBLIC_URL` and strips trailing slashes.
fn normalise_public_url(url: &str) -> Result<String, ServerError> {
    let trimmed = url.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(trimmed)
        .map_err(|e| ServerError::Config(format!("EBC_PUBLIC_URL: {e}")))?;
    let plain = matches!(parsed.scheme(), "http" | "https")
        && parsed.has_host()
        && parsed.path() == "/"
        && parsed.query().is_none()
        && parsed.fragment().is_none()
        && parsed.username().is_empty();
    if !plain {
        return Err(ServerError::Config(
            "EBC_PUBLIC_URL must be http(s)://host[:port] without path, query or credentials"
                .to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

/// Whether the public URL points at this machine (development).
fn is_development_url(public_url: &str) -> bool {
    reqwest::Url::parse(public_url).ok().and_then(|u| u.host_str().map(str::to_owned)).is_some_and(
        |host| {
            host == "localhost"
                || host.ends_with(".localhost")
                || host
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        },
    )
}

/// Loads the development secret from the data directory, creating it on first use.
fn dev_secret(data_dir: &Path) -> Result<Vec<u8>, ServerError> {
    let path = data_dir.join(DEV_SECRET_FILE);
    if let Ok(existing) = std::fs::read(&path)
        && existing.len() >= 32
    {
        return Ok(existing);
    }
    use crate::clock::Entropy;
    let mut bytes = [0u8; 32];
    crate::clock::OsEntropy.fill(&mut bytes).map_err(|e| ServerError::Config(e.to_string()))?;
    let secret = crate::codes::hex(&bytes).into_bytes();
    std::fs::create_dir_all(data_dir)
        .and_then(|()| std::fs::write(&path, &secret))
        .map_err(|e| ServerError::Storage(format!("{}: {e}", path.display())))?;
    tracing::warn!(
        path = %path.display(),
        "EBC_CODE_SECRET is not set: generated a development secret"
    );
    Ok(secret)
}

/// Parses a comma-separated list of e-mail domains.
fn parse_domains(list: &str) -> Result<Vec<String>, ServerError> {
    let mut domains = Vec::new();
    for raw in list.split(',') {
        let domain = raw.trim().trim_start_matches('@').to_ascii_lowercase();
        if domain.is_empty() {
            continue;
        }
        let ok = domain.len() <= 253
            && domain.contains('.')
            && domain.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    && !label.starts_with('-')
                    && !label.ends_with('-')
            });
        if !ok {
            return Err(ServerError::Config(format!(
                "EBC_SEND_DOMAINS: {raw:?} is not a domain name"
            )));
        }
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    Ok(domains)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> =
            pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn the_development_partners_file_of_the_readme_is_valid() {
        let file = PartnersFile::parse(include_str!("../partners.dev.toml")).unwrap();
        let ids: Vec<&str> = file.partners.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["septante", "grande-enseigne"]);
        assert!(file.partners[0].allows_url("http://127.0.0.1:8101/cadeau/activer"));
        assert!(!file.partners[0].allows_url("http://127.0.0.1:8102/cadeau/activer"));
        assert!(file.partners.iter().all(|p| p.name.contains("démo")), "demo names say so");
    }

    #[test]
    fn public_url_warnings_flag_hosts_that_are_hard_to_type() {
        let with = |url: &str| {
            let mut config = Config::for_tests();
            config.public_url = url.to_owned();
            config.public_url_warnings()
        };
        assert!(with("http://127.0.0.1:8080").is_empty(), "loopback is development");
        assert!(with("https://ebk.example").is_empty());
        assert_eq!(with("http://ebk.example").len(), 1, "plain http");
        assert_eq!(with("https://e-book4u.example").len(), 2, "digit and hyphen, and long");
        assert_eq!(with("https://cadeau.livres.example").len(), 1, "long");
    }

    #[test]
    fn defaults_match_the_contract() {
        let dir = tempfile::tempdir().unwrap();
        let config =
            Config::from_lookup(env(&[("EBC_DATA_DIR", dir.path().to_str().unwrap())])).unwrap();
        assert_eq!(config.bind, "127.0.0.1:8080");
        assert_eq!(config.public_url, "http://127.0.0.1:8080");
        assert_eq!(config.send_from, "livraison@localhost");
        assert_eq!(config.send_domains, ["kindle.com", "pbsync.com"]);
        assert_eq!(config.rate_unknown_per_10min, 10);
        assert_eq!(config.rate_unknown_global_per_10min, 500);
        assert!(!config.trust_proxy);
        assert!(config.smtp_url.is_none() && config.printer_url.is_none());
        assert!(config.web_dir.is_none() && config.partners.is_empty());
        assert_eq!(config.max_upload_bytes, 50 * 1024 * 1024);
        assert_eq!(config.pair_ttl_secs, 900);
        assert_eq!(config.max_download_sessions, 10);
        assert_eq!(config.partner_clock_skew_secs, 300);
        // A development secret was generated and is stable across restarts.
        assert!(config.code_secret.len() >= 32);
        let again =
            Config::from_lookup(env(&[("EBC_DATA_DIR", dir.path().to_str().unwrap())])).unwrap();
        assert_eq!(again.code_secret, config.code_secret);
    }

    #[test]
    fn secret_is_required_outside_development() {
        let error = Config::from_lookup(env(&[("EBC_PUBLIC_URL", "https://ebk.example")]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("EBC_CODE_SECRET"), "{error}");
        let config = Config::from_lookup(env(&[
            ("EBC_PUBLIC_URL", "https://ebk.example/"),
            ("EBC_CODE_SECRET", "0123456789abcdef0123"),
        ]))
        .unwrap();
        assert_eq!(config.public_url, "https://ebk.example");
        assert_eq!(config.public_host(), "ebk.example");
        assert_eq!(config.code_secret, b"0123456789abcdef0123");
        assert!(
            Config::from_lookup(env(&[
                ("EBC_PUBLIC_URL", "https://ebk.example"),
                ("EBC_CODE_SECRET", "short")
            ]))
            .is_err()
        );
    }

    #[test]
    fn unusable_values_are_refused() {
        let secret = ("EBC_CODE_SECRET", "0123456789abcdef0123");
        for (name, value) in [
            ("EBC_PUBLIC_URL", "ebk.example"),
            ("EBC_PUBLIC_URL", "https://ebk.example/path"),
            ("EBC_PUBLIC_URL", "https://user:pw@ebk.example"),
            ("EBC_PUBLIC_URL", "ftp://ebk.example"),
            ("EBC_PUBLIC_URL", "https://ebk.example?x=1"),
            ("EBC_SMTP_URL", "localhost:25"),
            ("EBC_SMTP_URL", "http://localhost"),
            ("EBC_PRINTER_URL", "printer.example"),
            ("EBC_RATE_UNKNOWN_PER_10MIN", "-1"),
            ("EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN", "many"),
            ("EBC_TRUST_PROXY", "maybe"),
            ("EBC_SEND_DOMAINS", "kindle.com, not a domain"),
            ("EBC_PARTNERS", "/nonexistent/partners.toml"),
        ] {
            let result = Config::from_lookup(env(&[secret, (name, value)]));
            assert!(result.is_err(), "{name}={value} should be refused");
        }
    }

    #[test]
    fn values_are_parsed() {
        let config = Config::from_lookup(env(&[
            ("EBC_CODE_SECRET", "0123456789abcdef0123"),
            ("EBC_BIND", "0.0.0.0:9000"),
            ("EBC_SMTP_URL", "smtp://127.0.0.1:2525"),
            ("EBC_SEND_FROM", "cadeau@ebk.example"),
            ("EBC_SEND_DOMAINS", " Kindle.com , @pbsync.com,kindle.com ,"),
            ("EBC_PRINTER_URL", "http://127.0.0.1:8091/"),
            ("EBC_RATE_UNKNOWN_PER_10MIN", "3"),
            ("EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN", "40"),
            ("EBC_WEB_DIR", "/srv/web"),
            ("EBC_TRUST_PROXY", "1"),
        ]))
        .unwrap();
        assert_eq!(config.bind, "0.0.0.0:9000");
        assert_eq!(config.smtp_url.as_deref(), Some("smtp://127.0.0.1:2525"));
        assert_eq!(config.send_from, "cadeau@ebk.example");
        assert_eq!(config.send_domains, ["kindle.com", "pbsync.com"]);
        assert_eq!(config.printer_url.as_deref(), Some("http://127.0.0.1:8091"));
        assert_eq!(config.rate_unknown_per_10min, 3);
        assert_eq!(config.rate_unknown_global_per_10min, 40);
        assert_eq!(config.web_dir.as_deref(), Some(Path::new("/srv/web")));
        assert!(config.trust_proxy);
        assert_eq!(config.public_host(), "127.0.0.1:8080");
    }

    const PARTNERS: &str = r#"
[[partners]]
id = "librairie-demo"
name = "Librairie Démo"
secret = "0123456789abcdef-demo"
webhook_url = "http://127.0.0.1:8090/webhooks/ebc"
redirect_hosts = ["127.0.0.1:8090", "store.example"]

[[partners]]
id = "autre"
secret = "another-secret-of-16+"
"#;

    #[test]
    fn partners_file_is_parsed_and_validated() {
        let file = PartnersFile::parse(PARTNERS).unwrap();
        assert_eq!(file.partners.len(), 2);
        let demo = &file.partners[0];
        assert_eq!(demo.display_name(), "Librairie Démo");
        assert_eq!(demo.webhook_url.as_deref(), Some("http://127.0.0.1:8090/webhooks/ebc"));
        assert_eq!(file.partners[1].display_name(), "autre");
        assert!(file.partners[1].redirect_hosts.is_empty());
        // The secret never appears in debug output.
        assert!(!format!("{demo:?}").contains("0123456789abcdef-demo"));

        let through_env = {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("partners.toml");
            std::fs::write(&path, PARTNERS).unwrap();
            Config::from_lookup(env(&[
                ("EBC_CODE_SECRET", "0123456789abcdef0123"),
                ("EBC_PARTNERS", path.to_str().unwrap()),
            ]))
            .unwrap()
        };
        assert!(through_env.partner("librairie-demo").is_some());
        assert!(through_env.partner("nobody").is_none());
        assert!(!format!("{through_env:?}").contains("0123456789abcdef"));

        for bad in [
            "partners = 3",
            "[[partners]]\nid = \"a\"\n",
            "[[partners]]\nid = \"a\"\nsecret = \"short\"\n",
            "[[partners]]\nid = \"a b\"\nsecret = \"0123456789abcdef\"\n",
            "[[partners]]\nid = \"\"\nsecret = \"0123456789abcdef\"\n",
            "[[partners]]\nid = \"a\"\nsecret = \"0123456789abcdef\"\nwebhook_url = \"ftp://x\"\n",
            "[[partners]]\nid = \"a\"\nsecret = \"0123456789abcdef\"\n[[partners]]\nid = \"a\"\nsecret = \"0123456789abcdef\"\n",
            "not toml at all [",
        ] {
            assert!(PartnersFile::parse(bad).is_err(), "{bad:?}");
        }
        assert!(PartnersFile::parse("").unwrap().partners.is_empty());
    }

    #[test]
    fn redirect_hosts_gate_delivery_urls() {
        let partner = &PartnersFile::parse(PARTNERS).unwrap().partners[0];
        for allowed in [
            "http://127.0.0.1:8090/gift/redeem",
            "https://store.example/gift/redeem?x=1",
            "https://STORE.EXAMPLE:8443/x",
            "http://store.example/",
        ] {
            assert!(partner.allows_url(allowed), "{allowed}");
        }
        for refused in [
            "http://127.0.0.1:8091/gift/redeem",
            "http://127.0.0.1/gift/redeem",
            "https://evil.example/gift/redeem",
            "https://store.example.evil.example/",
            "https://evil.example/?h=store.example",
            "https://store.example@evil.example/",
            "https://user:pw@store.example/",
            "javascript:alert(1)",
            "ftp://store.example/",
            "//store.example/x",
            "store.example",
            "",
        ] {
            assert!(!partner.allows_url(refused), "{refused}");
        }
        let nobody = &PartnersFile::parse(PARTNERS).unwrap().partners[1];
        assert!(!nobody.allows_url("https://store.example/"));
    }

    #[test]
    fn webhook_backoff_doubles_and_is_capped() {
        let policy =
            WebhookPolicy { max_attempts: 5, base_delay_ms: 100, max_delay_ms: 450, timeout_ms: 1 };
        let delays: Vec<u128> = (1..=5).map(|n| policy.delay_after(n).as_millis()).collect();
        assert_eq!(delays, [100, 200, 400, 450, 450]);
        assert_eq!(policy.delay_after(0).as_millis(), 100);
        assert_eq!(policy.delay_after(u32::MAX).as_millis(), 450);
    }
}
