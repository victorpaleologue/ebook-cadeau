//! Paint colours for print.

/// A paint colour.
///
/// Print-critical elements (text, QR codes, marks) should use [`Color::Cmyk`] or
/// [`Color::Gray`] so that black stays on the black plate only. [`Color::Rgb`] is
/// interpreted as sRGB and left to the printer's colour management.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    /// Process colour, each component in `0.0..=1.0` (1.0 = 100 % ink).
    Cmyk {
        /// Cyan.
        c: f32,
        /// Magenta.
        m: f32,
        /// Yellow.
        y: f32,
        /// Black (key).
        k: f32,
    },
    /// sRGB colour, each component in `0.0..=1.0`.
    Rgb {
        /// Red.
        r: f32,
        /// Green.
        g: f32,
        /// Blue.
        b: f32,
    },
    /// Grey level in `0.0..=1.0` where 0.0 is black and 1.0 is white (PDF `DeviceGray`).
    Gray(f32),
    /// The registration colour: prints at 100 % on **every** separation.
    /// Only for printer's marks — never for artwork.
    Registration,
}

impl Color {
    /// Pure black on the black plate only (C0 M0 Y0 K100). Use for text and QR codes.
    pub const BLACK: Self = Self::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 1.0 };
    /// Paper white (no ink).
    pub const WHITE: Self = Self::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 0.0 };

    /// Creates a CMYK colour, clamping every component to `0.0..=1.0`.
    #[must_use]
    pub fn cmyk(c: f32, m: f32, y: f32, k: f32) -> Self {
        Self::Cmyk { c: clamp01(c), m: clamp01(m), y: clamp01(y), k: clamp01(k) }
    }

    /// Creates an sRGB colour from floating-point components, clamped to `0.0..=1.0`.
    #[must_use]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::Rgb { r: clamp01(r), g: clamp01(g), b: clamp01(b) }
    }

    /// Creates an sRGB colour from 8-bit components.
    #[must_use]
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb { r: f32::from(r) / 255.0, g: f32::from(g) / 255.0, b: f32::from(b) / 255.0 }
    }

    /// Creates a grey level, clamped to `0.0..=1.0` (0.0 = black).
    #[must_use]
    pub fn gray(level: f32) -> Self {
        Self::Gray(clamp01(level))
    }

    /// Total ink coverage in percent (0–400) for CMYK colours and the registration colour;
    /// `None` for RGB and grey, whose separations are decided by the printer.
    #[must_use]
    pub fn total_ink_percent(&self) -> Option<f32> {
        match *self {
            Self::Cmyk { c, m, y, k } => Some((c + m + y + k) * 100.0),
            Self::Registration => Some(400.0),
            Self::Rgb { .. } | Self::Gray(_) => None,
        }
    }

    /// A device-independent-ish sRGB approximation of this colour, for on-screen previews
    /// and contrast estimates only. Not a colour-managed conversion.
    #[must_use]
    pub fn to_srgb_approx(&self) -> [f32; 3] {
        match *self {
            Self::Rgb { r, g, b } => [r, g, b],
            Self::Gray(v) => [v, v, v],
            Self::Cmyk { c, m, y, k } => {
                [(1.0 - c) * (1.0 - k), (1.0 - m) * (1.0 - k), (1.0 - y) * (1.0 - k)]
            }
            Self::Registration => [0.0, 0.0, 0.0],
        }
    }
}

fn clamp01(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_clamp() {
        assert_eq!(
            Color::cmyk(2.0, -1.0, 0.5, f32::NAN),
            Color::Cmyk { c: 1.0, m: 0.0, y: 0.5, k: 0.0 }
        );
        assert_eq!(Color::rgb8(255, 0, 0), Color::Rgb { r: 1.0, g: 0.0, b: 0.0 });
        assert_eq!(Color::gray(7.0), Color::Gray(1.0));
    }

    #[test]
    fn ink_coverage() {
        assert_eq!(Color::BLACK.total_ink_percent(), Some(100.0));
        assert_eq!(Color::WHITE.total_ink_percent(), Some(0.0));
        assert_eq!(Color::Registration.total_ink_percent(), Some(400.0));
        assert_eq!(Color::rgb8(1, 2, 3).total_ink_percent(), None);
    }

    #[test]
    fn srgb_approximation() {
        assert_eq!(Color::BLACK.to_srgb_approx(), [0.0, 0.0, 0.0]);
        assert_eq!(Color::WHITE.to_srgb_approx(), [1.0, 1.0, 1.0]);
        assert_eq!(Color::gray(0.5).to_srgb_approx(), [0.5, 0.5, 0.5]);
    }
}
