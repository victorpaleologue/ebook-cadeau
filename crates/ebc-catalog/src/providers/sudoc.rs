//! SUDOC, the union catalogue of French academic libraries (ABES). Two steps:
//! `isbn2ppn` gives record numbers (PPN), then `{ppn}.xml` gives the UNIMARC record.

use ebc_isbn::Isbn13;
use ebc_model::{HttpRequest, HttpResponse};
use serde_json::Value;

use crate::error::CatalogError;
use crate::marc::{self, Extracted};
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{get_xml, unknown_request};
use crate::util::{Found, Sniff, check_http, join, parse_json, path, sniff};
use crate::xml::{marc_records, parse_xml};

static INFO: ProviderInfo = ProviderInfo {
    id: "sudoc",
    name: "SUDOC (ABES)",
    homepage: "https://www.sudoc.abes.fr/",
    data_licence: "Licence Ouverte / Open Licence (Etalab)",
    terms_url: "https://abes.fr/reseau-sudoc/reutiliser-les-donnees-sudoc/",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: Some("Données bibliographiques : Sudoc (ABES)"),
    min_interval_ms: 1000,
    licence_id: Some("etalab-2.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["www.sudoc.fr"],
    notes: "Off by default (academic holdings; enable after a hit-rate test). The licence, \
            the terms URL, the https scheme and the `{ppn}.xml` record URL are from memory; \
            `isbn2ppn` was read in a 2022 client that used http. Attribution must name the \
            source and a date: show the retrieval date next to it. Limits unknown.",
    class: SourceClass::Open,
    default_enabled: false,
};

/// At most this many records are fetched per lookup.
const MAX_RECORDS: usize = 2;

/// SUDOC: `GET {base}/services/isbn2ppn/{isbn}` (JSON), then `GET {base}/{ppn}.xml`.
#[derive(Debug, Clone)]
pub struct Sudoc {
    base: String,
}

impl Default for Sudoc {
    fn default() -> Self {
        Self::new()
    }
}

impl Sudoc {
    /// The provider pointed at `https://www.sudoc.fr`.
    #[must_use]
    pub fn new() -> Self {
        Self { base: "https://www.sudoc.fr".to_owned() }
    }

    /// Replaces the site root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }
}

/// A PPN is eight digits and a check character (digit or `X`).
fn valid_ppn(ppn: &str) -> bool {
    ppn.len() == 9
        && ppn.bytes().take(8).all(|b| b.is_ascii_digit())
        && ppn.bytes().last().is_some_and(|b| b.is_ascii_digit() || b == b'X')
}

fn push_ppn(list: &mut Vec<String>, candidate: &str) {
    let candidate = candidate.trim();
    if valid_ppn(candidate) && !list.iter().any(|p| p == candidate) {
        list.push(candidate.to_owned());
    }
}

/// Collects every `"ppn"` member of a JSON answer, whatever the nesting (`result` is an
/// object for one hit and an array for several). Returns the service's error text, if any.
fn collect_ppns(value: &Value, list: &mut Vec<String>, error: &mut Option<String>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                match (key.as_str(), inner) {
                    ("ppn", Value::String(ppn)) => push_ppn(list, ppn),
                    ("ppn", Value::Number(ppn)) => push_ppn(list, &format!("{ppn:0>9}")),
                    ("error", Value::String(message)) => *error = Some(message.clone()),
                    _ => collect_ppns(inner, list, error),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_ppns(item, list, error)),
        _ => {}
    }
}

impl Provider for Sudoc {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let url = join(&self.base, &format!("/services/isbn2ppn/{isbn}"));
        vec![HttpRequest::get(url).with_header("Accept", "text/json")]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        let route = path(&request.url);
        let resolver = route.contains("/services/isbn2ppn/");
        let ppn = route
            .rsplit('/')
            .next()
            .and_then(|file| file.strip_suffix(".xml"))
            .filter(|ppn| valid_ppn(ppn));
        if !request.url.starts_with(&self.base) || (!resolver && ppn.is_none()) {
            return Err(unknown_request(&INFO, request));
        }
        // Both steps answer 404 for "nothing here".
        if check_http(INFO.id, response)? == Found::No {
            return Ok(Parsed::empty());
        }
        let mut parsed = Parsed::empty();
        if resolver {
            let mut ppns = Vec::new();
            let mut error = None;
            match sniff(&response.body) {
                Sniff::Xml => {
                    let root = parse_xml(INFO.id, response)?;
                    let mut nodes = Vec::new();
                    root.find_all("ppn", &mut nodes);
                    nodes.iter().for_each(|node| push_ppn(&mut ppns, &node.text));
                    let mut errors = Vec::new();
                    root.find_all("error", &mut errors);
                    error = errors.first().map(|node| node.text.trim().to_owned());
                }
                _ => collect_ppns(&parse_json(INFO.id, response)?, &mut ppns, &mut error),
            }
            if ppns.is_empty()
                && let Some(message) = error
            {
                parsed.notes.push(format!("isbn2ppn: {}", crate::error::shorten(&message)));
            }
            parsed.follow_up = ppns
                .iter()
                .take(MAX_RECORDS)
                .map(|ppn| get_xml(join(&self.base, &format!("/{ppn}.xml"))))
                .collect();
            return Ok(parsed);
        }
        let root = parse_xml(INFO.id, response)?;
        let candidates: Vec<Extracted> = marc_records(&root)
            .iter()
            .map(|record| {
                let mut extracted = marc::unimarc(record);
                if let Some(ppn) = ppn {
                    extracted.url = Some(format!("https://www.sudoc.fr/{ppn}"));
                }
                extracted
            })
            .collect();
        if candidates.is_empty() {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "XML",
                detail: "no MARC record in the answer".to_owned(),
            });
        }
        parsed.records = marc::select(isbn, candidates, &mut parsed.notes)
            .map(|chosen| finalize(&INFO, isbn, chosen))
            .into_iter()
            .collect();
        Ok(parsed)
    }
}
