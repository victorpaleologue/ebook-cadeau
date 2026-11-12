//! What the shelf does, independent of HTTP: sign-in links, the upload gate, gifts at the
//! core, the §4.5 download links, notice and action, appeals, the repeat-infringer
//! policy and retention. Handlers in `routes` only parse forms and render pages.

use ebc_epub::{FileKind, RightsBasis, UploadError};
use ebc_isbn::{Isbn10, Isbn13};
use ebc_model::{GiftCode, Locale};

use crate::clock::{civil_date, iso_date, iso_minute};
use crate::config::{Mode, normalise_email};
use crate::core::{BookPayload, Contributor, Delivery, NewGift};
use crate::crypto;
use crate::mail::OutgoingMail;
use crate::state::{AppState, Link, WriteError};
use crate::store::{
    Account, AccountStatus, Appeal, BlockEntry, BlockKind, Book, ContentEvent, Db, Declaration,
    DeliveryEvent, Gift, GiftState, LoginToken, Notice, NoticeCategory, NoticeDecision, Removal,
    Role, Session,
};
use crate::text::{S, s, sf};

/// Version of the rights declaration text (`S::DeclarationText`), recorded with every
/// upload.
pub const DECLARATION_VERSION: &str = "2026-11-12";

/// Open licences accepted (research 09 §3.5): id and display name.
pub const LICENCES: [(&str, &str); 7] = [
    ("cc0", "CC0 1.0"),
    ("cc-by", "CC BY 4.0"),
    ("cc-by-sa", "CC BY-SA 4.0"),
    ("cc-by-nd", "CC BY-ND 4.0"),
    ("cc-by-nc", "CC BY-NC 4.0"),
    ("cc-by-nc-sa", "CC BY-NC-SA 4.0"),
    ("cc-by-nc-nd", "CC BY-NC-ND 4.0"),
];

const DAY: i64 = 86_400;

/// Longest free texts.
const MAX_SHORT: usize = 300;
const MAX_MESSAGE: usize = 1_000;
const MAX_LONG: usize = 5_000;

/// Trims, drops control characters except new lines, bounds the length; `None` when
/// nothing is left or it is too long.
pub(crate) fn clean(text: &str, max_chars: usize, multiline: bool) -> Option<String> {
    let cleaned: String =
        text.trim().chars().filter(|c| !c.is_control() || (multiline && *c == '\n')).collect();
    let cleaned = cleaned.trim().to_owned();
    (!cleaned.is_empty() && cleaned.chars().count() <= max_chars).then_some(cleaned)
}

/// An optional text: empty is `Ok(None)`, too long is `Err(())`.
fn optional(text: &str, max_chars: usize, multiline: bool) -> Result<Option<String>, ()> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    clean(text, max_chars, multiline).map(Some).ok_or(())
}

/// The ISBN-13 (digits) of an ISBN-10 or -13 as typed.
pub(crate) fn parse_isbn(text: &str) -> Option<String> {
    Isbn13::parse(text)
        .ok()
        .or_else(|| Isbn10::parse(text).ok().map(|i| i.to_isbn13()))
        .map(|i| i.to_plain())
}

fn mb(locale: Locale, bytes: u64) -> String {
    if bytes < 1 << 20 {
        let kilobytes = bytes.div_ceil(1024).max(1).to_string();
        return sf(locale, S::Kilobytes, &[("n", &kilobytes)]);
    }
    let tenths = bytes.saturating_mul(10) / (1 << 20);
    let n = if tenths.is_multiple_of(10) {
        format!("{}", tenths / 10)
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    };
    sf(locale, S::Megabytes, &[("n", &n)])
}

/// Size in megabytes for display.
pub(crate) fn size_text(locale: Locale, bytes: u64) -> String {
    mb(locale, bytes)
}

// ---------------------------------------------------------------------------------------
// Sign-in
// ---------------------------------------------------------------------------------------

/// What happened to a sign-in request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LoginRequest {
    /// A link was sent (or the address may not sign in: the answer is the same).
    Sent,
    /// `own` mode, another address.
    Closed,
    /// Not an address.
    Invalid,
    /// The mail could not be sent.
    MailFailed,
    /// Rate limit: seconds to wait.
    TooMany(i64),
    /// Storage or randomness failed.
    Internal,
}

/// Why something failed for reasons outside the user's control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Internal(pub(crate) String);

fn internal<E: std::fmt::Display>(error: E) -> Internal {
    Internal(error.to_string())
}

impl<E: std::fmt::Debug> From<WriteError<E>> for Internal {
    fn from(error: WriteError<E>) -> Self {
        match error {
            WriteError::Refused(e) => Self(format!("refused: {e:?}")),
            WriteError::Storage(e) => Self(e.to_string()),
        }
    }
}

impl AppState {
    /// Whether `email` may hold an account here.
    fn may_sign_in(&self, email: &str) -> bool {
        self.config.mode == Mode::Multi || email == self.config.operator_email
    }

    /// Sends a sign-in link to `email_raw` (rate-limited per client network and per
    /// address). Answers [`LoginRequest::Sent`] whether or not the address has an
    /// account, so the page tells nobody which addresses exist.
    pub(crate) async fn request_login(
        &self,
        email_raw: &str,
        locale: Locale,
        client: &str,
    ) -> LoginRequest {
        let Some(email) = normalise_email(email_raw) else {
            return LoginRequest::Invalid;
        };
        let now = self.now();
        let limit = self.config.rate_login_per_hour;
        for key in [format!("login-net:{client}"), format!("login-mail:{email}")] {
            if let Err(wait) = self.counters.hit(&key, limit, 3_600, now) {
                return LoginRequest::TooMany(wait);
            }
        }
        if !self.may_sign_in(&email) {
            return LoginRequest::Closed;
        }
        let blocked = self.read(|db| {
            db.accounts.iter().any(|a| {
                a.email == email
                    && matches!(
                        a.status,
                        AccountStatus::Closed { .. } | AccountStatus::Banned { .. }
                    )
            })
        });
        if blocked {
            // A closed or banned account cannot come back under the same address.
            return LoginRequest::Sent;
        }
        let Ok(token) = self.random_hex(32) else {
            return LoginRequest::Internal;
        };
        let record = LoginToken {
            token_hash: crypto::sha256_hex(token.as_bytes()),
            email: email.clone(),
            locale: locale.code().to_owned(),
            expires_at: now + self.config.login_ttl_secs,
        };
        let stored = self.write(|db| {
            db.login_tokens.retain(|t| t.expires_at > now);
            if db.login_tokens.len() >= 10_000 {
                return Err(());
            }
            db.login_tokens.push(record);
            Ok(())
        });
        if stored.is_err() {
            return LoginRequest::Internal;
        }
        let minutes = (self.config.login_ttl_secs / 60).to_string();
        let shelf = self.config.name.as_str();
        let link = format!("{}/connexion/{token}", self.config.public_url);
        let mail = OutgoingMail {
            to: email,
            subject: sf(locale, S::MailLoginSubject, &[("shelf", shelf)]),
            text: sf(
                locale,
                S::MailLoginBody,
                &[("shelf", shelf), ("minutes", &minutes), ("link", &link)],
            ),
        };
        match self.mailer.send(mail, now).await {
            Ok(()) => LoginRequest::Sent,
            Err(error) => {
                tracing::warn!(%error, "sign-in mail not sent");
                LoginRequest::MailFailed
            }
        }
    }

    /// The pending sign-in link behind `token`, if it is still valid.
    pub(crate) fn login_token(&self, token: &str) -> Option<LoginToken> {
        let hash = crypto::sha256_hex(token.as_bytes());
        let now = self.now();
        self.read(|db| {
            db.login_tokens.iter().find(|t| t.token_hash == hash && t.expires_at > now).cloned()
        })
    }

    /// Uses a sign-in link (once): creates the account on first use, opens a session.
    /// Returns the session cookie value.
    pub(crate) fn consume_login(&self, token: &str) -> Result<Option<String>, Internal> {
        let hash = crypto::sha256_hex(token.as_bytes());
        let now = self.now();
        let session = self.random_hex(32).map_err(internal)?;
        let account_id = self.new_id("a_").map_err(internal)?;
        let session_hash = crypto::sha256_hex(session.as_bytes());
        let ttl = self.config.session_ttl_secs;
        let operator = self.config.operator_email.clone();
        let multi = self.config.mode == Mode::Multi;
        let result = self.write(|db| {
            let Some(position) =
                db.login_tokens.iter().position(|t| t.token_hash == hash && t.expires_at > now)
            else {
                return Err(());
            };
            let login = db.login_tokens.remove(position);
            if !multi && login.email != operator {
                return Err(());
            }
            let existing = db
                .accounts
                .iter()
                .find(|a| a.email == login.email)
                .map(|a| (a.id.clone(), a.status.clone()));
            let id = match existing {
                Some((_, AccountStatus::Closed { .. } | AccountStatus::Banned { .. })) => {
                    return Err(());
                }
                Some((id, _)) => id,
                None => {
                    let role =
                        if login.email == operator { Role::Operator } else { Role::RightsHolder };
                    db.accounts.push(Account {
                        id: account_id.clone(),
                        email: login.email.clone(),
                        role,
                        created_at: now,
                        verified_at: now,
                        locale: login.locale.clone(),
                        identity: None,
                        status: AccountStatus::Active,
                        upheld_notices: 0,
                        false_declarations: 0,
                        erased_at: None,
                    });
                    account_id.clone()
                }
            };
            db.sessions.retain(|s| s.expires_at > now);
            db.sessions.push(Session {
                token_hash: session_hash.clone(),
                account_id: id,
                expires_at: now + ttl,
            });
            Ok(())
        });
        match result {
            Ok(()) => Ok(Some(session)),
            Err(WriteError::Refused(())) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// The account of a session cookie value.
    pub(crate) fn session_account(&self, session: &str) -> Option<Account> {
        let hash = crypto::sha256_hex(session.as_bytes());
        let now = self.now();
        self.read(|db| {
            let session =
                db.sessions.iter().find(|s| s.token_hash == hash && s.expires_at > now)?;
            db.account(&session.account_id).cloned()
        })
    }

    /// Ends a session.
    pub(crate) fn sign_out(&self, session: &str) {
        let hash = crypto::sha256_hex(session.as_bytes());
        let _ = self.write(|db| {
            db.sessions.retain(|s| s.token_hash != hash);
            Ok::<(), ()>(())
        });
    }

    /// The form token of a session (`csrf` field of every signed-in form).
    pub(crate) fn csrf_token(&self, session: &str) -> String {
        crypto::hex(&crypto::hmac_sha256(
            self.config.secret.as_bytes(),
            format!("csrf.{session}").as_bytes(),
        ))
    }

    /// Records the identification data.
    pub(crate) fn save_identity(
        &self,
        account_id: &str,
        identity: crate::store::Identity,
    ) -> Result<(), Internal> {
        self.write(|db| {
            let account = db.account_mut(account_id).ok_or(())?;
            account.identity = Some(identity);
            Ok::<(), ()>(())
        })
        .map_err(Into::into)
    }
}

// ---------------------------------------------------------------------------------------
// Upload gate
// ---------------------------------------------------------------------------------------

/// The upload form.
#[derive(Debug, Clone, Default)]
pub(crate) struct UploadForm {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) isbn: String,
    pub(crate) language: String,
    pub(crate) basis: String,
    pub(crate) death_year: String,
    pub(crate) licence: String,
    pub(crate) licensor: String,
    pub(crate) declared: bool,
    pub(crate) file_name: String,
    pub(crate) bytes: Vec<u8>,
}

/// Why an upload is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UploadRefusal {
    CannotUpload,
    TooMany(i64),
    Title,
    Declaration,
    NotPublicDomain(i64),
    Licence,
    Licensor,
    Isbn,
    NoFile,
    TooLarge,
    Type,
    Damaged,
    Drm,
    BlockedFile,
    BlockedIsbn(String),
    QuotaAccount,
    QuotaTotal,
    Internal,
}

impl UploadRefusal {
    /// The sentence shown to the uploader.
    pub(crate) fn message(&self, state: &AppState, locale: Locale, account_id: &str) -> String {
        let config = &state.config;
        match self {
            Self::CannotUpload => s(locale, S::ErrCannotUpload).to_owned(),
            Self::TooMany(wait) => {
                sf(locale, S::TooMany, &[("minutes", &(wait / 60 + 1).to_string())])
            }
            Self::Title => s(locale, S::ErrTitle).to_owned(),
            Self::Declaration => s(locale, S::ErrDeclaration).to_owned(),
            Self::NotPublicDomain(year) => {
                sf(locale, S::ErrNotPublicDomain, &[("year", &year.to_string())])
            }
            Self::Licence => s(locale, S::ErrLicence).to_owned(),
            Self::Licensor => s(locale, S::ErrLicensor).to_owned(),
            Self::Isbn => s(locale, S::ErrIsbn).to_owned(),
            Self::NoFile => s(locale, S::ErrNoFile).to_owned(),
            Self::TooLarge => {
                sf(locale, S::ErrTooLarge, &[("max", &mb(locale, config.max_file_bytes))])
            }
            Self::Type => s(locale, S::ErrType).to_owned(),
            Self::Damaged => s(locale, S::ErrDamaged).to_owned(),
            Self::Drm => s(locale, S::ErrDrm).to_owned(),
            Self::BlockedFile => s(locale, S::ErrBlockedFile).to_owned(),
            Self::BlockedIsbn(isbn) => sf(locale, S::ErrBlockedIsbn, &[("isbn", isbn)]),
            Self::QuotaAccount => {
                let _ = account_id;
                sf(
                    locale,
                    S::ErrQuotaAccount,
                    &[("quota", &mb(locale, config.account_quota_bytes))],
                )
            }
            Self::QuotaTotal => s(locale, S::ErrQuotaTotal).to_owned(),
            Self::Internal => s(locale, S::ServerError).to_owned(),
        }
    }

    /// The machine-readable code (logs and tests).
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::CannotUpload => "cannot_upload",
            Self::TooMany(_) => "too_many_requests",
            Self::Title => "title_required",
            Self::Declaration => "declaration_required",
            Self::NotPublicDomain(_) => "not_public_domain",
            Self::Licence => "licence_required",
            Self::Licensor => "licensor_required",
            Self::Isbn => "invalid_isbn",
            Self::NoFile => "file_required",
            Self::TooLarge => "file_too_large",
            Self::Type => "unsupported_type",
            Self::Damaged => "invalid_epub",
            Self::Drm => "drm_protected",
            Self::BlockedFile => "blocked_file",
            Self::BlockedIsbn(_) => "blocked_isbn",
            Self::QuotaAccount => "account_quota_exceeded",
            Self::QuotaTotal => "storage_full",
            Self::Internal => "internal",
        }
    }
}

/// Whether an account may upload and create gifts.
pub(crate) fn may_publish(account: &Account) -> bool {
    matches!(account.status, AccountStatus::Active) && account.identity.is_some()
}

impl AppState {
    /// The upload gate (research 09 §3): rights declaration, ISBN, type sniffing and DRM
    /// refusal through `ebc-epub`, blocklist by hash and ISBN, quotas; then the file is
    /// stored under its hash and the book recorded with its declaration.
    pub(crate) fn upload(
        &self,
        account: &Account,
        form: UploadForm,
        connection: Option<String>,
    ) -> Result<Book, UploadRefusal> {
        let now = self.now();
        if !may_publish(account) {
            return Err(UploadRefusal::CannotUpload);
        }
        self.counters
            .hit(&format!("upload:{}", account.id), self.config.rate_uploads_per_hour, 3_600, now)
            .map_err(UploadRefusal::TooMany)?;
        let (Some(title), Some(author)) =
            (clean(&form.title, MAX_SHORT, false), clean(&form.author, MAX_SHORT, false))
        else {
            return Err(UploadRefusal::Title);
        };
        let basis = RightsBasis::parse(&form.basis).ok_or(UploadRefusal::Declaration)?;
        if !form.declared {
            return Err(UploadRefusal::Declaration);
        }
        let details = match basis {
            RightsBasis::OwnWork => String::new(),
            RightsBasis::PublicDomain => {
                let (year_now, _, _) = civil_date(now);
                // Life + 70 years, counted from 1 January after the death (CPI L123-1):
                // a work is free on 1 January of (death + 71).
                let latest = year_now - 71;
                let year = form.death_year.trim();
                let parsed = (year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()))
                    .then(|| year.parse::<i64>().ok())
                    .flatten();
                match parsed {
                    Some(death) if death <= latest => format!("†{death}"),
                    _ => return Err(UploadRefusal::NotPublicDomain(latest + 1)),
                }
            }
            RightsBasis::OpenLicence => {
                let licence = LICENCES
                    .iter()
                    .find(|(id, _)| *id == form.licence.trim())
                    .ok_or(UploadRefusal::Licence)?;
                licence.1.to_owned()
            }
            RightsBasis::LicensedDistributor => {
                clean(&form.licensor, MAX_SHORT, false).ok_or(UploadRefusal::Licensor)?
            }
        };
        let isbn13 = match form.isbn.trim() {
            "" => None,
            text => Some(parse_isbn(text).ok_or(UploadRefusal::Isbn)?),
        };
        let language = clean(&form.language, 35, false)
            .filter(|l| l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
            .map(|l| l.to_ascii_lowercase());
        if form.bytes.is_empty() {
            return Err(UploadRefusal::NoFile);
        }
        let kind =
            ebc_epub::check_upload(&form.bytes, self.config.max_file_bytes).map_err(|error| {
                match error {
                    UploadError::TooLarge { .. } => UploadRefusal::TooLarge,
                    UploadError::DrmProtected(_) => UploadRefusal::Drm,
                    UploadError::InvalidEpub(_) => UploadRefusal::Damaged,
                    UploadError::Empty => UploadRefusal::NoFile,
                    _ => UploadRefusal::Type,
                }
            })?;
        let sha256 = crypto::sha256_hex(&form.bytes);
        let mut embedded: Vec<String> = Vec::new();
        if kind == FileKind::Epub
            && let Some(meta) = ebc_epub::package_metadata(&form.bytes)
        {
            for identifier in &meta.identifiers {
                for isbn in ebc_isbn::extract_all(identifier) {
                    let plain = isbn.to_plain();
                    if !embedded.contains(&plain) {
                        embedded.push(plain);
                    }
                }
            }
        }
        let size = u64::try_from(form.bytes.len()).unwrap_or(u64::MAX);
        let file_name = file_name(&form.file_name, &title, kind);
        let book_id = self.new_id("b_").map_err(|_| UploadRefusal::Internal)?;
        let identity = account.identity.as_ref().map_or(String::new(), |i| i.name.clone());
        let book = Book {
            id: book_id,
            account_id: account.id.clone(),
            sha256: sha256.clone(),
            size_bytes: size,
            kind,
            file_name,
            title,
            author,
            isbn13: isbn13.clone(),
            embedded_isbns: embedded.clone(),
            language,
            declaration: Declaration {
                basis,
                details,
                text_version: DECLARATION_VERSION.to_owned(),
                declared_at: now,
                account_id: account.id.clone(),
                declarant: format!("{identity} <{}>", account.email),
            },
            created_at: now,
            removed: None,
            removed_at: None,
        };
        // Checks against the records, then the file, then the record: the blocklist and
        // the quotas are read again inside the write so that two uploads cannot both pass.
        let check = |db: &Db| -> Result<(), UploadRefusal> {
            if db.blocked(BlockKind::Sha256, &sha256).is_some() {
                return Err(UploadRefusal::BlockedFile);
            }
            for isbn in isbn13.iter().chain(embedded.iter()) {
                if db.blocked(BlockKind::Isbn, isbn).is_some() {
                    return Err(UploadRefusal::BlockedIsbn(isbn.clone()));
                }
            }
            if db.used_bytes(Some(&account.id)).saturating_add(size)
                > self.config.account_quota_bytes
            {
                return Err(UploadRefusal::QuotaAccount);
            }
            if db.used_bytes(None).saturating_add(size) > self.config.total_quota_bytes {
                return Err(UploadRefusal::QuotaTotal);
            }
            Ok(())
        };
        self.read(|db| check(db))?;
        self.store.put_blob(&sha256, &form.bytes).map_err(|error| {
            tracing::error!(%error, "cannot store an uploaded file");
            UploadRefusal::Internal
        })?;
        let event = ContentEvent {
            at: now,
            account_id: account.id.clone(),
            content_id: book.id.clone(),
            operation: "upload".to_owned(),
            connection,
        };
        let stored = book.clone();
        let result = self.write(|db| {
            check(db)?;
            db.books.push(stored);
            db.content_log.push(event);
            Ok(())
        });
        match result {
            Ok(()) => Ok(book),
            Err(WriteError::Refused(refusal)) => {
                self.delete_blob_if_unused(&sha256);
                Err(refusal)
            }
            Err(WriteError::Storage(error)) => {
                tracing::error!(%error, "cannot record an upload");
                self.delete_blob_if_unused(&sha256);
                Err(UploadRefusal::Internal)
            }
        }
    }

    /// Deletes a stored file unless a hosted book still uses it.
    pub(crate) fn delete_blob_if_unused(&self, sha256: &str) {
        let used = self.read(|db| db.books.iter().any(|b| b.is_live() && b.sha256 == sha256));
        if !used && let Err(error) = self.store.delete_blob(sha256) {
            tracing::error!(%error, "cannot delete a stored file");
        }
    }
}

/// The file name offered to devices: the uploaded name made safe, or one from the title,
/// with the sniffed extension.
fn file_name(uploaded: &str, title: &str, kind: FileKind) -> String {
    let encoded =
        percent_encoding::utf8_percent_encode(uploaded, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    ebc_epub::decode_file_name(&encoded, kind)
        .or_else(|| {
            let encoded =
                percent_encoding::utf8_percent_encode(title, percent_encoding::NON_ALPHANUMERIC)
                    .to_string();
            ebc_epub::decode_file_name(&format!("{encoded}.x"), kind)
        })
        .unwrap_or_else(|| format!("livre.{}", kind.extension()))
}

// ---------------------------------------------------------------------------------------
// Gifts
// ---------------------------------------------------------------------------------------

/// The gift form.
#[derive(Debug, Clone)]
pub(crate) struct GiftForm {
    pub(crate) count: u32,
    pub(crate) giver: Option<String>,
    pub(crate) recipient: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) locale: Locale,
    pub(crate) reveal_at: Option<i64>,
}

impl GiftForm {
    /// Parses and bounds the form fields; `None` when something is out of range.
    pub(crate) fn parse(
        count: &str,
        giver: &str,
        recipient: &str,
        message: &str,
        locale: &str,
        reveal: &str,
        max_count: u32,
    ) -> Option<Self> {
        let count = match count.trim() {
            "" => 1,
            text => text.parse::<u32>().ok()?,
        };
        if count == 0 || count > max_count {
            return None;
        }
        let reveal_at = match reveal.trim() {
            "" => None,
            text => Some(crate::clock::parse_iso_date(text)?),
        };
        Some(Self {
            count,
            giver: optional(giver, 80, false).ok()?,
            recipient: optional(recipient, 80, false).ok()?,
            message: optional(message, MAX_MESSAGE, true).ok()?,
            locale: Locale::from_tag(locale.trim())?,
            reveal_at,
        })
    }
}

/// Why a gift action failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GiftError {
    /// Not the owner's, or the book is gone, or the account may not publish.
    Forbidden,
    /// The core refused or did not answer (short description).
    Core(String),
    /// Storage or randomness.
    Internal,
}

impl AppState {
    /// Creates `form.count` gifts of a book at the core (`partner_file`, pointing at the
    /// §4.5 endpoint). Each is recorded first as pending, so a failure can be retried with
    /// the same `Idempotency-Key`. Returns the core's first refusal, if any.
    pub(crate) async fn create_gifts(
        &self,
        account: &Account,
        book_id: &str,
        form: &GiftForm,
        connection: Option<String>,
    ) -> Result<(), GiftError> {
        if !may_publish(account) {
            return Err(GiftError::Forbidden);
        }
        let now = self.now();
        let mut ids = Vec::new();
        for _ in 0..form.count {
            ids.push(self.new_id("c_").map_err(|_| GiftError::Internal)?);
        }
        let created = ids.clone();
        let result = self.write(|db| {
            let book =
                db.book(book_id).filter(|b| b.is_live() && b.account_id == account.id).ok_or(())?;
            let book_id = book.id.clone();
            for id in created {
                db.content_log.push(ContentEvent {
                    at: now,
                    account_id: account.id.clone(),
                    content_id: id.clone(),
                    operation: "gift".to_owned(),
                    connection: connection.clone(),
                });
                db.gifts.push(Gift {
                    id,
                    book_id: book_id.clone(),
                    account_id: account.id.clone(),
                    core_id: None,
                    code: None,
                    claim_url: None,
                    giver_name: form.giver.clone(),
                    recipient_name: form.recipient.clone(),
                    message: form.message.clone(),
                    locale: form.locale.code().to_owned(),
                    reveal_at: form.reveal_at,
                    created_at: now,
                    state: GiftState::Pending { error: None },
                    deliveries: 0,
                    last_delivery_at: None,
                });
            }
            Ok(())
        });
        match result {
            Ok(()) => {}
            Err(WriteError::Refused(())) => return Err(GiftError::Forbidden),
            Err(WriteError::Storage(_)) => return Err(GiftError::Internal),
        }
        let mut first_error = None;
        for id in &ids {
            if let Err(error) = self.push_gift(id).await
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    /// Sends a pending gift to the core (creation, or retry with the same key).
    pub(crate) async fn push_gift(&self, gift_id: &str) -> Result<(), GiftError> {
        let Some((gift, book)) = self.read(|db| {
            let gift = db.gift(gift_id)?.clone();
            let book = db.book(&gift.book_id)?.clone();
            Some((gift, book))
        }) else {
            return Err(GiftError::Forbidden);
        };
        if !matches!(gift.state, GiftState::Pending { .. }) || !book.is_live() {
            return Ok(());
        }
        let request = NewGift {
            order_ref: gift.id.clone(),
            book: BookPayload {
                title: book.title.clone(),
                isbn13: book.isbn13.clone(),
                contributors: vec![Contributor { name: book.author.clone() }],
                language: book.language.clone(),
            },
            giver_name: gift.giver_name.clone(),
            recipient_name: gift.recipient_name.clone(),
            message: gift.message.clone(),
            locale: gift.locale.clone(),
            reveal_at: gift.reveal_at,
            delivery: Delivery {
                kind: "partner_file",
                format: book.kind.extension(),
                url: self.config.link_endpoint(),
            },
        };
        match self.core().create_gift(&request, self.now()).await {
            Ok(issued) => {
                let _ = self.write(|db| {
                    let gift = db.gifts.iter_mut().find(|g| g.id == gift_id).ok_or(())?;
                    if matches!(gift.state, GiftState::Pending { .. }) {
                        gift.core_id = issued.id.clone();
                        gift.code = Some(issued.code_display.clone());
                        gift.claim_url = Some(issued.claim_url.clone());
                        gift.state = GiftState::Live;
                    }
                    Ok::<(), ()>(())
                });
                Ok(())
            }
            Err(error) => {
                tracing::warn!(gift = gift_id, %error, "gift creation at the core failed");
                let detail = match &error {
                    crate::core::CoreError::Api { code, .. } => code.clone(),
                    crate::core::CoreError::Transport(kind) => kind.clone(),
                    crate::core::CoreError::Decode(_) => "unexpected_answer".to_owned(),
                };
                let stored = detail.clone();
                let _ = self.write(|db| {
                    let gift = db.gifts.iter_mut().find(|g| g.id == gift_id).ok_or(())?;
                    gift.state = GiftState::Pending { error: Some(stored) };
                    Ok::<(), ()>(())
                });
                Err(GiftError::Core(detail))
            }
        }
    }

    /// The owner cancels a gift: its code answers `410` once the core is told (retried
    /// by maintenance if the core does not answer now). Names and message are deleted.
    pub(crate) async fn cancel_gift(
        &self,
        account: &Account,
        gift_id: &str,
    ) -> Result<(), GiftError> {
        let now = self.now();
        let pushed = self
            .write(|db| {
                let gift = db
                    .gifts
                    .iter_mut()
                    .find(|g| g.id == gift_id && g.account_id == account.id)
                    .ok_or(())?;
                Ok(cancel_record(gift, now))
            })
            .map_err(|e| match e {
                WriteError::Refused(()) => GiftError::Forbidden,
                WriteError::Storage(_) => GiftError::Internal,
            })?;
        if pushed {
            self.push_cancel(gift_id).await?;
        }
        Ok(())
    }

    /// Tells the core about a cancellation.
    pub(crate) async fn push_cancel(&self, gift_id: &str) -> Result<(), GiftError> {
        let Some(core_id) = self.read(|db| {
            db.gift(gift_id)
                .filter(|g| matches!(g.state, GiftState::CancelPending { .. }))
                .and_then(|g| g.core_id.clone())
        }) else {
            return Ok(());
        };
        match self.core().cancel(&core_id, self.now()).await {
            Ok(()) => {
                let now = self.now();
                let _ = self.write(|db| {
                    if let Some(gift) = db.gifts.iter_mut().find(|g| g.id == gift_id) {
                        gift.state = GiftState::Cancelled { at: now };
                    }
                    Ok::<(), ()>(())
                });
                Ok(())
            }
            Err(error) => {
                tracing::warn!(gift = gift_id, %error, "cancellation at the core failed; will retry");
                Err(GiftError::Core(error.to_string()))
            }
        }
    }

    /// A new code for a lost card.
    pub(crate) async fn reissue_gift(
        &self,
        account: &Account,
        gift_id: &str,
    ) -> Result<(), GiftError> {
        let core_id = self
            .read(|db| {
                db.gift(gift_id)
                    .filter(|g| g.account_id == account.id && g.is_live())
                    .and_then(|g| g.core_id.clone())
            })
            .ok_or(GiftError::Forbidden)?;
        let nonce = self.random_hex(8).map_err(|_| GiftError::Internal)?;
        let issued = self
            .core()
            .reissue(&core_id, &nonce, self.now())
            .await
            .map_err(|e| GiftError::Core(e.to_string()))?;
        self.write(|db| {
            let gift = db.gifts.iter_mut().find(|g| g.id == gift_id).ok_or(())?;
            gift.code = Some(issued.code_display.clone());
            gift.claim_url = Some(issued.claim_url.clone());
            Ok::<(), ()>(())
        })
        .map_err(|_| GiftError::Internal)
    }

    /// The owner deletes a book: the file is erased, every gift cancelled.
    pub(crate) async fn delete_book(
        &self,
        account: &Account,
        book_id: &str,
        connection: Option<String>,
    ) -> Result<(), GiftError> {
        let now = self.now();
        let (sha, cancels) = self
            .write(|db| {
                let book = db
                    .books
                    .iter_mut()
                    .find(|b| b.id == book_id && b.account_id == account.id && b.is_live())
                    .ok_or(())?;
                book.removed = Some(Removal::Deleted);
                book.removed_at = Some(now);
                let sha = book.sha256.clone();
                db.content_log.push(ContentEvent {
                    at: now,
                    account_id: account.id.clone(),
                    content_id: book_id.to_owned(),
                    operation: "delete".to_owned(),
                    connection,
                });
                let cancels = cancel_gifts_of(db, &[book_id.to_owned()], now);
                Ok((sha, cancels))
            })
            .map_err(|e| match e {
                WriteError::Refused(()) => GiftError::Forbidden,
                WriteError::Storage(_) => GiftError::Internal,
            })?;
        self.drop_links_of(&[book_id.to_owned()]);
        self.delete_blob_if_unused(&sha);
        for gift in cancels {
            let _ = self.push_cancel(&gift).await;
        }
        Ok(())
    }

    /// The holder closes their account: books erased, gifts cancelled, sessions ended.
    pub(crate) async fn close_account(&self, account: &Account) -> Result<(), Internal> {
        let now = self.now();
        let (hashes, cancels) = self.write(|db| {
            let (hashes, cancels) = remove_all_books(db, &account.id, now);
            if let Some(a) = db.account_mut(&account.id) {
                a.status = AccountStatus::Closed { at: now };
            }
            db.sessions.retain(|s| s.account_id != account.id);
            Ok::<_, ()>((hashes, cancels))
        })?;
        self.finish_removals(&hashes, &cancels).await;
        Ok(())
    }

    async fn finish_removals(&self, hashes: &[(String, String)], cancels: &[String]) {
        let books: Vec<String> = hashes.iter().map(|(book, _)| book.clone()).collect();
        self.drop_links_of(&books);
        for (_, sha) in hashes {
            self.delete_blob_if_unused(sha);
        }
        for gift in cancels {
            let _ = self.push_cancel(gift).await;
        }
    }
}

/// Marks a gift cancelled (pending at the core when it exists there); blanks names,
/// message and code. Returns whether the core must be told.
fn cancel_record(gift: &mut Gift, now: i64) -> bool {
    let push = match gift.state {
        GiftState::Live => {
            gift.state = GiftState::CancelPending { at: now };
            true
        }
        GiftState::Pending { .. } => {
            // Never confirmed by the core; if it did create it, the §4.5 endpoint refuses
            // it anyway because the gift is cancelled here.
            gift.state = GiftState::Cancelled { at: now };
            false
        }
        GiftState::CancelPending { .. } => true,
        GiftState::Cancelled { .. } => false,
    };
    gift.giver_name = None;
    gift.recipient_name = None;
    gift.message = None;
    gift.code = None;
    gift.claim_url = None;
    push
}

/// Cancels every gift of these books; returns those the core must be told about.
fn cancel_gifts_of(db: &mut Db, book_ids: &[String], now: i64) -> Vec<String> {
    db.gifts
        .iter_mut()
        .filter(|g| book_ids.contains(&g.book_id))
        .filter_map(|g| cancel_record(g, now).then(|| g.id.clone()))
        .collect()
}

/// Removes every live book of an account; returns `(book, sha)` pairs and the gifts the
/// core must be told about.
fn remove_all_books(
    db: &mut Db,
    account_id: &str,
    now: i64,
) -> (Vec<(String, String)>, Vec<String>) {
    let mut removed = Vec::new();
    for book in db.books.iter_mut().filter(|b| b.account_id == account_id && b.is_live()) {
        book.removed = Some(Removal::AccountClosed);
        book.removed_at = Some(now);
        removed.push((book.id.clone(), book.sha256.clone()));
    }
    let ids: Vec<String> = removed.iter().map(|(id, _)| id.clone()).collect();
    let cancels = cancel_gifts_of(db, &ids, now);
    (removed, cancels)
}

// ---------------------------------------------------------------------------------------
// §4.5 download links
// ---------------------------------------------------------------------------------------

/// Why a link request is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkRefusal {
    Malformed,
    Stale,
    BadSignature,
    Unknown,
    Gone,
    Internal,
}

/// The JSON answer of §4.5.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct LinkAnswer {
    pub(crate) download_url: String,
    pub(crate) file_name: String,
    pub(crate) media_type: &'static str,
    pub(crate) size_bytes: u64,
    pub(crate) sha256: String,
    pub(crate) expires_at: i64,
}

impl AppState {
    /// Answers the core's signed link request for a claim: checks the signature and its
    /// timestamp, that the gift is ours, live, and its book hosted; mints a 128-bit
    /// download token valid `EBC_SHELF_LINK_TTL_SECS`; counts the delivery.
    pub(crate) fn issue_link(
        &self,
        gift: &str,
        order: &str,
        ts: &str,
        sig: &str,
    ) -> Result<LinkAnswer, LinkRefusal> {
        let now = self.now();
        crypto::verify_link_request(&self.config.partner_secret, gift, order, ts, sig, now)
            .map_err(|e| match e {
                crypto::SignatureError::Stale => LinkRefusal::Stale,
                crypto::SignatureError::Mismatch => LinkRefusal::BadSignature,
                _ => LinkRefusal::Malformed,
            })?;
        let (local, book) = self
            .read(|db| {
                let local = db
                    .gifts
                    .iter()
                    .find(|g| g.id == order && g.core_id.as_deref() == Some(gift))?
                    .clone();
                let book = db.book(&local.book_id)?.clone();
                Some((local, book))
            })
            .ok_or(LinkRefusal::Unknown)?;
        if !local.is_live() || !book.is_live() {
            return Err(LinkRefusal::Gone);
        }
        let blob = self
            .store
            .open_blob(&book.sha256)
            .map_err(|_| LinkRefusal::Internal)?
            .ok_or(LinkRefusal::Gone)?;
        let token = self.random_hex(16).map_err(|_| LinkRefusal::Internal)?;
        let expires_at = now + self.config.link_ttl_secs;
        self.write(|db| {
            let gift = db.gifts.iter_mut().find(|g| g.id == local.id).ok_or(())?;
            gift.deliveries = gift.deliveries.saturating_add(1);
            gift.last_delivery_at = Some(now);
            Ok::<(), ()>(())
        })
        .map_err(|_| LinkRefusal::Internal)?;
        self.add_link(
            crypto::sha256_hex(token.as_bytes()),
            Link {
                gift_id: local.id.clone(),
                book_id: book.id.clone(),
                sha256: book.sha256.clone(),
                file_name: book.file_name.clone(),
                media_type: book.kind.media_type(),
                expires_at,
            },
        );
        let encoded =
            percent_encoding::utf8_percent_encode(&book.file_name, crate::routes::PATH_SEGMENT)
                .to_string();
        Ok(LinkAnswer {
            download_url: format!("{}/f/{token}/{encoded}", self.config.public_url),
            file_name: book.file_name,
            media_type: book.kind.media_type(),
            size_bytes: blob.len(),
            sha256: book.sha256,
            expires_at,
        })
    }

    /// Records one download (the client only as a keyed pseudonym of its network).
    pub(crate) fn record_download(&self, gift_id: &str, client_network: &str) {
        let pseudonym = crypto::hex(&crypto::hmac_sha256(
            self.config.secret.as_bytes(),
            format!("client.{client_network}").as_bytes(),
        ))[..16]
            .to_owned();
        let now = self.now();
        let _ = self.write(|db| {
            if db.deliveries.len() < 1_000_000 {
                db.deliveries.push(DeliveryEvent {
                    at: now,
                    gift_id: gift_id.to_owned(),
                    client: Some(pseudonym),
                });
            }
            Ok::<(), ()>(())
        });
    }
}

// ---------------------------------------------------------------------------------------
// Notice and action
// ---------------------------------------------------------------------------------------

/// The notice form.
#[derive(Debug, Clone, Default)]
pub(crate) struct NoticeForm {
    pub(crate) category: String,
    pub(crate) location: String,
    pub(crate) explanation: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) exception: bool,
    pub(crate) good_faith: bool,
}

/// Why a notice was not recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NoticeError {
    Invalid,
    TooMany(i64),
    Internal,
}

/// A decision on a notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Decision {
    Uphold { book_id: String, statement: String, false_declaration: bool },
    Reject { reason: String },
}

/// Why a decision was not recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DecideError {
    Invalid,
    Internal,
}

impl AppState {
    /// Records a notice (art. 16), acknowledges it by mail (art. 16(4)).
    pub(crate) async fn file_notice(
        &self,
        form: NoticeForm,
        locale: Locale,
        client: &str,
    ) -> Result<String, NoticeError> {
        let now = self.now();
        self.counters
            .hit(&format!("notice:{client}"), self.config.rate_notices_per_hour, 3_600, now)
            .map_err(NoticeError::TooMany)?;
        let category = match form.category.as_str() {
            "copyright" => NoticeCategory::Copyright,
            "other" => NoticeCategory::OtherIllegal,
            _ => return Err(NoticeError::Invalid),
        };
        let location = clean(&form.location, 2_000, true).ok_or(NoticeError::Invalid)?;
        let explanation = clean(&form.explanation, MAX_LONG, true).ok_or(NoticeError::Invalid)?;
        if !form.good_faith {
            return Err(NoticeError::Invalid);
        }
        let name = optional(&form.name, 200, false).map_err(|()| NoticeError::Invalid)?;
        let email = match form.email.trim() {
            "" => None,
            text => Some(normalise_email(text).ok_or(NoticeError::Invalid)?),
        };
        if !form.exception && (name.is_none() || email.is_none()) {
            return Err(NoticeError::Invalid);
        }
        let id = self.new_id("n_").map_err(|_| NoticeError::Internal)?;
        let book_id = self.read(|db| resolve_location(db, &location));
        let notice = Notice {
            id: id.clone(),
            received_at: now,
            locale: locale.code().to_owned(),
            category,
            location: location.clone(),
            book_id,
            explanation,
            notifier_name: name.unwrap_or_default(),
            notifier_email: email.clone().unwrap_or_default(),
            exception: form.exception,
            good_faith: true,
            decision: NoticeDecision::Open,
        };
        self.write(|db| {
            if db.notices.iter().filter(|n| matches!(n.decision, NoticeDecision::Open)).count()
                >= 10_000
            {
                return Err(());
            }
            db.notices.push(notice);
            Ok(())
        })
        .map_err(|_| NoticeError::Internal)?;
        tracing::info!(notice = %id, "notice received");
        if let Some(to) = email {
            let shelf = self.config.name.as_str();
            let mail = OutgoingMail {
                to,
                subject: sf(locale, S::MailNoticeAckSubject, &[("id", &id), ("shelf", shelf)]),
                text: sf(
                    locale,
                    S::MailNoticeAckBody,
                    &[("id", &id), ("date", &iso_minute(now)), ("location", &location)],
                ),
            };
            if let Err(error) = self.mailer.send(mail, now).await {
                tracing::warn!(%error, "notice acknowledgement not sent");
            }
        }
        Ok(id)
    }

    /// The operator decides a notice. Upholding removes every hosted copy of the file,
    /// cancels their gifts at the core (their codes answer `410`), blocklists the hash and
    /// the ISBNs, counts a strike (suspension per the repeat-infringer policy), and mails
    /// the statement of reasons (art. 17) to the uploader and the decision to the notifier.
    pub(crate) async fn decide_notice(
        &self,
        notice_id: &str,
        decision: Decision,
    ) -> Result<(), DecideError> {
        let now = self.now();
        let upheld_limit = self.config.suspend_after_upheld;
        let false_limit = self.config.suspend_after_false_declarations;
        let outcome = self.write(|db| -> Result<DecisionEffects, DecideError> {
            let notice = db
                .notices
                .iter()
                .find(|n| n.id == notice_id)
                .cloned()
                .ok_or(DecideError::Invalid)?;
            if !matches!(notice.decision, NoticeDecision::Open) {
                return Err(DecideError::Invalid);
            }
            let mut effects =
                DecisionEffects { notice: notice.clone(), ..DecisionEffects::default() };
            match &decision {
                Decision::Reject { reason } => {
                    let reason = clean(reason, MAX_LONG, true).ok_or(DecideError::Invalid)?;
                    effects.rejected = Some(reason.clone());
                    if let Some(n) = db.notices.iter_mut().find(|n| n.id == notice_id) {
                        n.decision = NoticeDecision::Rejected { at: now, reason };
                    }
                }
                Decision::Uphold { book_id, statement, false_declaration } => {
                    let statement = clean(statement, MAX_LONG, true).ok_or(DecideError::Invalid)?;
                    let book = db
                        .book(book_id.trim())
                        .filter(|b| b.is_live())
                        .cloned()
                        .ok_or(DecideError::Invalid)?;
                    // Every hosted copy of the same file goes: the hash is blocked anyway.
                    let mut removed = Vec::new();
                    for copy in
                        db.books.iter_mut().filter(|b| b.is_live() && b.sha256 == book.sha256)
                    {
                        copy.removed = Some(Removal::TakenDown { notice_id: notice_id.to_owned() });
                        copy.removed_at = Some(now);
                        removed.push(copy.clone());
                    }
                    let ids: Vec<String> = removed.iter().map(|b| b.id.clone()).collect();
                    effects.cancels = cancel_gifts_of(db, &ids, now);
                    let mut block = |kind: BlockKind, value: &str| {
                        if db.blocked(kind, value).is_none() {
                            db.blocklist.push(BlockEntry {
                                kind,
                                value: value.to_owned(),
                                added_at: now,
                                notice_id: notice_id.to_owned(),
                            });
                        }
                    };
                    block(BlockKind::Sha256, &book.sha256);
                    for isbn in book.isbns() {
                        block(BlockKind::Isbn, isbn);
                    }
                    // The strike goes to the uploader of the noticed book.
                    let mut suspended = false;
                    if let Some(account) = db.account_mut(&book.account_id) {
                        account.upheld_notices += 1;
                        if *false_declaration {
                            account.false_declarations += 1;
                        }
                        let over = account.upheld_notices >= upheld_limit
                            || account.false_declarations >= false_limit;
                        match account.status {
                            AccountStatus::Active if over => {
                                account.status = AccountStatus::Suspended { at: now };
                                suspended = true;
                            }
                            AccountStatus::Suspended { .. } => {
                                // Recurrence after suspension: ban.
                                account.status = AccountStatus::Banned { at: now };
                                effects.banned = Some(book.account_id.clone());
                            }
                            _ => {}
                        }
                    }
                    if let Some(banned) = effects.banned.clone() {
                        let (more, cancels) = remove_all_books(db, &banned, now);
                        effects.extra_removed = more;
                        effects.cancels.extend(cancels);
                        db.sessions.retain(|s| s.account_id != banned);
                    }
                    effects.suspended = suspended;
                    effects.removed = removed;
                    effects.statement = statement.clone();
                    effects.false_declaration = *false_declaration;
                    effects.book = Some(book.clone());
                    effects.owners = db
                        .accounts
                        .iter()
                        .filter(|a| effects.removed.iter().any(|b| b.account_id == a.id))
                        .cloned()
                        .collect();
                    if let Some(n) = db.notices.iter_mut().find(|n| n.id == notice_id) {
                        n.decision = NoticeDecision::Upheld {
                            at: now,
                            book_id: book.id.clone(),
                            account_id: book.account_id.clone(),
                            statement,
                            false_declaration: *false_declaration,
                            suspended,
                        };
                        n.book_id = Some(book.id.clone());
                    }
                }
            }
            Ok(effects)
        });
        let effects = match outcome {
            Ok(effects) => effects,
            Err(WriteError::Refused(error)) => return Err(error),
            Err(WriteError::Storage(_)) => return Err(DecideError::Internal),
        };
        tracing::info!(notice = notice_id, upheld = effects.book.is_some(), "notice decided");
        let mut hashes: Vec<(String, String)> =
            effects.removed.iter().map(|b| (b.id.clone(), b.sha256.clone())).collect();
        hashes.extend(effects.extra_removed.iter().cloned());
        self.finish_removals(&hashes, &effects.cancels).await;
        self.mail_decision(&effects, now).await;
        Ok(())
    }

    async fn mail_decision(&self, effects: &DecisionEffects, now: i64) {
        let shelf = self.config.name.as_str();
        let notice = &effects.notice;
        let notice_locale = Locale::from_tag(&notice.locale).unwrap_or_default();
        if !notice.notifier_email.is_empty() {
            let mut text = match &effects.rejected {
                Some(reason) => sf(
                    notice_locale,
                    S::MailNoticeRejected,
                    &[("id", &notice.id), ("reason", reason)],
                ),
                None => sf(notice_locale, S::MailNoticeUpheld, &[("id", &notice.id)]),
            };
            text.push_str(s(notice_locale, S::MailNotifierRedress));
            let mail = OutgoingMail {
                to: notice.notifier_email.clone(),
                subject: sf(
                    notice_locale,
                    S::MailNoticeDecisionSubject,
                    &[("id", &notice.id), ("shelf", shelf)],
                ),
                text,
            };
            if let Err(error) = self.mailer.send(mail, now).await {
                tracing::warn!(%error, "decision not mailed to the notifier");
            }
        }
        let Some(noticed) = &effects.book else { return };
        for owner in &effects.owners {
            if owner.email.is_empty() {
                continue;
            }
            let locale = Locale::from_tag(&owner.locale).unwrap_or_default();
            for book in effects.removed.iter().filter(|b| b.account_id == owner.id) {
                let ground = match notice.category {
                    NoticeCategory::Copyright => s(locale, S::GroundCopyright),
                    NoticeCategory::OtherIllegal => s(locale, S::GroundOther),
                };
                let is_noticed = book.id == noticed.id;
                let false_text = if is_noticed && effects.false_declaration {
                    s(locale, S::FalseDeclaration)
                } else {
                    ""
                };
                let link = format!("{}/compte/decisions/{}", self.config.public_url, notice.id);
                let mut text = sf(
                    locale,
                    S::MailSorBody,
                    &[
                        ("title", &book.title),
                        ("book", &book.id),
                        ("shelf", shelf),
                        ("date", &iso_date(now)),
                        ("notice", &notice.id),
                        ("received", &iso_date(notice.received_at)),
                        ("statement", &effects.statement),
                        ("ground", ground),
                        ("false", false_text),
                        ("link", &link),
                    ],
                );
                if is_noticed && (effects.suspended || effects.banned.is_some()) {
                    text.push_str(&sf(
                        locale,
                        S::MailSuspended,
                        &[
                            ("upheld", &owner.upheld_notices.to_string()),
                            ("false", &owner.false_declarations.to_string()),
                        ],
                    ));
                }
                let mail = OutgoingMail {
                    to: owner.email.clone(),
                    subject: sf(
                        locale,
                        S::MailSorSubject,
                        &[("title", &book.title), ("shelf", shelf)],
                    ),
                    text,
                };
                if let Err(error) = self.mailer.send(mail, now).await {
                    tracing::warn!(%error, "statement of reasons not mailed");
                }
            }
        }
    }

    /// The uploader contests an upheld decision (internal complaint).
    pub(crate) fn file_appeal(
        &self,
        account: &Account,
        notice_id: &str,
        text: &str,
    ) -> Result<(), DecideError> {
        let now = self.now();
        let text = clean(text, MAX_LONG, true).ok_or(DecideError::Invalid)?;
        let id = self.new_id("r_").map_err(|_| DecideError::Internal)?;
        let window = self.config.appeal_days * DAY;
        self.write(|db| {
            let decided_at =
                match db.notices.iter().find(|n| n.id == notice_id).map(|n| &n.decision) {
                    Some(NoticeDecision::Upheld { at, account_id, .. })
                        if *account_id == account.id =>
                    {
                        *at
                    }
                    _ => return Err(DecideError::Invalid),
                };
            if now > decided_at + window || db.appeals.iter().any(|a| a.notice_id == notice_id) {
                return Err(DecideError::Invalid);
            }
            db.appeals.push(Appeal {
                id,
                notice_id: notice_id.to_owned(),
                account_id: account.id.clone(),
                filed_at: now,
                text,
                granted: None,
                answer: None,
                decided_at: None,
            });
            Ok(())
        })
        .map_err(|e| match e {
            WriteError::Refused(e) => e,
            WriteError::Storage(_) => DecideError::Internal,
        })
    }

    /// The operator answers a complaint. Granting it reverses the strike, lifts the
    /// blocklist entries of the decision and a suspension it caused.
    pub(crate) async fn decide_appeal(
        &self,
        appeal_id: &str,
        grant: bool,
        answer: &str,
    ) -> Result<(), DecideError> {
        let now = self.now();
        let answer = clean(answer, MAX_LONG, true).ok_or(DecideError::Invalid)?;
        let upheld_limit = self.config.suspend_after_upheld;
        let false_limit = self.config.suspend_after_false_declarations;
        let stored = answer.clone();
        let (account, notice_id) = self
            .write(|db| {
                let appeal = db
                    .appeals
                    .iter_mut()
                    .find(|a| a.id == appeal_id && a.granted.is_none())
                    .ok_or(DecideError::Invalid)?;
                appeal.granted = Some(grant);
                appeal.answer = Some(stored);
                appeal.decided_at = Some(now);
                let (account_id, notice_id) = (appeal.account_id.clone(), appeal.notice_id.clone());
                if grant {
                    let false_declaration = matches!(
                        db.notices.iter().find(|n| n.id == notice_id).map(|n| &n.decision),
                        Some(NoticeDecision::Upheld { false_declaration: true, .. })
                    );
                    db.blocklist.retain(|e| e.notice_id != notice_id);
                    if let Some(account) = db.account_mut(&account_id) {
                        account.upheld_notices = account.upheld_notices.saturating_sub(1);
                        if false_declaration {
                            account.false_declarations =
                                account.false_declarations.saturating_sub(1);
                        }
                        let over = account.upheld_notices >= upheld_limit
                            || account.false_declarations >= false_limit;
                        if matches!(account.status, AccountStatus::Suspended { .. }) && !over {
                            account.status = AccountStatus::Active;
                        }
                    }
                }
                Ok((db.account(&account_id).cloned(), notice_id))
            })
            .map_err(|e| match e {
                WriteError::Refused(e) => e,
                WriteError::Storage(_) => DecideError::Internal,
            })?;
        if let Some(account) = account.filter(|a| !a.email.is_empty()) {
            let locale = Locale::from_tag(&account.locale).unwrap_or_default();
            let body = if grant { S::MailAppealGranted } else { S::MailAppealDismissed };
            let mail = OutgoingMail {
                to: account.email.clone(),
                subject: sf(locale, S::MailAppealSubject, &[("shelf", &self.config.name)]),
                text: sf(locale, body, &[("notice", &notice_id), ("answer", &answer)]),
            };
            if let Err(error) = self.mailer.send(mail, now).await {
                tracing::warn!(%error, "appeal answer not mailed");
            }
        }
        Ok(())
    }
}

/// What a decision changed (computed inside the write, acted upon after it).
#[derive(Debug, Clone, Default)]
struct DecisionEffects {
    notice: Notice,
    rejected: Option<String>,
    book: Option<Book>,
    removed: Vec<Book>,
    extra_removed: Vec<(String, String)>,
    owners: Vec<Account>,
    cancels: Vec<String>,
    statement: String,
    false_declaration: bool,
    suspended: bool,
    banned: Option<String>,
}

impl Default for Notice {
    fn default() -> Self {
        Self {
            id: String::new(),
            received_at: 0,
            locale: String::new(),
            category: NoticeCategory::Copyright,
            location: String::new(),
            book_id: None,
            explanation: String::new(),
            notifier_name: String::new(),
            notifier_email: String::new(),
            exception: false,
            good_faith: false,
            decision: NoticeDecision::Open,
        }
    }
}

/// Finds the book a notice points at: a book id (`b_…`, also inside a shelf URL), a gift
/// id of the shelf (`c_…`) or of the core, or the code printed on a card (alone or in a
/// claim URL).
pub(crate) fn resolve_location(db: &Db, location: &str) -> Option<String> {
    let book_of_gift = |gift: &Gift| Some(gift.book_id.clone());
    let tokens = location
        .split(|c: char| {
            c.is_whitespace()
                || matches!(c, ',' | ';' | '(' | ')' | '<' | '>' | '"' | '\'' | '?' | '&' | '=')
        })
        .flat_map(|t| t.split('/'))
        .map(|t| t.trim_matches(|c: char| matches!(c, '.' | ':' | '#')))
        .filter(|t| !t.is_empty());
    for token in tokens {
        if let Some(book) = db.books.iter().find(|b| b.id == token) {
            return Some(book.id.clone());
        }
        if let Some(gift) =
            db.gifts.iter().find(|g| g.id == token || g.core_id.as_deref() == Some(token))
        {
            return book_of_gift(gift);
        }
        if let Ok(code) = GiftCode::parse(token)
            && let Some(gift) = db.gifts.iter().find(|g| {
                g.code.as_deref().and_then(|c| GiftCode::parse(c).ok()).is_some_and(|c| c == code)
            })
        {
            return book_of_gift(gift);
        }
    }
    // A code typed with spaces ("BCDF GHJK LMNP").
    if let Ok(code) = GiftCode::parse_typed(location) {
        return db
            .gifts
            .iter()
            .find(|g| {
                g.code.as_deref().and_then(|c| GiftCode::parse(c).ok()).is_some_and(|c| c == code)
            })
            .and_then(book_of_gift);
    }
    None
}

// ---------------------------------------------------------------------------------------
// Retention and maintenance
// ---------------------------------------------------------------------------------------

impl AppState {
    /// Applies the retention periods (`Config::retention`): expired sign-in links and
    /// sessions; connection data of content creation; download pseudonyms and old
    /// delivery records; files of books whose gifts were all cancelled; declarations of
    /// removed books; old decided notices; identity of closed accounts. Returns the
    /// number of records changed.
    pub fn purge(&self) -> usize {
        let now = self.now();
        let r = self.config.retention.clone();
        let result = self.write(|db| {
            let mut changed = 0usize;
            let before = db.login_tokens.len() + db.sessions.len();
            db.login_tokens.retain(|t| t.expires_at > now);
            db.sessions.retain(|s| s.expires_at > now);
            changed += before - db.login_tokens.len() - db.sessions.len();
            for event in &mut db.content_log {
                if event.connection.is_some() && event.at + r.connection_days * DAY <= now {
                    event.connection = None;
                    changed += 1;
                }
            }
            let before = db.content_log.len();
            db.content_log.retain(|e| e.at + r.declaration_days.max(r.connection_days) * DAY > now);
            changed += before - db.content_log.len();
            for event in &mut db.deliveries {
                if event.client.is_some() && event.at + r.security_log_days * DAY <= now {
                    event.client = None;
                    changed += 1;
                }
            }
            let before = db.deliveries.len();
            db.deliveries.retain(|e| e.at + r.delivery_days * DAY > now);
            changed += before - db.deliveries.len();
            // Books whose gifts were all cancelled: the file goes after the idle period.
            let mut expired = Vec::new();
            for book in db.books.iter().filter(|b| b.is_live()) {
                let gifts: Vec<&Gift> = db.gifts.iter().filter(|g| g.book_id == book.id).collect();
                let last_cancel = gifts.iter().map(|g| match g.state {
                    GiftState::Cancelled { at } => Some(at),
                    _ => None,
                });
                let all_cancelled: Option<Vec<i64>> = last_cancel.collect();
                if let Some(times) = all_cancelled
                    && let Some(last) = times.iter().max()
                    && last + r.idle_file_days * DAY <= now
                {
                    expired.push(book.id.clone());
                }
            }
            for book in db.books.iter_mut().filter(|b| expired.contains(&b.id)) {
                book.removed = Some(Removal::Expired);
                book.removed_at = Some(now);
                changed += 1;
            }
            // Removed books keep their declaration for the limitation period.
            let gone: Vec<String> = db
                .books
                .iter()
                .filter(|b| b.removed_at.is_some_and(|at| at + r.declaration_days * DAY <= now))
                .map(|b| b.id.clone())
                .collect();
            db.books.retain(|b| !gone.contains(&b.id));
            db.gifts.retain(|g| !gone.contains(&g.book_id));
            changed += gone.len();
            // Decided notices (and their appeals) after the notice period.
            let old: Vec<String> = db
                .notices
                .iter()
                .filter(|n| {
                    !matches!(n.decision, NoticeDecision::Open)
                        && n.received_at + r.notice_days * DAY <= now
                })
                .map(|n| n.id.clone())
                .collect();
            db.notices.retain(|n| !old.contains(&n.id));
            db.appeals.retain(|a| !old.contains(&a.notice_id));
            changed += old.len();
            // Identity of closed or banned accounts.
            for account in &mut db.accounts {
                let ended = match account.status {
                    AccountStatus::Closed { at } | AccountStatus::Banned { at } => Some(at),
                    _ => None,
                };
                if let Some(at) = ended
                    && account.erased_at.is_none()
                    && at + r.identity_days * DAY <= now
                {
                    account.identity = None;
                    account.email.clear();
                    account.erased_at = Some(now);
                    changed += 1;
                }
            }
            Ok::<_, ()>((changed, expired))
        });
        match result {
            Ok((changed, expired)) => {
                let hashes: Vec<(String, String)> = self.read(|db| {
                    db.books
                        .iter()
                        .filter(|b| expired.contains(&b.id))
                        .map(|b| (b.id.clone(), b.sha256.clone()))
                        .collect()
                });
                self.drop_links_of(&expired);
                for (_, sha) in &hashes {
                    self.delete_blob_if_unused(sha);
                }
                changed
            }
            Err(error) => {
                tracing::error!(?error, "retention purge failed");
                0
            }
        }
    }

    /// Periodic work: [`AppState::purge`], then cancellations the core has not
    /// confirmed yet are sent again. The binary runs it every few minutes.
    pub async fn maintain(&self) {
        let _ = self.purge();
        let pending: Vec<String> = self.read(|db| {
            db.gifts
                .iter()
                .filter(|g| matches!(g.state, GiftState::CancelPending { .. }))
                .map(|g| g.id.clone())
                .collect()
        });
        for gift in pending {
            let _ = self.push_cancel(&gift).await;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn texts_are_cleaned_and_bounded() {
        assert_eq!(clean("  a\u{7}b \n c ", 10, false).as_deref(), Some("ab  c"));
        assert_eq!(clean("a\nb", 10, true).as_deref(), Some("a\nb"));
        assert_eq!(clean("   ", 10, false), None);
        assert_eq!(clean("abcdef", 5, false), None);
        assert_eq!(optional("", 5, false), Ok(None));
        assert_eq!(optional("abcdef", 5, false), Err(()));
    }

    #[test]
    fn isbns_are_normalised() {
        assert_eq!(parse_isbn("978-2-07-036002-4").as_deref(), Some("9782070360024"));
        assert_eq!(parse_isbn("2-07-036002-4").as_deref(), Some("9782070360024"));
        assert_eq!(parse_isbn("978-2-07-036002-5"), None);
        assert_eq!(parse_isbn("abc"), None);
    }

    #[test]
    fn gift_forms_are_bounded() {
        let parse = |count: &str, locale: &str, reveal: &str| {
            GiftForm::parse(count, "", "", "", locale, reveal, 50)
        };
        assert_eq!(parse("", "fr", "").map(|f| f.count), Some(1));
        assert_eq!(parse("50", "en", "2026-12-24").map(|f| f.reveal_at), Some(Some(1_798_070_400)));
        for (count, locale, reveal) in [
            ("0", "fr", ""),
            ("51", "fr", ""),
            ("x", "fr", ""),
            ("1", "xx", ""),
            ("1", "fr", "24/12/2026"),
        ] {
            assert!(parse(count, locale, reveal).is_none(), "{count} {locale} {reveal}");
        }
        assert!(GiftForm::parse("1", &"x".repeat(81), "", "", "fr", "", 5).is_none());
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(
            file_name("../../etc/Le Horla.EPUB", "Le Horla", FileKind::Epub),
            "Le Horla.epub"
        );
        assert_eq!(file_name("", "L'Étranger", FileKind::Pdf), "L'Étranger.pdf");
        assert_eq!(file_name("", "", FileKind::Epub), "livre.epub");
        assert_eq!(file_name("a\r\nb.pdf", "x", FileKind::Pdf), "ab.pdf");
    }

    #[test]
    fn sizes_are_shown_in_megabytes() {
        assert_eq!(size_text(Locale::Fr, 50 << 20), "50 Mo");
        assert_eq!(size_text(Locale::En, 3 << 19), "1.5 MB");
        assert_eq!(size_text(Locale::Fr, 1_500), "2 ko");
        assert_eq!(size_text(Locale::En, 0), "1 kB");
    }
}
