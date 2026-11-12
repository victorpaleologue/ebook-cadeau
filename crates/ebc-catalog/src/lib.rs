//! Sans-IO clients for ISBN and book-metadata databases, with record merging.
//!
//! A [`Provider`] never performs I/O: [`Provider::lookup`] describes the HTTP request(s)
//! to make for an ISBN and [`Provider::parse`] turns each response into records and, for
//! multi-step APIs, follow-up requests. The same code therefore runs on a server (with the
//! optional `client` feature, or any HTTP library), in WebAssembly (with `fetch`) and in
//! tests (with fixture files).
//!
//! * [`providers`] — the sources: BnF, DNB, K10plus, Library of Congress (SRU), Open
//!   Library, Google Books, Libris, Finna, lobid, SUDOC, Wikidata, Inventaire, Crossref.
//! * [`drive`] / [`Session`] — run one provider; [`lookup_all`] — run a cascade and merge.
//! * [`merge`] — reconcile the records of several providers; [`normalize`] — clean the
//!   ISBD and MARC artefacts of catalogue strings.
//! * [`default_providers`], [`providers_for`] — the provider order recommended by
//!   `docs/research/01-isbn-databases.md`, configured by [`Config`].
//!
//! ```
//! use ebc_catalog::{Config, LookupOptions, lookup_all, providers_for, wire_url};
//! use ebc_model::{HttpRequest, HttpResponse, Isbn13};
//!
//! let isbn = Isbn13::parse("978-2-07-036002-4").unwrap();
//! let providers = providers_for(&isbn, &Config::default());
//! assert_eq!(providers[0].info().id, "bnf"); // 978-2 is routed to the BnF first
//!
//! // Any transport will do; this one knows a single Open Library answer.
//! let exec = |request: &HttpRequest| -> Result<HttpResponse, String> {
//!     if !wire_url(request).starts_with("https://openlibrary.org/api/books") {
//!         return Err("offline".to_owned());
//!     }
//!     Ok(HttpResponse {
//!         status: 200,
//!         headers: vec![],
//!         body: br#"{"ISBN:9782070360024": {"details": {
//!             "title": "L'étranger", "authors": [{"name": "Albert Camus"}]}}}"#.to_vec(),
//!         final_url: request.url.clone(),
//!     })
//! };
//! let report = lookup_all(&providers, &isbn, exec, &LookupOptions::default());
//! let book = report.merged.unwrap();
//! assert_eq!(book.title, "L'étranger");
//! assert_eq!(book.author_line().as_deref(), Some("Albert Camus"));
//! ```
//!
//! **Status of the data.** No endpoint could be called while this crate was written. Request
//! shapes follow the research document; response parsing follows each API's documented
//! format and hand-written fixtures. Licence and terms statements in [`ProviderInfo`] come
//! from a research document that was not fact-checked. See the crate README.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use ebc_isbn::Isbn13;

#[cfg(all(feature = "client", not(target_arch = "wasm32")))]
pub mod client;
mod compose_table;
mod driver;
mod error;
mod marc;
mod merge;
pub mod normalize;
mod provider;
pub mod providers;
mod util;
mod xml;

pub use driver::{
    LookupOptions, LookupReport, Outcome, ProviderOutcome, Session, drive, is_denied_url,
    lookup_all, wire_url,
};
pub use error::CatalogError;
pub use merge::{combine, merge, same_person};
pub use provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass};

use providers::crossref::Crossref;
use providers::finna::Finna;
use providers::googlebooks::GoogleBooks;
use providers::inventaire::Inventaire;
use providers::libris::Libris;
use providers::lobid::Lobid;
use providers::openlibrary::OpenLibrary;
use providers::sru::SruProvider;
use providers::sudoc::Sudoc;
use providers::wikidata::Wikidata;

/// The `User-Agent` every request should carry: it names the application and where to
/// reach its maintainers, as Open Library, Libris, Crossref and Wikimedia ask.
pub const DEFAULT_USER_AGENT: &str = concat!(
    "ebook-cadeau/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/victorpaleologue/ebook-cadeau)"
);

/// Identifiers of every provider this crate implements, in general priority order:
/// national libraries, union catalogues, Open Library, Google Books, then the sources
/// that are off by default.
pub const PROVIDER_IDS: [&str; 13] = [
    "bnf",
    "dnb",
    "k10plus",
    "lobid",
    "libris",
    "finna",
    "openlibrary",
    "googlebooks",
    "sudoc",
    "loc",
    "inventaire",
    "wikidata",
    "crossref",
];

/// Hosts no provider, relay or fetcher of this project may contact (research §1.4 and
/// implication 37): their terms or the law bar our use. Subdomains are included.
pub const DENIED_HOSTS: [&str; 12] = [
    "goodreads.com",
    "librarything.com",
    "babelio.com",
    "fnac.com",
    "kobo.com",
    "kobobooks.com",
    "decitre.fr",
    "worldcat.org",
    "grp.isbn-international.org",
    "afnil.org",
    "media-amazon.com",
    "ssl-images-amazon.com",
];

/// `true` when `host` is on the deny-list: one of [`DENIED_HOSTS`] or a subdomain of one,
/// or any Amazon retail domain (`amazon.fr`, `www.amazon.co.uk`, `images-eu.amazon.com`…).
#[must_use]
pub fn is_denied_host(host: &str) -> bool {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    let listed = DENIED_HOSTS
        .iter()
        .any(|denied| host == *denied || host.ends_with(&format!(".{denied}")));
    // "amazon" as a whole label, except Amazon Web Services hosting.
    let amazon = host.split('.').any(|label| label == "amazon") && !host.ends_with("amazonaws.com");
    listed || amazon
}

/// Which providers to use and how to reach them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// Google Books API key. Without one Google Books is disabled (not failing).
    pub google_books_key: Option<String>,
    /// Contact e-mail sent to Crossref as `mailto` (polite pool).
    pub contact_email: Option<String>,
    /// Base-URL overrides by provider id: tests and the CLI point providers at a local
    /// fixture server. For SRU and SPARQL providers the value is the full endpoint URL;
    /// for the others it is the site root.
    pub base_urls: BTreeMap<String, String>,
    /// Providers to switch on although they are off by default (`"sudoc"`, `"loc"`,
    /// `"inventaire"`, `"wikidata"`, `"crossref"`).
    pub enable: Vec<String>,
    /// Providers to switch off.
    pub disable: Vec<String>,
}

impl Config {
    /// The default configuration: every default-enabled keyless provider, real endpoints.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Google Books API key, which enables Google Books.
    #[must_use]
    pub fn with_google_books_key(mut self, key: impl Into<String>) -> Self {
        self.google_books_key = Some(key.into()).filter(|k: &String| !k.trim().is_empty());
        self
    }

    /// Sets the contact e-mail given to services that ask for one.
    #[must_use]
    pub fn with_contact_email(mut self, email: impl Into<String>) -> Self {
        self.contact_email = Some(email.into());
        self
    }

    /// Points a provider at another base URL.
    #[must_use]
    pub fn with_base_url(mut self, provider: impl Into<String>, url: impl Into<String>) -> Self {
        self.base_urls.insert(provider.into(), url.into());
        self
    }

    /// Switches on a provider that is off by default.
    #[must_use]
    pub fn with_enabled(mut self, provider: impl Into<String>) -> Self {
        let provider = provider.into();
        self.disable.retain(|p| *p != provider);
        if !self.enable.contains(&provider) {
            self.enable.push(provider);
        }
        self
    }

    /// Switches a provider off.
    #[must_use]
    pub fn with_disabled(mut self, provider: impl Into<String>) -> Self {
        let provider = provider.into();
        self.enable.retain(|p| *p != provider);
        if !self.disable.contains(&provider) {
            self.disable.push(provider);
        }
        self
    }

    /// Whether the provider described by `info` is in use under this configuration. A
    /// provider that needs a key is off until the key is configured, whatever `enable`
    /// says.
    #[must_use]
    pub fn is_enabled(&self, info: &ProviderInfo) -> bool {
        if self.disable.iter().any(|p| p == info.id) {
            return false;
        }
        if info.requires_key && !(info.id == "googlebooks" && self.google_books_key.is_some()) {
            return false;
        }
        info.default_enabled || self.enable.iter().any(|p| p == info.id)
    }
}

/// Builds the provider called `id` (one of [`PROVIDER_IDS`]) with the base URL and
/// credentials of `config`, whether or not the configuration enables it.
#[must_use]
pub fn provider_by_id(id: &str, config: &Config) -> Option<Box<dyn Provider>> {
    let base = config.base_urls.get(id).map(String::as_str).filter(|u| !u.trim().is_empty());
    macro_rules! build {
        ($provider:expr) => {{
            let provider = $provider;
            Box::new(match base {
                Some(url) => provider.with_base_url(url),
                None => provider,
            }) as Box<dyn Provider>
        }};
    }
    Some(match id {
        "bnf" => build!(SruProvider::bnf()),
        "dnb" => build!(SruProvider::dnb()),
        "k10plus" => build!(SruProvider::k10plus()),
        "loc" => build!(SruProvider::loc()),
        "lobid" => build!(Lobid::new()),
        "libris" => build!(Libris::new()),
        "finna" => build!(Finna::new()),
        "openlibrary" => build!(OpenLibrary::new()),
        "googlebooks" => build!(GoogleBooks::new(config.google_books_key.clone())),
        "sudoc" => build!(Sudoc::new()),
        "inventaire" => build!(Inventaire::new()),
        "wikidata" => build!(Wikidata::new()),
        "crossref" => build!(Crossref::new(config.contact_email.clone())),
        _ => return None,
    })
}

/// The static description of every implemented provider, in [`PROVIDER_IDS`] order.
#[must_use]
pub fn provider_infos() -> Vec<ProviderInfo> {
    let config = Config::default();
    PROVIDER_IDS
        .iter()
        .filter_map(|id| provider_by_id(id, &config))
        .map(|provider| provider.info().clone())
        .collect()
}

fn build_enabled(ids: &[&str], config: &Config) -> Vec<Box<dyn Provider>> {
    let mut seen: Vec<&str> = Vec::new();
    ids.iter()
        .filter(|id| {
            let fresh = !seen.contains(*id);
            seen.push(**id);
            fresh
        })
        .filter_map(|id| provider_by_id(id, config))
        .filter(|provider| config.is_enabled(provider.info()))
        .collect()
}

/// Every provider the configuration enables, in the general priority order of
/// `docs/research/01-isbn-databases.md` §1.2: national libraries and union catalogues,
/// Open Library, Google Books (only with a key), then the optional sources that were
/// switched on.
///
/// When the ISBN is known, prefer [`providers_for`], which keeps only the national
/// library of the ISBN's registration group.
#[must_use]
pub fn default_providers(config: &Config) -> Vec<Box<dyn Provider>> {
    build_enabled(&PROVIDER_IDS, config)
}

/// The providers to query for `isbn`, in order: the national library (and its back-ups)
/// of the ISBN's registration group, then Open Library, then Google Books when a key is
/// configured, then the optional sources that were switched on (research §3.1).
///
/// | Registration group | First |
/// |---|---|
/// | 978-2 (French language), 979-10 (France) | BnF; SUDOC if enabled |
/// | 978-3 (German language) | DNB, then K10plus and lobid |
/// | 978-91 (Sweden) | Libris |
/// | 978-951, 978-952 (Finland) | Finna |
/// | 978-0, 978-1, 979-8 (English language, United States) | Library of Congress if enabled |
#[must_use]
pub fn providers_for(isbn: &Isbn13, config: &Config) -> Vec<Box<dyn Provider>> {
    let group = isbn.registration_group().map(|g| g.prefix);
    let mut ids: Vec<&str> = match group {
        Some("978-2" | "979-10") => vec!["bnf"],
        Some("978-3") => vec!["dnb", "k10plus", "lobid"],
        Some("978-91") => vec!["libris"],
        Some("978-951" | "978-952") => vec!["finna"],
        _ => Vec::new(),
    };
    ids.extend(["openlibrary", "googlebooks"]);
    match group {
        Some("978-2" | "979-10") => ids.push("sudoc"),
        Some("978-0" | "978-1" | "979-8") => ids.push("loc"),
        _ => {}
    }
    ids.extend(["inventaire", "wikidata", "crossref"]);
    build_enabled(&ids, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(providers: &[Box<dyn Provider>]) -> Vec<&'static str> {
        providers.iter().map(|p| p.info().id).collect()
    }

    #[test]
    fn default_order_follows_the_research() {
        assert_eq!(
            ids(&default_providers(&Config::default())),
            ["bnf", "dnb", "k10plus", "lobid", "libris", "finna", "openlibrary"]
        );
        let all = Config::default()
            .with_google_books_key("k")
            .with_enabled("sudoc")
            .with_enabled("loc")
            .with_enabled("inventaire")
            .with_enabled("wikidata")
            .with_enabled("crossref");
        assert_eq!(ids(&default_providers(&all)), PROVIDER_IDS);
        let fewer = Config::default().with_disabled("dnb").with_enabled("googlebooks");
        assert!(!ids(&default_providers(&fewer)).contains(&"dnb"));
        assert!(!ids(&default_providers(&fewer)).contains(&"googlebooks"), "no key, no Google");
    }

    #[test]
    fn isbns_are_routed_by_registration_group() {
        let config = Config::default().with_google_books_key("k");
        let route = |isbn: &str, config: &Config| {
            ids(&providers_for(&Isbn13::parse(isbn).expect("valid"), config))
        };
        assert_eq!(route("9782070360024", &config), ["bnf", "openlibrary", "googlebooks"]);
        assert_eq!(route("9791032102763", &config), ["bnf", "openlibrary", "googlebooks"]);
        assert_eq!(
            route("9783596294312", &config),
            ["dnb", "k10plus", "lobid", "openlibrary", "googlebooks"]
        );
        assert_eq!(route("9789100123451", &config), ["libris", "openlibrary", "googlebooks"]);
        assert_eq!(route("9789510123454", &config), ["finna", "openlibrary", "googlebooks"]);
        assert_eq!(route("9780141439518", &Config::default()), ["openlibrary"]);
        let wide = config.with_enabled("sudoc").with_enabled("loc").with_enabled("wikidata");
        assert_eq!(
            route("9782070360024", &wide),
            ["bnf", "openlibrary", "googlebooks", "sudoc", "wikidata"]
        );
        assert_eq!(
            route("9780141439518", &wide),
            ["openlibrary", "googlebooks", "loc", "wikidata"]
        );
    }

    #[test]
    fn every_provider_is_described() {
        let infos = provider_infos();
        assert_eq!(infos.len(), PROVIDER_IDS.len());
        for (info, id) in infos.iter().zip(PROVIDER_IDS) {
            assert_eq!(info.id, id);
            assert!(info.homepage.starts_with("https://"), "{id}");
            assert!(info.terms_url.starts_with("https://"), "{id}");
            assert!(!info.hosts.is_empty() && !info.notes.is_empty(), "{id}");
            assert!(info.min_interval_ms >= 1000, "{id}: at most one request per second");
            assert!(info.hosts.iter().all(|h| !is_denied_host(h)), "{id}");
        }
        assert!(provider_by_id("amazon", &Config::default()).is_none());
    }

    #[test]
    fn excluded_hosts_are_denied() {
        for host in [
            "www.amazon.fr",
            "amazon.com",
            "images-eu.amazon.co.uk",
            "m.media-amazon.com",
            "www.goodreads.com",
            "covers.librarything.com",
            "www.babelio.com",
            "FNAC.com.",
            "www.kobo.com",
            "www.decitre.fr",
            "search.worldcat.org",
            "grp.isbn-international.org",
        ] {
            assert!(is_denied_host(host), "{host}");
        }
        for host in ["catalogue.bnf.fr", "openlibrary.org", "s3.amazonaws.com", "notfnac.com"] {
            assert!(!is_denied_host(host), "{host}");
        }
        assert!(is_denied_url("https://www.amazon.fr/dp/2070360024"));
        assert!(!is_denied_url("https://openlibrary.org/isbn/2070360024.json"));
    }
}
