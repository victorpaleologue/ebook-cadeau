//! Font embedding, parsed back with `lopdf` and `ttf-parser`.

mod common;

use std::collections::BTreeSet;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Color, Document, FontId, Glyph, Item, MM_PER_PT, Page, PdfStandard, Point, Size};
use ttf_parser::{Face, GlyphId};

const READABLE: WriteOptions = WriteOptions { compress: false, keep_layers: false };

fn one_page(doc: &mut Document) -> &mut Page {
    doc.pages.push(Page::new(Size::new(200.0, 100.0), 0.0, 0.0));
    doc.pages.last_mut().expect("just pushed")
}

/// Bounding box of a glyph outline, or `None` for an empty glyph.
fn glyph_box(face: &Face<'_>, glyph: u16) -> Option<(i16, i16, i16, i16)> {
    face.glyph_bounding_box(GlyphId(glyph)).map(|r| (r.x_min, r.y_min, r.x_max, r.y_max))
}

#[test]
fn truetype_fonts_become_identity_h_cidfonttype2_subsets() {
    let text = "L'Étranger — cœur, façade, Noël";
    for standard in
        [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4]
    {
        for options in [READABLE, WriteOptions::default()] {
            let mut doc = base_doc(standard);
            let data = font("Spectral-Regular.ttf");
            let id = doc.add_font(data.clone(), 0);
            let origin = Point::new(12.0, 30.0);
            one_page(&mut doc).items.push(text_item(id, &data, text, 14.0, origin, Color::BLACK));
            let pdf = parse(&write_pdf(&doc, &options).expect("writable"));

            let info = font_info(&pdf, 1, "F0");
            assert_eq!(info.cid_subtype, "CIDFontType2");
            assert!(info.identity_cid_to_gid);
            assert_eq!(info.program_key, "FontFile2");
            assert_eq!(info.type0_name, info.base_font);
            let (tag, name) = info.base_font.split_once('+').expect("subset tag");
            assert!(tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()), "{tag}");
            assert_eq!(name, "Spectral-Regular");
            assert_eq!(get_name(&pdf, &info.descriptor, "FontName"), info.base_font);
            assert_eq!(
                get(&pdf, &info.program_dict, "Length1").as_i64().expect("integer"),
                i64::try_from(info.program.len()).expect("small"),
                "Length1 is the size of the decoded font program"
            );

            // The embedded program is a real, much smaller font.
            let original = Face::parse(&data, 0).expect("original parses");
            let subset = Face::parse(&info.program, 0).expect("subset parses");
            let distinct: BTreeSet<char> = text.chars().collect();
            assert!(info.program.len() < data.len() / 8, "{} bytes", info.program.len());
            assert!(usize::from(subset.number_of_glyphs()) > distinct.len(), "glyphs + .notdef");
            assert!(usize::from(subset.number_of_glyphs()) <= distinct.len() + 6);
            assert_eq!(subset.units_per_em(), original.units_per_em());

            // Walk the page like a PDF consumer: every shown CID must map back to the
            // right character, have the right width and the right outline.
            let content = get_stream_data(&pdf, page(&pdf, 1), "Contents");
            let glyphs = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
            assert_eq!(glyphs.len(), text.chars().count());
            let extracted: String =
                glyphs.iter().map(|g| info.to_unicode[&g.cid].as_str()).collect();
            assert_eq!(extracted, text, "{standard:?}");

            let upm = f64::from(original.units_per_em());
            for (glyph, c) in glyphs.iter().zip(text.chars()) {
                let original_id = original.glyph_index(c).expect("glyph").0;
                let advance =
                    f64::from(original.glyph_hor_advance(GlyphId(original_id)).expect("advance"));
                assert!((info.widths[&glyph.cid] - advance * 1000.0 / upm).abs() < 1e-3, "{c}");
                assert_eq!(
                    subset.glyph_hor_advance(GlyphId(glyph.cid)),
                    original.glyph_hor_advance(GlyphId(original_id)),
                    "W must agree with the embedded font program ({c})"
                );
                assert_eq!(glyph_box(&subset, glyph.cid), glyph_box(&original, original_id), "{c}");
                assert_eq!(glyph.font, "F0");
                assert!((glyph.size - 14.0 * MM_PER_PT).abs() < 1e-4);
                assert!((glyph.y - origin.y).abs() < 1e-6);
            }
            // Glyphs sit where the layout put them (font advances, to a micrometre).
            let (expected, _, _) = layout(&data, text, 14.0);
            for (glyph, wanted) in glyphs.iter().zip(&expected) {
                assert!((glyph.x - (origin.x + wanted.x)).abs() < 1e-3, "{glyph:?} vs {wanted:?}");
            }
        }
    }
}

#[test]
fn font_descriptor_comes_from_the_font_metrics() {
    let mut doc = base_doc(PdfStandard::X4);
    let fonts = [
        ("Spectral-Regular.ttf", false, false),
        ("Spectral-Italic.ttf", true, false),
        ("CourierPrime-Bold.ttf", false, true),
        ("AtkinsonHyperlegible-Bold.ttf", false, false),
    ];
    let mut items = Vec::new();
    for (i, (file, _, _)) in fonts.iter().enumerate() {
        let data = font(file);
        let id = doc.add_font(data.clone(), 0);
        items.push(text_item(
            id,
            &data,
            "Hamburgefonstiv",
            12.0,
            Point::new(10.0, 10.0 + 8.0 * i as f64),
            Color::BLACK,
        ));
    }
    one_page(&mut doc).items = items;
    let pdf = parse(&write_pdf(&doc, &READABLE).expect("writable"));

    let mut names = BTreeSet::new();
    for (i, (file, italic, monospaced)) in fonts.iter().enumerate() {
        let info = font_info(&pdf, 1, &format!("F{i}"));
        assert!(names.insert(info.base_font.clone()), "font names are unique");
        let data = font(file);
        let face = Face::parse(&data, 0).expect("parses");
        let scale = 1000.0 / f64::from(face.units_per_em());
        let d = &info.descriptor;

        let flags = get(&pdf, d, "Flags").as_i64().expect("integer");
        assert_eq!(flags & 4, 4, "{file}: symbolic (the font uses its own encoding)");
        assert_eq!(flags & 32, 0, "{file}: symbolic and non-symbolic are exclusive");
        assert_eq!(flags & 64 != 0, *italic, "{file}: italic flag");
        assert_eq!(flags & 1 != 0, *monospaced, "{file}: fixed-pitch flag");
        let angle = f64::from(get(&pdf, d, "ItalicAngle").as_float().expect("number"));
        assert_eq!(angle != 0.0, *italic, "{file}: italic angle {angle}");
        assert!((angle - f64::from(face.italic_angle())).abs() < 1e-3);

        let bbox = face.global_bounding_box();
        assert_close(
            &get_numbers(&pdf, d, "FontBBox"),
            &[
                f64::from(bbox.x_min) * scale,
                f64::from(bbox.y_min) * scale,
                f64::from(bbox.x_max) * scale,
                f64::from(bbox.y_max) * scale,
            ],
            0.01,
        );
        let number = |key: &str| f64::from(get(&pdf, d, key).as_float().expect("number"));
        assert!((number("Ascent") - f64::from(face.ascender()) * scale).abs() < 0.01);
        assert!((number("Descent") - f64::from(face.descender()) * scale).abs() < 0.01);
        assert!(number("Descent") < 0.0 && number("Ascent") > 0.0);
        let cap = f64::from(face.capital_height().expect("cap height")) * scale;
        assert!((number("CapHeight") - cap).abs() < 0.01);
        assert!(number("StemV") > 0.0);
        if file.contains("Bold") {
            assert!(number("StemV") > 150.0, "{file}: bold fonts have heavier stems");
        }
    }
}

#[test]
fn cff_fonts_become_cidfonttype0_with_a_bare_cff_program() {
    let text = "let côté = \"Noël\"; // 0O1lI";
    for standard in [PdfStandard::X1a2001, PdfStandard::X4] {
        let mut doc = base_doc(standard);
        let data = fixture("SourceCodePro-Regular.otf");
        let id = doc.add_font(data.clone(), 0);
        one_page(&mut doc).items.push(text_item(
            id,
            &data,
            text,
            10.0,
            Point::new(10.0, 20.0),
            Color::BLACK,
        ));
        let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("writable"));

        let info = font_info(&pdf, 1, "F0");
        assert_eq!(info.cid_subtype, "CIDFontType0");
        assert!(!info.identity_cid_to_gid, "CIDToGIDMap only exists for TrueType outlines");
        assert_eq!(info.program_key, "FontFile3");
        assert_eq!(get_name(&pdf, &info.program_dict, "Subtype"), "CIDFontType0C");
        assert!(!info.program_dict.has(b"Length1"));
        let (tag, name) = info.base_font.split_once('+').expect("subset tag");
        assert_eq!((tag.len(), name), (6, "SourceCodePro-Regular"));
        assert_eq!(info.type0_name, format!("{}-Identity-H", info.base_font));

        // The program is a bare CFF table (not an OpenType wrapper), CID-keyed, with only
        // the glyphs we use.
        assert_eq!(&info.program[..2], &[1, 0], "CFF header major/minor version");
        let cff = ttf_parser::cff::Table::parse(&info.program).expect("CFF parses");
        let distinct: BTreeSet<char> = text.chars().collect();
        assert_eq!(usize::from(cff.number_of_glyphs()), distinct.len() + 1);
        assert!(info.program.len() < 8_000, "{} bytes", info.program.len());

        let original = Face::parse(&data, 0).expect("original parses");
        let content = get_stream_data(&pdf, page(&pdf, 1), "Contents");
        let glyphs = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
        let extracted: String = glyphs.iter().map(|g| info.to_unicode[&g.cid].as_str()).collect();
        assert_eq!(extracted, text);
        struct Sink;
        impl ttf_parser::OutlineBuilder for Sink {
            fn move_to(&mut self, _: f32, _: f32) {}
            fn line_to(&mut self, _: f32, _: f32) {}
            fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
            fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
            fn close(&mut self) {}
        }
        for (glyph, c) in glyphs.iter().zip(text.chars()) {
            let original_id = original.glyph_index(c).expect("glyph");
            // In a CID-keyed CFF the charset maps glyph index -> CID; the subsetter makes
            // them equal, which is what `Identity-H` relies on.
            assert_eq!(cff.glyph_cid(GlyphId(glyph.cid)), Some(glyph.cid), "{c}");
            let embedded = cff.outline(GlyphId(glyph.cid), &mut Sink).ok();
            let expected = original.glyph_bounding_box(original_id);
            assert_eq!(
                embedded.map(|r| (r.x_min, r.y_min, r.x_max, r.y_max)),
                expected.map(|r| (r.x_min, r.y_min, r.x_max, r.y_max)),
                "outline of {c:?}"
            );
            let advance = f64::from(original.glyph_hor_advance(original_id).expect("advance"));
            assert!((info.widths[&glyph.cid] - advance).abs() < 1e-3, "upm is 1000");
        }
    }
}

#[test]
fn fonts_that_forbid_subsetting_are_embedded_whole() {
    let text = "Été 2026";
    // TrueType: the original file goes in untouched and CIDs are the original glyph ids.
    let original = font("AtkinsonHyperlegible-Regular.ttf");
    let whole_only = with_fs_type(&original, 0x0100);
    let mut doc = base_doc(PdfStandard::X4);
    let id = doc.add_font(whole_only.clone(), 0);
    one_page(&mut doc).items.push(text_item(
        id,
        &whole_only,
        text,
        12.0,
        Point::new(10.0, 20.0),
        Color::BLACK,
    ));
    let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("writable"));
    let info = font_info(&pdf, 1, "F0");
    assert_eq!(info.base_font, "AtkinsonHyperlegible-Regular", "no subset tag");
    assert_eq!(info.program, whole_only.to_vec(), "the complete font file is embedded");
    let face = Face::parse(&whole_only, 0).expect("parses");
    let content = get_stream_data(&pdf, page(&pdf, 1), "Contents");
    let glyphs = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
    let expected: Vec<u16> = text.chars().map(|c| face.glyph_index(c).expect("glyph").0).collect();
    assert_eq!(glyphs.iter().map(|g| g.cid).collect::<Vec<_>>(), expected);
    let extracted: String = glyphs.iter().map(|g| info.to_unicode[&g.cid].as_str()).collect();
    assert_eq!(extracted, text);

    // The same font without the flag is subset.
    let mut doc = base_doc(PdfStandard::X4);
    let id = doc.add_font(original.clone(), 0);
    one_page(&mut doc).items.push(text_item(
        id,
        &original,
        text,
        12.0,
        Point::new(10.0, 20.0),
        Color::BLACK,
    ));
    let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("writable"));
    let subset = font_info(&pdf, 1, "F0");
    assert!(subset.base_font.contains('+'));
    assert!(subset.program.len() < info.program.len() / 4);

    // CFF: every glyph is kept (the table is re-serialised as a CID-keyed font).
    let original = fixture("SourceCodePro-Regular.otf");
    let whole_only = with_fs_type(&original, 0x0100);
    let mut doc = base_doc(PdfStandard::X4);
    let id = doc.add_font(whole_only.clone(), 0);
    one_page(&mut doc).items.push(text_item(
        id,
        &whole_only,
        text,
        12.0,
        Point::new(10.0, 20.0),
        Color::BLACK,
    ));
    let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("writable"));
    let info = font_info(&pdf, 1, "F0");
    assert_eq!(info.base_font, "SourceCodePro-Regular");
    let face = Face::parse(&whole_only, 0).expect("parses");
    let cff = ttf_parser::cff::Table::parse(&info.program).expect("CFF parses");
    assert_eq!(cff.number_of_glyphs(), face.number_of_glyphs());
    let content = get_stream_data(&pdf, page(&pdf, 1), "Contents");
    let glyphs = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
    let expected: Vec<u16> = text.chars().map(|c| face.glyph_index(c).expect("glyph").0).collect();
    assert_eq!(glyphs.iter().map(|g| g.cid).collect::<Vec<_>>(), expected);
}

#[test]
fn glyphs_land_exactly_where_the_run_puts_them() {
    let data = font("AtkinsonHyperlegible-Regular.ttf");
    let face = Face::parse(&data, 0).expect("parses");
    let gid = |c: char| face.glyph_index(c).expect("glyph").0;
    let mut doc = base_doc(PdfStandard::X4);
    let id = doc.add_font(data.clone(), 0);
    let origin = Point::new(20.0, 40.0);
    // Letter-spaced, kerned backwards, raised (superscript), lowered, a big jump and two
    // glyphs on top of each other: nothing a simple "show string" could express.
    let run = vec![
        Glyph { id: gid('A'), x: 0.0, y: 0.0 },
        Glyph { id: gid('V'), x: 2.4, y: 0.0 },
        Glyph { id: gid('A'), x: 4.1, y: 0.0 },
        Glyph { id: gid('2'), x: 7.0, y: -1.5 },
        Glyph { id: gid('x'), x: 8.5, y: 0.0 },
        Glyph { id: gid('y'), x: 10.25, y: 0.75 },
        Glyph { id: gid('z'), x: 150.0, y: 0.0 },
        Glyph { id: gid('o'), x: 3.0, y: 0.0 },
        Glyph { id: gid('o'), x: 3.0, y: 0.0 },
    ];
    let glyph_text: Vec<String> = "AVA2xyzoo".chars().map(String::from).collect();
    one_page(&mut doc).items.push(Item::Glyphs {
        font: id,
        size_pt: 9.5,
        color: Color::BLACK,
        origin,
        glyphs: run.clone(),
        glyph_text,
    });
    let pdf = parse(&write_pdf(&doc, &READABLE).expect("writable"));
    let info = font_info(&pdf, 1, "F0");
    let content = get_stream_data(&pdf, page(&pdf, 1), "Contents");
    let placed = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
    assert_eq!(placed.len(), run.len());
    for (got, want) in placed.iter().zip(&run) {
        assert!((got.x - (origin.x + want.x)).abs() < 3e-4, "{got:?} vs {want:?}");
        assert!((got.y - (origin.y + want.y)).abs() < 1e-9, "{got:?} vs {want:?}");
        assert_eq!(info.to_unicode[&got.cid], face_char(&face, want.id));
    }
    // The text matrix flips text upright on the y-down page.
    let text = String::from_utf8_lossy(&content);
    assert!(text.contains("1 0 0 -1 20 40 Tm"), "{text}");
    assert!(text.contains("1 0 0 -1 27 38.5 Tm"), "raised glyph starts a new line: {text}");
}

fn face_char(face: &Face<'_>, glyph: u16) -> String {
    "AVA2xyzo"
        .chars()
        .find(|c| face.glyph_index(*c).map(|g| g.0) == Some(glyph))
        .map(String::from)
        .expect("glyph of the test run")
}

#[test]
fn naturally_spaced_text_needs_no_adjustments() {
    // With the font's own advances the whole line is one string: compact content, and a
    // sign that the widths we declare are the ones the layout used.
    for file in [
        "Spectral-Regular.ttf",
        "AtkinsonHyperlegible-Regular.ttf",
        "CourierPrime-Bold.ttf",
        "BebasNeue-Regular.ttf",
    ] {
        let data = font(file);
        let mut doc = base_doc(PdfStandard::X4);
        let id = doc.add_font(data.clone(), 0);
        let text = "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG 0123456789";
        one_page(&mut doc).items.push(text_item(
            id,
            &data,
            text,
            11.0,
            Point::new(5.0, 20.0),
            Color::BLACK,
        ));
        let pdf = parse(&write_pdf(&doc, &READABLE).expect("writable"));
        let content = page_content(&pdf, 1);
        let tj: Vec<&str> = content.lines().filter(|l| l.ends_with(" TJ")).collect();
        assert_eq!(tj.len(), 1, "{file}: {content}");
        let operations =
            lopdf::content::Content::decode(content.as_bytes()).expect("parses").operations;
        let array = operations
            .iter()
            .find(|op| op.operator == "TJ")
            .and_then(|op| op.operands[0].as_array().ok())
            .expect("TJ array");
        assert_eq!(array.len(), 1, "{file}: one string, no kerning numbers: {}", tj[0]);
    }
}

#[test]
fn to_unicode_handles_ligatures_astral_characters_and_conflicts() {
    let data = font("Spectral-Regular.ttf");
    let face = Face::parse(&data, 0).expect("parses");
    let gid = |c: char| face.glyph_index(c).expect("glyph").0;
    let mut doc = base_doc(PdfStandard::X4);
    let id = doc.add_font(data.clone(), 0);
    let glyphs = vec![
        Glyph { id: gid('f'), x: 0.0, y: 0.0 }, // stands for a whole ligature
        Glyph { id: gid('i'), x: 3.0, y: 0.0 }, // trailing component: no text
        Glyph { id: gid('G'), x: 6.0, y: 0.0 }, // astral character (surrogate pair)
        Glyph { id: gid(' '), x: 9.0, y: 0.0 }, // first seen as a no-break space ...
        Glyph { id: gid(' '), x: 12.0, y: 0.0 }, // ... then as a space: the first wins
        Glyph { id: gid('i'), x: 15.0, y: 0.0 }, // now with text: fills the gap
        Glyph { id: gid('x'), x: 18.0, y: 0.0 }, // never has text
    ];
    let glyph_text: Vec<String> =
        ["ffi", "", "🎁", "\u{a0}", " ", "i"].iter().map(|s| (*s).to_owned()).collect();
    // `glyph_text` is one entry short on purpose: missing entries mean "no text".
    one_page(&mut doc).items.push(Item::Glyphs {
        font: id,
        size_pt: 10.0,
        color: Color::BLACK,
        origin: Point::new(10.0, 10.0),
        glyphs,
        glyph_text,
    });
    let bytes = write_pdf(&doc, &READABLE).expect("writable");
    let pdf = parse(&bytes);
    let info = font_info(&pdf, 1, "F0");
    let content = get_stream_data(&pdf, page(&pdf, 1), "Contents");
    let placed = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
    let texts: Vec<Option<&str>> =
        placed.iter().map(|g| info.to_unicode.get(&g.cid).map(String::as_str)).collect();
    assert_eq!(
        texts,
        [Some("ffi"), Some("i"), Some("🎁"), Some("\u{a0}"), Some("\u{a0}"), Some("i"), None]
    );
    // The astral character is written as a UTF-16 surrogate pair.
    assert!(String::from_utf8_lossy(&bytes).contains("<D83CDF81>"));
}

#[test]
fn fonts_are_shared_between_pages_and_listed_only_where_used() {
    let spectral = font("Spectral-Regular.ttf");
    let atkinson = font("AtkinsonHyperlegible-Regular.ttf");
    let mut doc = base_doc(PdfStandard::X4);
    let unused = doc.add_font(font("BebasNeue-Regular.ttf"), 0);
    let serif = doc.add_font(spectral.clone(), 0);
    let sans = doc.add_font(atkinson.clone(), 0);
    assert_eq!((unused, serif, sans), (FontId(0), FontId(1), FontId(2)));
    let p1 = one_page(&mut doc);
    p1.items.push(text_item(serif, &spectral, "abc", 12.0, Point::new(10.0, 10.0), Color::BLACK));
    let p2 = one_page(&mut doc);
    p2.items.push(text_item(serif, &spectral, "cde", 12.0, Point::new(10.0, 10.0), Color::BLACK));
    p2.items.push(text_item(sans, &atkinson, "xyz", 12.0, Point::new(10.0, 20.0), Color::BLACK));
    // An empty run does not pull its font in.
    p2.items.push(Item::Glyphs {
        font: unused,
        size_pt: 12.0,
        color: Color::BLACK,
        origin: Point::new(0.0, 0.0),
        glyphs: vec![],
        glyph_text: vec![],
    });
    let _ = one_page(&mut doc);

    let bytes = write_pdf(&doc, &WriteOptions::default()).expect("writable");
    let pdf = parse(&bytes);
    let names = |n: u32| -> Vec<String> {
        resources(&pdf, n, "Font")
            .map(|d| d.iter().map(|(k, _)| String::from_utf8_lossy(k).into_owned()).collect())
            .unwrap_or_default()
    };
    assert_eq!(names(1), ["F0"]);
    assert_eq!(names(2), ["F0", "F1"]);
    assert!(names(3).is_empty());

    // Both pages point at the same font object, which holds the union of the glyphs.
    let reference = |n: u32| {
        resources(&pdf, n, "Font")
            .expect("fonts")
            .get(b"F0")
            .expect("F0")
            .as_reference()
            .expect("indirect")
    };
    assert_eq!(reference(1), reference(2));
    let shared = font_info(&pdf, 1, "F0");
    assert!(shared.base_font.ends_with("+Spectral-Regular"));
    let texts: BTreeSet<&str> = shared.to_unicode.values().map(String::as_str).collect();
    assert_eq!(texts, BTreeSet::from(["a", "b", "c", "d", "e"]));
    assert!(font_info(&pdf, 2, "F1").base_font.ends_with("+AtkinsonHyperlegible-Regular"));

    // The unused font is not embedded at all.
    let embedded = pdf
        .objects
        .values()
        .filter(|o| o.as_dict().is_ok_and(|d| d.has_type(b"FontDescriptor")))
        .count();
    assert_eq!(embedded, 2);
    assert!(!String::from_utf8_lossy(&bytes).contains("BebasNeue"));
}

#[test]
fn very_long_runs_are_split_into_arrays_and_strings_old_readers_accept() {
    let data = font("CourierPrime-Bold.ttf");
    let face = Face::parse(&data, 0).expect("parses");
    let a = face.glyph_index('a').expect("glyph").0;
    let b = face.glyph_index('b').expect("glyph").0;
    let mut doc = base_doc(PdfStandard::X1a2001);
    let id = doc.add_font(data.clone(), 0);
    // 30 000 glyphs, every one of them kerned: far more than one TJ array may hold.
    let kerned: Vec<Glyph> = (0..30_000u32)
        .map(|i| Glyph { id: if i % 2 == 0 { a } else { b }, x: f64::from(i) * 0.001, y: 0.0 })
        .collect();
    // 30 000 glyphs at their natural advance: far more than one string may hold.
    let advance = f64::from(face.glyph_hor_advance(ttf_parser::GlyphId(a)).expect("advance"))
        * 2.0
        * MM_PER_PT
        / f64::from(face.units_per_em());
    let natural: Vec<Glyph> =
        (0..30_000u32).map(|i| Glyph { id: a, x: f64::from(i) * advance, y: 0.0 }).collect();
    let page = one_page(&mut doc);
    for (glyphs, y) in [(kerned.clone(), 10.0), (natural.clone(), 20.0)] {
        page.items.push(Item::Glyphs {
            font: id,
            size_pt: 2.0,
            color: Color::BLACK,
            origin: Point::new(0.0, y),
            glyphs,
            glyph_text: vec![],
        });
    }
    let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("writable"));
    let info = font_info(&pdf, 1, "F0");
    assert!(info.to_unicode.is_empty(), "no text was supplied");
    let content = get_stream_data(&pdf, page_dict(&pdf), "Contents");
    // `placed_glyphs` asserts the array and string limits while it walks the stream.
    let placed = placed_glyphs(&content, &|_, cid| info.widths[&cid]);
    assert_eq!(placed.len(), 60_000);
    // Each run is cut into short segments that start at an absolute position.
    let text = String::from_utf8_lossy(&content);
    assert_eq!(text.matches("Tm").count(), 60, "1000 glyphs per segment");
    assert_eq!(text.matches("TJ").count(), 60);
    for (got, want) in placed.iter().zip(kerned.iter().chain(&natural)) {
        // Absolute positions are single-precision numbers in the file, so the tolerance
        // grows with the distance from the origin (these runs are up to 12 m long).
        assert!((got.x - want.x).abs() < 1e-4 + want.x * 1e-7, "{got:?} vs {want:?}");
    }
}

fn page_dict(pdf: &lopdf::Document) -> &Dictionary {
    page(pdf, 1)
}
