//! The crate's error type.

use ebc_scene::Size;

/// Why a card format, a print profile or their combination was rejected.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum LayoutError {
    /// A cover dimension is not finite or outside
    /// [`CardFormat::COVER_MIN_MM`](crate::CardFormat::COVER_MIN_MM)`..=`[`CardFormat::COVER_MAX_MM`](crate::CardFormat::COVER_MAX_MM).
    #[error("cover {dimension} of {value} mm is outside the supported range {min}..={max} mm")]
    CoverOutOfRange {
        /// `"width"` or `"height"`.
        dimension: &'static str,
        /// The rejected value, in millimetres (may be NaN or infinite).
        value: f64,
        /// Smallest accepted value, in millimetres.
        min: f64,
        /// Largest accepted value, in millimetres.
        max: f64,
    },
    /// The spine width is not finite or outside
    /// [`CardFormat::SPINE_MIN_MM`](crate::CardFormat::SPINE_MIN_MM)`..=`[`CardFormat::SPINE_MAX_MM`](crate::CardFormat::SPINE_MAX_MM).
    #[error("spine width of {value} mm is outside the supported range {min}..={max} mm")]
    SpineOutOfRange {
        /// The rejected value, in millimetres (may be NaN or infinite).
        value: f64,
        /// Smallest accepted value, in millimetres.
        min: f64,
        /// Largest accepted value, in millimetres.
        max: f64,
    },
    /// A numeric field of the [`PrintProfile`](crate::PrintProfile) is not finite, negative
    /// or otherwise outside what the layout can honour.
    #[error("print profile field `{field}` has an unusable value: {value}")]
    InvalidProfile {
        /// Name of the offending field, e.g. `"bleed"` or `"marks.weight_pt"`.
        field: &'static str,
        /// The rejected value.
        value: f64,
    },
    /// A [`Sheet::Custom`](crate::Sheet::Custom) size is not finite or not positive.
    #[error("sheet size {w} x {h} mm is not usable")]
    InvalidSheet {
        /// Sheet width as given, in millimetres.
        w: f64,
        /// Sheet height as given, in millimetres.
        h: f64,
    },
    /// The spread plus its bleed plus the printer's unprintable margin is larger than the
    /// sheet, in landscape as well as in portrait orientation.
    #[error(
        "the card needs {:.1} x {:.1} mm (spread + bleed + printable margin) but the sheet is only {:.1} x {:.1} mm",
        .needed.w, .needed.h, .sheet.w, .sheet.h
    )]
    DoesNotFit {
        /// Area the card needs on the sheet: spread + 2 × (bleed + printable margin).
        needed: Size,
        /// The sheet, in the orientation of the spread (landscape for a spread at least as
        /// wide as it is high) — the only orientation in which it could have fitted.
        sheet: Size,
    },
    /// The card fits, but the cut or fold marks the profile asks for do not: there is less
    /// than [`MIN_MARK_LENGTH_MM`](crate::MIN_MARK_LENGTH_MM) between the place where they
    /// would start and the edge of the slug (or of the printable area of the sheet), so
    /// [`marks`](crate::marks) leaves them out. Only reported by
    /// [`check_marks_fit`](crate::check_marks_fit).
    #[error(
        "no room for cut and fold marks: they need at least {needed_mm:.1} mm beyond the bleed but only {available_mm:.1} mm is available"
    )]
    MarksDoNotFit {
        /// Room between the start of the marks and the limit of the mark area, in
        /// millimetres (never negative).
        available_mm: f64,
        /// The shortest mark that is drawn, in millimetres.
        needed_mm: f64,
    },
}
