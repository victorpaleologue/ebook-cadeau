//! TEMPORARY (font-stack migration): dumps everything observable through the public API
//! into text files, so that the output of the old and the new font stack can be diffed.
//! Deleted once the migration is done.
#![allow(missing_docs)]

mod common;

use std::fmt::Write as _;

use ebc_scene::{PathSeg, Point};
use ebc_text::{Font, ShapeOptions, ShapedGlyph, ShapedText, shape};

fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3))
}

const TEXTS: &[&str] = &[
    "L'Étranger",
    "Affinité efficace — fjord, affliction, office",
    "À bientôt, Zoë ! Œuvre « offerte » : l’été à Noël…",
    "AVATAR To Wa Ty P. LT",
    "e\u{301}a\u{300}c\u{327} n\u{303}",
    "fi fl ffi ffl ff",
    "Tab\there\nnew\u{2028}line",
    "soft\u{ad}hyphen zero\u{200b}width non\u{a0}breaking narrow\u{202f}space",
    "日本語 emoji 🎁 mixed",
    "1234567890 1/2 3×4 = 12 €",
    "The quick brown fox jumps over the lazy dog",
    "Voix ambiguë d’un cœur qui, au zéphyr, préfère les jattes de kiwis",
    "שלום עולם",
    "",
    " ",
    "İstanbul fıstık",
    "مرحبا بالعالم",
    "สวัสดีครับ ที่นี่",
    "Ǎ ǎ ș ț ḍ ṅ Ω π",
];

fn option_sets() -> Vec<(&'static str, ShapeOptions)> {
    let d = ShapeOptions::default();
    vec![
        ("default", d.clone()),
        ("spaced", ShapeOptions { letter_spacing_em: 0.125, ..d.clone() }),
        ("tight", ShapeOptions { letter_spacing_em: -0.0625, ..d.clone() }),
        ("rtl", ShapeOptions { rtl: true, ..d.clone() }),
        ("fr", ShapeOptions { language: Some("fr".into()), ..d.clone() }),
        ("tr", ShapeOptions { language: Some("tr".into()), ..d.clone() }),
        ("noliga", ShapeOptions { features: vec![("liga".into(), 0)], ..d.clone() }),
        ("nokern", ShapeOptions { features: vec![("kern".into(), 0)], ..d.clone() }),
        ("smcp", ShapeOptions { features: vec![("smcp".into(), 1)], ..d.clone() }),
        ("onum", ShapeOptions { features: vec![("onum".into(), 1), ("dlig".into(), 1)], ..d.clone() }),
        (
            "spaced+liga",
            ShapeOptions { letter_spacing_em: 0.125, features: vec![("liga".into(), 1)], ..d },
        ),
    ]
}

/// A size at which one font unit is one millimetre (up to rounding).
fn unit_size(font: &Font) -> f64 {
    f64::from(font.metrics().units_per_em) / ebc_scene::MM_PER_PT
}

fn dump_font(label: &str, font: &Font, out: &mut String, outlines: &mut String) {
    let _ = writeln!(out, "== {label}");
    let _ = writeln!(out, "family={:?} style={:?} full={:?} ps={:?}", font.family_name(), font.style_name(), font.full_name(), font.postscript_name());
    let _ = writeln!(out, "version={:?} licence={:?} url={:?}", font.version(), font.licence_description().map(|s| fnv(s.as_bytes())), font.licence_url());
    for id in 0..26u16 {
        let _ = writeln!(out, "name[{id}]={:?}", font.name(id).map(|s| if s.len() > 60 { format!("#{:x}", fnv(s.as_bytes())) } else { s }));
    }
    let _ = writeln!(out, "fs_type={:?} embedding={:?} flags={:?} bundling={} subset={} outlines={}", font.fs_type(), font.embedding(), font.embedding_flags(), font.allows_bundling(), font.allows_subset_embedding(), font.allows_outlines());
    let _ = writeln!(out, "metrics={:?}", font.metrics());
    let _ = writeln!(out, "glyphs={} format={:?} weight={} italic={} mono={} variable={} oversized={}", font.glyph_count(), font.outline_format(), font.weight(), font.is_italic(), font.is_monospaced(), font.is_variable(), font.has_oversized_glyphs());
    let _ = writeln!(out, "axes={:?} variations={:?}", font.axes(), font.variations());

    // Character map.
    let mut cmap = String::new();
    let mut mapped = 0u32;
    for c in (0..0x3_0000u32).filter_map(char::from_u32) {
        if let Some(g) = font.glyph_id(c) {
            let _ = write!(cmap, "{:x}:{g} ", c as u32);
            mapped += 1;
        }
    }
    let _ = writeln!(out, "cmap mapped={mapped} hash={:x}", fnv(cmap.as_bytes()));
    let _ = writeln!(outlines, "== {label}\ncmap {cmap}");

    let size = unit_size(font);
    let scale = f64::from(font.metrics().units_per_em).recip() * size * ebc_scene::MM_PER_PT;
    for text in TEXTS {
        let _ = writeln!(out, "missing {text:?} -> {:?}", font.missing_chars(text));
        for (name, opts) in option_sets() {
            let run = shape(font, text, size, &opts);
            let _ = write!(out, "shape {name} {text:?} w={:.4}:", run.width / scale);
            for g in &run.glyphs {
                let _ = write!(out, " {}@{:.4},{:.4}+{:.4}{:?}", g.id, g.x / scale, g.y / scale, g.advance / scale, g.text);
            }
            let _ = writeln!(out);
        }
        let opts = ShapeOptions::default();
        let run = shape(font, text, 12.0, &opts);
        let ink = run.ink_bounds(font, Point::new(10.0, 20.0));
        let _ = writeln!(out, "ink {text:?} {:?}", ink.map(|r| format!("{:.5} {:.5} {:.5} {:.5}", r.x, r.y, r.w, r.h)));
        let _ = writeln!(out, "wrap {text:?} {:?}", ebc_text::wrap(font, text, 12.0, 40.0, &opts));
        let _ = writeln!(out, "fit {text:?} {:?}", ebc_text::fit_size(font, text, 60.0, 5.0, 4.0, 40.0, &opts).map(|s| format!("{s:.6}")));
        #[cfg(feature = "raster")]
        {
            let r = ebc_text::try_rasterize(font, text, 48.0, &opts);
            match r {
                Ok(r) => {
                    let _ = writeln!(out, "raster {text:?} {}x{} left={} top={} adv={:.4} sum={} hash={:x}", r.mask.width, r.mask.height, r.left, r.top, r.advance, r.mask.data.iter().map(|&b| u64::from(b)).sum::<u64>(), fnv(&r.mask.data));
                }
                Err(e) => {
                    let _ = writeln!(out, "raster {text:?} ERR {e:?}");
                }
            }
        }
    }

    // Every glyph's outline, in font units.
    let mut total = 0usize;
    for id in 0..font.glyph_count() {
        let glyph = ShapedGlyph { id, x: 0.0, y: 0.0, advance: 0.0, text: String::new() };
        let run = ShapedText { glyphs: vec![glyph], width: 0.0, size_pt: size };
        let path = run.try_to_outline(font, Point::new(0.0, 0.0));
        let _ = write!(outlines, "G{id}");
        match path {
            Ok(path) => {
                total += path.segs.len();
                for seg in &path.segs {
                    let p = |p: &Point| format!("{:.3},{:.3}", p.x / scale, -p.y / scale + 0.0);
                    let _ = match seg {
                        PathSeg::MoveTo(a) => write!(outlines, " M{}", p(a)),
                        PathSeg::LineTo(a) => write!(outlines, " L{}", p(a)),
                        PathSeg::QuadTo(a, b) => write!(outlines, " Q{} {}", p(a), p(b)),
                        PathSeg::CubicTo(a, b, c) => write!(outlines, " C{} {} {}", p(a), p(b), p(c)),
                        PathSeg::Close => write!(outlines, " Z"),
                    };
                }
            }
            Err(e) => {
                let _ = write!(outlines, " ERR {e:?}");
            }
        }
        let _ = writeln!(outlines);
    }
    let _ = writeln!(out, "outline segments total={total}");
}

#[test]
#[ignore = "temporary migration tool: needs EBC_DUMP_DIR"]
fn dump_everything() {
    let Ok(dir) = std::env::var("EBC_DUMP_DIR") else { return };
    let mut out = String::new();
    let mut outlines = String::new();
    for (label, bytes) in common::ALL {
        dump_font(label, &common::font(bytes), &mut out, &mut outlines);
    }
    // Synthetic fonts: static, variable instances, CFF, collection.
    let mut synthetic = common::SyntheticFont::new();
    synthetic.os2 = Some((4, 0, true));
    synthetic.names = vec![
        common::NameRecord::windows(1, "Synth"),
        common::NameRecord::windows(2, "Regular"),
        common::NameRecord::mac(4, b"Synth Caf\x8e"),
        common::NameRecord::windows(6, "Synth-Regular"),
    ];
    dump_font("synthetic", &Font::from_bytes(synthetic.build(), 0).expect("parses"), &mut out, &mut outlines);
    let mut bare = common::SyntheticFont::new();
    bare.ascender = 0;
    bare.descender = 0;
    dump_font("synthetic bare", &Font::from_bytes(bare.build(), 0).expect("parses"), &mut out, &mut outlines);
    synthetic.variable = true;
    let variable = Font::from_bytes(synthetic.build(), 0).expect("parses");
    for wght in [100.0f32, 250.0, 400.0, 650.0, 900.0, 2000.0] {
        let instance = variable.with_variations(&[("wght", wght)]);
        dump_font(&format!("synthetic wght={wght}"), &instance, &mut out, &mut outlines);
    }
    let collection = common::SyntheticFont::collection(&[&bare, &synthetic]);
    dump_font("collection face 1", &Font::from_bytes(collection, 1).expect("parses"), &mut out, &mut outlines);
    dump_font("synthetic cff", &Font::from_bytes(common::synthetic_cff_font(), 0).expect("parses"), &mut out, &mut outlines);

    // Extra font files from the machine (not fixtures).
    if let Ok(list) = std::env::var("EBC_DUMP_EXTRA") {
        for file in list.split(':').filter(|f| !f.is_empty()) {
            let bytes = std::fs::read(file).expect("readable font");
            for index in 0..4 {
                match Font::from_bytes(bytes.clone(), index) {
                    Ok(font) => {
                        dump_font(&format!("{file}#{index}"), &font, &mut out, &mut outlines);
                        if font.is_variable() {
                            let settings: Vec<(String, f32)> = font
                                .axes()
                                .iter()
                                .map(|a| (a.tag.clone(), a.default + (a.max - a.default) * 0.6))
                                .collect();
                            let settings: Vec<(&str, f32)> =
                                settings.iter().map(|(t, v)| (t.as_str(), *v)).collect();
                            dump_font(&format!("{file}#{index} towards max"), &font.with_variations(&settings), &mut out, &mut outlines);
                            let settings: Vec<(String, f32)> = font
                                .axes()
                                .iter()
                                .map(|a| (a.tag.clone(), a.default + (a.min - a.default) * 0.45))
                                .collect();
                            let settings: Vec<(&str, f32)> =
                                settings.iter().map(|(t, v)| (t.as_str(), *v)).collect();
                            dump_font(&format!("{file}#{index} towards min"), &font.with_variations(&settings), &mut out, &mut outlines);
                        }
                    }
                    Err(e) => {
                        let _ = writeln!(out, "== {file}#{index}\nERR {e:?}");
                    }
                }
                if !bytes.starts_with(b"ttcf") {
                    break;
                }
            }
        }
    }
    std::fs::write(format!("{dir}/summary.txt"), out).expect("writable");
    std::fs::write(format!("{dir}/outlines.txt"), outlines).expect("writable");
}
