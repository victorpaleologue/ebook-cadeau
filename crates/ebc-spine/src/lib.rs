//! Spine design: what is printed on the narrow panel between the back and the front of
//! the gift card (10 mm wide by default).
//!
//! [`design`] returns the [`Item`]s of the spine in page millimetres (origin top-left of
//! the trim box, y down) for the rectangle given in the [`SpineRequest`]:
//!
//! * a **background** that covers the whole rectangle, bleed included;
//! * the **title** and, when there is room, the **author**, typeset from the book's
//!   metadata along the spine and drawn as filled outlines (no font is embedded, any
//!   font works, variable fonts included).
//!
//! # What is and is not taken from the cover
//!
//! By default the spine is a **flat colour** with text typeset from metadata. The colour
//! is *computed* from the cover (its binding edge when that is plain, its frame, or a
//! dominant colour — see [`BackgroundStyle::Auto`]); no pixel, texture or gradient of the
//! cover is reproduced on the spine. This is the option the legal research rates as safe
//! (`docs/research/07-legal-covers-and-reseller-rights.md`, implementation items 11–13).
//!
//! Three things reproduce or alter someone's artwork and therefore exist **only as
//! explicit opt-ins, to be used with the rights-holder's authorisation**:
//!
//! * [`BackgroundStyle::EdgeContinuation`] — a mirrored, blurred strip of the cover;
//! * [`SpineSource::Provided`] — a spine image, re-fitted to the panel;
//! * [`SpineSource::FullWrap`] — the spine columns of a flat wraparound cover.
//!
//! [`BackgroundStyle::Auto`] never chooses any of them, and a design that uses one says
//! so in [`SpineDesign::requires_authorisation`]. This crate cannot check the
//! authorisation; the caller must.
//!
//! # Direction
//!
//! The outside of the card, seen flat, is `back cover | spine | front cover`.
//!
//! ```text
//!           TopToBottom                           BottomToTop
//!     rotated 90° clockwise              rotated 90° counter-clockwise
//!   (ISO 6357; en, nl, da, sv, nb)          (fr, de, it, es, pt)
//!
//!    back  │   spine   │ front            back  │   spine   │ front
//!          │ ┃ F       │                        │       l ┃ │
//!          │ ┃ i  │    │                        │    ▲  a ┃ │
//!          │ ┃ r  │    │                        │    │  n ┃ │
//!          │ ┃ s  ▼    │                        │    │  i ┃ │
//!          │ ┃ t       │                        │       F ┃ │
//!            ▲                                            ▲
//!   baseline on the back-cover side,        baseline on the front-cover side,
//!   tops of the letters towards the         tops of the letters towards the
//!   front cover; the first letter is        back cover; the first letter is
//!   nearest the head (top)                  nearest the foot (bottom)
//! ```
//!
//! Lay the folded card flat, front cover up, spine towards you: a top-to-bottom title
//! reads upright, a bottom-to-top title upside down. The direction is the caller's
//! choice ([`ebc_i18n::spine_direction_for_book`]); this crate only implements it.
//!
//! # Typography
//!
//! * One line. The order along the reading direction is author, then title, as on French,
//!   German and Italian paperbacks; the pair is centred along the spine between the head
//!   and foot margins ([`SpineOptions::end_margin`]).
//! * Across the spine the text is centred **optically**: the box between the baseline and
//!   the capital height of the title font is centred on the spine's centre line (not the
//!   em box, not the ink box). The author shares the title's baseline.
//! * Every bit of ink stays inside the band between the two safe margins
//!   ([`SpineRequest::safe_margin`]) — measured on the real outlines, accents and
//!   descenders included.
//! * Fitting ladder, stopping at the first step that works: the whole title at the
//!   largest size between [`SpineOptions::min_size_pt`] and [`SpineOptions::max_size_pt`]
//!   (the author is dropped before the title goes below
//!   [`SpineOptions::min_title_size_with_author_pt`]); tightened letter-spacing; the
//!   title cut at its first subtitle separator; truncation at a word boundary with an
//!   ellipsis. Text is never invented, re-cased or stacked.
//! * Characters the font cannot draw are left out and reported (never a `.notdef` box);
//!   [`SpineOptions::fallback_font`] is tried first.
//!
//! Defaults come from the research notes `docs/research/notes/spine-design.*.md`, which
//! are **unverified** (recalled or computed, checked on synthetic images only).
//!
//! # Limits
//!
//! * Left-to-right books only: the binding edge is the left edge of the front cover.
//! * No two-line setting, no vertical CJK setting, no publisher mark at the foot.
//! * The contrast figures are WCAG 2 ratios on sRGB values, not measurements of ink on
//!   paper; see [`SpineNote::MidToneBackground`].
//!
//! # Example
//!
//! ```
//! # fn demo(font: &ebc_text::Font) -> Result<(), ebc_spine::SpineError> {
//! use ebc_i18n::SpineDirection;
//! use ebc_model::BookMeta;
//! use ebc_scene::{Document, Rect};
//! use ebc_spine::{SpineRequest, SpineSource, design};
//!
//! let book = BookMeta::titled("Candide");
//! let mut doc = Document::default();
//! let spine = design(
//!     &SpineRequest {
//!         book: &book,
//!         cover: None,
//!         source: SpineSource::Synthesize,
//!         // 10 mm spine of a 178 mm high card, with 3 mm of bleed at top and bottom.
//!         rect: Rect::new(110.0, -3.0, 10.0, 184.0),
//!         bleed: 3.0,
//!         title_font: font,
//!         author_font: font,
//!         direction: SpineDirection::BottomToTop,
//!         safe_margin: 1.5,
//!     },
//!     &mut doc,
//! )?;
//! assert!(!spine.requires_authorisation);
//! # Ok(()) }
//! ```

#![forbid(unsafe_code)]

mod adapt;
mod background;
mod error;
mod geometry;
mod raster;
mod typeset;

use std::fmt;
use std::sync::Arc;

use ebc_color::{Rgb8, contrast_ratio, ensure_contrast, to_cmyk_naive, to_scene_rgb};
use ebc_i18n::SpineDirection;
use ebc_model::BookMeta;
use ebc_scene::{Color, Document, ImageData, Item, PixelFormat, Pixmap, Rect};
use ebc_text::Font;

pub use error::SpineError;

use crate::geometry::Geometry;
use crate::raster::Raster;
use crate::typeset::{SetRun, TitleStep};

/// Where the content of the spine comes from.
#[derive(Debug, Clone, Copy)]
pub enum SpineSource<'a> {
    /// Typeset the spine from the book's metadata on a ground chosen per
    /// [`SpineOptions::background`]. The default, and the only source that needs no
    /// authorisation when the ground is flat.
    Synthesize,
    /// **Opt-in — requires the rights-holder's authorisation.** An image of the real
    /// spine (upright, head at the top, trimmed: no bleed), re-fitted to the panel when
    /// it is compatible (see [`Adaptation`]); otherwise a spine is typeset and a note
    /// says why ([`Incompatibility`]).
    Provided(&'a Pixmap),
    /// **Opt-in — requires the rights-holder's authorisation.** A flat wraparound cover
    /// (back | spine | front, trimmed) and the pixel columns `start..end` of its spine;
    /// those columns are then treated like [`SpineSource::Provided`].
    FullWrap {
        /// The wraparound image.
        image: &'a Pixmap,
        /// The spine's columns as `(start, end)`, `end` exclusive.
        spine_px: (u32, u32),
    },
}

/// Everything [`design`] needs to know about one spine.
#[derive(Debug, Clone, Copy)]
pub struct SpineRequest<'a> {
    /// The book: title, subtitle, authors and language are read.
    pub book: &'a BookMeta,
    /// The front cover, used only to *compute* colours (unless
    /// [`BackgroundStyle::EdgeContinuation`] is asked for). It is assumed to fill the
    /// height of the front panel.
    pub cover: Option<&'a Pixmap>,
    /// Typeset a spine, or adapt a provided one.
    pub source: SpineSource<'a>,
    /// The spine panel in page millimetres, **including** the bleed at the top and at
    /// the bottom (and none at the sides: its width is the spine width).
    pub rect: Rect,
    /// The bleed included at the top and at the bottom of `rect`, in millimetres.
    pub bleed: f64,
    /// Font of the title.
    pub title_font: &'a Font,
    /// Font of the author's name.
    pub author_font: &'a Font,
    /// Which way the text reads (see the crate documentation).
    pub direction: SpineDirection,
    /// Distance kept clear of ink along each crease, in millimetres
    /// (`ebc_layout`'s `spine_safe_margin`).
    pub safe_margin: f64,
}

/// How the background of a typeset spine is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundStyle {
    /// A flat colour computed from the cover, never an image:
    ///
    /// 1. the strip along the binding edge (left 4 % of the cover) is near-uniform
    ///    ([`ebc_color::edge_uniformity`] ≥ [`ebc_color::UNIFORM_EDGE_THRESHOLD`]) → its
    ///    colour, so the spine continues the cover ([`BackgroundKind::EdgeColour`]);
    /// 2. the cover has a uniform frame ([`ebc_color::has_uniform_border`]) → the
    ///    frame's colour ([`BackgroundKind::BorderColour`]);
    /// 3. otherwise → among the cover's five largest palette swatches covering at least
    ///    10 % of it, the one nearest the mean colour of the binding edge
    ///    ([`BackgroundKind::DominantColour`]);
    /// 4. no cover → white ([`BackgroundKind::Neutral`]).
    ///
    /// When a provided spine turned out to be incompatible and has a plain ground, that
    /// colour is used instead ([`BackgroundKind::ProvidedSpineColour`]).
    #[default]
    Auto,
    /// This colour.
    Solid(Rgb8),
    /// The largest swatch of the cover's palette, whatever the binding edge looks like.
    DominantColour,
    /// **Opt-in — requires the rights-holder's authorisation.** An image: the strip of
    /// the cover next to the binding edge, as wide as the spine, mirrored about the
    /// fold, strongly blurred and pulled towards its mean colour, registered in the
    /// document at [`SpineOptions::image_ppi`]. If the text does not reach
    /// [`SpineOptions::min_contrast`] against the darkest or lightest part of the image,
    /// a flat band is laid behind it. Falls back to [`BackgroundStyle::Auto`] without a
    /// cover.
    EdgeContinuation,
}

/// The colour model of the flat background fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundPaint {
    /// sRGB, like the cover image next to it, so that both go through the same colour
    /// conversion at the printer and meet without a visible step at the fold.
    #[default]
    Rgb,
    /// Device CMYK through [`ebc_color::to_cmyk_naive`] (not colour-managed): needed for
    /// PDF/X-1a, which forbids RGB. The ground may then differ visibly from an RGB cover.
    Cmyk,
}

/// Tuning of [`design_with`]. [`Default`] gives the documented defaults.
#[derive(Debug, Clone)]
pub struct SpineOptions {
    /// Background of a typeset spine. Default [`BackgroundStyle::Auto`].
    pub background: BackgroundStyle,
    /// Colour model of flat background fills. Default [`BackgroundPaint::Rgb`].
    pub background_paint: BackgroundPaint,
    /// A preferred text colour (the cover title's colour, a user's choice). Used when it
    /// reaches `min_contrast` on the background; otherwise ignored with a note.
    /// Default `None`: white or black.
    pub text_color: Option<Rgb8>,
    /// Minimum WCAG 2 contrast ratio between text and background. Default 4.5.
    pub min_contrast: f64,
    /// Smallest title size, in points. Default 7.
    pub min_size_pt: f64,
    /// Largest title size, in points (the band between the safe margins usually limits
    /// the size first). Default 14.
    pub max_size_pt: f64,
    /// Letter-spacing that may be removed to make a long title fit, in em. Default 0.02.
    pub max_tightening_em: f32,
    /// Margin kept clear of text at the head and at the foot, from the trim edge, in
    /// millimetres. Default 8.
    pub end_margin: f64,
    /// Set the author's name when there is room. Default `true`.
    pub show_author: bool,
    /// Author size as a fraction of the title size. Default 0.7.
    pub author_scale: f64,
    /// Smallest author size, in points; below it the author is left out. Default 6.
    pub author_min_size_pt: f64,
    /// Space between author and title, in millimetres. Default 6.
    pub author_gap: f64,
    /// Largest share of the usable length the author may take. Default 0.35.
    pub author_max_share: f64,
    /// The title may be made smaller to leave room for the author, but not below this
    /// size in points: then the author is left out instead. Default 9.
    pub min_title_size_with_author_pt: f64,
    /// A font tried when the title or author font lacks a character. Default `None`.
    pub fallback_font: Option<Font>,
    /// Resolution of a generated background image
    /// ([`BackgroundStyle::EdgeContinuation`]), in pixels per inch, clamped to 72–600.
    /// Default 300.
    pub image_ppi: f64,
    /// A provided spine with fewer pixels per inch along the spine is not reused.
    /// Default 150.
    pub min_provided_ppi: f64,
}

impl Default for SpineOptions {
    fn default() -> Self {
        Self {
            background: BackgroundStyle::Auto,
            background_paint: BackgroundPaint::Rgb,
            text_color: None,
            min_contrast: 4.5,
            min_size_pt: 7.0,
            max_size_pt: 14.0,
            max_tightening_em: 0.02,
            end_margin: 8.0,
            show_author: true,
            author_scale: 0.7,
            author_min_size_pt: 6.0,
            author_gap: 6.0,
            author_max_share: 0.35,
            min_title_size_with_author_pt: 9.0,
            fallback_font: None,
            image_ppi: 300.0,
            min_provided_ppi: 150.0,
        }
    }
}

impl SpineOptions {
    fn validate(&self) -> Result<(), SpineError> {
        let bad = |what: &str| Err(SpineError::InvalidOptions(what.to_owned()));
        let positive = |v: f64| v.is_finite() && v > 0.0;
        let non_negative = |v: f64| v.is_finite() && v >= 0.0;
        if !(positive(self.min_size_pt) && positive(self.max_size_pt)) {
            return bad("the title sizes must be positive, finite numbers");
        }
        if self.min_size_pt > self.max_size_pt {
            return bad("the minimum title size is above the maximum");
        }
        if self.max_size_pt > ebc_text::MAX_SIZE_PT {
            return bad("the maximum title size is larger than any printable type");
        }
        if !(self.max_tightening_em.is_finite() && (0.0..=0.2).contains(&self.max_tightening_em)) {
            return bad("the letter-spacing reduction must be between 0 and 0.2 em");
        }
        if !non_negative(self.end_margin) || !non_negative(self.author_gap) {
            return bad("margins and gaps must be finite and not negative");
        }
        if !(positive(self.author_scale) && self.author_scale <= 1.0) {
            return bad("the author scale must be in 0–1");
        }
        if !positive(self.author_min_size_pt) || !positive(self.min_title_size_with_author_pt) {
            return bad("the author size limits must be positive, finite numbers");
        }
        if !(self.author_max_share.is_finite() && (0.0..=1.0).contains(&self.author_max_share)) {
            return bad("the author's share of the length must be in 0–1");
        }
        if !(self.min_contrast.is_finite() && (1.0..=21.0).contains(&self.min_contrast)) {
            return bad("the minimum contrast must be in 1–21");
        }
        if !positive(self.image_ppi) || !non_negative(self.min_provided_ppi) {
            return bad("resolutions must be finite and positive");
        }
        Ok(())
    }
}

/// What the background of a [`SpineDesign`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundKind {
    /// Flat: the colour of the cover's plain binding edge.
    EdgeColour,
    /// Flat: the colour of the cover's uniform frame.
    BorderColour,
    /// Flat: a dominant colour of the cover's palette.
    DominantColour,
    /// Flat: the colour given in [`BackgroundStyle::Solid`].
    Custom,
    /// Flat white: there was no cover to compute a colour from.
    Neutral,
    /// Flat: the plain ground colour of a provided spine that could not be reused.
    ProvidedSpineColour,
    /// An image generated from the cover ([`BackgroundStyle::EdgeContinuation`]).
    EdgeContinuation,
    /// The provided spine image itself.
    ProvidedSpine,
}

impl BackgroundKind {
    /// `true` when the background is one flat colour (no pixels of any artwork).
    #[must_use]
    pub const fn is_flat(self) -> bool {
        !matches!(self, Self::EdgeContinuation | Self::ProvidedSpine)
    }
}

/// How a provided spine was fitted to the panel. `w'` is its width after uniform scaling
/// to the height of the spine, `W` the width of the spine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Adaptation {
    /// 0.92 `W` ≤ `w'` ≤ 1.08 `W`: scaled across the spine only, by `percent`
    /// (negative = squeezed), at most 8 % either way.
    Stretched {
        /// Change of width in per cent.
        percent: f64,
    },
    /// 0.5 `W` ≤ `w'` < 0.92 `W` and plain long sides: scaled uniformly, centred, the
    /// sides filled with the image's side colours.
    Padded {
        /// Total width added, in millimetres.
        added_mm: f64,
    },
    /// `w'` > 1.08 `W`, plain long sides and content no wider than the band between the
    /// safe margins: scaled uniformly and cropped to `W`, centred on the content.
    Cropped {
        /// Total width removed, in millimetres.
        removed_mm: f64,
    },
    /// 1.08 `W` < `w'` ≤ 1.3 `W` and not croppable: scaled down uniformly to the width;
    /// the head and foot are filled with the image's end colours.
    Shrunk {
        /// Reduction in per cent.
        percent: f64,
    },
}

/// Why a provided spine was not reused (a spine was typeset instead).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Incompatibility {
    /// Too few pixels along the spine for print.
    LowResolution {
        /// Pixels per inch at final size.
        ppi: f64,
    },
    /// No text was found on it (heuristic), so there is nothing worth keeping.
    NoText,
    /// Scaled to the height of the spine it is less than half as wide as the spine.
    TooNarrow {
        /// Its width after scaling, in millimetres.
        width_mm: f64,
    },
    /// It is narrower than the spine and its long sides are not plain, so it cannot be
    /// padded.
    BusySides {
        /// Its width after scaling, in millimetres.
        width_mm: f64,
    },
    /// It is more than 1.3 times as wide as the spine and cannot be cropped (its sides
    /// are not plain background, or its content is wider than the band between the safe
    /// margins — typical of thick books with the title set across the spine).
    TooWide {
        /// Its width after scaling, in millimetres.
        width_mm: f64,
    },
    /// Its text would be illegibly small at final size.
    TextTooSmall {
        /// Estimated extent of the text across the spine at final size, in millimetres.
        height_mm: f64,
    },
}

impl fmt::Display for Incompatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::LowResolution { ppi } => {
                write!(f, "its resolution would be {ppi:.0} ppi, too low for print")
            }
            Self::NoText => write!(f, "no text was found on it"),
            Self::TooNarrow { width_mm } => write!(
                f,
                "at the height of the card it is only {width_mm:.1} mm wide, much narrower \
                 than the spine"
            ),
            Self::BusySides { width_mm } => write!(
                f,
                "at the height of the card it is {width_mm:.1} mm wide and its sides are not \
                 a plain colour that could be extended"
            ),
            Self::TooWide { width_mm } => write!(
                f,
                "at the height of the card it is {width_mm:.1} mm wide and cannot be cropped \
                 to the spine without cutting into its content"
            ),
            Self::TextTooSmall { height_mm } => {
                write!(f, "its text would be only {height_mm:.1} mm high, too small to read")
            }
        }
    }
}

/// Which line of text a note is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    /// The title.
    Title,
    /// The author's name.
    Author,
}

impl fmt::Display for TextRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Title => "title",
            Self::Author => "author's name",
        })
    }
}

/// Something the user should know about a design: a degradation, a fallback, or a use
/// of artwork that needs authorisation. [`SpineDesign::notes`] holds the same notes as
/// sentences.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SpineNote {
    /// The design reproduces pixels of the cover or of a provided spine, altered
    /// (blurred, stretched, cropped, padded or scaled): the rights-holder's authorisation
    /// is required.
    AuthorisationRequired,
    /// A provided spine was re-fitted to the panel.
    ProvidedSpineAdapted(Adaptation),
    /// A provided spine could not be reused; a spine was typeset instead.
    ProvidedSpineIncompatible(Incompatibility),
    /// No cover was given: the background is plain white.
    NoCover,
    /// An edge continuation was asked for without a cover; a flat colour is used.
    ContinuationWithoutCover,
    /// A flat band was laid behind the text because the background image is too light
    /// or too dark in places.
    ContrastBand,
    /// The preferred text colour was ignored: it only reaches this contrast ratio.
    TextColourRejected {
        /// The ratio it would have had.
        ratio: f64,
    },
    /// The text only reaches this contrast ratio, below the minimum asked for.
    LowContrast {
        /// The ratio reached.
        ratio: f64,
    },
    /// The background is a mid-tone (CIELAB L\* 46–59): on paper neither black nor white
    /// text may reach 4.5:1 there. An estimate from the research note, not a measurement.
    MidToneBackground,
    /// Light text on a darker ground below 8 pt: thin strokes may fill in on press.
    SmallReversedText {
        /// The size of the smallest such text, in points.
        size_pt: f64,
    },
    /// The title was longer than the 2 000 characters considered.
    TitleTooLong,
    /// Letter-spacing was tightened to make the title fit.
    TitleTightened,
    /// The title was cut at its first subtitle separator.
    TitleShortened {
        /// The text set.
        set: String,
    },
    /// The title was truncated, with an ellipsis.
    TitleTruncated {
        /// The text set.
        set: String,
    },
    /// The title was made smaller than it could have been, to leave room for the author.
    TitleReducedForAuthor,
    /// The author's name was left out: no room beside the title.
    AuthorDropped,
    /// Only the first author is named.
    AuthorShortened {
        /// The text set.
        set: String,
    },
    /// Some characters could not be drawn by any font supplied and were left out.
    MissingGlyphs {
        /// The line concerned.
        role: TextRole,
        /// The characters left out, e.g. `'Ω' (U+03A9)`.
        characters: String,
    },
    /// The fallback font was used because the intended font lacks some characters.
    FallbackFont {
        /// The line concerned.
        role: TextRole,
        /// The characters the intended font lacks.
        characters: String,
    },
    /// Nothing of the title can be drawn with the fonts supplied: the spine has no text.
    TitleNotDrawable,
    /// The spine is too narrow or too short for text at the minimum size: it has none.
    NoRoomForText,
}

impl fmt::Display for SpineNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorisationRequired => write!(
                f,
                "The spine reproduces altered pixels of the cover or of the spine image \
                 supplied: this requires the rights-holder's authorisation."
            ),
            Self::ProvidedSpineAdapted(how) => match *how {
                Adaptation::Stretched { percent } => write!(
                    f,
                    "The spine supplied was scaled across its width by {percent:+.1} % to fit."
                ),
                Adaptation::Padded { added_mm } => write!(
                    f,
                    "The spine supplied is narrower than the card's: {added_mm:.1} mm of its \
                     side colours were added."
                ),
                Adaptation::Cropped { removed_mm } => write!(
                    f,
                    "The spine supplied is wider than the card's: {removed_mm:.1} mm of plain \
                     background were cropped from its sides."
                ),
                Adaptation::Shrunk { percent } => write!(
                    f,
                    "The spine supplied is wider than the card's: it was reduced by \
                     {percent:.1} % and its head and foot were filled with its end colours."
                ),
            },
            Self::ProvidedSpineIncompatible(why) => write!(
                f,
                "The spine supplied was not used ({why}); the spine was typeset instead."
            ),
            Self::NoCover => write!(f, "No cover was given: the spine is plain white."),
            Self::ContinuationWithoutCover => write!(
                f,
                "The cover cannot be continued onto the spine without a cover image: a flat \
                 colour is used."
            ),
            Self::ContrastBand => write!(
                f,
                "A flat band was laid behind the text to keep it readable on the background \
                 image."
            ),
            Self::TextColourRejected { ratio } => write!(
                f,
                "The text colour asked for was not used: its contrast on this background \
                 would be {ratio:.1}:1."
            ),
            Self::LowContrast { ratio } => write!(
                f,
                "The contrast between text and background is only {ratio:.1}:1, below the \
                 minimum asked for."
            ),
            Self::MidToneBackground => write!(
                f,
                "The background is a mid-tone: once printed, the text may be less readable \
                 than on screen (estimate, not measured)."
            ),
            Self::SmallReversedText { size_pt } => write!(
                f,
                "Light text on a dark ground at {size_pt:.1} pt: thin strokes may fill in when \
                 printed."
            ),
            Self::TitleTooLong => {
                write!(f, "The title is extremely long: only its beginning was considered.")
            }
            Self::TitleTightened => {
                write!(f, "The letter-spacing of the title was tightened to make it fit.")
            }
            Self::TitleShortened { set } => {
                write!(f, "The title does not fit: only its main part is set (\"{set}\").")
            }
            Self::TitleTruncated { set } => {
                write!(f, "The title does not fit: it was truncated (\"{set}\").")
            }
            Self::TitleReducedForAuthor => {
                write!(f, "The title was set smaller to leave room for the author's name.")
            }
            Self::AuthorDropped => {
                write!(f, "The author's name was left out: there is no room beside the title.")
            }
            Self::AuthorShortened { set } => {
                write!(f, "Only the first author is named on the spine (\"{set}\").")
            }
            Self::MissingGlyphs { role, characters } => write!(
                f,
                "The {role} contains characters that no font supplied can draw; they were \
                 left out: {characters}."
            ),
            Self::FallbackFont { role, characters } => write!(
                f,
                "The {role} is set in the fallback font: the font chosen lacks {characters}."
            ),
            Self::TitleNotDrawable => write!(
                f,
                "The title cannot be drawn with the fonts supplied: the spine has no text."
            ),
            Self::NoRoomForText => {
                write!(f, "The spine is too narrow or too short for text: it has none.")
            }
        }
    }
}

/// A line of text as it was set.
#[derive(Debug, Clone, PartialEq)]
pub struct TextPlacement {
    /// The text actually set (after cleaning, shortening or truncation).
    pub text: String,
    /// Its size in points.
    pub size_pt: f64,
    /// The exact bounding box of its ink, in page millimetres.
    pub bounds: Rect,
    /// Index of its outline in [`SpineDesign::items`].
    pub item: usize,
}

/// The result of [`design`].
#[derive(Debug, Clone, PartialEq)]
pub struct SpineDesign {
    /// The items to paint, in order, in page millimetres. They lie inside
    /// [`SpineRequest::rect`]; the background covers it entirely.
    pub items: Vec<Item>,
    /// The background colour (for an image background: its mean colour, or the plain
    /// ground of an adapted spine). Reuse it for the back panel.
    pub background: Rgb8,
    /// The colour of the text (for an adapted spine, which carries its own text: black
    /// or white, whichever contrasts more with `background`).
    pub text_color: Rgb8,
    /// The size of the title in points; 0 when no title was typeset.
    pub title_size_pt: f64,
    /// Human-readable notes for the user: `details`, as English sentences, in the same
    /// order.
    pub notes: Vec<String>,
    /// The same notes, typed.
    pub details: Vec<SpineNote>,
    /// What the background is.
    pub background_kind: BackgroundKind,
    /// WCAG 2 contrast ratio between `text_color` and the background behind the text
    /// (the worst case for an image background). 0 when no text was typeset.
    pub contrast: f64,
    /// The title as set, if any.
    pub title: Option<TextPlacement>,
    /// The author's name as set, if any.
    pub author: Option<TextPlacement>,
    /// `true` when the design reproduces pixels of the cover or of a provided spine
    /// ([`BackgroundKind::EdgeContinuation`], [`BackgroundKind::ProvidedSpine`]): only
    /// to be printed with the rights-holder's authorisation. Always `false` for the
    /// default request (`SpineSource::Synthesize` with `BackgroundStyle::Auto`).
    pub requires_authorisation: bool,
}

/// Designs a spine with the default [`SpineOptions`].
///
/// # Errors
/// See [`design_with`].
pub fn design(req: &SpineRequest<'_>, doc: &mut Document) -> Result<SpineDesign, SpineError> {
    design_with(req, &SpineOptions::default(), doc)
}

/// Designs a spine.
///
/// Images, if any (only for the opt-in sources and background), are registered in
/// `doc`; nothing else of `doc` is touched. The same request always gives the same
/// result.
///
/// # Errors
/// * [`SpineError::InvalidGeometry`] / [`SpineError::InvalidOptions`] for unusable
///   numbers;
/// * [`SpineError::InvalidSpineRange`] for a [`SpineSource::FullWrap`] range that is
///   empty or outside the image;
/// * [`SpineError::EmptyTitle`] when a spine has to be typeset and the book has no title.
///
/// Everything else (long titles, missing glyphs, incompatible images, a spine too narrow
/// for text) yields a usable design with [`SpineDesign::notes`].
pub fn design_with(
    req: &SpineRequest<'_>,
    opts: &SpineOptions,
    doc: &mut Document,
) -> Result<SpineDesign, SpineError> {
    opts.validate()?;
    let geom = Geometry::new(req, opts)?;
    let mut notes: Vec<SpineNote> = Vec::new();

    let provided = match req.source {
        SpineSource::Synthesize => None,
        SpineSource::Provided(image) => Some((image, (0, image.width()))),
        SpineSource::FullWrap { image, spine_px: (start, end) } => {
            if start >= end || end > image.width() {
                return Err(SpineError::InvalidSpineRange { start, end, width: image.width() });
            }
            Some((image, (start, end)))
        }
    };

    let mut spine_colour = None;
    if let Some((image, columns)) = provided {
        match adapt::adapt(image, columns, &geom, req.safe_margin, opts.min_provided_ppi) {
            adapt::Outcome::Adapted(adapted) => {
                notes.push(SpineNote::ProvidedSpineAdapted(adapted.how));
                notes.push(SpineNote::AuthorisationRequired);
                return Ok(adapted_design(adapted, opts, doc, notes));
            }
            adapt::Outcome::Incompatible { reason, colour } => {
                notes.push(SpineNote::ProvidedSpineIncompatible(reason));
                spine_colour = colour;
            }
        }
    }
    synthesise(req, opts, &geom, doc, notes, spine_colour)
}

/// The rectangle, in page millimetres, in which every bit of ink of the typeset text
/// lies: across the spine the band between the two safe margins, along it the stretch
/// between the head and foot margins ([`SpineOptions::end_margin`] from the trim edges).
///
/// # Errors
/// As [`design_with`] for unusable geometry or options.
pub fn text_area(req: &SpineRequest<'_>, opts: &SpineOptions) -> Result<Rect, SpineError> {
    opts.validate()?;
    Ok(Geometry::new(req, opts)?.text_area())
}

/// Registers an RGB raster in the document.
fn register(doc: &mut Document, raster: Raster) -> ebc_scene::ImageId {
    let (width_px, height_px) = (raster.w as u32, raster.h as u32);
    let pixels: Arc<[u8]> = raster.data.into();
    doc.add_image(ImageData::Raw { pixels, width_px, height_px, format: PixelFormat::Rgb8 }, None)
}

fn flat_paint(colour: Rgb8, paint: BackgroundPaint) -> Color {
    match paint {
        BackgroundPaint::Rgb => to_scene_rgb(colour),
        BackgroundPaint::Cmyk => to_cmyk_naive(colour),
    }
}

/// Text is K-only black, paper white, or a CMYK colour — never RGB.
fn text_paint(colour: Rgb8) -> Color {
    if colour == Rgb8::BLACK {
        Color::BLACK
    } else if colour == Rgb8::WHITE {
        Color::WHITE
    } else {
        to_cmyk_naive(colour)
    }
}

fn finish(
    items: Vec<Item>,
    background: Rgb8,
    background_kind: BackgroundKind,
    text_color: Rgb8,
    details: Vec<SpineNote>,
) -> SpineDesign {
    SpineDesign {
        items,
        background,
        text_color,
        title_size_pt: 0.0,
        notes: details.iter().map(ToString::to_string).collect(),
        details,
        background_kind,
        contrast: 0.0,
        title: None,
        author: None,
        requires_authorisation: !background_kind.is_flat(),
    }
}

fn adapted_design(
    adapted: adapt::Adapted,
    opts: &SpineOptions,
    doc: &mut Document,
    notes: Vec<SpineNote>,
) -> SpineDesign {
    let mut items: Vec<Item> = adapted
        .fills
        .iter()
        .map(|fill| Item::filled_rect(fill.rect, flat_paint(fill.colour, opts.background_paint)))
        .collect();
    let image = register(doc, adapted.raster);
    items.push(Item::Image { image, rect: adapted.image_rect });
    let text = ebc_color::black_or_white(adapted.background);
    finish(items, adapted.background, BackgroundKind::ProvidedSpine, text, notes)
}

/// The ground of a typeset spine.
enum Ground {
    Flat(Rgb8, BackgroundKind),
    Image(background::Continuation),
}

fn choose_ground(
    req: &SpineRequest<'_>,
    opts: &SpineOptions,
    geom: &Geometry,
    spine_colour: Option<Rgb8>,
    notes: &mut Vec<SpineNote>,
) -> Ground {
    let auto = |notes: &mut Vec<SpineNote>| match (spine_colour, req.cover) {
        (Some(colour), _) => Ground::Flat(colour, BackgroundKind::ProvidedSpineColour),
        (None, Some(cover)) => {
            let (colour, kind) = background::auto_flat(cover);
            Ground::Flat(colour, kind)
        }
        (None, None) => {
            notes.push(SpineNote::NoCover);
            Ground::Flat(Rgb8::WHITE, BackgroundKind::Neutral)
        }
    };
    match (opts.background, req.cover) {
        (BackgroundStyle::Solid(colour), _) => Ground::Flat(colour, BackgroundKind::Custom),
        (BackgroundStyle::DominantColour, Some(cover)) => {
            Ground::Flat(background::dominant(cover), BackgroundKind::DominantColour)
        }
        (BackgroundStyle::EdgeContinuation, Some(cover)) => {
            Ground::Image(background::continuation(cover, geom, opts.image_ppi.clamp(72.0, 600.0)))
        }
        (BackgroundStyle::EdgeContinuation, None) => {
            notes.push(SpineNote::ContinuationWithoutCover);
            auto(notes)
        }
        (BackgroundStyle::Auto | BackgroundStyle::DominantColour, _) => auto(notes),
    }
}

/// The text colour on a flat ground and its contrast ratio.
fn text_on_flat(ground: Rgb8, opts: &SpineOptions, notes: &mut Vec<SpineNote>) -> (Rgb8, f64) {
    if let Some(wanted) = opts.text_color {
        let ratio = contrast_ratio(wanted, ground);
        if ratio >= opts.min_contrast {
            return (wanted, ratio);
        }
        notes.push(SpineNote::TextColourRejected { ratio });
    }
    let colour = background::auto_text_colour(ground, opts.min_contrast);
    (colour, contrast_ratio(colour, ground))
}

/// The lines of text, fitted but not yet placed.
struct Lines {
    title: SetRun,
    author: Option<SetRun>,
}

/// Longest title considered, in characters; what follows is ignored (and the ladder then
/// truncates with an ellipsis anyway). Bounds the cost of hostile input.
const MAX_TITLE_CHARS: usize = 2_000;
/// Longest author line considered, in characters.
const MAX_AUTHOR_CHARS: usize = 500;
/// Light-on-dark text below this size gets a [`SpineNote::SmallReversedText`].
const MIN_REVERSED_PT: f64 = 8.0;

fn missing_notes(resolved: &typeset::Resolved, role: TextRole, notes: &mut Vec<SpineNote>) {
    if resolved.fallback_used {
        notes.push(SpineNote::FallbackFont {
            role,
            characters: typeset::describe_chars(&resolved.primary_missing),
        });
    }
    if !resolved.missing.is_empty() {
        notes.push(SpineNote::MissingGlyphs {
            role,
            characters: typeset::describe_chars(&resolved.missing),
        });
    }
}

fn fit_lines(
    req: &SpineRequest<'_>,
    opts: &SpineOptions,
    geom: &Geometry,
    title: &str,
    notes: &mut Vec<SpineNote>,
) -> Option<Lines> {
    let language = req.book.language.as_deref();
    let fallback = opts.fallback_font.as_ref();

    let resolved = typeset::resolve(title, req.title_font, fallback);
    missing_notes(&resolved, TextRole::Title, notes);
    if resolved.text.is_empty() {
        notes.push(SpineNote::TitleNotDrawable);
        return None;
    }
    let Some(fit) = typeset::fit_title(&resolved.font, &resolved.text, language, geom, opts)
    else {
        notes.push(SpineNote::NoRoomForText);
        return None;
    };
    match fit.step {
        TitleStep::Whole => {}
        TitleStep::Tightened => notes.push(SpineNote::TitleTightened),
        TitleStep::MainPart => notes.push(SpineNote::TitleShortened { set: fit.run.text.clone() }),
        TitleStep::Truncated | TitleStep::TruncatedInWord => {
            notes.push(SpineNote::TitleTruncated { set: fit.run.text.clone() });
        }
    }
    let mut title = fit.run;

    let candidates = if opts.show_author { typeset::author_candidates(req.book) } else { vec![] };
    let mut author: Option<SetRun> = None;
    let mut author_notes: Vec<SpineNote> = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let candidate: String = candidate.chars().take(MAX_AUTHOR_CHARS).collect();
        let resolved = typeset::resolve(&candidate, req.author_font, fallback);
        if resolved.text.is_empty() {
            continue;
        }
        let mut attempt =
            typeset::fit_author(&resolved.font, &resolved.text, language, &title, geom, opts);
        let mut reduced = None;
        if attempt.is_none() && fit.step == TitleStep::Whole {
            // Make the title smaller, down to the floor, so that both fit:
            // title(s) + gap + author(scale · s) ≤ usable length.
            let shape = typeset::shape_options(0.0, language);
            if let Some(m) = typeset::measure(&resolved.font, &resolved.text, &shape) {
                let per_pt = title.measure.length() + opts.author_scale * m.length();
                let joint = (geom.live_len() - opts.author_gap) / per_pt * (1.0 - 1.0e-9);
                if per_pt > 0.0
                    && joint < title.size_pt
                    && joint >= opts.min_title_size_with_author_pt.max(opts.min_size_pt)
                {
                    let smaller = title.with_size(joint);
                    attempt = typeset::fit_author(
                        &resolved.font,
                        &resolved.text,
                        language,
                        &smaller,
                        geom,
                        opts,
                    );
                    if attempt.is_some() {
                        reduced = Some(smaller);
                    }
                }
            }
        }
        if let Some(run) = attempt {
            missing_notes(&resolved, TextRole::Author, &mut author_notes);
            if index > 0 {
                author_notes.push(SpineNote::AuthorShortened { set: run.text.clone() });
            }
            if let Some(smaller) = reduced {
                title = smaller;
                author_notes.push(SpineNote::TitleReducedForAuthor);
            }
            author = Some(run);
            break;
        }
    }
    if author.is_none() && !candidates.is_empty() {
        author_notes.push(SpineNote::AuthorDropped);
    }
    notes.append(&mut author_notes);
    Some(Lines { title, author })
}

fn synthesise(
    req: &SpineRequest<'_>,
    opts: &SpineOptions,
    geom: &Geometry,
    doc: &mut Document,
    mut notes: Vec<SpineNote>,
    spine_colour: Option<Rgb8>,
) -> Result<SpineDesign, SpineError> {
    let full_title = typeset::title_of(req.book);
    if full_title.is_empty() {
        return Err(SpineError::EmptyTitle);
    }
    let title: String = full_title.chars().take(MAX_TITLE_CHARS).collect();
    if title.len() < full_title.len() {
        notes.push(SpineNote::TitleTooLong);
    }
    let title = typeset::normalise(&title);

    // Background.
    let ground = choose_ground(req, opts, geom, spine_colour, &mut notes);
    let mut items: Vec<Item> = Vec::new();
    let (background, kind, mut text_colour, mut contrast, extremes) = match ground {
        Ground::Flat(colour, kind) => {
            items.push(Item::filled_rect(geom.rect, flat_paint(colour, opts.background_paint)));
            let (text, ratio) = text_on_flat(colour, opts, &mut notes);
            (colour, kind, text, ratio, None)
        }
        Ground::Image(image) => {
            let (mean, darkest, lightest) = (image.mean, image.darkest, image.lightest);
            let id = register(doc, image.raster);
            items.push(Item::Image { image: id, rect: geom.rect });
            let (text, _) = text_on_flat(mean, opts, &mut notes);
            let worst = contrast_ratio(text, darkest).min(contrast_ratio(text, lightest));
            (mean, BackgroundKind::EdgeContinuation, text, worst, Some((darkest, lightest)))
        }
    };
    if !kind.is_flat() {
        notes.push(SpineNote::AuthorisationRequired);
    }

    // Text.
    let Some(lines) = fit_lines(req, opts, geom, &title, &mut notes) else {
        return Ok(finish(items, background, kind, text_colour, notes));
    };
    let author_len = lines.author.as_ref().map_or(0.0, |a| a.length() + opts.author_gap);
    let start = ((geom.live_len() - author_len - lines.title.length()) / 2.0).max(0.0);
    let baseline = typeset::baseline_x(&lines.title, req.direction, geom);
    let placed_author = lines
        .author
        .as_ref()
        .and_then(|run| typeset::place(run, start, baseline, req.direction, geom));
    let Some((title_path, title_bounds)) =
        typeset::place(&lines.title, start + author_len, baseline, req.direction, geom)
    else {
        notes.push(SpineNote::TitleNotDrawable);
        return Ok(finish(items, background, kind, text_colour, notes));
    };

    // An image ground that is too light or too dark somewhere gets a flat band behind
    // the text, in the image's mean colour moved just far enough for the text to read.
    if extremes.is_some() && contrast < opts.min_contrast {
        let band_colour = ensure_contrast(background, text_colour, opts.min_contrast);
        let mut ink = title_bounds;
        if let Some((_, bounds)) = &placed_author {
            ink = ink.union(bounds);
        }
        let top = (ink.y - 3.0).max(geom.rect.y);
        let bottom = (ink.bottom() + 3.0).min(geom.rect.bottom());
        let band = Rect::new(geom.rect.x, top, geom.rect.w, bottom - top);
        items.push(Item::filled_rect(band, flat_paint(band_colour, opts.background_paint)));
        notes.push(SpineNote::ContrastBand);
        contrast = contrast_ratio(text_colour, band_colour);
        if contrast < opts.min_contrast {
            // `ensure_contrast` gave up on the hue: take plain black or white text.
            text_colour = ebc_color::black_or_white(band_colour);
            contrast = contrast_ratio(text_colour, band_colour);
        }
    }
    if contrast < opts.min_contrast {
        notes.push(SpineNote::LowContrast { ratio: contrast });
    }
    if kind.is_flat() && (46.0..=59.0).contains(&background.to_lab().l) {
        notes.push(SpineNote::MidToneBackground);
    }
    let smallest = lines.author.as_ref().map_or(lines.title.size_pt, |a| a.size_pt);
    if text_colour.relative_luminance() > background.relative_luminance()
        && smallest < MIN_REVERSED_PT
    {
        notes.push(SpineNote::SmallReversedText { size_pt: smallest });
    }

    let paint = text_paint(text_colour);
    let mut author = None;
    if let (Some(run), Some((path, bounds))) = (&lines.author, placed_author) {
        author = Some(TextPlacement {
            text: run.text.clone(),
            size_pt: run.size_pt,
            bounds,
            item: items.len(),
        });
        items.push(Item::fill(path, paint));
    }
    let title = TextPlacement {
        text: lines.title.text.clone(),
        size_pt: lines.title.size_pt,
        bounds: title_bounds,
        item: items.len(),
    };
    items.push(Item::fill(title_path, paint));

    let mut design = finish(items, background, kind, text_colour, notes);
    design.title_size_pt = title.size_pt;
    design.contrast = contrast;
    design.title = Some(title);
    design.author = author;
    Ok(design)
}
