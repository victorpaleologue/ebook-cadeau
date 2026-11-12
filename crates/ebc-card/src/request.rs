//! What the caller hands to the composer: fonts, the cover with its rights basis, the
//! output profile and the card request itself.

use std::sync::Arc;

use ebc_color::{Rgb8, SrgbToCmyk};
use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::{BookMeta, GiftCode, Locale};
use ebc_scene::{OutputIntent, Pixmap};
use ebc_text::Font;

use crate::error::CardError;
use crate::rights::RightsBasis;

/// The fonts of a card.
///
/// Body, bold and code fonts are embedded as subsets (real, selectable text); fonts that
/// are variable or forbid subset embedding are drawn as outlines instead.
#[derive(Debug, Clone)]
pub struct FontSet {
    /// Running text: steps, message, back text, small print.
    pub body: Font,
    /// Bold companion of `body`: headings, names, step numbers, the address.
    pub body_bold: Font,
    /// The redemption code (a monospaced bold face reads best).
    pub code: Font,
    /// Titles: the typographic cover, the spine, the headline inside.
    pub title: Font,
    /// A second display face, tried on the spine when `title` lacks a character.
    pub title_alt: Option<Font>,
}

/// The fonts shipped with the project (`assets/fonts`, all under the SIL Open Font
/// License 1.1): Atkinson Hyperlegible for text, Courier Prime Bold for the code,
/// Spectral SemiBold for titles and Bebas Neue as the alternative display face.
///
/// # Errors
/// [`CardError::Font`] if a bundled file cannot be parsed — which a test rules out for
/// the files compiled in; the `Result` only keeps a corrupted build from panicking.
#[cfg(feature = "bundled-fonts")]
pub fn bundled_fonts() -> Result<FontSet, CardError> {
    macro_rules! font {
        ($file:literal) => {
            Font::from_bytes(
                include_bytes!(concat!("../../../assets/fonts/", $file)).as_slice(),
                0,
            )?
        };
    }
    Ok(FontSet {
        body: font!("AtkinsonHyperlegible-Regular.ttf"),
        body_bold: font!("AtkinsonHyperlegible-Bold.ttf"),
        code: font!("CourierPrime-Bold.ttf"),
        title: font!("Spectral-SemiBold.ttf"),
        title_alt: Some(font!("BebasNeue-Regular.ttf")),
    })
}

/// The cover image, **with the rights basis under which it may be printed**.
///
/// There is no way to hand over pixels without a basis. With
/// [`RightsBasis::None`] — or [`CoverInput::None`] — the image never reaches the print
/// file and the typographic card is produced (see [`crate::rights`]).
#[derive(Debug, Clone, Default)]
pub enum CoverInput {
    /// An encoded image file: JPEG, PNG or WebP.
    Encoded {
        /// The file's bytes.
        bytes: Vec<u8>,
        /// Why this cover may be printed.
        rights: RightsBasis,
    },
    /// Decoded pixels (alpha is composited on white).
    Pixels {
        /// The image.
        pixmap: Pixmap,
        /// Why this cover may be printed.
        rights: RightsBasis,
    },
    /// No cover: the typographic card.
    #[default]
    None,
}

impl CoverInput {
    /// The rights basis the cover carries ([`RightsBasis::None`] without a cover).
    #[must_use]
    pub fn rights(&self) -> RightsBasis {
        match self {
            Self::Encoded { rights, .. } | Self::Pixels { rights, .. } => *rights,
            Self::None => RightsBasis::None,
        }
    }

    /// `true` when there is an image **and** a rights basis other than `none`.
    #[must_use]
    pub fn is_printable(&self) -> bool {
        !matches!(self, Self::None) && self.rights() != RightsBasis::None
    }
}

/// What goes on the spine.
#[derive(Debug, Clone, Default)]
pub enum SpineChoice {
    /// **Default.** Title and author typeset from the metadata on the card's flat ground
    /// colour. Nothing is taken from the cover.
    #[default]
    Typeset,
    /// Typeset on one flat colour *computed* from the cover (its frame, its plain binding
    /// edge or its dominant colour — never pixels); the back panel takes the same colour.
    /// That colour is the default ground of a card with a cover, so this choice only
    /// shows where the ground is something else: paper white (the default when print
    /// argues against the computed colour) or a [`CardRequest::ground`] of the caller's.
    /// Needs a printable cover, no authorisation.
    CoverColour,
    /// **Adaptation — needs authorisation.** A mirrored, blurred strip of the cover
    /// behind the typeset text.
    EdgeContinuation,
    /// **Adaptation — needs authorisation.** An image of the real spine (upright, head at
    /// the top, trimmed), re-fitted to the panel.
    Provided(Pixmap),
    /// **Adaptation — needs authorisation.** A flat wraparound cover and the pixel
    /// columns of its spine.
    FullWrap {
        /// The wraparound image (back | spine | front, trimmed).
        image: Pixmap,
        /// The spine's columns as `(start, end)`, `end` exclusive.
        spine_px: (u32, u32),
    },
}

impl SpineChoice {
    /// A short stable name: `"typeset"`, `"cover_colour"`, `"edge_continuation"`,
    /// `"provided"` or `"full_wrap"`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Typeset => "typeset",
            Self::CoverColour => "cover_colour",
            Self::EdgeContinuation => "edge_continuation",
            Self::Provided(_) => "provided",
            Self::FullWrap { .. } => "full_wrap",
        }
    }

    /// `true` for the styles that reproduce or alter artwork. They are refused unless
    /// the request says `adaptation_authorised` **and** the cover's rights basis is
    /// `print_licence` or `rights_holder_declared`.
    #[must_use]
    pub const fn requires_authorisation(&self) -> bool {
        matches!(self, Self::EdgeContinuation | Self::Provided(_) | Self::FullWrap { .. })
    }
}

/// The intended printing condition of a PDF/X file: a CMYK ICC profile and its names.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputProfile {
    icc: Arc<[u8]>,
    condition_identifier: String,
    info: String,
    printer_class: bool,
    class: String,
    fallback: bool,
}

impl OutputProfile {
    /// Wraps a CMYK ICC profile supplied by the caller.
    ///
    /// `condition_identifier` is the name of the characterisation in the ICC registry
    /// (`"FOGRA39"`, `"FOGRA51"`, `"FOGRA52"`, …) and `info` a human-readable name such
    /// as `"ISO Coated v2 300% (ECI)"`. For European printers pass the profile the
    /// printer names — typically ECI's FOGRA39 or FOGRA51 profiles, which this project
    /// does not ship.
    ///
    /// # Errors
    /// [`CardError::InvalidOutputProfile`] when the data is not an ICC profile, is not
    /// CMYK, or `condition_identifier` is blank. A profile that is not of class `prtr`
    /// is accepted and reported by a warning when the card is composed.
    pub fn new(
        icc: impl Into<Arc<[u8]>>,
        condition_identifier: &str,
        info: &str,
    ) -> Result<Self, CardError> {
        let icc: Arc<[u8]> = icc.into();
        let header = ebc_pdf::icc::inspect(&icc)
            .map_err(|reason| CardError::InvalidOutputProfile(reason.to_owned()))?;
        if header.components != 4 {
            return Err(CardError::InvalidOutputProfile(format!(
                "a CMYK profile is needed, this one has {} component(s)",
                header.components
            )));
        }
        let condition_identifier = condition_identifier.trim();
        if condition_identifier.is_empty() {
            return Err(CardError::InvalidOutputProfile(
                "the output condition identifier is empty".to_owned(),
            ));
        }
        Ok(Self {
            icc,
            condition_identifier: condition_identifier.to_owned(),
            info: info.trim().to_owned(),
            printer_class: header.is_output_class(),
            class: String::from_utf8_lossy(&header.device_class).trim().to_owned(),
            fallback: false,
        })
    }

    /// "ISO Coated v2 300% (basICColor)": a printer-class CMYK profile for the FOGRA39L
    /// characterisation, 300 % ink limit, under the zlib licence (see
    /// `crates/ebc-card/assets/icc/README.md` for provenance and caveats). The default
    /// output intent of PDF/X profiles.
    ///
    /// # Errors
    /// [`CardError::InvalidOutputProfile`] if the compiled-in file is corrupt (a test
    /// rules it out).
    #[cfg(feature = "bundled-fogra39")]
    pub fn bundled_fogra39() -> Result<Self, CardError> {
        Self::new(
            include_bytes!("../assets/icc/ISOcoated_v2_300_bas.icc").as_slice(),
            "FOGRA39",
            "ISO Coated v2 300% (basICColor)",
        )
    }

    /// The 8 kB CC0 profile of `assets/icc` (CGATS TR 001, a US web-offset condition).
    /// It is an *input*-class profile meant for viewing CMYK images: usable to make a
    /// PDF/X file well-formed in tests, **not** for an order sent to a European printer.
    /// Composing with it yields [`CardWarning::FallbackOutputProfile`](crate::CardWarning::FallbackOutputProfile).
    ///
    /// # Errors
    /// [`CardError::InvalidOutputProfile`] if the compiled-in file is corrupt (a test
    /// rules it out).
    #[cfg(feature = "bundled-icc")]
    pub fn bundled_cc0_fallback() -> Result<Self, CardError> {
        let mut profile = Self::new(
            include_bytes!("../../../assets/icc/CGATS001Compat-v2-micro.icc").as_slice(),
            "CGATS TR 001",
            "CGATS TR 001 (SWOP) compatible, compact CC0 profile",
        )?;
        profile.fallback = true;
        Ok(profile)
    }

    /// The profile used when the request supplies none: the bundled FOGRA39 profile,
    /// else the CC0 fallback, depending on the features compiled in.
    ///
    /// # Errors
    /// [`CardError::NoOutputProfile`] when neither is compiled in.
    pub fn default_for_build() -> Result<Self, CardError> {
        #[cfg(feature = "bundled-fogra39")]
        {
            Self::bundled_fogra39()
        }
        #[cfg(all(not(feature = "bundled-fogra39"), feature = "bundled-icc"))]
        {
            Self::bundled_cc0_fallback()
        }
        #[cfg(all(not(feature = "bundled-fogra39"), not(feature = "bundled-icc")))]
        {
            Err(CardError::NoOutputProfile)
        }
    }

    /// The ICC profile bytes.
    #[must_use]
    pub fn icc(&self) -> &Arc<[u8]> {
        &self.icc
    }

    /// The output condition identifier, e.g. `"FOGRA39"`.
    #[must_use]
    pub fn condition_identifier(&self) -> &str {
        &self.condition_identifier
    }

    /// The human-readable name.
    #[must_use]
    pub fn info(&self) -> &str {
        &self.info
    }

    /// `true` when the profile is of ICC class `prtr` (output device), the class PDF/X
    /// expects.
    #[must_use]
    pub fn is_printer_class(&self) -> bool {
        self.printer_class
    }

    /// The profile's ICC device class signature (`"prtr"`, `"scnr"`, `"mntr"`, …).
    #[must_use]
    pub fn device_class(&self) -> &str {
        &self.class
    }

    /// `true` for [`OutputProfile::bundled_cc0_fallback`].
    #[must_use]
    pub fn is_fallback(&self) -> bool {
        self.fallback
    }

    /// The colour-managed transform from sRGB to this profile's CMYK: relative
    /// colorimetric with black-point compensation
    /// (`docs/research/03-print-requirements.md` §3.4), sampled on a 33³ grid.
    ///
    /// # Errors
    /// [`ebc_color::IccError`] when the profile's tables cannot be read by
    /// `ebc_color::icc` (an ICC v4 profile with `mAB`/`mBA` tables, or an input-class
    /// profile without `B2A` tables such as the CC0 fallback).
    pub fn colour_link(&self) -> Result<SrgbToCmyk, ebc_color::IccError> {
        ebc_color::CmykProfile::parse(&self.icc).map(|profile| SrgbToCmyk::relative_bpc(&profile))
    }

    pub(crate) fn to_output_intent(&self) -> OutputIntent {
        OutputIntent {
            condition_identifier: self.condition_identifier.clone(),
            info: self.info.clone(),
            registry_name: "http://www.color.org".to_owned(),
            icc_profile: Some(Arc::clone(&self.icc)),
            components: 4,
        }
    }
}

/// Everything that goes on one card.
///
/// Build one with [`CardRequest::new`] and set the public fields that differ from the
/// defaults.
#[derive(Debug, Clone)]
pub struct CardRequest {
    /// The book: title, subtitle, authors, publisher and language are printed or used.
    /// Its description is never printed (blurbs need their own permission): pass
    /// `back_text` instead.
    pub book: BookMeta,
    /// The cover image and its rights basis. Default: none (the typographic card).
    pub cover: CoverInput,
    /// The cover may lose up to the bleed at its edges: full-bleed placement instead of
    /// the letterboxed default. Only say so when the title carries that authorisation.
    pub trim_authorised: bool,
    /// The artwork may be adapted (see [`SpineChoice::requires_authorisation`]). Only
    /// honoured with a rights basis of `print_licence` or `rights_holder_declared`.
    pub adaptation_authorised: bool,
    /// The credit printed for the cover, e.g. `"© Éditions du Lac / Jeanne Roy"`.
    /// Without it the line reads "Cover: © {publisher}" in the card's language.
    pub cover_credit: Option<String>,
    /// Text of the back panel; the default is a localised sentence saying what the card
    /// is.
    pub back_text: Option<String>,
    /// What goes on the spine. Default: typeset on the flat ground.
    pub spine: SpineChoice,
    /// The flat ground colour of the outside: back, spine and front. Used as given.
    /// Default (`None`): for a card with a cover, one colour computed from the cover (its
    /// frame, its plain binding edge or its dominant colour — never pixels), or paper
    /// white when that colour is too faint, too heavy in ink or leaves no legible text
    /// (see the crate documentation, "The ground of the outside"); for the typographic
    /// card, a colour picked from a fixed palette by a hash of the title.
    pub ground: Option<Rgb8>,
    /// The URL the QR code leads to, e.g. `"https://ebk.example/k/BCDFGHJKLMNP"`
    /// ([`ebc_model::claim_url`]). When it ends in `/k/<code>` the QR code carries it
    /// fully upper-cased ([`ebc_model::claim_url_qr`]).
    pub claim_url: String,
    /// The code typed by recipients without a phone; printed grouped.
    pub code: GiftCode,
    /// The address printed in clear, normally a bare host such as `"ebk.example"` (with
    /// its port when it is not the scheme's default). Printed in lower case.
    pub display_url: String,
    /// Who gives the book.
    pub giver: Option<String>,
    /// Who receives it.
    pub recipient: Option<String>,
    /// The personal message; without one, ruled space is left for handwriting.
    pub message: Option<String>,
    /// The name of the store that fulfils the gift. When set, steps 2 and 3 read "sign in
    /// to your {store} account" / "the book appears in your library", and the store is
    /// named in the imprint line.
    pub store_name: Option<String>,
    /// Extra legal or information lines for the inside (format, contact, …), set after
    /// the localised defaults.
    pub legal_lines: Vec<String>,
    /// Where to get help, printed in the legal lines (e.g. `"ebk.example/aide"`).
    pub help_url: Option<String>,
    /// When the gift must be collected by, as seconds since the Unix epoch; `None` prints
    /// "no time limit".
    pub expires_unix: Option<i64>,
    /// Language of the printed strings.
    pub locale: Locale,
    /// Size of the card.
    pub format: CardFormat,
    /// How it is printed. Sheet-based profiles are the home-print route, all others the
    /// professional route (see [`crate::rights::route_of`]).
    pub profile: PrintProfile,
    /// Creation time written to the PDF, as seconds since the Unix epoch. Required by
    /// PDF/X profiles; the composer never reads the clock.
    pub created_unix: Option<i64>,
    /// The output intent of a PDF/X file. Default: [`OutputProfile::default_for_build`].
    /// Ignored by plain-PDF (home) profiles.
    pub output_profile: Option<OutputProfile>,
    /// Add the proofing overlay of [`ebc_layout::guides`] (bleed, trim, creases, safe
    /// areas) on top of both pages, as a layer a viewer can hide. The result is a proof,
    /// not a print file: it is written as a plain PDF, without the PDF/X label and output
    /// intent of the profile.
    pub guides: bool,
}

impl CardRequest {
    /// A request for the default card — typographic, poche format, professional upload
    /// profile, French — for `book`, redeemable with `code` at `base_url`
    /// (e.g. `"https://ebk.example"`): the claim URL is [`ebc_model::claim_url`] and the
    /// address printed in clear is the host of `base_url`.
    #[must_use]
    pub fn new(book: BookMeta, code: GiftCode, base_url: &str) -> Self {
        Self {
            book,
            cover: CoverInput::None,
            trim_authorised: false,
            adaptation_authorised: false,
            cover_credit: None,
            back_text: None,
            spine: SpineChoice::Typeset,
            ground: None,
            claim_url: ebc_model::claim_url(base_url, &code),
            code,
            display_url: host_of(base_url),
            giver: None,
            recipient: None,
            message: None,
            store_name: None,
            legal_lines: Vec::new(),
            help_url: None,
            expires_unix: None,
            locale: Locale::default(),
            format: CardFormat::poche(),
            profile: PrintProfile::upload_eu(),
            created_unix: None,
            output_profile: None,
            guides: false,
        }
    }
}

/// The host of a base URL, for display: scheme, user info, path and a leading `www.`
/// removed, lower-cased. A port is kept unless it is the scheme's default (`:443` for
/// `https`, `:80` for `http`): an address typed without it would lead nowhere. (The
/// same rule as `typed_host` in `ebc-server`'s card composer.)
pub(crate) fn host_of(base_url: &str) -> String {
    let rest = base_url.trim();
    let (scheme, rest) = match rest.split_once("://") {
        Some((scheme, rest)) => (scheme.to_ascii_lowercase(), rest),
        None => (String::new(), rest),
    };
    let rest = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let rest = rest.rsplit_once('@').map_or(rest, |(_, r)| r);
    let (host, port) = match rest.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => {
            (host, Some(port.trim_start_matches('0')))
        }
        Some((host, "")) => (host, None),
        _ => (rest, None),
    };
    let host = host.to_lowercase();
    let host = host.strip_prefix("www.").map(str::to_owned).unwrap_or(host);
    let default = match scheme.as_str() {
        "https" => Some("443"),
        "http" => Some("80"),
        _ => None,
    };
    match port {
        Some(port) if Some(port) != default && !port.is_empty() => format!("{host}:{port}"),
        _ => host,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_host() {
        assert_eq!(host_of("https://ebk.example"), "ebk.example");
        assert_eq!(host_of("user@ebk.example/a"), "ebk.example");
        assert_eq!(host_of(""), "");
        // A port is part of the address, unless it is the scheme's default.
        assert_eq!(host_of("https://WWW.Ebk.Example:8443/x?y#z"), "ebk.example:8443");
        assert_eq!(host_of("http://127.0.0.1:8080"), "127.0.0.1:8080");
        assert_eq!(host_of("https://ebk.example:443/"), "ebk.example");
        assert_eq!(host_of("http://ebk.example:80"), "ebk.example");
        assert_eq!(host_of("http://ebk.example:443"), "ebk.example:443");
        assert_eq!(host_of("https://ebk.example:"), "ebk.example");
        assert_eq!(host_of("ebk.example:8080"), "ebk.example:8080");
    }

    #[test]
    fn spine_choices_that_adapt_artwork() {
        let px = Pixmap::filled(2, 2, [0, 0, 0, 255]);
        assert!(!SpineChoice::Typeset.requires_authorisation());
        assert!(!SpineChoice::CoverColour.requires_authorisation());
        assert!(SpineChoice::EdgeContinuation.requires_authorisation());
        assert!(SpineChoice::Provided(px.clone()).requires_authorisation());
        assert!(SpineChoice::FullWrap { image: px, spine_px: (0, 1) }.requires_authorisation());
        assert_eq!(SpineChoice::default().name(), "typeset");
    }

    #[test]
    fn a_cover_without_rights_is_not_printable() {
        let px = Pixmap::filled(2, 2, [0, 0, 0, 255]);
        assert!(!CoverInput::None.is_printable());
        assert!(
            !CoverInput::Pixels { pixmap: px.clone(), rights: RightsBasis::None }.is_printable()
        );
        assert!(
            CoverInput::Pixels { pixmap: px, rights: RightsBasis::PublicDomain }.is_printable()
        );
        assert_eq!(CoverInput::default().rights(), RightsBasis::None);
    }

    #[test]
    fn output_profiles_are_validated() {
        assert!(matches!(
            OutputProfile::new(vec![0u8; 16], "FOGRA39", "x"),
            Err(CardError::InvalidOutputProfile(_))
        ));
        #[cfg(feature = "bundled-fogra39")]
        {
            let p = OutputProfile::bundled_fogra39().expect("bundled profile");
            assert!(p.is_printer_class() && !p.is_fallback());
            assert_eq!((p.condition_identifier(), p.device_class()), ("FOGRA39", "prtr"));
            assert!(matches!(
                OutputProfile::new(Arc::clone(p.icc()), "  ", "x"),
                Err(CardError::InvalidOutputProfile(_))
            ));
        }
        #[cfg(feature = "bundled-icc")]
        {
            let p = OutputProfile::bundled_cc0_fallback().expect("bundled profile");
            assert!(!p.is_printer_class() && p.is_fallback());
            assert_eq!(p.device_class(), "scnr");
            let srgb = include_bytes!("../../../assets/icc/sRGB-v2-micro.icc").as_slice();
            assert!(matches!(
                OutputProfile::new(srgb, "sRGB", "x"),
                Err(CardError::InvalidOutputProfile(_))
            ));
        }
    }

    #[cfg(feature = "bundled-fonts")]
    #[test]
    fn bundled_fonts_load_and_may_be_embedded() {
        let fonts = bundled_fonts().expect("bundled fonts");
        for font in [&fonts.body, &fonts.body_bold, &fonts.code, &fonts.title] {
            assert!(font.allows_subset_embedding(), "{}", font.full_name());
            assert!(!font.is_variable());
        }
        assert!(fonts.code.is_monospaced());
        assert!(fonts.title_alt.is_some());
    }
}
