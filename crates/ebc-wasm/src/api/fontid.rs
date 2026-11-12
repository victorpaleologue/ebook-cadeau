//! The closest **open** font for the lettering of a cover the user supplied.
//!
//! The answer is always "the nearest font of the index" — never "the font of the
//! cover", and never generated lettering: the title on the card is typeset from the
//! metadata in a font the project may redistribute. With the bundled index the
//! candidates are the eight SIL OFL fonts of `assets/fonts`; the suggestion maps the
//! best match to one of the title faces the card can use
//! ([`super::card::TITLE_FONTS`]).
//!
//! Only the fast descriptor stage of `ebc-fontid` runs here (no render-and-compare
//! re-ranking). Without the cargo feature `fontid` nothing is identified and
//! [`AVAILABLE`] is `false`: the page then says that font detection is unavailable.

use ebc_scene::Pixmap;
#[cfg(feature = "fontid")]
use serde::{Deserialize, Serialize};

use super::ApiError;

/// Whether font identification is compiled in (cargo feature `fontid`).
pub const AVAILABLE: bool = cfg!(feature = "fontid");

/// Longest side of the image the lettering is read from, in pixels.
#[cfg(feature = "fontid")]
const FONTID_SIDE_PX: u32 = 1200;

/// Largest font index accepted from a page, in bytes.
pub const MAX_INDEX_BYTES: usize = 32 * 1024 * 1024;

/// Text regions of an analysed image as `(x, y, w, h, score)`, best first; empty when
/// font identification is not compiled in.
#[cfg(feature = "fontid")]
pub(crate) fn text_regions(image: &Pixmap) -> Vec<(u32, u32, u32, u32, f32)> {
    ebc_fontid::find_text_regions(image)
        .into_iter()
        .take(16)
        .map(|r| (r.x, r.y, r.w, r.h, r.score))
        .collect()
}

/// Text regions of an analysed image; empty: font identification is not compiled in.
#[cfg(not(feature = "fontid"))]
pub(crate) fn text_regions(_image: &Pixmap) -> Vec<(u32, u32, u32, u32, f32)> {
    Vec::new()
}

/// A region of the cover, in pixels of the original image.
#[cfg(feature = "fontid")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegionDto {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    #[serde(default)]
    score: f32,
}

#[cfg(feature = "fontid")]
#[derive(Serialize)]
struct MatchDto {
    family: String,
    style: String,
    licence: String,
    source_url: String,
    /// Similarity in `0..=1`.
    score: f32,
    /// The title face of the card this font stands for, if it is one of them.
    title_font: Option<&'static str>,
}

#[cfg(feature = "fontid")]
#[derive(Serialize)]
struct SuggestionDto {
    /// One of the ids of `card::TITLE_FONTS`.
    title_font: &'static str,
    family: String,
    /// `false`: "the nearest we have", not a match — keep the house face unless the
    /// user prefers this one.
    confident: bool,
}

#[cfg(feature = "fontid")]
#[derive(Serialize)]
struct IdentifyDto {
    available: bool,
    /// The region that was read, in pixels of the original image; `null` when no text
    /// was found.
    region: Option<RegionDto>,
    matches: Vec<MatchDto>,
    confident: bool,
    labelled_glyphs: usize,
    glyphs: usize,
    lines: usize,
    light_on_dark: bool,
    suggestion: Option<SuggestionDto>,
}

/// The title face that stands for an indexed font: the same file, else the same family.
#[cfg(feature = "fontid")]
fn title_font_for(family: &str, source_url: &str) -> Option<&'static str> {
    use super::card::TITLE_FONTS;
    let file = source_url.rsplit('/').next().unwrap_or(source_url);
    if let Some((id, _)) = TITLE_FONTS.iter().find(|(_, f)| *f == file) {
        return Some(id);
    }
    let family = family.to_ascii_lowercase();
    [
        ("spectral", "spectral_semibold"),
        ("bebas", "bebas_neue"),
        ("atkinson", "atkinson_bold"),
        ("courier prime", "courier_prime_bold"),
    ]
    .into_iter()
    .find(|(prefix, _)| family.starts_with(prefix))
    .map(|(_, id)| id)
}

/// Finds the title on a cover **the user supplied** and ranks the fonts of an index of
/// openly licensed fonts by resemblance to its lettering.
///
/// `region_json` is `{"x", "y", "w", "h"}` in pixels of the original image, or an empty
/// string (or `null`) to let the title be found; `title` is the title as the catalogue
/// has it (may be empty, which is less reliable); `index` is a serialised
/// `ebc_fontid::FontIndex`, or `None` for the bundled one.
///
/// Returns `{"available": true, "region", "matches": [{"family", "style", "licence",
/// "source_url", "score", "title_font"}], "confident", "labelled_glyphs", "glyphs",
/// "lines", "light_on_dark", "suggestion": {"title_font", "family", "confident"}}` —
/// or `{"available": false}` when the cargo feature `fontid` is off.
///
/// # Errors
/// `cover_decode`, `cover_too_large`, `invalid_json`, `invalid_font_index`,
/// `text_too_long`.
#[cfg(feature = "fontid")]
pub fn fontid_identify(
    cover: &[u8],
    region_json: &str,
    title: &str,
    index: Option<&[u8]>,
) -> Result<String, ApiError> {
    use ebc_fontid::{FontIndex, TextRegion};

    if title.len() > super::card::MAX_TEXT_BYTES {
        return Err(ApiError::new("text_too_long", "the title is too long"));
    }
    let wanted: Option<RegionDto> = super::from_json_or_default("region", region_json)?;
    let index = match index.filter(|bytes| !bytes.is_empty()) {
        Some(bytes) if bytes.len() > MAX_INDEX_BYTES => {
            return Err(ApiError::new("invalid_font_index", "the index is too large"));
        }
        Some(bytes) => FontIndex::from_bytes(bytes),
        None => ebc_fontid::bundled_index(),
    }
    .map_err(|e| ApiError::new("invalid_font_index", e.to_string()))?;

    let analysed = super::cover::decode_scaled(cover, FONTID_SIDE_PX)?;
    let scale = analysed.scale();
    let image = &analysed.pixmap;
    let title = title.trim();
    let known = (!title.is_empty()).then_some(title);
    let down = |v: u32| (f64::from(v) / scale).round() as u32;
    let up = |v: u32| (f64::from(v) * scale).round() as u32;

    let region = match wanted {
        Some(r) => Some(TextRegion {
            x: down(r.x),
            y: down(r.y),
            w: down(r.w).max(1),
            h: down(r.h).max(1),
            score: 1.0,
        }),
        None => match known {
            Some(title) => ebc_fontid::find_title_regions(image, title).into_iter().next(),
            None => ebc_fontid::find_text_regions(image).into_iter().next(),
        },
    };
    let Some(region) = region else {
        return super::to_json(&IdentifyDto {
            available: true,
            region: None,
            matches: Vec::new(),
            confident: false,
            labelled_glyphs: 0,
            glyphs: 0,
            lines: 0,
            light_on_dark: false,
            suggestion: None,
        });
    };
    let found = ebc_fontid::identify_detailed(image, &region, known, &index);
    let matches: Vec<MatchDto> = found
        .matches
        .iter()
        .take(8)
        .map(|m| MatchDto {
            title_font: title_font_for(&m.entry.family, &m.entry.source_url),
            family: m.entry.family.clone(),
            style: m.entry.style.clone(),
            licence: m.entry.licence.clone(),
            source_url: m.entry.source_url.clone(),
            score: m.score,
        })
        .collect();
    let suggestion = matches.iter().find_map(|m| {
        m.title_font.map(|id| SuggestionDto {
            title_font: id,
            family: m.family.clone(),
            // Confident only when the best match itself is a face the card has.
            confident: found.confident
                && matches.first().is_some_and(|first| first.title_font == Some(id)),
        })
    });
    super::to_json(&IdentifyDto {
        available: true,
        region: Some(RegionDto {
            x: up(region.x),
            y: up(region.y),
            w: up(region.w),
            h: up(region.h),
            score: region.score,
        }),
        matches,
        confident: found.confident,
        labelled_glyphs: found.labelled_glyphs,
        glyphs: found.glyphs,
        lines: found.lines,
        light_on_dark: found.light_on_dark,
        suggestion,
    })
}

/// Font identification is not compiled in (cargo feature `fontid` is off): answers
/// `{"available": false}` whatever the arguments.
///
/// # Errors
/// None.
#[cfg(not(feature = "fontid"))]
pub fn fontid_identify(
    _cover: &[u8],
    _region_json: &str,
    _title: &str,
    _index: Option<&[u8]>,
) -> Result<String, ApiError> {
    Ok(r#"{"available":false}"#.to_owned())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn value(json: &str) -> Value {
        serde_json::from_str(json).expect("the API returns JSON")
    }

    #[cfg(not(feature = "fontid"))]
    #[test]
    fn without_the_feature_it_says_unavailable() {
        const { assert!(!AVAILABLE) };
        assert_eq!(value(&fontid_identify(b"", "", "x", None).expect("ok"))["available"], false);
    }

    /// A cover with `title` set in the font of `file`, dark on cream, large and centred.
    #[cfg(feature = "fontid")]
    fn lettered_cover(file: &[u8], title: &str) -> Vec<u8> {
        use ebc_text::{Font, ShapeOptions, rasterize};
        let font = Font::from_bytes(file.to_vec(), 0).expect("a bundled font");
        let (w, h) = (900u32, 1350u32);
        let mask = rasterize(&font, title, 150.0, &ShapeOptions::default());
        assert!(mask.width > 0 && mask.width < w, "the title fits: {}", mask.width);
        let mut image = image::RgbImage::from_pixel(w, h, image::Rgb([244, 240, 228]));
        let (ox, oy) = ((w - mask.width) / 2, 260u32);
        for y in 0..mask.height {
            for x in 0..mask.width {
                let coverage = u32::from(mask.get(x, y));
                let mix = |bg: u8, fg: u8| {
                    ((u32::from(bg) * (255 - coverage) + u32::from(fg) * coverage) / 255) as u8
                };
                let px = image.get_pixel_mut(ox + x, oy + y);
                *px = image::Rgb([mix(px[0], 20), mix(px[1], 24), mix(px[2], 40)]);
            }
        }
        let mut out = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .expect("encoding a PNG in memory");
        out
    }

    #[cfg(feature = "fontid")]
    #[test]
    fn the_nearest_open_face_is_suggested_for_clean_lettering() {
        let cases: [(&[u8], &str); 2] = [
            (include_bytes!("../../../../assets/fonts/BebasNeue-Regular.ttf"), "bebas_neue"),
            (include_bytes!("../../../../assets/fonts/Spectral-SemiBold.ttf"), "spectral_semibold"),
        ];
        for (file, expected) in cases {
            let cover = lettered_cover(file, "Les Vagues");
            let v = value(&fontid_identify(&cover, "", "Les Vagues", None).expect("identified"));
            assert_eq!(v["available"], true);
            let region = &v["region"];
            assert!(region["w"].as_u64().expect("a region") > 200, "{region}");
            let matches = v["matches"].as_array().expect("matches");
            assert!(!matches.is_empty());
            assert!(matches.iter().all(|m| m["licence"].as_str().is_some_and(|l| l.contains("OFL"))), "{matches:?}");
            assert_eq!(v["suggestion"]["title_font"], expected, "{}", v["matches"]);
            // The same answer for a region drawn by the user around the title.
            let drawn = fontid_identify(&cover, &region.to_string(), "Les Vagues", None).expect("region");
            assert_eq!(value(&drawn)["suggestion"]["title_font"], expected);
        }
    }

    #[cfg(feature = "fontid")]
    #[test]
    fn hostile_input_is_refused_not_fatal() {
        const { assert!(AVAILABLE) };
        assert_eq!(fontid_identify(b"junk", "", "Title", None).expect_err("junk").code, "cover_decode");
        let blank = super::super::cover::tests::png(120, 180, |_, _| [255, 255, 255]);
        let v = value(&fontid_identify(&blank, "", "Title", None).expect("blank"));
        assert_eq!(v["available"], true);
        assert_eq!(v["matches"].as_array().map(Vec::len), Some(0));
        assert_eq!(v["suggestion"], Value::Null);
        // A region outside the image, a region of nonsense, a corrupt index.
        let outside = r#"{"x": 100000, "y": 100000, "w": 5, "h": 5}"#;
        assert!(fontid_identify(&blank, outside, "Title", None).is_ok());
        assert_eq!(fontid_identify(&blank, "{\"x\": -1}", "Title", None).expect_err("bad").code, "invalid_json");
        assert_eq!(
            fontid_identify(&blank, "", "Title", Some(b"not an index")).expect_err("index").code,
            "invalid_font_index"
        );
        assert_eq!(
            fontid_identify(&blank, "", &"t".repeat(10_000), None).expect_err("long").code,
            "text_too_long"
        );
    }

    #[cfg(feature = "fontid")]
    #[test]
    fn title_faces_stand_for_their_families() {
        assert_eq!(title_font_for("Spectral", "assets/fonts/Spectral-Italic.ttf"), Some("spectral_semibold"));
        assert_eq!(title_font_for("Bebas Neue", "assets/fonts/BebasNeue-Regular.ttf"), Some("bebas_neue"));
        assert_eq!(title_font_for("Atkinson Hyperlegible", "x/AtkinsonHyperlegible-Regular.ttf"), Some("atkinson_bold"));
        assert_eq!(title_font_for("Courier Prime", ""), Some("courier_prime_bold"));
        assert_eq!(title_font_for("Helvetica", "https://example.org/h.ttf"), None);
    }
}
