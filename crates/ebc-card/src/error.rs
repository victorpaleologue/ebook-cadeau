//! Errors and warnings of the composer.

use core::fmt;

use ebc_scene::PdfStandard;
use ebc_spine::SpineNote;

use crate::rights::{PrintRoute, RightsBasis};

/// Why a card could not be composed.
///
/// Anything that still yields a printable card (a long text, a missing glyph, a soft
/// cover image) is a [`CardWarning`] instead.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CardError {
    /// The book has no title (empty or blank): there is nothing to set on the cover and
    /// the spine, and PDF/X requires a document title.
    #[error("the book has no title")]
    EmptyTitle,
    /// The claim URL is empty, has no `http`/`https` scheme, or contains white space or
    /// control characters.
    #[error("the claim URL is not usable: {0}")]
    InvalidClaimUrl(&'static str),
    /// The format or the profile is not usable, or the card does not fit the sheet.
    #[error(transparent)]
    Layout(#[from] ebc_layout::LayoutError),
    /// The spine could not be designed.
    #[error(transparent)]
    Spine(#[from] ebc_spine::SpineError),
    /// The claim URL does not fit a QR code.
    #[error(transparent)]
    Qr(#[from] ebc_qr::QrError),
    /// The PDF writer refused the document.
    #[error(transparent)]
    Pdf(#[from] ebc_pdf::PdfError),
    /// A font could not be loaded.
    #[error(transparent)]
    Font(#[from] ebc_text::FontError),
    /// A font of the [`FontSet`](crate::FontSet) may neither be embedded nor drawn as
    /// outlines (its `OS/2.fsType` forbids both).
    #[error("font \"{name}\" may not be embedded or outlined in a print file")]
    FontNotUsable {
        /// The font's full name.
        name: String,
    },
    /// The cover bytes are not a JPEG, PNG or WebP image that can be decoded.
    #[error("the cover image cannot be decoded: {0}")]
    CoverDecode(String),
    /// The cover has more pixels than the composer accepts.
    #[error(
        "the cover image is too large: {width} x {height} px (at most {max_side} px per side and {max_pixels} pixels)"
    )]
    CoverTooLarge {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Largest side accepted.
        max_side: u32,
        /// Largest pixel count accepted.
        max_pixels: u64,
    },
    /// The cover's rights basis does not open the print route of the profile.
    #[error("a cover with rights basis `{basis}` cannot be printed on {route}")]
    RightsRouteRefused {
        /// The rights basis of the cover.
        basis: RightsBasis,
        /// The route the profile stands for.
        route: PrintRoute,
    },
    /// A spine style that samples the cover or re-fits a provided spine was asked for
    /// without `adaptation_authorised = true` **and** a rights basis of `print_licence`
    /// or `rights_holder_declared`.
    #[error(
        "the spine style {style} adapts artwork: it needs `adaptation_authorised` and a rights basis of print_licence or rights_holder_declared (got `{basis}`, authorised: {authorised})"
    )]
    AdaptationNotAuthorised {
        /// The spine style asked for.
        style: &'static str,
        /// The rights basis of the cover (`none` without a cover).
        basis: RightsBasis,
        /// Whether the request said `adaptation_authorised`.
        authorised: bool,
    },
    /// The profile's PDF standard cannot carry the cover: PDF/X-1a forbids RGB, and the
    /// composer has no colour-managed conversion to CMYK (it never converts a cover
    /// naively).
    #[error("{standard:?} cannot carry an RGB cover image and no colour-managed CMYK conversion is available")]
    CoverColourSpace {
        /// The standard of the profile.
        standard: PdfStandard,
    },
    /// A PDF/X profile needs a creation time and `created_unix` is `None` (the composer
    /// never reads the clock).
    #[error("a PDF/X profile needs `created_unix`")]
    MissingCreationDate,
    /// A PDF/X profile needs an output-intent ICC profile; none was supplied and none is
    /// compiled in (features `bundled-fogra39` and `bundled-icc` are both off).
    #[error("no output-intent ICC profile: supply `output_profile` or enable a bundled one")]
    NoOutputProfile,
    /// The output-intent profile supplied is not a CMYK ICC profile.
    #[error("the output-intent profile is not usable: {0}")]
    InvalidOutputProfile(String),
    /// The panels are too small for the content that must be on them (the QR code at
    /// 20 mm, the code and the three steps).
    #[error(
        "the {panel} panel is too small: its content needs {needed_mm:.0} mm of height at the smallest type size but only {available_mm:.0} mm is safe"
    )]
    FormatTooSmall {
        /// Which panel overflowed.
        panel: &'static str,
        /// Height the content needs, in millimetres.
        needed_mm: f64,
        /// Height available inside the safe area, in millimetres.
        available_mm: f64,
    },
    /// An unknown rights-basis name was parsed.
    #[error("unknown rights basis {0:?}")]
    UnknownRightsBasis(String),
    /// A bug: something would have been painted over the cover. The composer checks its
    /// own output and refuses to return such a document.
    #[error("internal error: an item overlaps the cover image ({0})")]
    CoverOverlap(String),
}

/// A piece of text placed on the card, named in warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextField {
    /// The book's title.
    Title,
    /// The book's subtitle.
    Subtitle,
    /// The author line.
    Author,
    /// The publisher's name.
    Publisher,
    /// The text of the back panel.
    BackText,
    /// The personal message.
    Message,
    /// The giver's name.
    Giver,
    /// The recipient's name.
    Recipient,
    /// The three steps.
    Steps,
    /// The line for people without a phone.
    NoPhone,
    /// The legal and information lines.
    Legal,
    /// The address printed in clear.
    DisplayUrl,
    /// The store's name.
    StoreName,
    /// The cover credit line.
    Credit,
    /// The product and imprint lines of the back panel.
    Imprint,
    /// The instructions printed in the margin of a home sheet.
    SheetHint,
}

impl fmt::Display for TextField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Title => "title",
            Self::Subtitle => "subtitle",
            Self::Author => "author",
            Self::Publisher => "publisher",
            Self::BackText => "back text",
            Self::Message => "message",
            Self::Giver => "giver's name",
            Self::Recipient => "recipient's name",
            Self::Steps => "steps",
            Self::NoPhone => "no-phone line",
            Self::Legal => "legal lines",
            Self::DisplayUrl => "address",
            Self::StoreName => "store name",
            Self::Credit => "cover credit",
            Self::Imprint => "imprint lines",
            Self::SheetHint => "sheet instructions",
        })
    }
}

/// Something the caller should know about a card that was composed all the same.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum CardWarning {
    /// A cover image was supplied with rights basis `none`: it was not used, and the
    /// typographic card was produced.
    #[error("the cover has no rights basis: it was left out and the typographic card was produced")]
    CoverWithoutRights,
    /// The cover is printed below the profile's warning level (`below_minimum`: below
    /// its minimum, where a printer may reject the file). It is never upscaled.
    #[error(
        "the cover is printed at {ppi:.0} ppi, below the {threshold_ppi:.0} ppi the profile asks for"
    )]
    LowResolution {
        /// Effective resolution at the placed size.
        ppi: f64,
        /// The threshold that was missed: the profile's minimum or its warning level.
        threshold_ppi: f64,
        /// `true` when the profile's hard minimum was missed.
        below_minimum: bool,
    },
    /// The cover had more pixels than print needs and was resampled.
    #[error("the cover was downsampled from {from_ppi:.0} to {to_ppi:.0} ppi")]
    CoverDownsampled {
        /// Effective resolution of the original at the placed size.
        from_ppi: f64,
        /// Resolution it was resampled to.
        to_ppi: f64,
    },
    /// Full-bleed placement (`trim_authorised`) hides a noticeable part of the cover
    /// because its proportions differ from the panel's.
    #[error("full-bleed placement crops {percent:.0} % of the cover's {dimension}")]
    HeavyCrop {
        /// Share of the dimension that is cropped, in per cent.
        percent: f64,
        /// `"width"` or `"height"`.
        dimension: &'static str,
    },
    /// No credit was supplied for the cover and the book has no publisher: a generic
    /// credit line was printed. Record it (`credit_missing`).
    #[error("no cover credit was supplied and the publisher is unknown: a generic credit is printed")]
    CoverCreditMissing,
    /// The cover is embedded as sRGB and left to the printer's colour management: no
    /// colour-managed conversion to the output intent is available in this build.
    #[error("the cover is embedded as sRGB, not converted to the output-intent colour space")]
    CoverNotConverted,
    /// The cover's embedded ICC profile is ignored: the image is treated as sRGB.
    #[error("the cover's embedded ICC profile is ignored: the image is treated as sRGB")]
    CoverProfileIgnored,
    /// A `user_private_copy` cover: the file is for the user's own printer only.
    #[error("private-copy cover: this file is to be printed on the user's own printer only")]
    OwnPrinterOnly,
    /// A note from the spine designer.
    #[error("spine: {0}")]
    Spine(SpineNote),
    /// A [`SpineChoice`](crate::SpineChoice) that needs a printable cover was asked for
    /// without one; the default typeset spine was used.
    #[error("the spine style {style} needs a printable cover: the default typeset spine was used")]
    SpineChoiceIgnored {
        /// [`SpineChoice::name`](crate::SpineChoice::name) of the style asked for.
        style: &'static str,
    },
    /// Characters that the font cannot draw were left out.
    #[error("the {field} contains characters the font cannot draw; left out: {characters}")]
    MissingGlyphs {
        /// The text concerned.
        field: TextField,
        /// The characters left out, e.g. `'語' (U+8A9E)`, at most twelve of them.
        characters: String,
    },
    /// A text did not fit its box and was cut, with an ellipsis.
    #[error("the {field} does not fit and was cut")]
    TextClamped {
        /// The text concerned.
        field: TextField,
    },
    /// The inside panel is tight: the steps are set smaller than the 12 pt the card is
    /// designed for.
    #[error("the steps are set at {size_pt:.1} pt, below the recommended 12 pt")]
    StepsReduced {
        /// The size used, in points.
        size_pt: f64,
    },
    /// The QR code's modules are smaller than the minimum for print.
    #[error("QR modules are {module_mm:.2} mm wide, below the {minimum_mm:.2} mm minimum for print")]
    QrModuleBelowMinimum {
        /// Module size, in millimetres.
        module_mm: f64,
        /// The minimum ([`ebc_qr::MIN_PRINT_MODULE_MM`]).
        minimum_mm: f64,
    },
    /// The QR code's modules are printable but smaller than recommended.
    #[error("QR modules are {module_mm:.2} mm wide, below the recommended {recommended_mm:.2} mm")]
    QrModuleSmall {
        /// Module size, in millimetres.
        module_mm: f64,
        /// The recommendation ([`ebc_qr::RECOMMENDED_PRINT_MODULE_MM`]).
        recommended_mm: f64,
    },
    /// The claim URL needs a dense QR code (version above 4): shorten the URL.
    #[error("the claim URL needs a version {version} QR code; a shorter URL scans more easily")]
    QrVersionHigh {
        /// The QR version (1–40).
        version: u8,
    },
    /// The claim URL is not of the form `<base>/k/<code>`, so it was encoded as given
    /// instead of fully upper-cased.
    #[error("the claim URL does not end in /k/<code>: it is encoded as given, not upper-cased")]
    QrPayloadNotUpperCased,
    /// The output intent is the bundled CC0 fallback profile, which is neither a printer
    /// profile nor a European condition.
    #[error("fallback output profile — supply FOGRA39/FOGRA51 for European printers")]
    FallbackOutputProfile,
    /// The output-intent profile supplied is not of ICC class `prtr` (output device).
    #[error("the output-intent profile is of ICC class '{class}', not a printer profile ('prtr')")]
    OutputProfileNotPrinterClass {
        /// The profile's device class signature.
        class: String,
    },
    /// The proofing overlay was added: the file is a proof, not a print file.
    #[error("the guides overlay is included: this file is a proof, not a print file")]
    GuidesIncluded,
}
