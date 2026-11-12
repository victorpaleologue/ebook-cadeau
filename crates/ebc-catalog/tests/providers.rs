//! Every provider against its fixtures: request building, found records, "not found",
//! rate limits, outages, wrong formats, answers about another ISBN, missing fields and
//! odd encodings.
//!
//! The fixtures are hand-written to each API's documented format — see
//! `tests/fixtures/README.md`. They are not recordings.

mod common;

use common::{DE, EN, FR, FR_979, Routes, SCHOLARLY, fixture, isbn, ok, response};
use ebc_catalog::providers::crossref::Crossref;
use ebc_catalog::providers::finna::Finna;
use ebc_catalog::providers::googlebooks::GoogleBooks;
use ebc_catalog::providers::inventaire::Inventaire;
use ebc_catalog::providers::libris::Libris;
use ebc_catalog::providers::lobid::Lobid;
use ebc_catalog::providers::openlibrary::OpenLibrary;
use ebc_catalog::providers::sru::SruProvider;
use ebc_catalog::providers::sudoc::Sudoc;
use ebc_catalog::providers::wikidata::Wikidata;
use ebc_catalog::{
    CatalogError, Config, PROVIDER_IDS, Parsed, Provider, drive, provider_by_id, wire_url,
};
use ebc_model::{BookFormat, BookMeta, ContributorRole, HttpRequest, HttpResponse};

/// Parses `response` as the answer to the provider's first request.
fn parse_first(provider: &dyn Provider, isbn_text: &str, response: &HttpResponse) -> Result<Parsed, CatalogError> {
    let isbn = isbn(isbn_text);
    let request = provider.lookup(&isbn).into_iter().next().expect("one request");
    provider.parse(&isbn, &request, response)
}

fn one_record(provider: &dyn Provider, isbn_text: &str, path: &str) -> BookMeta {
    let parsed = parse_first(provider, isbn_text, &ok(path)).expect("parses");
    assert_eq!(parsed.records.len(), 1, "{path}: {:?}", parsed.notes);
    let record = parsed.records.into_iter().next().expect("one record");
    assert_eq!(record.isbn13, Some(isbn(isbn_text)), "{path}: the queried ISBN is kept");
    assert_eq!(record.sources.len(), 1, "{path}");
    assert_eq!(record.sources[0].provider, provider.info().id);
    record
}

fn roles(book: &BookMeta) -> Vec<(&str, &ContributorRole)> {
    book.contributors.iter().map(|c| (c.name.as_str(), &c.role)).collect()
}

fn first_url(provider: &dyn Provider, isbn_text: &str) -> String {
    let requests = provider.lookup(&isbn(isbn_text));
    assert_eq!(requests.len(), 1);
    let url = requests[0].url.clone();
    assert!(
        !url.contains([' ', '"', '{', '}', '<', '>', '|']),
        "{url}: query parameters are percent-encoded"
    );
    url
}

// ---------------------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------------------

#[test]
fn request_urls_follow_the_research() {
    assert_eq!(
        first_url(&SruProvider::bnf(), FR),
        "https://catalogue.bnf.fr/api/SRU?version=1.2&operation=searchRetrieve&query=%28bib.isbn%20all%20%222070360024%22%20or%20bib.isbn%20all%20%229782070360024%22%29&recordSchema=unimarcXchange&maximumRecords=5"
    );
    // A 979 ISBN has no ISBN-10: a single operand.
    assert_eq!(
        first_url(&SruProvider::bnf().with_record_schema("dublincore"), FR_979),
        "https://catalogue.bnf.fr/api/SRU?version=1.2&operation=searchRetrieve&query=bib.isbn%20all%20%229791032102763%22&recordSchema=dublincore&maximumRecords=5"
    );
    assert_eq!(
        first_url(&SruProvider::dnb(), DE),
        "https://services.dnb.de/sru/dnb?version=1.1&operation=searchRetrieve&query=num%3D9783596294312&recordSchema=MARC21-xml&maximumRecords=5"
    );
    assert_eq!(
        first_url(&SruProvider::k10plus(), DE),
        "https://sru.k10plus.de/opac-de-627?version=1.1&operation=searchRetrieve&query=pica.isb%3D9783596294312&recordSchema=marcxml&maximumRecords=5"
    );
    assert_eq!(
        first_url(&SruProvider::loc(), EN),
        "https://lx2.loc.gov/sru/lcdb?version=1.1&operation=searchRetrieve&query=bath.isbn%3D9780141439518&recordSchema=marcxml&maximumRecords=5"
    );
    assert_eq!(
        first_url(&OpenLibrary::new(), FR),
        "https://openlibrary.org/api/books?bibkeys=ISBN%3A9782070360024&format=json&jscmd=details"
    );
    assert_eq!(
        first_url(&GoogleBooks::new(Some("SECRET".to_owned())), FR),
        "https://www.googleapis.com/books/v1/volumes?q=isbn%3A9782070360024&maxResults=5&key=SECRET"
    );
    assert_eq!(
        first_url(&GoogleBooks::new(None), FR),
        "https://www.googleapis.com/books/v1/volumes?q=isbn%3A9782070360024&maxResults=5"
    );
    assert_eq!(first_url(&Libris::new(), FR), "https://libris.kb.se/find.jsonld?q=9782070360024&_limit=5");
    assert!(first_url(&Finna::new(), FR).starts_with(
        "https://api.finna.fi/api/v1/search?lookfor=9782070360024&type=AllFields&limit=5&field%5B%5D=id&field%5B%5D=title"
    ));
    assert_eq!(
        first_url(&Lobid::new(), DE),
        "https://lobid.org/resources/search?q=isbn%3A9783596294312&format=json&size=5"
    );
    assert_eq!(first_url(&Sudoc::new(), FR), "https://www.sudoc.fr/services/isbn2ppn/9782070360024");
    assert_eq!(
        first_url(&Inventaire::new(), FR),
        "https://inventaire.io/api/entities?action=by-uris&uris=isbn%3A9782070360024"
    );
    assert_eq!(
        first_url(&Crossref::new(Some("contact@example.org".to_owned())), FR),
        "https://api.crossref.org/works?filter=isbn%3A9782070360024&rows=5&mailto=contact%40example.org"
    );
    assert_eq!(
        first_url(&Crossref::new(None), FR),
        "https://api.crossref.org/works?filter=isbn%3A9782070360024&rows=5"
    );
}

#[test]
fn wikidata_query_sends_every_isbn_form() {
    let url = first_url(&Wikidata::new(), FR);
    assert!(url.starts_with("https://query.wikidata.org/sparql?format=json&query=SELECT%20"));
    let sparql = Wikidata::sparql(&isbn(FR));
    for form in [
        "wdt:P212 \"978-2-07-036002-4\"",
        "wdt:P212 \"9782070360024\"",
        "wdt:P957 \"2-07-036002-4\"",
        "wdt:P957 \"2070360024\"",
    ] {
        assert!(sparql.contains(form), "{form}");
    }
    let without_isbn10 = Wikidata::sparql(&isbn(FR_979));
    assert!(without_isbn10.contains("9791032102763"));
    assert!(!without_isbn10.contains("{ ?item wdt:P957 \""), "979 ISBNs have no ISBN-10");
}

#[test]
fn base_urls_can_be_overridden_for_every_provider() {
    let mut config = Config::default().with_google_books_key("k");
    for id in PROVIDER_IDS {
        config = config.with_base_url(id, format!("http://127.0.0.1:8080/{id}"));
    }
    for id in PROVIDER_IDS {
        let provider = provider_by_id(id, &config).expect("known id");
        for request in provider.lookup(&isbn(FR)) {
            assert!(
                request.url.starts_with(&format!("http://127.0.0.1:8080/{id}")),
                "{id}: {}",
                request.url
            );
        }
    }
    // Follow-up requests stay on the overridden base.
    let provider = OpenLibrary::new().with_base_url("http://127.0.0.1:8080/ol/");
    let parsed = parse_first(&provider, FR, &ok("openlibrary/etranger-author-keys.json")).expect("parses");
    assert_eq!(
        parsed.follow_up.iter().map(wire_url).collect::<Vec<_>>(),
        ["http://127.0.0.1:8080/ol/authors/OL124171A.json", "http://127.0.0.1:8080/ol/authors/OL9999999A.json"]
    );
}

#[test]
fn requests_of_another_provider_are_refused() {
    let foreign = HttpRequest::get("https://example.org/api/books?bibkeys=ISBN:9782070360024");
    for id in PROVIDER_IDS {
        let provider = provider_by_id(id, &Config::default()).expect("known id");
        let result = provider.parse(&isbn(FR), &foreign, &ok("openlibrary/etranger.json"));
        assert!(matches!(result, Err(CatalogError::UnknownRequest { .. })), "{id}: {result:?}");
    }
}

// ---------------------------------------------------------------------------------------
// BnF
// ---------------------------------------------------------------------------------------

#[test]
fn bnf_unimarc_prefers_the_most_recent_record_and_cleans_it() {
    let book = one_record(&SruProvider::bnf(), FR, "bnf/etranger-unimarc.xml");
    assert_eq!(book.title, "L'étranger", "NSB/NSE marks and the medium designation are gone");
    assert_eq!(book.subtitle, None);
    assert_eq!(
        roles(&book),
        [
            ("Albert Camus", &ContributorRole::Author),
            ("Jacqueline Lévi-Valensi", &ContributorRole::Preface)
        ]
    );
    assert_eq!(book.publisher.as_deref(), Some("Gallimard"));
    assert_eq!(book.published.as_deref(), Some("2015"), "\"DL 2015\"");
    assert_eq!(book.language.as_deref(), Some("fr"));
    assert_eq!(book.page_count, Some(183), "\"1 vol. (183 p.)\"");
    assert_eq!(book.series.as_deref(), Some("Folio"));
    assert!(book.related_isbns.is_empty());
    let source = &book.sources[0];
    assert_eq!(source.record_url.as_deref(), Some("https://catalogue.bnf.fr/ark:/12148/cb44321021n"));
    assert_eq!(source.licence.as_deref(), Some("etalab-2.0"));
    assert!(source.attribution.as_deref().is_some_and(|a| a.contains("Bibliothèque nationale de France")));
}

#[test]
fn bnf_dublin_core_is_understood_too() {
    let provider = SruProvider::bnf().with_record_schema("dublincore");
    let book = one_record(&provider, FR, "bnf/etranger-dublincore.xml");
    assert_eq!(book.title, "L'étranger", "the statement of responsibility is cut");
    assert_eq!(roles(&book), [("Albert Camus", &ContributorRole::Author)]);
    assert_eq!(book.publisher.as_deref(), Some("Gallimard"), "\"Gallimard (Paris)\"");
    assert_eq!(book.published.as_deref(), Some("1971"));
    assert_eq!(book.language.as_deref(), Some("fr"));
    assert_eq!(book.page_count, Some(185));
    assert_eq!(book.series.as_deref(), Some("Folio"));
    assert_eq!(
        book.sources[0].record_url.as_deref(),
        Some("http://catalogue.bnf.fr/ark:/12148/cb35113948q")
    );
}

#[test]
fn bnf_latin1_and_minimal_records() {
    let latin1 = fixture("bnf/etranger-latin1.xml");
    assert!(std::str::from_utf8(&latin1).is_err(), "the fixture really is ISO-8859-1");
    let book = one_record(&SruProvider::bnf(), FR, "bnf/etranger-latin1.xml");
    assert_eq!(book.title, "L'étranger");
    assert_eq!(book.author_line().as_deref(), Some("Albert Camus"));
    assert_eq!(book.published.as_deref(), Some("1971"));
    assert_eq!(book.page_count, Some(185));
    assert_eq!(book.series.as_deref(), Some("Folio"), "\"Collection Folio\"");

    let minimal = one_record(&SruProvider::bnf(), FR, "bnf/minimal.xml");
    assert_eq!(minimal.title, "L'étranger");
    assert!(minimal.contributors.is_empty());
    assert_eq!((minimal.publisher, minimal.published, minimal.language, minimal.page_count), (None, None, None, None));
}

#[test]
fn bnf_not_found_other_isbn_and_diagnostics() {
    let bnf = SruProvider::bnf();
    let none = parse_first(&bnf, FR, &ok("bnf/not-found.xml")).expect("parses");
    assert!(none.is_empty() && none.notes.is_empty());

    let other = parse_first(&bnf, FR, &ok("bnf/other-isbn.xml")).expect("parses");
    assert!(other.records.is_empty());
    assert!(other.notes.iter().any(|n| n.contains("another ISBN") && n.contains("9782080700124")));

    let error = parse_first(&bnf, FR, &ok("bnf/diagnostic.xml")).expect_err("diagnostic");
    assert!(matches!(&error, CatalogError::Remote { provider: "bnf", message } if message.contains("Query syntax error")));
    assert!(!error.is_transient());
}

// ---------------------------------------------------------------------------------------
// MARC 21 catalogues
// ---------------------------------------------------------------------------------------

#[test]
fn dnb_marc21() {
    let book = one_record(&SruProvider::dnb(), DE, "dnb/verwandlung.xml");
    assert_eq!(book.title, "Die Verwandlung", "the non-sorting marks around the article are gone");
    assert_eq!(book.subtitle.as_deref(), Some("Erzählung"));
    assert_eq!(
        roles(&book),
        [("Franz Kafka", &ContributorRole::Author), ("Reiner Stach", &ContributorRole::Preface)],
        "the publisher's 710 entry is not a contributor"
    );
    assert_eq!(book.publisher.as_deref(), Some("S. Fischer Verlag"));
    assert_eq!(book.published.as_deref(), Some("2011"), "\"[2011]\" from the 264 _1 field");
    assert_eq!(book.language.as_deref(), Some("de"));
    assert_eq!(book.page_count, Some(96), "\"96 Seiten\"");
    assert_eq!(book.series.as_deref(), Some("Fischer Klassik"));
    assert_eq!(book.format, None);
    assert_eq!(book.sources[0].record_url.as_deref(), Some("https://d-nb.info/1012345678"));
    assert_eq!(book.sources[0].licence.as_deref(), Some("CC0-1.0"));

    let ebook = one_record(&SruProvider::dnb(), DE, "dnb/minimal-ebook.xml");
    assert_eq!(ebook.title, "Die Verwandlung");
    assert!(ebook.contributors.is_empty());
    assert_eq!(ebook.format, Some(BookFormat::Ebook));
    assert_eq!(ebook.page_count, Some(96), "\"1 Online-Ressource (96 Seiten)\"");
    assert_eq!(ebook.language.as_deref(), Some("de"), "from the 008 field");
    assert_eq!(ebook.published.as_deref(), Some("2019"));

    let dnb = SruProvider::dnb();
    assert!(parse_first(&dnb, DE, &ok("dnb/not-found.xml")).expect("parses").is_empty());
    assert!(parse_first(&dnb, DE, &ok("dnb/other-isbn.xml")).expect("parses").records.is_empty());
    assert!(matches!(
        parse_first(&dnb, DE, &ok("dnb/diagnostic.xml")),
        Err(CatalogError::Remote { message, .. }) if message.contains("Unsupported index")
    ));
}

#[test]
fn k10plus_marc21() {
    let book = one_record(&SruProvider::k10plus(), DE, "k10plus/verwandlung.xml");
    assert_eq!(book.title, "Die Verwandlung");
    assert_eq!(book.subtitle.as_deref(), Some("Erzählung"));
    assert_eq!(
        roles(&book),
        [("Franz Kafka", &ContributorRole::Author), ("Reiner Stach", &ContributorRole::Preface)]
    );
    assert_eq!(book.publisher.as_deref(), Some("Fischer"));
    assert_eq!(book.published.as_deref(), Some("2011"));
    assert_eq!(book.page_count, Some(96), "\"96 S.\"");
    assert_eq!(book.series.as_deref(), Some("Fischer Klassik"));
    assert_eq!(
        book.sources[0].record_url.as_deref(),
        Some("https://opac.k10plus.de/DB=2.299/PPNSET?PPN=66554433X")
    );
    assert!(parse_first(&SruProvider::k10plus(), DE, &ok("k10plus/not-found.xml")).expect("parses").is_empty());
}

#[test]
fn library_of_congress_marc21() {
    let book = one_record(&SruProvider::loc(), EN, "loc/pride.xml");
    assert_eq!(book.title, "Pride and prejudice", "the trailing \" /\" is gone");
    assert_eq!(book.author_line().as_deref(), Some("Jane Austen"), "\"Austen, Jane, 1775-1817.\"");
    // The 700 field has no relator: the role comes from the statement of responsibility.
    assert_eq!(roles(&book)[1], ("Vivien Jones", &ContributorRole::Preface));
    assert_eq!(book.contributors.len(), 2, "the 700 $t entry points at another work");
    assert_eq!(book.publisher.as_deref(), Some("Penguin Books"));
    assert_eq!(book.published.as_deref(), Some("2003"));
    assert_eq!(book.language.as_deref(), Some("en"), "from the 008 field");
    assert_eq!(book.page_count, Some(435), "\"xli, 435 p. ;\"");
    assert_eq!(book.series.as_deref(), Some("Penguin classics"));
    assert_eq!(book.sources[0].record_url.as_deref(), Some("https://lccn.loc.gov/2002512345"));
    assert_eq!(book.sources[0].licence, None);
}

// ---------------------------------------------------------------------------------------
// Open Library
// ---------------------------------------------------------------------------------------

#[test]
fn openlibrary_details_in_one_request() {
    let book = one_record(&OpenLibrary::new(), FR, "openlibrary/etranger.json");
    assert_eq!(book.title, "L'étranger", "\"L' étranger\"");
    assert_eq!(roles(&book), [("Albert Camus", &ContributorRole::Author)]);
    assert_eq!(book.publisher.as_deref(), Some("Gallimard"));
    assert_eq!(book.published.as_deref(), Some("1971"));
    assert_eq!(book.language.as_deref(), Some("fr"), "\"/languages/fre\"");
    assert_eq!(book.page_count, Some(185));
    assert_eq!(book.series.as_deref(), Some("Folio"), "\"Collection Folio ; 2\"");
    assert_eq!(book.format, Some(BookFormat::Paperback));
    assert_eq!(book.description.as_deref(), Some("Roman écrit en 1940, publié en 1942."), "HTML is stripped");
    assert_eq!(book.subjects.len(), 2);
    assert_eq!(book.covers.len(), 1, "the -1 placeholder is not a cover");
    assert_eq!(book.covers[0].url, "https://covers.openlibrary.org/b/id/8231856-L.jpg");
    assert_eq!(book.covers[0].source, "openlibrary");
    assert_eq!(
        book.sources[0].record_url.as_deref(),
        Some("https://openlibrary.org/books/OL8838946M/L'étranger")
    );

    let pride = one_record(&OpenLibrary::new(), EN, "openlibrary/pride.json");
    assert_eq!(pride.title, "Pride and Prejudice");
    assert_eq!(
        roles(&pride),
        [
            ("Jane Austen", &ContributorRole::Author),
            ("Vivien Jones", &ContributorRole::Editor),
            ("Tony Tanner", &ContributorRole::Preface)
        ]
    );
    assert_eq!(pride.published.as_deref(), Some("2002-12-31"), "\"December 31, 2002\"");
    assert_eq!(pride.language.as_deref(), Some("en"));
}

#[test]
fn openlibrary_follows_up_for_author_names() {
    let routes = Routes::of(&[
        ("/api/books", "openlibrary/etranger-author-keys.json"),
        ("/authors/OL124171A.json", "openlibrary/author-OL124171A.json"),
        ("/authors/OL9999999A.json", "openlibrary/author-redirect.json"),
    ]);
    let outcome = drive(&OpenLibrary::new(), &isbn(FR), routes.exec(), 6).expect("found");
    let book = outcome.record.expect("a record");
    assert_eq!(outcome.requests, 3, "the redirect points at an author already asked for");
    assert_eq!(book.title, "L'étranger", "title_prefix + title");
    assert_eq!(
        roles(&book),
        [("Albert Camus", &ContributorRole::Author), ("Jean-Paul Sartre", &ContributorRole::Preface)],
        "authors come first although their names arrived last"
    );
    assert_eq!(book.sources.len(), 1);
}

#[test]
fn openlibrary_asks_the_work_when_the_edition_has_no_author() {
    let routes = Routes::of(&[
        ("/api/books", "openlibrary/etranger-work-only.json"),
        ("/works/OL1168007W.json", "openlibrary/work-OL1168007W.json"),
        ("/authors/OL124171A.json", "openlibrary/author-OL124171A.json"),
    ]);
    let outcome = drive(&OpenLibrary::new(), &isbn(FR), routes.exec(), 6).expect("found");
    assert_eq!(outcome.requests, 3);
    let book = outcome.record.expect("a record");
    assert_eq!(book.author_line().as_deref(), Some("Albert Camus"));
    assert_eq!(book.published.as_deref(), Some("1972-01-05"), "\"Jan 05, 1972\"");
    assert_eq!(book.page_count, Some(185), "from `pagination`");
    assert!(book.description.as_deref().is_some_and(|d| d.contains("Meursault")));
    assert!(book.subjects.contains(&"French fiction".to_owned()));
}

#[test]
fn openlibrary_not_found_and_wrong_answers() {
    let provider = OpenLibrary::new();
    assert!(parse_first(&provider, FR, &ok("openlibrary/not-found.json")).expect("parses").is_empty());
    let other = parse_first(&provider, FR, &ok("openlibrary/other-isbn.json")).expect("parses");
    assert!(other.records.is_empty() && other.notes.len() == 1);
    let wrong = parse_first(&provider, FR, &ok("openlibrary/wrong-edition.json")).expect("parses");
    assert!(wrong.records.is_empty() && wrong.notes.len() == 1);
    // A linked author that no longer exists is not an error.
    let request = HttpRequest::get("https://openlibrary.org/authors/OL124171A.json");
    let gone = provider.parse(&isbn(FR), &request, &response(404, "text/html", b"<html>Not found</html>".to_vec()));
    assert!(gone.expect("tolerated").records.is_empty());
}

// ---------------------------------------------------------------------------------------
// Google Books
// ---------------------------------------------------------------------------------------

#[test]
fn googlebooks_volumes() {
    let provider = GoogleBooks::new(Some("SECRET".to_owned()));
    let parsed = parse_first(&provider, FR, &ok("googlebooks/etranger.json")).expect("parses");
    assert_eq!(parsed.notes.len(), 1, "the volume without the ISBN is skipped: {:?}", parsed.notes);
    let book = &parsed.records[0];
    assert_eq!(book.title, "L'étranger");
    assert_eq!(roles(book), [("Albert Camus", &ContributorRole::Author)]);
    assert_eq!(book.publisher.as_deref(), Some("Editions Gallimard"));
    assert_eq!(book.published.as_deref(), Some("1971-12"));
    assert_eq!(book.page_count, Some(191));
    assert_eq!(book.language.as_deref(), Some("fr"));
    assert_eq!(
        book.description.as_deref(),
        Some("« Aujourd'hui, maman est morte. »\nLe premier roman d'Albert Camus.")
    );
    assert_eq!(book.subjects, ["Fiction"]);
    assert_eq!(book.covers.len(), 2);
    assert!(book.covers[0].url.starts_with("https://books.google.com/books/content?id=sfUsAAAAYAAJ"));
    assert!(book.covers[0].url.contains("zoom=1"), "the larger image comes first");
    let source = &book.sources[0];
    assert_eq!(source.attribution.as_deref(), Some("Powered by Google"));
    assert_eq!(source.licence, None);
    assert!(source.record_url.as_deref().is_some_and(|u| u.starts_with("https://books.google.com/")));

    let ebook = one_record(&provider, DE, "googlebooks/verwandlung-ebook.json");
    assert_eq!(ebook.format, Some(BookFormat::Ebook));
    assert_eq!(ebook.subtitle.as_deref(), Some("Erzählung"));
    assert_eq!(ebook.published.as_deref(), Some("2011-05-12"));

    assert!(parse_first(&provider, FR, &ok("googlebooks/not-found.json")).expect("parses").is_empty());
    // The German fixture answers a French query: nothing is kept.
    assert!(parse_first(&provider, FR, &ok("googlebooks/verwandlung-ebook.json")).expect("parses").records.is_empty());
}

#[test]
fn googlebooks_error_envelopes() {
    let provider = GoogleBooks::new(Some("SECRET".to_owned()));
    let quota = response(403, "application/json; charset=UTF-8", fixture("googlebooks/quota.json"));
    let error = parse_first(&provider, FR, &quota).expect_err("quota");
    assert!(matches!(error, CatalogError::RateLimited { provider: "googlebooks", status: 403, .. }));
    assert!(error.is_transient());

    let key = response(400, "application/json; charset=UTF-8", fixture("googlebooks/key-invalid.json"));
    let error = parse_first(&provider, FR, &key).expect_err("bad key");
    assert!(matches!(&error, CatalogError::Unauthorized { message, .. } if message.contains("API key not valid")));
    assert!(!error.to_string().contains("SECRET"), "the key never appears in an error");

    // parse() refuses a request it did not issue, without leaking the key either.
    let foreign = HttpRequest::get("https://www.googleapis.com/other?key=SECRET");
    let error = provider.parse(&isbn(FR), &foreign, &ok("googlebooks/not-found.json")).expect_err("unknown");
    assert!(!error.to_string().contains("SECRET"), "{error}");
}

// ---------------------------------------------------------------------------------------
// Wikidata, Inventaire
// ---------------------------------------------------------------------------------------

#[test]
fn wikidata_sparql_results() {
    let book = one_record(&Wikidata::new(), FR, "wikidata/etranger.json");
    assert_eq!(book.title, "L'Étranger");
    assert_eq!(
        roles(&book),
        [("Albert Camus", &ContributorRole::Author)],
        "the work's author is used; the translator without a label (a bare Q-id) is dropped"
    );
    assert_eq!(book.publisher.as_deref(), Some("Éditions Gallimard"));
    assert_eq!(book.published.as_deref(), Some("1971"), "a year-precision date, not 1 January");
    assert_eq!(book.language.as_deref(), Some("fr"));
    assert_eq!(book.page_count, Some(185));
    assert_eq!(book.sources[0].record_url.as_deref(), Some("https://www.wikidata.org/wiki/Q112233445"));

    let parsed = parse_first(&Wikidata::new(), EN, &ok("wikidata/pride.json")).expect("parses");
    assert!(parsed.notes.iter().any(|n| n.contains("several")));
    assert_eq!(parsed.records[0].title, "Pride and Prejudice");
    assert_eq!(parsed.records[0].published.as_deref(), Some("2003-01-30"));
    assert_eq!(parsed.records[0].author_line().as_deref(), Some("Jane Austen"));

    assert!(parse_first(&Wikidata::new(), FR, &ok("wikidata/not-found.json")).expect("parses").is_empty());
    assert!(parse_first(&Wikidata::new(), FR, &ok("wikidata/other-isbn.json")).expect("parses").records.is_empty());
}

#[test]
fn inventaire_three_steps() {
    let routes = Routes::of(&[
        ("uris=isbn%3A9782070360024", "inventaire/edition.json"),
        ("uris=wd%3AQ11142%7Cwd%3AQ1492608%7Cinv%3A00112233445566778899aabbccddeeff", "inventaire/refs.json"),
        ("uris=wd%3AQ34670", "inventaire/authors.json"),
    ]);
    let outcome = drive(&Inventaire::new(), &isbn(FR), routes.exec(), 6).expect("found");
    assert_eq!(outcome.requests, 3, "{:?}", outcome.notes);
    let book = outcome.record.expect("a record");
    assert_eq!(book.title, "L'Étranger");
    assert_eq!(roles(&book), [("Albert Camus", &ContributorRole::Author)]);
    assert_eq!(book.publisher.as_deref(), Some("Éditions Gallimard"));
    assert_eq!(book.series.as_deref(), Some("Folio"));
    assert_eq!(book.published.as_deref(), Some("1971"));
    assert_eq!(book.language.as_deref(), Some("fr"), "wd:Q150");
    assert_eq!(book.page_count, Some(185));
    assert_eq!(book.covers[0].url, "https://inventaire.io/img/entities/3f2a9c1de0b54a7f8c6d1e2f3a4b5c6d7e8f9a0b");
    assert_eq!(book.sources[0].record_url.as_deref(), Some("https://inventaire.io/entity/isbn:9782070360024"));

    assert!(parse_first(&Inventaire::new(), FR, &ok("inventaire/not-found.json")).expect("parses").is_empty());
}

// ---------------------------------------------------------------------------------------
// Libris, Finna, lobid, SUDOC, Crossref
// ---------------------------------------------------------------------------------------

#[test]
fn libris_json_ld() {
    let parsed = parse_first(&Libris::new(), FR, &ok("libris/etranger.jsonld")).expect("parses");
    assert!(parsed.notes.iter().any(|n| n.contains("another ISBN")), "{:?}", parsed.notes);
    let book = &parsed.records[0];
    assert_eq!(book.title, "L'étranger");
    assert_eq!(roles(book), [("Albert Camus", &ContributorRole::Author)]);
    assert_eq!(book.publisher.as_deref(), Some("Gallimard"));
    assert_eq!(book.published.as_deref(), Some("1972"));
    assert_eq!(book.language.as_deref(), Some("fr"));
    assert_eq!(book.page_count, Some(185), "\"185, [2] s.\"");
    assert_eq!(book.series.as_deref(), Some("Folio"));
    assert_eq!(book.sources[0].record_url.as_deref(), Some("https://libris.kb.se/q7xz5dkc0t2wvnh"));
    assert!(parse_first(&Libris::new(), FR, &ok("libris/not-found.jsonld")).expect("parses").is_empty());
}

#[test]
fn finna_search() {
    let book = one_record(&Finna::new(), FR, "finna/etranger.json");
    assert_eq!(book.title, "L'étranger");
    assert_eq!(
        roles(&book),
        [("Albert Camus", &ContributorRole::Author), ("Kalle Kiviniemi", &ContributorRole::Translator)]
    );
    assert_eq!(book.publisher.as_deref(), Some("Gallimard"));
    assert_eq!(book.published.as_deref(), Some("1971"));
    assert_eq!(book.language.as_deref(), Some("fr"));
    assert_eq!(book.page_count, Some(185), "\"185 sivua\"");
    assert_eq!(book.series.as_deref(), Some("Folio"));
    assert_eq!(
        book.sources[0].record_url.as_deref(),
        Some("https://www.finna.fi/Record/helka.9916548763506253")
    );
    assert!(parse_first(&Finna::new(), FR, &ok("finna/not-found.json")).expect("parses").is_empty());
    assert!(matches!(
        parse_first(&Finna::new(), FR, &ok("finna/error.json")),
        Err(CatalogError::Remote { message, .. }) if message == "Invalid search type"
    ));
}

#[test]
fn lobid_search() {
    let book = one_record(&Lobid::new(), DE, "lobid/verwandlung.json");
    assert_eq!(book.title, "Die Verwandlung");
    assert_eq!(book.subtitle.as_deref(), Some("Erzählung"));
    assert_eq!(
        roles(&book),
        [("Franz Kafka", &ContributorRole::Author), ("Reiner Stach", &ContributorRole::Preface)],
        "the publisher's contribution is skipped"
    );
    assert_eq!(book.publisher.as_deref(), Some("Fischer"));
    assert_eq!(book.published.as_deref(), Some("2011"));
    assert_eq!(book.language.as_deref(), Some("de"));
    assert_eq!(book.page_count, Some(96));
    assert_eq!(book.series.as_deref(), Some("Fischer Klassik"));
    assert_eq!(
        book.sources[0].record_url.as_deref(),
        Some("http://lobid.org/resources/990123456780206441")
    );
    assert!(parse_first(&Lobid::new(), DE, &ok("lobid/not-found.json")).expect("parses").is_empty());
}

#[test]
fn sudoc_two_steps() {
    let routes = Routes::of(&[
        ("/services/isbn2ppn/9782070360024", "sudoc/isbn2ppn.json"),
        ("/000136085.xml", "sudoc/record-000136085.xml"),
        ("/12345678X.xml", "sudoc/record-12345678X.xml"),
    ]);
    let outcome = drive(&Sudoc::new(), &isbn(FR), routes.exec(), 6).expect("found");
    assert_eq!(outcome.requests, 3, "at most two records are fetched: {:?}", outcome.notes);
    let book = outcome.record.expect("a record");
    assert_eq!(book.title, "L'étranger", "the PICA \"@\" filing mark is gone");
    assert_eq!(
        roles(&book),
        [("Albert Camus", &ContributorRole::Author), ("Marie Dupont", &ContributorRole::Illustrator)]
    );
    assert_eq!(book.publisher.as_deref(), Some("Gallimard"));
    assert_eq!(book.published.as_deref(), Some("1984"), "\"impr. 1984, cop. 1942\": printing beats copyright");
    assert_eq!(book.page_count, Some(185));
    assert_eq!(book.series.as_deref(), Some("Folio"));
    let urls: Vec<_> = book.sources.iter().filter_map(|s| s.record_url.as_deref()).collect();
    assert_eq!(urls, ["https://www.sudoc.fr/000136085", "https://www.sudoc.fr/12345678X"]);

    // The resolver's other answer shapes.
    let sudoc = Sudoc::new();
    for path in ["sudoc/isbn2ppn-single.json", "sudoc/isbn2ppn.xml"] {
        let parsed = parse_first(&sudoc, FR, &ok(path)).expect("parses");
        let urls: Vec<_> = parsed.follow_up.iter().map(wire_url).collect();
        assert_eq!(urls, ["https://www.sudoc.fr/000136085.xml"], "{path}");
    }
    let none = parse_first(&sudoc, FR, &ok("sudoc/isbn2ppn-none.json")).expect("parses");
    assert!(none.is_empty() && none.notes[0].contains("Aucun PPN"));
    let missing = response(404, "text/html", b"<html><body>Not Found</body></html>".to_vec());
    assert!(parse_first(&sudoc, FR, &missing).expect("404 is not found").is_empty());
}

#[test]
fn crossref_works() {
    let parsed = parse_first(&Crossref::new(None), SCHOLARLY, &ok("crossref/book.json")).expect("parses");
    assert!(parsed.notes.iter().any(|n| n.contains("not whole books")), "the chapter is skipped");
    let book = &parsed.records[0];
    assert_eq!(book.title, "Reading the Absurd");
    assert_eq!(book.subtitle.as_deref(), Some("Camus and His Critics"));
    assert_eq!(
        roles(book),
        [("Ada Example", &ContributorRole::Author), ("Brian Sample", &ContributorRole::Editor)]
    );
    assert_eq!(book.publisher.as_deref(), Some("Example University Press"));
    assert_eq!(book.published.as_deref(), Some("2019-05-02"));
    assert_eq!(book.language.as_deref(), Some("en"));
    assert_eq!(book.related_isbns, [isbn(EN)], "the other ISBN of the record is kept as related");
    assert_eq!(book.sources[0].record_url.as_deref(), Some("https://doi.org/10.5555/9780061120084"));

    assert!(parse_first(&Crossref::new(None), SCHOLARLY, &ok("crossref/not-found.json")).expect("parses").is_empty());
    assert!(matches!(
        parse_first(&Crossref::new(None), SCHOLARLY, &ok("crossref/failed.json")),
        Err(CatalogError::Remote { message, .. }) if message.contains("no such filter")
    ));
}

// ---------------------------------------------------------------------------------------
// Failures common to every provider
// ---------------------------------------------------------------------------------------

#[test]
fn every_provider_reports_rate_limits_outages_and_wrong_formats() {
    let config = Config::default().with_google_books_key("SECRET");
    for id in PROVIDER_IDS {
        let provider = provider_by_id(id, &config).expect("known id");
        let provider = provider.as_ref();

        let mut limited = response(429, "application/json", fixture("common/too-many-requests.json"));
        limited.headers.push(("Retry-After".to_owned(), "30".to_owned()));
        let error = parse_first(provider, FR, &limited).expect_err("429");
        assert!(matches!(error, CatalogError::RateLimited { status: 429, retry_after_s: Some(30), .. }), "{id}: {error:?}");
        assert_eq!(error.provider(), id);
        assert!(error.is_transient());
        assert_eq!(error.retry_after_s(), Some(30));

        for status in [502, 503] {
            let down = response(status, "text/html", fixture("common/error.html"));
            let error = parse_first(provider, FR, &down).expect_err("5xx");
            assert!(matches!(error, CatalogError::Unavailable { .. }), "{id}: {error:?}");
            assert!(error.is_transient());
        }

        // A captive portal or CDN error page served with status 200.
        let html = response(200, "text/html; charset=utf-8", fixture("common/error.html"));
        let error = parse_first(provider, FR, &html).expect_err("HTML");
        assert!(
            matches!(error, CatalogError::UnexpectedContent { found: "HTML", .. }),
            "{id}: {error:?}"
        );

        let empty = response(200, "application/octet-stream", Vec::new());
        let error = parse_first(provider, FR, &empty).expect_err("empty");
        assert!(matches!(error, CatalogError::UnexpectedContent { found: "an empty body", .. }), "{id}: {error:?}");

        let forbidden = response(403, "text/plain", b"Forbidden".to_vec());
        let error = parse_first(provider, FR, &forbidden).expect_err("403");
        assert!(matches!(error, CatalogError::Unauthorized { status: 403, .. }), "{id}: {error:?}");

        let teapot = response(418, "text/plain", b"I'm a teapot".to_vec());
        assert!(matches!(parse_first(provider, FR, &teapot), Err(CatalogError::Status { status: 418, .. })), "{id}");

        let huge = response(200, "application/json", vec![b' '; 2 * 1024 * 1024 + 1]);
        assert!(matches!(parse_first(provider, FR, &huge), Err(CatalogError::TooLarge { .. })), "{id}");

        // Well-formed but not the documented structure.
        for (content_type, body) in [("application/json", &b"[1, 2, 3]"[..]), ("application/xml", &b"<unrelated/>"[..])] {
            let odd = response(200, content_type, body.to_vec());
            assert!(parse_first(provider, FR, &odd).is_err(), "{id}: {content_type} must not be read as \"not found\"");
        }
    }
}
