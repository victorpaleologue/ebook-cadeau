//! ISBN-10/ISBN-13/EAN parsing, validation, conversion and hyphenation.
//!
//! The canonical representation used across eBook Cadeau is [`Isbn13`]: thirteen ASCII
//! digits starting with `978` or `979`, with a valid EAN-13 check digit.
//!
//! This file currently holds the **minimal contract** other crates build against
//! (`Isbn13::parse`, `as_str`, `Display`, `FromStr`). See `docs/architecture.md` for the
//! full API this crate must grow into (ISBN-10, hyphenation, extraction from free text).

#![forbid(unsafe_code)]

use std::fmt;
use std::str::FromStr;

/// Why a string is not a valid ISBN.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IsbnError {
    /// The input has the wrong number of digits once separators are removed.
    #[error("expected 10 or 13 digits, found {0}")]
    Length(usize),
    /// The input contains a character that cannot appear in an ISBN.
    #[error("invalid character {0:?} in ISBN")]
    InvalidChar(char),
    /// A 13-digit number that does not start with 978 or 979.
    #[error("not a Bookland EAN: ISBN-13 must start with 978 or 979")]
    Prefix,
    /// The check digit does not match.
    #[error("wrong check digit: expected {expected}, found {found}")]
    CheckDigit {
        /// The check character computed from the other digits.
        expected: char,
        /// The check character found in the input.
        found: char,
    },
    /// A 979-prefixed ISBN-13 has no ISBN-10 equivalent.
    #[error("ISBN-13 with prefix 979 has no ISBN-10 form")]
    NoIsbn10,
}

/// A validated ISBN-13 (thirteen digits, prefix 978 or 979, correct check digit).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Isbn13([u8; 13]);

impl Isbn13 {
    /// Parses an ISBN-13 **or an ISBN-10** (which is converted), ignoring hyphens, spaces
    /// and an optional leading `ISBN`, `ISBN-13:` or `ISBN-10:` label.
    ///
    /// # Errors
    /// Returns an [`IsbnError`] describing the first problem found.
    pub fn parse(input: &str) -> Result<Self, IsbnError> {
        let mut s = input.trim();
        for label in ["ISBN-13", "ISBN-10", "ISBN13", "ISBN10", "ISBN"] {
            if s.len() >= label.len()
                && s.is_char_boundary(label.len())
                && s[..label.len()].eq_ignore_ascii_case(label)
            {
                s = s[label.len()..].trim_start_matches([':', ' ', '\u{a0}']);
                break;
            }
        }
        let mut chars: Vec<char> = Vec::with_capacity(13);
        for c in s.chars() {
            match c {
                '0'..='9' => chars.push(c),
                'x' | 'X' => chars.push('X'),
                '-' | ' ' | '\u{a0}' | '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' => {}
                other => return Err(IsbnError::InvalidChar(other)),
            }
        }
        match chars.len() {
            13 => Self::from_13(&chars),
            10 => Self::from_10(&chars),
            n => Err(IsbnError::Length(n)),
        }
    }

    fn from_13(chars: &[char]) -> Result<Self, IsbnError> {
        let mut d = [0u8; 13];
        for (slot, c) in d.iter_mut().zip(chars) {
            *slot = c.to_digit(10).ok_or(IsbnError::InvalidChar(*c))? as u8;
        }
        if !(d[0] == 9 && d[1] == 7 && (d[2] == 8 || d[2] == 9)) {
            return Err(IsbnError::Prefix);
        }
        let expected = ean13_check(&d[..12]);
        if expected != d[12] {
            return Err(IsbnError::CheckDigit {
                expected: char::from(b'0' + expected),
                found: char::from(b'0' + d[12]),
            });
        }
        Ok(Self(d))
    }

    fn from_10(chars: &[char]) -> Result<Self, IsbnError> {
        let mut sum = 0u32;
        let mut body = [0u8; 9];
        for (i, c) in chars.iter().enumerate() {
            let v = match (i, *c) {
                (9, 'X') => 10,
                (_, 'X') => return Err(IsbnError::InvalidChar('X')),
                (_, c) => c.to_digit(10).ok_or(IsbnError::InvalidChar(c))?,
            };
            if i < 9 {
                body[i] = v as u8;
            }
            sum += v * (10 - i as u32);
        }
        if !sum.is_multiple_of(11) {
            let partial: u32 =
                body.iter().enumerate().map(|(i, d)| u32::from(*d) * (10 - i as u32)).sum();
            let check = (11 - partial % 11) % 11;
            let expected = if check == 10 { 'X' } else { char::from(b'0' + check as u8) };
            return Err(IsbnError::CheckDigit { expected, found: chars[9] });
        }
        let mut d = [0u8; 13];
        d[..3].copy_from_slice(&[9, 7, 8]);
        d[3..12].copy_from_slice(&body);
        d[12] = ean13_check(&d[..12]);
        Ok(Self(d))
    }

    /// The thirteen digits as numbers `0..=9`.
    #[must_use]
    pub fn digits(&self) -> [u8; 13] {
        self.0
    }

    /// The thirteen digits as a plain string without separators.
    #[must_use]
    pub fn to_plain(&self) -> String {
        self.0.iter().map(|d| char::from(b'0' + d)).collect()
    }
}

/// Computes the EAN-13 check digit for twelve leading digits.
fn ean13_check(first12: &[u8]) -> u8 {
    let sum: u32 = first12
        .iter()
        .enumerate()
        .map(|(i, d)| u32::from(*d) * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    ((10 - sum % 10) % 10) as u8
}

impl fmt::Display for Isbn13 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_plain())
    }
}

impl fmt::Debug for Isbn13 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Isbn13({})", self.to_plain())
    }
}

impl FromStr for Isbn13 {
    type Err = IsbnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Isbn13 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_plain())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Isbn13 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_isbn13_with_and_without_separators() {
        let a = Isbn13::parse("9782070360024").expect("valid");
        let b = Isbn13::parse("ISBN 978-2-07-036002-4").expect("valid");
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "9782070360024");
    }

    #[test]
    fn converts_isbn10() {
        assert_eq!(Isbn13::parse("2-07-036002-4").expect("valid").to_string(), "9782070360024");
        assert_eq!(Isbn13::parse("080442957X").expect("valid").to_string(), "9780804429573");
        assert_eq!(
            Isbn13::parse("isbn-10: 0-8044-2957-x").expect("valid").to_string(),
            "9780804429573"
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(
            Isbn13::parse("9782070360025"),
            Err(IsbnError::CheckDigit { expected: '4', found: '5' })
        );
        assert_eq!(Isbn13::parse("4006381333931"), Err(IsbnError::Prefix));
        assert_eq!(Isbn13::parse("12345"), Err(IsbnError::Length(5)));
        assert_eq!(Isbn13::parse("97820703600é4"), Err(IsbnError::InvalidChar('é')));
        assert_eq!(
            Isbn13::parse("2070360025"),
            Err(IsbnError::CheckDigit { expected: '4', found: '5' })
        );
        assert_eq!(Isbn13::parse("X070360024"), Err(IsbnError::InvalidChar('X')));
        assert_eq!(Isbn13::parse(""), Err(IsbnError::Length(0)));
    }
}
