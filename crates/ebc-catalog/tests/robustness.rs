//! No provider's `parse` may panic, whatever the bytes: every fixture truncated at many
//! offsets, corrupted at random positions, and pseudo-random garbage, fed to every request
//! shape of every provider.

mod common;

use std::panic::{AssertUnwindSafe, catch_unwind};

use common::{FR, fixtures_dir, isbn, response};
use ebc_catalog::{Config, PROVIDER_IDS, Provider, provider_by_id};
use ebc_model::{HttpRequest, Isbn13};

/// Deterministic pseudo-random bytes (xorshift64*): tests must not depend on a clock or
/// an entropy source.
struct Noise(u64);

impl Noise {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % (bound.max(1) as u64)).expect("fits")
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| (self.next() >> 32) as u8).collect()
    }
}

/// Every fixture file: `(relative path, bytes)`.
fn all_fixtures() -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    let root = fixtures_dir();
    let mut directories: Vec<_> = std::fs::read_dir(&root)
        .expect("fixtures directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    directories.sort();
    for directory in directories {
        let mut entries: Vec<_> = std::fs::read_dir(&directory)
            .expect("provider directory")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        entries.sort();
        for path in entries {
            let name = path.strip_prefix(&root).expect("under root").display().to_string();
            files.push((name, std::fs::read(&path).expect("readable fixture")));
        }
    }
    assert!(files.len() >= 50, "found only {} fixtures", files.len());
    files
}

/// The request shapes of a provider: its first request(s) plus every follow-up it asks
/// for when given any fixture, transitively.
fn request_shapes(
    provider: &dyn Provider,
    isbn: &Isbn13,
    fixtures: &[(String, Vec<u8>)],
) -> Vec<HttpRequest> {
    let mut shapes = provider.lookup(isbn);
    let mut next = 0;
    while next < shapes.len() && shapes.len() < 12 {
        let request = shapes[next].clone();
        next += 1;
        for (_, body) in fixtures {
            if let Ok(parsed) = provider.parse(isbn, &request, &response(200, "", body.clone())) {
                for follow_up in parsed.follow_up {
                    if !shapes.iter().any(|known| known.url == follow_up.url) && shapes.len() < 12 {
                        shapes.push(follow_up);
                    }
                }
            }
        }
    }
    shapes
}

/// Parses `body` and checks the invariants of a successful parse. Returns a description
/// of the failure, if any.
fn check(
    provider: &dyn Provider,
    isbn: &Isbn13,
    request: &HttpRequest,
    status: u16,
    body: &[u8],
) -> Option<String> {
    let answer = response(status, "", body.to_vec());
    let result = catch_unwind(AssertUnwindSafe(|| provider.parse(isbn, request, &answer)));
    match result {
        Err(_) => Some("panicked".to_owned()),
        Ok(Err(error)) => (error.provider() != provider.info().id)
            .then(|| format!("error of another provider: {error}")),
        Ok(Ok(parsed)) => {
            for record in &parsed.records {
                if record.isbn13 != Some(*isbn) {
                    return Some(format!("a record lost the ISBN: {:?}", record.isbn13));
                }
                if record.covers.iter().any(|c| !c.url.starts_with("http")) {
                    return Some("a cover reference is not a web URL".to_owned());
                }
            }
            parsed
                .follow_up
                .iter()
                .find(|f| !f.url.starts_with("http://") && !f.url.starts_with("https://"))
                .map(|f| format!("a follow-up is not an http(s) URL: {}", f.url))
        }
    }
}

fn providers() -> Vec<Box<dyn Provider>> {
    let config = Config::default().with_google_books_key("SECRET");
    PROVIDER_IDS.iter().map(|id| provider_by_id(id, &config).expect("known id")).collect()
}

#[test]
fn truncated_fixtures_never_panic() {
    let fixtures = all_fixtures();
    let isbn = isbn(FR);
    let mut failures = Vec::new();
    let mut parses = 0_u64;
    for provider in providers() {
        let provider = provider.as_ref();
        let shapes = request_shapes(provider, &isbn, &fixtures);
        assert!(!shapes.is_empty());
        for request in &shapes {
            for (name, body) in &fixtures {
                // Every offset near the start, then about sixty spread over the file, and
                // the complete file.
                let step = (body.len() / 61).max(1);
                let offsets = (0..body.len().min(48))
                    .chain((48..body.len()).step_by(step))
                    .chain([body.len()]);
                for offset in offsets {
                    parses += 1;
                    if let Some(problem) = check(provider, &isbn, request, 200, &body[..offset]) {
                        failures.push(format!(
                            "{} {name} cut at {offset}: {problem}",
                            provider.info().id
                        ));
                    }
                }
                // The same bytes under an error status.
                for status in [404, 429, 500] {
                    parses += 1;
                    if let Some(problem) = check(provider, &isbn, request, status, body) {
                        failures.push(format!(
                            "{} {name} status {status}: {problem}",
                            provider.info().id
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
    assert!(parses > 20_000, "only {parses} parses ran");
}

#[test]
fn corrupted_fixtures_never_panic() {
    let fixtures = all_fixtures();
    let isbn = isbn(FR);
    let mut noise = Noise(0x9E37_79B9_7F4A_7C15);
    let mut failures = Vec::new();
    for provider in providers() {
        let provider = provider.as_ref();
        let shapes = request_shapes(provider, &isbn, &fixtures);
        for (name, body) in fixtures.iter().filter(|(_, body)| !body.is_empty()) {
            for round in 0..24 {
                let mut corrupted = body.clone();
                // Overwrite, delete or duplicate a few bytes.
                for _ in 0..=noise.below(4) {
                    let at = noise.below(corrupted.len());
                    match round % 3 {
                        0 => corrupted[at] = (noise.next() >> 24) as u8,
                        1 => {
                            corrupted.remove(at);
                        }
                        _ => {
                            let end = (at + noise.below(40)).min(corrupted.len());
                            let copy = corrupted[at..end].to_vec();
                            corrupted.splice(at..at, copy);
                        }
                    }
                    if corrupted.is_empty() {
                        break;
                    }
                }
                let request = &shapes[noise.below(shapes.len())];
                if let Some(problem) = check(provider, &isbn, request, 200, &corrupted) {
                    failures
                        .push(format!("{} {name} round {round}: {problem}", provider.info().id));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn garbage_never_panics() {
    let fixtures = all_fixtures();
    let isbn = isbn(FR);
    let mut noise = Noise(0x0123_4567_89AB_CDEF);
    let mut bodies: Vec<Vec<u8>> = Vec::new();
    for _ in 0..150 {
        let len = noise.below(1500);
        bodies.push(noise.bytes(len));
    }
    // Garbage that starts like the expected format, so that the real parsers run.
    for prefix in [
        &b"{"[..],
        b"[",
        b"{\"items\":[{",
        b"{\"entities\":{\"isbn:9782070360024\":{\"claims\":",
        b"{\"results\":{\"bindings\":[{\"item\":",
        b"{\"sudoc\":",
        b"<",
        b"<?xml version=\"1.0\"?><searchRetrieveResponse><records><record><recordData><record><datafield tag=\"",
        b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><record><controlfield tag=\"",
        b"\xEF\xBB\xBF",
        b"\xFF\xFE",
        b"\xFE\xFF<\x00",
    ] {
        for _ in 0..20 {
            let mut body = prefix.to_vec();
            let len = noise.below(600);
            body.extend(noise.bytes(len));
            bodies.push(body);
        }
    }
    // Pathological shapes: deep nesting, huge tokens, entity floods, lone surrogates.
    bodies.push("[".repeat(200_000).into_bytes());
    bodies.push("{\"a\":".repeat(100_000).into_bytes());
    bodies.push("<a>".repeat(100_000).into_bytes());
    bodies.push(format!("<a>{}</a>", "&amp;".repeat(100_000)).into_bytes());
    bodies.push(format!("<a b=\"{}\"/>", "&#x41;".repeat(50_000)).into_bytes());
    bodies.push(format!("{{\"title\":\"{}\"}}", "\\ud800".repeat(1000)).into_bytes());
    bodies.push(format!("\"{}", "é".repeat(100_000)).into_bytes());
    bodies.push(b"<!DOCTYPE a [<!ENTITY a \"&b;&b;\"><!ENTITY b \"&a;&a;\">]><a>&a;</a>".to_vec());
    bodies.push(vec![0; 4096]);
    bodies.push(vec![b'<'; 4096]);
    bodies.push(vec![b'&'; 4096]);

    let mut failures = Vec::new();
    for provider in providers() {
        let provider = provider.as_ref();
        let shapes = request_shapes(provider, &isbn, &fixtures);
        for request in &shapes {
            for (index, body) in bodies.iter().enumerate() {
                for status in [200, 404] {
                    if let Some(problem) = check(provider, &isbn, request, status, body) {
                        failures.push(format!(
                            "{} body #{index} status {status}: {problem}",
                            provider.info().id
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn hostile_headers_and_urls_never_panic() {
    let isbn = isbn(FR);
    for provider in providers() {
        let provider = provider.as_ref();
        let first = provider.lookup(&isbn).into_iter().next().expect("one request");
        let mut answer = response(200, "\u{0}; charset=\"\u{202e}", b"{}".to_vec());
        answer.headers.push(("Retry-After".to_owned(), "-1e99999999".to_owned()));
        answer.headers.push((String::new(), String::new()));
        let _ = provider.parse(&isbn, &first, &answer);
        answer.status = 429;
        let _ = provider.parse(&isbn, &first, &answer);
        // Requests parse() never issued, with URLs made to trip slicing.
        for url in [
            String::new(),
            "#".to_owned(),
            "://".to_owned(),
            "https://".to_owned(),
            format!("{}é#ebc=%", first.url),
            format!("{}#ebc=%FF%FE", first.url),
            format!("{}?uris=%E9|||&uris=", first.url),
            "https://[::1".to_owned(),
            "é".repeat(5000),
        ] {
            let _ = provider.parse(&isbn, &HttpRequest::get(url), &answer);
        }
    }
}
