//! Crossref REST API: scholarly books that have a DOI.

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, Contributor, ContributorRole, HttpRequest, HttpResponse};
use serde_json::Value;

use crate::error::{CatalogError, shorten};
use crate::marc::Extracted;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{get_json, unknown_request};
use crate::util::{
    Found, check_http, isbns_in, jarr, join, jstr, jtexts, parse_json, path, push_isbn,
    url_with_query,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "crossref",
    name: "Crossref REST API",
    homepage: "https://www.crossref.org/",
    data_licence: "No ownership claimed on individual items of bibliographic metadata",
    terms_url: "https://www.crossref.org/documentation/retrieve-metadata/rest-api/",
    requires_key: false,
    browser_cors: true,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: None,
    permanent_storage: Permission::Allowed,
    hosts: &["api.crossref.org"],
    notes: "Off by default (scholarly books only). \"May be cached and incorporated\" read in \
            Crossref's REST documentation on GitHub, which is marked deprecated; the terms \
            URL and CORS are from memory. Give a contact e-mail (`mailto`) to use the polite \
            pool; limits are announced in X-Rate-Limit-* headers and were lowered in \
            December 2025 (from memory).",
    class: SourceClass::Open,
    default_enabled: false,
};

/// Work types that are whole books (chapters and series are not).
const BOOK_TYPES: [&str; 5] = ["book", "monograph", "edited-book", "reference-book", "book-set"];

/// Crossref: `GET {base}/works?filter=isbn:{isbn}&rows=5[&mailto={contact}]`.
#[derive(Debug, Clone)]
pub struct Crossref {
    base: String,
    mailto: Option<String>,
}

impl Default for Crossref {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Crossref {
    /// The provider pointed at `https://api.crossref.org`; `mailto` is the contact address
    /// that routes requests to the polite pool.
    #[must_use]
    pub fn new(mailto: Option<String>) -> Self {
        Self {
            base: "https://api.crossref.org".to_owned(),
            mailto: mailto.map(|m| m.trim().to_owned()).filter(|m| m.contains('@')),
        }
    }

    /// Replaces the API root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }
}

fn people(item: &Value, key: &str, role: &ContributorRole, book: &mut BookMeta) {
    for person in jarr(item, key) {
        let name = match (jstr(person, "given"), jstr(person, "family")) {
            (Some(given), Some(family)) => format!("{given} {family}"),
            (None, Some(family)) => family.to_owned(),
            _ => match jstr(person, "name") {
                Some(name) => name.to_owned(),
                None => continue,
            },
        };
        book.contributors.push(Contributor { name, role: role.clone() });
    }
}

/// `{"date-parts": [[2019, 5, 2]]}` → `"2019-05-02"`.
fn date_parts(value: Option<&Value>) -> Option<String> {
    let parts = value?.get("date-parts")?.as_array()?.first()?.as_array()?;
    let number =
        |i: usize, max: u64| parts.get(i).and_then(Value::as_u64).filter(|n| (1..=max).contains(n));
    let year = number(0, 2100).filter(|y| *y >= 1000)?;
    Some(match (number(1, 12), number(2, 31)) {
        (Some(month), Some(day)) => format!("{year:04}-{month:02}-{day:02}"),
        (Some(month), None) => format!("{year:04}-{month:02}"),
        _ => format!("{year:04}"),
    })
}

fn work(item: &Value) -> Option<Extracted> {
    if !jstr(item, "type").is_some_and(|t| BOOK_TYPES.contains(&t)) {
        return None;
    }
    let mut isbns = Vec::new();
    let typed = jarr(item, "isbn-type").iter().filter_map(|t| jstr(t, "value")).map(str::to_owned);
    for text in jtexts(item, "ISBN", &[]).into_iter().chain(typed) {
        for isbn in isbns_in(&text) {
            push_isbn(&mut isbns, isbn);
        }
    }
    let mut book = BookMeta {
        title: jtexts(item, "title", &[]).into_iter().next()?,
        subtitle: jtexts(item, "subtitle", &[]).into_iter().next(),
        publisher: jstr(item, "publisher").map(str::to_owned),
        published: ["published-print", "published", "issued"]
            .iter()
            .find_map(|k| date_parts(item.get(*k))),
        language: jstr(item, "language").map(str::to_owned),
        ..BookMeta::default()
    };
    people(item, "author", &ContributorRole::Author, &mut book);
    people(item, "editor", &ContributorRole::Editor, &mut book);
    people(item, "translator", &ContributorRole::Translator, &mut book);
    let url = jstr(item, "DOI")
        .filter(|doi| doi.starts_with("10.") && !doi.contains(char::is_whitespace))
        .map(|doi| format!("https://doi.org/{doi}"));
    Some(Extracted { book, isbns, url })
}

impl Provider for Crossref {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let filter = format!("isbn:{isbn}");
        let mut params = vec![("filter", filter.as_str()), ("rows", "5")];
        if let Some(mailto) = &self.mailto {
            params.push(("mailto", mailto.as_str()));
        }
        vec![get_json(url_with_query(&join(&self.base, "/works"), &params))]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !request.url.starts_with(&self.base) || !path(&request.url).ends_with("/works") {
            return Err(unknown_request(&INFO, request));
        }
        if check_http(INFO.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        let message = json.get("message");
        match jstr(&json, "status") {
            Some("ok") => {}
            Some(_) => {
                let text = message
                    .and_then(|m| m.as_array())
                    .and_then(|list| list.first())
                    .and_then(|first| jstr(first, "message"))
                    .unwrap_or("status is not ok");
                return Err(CatalogError::Remote { provider: INFO.id, message: shorten(text) });
            }
            None => {
                return Err(CatalogError::Malformed {
                    provider: INFO.id,
                    format: "JSON",
                    detail: "no `status` in the answer".to_owned(),
                });
            }
        }
        let mut notes = Vec::new();
        let items = message.map_or(&[][..], |m| jarr(m, "items"));
        let candidates: Vec<Extracted> = items.iter().filter_map(work).collect();
        if candidates.len() < items.len() {
            notes.push("ignored items that are not whole books".to_owned());
        }
        let records = crate::marc::select(isbn, candidates, &mut notes)
            .map(|chosen| finalize(&INFO, isbn, chosen))
            .into_iter()
            .collect();
        Ok(Parsed { records, follow_up: Vec::new(), notes })
    }
}
