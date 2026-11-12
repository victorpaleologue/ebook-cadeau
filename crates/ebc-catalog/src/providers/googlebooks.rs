//! Google Books API v1 (`volumes?q=isbn:`). Class B: display-only, nothing stored.

use ebc_isbn::Isbn13;
use ebc_model::{
    BookFormat, BookMeta, Contributor, CoverKind, CoverRef, HttpRequest, HttpResponse,
};
use serde_json::Value;

use crate::error::{CatalogError, shorten};
use crate::marc::Extracted;
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{get_json, unknown_request};
use crate::util::{
    Found, Sniff, check_http, isbns_in, jarr, jnum, join, jstr, jtexts, parse_json, path,
    push_isbn, sniff, url_with_query,
};

static INFO: ProviderInfo = ProviderInfo {
    id: "googlebooks",
    name: "Google Books",
    homepage: "https://books.google.com/",
    data_licence: "Google APIs Terms of Service and Books API terms (proprietary)",
    terms_url: "https://developers.google.com/books/terms",
    requires_key: true,
    browser_cors: true,
    commercial_use: Permission::Conditional,
    attribution: Some("Powered by Google"),
    min_interval_ms: 1000,
    licence_id: None,
    permanent_storage: Permission::Forbidden,
    hosts: &["www.googleapis.com", "books.googleapis.com"],
    notes: "Every term here is from memory and unread: no scraping, no permanent copy (cache \
            in memory for the session only), \"Powered by Google\" branding with a link to \
            the volume page, about 1,000 requests/day per project by default — stop at 900. \
            Keyless calls may still be served but must not be used in production. The list \
            call has no `country` parameter; results depend on the caller's IP. Thumbnails \
            carry no reuse right.",
    class: SourceClass::TermsBound,
    default_enabled: true,
};

/// Image sizes of `volumeInfo.imageLinks`, best first.
const IMAGE_SIZES: [&str; 6] =
    ["extraLarge", "large", "medium", "small", "thumbnail", "smallThumbnail"];

/// Google Books: `GET {base}/books/v1/volumes?q=isbn:{isbn}&maxResults=5&key={key}`.
#[derive(Debug, Clone)]
pub struct GoogleBooks {
    base: String,
    key: Option<String>,
}

impl GoogleBooks {
    /// The provider pointed at `https://www.googleapis.com`. Without a key the `key`
    /// parameter is omitted; [`default_providers`](crate::default_providers) only includes
    /// Google Books when a key is configured.
    #[must_use]
    pub fn new(key: Option<String>) -> Self {
        Self {
            base: "https://www.googleapis.com".to_owned(),
            key: key.map(|k| k.trim().to_owned()).filter(|k| !k.is_empty()),
        }
    }

    /// Replaces the API root (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_owned();
        self
    }

    /// Turns Google's JSON error envelope into a typed error.
    fn api_error(json: &Value, status: u16) -> Option<CatalogError> {
        let error = json.get("error").filter(|e| e.is_object())?;
        let message = shorten(jstr(error, "message").unwrap_or("no message"));
        let mut reasons: Vec<String> = jarr(error, "errors")
            .iter()
            .filter_map(|e| jstr(e, "reason"))
            .map(str::to_owned)
            .collect();
        reasons.extend(jstr(error, "status").map(str::to_owned));
        let code = jnum(error.get("code")).and_then(|c| u16::try_from(c).ok()).unwrap_or(status);
        let any = |needles: &[&str]| {
            reasons.iter().any(|r| needles.iter().any(|n| r.eq_ignore_ascii_case(n)))
        };
        Some(
            if code == 429
                || any(&[
                    "rateLimitExceeded",
                    "userRateLimitExceeded",
                    "dailyLimitExceeded",
                    "quotaExceeded",
                    "RESOURCE_EXHAUSTED",
                ])
            {
                CatalogError::RateLimited { provider: INFO.id, status, retry_after_s: None }
            } else if matches!(code, 401 | 403)
                || any(&[
                    "keyInvalid",
                    "keyExpired",
                    "forbidden",
                    "PERMISSION_DENIED",
                    "accessNotConfigured",
                ])
                || message.contains("API key")
            {
                CatalogError::Unauthorized { provider: INFO.id, status, message }
            } else if (500..600).contains(&code) {
                CatalogError::Unavailable { provider: INFO.id, status, retry_after_s: None }
            } else {
                CatalogError::Remote { provider: INFO.id, message }
            },
        )
    }

    fn volume(item: &Value) -> Option<Extracted> {
        let info = item.get("volumeInfo").filter(|v| v.is_object())?;
        let mut isbns = Vec::new();
        for identifier in jarr(info, "industryIdentifiers") {
            if jstr(identifier, "type").is_some_and(|t| t.starts_with("ISBN"))
                && let Some(value) = jstr(identifier, "identifier")
            {
                for isbn in isbns_in(value) {
                    push_isbn(&mut isbns, isbn);
                }
            }
        }
        let mut book = BookMeta {
            title: jstr(info, "title")?.to_owned(),
            subtitle: jstr(info, "subtitle").map(str::to_owned),
            contributors: jtexts(info, "authors", &[])
                .into_iter()
                .map(Contributor::author)
                .collect(),
            publisher: jstr(info, "publisher").map(str::to_owned),
            published: jstr(info, "publishedDate").map(str::to_owned),
            language: jstr(info, "language").map(str::to_owned),
            page_count: jnum(info.get("pageCount")),
            description: jstr(info, "description").map(str::to_owned),
            subjects: jtexts(info, "categories", &[]),
            ..BookMeta::default()
        };
        let is_ebook = item
            .get("saleInfo")
            .and_then(|s| s.get("isEbook"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if is_ebook {
            book.format = Some(BookFormat::Ebook);
        }
        if let Some(links) = info.get("imageLinks") {
            for size in IMAGE_SIZES {
                if let Some(url) = jstr(links, size) {
                    book.covers.push(CoverRef {
                        // The API still hands out http:// links; the host serves https.
                        url: match url.strip_prefix("http://") {
                            Some(rest) => format!("https://{rest}"),
                            None => url.to_owned(),
                        },
                        kind: CoverKind::Front,
                        width_px: None,
                        height_px: None,
                        source: INFO.id.to_owned(),
                    });
                }
            }
        }
        let url = ["canonicalVolumeLink", "infoLink"]
            .iter()
            .find_map(|k| jstr(info, k))
            .filter(|u| u.starts_with("https://") || u.starts_with("http://"))
            .map(str::to_owned);
        Some(Extracted { book, isbns, url })
    }
}

impl Provider for GoogleBooks {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let query = format!("isbn:{isbn}");
        let mut params = vec![("q", query.as_str()), ("maxResults", "5")];
        if let Some(key) = &self.key {
            params.push(("key", key.as_str()));
        }
        vec![get_json(url_with_query(&join(&self.base, "/books/v1/volumes"), &params))]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        if !request.url.starts_with(&self.base)
            || !path(&request.url).ends_with("/books/v1/volumes")
        {
            return Err(unknown_request(&INFO, request));
        }
        // Google reports quota and key problems as JSON with a 4xx status: read that first.
        if !response.is_success()
            && sniff(&response.body) == Sniff::Json
            && let Ok(json) = parse_json(INFO.id, response)
            && let Some(error) = Self::api_error(&json, response.status)
        {
            return Err(error);
        }
        if check_http(INFO.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: INFO.id, status: response.status });
        }
        let json = parse_json(INFO.id, response)?;
        if let Some(error) = Self::api_error(&json, response.status) {
            return Err(error);
        }
        if !json.is_object() || (json.get("items").is_none() && json.get("totalItems").is_none()) {
            return Err(CatalogError::Malformed {
                provider: INFO.id,
                format: "JSON",
                detail: "neither `items` nor `totalItems` in the answer".to_owned(),
            });
        }
        let mut parsed = Parsed::empty();
        for item in jarr(&json, "items") {
            let Some(volume) = Self::volume(item) else {
                parsed.notes.push("ignored a volume without a title".to_owned());
                continue;
            };
            if !volume.isbns.contains(isbn) {
                parsed.notes.push("ignored a volume that does not carry the ISBN".to_owned());
                continue;
            }
            if parsed.records.is_empty() {
                parsed.records.push(finalize(&INFO, isbn, volume));
            }
        }
        Ok(parsed)
    }
}
