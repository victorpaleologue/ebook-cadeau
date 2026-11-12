//! ISBN-10/ISBN-13/EAN parsing, validation, conversion and hyphenation.
//!
//! The canonical representation used across eBook Cadeau is [`Isbn13`]: thirteen digits
//! starting with `978` or `979`, with a valid EAN-13 check digit. [`Isbn10`] exists for the
//! places that still want the legacy form (old catalogue records, some store URLs).
//!
//! ```
//! use ebc_isbn::{Isbn13, extract_all};
//!
//! let isbn = Isbn13::parse("ISBN 2-07-036002-4")?;            // ISBN-10 input is converted
//! assert_eq!(isbn.to_string(), "9782070360024");
//! assert_eq!(isbn.hyphenated(), "978-2-07-036002-4");
//! assert_eq!(isbn.to_isbn10()?.to_string(), "2070360024");
//! assert_eq!(isbn.registration_group().map(|g| g.name), Some("French language"));
//!
//! let found = extract_all("EAN 9782070360024, also sold as ISBN-10: 0-306-40615-2.");
//! assert_eq!(found.len(), 2);
//! # Ok::<(), ebc_isbn::IsbnError>(())
//! ```
//!
//! # Hyphenation
//!
//! Splitting an ISBN into its elements needs the range table published by the
//! International ISBN Agency. A snapshot is embedded (see [`range_data_date`]); ISBNs in
//! ranges assigned after that date are returned as plain digits by
//! [`Isbn13::hyphenated`], never mis-hyphenated.
//!
//! # Robustness
//!
//! Nothing in this crate panics on untrusted input, and parsing never allocates more than
//! a few bytes regardless of the input length.

#![forbid(unsafe_code)]

mod extract;
mod ranges;
mod ranges_data;

use std::fmt;
use std::str::FromStr;

pub use extract::extract_all;
pub use ranges::RegistrationGroup;

/// Why a string is not a valid ISBN.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
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

/// Date (ISO 8601) of the embedded ISBN range table snapshot.
///
/// Registrant ranges assigned by the International ISBN Agency after this date are unknown
/// to [`Isbn13::hyphenated`] and [`Isbn13::parts`].
#[must_use]
pub fn range_data_date() -> &'static str {
    ranges_data::RANGE_DATA_DATE
}

/// Where the embedded ISBN range table snapshot was taken from (name and version).
#[must_use]
pub fn range_data_source() -> &'static str {
    ranges_data::RANGE_DATA_SOURCE
}

/// Computes the EAN-13 (and therefore ISBN-13) check digit for twelve leading digits.
///
/// Each element is a digit **value** `0..=9`, as returned by [`Isbn13::digits`]; ASCII
/// digits `b'0'..=b'9'` are accepted too. Any other byte is reduced modulo 10 so that the
/// function is total. The result is in `0..=9`.
///
/// ```
/// assert_eq!(ebc_isbn::ean13_check_digit(&[9, 7, 8, 2, 0, 7, 0, 3, 6, 0, 0, 2]), 4);
/// assert_eq!(ebc_isbn::ean13_check_digit(b"978030640615"), 7);
/// ```
#[must_use]
pub fn ean13_check_digit(first12: &[u8; 12]) -> u8 {
    let sum: u32 = first12
        .iter()
        .enumerate()
        .map(|(i, d)| u32::from(digit_value(*d)) * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    ((10 - sum % 10) % 10) as u8
}

/// Computes the ISBN-10 check character (`'0'..='9'` or `'X'`) for nine leading digits.
///
/// Input conventions are those of [`ean13_check_digit`].
///
/// ```
/// assert_eq!(ebc_isbn::isbn10_check_char(&[0, 8, 0, 4, 4, 2, 9, 5, 7]), 'X');
/// assert_eq!(ebc_isbn::isbn10_check_char(b"207036002"), '4');
/// ```
#[must_use]
pub fn isbn10_check_char(first9: &[u8; 9]) -> char {
    check10_char(isbn10_check_value(first9))
}

/// Normalises a digit given either as a value or as an ASCII digit.
fn digit_value(byte: u8) -> u8 {
    if byte.is_ascii_digit() { byte - b'0' } else { byte % 10 }
}

/// The ISBN-10 check value in `0..=10` (10 is written `X`).
fn isbn10_check_value(first9: &[u8; 9]) -> u8 {
    let sum: u32 =
        first9.iter().enumerate().map(|(i, d)| u32::from(digit_value(*d)) * (10 - i as u32)).sum();
    ((11 - sum % 11) % 11) as u8
}

/// Renders an ISBN-10 check value as its character.
fn check10_char(value: u8) -> char {
    if value >= 10 { 'X' } else { char::from(b'0' + value) }
}

/// Characters that may separate the elements of a written ISBN: ASCII hyphen and space,
/// their typographic variants, and invisible formatting characters left by copy-paste.
const fn is_separator(c: char) -> bool {
    matches!(
        c,
        '-' | ' '
            | '\u{a0}'              // no-break space
            | '\u{2007}'            // figure space
            | '\u{2009}'            // thin space
            | '\u{202f}'            // narrow no-break space
            | '\u{2010}'..='\u{2013}' // hyphen, non-breaking hyphen, figure dash, en dash
            | '\u{2212}'            // minus sign
            | '\u{200b}'..='\u{200d}' // zero-width space / non-joiner / joiner
            | '\u{2060}'            // word joiner
            | '\u{feff}' // byte-order mark / zero-width no-break space
    )
}

/// A syntactically valid, checksum-verified ISBN in either form.
enum Parsed {
    /// Ten characters; the last value may be 10 (`X`).
    Ten([u8; 10]),
    /// Thirteen digits.
    Thirteen([u8; 13]),
}

/// Strips an optional `ISBN`, `ISBN-13:`, `EAN`, `urn:isbn:` … label.
///
/// Returns the remainder and, when a *numbered* label (`ISBN-10`, `ISBN13` …) was removed,
/// the alternative remainder obtained by removing only `ISBN`: `"ISBN-1334567890"` is the
/// label `ISBN` followed by an ISBN-10 that happens to start with `13`.
fn strip_label(s: &str) -> (&str, Option<&str>) {
    const LABEL_FILLER: [char; 6] = [':', ' ', '\t', '.', '\u{a0}', '\u{202f}'];
    fn strip<'a>(s: &'a str, label: &str) -> Option<&'a str> {
        (s.len() >= label.len()
            && s.is_char_boundary(label.len())
            && s[..label.len()].eq_ignore_ascii_case(label))
        .then(|| s[label.len()..].trim_start_matches(LABEL_FILLER))
    }
    let s = strip(s, "urn:isbn:").unwrap_or(s);
    for label in ["ISBN-13", "ISBN-10", "ISBN13", "ISBN10", "EAN-13", "EAN13"] {
        if let Some(rest) = strip(s, label) {
            let unnumbered = label.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-');
            return (rest, strip(s, unnumbered));
        }
    }
    for label in ["ISBN", "EAN"] {
        if let Some(rest) = strip(s, label) {
            return (rest, None);
        }
    }
    (s, None)
}

/// Parses an unlabelled ISBN-10 or ISBN-13, ignoring separators.
fn parse_unlabelled(s: &str) -> Result<Parsed, IsbnError> {
    // ASCII digits or `X`; characters beyond the thirteenth are only counted.
    let mut chars = [0u8; 13];
    let mut len = 0usize;
    for c in s.chars() {
        let byte = match c {
            '0'..='9' => c as u8,
            'x' | 'X' => b'X',
            c if is_separator(c) => continue,
            other => return Err(IsbnError::InvalidChar(other)),
        };
        if let Some(slot) = chars.get_mut(len) {
            *slot = byte;
        }
        len = len.saturating_add(1);
    }
    match len {
        13 => validate13(&chars).map(Parsed::Thirteen),
        10 => validate10(&chars).map(Parsed::Ten),
        n => Err(IsbnError::Length(n)),
    }
}

/// Parses a possibly labelled ISBN-10 or ISBN-13.
fn parse_any(input: &str) -> Result<Parsed, IsbnError> {
    let (rest, alternative) = strip_label(input.trim());
    match parse_unlabelled(rest) {
        Ok(parsed) => Ok(parsed),
        Err(error) => alternative.and_then(|alt| parse_unlabelled(alt).ok()).ok_or(error),
    }
}

/// Validates thirteen characters (ASCII digits or `X`) as an ISBN-13.
fn validate13(chars: &[u8; 13]) -> Result<[u8; 13], IsbnError> {
    let mut d = [0u8; 13];
    for (slot, c) in d.iter_mut().zip(chars) {
        if !c.is_ascii_digit() {
            return Err(IsbnError::InvalidChar(char::from(*c)));
        }
        *slot = c - b'0';
    }
    if !(d[0] == 9 && d[1] == 7 && (d[2] == 8 || d[2] == 9)) {
        return Err(IsbnError::Prefix);
    }
    let mut first12 = [0u8; 12];
    first12.copy_from_slice(&d[..12]);
    let expected = ean13_check_digit(&first12);
    if expected != d[12] {
        return Err(IsbnError::CheckDigit {
            expected: char::from(b'0' + expected),
            found: char::from(b'0' + d[12]),
        });
    }
    Ok(d)
}

/// Validates the first ten characters (ASCII digits or `X`) as an ISBN-10.
fn validate10(chars: &[u8; 13]) -> Result<[u8; 10], IsbnError> {
    let mut d = [0u8; 10];
    for (i, (slot, c)) in d.iter_mut().zip(chars).enumerate() {
        *slot = match (i, *c) {
            (9, b'X') => 10,
            (_, b'0'..=b'9') => c - b'0',
            (_, other) => return Err(IsbnError::InvalidChar(char::from(other))),
        };
    }
    let mut first9 = [0u8; 9];
    first9.copy_from_slice(&d[..9]);
    let expected = isbn10_check_value(&first9);
    if expected != d[9] {
        return Err(IsbnError::CheckDigit {
            expected: check10_char(expected),
            found: check10_char(d[9]),
        });
    }
    Ok(d)
}

/// Converts validated ISBN-10 digits to ISBN-13 digits (prefix 978, new check digit).
fn ten_to_thirteen(ten: &[u8; 10]) -> [u8; 13] {
    let mut d = [0u8; 13];
    d[..3].copy_from_slice(&[9, 7, 8]);
    d[3..12].copy_from_slice(&ten[..9]);
    let mut first12 = [0u8; 12];
    first12.copy_from_slice(&d[..12]);
    d[12] = ean13_check_digit(&first12);
    d
}

/// A validated ISBN-13 (thirteen digits, prefix 978 or 979, correct check digit).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Isbn13([u8; 13]);

/// The elements of an ISBN-13, as separated by hyphens in its printed form.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IsbnParts {
    /// GS1 prefix: `978` or `979`.
    pub gs1_prefix: u16,
    /// Registration group element, e.g. `"2"` (French language) or `"10"` (France, 979).
    pub group: String,
    /// Registrant (publisher) element, e.g. `"07"` for Gallimard.
    pub registrant: String,
    /// Publication element.
    pub publication: String,
    /// Check digit.
    pub check_digit: char,
}

impl Isbn13 {
    /// Parses an ISBN-13 **or an ISBN-10** (which is converted), ignoring hyphens, spaces
    /// and an optional leading `ISBN`, `ISBN-13:`, `ISBN-10:`, `EAN` or `urn:isbn:` label.
    ///
    /// # Errors
    /// Returns an [`IsbnError`] describing the first problem found.
    pub fn parse(input: &str) -> Result<Self, IsbnError> {
        Ok(match parse_any(input)? {
            Parsed::Thirteen(d) => Self(d),
            Parsed::Ten(d) => Self(ten_to_thirteen(&d)),
        })
    }

    /// Builds an ISBN-13 from thirteen already validated digit values.
    pub(crate) const fn from_valid_digits(digits: [u8; 13]) -> Self {
        Self(digits)
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

    /// The GS1 prefix: `978` or `979`.
    #[must_use]
    pub fn gs1_prefix(&self) -> u16 {
        if self.0[2] == 9 { 979 } else { 978 }
    }

    /// The check digit, `0..=9`.
    #[must_use]
    pub fn check_digit(&self) -> u8 {
        self.0[12]
    }

    /// The equivalent ISBN-10.
    ///
    /// # Errors
    /// [`IsbnError::NoIsbn10`] for 979-prefixed ISBNs, which have no ISBN-10 form.
    pub fn to_isbn10(&self) -> Result<Isbn10, IsbnError> {
        if self.gs1_prefix() != 978 {
            return Err(IsbnError::NoIsbn10);
        }
        let mut first9 = [0u8; 9];
        first9.copy_from_slice(&self.0[3..12]);
        let mut ten = [0u8; 10];
        ten[..9].copy_from_slice(&first9);
        ten[9] = isbn10_check_value(&first9);
        Ok(Isbn10(ten))
    }

    /// The registration group (language area, country or territory) of this ISBN, or
    /// `None` if its prefix is not in the embedded range table.
    #[must_use]
    pub fn registration_group(&self) -> Option<RegistrationGroup> {
        ranges::find_group(&self.0).map(|g| RegistrationGroup { prefix: g.prefix, name: g.name })
    }

    /// Splits the ISBN into its elements, or returns `None` if the registration group or
    /// the registrant range is not in the embedded range table.
    #[must_use]
    pub fn parts(&self) -> Option<IsbnParts> {
        let split = ranges::split(&self.0)?;
        let group_end = 3 + usize::from(split.group.len);
        let registrant_end = group_end + split.registrant_len?;
        let text = |from: usize, to: usize| -> String {
            self.0
                .iter()
                .skip(from)
                .take(to.saturating_sub(from))
                .map(|d| char::from(b'0' + d))
                .collect()
        };
        Some(IsbnParts {
            gs1_prefix: self.gs1_prefix(),
            group: text(3, group_end),
            registrant: text(group_end, registrant_end),
            publication: text(registrant_end, 12),
            check_digit: char::from(b'0' + self.0[12]),
        })
    }

    /// The ISBN with hyphens between its elements, e.g. `"978-2-07-036002-4"`.
    ///
    /// Falls back to the thirteen plain digits when the registration group or registrant
    /// range is unknown to the embedded range table (see [`range_data_date`]).
    #[must_use]
    pub fn hyphenated(&self) -> String {
        match self.parts() {
            Some(p) => format!(
                "{}-{}-{}-{}-{}",
                p.gs1_prefix, p.group, p.registrant, p.publication, p.check_digit
            ),
            None => self.to_plain(),
        }
    }
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

impl From<Isbn10> for Isbn13 {
    fn from(isbn: Isbn10) -> Self {
        isbn.to_isbn13()
    }
}

impl TryFrom<Isbn13> for Isbn10 {
    type Error = IsbnError;
    fn try_from(isbn: Isbn13) -> Result<Self, Self::Error> {
        isbn.to_isbn10()
    }
}

/// A validated ISBN-10 (nine digits and a check character `0..=9` or `X`).
///
/// ISBN-10 is the pre-2007 form; every ISBN-10 has an [`Isbn13`] equivalent with prefix
/// 978, but 979-prefixed ISBN-13s have no ISBN-10.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Isbn10([u8; 10]);

impl Isbn10 {
    /// Parses an ISBN-10, or a 978-prefixed ISBN-13 (which is converted), with the same
    /// leniency as [`Isbn13::parse`]: separators and a leading label are ignored and the
    /// check character may be a lower-case `x`.
    ///
    /// # Errors
    /// Returns an [`IsbnError`] describing the first problem found;
    /// [`IsbnError::NoIsbn10`] for a valid 979-prefixed ISBN-13.
    pub fn parse(input: &str) -> Result<Self, IsbnError> {
        match parse_any(input)? {
            Parsed::Ten(d) => Ok(Self(d)),
            Parsed::Thirteen(d) => Isbn13(d).to_isbn10(),
        }
    }

    /// The equivalent ISBN-13 (prefix 978).
    #[must_use]
    pub fn to_isbn13(&self) -> Isbn13 {
        Isbn13(ten_to_thirteen(&self.0))
    }

    /// The ten symbols as numbers: `0..=9`, and `10` for a final `X`.
    #[must_use]
    pub fn digits(&self) -> [u8; 10] {
        self.0
    }

    /// The check character: `'0'..='9'` or `'X'`.
    #[must_use]
    pub fn check_char(&self) -> char {
        check10_char(self.0[9])
    }

    /// The ten characters as a plain string without separators (`X` in upper case).
    #[must_use]
    pub fn to_plain(&self) -> String {
        self.0.iter().map(|d| check10_char(*d)).collect()
    }

    /// The registration group of this ISBN (see [`Isbn13::registration_group`]).
    #[must_use]
    pub fn registration_group(&self) -> Option<RegistrationGroup> {
        self.to_isbn13().registration_group()
    }

    /// The ISBN with hyphens between its elements, e.g. `"2-07-036002-4"`; plain
    /// characters when the range is unknown (see [`Isbn13::hyphenated`]).
    #[must_use]
    pub fn hyphenated(&self) -> String {
        match self.to_isbn13().parts() {
            Some(p) => {
                format!("{}-{}-{}-{}", p.group, p.registrant, p.publication, self.check_char())
            }
            None => self.to_plain(),
        }
    }
}

impl fmt::Display for Isbn10 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_plain())
    }
}

impl fmt::Debug for Isbn10 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Isbn10({})", self.to_plain())
    }
}

impl FromStr for Isbn10 {
    type Err = IsbnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use super::{Isbn10, Isbn13};
    use std::borrow::Cow;

    impl serde::Serialize for Isbn13 {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_plain())
        }
    }

    impl<'de> serde::Deserialize<'de> for Isbn13 {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = <Cow<'de, str>>::deserialize(deserializer)?;
            Self::parse(&s).map_err(serde::de::Error::custom)
        }
    }

    impl serde::Serialize for Isbn10 {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_plain())
        }
    }

    impl<'de> serde::Deserialize<'de> for Isbn10 {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = <Cow<'de, str>>::deserialize(deserializer)?;
            Self::parse(&s).map_err(serde::de::Error::custom)
        }
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
        assert_eq!(format!("{a:?}"), "Isbn13(9782070360024)");
        assert_eq!(a.digits(), [9, 7, 8, 2, 0, 7, 0, 3, 6, 0, 0, 2, 4]);
        assert_eq!(a.check_digit(), 4);
        assert_eq!("978 2 07 036002 4".parse::<Isbn13>().expect("valid"), a);
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
        assert_eq!(Isbn13::parse("978207036002X"), Err(IsbnError::InvalidChar('X')));
        assert_eq!(
            Isbn13::parse("080442957 3"),
            Err(IsbnError::CheckDigit { expected: 'X', found: '3' })
        );
        assert_eq!(Isbn13::parse("9782070360024.0"), Err(IsbnError::InvalidChar('.')));
        assert_eq!(Isbn13::parse("ISBN"), Err(IsbnError::Length(0)));
        assert_eq!(Isbn13::parse("97820703600244"), Err(IsbnError::Length(14)));
    }

    #[test]
    fn labels_and_typographic_separators() {
        let expected = Isbn13::parse("9782070360024").expect("valid");
        for input in [
            "ISBN-13: 978-2-07-036002-4",
            "isbn13 9782070360024",
            "ISBN\u{a0}: 978\u{2011}2\u{2011}07\u{2011}036002\u{2011}4",
            "ISBN 978\u{2013}2\u{2013}07\u{2013}036002\u{2013}4",
            "  \t9782070360024\r\n",
            "urn:isbn:9782070360024",
            "URN:ISBN:978-2-07-036002-4",
            "EAN 9782070360024",
            "EAN13: 9782070360024",
            "ISBN-10 2070360024",
            "ISBN 2\u{202f}07\u{202f}036002\u{202f}4",
            "\u{feff}9782070360024",
            "978\u{200b}2070360024",
            "ISBN-139782070360024",
        ] {
            assert_eq!(Isbn13::parse(input), Ok(expected), "input {input:?}");
        }
    }

    #[test]
    fn isbn_label_followed_by_a_number_that_looks_like_a_label_suffix() {
        // An ISBN-10 that starts with "13" or "10", written right after "ISBN-" or "ISBN".
        for body in [b"133456789", b"103456789"] {
            let plain = format!("{}{}", String::from_utf8_lossy(body), isbn10_check_char(body));
            let isbn = Isbn10::parse(&plain).expect("valid by construction");
            assert_eq!(Isbn10::parse(&format!("ISBN-{plain}")), Ok(isbn));
            assert_eq!(Isbn13::parse(&format!("ISBN{plain}")), Ok(isbn.to_isbn13()));
            assert_eq!(Isbn13::parse(&format!("isbn {plain}")), Ok(isbn.to_isbn13()));
        }
    }

    #[test]
    fn isbn10_api() {
        let ten = Isbn10::parse("0-8044-2957-x").expect("valid");
        assert_eq!(ten.to_string(), "080442957X");
        assert_eq!(format!("{ten:?}"), "Isbn10(080442957X)");
        assert_eq!(ten.check_char(), 'X');
        assert_eq!(ten.digits(), [0, 8, 0, 4, 4, 2, 9, 5, 7, 10]);
        assert_eq!(ten.to_isbn13().to_string(), "9780804429573");
        assert_eq!(ten.hyphenated(), "0-8044-2957-X");
        assert_eq!(Isbn13::from(ten).to_isbn10(), Ok(ten));
        assert_eq!(Isbn10::try_from(ten.to_isbn13()), Ok(ten));
        assert_eq!("080442957X".parse::<Isbn10>(), Ok(ten));
        // An ISBN-13 is accepted and converted when it has an ISBN-10 form.
        assert_eq!(Isbn10::parse("978-0-8044-2957-3"), Ok(ten));
        assert_eq!(Isbn10::parse("979-10-90636-07-1"), Err(IsbnError::NoIsbn10));
        assert_eq!(
            Isbn10::parse("9790000000000"),
            Err(IsbnError::CheckDigit { expected: '1', found: '0' })
        );
        assert_eq!(Isbn10::parse("08044X9573"), Err(IsbnError::InvalidChar('X')));
        assert_eq!(Isbn10::parse("123456789"), Err(IsbnError::Length(9)));
    }

    #[test]
    fn isbn13_to_isbn10() {
        let isbn = Isbn13::parse("9782070360024").expect("valid");
        assert_eq!(isbn.to_isbn10().expect("978").to_string(), "2070360024");
        assert_eq!(isbn.gs1_prefix(), 978);
        let fr979 = Isbn13::parse("9791090636071").expect("valid");
        assert_eq!(fr979.to_isbn10(), Err(IsbnError::NoIsbn10));
        assert_eq!(fr979.gs1_prefix(), 979);
    }

    #[test]
    fn check_digit_helpers() {
        assert_eq!(ean13_check_digit(&[9, 7, 8, 2, 0, 7, 0, 3, 6, 0, 0, 2]), 4);
        assert_eq!(ean13_check_digit(b"978207036002"), 4);
        assert_eq!(ean13_check_digit(b"400638133393"), 1);
        assert_eq!(ean13_check_digit(&[0; 12]), 0);
        // Total on arbitrary bytes: reduced modulo 10, never panics.
        assert!(ean13_check_digit(&[255; 12]) <= 9);
        assert_eq!(ean13_check_digit(&[19; 12]), ean13_check_digit(&[9; 12]));
        assert_eq!(isbn10_check_char(b"030640615"), '2');
        assert_eq!(isbn10_check_char(&[0; 9]), '0');
    }

    #[test]
    fn hyphenates_known_isbns() {
        for expected in [
            "978-2-07-036002-4",
            "978-0-306-40615-7",
            "978-3-16-148410-0",
            "979-10-90636-07-1",
            "978-1-4028-9462-6",
            "978-88-06-21935-2",
            "978-90-272-3961-7",
            "978-84-376-0494-7",
            "978-1-933988-03-0",
            "978-4-87311-336-4",
            "979-12-200-0852-5",
            "979-8-6024-0545-3",
            "978-99921-58-10-4",
            "978-972-0-04681-9",
        ] {
            let isbn = Isbn13::parse(expected).expect("valid");
            assert_eq!(isbn.hyphenated(), expected);
            assert_eq!(Isbn13::parse(&isbn.hyphenated()), Ok(isbn));
        }
    }

    #[test]
    fn rejects_a_plausible_looking_isbn_with_a_wrong_check_digit() {
        // "978-88-06-21935-0" circulates as an example; its check digit is wrong.
        assert_eq!(
            Isbn13::parse("978-88-06-21935-0"),
            Err(IsbnError::CheckDigit { expected: '2', found: '0' })
        );
    }

    #[test]
    fn hyphenates_isbn10() {
        for expected in ["2-07-036002-4", "0-306-40615-2", "3-16-148410-X", "88-06-21935-9"] {
            let isbn = Isbn10::parse(expected).expect("valid");
            assert_eq!(isbn.hyphenated(), expected);
        }
    }

    #[test]
    fn parts_and_registration_group() {
        let isbn = Isbn13::parse("9782070360024").expect("valid");
        assert_eq!(
            isbn.parts(),
            Some(IsbnParts {
                gs1_prefix: 978,
                group: "2".into(),
                registrant: "07".into(),
                publication: "036002".into(),
                check_digit: '4',
            })
        );
        let group = isbn.registration_group().expect("known group");
        assert_eq!((group.prefix, group.name), ("978-2", "French language"));
        let it = Isbn13::parse("9788806219352").expect("valid").registration_group();
        assert_eq!(it.map(|g| (g.prefix, g.name)), Some(("978-88", "Italy")));
        let ten = Isbn10::parse("3-16-148410-X").expect("valid").registration_group();
        assert_eq!(ten.map(|g| g.prefix), Some("978-3"));
    }

    #[test]
    fn unknown_ranges_fall_back_to_plain_digits() {
        // 978-6 is not a registration group (groups 600–649 are); 979-0 is ISMN territory.
        for body in ["978699999999", "979000000000", "979999999999"] {
            let mut first12 = [0u8; 12];
            first12.copy_from_slice(body.as_bytes());
            let plain = format!("{body}{}", ean13_check_digit(&first12));
            let isbn = Isbn13::parse(&plain).expect("valid checksum");
            assert_eq!(isbn.registration_group(), None, "{plain}");
            assert_eq!(isbn.parts(), None);
            assert_eq!(isbn.hyphenated(), plain);
        }
        // Known group, unassigned registrant range: 978-90 has no range starting with 91.
        let mut first12 = [0u8; 12];
        first12.copy_from_slice(b"978909100000");
        let plain = format!("978909100000{}", ean13_check_digit(&first12));
        let isbn = Isbn13::parse(&plain).expect("valid checksum");
        assert_eq!(isbn.registration_group().map(|g| g.prefix), Some("978-90"));
        assert_eq!(isbn.parts(), None);
        assert_eq!(isbn.hyphenated(), plain);
    }

    #[test]
    fn range_data_is_dated() {
        let date = range_data_date();
        assert_eq!(date.len(), 10, "ISO date expected, got {date:?}");
        assert!(date.starts_with("20"));
        assert!(!range_data_source().is_empty());
    }

    #[test]
    fn huge_and_hostile_inputs_do_not_panic() {
        let long = "9".repeat(200_000);
        assert_eq!(Isbn13::parse(&long), Err(IsbnError::Length(200_000)));
        assert_eq!(Isbn13::parse(&"-".repeat(100_000)), Err(IsbnError::Length(0)));
        assert_eq!(Isbn13::parse("ISBN-13"), Err(IsbnError::Length(0)));
        assert_eq!(Isbn13::parse("ISBNé"), Err(IsbnError::InvalidChar('é')));
        assert_eq!(Isbn13::parse("é"), Err(IsbnError::InvalidChar('é')));
        assert_eq!(Isbn13::parse("\u{0}9782070360024"), Err(IsbnError::InvalidChar('\u{0}')));
        assert_eq!(Isbn13::parse("٩٧٨٢٠٧٠٣٦٠٠٢٤"), Err(IsbnError::InvalidChar('٩')));
        assert!(Isbn10::parse("XXXXXXXXXX").is_err());
        assert!(Isbn13::parse("urn:isbn:").is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        // Exercised through `ebc-model`, which enables the feature and has serde_json.
        fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
        assert_serde::<Isbn13>();
        assert_serde::<Isbn10>();
    }
}
