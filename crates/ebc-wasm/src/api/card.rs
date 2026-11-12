//! The card: its request as JSON, the PDF, the SVG preview — and the rights gate.
//!
//! # The rights gate, as the page meets it
//!
//! * No cover, or a cover with rights basis `none`: the **typographic card** (title,
//!   author and publisher as text). This is the default.
//! * A cover the user picked from their own device, with a declaration: one of
//!   [`RIGHTS_CHOICES`]. A page can only declare `rights_holder_declared`,
//!   `public_domain` or `user_private_copy` — `print_licence` and `promo_all_media`
//!   travel with a reseller's feed, not with a file picked in a browser
//!   (`ebc_covers::AssetOrigin::LocalFile`).
//! * A professional print profile accepts `rights_holder_declared` and `public_domain`
//!   only; a private copy is for the user's own printer (`rights_route_refused`).
//! * Spine styles that reproduce artwork need `adaptation_authorised` **and** a basis
//!   that allows adaptation (`adaptation_not_authorised`).
//!
//! The conditions that go with a basis (the per-country public-domain test, one jacket
//! per gift code, an identified declarant for a professional order) are **not** checked
//! here: the composer only sees the basis. They are the deployment's to enforce.

use std::cell::RefCell;
use std::sync::OnceLock;

use ebc_card::rights::{allows_adaptation, route_of};
use ebc_card::{
    CardError, CardRequest, CardWarning, CoverInput, FontSet, OutputProfile, PrintRoute,
    RightsBasis, SpineChoice, bundled_fonts,
};
use ebc_color::Rgb8;
use ebc_covers::{AssetOrigin, CoversError, PrintableCover};
use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::{BookMeta, GiftCode, Locale};
use ebc_scene::{PdfStandard, Pixmap};
use serde::{Deserialize, Serialize};

use super::cover::{decode_scaled, format_by_name, profile_by_name};
use super::{ApiError, from_json, to_json};

// ---- The request -----------------------------------------------------------------------

/// The cover's rights, as declared in the page.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct CoverDto {
    /// The rights basis: `"none"` (default), `"rights_holder_declared"`,
    /// `"public_domain"` or `"user_private_copy"`.
    rights: RightsBasis,
    /// Which sentence of [`RIGHTS_CHOICES`] the user ticked (recorded, not interpreted).
    declaration: Option<String>,
    /// The cover may lose up to the bleed at its edges (full-bleed placement).
    trim_authorised: bool,
    /// The artwork may be adapted (spine styles that sample the cover).
    adaptation_authorised: bool,
    /// The credit line printed for the cover.
    credit: Option<String>,
}

/// One card, as the page describes it. Unknown fields are refused, so that a page and a
/// wasm package that disagree fail loudly instead of printing something else.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CardRequestDto {
    book: BookMeta,
    /// The gift code, in any spelling [`GiftCode::parse`] accepts.
    code: String,
    /// Base of the claim URL, e.g. `"https://ebk.example"`.
    base_url: String,
    /// The full claim URL when the server returned one (default: `base_url/k/code`).
    #[serde(default)]
    claim_url: Option<String>,
    /// The address printed in clear (default: the host of `base_url`).
    #[serde(default)]
    display_url: Option<String>,
    #[serde(default)]
    cover: CoverDto,
    /// `"typeset"` (default), `"cover_colour"` or `"edge_continuation"`.
    #[serde(default)]
    spine: Option<String>,
    /// Flat ground colour of the outside, `#rrggbb`.
    #[serde(default)]
    ground: Option<String>,
    /// One of the ids of [`TITLE_FONTS`].
    #[serde(default)]
    title_font: Option<String>,
    #[serde(default)]
    giver: Option<String>,
    #[serde(default)]
    recipient: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    store_name: Option<String>,
    #[serde(default)]
    back_text: Option<String>,
    #[serde(default)]
    legal_lines: Vec<String>,
    #[serde(default)]
    help_url: Option<String>,
    #[serde(default)]
    expires_unix: Option<i64>,
    /// `"fr"` (default), `"en"`, `"de"`, `"es"`, `"it"`, `"nl"`, `"pt"`.
    #[serde(default)]
    locale: Option<String>,
    /// `"poche"` (default), `"a6"`, `"digest"`, `"a5"`.
    #[serde(default)]
    format: Option<String>,
    /// `"home_a4"` (default), `"home_letter"`, `"home_a3"`, `"upload_eu"`,
    /// `"professional_eu"`.
    #[serde(default)]
    profile: Option<String>,
    /// Creation time for the PDF, seconds since the Unix epoch (`Date.now() / 1000`).
    /// Required by the professional (PDF/X) profiles: nothing here reads a clock.
    #[serde(default)]
    created_unix: Option<i64>,
    /// Add the proofing overlay (bleed, trim, creases, safe areas).
    #[serde(default)]
    guides: bool,
}

/// The profile a request gets when it names none: printing at home on A4.
pub const DEFAULT_PROFILE: &str = "home_a4";
/// The format a request gets when it names none.
pub const DEFAULT_FORMAT: &str = "poche";

/// Longest text accepted in any free-text field, in bytes. The composer clamps what does
/// not fit; the cap only bounds the work.
pub const MAX_TEXT_BYTES: usize = 4000;

fn text(field: &'static str, value: Option<String>) -> Result<Option<String>, ApiError> {
    match value {
        Some(value) if value.len() > MAX_TEXT_BYTES => Err(ApiError::new(
            "text_too_long",
            format!("{field}: {} bytes, more than the {MAX_TEXT_BYTES} accepted", value.len()),
        )),
        Some(value) if value.trim().is_empty() => Ok(None),
        other => Ok(other),
    }
}

/// The spine styles a page can ask for: name, whether it needs a printable cover and
/// whether it adapts artwork (needs authorisation). Styles that need an image of the
/// real spine are not offered by the page.
pub const SPINE_STYLES: [(&str, bool, bool); 3] =
    [("typeset", false, false), ("cover_colour", true, false), ("edge_continuation", true, true)];

fn spine_choice(name: Option<&str>) -> Result<SpineChoice, ApiError> {
    match name.unwrap_or("typeset") {
        "typeset" => Ok(SpineChoice::Typeset),
        "cover_colour" => Ok(SpineChoice::CoverColour),
        "edge_continuation" => Ok(SpineChoice::EdgeContinuation),
        _ => Err(ApiError::new(
            "unknown_spine",
            "expected typeset, cover_colour or edge_continuation",
        )),
    }
}

/// What the user can declare about a cover picked from their own device: the id of the
/// sentence shown, the rights basis it stands for, and whether that basis opens the
/// professional print route.
///
/// Three sentences stand for `rights_holder_declared` — the research's standalone
/// declaration "I am the author or publisher, or hold a licence"
/// (`docs/research/07-legal-covers-and-reseller-rights.md`, item 20), in plainer words.
pub const RIGHTS_CHOICES: [(&str, RightsBasis); 5] = [
    ("own_work", RightsBasis::RightsHolderDeclared),
    ("print_licence", RightsBasis::RightsHolderDeclared),
    ("publisher_authorised", RightsBasis::RightsHolderDeclared),
    ("public_domain", RightsBasis::PublicDomain),
    ("private_copy", RightsBasis::UserPrivateCopy),
];

/// The title faces a page can choose from — all bundled, all under the SIL Open Font
/// License: id and the file in `assets/fonts` it stands for.
pub const TITLE_FONTS: [(&str, &str); 4] = [
    ("spectral_semibold", "Spectral-SemiBold.ttf"),
    ("bebas_neue", "BebasNeue-Regular.ttf"),
    ("atkinson_bold", "AtkinsonHyperlegible-Bold.ttf"),
    ("courier_prime_bold", "CourierPrime-Bold.ttf"),
];

fn base_fonts() -> Result<&'static FontSet, ApiError> {
    static FONTS: OnceLock<Option<FontSet>> = OnceLock::new();
    FONTS
        .get_or_init(|| bundled_fonts().ok())
        .as_ref()
        .ok_or_else(|| ApiError::new("internal", "the bundled fonts cannot be read"))
}

/// The font set with the chosen face for titles (cover, spine, headline). The default
/// title face becomes the fallback of the chosen one.
fn fonts_for(title_font: Option<&str>) -> Result<FontSet, ApiError> {
    let base = base_fonts()?;
    let mut fonts = base.clone();
    match title_font.unwrap_or("spectral_semibold") {
        "spectral_semibold" => {}
        "bebas_neue" => {
            if let Some(alt) = base.title_alt.clone() {
                fonts.title = alt;
                fonts.title_alt = Some(base.title.clone());
            }
        }
        "atkinson_bold" => {
            fonts.title = base.body_bold.clone();
            fonts.title_alt = Some(base.title.clone());
        }
        "courier_prime_bold" => {
            fonts.title = base.code.clone();
            fonts.title_alt = Some(base.title.clone());
        }
        _ => {
            return Err(ApiError::new(
                "unknown_title_font",
                format!(
                    "expected one of {}",
                    TITLE_FONTS.iter().map(|(id, _)| *id).collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }
    Ok(fonts)
}

/// A cover as handed to the composer.
enum Cover {
    /// The file's bytes: what the PDF is made from.
    Encoded(Vec<u8>),
    /// A scaled-down copy: what previews are made from.
    Pixels(Pixmap),
}

/// A request ready for the composer.
struct Prepared {
    request: CardRequest,
    fonts: FontSet,
}

fn prepare(
    request_json: &str,
    cover: Option<Cover>,
    icc: Option<&[u8]>,
) -> Result<Prepared, ApiError> {
    let dto: CardRequestDto = from_json("request", request_json)?;
    let code = GiftCode::parse(&dto.code).map_err(|e| ApiError::new("invalid_code", e.to_string()))?;
    let locale = match dto.locale.as_deref() {
        None => Locale::default(),
        Some(tag) if tag.len() <= 35 => Locale::from_tag(tag)
            .ok_or_else(|| ApiError::new("unknown_locale", "expected fr, en, de, es, it, nl or pt"))?,
        Some(_) => return Err(ApiError::new("unknown_locale", "not a language tag")),
    };
    let format: CardFormat = format_by_name(dto.format.as_deref().unwrap_or(DEFAULT_FORMAT))?;
    let profile: PrintProfile = profile_by_name(dto.profile.as_deref().unwrap_or(DEFAULT_PROFILE))?;
    let route = route_of(&profile);
    let rights = dto.cover.rights;
    if let Some(declaration) = &dto.cover.declaration {
        let known = RIGHTS_CHOICES.iter().find(|(id, _)| id == declaration);
        match known {
            Some((_, basis)) if *basis == rights => {}
            _ => {
                return Err(ApiError::new(
                    "unknown_declaration",
                    "the declaration does not go with the rights basis",
                ));
            }
        }
    }
    if dto.base_url.len() > 2048 || dto.claim_url.as_ref().is_some_and(|u| u.len() > 2048) {
        return Err(ApiError::new("invalid_claim_url", "the URL is too long"));
    }

    let mut book = dto.book;
    // Publishers' descriptions are never printed, and nothing else reads them.
    book.description = None;
    let mut request = CardRequest::new(book, code, dto.base_url.trim());
    if let Some(url) = text("claim_url", dto.claim_url)? {
        request.claim_url = url.trim().to_owned();
    }
    if let Some(display) = text("display_url", dto.display_url)? {
        request.display_url = display.trim().to_owned();
    }

    request.cover = match cover {
        // A cover without a declared basis never reaches the composer's decoder.
        Some(_) if rights == RightsBasis::None => {
            CoverInput::Pixels { pixmap: Pixmap::filled(1, 1, [255; 4]), rights }
        }
        Some(cover) => {
            // A page can only vouch for a file its user picked.
            if !AssetOrigin::LocalFile.compatible_bases().contains(&rights) {
                return Err(ApiError::new(
                    "rights_basis_not_available",
                    format!("a file picked in a browser cannot carry rights basis `{rights}`"),
                ));
            }
            if !rights.allows(route) {
                return Err(route_refused(rights, route));
            }
            match cover {
                Cover::Encoded(bytes) => {
                    let printable = PrintableCover::new(bytes, rights, AssetOrigin::LocalFile)
                        .map_err(|e| covers_error(&e))?;
                    CoverInput::Encoded { bytes: printable.into_bytes(), rights }
                }
                Cover::Pixels(pixmap) => CoverInput::Pixels { pixmap, rights },
            }
        }
        None => CoverInput::None,
    };
    request.trim_authorised = dto.cover.trim_authorised;
    request.adaptation_authorised = dto.cover.adaptation_authorised;
    request.cover_credit = text("cover.credit", dto.cover.credit)?;
    request.spine = spine_choice(dto.spine.as_deref())?;
    if request.spine.requires_authorisation()
        && !(request.adaptation_authorised && allows_adaptation(rights))
    {
        return Err(ApiError::new(
            "adaptation_not_authorised",
            format!(
                "the spine style {} adapts artwork: it needs the authorisation to adapt and a \
                 rights basis that allows it (got `{rights}`)",
                request.spine.name()
            ),
        ));
    }
    request.ground = match dto.ground.as_deref().map(str::trim).filter(|g| !g.is_empty()) {
        Some(hex) if hex.len() <= 16 => Some(
            Rgb8::from_hex(hex)
                .ok_or_else(|| ApiError::new("invalid_colour", "expected a colour as #rrggbb"))?,
        ),
        Some(_) => return Err(ApiError::new("invalid_colour", "expected a colour as #rrggbb")),
        None => None,
    };
    request.giver = text("giver", dto.giver)?;
    request.recipient = text("recipient", dto.recipient)?;
    request.message = text("message", dto.message)?;
    request.store_name = text("store_name", dto.store_name)?;
    request.back_text = text("back_text", dto.back_text)?;
    request.help_url = text("help_url", dto.help_url)?;
    if dto.legal_lines.len() > 8 {
        return Err(ApiError::new("text_too_long", "legal_lines: at most eight lines"));
    }
    for line in dto.legal_lines {
        if let Some(line) = text("legal_lines", Some(line))? {
            request.legal_lines.push(line);
        }
    }
    request.expires_unix = dto.expires_unix;
    request.locale = locale;
    request.format = format;
    request.profile = profile;
    request.created_unix = dto.created_unix;
    request.guides = dto.guides;
    if let Some(icc) = icc.filter(|icc| !icc.is_empty()) {
        if request.profile.standard != PdfStandard::Plain {
            request.output_profile = Some(
                OutputProfile::new(icc.to_vec(), "FOGRA39", "ISO Coated v2 300% (basICColor)")
                    .map_err(|e| card_error(&e))?,
            );
        }
    }
    Ok(Prepared { request, fonts: fonts_for(dto.title_font.as_deref())? })
}

// ---- Errors and warnings ---------------------------------------------------------------

fn route_name(route: PrintRoute) -> &'static str {
    match route {
        PrintRoute::HomePrint => "home_print",
        PrintRoute::ProfessionalOrder => "professional_order",
    }
}

fn route_refused(basis: RightsBasis, route: PrintRoute) -> ApiError {
    ApiError::new(
        "rights_route_refused",
        format!("a cover with rights basis `{basis}` cannot go into {route}"),
    )
}

fn covers_error(error: &CoversError) -> ApiError {
    let code = match error {
        CoversError::NoRightsBasis => "no_rights_basis",
        CoversError::OriginMismatch { .. } => "rights_basis_not_available",
        CoversError::NotAnImage => "not_an_image",
        CoversError::Placeholder => "cover_placeholder",
        CoversError::RouteNotAllowed { .. } => "rights_route_refused",
        _ => "cover_refused",
    };
    ApiError::new(code, error.to_string())
}

fn card_error(error: &CardError) -> ApiError {
    let code = match error {
        CardError::EmptyTitle => "empty_title",
        CardError::InvalidClaimUrl(_) => "invalid_claim_url",
        CardError::Layout(_) => "layout",
        CardError::Spine(_) => "spine",
        CardError::Qr(_) => "qr_error",
        CardError::Pdf(_) => "pdf",
        CardError::Font(_) | CardError::FontNotUsable { .. } => "font",
        CardError::CoverDecode(_) => "cover_decode",
        CardError::CoverTooLarge { .. } => "cover_too_large",
        CardError::RightsRouteRefused { .. } => "rights_route_refused",
        CardError::AdaptationNotAuthorised { .. } => "adaptation_not_authorised",
        CardError::CoverColourSpace { .. } => "cover_colour_space",
        CardError::MissingCreationDate => "missing_creation_date",
        CardError::NoOutputProfile => "no_output_profile",
        CardError::InvalidOutputProfile(_) => "invalid_output_profile",
        CardError::FormatTooSmall { .. } => "format_too_small",
        _ => "card_error",
    };
    ApiError::new(code, error.to_string())
}

/// A warning, for the page: a stable code, the English text, and the values to put in
/// a translated sentence.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WarningDto {
    /// Stable identifier in `snake_case`.
    pub code: &'static str,
    /// The warning in English.
    pub message: String,
    /// Values for a translated template: `ppi`, `threshold`, `field`, `characters`, ….
    pub params: Vec<(&'static str, String)>,
}

fn warning_dto(warning: &CardWarning) -> WarningDto {
    let rounded = |v: f64| format!("{v:.0}");
    let (code, params): (&'static str, Vec<(&'static str, String)>) = match warning {
        CardWarning::CoverWithoutRights => ("cover_without_rights", vec![]),
        CardWarning::LowResolution { ppi, threshold_ppi, below_minimum } => (
            if *below_minimum { "resolution_below_minimum" } else { "low_resolution" },
            vec![("ppi", rounded(*ppi)), ("threshold", rounded(*threshold_ppi))],
        ),
        CardWarning::CoverDownsampled { from_ppi, to_ppi } => {
            ("cover_downsampled", vec![("from", rounded(*from_ppi)), ("to", rounded(*to_ppi))])
        }
        CardWarning::HeavyCrop { percent, dimension } => {
            ("heavy_crop", vec![("percent", rounded(*percent)), ("dimension", (*dimension).to_owned())])
        }
        CardWarning::CoverCreditMissing => ("cover_credit_missing", vec![]),
        CardWarning::CoverNotConverted => ("cover_not_converted", vec![]),
        CardWarning::CoverProfileIgnored => ("cover_profile_ignored", vec![]),
        CardWarning::OwnPrinterOnly => ("own_printer_only", vec![]),
        CardWarning::Spine(_) => ("spine", vec![]),
        CardWarning::SpineChoiceIgnored { style } => {
            ("spine_choice_ignored", vec![("style", (*style).to_owned())])
        }
        CardWarning::MissingGlyphs { field, characters } => {
            ("missing_glyphs", vec![("field", field.to_string()), ("characters", characters.clone())])
        }
        CardWarning::TextClamped { field } => ("text_clamped", vec![("field", field.to_string())]),
        CardWarning::StepsReduced { size_pt } => ("steps_reduced", vec![("size", format!("{size_pt:.1}"))]),
        CardWarning::CodeReduced { size_pt } => ("code_reduced", vec![("size", format!("{size_pt:.1}"))]),
        CardWarning::QrModuleBelowMinimum { .. } => ("qr_module_below_minimum", vec![]),
        CardWarning::QrModuleSmall { .. } => ("qr_module_small", vec![]),
        CardWarning::QrVersionHigh { version } => {
            ("qr_version_high", vec![("version", version.to_string())])
        }
        CardWarning::QrPayloadNotUpperCased => ("qr_payload_not_upper_cased", vec![]),
        CardWarning::FallbackOutputProfile => ("fallback_output_profile", vec![]),
        CardWarning::OutputProfileNotPrinterClass { class } => {
            ("output_profile_not_printer_class", vec![("class", class.clone())])
        }
        CardWarning::GuidesIncluded => ("guides_included", vec![]),
        _ => ("other", vec![]),
    };
    WarningDto { code, message: warning.to_string(), params }
}

// ---- Rendering -------------------------------------------------------------------------

/// A print file and what the page should say about it.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedCard {
    /// The PDF.
    pub pdf: Vec<u8>,
    /// The warnings, as a JSON array of `{"code", "message", "params": [[name, value]]}`.
    pub warnings_json: String,
}

/// Composes the card and writes the PDF, entirely in memory.
///
/// `request_json` is the card request (see the crate documentation for its fields);
/// `cover` is the image file the user picked, if any — it is printed only with a rights
/// basis other than `none` in `request.cover.rights`; `icc` is the CMYK output-intent
/// profile for the professional (PDF/X) profiles — FOGRA39 is assumed — without which
/// the bundled CC0 fallback is used and reported (`fallback_output_profile`).
///
/// The same arguments always give the same bytes.
///
/// # Errors
/// `invalid_json`, `invalid_code`, `unknown_locale`, `unknown_format`,
/// `unknown_profile`, `unknown_spine`, `unknown_title_font`, `invalid_colour`,
/// `text_too_long`; the rights gate: `rights_basis_not_available`,
/// `rights_route_refused`, `adaptation_not_authorised`, `unknown_declaration`; the
/// cover: `not_an_image`, `cover_placeholder`, `cover_decode`, `cover_too_large`; the
/// composer: `empty_title`, `invalid_claim_url`, `missing_creation_date`,
/// `invalid_output_profile`, `layout`, `format_too_small`, ….
pub fn render_card(
    request_json: &str,
    cover: Option<Vec<u8>>,
    icc: Option<&[u8]>,
) -> Result<RenderedCard, ApiError> {
    let prepared = prepare(request_json, cover.map(Cover::Encoded), icc)?;
    let (pdf, warnings) =
        ebc_card::render_pdf(&prepared.request, &prepared.fonts).map_err(|e| card_error(&e))?;
    let warnings: Vec<WarningDto> = warnings.iter().map(warning_dto).collect();
    Ok(RenderedCard { pdf, warnings_json: to_json(&warnings)? })
}

/// Longest side of the cover in a preview, in pixels: sharp enough on screen, and small
/// enough for the SVG to be rebuilt at every keystroke.
pub const PREVIEW_COVER_SIDE_PX: u32 = 720;

thread_local! {
    /// The last cover scaled for previews, keyed by a digest of its bytes: a page calls
    /// the preview again and again with the same file.
    static PREVIEW_COVER: RefCell<Option<(u64, usize, Pixmap)>> = const { RefCell::new(None) };
}

/// FNV-1a, for the preview cache only.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn preview_cover(bytes: &[u8]) -> Result<Pixmap, ApiError> {
    let key = (fnv1a(bytes), bytes.len());
    let cached = PREVIEW_COVER.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|(hash, len, _)| (*hash, *len) == key)
            .map(|(_, _, pixmap)| pixmap.clone())
    });
    if let Some(pixmap) = cached {
        return Ok(pixmap);
    }
    let pixmap = decode_scaled(bytes, PREVIEW_COVER_SIDE_PX)?.pixmap;
    PREVIEW_COVER.with(|cache| *cache.borrow_mut() = Some((key.0, key.1, pixmap.clone())));
    Ok(pixmap)
}

/// Warnings that describe the print file, not what a preview shows: the preview is made
/// from a scaled-down cover and has no output intent.
fn print_only(warning: &CardWarning) -> bool {
    matches!(
        warning,
        CardWarning::LowResolution { .. }
            | CardWarning::CoverDownsampled { .. }
            | CardWarning::CoverNotConverted
            | CardWarning::CoverProfileIgnored
            | CardWarning::FallbackOutputProfile
            | CardWarning::OutputProfileNotPrinterClass { .. }
    )
}

#[derive(Serialize)]
struct PreviewDto {
    /// `[outside, inside]` as standalone SVG documents (millimetre units).
    pages: Vec<String>,
    warnings: Vec<WarningDto>,
}

fn compose_preview(
    request_json: &str,
    cover: Option<&[u8]>,
) -> Result<(ebc_scene::Document, Vec<CardWarning>), ApiError> {
    let pixels = match cover {
        Some(bytes) => Some(Cover::Pixels(preview_cover(bytes)?)),
        None => None,
    };
    let mut prepared = prepare(request_json, pixels, None)?;
    // A preview needs no real creation time: any fixed one keeps it deterministic.
    prepared.request.created_unix.get_or_insert(0);
    ebc_card::compose(&prepared.request, &prepared.fonts).map_err(|e| card_error(&e))
}

/// Both sides of the card as SVG, with the warnings a preview can know about.
///
/// The cover is drawn from a copy at most [`PREVIEW_COVER_SIDE_PX`] pixels on its long
/// side, so resolution warnings are left to [`render_card`] and to `cover_probe`.
///
/// Returns `{"pages": [outside, inside], "warnings": [{"code", "message", "params"}]}`.
/// The SVG holds outlines and images only (no script, no external reference); show it
/// through an `<img>` all the same, since the texts in it come from the user.
///
/// # Errors
/// As [`render_card`].
pub fn preview_card(request_json: &str, cover: Option<&[u8]>) -> Result<String, ApiError> {
    let (document, warnings) = compose_preview(request_json, cover)?;
    let pages =
        (0..document.pages.len()).map(|i| ebc_card::svg::render_page(&document, i)).collect();
    let warnings = warnings.iter().filter(|w| !print_only(w)).map(warning_dto).collect();
    to_json(&PreviewDto { pages, warnings })
}

/// One side of the card as SVG: page 0 is the outside (back | spine | front), page 1
/// the inside.
///
/// # Errors
/// As [`render_card`]; `unknown_page` for any other page.
pub fn preview_svg(request_json: &str, cover: Option<&[u8]>, page: u32) -> Result<String, ApiError> {
    if page > 1 {
        return Err(ApiError::new("unknown_page", "a card has pages 0 (outside) and 1 (inside)"));
    }
    let (document, _) = compose_preview(request_json, cover)?;
    Ok(ebc_card::svg::render_page(&document, page as usize))
}

// ---- What the page can offer -----------------------------------------------------------

#[derive(Serialize)]
struct OptionsDto {
    formats: Vec<FormatDto>,
    profiles: Vec<ProfileDto>,
    rights: Vec<RightsDto>,
    spines: Vec<SpineDto>,
    title_fonts: Vec<FontDto>,
    locales: Vec<LocaleDto>,
    defaults: DefaultsDto,
    limits: LimitsDto,
}

#[derive(Serialize)]
struct FormatDto {
    name: &'static str,
    cover_mm: (f64, f64),
    spine_mm: f64,
    spread_mm: (f64, f64),
}

#[derive(Serialize)]
struct ProfileDto {
    name: &'static str,
    /// `"home_print"` or `"professional_order"`.
    route: &'static str,
    /// A PDF/X file: needs `created_unix` and an output-intent profile.
    pdfx: bool,
    bleed_mm: f64,
    /// `"a4"`, `"letter"`, … for home profiles.
    sheet: Option<&'static str>,
    marks: bool,
    /// The formats that fit this profile's page.
    formats: Vec<&'static str>,
}

#[derive(Serialize)]
struct RightsDto {
    /// Id of the sentence the user ticks.
    id: &'static str,
    /// The rights basis it stands for.
    basis: &'static str,
    home_print: bool,
    professional_order: bool,
    /// Whether the basis lets artwork be adapted once the user also says so.
    adaptation: bool,
}

#[derive(Serialize)]
struct SpineDto {
    name: &'static str,
    needs_cover: bool,
    needs_authorisation: bool,
}

#[derive(Serialize)]
struct FontDto {
    id: &'static str,
    file: &'static str,
    family: String,
    style: String,
    licence: &'static str,
}

#[derive(Serialize)]
struct LocaleDto {
    code: &'static str,
    bcp47: &'static str,
    name: &'static str,
    /// `"top_to_bottom"` or `"bottom_to_top"`: how a spine title reads by default.
    spine_direction: &'static str,
}

#[derive(Serialize)]
struct DefaultsDto {
    format: &'static str,
    profile: &'static str,
    locale: &'static str,
    spine: &'static str,
    title_font: &'static str,
}

#[derive(Serialize)]
struct LimitsDto {
    max_cover_bytes: usize,
    max_cover_side_px: u32,
    max_text_bytes: usize,
}

/// The profiles a page offers, home printing first.
pub const PROFILES: [&str; 5] =
    ["home_a4", "home_letter", "home_a3", "upload_eu", "professional_eu"];

/// Everything a page can offer, so that it hard-codes none of it: formats, print
/// profiles (with the formats that fit each), the rights declarations and the routes
/// each opens, spine styles, title fonts, locales, defaults and limits.
///
/// # Errors
/// `internal` if the bundled fonts cannot be read (a build problem).
pub fn card_options() -> Result<String, ApiError> {
    let presets = CardFormat::presets();
    let profiles = PROFILES
        .iter()
        .filter_map(|name| PrintProfile::by_name(name).map(|profile| (*name, profile)))
        .map(|(name, profile)| ProfileDto {
            name,
            route: route_name(route_of(&profile)),
            pdfx: profile.standard != PdfStandard::Plain,
            bleed_mm: profile.bleed,
            sheet: profile.sheet.map(|sheet| sheet.name()),
            marks: profile.marks.any(),
            formats: presets
                .iter()
                .filter(|(_, format)| {
                    ebc_layout::new_page(format, &profile).is_ok()
                        && ebc_layout::check_marks_fit(format, &profile).is_ok()
                })
                .map(|(name, _)| *name)
                .collect(),
        })
        .collect();
    let base = base_fonts()?;
    let face = |id: &str| match id {
        "bebas_neue" => base.title_alt.as_ref().unwrap_or(&base.title),
        "atkinson_bold" => &base.body_bold,
        "courier_prime_bold" => &base.code,
        _ => &base.title,
    };
    to_json(&OptionsDto {
        formats: presets
            .iter()
            .map(|(name, format)| FormatDto {
                name,
                cover_mm: (format.cover.w, format.cover.h),
                spine_mm: format.spine_width,
                spread_mm: (format.spread().w, format.spread().h),
            })
            .collect(),
        profiles,
        rights: RIGHTS_CHOICES
            .iter()
            .map(|(id, basis)| RightsDto {
                id,
                basis: basis.as_str(),
                home_print: basis.allows(PrintRoute::HomePrint),
                professional_order: basis.allows(PrintRoute::ProfessionalOrder),
                adaptation: allows_adaptation(*basis),
            })
            .collect(),
        spines: SPINE_STYLES
            .iter()
            .map(|(name, needs_cover, needs_authorisation)| SpineDto {
                name,
                needs_cover: *needs_cover,
                needs_authorisation: *needs_authorisation,
            })
            .collect(),
        title_fonts: TITLE_FONTS
            .iter()
            .map(|(id, file)| FontDto {
                id,
                file,
                family: face(id).family_name(),
                style: face(id).style_name(),
                licence: "OFL-1.1",
            })
            .collect(),
        locales: Locale::ALL
            .iter()
            .map(|locale| LocaleDto {
                code: locale.code(),
                bcp47: locale.bcp47(),
                name: ebc_i18n::native_name(*locale),
                spine_direction: ebc_i18n::spine_direction(*locale).as_str(),
            })
            .collect(),
        defaults: DefaultsDto {
            format: DEFAULT_FORMAT,
            profile: DEFAULT_PROFILE,
            locale: Locale::default().code(),
            spine: "typeset",
            title_font: "spectral_semibold",
        },
        limits: LimitsDto {
            max_cover_bytes: ebc_card::MAX_COVER_BYTES,
            max_cover_side_px: ebc_card::MAX_COVER_SIDE_PX,
            max_text_bytes: MAX_TEXT_BYTES,
        },
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::cover::tests::{framed_cover, png};
    use super::*;

    fn value(json: &str) -> Value {
        serde_json::from_str(json).expect("the API returns JSON")
    }

    fn code() -> String {
        GiftCode::from_payload(123_456_789).expect("in range").compact()
    }

    fn request() -> Value {
        json!({
            "book": {"title": "Les Vagues du soir", "contributors": [{"name": "Jeanne Roy"}]},
            "code": code(),
            "base_url": "https://ebk.example",
            "giver": "Alice",
            "recipient": "Bob",
            "message": "Bonne lecture !",
            "locale": "fr",
            "format": "poche",
            "profile": "home_a4"
        })
    }

    fn with(mut base: Value, patch: Value) -> String {
        let (Some(target), Some(source)) = (base.as_object_mut(), patch.as_object()) else {
            panic!("objects expected");
        };
        for (key, item) in source {
            target.insert(key.clone(), item.clone());
        }
        base.to_string()
    }

    fn warning_codes(json: &str) -> Vec<String> {
        value(json)
            .as_array()
            .expect("an array of warnings")
            .iter()
            .filter_map(|w| w["code"].as_str().map(str::to_owned))
            .collect()
    }

    /// Whether the PDF holds an image XObject (a cover) — the typographic card has none.
    fn has_image(pdf: &[u8]) -> bool {
        pdf.windows(14).any(|w| w == b"/Subtype /Image" || w.starts_with(b"/Subtype/Image"))
    }

    #[test]
    fn the_default_is_the_typographic_card() {
        let card = render_card(&request().to_string(), None, None).expect("rendered");
        assert!(card.pdf.starts_with(b"%PDF-"));
        assert!(card.pdf.len() > 10_000);
        assert!(!has_image(&card.pdf));
        assert_eq!(warning_codes(&card.warnings_json), Vec::<String>::new());
        // Deterministic.
        let again = render_card(&request().to_string(), None, None).expect("rendered");
        assert_eq!(card.pdf, again.pdf);
    }

    #[test]
    fn a_cover_without_a_declared_basis_is_never_printed() {
        let cover = framed_cover(600, 960);
        let card = render_card(&request().to_string(), Some(cover.clone()), None).expect("ok");
        assert!(!has_image(&card.pdf), "no rights basis: the typographic card");
        assert_eq!(warning_codes(&card.warnings_json), ["cover_without_rights"]);
        // Even bytes that are not an image are simply ignored without a basis.
        let card = render_card(&request().to_string(), Some(b"junk".to_vec()), None).expect("ok");
        assert!(!has_image(&card.pdf));
        // …and the preview agrees.
        let preview = value(&preview_card(&request().to_string(), Some(&cover)).expect("ok"));
        assert!(!preview["pages"][0].as_str().expect("svg").contains("<image"));
    }

    #[test]
    fn a_declared_cover_is_printed_at_home() {
        let cover = framed_cover(900, 1440);
        for (declaration, basis) in RIGHTS_CHOICES {
            let json = with(
                request(),
                json!({"cover": {"rights": basis.as_str(), "declaration": declaration}}),
            );
            let card = render_card(&json, Some(cover.clone()), None).expect(declaration);
            assert!(has_image(&card.pdf), "{declaration}");
            let codes = warning_codes(&card.warnings_json);
            assert_eq!(
                codes.contains(&"own_printer_only".to_owned()),
                basis == RightsBasis::UserPrivateCopy,
                "{declaration}: {codes:?}"
            );
        }
    }

    #[test]
    fn the_professional_route_refuses_a_private_copy() {
        let cover = framed_cover(900, 1440);
        let pro = json!({"profile": "upload_eu", "created_unix": 1_780_000_000});
        let private = with(
            with(request(), pro.clone()).parse().expect("json"),
            json!({"cover": {"rights": "user_private_copy", "declaration": "private_copy"}}),
        );
        let err = render_card(&private, Some(cover.clone()), None).expect_err("refused");
        assert_eq!(err.code, "rights_route_refused");
        assert_eq!(preview_card(&private, Some(&cover)).expect_err("refused").code, "rights_route_refused");

        // The bases that open the route do print — as PDF/X, with the fallback profile
        // reported when the page supplies none.
        for basis in ["rights_holder_declared", "public_domain"] {
            let json = with(with(request(), pro.clone()).parse().expect("json"), json!({"cover": {"rights": basis}}));
            let card = render_card(&json, Some(cover.clone()), None).expect(basis);
            assert!(has_image(&card.pdf), "{basis}");
            let codes = warning_codes(&card.warnings_json);
            assert!(codes.contains(&"cover_not_converted".to_owned()), "{codes:?}");
        }
    }

    #[test]
    fn a_page_cannot_claim_a_resellers_licence() {
        let cover = framed_cover(900, 1440);
        for basis in ["print_licence", "promo_all_media"] {
            let json = with(request(), json!({"cover": {"rights": basis}}));
            let err = render_card(&json, Some(cover.clone()), None).expect_err(basis);
            assert_eq!(err.code, "rights_basis_not_available", "{basis}");
        }
        // A declaration that does not go with the basis, or an invented basis.
        let json = with(request(), json!({"cover": {"rights": "public_domain", "declaration": "own_work"}}));
        assert_eq!(render_card(&json, Some(cover.clone()), None).expect_err("mismatch").code, "unknown_declaration");
        let json = with(request(), json!({"cover": {"rights": "fair_use"}}));
        assert_eq!(render_card(&json, Some(cover), None).expect_err("invented").code, "invalid_json");
    }

    #[test]
    fn placeholders_and_non_images_are_refused_as_covers() {
        let json = with(request(), json!({"cover": {"rights": "public_domain"}}));
        let err = render_card(&json, Some(b"<html>not an image</html>".to_vec()), None).expect_err("html");
        assert_eq!(err.code, "not_an_image");
        let pixel = png(1, 1, |_, _| [255, 255, 255]);
        assert_eq!(render_card(&json, Some(pixel), None).expect_err("1 px").code, "cover_placeholder");
        let mut truncated = framed_cover(600, 960);
        truncated.truncate(truncated.len() / 3);
        assert_eq!(render_card(&json, Some(truncated), None).expect_err("truncated").code, "cover_decode");
    }

    #[test]
    fn adapting_the_cover_needs_the_tick_and_a_basis_that_allows_it() {
        let cover = framed_cover(900, 1440);
        let edge = |cover_rights: Value| with(request(), json!({"spine": "edge_continuation", "cover": cover_rights}));
        // No tick.
        let err = render_card(&edge(json!({"rights": "rights_holder_declared"})), Some(cover.clone()), None)
            .expect_err("no tick");
        assert_eq!(err.code, "adaptation_not_authorised");
        // The tick, but a basis that cannot adapt.
        for basis in ["public_domain", "user_private_copy", "none"] {
            let err = render_card(
                &edge(json!({"rights": basis, "adaptation_authorised": true})),
                Some(cover.clone()),
                None,
            )
            .expect_err(basis);
            assert_eq!(err.code, "adaptation_not_authorised", "{basis}");
        }
        // Both.
        let card = render_card(
            &edge(json!({"rights": "rights_holder_declared", "adaptation_authorised": true})),
            Some(cover.clone()),
            None,
        )
        .expect("authorised");
        assert!(has_image(&card.pdf));
        // The computed flat colour needs neither.
        let flat = with(request(), json!({"spine": "cover_colour", "cover": {"rights": "public_domain"}}));
        assert!(render_card(&flat, Some(cover), None).is_ok());
        // Without a printable cover the style is ignored, with a warning.
        let card = render_card(&with(request(), json!({"spine": "cover_colour"})), None, None).expect("ok");
        assert_eq!(warning_codes(&card.warnings_json), ["spine_choice_ignored"]);
    }

    #[test]
    fn professional_profiles_need_a_creation_time_and_take_an_output_profile() {
        let json = with(request(), json!({"profile": "upload_eu"}));
        assert_eq!(render_card(&json, None, None).expect_err("no time").code, "missing_creation_date");
        // The preview does not need one.
        assert!(preview_card(&json, None).is_ok());

        let json = with(request(), json!({"profile": "upload_eu", "created_unix": 1_780_000_000}));
        assert_eq!(
            render_card(&json, None, Some(b"not an ICC profile")).expect_err("bad icc").code,
            "invalid_output_profile"
        );
        let srgb = include_bytes!("../../../../assets/icc/sRGB-v2-micro.icc");
        assert_eq!(render_card(&json, None, Some(srgb)).expect_err("RGB").code, "invalid_output_profile");
        let cmyk = include_bytes!("../../../../assets/icc/CGATS001Compat-v2-micro.icc");
        let card = render_card(&json, None, Some(cmyk)).expect("with a CMYK profile");
        assert!(card.pdf.windows(11).any(|w| w == b"/GTS_PDFX4)" || w == b"GTS_PDFX" || w.starts_with(b"/GTS_PDFX")));
        assert!(!warning_codes(&card.warnings_json).contains(&"fallback_output_profile".to_owned()));
        // A home profile ignores the output profile altogether.
        let home = render_card(&request().to_string(), None, Some(b"junk")).expect("ignored");
        assert_eq!(home.pdf, render_card(&request().to_string(), None, None).expect("same").pdf);
    }

    #[test]
    fn the_preview_has_two_pages_of_safe_svg() {
        let mut hostile = request();
        hostile["book"]["title"] = json!("</title><script>alert(1)</script> & \"q\"");
        hostile["message"] = json!("<img src=x onerror=alert(1)>");
        hostile["giver"] = json!("]]><svg onload=alert(1)>");
        let v = value(&preview_card(&hostile.to_string(), None).expect("preview"));
        let pages = v["pages"].as_array().expect("pages");
        assert_eq!(pages.len(), 2);
        for page in pages {
            let svg = page.as_str().expect("svg");
            assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
            assert!(svg.ends_with("</svg>"));
            let lower = svg.to_lowercase();
            assert!(!lower.contains("<script") && !lower.contains("onerror") && !lower.contains("onload"));
            assert!(!lower.contains("href=\"http"), "no external reference");
        }
        assert_eq!(preview_svg(&request().to_string(), None, 0).expect("page 0"), value(&preview_card(&request().to_string(), None).expect("both"))["pages"][0]);
        assert!(preview_svg(&request().to_string(), None, 1).expect("page 1").len() > 1000);
        assert_eq!(preview_svg(&request().to_string(), None, 2).expect_err("page 2").code, "unknown_page");
    }

    #[test]
    fn the_preview_draws_a_scaled_cover_and_keeps_resolution_warnings_for_the_pdf() {
        let small = framed_cover(240, 384); // far too few pixels for print
        let json = with(request(), json!({"cover": {"rights": "public_domain"}}));
        let preview = value(&preview_card(&json, Some(&small)).expect("preview"));
        assert!(preview["pages"][0].as_str().expect("svg").contains("<image"));
        let codes: Vec<&str> =
            preview["warnings"].as_array().expect("warnings").iter().filter_map(|w| w["code"].as_str()).collect();
        assert!(!codes.iter().any(|c| c.contains("resolution")), "{codes:?}");
        let card = render_card(&json, Some(small.clone()), None).expect("pdf");
        assert!(warning_codes(&card.warnings_json).iter().any(|c| c.contains("resolution")));
        // The cache serves the same file again and forgets it for another.
        let again = value(&preview_card(&json, Some(&small)).expect("cached"));
        assert_eq!(preview["pages"], again["pages"]);
        let other = framed_cover(300, 384);
        let changed = value(&preview_card(&json, Some(&other)).expect("another"));
        assert_ne!(preview["pages"][0], changed["pages"][0]);
    }

    #[test]
    fn the_request_is_validated_field_by_field() {
        let cases: [(Value, &str); 12] = [
            (json!({"code": "BCDF-GHJK-LMNA"}), "invalid_code"),
            (json!({"locale": "tlh"}), "unknown_locale"),
            (json!({"format": "folio"}), "unknown_format"),
            (json!({"profile": "home_moon"}), "unknown_profile"),
            (json!({"spine": "provided"}), "unknown_spine"),
            (json!({"title_font": "comic_sans"}), "unknown_title_font"),
            (json!({"ground": "red"}), "invalid_colour"),
            (json!({"message": "x".repeat(MAX_TEXT_BYTES + 1)}), "text_too_long"),
            (json!({"legal_lines": ["a"; 9]}), "text_too_long"),
            (json!({"base_url": "ftp://ebk.example"}), "invalid_claim_url"),
            (json!({"book": {"title": "   "}}), "empty_title"),
            (json!({"surprise": true}), "invalid_json"),
        ];
        for (patch, code) in cases {
            let err = render_card(&with(request(), patch.clone()), None, None).expect_err(code);
            assert_eq!(err.code, code, "{patch}");
        }
        assert_eq!(render_card("", None, None).expect_err("empty").code, "invalid_json");
        assert_eq!(render_card("{}", None, None).expect_err("no book").code, "invalid_json");
    }

    #[test]
    fn every_option_changes_the_card_and_none_breaks_it() {
        let base = render_card(&request().to_string(), None, None).expect("base").pdf;
        for patch in [
            json!({"ground": "#224466"}),
            json!({"title_font": "bebas_neue"}),
            json!({"title_font": "atkinson_bold"}),
            json!({"title_font": "courier_prime_bold"}),
            json!({"locale": "en"}),
            json!({"format": "a6"}),
            json!({"profile": "home_letter"}),
            json!({"guides": true}),
            json!({"message": null}),
            json!({"store_name": "Septante (démo)"}),
            json!({"expires_unix": 1_800_000_000}),
            json!({"back_text": "Un livre choisi pour vous."}),
            json!({"display_url": "ebk.example/app"}),
        ] {
            let card = render_card(&with(request(), patch.clone()), None, None)
                .unwrap_or_else(|e| panic!("{patch}: {e}"));
            assert!(card.pdf.starts_with(b"%PDF-"), "{patch}");
            assert_ne!(card.pdf, base, "{patch} should change the output");
        }
        // Every locale and every (profile, format) pair the options advertise renders.
        let options = value(&card_options().expect("options"));
        for locale in options["locales"].as_array().expect("locales") {
            assert!(render_card(&with(request(), json!({"locale": locale["code"]})), None, None).is_ok());
        }
        for profile in options["profiles"].as_array().expect("profiles") {
            for format in profile["formats"].as_array().expect("formats") {
                let json = with(
                    request(),
                    json!({"profile": profile["name"], "format": format, "created_unix": 1_780_000_000}),
                );
                assert!(render_card(&json, None, None).is_ok(), "{} / {format}", profile["name"]);
            }
        }
    }

    #[test]
    fn options_describe_the_rights_gate() {
        let v = value(&card_options().expect("options"));
        let rights = v["rights"].as_array().expect("rights");
        assert_eq!(rights.len(), 5);
        for choice in rights {
            assert_eq!(choice["home_print"], true, "{choice}");
            let private = choice["id"] == "private_copy";
            assert_eq!(choice["professional_order"], !private, "{choice}");
            assert_eq!(choice["adaptation"], choice["basis"] == "rights_holder_declared", "{choice}");
        }
        let profiles = v["profiles"].as_array().expect("profiles");
        let home = profiles.iter().find(|p| p["name"] == "home_a4").expect("home_a4");
        assert_eq!((home["route"].as_str(), home["pdfx"].as_bool()), (Some("home_print"), Some(false)));
        assert!(home["formats"].as_array().expect("formats").contains(&json!("poche")));
        assert!(!home["formats"].as_array().expect("formats").contains(&json!("a5")), "A5 spread is wider than A4");
        let pro = profiles.iter().find(|p| p["name"] == "upload_eu").expect("upload_eu");
        assert_eq!((pro["route"].as_str(), pro["pdfx"].as_bool()), (Some("professional_order"), Some(true)));
        assert_eq!(pro["formats"].as_array().map(Vec::len), Some(4));
        assert_eq!(v["defaults"]["profile"], DEFAULT_PROFILE);
        assert_eq!(v["locales"].as_array().map(Vec::len), Some(7));
        let fonts = v["title_fonts"].as_array().expect("fonts");
        assert_eq!(fonts[0]["family"], "Spectral");
        assert!(fonts.iter().all(|f| f["licence"] == "OFL-1.1"));
        let spines = v["spines"].as_array().expect("spines");
        assert_eq!(spines[0]["name"], "typeset");
        assert_eq!(spines[2]["needs_authorisation"], true);
    }
}
