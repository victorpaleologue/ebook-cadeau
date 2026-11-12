//! A gift: one eBook offered to one recipient, redeemable with a [`GiftCode`].
//!
//! These are plain serialisable records. Nothing here reads a clock or a random source:
//! timestamps and random bytes are supplied by the caller.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::book::BookMeta;
use crate::code::{GiftCode, claim_url};
use crate::locale::Locale;

/// Why a gift record is not acceptable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum GiftError {
    /// The identifier is empty, too long or contains a character outside `A-Z a-z 0-9 - _`.
    #[error("a gift id is 1 to {max} characters from A-Z, a-z, 0-9, '-' and '_'", max = GiftId::MAX_LEN)]
    InvalidId,
    /// Fewer random bytes were supplied than an identifier needs.
    #[error("a gift id needs at least {needed} random bytes, got {got}")]
    NotEnoughRandomness {
        /// Bytes required ([`GiftId::MIN_RANDOM_BYTES`]).
        needed: usize,
        /// Bytes supplied.
        got: usize,
    },
    /// A required text field is empty or blank.
    #[error("{field} must not be empty")]
    Empty {
        /// Name of the offending field.
        field: &'static str,
    },
    /// A text field is longer than allowed.
    #[error("{field} is too long: {len} characters, at most {max}")]
    TooLong {
        /// Name of the offending field.
        field: &'static str,
        /// Length found, in characters.
        len: usize,
        /// Maximum allowed, in characters.
        max: usize,
    },
    /// A text field contains a control character (other than a line break in the message).
    #[error("{field} contains a control character")]
    ControlChar {
        /// Name of the offending field.
        field: &'static str,
    },
    /// `expires_unix` is not after `created_unix`.
    #[error("the gift expires before or when it is created")]
    ExpiresBeforeCreation,
    /// `expires_unix` is not after `reveal_unix`: the gift could never be claimed.
    #[error("the gift expires before it is revealed")]
    ExpiresBeforeReveal,
    /// A delivery URL that is not an absolute `http://` or `https://` URL.
    #[error("delivery URL must be an absolute http(s) URL without spaces")]
    InvalidUrl,
    /// A SHA-256 digest that is not 64 hexadecimal characters.
    #[error("sha256 must be 64 hexadecimal characters")]
    InvalidSha256,
    /// A file name with a path separator, a control character, or a reserved name.
    #[error("file name must be a plain name without path separators")]
    InvalidFileName,
    /// A media type that does not look like `type/subtype`, or not the expected one.
    #[error("media type must be the type/subtype of the file format")]
    InvalidMediaType,
    /// A licence format that is not a short lower-case token such as `lcpl`.
    #[error("licence format must be a short lower-case token such as \"lcpl\"")]
    InvalidLicenceFormat,
    /// A partner download link whose `expires_at` is not in the future.
    #[error("the download link has already expired")]
    LinkExpired,
}

/// Opaque, non-secret identifier of a gift (unlike its [`GiftCode`], which is the secret
/// the recipient redeems with). Safe to put in URLs, file names and logs: 1 to 64
/// characters from `A-Z a-z 0-9 - _`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct GiftId(String);

impl GiftId {
    /// Longest accepted identifier, in characters.
    pub const MAX_LEN: usize = 64;
    /// Random bytes [`from_random_bytes`](Self::from_random_bytes) needs (128 bits).
    pub const MIN_RANDOM_BYTES: usize = 16;
    /// Prefix of generated identifiers.
    pub const PREFIX: &'static str = "g_";

    /// Wraps an existing identifier after validating it.
    ///
    /// # Errors
    /// [`GiftError::InvalidId`] if it is empty, longer than [`MAX_LEN`](Self::MAX_LEN) or
    /// contains anything but ASCII letters, digits, `-` and `_`.
    pub fn new(id: impl Into<String>) -> Result<Self, GiftError> {
        let id = id.into();
        let valid = !id.is_empty()
            && id.len() <= Self::MAX_LEN
            && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        if valid { Ok(Self(id)) } else { Err(GiftError::InvalidId) }
    }

    /// Builds an identifier from random bytes supplied by the caller: `g_` followed by
    /// the first sixteen bytes as 32 lower-case hexadecimal characters. Further bytes are
    /// ignored.
    ///
    /// # Errors
    /// [`GiftError::NotEnoughRandomness`] if fewer than sixteen bytes are supplied.
    pub fn from_random_bytes(bytes: &[u8]) -> Result<Self, GiftError> {
        let Some(head) = bytes.get(..Self::MIN_RANDOM_BYTES) else {
            return Err(GiftError::NotEnoughRandomness {
                needed: Self::MIN_RANDOM_BYTES,
                got: bytes.len(),
            });
        };
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut id = String::with_capacity(Self::PREFIX.len() + 2 * Self::MIN_RANDOM_BYTES);
        id.push_str(Self::PREFIX);
        for byte in head {
            id.push(char::from(HEX[usize::from(byte >> 4)]));
            id.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Ok(Self(id))
    }

    /// The identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GiftId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for GiftId {
    type Err = GiftError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for GiftId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GiftId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// Format of a DRM licence or fulfilment token handed to the recipient's reading system.
///
/// Serialises as a lower-case string: `"lcpl"`, `"acsm"`, or the custom token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LicenceFormat {
    /// Readium LCP licence document (`.lcpl`).
    Lcpl,
    /// Adobe Content Server fulfilment message (`.acsm`).
    Acsm,
    /// Another format, named by a short lower-case token.
    Other(String),
}

impl LicenceFormat {
    /// The token used in serialised form, which is also the usual file extension.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Lcpl => "lcpl",
            Self::Acsm => "acsm",
            Self::Other(token) => token,
        }
    }

    /// The registered media type of the licence file, when there is one.
    #[must_use]
    pub fn media_type(&self) -> Option<&'static str> {
        match self {
            Self::Lcpl => Some("application/vnd.readium.lcp.license.v1.0+json"),
            Self::Acsm => Some("application/vnd.adobe.adept+xml"),
            Self::Other(_) => None,
        }
    }

    /// Whether `token` is acceptable as a format name: 1 to 16 lower-case letters or digits.
    fn is_valid_token(token: &str) -> bool {
        (1..=16).contains(&token.len())
            && token.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    }
}

impl From<&str> for LicenceFormat {
    /// Case-insensitive for the known formats; anything else is kept as given.
    fn from(token: &str) -> Self {
        match token.to_ascii_lowercase().as_str() {
            "lcpl" => Self::Lcpl,
            "acsm" => Self::Acsm,
            _ => Self::Other(token.to_owned()),
        }
    }
}

impl fmt::Display for LicenceFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for LicenceFormat {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for LicenceFormat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        Ok(Self::from(s.as_ref()))
    }
}

/// Format of an eBook file served by a partner (`partner_file` deliveries).
///
/// Serialises as `"epub"` or `"pdf"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    /// EPUB (`application/epub+zip`).
    Epub,
    /// PDF (`application/pdf`).
    Pdf,
}

impl FileFormat {
    /// Every format.
    pub const ALL: [FileFormat; 2] = [Self::Epub, Self::Pdf];

    /// The serialised name, which is also the usual file extension: `epub`, `pdf`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Epub => "epub",
            Self::Pdf => "pdf",
        }
    }

    /// The media type of the file.
    #[must_use]
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Epub => "application/epub+zip",
            Self::Pdf => "application/pdf",
        }
    }

    /// Parses `epub` or `pdf` (case-insensitive).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|format| format.as_str().eq_ignore_ascii_case(value.trim()))
    }
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How the eBook reaches the recipient once the code is redeemed.
///
/// The service never stores book files: a file is always served by a registered partner
/// (a store, or a "shelf" service hosting files for their rights holders).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delivery {
    /// The partner serves the file. On each claim the service asks `url` (signed) for a
    /// short-lived download link ([`PartnerFileLink`]) and redirects the device to it.
    PartnerFile {
        /// Format of the file.
        format: FileFormat,
        /// Absolute URL at the partner that answers with a download link.
        url: String,
    },
    /// Reseller mode: the recipient is sent to the store, which fulfils the order in its
    /// own account system.
    ResellerRedirect {
        /// Absolute URL at the store.
        url: String,
    },
    /// Reseller mode: the store issues a DRM licence or fulfilment token that the
    /// recipient's reading system opens directly.
    ResellerLicence {
        /// Licence format, e.g. `lcpl` or `acsm`.
        format: LicenceFormat,
        /// Absolute URL the licence is fetched from.
        url: String,
    },
}

impl Delivery {
    /// The serialised `type` tag: `"partner_file"`, `"reseller_redirect"` or
    /// `"reseller_licence"`.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::PartnerFile { .. } => "partner_file",
            Self::ResellerRedirect { .. } => "reseller_redirect",
            Self::ResellerLicence { .. } => "reseller_licence",
        }
    }

    /// The partner URL of the delivery.
    #[must_use]
    pub fn url(&self) -> &str {
        match self {
            Self::PartnerFile { url, .. }
            | Self::ResellerRedirect { url }
            | Self::ResellerLicence { url, .. } => url,
        }
    }

    /// Checks the fields for syntactic sanity (see [`Gift::validate`]).
    ///
    /// URL checks are a sanity filter (absolute `http`/`https`, a host, no spaces or
    /// control characters, at most 2048 bytes) — not a defence against requests to
    /// internal hosts; a server that fetches these URLs must apply its own allow-list.
    ///
    /// # Errors
    /// The first [`GiftError`] found.
    pub fn validate(&self) -> Result<(), GiftError> {
        match self {
            Self::PartnerFile { url, .. } | Self::ResellerRedirect { url } => {
                if is_http_url(url) {
                    Ok(())
                } else {
                    Err(GiftError::InvalidUrl)
                }
            }
            Self::ResellerLicence { format, url } => {
                if !LicenceFormat::is_valid_token(format.as_str()) {
                    return Err(GiftError::InvalidLicenceFormat);
                }
                if is_http_url(url) { Ok(()) } else { Err(GiftError::InvalidUrl) }
            }
        }
    }
}

/// What a partner answers when the service asks for the file of a `partner_file` gift
/// (`docs/api.md` §4.5): a short-lived download link and what it serves.
///
/// Every member but `download_url` is optional; the service checks what is given
/// ([`validate`](Self::validate)) and derives the rest from the gift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartnerFileLink {
    /// Absolute `http(s)` URL the device is redirected to (short-lived, signed by the
    /// partner). Its host must be one of the partner's `redirect_hosts`.
    pub download_url: String,
    /// The file name the partner serves, e.g. `"candide.epub"`. A plain name, no path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// Media type served; must be the one of the gift's [`FileFormat`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Size of the file in bytes (at least 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// SHA-256 of the file as 64 hexadecimal characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// When `download_url` stops working (Unix seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

impl PartnerFileLink {
    /// Checks the answer for a gift of `format` at `now`: an absolute `http(s)` download
    /// URL; a plain file name; the media type of `format`; a non-zero size; a SHA-256 of
    /// 64 hexadecimal digits; an expiry in the future. (Whether the URL's host is one the
    /// partner may use is the caller's check.)
    ///
    /// # Errors
    /// The first [`GiftError`] found.
    pub fn validate(&self, format: FileFormat, now_unix: i64) -> Result<(), GiftError> {
        if !is_http_url(&self.download_url) {
            return Err(GiftError::InvalidUrl);
        }
        if self.file_name.as_deref().is_some_and(|name| !is_plain_file_name(name)) {
            return Err(GiftError::InvalidFileName);
        }
        if let Some(media_type) = self.media_type.as_deref() {
            let essence = media_type.split(';').next().unwrap_or("").trim();
            if !is_media_type(essence) || !essence.eq_ignore_ascii_case(format.media_type()) {
                return Err(GiftError::InvalidMediaType);
            }
        }
        if self.size_bytes == Some(0) {
            return Err(GiftError::Empty { field: "size_bytes" });
        }
        if self
            .sha256
            .as_deref()
            .is_some_and(|sha| sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(GiftError::InvalidSha256);
        }
        if self.expires_at.is_some_and(|at| at <= now_unix) {
            return Err(GiftError::LinkExpired);
        }
        Ok(())
    }
}

/// A plain file name: 1..=255 bytes, no path separator, no control character, not `.`/`..`.
fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && name != "."
        && name != ".."
        && !name.chars().any(|c| c == '/' || c == '\\' || c.is_control())
        && name.trim() == name
}

/// `type/subtype` made of RFC 6838 restricted-name characters.
fn is_media_type(value: &str) -> bool {
    fn is_name(part: &str) -> bool {
        (1..=127).contains(&part.len())
            && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&b))
    }
    value.split_once('/').is_some_and(|(kind, subtype)| is_name(kind) && is_name(subtype))
}

/// An absolute `http://` or `https://` URL with a host and no whitespace or control
/// characters.
fn is_http_url(url: &str) -> bool {
    if url.len() > 2048 || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    let lower = url.get(..8).unwrap_or(url).to_ascii_lowercase();
    let rest = if lower.starts_with("https://") {
        url.get(8..)
    } else if lower.starts_with("http://") {
        url.get(7..)
    } else {
        None
    };
    rest.and_then(|r| r.split(['/', '?', '#']).next()).is_some_and(|authority| {
        let host = authority.rsplit('@').next().unwrap_or(authority);
        !host.is_empty() && !host.starts_with(':')
    })
}

/// Where a gift is in its life (the states of `docs/api.md`).
///
/// ```text
/// created ──► redeemed ──► exhausted   (file: download-session limit reached)
///    │            └──────► fulfilled   (reseller confirmed the book is in an account)
///    └── any time ──► cancelled | expired | reissued
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GiftStatus {
    /// Issued; nobody has claimed it yet.
    #[default]
    Created,
    /// Claimed at least once. A file delivery stays downloadable by whoever holds the card.
    Redeemed,
    /// File delivery: the limit of download sessions has been reached.
    Exhausted,
    /// Reseller delivery: the store confirmed the book is in the recipient's account.
    Fulfilled,
    /// Cancelled by the giver, the store or an administrator (e.g. refund).
    Cancelled,
    /// Past its `expires_unix`.
    Expired,
    /// The code was voided because a new one was issued for the same gift (lost card).
    Reissued,
}

impl GiftStatus {
    /// Every status, in life-cycle order.
    pub const ALL: [GiftStatus; 7] = [
        Self::Created,
        Self::Redeemed,
        Self::Exhausted,
        Self::Fulfilled,
        Self::Cancelled,
        Self::Expired,
        Self::Reissued,
    ];

    /// The serialised name: `"created"`, `"redeemed"`, `"exhausted"`, `"fulfilled"`,
    /// `"cancelled"`, `"expired"`, `"reissued"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Redeemed => "redeemed",
            Self::Exhausted => "exhausted",
            Self::Fulfilled => "fulfilled",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::Reissued => "reissued",
        }
    }

    /// Whether the holder of the card may (still) obtain the eBook: on first claim
    /// (`Created`) or again afterwards (`Redeemed`).
    #[must_use]
    pub const fn allows_claim(self) -> bool {
        matches!(self, Self::Created | Self::Redeemed)
    }

    /// Whether the code no longer works for a reason the holder cannot fix — what the
    /// redemption page answers with `410 Gone`: `Cancelled`, `Expired`, `Reissued`.
    #[must_use]
    pub const fn is_gone(self) -> bool {
        matches!(self, Self::Cancelled | Self::Expired | Self::Reissued)
    }

    /// Whether the status can no longer change: everything except `Created` and
    /// `Redeemed`.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        !self.allows_claim()
    }
}

impl fmt::Display for GiftStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One eBook offered to one recipient.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gift {
    /// Non-secret identifier.
    pub id: GiftId,
    /// The secret redemption code printed on the card.
    pub code: GiftCode,
    /// The book being offered.
    pub book: BookMeta,
    /// Who offers the book, as it should appear on the card.
    #[serde(default)]
    pub giver_name: Option<String>,
    /// Who receives the book, as it should appear on the card.
    #[serde(default)]
    pub recipient_name: Option<String>,
    /// Personal message printed inside the card. Line breaks are allowed.
    #[serde(default)]
    pub message: Option<String>,
    /// Language of the card and of the redemption pages.
    #[serde(default)]
    pub locale: Locale,
    /// Creation time, seconds since the Unix epoch (UTC).
    pub created_unix: i64,
    /// Expiry time, seconds since the Unix epoch (UTC); `None` (the default, and the
    /// recommended setting for a paid title) if the gift never expires.
    #[serde(default)]
    pub expires_unix: Option<i64>,
    /// Reveal time, seconds since the Unix epoch (UTC): before it, the redemption page
    /// shows the date and no action (a present wrapped until the day). `None` reveals
    /// the gift at once.
    #[serde(default)]
    pub reveal_unix: Option<i64>,
    /// How the eBook reaches the recipient.
    pub delivery: Delivery,
}

impl Gift {
    /// Longest giver or recipient name, in characters.
    pub const MAX_NAME_CHARS: usize = 80;
    /// Longest personal message, in characters (what fits inside the card).
    pub const MAX_MESSAGE_CHARS: usize = 400;
    /// Most lines in a personal message (lines are separated by `\n`).
    pub const MAX_MESSAGE_LINES: usize = 8;

    /// The URL to encode in the card's QR code, see [`claim_url`].
    #[must_use]
    pub fn claim_url(&self, base_url: &str) -> String {
        claim_url(base_url, &self.code)
    }

    /// Whether the gift has expired at `now_unix` (the expiry instant itself counts as
    /// expired). Gifts without an expiry never expire.
    #[must_use]
    pub fn is_expired_at(&self, now_unix: i64) -> bool {
        self.expires_unix.is_some_and(|expires| now_unix >= expires)
    }

    /// Whether the gift may be shown and claimed at `now_unix`: always, unless a reveal
    /// time is set and has not been reached yet.
    #[must_use]
    pub fn is_revealed_at(&self, now_unix: i64) -> bool {
        self.reveal_unix.is_none_or(|reveal| now_unix >= reveal)
    }

    /// The status to act on at `now_unix`, given the status last `recorded` in storage:
    /// a gift that could still be claimed (`Created`, `Redeemed`) but is past its expiry
    /// is `Expired`; every other status is returned unchanged.
    #[must_use]
    pub fn status_at(&self, recorded: GiftStatus, now_unix: i64) -> GiftStatus {
        if recorded.allows_claim() && self.is_expired_at(now_unix) {
            GiftStatus::Expired
        } else {
            recorded
        }
    }

    /// Checks a record built from untrusted input (a web form, a partner API call).
    ///
    /// * the book has a non-blank title;
    /// * names are non-blank, at most [`MAX_NAME_CHARS`](Self::MAX_NAME_CHARS)
    ///   characters, and contain no control characters;
    /// * the message is non-blank, at most [`MAX_MESSAGE_CHARS`](Self::MAX_MESSAGE_CHARS)
    ///   characters on at most [`MAX_MESSAGE_LINES`](Self::MAX_MESSAGE_LINES) lines,
    ///   with no control characters other than line breaks;
    /// * `expires_unix`, when set, is after `created_unix` and after `reveal_unix`;
    /// * the delivery passes [`Delivery::validate`].
    ///
    /// Optional fields that are `None` are fine; `Some("")` is not (normalise with
    /// [`Gift::normalise_text`] first).
    ///
    /// # Errors
    /// The first [`GiftError`] found.
    pub fn validate(&self) -> Result<(), GiftError> {
        if self.book.title.trim().is_empty() {
            return Err(GiftError::Empty { field: "book title" });
        }
        check_line("giver name", self.giver_name.as_deref())?;
        check_line("recipient name", self.recipient_name.as_deref())?;
        if let Some(message) = self.message.as_deref() {
            check_text("message", message, Self::MAX_MESSAGE_CHARS, true)?;
            let lines = message.chars().filter(|c| *c == '\n').count() + 1;
            if lines > Self::MAX_MESSAGE_LINES {
                return Err(GiftError::TooLong {
                    field: "message (lines)",
                    len: lines,
                    max: Self::MAX_MESSAGE_LINES,
                });
            }
        }
        if self.expires_unix.is_some_and(|expires| expires <= self.created_unix) {
            return Err(GiftError::ExpiresBeforeCreation);
        }
        if let (Some(expires), Some(reveal)) = (self.expires_unix, self.reveal_unix)
            && expires <= reveal
        {
            return Err(GiftError::ExpiresBeforeReveal);
        }
        self.delivery.validate()
    }

    /// Normalises a free-text form field: trims it, converts `\r\n` and `\r` to `\n`,
    /// and maps a blank value to `None`.
    #[must_use]
    pub fn normalise_text(input: &str) -> Option<String> {
        let text = input.replace("\r\n", "\n").replace('\r', "\n");
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    }
}

/// Validates an optional single-line field.
fn check_line(field: &'static str, value: Option<&str>) -> Result<(), GiftError> {
    value.map_or(Ok(()), |text| check_text(field, text, Gift::MAX_NAME_CHARS, false))
}

/// Validates a text field: non-blank, bounded, free of control characters.
fn check_text(
    field: &'static str,
    text: &str,
    max_chars: usize,
    allow_line_breaks: bool,
) -> Result<(), GiftError> {
    if text.trim().is_empty() {
        return Err(GiftError::Empty { field });
    }
    // Bound the work on hostile input: no need to count a megabyte of characters.
    let len = text.chars().take(max_chars + 1).count();
    if len > max_chars {
        return Err(GiftError::TooLong { field, len: text.chars().count(), max: max_chars });
    }
    let forbidden = |c: char| c.is_control() && !(allow_line_breaks && c == '\n');
    if text.chars().any(forbidden) {
        return Err(GiftError::ControlChar { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book::Contributor;

    fn sample() -> Gift {
        let mut book = BookMeta::titled("L'Étranger");
        book.contributors.push(Contributor::author("Albert Camus"));
        book.isbn13 = "9782070360024".parse().ok();
        Gift {
            id: GiftId::from_random_bytes(&[0xab; 16]).expect("16 bytes"),
            code: GiftCode::from_random_bytes(&[1, 2, 3, 4, 5, 6, 7, 8]).expect("8 bytes"),
            book,
            giver_name: Some("Mamie Jeanne".into()),
            recipient_name: Some("Léo".into()),
            message: Some("Joyeux anniversaire !\nBonne lecture.".into()),
            locale: Locale::Fr,
            created_unix: 1_794_000_000,
            expires_unix: Some(1_794_000_000 + 365 * 86_400),
            reveal_unix: None,
            delivery: Delivery::PartnerFile {
                format: FileFormat::Epub,
                url: "https://shelf.example/ebc/file".into(),
            },
        }
    }

    #[test]
    fn sample_is_valid_and_round_trips_through_json() {
        let gift = sample();
        assert_eq!(gift.validate(), Ok(()));
        let json = serde_json::to_string(&gift).expect("serialises");
        let back: Gift = serde_json::from_str(&json).expect("parses");
        assert_eq!(back, gift);
        // Stable wire format: the pieces other components rely on.
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["id"], "g_abababababababababababababababab");
        assert_eq!(value["code"], gift.code.compact());
        assert_eq!(value["locale"], "fr");
        assert_eq!(value["delivery"]["type"], "partner_file");
        assert_eq!(value["delivery"]["format"], "epub");
        assert_eq!(value["reveal_unix"], serde_json::Value::Null);
        assert_eq!(value["delivery"]["url"], "https://shelf.example/ebc/file");
        assert_eq!(value["book"]["isbn13"], "9782070360024");
    }

    #[test]
    fn optional_fields_default_when_absent() {
        let json = r#"{
            "id": "g1", "code": "bbbb-bbbb-bbbb", "book": {"title": "T"},
            "created_unix": 10,
            "delivery": {"type": "reseller_redirect", "url": "https://store.example/r/1"}
        }"#;
        let gift: Gift = serde_json::from_str(json).expect("parses");
        assert_eq!(gift.giver_name, None);
        assert_eq!(gift.recipient_name, None);
        assert_eq!(gift.message, None);
        assert_eq!(gift.locale, Locale::Fr);
        assert_eq!(gift.expires_unix, None);
        assert_eq!(gift.reveal_unix, None);
        assert_eq!(gift.code.compact(), "BBBBBBBBBBBB");
        assert_eq!(gift.validate(), Ok(()));
        assert!(!gift.is_expired_at(i64::MAX));
        assert!(gift.is_revealed_at(i64::MIN));
    }

    #[test]
    fn malformed_json_records_are_rejected() {
        let base = serde_json::to_value(sample()).expect("serialises");
        let mutate = |pointer: &str, new: serde_json::Value| {
            let mut value = base.clone();
            *value.pointer_mut(pointer).expect("pointer exists") = new;
            serde_json::from_value::<Gift>(value)
        };
        assert!(mutate("/id", "../../etc/passwd".into()).is_err());
        assert!(mutate("/id", "".into()).is_err());
        assert!(mutate("/id", 12.into()).is_err());
        assert!(mutate("/code", "BBBBBBBBBBBC".into()).is_err(), "wrong check symbol");
        assert!(mutate("/code", "0000000000".into()).is_err(), "digits are not symbols");
        assert!(mutate("/code", "short".into()).is_err());
        assert!(mutate("/locale", "xx".into()).is_err());
        assert!(mutate("/delivery/type", "carrier_pigeon".into()).is_err());
        assert!(mutate("/delivery/format", "mobi".into()).is_err());
        assert!(mutate("/delivery/url", 7.into()).is_err());
        assert!(mutate("/created_unix", "yesterday".into()).is_err());
        assert!(mutate("/book/isbn13", "9782070360025".into()).is_err());
        assert!(mutate("/id", "ok_ID-1".into()).is_ok());
        assert!(serde_json::from_str::<Gift>("{}").is_err());
    }

    #[test]
    fn gift_id_validation() {
        assert_eq!(GiftId::new("abc-DEF_123").map(|id| id.to_string()), Ok("abc-DEF_123".into()));
        assert_eq!("x".parse::<GiftId>().map(|id| id.as_str().to_owned()), Ok("x".into()));
        assert!(GiftId::new("a".repeat(64)).is_ok());
        for bad in [
            String::new(),
            "a".repeat(65),
            "a b".into(),
            "a/b".into(),
            "..".into(),
            "a.b".into(),
            "é".into(),
            "a\n".into(),
            "a\u{0}".into(),
            "%2e%2e".into(),
            "a".repeat(1_000_000),
        ] {
            assert_eq!(
                GiftId::new(bad.clone()),
                Err(GiftError::InvalidId),
                "id {:?}",
                &bad[..bad.len().min(12)]
            );
        }
        let id = GiftId::from_random_bytes(&(0u8..32).collect::<Vec<_>>()).expect("enough");
        assert_eq!(id.as_str(), "g_000102030405060708090a0b0c0d0e0f");
        assert_eq!(GiftId::new(id.as_str()), Ok(id.clone()), "generated ids are valid ids");
        assert_eq!(id.as_ref(), id.as_str());
        assert_eq!(
            GiftId::from_random_bytes(&[0; 15]),
            Err(GiftError::NotEnoughRandomness { needed: 16, got: 15 })
        );
        assert_eq!(
            serde_json::to_string(&id).expect("serialises"),
            "\"g_000102030405060708090a0b0c0d0e0f\""
        );
    }

    #[test]
    fn expiry_reveal_and_status() {
        use GiftStatus::{Cancelled, Created, Exhausted, Expired, Fulfilled, Redeemed, Reissued};
        let gift = sample();
        let expires = gift.expires_unix.expect("sample expires");
        assert!(!gift.is_expired_at(gift.created_unix));
        assert!(!gift.is_expired_at(expires - 1));
        assert!(gift.is_expired_at(expires));
        assert!(gift.is_expired_at(i64::MAX));
        assert!(!gift.is_expired_at(i64::MIN));

        assert_eq!(gift.status_at(Created, expires - 1), Created);
        assert_eq!(gift.status_at(Created, expires), Expired);
        assert_eq!(gift.status_at(Redeemed, expires - 1), Redeemed);
        assert_eq!(gift.status_at(Redeemed, expires), Expired);
        for settled in [Exhausted, Fulfilled, Cancelled, Expired, Reissued] {
            assert_eq!(gift.status_at(settled, 0), settled);
            assert_eq!(gift.status_at(settled, expires + 1), settled);
        }

        let mut eternal = sample();
        eternal.expires_unix = None;
        assert_eq!(eternal.status_at(Created, i64::MAX), Created);
        assert_eq!(eternal.status_at(Redeemed, i64::MAX), Redeemed);

        let mut wrapped = sample();
        wrapped.reveal_unix = Some(wrapped.created_unix + 86_400);
        assert!(!wrapped.is_revealed_at(wrapped.created_unix));
        assert!(!wrapped.is_revealed_at(wrapped.created_unix + 86_399));
        assert!(wrapped.is_revealed_at(wrapped.created_unix + 86_400));
        assert!(sample().is_revealed_at(i64::MIN), "no reveal time: revealed at once");
        assert_eq!(wrapped.validate(), Ok(()));
        // A gift that expires before it can be opened is a configuration error.
        wrapped.reveal_unix = wrapped.expires_unix;
        assert_eq!(wrapped.validate(), Err(GiftError::ExpiresBeforeReveal));
        wrapped.expires_unix = None;
        assert_eq!(wrapped.validate(), Ok(()));
    }

    #[test]
    fn status_predicates_and_names() {
        use GiftStatus::{Cancelled, Created, Exhausted, Expired, Fulfilled, Redeemed, Reissued};
        assert_eq!(GiftStatus::default(), Created);
        let claimable: Vec<_> = GiftStatus::ALL.into_iter().filter(|s| s.allows_claim()).collect();
        assert_eq!(claimable, [Created, Redeemed]);
        let gone: Vec<_> = GiftStatus::ALL.into_iter().filter(|s| s.is_gone()).collect();
        assert_eq!(gone, [Cancelled, Expired, Reissued]);
        let terminal: Vec<_> = GiftStatus::ALL.into_iter().filter(|s| s.is_terminal()).collect();
        assert_eq!(terminal, [Exhausted, Fulfilled, Cancelled, Expired, Reissued]);
        // The names are the ones of the HTTP contract.
        let names: Vec<_> = GiftStatus::ALL.into_iter().map(GiftStatus::as_str).collect();
        assert_eq!(
            names,
            ["created", "redeemed", "exhausted", "fulfilled", "cancelled", "expired", "reissued"]
        );
        for status in GiftStatus::ALL {
            let json = serde_json::to_string(&status).expect("serialises");
            assert_eq!(json, format!("\"{}\"", status.as_str()));
            assert_eq!(status.to_string(), status.as_str());
            assert_eq!(serde_json::from_str::<GiftStatus>(&json).expect("parses"), status);
            // Nothing is both claimable and gone.
            assert!(!(status.allows_claim() && status.is_gone()));
        }
        assert!(serde_json::from_str::<GiftStatus>("\"CREATED\"").is_err());
        assert!(serde_json::from_str::<GiftStatus>("\"active\"").is_err());
    }

    #[test]
    fn text_fields_are_validated() {
        let with = |edit: fn(&mut Gift)| {
            let mut gift = sample();
            edit(&mut gift);
            gift.validate()
        };
        assert_eq!(
            with(|g| g.book.title = "  ".into()),
            Err(GiftError::Empty { field: "book title" })
        );
        assert_eq!(
            with(|g| g.giver_name = Some(String::new())),
            Err(GiftError::Empty { field: "giver name" })
        );
        assert_eq!(
            with(|g| g.recipient_name = Some(" \t".into())),
            Err(GiftError::Empty { field: "recipient name" })
        );
        assert_eq!(with(|g| g.giver_name = None), Ok(()));
        assert_eq!(with(|g| g.giver_name = Some("é".repeat(80))), Ok(()));
        assert_eq!(
            with(|g| g.giver_name = Some("é".repeat(81))),
            Err(GiftError::TooLong { field: "giver name", len: 81, max: 80 })
        );
        assert_eq!(
            with(|g| g.recipient_name = Some("Léo\nDROP TABLE".into())),
            Err(GiftError::ControlChar { field: "recipient name" })
        );
        assert_eq!(
            with(|g| g.giver_name = Some("a\u{1b}[31m".into())),
            Err(GiftError::ControlChar { field: "giver name" })
        );
        assert_eq!(with(|g| g.message = Some("x".repeat(400))), Ok(()));
        assert_eq!(
            with(|g| g.message = Some("x".repeat(401))),
            Err(GiftError::TooLong { field: "message", len: 401, max: 400 })
        );
        assert_eq!(
            with(|g| g.message = Some("x".repeat(2_000_000))),
            Err(GiftError::TooLong { field: "message", len: 2_000_000, max: 400 })
        );
        assert_eq!(with(|g| g.message = Some("a\nb\nc".into())), Ok(()));
        assert_eq!(
            with(|g| g.message = Some("a\tb".into())),
            Err(GiftError::ControlChar { field: "message" })
        );
        assert_eq!(
            with(|g| g.message = Some("a\r\nb".into())),
            Err(GiftError::ControlChar { field: "message" })
        );
        assert_eq!(
            with(|g| g.message = Some("\n\n".into())),
            Err(GiftError::Empty { field: "message" })
        );
        assert_eq!(with(|g| g.message = Some("a\n".repeat(7) + "a")), Ok(()));
        assert_eq!(
            with(|g| g.message = Some("a\n".repeat(8) + "a")),
            Err(GiftError::TooLong { field: "message (lines)", len: 9, max: 8 })
        );
        assert_eq!(
            with(|g| g.expires_unix = Some(g.created_unix)),
            Err(GiftError::ExpiresBeforeCreation)
        );
        assert_eq!(
            with(|g| g.expires_unix = Some(i64::MIN)),
            Err(GiftError::ExpiresBeforeCreation)
        );
        assert_eq!(with(|g| g.expires_unix = Some(g.created_unix + 1)), Ok(()));
    }

    #[test]
    fn normalise_text_cleans_form_input() {
        assert_eq!(Gift::normalise_text("  Léo \r\n"), Some("Léo".into()));
        assert_eq!(Gift::normalise_text("a\r\nb\rc\n"), Some("a\nb\nc".into()));
        assert_eq!(Gift::normalise_text(""), None);
        assert_eq!(Gift::normalise_text(" \r\n\t "), None);
    }

    #[test]
    fn partner_file_deliveries_and_links_are_validated() {
        let delivery = |url: &str| {
            Delivery::PartnerFile { format: FileFormat::Pdf, url: url.into() }.validate()
        };
        assert_eq!(delivery("https://shelf.example/f?x=1"), Ok(()));
        assert_eq!(delivery("ftp://shelf.example/f"), Err(GiftError::InvalidUrl));
        assert_eq!(delivery(""), Err(GiftError::InvalidUrl));
        assert_eq!(FileFormat::parse(" EPUB "), Some(FileFormat::Epub));
        assert_eq!(FileFormat::parse("mobi"), None);
        assert_eq!(FileFormat::Pdf.media_type(), "application/pdf");
        let parsed: Delivery = serde_json::from_str(
            r#"{"type":"partner_file","format":"epub","url":"https://s.example/f"}"#,
        )
        .expect("parses");
        assert_eq!(parsed.type_name(), "partner_file");
        assert_eq!(parsed.url(), "https://s.example/f");
        assert!(
            serde_json::from_str::<Delivery>(
                r#"{"type":"partner_file","format":"exe","url":"https://s.example/f"}"#
            )
            .is_err()
        );

        let sha = "9F86D081884C7D659A2FEAA0C55AD015A3BF4F1B2B0B822CD15D6C15B0F00A08";
        let good = PartnerFileLink {
            download_url: "https://cdn.shelf.example/dl/abc?sig=1".into(),
            file_name: Some("Été 2026 (v2).epub".into()),
            media_type: Some("application/epub+zip".into()),
            size_bytes: Some(345_678),
            sha256: Some(sha.into()),
            expires_at: Some(1_000),
        };
        assert_eq!(good.validate(FileFormat::Epub, 999), Ok(()));
        let minimal: PartnerFileLink =
            serde_json::from_str(r#"{"download_url":"http://127.0.0.1:9/x"}"#).expect("parses");
        assert_eq!(minimal.validate(FileFormat::Pdf, 0), Ok(()));
        assert_eq!(
            serde_json::to_string(&minimal).expect("json"),
            r#"{"download_url":"http://127.0.0.1:9/x"}"#
        );
        let with = |change: &dyn Fn(&mut PartnerFileLink)| {
            let mut link = good.clone();
            change(&mut link);
            link.validate(FileFormat::Epub, 999)
        };
        assert_eq!(with(&|l| l.download_url = "javascript:x".into()), Err(GiftError::InvalidUrl));
        assert_eq!(with(&|l| l.download_url = "/relative".into()), Err(GiftError::InvalidUrl));
        for bad_name in
            ["", ".", "..", "../x.epub", "a/b.epub", "a\\b.epub", "a\nb.epub", " x.epub"]
        {
            assert_eq!(
                with(&|l| l.file_name = Some(bad_name.into())),
                Err(GiftError::InvalidFileName),
                "{bad_name:?}"
            );
        }
        assert_eq!(with(&|l| l.file_name = Some("a".repeat(256))), Err(GiftError::InvalidFileName));
        assert_eq!(
            with(&|l| l.media_type = Some("application/pdf".into())),
            Err(GiftError::InvalidMediaType)
        );
        assert_eq!(
            with(&|l| l.media_type = Some("text/html".into())),
            Err(GiftError::InvalidMediaType)
        );
        assert_eq!(with(&|l| l.media_type = Some("application/EPUB+zip; x=1".into())), Ok(()));
        assert_eq!(
            with(&|l| l.size_bytes = Some(0)),
            Err(GiftError::Empty { field: "size_bytes" })
        );
        for bad_sha in ["", "abc", &sha[..63], &format!("{sha}0"), &sha.replace('9', "g")] {
            assert_eq!(with(&|l| l.sha256 = Some(bad_sha.into())), Err(GiftError::InvalidSha256));
        }
        assert_eq!(with(&|l| l.expires_at = Some(999)), Err(GiftError::LinkExpired));
    }

    #[test]
    fn reseller_deliveries_are_validated() {
        let redirect = |url: &str| Delivery::ResellerRedirect { url: url.into() }.validate();
        for good in [
            "https://store.example/gift/abc?x=1#y",
            "http://localhost:8081/redeem",
            "HTTPS://STORE.EXAMPLE",
            "https://user:pw@store.example/x",
            "https://[::1]:8443/x",
        ] {
            assert_eq!(redirect(good), Ok(()), "{good:?}");
        }
        let long = format!("https://store.example/{}", "a".repeat(2048));
        for bad in [
            "",
            "store.example/x",
            "//store.example/x",
            "/relative",
            "ftp://store.example/x",
            "javascript:alert(1)",
            "data:text/html,<script>",
            "file:///etc/passwd",
            "https://",
            "https:///path",
            "https://?q",
            "https://:443/x",
            "https://user@/x",
            "https://store.example/a b",
            "https://store.example/\n",
            "https://store.example/\u{0}",
            " https://store.example/",
            "htt",
            "é",
            long.as_str(),
        ] {
            assert_eq!(redirect(bad), Err(GiftError::InvalidUrl), "{bad:?}");
        }

        let licence = |format: LicenceFormat, url: &str| {
            Delivery::ResellerLicence { format, url: url.into() }.validate()
        };
        assert_eq!(licence(LicenceFormat::Lcpl, "https://lcp.example/l/1.lcpl"), Ok(()));
        assert_eq!(licence(LicenceFormat::Acsm, "https://acs.example/f/1"), Ok(()));
        assert_eq!(licence(LicenceFormat::Other("urms".into()), "https://x.example/1"), Ok(()));
        assert_eq!(licence(LicenceFormat::Lcpl, "nope"), Err(GiftError::InvalidUrl));
        for bad in ["", "LCPL", "a b", "../x", "averyveryverylongtoken", "é"] {
            assert_eq!(
                licence(LicenceFormat::Other(bad.into()), "https://x.example/1"),
                Err(GiftError::InvalidLicenceFormat),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn licence_format_strings() {
        assert_eq!(LicenceFormat::from("lcpl"), LicenceFormat::Lcpl);
        assert_eq!(LicenceFormat::from("ACSM"), LicenceFormat::Acsm);
        assert_eq!(LicenceFormat::from("urms"), LicenceFormat::Other("urms".into()));
        assert_eq!(LicenceFormat::Lcpl.to_string(), "lcpl");
        assert_eq!(
            LicenceFormat::Lcpl.media_type(),
            Some("application/vnd.readium.lcp.license.v1.0+json")
        );
        assert_eq!(LicenceFormat::Acsm.media_type(), Some("application/vnd.adobe.adept+xml"));
        assert_eq!(LicenceFormat::Other("urms".into()).media_type(), None);

        let delivery = Delivery::ResellerLicence {
            format: LicenceFormat::Lcpl,
            url: "https://lcp.example/l/1.lcpl".into(),
        };
        let json = serde_json::to_string(&delivery).expect("serialises");
        assert_eq!(
            json,
            r#"{"type":"reseller_licence","format":"lcpl","url":"https://lcp.example/l/1.lcpl"}"#
        );
        assert_eq!(serde_json::from_str::<Delivery>(&json).expect("parses"), delivery);
        let custom: Delivery = serde_json::from_str(
            r#"{"type":"reseller_licence","format":"urms","url":"https://x.example/1"}"#,
        )
        .expect("parses");
        assert_eq!(custom.type_name(), "reseller_licence");
        assert!(
            matches!(custom, Delivery::ResellerLicence { format: LicenceFormat::Other(ref t), .. } if t == "urms")
        );
        assert_eq!(
            Delivery::ResellerRedirect { url: String::new() }.type_name(),
            "reseller_redirect"
        );
        assert_eq!(sample().delivery.type_name(), "partner_file");
    }

    #[test]
    fn claim_url_uses_the_code() {
        let gift = sample();
        let url = gift.claim_url("https://ebk.example/");
        assert_eq!(url, format!("https://ebk.example/k/{}", gift.code.compact()));
        assert_eq!(GiftCode::from_claim_url(&url), Ok(gift.code));
    }

    #[test]
    fn debug_output_does_not_reveal_the_code() {
        let gift = sample();
        let debug = format!("{gift:?}");
        assert!(!debug.contains(&gift.code.compact()));
        assert!(!debug.contains(&gift.code.grouped()));
        assert!(debug.contains("g_abababab"), "the id is not secret");
    }
}
