//! `SyncDevice`: an e-reader tied to an ecosystem account, the way a Kobo syncs purchases.
//!
//! This is the channel the research ranks first for reseller-integrated gifts
//! (`ereader-ux.devices.md` §7 lesson 10; `ereader-ux.ecosystems.md` §0 point 2): the
//! recipient redeems on a phone into their store account, and the book *appears* on the
//! e-reader at the next sync — the e-reader's browser is never involved and nothing is typed
//! on e-ink once the device is linked.
//!
//! The store side is a small device API (implemented by `demo/stores` for its ecosystem
//! store, and by the mock in this crate's tests):
//!
//! | Request | Response |
//! |---|---|
//! | `POST {store}/api/device/link` `{"email", "password"}` | `200 {"device_token": "…"}`; `401`/`403` for bad credentials |
//! | `GET {store}/api/device/library` + `Authorization: Bearer {token}` | `200 [{"id", "title", "author", "format", "download_url"}]`; `401` if the token is no longer valid |
//! | `GET {download_url}` + `Authorization: Bearer {token}` | the file |
//!
//! This API is a **demo contract**, not any vendor's real sync protocol (Kobo's is private).
//! The device token is only ever sent to the store's own origin: a `download_url` pointing
//! elsewhere is skipped rather than followed with credentials.

use std::collections::HashSet;

use reqwest::Method;
use url::Url;

use crate::error::{ImportError, NetError, Result, SyncError};
use crate::format::FileFormat;
use crate::library::{ImportSource, IncomingFile, Library, LibraryEntry};
use crate::net::{self, Body, Exchange, Http, NetConfig, Request};
use crate::profile::DeviceProfile;
use crate::typing::{InputMode, TapLog, TypingCost};

/// Limits of the sync client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncConfig {
    /// Network limits.
    pub net: NetConfig,
    /// Largest JSON document accepted from the store.
    pub max_listing_bytes: u64,
    /// Most books considered per sync.
    pub max_books: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self { net: NetConfig::default(), max_listing_bytes: 1024 * 1024, max_books: 500 }
    }
}

/// Why a listed book was not added.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkipReason {
    /// The listing entry is unusable (missing id or download URL).
    BadEntry(String),
    /// The download URL is on another origin; the device token is not sent there.
    CrossOrigin(String),
    /// The download failed.
    Download(NetError),
    /// The file was downloaded but the device cannot use it.
    Import(ImportError),
}

/// A listed book that was not added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedBook {
    /// The store's id (may be empty for unusable entries).
    pub id: String,
    /// The listed title.
    pub title: String,
    /// Why.
    pub reason: SkipReason,
}

/// Result of one sync.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncReport {
    /// Books added by this sync.
    pub added: Vec<LibraryEntry>,
    /// Books the store listed that could not be added.
    pub skipped: Vec<SkippedBook>,
    /// Books listed that an earlier sync had already fetched.
    pub already_present: usize,
}

/// An account-linked e-reader.
#[derive(Debug)]
pub struct SyncDevice {
    profile: DeviceProfile,
    http: Http,
    config: SyncConfig,
    store: Url,
    token: Option<String>,
    library: Library,
    synced: HashSet<String>,
    exchanges: Vec<Exchange>,
    taps: TapLog,
}

impl SyncDevice {
    /// A device of `profile` belonging to the ecosystem of the store at `store_url`.
    pub fn new(profile: DeviceProfile, store_url: &str) -> Result<Self> {
        Self::with_config(profile, store_url, SyncConfig::default())
    }

    /// The same with explicit limits.
    pub fn with_config(
        profile: DeviceProfile,
        store_url: &str,
        config: SyncConfig,
    ) -> Result<Self> {
        let mut store = net::parse_url(store_url)?;
        store.set_query(None);
        store.set_fragment(None);
        if !store.path().ends_with('/') {
            let path = format!("{}/", store.path());
            store.set_path(&path);
        }
        let http = Http::new(profile.user_agent, config.net.clone())?;
        Ok(Self {
            profile,
            http,
            config,
            store,
            token: None,
            library: Library::new(),
            synced: HashSet::new(),
            exchanges: Vec::new(),
            taps: TapLog::default(),
        })
    }

    /// The device's library.
    pub fn library(&self) -> &Library {
        &self.library
    }

    /// Whether the device holds a device token.
    pub fn is_linked(&self) -> bool {
        self.token.is_some()
    }

    /// Forgets the device token (sign out).
    pub fn unlink(&mut self) {
        self.token = None;
    }

    /// Taps spent on the device so far (typing the account credentials, starting a sync).
    pub fn taps(&self) -> &TapLog {
        &self.taps
    }

    /// Every HTTP exchange made so far.
    pub fn exchanges(&self) -> &[Exchange] {
        &self.exchanges
    }

    fn endpoint(&self, path: &str) -> Result<Url> {
        self.store.join(path).map_err(|e| {
            NetError::InvalidUrl { url: format!("{}{path}", self.store), reason: e.to_string() }
                .into()
        })
    }

    /// Signs the device in with the account's e-mail and password.
    ///
    /// Taps: the credentials are typed on the device's keyboard (one tap per field to focus
    /// it, the keys, one tap to confirm) — the one-off cost of an ecosystem device.
    pub async fn link(&mut self, email: &str, password: &str) -> Result<()> {
        let keyboard = &self.profile.keyboard;
        let typing = TypingCost::of(email, keyboard, InputMode::Text)
            + TypingCost::of(password, keyboard, InputMode::Text);
        self.taps.record("sign in to the store account on the device", typing.taps + 3);

        let body = serde_json::json!({ "email": email, "password": password });
        let request = Request {
            method: Method::POST,
            url: self.endpoint("api/device/link")?,
            headers: vec![("accept", "application/json".to_owned())],
            body: Some(Body {
                content_type: "application/json".to_owned(),
                bytes: body.to_string().into_bytes(),
            }),
        };
        let (head, bytes) = self
            .http
            .fetch_bytes(request, self.config.max_listing_bytes, &mut self.exchanges)
            .await?;
        if matches!(head.status, 400 | 401 | 403) {
            return Err(SyncError::BadCredentials { status: head.status }.into());
        }
        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| SyncError::Protocol(format!("link response is not JSON: {e}")))?;
        let token = value
            .get("device_token")
            .and_then(|t| t.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty() && t.len() <= 4096 && t.chars().all(|c| c.is_ascii_graphic()))
            .ok_or_else(|| {
                SyncError::Protocol("link response has no usable device_token".to_owned())
            })?;
        self.token = Some(token.to_owned());
        Ok(())
    }

    /// Pulls the account's library: lists it, then downloads every book not fetched before.
    /// One tap (the "Sync" button; many devices also sync on their own).
    ///
    /// A book that cannot be fetched or opened is reported in [`SyncReport::skipped`] and does
    /// not stop the others; it is retried at the next sync.
    pub async fn sync(&mut self) -> Result<SyncReport> {
        let token = self.token.clone().ok_or(SyncError::NotLinked)?;
        self.taps.record("sync", 1);
        let bearer = format!("Bearer {token}");
        let request = Request::get(self.endpoint("api/device/library")?)
            .header("accept", "application/json")
            .header("authorization", bearer.clone());
        let (head, bytes) = self
            .http
            .fetch_bytes(request, self.config.max_listing_bytes, &mut self.exchanges)
            .await?;
        if matches!(head.status, 401 | 403) {
            self.token = None;
            return Err(SyncError::Unauthorized { status: head.status }.into());
        }
        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }
        let listing: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| SyncError::Protocol(format!("library listing is not JSON: {e}")))?;
        let books = listing
            .as_array()
            .ok_or_else(|| SyncError::Protocol("library listing is not a JSON array".to_owned()))?;

        let mut report = SyncReport::default();
        for book in books.iter().take(self.config.max_books) {
            let text = |key: &str| match book.get(key) {
                Some(serde_json::Value::String(s)) => {
                    s.trim().chars().take(500).collect::<String>()
                }
                Some(serde_json::Value::Number(n)) => n.to_string(),
                _ => String::new(),
            };
            let (id, title) = (text("id"), text("title"));
            let skip =
                |reason: SkipReason| SkippedBook { id: id.clone(), title: title.clone(), reason };
            if id.is_empty() {
                report.skipped.push(skip(SkipReason::BadEntry("no id".to_owned())));
                continue;
            }
            if self.synced.contains(&id) {
                report.already_present += 1;
                continue;
            }
            let raw_url = text("download_url");
            let Some(url) = self
                .store
                .join(&raw_url)
                .ok()
                .filter(|u| !raw_url.is_empty() && matches!(u.scheme(), "http" | "https"))
            else {
                report
                    .skipped
                    .push(skip(SkipReason::BadEntry("no usable download_url".to_owned())));
                continue;
            };
            if !net::same_origin(&self.store, &url) {
                report.skipped.push(skip(SkipReason::CrossOrigin(url.to_string())));
                continue;
            }
            let request = Request::get(url).header("authorization", bearer.clone());
            let fetched = async {
                let (head, response) = self.http.fetch(request, None, &mut self.exchanges).await?;
                if !head.is_success() {
                    return Err(NetError::Status {
                        url: head.url.to_string(),
                        status: head.status,
                    });
                }
                let bytes =
                    self.http.read(&head, response, self.profile.max_download_bytes).await?;
                Ok((head, bytes))
            }
            .await;
            let (head, bytes) = match fetched {
                Ok(ok) => ok,
                Err(err) => {
                    report.skipped.push(skip(SkipReason::Download(err)));
                    continue;
                }
            };
            let file_name = net::response_file_name(&head);
            let listed = text("format");
            let declared = head
                .header("content-type")
                .and_then(FileFormat::from_media_type)
                .or_else(|| FileFormat::from_media_type(&listed))
                .or_else(|| {
                    FileFormat::from_file_name(&format!("x.{}", listed.to_ascii_lowercase()))
                })
                .or_else(|| FileFormat::from_file_name(&file_name));
            match self.library.import(
                &self.profile,
                IncomingFile { bytes: &bytes, file_name: &file_name, declared },
                ImportSource::Sync { id: id.clone() },
            ) {
                Ok(entry) => {
                    self.synced.insert(id);
                    report.added.push(entry);
                }
                Err(err) => report.skipped.push(skip(SkipReason::Import(err))),
            }
        }
        Ok(report)
    }
}
