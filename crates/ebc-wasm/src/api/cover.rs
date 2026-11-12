//! Covers: where one can be **looked at** (candidates — never printable), what an image
//! file is worth for print, what a product page says, and the colours of an image the
//! user supplied.
//!
//! Nothing here turns a looked-up cover into a printable one: candidates carry no
//! pixels and always have the rights basis `none`. A cover enters a card only as bytes
//! the user picked, with the rights basis the user declared (see [`super::card`]).

use std::io::Cursor;

use ebc_color::{Edge, Rgb8, contrast_ratio, edge_palette, edge_uniformity, palette};
use ebc_covers::quality::Thresholds;
use ebc_covers::scrape::Via;
use ebc_covers::{CandidateRole, Policy, Risk, Verdict};
use ebc_layout::{CardFormat, PrintProfile, safe_rect};
use ebc_model::{BookMeta, CoverKind};
use ebc_scene::Pixmap;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};
use serde::{Deserialize, Serialize};

use super::{ApiError, from_json, from_json_or_default, to_json};

// ---- Candidates ------------------------------------------------------------------------

/// What the page may choose about the cascade. The risk ceiling is not one of them: it
/// stays at the hosted service's level, and no operator rights basis can be set.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PolicyDto {
    /// Source ids to use, in lookup order; default: the hosted cascade.
    allowed_sources: Option<Vec<String>>,
    /// Google Books API key; without one that source is skipped.
    google_books_api_key: Option<String>,
    /// Related ISBNs (other editions) to look up as well; default 1, at most 4.
    max_related_isbns: Option<usize>,
}

impl PolicyDto {
    fn to_policy(&self) -> Policy {
        let mut policy = Policy::hosted();
        if let Some(sources) = &self.allowed_sources {
            policy.allowed_sources = sources.iter().take(32).cloned().collect();
        }
        policy.google_books_api_key =
            self.google_books_api_key.clone().filter(|key| !key.trim().is_empty());
        if let Some(related) = self.max_related_isbns {
            policy.max_related_isbns = related.min(4);
        }
        policy
    }
}

#[derive(Serialize)]
struct CandidateDto {
    url: String,
    method: &'static str,
    /// `"locate"`, `"probe"`, `"preview"`, `"link"` or `"page"`.
    role: &'static str,
    source: &'static str,
    source_name: &'static str,
    source_homepage: &'static str,
    kind: CoverKind,
    risk: String,
    note: &'static str,
    isbn: Option<String>,
    other_edition: bool,
    declared_px: Option<(u32, u32)>,
    min_interval_ms: u32,
    /// Always `"none"`: a lookup never yields a right to print.
    rights_basis: &'static str,
    /// Always `false`, for the same reason.
    printable: bool,
}

fn role_name(role: CandidateRole) -> Option<&'static str> {
    Some(match role {
        CandidateRole::Locate => "locate",
        CandidateRole::Probe => "probe",
        CandidateRole::Preview => "preview",
        CandidateRole::Link => "link",
        CandidateRole::Page => "page",
        // Operator downloads are never produced for a page.
        CandidateRole::Download => return None,
    })
}

/// Where the cover of `book` can be located and looked at, as the steps of the hosted
/// cascade.
///
/// `book_json` is a `BookMeta`; `policy_json` is `{"allowed_sources": [id],
/// "google_books_api_key", "max_related_isbns"}` (all optional; an empty string is the
/// default).
///
/// Returns `[{"url", "method", "role", "source", "source_name", "source_homepage",
/// "kind", "risk", "note", "isbn", "other_edition", "declared_px", "min_interval_ms",
/// "rights_basis": "none", "printable": false}]`. A `preview` URL goes into an `<img>`
/// (never a canvas, never WebAssembly); a `link` URL is shown as a plain link. **No
/// candidate may be drawn into a card.**
///
/// # Errors
/// `invalid_json`.
pub fn cover_candidates(book_json: &str, policy_json: &str) -> Result<String, ApiError> {
    let book: BookMeta = from_json("book", book_json)?;
    let policy: PolicyDto = from_json_or_default("policy", policy_json)?;
    let list: Vec<CandidateDto> = ebc_covers::candidates(&book, &policy.to_policy())
        .into_iter()
        .filter(|c| c.risk <= Risk::Medium)
        .filter_map(|c| {
            let role = role_name(c.role)?;
            let info = c.source_info();
            Some(CandidateDto {
                url: c.request.url.clone(),
                method: c.request.method.as_str(),
                role,
                source: c.source,
                source_name: info.map_or(c.source, |i| i.name),
                source_homepage: info.map_or("", |i| i.homepage),
                kind: c.kind,
                risk: c.risk.to_string(),
                note: c.note,
                isbn: c.isbn.map(|isbn| isbn.to_plain()),
                other_edition: c.other_edition,
                declared_px: c.declared_px,
                min_interval_ms: c.min_interval_ms,
                rights_basis: c.rights_basis().as_str(),
                printable: false,
            })
        })
        .collect();
    to_json(&list)
}

// ---- Probing an image file -------------------------------------------------------------

#[derive(Serialize)]
struct ProbeDto {
    /// `"jpeg"`, `"png"`, `"gif"`, `"webp"`, `"avif"` or `"heif"`.
    format: &'static str,
    mime: &'static str,
    /// Pixel size as displayed (EXIF orientation applied).
    width: u32,
    height: u32,
    animated: bool,
    /// The card composer reads JPEG, PNG and WebP.
    usable_for_card: bool,
    /// A "no cover available" image.
    placeholder: bool,
    /// Not certainly a placeholder, but odd (shape, size, weight).
    suspicious: bool,
    reasons: Vec<String>,
    quality: QualityDto,
}

#[derive(Serialize)]
struct QualityDto {
    /// `"excellent"`, `"acceptable"`, `"low"` or `"insufficient"`.
    verdict: &'static str,
    effective_ppi: f64,
    target_ppi: f64,
    warn_ppi: f64,
    min_ppi: f64,
    /// Size the image takes inside the front panel's safe area, in millimetres.
    placed_mm: (f64, f64),
    /// The box it is fitted in, in millimetres.
    box_mm: (f64, f64),
}

fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Insufficient => "insufficient",
        Verdict::Low => "low",
        Verdict::Acceptable => "acceptable",
        Verdict::Excellent => "excellent",
    }
}

/// Rounds for display: two decimals are plenty for millimetres and pixels per inch.
fn round2(value: f64) -> f64 {
    if value.is_finite() { (value * 100.0).round() / 100.0 } else { 0.0 }
}

pub(crate) fn format_by_name(name: &str) -> Result<CardFormat, ApiError> {
    CardFormat::by_name(name).ok_or_else(|| {
        ApiError::new(
            "unknown_format",
            format!("expected one of {}", CardFormat::PRESET_NAMES.join(", ")),
        )
    })
}

pub(crate) fn profile_by_name(name: &str) -> Result<PrintProfile, ApiError> {
    // `home_custom` and other free-form names are not offered to pages.
    if name.len() > 32 {
        return Err(ApiError::new("unknown_profile", "not a profile name"));
    }
    PrintProfile::by_name(name).ok_or_else(|| {
        ApiError::new(
            "unknown_profile",
            format!("expected one of {}", PrintProfile::NAMES.join(", ")),
        )
    })
}

/// Judges an image file from its header: format, pixel size, whether it is a "no cover"
/// placeholder, and how it would print on the front panel of `format` (`"poche"`,
/// `"a6"`, `"digest"`, `"a5"`) with `profile` (`"upload_eu"`, `"professional_eu"`,
/// `"home_a4"`, `"home_letter"`, …).
///
/// Returns `{"format", "mime", "width", "height", "animated", "usable_for_card",
/// "placeholder", "suspicious", "reasons", "quality": {"verdict", "effective_ppi",
/// "target_ppi", "warn_ppi", "min_ppi", "placed_mm", "box_mm"}}`.
///
/// # Errors
/// `not_an_image` when the bytes are not a recognised image; `unknown_format`,
/// `unknown_profile`.
pub fn cover_probe(bytes: &[u8], format: &str, profile: &str) -> Result<String, ApiError> {
    let card_format = format_by_name(format)?;
    let profile = profile_by_name(profile)?;
    let details = ebc_covers::sniff::probe_details(bytes)
        .ok_or_else(|| ApiError::new("not_an_image", "not a JPEG, PNG, GIF, WebP or AVIF image"))?;
    let info = details.oriented();
    let assessment = ebc_covers::placeholder::assess(&info, bytes);
    let target = safe_rect(card_format.front(), &profile);
    let thresholds = Thresholds {
        target_ppi: profile.target_ppi,
        warn_ppi: profile.warn_ppi(),
        min_ppi: profile.min_ppi,
    };
    let quality = ebc_covers::print_quality_with(&info, target.size(), &thresholds);
    let mut reasons: Vec<String> =
        assessment.reasons.iter().map(|reason| format!("{reason:?}")).collect();
    reasons.extend(assessment.suspicions.iter().map(|suspicion| format!("{suspicion:?}")));
    to_json(&ProbeDto {
        format: info.format.extension(),
        mime: info.format.mime(),
        width: info.width,
        height: info.height,
        animated: details.animated,
        usable_for_card: matches!(
            info.format,
            ebc_covers::Format::Jpeg | ebc_covers::Format::Png | ebc_covers::Format::WebP
        ),
        placeholder: assessment.is_placeholder(),
        suspicious: assessment.is_suspicious(),
        reasons,
        quality: QualityDto {
            verdict: verdict_name(quality.verdict),
            effective_ppi: round2(quality.effective_ppi),
            target_ppi: thresholds.target_ppi,
            warn_ppi: thresholds.warn_ppi,
            min_ppi: thresholds.min_ppi,
            placed_mm: (round2(quality.placed_mm.w), round2(quality.placed_mm.h)),
            box_mm: (round2(target.w), round2(target.h)),
        },
    })
}

// ---- Product pages ---------------------------------------------------------------------

#[derive(Serialize)]
struct PageDto {
    images: Vec<FoundDto>,
    isbns: Vec<String>,
    title: Option<String>,
    tdm_reservation: Option<bool>,
    noai: bool,
    truncated: bool,
}

#[derive(Serialize)]
struct FoundDto {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    via: &'static str,
    confidence: f32,
    isbns: Vec<String>,
    /// Why the URL must not be fetched, if it must not.
    refusal: Option<String>,
}

fn via_name(via: Via) -> &'static str {
    match via {
        Via::JsonLd => "json_ld",
        Via::OpenGraph => "open_graph",
        Via::TwitterCard => "twitter_card",
        Via::LinkImageSrc => "link_image_src",
        Via::Microdata => "microdata",
    }
}

/// Reads a product page the user pasted the address of: the image URLs it declares
/// (JSON-LD, Open Graph, …), the ISBNs it mentions, its title and its text-and-data-mining
/// signals. The HTML is attacker-controlled: nothing in the result is markup.
///
/// Returns `{"images": [{"url", "width", "height", "via", "confidence", "isbns",
/// "refusal"}], "isbns", "title", "tdm_reservation", "noai", "truncated"}`. The images
/// are for locating and previewing only.
///
/// # Errors
/// None in practice (see [`ApiError`]): unreadable HTML gives an empty result.
pub fn scrape_extract(html: &str, base_url: &str) -> Result<String, ApiError> {
    let page = ebc_covers::scrape::extract(html, base_url);
    to_json(&PageDto {
        images: page
            .images
            .iter()
            .map(|found| FoundDto {
                url: found.url.clone(),
                width: found.width,
                height: found.height,
                via: via_name(found.via),
                confidence: found.confidence,
                isbns: found.isbns.iter().map(|isbn| isbn.to_plain()).collect(),
                refusal: found.refusal.map(|refusal| refusal.to_string()),
            })
            .collect(),
        isbns: page.isbns.iter().map(|isbn| isbn.to_plain()).collect(),
        title: page.title.clone(),
        tdm_reservation: page.tdm_reservation,
        noai: page.noai,
        truncated: page.truncated,
    })
}

// ---- Decoding and colour analysis ------------------------------------------------------

/// A decoded image, scaled down for analysis.
pub(crate) struct Analysed {
    /// The pixels, at most the requested side.
    pub(crate) pixmap: Pixmap,
    /// Width of the file's image as displayed.
    pub(crate) width: u32,
    /// Height of the file's image as displayed.
    pub(crate) height: u32,
}

impl Analysed {
    /// Original pixels per analysed pixel.
    pub(crate) fn scale(&self) -> f64 {
        f64::from(self.width) / f64::from(self.pixmap.width().max(1))
    }
}

fn decode_error(message: impl Into<String>) -> ApiError {
    ApiError::new("cover_decode", message)
}

/// Decodes a JPEG, PNG or WebP file under the composer's limits and scales it so that
/// its longer side is at most `max_side` pixels (transparency composited on white).
pub(crate) fn decode_scaled(bytes: &[u8], max_side: u32) -> Result<Analysed, ApiError> {
    if bytes.len() > ebc_card::MAX_COVER_BYTES {
        return Err(decode_error("the file is larger than the composer accepts"));
    }
    let format = image::guess_format(bytes).map_err(|e| decode_error(e.to_string()))?;
    if !matches!(format, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
        return Err(decode_error("only JPEG, PNG and WebP images can be read"));
    }
    let check = |width: u32, height: u32| {
        let pixels = u64::from(width) * u64::from(height);
        if width == 0
            || height == 0
            || width > ebc_card::MAX_COVER_SIDE_PX
            || height > ebc_card::MAX_COVER_SIDE_PX
            || pixels > ebc_card::MAX_COVER_PIXELS
        {
            Err(ApiError::new(
                "cover_too_large",
                format!("{width} x {height} px is more than the composer accepts"),
            ))
        } else {
            Ok(())
        }
    };
    // The size the header announces is checked before a decoder allocates anything.
    if let Some(info) = ebc_covers::probe(bytes) {
        check(info.width, info.height)?;
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(ebc_card::MAX_COVER_SIDE_PX);
    limits.max_image_height = Some(ebc_card::MAX_COVER_SIDE_PX);
    limits.max_alloc = Some(ebc_card::MAX_COVER_PIXELS * 8);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| decode_error(e.to_string()))?;
    let (width, height) = decoder.dimensions();
    check(width, height)?;
    let orientation =
        decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image =
        DynamicImage::from_decoder(decoder).map_err(|e| decode_error(e.to_string()))?;
    image.apply_orientation(orientation);
    let (width, height) = (image.width(), image.height());
    check(width, height)?;

    let max_side = max_side.max(1);
    let small = if width.max(height) > max_side {
        image.thumbnail(max_side, max_side)
    } else {
        image
    };
    let mut rgba = small.into_rgba8();
    for px in rgba.pixels_mut() {
        let alpha = u32::from(px[3]);
        for channel in 0..3 {
            let value = u32::from(px[channel]);
            px[channel] = ((value * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        }
        px[3] = 255;
    }
    let (w, h) = (rgba.width(), rgba.height());
    let pixmap = Pixmap::from_rgba8(w, h, rgba.into_raw())
        .ok_or_else(|| decode_error("the decoder returned an empty image"))?;
    Ok(Analysed { pixmap, width, height })
}

#[derive(Serialize)]
struct SwatchDto {
    color: String,
    share: f32,
}

#[derive(Serialize)]
struct EdgeDto {
    /// Dominant colour of the strip along the edge.
    color: String,
    /// 1 for a single colour, 0 for a busy edge.
    uniformity: f32,
    uniform: bool,
}

#[derive(Serialize)]
struct EdgesDto {
    left: EdgeDto,
    right: EdgeDto,
    top: EdgeDto,
    bottom: EdgeDto,
}

#[derive(Serialize)]
struct SpineSuggestion {
    /// A flat colour computed from the cover: its binding edge when that is plain, its
    /// frame, or its dominant colour. No pixel of the cover is reproduced.
    background: String,
    /// Black or white, whichever reads better on it.
    text: String,
    contrast: f64,
    /// `"binding_edge"`, `"border"` or `"dominant"`.
    from: &'static str,
}

#[derive(Serialize)]
struct RegionDto {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    score: f32,
}

#[derive(Serialize)]
struct AnalysisDto {
    width: u32,
    height: u32,
    palette: Vec<SwatchDto>,
    edges: EdgesDto,
    uniform_border: Option<String>,
    spine: SpineSuggestion,
    /// Where text was found, in pixels of the original image; empty when font
    /// identification is not compiled in.
    text_regions: Vec<RegionDto>,
    text_regions_available: bool,
}

/// Thickness of the edge strips, as a fraction of the image.
const EDGE_STRIP: f32 = 0.04;
/// Longest side of the image the colours are computed on.
const ANALYSIS_SIDE_PX: u32 = 512;

fn edge_dto(image: &Pixmap, edge: Edge) -> (EdgeDto, Rgb8) {
    let color =
        edge_palette(image, edge, EDGE_STRIP, 1).first().map_or(Rgb8::WHITE, |s| s.color);
    let uniformity = edge_uniformity(image, edge, EDGE_STRIP);
    let dto = EdgeDto {
        color: color.to_hex(),
        uniformity,
        uniform: uniformity >= ebc_color::UNIFORM_EDGE_THRESHOLD,
    };
    (dto, color)
}

/// Analyses an image **the user supplied**: its palette, how plain its four edges are,
/// a flat spine colour computed from it with a readable text colour, and where its text
/// is (for font identification).
///
/// Returns `{"width", "height", "palette": [{"color", "share"}], "edges": {"left":
/// {"color", "uniformity", "uniform"}, …}, "uniform_border", "spine": {"background",
/// "text", "contrast", "from"}, "text_regions": [{"x", "y", "w", "h", "score"}],
/// "text_regions_available"}`. Colours are `#rrggbb`.
///
/// # Errors
/// `cover_decode` when the bytes are not a readable JPEG, PNG or WebP image;
/// `cover_too_large`.
pub fn analyze_cover(bytes: &[u8]) -> Result<String, ApiError> {
    let analysed = decode_scaled(bytes, ANALYSIS_SIDE_PX)?;
    let image = &analysed.pixmap;
    let swatches = palette(image, 6);
    let (left, left_color) = edge_dto(image, Edge::Left);
    let (right, _) = edge_dto(image, Edge::Right);
    let (top, _) = edge_dto(image, Edge::Top);
    let (bottom, _) = edge_dto(image, Edge::Bottom);
    let border = ebc_color::has_uniform_border(image);
    let (background, from) = if left.uniform {
        (left_color, "binding_edge")
    } else if let Some(color) = border {
        (color, "border")
    } else {
        (swatches.first().map_or(Rgb8::WHITE, |s| s.color), "dominant")
    };
    let text = ebc_color::black_or_white(background);
    let scale = analysed.scale();
    let text_regions = super::fontid::text_regions(image)
        .into_iter()
        .map(|(x, y, w, h, score)| {
            let up = |v: u32| (f64::from(v) * scale).round() as u32;
            RegionDto { x: up(x), y: up(y), w: up(w), h: up(h), score }
        })
        .collect();
    to_json(&AnalysisDto {
        width: analysed.width,
        height: analysed.height,
        palette: swatches
            .iter()
            .map(|s| SwatchDto { color: s.color.to_hex(), share: s.share })
            .collect(),
        edges: EdgesDto { left, right, top, bottom },
        uniform_border: border.map(Rgb8::to_hex),
        spine: SpineSuggestion {
            background: background.to_hex(),
            text: text.to_hex(),
            contrast: round2(contrast_ratio(text, background)),
            from,
        },
        text_regions,
        text_regions_available: super::fontid::AVAILABLE,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn value(json: &str) -> Value {
        serde_json::from_str(json).expect("the API returns JSON")
    }

    /// A PNG of `w` x `h` pixels painted by `paint`.
    pub(crate) fn png(w: u32, h: u32, paint: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
        let image = image::RgbImage::from_fn(w, h, |x, y| image::Rgb(paint(x, y)));
        let mut out = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .expect("encoding a PNG in memory");
        out
    }

    /// A cover-like image: a navy frame around a noisy cream field.
    pub(crate) fn framed_cover(w: u32, h: u32) -> Vec<u8> {
        png(w, h, |x, y| {
            let frame = x < w / 10 || y < h / 10 || x >= w - w / 10 || y >= h - h / 10;
            if frame {
                [20, 30, 90]
            } else {
                let n = ((x * 31 + y * 17) % 23) as u8;
                [230 - n, 220 - n, 190 + n]
            }
        })
    }

    #[test]
    fn candidates_are_for_looking_only() {
        let book = json!({"title": "L'Étranger", "isbn13": "9782070360024"});
        let v = value(&cover_candidates(&book.to_string(), "").expect("candidates"));
        let list = v.as_array().expect("array");
        assert!(!list.is_empty());
        assert_eq!(list[0]["source"], "openlibrary");
        for candidate in list {
            assert_eq!(candidate["rights_basis"], "none");
            assert_eq!(candidate["printable"], false);
            assert_ne!(candidate["role"], "download");
            assert_ne!(candidate["risk"], "high");
            assert!(candidate["url"].as_str().expect("url").starts_with("https://"));
        }
        assert!(list.iter().any(|c| c["role"] == "preview"), "something to show inline");
    }

    #[test]
    fn the_page_cannot_raise_the_risk_or_claim_operator_rights() {
        let book = json!({"title": "L'Étranger", "isbn13": "9782070360024"});
        let policy = json!({
            "max_risk": "high",
            "operator_rights_basis": "print_licence",
            "mode": "operator",
            "allowed_sources": ["openlibrary", "amazon", "bnf"]
        });
        let v = value(&cover_candidates(&book.to_string(), &policy.to_string()).expect("ok"));
        for candidate in v.as_array().expect("array") {
            assert_ne!(candidate["role"], "download");
            assert_ne!(candidate["source"], "amazon");
            assert_eq!(candidate["rights_basis"], "none");
        }
        // A book without an ISBN, hostile JSON: no panic.
        assert!(cover_candidates(r#"{"title": ""}"#, "").is_ok());
        assert_eq!(cover_candidates("[", "").expect_err("bad").code, "invalid_json");
        assert_eq!(cover_candidates("{}", "").expect_err("no title").code, "invalid_json");
    }

    #[test]
    fn probe_rates_an_image_for_the_front_panel() {
        // 300 ppi on the 100 x 168 mm safe area of a poche front is 1181 x 1984 px.
        let good = png(1200, 2000, |_, _| [200, 10, 10]);
        let v = value(&cover_probe(&good, "poche", "upload_eu").expect("probed"));
        assert_eq!((v["format"].as_str(), v["mime"].as_str()), (Some("png"), Some("image/png")));
        assert_eq!((v["width"].as_u64(), v["height"].as_u64()), (Some(1200), Some(2000)));
        assert_eq!(v["usable_for_card"], true);
        assert_eq!(v["quality"]["verdict"], "excellent");
        assert_eq!(v["quality"]["box_mm"], json!([100.0, 168.0]));

        let small = png(300, 500, |x, y| [(x % 256) as u8, (y % 256) as u8, 90]);
        let v = value(&cover_probe(&small, "poche", "upload_eu").expect("probed"));
        assert_eq!(v["quality"]["verdict"], "insufficient");
        assert!(v["quality"]["effective_ppi"].as_f64().expect("ppi") < 100.0);
        // The home profile asks for less.
        let home = value(&cover_probe(&small, "poche", "home_a4").expect("probed"));
        assert!(home["quality"]["min_ppi"].as_f64() < v["quality"]["min_ppi"].as_f64());
    }

    #[test]
    fn probe_flags_placeholders_and_refuses_non_images() {
        let pixel = png(1, 1, |_, _| [255, 255, 255]);
        let v = value(&cover_probe(&pixel, "poche", "upload_eu").expect("probed"));
        assert_eq!(v["placeholder"], true);
        assert!(!v["reasons"].as_array().expect("reasons").is_empty());

        for junk in [&b""[..], b"GIF", b"<svg xmlns='http://www.w3.org/2000/svg'/>", &[0xff; 64]] {
            assert_eq!(cover_probe(junk, "poche", "upload_eu").expect_err("junk").code, "not_an_image");
        }
        let ok = png(10, 10, |_, _| [0, 0, 0]);
        assert_eq!(cover_probe(&ok, "folio", "upload_eu").expect_err("format").code, "unknown_format");
        assert_eq!(cover_probe(&ok, "poche", "press").expect_err("profile").code, "unknown_profile");
        assert_eq!(
            cover_probe(&ok, "poche", &"home_".repeat(100)).expect_err("profile").code,
            "unknown_profile"
        );
        // A truncated PNG still has a readable header; nothing panics.
        assert!(cover_probe(&good_prefix(), "poche", "upload_eu").is_ok());
    }

    fn good_prefix() -> Vec<u8> {
        let mut bytes = png(400, 600, |x, _| [(x % 200) as u8, 0, 0]);
        bytes.truncate(64);
        bytes
    }

    #[test]
    fn scrape_returns_data_not_markup() {
        let html = r#"<html><head><title>Le Horla &lt;b&gt;</title>
            <meta property="og:image" content="/img/horla.jpg">
            <meta property="og:image:width" content="800">
            <script type="application/ld+json">{"@type":"Book","isbn":"9782070360024",
              "image":"https://cdn.example.org/horla-large.jpg"}</script>
            </head><body><img src=x onerror=alert(1)></body></html>"#;
        let v = value(&scrape_extract(html, "https://editeur.example.org/livre/horla").expect("page"));
        let images = v["images"].as_array().expect("images");
        assert!(images.iter().any(|i| i["url"] == "https://editeur.example.org/img/horla.jpg"));
        assert!(images.iter().any(|i| i["via"] == "json_ld"));
        assert_eq!(v["isbns"][0], "9782070360024");
        // Hostile input: empty, binary, enormous, a denied host.
        assert!(scrape_extract("", "").is_ok());
        assert!(scrape_extract("\0\u{feff}<<<>>>", "not a url").is_ok());
        assert!(scrape_extract(&"<meta property=og:image content=x>".repeat(50_000), "https://a.example").is_ok());
        let denied = value(
            &scrape_extract(
                r#"<meta property="og:image" content="https://m.media-amazon.com/images/I/1.jpg">"#,
                "https://editeur.example.org/",
            )
            .expect("page"),
        );
        for image in denied["images"].as_array().expect("images") {
            assert!(image["refusal"].is_string(), "a denied host is refused: {image}");
        }
    }

    #[test]
    fn analysis_suggests_a_flat_spine_colour_from_the_frame() {
        let v = value(&analyze_cover(&framed_cover(300, 480)).expect("analysed"));
        assert_eq!((v["width"].as_u64(), v["height"].as_u64()), (Some(300), Some(480)));
        assert_eq!(v["edges"]["left"]["uniform"], true);
        assert_eq!(v["edges"]["left"]["color"], "#141e5a");
        assert_eq!(v["spine"]["background"], "#141e5a");
        assert_eq!(v["spine"]["from"], "binding_edge");
        assert_eq!(v["spine"]["text"], "#ffffff");
        assert!(v["spine"]["contrast"].as_f64().expect("ratio") >= 4.5);
        let shares: f64 =
            v["palette"].as_array().expect("palette").iter().filter_map(|s| s["share"].as_f64()).sum();
        assert!((shares - 1.0).abs() < 1e-3);
        assert_eq!(v["text_regions_available"], crate::api::fontid::AVAILABLE);
    }

    #[test]
    fn analysis_is_deterministic_and_scales_large_images_down() {
        let big = png(1800, 2700, |x, y| [(x / 8) as u8, (y / 11) as u8, 128]);
        let a = analyze_cover(&big).expect("analysed");
        assert_eq!(a, analyze_cover(&big).expect("analysed again"));
        let v = value(&a);
        assert_eq!((v["width"].as_u64(), v["height"].as_u64()), (Some(1800), Some(2700)));
        assert_eq!(v["edges"]["left"]["uniform"], false);
    }

    #[test]
    fn analysis_refuses_what_is_not_a_readable_image() {
        for junk in [&b""[..], b"not an image", &[0u8; 4096]] {
            assert_eq!(analyze_cover(junk).expect_err("junk").code, "cover_decode");
        }
        let mut truncated = framed_cover(200, 300);
        truncated.truncate(truncated.len() / 2);
        assert_eq!(analyze_cover(&truncated).expect_err("truncated").code, "cover_decode");
        // A PNG header announcing 60000 x 60000 pixels is refused before decoding.
        let mut bomb = png(2, 2, |_, _| [0, 0, 0]);
        bomb[16..20].copy_from_slice(&60_000u32.to_be_bytes());
        bomb[20..24].copy_from_slice(&60_000u32.to_be_bytes());
        let code = analyze_cover(&bomb).expect_err("bomb").code;
        assert!(code == "cover_too_large" || code == "cover_decode", "{code}");
        // GIF is recognised by the probe but not composable.
        let gif = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00;";
        assert_eq!(analyze_cover(gif).expect_err("gif").code, "cover_decode");
    }

    #[test]
    fn transparency_is_composited_on_white() {
        let image = image::RgbaImage::from_pixel(20, 30, image::Rgba([255, 0, 0, 0]));
        let mut bytes = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).expect("encode");
        let analysed = decode_scaled(&bytes, 64).expect("decoded");
        assert_eq!(analysed.pixmap.get(3, 3), Some([255, 255, 255, 255]));
        assert_eq!((analysed.width, analysed.height), (20, 30));
        assert!((analysed.scale() - 1.0).abs() < 1e-9);
    }
}
