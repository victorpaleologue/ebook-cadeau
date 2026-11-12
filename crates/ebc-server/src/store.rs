//! Storage: the records the server keeps, the [`Store`] persistence trait and its two
//! implementations ([`MemoryStore`], [`FileStore`]).
//!
//! The whole data set ([`Db`]) is held in memory by a [`Database`], which serialises
//! every change and hands the result to the [`Store`]. [`FileStore`] writes it as one
//! JSON snapshot (`db.json`, replaced atomically) under `EBC_DATA_DIR`, and keeps
//! uploaded files next to it, named by their SHA-256 (`files/<sha256>`).
//!
//! **Limits, stated plainly**: a snapshot is rewritten on every change, which is fine for
//! thousands of gifts and wrong for millions; there is one writer (a single server
//! process per data directory). Swapping in a real database only needs another `Store`.
//!
//! No clear gift code, pair code, manage token or download token is ever stored: only
//! HMACs and hashes (see [`crate::codes`]).

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use ebc_model::{BookMeta, Delivery, GiftStatus, Locale};
use serde::{Deserialize, Serialize};

use crate::upload::RightsBasis;

/// The storage failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// Reading or writing the data directory failed.
    #[error("storage i/o: {0}")]
    Io(String),
    /// The snapshot on disk cannot be understood.
    #[error("storage is corrupt: {0}")]
    Corrupt(String),
    /// A blob name that is not a SHA-256 in hexadecimal.
    #[error("invalid blob name")]
    InvalidBlobName,
}

/// A stored image or PDF attached to a gift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobRef {
    /// SHA-256 of the bytes (the blob name).
    pub sha256: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Sniffed media type.
    pub media_type: String,
}

/// A print order forwarded to the print provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintOrderRecord {
    /// The provider's order id.
    pub order_id: String,
    /// The product ordered.
    pub product: String,
    /// When the order was placed.
    pub created_at: i64,
}

/// One gift, as stored. Unlike [`ebc_model::Gift`] it never holds the clear code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GiftRecord {
    /// Non-secret identifier (`g_…`).
    pub id: String,
    /// HMAC of the current code ([`crate::codes::gift_code_hash`]).
    pub code_hash: String,
    /// Partner gifts only: the nonce the code is derived from
    /// ([`crate::codes::derive_gift_code`]), so that the card can be rendered server-side.
    #[serde(default)]
    pub code_nonce: Option<String>,
    /// Counter that goes with [`code_nonce`](Self::code_nonce).
    #[serde(default)]
    pub code_counter: u32,
    /// Partner that created the gift; `None` in standalone mode.
    #[serde(default)]
    pub partner: Option<String>,
    /// The partner's order reference.
    #[serde(default)]
    pub order_ref: Option<String>,
    /// SHA-256 of the giver's manage token (standalone mode).
    #[serde(default)]
    pub manage_token_hash: Option<String>,
    /// The book.
    pub book: BookMeta,
    /// Who gives.
    #[serde(default)]
    pub giver_name: Option<String>,
    /// Who receives.
    #[serde(default)]
    pub recipient_name: Option<String>,
    /// Personal message.
    #[serde(default)]
    pub message: Option<String>,
    /// Language of the pages.
    #[serde(default)]
    pub locale: Locale,
    /// How the book is delivered.
    pub delivery: Delivery,
    /// Hosted files: the upload the file came from.
    #[serde(default)]
    pub upload_id: Option<String>,
    /// Hosted files: the declaration made with the upload.
    #[serde(default)]
    pub rights_basis: Option<RightsBasis>,
    /// The rights basis the partner declared for the cover (`none` unless declared): a
    /// cover is only printed on a card under a declared basis.
    #[serde(default)]
    pub cover_rights: crate::card::CoverRights,
    /// Status last recorded (expiry is applied on read, see [`GiftRecord::status_at`]).
    pub status: GiftStatus,
    /// Creation time.
    pub created_at: i64,
    /// Optional expiry.
    #[serde(default)]
    pub expires_at: Option<i64>,
    /// Optional reveal date: no action before it.
    #[serde(default)]
    pub reveal_at: Option<i64>,
    /// First claim.
    #[serde(default)]
    pub redeemed_at: Option<i64>,
    /// Latest claim.
    #[serde(default)]
    pub last_claim_at: Option<i64>,
    /// Claims so far: download sessions minted, files sent to a device, redirects.
    #[serde(default)]
    pub claims: u32,
    /// Reseller confirmation: SHA-256 of the `account_ref` (the reference itself is not
    /// kept).
    #[serde(default)]
    pub account_ref_hash: Option<String>,
    /// The cover the partner supplied.
    #[serde(default)]
    pub cover: Option<BlobRef>,
    /// The card PDF the giver uploaded for printing.
    #[serde(default)]
    pub card_pdf: Option<BlobRef>,
    /// Print orders placed for this gift.
    #[serde(default)]
    pub print_orders: Vec<PrintOrderRecord>,
    /// Times at which the file was sent to a device address (addresses are not kept).
    #[serde(default)]
    pub sends: Vec<i64>,
    /// When the gift was cancelled (retention of cancelled gifts runs from here).
    #[serde(default)]
    pub ended_at: Option<i64>,
    /// When the record was anonymised after the retention period (`EBC_RETENTION_DAYS`):
    /// names, message, sendings and files are gone; id, status, dates and code hash stay
    /// so that the code keeps answering `410` and the partner can still read the status.
    #[serde(default)]
    pub anonymised_at: Option<i64>,
}

impl GiftRecord {
    /// The status to act on at `now`: a claimable gift past its expiry is `Expired`.
    #[must_use]
    pub fn status_at(&self, now: i64) -> GiftStatus {
        if self.status.allows_claim() && self.expires_at.is_some_and(|at| now >= at) {
            GiftStatus::Expired
        } else {
            self.status
        }
    }

    /// Whether the retention period of a cancelled or expired gift is over at `now`.
    #[must_use]
    pub fn retention_over(&self, now: i64, retention_secs: i64) -> bool {
        let ended = match self.status_at(now) {
            GiftStatus::Cancelled => self.ended_at,
            GiftStatus::Expired => self.expires_at,
            _ => None,
        };
        ended.is_some_and(|at| now.saturating_sub(at) >= retention_secs)
    }

    /// Forgets the personal data of the gift: names and message (the book stays, it is
    /// not about a person). Files are handled by the caller.
    pub fn forget_personal_data(&mut self) {
        self.giver_name = None;
        self.recipient_name = None;
        self.message = None;
        self.sends.clear();
    }

    /// Whether the gift may be shown and claimed at `now` (reveal date reached).
    #[must_use]
    pub fn is_revealed_at(&self, now: i64) -> bool {
        self.reveal_at.is_none_or(|at| now >= at)
    }
}

/// An uploaded eBook waiting for (or attached to) a gift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadRecord {
    /// Identifier quoted in a `hosted_file` delivery (`u_…`).
    pub id: String,
    /// SHA-256 of the bytes (the blob name).
    pub sha256: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Sniffed media type.
    pub media_type: String,
    /// The uploader's file name, sanitised.
    #[serde(default)]
    pub file_name: Option<String>,
    /// The mandatory declaration.
    pub rights_basis: RightsBasis,
    /// Partner that uploaded; `None` in standalone mode.
    #[serde(default)]
    pub partner: Option<String>,
    /// Upload time.
    pub created_at: i64,
    /// The gift the file now belongs to.
    #[serde(default)]
    pub gift_id: Option<String>,
}

/// A download session: a short-lived URL token giving access to one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    /// The gift.
    pub gift_id: String,
    /// ASCII file name used in the URL and as `filename=`.
    pub file_name: String,
    /// Full file name offered through `filename*`.
    pub display_name: String,
    /// Media type served.
    pub media_type: String,
    /// The blob served.
    pub sha256: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// When the session ends.
    pub expires_at: i64,
    /// Device class chosen when the session was minted (`kobo`, `mobile`…).
    #[serde(default)]
    pub device: String,
    /// Whether the blob is a licence fetched for this session (deleted with it).
    #[serde(default)]
    pub licence: bool,
}

/// A live pair code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairRecord {
    /// The gift.
    pub gift_id: String,
    /// When the code stops working.
    pub expires_at: i64,
}

/// What an idempotent request produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdempotentOutcome {
    /// A response replayed as stored.
    Response {
        /// HTTP status.
        status: u16,
        /// JSON body.
        body: String,
    },
    /// A response that carried a gift code: rebuilt on replay from the gift's nonce, so
    /// that the clear code is not stored here either.
    GiftCode {
        /// HTTP status.
        status: u16,
        /// The gift.
        gift_id: String,
        /// Whether the response was a reissue (no `id`/`status` members).
        reissue: bool,
    },
}

/// A remembered `Idempotency-Key`, or a used request signature (a nonce).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    /// SHA-256 of the request body.
    pub body_sha256: String,
    /// When the key was first seen.
    pub created_at: i64,
    /// What to answer on replay.
    pub outcome: IdempotentOutcome,
    /// When the record may be forgotten; `None`: after the idempotency TTL. Signature
    /// records only need to outlive the timestamp window of the signature.
    #[serde(default)]
    pub expires_at: Option<i64>,
}

impl IdempotencyRecord {
    /// Whether the record still counts at `now`.
    #[must_use]
    pub fn is_live(&self, now: i64, idempotency_ttl_secs: i64) -> bool {
        match self.expires_at {
            Some(at) => now < at,
            None => now.saturating_sub(self.created_at) < idempotency_ttl_secs,
        }
    }
}

/// What [`Db::purge`] drops, and when (all in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurgePolicy {
    /// Lifetime of a remembered `Idempotency-Key`.
    pub idempotency_ttl_secs: i64,
    /// Lifetime of a partner upload never attached to a gift.
    pub upload_orphan_ttl_secs: i64,
    /// Lifetime of an anonymous upload never attached to a gift.
    pub anonymous_upload_orphan_ttl_secs: i64,
    /// Time a cancelled or expired gift is kept before it is anonymised.
    pub retention_secs: i64,
}

/// A webhook event not yet acknowledged by the partner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookRecord {
    /// Recipient.
    pub partner: String,
    /// The JSON event, exactly as it is (re)sent.
    pub body: String,
    /// When the event happened.
    pub created_at: i64,
}

/// Everything the server remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Db {
    /// Gifts by id.
    #[serde(default)]
    pub gifts: BTreeMap<String, GiftRecord>,
    /// Live codes: code hash → gift id.
    #[serde(default)]
    pub codes: BTreeMap<String, String>,
    /// Codes voided by a reissue: code hash → when. They answer `410`.
    #[serde(default)]
    pub voided: BTreeMap<String, i64>,
    /// Uploads by id.
    #[serde(default)]
    pub uploads: BTreeMap<String, UploadRecord>,
    /// Download sessions by token hash.
    #[serde(default)]
    pub sessions: BTreeMap<String, SessionRecord>,
    /// Pair codes by code hash.
    #[serde(default)]
    pub pairs: BTreeMap<String, PairRecord>,
    /// Idempotency keys by `partner \n method \n path \n key`.
    #[serde(default)]
    pub idempotency: BTreeMap<String, IdempotencyRecord>,
    /// Webhook outbox by event id.
    #[serde(default)]
    pub webhooks: BTreeMap<String, WebhookRecord>,
}

impl Db {
    /// Whether any record still needs the blob `sha256`.
    #[must_use]
    pub fn blob_in_use(&self, sha256: &str) -> bool {
        self.uploads.values().any(|u| u.sha256 == sha256)
            || self.sessions.values().any(|s| s.sha256 == sha256)
            || self.gifts.values().any(|g| {
                g.cover.as_ref().is_some_and(|c| c.sha256 == sha256)
                    || g.card_pdf.as_ref().is_some_and(|c| c.sha256 == sha256)
                    || (g.status != GiftStatus::Cancelled
                        && g.anonymised_at.is_none()
                        && matches!(&g.delivery, Delivery::HostedFile { sha256: s, .. } if s == sha256))
            })
    }

    /// Drops what has expired at `now`: download sessions, pair codes, idempotency keys
    /// and used signatures, uploads never attached to a gift in time; blanks names and
    /// message of cancelled gifts and anonymises cancelled or expired gifts whose
    /// retention period is over (their files go too). Returns the blobs that may now be
    /// unused (check with [`blob_in_use`](Self::blob_in_use) before deleting).
    pub fn purge(&mut self, now: i64, policy: &PurgePolicy) -> Vec<String> {
        let mut candidates = Vec::new();
        self.sessions.retain(|_, session| {
            let keep = session.expires_at > now;
            if !keep && session.licence {
                candidates.push(session.sha256.clone());
            }
            keep
        });
        self.pairs.retain(|_, pair| pair.expires_at > now);
        self.idempotency.retain(|_, record| record.is_live(now, policy.idempotency_ttl_secs));
        let mut dropped_uploads = Vec::new();
        for gift in self.gifts.values_mut() {
            if gift.status == GiftStatus::Cancelled {
                gift.forget_personal_data();
            }
            if gift.anonymised_at.is_some() || !gift.retention_over(now, policy.retention_secs) {
                continue;
            }
            gift.forget_personal_data();
            gift.anonymised_at = Some(now);
            if let Delivery::HostedFile { sha256, .. } = &gift.delivery {
                candidates.push(sha256.clone());
            }
            for blob in [gift.cover.take(), gift.card_pdf.take()].into_iter().flatten() {
                candidates.push(blob.sha256);
            }
            dropped_uploads.extend(gift.upload_id.clone());
            let id = gift.id.clone();
            self.sessions.retain(|_, session| session.gift_id != id);
            self.pairs.retain(|_, pair| pair.gift_id != id);
        }
        for upload_id in dropped_uploads {
            if let Some(upload) = self.uploads.remove(&upload_id) {
                candidates.push(upload.sha256);
            }
        }
        self.uploads.retain(|_, upload| {
            let ttl = if upload.partner.is_some() {
                policy.upload_orphan_ttl_secs
            } else {
                policy.anonymous_upload_orphan_ttl_secs
            };
            let keep = upload.gift_id.is_some() || now.saturating_sub(upload.created_at) < ttl;
            if !keep {
                candidates.push(upload.sha256.clone());
            }
            keep
        });
        candidates
    }

    /// Bytes held by anonymous (creator API) uploads, attached or not.
    #[must_use]
    pub fn anonymous_upload_bytes(&self) -> u64 {
        self.uploads
            .values()
            .filter(|upload| upload.partner.is_none())
            .fold(0u64, |sum, upload| sum.saturating_add(upload.size_bytes))
    }
}

/// A stored file, ready to be served.
#[derive(Debug, Clone)]
pub enum Blob {
    /// Held in memory.
    Memory(Bytes),
    /// A file on disk.
    File {
        /// Where it is.
        path: PathBuf,
        /// Its length in bytes.
        len: u64,
    },
}

impl Blob {
    /// Length in bytes.
    #[must_use]
    pub fn len(&self) -> u64 {
        match self {
            Self::Memory(bytes) => bytes.len() as u64,
            Self::File { len, .. } => *len,
        }
    }

    /// Whether the blob has no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads the whole blob into memory.
    ///
    /// # Errors
    /// [`StoreError::Io`] when the file cannot be read.
    pub fn read_all(&self) -> Result<Bytes, StoreError> {
        match self {
            Self::Memory(bytes) => Ok(bytes.clone()),
            Self::File { path, .. } => std::fs::read(path)
                .map(Bytes::from)
                .map_err(|e| StoreError::Io(format!("{}: {e}", path.display()))),
        }
    }
}

/// Where the data set and the files are persisted.
pub trait Store: Send + Sync + 'static {
    /// Loads the data set (an empty one on first start).
    ///
    /// # Errors
    /// When the snapshot is unreadable or corrupt.
    fn load(&self) -> Result<Db, StoreError>;

    /// Persists the data set.
    ///
    /// # Errors
    /// When the snapshot cannot be written.
    fn save(&self, db: &Db) -> Result<(), StoreError>;

    /// Stores a file under its SHA-256 (64 hexadecimal characters). Idempotent.
    ///
    /// # Errors
    /// [`StoreError::InvalidBlobName`] or an I/O failure.
    fn put_blob(&self, sha256: &str, bytes: &[u8]) -> Result<(), StoreError>;

    /// Opens a stored file; `None` when there is none under that name.
    ///
    /// # Errors
    /// [`StoreError::InvalidBlobName`] or an I/O failure.
    fn open_blob(&self, sha256: &str) -> Result<Option<Blob>, StoreError>;

    /// Deletes a stored file; deleting a missing file is not an error.
    ///
    /// # Errors
    /// [`StoreError::InvalidBlobName`] or an I/O failure.
    fn delete_blob(&self, sha256: &str) -> Result<(), StoreError>;
}

/// Only names that are a SHA-256 in lower-case hexadecimal ever reach the file system.
fn check_blob_name(sha256: &str) -> Result<(), StoreError> {
    let ok = sha256.len() == 64 && sha256.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    if ok { Ok(()) } else { Err(StoreError::InvalidBlobName) }
}

/// Keeps everything in memory; nothing survives the process. For tests and demos.
#[derive(Debug, Default)]
pub struct MemoryStore {
    blobs: Mutex<HashMap<String, Bytes>>,
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn blobs(&self) -> std::sync::MutexGuard<'_, HashMap<String, Bytes>> {
        self.blobs.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Number of files held (for tests).
    #[must_use]
    pub fn blob_count(&self) -> usize {
        self.blobs().len()
    }
}

impl Store for MemoryStore {
    fn load(&self) -> Result<Db, StoreError> {
        Ok(Db::default())
    }

    fn save(&self, _db: &Db) -> Result<(), StoreError> {
        Ok(())
    }

    fn put_blob(&self, sha256: &str, bytes: &[u8]) -> Result<(), StoreError> {
        check_blob_name(sha256)?;
        self.blobs().entry(sha256.to_owned()).or_insert_with(|| Bytes::copy_from_slice(bytes));
        Ok(())
    }

    fn open_blob(&self, sha256: &str) -> Result<Option<Blob>, StoreError> {
        check_blob_name(sha256)?;
        Ok(self.blobs().get(sha256).cloned().map(Blob::Memory))
    }

    fn delete_blob(&self, sha256: &str) -> Result<(), StoreError> {
        check_blob_name(sha256)?;
        self.blobs().remove(sha256);
        Ok(())
    }
}

/// Persists under a directory: `db.json` (one JSON snapshot, replaced atomically) and
/// `files/<sha256>`.
#[derive(Debug)]
pub struct FileStore {
    dir: PathBuf,
}

impl FileStore {
    /// Opens (creating it if needed) the data directory.
    ///
    /// # Errors
    /// When the directory cannot be created.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let dir = dir.into();
        std::fs::create_dir_all(dir.join("files")).map_err(|e| io_error(&dir, &e))?;
        Ok(Self { dir })
    }

    fn snapshot_path(&self) -> PathBuf {
        self.dir.join("db.json")
    }

    fn blob_path(&self, sha256: &str) -> Result<PathBuf, StoreError> {
        check_blob_name(sha256)?;
        Ok(self.dir.join("files").join(sha256))
    }
}

fn io_error(path: &Path, error: &std::io::Error) -> StoreError {
    StoreError::Io(format!("{}: {error}", path.display()))
}

/// Writes `bytes` to `path` through a temporary file and a rename, so that a crash never
/// leaves a half-written file under the final name.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, bytes).map_err(|e| io_error(&temporary, &e))?;
    std::fs::rename(&temporary, path).map_err(|e| io_error(path, &e))
}

impl Store for FileStore {
    fn load(&self) -> Result<Db, StoreError> {
        let path = self.snapshot_path();
        match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| StoreError::Corrupt(format!("{}: {e}", path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Db::default()),
            Err(e) => Err(io_error(&path, &e)),
        }
    }

    fn save(&self, db: &Db) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec(db).map_err(|e| StoreError::Corrupt(e.to_string()))?;
        write_atomically(&self.snapshot_path(), &bytes)
    }

    fn put_blob(&self, sha256: &str, bytes: &[u8]) -> Result<(), StoreError> {
        let path = self.blob_path(sha256)?;
        if path.is_file() {
            return Ok(());
        }
        write_atomically(&path, bytes)
    }

    fn open_blob(&self, sha256: &str) -> Result<Option<Blob>, StoreError> {
        let path = self.blob_path(sha256)?;
        match std::fs::metadata(&path) {
            Ok(meta) if meta.is_file() => Ok(Some(Blob::File { len: meta.len(), path })),
            Ok(_) => Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_error(&path, &e)),
        }
    }

    fn delete_blob(&self, sha256: &str) -> Result<(), StoreError> {
        let path = self.blob_path(sha256)?;
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io_error(&path, &e)),
        }
    }
}

/// The data set in memory, kept in step with a [`Store`].
pub struct Database {
    db: Mutex<Db>,
    store: Arc<dyn Store>,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}

impl Database {
    /// Loads the data set from `store`.
    ///
    /// # Errors
    /// When the store cannot be read.
    pub fn open(store: Arc<dyn Store>) -> Result<Self, StoreError> {
        Ok(Self { db: Mutex::new(store.load()?), store })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Db> {
        // A poisoned lock means a request panicked mid-change; the data is still the
        // best we have and every change is validated before it is applied.
        self.db.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Reads from the data set.
    pub fn view<R>(&self, read: impl FnOnce(&Db) -> R) -> R {
        read(&self.lock())
    }

    /// Changes the data set and persists it before returning.
    ///
    /// # Errors
    /// When persisting fails. The change is then kept in memory and written with the
    /// next successful change.
    pub fn update<R>(&self, change: impl FnOnce(&mut Db) -> R) -> Result<R, StoreError> {
        let mut db = self.lock();
        let result = change(&mut db);
        self.store.save(&db)?;
        Ok(result)
    }

    /// The underlying store (for files).
    #[must_use]
    pub fn store(&self) -> &Arc<dyn Store> {
        &self.store
    }

    /// Deletes each of `candidates` that no record needs any more.
    ///
    /// The check and the deletion happen under the data lock, so a file cannot be
    /// deleted after a record referencing it has been committed. Writers store the file,
    /// commit the record, then store the file again (a no-op unless a deletion slipped in
    /// between the first write and the commit).
    pub fn delete_unused_blobs(&self, candidates: &[String]) {
        let db = self.lock();
        for sha256 in candidates {
            if !db.blob_in_use(sha256)
                && let Err(error) = self.store.delete_blob(sha256)
            {
                tracing::warn!(%error, "could not delete an unused file");
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const SHA: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    fn gift(id: &str) -> GiftRecord {
        GiftRecord {
            id: id.to_owned(),
            code_hash: format!("hash-{id}"),
            code_nonce: None,
            code_counter: 0,
            partner: None,
            order_ref: None,
            manage_token_hash: None,
            book: BookMeta::titled("Le Horla"),
            giver_name: None,
            recipient_name: None,
            message: None,
            locale: Locale::Fr,
            delivery: Delivery::HostedFile {
                file_name: "le-horla.epub".to_owned(),
                media_type: "application/epub+zip".to_owned(),
                size_bytes: 4,
                sha256: SHA.to_owned(),
            },
            upload_id: None,
            rights_basis: Some(RightsBasis::PublicDomain),
            cover_rights: crate::card::CoverRights::None,
            status: GiftStatus::Created,
            created_at: 100,
            expires_at: None,
            reveal_at: None,
            redeemed_at: None,
            last_claim_at: None,
            claims: 0,
            account_ref_hash: None,
            cover: None,
            card_pdf: None,
            print_orders: Vec::new(),
            sends: Vec::new(),
            ended_at: None,
            anonymised_at: None,
        }
    }

    const POLICY: PurgePolicy = PurgePolicy {
        idempotency_ttl_secs: 1000,
        upload_orphan_ttl_secs: 10,
        anonymous_upload_orphan_ttl_secs: 10,
        retention_secs: 30,
    };

    #[test]
    fn file_store_round_trips_the_snapshot_and_files() {
        let dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> = Arc::new(FileStore::open(dir.path()).unwrap());
        let database = Database::open(Arc::clone(&store)).unwrap();
        database.update(|db| db.gifts.insert("g_1".to_owned(), gift("g_1"))).unwrap();
        store.put_blob(SHA, b"test").unwrap();
        store.put_blob(SHA, b"ignored: same name, already there").unwrap();

        // A second process start sees the same data.
        let reopened = Database::open(Arc::new(FileStore::open(dir.path()).unwrap())).unwrap();
        assert_eq!(reopened.view(|db| db.gifts.get("g_1").cloned()), Some(gift("g_1")));
        let blob = reopened.store().open_blob(SHA).unwrap().unwrap();
        assert_eq!(blob.len(), 4);
        assert_eq!(blob.read_all().unwrap().as_ref(), b"test");
        assert!(!dir.path().join("db.tmp").exists());

        store.delete_blob(SHA).unwrap();
        store.delete_blob(SHA).unwrap();
        assert!(store.open_blob(SHA).unwrap().is_none());
    }

    #[test]
    fn a_corrupt_snapshot_is_an_error_not_an_empty_database() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("db.json"), b"{ not json").unwrap();
        let store = FileStore::open(dir.path()).unwrap();
        assert!(matches!(store.load(), Err(StoreError::Corrupt(_))));
    }

    #[test]
    fn blob_names_cannot_escape_the_directory() {
        let dir = tempfile::tempdir().unwrap();
        let file = FileStore::open(dir.path()).unwrap();
        let memory = MemoryStore::new();
        for name in [
            "",
            "../db.json",
            "..",
            "/etc/passwd",
            &SHA.to_uppercase(),
            &format!("{SHA}/x"),
            &SHA[..63],
        ] {
            assert_eq!(file.put_blob(name, b"x"), Err(StoreError::InvalidBlobName), "{name}");
            assert!(matches!(file.open_blob(name), Err(StoreError::InvalidBlobName)), "{name}");
            assert_eq!(file.delete_blob(name), Err(StoreError::InvalidBlobName), "{name}");
            assert_eq!(memory.put_blob(name, b"x"), Err(StoreError::InvalidBlobName), "{name}");
        }
    }

    #[test]
    fn purge_drops_expired_records_and_reports_orphan_files() {
        let mut db = Db::default();
        let session = |expires_at, licence| SessionRecord {
            gift_id: "g_1".to_owned(),
            file_name: "a.epub".to_owned(),
            display_name: "a.epub".to_owned(),
            media_type: "application/epub+zip".to_owned(),
            sha256: SHA.to_owned(),
            size_bytes: 4,
            expires_at,
            device: "desktop".to_owned(),
            licence,
        };
        db.sessions.insert("old".to_owned(), session(50, true));
        db.sessions.insert("live".to_owned(), session(500, false));
        db.pairs
            .insert("old".to_owned(), PairRecord { gift_id: "g_1".to_owned(), expires_at: 100 });
        db.pairs
            .insert("live".to_owned(), PairRecord { gift_id: "g_1".to_owned(), expires_at: 101 });
        let upload = |created_at, gift_id: Option<&str>| UploadRecord {
            id: "u".to_owned(),
            sha256: "ab".repeat(32),
            size_bytes: 1,
            media_type: "application/pdf".to_owned(),
            file_name: None,
            rights_basis: RightsBasis::OwnWork,
            partner: None,
            created_at,
            gift_id: gift_id.map(str::to_owned),
        };
        db.uploads.insert("orphan".to_owned(), upload(0, None));
        db.uploads.insert("attached".to_owned(), upload(0, Some("g_1")));
        db.uploads.insert("fresh".to_owned(), upload(95, None));

        let candidates = db.purge(100, &POLICY);
        assert_eq!(db.sessions.keys().collect::<Vec<_>>(), ["live"]);
        assert_eq!(db.pairs.keys().collect::<Vec<_>>(), ["live"]);
        assert_eq!(db.uploads.keys().collect::<Vec<_>>(), ["attached", "fresh"]);
        assert_eq!(candidates, [SHA.to_owned(), "ab".repeat(32)]);
        // The live session still needs the first file; the attached upload the second.
        assert!(db.blob_in_use(SHA));
        assert!(db.blob_in_use(&"ab".repeat(32)));
    }

    #[test]
    fn retention_blanks_cancelled_gifts_and_anonymises_ended_ones_with_their_files() {
        let mut db = Db::default();
        let personal = |mut g: GiftRecord| {
            g.giver_name = Some("Camille".to_owned());
            g.recipient_name = Some("Sacha".to_owned());
            g.message = Some("Bon anniversaire".to_owned());
            g.sends = vec![90];
            g
        };
        let mut cancelled = personal(gift("g_cancelled"));
        cancelled.status = GiftStatus::Cancelled;
        cancelled.ended_at = Some(100);
        let mut expired = personal(gift("g_expired"));
        expired.expires_at = Some(100);
        expired.upload_id = Some("u_1".to_owned());
        expired.cover = Some(BlobRef {
            sha256: "cd".repeat(32),
            size_bytes: 1,
            media_type: "image/png".to_owned(),
        });
        let mut live = personal(gift("g_live"));
        if let Delivery::HostedFile { sha256, .. } = &mut live.delivery {
            *sha256 = "ef".repeat(32);
        }
        db.gifts.insert(cancelled.id.clone(), cancelled);
        db.gifts.insert(expired.id.clone(), expired);
        db.gifts.insert(live.id.clone(), live);
        db.uploads.insert(
            "u_1".to_owned(),
            UploadRecord {
                id: "u_1".to_owned(),
                sha256: SHA.to_owned(),
                size_bytes: 4,
                media_type: "application/epub+zip".to_owned(),
                file_name: Some("le-horla.epub".to_owned()),
                rights_basis: RightsBasis::PublicDomain,
                partner: None,
                created_at: 0,
                gift_id: Some("g_expired".to_owned()),
            },
        );
        assert_eq!(db.anonymous_upload_bytes(), 4);

        // Within the retention period: cancelled gifts lose names and message at once,
        // expired ones keep everything.
        assert!(db.purge(110, &POLICY).is_empty());
        let named = |db: &Db, id: &str| db.gifts[id].giver_name.is_some();
        assert!(!named(&db, "g_cancelled") && db.gifts["g_cancelled"].message.is_none());
        assert!(named(&db, "g_expired") && named(&db, "g_live"));
        assert!(db.blob_in_use(SHA));

        // After it: anonymised, files released, the live gift untouched.
        let candidates = db.purge(130, &POLICY);
        let expired = &db.gifts["g_expired"];
        assert_eq!(expired.anonymised_at, Some(130));
        assert!(expired.giver_name.is_none() && expired.recipient_name.is_none());
        assert!(expired.message.is_none() && expired.sends.is_empty() && expired.cover.is_none());
        assert!(candidates.contains(&SHA.to_owned()) && candidates.contains(&"cd".repeat(32)));
        assert!(db.uploads.is_empty() && db.anonymous_upload_bytes() == 0);
        assert!(!db.blob_in_use(SHA), "the expired gift's file may be deleted");
        assert_eq!(db.gifts["g_cancelled"].anonymised_at, Some(130));
        assert!(named(&db, "g_live") && db.gifts["g_live"].anonymised_at.is_none());
        // Idempotent.
        assert!(db.purge(200, &POLICY).is_empty());
    }

    #[test]
    fn anonymous_uploads_are_dropped_sooner_than_partner_uploads() {
        let mut db = Db::default();
        let upload = |id: &str, partner: Option<&str>| UploadRecord {
            id: id.to_owned(),
            sha256: "ab".repeat(32),
            size_bytes: 1,
            media_type: "application/pdf".to_owned(),
            file_name: None,
            rights_basis: RightsBasis::OwnWork,
            partner: partner.map(str::to_owned),
            created_at: 0,
            gift_id: None,
        };
        db.uploads.insert("anon".to_owned(), upload("anon", None));
        db.uploads.insert("partner".to_owned(), upload("partner", Some("septante")));
        let policy = PurgePolicy { anonymous_upload_orphan_ttl_secs: 5, ..POLICY };
        let _ = db.purge(6, &policy);
        assert_eq!(db.uploads.keys().collect::<Vec<_>>(), ["partner"]);
    }

    #[test]
    fn signature_records_expire_on_their_own_clock() {
        let record = |expires_at| IdempotencyRecord {
            body_sha256: String::new(),
            created_at: 100,
            outcome: IdempotentOutcome::Response { status: 200, body: "{}".to_owned() },
            expires_at,
        };
        assert!(record(None).is_live(1099, 1000) && !record(None).is_live(1100, 1000));
        assert!(record(Some(701)).is_live(700, 1000) && !record(Some(701)).is_live(701, 1000));
    }

    #[test]
    fn a_cancelled_gift_no_longer_holds_its_file() {
        let mut db = Db::default();
        db.gifts.insert("g_1".to_owned(), gift("g_1"));
        assert!(db.blob_in_use(SHA));
        if let Some(record) = db.gifts.get_mut("g_1") {
            record.status = GiftStatus::Cancelled;
        }
        assert!(!db.blob_in_use(SHA));
    }

    #[test]
    fn expiry_and_reveal_are_applied_on_read() {
        let mut record = gift("g_1");
        record.expires_at = Some(200);
        record.reveal_at = Some(150);
        assert_eq!(record.status_at(199), GiftStatus::Created);
        assert_eq!(record.status_at(200), GiftStatus::Expired);
        assert!(!record.is_revealed_at(149));
        assert!(record.is_revealed_at(150));
        record.status = GiftStatus::Cancelled;
        assert_eq!(record.status_at(999), GiftStatus::Cancelled);
    }
}
