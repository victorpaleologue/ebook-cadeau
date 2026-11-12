//! The `client` feature against a server on the loopback interface: no request leaves
//! the machine.

mod common;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{FR, fixture, isbn};
use ebc_catalog::client::{Cache, Client, ClientConfig, ClientError};
use ebc_catalog::providers::openlibrary::OpenLibrary;
use ebc_catalog::{
    CatalogError, Config, DEFAULT_USER_AGENT, LookupOptions, ProviderInfo, providers_for, wire_url,
};
use ebc_model::{HttpRequest, HttpResponse};

/// A canned answer: status, extra header line, body.
type Answer = (u16, &'static str, Vec<u8>);

/// A minimal HTTP/1.1 server: answers by the first route whose pattern occurs in the
/// request target, 404 otherwise, and records every request head.
struct Server {
    base: String,
    heads: Arc<Mutex<Vec<String>>>,
}

impl Server {
    fn start(routes: Vec<(&'static str, Answer)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let base = format!("http://{}", listener.local_addr().expect("address"));
        let heads = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&heads);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut head = Vec::new();
                let mut buffer = [0_u8; 1024];
                while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => head.extend_from_slice(&buffer[..n]),
                    }
                }
                let head = String::from_utf8_lossy(&head).into_owned();
                let target = head.split_whitespace().nth(1).unwrap_or("").to_owned();
                seen.lock().expect("lock").push(head);
                let (status, header, body) = routes
                    .iter()
                    .find(|(pattern, _)| target.contains(pattern))
                    .map_or((404, "", b"not found".to_vec()), |(_, answer)| answer.clone());
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n{header}Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Self { base, heads }
    }

    fn heads(&self) -> Vec<String> {
        self.heads.lock().expect("lock").clone()
    }
}

fn open_library_routes() -> Vec<(&'static str, Answer)> {
    vec![
        ("/api/books", (200, "", fixture("openlibrary/etranger-author-keys.json"))),
        ("/authors/OL124171A.json", (200, "", fixture("openlibrary/author-OL124171A.json"))),
        ("/authors/OL9999999A.json", (200, "", fixture("openlibrary/author-redirect.json"))),
    ]
}

fn local_client(min_interval: Duration) -> Client {
    Client::new(ClientConfig {
        min_interval: Some(min_interval),
        use_system_proxy: false,
        timeout: Duration::from_secs(5),
        ..ClientConfig::default()
    })
    .expect("client")
}

#[tokio::test]
async fn a_provider_runs_against_a_local_server() {
    let server = Server::start(open_library_routes());
    let provider = OpenLibrary::new().with_base_url(&server.base);
    let outcome =
        local_client(Duration::ZERO).lookup(&provider, &isbn(FR), 6).await.expect("found");
    assert_eq!(outcome.requests, 3);
    let book = outcome.record.expect("a record");
    assert_eq!(book.title, "L'étranger");
    assert_eq!(book.author_line().as_deref(), Some("Albert Camus"));

    let heads = server.heads();
    assert_eq!(heads.len(), 3);
    for head in &heads {
        let lower = head.to_lowercase();
        assert!(
            lower.contains(&format!("user-agent: {}", DEFAULT_USER_AGENT.to_lowercase())),
            "{head}"
        );
        assert!(lower.contains("accept: application/json"), "{head}");
        assert!(!head.lines().next().unwrap_or("").contains('#'), "fragments are not sent: {head}");
    }
    assert!(DEFAULT_USER_AGENT.starts_with("ebook-cadeau/"));
    assert!(DEFAULT_USER_AGENT.ends_with("(+https://github.com/victorpaleologue/ebook-cadeau)"));
}

#[tokio::test]
async fn requests_to_one_host_are_spaced() {
    let server = Server::start(open_library_routes());
    let provider = OpenLibrary::new().with_base_url(&server.base);
    let interval = Duration::from_millis(120);
    let started = Instant::now();
    let outcome = local_client(interval).lookup(&provider, &isbn(FR), 6).await.expect("found");
    assert_eq!(outcome.requests, 3);
    assert!(
        started.elapsed() >= interval * 2,
        "three requests need two intervals: {:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn excluded_hosts_are_refused_before_any_io() {
    let client = local_client(Duration::ZERO);
    for url in [
        "https://www.amazon.fr/dp/2070360024",
        "https://www.goodreads.com/book/isbn/2070360024",
        "https://search.worldcat.org/api/search?q=bn:9782070360024&key=SECRET",
        "ftp://example.org/x",
        "not a url",
    ] {
        let error =
            client.execute(&HttpRequest::get(url), Duration::ZERO).await.expect_err("refused");
        assert!(matches!(error, ClientError::Denied(_)), "{url}: {error:?}");
        assert!(!error.to_string().contains("SECRET"));
    }
    // A redirect towards an excluded host is not followed.
    let server = Server::start(vec![(
        "/hop",
        (302, "Location: https://www.amazon.fr/dp/2070360024\r\n", Vec::new()),
    )]);
    let request = HttpRequest::get(format!("{}/hop", server.base));
    let error = client.execute(&request, Duration::ZERO).await.expect_err("refused");
    assert!(matches!(error, ClientError::Http(_)), "{error:?}");
}

#[tokio::test]
async fn oversized_bodies_and_dead_servers_are_errors() {
    let server = Server::start(vec![("/big", (200, "", vec![b' '; 5000]))]);
    let client = Client::new(ClientConfig {
        max_body_bytes: 1000,
        min_interval: Some(Duration::ZERO),
        use_system_proxy: false,
        ..ClientConfig::default()
    })
    .expect("client");
    let error = client
        .execute(&HttpRequest::get(format!("{}/big", server.base)), Duration::ZERO)
        .await
        .expect_err("too large");
    assert_eq!(error, ClientError::TooLarge(1000));

    // Nothing listens on a port that was just released.
    let free = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        format!("http://{}", listener.local_addr().expect("address"))
    };
    let provider = OpenLibrary::new().with_base_url(free);
    let error = client.lookup(&provider, &isbn(FR), 6).await.expect_err("unreachable");
    assert!(matches!(error, CatalogError::Transport { provider: "openlibrary", .. }), "{error:?}");
}

#[derive(Default)]
struct MemoryCache {
    entries: Mutex<HashMap<String, HttpResponse>>,
    offered_by: Mutex<Vec<&'static str>>,
}

impl Cache for MemoryCache {
    fn get(&self, _provider: &ProviderInfo, request: &HttpRequest) -> Option<HttpResponse> {
        self.entries.lock().expect("lock").get(wire_url(request)).cloned()
    }

    fn put(&self, provider: &ProviderInfo, request: &HttpRequest, response: &HttpResponse) {
        self.offered_by.lock().expect("lock").push(provider.id);
        self.entries.lock().expect("lock").insert(wire_url(request).to_owned(), response.clone());
    }
}

#[tokio::test]
async fn the_cache_hook_is_consulted_and_fed() {
    let server = Server::start(open_library_routes());
    let provider = OpenLibrary::new().with_base_url(&server.base);
    let cache = Arc::new(MemoryCache::default());
    let client = local_client(Duration::ZERO).with_cache(cache.clone());
    let first = client.lookup(&provider, &isbn(FR), 6).await.expect("found");
    assert_eq!(server.heads().len(), 3);
    assert_eq!(*cache.offered_by.lock().expect("lock"), ["openlibrary"; 3]);
    let second = client.lookup(&provider, &isbn(FR), 6).await.expect("found");
    assert_eq!(server.heads().len(), 3, "the second lookup is served from the cache");
    assert_eq!(first.record, second.record);
}

#[tokio::test]
async fn a_cascade_runs_with_base_url_overrides() {
    let server = Server::start(vec![
        ("/bnf", (200, "", fixture("bnf/not-found.xml"))),
        ("/ol/api/books", (200, "", fixture("openlibrary/etranger.json"))),
        ("/google", (500, "", Vec::new())),
    ]);
    let config = Config::default()
        .with_google_books_key("SECRET")
        .with_base_url("bnf", format!("{}/bnf", server.base))
        .with_base_url("openlibrary", format!("{}/ol", server.base))
        .with_base_url("googlebooks", format!("{}/google", server.base));
    let providers = providers_for(&isbn(FR), &config);
    let client = local_client(Duration::ZERO);
    let report = client.lookup_all(&providers, &isbn(FR), &LookupOptions::default()).await;
    assert_eq!(report.skipped, ["googlebooks"]);
    assert_eq!(report.merged.expect("merged").author_line().as_deref(), Some("Albert Camus"));
    assert_eq!(server.heads().len(), 2);

    let all = LookupOptions { stop_when_complete: false, ..LookupOptions::default() };
    let report = client.lookup_all(&providers, &isbn(FR), &all).await;
    assert!(matches!(
        report.providers[2].result,
        Err(CatalogError::Unavailable { status: 500, .. })
    ));
    assert!(report.merged.is_some());
}
