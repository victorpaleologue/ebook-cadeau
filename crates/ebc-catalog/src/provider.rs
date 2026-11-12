//! The provider contract: what a source is ([`ProviderInfo`]), what it asks the host to
//! fetch and how it reads the answer ([`Provider`]).

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, HttpRequest, HttpResponse, SourceRef};

use crate::error::CatalogError;
use crate::marc::Extracted;
use crate::normalize::normalize_record;

/// The licence class of a source, as defined in `docs/research/01-isbn-databases.md` §5.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceClass {
    /// Class A — open: no key, a licence that permits commercial reuse and storage.
    Open,
    /// Class B — terms-bound: a free API whose terms limit storage, branding or use.
    TermsBound,
    /// Class C — contract: paid or negotiated access.
    Contract,
}

/// Whether the terms of a source allow something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Allowed without further conditions than attribution, if any (CC0, Licence Ouverte,
    /// public domain).
    Allowed,
    /// Allowed under conditions written in the terms (branding, API agreement, time-limited
    /// caching…). Read [`ProviderInfo::terms_url`] before relying on it.
    Conditional,
    /// The terms forbid it.
    Forbidden,
    /// The terms could not be found or do not say.
    Unknown,
}

/// Static description of a source: identity, licence and etiquette.
///
/// The values encode what `docs/research/01-isbn-databases.md` found. That document was not
/// fact-checked and most licence and quota statements in it are marked "from memory, not
/// verified against the operator's live site": [`ProviderInfo::notes`] says so per source,
/// and the crate README repeats the table. All of it lives in one `static` per provider so
/// that a correction is a one-line change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    /// Stable machine identifier, e.g. `"bnf"`. Used in [`SourceRef::provider`].
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Home page of the source.
    pub homepage: &'static str,
    /// Licence of the data, in words.
    pub data_licence: &'static str,
    /// Where the operator publishes its terms of use or API policy.
    pub terms_url: &'static str,
    /// `true` if the source must not be queried without an API key or account, because it
    /// refuses the request or because its terms (or our policy for it) say so.
    pub requires_key: bool,
    /// `true` only if the source is known to send CORS headers that let a browser page call
    /// it directly. `false` means "unknown or no": relay through a server.
    pub browser_cors: bool,
    /// May the data be used in a commercial product?
    pub commercial_use: Permission,
    /// Attribution to display where the data is shown, when the licence or terms ask for one.
    pub attribution: Option<&'static str>,
    /// Minimum delay between two requests to this source, in milliseconds.
    pub min_interval_ms: u32,
    /// Short licence identifier copied into [`SourceRef::licence`] (SPDX where one exists:
    /// `"CC0-1.0"`, `"etalab-2.0"`), or `None` for proprietary terms.
    pub licence_id: Option<&'static str>,
    /// May records be stored permanently (beyond a session cache)?
    pub permanent_storage: Permission,
    /// Hosts this provider sends requests to. Follow-up requests to any other host are
    /// refused by [`Session`](crate::Session); hosts can also use the list for per-host rate
    /// limiting or a Content-Security-Policy.
    pub hosts: &'static [&'static str],
    /// Caveats: what is unverified, quotas, etiquette.
    pub notes: &'static str,
    /// Licence class (A open, B terms-bound, C contract).
    pub class: SourceClass,
    /// Whether [`default_providers`](crate::default_providers) includes the source without
    /// being asked to. Sources the research ranks "optional", "hold" or "skip" are off.
    pub default_enabled: bool,
}

impl ProviderInfo {
    /// A [`SourceRef`] crediting this provider for a record.
    #[must_use]
    pub fn source_ref(&self, record_url: Option<String>) -> SourceRef {
        SourceRef {
            provider: self.id.to_owned(),
            record_url,
            licence: self.licence_id.map(str::to_owned),
            attribution: self.attribution.map(str::to_owned),
        }
    }
}

/// Turns a raw extracted record into the provider's answer for `isbn`: the queried ISBN
/// becomes the record's ISBN (it is never lost), other ISBNs of the record become related
/// ISBNs, the source is credited and every field is normalised.
pub(crate) fn finalize(info: &ProviderInfo, isbn: &Isbn13, extracted: Extracted) -> BookMeta {
    let Extracted { mut book, isbns, url } = extracted;
    book.isbn13 = Some(*isbn);
    for other in isbns {
        if other != *isbn && !book.related_isbns.contains(&other) {
            book.related_isbns.push(other);
        }
    }
    book.sources.push(info.source_ref(url));
    normalize_record(&mut book);
    book
}

/// What a provider made of one HTTP response.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parsed {
    /// Records found in the response.
    ///
    /// A record may be a **fragment**: the answer to a follow-up request often carries only
    /// a few fields (say, one contributor) and leaves `title` empty. All records a provider
    /// returns for one ISBN describe that ISBN and are meant to be combined with
    /// [`merge`](crate::merge) in the order the requests were issued — which is what
    /// [`Session`](crate::Session) and [`drive`](crate::drive) do.
    pub records: Vec<BookMeta>,
    /// Further requests to make. Hand each one back, unchanged, to
    /// [`Provider::parse`] together with its response.
    pub follow_up: Vec<HttpRequest>,
    /// Human-readable remarks that are not errors, e.g. "ignored a record about another
    /// ISBN".
    pub notes: Vec<String>,
}

impl Parsed {
    /// Nothing found, nothing more to ask.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// `true` when there is neither a record nor a follow-up request.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty() && self.follow_up.is_empty()
    }
}

/// A source of book metadata, as a sans-IO state machine.
///
/// A provider never performs I/O. [`lookup`](Provider::lookup) describes the first
/// request(s); the host executes them and passes each response to
/// [`parse`](Provider::parse), which returns records and, for multi-step APIs, follow-up
/// requests to treat the same way.
///
/// Providers hold no per-lookup state. When a follow-up needs context (which author a
/// response is about, say), the provider stores it in the **URL fragment** of the follow-up
/// request (`…#ebc=…`). HTTP clients never transmit fragments, so nothing leaks to the
/// remote service; the host only has to hand the same [`HttpRequest`] back to `parse`.
///
/// `parse` must never panic, whatever the bytes.
pub trait Provider: Send + Sync {
    /// Identity, licence and etiquette of the source.
    fn info(&self) -> &ProviderInfo;

    /// The first request(s) to make for `isbn`.
    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest>;

    /// Interprets the response to a request issued by this provider for `isbn`.
    ///
    /// "Not found" is `Ok` with no record. Records that are about another ISBN than the one
    /// asked for are dropped and mentioned in [`Parsed::notes`].
    ///
    /// # Errors
    /// A [`CatalogError`] when the response is an HTTP error, is not in the expected format
    /// or reports a server-side failure.
    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError>;

    /// Shorthand for `self.info().id`.
    fn id(&self) -> &'static str {
        self.info().id
    }
}

impl<P: Provider + ?Sized> Provider for Box<P> {
    fn info(&self) -> &ProviderInfo {
        (**self).info()
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        (**self).lookup(isbn)
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        (**self).parse(isbn, request, response)
    }
}

impl<P: Provider + ?Sized> Provider for &P {
    fn info(&self) -> &ProviderInfo {
        (**self).info()
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        (**self).lookup(isbn)
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        (**self).parse(isbn, request, response)
    }
}
