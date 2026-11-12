//! Print profiles: how much bleed, which marks, which sheet, which PDF flavour.
//!
//! # Where the numbers come from
//!
//! `docs/research/03-print-requirements.md` did not exist when this module was written, so
//! the values are the well-established defaults given in the crate's brief, cross-checked
//! against the raw research notes in `docs/research/notes/print-requirements.prepress.md`
//! and `docs/research/notes/print-requirements.folding.md` (search-extract based, largely
//! unverified — see their "Could not verify" sections). When the consolidated document
//! lands, its "Implications for the implementation" section takes precedence; every number
//! lives in this file.

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

/// How printer's marks are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MarkStyle {
    /// Solid hairlines in the **registration colour** (100 % on every separation), the
    /// convention for a press.
    #[default]
    Press,
    /// **Light grey, black-only** cut and fold guides for a home printer: no registration
    /// colour (which a home printer would render as a heavy four-colour black), and the
    /// fold marks get a dashed extension so they are not mistaken for cut marks.
    Guide,
}

/// Which printer's marks to draw around the spread, and how.
///
/// All marks live strictly outside the bleed box. The flags select what
/// [`marks`](crate::marks) puts in its layer; the individual generators
/// ([`crop_marks`](crate::crop_marks), …) ignore the flags.
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
    /// Marks never enter the bleed, so the effective offset is `max(offset, bleed)`.
    pub offset: f64,
    /// Length of crop and fold marks, in millimetres. Shortened when the slug (or the
    /// printable area of a home sheet) is too small; marks shorter than
    /// [`MIN_MARK_LENGTH_MM`](crate::MIN_MARK_LENGTH_MM) are omitted.
    pub length: f64,
    /// Stroke weight of all marks, in points.
    pub weight_pt: f64,
    /// Colour and line style of the marks.
    pub style: MarkStyle,
    /// A scale-check ruler (a line of known length with a tick every 10 mm) above the
    /// spread, so that a home user can verify the sheet was printed at 100 %.
    pub scale_ruler: bool,
}

impl Marks {
    /// The marks of [`PrintProfile::professional_eu`]: crop marks 5 mm long starting 3 mm
    /// beyond the trim (i.e. at the bleed edge), fold marks and registration targets,
    /// 0.25 pt, registration colour; no colour bar.
    #[must_use]
    pub const fn press() -> Self {
        Self {
            crop: true,
            fold: true,
            registration: true,
            color_bar: false,
            offset: 3.0,
            length: 5.0,
            weight_pt: 0.25,
            style: MarkStyle::Press,
            scale_ruler: false,
        }
    }

    /// The marks of [`PrintProfile::home`]: light grey cut and fold guides starting at the
    /// 2 mm bleed edge, 0.5 pt (a grey hairline is unreliable on home printers), a scale
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
    /// [`MarkStyle::Press`], a black-only light grey for [`MarkStyle::Guide`].
    #[must_use]
    pub const fn color(&self) -> Color {
        match self.style {
            MarkStyle::Press => Color::Registration,
            MarkStyle::Guide => Color::Gray(GUIDE_GRAY_LEVEL),
        }
    }

    /// The solid stroke marks are drawn with (butt caps, so a mark ends exactly where its
    /// path ends).
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

/// How an image's effective resolution compares with a profile's requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resolution {
    /// At or above the profile's `target_ppi`.
    Good,
    /// Between `min_ppi` and `target_ppi`: printable, but worth a warning.
    Low,
    /// Below `min_ppi` (or not a number): visibly pixelated; a printer may reject it.
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
    /// Total area coverage limit (sum of C, M, Y and K), in percent.
    pub max_total_ink_percent: f32,
    /// When set, the PDF page is this sheet with the spread centred on it (home printing);
    /// when `None` the page is trim + bleed + slug (the printer imposes it).
    pub sheet: Option<Sheet>,
    /// Width of the border of the [`sheet`](Self::sheet) the printer cannot print on, in
    /// millimetres. Marks stay inside it. Ignored when `sheet` is `None`.
    pub printable_margin: f64,
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
    /// | bleed | 3 mm (the most common European value; 2 mm and 5 mm also exist) |
    /// | safe margin | 4 mm from trim and creases |
    /// | spine safe margin | 1.5 mm from each crease (a 7 mm text band on a 10 mm spine) |
    /// | slug | 10 mm |
    /// | marks | crop marks 5 mm long starting at the bleed edge, fold marks, registration targets; 0.25 pt, registration colour |
    /// | PDF | PDF/X-4 |
    /// | images | 300 ppi target, 150 ppi minimum |
    /// | total ink | 300 % |
    ///
    /// Many web-to-print portals (Pixartprinting, Flyeralarm, Printoclock, Bookvault, …)
    /// explicitly want **no** marks: use [`PrintProfile::upload_eu`] for those.
    #[must_use]
    pub fn professional_eu() -> Self {
        Self {
            name: "professional_eu".to_owned(),
            bleed: 3.0,
            safe_margin: 4.0,
            spine_safe_margin: 1.5,
            slug: 10.0,
            marks: Marks::press(),
            standard: PdfStandard::X4,
            target_ppi: 300.0,
            min_ppi: 150.0,
            max_total_ink_percent: 300.0,
            sheet: None,
            printable_margin: 0.0,
        }
    }

    /// [`professional_eu`](Self::professional_eu) without marks and without a slug, for
    /// web-to-print upload portals: `MediaBox = BleedBox = trim + 3 mm`, `TrimBox = trim`.
    ///
    /// Not part of the original contract; added because the research notes
    /// (`print-requirements.prepress.md` §1.2 and §5) found that most online printers
    /// reject files with marks. Unverified against a live printer.
    #[must_use]
    pub fn upload_eu() -> Self {
        let mut p = Self::professional_eu().without_marks();
        p.name = "upload_eu".to_owned();
        p
    }

    /// Printing at home: the PDF page is `sheet` (landscape whenever the spread fits that
    /// way) with the spread centred on it, so that both sides register when the sheet is
    /// turned over.
    ///
    /// | | |
    /// |---|---|
    /// | bleed | 2 mm |
    /// | safe margin | 5 mm (hand cutting and home duplex registration are 1–2 mm off) |
    /// | spine safe margin | 2 mm (a 6 mm text band on a 10 mm spine) |
    /// | marks | light grey black-only cut and fold guides, 0.5 pt, scale ruler; no registration colour, no targets |
    /// | printable margin | 5 mm (home printers need 3–6.35 mm) |
    /// | PDF | plain PDF (no PDF/X claims, no output intent needed) |
    /// | images | 200 ppi target, 100 ppi minimum |
    ///
    /// The safe margins, stroke weight, resolution figures and the plain PDF flavour are
    /// this crate's assumptions (the brief fixes only bleed, guides and printable margin).
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
            max_total_ink_percent: 300.0,
            sheet: Some(sheet),
            printable_margin: 5.0,
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

    /// Classifies an effective image resolution against this profile.
    #[must_use]
    pub fn judge_ppi(&self, ppi: f64) -> Resolution {
        if ppi >= self.target_ppi {
            Resolution::Good
        } else if ppi >= self.min_ppi {
            Resolution::Low
        } else {
            Resolution::TooLow
        }
    }

    /// Pixel dimensions an image needs to fill `size` millimetres at `target_ppi`, rounded
    /// up. For a cover that bleeds, pass the panel **including** its bleed (see
    /// [`Panels::with_bleed`](crate::Panels::with_bleed)): a 110 × 178 mm poche front with
    /// 3 mm bleed needs 1335 × 2174 px at 300 ppi.
    #[must_use]
    pub fn target_px(&self, size: Size) -> (u32, u32) {
        px_at(size, self.target_ppi)
    }

    /// Pixel dimensions below which an image filling `size` millimetres falls under
    /// `min_ppi`, rounded up.
    #[must_use]
    pub fn min_px(&self, size: Size) -> (u32, u32) {
        px_at(size, self.min_ppi)
    }
}

fn px_at(size: Size, ppi: f64) -> (u32, u32) {
    // `as` saturates and maps NaN to 0, so hostile sizes cannot overflow or panic.
    let px = |mm: f64| (mm / MM_PER_INCH * ppi).ceil().max(0.0) as u32;
    (px(size.w), px(size.h))
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
        assert_eq!(p.safe_margin, 4.0);
        assert_eq!(p.spine_safe_margin, 1.5);
        assert_eq!(p.slug, 10.0);
        assert_eq!(p.standard, PdfStandard::X4);
        assert_eq!(p.target_ppi, 300.0);
        assert_eq!(p.min_ppi, 150.0);
        assert_eq!(p.max_total_ink_percent, 300.0);
        assert_eq!(p.sheet, None);
        let m = p.marks;
        assert!(m.crop && m.fold && m.registration && !m.color_bar && !m.scale_ruler);
        assert_eq!((m.offset, m.length, m.weight_pt), (3.0, 5.0, 0.25));
        // Crop marks start at the bleed edge and fit in the slug.
        assert_eq!(m.offset, p.bleed);
        assert!(m.offset + m.length <= p.bleed + p.slug);
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
            assert_eq!(p.printable_margin, 5.0);
            assert_eq!(p.sheet, Some(sheet));
            assert!(p.marks.crop && p.marks.fold && !p.marks.registration && !p.marks.color_bar);
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
        let p = PrintProfile::professional_eu();
        assert_eq!(p.judge_ppi(300.0), Resolution::Good);
        assert_eq!(p.judge_ppi(299.9), Resolution::Low);
        assert_eq!(p.judge_ppi(150.0), Resolution::Low);
        assert_eq!(p.judge_ppi(149.9), Resolution::TooLow);
        assert_eq!(p.judge_ppi(f64::NAN), Resolution::TooLow);
        assert_eq!(p.judge_ppi(-1.0), Resolution::TooLow);
        assert_eq!(p.judge_ppi(f64::INFINITY), Resolution::Good);
    }

    #[test]
    fn pixel_requirements() {
        let p = PrintProfile::professional_eu();
        // 110 + 3 mm by 178 + 6 mm at 300 ppi.
        assert_eq!(p.target_px(Size::new(113.0, 184.0)), (1335, 2174));
        assert_eq!(p.min_px(Size::new(25.4, 50.8)), (150, 300));
        // Hostile sizes saturate instead of panicking.
        assert_eq!(p.target_px(Size::new(f64::NAN, -5.0)), (0, 0));
        assert_eq!(p.target_px(Size::new(f64::INFINITY, 1e300)), (u32::MAX, u32::MAX));
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
    }
}
