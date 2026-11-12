//! Helpers shared by the integration tests: the bundled fonts and synthetic covers.
#![allow(dead_code)]

use ebc_fontid::synth::{Background, CoverSpec, Degradation, SynthCover, render_cover};
use ebc_fontid::{BUNDLED_FONT_FILES, FontIndex, IndexEntry, IndexEntryMeta};
use ebc_text::Font;

/// The eight bundled fonts with the metadata the bundled index records for them.
pub fn bundled_fonts() -> Vec<(IndexEntryMeta, Font)> {
    BUNDLED_FONT_FILES
        .iter()
        .map(|file| {
            let path = format!("{}/../../assets/fonts/{file}", env!("CARGO_MANIFEST_DIR"));
            let data = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let font = Font::from_bytes(data, 0).unwrap_or_else(|e| panic!("{path}: {e}"));
            let meta = IndexEntryMeta {
                family: font.family_name(),
                style: font.style_name(),
                licence: "OFL-1.1".to_string(),
                source_url: format!("assets/fonts/{file}"),
            };
            (meta, font)
        })
        .collect()
}

/// The index of the bundled fonts, built afresh (not the embedded bytes).
pub fn fresh_index(fonts: &[(IndexEntryMeta, Font)]) -> FontIndex {
    FontIndex::build_checked(fonts).expect("the bundled fonts are OFL and installable")
}

/// A clean 420 x 630 cover: flat background, no degradation.
pub fn clean_spec(lines: &[&str], cap_height_px: f32, seed: u64) -> CoverSpec {
    CoverSpec {
        width: 420,
        height: 630,
        title_lines: lines.iter().map(|l| (*l).to_string()).collect(),
        cap_height_px,
        letter_spacing_em: 0.0,
        light_on_dark: seed % 2 == 1,
        background: Background::Flat,
        author: Some("Camille Vasseur".to_string()),
        ornaments: seed % 3 == 0,
        degradation: Degradation::default(),
        seed,
    }
}

/// A "typical web cover": gradient, slight rotation, blur, noise, JPEG.
pub fn degraded_spec(lines: &[&str], cap_height_px: f32, seed: u64) -> CoverSpec {
    CoverSpec {
        background: Background::Gradient,
        letter_spacing_em: 0.02,
        degradation: Degradation {
            rotation_degrees: if seed % 2 == 0 { 0.8 } else { -0.8 },
            blur_sigma: 0.6,
            noise_sigma: 3.0,
            jpeg_quality: Some(75),
        },
        ..clean_spec(lines, cap_height_px, seed)
    }
}

/// Draws a cover with font `i` of `fonts` (the author line uses the next font).
pub fn cover(fonts: &[(IndexEntryMeta, Font)], i: usize, spec: &CoverSpec) -> SynthCover {
    render_cover(spec, &fonts[i].1, &fonts[(i + 1) % fonts.len()].1).expect("drawable cover")
}

/// Is `entry` the font with this metadata?
pub fn is(entry: &IndexEntry, meta: &IndexEntryMeta) -> bool {
    entry.family == meta.family && entry.style == meta.style
}
