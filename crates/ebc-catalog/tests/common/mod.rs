//! Helpers shared by the integration tests: fixture files as HTTP responses.
#![allow(dead_code)]

use std::path::PathBuf;

use ebc_catalog::wire_url;
use ebc_model::{HttpRequest, HttpResponse, Isbn13};

/// *L'Étranger*, Folio.
pub const FR: &str = "9782070360024";
/// *Die Verwandlung* (invented edition).
pub const DE: &str = "9783596294312";
/// *Pride and Prejudice*, Penguin Classics.
pub const EN: &str = "9780141439518";
/// A 979-10 ISBN: no ISBN-10 form.
pub const FR_979: &str = "9791032102763";
/// The invented scholarly book of the Crossref fixture.
pub const SCHOLARLY: &str = "9780061120084";

pub fn isbn(text: &str) -> Isbn13 {
    Isbn13::parse(text).expect("test ISBN is valid")
}

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

/// The bytes of `tests/fixtures/<path>`.
pub fn fixture(path: &str) -> Vec<u8> {
    let file = fixtures_dir().join(path);
    std::fs::read(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

pub fn response(status: u16, content_type: &str, body: Vec<u8>) -> HttpResponse {
    HttpResponse {
        status,
        headers: vec![("Content-Type".to_owned(), content_type.to_owned())],
        body,
        final_url: "https://fixture.invalid/".to_owned(),
    }
}

/// A 200 response serving a fixture, with the content type its extension suggests.
pub fn ok(path: &str) -> HttpResponse {
    let content_type = if path.ends_with(".xml") {
        "application/xml; charset=utf-8"
    } else if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else {
        "application/json; charset=utf-8"
    };
    response(200, content_type, fixture(path))
}

/// A fixture map: the first route whose pattern occurs in the request URL (without its
/// fragment) answers. Unknown URLs are a transport error.
pub struct Routes(pub Vec<(&'static str, HttpResponse)>);

impl Routes {
    pub fn of(routes: &[(&'static str, &str)]) -> Self {
        Self(routes.iter().map(|(pattern, path)| (*pattern, ok(path))).collect())
    }

    pub fn exec(&self) -> impl FnMut(&HttpRequest) -> Result<HttpResponse, String> + '_ {
        move |request| {
            let url = wire_url(request);
            assert!(!url.contains('#'), "the wire URL carries no fragment");
            self.0
                .iter()
                .find(|(pattern, _)| url.contains(pattern))
                .map(|(_, response)| response.clone())
                .ok_or_else(|| format!("no fixture for {url}"))
        }
    }
}
