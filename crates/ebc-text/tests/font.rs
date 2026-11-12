//! Font loading: names, licence, embedding permission, metrics, coverage, variations.

mod common;

use std::sync::Arc;

use common::{NameRecord, SyntheticFont, close};
use ebc_scene::Point;
use ebc_text::{
    EmbeddingFlags, EmbeddingPermission, Font, FontError, OutlineFormat, ShapeOptions, shape,
};

fn synthetic(configure: impl FnOnce(&mut SyntheticFont)) -> Font {
    let mut spec = SyntheticFont::new();
    configure(&mut spec);
    Font::from_bytes(spec.build(), 0).expect("synthetic font parses")
}

// --- names and licence -------------------------------------------------------------

#[test]
fn bundled_fonts_report_their_names() {
    let expected = [
        (common::ATKINSON, "Atkinson Hyperlegible", "Regular", "AtkinsonHyperlegible-Regular"),
        (common::ATKINSON_BOLD, "Atkinson Hyperlegible", "Bold", "AtkinsonHyperlegible-Bold"),
        (common::ATKINSON_ITALIC, "Atkinson Hyperlegible", "Italic", "AtkinsonHyperlegible-Italic"),
        (common::SPECTRAL, "Spectral", "Regular", "Spectral-Regular"),
        (common::SPECTRAL_ITALIC, "Spectral", "Italic", "Spectral-Italic"),
        // The legacy family (name ID 1) of this face is "Spectral SemiBold": the
        // typographic family (ID 16) must win.
        (common::SPECTRAL_SEMIBOLD, "Spectral", "SemiBold", "Spectral-SemiBold"),
        (common::BEBAS, "Bebas Neue", "Regular", "BebasNeue-Regular"),
        (common::COURIER_BOLD, "Courier Prime", "Bold", "CourierPrime-Bold"),
    ];
    for (bytes, family, style, postscript) in expected {
        let font = common::font(bytes);
        assert_eq!(font.family_name(), family);
        assert_eq!(font.style_name(), style);
        assert_eq!(font.full_name(), format!("{family} {style}"));
        assert_eq!(font.postscript_name().as_deref(), Some(postscript));
        assert_eq!(font.face_index(), 0);
    }
}

#[test]
fn bundled_fonts_carry_the_open_font_licence_and_may_be_embedded() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let licence = font.licence_description().unwrap_or_default();
        assert!(licence.contains("SIL Open Font License"), "{label}: {licence:?}");
        let url = font.licence_url().unwrap_or_default();
        assert!(
            url.contains("scripts.sil.org/OFL") || url.contains("openfontlicense.org"),
            "{label}: {url:?}"
        );
        assert_eq!(font.fs_type(), Some(0), "{label}");
        assert_eq!(font.embedding(), EmbeddingPermission::Installable, "{label}");
        assert_eq!(font.embedding_flags(), EmbeddingFlags::default(), "{label}");
        assert!(font.allows_bundling() && font.allows_subset_embedding(), "{label}");
        assert!(font.allows_outlines(), "{label}");
        assert!(font.version().is_some_and(|v| v.starts_with("Version")), "{label}");
        assert_eq!(font.outline_format(), OutlineFormat::TrueType, "{label}");
        assert!(!font.is_variable(), "{label}");
        assert!(font.axes().is_empty() && font.variations().is_empty(), "{label}");
    }
}

#[test]
fn style_attributes_come_from_the_font() {
    assert_eq!(common::font(common::ATKINSON).weight(), 400);
    assert_eq!(common::font(common::ATKINSON_BOLD).weight(), 700);
    assert_eq!(common::font(common::SPECTRAL_SEMIBOLD).weight(), 600);
    assert!(common::font(common::ATKINSON_ITALIC).is_italic());
    assert!(common::font(common::SPECTRAL_ITALIC).is_italic());
    assert!(!common::font(common::SPECTRAL).is_italic());
    assert!(common::font(common::COURIER_BOLD).is_monospaced());
    assert!(!common::font(common::BEBAS).is_monospaced());
}

#[test]
fn any_name_record_can_be_read() {
    let font = common::atkinson();
    let copyright = font.name(0).expect("copyright notice");
    assert!(copyright.contains("Copyright"), "{copyright:?}");
    assert!(font.name(5).expect("version string").starts_with("Version"));
    assert_eq!(font.name(1).as_deref(), Some("Atkinson Hyperlegible"));
    assert_eq!(font.name(999), None);
    assert_eq!(font.name(u16::MAX), None);
}

#[test]
fn typographic_names_win_over_legacy_names_and_full_name_is_synthesised() {
    let rich = synthetic(|f| {
        f.names = vec![
            NameRecord::windows(1, "Legacy Fam Display"),
            NameRecord::windows(2, "Bold"),
            NameRecord::windows(4, "Typo Fam Display Bold"),
            NameRecord::windows(6, "TypoFam-DisplayBold"),
            NameRecord::windows(13, "You may do anything."),
            NameRecord::windows(14, "https://licence.example/free"),
            NameRecord::windows(16, "Typo Fam"),
            NameRecord::windows(17, "Display Bold"),
        ];
    });
    assert_eq!(rich.family_name(), "Typo Fam");
    assert_eq!(rich.style_name(), "Display Bold");
    assert_eq!(rich.full_name(), "Typo Fam Display Bold");
    assert_eq!(rich.postscript_name().as_deref(), Some("TypoFam-DisplayBold"));
    assert_eq!(rich.licence_description().as_deref(), Some("You may do anything."));
    assert_eq!(rich.licence_url().as_deref(), Some("https://licence.example/free"));

    let legacy = synthetic(|f| {
        f.names = vec![NameRecord::windows(1, "Old Sans"), NameRecord::windows(2, "Bold Italic")];
    });
    assert_eq!(legacy.family_name(), "Old Sans");
    assert_eq!(legacy.style_name(), "Bold Italic");
    assert_eq!(legacy.full_name(), "Old Sans Bold Italic");
    assert_eq!(legacy.postscript_name(), None);
    assert_eq!(legacy.licence_description(), None);
    assert_eq!(legacy.licence_url(), None);

    let regular = synthetic(|f| {
        f.names = vec![NameRecord::windows(1, "Old Sans"), NameRecord::windows(2, "Regular")];
    });
    assert_eq!(regular.full_name(), "Old Sans", "\"Regular\" is not repeated in the full name");

    let nameless = synthetic(|_| {});
    assert_eq!(nameless.family_name(), "");
    assert_eq!(nameless.style_name(), "");
    assert_eq!(nameless.full_name(), "");
    assert_eq!(nameless.postscript_name(), None);
}

#[test]
fn name_records_are_chosen_by_platform_and_language() {
    // Macintosh Roman only: decoded (0x8E is "é" in Mac Roman).
    let mac = synthetic(|f| f.names = vec![NameRecord::mac(1, b"Caf\x8e Sans")]);
    assert_eq!(mac.family_name(), "Café Sans");

    // Windows US English beats French and Macintosh, whatever the record order.
    let french = NameRecord {
        platform: 3,
        encoding: 1,
        language: 0x040C,
        name_id: 1,
        bytes: "Élégante".encode_utf16().flat_map(u16::to_be_bytes).collect(),
    };
    let mixed = synthetic(|f| {
        f.names =
            vec![NameRecord::mac(1, b"Mac Name"), french, NameRecord::windows(1, "English Name")];
    });
    assert_eq!(mixed.family_name(), "English Name");

    // Only a French record: better than nothing.
    let only_french = synthetic(|f| {
        f.names = vec![NameRecord {
            platform: 3,
            encoding: 1,
            language: 0x040C,
            name_id: 1,
            bytes: "Élégante".encode_utf16().flat_map(u16::to_be_bytes).collect(),
        }];
    });
    assert_eq!(only_french.family_name(), "Élégante");
}

#[test]
fn hostile_name_records_are_cleaned_or_skipped() {
    // NULs, surrounding white space, an unpaired surrogate.
    let dirty = synthetic(|f| {
        let mut bytes: Vec<u8> = " A\0B ".encode_utf16().flat_map(u16::to_be_bytes).collect();
        bytes.extend_from_slice(&[0xD8, 0x00]); // lone high surrogate
        f.names =
            vec![NameRecord { platform: 3, encoding: 1, language: 0x0409, name_id: 1, bytes }];
    });
    assert_eq!(dirty.family_name(), "AB \u{fffd}");

    // Odd byte length is not UTF-16: the record is skipped, the Mac record is used.
    let odd = synthetic(|f| {
        f.names = vec![
            NameRecord {
                platform: 3,
                encoding: 1,
                language: 0x0409,
                name_id: 1,
                bytes: vec![0, 65, 0],
            },
            NameRecord::mac(1, b"Fallback"),
        ];
    });
    assert_eq!(odd.family_name(), "Fallback");

    // Empty and blank names count as absent.
    let blank = synthetic(|f| {
        f.names = vec![NameRecord::windows(1, "   "), NameRecord::windows(6, "")];
    });
    assert_eq!(blank.family_name(), "");
    assert_eq!(blank.postscript_name(), None);

    // A record pointing outside the string storage must not hide the valid ones after it.
    let mut spec = SyntheticFont::new();
    spec.names = vec![NameRecord::windows(1, "First"), NameRecord::windows(6, "Second-PS")];
    let mut bytes = spec.build();
    let (name_table, _) = common::find_table(&bytes, b"name").expect("name table");
    // First record: set its string length to 0xFFFF.
    bytes[name_table + 6 + 8..name_table + 6 + 10].copy_from_slice(&[0xFF, 0xFF]);
    let font = Font::from_bytes(bytes, 0).expect("still a font");
    assert_eq!(font.family_name(), "");
    assert_eq!(font.postscript_name().as_deref(), Some("Second-PS"));
}

// --- embedding permission ----------------------------------------------------------

#[test]
fn fs_type_is_read_from_the_os2_table() {
    use EmbeddingPermission as P;
    // The table of docs/research/08-legal-fonts-and-genai.md §4.3, row by row:
    // (fsType, permission, no_subsetting, bitmap_only,
    //  may be bundled, may be subset-embedded, user font may be outlined)
    let cases = [
        (0x0000, P::Installable, false, false, true, true, true),
        (0x0004, P::PreviewAndPrint, false, false, false, true, true),
        (0x0008, P::Editable, false, false, false, true, true),
        (0x0002, P::Restricted, false, false, false, false, false),
        (0x0100, P::Installable, true, false, false, false, true),
        (0x0104, P::PreviewAndPrint, true, false, false, false, true),
        (0x0200, P::Installable, false, true, false, false, false),
        (0x0308, P::Editable, true, true, false, false, false),
        (0x0302, P::Restricted, true, true, false, false, false),
    ];
    for (fs_type, permission, no_subsetting, bitmap_only, bundle, subset, outline) in cases {
        let patched = common::with_fs_type(common::ATKINSON, fs_type, None);
        let font = Font::from_bytes(patched, 0).expect("patched font parses");
        assert_eq!(font.fs_type(), Some(fs_type));
        assert_eq!(font.embedding(), permission, "fsType {fs_type:#06x}");
        assert_eq!(
            font.embedding_flags(),
            EmbeddingFlags { no_subsetting, bitmap_only },
            "fsType {fs_type:#06x}"
        );
        assert_eq!(font.allows_bundling(), bundle, "fsType {fs_type:#06x}");
        assert_eq!(font.allows_subset_embedding(), subset, "fsType {fs_type:#06x}");
        assert_eq!(font.allows_outlines(), outline, "fsType {fs_type:#06x}");
        assert_eq!(font.embedding().allows_embedding(), permission != P::Restricted);
        // The rest of the font is untouched.
        assert_eq!(font.family_name(), "Atkinson Hyperlegible");
    }
}

#[test]
fn conflicting_fs_type_bits_are_read_by_table_version() {
    use EmbeddingPermission as P;
    // Restricted + preview&print set together.
    let modern = common::with_fs_type(common::ATKINSON, 0x0006, Some(4));
    assert_eq!(Font::from_bytes(modern, 0).expect("font").embedding(), P::Restricted);
    let old = common::with_fs_type(common::ATKINSON, 0x0006, Some(2));
    assert_eq!(Font::from_bytes(old, 0).expect("font").embedding(), P::PreviewAndPrint);

    // The same through synthetic OS/2 tables of every version.
    for version in 0..=5u16 {
        let font = synthetic(|f| f.os2 = Some((version, 0x000A, true)));
        let expected = if version <= 2 { P::Editable } else { P::Restricted };
        assert_eq!(font.embedding(), expected, "OS/2 version {version}");
        let plain = synthetic(|f| f.os2 = Some((version, 0x0004, true)));
        assert_eq!(plain.embedding(), P::PreviewAndPrint, "OS/2 version {version}");
    }
}

#[test]
fn a_font_without_os2_has_unknown_permissions_and_is_refused() {
    // Last row of the table: no OS/2 table → unknown → reject / refuse.
    for font in [
        synthetic(|_| {}),
        Font::from_bytes(common::without_table(common::SPECTRAL, b"OS/2"), 0)
            .expect("font without OS/2"),
    ] {
        assert_eq!(font.fs_type(), None, "\"unknown\" can be told from \"restricted\"");
        assert_eq!(font.embedding(), EmbeddingPermission::Restricted);
        assert_eq!(font.embedding_flags(), EmbeddingFlags::default());
        assert!(!font.allows_bundling());
        assert!(!font.allows_subset_embedding());
        assert!(!font.allows_outlines());
        assert_eq!(font.weight(), 400);
    }
    // The same synthetic font with an OS/2 table that says "installable" passes.
    let declared = synthetic(|f| f.os2 = Some((4, 0, true)));
    assert_eq!(declared.embedding(), EmbeddingPermission::Installable);
    assert!(declared.allows_bundling() && declared.allows_subset_embedding());
    assert!(declared.allows_outlines());
}

#[test]
fn a_restriction_in_a_malformed_os2_table_is_still_honoured() {
    // A 78-byte (version 0) table that claims to be version 4: too short to be parsed
    // as such, but the fsType field is there and says "restricted".
    let mut spec = SyntheticFont::new();
    spec.os2 = Some((0, 0x0002, false));
    let mut bytes = spec.build();
    let (os2, length) = common::find_table(&bytes, b"OS/2").expect("OS/2");
    assert_eq!(length, 78);
    bytes[os2..os2 + 2].copy_from_slice(&4u16.to_be_bytes());
    let font = Font::from_bytes(bytes, 0).expect("font");
    assert_eq!(font.fs_type(), Some(0x0002));
    assert_eq!(font.embedding(), EmbeddingPermission::Restricted);
    assert!(!font.allows_subset_embedding() && !font.allows_outlines());
}

// --- metrics -----------------------------------------------------------------------

#[test]
fn metrics_of_bundled_fonts_are_sane_and_convert_to_millimetres() {
    for (label, bytes) in common::ALL {
        let m = common::font(bytes).metrics();
        assert!(m.units_per_em == 1000 || m.units_per_em == 2048, "{label}: {m:?}");
        assert!(m.ascender > m.cap_height, "{label}: {m:?}");
        assert!(m.cap_height >= m.x_height, "{label}: {m:?}");
        assert!(m.x_height > 0 && m.descender < 0, "{label}: {m:?}");
        assert!(m.line_gap >= 0, "{label}: {m:?}");

        let em = f64::from(m.units_per_em);
        // At 72 pt one em is one inch.
        assert!(close(m.cap_height_mm(72.0), f64::from(m.cap_height) / em * 25.4, 1e-9));
        assert!(close(m.x_height_mm(72.0), f64::from(m.x_height) / em * 25.4, 1e-9));
        assert!(close(m.ascender_mm(72.0), f64::from(m.ascender) / em * 25.4, 1e-9));
        assert!(m.descender_mm(72.0) < 0.0, "descender keeps its sign");
        assert!(close(
            m.line_height_mm(10.0),
            m.ascender_mm(10.0) - m.descender_mm(10.0) + m.line_gap_mm(10.0),
            1e-9
        ));
        // Helpers are linear in the size.
        assert!(close(m.cap_height_mm(30.0), 3.0 * m.cap_height_mm(10.0), 1e-9));
        assert_eq!(m.cap_height_mm(0.0), 0.0);
        let size = m.size_for_cap_height(6.0);
        assert!(close(m.cap_height_mm(size), 6.0, 1e-9), "{label}");
    }
    let atkinson = common::atkinson().metrics();
    assert_eq!((atkinson.units_per_em, atkinson.cap_height, atkinson.x_height), (1000, 668, 496));
    assert_eq!(common::courier().metrics().units_per_em, 2048);
    // Bebas Neue has no lower case: its "x" is a capital.
    let bebas = common::bebas().metrics();
    assert_eq!(bebas.cap_height, bebas.x_height);
}

#[test]
fn cap_height_and_x_height_agree_with_the_outlines() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let m = font.metrics();
        let opts = ShapeOptions::default();
        for (letter, expected_mm) in [("H", m.cap_height_mm(100.0)), ("x", m.x_height_mm(100.0))] {
            let ink = shape(&font, letter, 100.0, &opts)
                .ink_bounds(&font, Point::new(0.0, 0.0))
                .expect("letter has ink");
            // The letter stands on the baseline (y = 0, y down) and rises to the metric.
            assert!(close(ink.bottom(), 0.0, 0.01 * expected_mm), "{label} {letter}: {ink:?}");
            assert!(
                close(ink.h, expected_mm, 0.03 * expected_mm),
                "{label} {letter}: ink {} mm vs metric {expected_mm} mm",
                ink.h
            );
        }
    }
}

#[test]
fn metrics_fall_back_to_outlines_and_then_to_conventions() {
    // A real font robbed of its OS/2 table: heights are measured on "H" and "x".
    let reference = common::atkinson().metrics();
    let stripped = Font::from_bytes(common::without_table(common::ATKINSON, b"OS/2"), 0)
        .expect("font without OS/2")
        .metrics();
    assert!((stripped.cap_height - reference.cap_height).abs() <= 3, "{stripped:?}");
    assert!((stripped.x_height - reference.x_height).abs() <= 12, "{stripped:?}");
    assert!(stripped.ascender > stripped.cap_height && stripped.descender < 0);

    // Synthetic font: "H" is 700 units tall, "x" 500.
    let measured = synthetic(|_| {}).metrics();
    assert_eq!((measured.cap_height, measured.x_height), (700, 500));
    assert_eq!((measured.ascender, measured.descender, measured.line_gap), (800, -200, 0));
    // OS/2 version 4 with explicit heights: the table wins.
    let declared = synthetic(|f| f.os2 = Some((4, 0, true))).metrics();
    assert_eq!((declared.cap_height, declared.x_height), (680, 480));
    // OS/2 version 1 has no height fields, and zeros do not count: outlines again.
    let v1 = synthetic(|f| f.os2 = Some((1, 0, true))).metrics();
    assert_eq!((v1.cap_height, v1.x_height), (700, 500));
    let zeros = synthetic(|f| f.os2 = Some((4, 0, false))).metrics();
    assert_eq!((zeros.cap_height, zeros.x_height), (700, 500));

    // No vertical metrics at all: the font bounding box (-200..800) is used.
    let boxed = synthetic(|f| {
        f.ascender = 0;
        f.descender = 0;
    })
    .metrics();
    assert_eq!((boxed.ascender, boxed.descender), (800, -200));
    assert!(boxed.line_height_mm(10.0) > 0.0);
}

// --- coverage ----------------------------------------------------------------------

#[test]
fn missing_chars_lists_what_the_font_cannot_draw() {
    let font = common::atkinson();
    assert!(font.missing_chars("L'Étranger — efficace œuvre").is_empty());
    assert!(font.missing_chars("Zażółć gęślą jaźń — Straße, Ærø, Ñandú").is_empty());
    assert!(font.missing_chars("").is_empty());
    // Deduplicated, in order of first appearance.
    assert_eq!(font.missing_chars("好 Zażółć 你好 🎁 你"), ['好', '你', '🎁']);
    // The compatibility ligature U+FB01 is not in this font and has no canonical
    // decomposition the shaper could use.
    assert_eq!(font.missing_chars("ﬁn"), ['ﬁ']);

    let tiny = synthetic(|_| {}); // only "A", "H", "x" and the space
    assert!(tiny.missing_chars("A Hx  xHA").is_empty());
    assert_eq!(tiny.missing_chars("AxB b,Hé"), ['B', 'b', ',', 'é']);
}

#[test]
fn invisible_and_substitutable_characters_are_not_reported_missing() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        // Line feeds, tabs and other controls are set as spaces.
        assert!(font.missing_chars("a\nb\tc\r\u{85}\u{2028}\u{2029}\u{0}").is_empty(), "{label}");
        // Invisible format characters are hidden by the shaper.
        let invisible = "\u{ad}\u{200b}\u{200c}\u{200d}\u{200e}\u{2060}\u{feff}\u{fe0f}";
        assert!(font.missing_chars(invisible).is_empty(), "{label}");
        // Space variants are synthesised from the space glyph when the font lacks them.
        let spaces = "\u{a0}\u{2002}\u{2003}\u{2009}\u{200a}\u{202f}";
        assert!(font.missing_chars(spaces).is_empty(), "{label}");
    }
    // The narrow no-break space of French typography is such a synthesised space here.
    let atkinson = common::atkinson();
    assert_eq!(atkinson.glyph_id('\u{202f}'), None);
    let run = shape(&atkinson, "12\u{202f}€", 10.0, &ShapeOptions::default());
    assert!(run.glyphs.iter().all(|g| g.id != 0));
}

#[test]
fn decomposable_characters_missing_from_the_cmap_are_composed_by_the_shaper() {
    let opts = ShapeOptions::default();
    // Atkinson Hyperlegible has no "ẽ" (U+1EBD), but it has "e" and a combining tilde:
    // the shaper draws the letter from the two, so it is not missing.
    let atkinson = common::atkinson();
    assert_eq!(atkinson.glyph_id('ẽ'), None);
    assert!(atkinson.missing_chars("ẽ").is_empty());
    let run = shape(&atkinson, "ẽ", 10.0, &opts);
    let ids: Vec<u16> = run.glyphs.iter().map(|g| g.id).collect();
    assert_eq!(
        ids,
        [atkinson.glyph_id('e').expect("e"), atkinson.glyph_id('\u{303}').expect("tilde")]
    );
    // One source character, two glyphs: the text sits on the first one.
    let texts: Vec<&str> = run.glyphs.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(texts, ["ẽ", ""]);

    // Courier Prime has no combining marks to build it from: there it is missing.
    let courier = common::courier();
    assert_eq!(courier.glyph_id('\u{303}'), None);
    assert_eq!(courier.missing_chars("peẽ"), ['ẽ']);
    assert!(shape(&courier, "ẽ", 10.0, &opts).glyphs.iter().any(|g| g.id == 0));

    // In general: `missing_chars` agrees with what shaping actually draws.
    let candidates = "ẽĩũỹṽǘǜǚǖȫȭṏṍṹṻḗḕṓṑẫẵỗǰẗẘẙḧẍṅṁḃċḋėḟġȧṫẋẏżǻǽǿȁȃȅȇ";
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        for c in candidates.chars() {
            let drawable =
                shape(&font, &c.to_string(), 10.0, &opts).glyphs.iter().all(|g| g.id != 0);
            assert_eq!(font.missing_chars(&c.to_string()).is_empty(), drawable, "{label}: {c:?}");
        }
    }
}

#[test]
fn glyph_lookup() {
    let font = common::spectral();
    let a = font.glyph_id('A').expect("A");
    assert!(a > 0 && a < font.glyph_count());
    assert_ne!(font.glyph_id('É'), font.glyph_id('E'));
    assert_eq!(font.glyph_id('你'), None);
    assert_eq!(font.glyph_id('\u{10FFFF}'), None);
    assert_eq!(common::font(common::ATKINSON).glyph_count(), 369);
}

// --- loading -----------------------------------------------------------------------

#[test]
fn garbage_is_rejected_with_an_error_and_never_panics() {
    let mut rng = common::Rng(0x5EED_CAFE);
    let random: Vec<u8> = (0..4096).map(|_| rng.next_u64() as u8).collect();
    let mut random_with_magic = random.clone();
    random_with_magic[..4].copy_from_slice(&[0, 1, 0, 0]);
    let mut random_otto = random.clone();
    random_otto[..4].copy_from_slice(b"OTTO");
    let mut random_ttc = random.clone();
    random_ttc[..4].copy_from_slice(b"ttcf");
    let mut huge_table_count = vec![0u8, 1, 0, 0, 0xFF, 0xFF];
    huge_table_count.extend_from_slice(&[0; 64]);

    let inputs: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("one byte", vec![0]),
        ("zeros", vec![0; 64]),
        ("magic only", vec![0, 1, 0, 0]),
        ("OTTO only", b"OTTO".to_vec()),
        ("ttcf only", b"ttcf".to_vec()),
        ("ttcf with huge face count", b"ttcf\0\x01\0\0\xff\xff\xff\xff".to_vec()),
        ("huge table count", huge_table_count),
        ("png", b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec()),
        ("html", b"<!doctype html><html><body>404</body></html>".to_vec()),
        ("woff2 signature", b"wOF2\0\x01\0\0".to_vec()),
        ("random", random),
        ("random after TrueType magic", random_with_magic),
        ("random after OTTO", random_otto),
        ("random after ttcf", random_ttc),
        ("truncated directory", common::ATKINSON[..12].to_vec()),
        ("directory without tables", common::ATKINSON[..300].to_vec()),
        ("no head", common::without_table(common::ATKINSON, b"head")),
        ("no hhea", common::without_table(common::ATKINSON, b"hhea")),
        ("no maxp", common::without_table(common::ATKINSON, b"maxp")),
    ];
    for (label, bytes) in inputs {
        match Font::from_bytes(bytes, 0) {
            Err(FontError::Parse(message)) => assert!(!message.is_empty(), "{label}"),
            other => panic!("{label}: expected a parse error, got {other:?}"),
        }
    }
}

#[test]
fn fonts_with_impossible_headers_are_rejected() {
    // unitsPerEm = 0 (offset 18 in `head`).
    let mut bytes = common::ATKINSON.to_vec();
    let (head, _) = common::find_table(&bytes, b"head").expect("head");
    bytes[head + 18..head + 20].copy_from_slice(&[0, 0]);
    assert!(matches!(Font::from_bytes(bytes, 0), Err(FontError::Parse(_))));

    // numGlyphs = 0 (offset 4 in `maxp`).
    let mut bytes = common::ATKINSON.to_vec();
    let (maxp, _) = common::find_table(&bytes, b"maxp").expect("maxp");
    bytes[maxp + 4..maxp + 6].copy_from_slice(&[0, 0]);
    assert!(matches!(Font::from_bytes(bytes, 0), Err(FontError::Parse(_))));
}

#[test]
fn face_index_selects_a_face_of_a_collection() {
    // A plain font file has exactly one face.
    assert_eq!(
        Font::from_bytes(common::ATKINSON, 1).expect_err("a plain font has one face"),
        FontError::FaceIndex { index: 1, count: 1 }
    );

    let mut alpha = SyntheticFont::new();
    alpha.names = vec![NameRecord::windows(1, "Alpha")];
    let mut beta = SyntheticFont::new();
    beta.names = vec![NameRecord::windows(1, "Beta")];
    beta.variable = true;
    let collection: Arc<[u8]> = SyntheticFont::collection(&[&alpha, &beta]).into();

    let first = Font::from_bytes(Arc::clone(&collection), 0).expect("face 0");
    let second = Font::from_bytes(Arc::clone(&collection), 1).expect("face 1");
    assert_eq!((first.family_name().as_str(), first.face_index()), ("Alpha", 0));
    assert_eq!((second.family_name().as_str(), second.face_index()), ("Beta", 1));
    assert!(!first.is_variable() && second.is_variable());
    assert_ne!(first, second, "same data, different faces");
    // Both faces shape.
    let opts = ShapeOptions::default();
    assert!(shape(&first, "AxH", 10.0, &opts).width > 0.0);
    assert!(shape(&second, "AxH", 10.0, &opts).width > 0.0);

    assert_eq!(
        Font::from_bytes(Arc::clone(&collection), 2).expect_err("only two faces"),
        FontError::FaceIndex { index: 2, count: 2 }
    );
    assert!(matches!(
        Font::from_bytes(collection, u32::MAX),
        Err(FontError::FaceIndex { index: u32::MAX, count: 2 })
    ));
}

#[test]
fn fonts_are_cheap_to_clone_comparable_and_thread_safe() {
    fn assert_send_sync<T: Send + Sync + Clone + std::fmt::Debug + PartialEq>() {}
    assert_send_sync::<Font>();

    let shared: Arc<[u8]> = Arc::from(common::SPECTRAL);
    let font = Font::from_bytes(Arc::clone(&shared), 0).expect("font");
    assert!(Arc::ptr_eq(font.data(), &shared), "an Arc is adopted, not copied");
    let copy = font.clone();
    assert!(Arc::ptr_eq(font.data(), copy.data()));
    assert_eq!(font, copy);
    // Equal content in another allocation is the same font; another font is not.
    assert_eq!(font, common::spectral());
    assert_ne!(font, common::atkinson());

    let debug = format!("{font:?}");
    assert!(debug.contains("Spectral") && debug.len() < 300, "{debug}");

    // Usable from several threads at once.
    let widths: Vec<f64> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| shape(&font, "L'Étranger", 12.0, &ShapeOptions::default()).width)
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("thread")).collect()
    });
    assert!(widths.windows(2).all(|w| w[0] == w[1]) && widths[0] > 0.0);
}

// --- variable fonts ----------------------------------------------------------------

#[test]
fn variable_font_axes_are_described() {
    let font = synthetic(|f| {
        f.variable = true;
        f.names = vec![NameRecord::windows(1, "Flex"), NameRecord::windows(256, "Weight")];
    });
    assert!(font.is_variable());
    let axes = font.axes();
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].tag, "wght");
    assert_eq!(axes[0].name.as_deref(), Some("Weight"));
    assert_eq!((axes[0].min, axes[0].default, axes[0].max), (100.0, 400.0, 900.0));
    assert!(!axes[0].hidden);
    assert_eq!(font.variations(), [("wght".to_owned(), 400.0)]);
}

#[test]
fn variation_instances_change_advances_and_outlines() {
    let font = synthetic(|f| f.variable = true);
    let opts = ShapeOptions::default();
    let m = font.metrics();
    // Width of "A" in font units at a given weight.
    let advance = |f: &Font| shape(f, "A", 10.0, &opts).width / m.mm_per_unit(10.0);
    let ink_width = |f: &Font| {
        let ink = shape(f, "A", 10.0, &opts).ink_bounds(f, Point::new(0.0, 0.0)).expect("ink");
        (ink.w / m.mm_per_unit(10.0), ink.h / m.mm_per_unit(10.0))
    };

    assert!(close(advance(&font), 600.0, 1e-6));
    assert!(close(ink_width(&font).0, 400.0, 1e-6));

    let black = font.with_variations(&[("wght", 900.0)]);
    assert_eq!(black.variations(), [("wght".to_owned(), 900.0)]);
    assert!(close(advance(&black), 700.0, 1e-6), "{}", advance(&black));
    assert!(close(ink_width(&black).0, 500.0, 1e-6));
    assert!(close(ink_width(&black).1, 700.0, 1e-6), "height does not vary");

    let thin = font.with_variations(&[("wght", 100.0)]);
    assert!(close(advance(&thin), 550.0, 1e-6), "{}", advance(&thin));
    assert!(close(ink_width(&thin).0, 350.0, 1e-6));

    // Half-way between the default (400) and the maximum (900): half the delta.
    let medium = font.with_variations(&[("wght", 650.0)]);
    assert!(close(advance(&medium), 650.0, 1.0), "{}", advance(&medium));

    // The instance shares the font data and leaves the original untouched.
    assert!(Arc::ptr_eq(font.data(), black.data()));
    assert!(close(advance(&font), 600.0, 1e-6));
    assert_ne!(font, black);
    assert_eq!(font.metrics(), black.metrics(), "no MVAR table: global metrics do not vary");
}

#[test]
fn variation_settings_are_clamped_validated_and_normalised() {
    let font = synthetic(|f| f.variable = true);
    let black = font.with_variations(&[("wght", 900.0)]);

    // Out-of-range values are clamped to the axis.
    assert_eq!(font.with_variations(&[("wght", 5000.0)]), black);
    assert_eq!(font.with_variations(&[("wght", f32::MAX)]), black);
    assert_eq!(font.with_variations(&[("wght", -1.0)]), font.with_variations(&[("wght", 100.0)]));
    // Unknown axes, malformed tags and non-finite values are ignored.
    for ignored in [("wdth", 75.0), ("weight", 900.0), ("", 900.0), ("wg\u{e9}t", 900.0)] {
        assert_eq!(font.with_variations(&[ignored]), font, "{ignored:?}");
    }
    assert_eq!(font.with_variations(&[("wght", f32::NAN)]), font);
    assert_eq!(font.with_variations(&[("wght", f32::INFINITY)]), font);
    assert_eq!(font.with_variations(&[]), font);
    // Later settings override earlier ones, within a call and across calls.
    assert_eq!(font.with_variations(&[("wght", 100.0), ("wght", 900.0)]), black);
    assert_eq!(font.with_variations(&[("wght", 100.0)]).with_variations(&[("wght", 900.0)]), black);
    // Going back to the default value gives the default instance again.
    assert_eq!(black.with_variations(&[("wght", 400.0)]), font);
    // Valid settings survive next to ignored ones.
    assert_eq!(font.with_variations(&[("nope", 1.0), ("wght", 900.0)]), black);
}

#[test]
fn static_fonts_ignore_variation_settings() {
    for font in [common::atkinson(), synthetic(|_| {})] {
        let same = font.with_variations(&[("wght", 700.0), ("wdth", 80.0)]);
        assert_eq!(same, font);
        assert!(Arc::ptr_eq(same.data(), font.data()));
        assert!(same.variations().is_empty());
    }
}
