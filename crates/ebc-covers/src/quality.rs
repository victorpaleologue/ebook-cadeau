//! Print-quality rating of a cover image against the size it will be printed at.
//!
//! The rule (see `docs/research/03-print-requirements.md`, §4.2 and check E1, and
//! `docs/research/02-cover-sources.md`, implications 38–39):
//!
//! * resolution is measured on the **placed** image — native pixels divided by the printed
//!   size — never read from the file's metadata;
//! * the cover is placed with **uniform scaling, never cropped** (letterboxed inside the
//!   target box), so one number describes both axes;
//! * **resampling is not print quality**: the rating uses the native pixels, and the
//!   upscale factor needed to reach the target is reported as a separate value. An
//!   upscaled 500 px image is "71 ppi native, upscaled 4.2×", never "300 ppi".

use ebc_scene::{MM_PER_INCH, Size};

use crate::sniff::ImageInfo;

/// Resolution thresholds, in pixels per inch at the printed size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// At or above: [`Verdict::Excellent`].
    pub target_ppi: f64,
    /// At or above (and below the target): [`Verdict::Acceptable`]. Below: a warning.
    pub warn_ppi: f64,
    /// At or above (and below the warning level): [`Verdict::Low`]. Below:
    /// [`Verdict::Insufficient`] — a professional order is refused.
    pub min_ppi: f64,
}

impl Thresholds {
    /// Professional print: target 300, warning below 250, error below 150 ppi
    /// (`docs/research/03-print-requirements.md`: printers ask for 300 ppi; the two lower
    /// levels are house rules).
    pub const PROFESSIONAL: Self = Self { target_ppi: 300.0, warn_ppi: 250.0, min_ppi: 150.0 };

    /// Home print: target 200, warning below 150, error below 100 ppi (house rule: home
    /// devices and viewing expectations are lower).
    pub const HOME: Self = Self { target_ppi: 200.0, warn_ppi: 150.0, min_ppi: 100.0 };
}

impl Default for Thresholds {
    /// [`Thresholds::PROFESSIONAL`].
    fn default() -> Self {
        Self::PROFESSIONAL
    }
}

/// How a native resolution compares with the thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Verdict {
    /// Below the minimum: not printable professionally; home print only with a visible
    /// quality label.
    Insufficient,
    /// Between the minimum and the warning level: printable, with a warning.
    Low,
    /// Between the warning level and the target: fine in practice.
    Acceptable,
    /// At or above the target resolution.
    Excellent,
}

impl Verdict {
    /// `true` unless the image is below the minimum resolution.
    #[must_use]
    pub const fn is_printable(self) -> bool {
        !matches!(self, Self::Insufficient)
    }

    /// `true` when the user should see a quality warning ([`Verdict::Low`] or worse).
    #[must_use]
    pub const fn needs_warning(self) -> bool {
        matches!(self, Self::Low | Self::Insufficient)
    }
}

/// The result of [`print_quality`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quality {
    /// Native pixels per inch at the placed size. `0.0` when the input is degenerate.
    pub effective_ppi: f64,
    /// Rating of [`Quality::effective_ppi`] against the thresholds.
    pub verdict: Verdict,
    /// Linear factor by which the image must be resampled to reach the target
    /// resolution; `1.0` when it already does. Infinite when the input is degenerate.
    /// Report it next to the native resolution, never instead of it.
    pub upscale_factor: f64,
    /// The size the image occupies inside the target box (uniform scaling, letterboxed).
    pub placed_mm: Size,
    /// Pixel size to resample to for the target resolution: the native size when no
    /// upscaling is needed, otherwise the native size times
    /// [`Quality::upscale_factor`], rounded up. `(0, 0)` when the input is degenerate.
    pub target_px: (u32, u32),
    /// The thresholds the verdict was computed against.
    pub thresholds: Thresholds,
}

/// Tolerance, in ppi, when comparing against a threshold: a 1299 px image on a 110 mm
/// panel is 299.95 ppi and counts as 300.
const PPI_TOLERANCE: f64 = 0.5;

/// Rates `info` (native pixel size, **as displayed** — see
/// [`crate::sniff::Details::oriented`]) for printing inside a box of `target_mm`, against
/// the professional thresholds (300 / 250 / 150 ppi).
///
/// ```
/// use ebc_covers::{print_quality, Verdict};
/// use ebc_covers::sniff::{Format, ImageInfo};
/// use ebc_scene::Size;
///
/// // An Open Library "-L" cover (500 px tall at most) on a 110 x 178 mm poche front.
/// let q = print_quality(&ImageInfo::new(Format::Jpeg, 309, 500), Size::new(110.0, 178.0));
/// assert_eq!(q.verdict, Verdict::Insufficient);
/// assert!((q.effective_ppi - 71.3).abs() < 0.5);
/// assert!((q.upscale_factor - 4.2).abs() < 0.05);
/// ```
#[must_use]
pub fn print_quality(info: &ImageInfo, target_mm: Size) -> Quality {
    print_quality_with(info, target_mm, &Thresholds::PROFESSIONAL)
}

/// Like [`print_quality`], against the given thresholds (e.g. [`Thresholds::HOME`]).
#[must_use]
pub fn print_quality_with(info: &ImageInfo, target_mm: Size, thresholds: &Thresholds) -> Quality {
    let (w_px, h_px) = (f64::from(info.width), f64::from(info.height));
    let usable = info.width > 0
        && info.height > 0
        && target_mm.w.is_finite()
        && target_mm.h.is_finite()
        && target_mm.w > 0.0
        && target_mm.h > 0.0;
    if !usable {
        return Quality {
            effective_ppi: 0.0,
            verdict: Verdict::Insufficient,
            upscale_factor: f64::INFINITY,
            placed_mm: Size::new(0.0, 0.0),
            target_px: (0, 0),
            thresholds: *thresholds,
        };
    }
    // Uniform scale that fits the image inside the box: millimetres per pixel.
    let mm_per_px = (target_mm.w / w_px).min(target_mm.h / h_px);
    let placed_mm = Size::new(w_px * mm_per_px, h_px * mm_per_px);
    let effective_ppi = MM_PER_INCH / mm_per_px;
    let verdict = if effective_ppi + PPI_TOLERANCE >= thresholds.target_ppi {
        Verdict::Excellent
    } else if effective_ppi + PPI_TOLERANCE >= thresholds.warn_ppi {
        Verdict::Acceptable
    } else if effective_ppi + PPI_TOLERANCE >= thresholds.min_ppi {
        Verdict::Low
    } else {
        Verdict::Insufficient
    };
    let upscale_factor = if verdict == Verdict::Excellent || !thresholds.target_ppi.is_finite() {
        1.0
    } else {
        (thresholds.target_ppi / effective_ppi).max(1.0)
    };
    let scaled = |px: f64| {
        let v = (px * upscale_factor).ceil();
        // `as` saturates; the cap keeps the result a sane image dimension.
        if v >= f64::from(u32::MAX) { u32::MAX } else { v as u32 }
    };
    let target_px =
        if upscale_factor > 1.0 { (scaled(w_px), scaled(h_px)) } else { (info.width, info.height) };
    Quality {
        effective_ppi,
        verdict,
        upscale_factor,
        placed_mm,
        target_px,
        thresholds: *thresholds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sniff::Format;

    const POCHE: Size = Size::new(110.0, 178.0);

    fn rate(w: u32, h: u32, target: Size) -> Quality {
        print_quality(&ImageInfo::new(Format::Jpeg, w, h), target)
    }

    #[test]
    fn research_table_sizes_hit_their_thresholds() {
        // docs/research/02 §3.1: 110 x 178 mm at 300 / 250 / 150 ppi.
        assert_eq!(rate(1299, 2102, POCHE).verdict, Verdict::Excellent);
        assert_eq!(rate(1083, 1752, POCHE).verdict, Verdict::Acceptable);
        assert_eq!(rate(650, 1051, POCHE).verdict, Verdict::Low);
        assert_eq!(rate(640, 1040, POCHE).verdict, Verdict::Insufficient);
        // One pixel row short of each boundary (beyond the tolerance) falls a class.
        assert_eq!(rate(1290, 2090, POCHE).verdict, Verdict::Acceptable);
        assert_eq!(rate(1070, 1730, POCHE).verdict, Verdict::Low);
    }

    #[test]
    fn source_table_ppi_values() {
        // docs/research/02 §3.2: long side on a 178 mm front.
        let cases = [(500u32, 71.0), (800, 114.0), (1200, 171.0), (1224, 175.0), (2560, 365.0)];
        for (long_side, expected) in cases {
            // Same aspect ratio as the panel, so the height binds exactly.
            let w = (f64::from(long_side) * 110.0 / 178.0).round() as u32;
            let q = rate(w, long_side, POCHE);
            assert!(
                (q.effective_ppi - expected).abs() < 1.0,
                "{long_side}px -> {} ppi, expected ~{expected}",
                q.effective_ppi
            );
        }
        // The Kobo store average (800 x 1224) is wider than the poche panel, so the width
        // binds: 800 px on 110 mm is 185 ppi, and reaching 300 ppi takes about 1.6x.
        let kobo = rate(800, 1224, POCHE);
        assert_eq!(kobo.verdict, Verdict::Low);
        assert!((kobo.effective_ppi - 184.7).abs() < 0.1, "{}", kobo.effective_ppi);
        assert!((kobo.upscale_factor - 1.62).abs() < 0.03, "{}", kobo.upscale_factor);
    }

    #[test]
    fn letterboxing_uses_the_binding_dimension() {
        // A square image in a portrait box is limited by the box width.
        let q = rate(1000, 1000, POCHE);
        assert!((q.placed_mm.w - 110.0).abs() < 1e-9);
        assert!((q.placed_mm.h - 110.0).abs() < 1e-9);
        assert!((q.effective_ppi - 1000.0 / (110.0 / 25.4)).abs() < 1e-9);
        // A very tall image is limited by the box height.
        let q = rate(100, 1780, POCHE);
        assert!((q.placed_mm.h - 178.0).abs() < 1e-9);
        assert!((q.placed_mm.w - 10.0).abs() < 1e-9);
        assert!((q.effective_ppi - 254.0).abs() < 1e-9);
    }

    #[test]
    fn upscale_factor_and_target_pixels() {
        let q = rate(325, 500, Size::new(115.7, 178.0));
        assert_eq!(q.verdict, Verdict::Insufficient);
        assert!((q.upscale_factor - 300.0 / q.effective_ppi).abs() < 1e-12);
        assert_eq!(q.target_px.0, (325.0 * q.upscale_factor).ceil() as u32);
        assert_eq!(q.target_px.1, (500.0 * q.upscale_factor).ceil() as u32);
        // Upscaled pixels reach the target at the placed size.
        let ppi_after = f64::from(q.target_px.1) / (q.placed_mm.h / 25.4);
        assert!((300.0..301.0).contains(&ppi_after), "{ppi_after}");

        let big = rate(3000, 4800, POCHE);
        assert_eq!(big.verdict, Verdict::Excellent);
        assert_eq!(big.upscale_factor, 1.0);
        assert_eq!(big.target_px, (3000, 4800));
    }

    #[test]
    fn home_thresholds_are_more_tolerant() {
        let info = ImageInfo::new(Format::Jpeg, 800, 1224);
        assert_eq!(
            print_quality_with(&info, POCHE, &Thresholds::HOME).verdict,
            Verdict::Acceptable
        );
        assert_eq!(print_quality(&info, POCHE).verdict, Verdict::Low);
        assert_eq!(Thresholds::default(), Thresholds::PROFESSIONAL);
    }

    #[test]
    fn degenerate_input_is_insufficient_not_a_panic() {
        for target in [
            Size::new(0.0, 178.0),
            Size::new(110.0, -1.0),
            Size::new(f64::NAN, 178.0),
            Size::new(110.0, f64::INFINITY),
        ] {
            let q = rate(1000, 1500, target);
            assert_eq!(q.verdict, Verdict::Insufficient);
            assert_eq!(q.effective_ppi, 0.0);
            assert!(q.upscale_factor.is_infinite());
            assert_eq!(q.target_px, (0, 0));
        }
        let q = print_quality(&ImageInfo::new(Format::Png, 0, 10), POCHE);
        assert_eq!(q.verdict, Verdict::Insufficient);
        // Absurdly small target with a huge image must not overflow.
        let q = rate(u32::MAX, u32::MAX, Size::new(1e-300, 1e-300));
        assert_eq!(q.verdict, Verdict::Excellent);
    }

    #[test]
    fn verdict_helpers_and_order() {
        assert!(Verdict::Excellent > Verdict::Acceptable);
        assert!(Verdict::Acceptable > Verdict::Low);
        assert!(Verdict::Low > Verdict::Insufficient);
        assert!(Verdict::Low.is_printable() && Verdict::Low.needs_warning());
        assert!(!Verdict::Insufficient.is_printable());
        assert!(!Verdict::Acceptable.needs_warning());
    }
}
