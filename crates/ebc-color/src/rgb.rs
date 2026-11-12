//! The 8-bit sRGB colour type used throughout the crate.

use std::fmt;

use crate::lab::Lab;

/// An 8-bit-per-channel sRGB colour.
///
/// Ordering is lexicographic on `(r, g, b)`; it has no perceptual meaning and only exists
/// to give palettes a stable order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Rgb8 {
    /// Red, 0–255.
    pub r: u8,
    /// Green, 0–255.
    pub g: u8,
    /// Blue, 0–255.
    pub b: u8,
}

impl Rgb8 {
    /// sRGB black, `#000000`.
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    /// sRGB white, `#ffffff`.
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };

    /// Creates a colour from its three channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parses `#rrggbb`, `rrggbb`, `#rgb` or `rgb` (case-insensitive, surrounding
    /// whitespace ignored). Returns `None` for anything else.
    ///
    /// ```
    /// use ebc_color::Rgb8;
    /// assert_eq!(Rgb8::from_hex("#777"), Some(Rgb8::new(0x77, 0x77, 0x77)));
    /// assert_eq!(Rgb8::from_hex("1A2b3C"), Some(Rgb8::new(0x1a, 0x2b, 0x3c)));
    /// assert_eq!(Rgb8::from_hex("#12345"), None);
    /// ```
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        let text = text.trim();
        let digits = text.strip_prefix('#').unwrap_or(text).as_bytes();
        let nibble = |b: u8| -> Option<u8> {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            }
        };
        match *digits {
            [r, g, b] => {
                let (r, g, b) = (nibble(r)?, nibble(g)?, nibble(b)?);
                Some(Self::new(r * 17, g * 17, b * 17))
            }
            [r1, r0, g1, g0, b1, b0] => Some(Self::new(
                nibble(r1)? << 4 | nibble(r0)?,
                nibble(g1)? << 4 | nibble(g0)?,
                nibble(b1)? << 4 | nibble(b0)?,
            )),
            _ => None,
        }
    }

    /// Formats the colour as lower-case `#rrggbb`.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.to_string()
    }

    /// The colour in CIELAB (D65). Shorthand for [`Lab::from_rgb8`].
    #[must_use]
    pub fn to_lab(self) -> Lab {
        Lab::from_rgb8(self)
    }

    /// WCAG 2.x relative luminance, `0.0..=1.0`. Shorthand for
    /// [`relative_luminance`](crate::relative_luminance).
    #[must_use]
    pub fn relative_luminance(self) -> f64 {
        crate::contrast::relative_luminance(self)
    }

    /// Packs the colour into `0x00RRGGBB`.
    #[must_use]
    pub(crate) const fn to_key(self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }

    /// Inverse of [`to_key`](Self::to_key); the top byte is ignored.
    #[must_use]
    pub(crate) const fn from_key(key: u32) -> Self {
        Self { r: (key >> 16) as u8, g: (key >> 8) as u8, b: key as u8 }
    }
}

impl fmt::Display for Rgb8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl From<[u8; 3]> for Rgb8 {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb8> for [u8; 3] {
    fn from(c: Rgb8) -> Self {
        [c.r, c.g, c.b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_for_every_channel_value() {
        for v in 0..=255u8 {
            let c = Rgb8::new(v, v.wrapping_mul(7), v.wrapping_add(91));
            assert_eq!(Rgb8::from_hex(&c.to_hex()), Some(c), "{c}");
            assert_eq!(Rgb8::from_hex(&c.to_hex().to_uppercase()), Some(c));
        }
    }

    #[test]
    fn hex_accepts_the_documented_forms_only() {
        assert_eq!(Rgb8::from_hex(" #fff "), Some(Rgb8::WHITE));
        assert_eq!(Rgb8::from_hex("000"), Some(Rgb8::BLACK));
        assert_eq!(Rgb8::from_hex("#0a0B0c"), Some(Rgb8::new(10, 11, 12)));
        for bad in ["", "#", "#ff", "#ffff", "#fffff", "#fffffff", "#ggg", "##fff", "#ff ff", "é€"]
        {
            assert_eq!(Rgb8::from_hex(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn display_is_lower_case_hex() {
        assert_eq!(Rgb8::new(0xAB, 0x0C, 0xEF).to_string(), "#ab0cef");
    }

    #[test]
    fn key_round_trips_and_orders_like_the_tuple() {
        let a = Rgb8::new(1, 200, 3);
        let b = Rgb8::new(2, 0, 0);
        assert_eq!(Rgb8::from_key(a.to_key()), a);
        assert!(a < b && a.to_key() < b.to_key());
        assert_eq!(<[u8; 3]>::from(a), [1, 200, 3]);
        assert_eq!(Rgb8::from([1, 200, 3]), a);
    }
}
