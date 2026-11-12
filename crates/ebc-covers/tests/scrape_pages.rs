//! `scrape::extract` on hand-written pages imitating publisher and bookseller markup.

mod common;

use common::fixture;
use ebc_covers::Refusal;
use ebc_covers::scrape::{Found, IsbnMatch, MAX_HTML_BYTES, MAX_IMAGES, PageInfo, Via, extract};
use ebc_isbn::Isbn13;
use ebc_model::BookMeta;

const EBOOK: &str = "9782999900011";
const PAPER: &str = "9782999900028";
const OTHER: &str = "9782999900035";
const GERMAN: &str = "9783999900018";

fn isbn(s: &str) -> Isbn13 {
    s.parse().expect("valid ISBN")
}

fn isbns(page: &PageInfo) -> Vec<String> {
    page.isbns.iter().map(ToString::to_string).collect()
}

fn summary(page: &PageInfo) -> Vec<(&str, Via, Option<u32>, Option<u32>)> {
    page.images.iter().map(|f| (f.url.as_str(), f.via, f.width, f.height)).collect()
}

fn find<'a>(page: &'a PageInfo, needle: &str) -> &'a Found {
    page.images
        .iter()
        .find(|f| f.url.contains(needle))
        .unwrap_or_else(|| panic!("no image matching {needle}"))
}

#[test]
fn publisher_page_with_a_book_node() {
    let html = fixture("pages/publisher_jsonld_book.html");
    let page =
        extract(&html, "https://www.phare-imaginaire.example/catalogue/le-phare-des-brumes?utm=x");
    assert_eq!(
        summary(&page),
        [
            // Relative to <base href>, not to the page URL.
            (
                "https://www.phare-imaginaire.example/catalogue/couvertures/le-phare-des-brumes-hd.jpg",
                Via::JsonLd,
                None,
                None
            ),
            // Entity-encoded attribute: &amp; becomes &.
            (
                "https://cdn.phare-imaginaire.example/couv/9782999900011.jpg?w=600&h=900&fit=max",
                Via::OpenGraph,
                Some(600),
                Some(900)
            ),
            (
                "https://www.phare-imaginaire.example/media/social/le-phare-des-brumes-tw.jpg",
                Via::TwitterCard,
                None,
                None
            ),
            (
                "https://www.phare-imaginaire.example/media/couvertures/le-phare-des-brumes.jpg",
                Via::LinkImageSrc,
                None,
                None
            ),
        ]
    );
    // The author's portrait and the publisher's logo are not cover candidates.
    assert!(!page.images.iter().any(|f| f.url.contains("auteurs") || f.url.contains("logo")));
    assert!(page.images.windows(2).all(|w| w[0].confidence >= w[1].confidence));
    assert_eq!(page.images[0].confidence, 0.95);
    assert_eq!(page.images[0].isbns, [isbn(EBOOK)]);
    assert!(page.images.iter().all(|f| f.refusal.is_none()));
    // Structured ISBN first, then the one found only in the visible text (written with
    // numeric character references for its hyphens).
    assert_eq!(isbns(&page), [EBOOK, PAPER]);
    assert_eq!(page.title.as_deref(), Some("Le Phare des brumes"));
    assert_eq!(page.tdm_reservation, None);
    assert!(!page.noai && !page.truncated);

    // Confirmation against the requested book.
    let mut book = BookMeta::titled("Le Phare des brumes");
    assert_eq!(page.confirms(&book), IsbnMatch::NotFound);
    book.isbn13 = Some(isbn(EBOOK));
    assert_eq!(page.confirms(&book), IsbnMatch::Exact);
    book.isbn13 = Some(isbn(OTHER));
    assert_eq!(page.confirms(&book), IsbnMatch::NotFound);
    book.related_isbns = vec![isbn(PAPER)];
    assert_eq!(page.confirms(&book), IsbnMatch::OtherEdition);
}

#[test]
fn graph_with_image_objects_and_nested_editions() {
    let html = fixture("pages/publisher_graph_imageobject.html");
    let page = extract(&html, "https://www.phare-imaginaire.example/livre/le-phare-des-brumes/");
    let base = "https://www.phare-imaginaire.example/wp-content/uploads/2025/03";
    assert_eq!(
        summary(&page),
        [
            (format!("{base}/phare-des-brumes.jpg").as_str(), Via::JsonLd, Some(1400), Some(2100)),
            (format!("{base}/phare-broche.jpg").as_str(), Via::JsonLd, None, None),
            (format!("{base}/phare-broche-4e.jpg").as_str(), Via::JsonLd, Some(1400), Some(2100)),
            (
                format!("{base}/phare-social-1200x630.jpg").as_str(),
                Via::OpenGraph,
                Some(1200),
                Some(630)
            ),
        ]
    );
    // ["http://schema.org/Book", "Product"] takes the Book confidence; the edition
    // nested under workExample is a step lower and carries its own ISBN.
    assert_eq!(page.images[0].confidence, 0.95);
    assert_eq!(page.images[0].isbns, [isbn(EBOOK)], "gtin13 given as a JSON number");
    assert!((page.images[1].confidence - 0.8).abs() < 1e-6);
    assert_eq!(page.images[1].isbns, [isbn(PAPER)]);
    // Organization logo and WebPage image references are ignored.
    assert!(!page.images.iter().any(|f| f.url.contains("logo")));
    assert_eq!(isbns(&page), [EBOOK, PAPER]);
    assert_eq!(page.title.as_deref(), Some("Le Phare des brumes"));
}

#[test]
fn bookseller_page_with_cms_defects() {
    let html = fixture("pages/bookseller_product.html");
    let page = extract(&html, "https://www.coin-fictif.example/livre/9782999900011");
    let img = "https://www.coin-fictif.example/img";
    assert_eq!(
        summary(&page),
        [
            // Found five times (JSON-LD, og:image:secure_url, a duplicate og:image,
            // twitter:image:src, itemprop): listed once, with the declared size merged in.
            (
                format!("{img}/produits/9782999900011_1_75.jpg").as_str(),
                Via::JsonLd,
                Some(340),
                Some(500)
            ),
            (format!("{img}/produits/9782999900011_2_75.jpg").as_str(), Via::JsonLd, None, None),
            (format!("{img}/produits/9782999900035_1_75.jpg").as_str(), Via::JsonLd, None, None),
            (
                "http://www.coin-fictif.example/img/produits/9782999900011_1_75.jpg",
                Via::OpenGraph,
                Some(340),
                Some(500)
            ),
            (format!("{img}/boutique/vitrine.jpg").as_str(), Via::OpenGraph, None, None),
            (format!("{img}/produits/9782999900011_zoom.jpg").as_str(), Via::Microdata, None, None),
        ]
    );
    // The JSON-LD block survived "</script>" inside a string, a raw newline and a raw
    // tab; the plain http URL is listed but refused.
    assert_eq!(page.images[0].isbns, [isbn(EBOOK)]);
    assert_eq!(find(&page, "http://").refusal, Some(Refusal::NotHttps));
    // "You may also like": lower confidence, and tied to another ISBN.
    let related = find(&page, "9782999900035_1_75");
    assert!(related.confidence < page.images[0].confidence);
    assert_eq!(related.isbns, [isbn(OTHER)]);
    // Nothing from the comment, the inline script or the style sheet.
    for ghost in ["commented-out", "from-javascript", "from-css"] {
        assert!(!page.images.iter().any(|f| f.url.contains(ghost)), "{ghost}");
    }
    assert_eq!(isbns(&page), [EBOOK, OTHER], "the ISBN-10 in the text is the same book");
    assert_eq!(page.title.as_deref(), Some("Le Phare des brumes"));
    assert_eq!(page.usable().count(), 5);
}

#[test]
fn malformed_markup() {
    let html = fixture("pages/malformed.html");
    let page = extract(&html, "https://www.nebelmeer-verlag.example/buch/nebelmeer");
    let host = "https://www.nebelmeer-verlag.example";
    assert_eq!(
        summary(&page),
        [
            // From the entity-encoded, CDATA-wrapped JSON-LD block; the broken block
            // before it and the empty image array after it give nothing.
            (format!("{host}/cover/9783999900018-ld.jpg").as_str(), Via::JsonLd, None, None),
            // Unquoted, upper-case attributes; width and height in separate tags with
            // odd spacing and attribute order.
            (
                format!("{host}/cover/9783999900018.jpg").as_str(),
                Via::OpenGraph,
                Some(800),
                Some(1200)
            ),
            (format!("{host}/cover/second.jpg").as_str(), Via::OpenGraph, None, None),
            (format!("{host}/cover/tw/9783999900018.jpg").as_str(), Via::TwitterCard, None, None),
            (format!("{host}/cover/src.jpg").as_str(), Via::LinkImageSrc, None, None),
            (format!("{host}/buch/cover.jpg").as_str(), Via::Microdata, Some(400), None),
        ]
    );
    assert!(!page.images.iter().any(|f| f.url.contains("thing") || f.url.contains("in-comment")));
    assert_eq!(isbns(&page), [GERMAN], "the ISBN with a wrong check digit is not one");
    // The Book node's name beats the CreativeWork's and the <TITLE>.
    assert_eq!(page.title.as_deref(), Some("Nebelmeer"));
}

#[test]
fn title_fallbacks() {
    let og = extract(
        "<title>Site | Livre</title><meta property='og:title' content=' Le  Livre '>",
        "https://a.example/",
    );
    assert_eq!(og.title.as_deref(), Some("Le Livre"));
    let plain = extract(
        "<TITLE>\n  L&rsquo;&Eacute;t&eacute; &amp; l'hiver \n</TITLE><title>second</title>",
        "https://a.example/",
    );
    assert_eq!(plain.title.as_deref(), Some("L\u{2019}Été & l'hiver"));
    assert_eq!(extract("<p>no title</p>", "https://a.example/").title, None);
    let long = format!("<title>{}</title>", "é".repeat(5000));
    assert_eq!(extract(&long, "https://a.example/").title.map(|t| t.chars().count()), Some(300));
}

#[test]
fn hostile_page_every_trap_is_flagged() {
    let html = fixture("pages/hostile.html");
    let page = extract(&html, "https://www.piege.example/livre");
    // Exactly one image may be requested or linked.
    let usable: Vec<&str> = page.usable().map(|f| f.url.as_str()).collect();
    assert_eq!(usable, ["https://www.piege.example/ok/cover.jpg"]);
    // Non-web schemes are not even listed.
    for scheme in ["javascript:", "data:", "file:", "ftp:"] {
        assert!(!page.images.iter().any(|f| f.url.starts_with(scheme)), "{scheme}");
    }
    let refusal = |needle: &str| find(&page, needle).refusal;
    assert_eq!(refusal("169.254.169.254"), Some(Refusal::NotHttps));
    assert_eq!(refusal("localhost"), Some(Refusal::LocalName));
    assert_eq!(refusal("[::1]"), Some(Refusal::IpAddress));
    assert_eq!(
        refusal("127.0.0.1"),
        Some(Refusal::IpAddress),
        "decimal host 2130706433 is an address"
    );
    assert_eq!(refusal("//intranet/"), Some(Refusal::LocalName));
    assert_eq!(refusal("secret@"), Some(Refusal::Credentials));
    assert_eq!(refusal(":8443"), Some(Refusal::Port));
    assert_eq!(refusal("metadata.google.internal"), Some(Refusal::LocalName));
    assert_eq!(refusal("ssl-images-amazon"), Some(Refusal::DeniedHost("Amazon")));
    assert_eq!(refusal("gr-assets"), Some(Refusal::DeniedHost("Goodreads")));
    assert_eq!(refusal("fnac-static"), Some(Refusal::DeniedHost("Fnac")));
    assert_eq!(refusal("no-img-sm.gif"), Some(Refusal::PlaceholderUrl));
    assert_eq!(page.images.iter().filter(|f| f.refusal.is_some()).count(), 12);
    // Opt-out signals of the document.
    assert_eq!(page.tdm_reservation, Some(true));
    assert!(page.noai);
    assert_eq!(isbns(&page), [EBOOK]);
}

#[test]
fn base_href_cannot_smuggle_an_internal_host() {
    let html = r#"<base href="https://localhost:443/x/"><meta property="og:image" content="cover.jpg">
                  <meta name="twitter:image" content="https://www.editeur.example/c.jpg">"#;
    let page = extract(html, "https://www.editeur.example/livre");
    assert_eq!(page.images[0].url, "https://localhost/x/cover.jpg");
    assert_eq!(page.images[0].refusal, Some(Refusal::LocalName));
    assert_eq!(page.usable().count(), 1);
    // A relative <base> is resolved against the page; only the first <base> counts.
    let html = r#"<base href="/a/b/"><base href="https://other.example/"><link rel=image_src href="c.jpg">"#;
    assert_eq!(
        extract(html, "https://www.editeur.example/livre/1").images[0].url,
        "https://www.editeur.example/a/b/c.jpg"
    );
}

#[test]
fn relative_urls_need_a_base() {
    let html = r#"<meta property="og:image" content="/rel.jpg"><meta name="twitter:image" content="https://a.example/abs.jpg">
                  <link rel="image_src" href="//cdn.example.org/proto-relative.jpg">"#;
    for bad_base in ["", "not a url", "mailto:x@example.org", "file:///tmp/page.html"] {
        let page = extract(html, bad_base);
        assert_eq!(
            summary(&page),
            [("https://a.example/abs.jpg", Via::TwitterCard, None, None)],
            "{bad_base:?}"
        );
    }
    let page = extract(html, "https://www.editeur.example/a/b?c#d");
    assert_eq!(page.images.len(), 3);
    assert_eq!(page.images[0].url, "https://www.editeur.example/rel.jpg");
    assert_eq!(find(&page, "proto-relative").url, "https://cdn.example.org/proto-relative.jpg");
    // Fragments are dropped, queries kept.
    let page =
        extract(r#"<meta property="og:image" content="c.jpg?v=2#top">"#, "https://a.example/x/");
    assert_eq!(page.images[0].url, "https://a.example/x/c.jpg?v=2");
}

#[test]
fn open_graph_structured_properties() {
    // Dimensions belong to the image they follow; a size declared before any image is
    // given to the first one; og:image:url is an alias of og:image.
    let html = r#"
      <meta property="og:image:width" content="111">
      <meta property="og:image" content="https://a.example/1.jpg">
      <meta property="og:image:height" content="222">
      <meta property="og:image:url" content="https://a.example/2.jpg">
      <meta property="og:image:width" content="333px">
      <meta property="og:image:height" content="0">
      <meta property="og:image:width" content="999">
      <meta name="og:image" content="https://a.example/3.jpg">
      <meta property="og:image:width" content="banana">"#;
    let page = extract(html, "https://a.example/");
    assert_eq!(
        summary(&page),
        [
            ("https://a.example/1.jpg", Via::OpenGraph, Some(111), Some(222)),
            ("https://a.example/2.jpg", Via::OpenGraph, Some(333), None),
            ("https://a.example/3.jpg", Via::OpenGraph, None, None),
        ]
    );
}

#[test]
fn json_ld_shapes_seen_in_the_wild() {
    let cases: [(&str, &[&str]); 9] = [
        // image as a plain string, type with the schema.org URL prefix
        (r#"{"@type":"https://schema.org/Book","image":"/a.jpg"}"#, &["/a.jpg"]),
        // array of strings and ImageObjects mixed, contentUrl, @id
        (
            r#"{"@type":"Product","image":["/a.jpg",{"@type":"ImageObject","contentUrl":"/b.jpg"},{"@id":"/c.jpg"},42,null]}"#,
            &["/a.jpg", "/b.jpg", "/c.jpg"],
        ),
        // top-level array of nodes
        (
            r#"[{"@type":"Organization","image":"/logo.jpg"},{"@type":"Book","image":{"url":"/a.jpg"}}]"#,
            &["/a.jpg"],
        ),
        // mainEntity nesting below an untyped-for-us node
        (
            r#"{"@type":"WebPage","image":"/page.jpg","mainEntity":{"@type":"Book","image":"/a.jpg"}}"#,
            &["/a.jpg"],
        ),
        // lower-case and "type" instead of "@type"
        (r#"{"type":"book","image":"/a.jpg"}"#, &["/a.jpg"]),
        // CreativeWork
        (r#"{"@type":["Thing","CreativeWork"],"image":"/a.jpg"}"#, &["/a.jpg"]),
        // not a type we read
        (r#"{"@type":"Person","image":"/me.jpg"}"#, &[]),
        // image of the wrong JSON type
        (r#"{"@type":"Book","image":{"url":{"nested":"/x.jpg"}}}"#, &[]),
        // no type at all
        (r#"{"image":"/a.jpg"}"#, &[]),
    ];
    for (json, expected) in cases {
        let html = format!(r#"<script type="application/ld+json">{json}</script>"#);
        let page = extract(&html, "https://a.example/");
        let urls: Vec<String> =
            page.images.iter().map(|f| f.url.replace("https://a.example", "")).collect();
        assert_eq!(urls, expected, "{json}");
        assert!(page.images.iter().all(|f| f.via == Via::JsonLd));
    }
    // A script of another type is not JSON-LD, whatever it contains.
    let html = r#"<script type="application/json">{"@type":"Book","image":"/a.jpg"}</script>
                  <script>{"@type":"Book","image":"/b.jpg"}</script>"#;
    assert!(extract(html, "https://a.example/").images.is_empty());
}

#[test]
fn script_end_inside_a_json_string() {
    // Tolerated when the block parses that way…
    let html = r#"<script type="application/ld+json">{"@type":"Book","description":"a </script> b","image":"/after.jpg"}</script>
                  <meta property="og:image" content="/og.jpg">"#;
    let page = extract(html, "https://a.example/");
    assert_eq!(
        summary(&page).iter().map(|s| s.0).collect::<Vec<_>>(),
        ["https://a.example/after.jpg", "https://a.example/og.jpg"]
    );
    // …and when it does not (an unterminated string), the HTML rule applies and the
    // rest of the page is still read.
    let html = r#"<script type="application/ld+json">{"@type":"Book","name":"never closed}</script>
                  <meta property="og:image" content="/og.jpg"><p>ISBN 978-2-9999-0001-1</p>"#;
    let page = extract(html, "https://a.example/");
    assert_eq!(
        summary(&page).iter().map(|s| s.0).collect::<Vec<_>>(),
        ["https://a.example/og.jpg"]
    );
    assert_eq!(isbns(&page), [EBOOK]);
    // An escaped quote before the closing tag keeps the string open, correctly.
    let html = r#"<script type="application/ld+json">{"@type":"Book","name":"a \" </script> b","image":"/x.jpg"}</script>"#;
    assert_eq!(extract(html, "https://a.example/").images.len(), 1);
}

#[test]
fn work_is_bounded_on_huge_and_repetitive_input() {
    // 5 MB of junk after a small head: the head is read, the input is cut at 2 MiB.
    let mut html = String::from(
        r#"<html><head><meta property="og:image" content="/cover.jpg"></head><body>ISBN 9782999900011 "#,
    );
    let junk = [
        "<<<<<<<<",
        "<a href=",
        "<div class='x' ",
        "&amp;&&&#;",
        "</",
        "<!-",
        "\"'\"'",
        "<script>",
        "é€",
        " 978 ",
    ];
    let mut i = 0usize;
    while html.len() < 5 * 1024 * 1024 {
        html.push_str(junk[i % junk.len()]);
        i += 1;
    }
    html.push_str(r#"<meta property="og:image" content="/too-late.jpg">"#);
    let page = extract(&html, "https://a.example/");
    assert!(page.truncated);
    assert_eq!(page.images.len(), 1);
    assert_eq!(page.images[0].url, "https://a.example/cover.jpg");
    assert_eq!(isbns(&page), [EBOOK]);
    assert!(html.len() > MAX_HTML_BYTES);

    // Hundreds of unterminated JSON-LD blocks: the per-page cap stops the rescans.
    let mut html = String::new();
    for _ in 0..400 {
        html.push_str(
            r#"<script type="application/ld+json">{"@type":"Book","image":"/x.jpg","name":"open"#,
        );
        html.push_str(&"x".repeat(2000));
    }
    let _ = extract(&html, "https://a.example/");

    // Many valid blocks: only the first 32 are read, and images are capped.
    let mut html = String::new();
    for i in 0..200 {
        html.push_str(&format!(
            r#"<script type="application/ld+json">{{"@type":"Book","image":"/{i}.jpg"}}</script>"#
        ));
        html.push_str(&format!(r#"<meta name="twitter:image" content="/t{i}.jpg">"#));
    }
    let page = extract(&html, "https://a.example/");
    assert_eq!(page.images.iter().filter(|f| f.via == Via::JsonLd).count(), 32);
    assert_eq!(page.images.len(), MAX_IMAGES);

    // Deeply nested JSON and a node flood do not overflow the stack or run long.
    let deep = format!(
        r#"<script type="application/ld+json">{}{}</script>"#,
        "[".repeat(50_000),
        "]".repeat(50_000)
    );
    assert!(extract(&deep, "https://a.example/").images.is_empty());
    let nodes = format!(
        r#"<script type="application/ld+json">[{}{{"@type":"Book","image":"/last.jpg"}}]</script>"#,
        r#"{"a":[{"b":{}}]},"#.repeat(30_000)
    );
    let _ = extract(&nodes, "https://a.example/");

    // One enormous attribute, one enormous tag, one enormous comment.
    let big_attr = format!(
        r#"<meta property="og:image" content="/{}.jpg"><meta name="twitter:image" content="/ok.jpg">"#,
        "a".repeat(100_000)
    );
    assert_eq!(extract(&big_attr, "https://a.example/").images.len(), 1);
    let big_tag = format!("<meta {} property=og:image content=/ok.jpg>", "x=y ".repeat(50_000));
    let _ = extract(&big_tag, "https://a.example/");
    let big_comment =
        format!("<!--{}--><meta property=og:image content=/ok.jpg>", "-".repeat(1_000_000));
    assert_eq!(extract(&big_comment, "https://a.example/").images.len(), 1);
}

#[test]
fn isbn_sources() {
    let html = r#"
      <meta property="book:isbn" content="978-2-9999-0002-8">
      <meta itemprop="isbn" content="9782999900035">
      <script type="application/ld+json">{"@type":"Book","isbn":["9783999900018"],"sameAs":"https://a.example/ean/9782999900011"}</script>
      <p>ISBN&nbsp;: 2-9999-0001-5</p><script>var ean = "9791099990013";</script>"#;
    let page = extract(html, "https://a.example/");
    // Structured fields in document order, then the JSON-LD text, then visible text;
    // a number inside an ordinary script is not page content.
    assert_eq!(isbns(&page), [PAPER, OTHER, GERMAN, EBOOK]);
    assert!(page.mentions(&isbn(EBOOK)));
    assert!(!page.mentions(&isbn("9791099990013")));
}
