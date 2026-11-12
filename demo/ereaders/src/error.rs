//! Error types.
//!
//! The crate exposes one top-level [`Error`] enum (as every `ebc-*` crate does); each component
//! contributes a nested, typed enum so tests can assert *why* something failed, e.g.
//! `Error::Import(ImportError::UnsupportedFormat { .. })` when a Kindle is handed an EPUB.

use crate::format::FileFormat;
use crate::profile::DrmScheme;

/// Convenience alias used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Top-level error of the simulated e-readers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Transport-level failure (URL, connection, timeout, redirect, size cap).
    #[error(transparent)]
    Net(#[from] NetError),
    /// The simulated browser could not do what the "person" asked.
    #[error(transparent)]
    Browser(#[from] BrowserError),
    /// A file reached the device but could not be added to its library.
    #[error(transparent)]
    Import(#[from] ImportError),
    /// OPDS feed problems.
    #[error(transparent)]
    Opds(#[from] OpdsError),
    /// Ecosystem-account sync problems.
    #[error(transparent)]
    Sync(#[from] SyncError),
    /// Simulated LCP flow problems.
    #[error(transparent)]
    Lcp(#[from] LcpError),
    /// Mail-to-device simulation problems (socket level).
    #[error(transparent)]
    Mail(#[from] MailError),
}

/// Transport-level failures. Every network operation in this crate is bounded in time and size,
/// so hostile servers produce one of these instead of a hang or an out-of-memory.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NetError {
    /// The URL could not be parsed or resolved.
    #[error("invalid URL {url:?}: {reason}")]
    InvalidUrl {
        /// The offending input.
        url: String,
        /// Parser message.
        reason: String,
    },
    /// Only `http` and `https` are fetched.
    #[error("unsupported URL scheme {scheme:?} in {url}")]
    UnsupportedScheme {
        /// The URL.
        url: String,
        /// Its scheme.
        scheme: String,
    },
    /// The HTTP client could not be constructed.
    #[error("cannot build the HTTP client: {0}")]
    Client(String),
    /// Connection or protocol failure.
    #[error("request to {url} failed: {reason}")]
    Request {
        /// The URL being fetched.
        url: String,
        /// Underlying error text.
        reason: String,
    },
    /// The exchange did not finish within the configured time.
    #[error("request to {url} timed out")]
    Timeout {
        /// The URL being fetched.
        url: String,
    },
    /// The redirect chain exceeded the bound.
    #[error("more than {limit} redirects starting at {url}")]
    TooManyRedirects {
        /// First URL of the chain.
        url: String,
        /// The configured bound.
        limit: usize,
    },
    /// A 3xx response without a usable `Location`.
    #[error("redirect from {url} has no usable Location header")]
    BadRedirect {
        /// The URL that answered 3xx.
        url: String,
    },
    /// The body is larger than the consumer tolerates.
    #[error("response from {url} is larger than {limit} bytes")]
    BodyTooLarge {
        /// The URL being fetched.
        url: String,
        /// The cap that was hit.
        limit: u64,
    },
    /// A non-success status where a document or file was required.
    #[error("HTTP {status} from {url}")]
    Status {
        /// The URL.
        url: String,
        /// The status code.
        status: u16,
    },
}

/// Things the simulated browser refuses or cannot do.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BrowserError {
    /// No page is displayed yet.
    #[error("no page is open")]
    NoPage,
    /// Nothing on the page matches the id / name / visible text.
    #[error("nothing on the page matches {query:?}")]
    NotFound {
        /// What the caller looked for.
        query: String,
    },
    /// Several elements match the visible text; a person could not tell which one is meant.
    #[error("{count} elements match {query:?}; use an id")]
    Ambiguous {
        /// What the caller looked for.
        query: String,
        /// How many elements match.
        count: usize,
    },
    /// The element exists but tapping it does nothing in a browser.
    #[error("{element} is not something a person can activate")]
    NotActionable {
        /// Short description of the element.
        element: String,
    },
    /// The element only works with JavaScript, which this browser never runs.
    #[error("{element} needs JavaScript ({reason}); e-reader flows must work without it")]
    NeedsJavaScript {
        /// Short description of the element.
        element: String,
        /// Why script is needed.
        reason: String,
    },
    /// The field cannot receive typed text (hidden, disabled, read-only, a button…).
    #[error("{field} cannot be typed into: {reason}")]
    NotTypeable {
        /// Short description of the field.
        field: String,
        /// Why.
        reason: String,
    },
    /// The form cannot be submitted by this browser.
    #[error("form {form} cannot be submitted: {reason}")]
    FormNotSubmittable {
        /// Short description of the form.
        form: String,
        /// Why.
        reason: String,
    },
    /// A link uses a scheme the built-in browser does not handle (`mailto:`, `opds:`…).
    #[error("the browser cannot open {scheme}: links ({href})")]
    UnsupportedLinkScheme {
        /// The scheme.
        scheme: String,
        /// The raw `href`.
        href: String,
    },
    /// The device's browser will not save this kind of file.
    #[error("the {device} browser refuses to download {file_name:?}: {reason}")]
    DownloadRefused {
        /// Profile id.
        device: &'static str,
        /// File name the server proposed.
        file_name: String,
        /// The format the browser inferred from the response headers, if any.
        format: Option<FileFormat>,
        /// Why it was refused.
        reason: DownloadRefusal,
    },
    /// The page (HTML) is heavier than the device's browser tolerates.
    #[error("page {url} weighs more than the {limit} bytes this browser tolerates")]
    PageTooHeavy {
        /// The page URL.
        url: String,
        /// The profile's hard limit.
        limit: usize,
    },
    /// Non-standard port while the profile's port quirk is enforced.
    #[error("{device} browsers are unreliable on non-standard port {port}")]
    NonStandardPort {
        /// Profile id.
        device: &'static str,
        /// The port.
        port: u16,
    },
    /// `back()` with no previous page.
    #[error("there is no previous page")]
    NoHistory,
    /// The page has no `<meta http-equiv="refresh">`.
    #[error("the page declares no meta refresh")]
    NoMetaRefresh,
    /// A CSS selector could not be parsed.
    #[error("invalid CSS selector {selector:?}")]
    BadSelector {
        /// The selector.
        selector: String,
    },
}

/// Why a browser refused a download before saving it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DownloadRefusal {
    /// The file type is not one this browser saves (e.g. EPUB on a Kindle).
    UnsupportedType,
    /// The response does not tell what the file is (no usable type or extension).
    UnknownType,
    /// The file name contains characters the browser cannot handle.
    FileName,
}

impl std::fmt::Display for DownloadRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnsupportedType => "this browser does not download that file type",
            Self::UnknownType => "the response does not say what kind of file this is",
            Self::FileName => "the file name contains characters this browser cannot handle",
        })
    }
}

/// A file reached the device but was not added to the library.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ImportError {
    /// The reading system (or the delivery service) does not take this format.
    #[error("{device} cannot open {format:?} files")]
    UnsupportedFormat {
        /// The detected format.
        format: FileFormat,
        /// Profile or service id.
        device: &'static str,
    },
    /// The bytes are not what the server said they were.
    #[error("declared {declared:?} but the content looks like {actual:?}")]
    ContentMismatch {
        /// Format announced by headers / file name.
        declared: FileFormat,
        /// Format found by looking at the bytes (`None`: unrecognised).
        actual: Option<FileFormat>,
    },
    /// The bytes are not any known book format.
    #[error("the content is not a recognised book file")]
    Unrecognized,
    /// Structurally broken EPUB.
    #[error("invalid EPUB: {0}")]
    InvalidEpub(String),
    /// Structurally broken PDF.
    #[error("invalid PDF: {0}")]
    InvalidPdf(String),
    /// Structurally broken file of another format.
    #[error("invalid {format:?} file: {reason}")]
    InvalidFile {
        /// The format.
        format: FileFormat,
        /// Why.
        reason: String,
    },
    /// The file carries DRM this device / channel cannot unlock.
    #[error("the file is protected with {scheme:?}, which this device cannot open this way")]
    DrmProtected {
        /// The scheme detected in the container.
        scheme: DrmScheme,
    },
    /// The file exceeds the device or service size limit.
    #[error("file of {size} bytes exceeds the {limit} byte limit")]
    TooLarge {
        /// Actual size.
        size: u64,
        /// The limit.
        limit: u64,
    },
    /// Empty file.
    #[error("the file is empty")]
    Empty,
    /// Device quirk: the download completes but the reading system reports the file as damaged.
    #[error("{device} reports the downloaded file as damaged (known firmware quirk)")]
    DamagedOnDevice {
        /// Profile id.
        device: &'static str,
    },
    /// The delivery service refuses the attachment's file name.
    #[error("the service cannot handle the file name {file_name:?}: {reason}")]
    BadFileName {
        /// The name.
        file_name: String,
        /// Why.
        reason: &'static str,
    },
}

/// OPDS client errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum OpdsError {
    /// The document is neither an OPDS 1.x Atom feed/entry nor an OPDS 2.0 JSON feed.
    #[error("not an OPDS feed: {0}")]
    NotOpds(String),
    /// The XML or JSON is malformed.
    #[error("malformed OPDS document: {0}")]
    Malformed(String),
    /// The publication has no acquisition link of an acceptable type.
    #[error("no acquisition link the reader can use (wanted one of {wanted:?})")]
    NoAcquisition {
        /// Media types the reader would have accepted.
        wanted: Vec<String>,
    },
}

/// Ecosystem-account sync errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SyncError {
    /// `sync()` before a successful `link()`.
    #[error("the device is not linked to an account")]
    NotLinked,
    /// The store rejected the e-mail / password.
    #[error("the store rejected the account credentials (HTTP {status})")]
    BadCredentials {
        /// Status code returned by the store.
        status: u16,
    },
    /// The device token is no longer accepted.
    #[error("the store no longer accepts this device (HTTP {status})")]
    Unauthorized {
        /// Status code returned by the store.
        status: u16,
    },
    /// The store answered something that is not the documented JSON.
    #[error("unexpected answer from the store: {0}")]
    Protocol(String),
}

/// Errors of the **simulated** LCP reader.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LcpError {
    /// The device profile has no LCP-capable reading system.
    #[error("{device} has no LCP-capable reading system")]
    DeviceLacksLcp {
        /// Profile id.
        device: &'static str,
    },
    /// The licence document is not valid JSON of the expected shape.
    #[error("malformed licence document: {0}")]
    MalformedLicence(String),
    /// The licence has no `publication` link.
    #[error("the licence has no publication link")]
    NoPublicationLink,
    /// The passphrase does not open this licence. Carries the hint a reading system must show.
    #[error("wrong passphrase (hint: {hint:?})")]
    WrongPassphrase {
        /// `encryption.user_key.text_hint` of the licence.
        hint: String,
    },
    /// The licence status document says the licence cannot be used.
    #[error("the licence is {status}")]
    NotUsable {
        /// `revoked`, `returned`, `cancelled` or `expired`.
        status: String,
    },
    /// The downloaded publication does not match the licence (`length` / `hash`).
    #[error("the publication does not match the licence: {0}")]
    PublicationMismatch(String),
    /// The licence cannot be checked by this simulation.
    #[error("this simulation cannot verify the licence: {0}")]
    Unverifiable(String),
}

/// Mail-to-device simulation errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MailError {
    /// The SMTP listener could not be started.
    #[error("cannot start the SMTP listener: {0}")]
    Bind(#[source] std::io::Error),
    /// The device address is not a plausible e-mail address for the service.
    #[error("{address:?} is not a valid device address for {service}")]
    BadDeviceAddress {
        /// The address.
        address: String,
        /// Service name.
        service: &'static str,
    },
    /// No held mail with that id.
    #[error("no held mail with id {0}")]
    UnknownMail(u64),
}
