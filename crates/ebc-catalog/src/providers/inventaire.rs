//! Inventaire.io entities API. Three steps, all on the same endpoint:
//!
//! 1. `by-uris&uris=isbn:{isbn}` → the edition (title, date, pages, language) and the URIs
//!    of its work(s), publisher, collection and translators;
//! 2. `by-uris&uris={those}` → publisher and collection labels, translator names, and the
//!    author URIs of the work(s);
//! 3. `by-uris&uris={authors}` → author names.
//!
//! The step is carried in the URL fragment of the follow-up requests.

use ebc_isbn::Isbn13;
use ebc_model::{
    BookMeta, Contributor, ContributorRole, CoverKind, CoverRef, HttpRequest, HttpResponse,
};
use serde_json::Value;

use crate::error::CatalogError;
use crate::marc::Extracted;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{fragment, get_json, is_qid, unknown_request};
use crate::util::{
    Found, check_http, context, isbns_in, jlist, join, jstr, jtext, parse_json, path, push_isbn,
    query_param, url_with_query, with_context,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "inventaire",
    name: "Inventaire.io",
    homepage: "https://inventaire.io/",
    data_licence: "CC0 1.0",
    terms_url: "https://wiki.inventaire.io/wiki/Terms_of_Use",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 2000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["inventaire.io"],
    notes: "Off by default (last resort: it re-queries BnF, BNE, Open Library and Wikidata). \
            CC0 for the data, the terms URL, the limits and CORS are from memory. The server \
            code is AGPL-3.0 and was read for facts only. Images are user uploads: cover \
            references are candidates for preview only.",
    class: SourceClass::Open,
    default_enabled: false,
};

/// Caps on the URIs asked for in steps 2 and 3.
const MAX_REFS: usize = 8;
const MAX_AUTHORS: usize = 5;

/// Wikidata items of common languages, for `wdt:P407`.
const LANGUAGES: [(&str, &str); 12] = [
    ("wd:Q150", "fr"),
    ("wd:Q1860", "en"),
    ("wd:Q188", "de"),
    ("wd:Q1321", "es"),
    ("wd:Q652", "it"),
    ("wd:Q7411", "nl"),
    ("wd:Q5146", "pt"),
    ("wd:Q9027", "sv"),
    ("wd:Q1412", "fi"),
    ("wd:Q7737", "ru"),
    ("wd:Q809", "pl"),
    ("wd:Q397", "la"),
];

/// Inventaire: `GET {base}/api/entities?action=by-uris&uris=isbn:{isbn}`.
#[derive(Debug, Clone)]
pub struct Inventaire {
    base: String,
}

impl Default for Inventaire {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventaire {
    /// The provider pointed at `https://inventaire.io`.
    #[must_use]
    pub fn new() -> Self {
        Self { base: "https://inventaire.io".to_owned() }
    }

    /// Replaces the site root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }

    fn request(&self, uris: &[String], step: Option<&str>) -> HttpRequest {
        let joined = uris.join("|");
        let url = url_with_query(
            &join(&self.base, "/api/entities"),
            &[("action", "by-uris"), ("uris", &joined)],
        );
        get_json(match step {
            Some(step) => with_context(&url, step),
            None => url,
        })
    }
}

/// `wd:Q42` or `inv:` followed by 32 hexadecimal digits: the only URIs put back in a URL.
fn valid_uri(uri: &str) -> bool {
    match uri.split_once(':') {
        Some(("wd", id)) => is_qid(id) && id.len() <= 12,
        Some(("inv", id)) => id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()),
        _ => false,
    }
}

fn claims(entity: &Value, property: &str) -> Vec<String> {
    jlist(entity.get("claims").and_then(|c| c.get(property)))
        .into_iter()
        .filter_map(|claim| jtext(claim, &["value"]))
        .collect()
}

fn uris(entity: &Value, property: &str) -> Vec<String> {
    claims(entity, property).into_iter().filter(|uri| valid_uri(uri)).collect()
}

fn label(entity: &Value) -> Option<String> {
    let labels = entity.get("labels")?.as_object()?;
    ["fr", "en", "de", "es", "it", "fromclaims"]
        .iter()
        .find_map(|lang| labels.get(*lang))
        .or_else(|| labels.values().next())
        .and_then(|l| jtext(l, &["value"]))
        .filter(|l| !is_qid(l))
}

impl Provider for Inventaire {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        vec![self.request(&[format!("isbn:{isbn}")], None)]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !request.url.starts_with(&self.base) || !path(&request.url).ends_with("/api/entities") {
            return Err(unknown_request(&INFO, request));
        }
        let step = context(&request.url);
        if check_http(INFO.id, response)? == Found::No {
            // The API answers 200 with `notFound` for unknown URIs.
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        let Some(entities) = json.get("entities").and_then(Value::as_object) else {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "JSON",
                detail: "no `entities` object in the answer".to_owned(),
            });
        };
        let mut parsed = Parsed::empty();
        let asked: Vec<String> = query_param(&request.url, "uris")
            .map(|list| list.split('|').map(str::to_owned).collect())
            .unwrap_or_default();
        // Entities in the order they were asked for; redirected URIs come back under
        // another key, so fall back to document order for those.
        let mut ordered: Vec<&Value> = asked.iter().filter_map(|uri| entities.get(uri)).collect();
        for entity in entities.values() {
            if !ordered.iter().any(|known| std::ptr::eq(*known, entity)) {
                ordered.push(entity);
            }
        }

        match step.as_deref() {
            None => {
                let Some(edition) =
                    ordered.iter().find(|e| jstr(e, "type").is_none_or(|t| t == "edition"))
                else {
                    return Ok(parsed);
                };
                let mut isbns = Vec::new();
                for property in ["wdt:P212", "wdt:P957"] {
                    for value in claims(edition, property) {
                        for found in isbns_in(&value) {
                            push_isbn(&mut isbns, found);
                        }
                    }
                }
                if !isbns.is_empty() && !isbns.contains(isbn) {
                    parsed.notes.push("ignored an edition that carries other ISBNs".to_owned());
                    return Ok(parsed);
                }
                let Some(title) =
                    claims(edition, "wdt:P1476").into_iter().next().or_else(|| label(edition))
                else {
                    parsed.notes.push("ignored an edition without a title".to_owned());
                    return Ok(parsed);
                };
                let mut book = BookMeta {
                    title,
                    subtitle: claims(edition, "wdt:P1680").into_iter().next(),
                    published: claims(edition, "wdt:P577").into_iter().next(),
                    page_count: claims(edition, "wdt:P1104")
                        .first()
                        .and_then(|p| p.split('.').next()?.parse::<u32>().ok()),
                    language: claims(edition, "wdt:P407").iter().find_map(|uri| {
                        LANGUAGES.iter().find(|(q, _)| q == uri).map(|(_, code)| (*code).to_owned())
                    }),
                    ..BookMeta::default()
                };
                if let Some(image) = edition.get("image").and_then(|i| jtext(i, &["url"]))
                    && image.starts_with("/img/")
                    && !image.contains(['?', '#', ' ', '\\'])
                {
                    book.covers.push(CoverRef {
                        url: format!("https://inventaire.io{image}"),
                        kind: CoverKind::Front,
                        width_px: None,
                        height_px: None,
                        source: INFO.id.to_owned(),
                    });
                }
                let mut refs: Vec<String> = Vec::new();
                for property in ["wdt:P629", "wdt:P123", "wdt:P195", "wdt:P655"] {
                    for uri in uris(edition, property) {
                        if !refs.contains(&uri) && refs.len() < MAX_REFS {
                            refs.push(uri);
                        }
                    }
                }
                if !refs.is_empty() {
                    parsed.follow_up.push(self.request(&refs, Some("refs")));
                }
                let url = Some(format!("https://inventaire.io/entity/isbn:{isbn}"));
                parsed.records.push(finalize(&INFO, isbn, Extracted { book, isbns, url }));
            }
            Some("refs") => {
                let mut book = BookMeta::default();
                let mut authors: Vec<String> = Vec::new();
                for entity in ordered {
                    match jstr(entity, "type") {
                        Some("work" | "serie") => {
                            for uri in uris(entity, "wdt:P50") {
                                if !authors.contains(&uri) && authors.len() < MAX_AUTHORS {
                                    authors.push(uri);
                                }
                            }
                        }
                        Some("publisher") => {
                            book.publisher = book.publisher.or_else(|| label(entity))
                        }
                        Some("collection") => book.series = book.series.or_else(|| label(entity)),
                        Some("human") => {
                            if let Some(name) = label(entity) {
                                book.contributors
                                    .push(Contributor { name, role: ContributorRole::Translator });
                            }
                        }
                        _ => {}
                    }
                }
                if !authors.is_empty() {
                    parsed.follow_up.push(self.request(&authors, Some("authors")));
                }
                if book != BookMeta::default() {
                    parsed.records.push(fragment(isbn, book));
                }
            }
            Some("authors") => {
                let contributors: Vec<Contributor> =
                    ordered.into_iter().filter_map(label).map(Contributor::author).collect();
                if !contributors.is_empty() {
                    parsed
                        .records
                        .push(fragment(isbn, BookMeta { contributors, ..BookMeta::default() }));
                }
            }
            Some(_) => return Err(unknown_request(&INFO, request)),
        }
        Ok(parsed)
    }
}
