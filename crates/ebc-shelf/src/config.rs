//! Configuration: environment variables (`crates/ebc-shelf/README.md`), with the defaults
//! a loopback development run needs and the checks a public deployment needs.

use std::path::PathBuf;

/// Who may upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `own`: only the operator's own account exists and uploads — an author or publisher
    /// hosting their own books. Sign-up is closed.
    Own,
    /// `multi`: open sign-up for rights holders; the operator moderates.
    Multi,
}

impl Mode {
    /// `own` or `multi`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Own => "own",
            Self::Multi => "multi",
        }
    }
}

/// How long each kind of record is kept (`docs/research/09` §9.1; every period is a
/// proposal **to confirm with counsel**, see the README).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retention {
    /// Civil identity (name, address, birth data) after an account is closed (décret
    /// 2021-1362 as recalled: five years after the end of the contract).
    pub identity_days: i64,
    /// Connection data of content creation (IP address of an upload, deletion or gift
    /// creation; décret 2021-1362 as recalled: one year).
    pub connection_days: i64,
    /// Rights declarations after the file is deleted ("life of the hosting plus the
    /// limitation period": five years, Code civil art. 2224).
    pub declaration_days: i64,
    /// Notices, decisions, statements of reasons and appeals (research 09 item 18: six
    /// years).
    pub notice_days: i64,
    /// Pseudonymised client networks of downloads (security log, at most six months).
    pub security_log_days: i64,
    /// Delivery records without any client data (proof of supply, five years).
    pub delivery_days: i64,
    /// A book whose gifts were all cancelled keeps its file this many days (the rights
    /// holder may make new gifts meanwhile), then the file is deleted.
    pub idle_file_days: i64,
}

impl Default for Retention {
    fn default() -> Self {
        Self {
            identity_days: 1_826,
            connection_days: 365,
            declaration_days: 1_826,
            notice_days: 2_192,
            security_log_days: 180,
            delivery_days: 1_826,
            idle_file_days: 30,
        }
    }
}

/// The identities shown on the legal notice page (LCEN art. 6 III: publisher and host).
/// Unset values are shown as "to be completed".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegalInfo {
    /// Operator (natural person's name or company name).
    pub operator_name: Option<String>,
    /// Operator's postal address.
    pub operator_address: Option<String>,
    /// Company registration (RCS/SIREN), share capital, VAT number — free text.
    pub operator_registration: Option<String>,
    /// Director of publication.
    pub publication_director: Option<String>,
    /// The technical host's name.
    pub host_name: Option<String>,
    /// The technical host's address.
    pub host_address: Option<String>,
    /// The technical host's telephone number.
    pub host_phone: Option<String>,
}

/// The shelf's configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// `EBC_SHELF_BIND`: listen address.
    pub bind: String,
    /// `EBC_SHELF_PUBLIC_URL`: `http(s)://host[:port]`, no path. Download links and the
    /// §4.5 endpoint are on this host, which must be the shelf's `redirect_hosts` entry at
    /// the core.
    pub public_url: String,
    /// `EBC_SHELF_DATA_DIR`: `db.json` and `files/<sha256>`.
    pub data_dir: PathBuf,
    /// `EBC_SHELF_NAME`: the name shown on every page.
    pub name: String,
    /// `EBC_SHELF_MODE`: `own` (default) or `multi`.
    pub mode: Mode,
    /// `EBC_SHELF_OPERATOR_EMAIL`: the operator's account (moderation; in `own` mode the
    /// only account).
    pub operator_email: String,
    /// `EBC_SHELF_SECRET`: HMAC key for form tokens and pseudonyms (≥ 16 characters).
    pub secret: String,
    /// `EBC_CORE_URL`: the eBook Cadeau core.
    pub core_url: String,
    /// `EBC_CORE_PUBLIC_URL`: the core as browsers reach it (designer link); defaults to
    /// `core_url`.
    pub core_public_url: String,
    /// `EBC_SHELF_PARTNER_ID`: this shelf's `X-EBC-Partner` at the core.
    pub partner_id: String,
    /// `EBC_SHELF_PARTNER_SECRET`: the secret shared with the core (≥ 16 characters).
    pub partner_secret: String,
    /// `EBC_SMTP_URL`: sign-in links, acknowledgements, statements of reasons.
    pub smtp_url: Option<String>,
    /// `EBC_SHELF_MAIL_FROM`: sender of every mail.
    pub mail_from: String,
    /// `EBC_SHELF_CONTACT_EMAIL`: single point of contact (DSA arts 11–12).
    pub contact_email: Option<String>,
    /// `EBC_SHELF_MAX_FILE_MB`: largest file accepted.
    pub max_file_bytes: u64,
    /// `EBC_SHELF_QUOTA_ACCOUNT_MB`: storage per account.
    pub account_quota_bytes: u64,
    /// `EBC_SHELF_QUOTA_TOTAL_MB`: storage of the whole shelf.
    pub total_quota_bytes: u64,
    /// `EBC_SHELF_LOGIN_TTL_MIN`: lifetime of a sign-in link, in seconds.
    pub login_ttl_secs: i64,
    /// `EBC_SHELF_SESSION_DAYS`: lifetime of a session, in seconds.
    pub session_ttl_secs: i64,
    /// `EBC_SHELF_LINK_TTL_SECS`: lifetime of a download link handed to the core.
    pub link_ttl_secs: i64,
    /// `EBC_SHELF_SUSPEND_AFTER_UPHELD`: upheld notices before an account is suspended.
    pub suspend_after_upheld: u32,
    /// `EBC_SHELF_SUSPEND_AFTER_FALSE_DECLARATIONS`: false declarations before suspension.
    pub suspend_after_false_declarations: u32,
    /// `EBC_SHELF_APPEAL_DAYS`: how long a decision can be contested (DSA art. 20: six
    /// months).
    pub appeal_days: i64,
    /// `EBC_SHELF_RETAIN_*`: retention periods.
    pub retention: Retention,
    /// `EBC_SHELF_LEGAL_*`: identities of the legal notice.
    pub legal: LegalInfo,
    /// `EBC_SHELF_RATE_LOGIN_PER_HOUR`: sign-in requests per client network and per
    /// address.
    pub rate_login_per_hour: u32,
    /// `EBC_SHELF_RATE_NOTICES_PER_HOUR`: notices per client network.
    pub rate_notices_per_hour: u32,
    /// `EBC_SHELF_RATE_UPLOADS_PER_HOUR`: uploads per account.
    pub rate_uploads_per_hour: u32,
    /// `EBC_SHELF_RATE_FAIL_PER_10MIN`: refused link requests or unknown download links
    /// per client network before `429`.
    pub rate_fail_per_10min: u32,
    /// `EBC_SHELF_MAX_GIFTS_PER_REQUEST`: cards made at once.
    pub max_gifts_per_request: u32,
    /// `EBC_SHELF_HEADER_TIMEOUT_SECS`.
    pub header_read_timeout_secs: u64,
    /// `EBC_SHELF_REQUEST_TIMEOUT_SECS`.
    pub request_timeout_secs: u64,
    /// `EBC_SHELF_BODY_IDLE_TIMEOUT_SECS`.
    pub body_idle_timeout_secs: u64,
    /// `EBC_SHELF_MAX_CONNECTIONS`.
    pub max_connections: u32,
    /// `EBC_SHELF_MAX_CONCURRENT_UPLOADS`: uploads processed at once (each holds the file
    /// in memory while it is checked).
    pub max_concurrent_uploads: u32,
    /// `EBC_SHELF_TRUST_PROXY`: identify clients by the last `X-Forwarded-For` entry.
    pub trust_proxy: bool,
    /// Time-out of calls to the core and to the SMTP server, in milliseconds.
    pub outbound_timeout_ms: u64,
}

/// A configuration problem.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{name}: {problem}")]
pub struct ConfigError {
    /// The variable.
    pub name: &'static str,
    /// What is wrong with it.
    pub problem: String,
}

fn bad(name: &'static str, problem: impl Into<String>) -> ConfigError {
    ConfigError { name, problem: problem.into() }
}

impl Config {
    /// A configuration for tests: loopback URLs, `multi` mode, public test secrets.
    #[must_use]
    pub fn for_tests() -> Self {
        Self {
            bind: "127.0.0.1:0".to_owned(),
            public_url: "http://127.0.0.1:8104".to_owned(),
            data_dir: PathBuf::from("./var-shelf"),
            name: "Étagère (test)".to_owned(),
            mode: Mode::Multi,
            operator_email: "moderation@etagere.test".to_owned(),
            secret: "test-shelf-secret-0123456789".to_owned(),
            core_url: "http://127.0.0.1:8080".to_owned(),
            core_public_url: "http://127.0.0.1:8080".to_owned(),
            partner_id: "etagere".to_owned(),
            partner_secret: "dev-secret-etagere-0123456789".to_owned(),
            smtp_url: None,
            mail_from: "etagere@localhost".to_owned(),
            contact_email: Some("contact@etagere.test".to_owned()),
            max_file_bytes: 50 << 20,
            account_quota_bytes: 200 << 20,
            total_quota_bytes: 5 << 30,
            login_ttl_secs: 15 * 60,
            session_ttl_secs: 7 * 86_400,
            link_ttl_secs: 300,
            suspend_after_upheld: 2,
            suspend_after_false_declarations: 1,
            appeal_days: 182,
            retention: Retention::default(),
            legal: LegalInfo::default(),
            rate_login_per_hour: 5,
            rate_notices_per_hour: 10,
            rate_uploads_per_hour: 30,
            rate_fail_per_10min: 30,
            max_gifts_per_request: 50,
            header_read_timeout_secs: 30,
            request_timeout_secs: 300,
            body_idle_timeout_secs: 30,
            max_connections: 1_024,
            max_concurrent_uploads: 4,
            trust_proxy: false,
            outbound_timeout_ms: 15_000,
        }
    }

    /// Reads the environment (see the README for every variable).
    ///
    /// # Errors
    /// [`ConfigError`] for a missing required value or an unusable one.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Like [`Config::from_env`], reading variables through `get` (for tests).
    ///
    /// # Errors
    /// See [`Config::from_env`].
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let text = |name: &str| get(name).map(|v| v.trim().to_owned()).filter(|v| !v.is_empty());
        let number = |name: &'static str, default: u64| -> Result<u64, ConfigError> {
            match text(name) {
                Some(value) => {
                    value.parse().map_err(|_| bad(name, format!("not a number: {value:?}")))
                }
                None => Ok(default),
            }
        };
        let days = |name: &'static str, default: i64| -> Result<i64, ConfigError> {
            let value = number(name, u64::try_from(default).unwrap_or(0))?;
            let value = i64::try_from(value).map_err(|_| bad(name, "too large"))?;
            if value < 1 {
                return Err(bad(name, "must be at least 1"));
            }
            Ok(value)
        };
        let small = |name: &'static str, default: u32| -> Result<u32, ConfigError> {
            u32::try_from(number(name, u64::from(default))?).map_err(|_| bad(name, "too large"))
        };
        let defaults = Self::for_tests();

        let bind = text("EBC_SHELF_BIND").unwrap_or_else(|| "127.0.0.1:8104".to_owned());
        let public_url =
            text("EBC_SHELF_PUBLIC_URL").unwrap_or_else(|| "http://127.0.0.1:8104".to_owned());
        check_base_url("EBC_SHELF_PUBLIC_URL", &public_url)?;
        let public_url = public_url.trim_end_matches('/').to_owned();
        let loopback = is_loopback(&public_url);

        let mode = match text("EBC_SHELF_MODE").as_deref() {
            None | Some("own") => Mode::Own,
            Some("multi") => Mode::Multi,
            Some(other) => {
                return Err(bad("EBC_SHELF_MODE", format!("own or multi, not {other:?}")));
            }
        };
        let operator_email = text("EBC_SHELF_OPERATOR_EMAIL")
            .ok_or_else(|| bad("EBC_SHELF_OPERATOR_EMAIL", "required (the operator's account)"))?;
        let operator_email = normalise_email(&operator_email)
            .ok_or_else(|| bad("EBC_SHELF_OPERATOR_EMAIL", "not an e-mail address"))?;

        let secret = match text("EBC_SHELF_SECRET") {
            Some(secret) if secret.chars().count() >= 16 => secret,
            Some(_) => return Err(bad("EBC_SHELF_SECRET", "at least 16 characters")),
            // Generated by the binary in the data directory on a loopback URL.
            None if loopback => String::new(),
            None => return Err(bad("EBC_SHELF_SECRET", "required outside a loopback URL")),
        };
        let core_url = text("EBC_CORE_URL").unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        check_base_url("EBC_CORE_URL", &core_url)?;
        let core_url = core_url.trim_end_matches('/').to_owned();
        let core_public_url = text("EBC_CORE_PUBLIC_URL").unwrap_or_else(|| core_url.clone());
        check_base_url("EBC_CORE_PUBLIC_URL", &core_public_url)?;
        let partner_id = text("EBC_SHELF_PARTNER_ID").unwrap_or_else(|| "etagere".to_owned());
        if !is_token(&partner_id) {
            return Err(bad("EBC_SHELF_PARTNER_ID", "letters, digits, '-' and '_' only"));
        }
        let partner_secret = text("EBC_SHELF_PARTNER_SECRET").ok_or_else(|| {
            bad("EBC_SHELF_PARTNER_SECRET", "required (shared with the core's EBC_PARTNERS)")
        })?;
        if partner_secret.chars().count() < 16 {
            return Err(bad("EBC_SHELF_PARTNER_SECRET", "at least 16 characters"));
        }
        let mb = |name: &'static str, default: u64| -> Result<u64, ConfigError> {
            let value = number(name, default)?;
            if value == 0 {
                return Err(bad(name, "must be at least 1"));
            }
            value.checked_mul(1 << 20).ok_or_else(|| bad(name, "too large"))
        };
        let contact_email = match text("EBC_SHELF_CONTACT_EMAIL") {
            Some(value) => Some(
                normalise_email(&value)
                    .ok_or_else(|| bad("EBC_SHELF_CONTACT_EMAIL", "not an e-mail address"))?,
            ),
            None => None,
        };
        let r = Retention::default();
        let retention = Retention {
            identity_days: days("EBC_SHELF_RETAIN_IDENTITY_DAYS", r.identity_days)?,
            connection_days: days("EBC_SHELF_RETAIN_CONNECTION_DAYS", r.connection_days)?,
            declaration_days: days("EBC_SHELF_RETAIN_DECLARATION_DAYS", r.declaration_days)?,
            notice_days: days("EBC_SHELF_RETAIN_NOTICE_DAYS", r.notice_days)?,
            security_log_days: days("EBC_SHELF_RETAIN_SECURITY_LOG_DAYS", r.security_log_days)?,
            delivery_days: days("EBC_SHELF_RETAIN_DELIVERY_DAYS", r.delivery_days)?,
            idle_file_days: days("EBC_SHELF_IDLE_FILE_DAYS", r.idle_file_days)?,
        };
        let legal = LegalInfo {
            operator_name: text("EBC_SHELF_LEGAL_OPERATOR_NAME"),
            operator_address: text("EBC_SHELF_LEGAL_OPERATOR_ADDRESS"),
            operator_registration: text("EBC_SHELF_LEGAL_OPERATOR_REGISTRATION"),
            publication_director: text("EBC_SHELF_LEGAL_PUBLICATION_DIRECTOR"),
            host_name: text("EBC_SHELF_LEGAL_HOST_NAME"),
            host_address: text("EBC_SHELF_LEGAL_HOST_ADDRESS"),
            host_phone: text("EBC_SHELF_LEGAL_HOST_PHONE"),
        };
        let suspend_after_upheld = small("EBC_SHELF_SUSPEND_AFTER_UPHELD", 2)?;
        let suspend_after_false_declarations =
            small("EBC_SHELF_SUSPEND_AFTER_FALSE_DECLARATIONS", 1)?;
        if suspend_after_upheld == 0 || suspend_after_false_declarations == 0 {
            return Err(bad("EBC_SHELF_SUSPEND_AFTER_*", "must be at least 1"));
        }
        let config = Self {
            bind,
            public_url,
            data_dir: PathBuf::from(
                text("EBC_SHELF_DATA_DIR").unwrap_or_else(|| "./var-shelf".to_owned()),
            ),
            name: text("EBC_SHELF_NAME").unwrap_or_else(|| "Étagère eBook Cadeau".to_owned()),
            mode,
            operator_email,
            secret,
            core_url,
            core_public_url: core_public_url.trim_end_matches('/').to_owned(),
            partner_id,
            partner_secret,
            smtp_url: text("EBC_SMTP_URL"),
            mail_from: text("EBC_SHELF_MAIL_FROM")
                .unwrap_or_else(|| "etagere@localhost".to_owned()),
            contact_email,
            max_file_bytes: mb("EBC_SHELF_MAX_FILE_MB", 50)?,
            account_quota_bytes: mb("EBC_SHELF_QUOTA_ACCOUNT_MB", 200)?,
            total_quota_bytes: mb("EBC_SHELF_QUOTA_TOTAL_MB", 5_120)?,
            login_ttl_secs: days("EBC_SHELF_LOGIN_TTL_MIN", 15)? * 60,
            session_ttl_secs: days("EBC_SHELF_SESSION_DAYS", 7)? * 86_400,
            link_ttl_secs: days("EBC_SHELF_LINK_TTL_SECS", 300)?,
            suspend_after_upheld,
            suspend_after_false_declarations,
            appeal_days: days("EBC_SHELF_APPEAL_DAYS", 182)?,
            retention,
            legal,
            rate_login_per_hour: small(
                "EBC_SHELF_RATE_LOGIN_PER_HOUR",
                defaults.rate_login_per_hour,
            )?,
            rate_notices_per_hour: small(
                "EBC_SHELF_RATE_NOTICES_PER_HOUR",
                defaults.rate_notices_per_hour,
            )?,
            rate_uploads_per_hour: small(
                "EBC_SHELF_RATE_UPLOADS_PER_HOUR",
                defaults.rate_uploads_per_hour,
            )?,
            rate_fail_per_10min: small(
                "EBC_SHELF_RATE_FAIL_PER_10MIN",
                defaults.rate_fail_per_10min,
            )?,
            max_gifts_per_request: small("EBC_SHELF_MAX_GIFTS_PER_REQUEST", 50)?.clamp(1, 500),
            header_read_timeout_secs: number("EBC_SHELF_HEADER_TIMEOUT_SECS", 30)?,
            request_timeout_secs: number("EBC_SHELF_REQUEST_TIMEOUT_SECS", 300)?,
            body_idle_timeout_secs: number("EBC_SHELF_BODY_IDLE_TIMEOUT_SECS", 30)?,
            max_connections: small("EBC_SHELF_MAX_CONNECTIONS", 1_024)?,
            max_concurrent_uploads: small("EBC_SHELF_MAX_CONCURRENT_UPLOADS", 4)?.max(1),
            trust_proxy: matches!(text("EBC_SHELF_TRUST_PROXY").as_deref(), Some("1" | "true")),
            outbound_timeout_ms: 15_000,
        };
        if config.link_ttl_secs > 3_600 {
            return Err(bad("EBC_SHELF_LINK_TTL_SECS", "at most 3600 (links must be short-lived)"));
        }
        Ok(config)
    }

    /// Whether the public URL is `https` (the session cookie is then `Secure` and
    /// `__Host-` prefixed).
    #[must_use]
    pub fn is_https(&self) -> bool {
        self.public_url.starts_with("https://")
    }

    /// `host[:port]` of the public URL: the entry to list in the core's `redirect_hosts`.
    #[must_use]
    pub fn public_host(&self) -> &str {
        self.public_url.split_once("://").map_or(self.public_url.as_str(), |(_, host)| host)
    }

    /// Whether the public URL points to this machine (development).
    #[must_use]
    pub fn is_loopback(&self) -> bool {
        is_loopback(&self.public_url)
    }

    /// The §4.5 endpoint as the core must call it (`delivery.url` of every gift).
    #[must_use]
    pub fn link_endpoint(&self) -> String {
        format!("{}/api/ebc/link", self.public_url)
    }
}

fn check_base_url(name: &'static str, url: &str) -> Result<(), ConfigError> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| bad(name, "must start with http:// or https://"))?;
    let rest = rest.trim_end_matches('/');
    if rest.is_empty()
        || rest.contains(['/', '?', '#', '@', ' '])
        || rest.chars().any(char::is_control)
    {
        return Err(bad(name, "scheme and host[:port] only, no path"));
    }
    Ok(())
}

fn is_loopback(url: &str) -> bool {
    let host = url.split_once("://").map_or(url, |(_, h)| h);
    let host = host
        .rsplit_once(':')
        .map_or(host, |(h, port)| if port.bytes().all(|b| b.is_ascii_digit()) { h } else { host });
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

fn is_token(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 64
        && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// An e-mail address in canonical form (trimmed, lower case), or `None` when it is not a
/// plain `local@domain` address that can safely go into a mail header.
#[must_use]
pub fn normalise_email(text: &str) -> Option<String> {
    let text = text.trim().to_lowercase();
    let (local, domain) = text.split_once('@')?;
    let plain = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '+' | '\''))
    };
    let ok = text.len() <= 254
        && plain(local)
        && plain(domain)
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !text.contains("..");
    ok.then_some(text)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> =
            pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        move |name| map.get(name).cloned()
    }

    const BASE: [(&str, &str); 2] = [
        ("EBC_SHELF_OPERATOR_EMAIL", " Autrice@Exemple.FR "),
        ("EBC_SHELF_PARTNER_SECRET", "dev-secret-etagere-0123456789"),
    ];

    #[test]
    fn defaults_are_a_loopback_single_author_shelf() {
        let config = Config::from_lookup(lookup(&BASE)).unwrap();
        assert_eq!(config.mode, Mode::Own);
        assert_eq!(config.operator_email, "autrice@exemple.fr");
        assert_eq!(config.public_host(), "127.0.0.1:8104");
        assert_eq!(config.link_endpoint(), "http://127.0.0.1:8104/api/ebc/link");
        assert!(config.secret.is_empty(), "generated by the binary on loopback");
        assert_eq!(config.max_file_bytes, 50 << 20);
        assert_eq!(config.retention, Retention::default());
        assert!(!config.is_https());
    }

    #[test]
    fn a_public_shelf_needs_its_secrets() {
        let mut pairs = BASE.to_vec();
        pairs.push(("EBC_SHELF_PUBLIC_URL", "https://etagere.example"));
        let error = Config::from_lookup(lookup(&pairs)).unwrap_err();
        assert_eq!(error.name, "EBC_SHELF_SECRET");
        pairs.push(("EBC_SHELF_SECRET", "short"));
        assert_eq!(Config::from_lookup(lookup(&pairs)).unwrap_err().name, "EBC_SHELF_SECRET");
        pairs.pop();
        pairs.push(("EBC_SHELF_SECRET", "a-long-enough-secret-value"));
        pairs.push(("EBC_SHELF_MODE", "multi"));
        let config = Config::from_lookup(lookup(&pairs)).unwrap();
        assert!(config.is_https() && config.mode == Mode::Multi);
        assert_eq!(config.public_host(), "etagere.example");
    }

    #[test]
    fn bad_values_are_named() {
        let cases: [(&str, &str, &str); 9] = [
            ("EBC_SHELF_MODE", "shared", "EBC_SHELF_MODE"),
            ("EBC_SHELF_PUBLIC_URL", "https://x.example/path", "EBC_SHELF_PUBLIC_URL"),
            ("EBC_SHELF_PUBLIC_URL", "ftp://x.example", "EBC_SHELF_PUBLIC_URL"),
            ("EBC_CORE_URL", "x.example", "EBC_CORE_URL"),
            ("EBC_SHELF_MAX_FILE_MB", "0", "EBC_SHELF_MAX_FILE_MB"),
            ("EBC_SHELF_MAX_FILE_MB", "lots", "EBC_SHELF_MAX_FILE_MB"),
            ("EBC_SHELF_RETAIN_IDENTITY_DAYS", "0", "EBC_SHELF_RETAIN_IDENTITY_DAYS"),
            ("EBC_SHELF_LINK_TTL_SECS", "86400", "EBC_SHELF_LINK_TTL_SECS"),
            ("EBC_SHELF_PARTNER_ID", "a b", "EBC_SHELF_PARTNER_ID"),
        ];
        for (name, value, expected) in cases {
            let mut pairs = BASE.to_vec();
            pairs.push((name, value));
            let error = Config::from_lookup(lookup(&pairs)).unwrap_err();
            assert_eq!(error.name, expected, "{name}={value}");
        }
        let no_operator = lookup(&[("EBC_SHELF_PARTNER_SECRET", "dev-secret-etagere-0123456789")]);
        assert_eq!(Config::from_lookup(no_operator).unwrap_err().name, "EBC_SHELF_OPERATOR_EMAIL");
    }

    #[test]
    fn e_mail_addresses_are_plain() {
        assert_eq!(normalise_email(" A.B+c@Exemple.fr ").as_deref(), Some("a.b+c@exemple.fr"));
        for bad in [
            "",
            "a",
            "a@b",
            "@b.fr",
            "a@.fr",
            "a@b.fr.",
            "a b@c.fr",
            "a@b.fr\r\nBcc: x@y.fr",
            "\"a\"@b.fr",
            "a@b@c.fr",
            "a..b@c.fr",
            "<a@b.fr>",
        ] {
            assert_eq!(normalise_email(bad), None, "{bad:?}");
        }
    }
}
