//! Wikidata Query Service (SPARQL): one query finds the edition by P212 / P957 and
//! returns its title, authors (the edition's, else the work's), translators, publisher,
//! date, language and page count.

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, Contributor, ContributorRole, HttpRequest, HttpResponse};
use serde_json::Value;

use crate::error::CatalogError;
use crate::marc::Extracted;
use crate::normalize;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{is_qid, unknown_request};
use crate::util::{
    Found, check_http, isbns_in, jarr, jstr, parse_json, push_isbn, url_with_query,
    without_fragment,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "wikidata",
    name: "Wikidata Query Service",
    homepage: "https://www.wikidata.org/",
    data_licence: "CC0 1.0",
    terms_url: "https://www.mediawiki.org/wiki/Wikidata_Query_Service/User_Manual",
    requires_key: false,
    browser_cors: true,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 2000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["query.wikidata.org"],
    notes: "Off by default (last resort; low hit rate for editions). CC0, CORS and the \
            query-service limits (60 s timeout, 5 parallel queries per IP, User-Agent \
            policy) are from memory.",
    class: SourceClass::Open,
    default_enabled: false,
};

/// Wikidata: `GET {base}?format=json&query={SPARQL}` with base
/// `https://query.wikidata.org/sparql`.
#[derive(Debug, Clone)]
pub struct Wikidata {
    endpoint: String,
}

impl Default for Wikidata {
    fn default() -> Self {
        Self::new()
    }
}

impl Wikidata {
    /// The provider pointed at `https://query.wikidata.org/sparql`.
    #[must_use]
    pub fn new() -> Self {
        Self { endpoint: "https://query.wikidata.org/sparql".to_owned() }
    }

    /// Replaces the SPARQL endpoint URL (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into().trim_end_matches(['?', '&']).to_owned();
        self
    }

    /// The SPARQL query for `isbn`. ISBNs are digits, hyphens and `X`, so they are safe
    /// inside a string literal.
    #[must_use]
    pub fn sparql(isbn: &Isbn13) -> String {
        let mut forms = vec![("P212", isbn.hyphenated()), ("P212", isbn.to_plain())];
        if let Ok(ten) = isbn.to_isbn10() {
            forms.push(("P957", ten.hyphenated()));
            forms.push(("P957", ten.to_plain()));
        }
        forms.dedup();
        let matches: Vec<String> =
            forms.iter().map(|(p, value)| format!("{{ ?item wdt:{p} \"{value}\" }}")).collect();
        format!(
            "SELECT ?item ?itemLabel ?title ?subtitle ?authorLabel ?workAuthorLabel \
             ?translatorLabel ?publisherLabel ?date ?langCode ?pages ?isbn13 ?isbn10 WHERE {{ \
             {} \
             OPTIONAL {{ ?item wdt:P1476 ?title }} \
             OPTIONAL {{ ?item wdt:P1680 ?subtitle }} \
             OPTIONAL {{ ?item wdt:P50 ?author }} \
             OPTIONAL {{ ?item wdt:P629 ?work . ?work wdt:P50 ?workAuthor }} \
             OPTIONAL {{ ?item wdt:P655 ?translator }} \
             OPTIONAL {{ ?item wdt:P123 ?publisher }} \
             OPTIONAL {{ ?item wdt:P577 ?date }} \
             OPTIONAL {{ ?item wdt:P407 ?lang . ?lang wdt:P218 ?langCode }} \
             OPTIONAL {{ ?item wdt:P1104 ?pages }} \
             OPTIONAL {{ ?item wdt:P212 ?isbn13 }} \
             OPTIONAL {{ ?item wdt:P957 ?isbn10 }} \
             SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"fr,en,de,es,it,mul\" . }} \
             }} LIMIT 60",
            matches.join(" UNION ")
        )
    }
}

fn binding<'a>(row: &'a Value, name: &str) -> Option<&'a str> {
    row.get(name).and_then(|b| jstr(b, "value"))
}

fn push_unique(list: &mut Vec<String>, value: &str) {
    if !is_qid(value) && !list.iter().any(|v| v == value) {
        list.push(value.to_owned());
    }
}

impl Provider for Wikidata {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let query = Self::sparql(isbn);
        let url = url_with_query(&self.endpoint, &[("format", "json"), ("query", &query)]);
        vec![HttpRequest::get(url).with_header("Accept", "application/sparql-results+json")]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !without_fragment(&request.url).starts_with(&self.endpoint) {
            return Err(unknown_request(&INFO, request));
        }
        if check_http(INFO.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        let Some(results) = json.get("results").filter(|r| r.get("bindings").is_some_and(Value::is_array))
        else {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "JSON",
                detail: "no `results.bindings` array (not a SPARQL result)".to_owned(),
            });
        };
        let rows = jarr(results, "bindings");
        let mut parsed = Parsed::empty();
        let Some(item) = rows.iter().find_map(|row| binding(row, "item")) else {
            return Ok(parsed);
        };
        if rows.iter().filter_map(|row| binding(row, "item")).any(|other| other != item) {
            parsed.notes.push("several Wikidata items carry this ISBN; kept the first".to_owned());
        }

        let mut book = BookMeta::default();
        let mut isbns = Vec::new();
        let (mut authors, mut work_authors, mut translators) = (Vec::new(), Vec::new(), Vec::new());
        let mut label = None;
        for row in rows.iter().filter(|row| binding(row, "item") == Some(item)) {
            for name in ["isbn13", "isbn10"] {
                for found in binding(row, name).into_iter().flat_map(isbns_in) {
                    push_isbn(&mut isbns, found);
                }
            }
            if book.title.is_empty() {
                book.title = binding(row, "title").unwrap_or("").to_owned();
            }
            label = label.or_else(|| binding(row, "itemLabel").filter(|l| !is_qid(l)));
            book.subtitle = book.subtitle.or_else(|| binding(row, "subtitle").map(str::to_owned));
            for (name, list) in [
                ("authorLabel", &mut authors),
                ("workAuthorLabel", &mut work_authors),
                ("translatorLabel", &mut translators),
            ] {
                if let Some(value) = binding(row, name) {
                    push_unique(list, value);
                }
            }
            book.publisher = book
                .publisher
                .or_else(|| binding(row, "publisherLabel").filter(|p| !is_qid(p)).map(str::to_owned));
            book.published = book.published.or_else(|| {
                binding(row, "date").and_then(|date| {
                    // Year-precision dates are served as 1 January, midnight.
                    match date.strip_suffix("-01-01T00:00:00Z") {
                        Some(year) => normalize::date(year),
                        None => normalize::date(date),
                    }
                })
            });
            book.language = book.language.or_else(|| binding(row, "langCode").map(str::to_owned));
            book.page_count = book.page_count.or_else(|| {
                binding(row, "pages").and_then(|p| p.split('.').next()?.parse::<u32>().ok())
            });
        }
        if book.title.is_empty() {
            book.title = label.unwrap_or("").to_owned();
        }
        if !isbns.is_empty() && !isbns.contains(isbn) {
            parsed.notes.push("ignored an item that carries other ISBNs".to_owned());
            return Ok(parsed);
        }
        if book.title.is_empty() {
            parsed.notes.push("ignored an item without a title".to_owned());
            return Ok(parsed);
        }
        let authors = if authors.is_empty() { work_authors } else { authors };
        book.contributors = authors.into_iter().map(Contributor::author).collect();
        book.contributors.extend(
            translators
                .into_iter()
                .map(|name| Contributor { name, role: ContributorRole::Translator }),
        );
        let url = item
            .rsplit('/')
            .next()
            .filter(|id| is_qid(id))
            .map(|id| format!("https://www.wikidata.org/wiki/{id}"));
        parsed.records.push(finalize(&INFO, isbn, Extracted { book, isbns, url }));
        Ok(parsed)
    }
}
