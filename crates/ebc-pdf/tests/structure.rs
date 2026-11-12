//! Document structure, parsed back with `lopdf`: page boxes, Info dictionary, XMP,
//! output intent, header version, trailer ID, page labels.

mod common;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Color, Item, Page, PdfStandard, Rect, Size};

const ALL: [PdfStandard; 4] =
    [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4];

fn simple_doc(standard: PdfStandard) -> ebc_scene::Document {
    let mut doc = base_doc(standard);
    let mut page = Page::new(Size::new(230.0, 178.0), 3.0, 7.0);
    page.items.push(Item::filled_rect(
        Rect::new(-3.0, -3.0, 236.0, 184.0),
        Color::cmyk(0.1, 0.0, 0.0, 0.0),
    ));
    doc.pages.push(page);
    doc
}

fn write(doc: &ebc_scene::Document) -> Vec<u8> {
    write_pdf(doc, &WriteOptions::default()).expect("document is writable")
}

#[test]
fn page_boxes_follow_trim_bleed_and_slug() {
    let mut doc = simple_doc(PdfStandard::X4);
    // A second page without bleed or slug, and a third with bleed only.
    doc.pages.push(Page::new(Size::new(210.0, 297.0), 0.0, 0.0));
    doc.pages.push(Page::new(Size::new(110.0, 178.0), 5.0, 0.0));
    let pdf = parse(&write(&doc));
    assert_eq!(pdf.get_pages().len(), 3);

    // Page 1: trim 230 x 178, bleed 3, slug 7 -> media is trim + 10 mm on every side.
    let p1 = page(&pdf, 1);
    assert_close(&get_numbers(&pdf, p1, "MediaBox"), &[0.0, 0.0, pt(250.0), pt(198.0)], 1e-3);
    assert_close(&get_numbers(&pdf, p1, "CropBox"), &[0.0, 0.0, pt(250.0), pt(198.0)], 1e-3);
    assert_close(
        &get_numbers(&pdf, p1, "BleedBox"),
        &[pt(7.0), pt(7.0), pt(243.0), pt(191.0)],
        1e-3,
    );
    assert_close(
        &get_numbers(&pdf, p1, "TrimBox"),
        &[pt(10.0), pt(10.0), pt(240.0), pt(188.0)],
        1e-3,
    );
    assert!(!p1.has(b"ArtBox"), "PDF/X allows TrimBox or ArtBox, not both");

    // Page 2: all boxes coincide.
    let p2 = page(&pdf, 2);
    let a4 = [0.0, 0.0, pt(210.0), pt(297.0)];
    for key in ["MediaBox", "CropBox", "BleedBox", "TrimBox"] {
        assert_close(&get_numbers(&pdf, p2, key), &a4, 1e-3);
    }

    // Page 3: media = bleed box, trim inset by the bleed.
    let p3 = page(&pdf, 3);
    assert_close(&get_numbers(&pdf, p3, "MediaBox"), &[0.0, 0.0, pt(120.0), pt(188.0)], 1e-3);
    assert_close(&get_numbers(&pdf, p3, "BleedBox"), &[0.0, 0.0, pt(120.0), pt(188.0)], 1e-3);
    assert_close(
        &get_numbers(&pdf, p3, "TrimBox"),
        &[pt(5.0), pt(5.0), pt(115.0), pt(183.0)],
        1e-3,
    );

    // Boxes nest exactly (no rounding crossover), on every page.
    for n in 1..=3 {
        let p = page(&pdf, n);
        let (m, b, t) = (
            get_numbers(&pdf, p, "MediaBox"),
            get_numbers(&pdf, p, "BleedBox"),
            get_numbers(&pdf, p, "TrimBox"),
        );
        assert!(m[0] <= b[0] && b[0] <= t[0] && m[1] <= b[1] && b[1] <= t[1], "page {n}");
        assert!(t[2] <= b[2] && b[2] <= m[2] && t[3] <= b[3] && b[3] <= m[3], "page {n}");
    }
}

#[test]
fn poche_card_boxes_match_the_print_requirements_table() {
    // docs/research/03-print-requirements.md, section 2.2: the poche spread (230 x 178 mm)
    // for the upload profile (bleed 3 mm, no slug) and the marks profile (slug 10 mm).
    let cases: [(f64, f64, [f64; 4], [f64; 4], [f64; 4]); 2] = [
        (
            3.0,
            0.0,
            [0.0, 0.0, 668.976, 521.575],
            [0.0, 0.0, 668.976, 521.575],
            [8.504, 8.504, 660.472, 513.071],
        ),
        (
            3.0,
            10.0,
            [0.0, 0.0, 725.669, 578.268],
            [28.346, 28.346, 697.323, 549.921],
            [36.850, 36.850, 688.819, 541.417],
        ),
    ];
    for (bleed, slug, media, bleed_box, trim) in cases {
        let mut doc = base_doc(PdfStandard::X4);
        doc.pages.push(Page::new(Size::new(230.0, 178.0), bleed, slug));
        doc.pages.push(Page::new(Size::new(230.0, 178.0), bleed, slug));
        let pdf = parse(&write(&doc));
        for n in 1..=2 {
            let p = page(&pdf, n);
            assert_close(&get_numbers(&pdf, p, "MediaBox"), &media, 1e-3);
            assert_close(&get_numbers(&pdf, p, "CropBox"), &media, 1e-3);
            assert_close(&get_numbers(&pdf, p, "BleedBox"), &bleed_box, 1e-3);
            assert_close(&get_numbers(&pdf, p, "TrimBox"), &trim, 1e-3);
            assert!(!p.has(b"ArtBox") && !p.has(b"Rotate") && !p.has(b"UserUnit"));
        }
    }
}

#[test]
fn content_is_mapped_from_top_left_millimetres_to_bottom_left_points() {
    let doc = simple_doc(PdfStandard::X4);
    let pdf = write_pdf(&doc, &WriteOptions { compress: false, keep_layers: false }).expect("ok");
    let content = page_content(&parse(&pdf), 1);
    // One cm: scale mm -> pt, flip y, move the origin to the trim's top-left corner,
    // which is 10 mm from the left and 188 mm above the bottom of the media box.
    let first = content.lines().next().expect("non-empty content");
    let numbers: Vec<f64> = first
        .strip_suffix(" cm")
        .expect("the stream starts with a cm operator")
        .split(' ')
        .map(|n| n.parse().expect("number"))
        .collect();
    assert_close(&numbers, &[pt(1.0), 0.0, 0.0, -pt(1.0), pt(10.0), pt(188.0)], 1e-3);
    // Afterwards coordinates are the scene's millimetres, untouched.
    assert!(content.contains("-3 -3 m\n233 -3 l\n233 181 l\n-3 181 l\nh\nf"), "{content}");
}

#[test]
fn header_version_per_standard() {
    for (standard, header) in [
        (PdfStandard::X1a2001, "%PDF-1.3"),
        (PdfStandard::X3_2002, "%PDF-1.3"),
        (PdfStandard::X4, "%PDF-1.6"),
        (PdfStandard::Plain, "%PDF-1.7"),
    ] {
        let pdf = write(&simple_doc(standard));
        assert!(pdf.starts_with(header.as_bytes()), "{standard:?}");
        // A binary comment follows so that transfer tools treat the file as binary.
        assert_eq!(pdf[8], b'\n');
        assert_eq!(pdf[9], b'%');
        assert!(pdf[10..14].iter().all(|b| *b >= 0x80), "{standard:?}");
        assert!(pdf.ends_with(b"%%EOF"));
        // PDF 1.3 has no object or cross-reference streams; we never use them.
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("\nxref\n"), "{standard:?}");
        assert!(!text.contains("/ObjStm") && !text.contains("/XRef"), "{standard:?}");
        assert_eq!(parse(&pdf).version, &header[5..]);
    }
}

#[test]
fn info_dictionary_carries_metadata_and_pdfx_keys() {
    for standard in ALL {
        let mut doc = simple_doc(standard);
        doc.info.subject = Some("Carte à imprimer".into());
        doc.info.keywords = vec!["cadeau".into(), "  ".into(), "eBook".into()];
        doc.info.custom = vec![
            ("EBC.SpineWidthMm".into(), "10".into()),
            ("EBC.FoldXMm".into(), "110,120".into()),
            ("EBC.Note".into(), "pliée à la main".into()),
        ];
        let pdf = parse(&write(&doc));
        let info = info(&pdf);

        assert_eq!(get_text(&pdf, info, "Title"), "Carte cadeau — L'Étranger");
        assert_eq!(get_text(&pdf, info, "Author"), "Albert Camus");
        assert_eq!(get_text(&pdf, info, "Subject"), "Carte à imprimer");
        assert_eq!(get_text(&pdf, info, "Keywords"), "cadeau, eBook");
        assert_eq!(get_text(&pdf, info, "Creator"), "ebc-pdf tests");
        assert!(get_text(&pdf, info, "Producer").starts_with("ebc-pdf "));
        assert_eq!(get_text(&pdf, info, "CreationDate"), "D:20261112083000Z");
        assert_eq!(get_text(&pdf, info, "ModDate"), "D:20261112083000Z");
        assert_eq!(get_name(&pdf, info, "Trapped"), "False");
        assert_eq!(get_text(&pdf, info, "EBC.SpineWidthMm"), "10");
        assert_eq!(get_text(&pdf, info, "EBC.FoldXMm"), "110,120");
        assert_eq!(get_text(&pdf, info, "EBC.Note"), "pliée à la main");

        let version = info.has(b"GTS_PDFXVersion").then(|| get_text(&pdf, info, "GTS_PDFXVersion"));
        let conformance =
            info.has(b"GTS_PDFXConformance").then(|| get_text(&pdf, info, "GTS_PDFXConformance"));
        let expected = match standard {
            PdfStandard::Plain => (None, None),
            PdfStandard::X1a2001 => (Some("PDF/X-1:2001"), Some("PDF/X-1a:2001")),
            PdfStandard::X3_2002 => (Some("PDF/X-3:2002"), None),
            PdfStandard::X4 => (Some("PDF/X-4"), None),
        };
        assert_eq!((version.as_deref(), conformance.as_deref()), expected, "{standard:?}");

        assert_eq!(get_text(&pdf, catalog(&pdf), "Lang"), "fr-FR");
    }
}

#[test]
fn plain_pdf_needs_no_metadata_at_all() {
    let mut doc = ebc_scene::Document { standard: PdfStandard::Plain, ..Default::default() };
    doc.pages.push(Page::new(Size::new(100.0, 100.0), 0.0, 0.0));
    let pdf = parse(&write(&doc));
    let info = info(&pdf);
    for key in ["Title", "Author", "Subject", "Keywords", "Creator", "CreationDate", "ModDate"] {
        assert!(!info.has(key.as_bytes()), "/{key} should be omitted");
    }
    assert!(info.has(b"Producer"));
    assert_eq!(get_name(&pdf, info, "Trapped"), "False");
    let catalog = catalog(&pdf);
    assert!(!catalog.has(b"OutputIntents"));
    assert!(!catalog.has(b"Lang"));
    assert!(!catalog.has(b"OCProperties"));
    assert!(!catalog.has(b"PageLabels"));
    let xmp = xmp(&pdf);
    assert!(!xmp.contains("dc:title") && !xmp.contains("CreateDate") && !xmp.contains("GTS_"));
}

#[test]
fn xmp_packet_is_uncompressed_and_agrees_with_the_info_dictionary() {
    for standard in ALL {
        let mut doc = simple_doc(standard);
        doc.info.subject = Some("Sujet <1> & \"2\"".into());
        doc.info.keywords = vec!["a".into(), "b".into()];
        let bytes = write(&doc);
        let pdf = parse(&bytes);

        let metadata = get(&pdf, catalog(&pdf), "Metadata").as_stream().expect("stream");
        assert_eq!(
            metadata.dict.get(b"Type").and_then(Object::as_name).expect("Type"),
            b"Metadata"
        );
        assert_eq!(
            metadata.dict.get(b"Subtype").and_then(Object::as_name).expect("Subtype"),
            b"XML"
        );
        assert!(!metadata.dict.has(b"Filter"), "the XMP packet must stay uncompressed");

        // ... so a plain byte scan of the file finds it, even in a compressed PDF.
        let needle = b"<?xpacket begin=";
        assert!(bytes.windows(needle.len()).any(|w| w == needle), "{standard:?}");

        let xmp = xmp(&pdf);
        assert!(xmp.contains("<rdf:li xml:lang=\"x-default\">Carte cadeau — L'Étranger</rdf:li>"));
        assert!(
            xmp.contains(
                "<dc:creator><rdf:Seq><rdf:li>Albert Camus</rdf:li></rdf:Seq></dc:creator>"
            )
        );
        assert!(xmp.contains("Sujet &lt;1&gt; &amp; &quot;2&quot;"));
        assert!(xmp.contains("<pdf:Keywords>a, b</pdf:Keywords>"));
        assert!(xmp.contains("<xmp:CreatorTool>ebc-pdf tests</xmp:CreatorTool>"));
        for tag in ["xmp:CreateDate", "xmp:ModifyDate", "xmp:MetadataDate"] {
            assert!(xmp.contains(&format!("<{tag}>2026-11-12T08:30:00Z</{tag}>")), "{tag}");
        }
        assert!(xmp.contains("<pdf:Trapped>False</pdf:Trapped>"));
        assert!(
            xmp.contains("<dc:language><rdf:Bag><rdf:li>fr-FR</rdf:li></rdf:Bag></dc:language>")
        );
        let producer = get_text(&pdf, info(&pdf), "Producer");
        assert!(xmp.contains(&format!("<pdf:Producer>{producer}</pdf:Producer>")));
        assert!(xmp.contains("<xmpMM:DocumentID>uuid:"));

        let marker = match standard {
            PdfStandard::Plain => None,
            PdfStandard::X1a2001 => {
                Some("<pdfx:GTS_PDFXVersion>PDF/X-1:2001</pdfx:GTS_PDFXVersion>")
            }
            PdfStandard::X3_2002 => {
                Some("<pdfx:GTS_PDFXVersion>PDF/X-3:2002</pdfx:GTS_PDFXVersion>")
            }
            PdfStandard::X4 => Some("<pdfxid:GTS_PDFXVersion>PDF/X-4</pdfxid:GTS_PDFXVersion>"),
        };
        match marker {
            Some(marker) => assert!(xmp.contains(marker), "{standard:?}: {xmp}"),
            None => assert!(!xmp.contains("GTS_PDFXVersion")),
        }
        assert_eq!(
            xmp.contains("<pdfx:GTS_PDFXConformance>PDF/X-1a:2001</pdfx:GTS_PDFXConformance>"),
            standard == PdfStandard::X1a2001
        );
    }
}

#[test]
fn output_intent_embeds_the_destination_profile() {
    for standard in ALL {
        let pdf = parse(&write(&simple_doc(standard)));
        let intents = get(&pdf, catalog(&pdf), "OutputIntents").as_array().expect("array");
        assert_eq!(intents.len(), 1);
        let intent = resolve(&pdf, &intents[0]).as_dict().expect("dictionary");
        assert_eq!(get_name(&pdf, intent, "Type"), "OutputIntent");
        assert_eq!(get_name(&pdf, intent, "S"), "GTS_PDFX");
        assert_eq!(get_text(&pdf, intent, "OutputConditionIdentifier"), "CGATS TR 001");
        assert_eq!(get_text(&pdf, intent, "RegistryName"), "http://www.color.org");
        assert_eq!(get_text(&pdf, intent, "Info"), "SWOP (CGATS TR 001) compatible");

        let profile = get(&pdf, intent, "DestOutputProfile");
        let dict = &profile.as_stream().expect("stream").dict;
        assert_eq!(get(&pdf, dict, "N").as_i64().expect("integer"), 4);
        assert_eq!(stream_data(profile), cmyk_profile().to_vec(), "profile is embedded verbatim");
    }
}

#[test]
fn registered_condition_without_profile_is_enough_for_x1a_and_x3() {
    for standard in [PdfStandard::X1a2001, PdfStandard::X3_2002] {
        let mut doc = simple_doc(standard);
        let intent = doc.output_intent.as_mut().expect("intent");
        intent.icc_profile = None;
        intent.condition_identifier = "FOGRA39".into();
        intent.info = String::new();
        let pdf = parse(&write(&doc));
        let intents = get(&pdf, catalog(&pdf), "OutputIntents").as_array().expect("array");
        let intent = resolve(&pdf, &intents[0]).as_dict().expect("dictionary");
        assert_eq!(get_text(&pdf, intent, "OutputConditionIdentifier"), "FOGRA39");
        assert!(!intent.has(b"DestOutputProfile"));
        assert!(!intent.has(b"Info"), "empty strings are omitted");
    }
}

#[test]
fn trailer_id_is_a_content_hash() {
    let id_of = |bytes: &[u8]| -> (Vec<u8>, Vec<u8>) {
        let pdf = parse(bytes);
        let id = pdf.trailer.get(b"ID").and_then(Object::as_array).expect("trailer /ID");
        assert_eq!(id.len(), 2);
        (id[0].as_str().expect("string").to_vec(), id[1].as_str().expect("string").to_vec())
    };

    let doc = simple_doc(PdfStandard::X4);
    let (a, b) = id_of(&write(&doc));
    assert_eq!(a.len(), 16);
    assert_eq!(a, b, "a freshly written file has two equal identifiers");
    assert_eq!(id_of(&write(&doc)).0, a, "same input, same identifier");

    // Any change to the content or the metadata changes the identifier.
    let mut recoloured = doc.clone();
    recoloured.pages[0].items[0] =
        Item::filled_rect(Rect::new(-3.0, -3.0, 236.0, 184.0), Color::cmyk(0.11, 0.0, 0.0, 0.0));
    let mut retitled = doc.clone();
    retitled.info.title.push('!');
    let mut redated = doc.clone();
    redated.info.created_unix = Some(CREATED + 1);
    let mut restandard = doc.clone();
    restandard.standard = PdfStandard::X3_2002;
    let ids: Vec<Vec<u8>> =
        [recoloured, retitled, redated, restandard].iter().map(|d| id_of(&write(d)).0).collect();
    for (i, id) in ids.iter().enumerate() {
        assert_ne!(*id, a, "variant {i}");
        for other in &ids[..i] {
            assert_ne!(id, other, "variant {i}");
        }
    }

    // The XMP document id follows the content as well.
    let xmp_id = |d: &ebc_scene::Document| {
        let xmp = xmp(&parse(&write(d)));
        let start = xmp.find("<xmpMM:DocumentID>").expect("document id") + 18;
        xmp[start..start + 41].to_owned()
    };
    assert_eq!(xmp_id(&doc), xmp_id(&doc.clone()));
    assert_ne!(xmp_id(&doc), xmp_id(&retitled_copy(&doc)));
}

fn retitled_copy(doc: &ebc_scene::Document) -> ebc_scene::Document {
    let mut copy = doc.clone();
    copy.info.title = "Autre titre".into();
    copy
}

#[test]
fn page_labels_name_the_sides_of_the_card() {
    let mut doc = simple_doc(PdfStandard::X4);
    doc.pages[0].label = Some("outside".into());
    doc.pages.push(Page::new(Size::new(230.0, 178.0), 3.0, 7.0));
    doc.pages.push(Page::new(Size::new(230.0, 178.0), 3.0, 7.0));
    doc.pages[2].label = Some("intérieur".into());
    let pdf = parse(&write(&doc));
    let labels = get_dict(&pdf, catalog(&pdf), "PageLabels");
    let nums = get(&pdf, labels, "Nums").as_array().expect("array");
    assert_eq!(nums.len(), 6);
    assert_eq!(nums[0].as_i64().expect("index"), 0);
    assert_eq!(get_text(&pdf, nums[1].as_dict().expect("dict"), "P"), "outside");
    assert_eq!(nums[2].as_i64().expect("index"), 1);
    let unnamed = nums[3].as_dict().expect("dict");
    assert_eq!(get_name(&pdf, unnamed, "S"), "D");
    assert_eq!(get(&pdf, unnamed, "St").as_i64().expect("integer"), 2);
    assert_eq!(get_text(&pdf, nums[5].as_dict().expect("dict"), "P"), "intérieur");

    // A run of unlabelled pages needs a single entry; they count on from its start.
    let mut doc = simple_doc(PdfStandard::Plain);
    doc.pages[0].label = Some("  cover  ".into());
    for _ in 0..5 {
        doc.pages.push(Page::new(Size::new(100.0, 100.0), 0.0, 0.0));
    }
    doc.pages[4].label = Some("\u{1}".into()); // nothing printable: not a label
    let pdf = parse(&write(&doc));
    let labels = get_dict(&pdf, catalog(&pdf), "PageLabels");
    let nums = get(&pdf, labels, "Nums").as_array().expect("array");
    assert_eq!(nums.len(), 4, "{nums:?}");
    assert_eq!(get_text(&pdf, nums[1].as_dict().expect("dict"), "P"), "cover");
    assert_eq!(nums[2].as_i64().expect("index"), 1);
    assert_eq!(get(&pdf, nums[3].as_dict().expect("dict"), "St").as_i64().expect("integer"), 2);
}

#[test]
fn compression_can_be_turned_off_for_inspection() {
    let doc = simple_doc(PdfStandard::X4);
    let compact =
        write_pdf(&doc, &WriteOptions { compress: true, keep_layers: false }).expect("ok");
    let readable =
        write_pdf(&doc, &WriteOptions { compress: false, keep_layers: false }).expect("ok");
    let text = String::from_utf8_lossy(&readable);
    assert!(text.contains("0.1 0 0 0 k"), "operators are readable");
    assert!(!text.contains("FlateDecode"));
    assert!(String::from_utf8_lossy(&compact).contains("FlateDecode"));
    // Both describe the same page.
    let (a, b) = (parse(&compact), parse(&readable));
    assert_eq!(
        page_content(&a, 1).split_whitespace().collect::<Vec<_>>(),
        page_content(&b, 1).split_whitespace().collect::<Vec<_>>()
    );
}

#[test]
fn default_options_compress_and_flatten() {
    assert_eq!(WriteOptions::default(), WriteOptions { compress: true, keep_layers: false });
}
