//! The candidate cascade and the policy around it.

mod common;

use common::fixture;
use ebc_covers::cascade::{
    DEFAULT_USER_AGENT, OpenLibrarySize, bnf_ark_in, bnf_cover_url, google_books_image_url,
    openlibrary_cover_id, openlibrary_id_image_url, openlibrary_isbn_image_url,
};
use ebc_covers::robots::Robots;
use ebc_covers::scrape::extract;
use ebc_covers::{
    CandidateRole, CoverCandidate, CoverKind, CoversError, Mode, Policy, Refusal, RightsBasis,
    Risk, SOURCES, SourceStatus, Use, candidates, check_url, page_candidate, page_image_candidates,
    source,
};
use ebc_model::{BookMeta, CoverRef, Method, SourceRef};

fn book(isbn: &str) -> BookMeta {
    let mut b = BookMeta::titled("Le Phare des brumes");
    b.isbn13 = Some(isbn.parse().expect("isbn"));
    b.publisher = Some("Éditions du Phare Imaginaire".to_owned());
    b
}

fn cover_ref(url: &str, source: &str) -> CoverRef {
    CoverRef {
        url: url.to_owned(),
        kind: CoverKind::Front,
        width_px: None,
        height_px: None,
        source: source.to_owned(),
    }
}

fn shape(list: &[CoverCandidate]) -> Vec<(&'static str, CandidateRole, &str)> {
    list.iter().map(|c| (c.source, c.role, c.request.url.as_str())).collect()
}

const FR: &str = "9782999900011";
const FR_PRINT: &str = "9782999900028";
const DE: &str = "9783999900018";
const FR_979: &str = "9791099990013";
const EN: &str = "9781999900014";

#[test]
fn default_cascade_is_open_library_only() {
    let list = candidates(&book(FR), &Policy::default());
    assert_eq!(
        shape(&list),
        [
            (
                "openlibrary",
                CandidateRole::Locate,
                "https://covers.openlibrary.org/b/isbn/9782999900011.json"
            ),
            (
                "openlibrary",
                CandidateRole::Preview,
                "https://covers.openlibrary.org/b/isbn/9782999900011-M.jpg?default=false"
            ),
            (
                "openlibrary",
                CandidateRole::Link,
                "https://covers.openlibrary.org/b/isbn/9782999900011.jpg?default=false"
            ),
        ]
    );
    for c in &list {
        assert_eq!(c.rights_basis(), RightsBasis::None);
        assert_eq!(c.risk, Risk::Medium);
        assert_eq!(c.kind, CoverKind::Front);
        assert_eq!(c.min_interval_ms, 3000, "ISBN-keyed: one request every 3 s");
        assert!(!c.other_edition);
        assert_eq!(c.isbn.map(|i| i.to_string()).as_deref(), Some(FR));
        assert_eq!(c.request.method, Method::Get);
        assert!(c.request.body.is_none());
        let ua = c.request.headers.iter().find(|(n, _)| n == "User-Agent").map(|(_, v)| v.as_str());
        assert_eq!(ua, Some(DEFAULT_USER_AGENT));
        assert!(!c.note.is_empty());
        assert_eq!(c.source_info().map(|s| s.id), Some("openlibrary"));
    }
    // A book without ISBN and without references has nothing to look up.
    assert!(candidates(&BookMeta::titled("Sans ISBN"), &Policy::default()).is_empty());
}

#[test]
fn cascade_order_with_every_source_enabled() {
    let policy = Policy { google_books_api_key: Some("k e&y=1".to_owned()), ..Policy::default() }
        .with_source("bnf")
        .with_source("dnb");
    // French ISBN: Open Library, Google Books, BnF (no DNB).
    let list = candidates(&book(FR), &policy);
    let sources: Vec<&str> = list.iter().map(|c| c.source).collect();
    assert_eq!(sources, ["openlibrary", "openlibrary", "openlibrary", "googlebooks", "bnf"]);
    let google = &list[3];
    assert_eq!(google.role, CandidateRole::Locate);
    assert_eq!(
        google.request.url,
        "https://www.googleapis.com/books/v1/volumes?q=isbn%3A9782999900011&maxResults=1&key=k+e%26y%3D1"
    );
    assert_eq!(google.min_interval_ms, 1000);
    let bnf = &list[4];
    assert_eq!(bnf.role, CandidateRole::Locate);
    assert_eq!(
        bnf.request.url,
        "https://catalogue.bnf.fr/api/SRU?version=1.2&operation=searchRetrieve&query=bib.isbn%20all%20%229782999900011%22&recordSchema=dublincore&maximumRecords=1"
    );
    assert_eq!(bnf.min_interval_ms, 2000);

    // 979-10 is French too; 978-3 goes to the DNB with a HEAD probe; 978-1 to neither.
    let sources = |isbn: &str| {
        candidates(&book(isbn), &policy).iter().map(|c| (c.source, c.role)).collect::<Vec<_>>()
    };
    assert_eq!(sources(FR_979)[4..], [("bnf", CandidateRole::Locate)]);
    assert_eq!(sources(DE)[4..], [("dnb", CandidateRole::Probe), ("dnb", CandidateRole::Link)]);
    assert_eq!(sources(EN).len(), 4);
    let dnb = &candidates(&book(DE), &policy)[4];
    assert_eq!(
        (dnb.request.method, dnb.request.url.as_str()),
        (Method::Head, "https://portal.dnb.de/opac/mvb/cover?isbn=9783999900018")
    );

    // With the ARK already known from the catalogue lookup: probe and link, no SRU.
    let mut known = book(FR);
    known.sources.push(SourceRef {
        provider: "bnf".to_owned(),
        record_url: Some("https://catalogue.bnf.fr/ark:/12148/cb34567890z".to_owned()),
        licence: None,
        attribution: None,
    });
    let list = candidates(&known, &policy);
    let cover =
        "https://catalogue.bnf.fr/couverture?appName=NE&idArk=ark:/12148/cb34567890z&couverture=1";
    assert_eq!(
        shape(&list)[4..],
        [("bnf", CandidateRole::Probe, cover), ("bnf", CandidateRole::Link, cover)]
    );
    assert_eq!(list[4].request.method, Method::Head);

    // Everything stays at Medium and nothing is a download.
    for isbn in [FR, FR_979, DE, EN] {
        for c in candidates(&book(isbn), &policy) {
            assert_eq!(c.risk, Risk::Medium);
            assert_ne!(c.role, CandidateRole::Download);
            assert!(check_url(&c.request.url).is_ok(), "{}", c.request.url);
        }
    }
}

#[test]
fn sources_off_by_default_stay_off() {
    // No key: no Google request, even when a blank key is configured.
    for key in [None, Some(String::new()), Some("   ".to_owned())] {
        let policy = Policy { google_books_api_key: key, ..Policy::default() };
        assert!(candidates(&book(FR), &policy).iter().all(|c| c.source != "googlebooks"));
    }
    // BnF and DNB need an explicit decision.
    assert!(!Policy::default().allows_source("bnf"));
    assert!(!Policy::default().allows_source("dnb"));
    assert!(Policy::default().with_source("bnf").allows_source("bnf"));
    assert!(candidates(&book(DE), &Policy::default()).iter().all(|c| c.source == "openlibrary"));
    // Removing a source from the list removes its candidates.
    let policy = Policy { allowed_sources: vec!["googlebooks".to_owned()], ..Policy::default() };
    assert!(candidates(&book(FR), &policy).is_empty());
}

#[test]
fn max_risk_filters_and_never_sources_cannot_be_enabled() {
    // Every lookup is Medium: a Low ceiling leaves nothing in standalone mode.
    let low = Policy { max_risk: Risk::Low, ..Policy::default() };
    assert!(candidates(&book(FR), &low).is_empty());
    assert!(matches!(
        page_candidate("https://www.editeur.example/livre", &low),
        Err(CoversError::UrlRefused(Refusal::LookupsDisabled))
    ));

    // Listing a ruled-out source and raising the ceiling changes nothing.
    let mut reckless = Policy::operator(RightsBasis::RightsHolderDeclared);
    for info in SOURCES {
        reckless = reckless.with_source(info.id);
    }
    for info in SOURCES {
        let usable =
            matches!(info.status, SourceStatus::Enabled | SourceStatus::DisabledUntilTermsRead);
        assert_eq!(reckless.allows_source(info.id), usable, "{}", info.id);
    }
    for id in [
        "amazon",
        "retailers",
        "library_subscriptions",
        "trade_databases",
        "librarything",
        "nonexistent",
    ] {
        assert!(!reckless.allows_source(id), "{id}");
    }
    let list = candidates(&book(FR), &reckless);
    assert!(list.iter().all(|c| ["openlibrary", "bnf"].contains(&c.source)), "{:?}", shape(&list));
}

#[test]
fn downloads_are_an_explicit_operator_opt_in() {
    let only_downloads = |policy: &Policy| -> Vec<String> {
        candidates(&book(FR), policy)
            .into_iter()
            .filter(|c| c.role == CandidateRole::Download)
            .map(|c| format!("{} {} {}", c.source, c.risk, c.request.url))
            .collect()
    };
    // The hosted defaults: none.
    assert!(only_downloads(&Policy::default()).is_empty());
    assert!(only_downloads(&Policy::hosted()).is_empty());
    // A rights basis without the raised ceiling, or the ceiling without a basis: none.
    let basis_only =
        Policy { operator_rights_basis: Some(RightsBasis::PrintLicence), ..Policy::default() };
    assert!(only_downloads(&basis_only).is_empty());
    let ceiling_only = Policy { max_risk: Risk::High, ..Policy::default() };
    assert!(only_downloads(&ceiling_only).is_empty());
    assert!(only_downloads(&Policy::operator(RightsBasis::None)).is_empty());
    // Both: the Open Library original, rated High; never Google, BnF or DNB.
    let operator = Policy {
        google_books_api_key: Some("key".to_owned()),
        ..Policy::operator(RightsBasis::RightsHolderDeclared)
    }
    .with_source("bnf")
    .with_source("dnb");
    assert!(operator.downloads_enabled());
    assert_eq!(
        only_downloads(&operator),
        ["openlibrary high https://covers.openlibrary.org/b/isbn/9782999900011.jpg?default=false"]
    );
    let german: Vec<_> = candidates(&book(DE), &operator)
        .into_iter()
        .filter(|c| c.role == CandidateRole::Download)
        .collect();
    assert!(german.iter().all(|c| c.source == "openlibrary"));
    // Even then the candidate itself carries no right: the tool builds the printable
    // asset from the operator's basis.
    assert!(candidates(&book(FR), &operator).iter().all(|c| c.rights_basis() == RightsBasis::None));
}

#[test]
fn related_isbn_is_retried_once_and_flagged() {
    let mut ebook = book(FR);
    ebook.related_isbns = vec![
        FR.parse().expect("isbn"),
        FR_PRINT.parse().expect("isbn"),
        FR_979.parse().expect("isbn"),
    ];
    let list = candidates(&ebook, &Policy::default());
    let per_isbn: Vec<(String, bool)> =
        list.iter().map(|c| (c.isbn.expect("isbn").to_string(), c.other_edition)).collect();
    assert_eq!(
        per_isbn[..3],
        [(FR.to_owned(), false), (FR.to_owned(), false), (FR.to_owned(), false)]
    );
    assert_eq!(
        per_isbn[3..],
        [(FR_PRINT.to_owned(), true), (FR_PRINT.to_owned(), true), (FR_PRINT.to_owned(), true)]
    );
    let none = Policy { max_related_isbns: 0, ..Policy::default() };
    assert_eq!(candidates(&ebook, &none).len(), 3);
    let all = Policy { max_related_isbns: 9, ..Policy::default() };
    assert_eq!(candidates(&ebook, &all).len(), 9);
}

#[test]
fn references_in_the_record_are_routed_to_their_source() {
    let mut b = book(FR);
    b.covers = vec![
        // Open Library by id, as a catalogue lookup returns it — plain http, -L size.
        CoverRef {
            width_px: Some(325),
            height_px: Some(500),
            ..cover_ref("http://covers.openlibrary.org/b/id/8739161-L.jpg", "openlibrary")
        },
        // Google thumbnail with the page curl, plain http.
        cover_ref(
            "http://books.google.com/books/content?id=AbC_dEf&printsec=frontcover&img=1&zoom=1&edge=curl&source=gbs_api",
            "googlebooks",
        ),
        // A publisher's own image.
        cover_ref("https://www.phare-imaginaire.example/media/couv.jpg", "onix"),
        // Hosts that are never contacted or linked, whatever the record says.
        cover_ref(
            "https://images-na.ssl-images-amazon.com/images/P/2999900015.01.LZZZZZZZ.jpg",
            "somewhere",
        ),
        cover_ref("https://m.media-amazon.com/images/I/01RmK+J4pJL.jpg", "somewhere"),
        cover_ref("https://i.gr-assets.com/images/x.jpg", "somewhere"),
        cover_ref("https://static.fnac-static.com/multimedia/x.jpg", "somewhere"),
        // Not https, an address, a placeholder asset, garbage.
        cover_ref("http://www.phare-imaginaire.example/media/couv.jpg", "onix"),
        cover_ref("https://192.168.1.10/couv.jpg", "onix"),
        cover_ref("https://cdn.example.org/images/G/01/x-site/icons/no-img-sm.gif", "onix"),
        cover_ref("not a url", "onix"),
    ];
    // Without a Google key the Google reference is skipped.
    let list = candidates(&b, &Policy::default());
    assert_eq!(
        shape(&list)[..3],
        [
            (
                "openlibrary",
                CandidateRole::Preview,
                "https://covers.openlibrary.org/b/id/8739161-M.jpg?default=false"
            ),
            (
                "openlibrary",
                CandidateRole::Link,
                "https://covers.openlibrary.org/b/id/8739161.jpg?default=false"
            ),
            (
                "reference",
                CandidateRole::Link,
                "https://www.phare-imaginaire.example/media/couv.jpg"
            ),
        ]
    );
    // Id-keyed requests: one per second; the declared size travels with the candidate.
    assert_eq!((list[0].min_interval_ms, list[0].declared_px), (1000, Some((325, 500))));
    assert_eq!(list[2].min_interval_ms, 5000);
    // Then the ISBN-keyed lookup, as usual.
    assert_eq!(list[3].role, CandidateRole::Locate);
    assert_eq!(list.len(), 6);

    let with_key = Policy { google_books_api_key: Some("key".to_owned()), ..Policy::default() };
    let list = candidates(&b, &with_key);
    assert_eq!(
        shape(&list)[2..4],
        [
            (
                "googlebooks",
                CandidateRole::Preview,
                "https://books.google.com/books/content?id=AbC_dEf&printsec=frontcover&img=1&zoom=1&source=gbs_api"
            ),
            (
                "googlebooks",
                CandidateRole::Link,
                "https://books.google.com/books/content?id=AbC_dEf&printsec=frontcover&img=1&source=gbs_api&zoom=0"
            ),
        ]
    );
    // No candidate, under any policy, points at a denied host or a non-https URL.
    for policy in [
        Policy::default(),
        with_key,
        Policy::operator(RightsBasis::PrintLicence).with_source("bnf").with_source("dnb"),
    ] {
        for c in candidates(&b, &policy) {
            assert!(check_url(&c.request.url).is_ok(), "{}", c.request.url);
        }
    }
}

#[test]
fn url_builders() {
    let isbn = FR.parse().expect("isbn");
    assert_eq!(
        openlibrary_isbn_image_url(&isbn, OpenLibrarySize::Large),
        "https://covers.openlibrary.org/b/isbn/9782999900011-L.jpg?default=false"
    );
    assert_eq!(
        openlibrary_id_image_url(42, OpenLibrarySize::Small),
        "https://covers.openlibrary.org/b/id/42-S.jpg?default=false"
    );
    for (url, id) in [
        ("https://covers.openlibrary.org/b/id/8739161-L.jpg", Some(8_739_161)),
        ("https://covers.openlibrary.org/b/id/8739161.jpg?default=false", Some(8_739_161)),
        ("http://covers.openlibrary.org/b/id/12-S.jpg", Some(12)),
        ("https://covers.openlibrary.org/b/id/12.json", Some(12)),
        ("https://covers.openlibrary.org/b/isbn/9782999900011-L.jpg", None),
        ("https://covers.openlibrary.org/b/id/-L.jpg", None),
        ("https://covers.openlibrary.org/b/id/0.jpg", None),
        ("https://covers.openlibrary.org/b/id/12/../../x.jpg", None),
        ("https://covers.openlibrary.org.evil.example/b/id/12-L.jpg", None),
        ("https://evil.example/covers.openlibrary.org/b/id/12-L.jpg", None),
        ("covers.openlibrary.org/b/id/12-L.jpg", None),
    ] {
        assert_eq!(openlibrary_cover_id(url), id, "{url}");
    }
    assert_eq!(
        google_books_image_url(
            "http://books.google.com/books?id=x1&printsec=frontcover&img=1&zoom=5&edge=curl",
            Some(1)
        )
        .as_deref(),
        Some("https://books.google.com/books?id=x1&printsec=frontcover&img=1&zoom=1")
    );
    for not_google in [
        "https://books.google.com.evil.example/books?id=x1",
        "https://evil.example/books?id=x1",
        "https://books.google.com/books?printsec=frontcover",
        "https://books.google.com/other?id=x1",
        "javascript://books.google.com/books?id=x1",
        "",
    ] {
        assert_eq!(google_books_image_url(not_google, None), None, "{not_google}");
    }
    assert_eq!(
        bnf_ark_in("<identifier>https://catalogue.bnf.fr/ark:/12148/cb34567890z</identifier>")
            .as_deref(),
        Some("ark:/12148/cb34567890z")
    );
    assert_eq!(bnf_ark_in("ark:/12148/cb12"), None);
    assert_eq!(bnf_ark_in("ark:/99999/cb34567890z"), None);
    assert_eq!(
        bnf_cover_url("ark:/12148/cb34567890z&evil=1 x").as_deref(),
        Some(
            "https://catalogue.bnf.fr/couverture?appName=NE&idArk=ark:/12148/cb34567890z&couverture=1"
        )
    );
    assert_eq!(bnf_cover_url("cb34567890z"), None);
}

#[test]
fn publisher_opt_out_stops_everything() {
    let mut policy = Policy::default();
    policy.opt_out.isbn_prefixes = vec!["978-2-9999".to_owned()];
    assert!(candidates(&book(FR), &policy).is_empty());
    assert_eq!(candidates(&book(DE), &policy).len(), 3);
    // A related ISBN of a listed publisher blocks too.
    let mut related = book(DE);
    related.related_isbns = vec![FR_PRINT.parse().expect("isbn")];
    assert!(candidates(&related, &policy).is_empty());

    let mut policy = Policy::default();
    policy.opt_out.publishers = vec!["  éditions du phare imaginaire ".to_owned()];
    assert!(candidates(&book(FR), &policy).is_empty());
    let mut imprint = book(DE);
    imprint.publisher = Some("Groupe Fictif".to_owned());
    imprint.imprint = Some("ÉDITIONS DU PHARE IMAGINAIRE".to_owned());
    assert!(candidates(&imprint, &policy).is_empty());

    // A host opt-out removes references and pasted pages on that host and its subdomains.
    let mut policy = Policy::default();
    policy.opt_out.hosts = vec!["phare-imaginaire.example".to_owned()];
    let mut b = book(FR);
    b.publisher = None;
    b.covers = vec![cover_ref("https://cdn.phare-imaginaire.example/couv.jpg", "onix")];
    assert!(candidates(&b, &policy).iter().all(|c| c.source == "openlibrary"));
    assert_eq!(
        page_candidate("https://www.phare-imaginaire.example/livre", &policy).err(),
        Some(CoversError::UrlRefused(Refusal::OptedOut))
    );
    assert!(page_candidate("https://www.autre-phare-imaginaire.example/livre", &policy).is_ok());
    assert_eq!(
        policy
            .may_fetch("https://www.phare-imaginaire.example/x", Some(&Robots::allow_all()))
            .err(),
        Some(Refusal::OptedOut)
    );
    // Empty entries block nothing.
    let mut policy = Policy::default();
    policy.opt_out.isbn_prefixes = vec![String::new(), "-".to_owned()];
    policy.opt_out.publishers = vec![String::new()];
    policy.opt_out.hosts = vec![String::new(), ".".to_owned()];
    assert_eq!(candidates(&book(FR), &policy).len(), 3);
    assert!(page_candidate("https://www.editeur.example/livre", &policy).is_ok());
}

#[test]
fn reseller_mode_never_looks_anything_up() {
    let mut b = book(FR);
    b.covers = vec![
        // The reseller's own CDN may well be a host the lookup denylist names.
        CoverRef {
            kind: CoverKind::Front,
            width_px: Some(1400),
            height_px: Some(2100),
            ..cover_ref("https://static.fnac-static.com/multimedia/feed/9782999900011.jpg", "feed")
        },
        CoverRef {
            kind: CoverKind::FullWrap,
            ..cover_ref("https://feed.revendeur.example/flat/9782999900011.pdf.jpg", "feed")
        },
        cover_ref("http://feed.revendeur.example/insecure.jpg", "feed"),
        cover_ref("https://10.0.0.5/internal.jpg", "feed"),
        cover_ref("https://feed.revendeur.example/images/G/01/x-site/icons/no-img-sm.gif", "feed"),
    ];
    let list = candidates(&b, &Policy::reseller());
    assert_eq!(
        shape(&list),
        [
            (
                "reseller_feed",
                CandidateRole::Link,
                "https://static.fnac-static.com/multimedia/feed/9782999900011.jpg"
            ),
            (
                "reseller_feed",
                CandidateRole::Link,
                "https://feed.revendeur.example/flat/9782999900011.pdf.jpg"
            ),
        ]
    );
    assert_eq!(
        (list[0].risk, list[0].declared_px, list[1].kind),
        (Risk::Low, Some((1400, 2100)), CoverKind::FullWrap)
    );
    // Still no right to print: that comes from the reseller's per-title flag.
    assert!(list.iter().all(|c| c.rights_basis() == RightsBasis::None));
    // No feed cover: nothing at all — the card is typographic.
    assert!(candidates(&book(FR), &Policy::reseller()).is_empty());
    assert_eq!(
        page_candidate("https://www.editeur.example/livre", &Policy::reseller()).err(),
        Some(CoversError::UrlRefused(Refusal::LookupsDisabled))
    );
    assert_eq!(Policy::reseller().mode, Mode::Reseller);
    // Even with every key and switch on.
    let eager = Policy {
        mode: Mode::Reseller,
        google_books_api_key: Some("k".to_owned()),
        ..Policy::operator(RightsBasis::PrintLicence)
    }
    .with_source("bnf")
    .with_source("dnb");
    assert!(candidates(&book(FR), &eager).is_empty());
}

#[test]
fn pasted_page_candidates() {
    let policy = Policy::default();
    let page = page_candidate(
        "  https://www.phare-imaginaire.example/catalogue/le-phare-des-brumes ",
        &policy,
    )
    .expect("page");
    assert_eq!(
        (page.source, page.role, page.risk),
        ("publisher_page", CandidateRole::Page, Risk::Medium)
    );
    assert_eq!(
        page.request.url,
        "https://www.phare-imaginaire.example/catalogue/le-phare-des-brumes"
    );
    assert_eq!(page.min_interval_ms, 5000);
    for (url, refusal) in [
        ("https://www.amazon.fr/dp/2999900015", Refusal::DeniedHost("Amazon")),
        ("https://www.fnac.com/a123/Le-Phare", Refusal::DeniedHost("Fnac")),
        ("https://www.decitre.fr/livres/x.html", Refusal::DeniedHost("Decitre")),
        ("https://www.babelio.com/livres/x", Refusal::DeniedHost("Babelio")),
        ("https://www.goodreads.com/book/show/1", Refusal::DeniedHost("Goodreads")),
        ("http://www.phare-imaginaire.example/x", Refusal::NotHttps),
        ("https://localhost/x", Refusal::LocalName),
        ("https://[fd00::1]/x", Refusal::IpAddress),
        ("", Refusal::Unparseable),
    ] {
        assert_eq!(
            page_candidate(url, &policy).err(),
            Some(CoversError::UrlRefused(refusal)),
            "{url}"
        );
    }
    assert!(
        page_candidate("https://www.amazon.fr/dp/x", &policy)
            .expect_err("refused")
            .to_string()
            .contains("supply the image file directly")
    );

    // From the hostile fixture only the one clean image becomes a link.
    let found = extract(&fixture("pages/hostile.html"), "https://www.piege.example/livre");
    let links = page_image_candidates(&found, &book(FR), &policy);
    assert_eq!(
        shape(&links),
        [("publisher_page", CandidateRole::Link, "https://www.piege.example/ok/cover.jpg")]
    );
    // An operator who holds the rights also gets the download step, rated High.
    let operator = Policy::operator(RightsBasis::RightsHolderDeclared);
    let steps = page_image_candidates(&found, &book(FR), &operator);
    assert_eq!(
        steps.iter().map(|c| (c.role, c.risk)).collect::<Vec<_>>(),
        [(CandidateRole::Link, Risk::Medium), (CandidateRole::Download, Risk::High)]
    );
    // The page states a related ISBN only: flagged as another edition.
    let found = extract(
        &fixture("pages/publisher_jsonld_book.html"),
        "https://www.phare-imaginaire.example/catalogue/x",
    );
    let mut paper_only = book("9782999900035");
    paper_only.related_isbns = vec![FR_PRINT.parse().expect("isbn")];
    let links = page_image_candidates(&found, &paper_only, &policy);
    assert_eq!(links.len(), 4);
    assert!(links.iter().all(|c| c.other_edition && c.role == CandidateRole::Link));
    assert_eq!(links[0].declared_px, None);
    assert_eq!(links[1].declared_px, Some((600, 900)));
    // Reseller mode and opt-outs give nothing.
    assert!(page_image_candidates(&found, &book(FR), &Policy::reseller()).is_empty());
}

#[test]
fn fetch_permission_combines_guard_and_robots() {
    let policy = Policy::default();
    let robots = Robots::parse(&fixture("robots/publisher.txt"));
    let host = "https://www.phare-imaginaire.example";
    let verdict =
        |path: &str| policy.may_fetch(&format!("{host}{path}"), Some(&robots)).map(|_| ());
    // Our own group applies, not the `*` group.
    assert_eq!(verdict("/catalogue/le-phare-des-brumes"), Ok(()));
    assert_eq!(verdict("/espace-presse/couvertures-hd/phare.jpg"), Err(Refusal::RobotsDisallow));
    assert_eq!(verdict("/espace-presse/communiques/2025.html"), Ok(()));
    assert_eq!(verdict("/media/logo.png"), Ok(()));
    assert_eq!(verdict("/robots.txt"), Ok(()));
    // Another bot gets the `*` group.
    let other = Policy {
        user_agent: "AutreBot/1.0 (+https://autre.example/bot)".to_owned(),
        ..Policy::default()
    };
    assert_eq!(
        other.may_fetch(&format!("{host}/media/logo.png"), Some(&robots)).err(),
        Some(Refusal::RobotsDisallow)
    );
    assert!(other.may_fetch(&format!("{host}/media/couvertures/phare.jpg"), Some(&robots)).is_ok());
    assert_eq!(
        other.may_fetch(&format!("{host}/catalogue?tri=prix"), Some(&robots)).err(),
        Some(Refusal::RobotsDisallow)
    );
    assert_eq!(
        other.may_fetch(&format!("{host}/doc/extrait.pdf"), Some(&robots)).err(),
        Some(Refusal::RobotsDisallow)
    );

    // No verdict, no fetch; an unreachable robots.txt closes the site; a 404 opens it.
    assert_eq!(policy.may_fetch(&format!("{host}/x"), None).err(), Some(Refusal::RobotsNotChecked));
    assert_eq!(
        policy.may_fetch(&format!("{host}/x"), Some(&Robots::unreachable())).err(),
        Some(Refusal::RobotsDisallow)
    );
    assert_eq!(
        policy.may_fetch(&format!("{host}/x"), Some(&Robots::from_response(503, b""))).err(),
        Some(Refusal::RobotsDisallow)
    );
    assert!(policy.may_fetch(&format!("{host}/x"), Some(&Robots::from_response(404, b""))).is_ok());
    // The guard comes first, and robots.txt cannot open what the guard closes.
    assert_eq!(
        policy.may_fetch("https://www.amazon.fr/x", Some(&Robots::allow_all())).err(),
        Some(Refusal::DeniedHost("Amazon"))
    );
    // An operator may switch robots off for hosts it controls; the guard stays.
    let own = Policy { obey_robots: false, ..Policy::default() };
    assert!(own.may_fetch(&format!("{host}/espace-presse/x"), None).is_ok());
    assert!(own.may_fetch("https://127.0.0.1/x", None).is_err());

    // Delays: the candidate's interval, or the Crawl-delay when longer.
    let list = candidates(&book(FR), &policy);
    assert_eq!(policy.delay_ms(&list[0], None), 3000);
    assert_eq!(policy.delay_ms(&list[0], Some(&robots)), 3000, "our group asks for 2 s");
    assert_eq!(other.delay_ms(&list[0], Some(&robots)), 10_000, "the * group asks for 10 s");
    assert_eq!(own.delay_ms(&list[0], Some(&robots)), 3000);
    let slow = Robots::parse("User-agent: *\nCrawl-delay: 1e12\n");
    assert_eq!(policy.delay_ms(&list[0], Some(&slow)), 3_600_000, "capped at one hour");

    // The companion requests of a pasted page.
    let robots_request =
        policy.robots_request(&format!("{host}/catalogue/x?y=1")).expect("request");
    assert_eq!(robots_request.url, "https://www.phare-imaginaire.example/robots.txt");
    let tdm = policy.tdmrep_request(&format!("{host}/catalogue/x")).expect("request");
    assert_eq!(tdm.url, "https://www.phare-imaginaire.example/.well-known/tdmrep.json");
    assert!(policy.robots_request("https://www.amazon.fr/x").is_none());
    assert!(policy.tdmrep_request("http://insecure.example/x").is_none());
}

#[test]
fn lookup_order_follows_the_policy() {
    let reordered = Policy {
        allowed_sources: ["bnf", "openlibrary", "bnf"].map(String::from).to_vec(),
        ..Policy::default()
    };
    let sources: Vec<&str> = candidates(&book(FR), &reordered).iter().map(|c| c.source).collect();
    assert_eq!(sources, ["bnf", "openlibrary", "openlibrary", "openlibrary"]);
}

#[test]
fn pasted_pages_can_be_restricted_to_listed_publishers() {
    let policy = Policy {
        page_hosts: vec!["phare-imaginaire.example".to_owned(), String::new()],
        ..Policy::default()
    };
    assert!(page_candidate("https://www.phare-imaginaire.example/livre", &policy).is_ok());
    assert!(page_candidate("https://phare-imaginaire.example/livre", &policy).is_ok());
    for other in ["https://blog.example.org/ma-chronique", "https://notphare-imaginaire.example/x"]
    {
        assert_eq!(
            page_candidate(other, &policy).err(),
            Some(CoversError::UrlRefused(Refusal::NotAllowListed)),
            "{other}"
        );
    }
    // The denylist still comes first.
    assert_eq!(
        page_candidate("https://www.amazon.fr/x", &policy).err(),
        Some(CoversError::UrlRefused(Refusal::DeniedHost("Amazon")))
    );
}

#[test]
fn accepting_a_response() {
    use ebc_model::HttpResponse;
    let response =
        |status: u16, headers: &[(&str, &str)], body: &[u8], final_url: &str| HttpResponse {
            status,
            headers: headers.iter().map(|(n, v)| ((*n).to_owned(), (*v).to_owned())).collect(),
            body: body.to_vec(),
            final_url: final_url.to_owned(),
        };
    let policy =
        Policy::operator(RightsBasis::RightsHolderDeclared).with_source("bnf").with_source("dnb");
    let list = candidates(&book(FR), &policy);
    let by_role = |role: CandidateRole| {
        list.iter().find(|c| c.role == role && c.source == "openlibrary").expect("candidate")
    };
    let locate = by_role(CandidateRole::Locate);
    let preview = by_role(CandidateRole::Preview);
    let download = by_role(CandidateRole::Download);
    let ol = "https://covers.openlibrary.org/b/id/1.json";

    // Status decides first: `default=false` turns "no cover" into a 404.
    assert!(locate.accepts(&response(200, &[], b"{\"id\":1,\"width\":300,\"height\":450}", ol)));
    assert!(!locate.accepts(&response(404, &[], b"not found", ol)));
    assert!(!preview.accepts(&response(404, &[], b"", ol)));
    assert!(!preview.accepts(&response(429, &[], b"", ol)));
    assert!(preview.accepts(&response(
        200,
        &[("Content-Type", "image/jpeg")],
        b"",
        "https://ia800100.us.archive.org/1/items/x/cover.jpg"
    )));
    // An HTML error page with a 200, a redirect that ends on a denied or internal host.
    assert!(!preview.accepts(&response(
        200,
        &[("content-type", "text/html; charset=utf-8")],
        b"",
        ol
    )));
    assert!(!preview.accepts(&response(
        200,
        &[],
        b"",
        "https://images-na.ssl-images-amazon.com/images/P/x.jpg"
    )));
    assert!(!preview.accepts(&response(200, &[], b"", "https://192.168.0.1/x.jpg")));
    assert!(!preview.accepts(&response(
        200,
        &[],
        b"",
        "http://covers.openlibrary.org/b/id/1-M.jpg"
    )));
    // A downloaded body must be a real image and not a placeholder.
    let cover = common::png_rgb(300, 450);
    assert!(download.accepts(&response(200, &[], &cover, ol)));
    assert!(!download.accepts(&response(200, &[], &common::gif(1, 1), ol)));
    assert!(!download.accepts(&response(200, &[], b"<html>Image not available</html>", ol)));
    assert!(!download.accepts(&response(200, &[], &cover[..20], ol)));

    // BnF: the two placeholder lengths; DNB: any error.
    let mut with_ark = book(FR);
    with_ark.sources.push(SourceRef {
        provider: "bnf".to_owned(),
        record_url: Some("ark:/12148/cb34567890z".to_owned()),
        licence: None,
        attribution: None,
    });
    let bnf = candidates(&with_ark, &policy)
        .into_iter()
        .find(|c| c.source == "bnf" && c.role == CandidateRole::Probe)
        .expect("bnf probe");
    let bnf_url = bnf.request.url.clone();
    assert!(bnf.accepts(&response(200, &[("Content-Length", "48213")], b"", &bnf_url)));
    assert!(bnf.accepts(&response(200, &[], b"", &bnf_url)));
    for placeholder in ["4566", " 4658 "] {
        assert!(
            !bnf.accepts(&response(200, &[("content-length", placeholder)], b"", &bnf_url)),
            "{placeholder}"
        );
    }
    assert!(!bnf.accepts(&response(500, &[("Content-Length", "48213")], b"", &bnf_url)));
    let dnb =
        candidates(&book(DE), &policy).into_iter().find(|c| c.source == "dnb").expect("dnb probe");
    assert!(dnb.accepts(&response(200, &[("Content-Type", "image/jpeg")], b"", &dnb.request.url)));
    assert!(!dnb.accepts(&response(404, &[], b"", &dnb.request.url)));
    // The same length is not a placeholder elsewhere.
    assert!(preview.accepts(&response(200, &[("Content-Length", "4566")], b"", ol)));
}

#[test]
fn user_agent_rules() {
    let policy = Policy::default();
    assert_eq!(policy.product_token(), "eBookCadeauBot");
    assert!(policy.user_agent_problems().is_empty(), "{:?}", policy.user_agent_problems());
    assert!(DEFAULT_USER_AGENT.to_ascii_lowercase().contains("bot"));
    let problems = |ua: &str| {
        Policy { user_agent: ua.to_owned(), ..Policy::default() }.user_agent_problems().len()
    };
    assert_eq!(
        problems("eBookCadeauBot/1.0 (+https://cadeau.example/bot; contact@cadeau.example)"),
        0
    );
    assert!(problems("") >= 2);
    assert!(
        problems("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120 Safari/537.36")
            >= 1
    );
    assert!(problems("eBookCadeauBot/1.0") >= 1, "no URL, no contact");
    assert!(problems("Amazon CloudFront (+https://x.example)") >= 1);
    assert!(problems("eBookCadeauBot/1.0 (+https://x.example)\r\nX-Injected: 1") >= 1);
    assert!(problems("1.0 (+https://x.example)") >= 1);
}

#[test]
fn source_table_invariants() {
    let mut ids: Vec<&str> = SOURCES.iter().map(|s| s.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), SOURCES.len(), "ids are unique");
    for info in SOURCES {
        assert!(!info.name.is_empty() && info.note.len() > 40, "{}", info.id);
        assert!(
            info.preview_risk <= info.store_risk && info.store_risk <= info.print_risk,
            "{}",
            info.id
        );
        match info.status {
            SourceStatus::Never | SourceStatus::ContractOnly => {
                assert!(info.uses.is_empty() && info.hosts.is_empty(), "{}", info.id);
                assert_eq!(info.preview_risk, Risk::High);
            }
            _ => {
                assert!(!info.uses.is_empty(), "{}", info.id);
                // The hosted service refuses any source whose preview risk is High.
                assert!(info.preview_risk <= Risk::Medium, "{}", info.id);
            }
        }
        // No lookup source lets the hosted service store or print.
        if info.id != "reseller_feed" {
            assert_eq!((info.store_risk, info.print_risk), (Risk::High, Risk::High), "{}", info.id);
        }
        // Allow-listed hosts are never on the denylist.
        for host in info.hosts {
            assert!(!ebc_covers::is_denied_host(host), "{host}");
        }
        // Honest about what was verified: nobody has read any terms at source yet.
        assert_eq!(info.terms_last_read, None, "{}", info.id);
        assert_eq!(source(info.id), Some(info));
    }
    // Google images are never downloaded; national catalogues are link-only.
    assert!(!source("googlebooks").expect("source").allows(Use::DownloadCli));
    for id in ["bnf", "dnb"] {
        assert_eq!(source(id).expect("source").uses, [Use::LinkOnly]);
        assert_eq!(source(id).expect("source").status, SourceStatus::DisabledUntilTermsRead);
    }
    assert!(source("openlibrary").expect("source").allows(Use::PreviewInline));
    assert!(Risk::Low < Risk::Medium && Risk::Medium < Risk::High);
    assert_eq!(Policy::default().max_risk, Risk::Medium);
}
