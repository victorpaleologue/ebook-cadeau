//! `Session`, `drive` and `lookup_all`: follow-ups, request budgets, refusals, errors and
//! the provider cascade.

mod common;

use std::cell::RefCell;

use common::{DE, FR, Routes, fixture, isbn, ok, response};
use ebc_catalog::providers::openlibrary::OpenLibrary;
use ebc_catalog::{
    CatalogError, Config, LookupOptions, Parsed, Permission, Provider, ProviderInfo, Session,
    SourceClass, drive, lookup_all, providers_for, wire_url,
};
use ebc_model::{BookMeta, Contributor, HttpRequest, HttpResponse, Isbn13};

static SCRIPTED: ProviderInfo = ProviderInfo {
    id: "scripted",
    name: "Scripted test provider",
    homepage: "https://scripted.example/",
    data_licence: "none",
    terms_url: "https://scripted.example/terms",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Unknown,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: None,
    permanent_storage: Permission::Unknown,
    hosts: &["scripted.example", "cdn.scripted.example"],
    notes: "Test double.",
    class: SourceClass::Open,
    default_enabled: false,
};

/// A provider whose responses are scripts, one instruction per line:
/// `title:<t>`, `author:<name>`, `follow:<url>`, `fail`.
struct Scripted;

impl Provider for Scripted {
    fn info(&self) -> &ProviderInfo {
        &SCRIPTED
    }

    fn lookup(&self, _isbn: &Isbn13) -> Vec<HttpRequest> {
        vec![HttpRequest::get("https://scripted.example/start")]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        _request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        let mut parsed = Parsed::empty();
        let mut book = BookMeta { isbn13: Some(*isbn), ..BookMeta::default() };
        for line in response.text().lines() {
            match line.split_once(':') {
                Some(("title", title)) => book.title = title.to_owned(),
                Some(("author", name)) => book.contributors.push(Contributor::author(name)),
                Some(("follow", url)) => parsed.follow_up.push(HttpRequest::get(url)),
                _ if line == "fail" => {
                    return Err(CatalogError::Remote {
                        provider: "scripted",
                        message: "scripted failure".to_owned(),
                    });
                }
                _ => {}
            }
        }
        if !book.title.is_empty() || !book.contributors.is_empty() {
            parsed.records.push(book);
        }
        Ok(parsed)
    }
}

fn scripted(
    script: &'static [(&'static str, &'static str)],
) -> impl FnMut(&HttpRequest) -> Result<HttpResponse, String> {
    move |request| {
        script
            .iter()
            .find(|(url, _)| *url == wire_url(request))
            .map(|(_, body)| response(200, "text/plain", body.as_bytes().to_vec()))
            .ok_or_else(|| format!("connection refused: {}", request.url))
    }
}

#[test]
fn follow_ups_are_executed_in_order_and_assembled() {
    let outcome = drive(
        &Scripted,
        &isbn(FR),
        scripted(&[
            (
                "https://scripted.example/start",
                "title:T\nfollow:https://scripted.example/a\nfollow:https://cdn.scripted.example/b",
            ),
            (
                "https://scripted.example/a",
                "author:First Author\nfollow:https://scripted.example/c",
            ),
            (
                "https://cdn.scripted.example/b",
                "author:Second Author\ntitle:ignored, the first title wins",
            ),
            ("https://scripted.example/c", "author:Third Author"),
        ]),
        10,
    )
    .expect("found");
    assert_eq!(outcome.requests, 4);
    assert!(!outcome.truncated);
    let book = outcome.record.expect("a record");
    assert_eq!(book.title, "T");
    assert_eq!(book.author_line().as_deref(), Some("First Author, Second Author & Third Author"));
}

#[test]
fn the_request_budget_stops_runaway_chains() {
    // Every response asks for two more pages.
    let calls = RefCell::new(Vec::new());
    let exec = |request: &HttpRequest| -> Result<HttpResponse, String> {
        calls.borrow_mut().push(request.url.clone());
        let n = calls.borrow().len();
        let body = format!(
            "title:T\nfollow:https://scripted.example/p{}\nfollow:https://scripted.example/p{}",
            2 * n,
            2 * n + 1
        );
        Ok(response(200, "text/plain", body.into_bytes()))
    };
    let outcome = drive(&Scripted, &isbn(FR), exec, 5).expect("found");
    assert_eq!(outcome.requests, 5);
    assert_eq!(calls.borrow().len(), 5);
    assert!(outcome.truncated);
    assert!(outcome.notes.iter().any(|n| n.contains("stopped after 5 requests")));

    let nothing = drive(&Scripted, &isbn(FR), scripted(&[]), 0).expect("no request, no error");
    assert_eq!((nothing.requests, nothing.record, nothing.truncated), (0, None, true));
}

#[test]
fn loops_and_foreign_hosts_are_not_followed() {
    let calls = RefCell::new(Vec::new());
    let exec = |request: &HttpRequest| -> Result<HttpResponse, String> {
        calls.borrow_mut().push(request.url.clone());
        let body = "title:T\n\
                    follow:https://scripted.example/start\n\
                    follow:https://evil.example/steal\n\
                    follow:http://scripted.example/downgrade\n\
                    follow:https://scripted.example@evil.example/x\n\
                    follow:https://www.amazon.fr/dp/2070360024\n\
                    follow:file:///etc/passwd\n\
                    follow:javascript:alert(1)";
        Ok(response(200, "text/plain", body.as_bytes().to_vec()))
    };
    let outcome = drive(&Scripted, &isbn(FR), exec, 10).expect("found");
    assert_eq!(*calls.borrow(), ["https://scripted.example/start"], "nothing else was requested");
    assert_eq!(outcome.requests, 1);
    assert_eq!(
        outcome.notes.iter().filter(|n| n.contains("refused a request")).count(),
        6,
        "{:?}",
        outcome.notes
    );
    assert!(outcome.record.is_some(), "the record survives the refused follow-ups");
}

#[test]
fn a_denied_base_url_is_never_contacted() {
    let provider = OpenLibrary::new().with_base_url("https://www.goodreads.com");
    let error = drive(
        &provider,
        &isbn(FR),
        |_: &HttpRequest| -> Result<HttpResponse, String> { panic!("no request may be issued") },
        6,
    )
    .expect_err("refused");
    assert!(
        matches!(error, CatalogError::ForbiddenFollowUp { provider: "openlibrary", .. }),
        "{error:?}"
    );
}

#[test]
fn errors_are_fatal_only_without_a_record() {
    // First request fails: an error.
    let error =
        drive(&Scripted, &isbn(FR), scripted(&[("https://scripted.example/start", "fail")]), 5)
            .expect_err("fails");
    assert!(matches!(error, CatalogError::Remote { .. }));

    // A follow-up fails: the record is kept and the failure is noted.
    let outcome = drive(
        &Scripted,
        &isbn(FR),
        scripted(&[
            (
                "https://scripted.example/start",
                "title:T\nfollow:https://scripted.example/a\nfollow:https://scripted.example/gone",
            ),
            ("https://scripted.example/a", "fail"),
        ]),
        5,
    )
    .expect("partial result");
    assert_eq!(outcome.record.expect("a record").title, "T");
    assert_eq!(outcome.requests, 3);
    assert!(outcome.notes.iter().any(|n| n.contains("scripted failure")));
    assert!(
        outcome
            .notes
            .iter()
            .any(|n| n.contains("transport error") && n.contains("connection refused"))
    );

    // The transport fails on the first request.
    let error = drive(&Scripted, &isbn(FR), scripted(&[]), 5).expect_err("fails");
    assert!(
        matches!(&error, CatalogError::Transport { provider: "scripted", message } if message.contains("connection refused"))
    );
    assert!(error.is_transient());
}

#[test]
fn transport_errors_do_not_leak_api_keys() {
    let provider = ebc_catalog::providers::googlebooks::GoogleBooks::new(Some("SECRET".to_owned()));
    let error = drive(
        &provider,
        &isbn(FR),
        |r: &HttpRequest| -> Result<HttpResponse, String> {
            Err(format!("timeout while fetching {}", wire_url(r).split('?').next().unwrap_or("")))
        },
        3,
    )
    .expect_err("fails");
    assert!(!error.to_string().contains("SECRET"), "{error}");
    assert!(error.to_string().contains("key=…"), "{error}");
}

#[test]
fn a_session_can_be_stepped_by_hand() {
    let provider = OpenLibrary::new();
    let isbn = isbn(FR);
    let mut session = Session::new(&provider, &isbn, 6);
    let first = session.next_request().expect("first request");
    assert!(session.next_request().is_none(), "nothing else until the response is fed");
    session.feed(&first, &ok("openlibrary/etranger-author-keys.json"));
    // Both author requests can be taken before either response arrives (concurrent hosts).
    let a = session.next_request().expect("author 1");
    let b = session.next_request().expect("author 2");
    assert!(session.next_request().is_none());
    session.feed(&b, &ok("openlibrary/author-redirect.json"));
    session.feed(&a, &ok("openlibrary/author-OL124171A.json"));
    assert!(session.next_request().is_none(), "the redirect target was already requested");
    let outcome = session.finish().expect("found");
    assert_eq!(outcome.requests, 3);
    assert_eq!(outcome.record.expect("a record").author_line().as_deref(), Some("Albert Camus"));
}

fn cascade_routes() -> Routes {
    Routes(vec![
        ("catalogue.bnf.fr", ok("bnf/not-found.xml")),
        ("openlibrary.org/api/books", ok("openlibrary/etranger.json")),
        ("googleapis.com", ok("googlebooks/etranger.json")),
        ("services.dnb.de", response(503, "text/html", fixture("common/error.html"))),
        ("sru.k10plus.de", ok("k10plus/verwandlung.xml")),
        ("lobid.org", ok("lobid/verwandlung.json")),
    ])
}

#[test]
fn lookup_all_stops_once_the_record_is_complete() {
    let config = Config::default().with_google_books_key("SECRET");
    let providers = providers_for(&isbn(FR), &config);
    let routes = cascade_routes();
    let report = lookup_all(&providers, &isbn(FR), routes.exec(), &LookupOptions::default());
    let asked: Vec<&str> = report.providers.iter().map(|p| p.provider).collect();
    assert_eq!(asked, ["bnf", "openlibrary"]);
    assert_eq!(
        report.skipped,
        ["googlebooks"],
        "Google Books is asked only when the others leave gaps"
    );
    let merged = report.merged.as_ref().expect("merged");
    assert_eq!(merged.title, "L'étranger");
    assert_eq!(merged.sources.len(), 1);
    assert_eq!(report.records().map(|(id, _)| id).collect::<Vec<_>>(), ["openlibrary"]);
    assert_eq!(report.errors().count(), 0);

    let everything = LookupOptions { stop_when_complete: false, ..LookupOptions::default() };
    let report = lookup_all(&providers, &isbn(FR), routes.exec(), &everything);
    assert!(report.skipped.is_empty());
    let merged = report.merged.expect("merged");
    let sources: Vec<&str> = merged.sources.iter().map(|s| s.provider.as_str()).collect();
    assert_eq!(sources, ["openlibrary", "googlebooks"]);
    assert_eq!(merged.publisher.as_deref(), Some("Gallimard"));
}

#[test]
fn lookup_all_survives_failing_providers() {
    let providers = providers_for(&isbn(DE), &Config::default());
    let ids: Vec<&str> = providers.iter().map(|p| p.info().id).collect();
    assert_eq!(ids, ["dnb", "k10plus", "lobid", "openlibrary"]);
    let routes = cascade_routes();
    let report = lookup_all(&providers, &isbn(DE), routes.exec(), &LookupOptions::default());
    // DNB is down; K10plus answers; the record is complete, so lobid and Open Library rest.
    assert!(matches!(
        report.providers[0].result,
        Err(CatalogError::Unavailable { provider: "dnb", status: 503, .. })
    ));
    assert_eq!(report.errors().count(), 1);
    assert_eq!(report.skipped, ["lobid", "openlibrary"]);
    let merged = report.merged.expect("merged");
    assert_eq!(merged.title, "Die Verwandlung");
    assert_eq!(merged.author_line().as_deref(), Some("Franz Kafka"));

    // Everything fails: no record, every error reported, no panic.
    let offline = lookup_all(
        &providers,
        &isbn(DE),
        |_: &HttpRequest| -> Result<HttpResponse, String> { Err("network unreachable".to_owned()) },
        &LookupOptions::default(),
    );
    assert_eq!(offline.merged, None);
    assert_eq!(offline.errors().count(), 4);
    assert!(offline.errors().all(CatalogError::is_transient));

    let none = lookup_all(&[], &isbn(DE), cascade_routes().exec(), &LookupOptions::default());
    assert_eq!(none.merged, None);
}
