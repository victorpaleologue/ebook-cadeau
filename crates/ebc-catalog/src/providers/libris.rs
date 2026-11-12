//! Libris, the Swedish union catalogue (KB): the JSON-LD `find` API.

use ebc_isbn::Isbn13;
use ebc_model::{BookFormat, BookMeta, Contributor, ContributorRole, HttpRequest, HttpResponse};
use serde_json::Value;

use crate::error::CatalogError;
use crate::marc::{Extracted, role_from_term};
use crate::normalize;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::unknown_request;
use crate::util::{
    Found, check_http, isbns_in, jlist, jstr, jtext, join, parse_json, path, push_isbn,
    url_with_query,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "libris",
    name: "Libris (Kungliga biblioteket)",
    homepage: "https://libris.kb.se/",
    data_licence: "CC0 1.0",
    terms_url: "https://libris.kb.se/api-docs/",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["libris.kb.se"],
    notes: "CC0 and \"freely available without restrictions\" read in KB's API documentation \
            repository; the terms URL is from memory. A User-Agent is required: anonymous \
            clients may be rate-limited or blocked without warning. The documentation shows \
            a browser fetch(), but the CORS header itself is unverified.",
    class: SourceClass::Open,
    default_enabled: true,
};

/// Libris: `GET {base}/find.jsonld?q={isbn}&_limit=5`.
#[derive(Debug, Clone)]
pub struct Libris {
    base: String,
}

impl Default for Libris {
    fn default() -> Self {
        Self::new()
    }
}

impl Libris {
    /// The provider pointed at `https://libris.kb.se`.
    #[must_use]
    pub fn new() -> Self {
        Self { base: "https://libris.kb.se".to_owned() }
    }

    /// Replaces the site root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }
}

/// Last segment of an `@id` such as `https://id.kb.se/language/fre`.
fn id_tail(value: &Value) -> Option<&str> {
    let id = match value {
        Value::String(s) => s.as_str(),
        other => jstr(other, "@id")?,
    };
    id.trim_end_matches('/').rsplit(['/', '#']).next().filter(|t| !t.is_empty())
}

fn agent_name(agent: &Value) -> Option<String> {
    match (jstr(agent, "givenName"), jstr(agent, "familyName")) {
        (Some(given), Some(family)) => Some(format!("{given} {family}")),
        (None, Some(family)) => Some(family.to_owned()),
        _ => ["name", "label", "prefLabel"].iter().find_map(|k| {
            jlist(agent.get(*k)).into_iter().find_map(|v| jtext(v, &[]))
        }),
    }
}

fn instance(item: &Value) -> Option<Extracted> {
    let mut book = BookMeta::default();
    let mut isbns = Vec::new();
    for identifier in jlist(item.get("identifiedBy")) {
        if jstr(identifier, "@type") == Some("ISBN")
            && let Some(value) = jstr(identifier, "value")
        {
            for isbn in isbns_in(value) {
                push_isbn(&mut isbns, isbn);
            }
        }
    }
    let titles = jlist(item.get("hasTitle"));
    let title = titles
        .iter()
        .find(|t| jstr(t, "@type") == Some("Title"))
        .or_else(|| titles.first())?;
    book.title = jlist(title.get("mainTitle")).into_iter().find_map(|v| jtext(v, &[]))?;
    book.subtitle = jlist(title.get("subtitle")).into_iter().find_map(|v| jtext(v, &[]));

    let work = jlist(item.get("instanceOf")).into_iter().next();
    if let Some(work) = work {
        book.language = jlist(work.get("language")).into_iter().find_map(|language| {
            jstr(language, "code").or_else(|| id_tail(language)).map(str::to_owned)
        });
        for contribution in jlist(work.get("contribution")) {
            let Some(name) =
                jlist(contribution.get("agent")).into_iter().find_map(agent_name)
            else {
                continue;
            };
            let coded = jlist(contribution.get("role"))
                .into_iter()
                .filter_map(id_tail)
                .find_map(role_from_term);
            let role = coded.unwrap_or_else(|| {
                if jstr(contribution, "@type") == Some("PrimaryContribution") {
                    ContributorRole::Author
                } else {
                    ContributorRole::Other("contributor".to_owned())
                }
            });
            book.contributors.push(Contributor { name, role });
        }
    }
    let publications = jlist(item.get("publication"));
    if let Some(publication) = publications
        .iter()
        .find(|p| jstr(p, "@type") == Some("PrimaryPublication"))
        .or_else(|| publications.first())
    {
        book.publisher =
            jlist(publication.get("agent")).into_iter().find_map(agent_name);
        book.published = ["year", "date", "startYear"]
            .iter()
            .find_map(|k| publication.get(*k).and_then(|v| jtext(v, &[])))
            .filter(|d| normalize::date(d).is_some());
    }
    book.page_count = jlist(item.get("extent"))
        .into_iter()
        .flat_map(|extent| jlist(extent.get("label")))
        .filter_map(|label| jtext(label, &[]))
        .find_map(|label| normalize::page_count(&label));
    book.series = jlist(item.get("seriesMembership"))
        .into_iter()
        .find_map(|series| jtext(series, &["seriesStatement"]));
    if jstr(item, "@type") == Some("Electronic") {
        book.format = Some(BookFormat::Ebook);
    }
    let url = jstr(item, "@id")
        .map(|id| id.split('#').next().unwrap_or(id))
        .filter(|id| id.starts_with("https://") || id.starts_with("http://"))
        .map(str::to_owned);
    Some(Extracted { book, isbns, url })
}

impl Provider for Libris {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let plain = isbn.to_plain();
        let url =
            url_with_query(&join(&self.base, "/find.jsonld"), &[("q", &plain), ("_limit", "5")]);
        vec![HttpRequest::get(url).with_header("Accept", "application/ld+json, application/json")]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !request.url.starts_with(&self.base) || !path(&request.url).ends_with("/find.jsonld") {
            return Err(unknown_request(&INFO, request));
        }
        if check_http(INFO.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        if !json.is_object() || (json.get("items").is_none() && json.get("totalItems").is_none()) {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "JSON",
                detail: "neither `items` nor `totalItems` in the answer".to_owned(),
            });
        }
        let mut notes = Vec::new();
        let candidates: Vec<Extracted> = jlist(json.get("items"))
            .into_iter()
            .filter_map(instance)
            // A free-text search can match the digits anywhere: insist on the identifier.
            .filter(|candidate| !candidate.isbns.is_empty())
            .collect();
        let records = crate::marc::select(isbn, candidates, &mut notes)
            .map(|chosen| finalize(&INFO, isbn, chosen))
            .into_iter()
            .collect();
        Ok(Parsed { records, follow_up: Vec::new(), notes })
    }
}
