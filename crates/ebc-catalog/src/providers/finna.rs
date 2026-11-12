//! Finna, the search service of Finnish libraries, archives and museums (REST JSON).

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, Contributor, ContributorRole, HttpRequest, HttpResponse};

use crate::error::{CatalogError, shorten};
use crate::marc::{Extracted, role_from_term};
use crate::normalize;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{format_from_text, get_json, unknown_request};
use crate::util::{
    Found, check_http, encode_component, isbns_in, jarr, join, jstr, jtext, jtexts, parse_json,
    path, push_isbn, url_with_query,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "finna",
    name: "Finna (National Library of Finland)",
    homepage: "https://www.finna.fi/",
    data_licence: "CC0 1.0 (metadata; images excluded)",
    terms_url: "https://www.finna.fi/Content/terms",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["api.finna.fi"],
    notes: "CC0 for metadata, the terms URL and CORS support are from memory. Endpoint read \
            in the finna-client package. Images have per-record rights and are not returned.",
    class: SourceClass::Open,
    default_enabled: true,
};

/// Record fields asked for (the API returns a short default set otherwise).
const FIELDS: [&str; 12] = [
    "id",
    "title",
    "subTitle",
    "nonPresenterAuthors",
    "publishers",
    "year",
    "languages",
    "isbns",
    "cleanIsbn",
    "series",
    "physicalDescriptions",
    "formats",
];

/// Finna: `GET {base}/api/v1/search?lookfor={isbn}&type=AllFields&limit=5&field[]=…`.
#[derive(Debug, Clone)]
pub struct Finna {
    base: String,
}

impl Default for Finna {
    fn default() -> Self {
        Self::new()
    }
}

impl Finna {
    /// The provider pointed at `https://api.finna.fi`.
    #[must_use]
    pub fn new() -> Self {
        Self { base: "https://api.finna.fi".to_owned() }
    }

    /// Replaces the API root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }
}

fn record(value: &serde_json::Value) -> Option<Extracted> {
    let mut isbns = Vec::new();
    for text in jtexts(value, "isbns", &[]).into_iter().chain(jtexts(value, "cleanIsbn", &[])) {
        for isbn in isbns_in(&text) {
            push_isbn(&mut isbns, isbn);
        }
    }
    let mut book = BookMeta {
        title: jstr(value, "title")?.to_owned(),
        subtitle: jstr(value, "subTitle").map(str::to_owned),
        publisher: jtexts(value, "publishers", &[]).into_iter().next(),
        published: jtexts(value, "year", &[]).into_iter().next(),
        language: jtexts(value, "languages", &[]).into_iter().next(),
        series: jarr(value, "series").iter().find_map(|s| jtext(s, &["name"])),
        page_count: jtexts(value, "physicalDescriptions", &[])
            .iter()
            .find_map(|d| normalize::page_count(d)),
        format: jarr(value, "formats")
            .iter()
            .filter_map(|f| jtext(f, &["value"]))
            .find_map(|f| format_from_text(&f)),
        ..BookMeta::default()
    };
    for author in jarr(value, "nonPresenterAuthors") {
        let Some(name) = jtext(author, &["name"]) else { continue };
        let role = match jstr(author, "role") {
            // No role given: Finna lists the main author first without one.
            None => Some(ContributorRole::Author),
            Some(term) => role_from_term(term),
        };
        if let Some(role) = role {
            book.contributors.push(Contributor { name, role });
        }
    }
    let url =
        jstr(value, "id").map(|id| format!("https://www.finna.fi/Record/{}", encode_component(id)));
    Some(Extracted { book, isbns, url })
}

impl Provider for Finna {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let plain = isbn.to_plain();
        let mut params = vec![("lookfor", plain.as_str()), ("type", "AllFields"), ("limit", "5")];
        params.extend(FIELDS.iter().map(|field| ("field%5B%5D", *field)));
        vec![get_json(url_with_query(&join(&self.base, "/api/v1/search"), &params))]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !request.url.starts_with(&self.base) || !path(&request.url).ends_with("/api/v1/search") {
            return Err(unknown_request(&INFO, request));
        }
        if check_http(INFO.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        match jstr(&json, "status") {
            Some("OK") => {}
            Some(_) => {
                return Err(CatalogError::Remote {
                    provider: INFO.id,
                    message: shorten(jstr(&json, "statusMessage").unwrap_or("status is not OK")),
                });
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
        let candidates: Vec<Extracted> = jarr(&json, "records")
            .iter()
            .filter_map(record)
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
