//! `merge` and the bibliographic normalisation it applies.

mod common;

use common::{EN, FR, isbn, ok};
use ebc_catalog::providers::googlebooks::GoogleBooks;
use ebc_catalog::providers::openlibrary::OpenLibrary;
use ebc_catalog::providers::sru::SruProvider;
use ebc_catalog::{Provider, combine, merge, normalize};
use ebc_model::{BookMeta, Contributor, ContributorRole, CoverKind, CoverRef, SourceRef};

fn record(provider: &dyn Provider, path: &str) -> BookMeta {
    let isbn = isbn(FR);
    let request = provider.lookup(&isbn).into_iter().next().expect("one request");
    let parsed = provider.parse(&isbn, &request, &ok(path)).expect("parses");
    parsed.records.into_iter().next().expect("one record")
}

fn book(title: &str, authors: &[&str]) -> BookMeta {
    BookMeta {
        isbn13: Some(isbn(FR)),
        title: title.to_owned(),
        contributors: authors.iter().map(|a| Contributor::author(*a)).collect(),
        ..BookMeta::default()
    }
}

fn source(provider: &str) -> SourceRef {
    SourceRef { provider: provider.to_owned(), record_url: None, licence: None, attribution: None }
}

#[test]
fn three_sources_for_one_book() {
    let bnf = record(&SruProvider::bnf(), "bnf/etranger-unimarc.xml");
    let openlibrary = record(&OpenLibrary::new(), "openlibrary/etranger.json");
    let google = record(&GoogleBooks::new(Some("k".to_owned())), "googlebooks/etranger.json");
    let merged = merge(&[bnf.clone(), openlibrary.clone(), google.clone()]).expect("merged");

    assert_eq!(merged.isbn13, Some(isbn(FR)));
    assert_eq!(merged.title, "L'étranger");
    assert_eq!(
        merged.publisher.as_deref(),
        Some("Gallimard"),
        "\"Editions Gallimard\" is the same publisher"
    );
    // The national library says 2015 (this printing); two others say 1971. The preferred
    // source keeps the field unless three sources agree against it.
    assert_eq!(merged.published.as_deref(), Some("2015"));
    assert_eq!(merged.page_count, Some(183));
    assert_eq!(merged.language.as_deref(), Some("fr"));
    assert_eq!(merged.series.as_deref(), Some("Folio"));
    assert_eq!(merged.author_line().as_deref(), Some("Albert Camus"));
    assert_eq!(merged.contributors.len(), 2, "the preface writer known to the BnF only is kept");
    assert_eq!(merged.description, openlibrary.description, "from the first source that has one");
    assert_eq!(merged.format, openlibrary.format);
    let providers: Vec<&str> = merged.sources.iter().map(|s| s.provider.as_str()).collect();
    assert_eq!(
        providers,
        ["bnf", "openlibrary", "googlebooks"],
        "every source is credited, in order"
    );
    assert_eq!(merged.covers.len(), 3, "one Open Library and two Google references");
    assert_eq!(merged.covers[0].source, "openlibrary");
    assert_eq!(merged.subjects.len(), 3);

    // Priority is the order of the slice.
    let reversed = merge(&[google, openlibrary, bnf]).expect("merged");
    assert_eq!(
        reversed.published.as_deref(),
        Some("1971-12"),
        "the most precise of the agreeing dates"
    );
    assert_eq!(reversed.page_count, Some(191));
    assert_eq!(reversed.publisher.as_deref(), Some("Editions Gallimard"));

    // Merging is idempotent.
    assert_eq!(merge(std::slice::from_ref(&merged)).as_ref(), Some(&merged));
}

#[test]
fn three_agreeing_sources_overrule_the_preferred_one() {
    let dated = |date: &str| BookMeta { published: Some(date.to_owned()), ..book("T", &["A B"]) };
    let two = merge(&[dated("1971"), dated("1942"), dated("1942")]).expect("merged");
    assert_eq!(two.published.as_deref(), Some("1971"));
    let three =
        merge(&[dated("1971"), dated("1942"), dated("1942-05"), dated("1942")]).expect("merged");
    assert_eq!(three.published.as_deref(), Some("1942-05"));

    let titled = |title: &str| book(title, &["A B"]);
    let merged = merge(&[
        titled("Wrong title"),
        titled("The Stranger"),
        titled("the stranger"),
        titled("THE STRANGER"),
    ])
    .expect("merged");
    assert_eq!(merged.title, "The Stranger");
}

#[test]
fn shouting_and_accentless_spellings_lose() {
    let merged =
        merge(&[book("L'ETRANGER", &["CAMUS, Albert"]), book("L'Étranger", &["Albert Camus"])])
            .expect("merged");
    assert_eq!(merged.title, "L'Étranger", "same title; the mixed-case spelling is kept");
    assert_eq!(
        merged.contributors.len(),
        1,
        "\"CAMUS, Albert\" and \"Albert Camus\" are one person"
    );
    assert_eq!(
        merged.contributors[0].name, "Albert CAMUS",
        "the preferred source's spelling, in natural order"
    );
}

#[test]
fn authors_need_agreement_other_roles_do_not() {
    let top = book("The Stranger", &["Albert Camus"]);
    let mislabelled = book("The Stranger", &["A. Camus", "Matthew Ward"]);
    let merged = merge(&[top.clone(), mislabelled.clone()]).expect("merged");
    assert_eq!(
        merged.author_line().as_deref(),
        Some("Albert Camus"),
        "one source alone cannot add an author"
    );

    let merged = merge(&[top.clone(), mislabelled.clone(), mislabelled]).expect("merged");
    assert_eq!(
        merged.author_line().as_deref(),
        Some("Albert Camus & Matthew Ward"),
        "two sources can"
    );

    let with_translator = BookMeta {
        contributors: vec![
            Contributor { name: "Ward, Matthew".to_owned(), role: ContributorRole::Translator },
            Contributor::author("Camus, Albert (1913-1960)"),
        ],
        ..book("The Stranger", &[])
    };
    let merged = merge(&[top, with_translator.clone()]).expect("merged");
    assert_eq!(merged.author_line().as_deref(), Some("Albert Camus"));
    assert_eq!(
        merged.contributors[1],
        Contributor { name: "Matthew Ward".to_owned(), role: ContributorRole::Translator }
    );

    // Without any author in the preferred record, the next one supplies them.
    let anonymous = book("The Stranger", &[]);
    let merged = merge(&[anonymous, with_translator]).expect("merged");
    assert_eq!(merged.contributors[0], Contributor::author("Albert Camus"), "authors come first");
}

#[test]
fn the_isbn_is_never_lost() {
    let mut first = book("Pride and Prejudice", &["Jane Austen"]);
    first.isbn13 = None;
    let second = BookMeta { related_isbns: vec![isbn(EN)], ..book("Pride and prejudice", &[]) };
    let third = BookMeta { isbn13: Some(isbn(EN)), ..book("Pride and Prejudice", &[]) };
    let merged = merge(&[first, second, third]).expect("merged");
    assert_eq!(merged.isbn13, Some(isbn(FR)), "the first ISBN stated");
    assert_eq!(merged.related_isbns, [isbn(EN)], "another edition's ISBN is kept once, as related");
}

#[test]
fn sources_and_covers_are_united_without_duplicates() {
    let cover = |url: &str, source: &str| CoverRef {
        url: url.to_owned(),
        kind: CoverKind::Front,
        width_px: None,
        height_px: None,
        source: source.to_owned(),
    };
    let a = BookMeta {
        sources: vec![source("bnf")],
        covers: vec![
            cover("https://example.org/a.jpg", "x"),
            cover("ftp://example.org/b.jpg", "x"),
        ],
        subjects: vec!["Roman français".to_owned()],
        ..book("T", &["A B"])
    };
    let b = BookMeta {
        sources: vec![source("bnf"), source("openlibrary")],
        covers: vec![
            cover("https://example.org/a.jpg", "y"),
            cover("https://example.org/c.jpg", "y"),
        ],
        subjects: vec!["roman francais".to_owned(), "Fiction".to_owned()],
        ..book("T", &["A B"])
    };
    let merged = merge(&[a, b]).expect("merged");
    assert_eq!(merged.sources, [source("bnf"), source("openlibrary")]);
    let urls: Vec<&str> = merged.covers.iter().map(|c| c.url.as_str()).collect();
    assert_eq!(
        urls,
        ["https://example.org/a.jpg", "https://example.org/c.jpg"],
        "web URLs only, once each"
    );
    assert_eq!(merged.subjects, ["Roman français", "Fiction"]);
}

#[test]
fn nothing_to_merge() {
    assert_eq!(merge(&[]), None);
    assert_eq!(merge(&[BookMeta::default(), book("  ", &["A B"])]), None, "no record has a title");
    assert_eq!(combine(&[]), None);
}

#[test]
fn merge_normalises_catalogue_artefacts() {
    let raw = BookMeta {
        isbn13: Some(isbn(FR)),
        title: "\u{98}L'\u{9c}étranger [Texte imprimé] : roman / Albert Camus ; préface de X"
            .to_owned(),
        contributors: vec![
            Contributor::author("Camus, Albert (1913-1960)"),
            Contributor::author("CAMUS Albert"),
        ],
        publisher: Some("[Paris] : Gallimard, DL 2015".to_owned()),
        series: Some("Collection Folio ; 2".to_owned()),
        published: Some("impr. 2011, cop. 1942".to_owned()),
        language: Some("fre".to_owned()),
        description: Some(
            "<p>Un <b>roman</b> &eacute;trange.</p>\n<script>alert(1)</script>".to_owned(),
        ),
        ..BookMeta::default()
    };
    let merged = merge(&[raw]).expect("merged");
    assert_eq!(merged.title, "L'étranger");
    assert_eq!(merged.subtitle.as_deref(), Some("roman"));
    assert_eq!(merged.contributors, [Contributor::author("Albert Camus")]);
    assert_eq!(merged.publisher.as_deref(), Some("Gallimard"));
    assert_eq!(merged.series.as_deref(), Some("Folio"));
    assert_eq!(merged.published.as_deref(), Some("2011"));
    assert_eq!(merged.language.as_deref(), Some("fr"));
    assert_eq!(merged.description.as_deref(), Some("Un roman étrange."));
}

/// The rules the brief lists, one by one.
#[test]
fn normalisation_rules() {
    use normalize::{date, language, page_count, person_name, publisher, strip_html, title};
    // General material designations.
    assert_eq!(title("L'étranger [Texte imprimé]"), "L'étranger");
    assert_eq!(title("Der Process [Elektronische Ressource]"), "Der Process");
    assert_eq!(title("Emma [electronic resource] /"), "Emma");
    assert_eq!(title("Poèmes [1920-1930]"), "Poèmes [1920-1930]", "other brackets are content");
    // Statement of responsibility.
    assert_eq!(title("L'étranger / Albert Camus"), "L'étranger");
    // Non-sorting marks.
    assert_eq!(title("\u{98}Der \u{9c}Zauberberg"), "Der Zauberberg");
    assert_eq!(title("<<Der>> Zauberberg"), "Der Zauberberg");
    assert_eq!(title("<<L'>>étranger"), "L'étranger");
    // Names.
    assert_eq!(person_name("Camus, Albert (1913-1960)"), "Albert Camus");
    assert_eq!(person_name("Kafka, Franz, 1883-1924"), "Franz Kafka");
    assert_eq!(person_name("Tolkien, J. R. R."), "J. R. R. Tolkien");
    assert_eq!(person_name("Molière (1622-1673)"), "Molière");
    // Languages.
    for code in ["fre", "fra", "fr", "FRE", "French", "/languages/fre"] {
        assert_eq!(language(code).as_deref(), Some("fr"), "{code}");
    }
    assert_eq!(language("ger").as_deref(), Some("de"));
    assert_eq!(language("und"), None);
    // Dates.
    assert_eq!(date("DL 2015").as_deref(), Some("2015"));
    assert_eq!(date("cop. 1942").as_deref(), Some("1942"));
    assert_eq!(date("[1972]").as_deref(), Some("1972"));
    assert_eq!(date("impr. 2011").as_deref(), Some("2011"));
    assert_eq!(date("c1988.").as_deref(), Some("1988"));
    assert_eq!(date("[s.d.]"), None);
    // Extents.
    assert_eq!(page_count("1 vol. (185 p.)"), Some(185));
    assert_eq!(page_count("XII-185 p."), Some(185));
    assert_eq!(page_count("185, [2] s."), Some(185));
    assert_eq!(page_count("3 vol."), None);
    // Publication statements.
    assert_eq!(publisher("[Paris] : Gallimard").as_deref(), Some("Gallimard"));
    assert_eq!(
        publisher("Frankfurt am Main : S. Fischer Verlag, 2011").as_deref(),
        Some("S. Fischer Verlag")
    );
    assert_eq!(publisher("[S.l.] : [s.n.]"), None);
    // HTML.
    assert_eq!(strip_html("<i>Roman</i>&nbsp;:<br/>1942"), "Roman :\n1942");
}
