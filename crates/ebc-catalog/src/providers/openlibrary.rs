//! Open Library (Internet Archive).
//!
//! One request to the Books API with `jscmd=details` returns the whole edition record with
//! author names already resolved. Follow-ups are needed only for records that lack them:
//! `/authors/OL…A.json` for an author given by key alone, `/works/OL…W.json` when the
//! edition lists no author at all (the work then names them).

use ebc_isbn::Isbn13;
use ebc_model::{
    BookMeta, Contributor, ContributorRole, CoverKind, CoverRef, HttpRequest, HttpResponse,
};
use serde_json::Value;

use crate::error::CatalogError;
use crate::marc::{Extracted, role_from_term};
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{format_from_text, fragment, get_json, unknown_request};
use crate::util::{
    Found, check_http, isbns_in, jarr, jlist, jnum, jstr, jtext, jtexts, join, parse_json, path,
    push_isbn, url_with_query,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "openlibrary",
    name: "Open Library",
    homepage: "https://openlibrary.org/",
    data_licence: "CC0 1.0 (contributions to Open Library)",
    terms_url: "https://openlibrary.org/developers/api",
    requires_key: false,
    browser_cors: true,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["openlibrary.org"],
    notes: "Verified in Open Library's source: 180 requests/min per IP on /api and .json \
            paths, CORS `*` on those paths, contributions under CC0. Send a User-Agent with \
            an application name and a contact. Cover images carry no licence: the cover \
            references returned here are candidates for preview only.",
    class: SourceClass::Open,
    default_enabled: true,
};

/// At most this many author follow-ups per lookup.
const MAX_AUTHORS: usize = 5;

/// Open Library: `GET {base}/api/books?bibkeys=ISBN:{isbn}&format=json&jscmd=details`.
#[derive(Debug, Clone)]
pub struct OpenLibrary {
    base: String,
    covers_base: String,
}

impl Default for OpenLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenLibrary {
    /// The provider pointed at `https://openlibrary.org`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: "https://openlibrary.org".to_owned(),
            covers_base: "https://covers.openlibrary.org".to_owned(),
        }
    }

    /// Replaces the site root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }

    /// `/authors/OL123A` or `/works/OL123W`, and nothing else: keys come from the network
    /// and end up in a URL.
    fn valid_key(key: &str, folder: &str, suffix: char) -> bool {
        key.strip_prefix(folder)
            .and_then(|rest| rest.strip_prefix("OL"))
            .and_then(|rest| rest.strip_suffix(suffix))
            .is_some_and(|digits| {
                !digits.is_empty() && digits.len() <= 12 && digits.bytes().all(|b| b.is_ascii_digit())
            })
    }

    fn entity_request(&self, key: &str) -> HttpRequest {
        get_json(format!("{}.json", join(&self.base, key)))
    }

    fn parse_edition(&self, isbn: &Isbn13, response: &HttpResponse) -> Result<Parsed, CatalogError> {
        let json = parse_json(INFO.id, response)?;
        let Some(object) = json.as_object() else {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "JSON",
                detail: "expected an object keyed by bibkey".to_owned(),
            });
        };
        let mut parsed = Parsed::empty();
        let Some(entry) = object.get(&format!("ISBN:{isbn}")) else {
            if !object.is_empty() {
                parsed.notes.push("ignored an answer keyed by another ISBN".to_owned());
            }
            return Ok(parsed);
        };
        let details = entry.get("details").filter(|d| d.is_object()).unwrap_or(entry);

        let mut isbns = Vec::new();
        for key in ["isbn_13", "isbn_10"] {
            for value in jtexts(details, key, &[]) {
                for found in isbns_in(&value) {
                    push_isbn(&mut isbns, found);
                }
            }
        }
        if !isbns.is_empty() && !isbns.contains(isbn) {
            parsed.notes.push("ignored an edition that carries other ISBNs".to_owned());
            return Ok(parsed);
        }

        let mut book = BookMeta::default();
        let title = jstr(details, "title").unwrap_or("");
        book.title = match jstr(details, "title_prefix") {
            // Old records split the article off: "L'" + "étranger", "The" + "stranger".
            Some(prefix) if !title.is_empty() && prefix.ends_with(['\'', '’']) => {
                format!("{prefix}{title}")
            }
            Some(prefix) if !title.is_empty() => format!("{prefix} {title}"),
            _ => title.to_owned(),
        };
        if book.title.is_empty() {
            parsed.notes.push("ignored an edition without a title".to_owned());
            return Ok(parsed);
        }
        book.subtitle = jstr(details, "subtitle").map(str::to_owned);

        let mut author_keys: Vec<&str> = Vec::new();
        for author in jarr(details, "authors") {
            let key = jstr(author, "key").filter(|k| Self::valid_key(k, "/authors/", 'A'));
            match (jstr(author, "name"), key) {
                (Some(name), _) => book.contributors.push(Contributor::author(name)),
                (None, Some(key)) => author_keys.push(key),
                (None, None) => {}
            }
        }
        for contributor in jarr(details, "contributors") {
            if let Some(name) = jstr(contributor, "name") {
                let role = jstr(contributor, "role")
                    .and_then(role_from_term)
                    .unwrap_or_else(|| ContributorRole::Other("contributor".to_owned()));
                book.contributors.push(Contributor { name: name.to_owned(), role });
            }
        }
        if book.authors().next().is_none() && author_keys.is_empty() {
            // No author on the edition: the work names them.
            if let Some(work) = jarr(details, "works")
                .iter()
                .filter_map(|w| jstr(w, "key"))
                .find(|k| Self::valid_key(k, "/works/", 'W'))
            {
                parsed.follow_up.push(self.entity_request(work));
            }
        }
        for key in author_keys.into_iter().take(MAX_AUTHORS) {
            parsed.follow_up.push(self.entity_request(key));
        }

        book.publisher = jtexts(details, "publishers", &["name"]).into_iter().next();
        book.published = jstr(details, "publish_date").map(str::to_owned);
        book.language = jtexts(details, "languages", &["key"]).into_iter().next();
        book.page_count = jnum(details.get("number_of_pages")).or_else(|| {
            jstr(details, "pagination").and_then(crate::normalize::page_count)
        });
        book.series = jtexts(details, "series", &["name"]).into_iter().next();
        book.description = details.get("description").and_then(|d| jtext(d, &["value"]));
        book.subjects = jtexts(details, "subjects", &["name"]);
        book.format = jstr(details, "physical_format").and_then(format_from_text);

        for id in jarr(details, "covers").iter().filter_map(Value::as_i64).filter(|id| *id > 0) {
            book.covers.push(CoverRef {
                url: format!("{}/b/id/{id}-L.jpg", self.covers_base),
                kind: CoverKind::Front,
                width_px: None,
                height_px: None,
                source: INFO.id.to_owned(),
            });
        }
        if book.covers.is_empty()
            && let Some(thumbnail) = jstr(entry, "thumbnail_url")
        {
            book.covers.push(CoverRef {
                url: thumbnail.replace("-S.jpg", "-L.jpg"),
                kind: CoverKind::Front,
                width_px: None,
                height_px: None,
                source: INFO.id.to_owned(),
            });
        }
        let url = jstr(entry, "info_url")
            .filter(|u| u.starts_with("https://") || u.starts_with("http://"))
            .map(str::to_owned)
            .or_else(|| {
                jstr(details, "key")
                    .filter(|k| k.starts_with("/books/") && !k.contains(['?', '#', ' ']))
                    .map(|k| format!("https://openlibrary.org{k}"))
            });
        parsed.records.push(finalize(&INFO, isbn, Extracted { book, isbns, url }));
        Ok(parsed)
    }

    fn parse_author(&self, isbn: &Isbn13, response: &HttpResponse) -> Result<Parsed, CatalogError> {
        let json = parse_json(INFO.id, response)?;
        let mut parsed = Parsed::empty();
        let kind = json.get("type").and_then(|t| jtext(t, &["key"])).unwrap_or_default();
        if kind == "/type/redirect" {
            if let Some(location) =
                jstr(&json, "location").filter(|k| Self::valid_key(k, "/authors/", 'A'))
            {
                parsed.follow_up.push(self.entity_request(location));
            }
            return Ok(parsed);
        }
        match jstr(&json, "name").or_else(|| jstr(&json, "personal_name")) {
            Some(name) => {
                let book =
                    BookMeta { contributors: vec![Contributor::author(name)], ..BookMeta::default() };
                parsed.records.push(fragment(isbn, book));
            }
            None => parsed.notes.push("an author record has no name".to_owned()),
        }
        Ok(parsed)
    }

    fn parse_work(&self, isbn: &Isbn13, response: &HttpResponse) -> Result<Parsed, CatalogError> {
        let json = parse_json(INFO.id, response)?;
        let mut parsed = Parsed::empty();
        for entry in jarr(&json, "authors") {
            let key = entry
                .get("author")
                .and_then(|a| jtext(a, &["key"]))
                .or_else(|| jstr(entry, "key").map(str::to_owned));
            if let Some(key) = key.filter(|k| Self::valid_key(k, "/authors/", 'A'))
                && parsed.follow_up.len() < MAX_AUTHORS
            {
                parsed.follow_up.push(self.entity_request(&key));
            }
        }
        let book = BookMeta {
            description: json.get("description").and_then(|d| jtext(d, &["value"])),
            subjects: jlist(json.get("subjects")).into_iter().filter_map(|s| jtext(s, &[])).collect(),
            ..BookMeta::default()
        };
        if book.description.is_some() || !book.subjects.is_empty() {
            parsed.records.push(fragment(isbn, book));
        }
        Ok(parsed)
    }
}

impl Provider for OpenLibrary {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let bibkey = format!("ISBN:{isbn}");
        vec![get_json(url_with_query(
            &join(&self.base, "/api/books"),
            &[("bibkeys", &bibkey), ("format", "json"), ("jscmd", "details")],
        ))]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        let route = path(&request.url);
        let edition = route.ends_with("/api/books");
        let author = route.contains("/authors/") && route.ends_with(".json");
        let work = route.contains("/works/") && route.ends_with(".json");
        if !request.url.starts_with(&self.base) || !(edition || author || work) {
            return Err(unknown_request(&INFO, request));
        }
        if check_http(INFO.id, response)? == Found::No {
            if edition {
                // The Books API answers 200 `{}` for unknown ISBNs: a 404 is an outage.
                return Err(CatalogError::Status { provider: INFO.id, status: response.status });
            }
            let mut parsed = Parsed::empty();
            parsed.notes.push("a linked author or work record no longer exists".to_owned());
            return Ok(parsed);
        }
        if edition {
            self.parse_edition(isbn, response)
        } else if author {
            self.parse_author(isbn, response)
        } else {
            self.parse_work(isbn, response)
        }
    }
}
