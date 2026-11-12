//! Running a provider's state machine: [`Session`] for hosts that do their own (possibly
//! asynchronous) I/O, [`drive`] for a synchronous executor, [`lookup_all`] for a whole
//! provider cascade.

use std::collections::VecDeque;
use std::fmt::Display;

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, ContributorRole, HttpRequest, HttpResponse};

use crate::error::{CatalogError, shorten};
use crate::merge::{combine, merge};
use crate::provider::Provider;
use crate::util::{host, origin, redact, without_fragment};
use crate::{is_denied_host, util};

/// What one provider found for one ISBN.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Outcome {
    /// The record, assembled from every response; `None` when the source does not know
    /// the ISBN.
    pub record: Option<BookMeta>,
    /// Remarks that are not fatal: ignored records, failed follow-up requests…
    pub notes: Vec<String>,
    /// Number of requests issued.
    pub requests: usize,
    /// `true` when follow-up requests were dropped because the request budget ran out.
    pub truncated: bool,
}

/// The URL to put on the wire for a request: providers keep private context in the URL
/// fragment, which HTTP never transmits. `reqwest` and browsers' `fetch` drop it by
/// themselves; fixture maps should be keyed by this.
#[must_use]
pub fn wire_url(request: &HttpRequest) -> &str {
    without_fragment(&request.url)
}

/// One lookup with one provider, as a sans-IO state machine.
///
/// ```
/// use ebc_catalog::{Session, providers::openlibrary::OpenLibrary};
/// use ebc_model::{HttpResponse, Isbn13};
///
/// let isbn = Isbn13::parse("9782070360024").unwrap();
/// let provider = OpenLibrary::new();
/// let mut session = Session::new(&provider, &isbn, 6);
/// while let Some(request) = session.next_request() {
///     // Execute `request` with any transport, then:
///     let response = HttpResponse {
///         status: 200,
///         headers: vec![],
///         body: b"{}".to_vec(), // Open Library's "not found"
///         final_url: request.url.clone(),
///     };
///     session.feed(&request, &response);
/// }
/// assert!(session.finish().unwrap().record.is_none());
/// ```
pub struct Session<'a, P: Provider + ?Sized> {
    provider: &'a P,
    isbn: Isbn13,
    queue: VecDeque<HttpRequest>,
    issued: Vec<String>,
    origins: Vec<String>,
    max_requests: usize,
    records: Vec<BookMeta>,
    notes: Vec<String>,
    first_error: Option<CatalogError>,
    truncated: bool,
}

impl<'a, P: Provider + ?Sized> Session<'a, P> {
    /// Starts a lookup. At most `max_requests` requests will be handed out.
    #[must_use]
    pub fn new(provider: &'a P, isbn: &Isbn13, max_requests: usize) -> Self {
        let queue: VecDeque<HttpRequest> = provider.lookup(isbn).into();
        // Follow-ups may only go where the first requests went (which covers base-URL
        // overrides) or to the provider's declared hosts.
        let mut origins: Vec<String> = queue.iter().filter_map(|r| origin(&r.url)).collect();
        origins.dedup();
        Self {
            provider,
            isbn: *isbn,
            queue,
            issued: Vec::new(),
            origins,
            max_requests,
            records: Vec::new(),
            notes: Vec::new(),
            first_error: None,
            truncated: false,
        }
    }

    fn allowed(&self, url: &str) -> bool {
        let Some(request_host) = host(url) else { return false };
        if is_denied_host(&request_host) {
            return false;
        }
        origin(url).is_some_and(|o| self.origins.contains(&o))
            || (url.starts_with("https://")
                && self.provider.info().hosts.iter().any(|h| *h == request_host))
    }

    fn record_error(&mut self, error: CatalogError) {
        self.notes.push(error.to_string());
        self.first_error.get_or_insert(error);
    }

    /// The next request to execute, or `None` when the lookup is over. Requests may be
    /// executed one at a time or concurrently; give every response back with
    /// [`feed`](Self::feed) (or [`fail`](Self::fail)) before calling [`finish`](Self::finish).
    pub fn next_request(&mut self) -> Option<HttpRequest> {
        while let Some(request) = self.queue.pop_front() {
            if self.issued.iter().any(|url| *url == request.url) {
                continue;
            }
            if !self.allowed(&request.url) {
                let error = CatalogError::ForbiddenFollowUp {
                    provider: self.provider.info().id,
                    url: redact(&request.url),
                };
                self.record_error(error);
                continue;
            }
            if self.issued.len() >= self.max_requests {
                self.truncated = true;
                self.queue.clear();
                self.notes.push(format!(
                    "stopped after {} requests; further follow-up requests were dropped",
                    self.max_requests
                ));
                return None;
            }
            self.issued.push(request.url.clone());
            return Some(request);
        }
        None
    }

    /// Gives the response to a request obtained from [`next_request`](Self::next_request).
    pub fn feed(&mut self, request: &HttpRequest, response: &HttpResponse) {
        match self.provider.parse(&self.isbn, request, response) {
            Ok(parsed) => {
                self.records.extend(parsed.records);
                self.notes.extend(parsed.notes);
                self.queue.extend(parsed.follow_up);
            }
            Err(error) => self.record_error(error),
        }
    }

    /// Reports that a request could not be executed at all.
    pub fn fail(&mut self, request: &HttpRequest, message: &str) {
        let error = CatalogError::Transport {
            provider: self.provider.info().id,
            message: shorten(&format!("{}: {message}", redact(&request.url))),
        };
        self.record_error(error);
    }

    /// Ends the lookup.
    ///
    /// # Errors
    /// The first error met, when no record at all could be read. When at least one
    /// response gave a record, later failures are only mentioned in [`Outcome::notes`].
    pub fn finish(self) -> Result<Outcome, CatalogError> {
        let record = combine(&self.records).filter(|r| !r.title.trim().is_empty());
        match (record, self.first_error) {
            (None, Some(error)) => Err(error),
            (record, _) => Ok(Outcome {
                record,
                notes: self.notes,
                requests: self.issued.len(),
                truncated: self.truncated,
            }),
        }
    }
}

/// Runs one provider to completion with a synchronous executor.
///
/// `exec` performs one HTTP request — real I/O, or a lookup in a fixture map (key it by
/// [`wire_url`]). Follow-up requests are executed in the order the provider asked for
/// them, never more than `max_requests` in total.
///
/// # Errors
/// The first error met ([`CatalogError::Transport`] for errors of `exec`), when no record
/// could be read. See [`Session::finish`].
pub fn drive<P, E, F>(
    provider: &P,
    isbn: &Isbn13,
    mut exec: F,
    max_requests: usize,
) -> Result<Outcome, CatalogError>
where
    P: Provider + ?Sized,
    E: Display,
    F: FnMut(&HttpRequest) -> Result<HttpResponse, E>,
{
    let mut session = Session::new(provider, isbn, max_requests);
    while let Some(request) = session.next_request() {
        match exec(&request) {
            Ok(response) => session.feed(&request, &response),
            Err(error) => session.fail(&request, &error.to_string()),
        }
    }
    session.finish()
}

/// How [`lookup_all`] walks the provider list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupOptions {
    /// Request budget per provider (follow-ups included).
    pub max_requests_per_provider: usize,
    /// Number of providers, from the head of the list, that are always queried — the
    /// routed national library and Open Library in the default order.
    pub always_query: usize,
    /// Stop as soon as the merged record has a title and an author, once the first
    /// `always_query` providers have been asked. `false` queries every provider.
    pub stop_when_complete: bool,
}

impl Default for LookupOptions {
    fn default() -> Self {
        Self { max_requests_per_provider: 6, always_query: 2, stop_when_complete: true }
    }
}

/// What one provider contributed to a [`LookupReport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderOutcome {
    /// Provider identifier.
    pub provider: &'static str,
    /// The provider's answer, or why there is none.
    pub result: Result<Outcome, CatalogError>,
}

/// The result of [`lookup_all`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookupReport {
    /// The merged record; `None` when no provider knows the ISBN.
    pub merged: Option<BookMeta>,
    /// One entry per provider that was queried, in order.
    pub providers: Vec<ProviderOutcome>,
    /// Providers that were not queried because the record was already complete.
    pub skipped: Vec<&'static str>,
}

impl LookupReport {
    /// The records found, in provider order.
    pub fn records(&self) -> impl Iterator<Item = (&'static str, &BookMeta)> {
        self.providers.iter().filter_map(|p| {
            p.result.as_ref().ok().and_then(|o| o.record.as_ref()).map(|r| (p.provider, r))
        })
    }

    /// The errors met, in provider order.
    pub fn errors(&self) -> impl Iterator<Item = &CatalogError> {
        self.providers.iter().filter_map(|p| p.result.as_ref().err())
    }
}

/// A record is complete enough to stop the cascade when it has a title and an author.
pub(crate) fn is_complete(book: &BookMeta) -> bool {
    !book.title.trim().is_empty()
        && book.contributors.iter().any(|c| c.role == ContributorRole::Author)
}

/// Queries `providers` in order for `isbn` and merges what they return.
///
/// The list is in priority order (see [`providers_for`](crate::providers_for)). A failing
/// provider never fails the lookup: its error is reported in the
/// [`LookupReport`] and the next provider is tried.
pub fn lookup_all<E, F>(
    providers: &[Box<dyn Provider>],
    isbn: &Isbn13,
    mut exec: F,
    options: &LookupOptions,
) -> LookupReport
where
    E: Display,
    F: FnMut(&HttpRequest) -> Result<HttpResponse, E>,
{
    let mut report = LookupReport::default();
    let mut records: Vec<BookMeta> = Vec::new();
    for (index, provider) in providers.iter().enumerate() {
        let id = provider.info().id;
        if options.stop_when_complete
            && index >= options.always_query
            && merge(&records).as_ref().is_some_and(is_complete)
        {
            report.skipped.push(id);
            continue;
        }
        let result = drive(provider, isbn, &mut exec, options.max_requests_per_provider);
        if let Ok(Outcome { record: Some(record), .. }) = &result {
            records.push(record.clone());
        }
        report.providers.push(ProviderOutcome { provider: id, result });
    }
    report.merged = merge(&records);
    report
}

/// `true` when `url` targets a host this project must never contact.
#[must_use]
pub fn is_denied_url(url: &str) -> bool {
    util::host(url).is_some_and(|h| is_denied_host(&h))
}
