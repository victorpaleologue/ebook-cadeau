//! lobid-resources, the hbz union catalogue of North Rhine-Westphalia (JSON-LD REST).

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, Contributor, ContributorRole, HttpRequest, HttpResponse};
use serde_json::Value;

use crate::error::CatalogError;
use crate::marc::{Extracted, role_from_term};
use crate::normalize;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{format_from_text, get_json, unknown_request};
use crate::util::{
    Found, check_http, isbns_in, jarr, jlist, join, jstr, jtext, jtexts, parse_json, path,
    push_isbn, url_with_query,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "lobid",
    name: "lobid-resources (hbz)",
    homepage: "https://lobid.org/resources",
    data_licence: "CC0 1.0",
    terms_url: "https://lobid.org/usage-policy/",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["lobid.org"],
    notes: "CC0 is from memory; a usage policy exists on lobid.org and could not be read. \
            The `isbn:` query field name is from memory. JSONP is documented; CORS headers \
            are unverified.",
    class: SourceClass::Open,
    default_enabled: true,
};

/// lobid: `GET {base}/resources/search?q=isbn:{isbn}&format=json&size=5`.
#[derive(Debug, Clone)]
pub struct Lobid {
    base: String,
}

impl Default for Lobid {
    fn default() -> Self {
        Self::new()
    }
}

impl Lobid {
    /// The provider pointed at `https://lobid.org`.
    #[must_use]
    pub fn new() -> Self {
        Self { base: "https://lobid.org".to_owned() }
    }

    /// Replaces the site root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }
}

fn member(value: &Value) -> Option<Extracted> {
    let mut isbns = Vec::new();
    for text in jtexts(value, "isbn", &[]) {
        for isbn in isbns_in(&text) {
            push_isbn(&mut isbns, isbn);
        }
    }
    let mut book = BookMeta { title: jstr(value, "title")?.to_owned(), ..BookMeta::default() };
    let other = jtexts(value, "otherTitleInformation", &[]);
    if !other.is_empty() {
        book.subtitle = Some(other.join(" : "));
    }
    for contribution in jarr(value, "contribution") {
        let Some(name) = contribution.get("agent").and_then(|a| jtext(a, &["label"])) else {
            continue;
        };
        let role = match contribution.get("role") {
            // Roles are MARC relators: "http://id.loc.gov/vocabulary/relators/aut".
            Some(role) => {
                let code = jstr(role, "id").and_then(|id| id.rsplit('/').next()).unwrap_or("");
                let from_code = match code {
                    "aut" | "cre" => Some(ContributorRole::Author),
                    "trl" => Some(ContributorRole::Translator),
                    "ill" | "art" => Some(ContributorRole::Illustrator),
                    "edt" | "com" => Some(ContributorRole::Editor),
                    "aui" | "aft" | "wpr" => Some(ContributorRole::Preface),
                    "nrt" => Some(ContributorRole::Narrator),
                    "pbl" | "prt" => continue,
                    _ => None,
                };
                from_code
                    .or_else(|| jstr(role, "label").and_then(role_from_term))
                    .unwrap_or_else(|| ContributorRole::Other("contributor".to_owned()))
            }
            None => ContributorRole::Author,
        };
        book.contributors.push(Contributor { name, role });
    }
    if let Some(publication) = jarr(value, "publication").first() {
        book.publisher = jtexts(publication, "publishedBy", &[]).into_iter().next();
        book.published = jtexts(publication, "startDate", &[])
            .into_iter()
            .find(|d| normalize::date(d).is_some());
    }
    book.language = jarr(value, "language").iter().find_map(|language| {
        jstr(language, "id").and_then(|id| id.rsplit('/').next()).map(str::to_owned)
    });
    book.page_count =
        jtexts(value, "extent", &[]).iter().find_map(|extent| normalize::page_count(extent));
    book.series = jarr(value, "isPartOf").iter().find_map(|part| {
        jlist(part.get("hasSuperordinate")).into_iter().find_map(|s| jtext(s, &["label"]))
    });
    book.format = jarr(value, "medium")
        .iter()
        .filter_map(|m| jtext(m, &["label"]))
        .find_map(|label| format_from_text(&label));
    let url = jstr(value, "id")
        .map(|id| id.split('#').next().unwrap_or(id))
        .filter(|id| id.starts_with("https://") || id.starts_with("http://"))
        .map(str::to_owned);
    Some(Extracted { book, isbns, url })
}

impl Provider for Lobid {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let query = format!("isbn:{isbn}");
        vec![get_json(url_with_query(
            &join(&self.base, "/resources/search"),
            &[("q", &query), ("format", "json"), ("size", "5")],
        ))]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !request.url.starts_with(&self.base)
            || !path(&request.url).ends_with("/resources/search")
        {
            return Err(unknown_request(&INFO, request));
        }
        if check_http(INFO.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        if !json.is_object() || (json.get("member").is_none() && json.get("totalItems").is_none()) {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "JSON",
                detail: "neither `member` nor `totalItems` in the answer".to_owned(),
            });
        }
        let mut notes = Vec::new();
        let candidates: Vec<Extracted> = jarr(&json, "member").iter().filter_map(member).collect();
        let records = crate::marc::select(isbn, candidates, &mut notes)
            .map(|chosen| finalize(&INFO, isbn, chosen))
            .into_iter()
            .collect();
        Ok(Parsed { records, follow_up: Vec::new(), notes })
    }
}
