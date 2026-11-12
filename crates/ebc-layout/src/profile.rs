//! Print profiles: how much bleed, which marks, which sheet, which PDF flavour.
//!
//! # Where the numbers come from
//!
//! `docs/research/03-print-requirements.md` — §1.2 (the default profile), §1.3 (the three
//! outputs), §2.4 (marks) and §9 (print at home) — as summarised in its "Implications for
//! the implementation" (items 2 to 4). That document is itself a synthesis of search
//! extracts: mark geometry and the home-printing figures are trade convention or house
//! rules, not printer specifications, and every printer says its product data sheet
//! overrides its general guide. Every number lives in this file so that it can be changed
//! in one place.
//!
//! Not implemented from that document: per-printer profile records and their stricter
//! "allowed ranges" (items 5 and 6 — [`PrintProfile::validate`] only rejects values the
//! layout cannot honour), and a stored warning threshold for image resolution (item 4 is
//! covered by [`PrintProfile::warn_ppi`], which derives it).

use ebc_scene::{Color, MM_PER_INCH, PdfStandard, Size, Stroke};

use crate::error::LayoutError;

/// Largest bleed, slug, margin, mark offset or mark length a profile may ask for.
/// A sanity bound (nothing in print needs more), which also keeps all arithmetic finite.
pub const MAX_MARGIN_MM: f64 = 100.0;
/// Largest stroke weight accepted for printer's marks, in points. Marks are hairlines;
/// the bound keeps a stroke's half-width well below the clearances between marks.
pub const MAX_MARK_WEIGHT_PT: f64 = 2.0;
/// Largest sheet edge accepted: the 200-inch limit of a PDF page at the default user unit.
pub const MAX_SHEET_MM: f64 = 5080.0;
/// Grey level (`0.0` = black, `1.0` = white) of [`MarkStyle::Guide`] marks: 45 % black.
/// Light enough not to shout if a sliver survives the cut, dark enough to survive a
/// draft-mode home print.
pub const GUIDE_GRAY_LEVEL: f32 = 0.55;
/// The unprintable border [`PrintProfile::home`] assumes on every side of the sheet:
/// 6.35 mm, a quarter of an inch.
///
/// Why this value: it is the usual hardware margin of laser printers and of inkjets on US
/// Letter, and the largest of the 3 to 6.35 mm that home printers publish for the left,
/// right and top edges; A4 devices often manage 3 to 5 mm, so on those the marks simply
/// sit a little further in than they had to. With it the default card (110 × 178 mm
/// covers) still gets its full 5 mm cut marks and its 100 mm ruler on A4 (7.65 mm
/// between the bleed and the printable edge, above and below) and on Letter (10.6 mm). A
/// larger default would cost that: at 10 mm the A4 marks shrink to 4 mm and the ruler to
/// 50 mm.
///
/// What it does not cover: some inkjets cannot print the last 12 to 15 mm at the *bottom*
/// of the sheet. There the lower cut and fold marks are clipped (the upper ones and the
/// card itself are not); raise `printable_margin` for such a device.
///
/// A house default from `docs/research/03-print-requirements.md` §9.2 (not fact-checked),
/// not a specification of any printer.
pub const HOME_PRINTABLE_MARGIN_MM: f64 = 6.35;

/// Total ink limit of [`PrintProfile::upload_eu`], in per cent: what upload and
/// print-on-demand printers accept.
///
/// Source: IngramSpark caps total ink at 240 % and recommends C60 M40 Y40 K100 (= 240 %) as
/// its rich black; digital presses and print-on-demand services generally ask for 240–
/// 260 % (Onlineprinters express 260 %, Lulu 270 %, `docs/research/03-print-requirements.md`
/// §4.3). 240 % is the strictest of these, so a file built under it passes all of them;
/// dark colours keep their look because they are rebuilt with more black and less colour
/// ink (`ebc_color::ink`), not scaled down. Owner-approved default; not confirmed with a
/// particular printer.
pub const UPLOAD_MAX_TOTAL_INK_PERCENT: f32 = 240.0;
/// Total ink limit of [`PrintProfile::professional_eu`], in per cent: offset on coated
/// stock.
///
/// Source: the ceiling of the FOGRA39 "ISO Coated v2 300 %" and FOGRA51 "PSO Coated v3"
/// conditions (300 %; the plain ECI "ISO Coated v2" allows 330 %), and the limit published
/// by Onlineprinters, Saxoprint, Exaprint and Flyeralarm (`docs/research/
/// 03-print-requirements.md` §3.5, §4.3). Also the nominal value of the home profiles,
/// whose content is sRGB and grey (a home printer's driver applies its own limit), so
/// nothing there is ever limited.
pub const OFFSET_MAX_TOTAL_INK_PERCENT: f32 = 300.0;

/// Image-resolution verdicts within this relative distance of a threshold count as
/// reaching it, so that a scan made at exactly 300 ppi is not judged "below 300" because
/// `152.4 / 25.4` is not exactly 6 in binary floating point.
const PPI_RELATIVE_TOLERANCE: f64 = 1e-9;
/// How far below `target_ppi` an image may be before it deserves a warning, in pixels per
/// inch. See [`PrintProfile::warn_ppi`].
const WARN_BELOW_TARGET_PPI: f64 = 50.0;

/// How printer's marks are drawn.
///
/// In every style crop marks are **solid** and fold marks are **dashed**, so that a crease
/// is never mistaken for a cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MarkStyle {
    /// Hairlines in the **registration colour** (100 % on every separation), the
    /// convention for an offset press.
    #[default]
    Press,
    /// Hairlines in **100 % black only**, for a job known to be printed digitally (where
    /// registration colour means four toners on top of each other).
    Black,
    /// **Light grey, black-only** cut and fold guides for a home printer: no registration
    /// colour (which a home printer would render as a heavy four-colour black), and the
    /// dashed fold marks run on to the edge of the printable area so that a ruler has
    /// something to align with.
    Guide,
}

/// Which printer's marks to draw around the spread, and how.
///
/// All marks live strictly outside the bleed box. The flags select what
/// [`marks`](crate::marks) puts in its layer; the individual generators
/// ([`crop_marks`](crate::crop_marks), …) ignore the flags.
///
/// `style` and `scale_ruler` are additions to the field list of `docs/architecture.md`;
/// build a value with [`Marks::press`], [`Marks::guides`], [`Marks::none`] or
/// `..Default::default()` (= [`Marks::press`]) rather than a full literal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marks {
    /// Crop (trim) marks at the four corners.
    pub crop: bool,
    /// Fold marks above and below the spread at both creases.
    pub fold: bool,
    /// Registration targets (circle + cross) centred on each side.
    pub registration: bool,
    /// A colour control strip (CMYK solids, overprints and 50 % tints) below the spread.
    pub color_bar: bool,
    /// Distance from the **trim** edge to the start of crop and fold marks, in millimetres.
    /// Marks never touch the bleed: the effective offset is
    /// `max(offset, bleed + `[`MARK_BLEED_GAP_MM`](crate::MARK_BLEED_GAP_MM)`)`.
    pub offset: f64,
    /// Length of crop and fold marks, in millimetres. Shortened when the slug (or the
    /// printable area of a home sheet) is too small; marks that would be shorter than
    /// [`MIN_MARK_LENGTH_MM`](crate::MIN_MARK_LENGTH_MM) are omitted
    /// ([`check_marks_fit`](crate::check_marks_fit) tells when that happens).
    pub length: f64,
    /// Stroke weight of all marks, in points.
    pub weight_pt: f64,
    /// Colour and line style of the marks.
    pub style: MarkStyle,
    /// A scale-check ruler (a line of known length with a tick every 10 mm) above the
    /// spread, so that a home user can verify the sheet was printed at 100 %.
    pub scale_ruler: bool,
}

impl Default for Marks {
    /// [`Marks::press`].
    fn default() -> Self {
        Self::press()
    }
}

impl Marks {
    /// The marks of [`PrintProfile::professional_eu`]: solid crop marks and dashed fold
    /// marks, 5 mm long, asked for 3 mm beyond the trim — the bleed edge, so they start
    /// [`MARK_BLEED_GAP_MM`](crate::MARK_BLEED_GAP_MM) (0.5 mm) beyond it —, 0.25 pt,
    /// registration colour. No registration targets and no colour bar: no surveyed printer
    /// asks for them (`docs/research/03-print-requirements.md` §2.4); switch them on per
    /// printer.
    #[must_use]
    pub const fn press() -> Self {
        Self {
            crop: true,
            fold: true,
            registration: false,
            color_bar: false,
            offset: 3.0,
            length: 5.0,
            weight_pt: 0.25,
            style: MarkStyle::Press,
            scale_ruler: false,
        }
    }

    /// The marks of [`PrintProfile::home`]: light grey cut and fold guides starting 0.5 mm
    /// beyond the 2 mm bleed edge, 0.5 pt (a grey hairline is unreliable on home printers), a scale
    /// ruler, no registration targets and no colour bar.
    #[must_use]
    pub const fn guides() -> Self {
        Self {
            crop: true,
            fold: true,
            registration: false,
            color_bar: false,
            offset: 2.0,
            length: 5.0,
            weight_pt: 0.5,
            style: MarkStyle::Guide,
            scale_ruler: true,
        }
    }

    /// No marks at all (geometry fields keep the press defaults).
    #[must_use]
    pub const fn none() -> Self {
        Self {
            crop: false,
            fold: false,
            registration: false,
            color_bar: false,
            scale_ruler: false,
            ..Self::press()
        }
    }

    /// Whether any mark is enabled.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.crop || self.fold || self.registration || self.color_bar || self.scale_ruler
    }

    /// The colour marks are stroked with: [`Color::Registration`] for
    /// [`MarkStyle::Press`], 100 % black for [`MarkStyle::Black`], a black-only light grey
    /// for [`MarkStyle::Guide`].
    #[must_use]
    pub const fn color(&self) -> Color {
        match self.style {
            MarkStyle::Press => Color::Registration,
            MarkStyle::Black => Color::BLACK,
            MarkStyle::Guide => Color::Gray(GUIDE_GRAY_LEVEL),
        }
    }

    /// The solid stroke crop marks, registration targets and the scale ruler are drawn
    /// with (butt caps, so a mark ends exactly where its path ends). Fold marks use the
    /// same colour and weight, dashed.
    #[must_use]
    pub fn stroke(&self) -> Stroke {
        Stroke::solid(self.color(), self.weight_pt)
    }

    fn validate(&self) -> Result<(), LayoutError> {
        check_range("marks.offset", self.offset, 0.0, MAX_MARGIN_MM)?;
        check_range("marks.length", self.length, 0.0, MAX_MARGIN_MM)?;
        check_range("marks.weight_pt", self.weight_pt, 0.0, MAX_MARK_WEIGHT_PT)?;
        if self.weight_pt <= 0.0 {
            return Err(LayoutError::InvalidProfile {
                field: "marks.weight_pt",
                value: self.weight_pt,
            });
        }
        Ok(())
    }
}

/// A sheet of paper the spread is printed on at home (or imposed on, one-up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sheet {
    /// ISO A4, 210 × 297 mm.
    A4,
    /// US Letter, 8.5 × 11 in (215.9 × 279.4 mm).
    Letter,
    /// ISO A3, 297 × 420 mm.
    A3,
    /// SRA3, 320 × 450 mm (the usual digital-press sheet).
    Sra3,
    /// Any other sheet; the orientation in which it is given does not matter.
    Custom(Size),
}

impl Sheet {
    /// The named sheets, with the names accepted by [`Sheet::by_name`].
    pub const NAMED: [(&'static str, Self); 4] =
        [("a4", Self::A4), ("letter", Self::Letter), ("a3", Self::A3), ("sra3", Self::Sra3)];

    /// The sheet size as conventionally written (portrait for the named sheets, as given
    /// for [`Sheet::Custom`]).
    #[must_use]
    pub const fn size(&self) -> Size {
        match *self {
            Self::A4 => Size::new(210.0, 297.0),
            Self::Letter => Size::new(215.9, 279.4),
            Self::A3 => Size::new(297.0, 420.0),
            Self::Sra3 => Size::new(320.0, 450.0),
            Self::Custom(size) => size,
        }
    }

    /// The sheet with its long edge horizontal.
    #[must_use]
    pub fn landscape(&self) -> Size {
        let s = self.size();
        Size::new(s.w.max(s.h), s.w.min(s.h))
    }

    /// The sheet with its long edge vertical.
    #[must_use]
    pub fn portrait(&self) -> Size {
        let s = self.size();
        Size::new(s.w.min(s.h), s.w.max(s.h))
    }

    /// A short lower-case name: `"a4"`, `"letter"`, `"a3"`, `"sra3"` or `"custom"`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::A4 => "a4",
            Self::Letter => "letter",
            Self::A3 => "a3",
            Self::Sra3 => "sra3",
            Self::Custom(_) => "custom",
        }
    }

    /// Looks a named sheet up (case-insensitive, surrounding whitespace ignored).
    /// `"custom"` is not a name: build [`Sheet::Custom`] directly.
    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::NAMED.into_iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, s)| s)
    }

    pub(crate) fn validate(&self) -> Result<(), LayoutError> {
        let s = self.size();
        let ok = |v: f64| v > 0.0 && v <= MAX_SHEET_MM;
        if ok(s.w) && ok(s.h) { Ok(()) } else { Err(LayoutError::InvalidSheet { w: s.w, h: s.h }) }
    }
}

/// How an image's effective resolution compares with a profile's requirements
/// ([`PrintProfile::judge_ppi`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resolution {
    /// At or above the profile's `target_ppi`.
    Good,
    /// Below `target_ppi` but at or above [`PrintProfile::warn_ppi`]: slightly soft at
    /// most; no warning.
    Acceptable,
    /// Below [`PrintProfile::warn_ppi`] but at or above `min_ppi`: printable, but worth a
    /// **warning**.
    Low,
    /// Below `min_ppi` (or not a number): visibly pixelated, an **error**; a printer may
    /// reject it.
    TooLow,
}

/// Everything about the printing process that changes the page geometry or the checks
/// applied to the artwork.
///
/// The fields are public so a profile can be adapted to a printer's data sheet (every
/// surveyed printer says the product data sheet overrides its general guide);
/// [`validate`](Self::validate) is re-run by everything that consumes a profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PrintProfile {
    /// Machine-friendly name, e.g. `"professional_eu"` or `"home_a4"`.
    pub name: String,
    /// Bleed beyond the trim on every side, in millimetres.
    pub bleed: f64,
    /// Distance text and other important content keeps from the trim edges **and** from
    /// the creases on the cover panels, in millimetres.
    pub safe_margin: f64,
    /// Distance spine text keeps from each crease, in millimetres (the spine is narrow, so
    /// it gets its own, tighter margin).
    pub spine_safe_margin: f64,
    /// Margin beyond the bleed reserved for marks, in millimetres. Ignored when
    /// [`sheet`](Self::sheet) is set: the sheet then decides how much room there is.
    pub slug: f64,
    /// Which printer's marks to draw.
    pub marks: Marks,
    /// The PDF flavour to write.
    pub standard: PdfStandard,
    /// Image resolution to aim for at final size, in pixels per inch.
    pub target_ppi: f64,
    /// Image resolution below which the result is considered unprintable.
    pub min_ppi: f64,
    /// Total area coverage limit (sum of C, M, Y and K), in percent: the most ink any
    /// point of the CMYK content may carry. The card composer builds every process colour
    /// under it — grounds, the converted cover and spine images — by grey-component
    /// replacement through the output profile (`ebc_color::ink`), and the preflight
    /// checks fills, strokes and losslessly stored images against it. The defaults are
    /// [`UPLOAD_MAX_TOTAL_INK_PERCENT`] for upload/print-on-demand,
    /// [`OFFSET_MAX_TOTAL_INK_PERCENT`] for offset on coated stock and for the nominal
    /// value of home profiles (whose content is sRGB and grey, never limited).
    pub max_total_ink_percent: f32,
    /// When set, the PDF page is this sheet with the spread centred on it (home printing);
    /// when `None` the page is trim + bleed + slug (the printer imposes it).
    pub sheet: Option<Sheet>,
    /// Width of the border of the [`sheet`](Self::sheet) the printer cannot print on, in
    /// millimetres. Marks stay inside it. Ignored when `sheet` is `None`.
    pub printable_margin: f64,
    /// Whether the file may carry RGB (tagged sRGB images) at all. `true` for print at
    /// home (plain PDF, sRGB images passed through); `false` for the professional
    /// profiles, whose content is CMYK and grey only
    /// (`docs/research/03-print-requirements.md` §3.2, §3.4, implication 23). Set it for
    /// a printer that says it converts RGB itself: a card composer may then fall back to
    /// tagged sRGB when it cannot convert to the output profile. Preflight takes its
    /// colour expectation from this flag.
    pub allow_rgb: bool,
}

impl PrintProfile {
    /// Names accepted by [`PrintProfile::by_name`] (besides `home_<sheet>`).
    pub const NAMES: [&'static str; 6] =
        ["professional_eu", "upload_eu", "home_a4", "home_letter", "home_a3", "home_sra3"];

    /// A conservative profile for European professional printers that accept (or want)
    /// marks: traditional and local print shops.
    ///
    /// | | |
    /// |---|---|
    /// | bleed | 3 mm (the most common European value; 1, 2, 3.175 and 5 mm also exist) |
    /// | safe margin | 5 mm from trim and creases (the largest of the published 3, 4 and 5 mm) |
    /// | spine safe margin | 2 mm from each crease (a 6 mm text band on a 10 mm spine) |
    /// | slug | 10 mm |
    /// | marks | solid crop marks and dashed fold marks, 5 mm long, starting 0.5 mm beyond the bleed edge, one object each; 0.25 pt, registration colour; no registration targets, no colour bar |
    /// | PDF | PDF/X-4 |
    /// | images | 300 ppi target, warning below 250 ppi, error below 150 ppi |
    /// | total ink | 300 % ([`OFFSET_MAX_TOTAL_INK_PERCENT`]: offset, coated stock) |
    /// | colour | CMYK and grey only (`allow_rgb = false`) |
    ///
    /// Many web-to-print portals (Pixartprinting, Flyeralarm, Printoclock, Bookvault, …)
    /// explicitly want **no** marks: use [`PrintProfile::upload_eu`] for those.
    #[must_use]
    pub fn professional_eu() -> Self {
        Self {
            name: "professional_eu".to_owned(),
            bleed: 3.0,
            safe_margin: 5.0,
            spine_safe_margin: 2.0,
            slug: 10.0,
            marks: Marks::press(),
            standard: PdfStandard::X4,
            target_ppi: 300.0,
            min_ppi: 150.0,
            max_total_ink_percent: OFFSET_MAX_TOTAL_INK_PERCENT,
            sheet: None,
            printable_margin: 0.0,
            allow_rgb: false,
        }
    }

    /// [`professional_eu`](Self::professional_eu) without marks and without a slug, for
    /// web-to-print upload portals: `MediaBox = BleedBox = trim + 3 mm`, `TrimBox = trim`,
    /// and a total ink limit of 240 % ([`UPLOAD_MAX_TOTAL_INK_PERCENT`]), the strictest
    /// value of print-on-demand and digital presses.
    ///
    /// Not part of the original contract; added because most online printers reject files
    /// with marks (`docs/research/03-print-requirements.md` §2.3), which makes this the
    /// default professional output. Unverified against a live printer.
    #[must_use]
    pub fn upload_eu() -> Self {
        let mut p = Self::professional_eu().without_marks();
        p.name = "upload_eu".to_owned();
        p.max_total_ink_percent = UPLOAD_MAX_TOTAL_INK_PERCENT;
        p
    }

    /// Printing at home: the spread centred on `sheet`, which is used in the orientation of
    /// the spread (landscape for every preset), so that the trim boxes of the two sides
    /// coincide when the sheet is turned over.
    ///
    /// **Duplex setting.** The inside page is laid out for a sheet turned about the axis
    /// parallel to the creases. On a landscape sheet that is "flip on the **short** edge";
    /// the print-dialog default, "flip on long edge", prints the inside upside down behind
    /// the wrong cover (and nothing looks wrong until the card is opened, because the
    /// spread is symmetric). [`SheetPlacement::duplex_flip`](crate::SheetPlacement::duplex_flip)
    /// says which setting a given card needs; the composer must print it on the sheet.
    ///
    /// | | |
    /// |---|---|
    /// | bleed | 2 mm |
    /// | safe margin | 5 mm (hand cutting and home duplex registration are 1–2 mm off) |
    /// | spine safe margin | 2 mm (a 6 mm text band on a 10 mm spine) |
    /// | marks | light grey black-only guides, 0.5 pt: solid cut marks, dashed fold marks, a scale ruler; no registration colour, no targets |
    /// | printable margin | [`HOME_PRINTABLE_MARGIN_MM`]: 6.35 mm (0.25 in), the usual hardware margin of laser printers and of inkjets on Letter |
    /// | PDF | plain PDF (no PDF/X claims, no output intent needed) |
    /// | images | 200 ppi target, warning below 150 ppi, error below 100 ppi |
    /// | colour | sRGB images and colours allowed (`allow_rgb = true`) |
    /// | total ink | nominally 300 % ([`OFFSET_MAX_TOTAL_INK_PERCENT`]); no CMYK content is written, so nothing is limited — the printer driver applies its own limit |
    ///
    /// Values from `docs/research/03-print-requirements.md` §9.2; the unprintable margin of
    /// a given printer may still be larger at the bottom edge (12–15 mm on some old
    /// inkjets), which would clip the lower marks but not the card.
    ///
    /// The PDF page is the sheet itself — see [`new_page`](crate::new_page).
    #[must_use]
    pub fn home(sheet: Sheet) -> Self {
        Self {
            name: format!("home_{}", sheet.name()),
            bleed: 2.0,
            safe_margin: 5.0,
            spine_safe_margin: 2.0,
            slug: 0.0,
            marks: Marks::guides(),
            standard: PdfStandard::Plain,
            target_ppi: 200.0,
            min_ppi: 100.0,
            max_total_ink_percent: OFFSET_MAX_TOTAL_INK_PERCENT,
            sheet: Some(sheet),
            printable_margin: HOME_PRINTABLE_MARGIN_MM,
            allow_rgb: true,
        }
    }

    /// Looks a built-in profile up by name: `"professional_eu"`, `"upload_eu"`, or
    /// `"home_<sheet>"` with a [`Sheet::by_name`] name (`"home"` alone means A4).
    /// Case-insensitive; `-` and `_` are interchangeable.
    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        let name = name.trim().to_ascii_lowercase().replace('-', "_");
        match name.as_str() {
            "professional_eu" => Some(Self::professional_eu()),
            "upload_eu" => Some(Self::upload_eu()),
            "home" => Some(Self::home(Sheet::A4)),
            other => other.strip_prefix("home_").and_then(Sheet::by_name).map(Self::home),
        }
    }

    /// This profile with every mark switched off and no slug, keeping bleed and the rest.
    /// For a sheet-based profile the page stays the sheet.
    #[must_use]
    pub fn without_marks(mut self) -> Self {
        self.marks = Marks { style: self.marks.style, ..Marks::none() };
        self.slug = 0.0;
        self
    }

    /// Checks that every numeric field is finite and within sane bounds
    /// (lengths `0..=`[`MAX_MARGIN_MM`], `0 < min_ppi <= target_ppi`,
    /// `0 < max_total_ink_percent <= 400`, a usable sheet).
    ///
    /// # Errors
    /// [`LayoutError::InvalidProfile`] naming the first offending field, or
    /// [`LayoutError::InvalidSheet`].
    pub fn validate(&self) -> Result<(), LayoutError> {
        check_range("bleed", self.bleed, 0.0, MAX_MARGIN_MM)?;
        check_range("safe_margin", self.safe_margin, 0.0, MAX_MARGIN_MM)?;
        check_range("spine_safe_margin", self.spine_safe_margin, 0.0, MAX_MARGIN_MM)?;
        check_range("slug", self.slug, 0.0, MAX_MARGIN_MM)?;
        check_range("printable_margin", self.printable_margin, 0.0, MAX_MARGIN_MM)?;
        self.marks.validate()?;
        check_range("target_ppi", self.target_ppi, 1.0, 10_000.0)?;
        check_range("min_ppi", self.min_ppi, 1.0, self.target_ppi)?;
        let ink = f64::from(self.max_total_ink_percent);
        if !(ink > 0.0 && ink <= 400.0) {
            return Err(LayoutError::InvalidProfile { field: "max_total_ink_percent", value: ink });
        }
        if let Some(sheet) = &self.sheet {
            sheet.validate()?;
        }
        Ok(())
    }

    /// The resolution below which an image deserves a **warning**, in pixels per inch:
    /// 50 ppi under `target_ppi`, but never under `min_ppi`.
    ///
    /// `docs/research/03-print-requirements.md` §1.3 sets three thresholds per profile —
    /// 300 / 250 / 150 ppi for professional printing and 200 / 150 / 100 ppi at home — as
    /// house rules. Both warning levels are "target − 50", which is the rule used here so
    /// that [`PrintProfile`] keeps the field list of `docs/architecture.md`. A profile that
    /// needs an independent warning level would have to turn this into a field.
    #[must_use]
    pub fn warn_ppi(&self) -> f64 {
        // `f64::max` ignores a NaN operand, so a hostile profile cannot produce NaN here
        // unless both fields are NaN (and then every comparison below is false).
        (self.target_ppi - WARN_BELOW_TARGET_PPI).max(self.min_ppi)
    }

    /// Classifies an effective image resolution (see
    /// [`effective_ppi`](crate::effective_ppi)) against this profile: at or above
    /// `target_ppi` is [`Resolution::Good`], then [`Resolution::Acceptable`] down to
    /// [`warn_ppi`](Self::warn_ppi), [`Resolution::Low`] down to `min_ppi`, and
    /// [`Resolution::TooLow`] below that or for NaN.
    ///
    /// A threshold counts as reached from one part in a billion below it, so that
    /// floating-point rounding in the pixels-to-ppi conversion never demotes an image
    /// that has exactly the pixels [`target_px`](Self::target_px) or
    /// [`min_px`](Self::min_px) ask for.
    #[must_use]
    pub fn judge_ppi(&self, ppi: f64) -> Resolution {
        // Written so that NaN (in `ppi` or in a hostile profile) fails every test.
        let reaches = |threshold: f64| ppi >= threshold * (1.0 - PPI_RELATIVE_TOLERANCE);
        if reaches(self.target_ppi) {
            Resolution::Good
        } else if reaches(self.warn_ppi()) {
            Resolution::Acceptable
        } else if reaches(self.min_ppi) {
            Resolution::Low
        } else {
            Resolution::TooLow
        }
    }

    /// The smallest pixel dimensions with which an image filling `size` millimetres
    /// reaches `target_ppi`. A 6 × 9 in (152.4 × 228.6 mm) panel needs exactly
    /// 1800 × 2700 px at 300 ppi; the default cover box of the poche card (the front
    /// panel inset by the 5 mm safe margin, 100 × 168 mm) needs 1182 × 1985 px, and a
    /// full-bleed poche front (113 × 184 mm with its 3 mm bleed on three sides) needs
    /// 1335 × 2174 px.
    #[must_use]
    pub fn target_px(&self, size: Size) -> (u32, u32) {
        px_at(size, self.target_ppi)
    }

    /// The smallest pixel dimensions with which an image filling `size` millimetres
    /// reaches `min_ppi`; anything smaller is [`Resolution::TooLow`].
    #[must_use]
    pub fn min_px(&self, size: Size) -> (u32, u32) {
        px_at(size, self.min_ppi)
    }
}

/// Pixels needed along `mm` millimetres at `ppi`: `mm × ppi / 25.4`, rounded up — unless
/// the product is a whole number but for floating-point rounding (`152.4 / 25.4 × 300` is
/// `1800.0000000000002`), in which case it is that whole number.
fn px_along(mm: f64, ppi: f64) -> u32 {
    // Multiplying before dividing keeps inch-based sizes exact more often.
    let exact = mm * ppi / MM_PER_INCH;
    let nearest = exact.round();
    let px = if (exact - nearest).abs() <= PPI_RELATIVE_TOLERANCE * nearest.abs().max(1.0) {
        nearest
    } else {
        exact.ceil()
    };
    // `f64::max` drops NaN and `as` saturates, so hostile sizes cannot overflow or panic.
    px.max(0.0) as u32
}

fn px_at(size: Size, ppi: f64) -> (u32, u32) {
    (px_along(size.w, ppi), px_along(size.h, ppi))
}

fn check_range(field: &'static str, value: f64, min: f64, max: f64) -> Result<(), LayoutError> {
    // Written so that NaN fails.
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(LayoutError::InvalidProfile { field, value })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn professional_eu_matches_the_brief() {
        let p = PrintProfile::professional_eu();
        assert_eq!(p.name, "professional_eu");
        assert_eq!(p.bleed, 3.0);
        assert_eq!(p.safe_margin, 5.0);
        assert_eq!(p.spine_safe_margin, 2.0);
        assert_eq!(p.slug, 10.0);
        assert_eq!(p.standard, PdfStandard::X4);
        assert_eq!(p.target_ppi, 300.0);
        assert_eq!(p.min_ppi, 150.0);
        assert_eq!(p.max_total_ink_percent, 300.0);
        assert_eq!(p.sheet, None);
        assert!(!p.allow_rgb, "professional output is CMYK and grey only");
        assert!(!PrintProfile::upload_eu().allow_rgb);
        assert_eq!(p.warn_ppi(), 250.0);
        let m = p.marks;
        // Crop and fold marks only: no surveyed printer asks for targets or a colour bar.
        assert!(m.crop && m.fold && !m.registration && !m.color_bar && !m.scale_ruler);
        assert_eq!((m.offset, m.length, m.weight_pt), (3.0, 5.0, 0.25));
        // The offset asks for the bleed edge: the marks start MARK_BLEED_GAP_MM beyond it
        // and still fit in the slug at full length.
        assert_eq!(m.offset, p.bleed);
        assert!(p.bleed + crate::MARK_BLEED_GAP_MM + m.length <= p.bleed + p.slug);
        assert_eq!(m.style, MarkStyle::Press);
        assert_eq!(m.color(), Color::Registration);
        assert_eq!(p.validate(), Ok(()));
    }

    #[test]
    fn home_matches_the_brief() {
        for (name, sheet) in Sheet::NAMED {
            let p = PrintProfile::home(sheet);
            assert_eq!(p.name, format!("home_{name}"));
            assert_eq!(p.bleed, 2.0);
            assert_eq!((p.safe_margin, p.spine_safe_margin), (5.0, 2.0));
            // A quarter of an inch: the largest hardware margin of common home printers.
            assert_eq!(p.printable_margin, 6.35);
            assert_eq!(p.printable_margin, HOME_PRINTABLE_MARGIN_MM);
            assert_eq!((p.target_ppi, p.warn_ppi(), p.min_ppi), (200.0, 150.0, 100.0));
            assert_eq!(p.standard, PdfStandard::Plain);
            assert!(p.allow_rgb, "home printers work in RGB");
            assert_eq!(p.sheet, Some(sheet));
            assert!(p.marks.crop && p.marks.fold && !p.marks.registration && !p.marks.color_bar);
            assert!(p.marks.scale_ruler);
            assert_eq!(p.marks.weight_pt, 0.5);
            assert_eq!(p.marks.style, MarkStyle::Guide);
            // Light grey, black only: never the registration colour, never RGB.
            match p.marks.color() {
                Color::Gray(v) => assert!((0.4..=0.8).contains(&v), "grey level {v}"),
                other => panic!("home marks must be grey, got {other:?}"),
            }
            assert_eq!(p.marks.offset, p.bleed);
            assert_eq!(p.validate(), Ok(()));
        }
        assert_eq!(PrintProfile::home(Sheet::Custom(Size::new(400.0, 300.0))).name, "home_custom");
    }

    #[test]
    fn upload_profile_has_no_marks_and_no_slug() {
        let p = PrintProfile::upload_eu();
        assert_eq!(p.name, "upload_eu");
        assert!(!p.marks.any());
        assert_eq!(p.slug, 0.0);
        assert_eq!(p.bleed, 3.0);
        assert_eq!(p.standard, PdfStandard::X4);
        // Print-on-demand ink limit; offset keeps 300 %.
        assert_eq!(p.max_total_ink_percent, 240.0);
        assert_eq!(PrintProfile::home(Sheet::A4).max_total_ink_percent, 300.0);
        assert_eq!(p.validate(), Ok(()));
        // `without_marks` keeps the style so that re-enabling one mark stays consistent.
        let h = PrintProfile::home(Sheet::A4).without_marks();
        assert!(!h.marks.any());
        assert_eq!(h.marks.style, MarkStyle::Guide);
        assert_eq!(h.sheet, Some(Sheet::A4));
    }

    #[test]
    fn by_name_finds_every_listed_profile() {
        for name in PrintProfile::NAMES {
            let p = PrintProfile::by_name(name).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(p.name, name);
        }
        assert_eq!(PrintProfile::by_name(" Home-Letter "), Some(PrintProfile::home(Sheet::Letter)));
        assert_eq!(PrintProfile::by_name("home"), Some(PrintProfile::home(Sheet::A4)));
        assert_eq!(PrintProfile::by_name("PROFESSIONAL-EU"), Some(PrintProfile::professional_eu()));
        for bad in ["", "home_", "home_custom", "home_a5", "pro", "professional_us", "\u{0}"] {
            assert_eq!(PrintProfile::by_name(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn sheets() {
        assert_eq!(Sheet::A4.size(), Size::new(210.0, 297.0));
        assert_eq!(Sheet::A4.landscape(), Size::new(297.0, 210.0));
        assert_eq!(Sheet::A4.portrait(), Size::new(210.0, 297.0));
        assert_eq!(Sheet::Letter.landscape(), Size::new(279.4, 215.9));
        assert_eq!(Sheet::A3.landscape(), Size::new(420.0, 297.0));
        assert_eq!(Sheet::Sra3.landscape(), Size::new(450.0, 320.0));
        // A custom sheet may be given in either orientation.
        let c = Sheet::Custom(Size::new(400.0, 250.0));
        assert_eq!(c.size(), Size::new(400.0, 250.0));
        assert_eq!(c.landscape(), Size::new(400.0, 250.0));
        assert_eq!(c.portrait(), Size::new(250.0, 400.0));
        assert_eq!(Sheet::by_name("SRA3"), Some(Sheet::Sra3));
        assert_eq!(Sheet::by_name("custom"), None);
        for (name, sheet) in Sheet::NAMED {
            assert_eq!(sheet.name(), name);
            assert_eq!(sheet.validate(), Ok(()));
        }
    }

    #[test]
    fn hostile_sheets_are_rejected() {
        for (w, h) in [
            (0.0, 100.0),
            (100.0, 0.0),
            (-210.0, 297.0),
            (f64::NAN, 297.0),
            (210.0, f64::INFINITY),
            (f64::MAX, f64::MAX),
            (5080.1, 100.0),
        ] {
            let sheet = Sheet::Custom(Size::new(w, h));
            assert!(matches!(sheet.validate(), Err(LayoutError::InvalidSheet { .. })), "{w} x {h}");
            let p = PrintProfile::home(sheet);
            assert!(matches!(p.validate(), Err(LayoutError::InvalidSheet { .. })));
        }
    }

    #[test]
    fn every_numeric_field_is_validated() {
        type Set = fn(&mut PrintProfile, f64);
        let fields: [(&str, Set); 10] = [
            ("bleed", |p, v| p.bleed = v),
            ("safe_margin", |p, v| p.safe_margin = v),
            ("spine_safe_margin", |p, v| p.spine_safe_margin = v),
            ("slug", |p, v| p.slug = v),
            ("printable_margin", |p, v| p.printable_margin = v),
            ("marks.offset", |p, v| p.marks.offset = v),
            ("marks.length", |p, v| p.marks.length = v),
            ("marks.weight_pt", |p, v| p.marks.weight_pt = v),
            ("target_ppi", |p, v| p.target_ppi = v),
            ("min_ppi", |p, v| p.min_ppi = v),
        ];
        for (name, set) in fields {
            for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 1e9] {
                let mut p = PrintProfile::professional_eu();
                set(&mut p, v);
                match p.validate() {
                    Err(LayoutError::InvalidProfile { field, .. }) => {
                        // min_ppi is checked against target_ppi, so a bad target may be
                        // reported under either name.
                        assert!(field == name || name.ends_with("ppi"), "{name} = {v}: {field}");
                    }
                    other => panic!("{name} = {v} gave {other:?}"),
                }
            }
        }
        for v in [f32::NAN, 0.0, -10.0, 400.5, f32::INFINITY] {
            let mut p = PrintProfile::professional_eu();
            p.max_total_ink_percent = v;
            assert!(
                matches!(
                    p.validate(),
                    Err(LayoutError::InvalidProfile { field: "max_total_ink_percent", .. })
                ),
                "ink {v}"
            );
        }
        // Zero stroke weight would make the marks invisible.
        let mut p = PrintProfile::professional_eu();
        p.marks.weight_pt = 0.0;
        assert!(p.validate().is_err());
        // min above target is contradictory.
        let mut p = PrintProfile::professional_eu();
        p.min_ppi = 301.0;
        assert!(matches!(p.validate(), Err(LayoutError::InvalidProfile { field: "min_ppi", .. })));
        // Zero is fine for margins.
        let mut p = PrintProfile::professional_eu();
        (p.bleed, p.slug, p.safe_margin, p.spine_safe_margin) = (0.0, 0.0, 0.0, 0.0);
        (p.marks.offset, p.marks.length) = (0.0, 0.0);
        assert_eq!(p.validate(), Ok(()));
    }

    #[test]
    fn resolution_verdicts() {
        // Professional: 300 good, warning below 250, error below 150.
        let p = PrintProfile::professional_eu();
        assert_eq!(p.judge_ppi(300.0), Resolution::Good);
        assert_eq!(p.judge_ppi(299.9), Resolution::Acceptable);
        assert_eq!(p.judge_ppi(250.0), Resolution::Acceptable);
        assert_eq!(p.judge_ppi(249.9), Resolution::Low);
        assert_eq!(p.judge_ppi(150.0), Resolution::Low);
        assert_eq!(p.judge_ppi(149.9), Resolution::TooLow);
        assert_eq!(p.judge_ppi(f64::NAN), Resolution::TooLow);
        assert_eq!(p.judge_ppi(-1.0), Resolution::TooLow);
        assert_eq!(p.judge_ppi(f64::INFINITY), Resolution::Good);
        // Home: 200 good, warning below 150, error below 100.
        let h = PrintProfile::home(Sheet::A4);
        for (ppi, want) in [
            (200.0, Resolution::Good),
            (199.0, Resolution::Acceptable),
            (150.0, Resolution::Acceptable),
            (149.0, Resolution::Low),
            (100.0, Resolution::Low),
            (99.0, Resolution::TooLow),
        ] {
            assert_eq!(h.judge_ppi(ppi), want, "{ppi} ppi at home");
        }
    }

    #[test]
    fn verdicts_tolerate_rounding_but_not_a_real_shortfall() {
        let p = PrintProfile::professional_eu();
        // What `1800 px / (152.4 mm / 25.4)` gives in f64: one ulp under 300.
        assert_eq!(p.judge_ppi(299.999_999_999_999_94), Resolution::Good);
        assert_eq!(p.judge_ppi(149.999_999_999_999_97), Resolution::Low);
        // A millionth of a ppi short is short.
        assert_eq!(p.judge_ppi(299.999_999), Resolution::Acceptable);
        assert_eq!(p.judge_ppi(149.999_999), Resolution::TooLow);
    }

    #[test]
    fn warning_threshold_never_drops_below_the_minimum() {
        let mut p = PrintProfile::professional_eu();
        (p.target_ppi, p.min_ppi) = (120.0, 100.0);
        assert_eq!(p.warn_ppi(), 100.0);
        // With no room between warning and error there is no `Low` band.
        assert_eq!(p.judge_ppi(110.0), Resolution::Acceptable);
        assert_eq!(p.judge_ppi(99.0), Resolution::TooLow);
        // Hostile profiles: no panic, and NaN never passes for good.
        (p.target_ppi, p.min_ppi) = (f64::NAN, f64::NAN);
        assert!(p.warn_ppi().is_nan());
        assert_eq!(p.judge_ppi(1e9), Resolution::TooLow);
    }

    #[test]
    fn pixel_requirements() {
        let p = PrintProfile::professional_eu();
        // The figures of docs/research/03-print-requirements.md §4.2: full-bleed poche
        // front, 110 + 3 mm by 178 + 6 mm, at 300 ppi and at the 150 ppi floor.
        assert_eq!(p.target_px(Size::new(113.0, 184.0)), (1335, 2174));
        assert_eq!(p.min_px(Size::new(113.0, 184.0)), (668, 1087));
        // The default inset box, 100 x 168 mm.
        assert_eq!(p.target_px(Size::new(100.0, 168.0)), (1182, 1985));
        assert_eq!(p.min_px(Size::new(100.0, 168.0)), (591, 993));
        assert_eq!(p.min_px(Size::new(25.4, 50.8)), (150, 300));
        // Inch-based sizes are exact: 6 x 9 in is 1800 x 2700 px, not 1801.
        assert_eq!(p.target_px(Size::new(152.4, 228.6)), (1800, 2700));
        assert_eq!(p.target_px(Size::new(76.2, 177.8)), (900, 2100));
        // A hair above a whole number of pixels still rounds up.
        assert_eq!(p.target_px(Size::new(152.4001, 25.4)), (1801, 300));
        assert_eq!(p.target_px(Size::new(0.0, 1e-6)), (0, 1));
        // Hostile sizes saturate instead of panicking.
        assert_eq!(p.target_px(Size::new(f64::NAN, -5.0)), (0, 0));
        assert_eq!(p.target_px(Size::new(f64::INFINITY, 1e300)), (u32::MAX, u32::MAX));
        assert_eq!(p.target_px(Size::new(f64::NEG_INFINITY, f64::MIN)), (0, 0));
    }

    #[test]
    fn mark_presets() {
        assert!(Marks::press().any());
        assert!(Marks::guides().any());
        assert!(!Marks::none().any());
        let s = Marks::press().stroke();
        assert_eq!(s.width_pt, 0.25);
        assert!(s.dash_mm.is_empty());
        assert_eq!(s.color, Color::Registration);
        assert_eq!(Marks::guides().stroke().color, Color::Gray(GUIDE_GRAY_LEVEL));
        // The contract's field list plus `..Default::default()` is enough to build marks.
        let m = Marks {
            crop: true,
            fold: false,
            registration: false,
            color_bar: false,
            offset: 4.0,
            length: 3.0,
            weight_pt: 0.3,
            ..Default::default()
        };
        assert_eq!((m.style, m.scale_ruler), (MarkStyle::Press, false));
        assert_eq!(Marks::default(), Marks::press());
        // Digital jobs: the same marks in black only, never registration colour.
        let black = Marks { style: MarkStyle::Black, ..Marks::press() };
        assert_eq!(black.color(), Color::BLACK);
        assert_eq!(black.color().total_ink_percent(), Some(100.0));
    }
}
