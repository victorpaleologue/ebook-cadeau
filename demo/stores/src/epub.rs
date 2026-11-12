//! Minimal, valid EPUB 3 files for the demo catalogue, with an optional visible watermark.
//!
//! Layout of the container:
//!
//! ```text
//! mimetype                  stored, first, no extra field ("application/epub+zip")
//! META-INF/container.xml
//! OEBPS/package.opf         identifiers, title, creator, language, modified date
//! OEBPS/nav.xhtml           EPUB 3 navigation document
//! OEBPS/toc.ncx             EPUB 2 table of contents, for old reading systems
//! OEBPS/style.css
//! OEBPS/cover.png           small rendition of the store's typographic cover
//! OEBPS/title.xhtml
//! OEBPS/watermark.xhtml     only in stamped copies
//! OEBPS/chapter-1.xhtml     abridged opening of the public-domain text
//! ```
//!
//! Output is deterministic: entries carry the fixed DOS timestamp 1980-01-01 and the
//! `dcterms:modified` date is supplied by the caller.

use std::io::{Cursor, Write};

use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::catalogue::{Book, ISBN_LABEL, PUBLISHER};
use crate::covers::xml_escape;
use crate::error::StoreError;
use crate::time;

/// Media type of an EPUB file.
pub const MEDIA_TYPE: &str = "application/epub+zip";

/// `dcterms:modified` of the unstamped demo editions (2026-01-01T00:00:00Z).
pub const BASE_EDITION_UNIX: i64 = 1_767_225_600;

/// A personalisation stamped into a copy: a visible page after the title page, plus
/// metadata in the package document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watermark {
    /// The store's trading name.
    pub store_name: String,
    /// Order (line) reference, e.g. `SEP-2026-000001`.
    pub order_ref: String,
    /// Who the copy is for, e.g. `Offert à Sacha par Camille` or `Exemplaire de a@b.example`.
    pub licensee: String,
    /// When the copy was prepared (Unix seconds); also used as `dcterms:modified`.
    pub stamped_at: i64,
}

/// Builds the EPUB of `book`. `cover_png` is embedded as the cover image; `watermark`, when
/// present, adds the watermark page and metadata.
///
/// # Errors
/// [`StoreError::Epub`] when the ZIP container cannot be written.
pub fn build(
    book: &Book,
    cover_png: &[u8],
    watermark: Option<&Watermark>,
) -> Result<Vec<u8>, StoreError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::DEFAULT.compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::DEFAULT.compression_method(CompressionMethod::Deflated);

    // The OCF container requires `mimetype` to be the first entry, uncompressed and
    // without extra field, so that the type can be sniffed at a fixed offset.
    add(&mut zip, "mimetype", stored, MEDIA_TYPE.as_bytes())?;
    add(&mut zip, "META-INF/container.xml", deflated, CONTAINER_XML.as_bytes())?;
    add(&mut zip, "OEBPS/package.opf", deflated, package_opf(book, watermark).as_bytes())?;
    add(&mut zip, "OEBPS/nav.xhtml", deflated, nav_xhtml(book, watermark.is_some()).as_bytes())?;
    add(&mut zip, "OEBPS/toc.ncx", deflated, toc_ncx(book, watermark.is_some()).as_bytes())?;
    add(&mut zip, "OEBPS/style.css", deflated, STYLE_CSS.as_bytes())?;
    // PNG data is already compressed.
    add(&mut zip, "OEBPS/cover.png", stored, cover_png)?;
    add(&mut zip, "OEBPS/title.xhtml", deflated, title_xhtml(book).as_bytes())?;
    if let Some(mark) = watermark {
        add(&mut zip, "OEBPS/watermark.xhtml", deflated, watermark_xhtml(book, mark).as_bytes())?;
    }
    add(&mut zip, "OEBPS/chapter-1.xhtml", deflated, chapter_xhtml(book).as_bytes())?;

    let cursor = zip.finish().map_err(|e| StoreError::Epub(format!("closing the archive: {e}")))?;
    Ok(cursor.into_inner())
}

fn add(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &str,
    options: SimpleFileOptions,
    data: &[u8],
) -> Result<(), StoreError> {
    zip.start_file(name, options).map_err(|e| StoreError::Epub(format!("{name}: {e}")))?;
    zip.write_all(data).map_err(|e| StoreError::Epub(format!("{name}: {e}")))
}

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

const STYLE_CSS: &str = "body { margin: 5%; line-height: 1.5; font-family: serif; }\n\
h1 { font-size: 1.6em; text-align: center; margin: 2em 0 1em; }\n\
h2 { font-size: 1.2em; margin: 1.5em 0 1em; }\n\
p { margin: 0; text-indent: 1.2em; text-align: justify; }\n\
p.verse { text-indent: 0; text-align: left; margin: 0 0 1em 1.2em; }\n\
p.center { text-indent: 0; text-align: center; margin: 1em 0; }\n\
p.note { text-indent: 0; font-size: 0.85em; margin-top: 2em; }\n\
div.watermark { margin-top: 30%; border: 1px solid #000; padding: 1em; }\n";

/// The publication's unique identifier.
fn urn(book: &Book) -> String {
    format!("urn:isbn:{}", book.isbn13)
}

fn package_opf(book: &Book, watermark: Option<&Watermark>) -> String {
    let modified = time::iso8601(watermark.map_or(BASE_EDITION_UNIX, |w| w.stamped_at));
    let mut s = String::with_capacity(2048);
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(
        "<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" \
         unique-identifier=\"pub-id\" xml:lang=\"fr\" \
         prefix=\"demo: https://ebook-cadeau.example/ns/demo-store#\">\n",
    );
    s.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");
    s.push_str(&format!("    <dc:identifier id=\"pub-id\">{}</dc:identifier>\n", urn(book)));
    s.push_str(&format!("    <dc:title>{}</dc:title>\n", xml_escape(&book.full_title())));
    s.push_str(&format!(
        "    <dc:creator id=\"creator\">{}</dc:creator>\n",
        xml_escape(book.author)
    ));
    s.push_str("    <dc:language>fr</dc:language>\n");
    s.push_str(&format!("    <dc:publisher>{}</dc:publisher>\n", xml_escape(PUBLISHER)));
    s.push_str(&format!("    <dc:date>{}</dc:date>\n", book.year));
    s.push_str(&format!(
        "    <dc:description>{}</dc:description>\n",
        xml_escape(book.description)
    ));
    let rights = match watermark {
        Some(w) => format!(
            "Texte du domaine public. Exemplaire personnel — {} — commande {} — {}.",
            w.licensee, w.order_ref, w.store_name
        ),
        None => "Texte du domaine public. Édition de démonstration, extrait abrégé.".to_owned(),
    };
    s.push_str(&format!("    <dc:rights>{}</dc:rights>\n", xml_escape(&rights)));
    s.push_str(&format!("    <meta property=\"dcterms:modified\">{modified}</meta>\n"));
    if let Some(w) = watermark {
        s.push_str(&format!(
            "    <meta property=\"demo:store\">{}</meta>\n\
             \x20   <meta property=\"demo:order\">{}</meta>\n\
             \x20   <meta property=\"demo:licensee\">{}</meta>\n",
            xml_escape(&w.store_name),
            xml_escape(&w.order_ref),
            xml_escape(&w.licensee)
        ));
    }
    s.push_str("    <meta name=\"cover\" content=\"cover-image\"/>\n");
    s.push_str("  </metadata>\n  <manifest>\n");
    s.push_str(
        "    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" \
         properties=\"nav\"/>\n\
         \x20   <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n\
         \x20   <item id=\"css\" href=\"style.css\" media-type=\"text/css\"/>\n\
         \x20   <item id=\"cover-image\" href=\"cover.png\" media-type=\"image/png\" \
         properties=\"cover-image\"/>\n\
         \x20   <item id=\"title\" href=\"title.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
    );
    if watermark.is_some() {
        s.push_str(
            "    <item id=\"watermark\" href=\"watermark.xhtml\" \
             media-type=\"application/xhtml+xml\"/>\n",
        );
    }
    s.push_str(
        "    <item id=\"chapter-1\" href=\"chapter-1.xhtml\" \
         media-type=\"application/xhtml+xml\"/>\n  </manifest>\n  <spine toc=\"ncx\">\n\
         \x20   <itemref idref=\"title\"/>\n",
    );
    if watermark.is_some() {
        s.push_str("    <itemref idref=\"watermark\"/>\n");
    }
    s.push_str("    <itemref idref=\"chapter-1\"/>\n  </spine>\n</package>\n");
    s
}

fn xhtml_page(title: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n\
         <html xmlns=\"http://www.w3.org/1999/xhtml\" \
         xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"fr\" xml:lang=\"fr\">\n\
         <head>\n<meta charset=\"utf-8\"/>\n<title>{}</title>\n\
         <link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\"/>\n</head>\n\
         <body>\n{body}</body>\n</html>\n",
        xml_escape(title)
    )
}

fn nav_xhtml(book: &Book, stamped: bool) -> String {
    let mut items = String::from("<li><a href=\"title.xhtml\">Page de titre</a></li>\n");
    if stamped {
        items.push_str("<li><a href=\"watermark.xhtml\">Exemplaire personnel</a></li>\n");
    }
    items.push_str(&format!(
        "<li><a href=\"chapter-1.xhtml\">{}</a></li>\n",
        xml_escape(book.chapter_title)
    ));
    xhtml_page(
        "Sommaire",
        &format!("<nav epub:type=\"toc\" id=\"toc\">\n<h1>Sommaire</h1>\n<ol>\n{items}</ol>\n</nav>\n"),
    )
}

fn toc_ncx(book: &Book, stamped: bool) -> String {
    let mut points = String::new();
    let mut order = 0;
    let mut point = |label: &str, src: &str| {
        order += 1;
        points.push_str(&format!(
            "    <navPoint id=\"np-{order}\" playOrder=\"{order}\">\n      \
             <navLabel><text>{}</text></navLabel>\n      <content src=\"{src}\"/>\n    \
             </navPoint>\n",
            xml_escape(label)
        ));
    };
    point("Page de titre", "title.xhtml");
    if stamped {
        point("Exemplaire personnel", "watermark.xhtml");
    }
    point(book.chapter_title, "chapter-1.xhtml");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\" xml:lang=\"fr\">\n  \
         <head>\n    <meta name=\"dtb:uid\" content=\"{uid}\"/>\n    \
         <meta name=\"dtb:depth\" content=\"1\"/>\n    \
         <meta name=\"dtb:totalPageCount\" content=\"0\"/>\n    \
         <meta name=\"dtb:maxPageNumber\" content=\"0\"/>\n  </head>\n  \
         <docTitle><text>{title}</text></docTitle>\n  <navMap>\n{points}  </navMap>\n</ncx>\n",
        uid = urn(book),
        title = xml_escape(&book.full_title()),
    )
}

fn title_xhtml(book: &Book) -> String {
    let mut body = format!("<h1>{}</h1>\n", xml_escape(book.title));
    if let Some(sub) = book.subtitle {
        body.push_str(&format!("<p class=\"center\">{}</p>\n", xml_escape(sub)));
    }
    body.push_str(&format!(
        "<p class=\"center\">{}</p>\n<p class=\"center\">{}</p>\n\
         <p class=\"note\">{} — {ISBN_LABEL} : {}. Extrait abrégé d'un texte du domaine public, \
         préparé pour une démonstration ; ce n'est pas une édition de référence.</p>\n",
        xml_escape(book.author),
        book.year,
        xml_escape(PUBLISHER),
        book.isbn13,
    ));
    xhtml_page(&book.full_title(), &body)
}

fn watermark_xhtml(book: &Book, mark: &Watermark) -> String {
    let body = format!(
        "<div class=\"watermark\" id=\"watermark\">\n<h2>Exemplaire personnel</h2>\n\
         <p class=\"center\" id=\"watermark-licensee\">{licensee}</p>\n\
         <p class=\"center\">« {title} » — {author}</p>\n\
         <p class=\"center\">Commande <span id=\"watermark-order\">{order}</span> — {store}, \
         le {date}</p>\n\
         <p class=\"note\">Ce fichier est sans verrou numérique (DRM). Il porte ce marquage \
         visible et des métadonnées d'identification : merci de ne pas le diffuser.</p>\n</div>\n",
        licensee = xml_escape(&mark.licensee),
        title = xml_escape(&book.full_title()),
        author = xml_escape(book.author),
        order = xml_escape(&mark.order_ref),
        store = xml_escape(&mark.store_name),
        date = time::french_date(mark.stamped_at),
    );
    xhtml_page("Exemplaire personnel", &body)
}

fn chapter_xhtml(book: &Book) -> String {
    let mut body = format!("<h2>{}</h2>\n", xml_escape(book.chapter_title));
    for paragraph in book.paragraphs {
        if paragraph.contains('\n') {
            let lines: Vec<String> = paragraph.lines().map(xml_escape).collect();
            body.push_str(&format!("<p class=\"verse\">{}</p>\n", lines.join("<br/>\n")));
        } else {
            body.push_str(&format!("<p>{}</p>\n", xml_escape(paragraph)));
        }
    }
    body.push_str("<p class=\"note\">[…] Fin de l'extrait de démonstration.</p>\n");
    xhtml_page(book.chapter_title, &body)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use zip::ZipArchive;

    use super::*;
    use crate::catalogue;

    const FAKE_PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot really a picture";

    fn open(bytes: &[u8]) -> ZipArchive<Cursor<&[u8]>> {
        ZipArchive::new(Cursor::new(bytes)).expect("a readable zip")
    }

    fn read(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> String {
        let mut out = String::new();
        archive.by_name(name).unwrap_or_else(|_| panic!("{name} missing")).read_to_string(&mut out).unwrap();
        out
    }

    fn mark(licensee: &str, order: &str) -> Watermark {
        Watermark {
            store_name: "Septante".into(),
            order_ref: order.into(),
            licensee: licensee.into(),
            stamped_at: 1_794_441_600,
        }
    }

    #[test]
    fn mimetype_is_first_stored_and_sniffable_at_fixed_offsets() {
        for book in catalogue::books() {
            let bytes = build(book, FAKE_PNG, None).unwrap();
            // Local file header: signature, …, compression method at 8, name length at 26,
            // extra length at 28, name at 30, data right after.
            assert_eq!(&bytes[0..4], b"PK\x03\x04");
            assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), 0, "stored");
            assert_eq!(u16::from_le_bytes([bytes[26], bytes[27]]), 8, "name length");
            assert_eq!(u16::from_le_bytes([bytes[28], bytes[29]]), 0, "no extra field");
            assert_eq!(&bytes[30..38], b"mimetype");
            assert_eq!(&bytes[38..58], b"application/epub+zip");

            let mut archive = open(&bytes);
            let first = archive.by_index(0).unwrap();
            assert_eq!(first.name().unwrap(), "mimetype");
            assert_eq!(first.compression(), CompressionMethod::Stored);
            assert_eq!(first.size(), 20);
        }
    }

    #[test]
    fn package_is_consistent_with_the_container() {
        for book in catalogue::books() {
            for stamped in [false, true] {
                let wm = mark("Offert à Sacha", "SEP-2026-000001");
                let bytes = build(book, FAKE_PNG, stamped.then_some(&wm)).unwrap();
                let mut archive = open(&bytes);
                let container = read(&mut archive, "META-INF/container.xml");
                assert!(container.contains("full-path=\"OEBPS/package.opf\""));
                let opf = read(&mut archive, "OEBPS/package.opf");
                assert!(opf.contains("version=\"3.0\""));
                assert!(opf.contains(&format!(
                    "<dc:identifier id=\"pub-id\">urn:isbn:{}</dc:identifier>",
                    book.isbn13
                )));
                assert!(opf.contains("unique-identifier=\"pub-id\""));
                assert!(opf.contains(&format!("<dc:title>{}</dc:title>", xml_escape(&book.full_title()))));
                assert!(opf.contains(&format!(">{}</dc:creator>", book.author)));
                assert!(opf.contains("<dc:language>fr</dc:language>"));
                assert!(opf.contains("<meta property=\"dcterms:modified\">20"));
                assert!(opf.contains("properties=\"nav\""));

                // Every manifest href exists in the archive, and nothing else is shipped.
                let hrefs: Vec<&str> = opf
                    .split("href=\"")
                    .skip(1)
                    .filter_map(|rest| rest.split('"').next())
                    .collect();
                assert_eq!(hrefs.len(), if stamped { 7 } else { 6 });
                for href in &hrefs {
                    assert!(archive.by_name(&format!("OEBPS/{href}")).is_ok(), "{href} missing");
                }
                assert_eq!(archive.len(), hrefs.len() + 3, "mimetype, container, opf + manifest");

                // Every spine itemref points at a manifest item.
                for idref in opf.split("idref=\"").skip(1).filter_map(|r| r.split('"').next()) {
                    assert!(opf.contains(&format!("<item id=\"{idref}\"")), "{idref}");
                }
                assert_eq!(opf.contains("idref=\"watermark\""), stamped);

                let nav = read(&mut archive, "OEBPS/nav.xhtml");
                assert!(nav.contains("epub:type=\"toc\""));
                assert!(nav.contains("href=\"chapter-1.xhtml\""));
                let ncx = read(&mut archive, "OEBPS/toc.ncx");
                assert!(ncx.contains(&format!("content=\"urn:isbn:{}\"", book.isbn13)));
                let chapter = read(&mut archive, "OEBPS/chapter-1.xhtml");
                let opening: String = book.paragraphs[0].chars().take(40).collect();
                let opening = opening.lines().next().unwrap().to_owned();
                assert!(chapter.contains(&xml_escape(&opening)), "{}", book.id);
                let mut cover = Vec::new();
                archive.by_name("OEBPS/cover.png").unwrap().read_to_end(&mut cover).unwrap();
                assert_eq!(cover, FAKE_PNG);
            }
        }
    }

    #[test]
    fn every_xml_part_is_well_formed() {
        let wm = mark("Offert à <Sacha> & \"Cie\" ]]> <!-- x -->", "SEP-2026-000001");
        for book in catalogue::books() {
            let bytes = build(book, FAKE_PNG, Some(&wm)).unwrap();
            let mut archive = open(&bytes);
            let names: Vec<String> = archive.file_names().map(|n| n.unwrap().into_owned()).collect();
            for name in names {
                if name.ends_with(".xml")
                    || name.ends_with(".opf")
                    || name.ends_with(".xhtml")
                    || name.ends_with(".ncx")
                {
                    let text = read(&mut archive, &name);
                    let options = resvg::usvg::roxmltree::ParsingOptions {
                        allow_dtd: true,
                        ..Default::default()
                    };
                    resvg::usvg::roxmltree::Document::parse_with_options(&text, options)
                        .unwrap_or_else(|e| panic!("{name} of {} is not well-formed: {e}", book.id));
                }
            }
        }
    }

    #[test]
    fn watermark_is_visible_in_metadata_and_page_and_escaped() {
        let book = catalogue::find("candide").unwrap();
        let hostile = "Offert à <script>alert(1)</script> & \"Bob\"";
        let wm = mark(hostile, "SEP-2026-000042");
        let bytes = build(book, FAKE_PNG, Some(&wm)).unwrap();
        let mut archive = open(&bytes);
        let page = read(&mut archive, "OEBPS/watermark.xhtml");
        assert!(page.contains("SEP-2026-000042"));
        assert!(page.contains("12 novembre 2026"));
        assert!(page.contains("Offert à &lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;Bob&quot;"));
        assert!(!page.contains("<script"));
        let opf = read(&mut archive, "OEBPS/package.opf");
        assert!(opf.contains("<meta property=\"demo:order\">SEP-2026-000042</meta>"));
        assert!(opf.contains("<meta property=\"demo:licensee\">Offert à &lt;script&gt;"));
        assert!(opf.contains("<meta property=\"dcterms:modified\">2026-11-12T00:00:00Z</meta>"));
        assert!(!opf.contains("<script"));

        // The unstamped edition has neither.
        let plain = build(book, FAKE_PNG, None).unwrap();
        let mut archive = open(&plain);
        assert!(archive.by_name("OEBPS/watermark.xhtml").is_err());
        let opf = read(&mut archive, "OEBPS/package.opf");
        assert!(!opf.contains("demo:order"));
        assert!(opf.contains("<meta property=\"dcterms:modified\">2026-01-01T00:00:00Z</meta>"));
    }

    #[test]
    fn output_is_deterministic_and_stamps_differ() {
        let book = catalogue::find("le-horla").unwrap();
        let a = build(book, FAKE_PNG, None).unwrap();
        let b = build(book, FAKE_PNG, None).unwrap();
        assert_eq!(a, b);
        let s1 = build(book, FAKE_PNG, Some(&mark("Offert à A", "SEP-2026-000001"))).unwrap();
        let s2 = build(book, FAKE_PNG, Some(&mark("Offert à B", "SEP-2026-000001"))).unwrap();
        assert_ne!(s1, s2);
        assert_ne!(a, s1);
        assert_eq!(s1, build(book, FAKE_PNG, Some(&mark("Offert à A", "SEP-2026-000001"))).unwrap());
    }

    #[test]
    fn verse_keeps_its_line_breaks() {
        let book = catalogue::find("les-fleurs-du-mal").unwrap();
        let bytes = build(book, FAKE_PNG, None).unwrap();
        let chapter = read(&mut open(&bytes), "OEBPS/chapter-1.xhtml");
        assert!(chapter.contains("<p class=\"verse\">La sottise, l&#39;erreur, le péché, la lésine,<br/>"));
        assert_eq!(chapter.matches("<br/>").count(), 9, "three quatrains");
    }
}
