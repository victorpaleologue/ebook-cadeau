//! `SimulatedLcpReader`: the **shape** of the Readium LCP flow, with **no cryptography**.
//!
//! # This is not LCP
//!
//! Nothing here encrypts, decrypts, signs or verifies anything. Real LCP needs EDRLab's
//! confidential material and a certified reading system, and "browser based applications
//! cannot integrate LCP" (`ereader-ux.ecosystems.md` §4.2 \[V1\]). This module exists so the
//! integration tests can exercise what a gift flow *looks like* when delivery is a licence:
//!
//! 1. the reading system receives a licence document (`.lcpl`, JSON) with an `id`, a
//!    `provider`, `encryption.user_key.text_hint` and a link `rel="publication"` \[V1\];
//! 2. it must show the hint (and the `rel="hint"` URL) when asking for the passphrase \[V1\];
//!    stored keys are tried first, so a second book from the same provider opens without a
//!    prompt \[V1\];
//! 3. the user types the passphrase; a wrong one is refused **locally**, with the hint;
//! 4. the reader then downloads the publication from the licence's link and adds it to the
//!    library.
//!
//! ## The simulated key check
//!
//! A real licence carries `encryption.user_key.key_check`, the licence id encrypted with the
//! user key (SHA-256 of the passphrase). The simulation keeps the *field* and the *idea* but
//! replaces encryption with a plain hash, prefixed so nobody mistakes one for the other:
//!
//! ```text
//! user_key  = hex(SHA-256(passphrase))
//! key_check = "SIMULATED-not-lcp:" + hex(SHA-256(licence_id + "\n" + user_key))
//! ```
//!
//! A licence without that prefix (i.e. a real LCP licence) is rejected as
//! [`LcpError::Unverifiable`]. The licence signature, the content key, rights (print / copy /
//! dates) and device registration with the status server are ignored; only the status
//! document's `status` is read, so a cancelled gift is refused.

use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{Error, LcpError, NetError, Result};
use crate::format::FileFormat;
use crate::library::{ImportSource, IncomingFile, Library, LibraryEntry};
use crate::net::{self, Exchange, Http, NetConfig, Request};
use crate::profile::{DeviceProfile, DrmScheme};
use crate::typing::{InputMode, TapLog, TypingCost};

/// Prefix of a simulated `key_check` value.
pub const SIMULATED_KEY_CHECK_PREFIX: &str = "SIMULATED-not-lcp:";
/// Media type of an LCP licence document.
pub const LICENCE_MEDIA_TYPE: &str = "application/vnd.readium.lcp.license.v1.0+json";
/// Largest licence or status document accepted.
const MAX_LICENCE_BYTES: u64 = 256 * 1024;

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The simulated user key: hex SHA-256 of the passphrase. **Not** a secret-keeping scheme.
pub fn simulated_user_key(passphrase: &str) -> String {
    sha256_hex(passphrase.as_bytes())
}

/// The simulated `key_check` for a licence id and passphrase (see the module docs).
pub fn simulated_key_check(licence_id: &str, passphrase: &str) -> String {
    key_check_for_user_key(licence_id, &simulated_user_key(passphrase))
}

fn key_check_for_user_key(licence_id: &str, user_key: &str) -> String {
    format!(
        "{SIMULATED_KEY_CHECK_PREFIX}{}",
        sha256_hex(format!("{licence_id}\n{user_key}").as_bytes())
    )
}

/// The `rel="publication"` link of a licence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationLink {
    /// Where the (notionally protected) publication is downloaded from.
    pub url: Url,
    /// Declared media type.
    pub media_type: String,
    /// Declared size in bytes.
    pub length: Option<u64>,
    /// Declared SHA-256 of the file, lower-case hex.
    pub hash: Option<String>,
}

/// A parsed **simulated** licence document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatedLicence {
    /// Licence id.
    pub id: String,
    /// Provider URI.
    pub provider: String,
    /// Hint shown when asking for the passphrase.
    pub text_hint: String,
    /// `rel="hint"` link.
    pub hint_url: Option<Url>,
    /// `rel="publication"` link.
    pub publication: PublicationLink,
    /// `rel="status"` link (licence status document).
    pub status_url: Option<Url>,
    key_check: String,
}

impl SimulatedLicence {
    /// Parses a licence document. Relative links are resolved against `base` (the URL the
    /// licence was fetched from), if given.
    pub fn parse(bytes: &[u8], base: Option<&Url>) -> Result<Self, LcpError> {
        if bytes.len() as u64 > MAX_LICENCE_BYTES {
            return Err(LcpError::MalformedLicence("larger than 256 KiB".to_owned()));
        }
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| LcpError::MalformedLicence(e.to_string()))?;
        let string = |pointer: &str| {
            value
                .pointer(pointer)
                .and_then(|v| v.as_str())
                .map(|s| s.trim().chars().take(2000).collect::<String>())
                .filter(|s| !s.is_empty())
        };
        let id = string("/id").ok_or_else(|| LcpError::MalformedLicence("no id".to_owned()))?;
        let provider = string("/provider")
            .ok_or_else(|| LcpError::MalformedLicence("no provider".to_owned()))?;
        let text_hint = string("/encryption/user_key/text_hint").ok_or_else(|| {
            LcpError::MalformedLicence("no encryption.user_key.text_hint".to_owned())
        })?;
        let resolve = |href: &str| -> Option<Url> {
            let url = match base {
                Some(base) => base.join(href.trim()).ok()?,
                None => Url::parse(href.trim()).ok()?,
            };
            matches!(url.scheme(), "http" | "https").then_some(url)
        };
        let links = value
            .get("links")
            .and_then(|l| l.as_array())
            .ok_or_else(|| LcpError::MalformedLicence("no links array".to_owned()))?;
        let link = |rel: &str| {
            links.iter().take(64).find(|l| match l.get("rel") {
                Some(serde_json::Value::String(r)) => r == rel,
                Some(serde_json::Value::Array(rels)) => {
                    rels.iter().any(|r| r.as_str() == Some(rel))
                }
                _ => false,
            })
        };
        let href = |l: &serde_json::Value| l.get("href").and_then(|h| h.as_str()).and_then(resolve);
        let publication = link("publication").ok_or(LcpError::NoPublicationLink)?;
        let publication = PublicationLink {
            url: href(publication).ok_or(LcpError::NoPublicationLink)?,
            media_type: publication.get("type").and_then(|t| t.as_str()).unwrap_or("").to_owned(),
            length: publication.get("length").and_then(serde_json::Value::as_u64),
            hash: publication
                .get("hash")
                .and_then(|h| h.as_str())
                .map(|h| h.trim().to_ascii_lowercase())
                .filter(|h| !h.is_empty()),
        };
        let key_check = string("/encryption/user_key/key_check").unwrap_or_default();
        if !key_check.starts_with(SIMULATED_KEY_CHECK_PREFIX) {
            return Err(LcpError::Unverifiable(
                "the key_check is not a simulated one; real LCP licences need a certified reading system"
                    .to_owned(),
            ));
        }
        Ok(Self {
            id,
            provider,
            text_hint,
            hint_url: link("hint").and_then(href),
            publication,
            status_url: link("status").and_then(href),
            key_check,
        })
    }

    /// Whether `passphrase` opens this licence (simulated check, done locally).
    pub fn accepts(&self, passphrase: &str) -> bool {
        self.accepts_user_key(&simulated_user_key(passphrase))
    }

    fn accepts_user_key(&self, user_key: &str) -> bool {
        key_check_for_user_key(&self.id, user_key) == self.key_check
    }
}

/// Builds a **simulated** licence document, for demo stores and tests.
#[derive(Debug, Clone)]
pub struct SimulatedLicenceBuilder {
    id: String,
    provider: String,
    publication_url: String,
    passphrase: String,
    text_hint: String,
    hint_url: Option<String>,
    status_url: Option<String>,
    length: Option<u64>,
    hash: Option<String>,
}

impl SimulatedLicenceBuilder {
    /// A licence `id` issued by `provider` for the publication at `publication_url`, opened by
    /// `passphrase`.
    pub fn new(id: &str, provider: &str, publication_url: &str, passphrase: &str) -> Self {
        Self {
            id: id.to_owned(),
            provider: provider.to_owned(),
            publication_url: publication_url.to_owned(),
            passphrase: passphrase.to_owned(),
            text_hint: "Le code imprimé dans votre carte".to_owned(),
            hint_url: None,
            status_url: None,
            length: None,
            hash: None,
        }
    }

    /// Sets the passphrase hint.
    pub fn text_hint(mut self, hint: &str) -> Self {
        self.text_hint = hint.to_owned();
        self
    }

    /// Adds a `rel="hint"` link.
    pub fn hint_url(mut self, url: &str) -> Self {
        self.hint_url = Some(url.to_owned());
        self
    }

    /// Adds a `rel="status"` link.
    pub fn status_url(mut self, url: &str) -> Self {
        self.status_url = Some(url.to_owned());
        self
    }

    /// Declares the publication's `length` and SHA-256 `hash` from its bytes.
    pub fn describing(mut self, publication: &[u8]) -> Self {
        self.length = Some(publication.len() as u64);
        self.hash = Some(sha256_hex(publication));
        self
    }

    /// Serialises the licence document (deterministic).
    pub fn build(&self) -> String {
        let mut publication = serde_json::json!({
            "rel": "publication",
            "href": self.publication_url,
            "type": "application/epub+zip",
        });
        if let (Some(length), Some(hash)) = (self.length, &self.hash) {
            publication["length"] = serde_json::json!(length);
            publication["hash"] = serde_json::json!(hash);
        }
        let mut links = vec![publication];
        if let Some(url) = &self.hint_url {
            links.push(serde_json::json!({"rel": "hint", "href": url, "type": "text/html"}));
        }
        if let Some(url) = &self.status_url {
            links.push(serde_json::json!({
                "rel": "status", "href": url,
                "type": "application/vnd.readium.license.status.v1.0+json"
            }));
        }
        serde_json::json!({
            "id": self.id,
            "issued": "2026-01-01T00:00:00Z",
            "provider": self.provider,
            "encryption": {
                "profile": "urn:ebook-cadeau:SIMULATED-lcp-shape:no-cryptography",
                "content_key": {"algorithm": "none", "encrypted_value": "SIMULATED"},
                "user_key": {
                    "algorithm": "http://www.w3.org/2001/04/xmlenc#sha256",
                    "text_hint": self.text_hint,
                    "key_check": simulated_key_check(&self.id, &self.passphrase),
                },
            },
            "links": links,
            "signature": {"algorithm": "none", "certificate": "SIMULATED", "value": "SIMULATED"},
        })
        .to_string()
    }
}

/// What the reading system shows when it needs the passphrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassphrasePrompt {
    /// Licence id.
    pub licence_id: String,
    /// Provider URI.
    pub provider: String,
    /// The hint text (must be displayed).
    pub text_hint: String,
    /// The hint link (must be displayed when present).
    pub hint_url: Option<Url>,
    /// Wrong passphrases entered for this licence so far.
    pub failed_attempts: u32,
}

/// Result of handing a licence to the reading system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceStep {
    /// A stored key opened the licence: the publication is in the library.
    Opened(LibraryEntry),
    /// The user must type the passphrase: call
    /// [`SimulatedLcpReader::enter_passphrase`].
    NeedsPassphrase(PassphrasePrompt),
}

/// A reading system that goes through the motions of LCP. **No real cryptography.**
#[derive(Debug)]
pub struct SimulatedLcpReader {
    profile: DeviceProfile,
    http: Http,
    library: Library,
    stored_user_keys: Vec<String>,
    pending: Option<(SimulatedLicence, u32)>,
    exchanges: Vec<Exchange>,
    taps: TapLog,
}

/// The name used in the task brief; the type is a simulation — see [`SimulatedLcpReader`].
pub type LcpReader = SimulatedLcpReader;

impl SimulatedLcpReader {
    /// A reader for `profile`. Fails with [`LcpError::DeviceLacksLcp`] when the profile's
    /// reading system has no LCP support (Kindle, Kobo…).
    pub fn new(profile: DeviceProfile) -> Result<Self> {
        Self::with_config(profile, NetConfig::default())
    }

    /// The same with explicit network limits.
    pub fn with_config(profile: DeviceProfile, net: NetConfig) -> Result<Self> {
        if !profile.unlocks(DrmScheme::ReadiumLcp) {
            return Err(LcpError::DeviceLacksLcp { device: profile.id }.into());
        }
        let http = Http::new(profile.user_agent, net)?;
        Ok(Self {
            profile,
            http,
            library: Library::new(),
            stored_user_keys: Vec::new(),
            pending: None,
            exchanges: Vec::new(),
            taps: TapLog::default(),
        })
    }

    /// The reader's library.
    pub fn library(&self) -> &Library {
        &self.library
    }

    /// Taps spent typing passphrases.
    pub fn taps(&self) -> &TapLog {
        &self.taps
    }

    /// Every HTTP exchange made so far.
    pub fn exchanges(&self) -> &[Exchange] {
        &self.exchanges
    }

    /// Number of user keys remembered from earlier licences.
    pub fn stored_keys(&self) -> usize {
        self.stored_user_keys.len()
    }

    /// The prompt of the licence currently waiting for a passphrase.
    pub fn pending_prompt(&self) -> Option<PassphrasePrompt> {
        self.pending.as_ref().map(|(licence, failed)| prompt(licence, *failed))
    }

    /// Downloads a licence document and opens it.
    pub async fn open_licence_url(&mut self, url: &str) -> Result<LicenceStep> {
        let url = net::parse_url(url)?;
        let request =
            Request::get(url).header("accept", format!("{LICENCE_MEDIA_TYPE}, application/json"));
        let (head, bytes) =
            self.http.fetch_bytes(request, MAX_LICENCE_BYTES, &mut self.exchanges).await?;
        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }
        let licence = SimulatedLicence::parse(&bytes, Some(&head.url))?;
        self.open(licence).await
    }

    /// Opens a licence document already on the device (e.g. downloaded by the browser).
    /// `source_url`, when known, is used to resolve relative links.
    pub async fn open_licence(
        &mut self,
        bytes: &[u8],
        source_url: Option<&Url>,
    ) -> Result<LicenceStep> {
        let licence = SimulatedLicence::parse(bytes, source_url)?;
        self.open(licence).await
    }

    async fn open(&mut self, licence: SimulatedLicence) -> Result<LicenceStep> {
        self.check_status(&licence).await?;
        if self.stored_user_keys.iter().any(|key| licence.accepts_user_key(key)) {
            let entry = self.fetch_publication(&licence).await?;
            return Ok(LicenceStep::Opened(entry));
        }
        let step = LicenceStep::NeedsPassphrase(prompt(&licence, 0));
        self.pending = Some((licence, 0));
        Ok(step)
    }

    /// Types the passphrase for the pending licence. A wrong passphrase is refused locally
    /// (nothing is downloaded) and the licence keeps waiting; the right one downloads the
    /// publication into the library and the key is remembered.
    ///
    /// Taps: one to focus the field, the keys, one to confirm.
    pub async fn enter_passphrase(&mut self, passphrase: &str) -> Result<LibraryEntry> {
        let Some((licence, failed)) = self.pending.take() else {
            return Err(LcpError::NoPendingLicence.into());
        };
        let cost = TypingCost::of(passphrase, &self.profile.keyboard, InputMode::Text);
        self.taps.record("type the LCP passphrase", cost.taps + 2);
        if !licence.accepts(passphrase) {
            let hint = licence.text_hint.clone();
            self.pending = Some((licence, failed.saturating_add(1)));
            return Err(LcpError::WrongPassphrase { hint }.into());
        }
        match self.fetch_publication(&licence).await {
            Ok(entry) => {
                let key = simulated_user_key(passphrase);
                if !self.stored_user_keys.contains(&key) && self.stored_user_keys.len() < 64 {
                    self.stored_user_keys.push(key);
                }
                Ok(entry)
            }
            Err(err) => {
                // Keep the licence so the user can retry once the network is back.
                self.pending = Some((licence, failed));
                Err(err)
            }
        }
    }

    /// Reads the licence status document, if the licence links to one. Unreachable or
    /// malformed status documents are ignored, as an offline reading system would.
    async fn check_status(&mut self, licence: &SimulatedLicence) -> Result<()> {
        let Some(url) = licence.status_url.clone() else { return Ok(()) };
        let request = Request::get(url).header("accept", "application/json");
        let Ok((head, bytes)) =
            self.http.fetch_bytes(request, MAX_LICENCE_BYTES, &mut self.exchanges).await
        else {
            return Ok(());
        };
        if !head.is_success() {
            return Ok(());
        }
        let status = serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|v| v.get("status").and_then(|s| s.as_str()).map(str::to_ascii_lowercase));
        match status.as_deref() {
            Some(status @ ("revoked" | "returned" | "cancelled" | "expired")) => {
                Err(LcpError::NotUsable { status: status.to_owned() }.into())
            }
            _ => Ok(()),
        }
    }

    async fn fetch_publication(&mut self, licence: &SimulatedLicence) -> Result<LibraryEntry> {
        let link = &licence.publication;
        let (head, response) =
            self.http.fetch(Request::get(link.url.clone()), None, &mut self.exchanges).await?;
        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }
        let bytes = self.http.read(&head, response, self.profile.max_download_bytes).await?;
        if let Some(length) = link.length
            && length != bytes.len() as u64
        {
            return Err(Error::from(LcpError::PublicationMismatch(format!(
                "the licence announces {length} bytes, the download has {}",
                bytes.len()
            ))));
        }
        if let Some(hash) = &link.hash
            && *hash != sha256_hex(&bytes)
        {
            return Err(LcpError::PublicationMismatch(
                "SHA-256 differs from the licence's hash".to_owned(),
            )
            .into());
        }
        let file_name = net::response_file_name(&head);
        let declared = FileFormat::from_media_type(&link.media_type)
            .or_else(|| head.header("content-type").and_then(FileFormat::from_media_type));
        let entry = self.library.import_with(
            &self.profile,
            IncomingFile { bytes: &bytes, file_name: &file_name, declared },
            ImportSource::SimulatedLcp { licence_id: licence.id.clone() },
            |entry| {
                entry.drm = Some(DrmScheme::ReadiumLcp);
                entry.notes.push(
                    "opened through the SIMULATED LCP flow (no cryptography involved)".to_owned(),
                );
            },
        )?;
        Ok(entry)
    }
}

fn prompt(licence: &SimulatedLicence, failed_attempts: u32) -> PassphrasePrompt {
    PassphrasePrompt {
        licence_id: licence.id.clone(),
        provider: licence.provider.clone(),
        text_hint: licence.text_hint.clone(),
        hint_url: licence.hint_url.clone(),
        failed_attempts,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn licence_json() -> String {
        SimulatedLicenceBuilder::new(
            "ef15e740-697f-11e3-949a-0800200c9a66",
            "https://store.example",
            "https://store.example/pub/le-horla.epub",
            "ABCDE-12345",
        )
        .hint_url("https://store.example/lcp/hint")
        .status_url("https://store.example/lcp/status/ef15")
        .describing(b"0123456789")
        .build()
    }

    #[test]
    fn built_licences_have_the_lcp_shape_and_parse_back() {
        let json = licence_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        // The members the LCP specification makes mandatory are present.
        for pointer in [
            "/id",
            "/issued",
            "/provider",
            "/encryption/profile",
            "/encryption/content_key",
            "/encryption/user_key/text_hint",
            "/encryption/user_key/key_check",
            "/links",
            "/signature",
        ] {
            assert!(value.pointer(pointer).is_some(), "missing {pointer}");
        }
        assert!(json.contains("SIMULATED"), "the document must say it is not real LCP");
        let licence = SimulatedLicence::parse(json.as_bytes(), None).unwrap();
        assert_eq!(licence.id, "ef15e740-697f-11e3-949a-0800200c9a66");
        assert_eq!(licence.provider, "https://store.example");
        assert_eq!(licence.text_hint, "Le code imprimé dans votre carte");
        assert_eq!(licence.hint_url.as_ref().unwrap().path(), "/lcp/hint");
        assert_eq!(licence.status_url.as_ref().unwrap().path(), "/lcp/status/ef15");
        assert_eq!(licence.publication.length, Some(10));
        assert_eq!(licence.publication.hash.as_deref(), Some(sha256_hex(b"0123456789").as_str()));
        assert_eq!(json, licence_json(), "deterministic");
    }

    #[test]
    fn only_the_right_passphrase_opens_the_licence() {
        let licence = SimulatedLicence::parse(licence_json().as_bytes(), None).unwrap();
        assert!(licence.accepts("ABCDE-12345"));
        for wrong in ["", "abcde-12345", "ABCDE-12346", "ABCDE12345", " ABCDE-12345"] {
            assert!(!licence.accepts(wrong), "{wrong:?}");
        }
        // The check is bound to the licence id: the same passphrase gives another key_check.
        assert_ne!(simulated_key_check("a", "p"), simulated_key_check("b", "p"));
        assert!(simulated_key_check("a", "p").starts_with(SIMULATED_KEY_CHECK_PREFIX));
    }

    #[test]
    fn real_or_broken_licences_are_rejected_with_typed_errors() {
        let good: serde_json::Value = serde_json::from_str(&licence_json()).unwrap();
        let without = |pointer: &str| {
            let mut v = good.clone();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            v.pointer_mut(parent).unwrap().as_object_mut().unwrap().remove(key);
            v.to_string()
        };
        assert!(matches!(
            SimulatedLicence::parse(without("/id").as_bytes(), None),
            Err(LcpError::MalformedLicence(m)) if m.contains("id")
        ));
        assert!(matches!(
            SimulatedLicence::parse(without("/encryption/user_key/text_hint").as_bytes(), None),
            Err(LcpError::MalformedLicence(_))
        ));
        assert!(matches!(
            SimulatedLicence::parse(without("/links").as_bytes(), None),
            Err(LcpError::MalformedLicence(_))
        ));
        // A real licence's key_check (base64 of AES output) is not something we can verify.
        let mut real = good.clone();
        real["encryption"]["user_key"]["key_check"] =
            serde_json::json!("jJEjUDipHK3OjGt6kFq7dcOLZuicQFUYwQ+TYkAIWKm6Xv6kpHFhF7LOkUK/Owww");
        assert!(matches!(
            SimulatedLicence::parse(real.to_string().as_bytes(), None),
            Err(LcpError::Unverifiable(_))
        ));
        let mut no_publication = good.clone();
        no_publication["links"] =
            serde_json::json!([{"rel": "hint", "href": "https://x.example/"}]);
        assert_eq!(
            SimulatedLicence::parse(no_publication.to_string().as_bytes(), None),
            Err(LcpError::NoPublicationLink)
        );
        let mut bad_scheme = good.clone();
        bad_scheme["links"][0]["href"] = serde_json::json!("file:///etc/passwd");
        assert_eq!(
            SimulatedLicence::parse(bad_scheme.to_string().as_bytes(), None),
            Err(LcpError::NoPublicationLink)
        );
        for garbage in [&b""[..], b"{", b"[]", b"null", b"\"x\"", b"\xff\xff", b"{\"id\": 7}"] {
            assert!(SimulatedLicence::parse(garbage, None).is_err());
        }
        let huge = vec![b' '; 300 * 1024];
        assert!(matches!(SimulatedLicence::parse(&huge, None), Err(LcpError::MalformedLicence(_))));
    }

    #[test]
    fn relative_links_need_a_base() {
        let json =
            SimulatedLicenceBuilder::new("l1", "https://store.example", "/pub/x.epub", "p").build();
        assert_eq!(
            SimulatedLicence::parse(json.as_bytes(), None),
            Err(LcpError::NoPublicationLink)
        );
        let base = Url::parse("http://127.0.0.1:9/d/session/x.lcpl").unwrap();
        let licence = SimulatedLicence::parse(json.as_bytes(), Some(&base)).unwrap();
        assert_eq!(licence.publication.url.as_str(), "http://127.0.0.1:9/pub/x.epub");
    }

    #[test]
    fn devices_without_lcp_cannot_host_the_reader() {
        for profile in [
            DeviceProfile::kobo_clara(),
            DeviceProfile::kindle_paperwhite(),
            DeviceProfile::tolino(),
        ] {
            let id = profile.id;
            let err = SimulatedLcpReader::new(profile).unwrap_err();
            assert!(matches!(err, Error::Lcp(LcpError::DeviceLacksLcp { device }) if device == id));
        }
        assert!(SimulatedLcpReader::new(DeviceProfile::vivlio()).is_ok());
        assert!(SimulatedLcpReader::new(DeviceProfile::thorium_desktop()).is_ok());
    }
}
