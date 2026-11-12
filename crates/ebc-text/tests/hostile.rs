//! Hostile input: truncated and corrupted fonts, pathological text and options.
//! Nothing here may panic, hang or lose text.

mod common;

use common::Rng;
use ebc_scene::Point;
use ebc_text::{Font, ShapeOptions, fit_size, outline_bounds, shape, wrap, wrap_ranges};

const TEXT: &str = "L'Étranger — efficace œuvre, AV fi\u{a0}! e\u{301} 你";

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Runs the whole API on a font that parsed but may be arbitrarily broken inside, and
/// checks the properties that must hold for *any* font.
fn exercise(font: &Font, label: &str) {
    let opts = ShapeOptions::default();
    let _ = (font.family_name(), font.full_name(), font.postscript_name(), font.name(0));
    let _ = (font.licence_description(), font.licence_url(), font.embedding(), font.fs_type());
    let _ = (font.axes(), font.variations(), font.weight(), font.outline_format());
    let metrics = font.metrics();
    assert!((16..=16384).contains(&metrics.units_per_em), "{label}: {metrics:?}");
    assert!(metrics.cap_height_mm(12.0).is_finite() && metrics.line_height_mm(12.0).is_finite());

    let missing = font.missing_chars(TEXT);
    assert!(missing.iter().all(|c| TEXT.contains(*c)), "{label}");

    for candidate in [font.clone(), font.with_variations(&[("wght", 650.0), ("wdth", 80.0)])] {
        let run = shape(&candidate, TEXT, 12.0, &opts);
        let reassembled: String = run.glyphs.iter().map(|g| g.text.as_str()).collect();
        assert!(run.glyphs.is_empty() || reassembled == TEXT, "{label}: {reassembled:?}");
        assert!(run.width.is_finite(), "{label}");
        assert!(run.glyphs.iter().all(|g| g.x.is_finite() && g.y.is_finite()), "{label}");

        let outline = run.to_outline(&candidate, Point::new(10.0, 20.0));
        if let Some(bounds) = outline_bounds(&outline) {
            assert!(bounds.w.is_finite() && bounds.h.is_finite(), "{label}");
        }

        if let Some(size) = fit_size(&candidate, TEXT, 80.0, 5.0, 4.0, 40.0, &opts) {
            assert!((4.0..=40.0).contains(&size), "{label}: {size}");
            assert!(shape(&candidate, TEXT, size, &opts).width <= 80.0, "{label}");
        }

        let lines = wrap(&candidate, TEXT, 12.0, 30.0, &opts);
        assert_eq!(squeeze(&lines.concat()), squeeze(TEXT), "{label}: wrap lost text");
        let ranges = wrap_ranges(&candidate, TEXT, 12.0, 30.0, &opts);
        assert_eq!(ranges.first().map(|r| r.start), Some(0), "{label}");
        assert_eq!(ranges.last().map(|r| r.end), Some(TEXT.len()), "{label}");

        #[cfg(feature = "raster")]
        if let Ok(raster) = ebc_text::try_rasterize(&candidate, "Été fi AV", 24.0, &opts) {
            let mask = raster.mask;
            assert_eq!(mask.data.len(), mask.width as usize * mask.height as usize, "{label}");
            assert!(raster.advance.is_finite(), "{label}");
        }
    }
}

#[test]
fn truncated_fonts_are_rejected_or_degrade_gracefully() {
    let mut accepted = 0;
    for (label, bytes) in [
        ("Atkinson", common::ATKINSON),
        ("Spectral", common::SPECTRAL),
        ("Courier", common::COURIER_BOLD),
    ] {
        // Every prefix in the table directory, then one cut every ~1.6 kB.
        let cuts = (0..400).chain((400..bytes.len()).step_by(1607));
        for cut in cuts {
            if let Ok(font) = Font::from_bytes(&bytes[..cut], 0) {
                accepted += 1;
                exercise(&font, &format!("{label} cut at {cut}"));
            }
        }
    }
    // Truncating after the table directory leaves a parseable font with missing
    // tables: the test must really have exercised such fonts.
    assert!(accepted > 20, "only {accepted} truncated fonts were accepted");
}

#[test]
fn the_synthetic_font_survives_every_truncation() {
    let bytes = common::SyntheticFont {
        variable: true,
        os2: Some((4, 0, true)),
        names: vec![common::NameRecord::windows(1, "Fuzz")],
        ..common::SyntheticFont::new()
    }
    .build();
    for cut in 0..bytes.len() {
        if let Ok(font) = Font::from_bytes(&bytes[..cut], 0) {
            exercise(&font, &format!("synthetic cut at {cut}"));
        }
    }
    exercise(&Font::from_bytes(bytes, 0).expect("whole font"), "synthetic");
}

/// Overwrites a few bytes of `font`, preferably inside one table, with values that
/// parsers find hard to digest.
fn corrupt(font: &mut [u8], rng: &mut Rng) {
    let table_count = u16::from_be_bytes([font[4], font[5]]) as usize;
    let (start, length) = if rng.below(8) == 0 {
        (0, font.len()) // anywhere, including the table directory
    } else {
        let record = 12 + 16 * rng.below(table_count);
        let offset = u32::from_be_bytes([
            font[record + 8],
            font[record + 9],
            font[record + 10],
            font[record + 11],
        ]);
        let length = u32::from_be_bytes([
            font[record + 12],
            font[record + 13],
            font[record + 14],
            font[record + 15],
        ]);
        (offset as usize, length as usize)
    };
    if length == 0 || start + length > font.len() {
        return;
    }
    for _ in 0..1 + rng.below(6) {
        // Bias towards the beginning of the table, where headers and counts live.
        let span = if rng.below(2) == 0 { length.min(64) } else { length };
        let at = start + rng.below(span);
        let poison = [0x00, 0xFF, 0x7F, 0x80, 0x01, rng.next_u64() as u8];
        let run = 1 + rng.below(4);
        for byte in font[at..(at + run).min(start + length)].iter_mut() {
            *byte = poison[rng.below(poison.len())];
        }
    }
}

#[test]
fn corrupted_fonts_never_panic() {
    let synthetic = common::SyntheticFont {
        variable: true,
        os2: Some((4, 0, true)),
        names: vec![common::NameRecord::windows(1, "Fuzz"), common::NameRecord::mac(4, b"Fuzz")],
        ..common::SyntheticFont::new()
    }
    .build();
    let sources: [(&str, &[u8], usize); 4] = [
        ("Atkinson", common::ATKINSON, 150),
        ("Spectral", common::SPECTRAL, 100),
        ("Bebas", common::BEBAS, 100),
        ("synthetic", &synthetic, 400),
    ];
    let mut rng = Rng(0x00C0_FFEE_D15E_A5E5);
    let (mut parsed, mut rejected) = (0, 0);
    for (label, bytes, rounds) in sources {
        for round in 0..rounds {
            let mut mutated = bytes.to_vec();
            corrupt(&mut mutated, &mut rng);
            match Font::from_bytes(mutated, 0) {
                Ok(font) => {
                    parsed += 1;
                    exercise(&font, &format!("{label} round {round}"));
                }
                Err(_) => rejected += 1,
            }
        }
    }
    // The corruption is mild enough that most fonts still load — which is the
    // interesting case — and harsh enough that some do not.
    assert!(parsed > 400 && rejected > 0, "{parsed} parsed, {rejected} rejected");
}

#[test]
fn pathological_text_is_shaped_and_wrapped_without_loss() {
    let font = common::atkinson();
    let opts = ShapeOptions::default();
    let marks = format!("a{}", "\u{301}".repeat(5_000));
    let joiners = "👩\u{200d}".repeat(2_000);
    let long_line = "Lorem ipsum dolor sit amet. ".repeat(4_000); // 112 000 bytes
    let controls: String =
        (0u8..32).map(char::from).chain("\u{7f}\u{85}\u{2028}".chars()).collect();
    let texts = [
        marks.as_str(),
        joiners.as_str(),
        long_line.as_str(),
        controls.as_str(),
        "\u{202e}\u{202d}\u{2066}\u{2069}\u{200f}abc",
        "\u{ffff}\u{10ffff}\u{e000}\u{fdd0}",
        "\u{feff}\u{feff}\u{feff}",
        "اللغة العربية שלום עולם हिन्दी ไทย 日本語",
    ];
    for text in texts {
        let label: String = text.chars().take(12).collect();
        let run = shape(&font, text, 10.0, &opts);
        let reassembled: String = run.glyphs.iter().map(|g| g.text.as_str()).collect();
        assert_eq!(reassembled, text, "{label:?}");
        assert!(run.width.is_finite());

        for max_width in [0.0, 25.0, 1e9] {
            let ranges = wrap_ranges(&font, text, 10.0, max_width, &opts);
            let mut at = 0;
            for range in &ranges {
                assert_eq!(range.start, at, "{label:?}");
                at = range.end;
            }
            assert_eq!(at, text.len(), "{label:?} at {max_width}");
        }
        let lines = wrap(&font, text, 10.0, 25.0, &opts);
        assert_eq!(squeeze(&lines.concat()), squeeze(text), "{label:?}");
        let _ = fit_size(&font, text, 100.0, 5.0, 1.0, 50.0, &opts);
        let _ = font.missing_chars(text);
    }
}

#[test]
fn pathological_options_are_harmless() {
    let font = common::spectral();
    let many_features: Vec<(String, u32)> =
        (0..5_000).map(|i| (format!("f{:03}", i % 1000), u32::MAX - i)).collect();
    let option_sets = [
        ShapeOptions { features: many_features, ..ShapeOptions::default() },
        ShapeOptions { letter_spacing_em: f32::MAX, ..ShapeOptions::default() },
        ShapeOptions { letter_spacing_em: f32::MIN, ..ShapeOptions::default() },
        ShapeOptions { letter_spacing_em: f32::NAN, rtl: true, ..ShapeOptions::default() },
        ShapeOptions { language: Some("\0\u{ffff}".repeat(100)), ..ShapeOptions::default() },
        ShapeOptions {
            features: vec![("liga".into(), u32::MAX), ("kern".into(), 7), ("\0\0\0\0".into(), 1)],
            ..ShapeOptions::default()
        },
    ];
    for opts in &option_sets {
        let run = shape(&font, TEXT, 12.0, opts);
        let logical: String = if opts.rtl {
            run.glyphs.iter().rev().map(|g| g.text.as_str()).collect()
        } else {
            run.glyphs.iter().map(|g| g.text.as_str()).collect()
        };
        assert_eq!(logical, TEXT);
        assert!(run.width.is_finite(), "{}", run.width);
        assert!(run.glyphs.iter().all(|g| g.x.is_finite()));
        let _ = run.to_outline(&font, Point::new(0.0, 0.0));
        let lines = wrap(&font, TEXT, 12.0, 40.0, opts);
        assert_eq!(squeeze(&lines.concat()), squeeze(TEXT));
        let _ = fit_size(&font, TEXT, 40.0, 5.0, 1.0, 50.0, opts);
        #[cfg(feature = "raster")]
        let _ = ebc_text::rasterize(&font, TEXT, 32.0, opts);
    }
}
