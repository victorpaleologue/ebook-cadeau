//! Records and their persistence: one JSON snapshot (`db.json`, replaced atomically on
//! every change) and the files under `files/<sha256>` — or both in memory for tests.
//!
//! Secrets are never stored in clear: sign-in links and sessions are kept as SHA-256 of
//! the token. Gift codes **are** kept (the rights holder needs them to print cards); they
//! are the partner's own data, as a store keeps its orders.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use ebc_epub::{FileKind, RightsBasis};
use serde::{Deserialize, Serialize};

/// A persistence failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// Reading or writing failed.
    #[error("storage I/O on {path}: {message}")]
    Io {
        /// The file.
        path: String,
        /// The operating system's message.
        message: String,
    },
    /// The snapshot cannot be parsed.
    #[error("the database is corrupt: {0}")]
    Corrupt(String),
    /// A blob name that is not a SHA-256.
    #[error("invalid file name")]
    InvalidBlobName,
}

/// What an account may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The operator: moderates notices; in `own` mode the only account.
    Operator,
    /// A rights holder (author, publisher, distributor).
    RightsHolder,
}

/// Natural person or organisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartyKind {
    /// A natural person.
    Person,
    /// A company or association.
    Organisation,
}

/// The identification data the LCEN asks hosts to keep about content providers (art. 6
/// II; décret 2021-1362 — the field list is **to confirm with counsel**).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Person or organisation.
    pub kind: PartyKind,
    /// Full name, or company name.
    pub name: String,
    /// For an organisation: the person acting for it.
    pub representative: Option<String>,
    /// Postal address.
    pub postal_address: String,
    /// Telephone number (optional).
    pub phone: Option<String>,
    /// Date of birth (`YYYY-MM-DD`, optional unless counsel says otherwise).
    pub birth_date: Option<String>,
    /// Place of birth (optional unless counsel says otherwise).
    pub birth_place: Option<String>,
    /// When it was last changed.
    pub updated_at: i64,
}

/// Account state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AccountStatus {
    /// Normal.
    Active,
    /// Repeat-infringer policy: no upload, no new gift (existing gifts keep working).
    Suspended {
        /// Since when.
        at: i64,
    },
    /// Closed by its holder: books removed, gifts cancelled; identity kept for the
    /// legal period, then erased.
    Closed {
        /// When.
        at: i64,
    },
    /// Repeat-infringer policy, recurrence after a suspension: books removed, gifts
    /// cancelled, no sign-in; identity kept for the legal period, then erased.
    Banned {
        /// When.
        at: i64,
    },
}

/// A rights holder's (or the operator's) account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// `a_` + 16 hex digits.
    pub id: String,
    /// Verified e-mail address (lower case); empty once erased.
    pub email: String,
    /// Operator or rights holder.
    pub role: Role,
    /// When the account was created (= first verified sign-in).
    pub created_at: i64,
    /// When the address was verified (the first sign-in link used).
    pub verified_at: i64,
    /// Language of mails.
    pub locale: String,
    /// LCEN identification data; required before the first upload.
    pub identity: Option<Identity>,
    /// State.
    pub status: AccountStatus,
    /// Upheld notices against this account's books.
    pub upheld_notices: u32,
    /// Upheld notices that found the rights declaration false.
    pub false_declarations: u32,
    /// When the identity and e-mail were erased (retention).
    #[serde(default)]
    pub erased_at: Option<i64>,
}

/// A single-use sign-in link (stored as the SHA-256 of its token).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginToken {
    /// Hex SHA-256 of the token.
    pub token_hash: String,
    /// The address it was sent to.
    pub email: String,
    /// Language of the page that asked.
    pub locale: String,
    /// Expiry.
    pub expires_at: i64,
}

/// A signed-in browser (stored as the SHA-256 of the cookie value).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Hex SHA-256 of the cookie value.
    pub token_hash: String,
    /// Whose.
    pub account_id: String,
    /// Expiry.
    pub expires_at: i64,
}

/// The rights declaration made with an upload (`docs/research/09` §3.3 item 2, item 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    /// Basis from the closed list.
    pub basis: RightsBasis,
    /// Licence id (open licence), author's year of death (public domain), licensor and
    /// contract reference (licensed distributor).
    pub details: String,
    /// Version of the declaration text accepted.
    pub text_version: String,
    /// When it was made.
    pub declared_at: i64,
    /// By which account.
    pub account_id: String,
    /// The declarant's name and e-mail at that time.
    pub declarant: String,
}

/// Why a book is no longer hosted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Removal {
    /// Deleted by its rights holder.
    Deleted,
    /// Removed after an upheld notice.
    TakenDown {
        /// The notice.
        notice_id: String,
    },
    /// Every gift was cancelled and the idle period passed.
    Expired,
    /// The account was closed.
    AccountClosed,
}

/// An uploaded book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    /// `b_` + 16 hex digits.
    pub id: String,
    /// Owner.
    pub account_id: String,
    /// SHA-256 of the file (its storage name).
    pub sha256: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// EPUB or PDF (sniffed).
    pub kind: FileKind,
    /// File name offered to devices.
    pub file_name: String,
    /// Title (declared).
    pub title: String,
    /// Author (declared).
    pub author: String,
    /// Declared ISBN-13 (digits).
    pub isbn13: Option<String>,
    /// ISBN-13s found in the file's package metadata.
    #[serde(default)]
    pub embedded_isbns: Vec<String>,
    /// Language of the book (BCP 47), when given.
    pub language: Option<String>,
    /// The rights declaration.
    pub declaration: Declaration,
    /// Upload time.
    pub created_at: i64,
    /// `None` while hosted.
    pub removed: Option<Removal>,
    /// When it stopped being hosted.
    pub removed_at: Option<i64>,
}

impl Book {
    /// Whether the file is hosted and may be delivered.
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.removed.is_none()
    }

    /// Every ISBN this book is known by (declared and embedded).
    pub fn isbns(&self) -> impl Iterator<Item = &str> {
        self.isbn13.iter().map(String::as_str).chain(self.embedded_isbns.iter().map(String::as_str))
    }
}

/// Where a gift stands at the core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum GiftState {
    /// Recorded here, not (yet) confirmed by the core: creation is retried with the same
    /// `Idempotency-Key`.
    Pending {
        /// The core's last refusal, if any.
        error: Option<String>,
    },
    /// Created at the core; the code works.
    Live,
    /// Cancelled here; the core is still to be told (retried by maintenance).
    CancelPending {
        /// Since when.
        at: i64,
    },
    /// Cancelled at the core: the code answers `410`.
    Cancelled {
        /// When.
        at: i64,
    },
}

/// One gift (one card) of a book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gift {
    /// `c_` + 16 hex digits; also the `order_ref` and `Idempotency-Key` at the core.
    pub id: String,
    /// The book.
    pub book_id: String,
    /// Its owner.
    pub account_id: String,
    /// The core's gift id (`g_…`), once created.
    pub core_id: Option<String>,
    /// The code as printed (`XXXX-XXXX-XXXX`), once created; blanked when cancelled.
    pub code: Option<String>,
    /// The claim URL, once created; blanked when cancelled.
    pub claim_url: Option<String>,
    /// Giver's name (optional).
    pub giver_name: Option<String>,
    /// Recipient's name (optional).
    pub recipient_name: Option<String>,
    /// Message (optional).
    pub message: Option<String>,
    /// Locale of the card and pages.
    pub locale: String,
    /// Reveal date.
    pub reveal_at: Option<i64>,
    /// Creation time.
    pub created_at: i64,
    /// State.
    pub state: GiftState,
    /// Download links handed to the core (one per claim).
    pub deliveries: u32,
    /// Last claim.
    pub last_delivery_at: Option<i64>,
}

impl Gift {
    /// Whether the code can still deliver.
    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self.state, GiftState::Live)
    }

    /// Whether the gift is cancelled (or about to be).
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        matches!(self.state, GiftState::Cancelled { .. } | GiftState::CancelPending { .. })
    }
}

/// What a notice is about (DSA art. 16(2)(a)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeCategory {
    /// Copyright or related rights.
    Copyright,
    /// Other illegal content.
    OtherIllegal,
}

/// The operator's decision on a notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum NoticeDecision {
    /// Not decided yet.
    Open,
    /// The content was removed.
    Upheld {
        /// When.
        at: i64,
        /// The book removed.
        book_id: String,
        /// Its owner.
        account_id: String,
        /// The statement of reasons sent to the owner (facts and grounds).
        statement: String,
        /// Whether the rights declaration was found false.
        false_declaration: bool,
        /// Whether the account was suspended as a result.
        suspended: bool,
    },
    /// The notice was not acted upon.
    Rejected {
        /// When.
        at: i64,
        /// Why (sent to the notifier).
        reason: String,
    },
}

/// A notice (DSA art. 16).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    /// `n_` + 16 hex digits.
    pub id: String,
    /// Received at.
    pub received_at: i64,
    /// Language of the form.
    pub locale: String,
    /// Copyright or other.
    pub category: NoticeCategory,
    /// The exact location as given (URL, book or gift id, code printed on a card).
    pub location: String,
    /// The book it was resolved to, when it could be.
    pub book_id: Option<String>,
    /// Why the content is illegal (art. 16(2)(a)).
    pub explanation: String,
    /// Notifier's name (art. 16(2)(c); empty under the exception).
    pub notifier_name: String,
    /// Notifier's e-mail (art. 16(2)(c); empty under the exception).
    pub notifier_email: String,
    /// The art. 16(2)(c) exception (offences of Directive 2011/93/EU arts 3–7).
    pub exception: bool,
    /// The good-faith statement (art. 16(2)(d)) was made.
    pub good_faith: bool,
    /// The decision.
    pub decision: NoticeDecision,
}

/// An internal complaint against a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appeal {
    /// `r_` + 16 hex digits.
    pub id: String,
    /// The contested notice decision.
    pub notice_id: String,
    /// Who contests.
    pub account_id: String,
    /// When.
    pub filed_at: i64,
    /// The uploader's arguments.
    pub text: String,
    /// `None` while open; `Some(true)` granted, `Some(false)` dismissed.
    pub granted: Option<bool>,
    /// The operator's answer.
    pub answer: Option<String>,
    /// When decided.
    pub decided_at: Option<i64>,
}

/// What a blocklist entry matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    /// SHA-256 of a file.
    Sha256,
    /// ISBN-13.
    Isbn,
}

/// A blocked file or ISBN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockEntry {
    /// Kind.
    pub kind: BlockKind,
    /// The hash or the ISBN-13 digits.
    pub value: String,
    /// Since when.
    pub added_at: i64,
    /// The upheld notice behind it.
    pub notice_id: String,
}

/// One content-creation event, kept for the LCEN identification duty (décret 2021-1362,
/// as recalled: connection identifier, content identifier, operation, time, account).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentEvent {
    /// When.
    pub at: i64,
    /// Who.
    pub account_id: String,
    /// The book or gift id.
    pub content_id: String,
    /// `upload`, `delete`, `gift`.
    pub operation: String,
    /// IP address of the connection; erased after the retention period.
    pub connection: Option<String>,
}

/// One download of a gift's file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryEvent {
    /// When the file was served.
    pub at: i64,
    /// Which gift.
    pub gift_id: String,
    /// HMAC of the client's network (IPv4 /24, IPv6 /56) under the shelf secret — a
    /// pseudonym, never the address; erased after the security-log period.
    pub client: Option<String>,
}

/// The whole data set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Db {
    /// Accounts.
    #[serde(default)]
    pub accounts: Vec<Account>,
    /// Pending sign-in links.
    #[serde(default)]
    pub login_tokens: Vec<LoginToken>,
    /// Sessions.
    #[serde(default)]
    pub sessions: Vec<Session>,
    /// Books (live and removed; removed ones keep their declaration for the retention
    /// period).
    #[serde(default)]
    pub books: Vec<Book>,
    /// Gifts.
    #[serde(default)]
    pub gifts: Vec<Gift>,
    /// Notices.
    #[serde(default)]
    pub notices: Vec<Notice>,
    /// Internal complaints.
    #[serde(default)]
    pub appeals: Vec<Appeal>,
    /// Blocked hashes and ISBNs.
    #[serde(default)]
    pub blocklist: Vec<BlockEntry>,
    /// LCEN content-creation log.
    #[serde(default)]
    pub content_log: Vec<ContentEvent>,
    /// Delivery log.
    #[serde(default)]
    pub deliveries: Vec<DeliveryEvent>,
}

impl Db {
    /// The account with this id.
    #[must_use]
    pub fn account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// The account with this id, mutable.
    pub fn account_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// The book with this id.
    #[must_use]
    pub fn book(&self, id: &str) -> Option<&Book> {
        self.books.iter().find(|b| b.id == id)
    }

    /// The gift with this local id.
    #[must_use]
    pub fn gift(&self, id: &str) -> Option<&Gift> {
        self.gifts.iter().find(|g| g.id == id)
    }

    /// Whether a hash or ISBN is blocked.
    #[must_use]
    pub fn blocked(&self, kind: BlockKind, value: &str) -> Option<&BlockEntry> {
        self.blocklist.iter().find(|e| e.kind == kind && e.value == value)
    }

    /// Bytes of live files of an account.
    #[must_use]
    pub fn used_bytes(&self, account_id: Option<&str>) -> u64 {
        // A file uploaded twice by the same account is stored once but counted twice:
        // simple, and errs on the safe side.
        self.books
            .iter()
            .filter(|b| b.is_live() && account_id.is_none_or(|id| b.account_id == id))
            .map(|b| b.size_bytes)
            .sum()
    }
}

/// A stored file.
#[derive(Debug, Clone)]
pub enum Blob {
    /// In memory (tests).
    Memory(Arc<Vec<u8>>),
    /// On disk.
    File {
        /// Where.
        path: PathBuf,
        /// Length.
        len: u64,
    },
}

impl Blob {
    /// Length in bytes.
    #[must_use]
    pub fn len(&self) -> u64 {
        match self {
            Self::Memory(bytes) => u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            Self::File { len, .. } => *len,
        }
    }

    /// Whether it is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Where records and files live.
pub trait Store: Send + Sync + 'static {
    /// Loads the data set (empty on first start).
    ///
    /// # Errors
    /// When the snapshot is unreadable or corrupt.
    fn load(&self) -> Result<Db, StoreError>;

    /// Persists the data set.
    ///
    /// # Errors
    /// When it cannot be written.
    fn save(&self, db: &Db) -> Result<(), StoreError>;

    /// Stores a file under its SHA-256. Idempotent.
    ///
    /// # Errors
    /// [`StoreError::InvalidBlobName`] or an I/O failure.
    fn put_blob(&self, sha256: &str, bytes: &[u8]) -> Result<(), StoreError>;

    /// Opens a stored file.
    ///
    /// # Errors
    /// [`StoreError::InvalidBlobName`] or an I/O failure.
    fn open_blob(&self, sha256: &str) -> Result<Option<Blob>, StoreError>;

    /// Deletes a stored file (missing is fine).
    ///
    /// # Errors
    /// [`StoreError::InvalidBlobName`] or an I/O failure.
    fn delete_blob(&self, sha256: &str) -> Result<(), StoreError>;
}

fn check_blob_name(sha256: &str) -> Result<(), StoreError> {
    if sha256.len() == 64 && sha256.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        Ok(())
    } else {
        Err(StoreError::InvalidBlobName)
    }
}

/// Everything in memory (tests and demos).
#[derive(Debug, Default)]
pub struct MemoryStore {
    blobs: Mutex<HashMap<String, Arc<Vec<u8>>>>,
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn blobs(&self) -> MutexGuard<'_, HashMap<String, Arc<Vec<u8>>>> {
        self.blobs.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Hashes of the stored files (for tests).
    #[must_use]
    pub fn blob_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.blobs().keys().cloned().collect();
        names.sort();
        names
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
        self.blobs().entry(sha256.to_owned()).or_insert_with(|| Arc::new(bytes.to_vec()));
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

/// `db.json` and `files/<sha256>` under a directory.
#[derive(Debug)]
pub struct FileStore {
    dir: PathBuf,
}

fn io_error(path: &Path, error: &std::io::Error) -> StoreError {
    StoreError::Io { path: path.display().to_string(), message: error.to_string() }
}

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, bytes).map_err(|e| io_error(&temporary, &e))?;
    std::fs::rename(&temporary, path).map_err(|e| io_error(path, &e))
}

impl FileStore {
    /// Opens (creating if needed) the data directory.
    ///
    /// # Errors
    /// When the directories cannot be created.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let dir = dir.into();
        let files = dir.join("files");
        std::fs::create_dir_all(&files).map_err(|e| io_error(&files, &e))?;
        Ok(Self { dir })
    }

    /// The data directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn blob_path(&self, sha256: &str) -> Result<PathBuf, StoreError> {
        check_blob_name(sha256)?;
        Ok(self.dir.join("files").join(sha256))
    }
}

impl Store for FileStore {
    fn load(&self) -> Result<Db, StoreError> {
        let path = self.dir.join("db.json");
        match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| StoreError::Corrupt(format!("{}: {e}", path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Db::default()),
            Err(e) => Err(io_error(&path, &e)),
        }
    }

    fn save(&self, db: &Db) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec(db).map_err(|e| StoreError::Corrupt(e.to_string()))?;
        write_atomically(&self.dir.join("db.json"), &bytes)
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn file_store_round_trips_and_refuses_odd_names() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileStore::open(dir.path()).unwrap();
        assert_eq!(store.load().unwrap(), Db::default());
        let mut db = Db::default();
        db.blocklist.push(BlockEntry {
            kind: BlockKind::Isbn,
            value: "9782070360024".to_owned(),
            added_at: 1,
            notice_id: "n_1".to_owned(),
        });
        store.save(&db).unwrap();
        assert_eq!(store.load().unwrap(), db);
        let hash = crate::crypto::sha256_hex(b"book");
        store.put_blob(&hash, b"book").unwrap();
        assert_eq!(store.open_blob(&hash).unwrap().unwrap().len(), 4);
        store.delete_blob(&hash).unwrap();
        assert!(store.open_blob(&hash).unwrap().is_none());
        store.delete_blob(&hash).unwrap();
        for bad in ["../db.json", "", "ABC", &"g".repeat(64), &hash.to_uppercase()] {
            assert_eq!(store.put_blob(bad, b"x"), Err(StoreError::InvalidBlobName), "{bad}");
        }
        std::fs::write(dir.path().join("db.json"), b"{not json").unwrap();
        assert!(matches!(store.load(), Err(StoreError::Corrupt(_))));
    }
}
