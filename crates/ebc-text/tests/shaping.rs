//! Shaping: glyph positions, cluster text, options, glyph items and outlines.

mod common;

use common::close;
use ebc_scene::{Color, FontId, Item, MM_PER_PT, PathSeg, Point, Transform};
use ebc_text::{
    Font, MAX_SIZE_PT, OutlineFormat, ShapeOptions, ShapedGlyph, ShapedText, outline_bounds, shape,
};

const FRENCH: &str = "L'Étranger — efficace œuvre";

fn default_opts() -> ShapeOptions {
    ShapeOptions::default()
}

fn texts(run: &ShapedText) -> Vec<&str> {
    run.glyphs.iter().map(|g| g.text.as_str()).collect()
}

fn ids(run: &ShapedText) -> Vec<u16> {
    run.glyphs.iter().map(|g| g.id).collect()
}

/// The source text reassembled from the glyphs of a left-to-right run.
fn reassembled(run: &ShapedText) -> String {
    run.glyphs.iter().map(|g| g.text.as_str()).collect()
}

// --- positions and cluster text ----------------------------------------------------

#[test]
fn french_text_with_accents_shapes_in_every_bundled_font() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let run = shape(&font, FRENCH, 12.0, &default_opts());

        assert!(run.glyphs.iter().all(|g| g.id != 0), "{label}: no .notdef");
        assert_eq!(reassembled(&run), FRENCH, "{label}: glyph text reassembles the source");
        assert_eq!(run.size_pt, 12.0);

        // Accented and special letters are single precomposed glyphs carrying their text.
        for letter in ['É', 'œ', '—', '\''] {
            let expected = font.glyph_id(letter).expect("letter is in the font");
            let glyph = run
                .glyphs
                .iter()
                .find(|g| g.text == letter.to_string())
                .unwrap_or_else(|| panic!("{label}: a glyph stands for {letter:?}"));
            assert_eq!(glyph.id, expected, "{label}: {letter:?}");
        }

        // Left to right on one baseline: positions grow, advances add up to the width.
        assert!(run.glyphs.windows(2).all(|w| w[0].x < w[1].x), "{label}: x increases");
        assert!(run.glyphs.iter().all(|g| g.y == 0.0 && g.advance > 0.0), "{label}");
        let total: f64 = run.glyphs.iter().map(|g| g.advance).sum();
        assert!(close(total, run.width, 1e-9), "{label}: {total} vs {}", run.width);
        let last = run.glyphs.last().expect("glyphs");
        assert!(close(last.x + last.advance, run.width, 1e-9), "{label}");
        // A plausible width: 27 characters at 12 pt are between 30 and 80 mm long.
        assert!(run.width > 30.0 && run.width < 80.0, "{label}: {}", run.width);
    }
}

#[test]
fn ligatures_carry_the_text_of_all_their_letters() {
    // Spectral has an "ffi" ligature; "efficace" is 8 letters but 6 glyphs.
    let font = common::spectral();
    let run = shape(&font, "efficace", 12.0, &default_opts());
    assert_eq!(texts(&run), ["e", "ffi", "c", "a", "c", "e"]);
    let ligature = &run.glyphs[1];
    assert_ne!(Some(ligature.id), font.glyph_id('f'));
    // The ligature takes about the room of its three letters (exactly so in Spectral).
    let f = shape(&font, "f", 12.0, &default_opts()).width;
    let i = shape(&font, "i", 12.0, &default_opts()).width;
    assert!(close(ligature.advance, 2.0 * f + i, 0.1 * f), "{}", ligature.advance);

    let whole = shape(&font, FRENCH, 12.0, &default_opts());
    assert_eq!(whole.glyphs.len(), FRENCH.chars().count() - 2);
    assert!(texts(&whole).contains(&"ffi"));
    assert_eq!(reassembled(&whole), FRENCH);

    // Atkinson Hyperlegible deliberately has no such ligature: one glyph per letter.
    let plain = shape(&common::atkinson(), "efficace", 12.0, &default_opts());
    assert_eq!(texts(&plain), ["e", "f", "f", "i", "c", "a", "c", "e"]);
}

#[test]
fn clusters_of_several_glyphs_put_the_text_on_the_first_glyph() {
    // "x" + combining acute has no precomposed form: base and mark are two glyphs of
    // one cluster. The base carries the text of the whole cluster, the mark nothing.
    let font = common::spectral();
    assert!(font.glyph_id('\u{301}').is_some(), "Spectral has a combining acute");
    let run = shape(&font, "ax\u{301}b", 20.0, &default_opts());
    assert_eq!(texts(&run), ["a", "x\u{301}", "", "b"]);
    assert_eq!(run.glyphs[1].id, font.glyph_id('x').expect("x"));
    let (base, mark) = (&run.glyphs[1], &run.glyphs[2]);
    // The mark does not advance the pen and sits over the base, raised above x-height.
    assert_eq!(mark.advance, 0.0);
    assert!(mark.x > base.x - 0.01 && mark.x < base.x + base.advance + 0.01, "{mark:?}");
    assert!(close(base.x + base.advance, run.glyphs[3].x, 1e-9));
    assert_eq!(reassembled(&run), "ax\u{301}b");

    // A decomposed "é" is recomposed to the font's precomposed glyph: one glyph, and
    // the text is still what the caller wrote.
    let composed = shape(&font, "e\u{301}", 20.0, &default_opts());
    assert_eq!(ids(&composed), [font.glyph_id('é').expect("é")]);
    assert_eq!(texts(&composed), ["e\u{301}"]);
}

#[test]
fn characters_missing_from_the_font_become_notdef_but_keep_their_text() {
    let font = common::atkinson();
    let run = shape(&font, "a你b🎁", 12.0, &default_opts());
    assert_eq!(ids(&run), [font.glyph_id('a').expect("a"), 0, font.glyph_id('b').expect("b"), 0]);
    assert_eq!(texts(&run), ["a", "你", "b", "🎁"]);
    assert!(run.width > 0.0);
    assert_eq!(font.missing_chars("a你b🎁"), ['你', '🎁']);
}

#[test]
fn control_characters_are_set_as_spaces() {
    let font = common::atkinson();
    let space = font.glyph_id(' ').expect("space");
    let text = "a\nb\tc\u{2028}d\r\n";
    let run = shape(&font, text, 12.0, &default_opts());
    let expected = [
        font.glyph_id('a').expect("a"),
        space,
        font.glyph_id('b').expect("b"),
        space,
        font.glyph_id('c').expect("c"),
        space,
        font.glyph_id('d').expect("d"),
        space,
        space,
    ];
    assert_eq!(ids(&run), expected);
    assert_eq!(reassembled(&run), text, "the original characters are kept as glyph text");
    assert_eq!(run.width, shape(&font, "a b c d  ", 12.0, &default_opts()).width);
}

#[test]
fn soft_hyphens_and_joiners_are_invisible() {
    let font = common::atkinson();
    let plain = shape(&font, "ab", 12.0, &default_opts());
    for invisible in ['\u{ad}', '\u{200b}', '\u{200d}', '\u{feff}'] {
        let text = format!("a{invisible}b");
        let run = shape(&font, &text, 12.0, &default_opts());
        assert!(run.glyphs.iter().all(|g| g.id != 0), "{invisible:?}");
        assert!(close(run.width, plain.width, 1e-9), "{invisible:?} takes no room");
        assert_eq!(reassembled(&run), text);
    }
}

#[test]
fn glyph_text_is_never_lost_for_any_input() {
    let long = "Les Misérables — tome 1 : Fantine. ".repeat(40);
    let corpus = [
        "",
        " ",
        "a",
        FRENCH,
        "ffi fl ff ffl fi",
        "e\u{301}\u{302}\u{303}\u{304}",
        "\u{301}",
        "👩‍👩‍👧‍👦 🏳️‍🌈",
        "\u{feff}a\u{200d}b\u{200c}",
        "\n\n\t",
        "عربى مرحبا",
        "שלום",
        "नमस्ते",
        "한국어 日本語 中文",
        "𝔘𝔫𝔦𝔠𝔬𝔡𝔢",
        "A\u{fe0f}\u{20e3}",
        "\u{202e}abc\u{202c}",
        "a\u{ad}b",
        "\u{0}\u{1}\u{7f}\u{80}\u{9f}",
        long.as_str(),
    ];
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        for text in corpus {
            let ltr = shape(&font, text, 11.0, &default_opts());
            assert_eq!(reassembled(&ltr), text, "{label}: {text:?}");
            assert!(ltr.width.is_finite() && ltr.width >= 0.0, "{label}: {text:?}");
            assert!(
                ltr.glyphs.iter().all(|g| g.x.is_finite() && g.y.is_finite()),
                "{label}: {text:?}"
            );

            // Right to left: glyphs come in visual order, i.e. clusters reversed.
            let opts = ShapeOptions { rtl: true, ..default_opts() };
            let rtl = shape(&font, text, 11.0, &opts);
            let logical: String = rtl.glyphs.iter().rev().map(|g| g.text.as_str()).collect();
            assert_eq!(logical, text, "{label} rtl: {text:?}");
        }
    }
}

// --- size --------------------------------------------------------------------------

#[test]
fn width_grows_with_the_size_and_is_exactly_proportional() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let sizes = [0.5, 1.0, 6.0, 9.5, 12.0, 24.0, 72.0, 300.0];
        let widths: Vec<f64> =
            sizes.iter().map(|&s| shape(&font, FRENCH, s, &default_opts()).width).collect();
        assert!(widths.windows(2).all(|w| w[0] < w[1]), "{label}: monotonic {widths:?}");
        let per_point = widths[1];
        for (size, width) in sizes.iter().zip(&widths) {
            assert!(
                close(width / size, per_point, 1e-12 * per_point.max(1.0)),
                "{label}: {size} pt"
            );
        }

        // Glyph positions scale too.
        let small = shape(&font, FRENCH, 10.0, &default_opts());
        let large = shape(&font, FRENCH, 30.0, &default_opts());
        assert_eq!(ids(&small), ids(&large), "{label}: shaping does not depend on the size");
        for (a, b) in small.glyphs.iter().zip(&large.glyphs) {
            assert!(close(3.0 * a.x, b.x, 1e-9) && close(3.0 * a.advance, b.advance, 1e-9));
        }
    }
}

#[test]
fn an_em_is_the_font_size() {
    // The synthetic "A" advances 600 of 1000 units per em: at 10 pt that is 6 pt.
    let font = Font::from_bytes(common::SyntheticFont::new().build(), 0).expect("font");
    let run = shape(&font, "AAAAA", 10.0, &default_opts());
    assert!(close(run.width, 30.0 * MM_PER_PT, 1e-12), "{}", run.width);
    for (k, glyph) in run.glyphs.iter().enumerate() {
        assert!(close(glyph.x, k as f64 * 6.0 * MM_PER_PT, 1e-12));
        assert!(close(glyph.advance, 6.0 * MM_PER_PT, 1e-12));
    }
    // 72 pt = 1 inch: the same five letters are 5 x 0.6 x 25.4 mm long.
    assert!(close(shape(&font, "AAAAA", 72.0, &default_opts()).width, 76.2, 1e-9));

    // Courier Prime is monospaced, about 0.6 em per character.
    let run = shape(&common::courier(), "0123456789 iMW.", 10.0, &default_opts());
    let pitch = run.glyphs[0].advance;
    assert!(close(pitch, 6.0 * MM_PER_PT, 0.01), "{pitch}");
    assert!(run.glyphs.iter().all(|g| g.advance == pitch));
}

#[test]
fn degenerate_sizes_are_clamped_instead_of_producing_nonsense() {
    let font = common::atkinson();
    for size in [0.0, -12.0, f64::NAN, f64::NEG_INFINITY] {
        let run = shape(&font, "abc", size, &default_opts());
        assert_eq!(run.size_pt, 0.0);
        assert_eq!(run.width, 0.0);
        assert_eq!(run.glyphs.len(), 3);
        assert!(run.glyphs.iter().all(|g| g.x == 0.0 && g.y == 0.0 && g.advance == 0.0));
        assert!(run.to_outline(&font, Point::new(1.0, 2.0)).control_bounds().is_some());
    }
    for size in [f64::INFINITY, 1e300, MAX_SIZE_PT * 2.0] {
        let run = shape(&font, "abc", size, &default_opts());
        assert_eq!(run.size_pt, MAX_SIZE_PT);
        assert!(run.width.is_finite() && run.width > 0.0);
        let bounds = run.ink_bounds(&font, Point::new(0.0, 0.0)).expect("ink");
        assert!(bounds.w.is_finite() && bounds.h.is_finite());
    }
}

#[test]
fn empty_text_is_an_empty_run() {
    let font = common::atkinson();
    let run = shape(&font, "", 12.0, &default_opts());
    assert!(run.glyphs.is_empty());
    assert_eq!((run.width, run.size_pt), (0.0, 12.0));
    assert!(run.to_outline(&font, Point::new(5.0, 5.0)).is_empty());
    assert_eq!(run.ink_bounds(&font, Point::new(5.0, 5.0)), None);
    // Blank text advances but draws nothing.
    let blank = shape(&font, "   ", 12.0, &default_opts());
    assert!(blank.width > 0.0);
    assert!(blank.to_outline(&font, Point::new(5.0, 5.0)).is_empty());
    assert_eq!(blank.ink_bounds(&font, Point::new(5.0, 5.0)), None);
}

// --- options -----------------------------------------------------------------------

#[test]
fn kerning_is_applied_and_can_be_switched_off() {
    let width = |font: &Font, text: &str, opts: &ShapeOptions| shape(font, text, 12.0, opts).width;
    let no_kerning = ShapeOptions { features: vec![("kern".into(), 0)], ..default_opts() };
    for font in [common::spectral(), common::atkinson(), common::bebas()] {
        let apart = width(&font, "A", &default_opts()) + width(&font, "V", &default_opts());
        let kerned = width(&font, "AV", &default_opts());
        assert!(kerned < apart - 0.05, "{}: {kerned} vs {apart}", font.family_name());
        assert!(close(width(&font, "AV", &no_kerning), apart, 1e-9), "{}", font.family_name());
    }
    // A monospaced font has nothing to kern.
    let mono = common::courier();
    assert!(close(
        width(&mono, "AV", &default_opts()),
        2.0 * width(&mono, "A", &default_opts()),
        1e-9
    ));
}

#[test]
fn letter_spacing_goes_between_letters_only() {
    let font = common::atkinson();
    let size = 20.0;
    let em_mm = size * MM_PER_PT;
    let text = "Étranger"; // 8 letters, 7 gaps
    let plain = shape(&font, text, size, &default_opts());
    for spacing in [0.1f32, 0.5, -0.05] {
        let opts = ShapeOptions { letter_spacing_em: spacing, ..default_opts() };
        let spaced = shape(&font, text, size, &opts);
        let expected = plain.width + 7.0 * f64::from(spacing) * em_mm;
        assert!(close(spaced.width, expected, 1e-6), "{spacing}: {} vs {expected}", spaced.width);
        // Glyph k is shifted by k gaps; the last advance carries no trailing gap.
        for (k, (a, b)) in plain.glyphs.iter().zip(&spaced.glyphs).enumerate() {
            let shift = k as f64 * f64::from(spacing) * em_mm;
            assert!(close(b.x, a.x + shift, 1e-6), "glyph {k}");
        }
        let (last_plain, last_spaced) =
            (plain.glyphs.last().expect("glyph"), spaced.glyphs.last().expect("glyph"));
        assert!(close(last_plain.advance, last_spaced.advance, 1e-9));
        let total: f64 = spaced.glyphs.iter().map(|g| g.advance).sum();
        assert!(close(total, spaced.width, 1e-9));
    }
    // A single letter has no gap to widen.
    let opts = ShapeOptions { letter_spacing_em: 0.5, ..default_opts() };
    assert_eq!(
        shape(&font, "É", size, &opts).width,
        shape(&font, "É", size, &default_opts()).width
    );
    // Non-finite spacing is ignored.
    for bad in [f32::NAN, f32::INFINITY] {
        let opts = ShapeOptions { letter_spacing_em: bad, ..default_opts() };
        assert_eq!(shape(&font, text, size, &opts), plain);
    }
}

#[test]
fn letter_spacing_breaks_ligatures_and_treats_a_cluster_as_one_letter() {
    let font = common::spectral();
    let spaced = ShapeOptions { letter_spacing_em: 0.1, ..default_opts() };
    let run = shape(&font, "efficace", 12.0, &spaced);
    assert_eq!(texts(&run), ["e", "f", "f", "i", "c", "a", "c", "e"], "no ligature when spaced");

    // …unless the caller insists on ligatures.
    let insisting = ShapeOptions {
        letter_spacing_em: 0.1,
        features: vec![("liga".into(), 1)],
        ..default_opts()
    };
    assert_eq!(texts(&shape(&font, "efficace", 12.0, &insisting)).len(), 6);

    // Base + combining mark is one letter: no gap between the two glyphs.
    let gap = 0.1 * 12.0 * MM_PER_PT;
    let plain = shape(&font, "ax\u{301}b", 12.0, &default_opts());
    let run = shape(&font, "ax\u{301}b", 12.0, &spaced);
    assert_eq!(run.glyphs.len(), 4);
    assert!(close(run.width, plain.width + 2.0 * gap, 1e-6), "3 letters, 2 gaps");
    assert!(close(run.glyphs[2].x - run.glyphs[1].x, plain.glyphs[2].x - plain.glyphs[1].x, 1e-9));
}

#[test]
fn opentype_features_can_be_switched() {
    let font = common::spectral();
    let off = ShapeOptions { features: vec![("liga".into(), 0)], ..default_opts() };
    assert_eq!(shape(&font, "efficace", 12.0, &off).glyphs.len(), 8);

    // Small capitals replace the lower-case glyphs (Spectral has `smcp`).
    let small_caps = ShapeOptions { features: vec![("smcp".into(), 1)], ..default_opts() };
    let plain = shape(&font, "roman", 12.0, &default_opts());
    let caps = shape(&font, "roman", 12.0, &small_caps);
    assert_eq!(texts(&caps), texts(&plain));
    assert!(ids(&caps).iter().zip(ids(&plain)).all(|(a, b)| *a != b), "every glyph is swapped");

    // Unknown or malformed features are ignored.
    let odd = ShapeOptions {
        features: vec![("zzzz".into(), 1), ("way too long".into(), 1), (String::new(), 3)],
        ..default_opts()
    };
    assert_eq!(shape(&font, "roman", 12.0, &odd), plain);
}

#[test]
fn language_is_passed_to_the_shaper_and_never_trusted() {
    let font = common::spectral();
    let plain = shape(&font, FRENCH, 12.0, &default_opts());
    let very_long = "x".repeat(5000);
    for language in ["fr", "fr-FR", "fr_FR", "tr", "", "  ", "🙂", "en-", very_long.as_str()] {
        let opts = ShapeOptions { language: Some(language.to_owned()), ..default_opts() };
        let run = shape(&font, FRENCH, 12.0, &opts);
        assert_eq!(reassembled(&run), FRENCH, "{language:?}");
        assert!(close(run.width, plain.width, 1e-9), "{language:?}");
    }
}

#[test]
fn right_to_left_runs_are_in_visual_order() {
    let font = common::courier();
    let ltr = shape(&font, "abc", 12.0, &default_opts());
    let rtl = shape(&font, "abc", 12.0, &ShapeOptions { rtl: true, ..default_opts() });
    assert_eq!(texts(&rtl), ["c", "b", "a"]);
    assert!(close(rtl.width, ltr.width, 1e-9));
    assert!(rtl.glyphs.windows(2).all(|w| w[0].x < w[1].x), "still laid out left to right");
    assert_eq!(rtl.glyphs[0].id, font.glyph_id('c').expect("c"));
}

// --- glyph items -------------------------------------------------------------------

#[test]
fn glyph_item_mirrors_the_run() {
    let font = common::spectral();
    let run = shape(&font, FRENCH, 14.0, &default_opts());
    let origin = Point::new(12.5, 40.0);
    let item = run.to_glyph_item(FontId(3), Color::BLACK, origin);
    let Item::Glyphs { font: id, size_pt, color, origin: at, glyphs, glyph_text } = item else {
        panic!("expected a glyph run");
    };
    assert_eq!(id, FontId(3));
    assert_eq!(size_pt, 14.0);
    assert_eq!(color, Color::BLACK);
    assert_eq!(at, origin);
    assert_eq!(glyphs.len(), run.glyphs.len());
    assert_eq!(glyph_text.len(), glyphs.len(), "one text entry per glyph");
    for ((glyph, text), shaped) in glyphs.iter().zip(&glyph_text).zip(&run.glyphs) {
        assert_eq!((glyph.id, glyph.x, glyph.y), (shaped.id, shaped.x, shaped.y));
        assert_eq!(text, &shaped.text);
    }
    assert_eq!(glyph_text.concat(), FRENCH, "ToUnicode text is complete");
    assert!(glyph_text.iter().any(|t| t == "ffi"));

    // An empty run is an empty, valid item.
    let empty =
        shape(&font, "", 14.0, &default_opts()).to_glyph_item(FontId(0), Color::BLACK, origin);
    assert!(matches!(empty, Item::Glyphs { ref glyphs, ref glyph_text, .. }
        if glyphs.is_empty() && glyph_text.is_empty()));
}

// --- outlines ----------------------------------------------------------------------

#[test]
fn outline_bounding_box_is_consistent_with_the_metrics() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let m = font.metrics();
        let size = 24.0;
        let origin = Point::new(20.0, 100.0);
        let run = shape(&font, FRENCH, size, &default_opts());
        let path = run.to_outline(&font, origin);
        let ink = outline_bounds(&path).expect("ink");
        let em = size * MM_PER_PT;

        // Vertically inside the line box: above the descender, below the ascender —
        // except that accented capitals may poke above it ("É" in Courier Prime does,
        // by 0.06 em).
        assert!(ink.y >= origin.y - m.ascender_mm(size) - 0.1 * em, "{label}: {ink:?}");
        assert!(ink.bottom() <= origin.y - m.descender_mm(size) + 1e-9, "{label}: {ink:?}");
        // The text has a "g" below the baseline and capitals above the x-height.
        assert!(ink.bottom() > origin.y || label == "Bebas Neue", "{label}: {ink:?}");
        assert!(ink.y < origin.y - m.x_height_mm(size), "{label}: {ink:?}");
        // Horizontally: starts at the origin and ends at the advance width, give or
        // take a side bearing.
        assert!(close(ink.x, origin.x, 0.15 * em), "{label}: {ink:?}");
        assert!(close(ink.right(), origin.x + run.width, 0.15 * em), "{label}: {ink:?}");

        // The exact box lies inside the control-point box, and ink_bounds agrees.
        let control = path.control_bounds().expect("points");
        assert!(control.contains_rect(&ink, 1e-9), "{label}");
        assert_eq!(run.ink_bounds(&font, origin), Some(ink), "{label}");
    }
}

#[test]
fn outlines_keep_truetype_curves_and_closed_contours() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let path =
            shape(&font, FRENCH, 12.0, &default_opts()).to_outline(&font, Point::new(0.0, 0.0));
        let count = |f: fn(&PathSeg) -> bool| path.segs.iter().filter(|s| f(s)).count();
        let moves = count(|s| matches!(s, PathSeg::MoveTo(_)));
        let closes = count(|s| matches!(s, PathSeg::Close));
        let quads = count(|s| matches!(s, PathSeg::QuadTo(..)));
        let cubics = count(|s| matches!(s, PathSeg::CubicTo(..)));
        // 24 inked glyphs (27 characters minus 3 spaces), several with 2+ contours.
        assert!(moves >= 30, "{label}: {moves} contours");
        assert_eq!(moves, closes, "{label}: every contour is closed");
        assert!(quads > 50, "{label}: TrueType curves stay quadratic ({quads})");
        assert_eq!(cubics, 0, "{label}: nothing is converted to cubics");
        assert!(matches!(path.segs.first(), Some(PathSeg::MoveTo(_))), "{label}");
        assert!(matches!(path.segs.last(), Some(PathSeg::Close)), "{label}");
    }
}

#[test]
fn outline_of_a_rectangular_glyph_is_exact() {
    // The synthetic "A" is the rectangle 100..500 x 0..700 (font units, y up) with a
    // 600-unit advance; at 1000 pt one unit is one point.
    let font = Font::from_bytes(common::SyntheticFont::new().build(), 0).expect("font");
    let run = shape(&font, "AA", 1000.0, &default_opts());
    let path = run.to_outline(&font, Point::new(10.0, 500.0));
    let pt = MM_PER_PT;
    let expected: Vec<PathSeg> = [0.0, 600.0]
        .iter()
        .flat_map(|pen| {
            let x = |units: f64| 10.0 + (pen + units) * pt;
            let y = |units: f64| 500.0 - units * pt;
            [
                PathSeg::MoveTo(Point::new(x(100.0), y(0.0))),
                PathSeg::LineTo(Point::new(x(100.0), y(700.0))),
                PathSeg::LineTo(Point::new(x(500.0), y(700.0))),
                PathSeg::LineTo(Point::new(x(500.0), y(0.0))),
                PathSeg::LineTo(Point::new(x(100.0), y(0.0))),
                PathSeg::Close,
            ]
        })
        .collect();
    assert_eq!(path.segs.len(), expected.len(), "{:?}", path.segs);
    for (got, want) in path.segs.iter().zip(&expected) {
        match (got, want) {
            (PathSeg::MoveTo(a), PathSeg::MoveTo(b)) | (PathSeg::LineTo(a), PathSeg::LineTo(b)) => {
                assert!(close(a.x, b.x, 1e-9) && close(a.y, b.y, 1e-9), "{got:?} vs {want:?}");
            }
            (PathSeg::Close, PathSeg::Close) => {}
            _ => panic!("{got:?} vs {want:?}"),
        }
    }
    assert!(close(run.width, 1200.0 * pt, 1e-9));
}

#[test]
fn postscript_outlines_stay_cubic() {
    // A synthetic OpenType/CFF font: "O" is a circle of radius 300 units centred on
    // (300, 350), drawn with four cubic Béziers; "H" is a rectangle of straight lines.
    let font = Font::from_bytes(common::synthetic_cff_font(), 0).expect("CFF font parses");
    assert_eq!(font.outline_format(), OutlineFormat::Cff);
    assert_eq!(font.glyph_count(), 4);

    // At 1000 pt one font unit is one point.
    let pt = MM_PER_PT;
    let run = shape(&font, "O", 1000.0, &default_opts());
    assert!(close(run.width, 700.0 * pt, 1e-9));
    let path = run.to_outline(&font, Point::new(0.0, 0.0));
    let cubics: Vec<_> = path
        .segs
        .iter()
        .filter_map(|s| match s {
            PathSeg::CubicTo(c1, c2, end) => Some((*c1, *c2, *end)),
            _ => None,
        })
        .collect();
    assert_eq!(cubics.len(), 4, "{:?}", path.segs);
    assert!(!path.segs.iter().any(|s| matches!(s, PathSeg::QuadTo(..))), "no conversion");
    assert!(matches!(path.segs.first(), Some(PathSeg::MoveTo(p))
        if close(p.x, 600.0 * pt, 1e-9) && close(p.y, -350.0 * pt, 1e-9)));
    assert!(matches!(path.segs.last(), Some(PathSeg::Close)));
    // First curve: from the rightmost point up to the top, y flipped for the page.
    let (c1, c2, end) = cubics[0];
    assert!(close(c1.x, 600.0 * pt, 1e-9) && close(c1.y, -516.0 * pt, 1e-9), "{c1:?}");
    assert!(close(c2.x, 466.0 * pt, 1e-9) && close(c2.y, -650.0 * pt, 1e-9), "{c2:?}");
    assert!(close(end.x, 300.0 * pt, 1e-9) && close(end.y, -650.0 * pt, 1e-9), "{end:?}");
    // The exact bounds are the circle's square.
    let ink = outline_bounds(&path).expect("ink");
    assert!(close(ink.x, 0.0, 1e-9) && close(ink.w, 600.0 * pt, 1e-9), "{ink:?}");
    assert!(close(ink.y, -650.0 * pt, 1e-9) && close(ink.h, 600.0 * pt, 1e-9), "{ink:?}");

    // Straight-line glyphs stay straight, and the two glyphs sit side by side.
    let both = shape(&font, "HO", 1000.0, &default_opts());
    let path = both.to_outline(&font, Point::new(0.0, 0.0));
    let lines = path.segs.iter().filter(|s| matches!(s, PathSeg::LineTo(_))).count();
    let cubics = path.segs.iter().filter(|s| matches!(s, PathSeg::CubicTo(..))).count();
    assert!(lines >= 3, "{:?}", path.segs);
    assert_eq!(cubics, 4);
    let ink = outline_bounds(&path).expect("ink");
    assert!(close(ink.x, 100.0 * pt, 1e-9), "{ink:?}");
    assert!(close(ink.right(), (600.0 + 600.0) * pt, 1e-9), "{ink:?}");

    // No OS/2 table: cap height is measured on "H"; there is no "x", so the x-height
    // falls back to half an em. Unknown embedding permission is refused.
    let m = font.metrics();
    assert_eq!((m.cap_height, m.x_height), (700, 500));
    assert!(!font.allows_outlines() && font.fs_type().is_none());
}

#[test]
fn outlines_translate_with_the_origin_and_scale_with_the_size() {
    let font = common::spectral();
    let run = shape(&font, "Œuvre fière", 18.0, &default_opts());
    let at_zero = run.to_outline(&font, Point::new(0.0, 0.0));
    let moved = run.to_outline(&font, Point::new(33.0, -7.5));
    let expected = at_zero.transformed(&Transform::translate(33.0, -7.5));
    assert_eq!(moved.segs.len(), expected.segs.len());
    let (a, b) = (outline_bounds(&moved).expect("ink"), outline_bounds(&expected).expect("ink"));
    assert!(close(a.x, b.x, 1e-9) && close(a.y, b.y, 1e-9) && close(a.w, b.w, 1e-9));

    let double = shape(&font, "Œuvre fière", 36.0, &default_opts())
        .ink_bounds(&font, Point::new(0.0, 0.0))
        .expect("ink");
    let single = outline_bounds(&at_zero).expect("ink");
    assert!(close(double.w, 2.0 * single.w, 1e-9) && close(double.h, 2.0 * single.h, 1e-9));
    assert!(close(double.x, 2.0 * single.x, 1e-9) && close(double.y, 2.0 * single.y, 1e-9));
}

#[test]
fn hand_built_runs_with_bad_glyphs_are_harmless() {
    let font = common::atkinson();
    let glyph = |id: u16, x: f64| ShapedGlyph { id, x, y: 0.0, advance: 1.0, text: String::new() };
    let run = ShapedText {
        glyphs: vec![glyph(u16::MAX, 0.0), glyph(40_000, 1.0), glyph(0, 2.0), glyph(5, f64::NAN)],
        width: 4.0,
        size_pt: 12.0,
    };
    let path = run.to_outline(&font, Point::new(0.0, 0.0));
    // Out-of-range ids draw nothing; whatever is drawn consists of whole contours.
    let moves = path.segs.iter().filter(|s| matches!(s, PathSeg::MoveTo(_))).count();
    let closes = path.segs.iter().filter(|s| matches!(s, PathSeg::Close)).count();
    assert_eq!(moves, closes);
    // NaN coordinates make the exact bounds undefined rather than wrong.
    assert_eq!(run.ink_bounds(&font, Point::new(0.0, 0.0)), None);
    let Item::Glyphs { glyphs, .. } = run.to_glyph_item(FontId(0), Color::BLACK, Point::default())
    else {
        panic!("expected a glyph run");
    };
    assert_eq!(glyphs.len(), 4);
}

#[test]
fn shaping_is_deterministic() {
    let font = common::spectral();
    let opts = ShapeOptions {
        letter_spacing_em: 0.02,
        language: Some("fr".into()),
        rtl: false,
        features: vec![("onum".into(), 1)],
    };
    let first = shape(&font, FRENCH, 13.3, &opts);
    for _ in 0..3 {
        let again = shape(&common::spectral(), FRENCH, 13.3, &opts);
        assert_eq!(again, first);
        assert_eq!(
            again.to_outline(&font, Point::new(1.0, 2.0)),
            first.to_outline(&font, Point::new(1.0, 2.0))
        );
    }
}
