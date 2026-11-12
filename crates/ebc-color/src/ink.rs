//! Ink limiting by grey-component replacement: the closest colour a press can print
//! under a total-ink limit.
//!
//! # Why
//!
//! A CMYK output profile has its own ink limit (the bundled "ISO Coated v2 300 %" builds
//! its darkest colours at about 298 %), but a printer may ask for less: print-on-demand
//! and many digital presses cap total ink at 240 % (IngramSpark's limit, and its
//! recommended rich black C60 M40 Y40 K100), offset on coated stock usually at 280–300 %
//! (`docs/research/03-print-requirements.md` §4.3). Scaling C, M, Y and K down until the
//! sum fits (what [`crate::limit_total_ink`] does) makes dark colours visibly lighter and
//! duller. Printers instead *rebuild* dark colours with more black and less colour ink —
//! grey-component replacement (GCR) or under-colour removal (UCR) — which keeps them dark.
//!
//! # The method ([`CmykProfile::limit_ink`])
//!
//! For a CMYK colour above the limit:
//!
//! 1. its appearance is measured: CIELAB (D50) through the profile's `A2B1` table
//!    (relative colorimetric, the same tables the transform was built from);
//! 2. candidate builds are generated with two parameters: the grey component `g` taken
//!    off each of C, M and Y (from nothing up to the smallest of the three), and the black
//!    `K'` (from the original K up to 100 %, or a lower maximum black if one is given).
//!    If C + M + Y + K' is still above the limit, C, M and Y are scaled down by one common
//!    factor, which keeps their proportions and therefore, roughly, the hue;
//! 3. each candidate is measured through `A2B1` too, and the one with the smallest
//!    **ΔE2000** to the original wins: a 9 × 9 grid over (`g`, `K'`), then a pattern
//!    search around the best point (step halved when nothing improves, 16 rounds).
//!
//! The result never exceeds the limit (to rounding of a few 1e-7) and K never exceeds the
//! maximum black. About 150 table evaluations per colour; colours already within the
//! limit are returned unchanged at no cost. Everything uses `+ − × ÷` and `sqrt` only
//! ([`delta_e2000_lab`] included), so the chosen build is the same natively and in
//! WebAssembly.
//!
//! Images do not run this search per pixel: [`SrgbToCmyk::with_ink_limit`] applies it once
//! to every node of the sampled transform's grid that is above the limit, and a pixel
//! above the limit is then interpolated between ink-limited nodes. Interpolation is a
//! weighted average with non-negative weights, so the pixel cannot exceed the limit
//! either. See [`SrgbToCmyk::with_ink_limit`].
//!
//! # Limits of the method
//!
//! It is a search over a two-parameter family, not a full inversion of the profile: a
//! colour whose best build would shift ink between C, M and Y (rather than remove grey and
//! scale) is approximated. The appearance is that of the profile's tables, i.e. of the
//! characterisation of the printing condition (FOGRA39 for the bundled profile), not of a
//! measured press. Validated against Little CMS (Pillow's `ImageCms`) through the
//! `A2B1` tables: `tests/fixtures/lcms-a2b-fogra39.tsv`.
//!
//! [`SrgbToCmyk`]: crate::SrgbToCmyk
//! [`SrgbToCmyk::with_ink_limit`]: crate::SrgbToCmyk::with_ink_limit

use crate::delta_e::delta_e2000_lab;
use crate::icc::{CmykProfile, RenderingIntent};
use crate::lab::Lab;

/// The highest total ink an [`InkLimit`] can express: four solid inks.
pub const MAX_TOTAL_INK_PERCENT: f32 = 400.0;
/// Slack below which a total counts as within the limit (a few units of `f32` rounding,
/// as a fraction of solid ink: 0.0001 %).
const WITHIN: f64 = 1.0e-6;
/// Grid steps per parameter of the coarse search.
const COARSE_STEPS: u32 = 8;
/// Rounds of the pattern search after the coarse grid.
const REFINE_ROUNDS: u32 = 16;

/// A total area coverage limit, and the most black ink a build may use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InkLimit {
    /// The limit as a fraction of one solid ink (2.4 = 240 %).
    total: f64,
    /// Maximum black, 0–1.
    max_black: f64,
}

impl InkLimit {
    /// A limit of `total_percent` per cent total ink (C + M + Y + K), with black allowed up
    /// to 100 %. `None` for a limit that is not a number or not above 0; limits above
    /// [`MAX_TOTAL_INK_PERCENT`] are clamped to it.
    #[must_use]
    pub fn new(total_percent: f32) -> Option<Self> {
        let total = f64::from(total_percent);
        (total.is_finite() && total > 0.0).then(|| Self {
            total: total.min(f64::from(MAX_TOTAL_INK_PERCENT)) / 100.0,
            max_black: 1.0,
        })
    }

    /// The same limit with black capped at `max_black_percent` (a printer's or a
    /// separation's maximum black). ICC profiles do not record one; values outside
    /// 0–100 % are clamped, NaN is ignored.
    #[must_use]
    pub fn with_max_black(mut self, max_black_percent: f32) -> Self {
        let k = f64::from(max_black_percent);
        if !k.is_nan() {
            self.max_black = (k / 100.0).clamp(0.0, 1.0);
        }
        self
    }

    /// The total ink limit, in per cent.
    #[must_use]
    pub fn total_percent(&self) -> f32 {
        // At most 400: the conversion is exact enough.
        #[allow(clippy::cast_possible_truncation)]
        let percent = (self.total * 100.0) as f32;
        percent
    }

    /// The maximum black, in per cent.
    #[must_use]
    pub fn max_black_percent(&self) -> f32 {
        #[allow(clippy::cast_possible_truncation)]
        let percent = (self.max_black * 100.0) as f32;
        percent
    }

    /// The limit as a fraction of one solid ink (2.4 for 240 %).
    #[must_use]
    pub fn total_fraction(&self) -> f64 {
        self.total
    }

    /// Whether `cmyk` (each 0–1) is within the limit and the maximum black.
    #[must_use]
    pub fn admits(&self, cmyk: [f64; 4]) -> bool {
        total(cmyk) <= self.total + WITHIN && cmyk[3] <= self.max_black + WITHIN
    }
}

/// The outcome of [`CmykProfile::limit_ink`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InkLimited {
    /// The build that is printed, each channel 0–1.
    pub cmyk: [f64; 4],
    /// CIEDE2000 between the original and this build, both measured through the profile's
    /// `A2B1` table; 0 when the original was within the limit.
    pub delta_e: f64,
}

fn total(cmyk: [f64; 4]) -> f64 {
    cmyk[0] + cmyk[1] + cmyk[2] + cmyk[3]
}

fn clean(v: f64) -> f64 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

/// The search space of one colour: see the module documentation.
struct Family {
    cmy: [f64; 3],
    /// Largest grey component (the smallest of C, M, Y).
    grey: f64,
    k_lo: f64,
    k_hi: f64,
    limit: f64,
}

impl Family {
    /// The candidate with grey component `g` removed and black `k`.
    fn build(&self, g: f64, k: f64) -> [f64; 4] {
        let g = g.clamp(0.0, self.grey);
        let k = k.clamp(self.k_lo, self.k_hi);
        let mut cmy = self.cmy.map(|v| (v - g).max(0.0));
        let budget = (self.limit - k).max(0.0);
        let sum = cmy[0] + cmy[1] + cmy[2];
        if sum > budget {
            let scale = if sum > 0.0 { budget / sum } else { 0.0 };
            cmy = cmy.map(|v| v * scale);
        }
        [cmy[0], cmy[1], cmy[2], k]
    }
}

impl CmykProfile {
    /// CIELAB (D50) of a CMYK colour through the relative colorimetric `A2B1` table, as a
    /// [`Lab`] value (for [`delta_e2000_lab`]).
    fn appearance(&self, cmyk: [f64; 4]) -> Lab {
        let [l, a, b] = self.cmyk_to_lab(cmyk, RenderingIntent::RelativeColorimetric);
        Lab::new(l, a, b)
    }

    /// CIEDE2000 between two CMYK colours as this profile prints them (both measured
    /// through the relative colorimetric `A2B1` table).
    #[must_use]
    pub fn delta_e(&self, a: [f64; 4], b: [f64; 4]) -> f64 {
        delta_e2000_lab(self.appearance(a), self.appearance(b))
    }

    /// The build closest to `cmyk` (each 0–1) under `limit`, by grey-component
    /// replacement then a common reduction of C, M and Y, chosen by the smallest ΔE2000
    /// through this profile (see [the module documentation](crate::ink)).
    ///
    /// A colour already within the limit is returned unchanged with `delta_e` 0. Channels
    /// that are not numbers count as 0 and channels are clamped to 0–1 first. If the limit
    /// is below the black of the colour, black itself is reduced.
    #[must_use]
    pub fn limit_ink(&self, cmyk: [f64; 4], limit: InkLimit) -> InkLimited {
        let original = cmyk.map(clean);
        if limit.admits(original) {
            return InkLimited { cmyk: original, delta_e: 0.0 };
        }
        let target = self.appearance(original);
        let [c, m, y, k] = original;
        let k_hi = limit.max_black.min(limit.total);
        let family = Family {
            cmy: [c, m, y],
            grey: c.min(m).min(y),
            k_lo: k.min(k_hi),
            k_hi,
            limit: limit.total,
        };
        let score = |g: f64, k: f64| {
            let build = family.build(g, k);
            (delta_e2000_lab(self.appearance(build), target), build)
        };

        // Coarse grid; ties keep the first (least GCR, least black) candidate.
        let steps = f64::from(COARSE_STEPS);
        let k_span = family.k_hi - family.k_lo;
        let mut best = (f64::INFINITY, 0.0, family.k_lo, family.build(0.0, family.k_lo));
        for i in 0..=COARSE_STEPS {
            let g = family.grey * f64::from(i) / steps;
            for j in 0..=COARSE_STEPS {
                let k = family.k_lo + k_span * f64::from(j) / steps;
                let (de, build) = score(g, k);
                if de < best.0 {
                    best = (de, g, k, build);
                }
            }
        }
        // Pattern search around it.
        let (mut step_g, mut step_k) = (family.grey / steps, k_span / steps);
        for _ in 0..REFINE_ROUNDS {
            let mut moved = false;
            for (dg, dk) in [(step_g, 0.0), (-step_g, 0.0), (0.0, step_k), (0.0, -step_k)] {
                let g = (best.1 + dg).clamp(0.0, family.grey);
                let k = (best.2 + dk).clamp(family.k_lo, family.k_hi);
                let (de, build) = score(g, k);
                if de < best.0 {
                    best = (de, g, k, build);
                    moved = true;
                }
            }
            if !moved {
                step_g /= 2.0;
                step_k /= 2.0;
            }
        }
        let (delta_e, _, _, build) = best;
        InkLimited { cmyk: build, delta_e: if delta_e.is_finite() { delta_e } else { 0.0 } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_are_validated() {
        assert!(InkLimit::new(f32::NAN).is_none());
        assert!(InkLimit::new(0.0).is_none());
        assert!(InkLimit::new(-5.0).is_none());
        assert!(InkLimit::new(f32::INFINITY).is_none());
        let limit = InkLimit::new(900.0).expect("clamped");
        assert_eq!(limit.total_percent(), 400.0);
        let limit = InkLimit::new(240.0).expect("valid");
        assert_eq!(limit.total_percent(), 240.0);
        assert_eq!(limit.max_black_percent(), 100.0);
        assert_eq!(limit.with_max_black(95.0).max_black_percent(), 95.0);
        assert_eq!(limit.with_max_black(150.0).max_black_percent(), 100.0);
        assert_eq!(limit.with_max_black(f32::NAN).max_black_percent(), 100.0);
        assert!(limit.admits([0.6, 0.4, 0.4, 1.0]));
        assert!(!limit.admits([0.7, 0.4, 0.4, 1.0]));
        assert!(!limit.with_max_black(90.0).admits([0.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn family_builds_respect_the_limit() {
        let family = Family { cmy: [0.9, 0.8, 0.7], grey: 0.7, k_lo: 0.6, k_hi: 1.0, limit: 2.4 };
        for i in 0..=10 {
            for j in 0..=10 {
                let build = family.build(0.07 * f64::from(i), 0.6 + 0.04 * f64::from(j));
                assert!(total(build) <= 2.4 + 1e-12, "{build:?}");
                assert!(build.iter().all(|v| (0.0..=1.0).contains(v)), "{build:?}");
            }
        }
        // Out-of-range parameters are clamped, not trusted.
        assert_eq!(family.build(5.0, 7.0)[3], 1.0);
        assert_eq!(family.build(-1.0, -1.0), family.build(0.0, 0.6));
    }
}
