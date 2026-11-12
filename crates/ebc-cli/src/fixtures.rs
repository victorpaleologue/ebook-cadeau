//! `--offline-fixtures DIR`: HTTP answers read from a directory instead of the network.
//!
//! The directory holds an `index.json` — a JSON array, the format written by
//! `cargo run -p ebc-catalog --example record-fixtures` — and the response bodies:
//!
//! ```json
//! [
//!   { "url": "https://openlibrary.org/api/books?bibkeys=ISBN:9782070360024&format=json&jscmd=details",
//!     "status": 200, "content_type": "application/json", "file": "openlibrary/9782070360024-1.json" },
//!   { "url_contains": "/robots.txt", "status": 404 }
//! ]
//! ```
//!
//! An entry answers a request when its `url` equals the request URL (without fragment)
//! or, failing an exact match anywhere in the index, when its `url_contains` occurs in
//! it; the first such entry wins. `status` defaults to 200; the body is `file` (relative
//! to the directory, never outside it), or the inline `body` string, or empty.
//! `headers` (an object) and `final_url` are optional; unknown keys are ignored. A
//! request without an entry fails like an unreachable host.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use ebc_catalog::wire_url;
use ebc_model::{HttpRequest, HttpResponse, Method};
use serde::Deserialize;

use crate::error::CliError;

/// Largest response body read from a fixture file.
const MAX_FIXTURE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct Entry {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    url_contains: Option<String>,
    #[serde(default)]
    status: Option<u16>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    final_url: Option<String>,
}

/// A directory of recorded or hand-written HTTP answers.
#[derive(Debug)]
pub(crate) struct Fixtures {
    dir: PathBuf,
    entries: Vec<Entry>,
}

impl Fixtures {
    /// Reads `<dir>/index.json`.
    pub(crate) fn open(dir: &Path) -> Result<Self, CliError> {
        let index = dir.join("index.json");
        let text = std::fs::read(&index).map_err(|e| CliError::file(&index, e))?;
        let entries: Vec<Entry> = serde_json::from_slice(&text).map_err(|e| {
            CliError::failure(format!("{}: not a fixture index: {e}", index.display()))
        })?;
        Ok(Self { dir: dir.to_path_buf(), entries })
    }

    fn find(&self, url: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.url.as_deref() == Some(url)).or_else(|| {
            self.entries.iter().find(|entry| {
                entry.url_contains.as_deref().is_some_and(|part| !part.is_empty() && url.contains(part))
            })
        })
    }

    fn body_of(&self, entry: &Entry) -> Result<Vec<u8>, String> {
        let Some(file) = &entry.file else {
            return Ok(entry.body.clone().unwrap_or_default().into_bytes());
        };
        let relative = Path::new(file);
        if !relative.components().all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err(format!("fixture file {file:?} is not a path inside the fixture directory"));
        }
        let path = self.dir.join(relative);
        let size = std::fs::metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?.len();
        if size > MAX_FIXTURE_BYTES {
            return Err(format!("{}: fixture larger than {MAX_FIXTURE_BYTES} bytes", path.display()));
        }
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The recorded answer to `request`.
    ///
    /// # Errors
    /// A message, in the manner of a transport error, when no entry matches or the body
    /// file cannot be read.
    pub(crate) fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, String> {
        let url = wire_url(request);
        let entry = self.find(url).ok_or_else(|| "no fixture for this URL (offline)".to_owned())?;
        let body = self.body_of(entry)?;
        let mut headers: Vec<(String, String)> =
            entry.headers.iter().map(|(name, value)| (name.clone(), value.clone())).collect();
        let has = |headers: &[(String, String)], name: &str| {
            headers.iter().any(|(n, _)| n.eq_ignore_ascii_case(name))
        };
        if let Some(content_type) = &entry.content_type
            && !has(&headers, "content-type")
        {
            headers.push(("Content-Type".to_owned(), content_type.clone()));
        }
        if !has(&headers, "content-length") {
            headers.push(("Content-Length".to_owned(), body.len().to_string()));
        }
        Ok(HttpResponse {
            status: entry.status.unwrap_or(200),
            headers,
            // A HEAD answer announces the body's length but carries none.
            body: if request.method == Method::Head { Vec::new() } else { body },
            final_url: entry.final_url.clone().unwrap_or_else(|| url.to_owned()),
        })
    }
}
