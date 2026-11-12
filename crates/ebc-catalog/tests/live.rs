//! Live tests against the real services. They are `#[ignore]`d **and** do nothing unless
//! `EBC_LIVE=1`, so that neither `cargo test` nor `cargo test -- --ignored` in CI sends a
//! request by accident:
//!
//! ```sh
//! EBC_LIVE=1 cargo test -p ebc-catalog --features client --test live -- --ignored --nocapture
//! ```
//!
//! They have never been run: the environment this crate was written in has no access to
//! these hosts. A failure here means a fixture (and the parser built on it) encodes a
//! wrong assumption about the service — see the crate README.

use ebc_catalog::client::{Client, ClientConfig};
use ebc_catalog::{Config, LookupOptions, SourceClass, provider_by_id, providers_for};
use ebc_model::Isbn13;

fn live() -> bool {
    let enabled = std::env::var("EBC_LIVE").as_deref() == Ok("1");
    if !enabled {
        eprintln!("skipped: set EBC_LIVE=1 to send real requests");
    }
    enabled
}

fn config() -> Config {
    let mut config = Config::default();
    if let Ok(key) = std::env::var("EBC_GOOGLE_BOOKS_KEY") {
        config = config.with_google_books_key(key);
    }
    if let Ok(contact) = std::env::var("EBC_CONTACT") {
        config = config.with_contact_email(contact);
    }
    config
}

/// One request (plus follow-ups) to one provider; the record must name the book.
async fn expect(id: &str, isbn: &str, title_word: &str, author_word: &str) {
    if !live() {
        return;
    }
    let client = Client::new(ClientConfig::default()).expect("client");
    let provider = provider_by_id(id, &config()).expect("known provider");
    let isbn = Isbn13::parse(isbn).expect("valid ISBN");
    let outcome = client.lookup(&provider, &isbn, 6).await.unwrap_or_else(|e| panic!("{id}: {e}"));
    eprintln!("{id}: {} request(s), notes {:?}", outcome.requests, outcome.notes);
    let book = outcome.record.unwrap_or_else(|| panic!("{id}: {isbn} not found"));
    eprintln!("{id}: {book:#?}");
    let fold = |s: &str| ebc_catalog::normalize::fold_key(s);
    assert!(fold(&book.title).contains(&fold(title_word)), "{id}: title {:?}", book.title);
    assert!(
        book.contributors.iter().any(|c| fold(&c.name).contains(&fold(author_word))),
        "{id}: contributors {:?}",
        book.contributors
    );
    assert_eq!(book.isbn13, Some(isbn));
    if provider.info().class == SourceClass::Open {
        assert!(book.sources[0].record_url.is_some(), "{id}: a link to the record");
    }
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_bnf() {
    expect("bnf", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_dnb() {
    expect("dnb", "9783257229530", "Vorleser", "Schlink").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_k10plus() {
    expect("k10plus", "9783257229530", "Vorleser", "Schlink").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_lobid() {
    expect("lobid", "9783257229530", "Vorleser", "Schlink").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_openlibrary() {
    expect("openlibrary", "9780141439518", "Pride", "Austen").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_libris() {
    expect("libris", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_finna() {
    expect("finna", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_sudoc() {
    expect("sudoc", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_loc() {
    expect("loc", "9780141439518", "Pride", "Austen").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_wikidata() {
    expect("wikidata", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_inventaire() {
    expect("inventaire", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access, EBC_LIVE=1 and EBC_GOOGLE_BOOKS_KEY"]
async fn live_googlebooks() {
    if std::env::var("EBC_GOOGLE_BOOKS_KEY").is_err() {
        eprintln!("skipped: set EBC_GOOGLE_BOOKS_KEY");
        return;
    }
    expect("googlebooks", "9782070360024", "étranger", "Camus").await;
}

#[tokio::test]
#[ignore = "live: needs network access and EBC_LIVE=1"]
async fn live_cascade() {
    if !live() {
        return;
    }
    let client = Client::new(ClientConfig::default()).expect("client");
    let isbn = Isbn13::parse("9782070360024").expect("valid ISBN");
    let providers = providers_for(&isbn, &config());
    let report = client.lookup_all(&providers, &isbn, &LookupOptions::default()).await;
    for provider in &report.providers {
        eprintln!("{}: {:?}", provider.provider, provider.result.as_ref().map(|o| o.requests));
    }
    let merged = report.merged.expect("found somewhere");
    eprintln!("{merged:#?}");
    assert_eq!(merged.author_line().as_deref(), Some("Albert Camus"));
}
