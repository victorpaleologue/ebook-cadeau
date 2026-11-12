//! The redemption code printed on the card and encoded in its QR code.
//!
//! A [`GiftCode`] is ten symbols of Crockford's base 32 alphabet: nine random data symbols
//! (45 bits) followed by one check symbol. It is what the recipient types on an e-reader
//! when no camera is at hand, so it is short, case-insensitive, free of look-alike
//! characters, and protected against typing mistakes.
//!
//! # Check symbol
//!
//! The check symbol is computed with the **Damm algorithm**: starting from 0, each symbol
//! `s` updates an interim value `i ← i ∘ s`, where `∘` is a quasigroup operation on the 32
//! symbol values; the check symbol is the value that brings the interim back to 0.
//!
//! The quasigroup used here is built from the finite field GF(2⁵) (polynomial
//! x⁵ + x² + 1): `a ∘ b = α·(a ⊕ b)`, where `⊕` is XOR (field addition) and `α` is the
//! field element `x` (the value 2). It has the three properties Damm's scheme needs:
//!
//! 1. it is a **Latin square** — `c ∘ x = c ∘ y ⇒ x = y` and `x ∘ c = y ∘ c ⇒ x = y`,
//!    because XOR with a constant and multiplication by `α ≠ 0` are bijections — so a
//!    wrong symbol always changes the interim value and the change is never undone:
//!    **every single-symbol substitution is detected**;
//! 2. it is **weakly totally anti-symmetric** — `(c ∘ x) ∘ y = (c ∘ y) ∘ x ⇒ x = y`,
//!    because the two sides differ by `(α² ⊕ α)·(x ⊕ y)` and `α² ⊕ α = α·(α ⊕ 1) ≠ 0` — so
//!    **every transposition of two adjacent symbols is detected**;
//! 3. its **diagonal is zero** — `x ∘ x = 0` — so the check symbol is simply the interim
//!    value after the data symbols, and a valid code folds to 0.
//!
//! These are the properties of the quasigroups Damm calls *weak totally anti-symmetric*
//! (the form with a zero diagonal that his check equation uses; it is necessarily
//! commutative here, which is irrelevant to error detection). All three are verified
//! exhaustively over the 32 × 32 table in this module's tests.
//!
//! Because the operation is linear over GF(2⁵), the check is also a weighted checksum
//! `Σ αⁿ⁻ⁱ⁺¹·sᵢ = 0` whose ten weights are distinct powers of a primitive element. That
//! gives more than the two guarantees above: transpositions of **any** two symbols (not
//! just adjacent ones), twin errors (`aa → bb`) and jump twin errors (`aca → bcb`) are
//! all detected as well. The tests check this for every position of many random codes.
//!
//! # Security
//!
//! The check symbol is **not** a secret and adds no entropy: anyone can compute it. A code
//! carries 45 bits of entropy; it is a bearer token and must be protected by rate
//! limiting on the server. The value of the check symbol for the server is that it tells
//! a *typo* ([`CodeError::Check`], [`CodeError::Length`], [`CodeError::InvalidChar`] —
//! none of which can match a stored code, so they need not count as guessing attempts)
//! from a *well-formed* code that must be looked up (and counted if it is unknown).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Why a string is not a gift code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodeError {
    /// Fewer random bytes were supplied than a code needs.
    #[error("a gift code needs at least {needed} random bytes, got {got}")]
    NotEnoughRandomness {
        /// Bytes required ([`GiftCode::MIN_RANDOM_BYTES`]).
        needed: usize,
        /// Bytes supplied.
        got: usize,
    },
    /// A payload that does not fit in 45 bits.
    #[error("gift code payload does not fit in 45 bits")]
    PayloadTooLarge,
    /// The input does not have ten symbols once separators are removed.
    #[error("a gift code has {expected} characters, found {found}", expected = GiftCode::LEN)]
    Length {
        /// Number of symbols found.
        found: usize,
    },
    /// The input contains a character that is neither a symbol nor an ignored separator.
    #[error("character {0:?} cannot appear in a gift code")]
    InvalidChar(char),
    /// Ten valid symbols whose check symbol does not match: almost certainly a typo.
    /// Servers should *not* count this as an attempt to guess a code.
    #[error("the gift code is mistyped (check character does not match)")]
    Check,
    /// The URL does not contain a `/k/<code>` path.
    #[error("not a claim URL: no /k/<code> path found")]
    NotAClaimUrl,
}

/// GF(2⁵) reduction polynomial x⁵ + x² + 1.
const GF32_POLY: u8 = 0b10_0101;

/// Multiplies a GF(2⁵) element by `α` (the element `x`, value 2).
const fn gf32_mul_alpha(v: u8) -> u8 {
    let shifted = (v & 0x1f) << 1;
    if shifted & 0x20 != 0 { shifted ^ GF32_POLY } else { shifted }
}

/// The quasigroup operation `a ∘ b = α·(a ⊕ b)`.
const fn quasigroup_op(a: u8, b: u8) -> u8 {
    gf32_mul_alpha(a ^ b)
}

/// Builds the 32 × 32 operation table at compile time.
const fn build_quasigroup() -> [[u8; 32]; 32] {
    let mut table = [[0u8; 32]; 32];
    let mut a = 0usize;
    while a < 32 {
        let mut b = 0usize;
        while b < 32 {
            table[a][b] = quasigroup_op(a as u8, b as u8);
            b += 1;
        }
        a += 1;
    }
    table
}

/// `QUASIGROUP[a][b] = a ∘ b`. See the module documentation for its properties.
const QUASIGROUP: [[u8; 32]; 32] = build_quasigroup();

/// Folds symbol values with the Damm recurrence, starting from 0.
fn damm(symbols: &[u8]) -> u8 {
    symbols.iter().fold(0u8, |interim, symbol| {
        QUASIGROUP[usize::from(interim & 0x1f)][usize::from(symbol & 0x1f)]
    })
}

/// The symbol alphabet as bytes.
const ALPHABET_BYTES: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Decodes one character to its symbol value, applying Crockford's lenient reading.
fn symbol_value(c: char) -> Option<u8> {
    let upper = c.to_ascii_uppercase();
    let canonical = match upper {
        'I' | 'L' => '1',
        'O' => '0',
        other => other,
    };
    ALPHABET_BYTES.iter().position(|b| char::from(*b) == canonical).map(|i| i as u8)
}

/// Characters that are skipped when parsing: the separators people add when copying a code
/// by hand, and what a PDF viewer or a phone keyboard may insert on copy-paste.
fn is_ignored(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '-' | '.' | '_'
                | '\u{2010}'..='\u{2015}' // typographic hyphens and dashes
                | '\u{2212}'              // minus sign
                | '\u{00ad}'              // soft hyphen
                | '\u{200b}'..='\u{200d}' // zero-width space / non-joiner / joiner
                | '\u{2060}'              // word joiner
                | '\u{feff}' // byte-order mark
        )
}

/// The human-typed redemption code printed on the card.
///
/// Ten symbols: nine random data symbols and a check symbol (see the [module
/// documentation](self) for the error-detection guarantees). The canonical forms are
/// [`compact`](Self::compact) (`"7ZK3M9QXRT"`, used in URLs and storage) and
/// [`grouped`](Self::grouped) (`"7ZK3M-9QXRT"`, used in print and by `Display`).
///
/// The code is a bearer secret. To keep it out of logs, `Debug` shows only the first two
/// symbols; use `Display`, [`compact`](Self::compact) or [`grouped`](Self::grouped) when
/// the full code is really wanted. It serialises (serde) as its compact string and
/// deserialises with the same leniency as [`parse`](Self::parse).
///
/// ```
/// use ebc_model::{CodeError, GiftCode};
///
/// let code = GiftCode::from_random_bytes(&[0x3a, 0x91, 0x07, 0xee, 0x52, 0xc4])?;
/// assert_eq!(code.grouped().len(), 11);
/// // What people type is accepted: lower case, separators, O for 0, I or L for 1.
/// let typed = code.grouped().to_lowercase().replace('0', "o").replace('1', "l");
/// assert_eq!(GiftCode::parse(&typed)?, code);
/// // A typo is reported as such.
/// let mut typo = code.compact().into_bytes();
/// typo[4] = if typo[4] == b'7' { b'8' } else { b'7' };
/// assert_eq!(GiftCode::parse(std::str::from_utf8(&typo).unwrap()), Err(CodeError::Check));
/// # Ok::<(), CodeError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GiftCode([u8; GiftCode::LEN]);

impl GiftCode {
    /// The symbol alphabet: Crockford's base 32 (no `I`, `L`, `O`, `U`).
    pub const ALPHABET: &'static str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    /// Number of random data symbols (5 bits each: 45 bits of entropy).
    pub const DATA_SYMBOLS: usize = 9;
    /// Total number of symbols: data symbols plus one check symbol.
    pub const LEN: usize = Self::DATA_SYMBOLS + 1;
    /// Bits of entropy carried by a code.
    pub const ENTROPY_BITS: u32 = 5 * Self::DATA_SYMBOLS as u32;
    /// Random bytes [`from_random_bytes`](Self::from_random_bytes) needs.
    pub const MIN_RANDOM_BYTES: usize = 6;
    /// Position of the separator in the grouped form (`"XXXXX-XXXXX"`).
    const GROUP: usize = 5;

    /// Builds a code from random bytes supplied by the caller, who must take them from a
    /// cryptographically secure generator. The first six bytes are used (their 45 most
    /// significant bits); any further bytes are ignored.
    ///
    /// The mapping is uniform: uniformly random bytes give uniformly random codes.
    ///
    /// # Errors
    /// [`CodeError::NotEnoughRandomness`] if fewer than six bytes are supplied.
    pub fn from_random_bytes(bytes: &[u8]) -> Result<Self, CodeError> {
        let Some(head) = bytes.get(..Self::MIN_RANDOM_BYTES) else {
            return Err(CodeError::NotEnoughRandomness {
                needed: Self::MIN_RANDOM_BYTES,
                got: bytes.len(),
            });
        };
        let value = head.iter().fold(0u64, |acc, b| (acc << 8) | u64::from(*b));
        Ok(Self::from_payload_unchecked(value >> (48 - Self::ENTROPY_BITS)))
    }

    /// Builds the code whose data symbols encode `payload` (most significant symbol
    /// first). Meant for tests, fixtures and migrations; real codes come from
    /// [`from_random_bytes`](Self::from_random_bytes).
    ///
    /// # Errors
    /// [`CodeError::PayloadTooLarge`] if `payload` needs more than 45 bits.
    pub fn from_payload(payload: u64) -> Result<Self, CodeError> {
        if payload >> Self::ENTROPY_BITS != 0 {
            return Err(CodeError::PayloadTooLarge);
        }
        Ok(Self::from_payload_unchecked(payload))
    }

    fn from_payload_unchecked(payload: u64) -> Self {
        let mut symbols = [0u8; Self::LEN];
        for (i, slot) in symbols.iter_mut().take(Self::DATA_SYMBOLS).enumerate() {
            let shift = 5 * (Self::DATA_SYMBOLS - 1 - i) as u32;
            *slot = ((payload >> shift) & 0x1f) as u8;
        }
        symbols[Self::DATA_SYMBOLS] = damm(&symbols[..Self::DATA_SYMBOLS]);
        Self(symbols)
    }

    /// Parses a code as typed or pasted by a person.
    ///
    /// Lenient on purpose: case-insensitive; `I` and `L` read as `1`, `O` as `0`;
    /// hyphens, dots, underscores, any whitespace, typographic dashes and invisible
    /// formatting characters are ignored wherever they appear.
    ///
    /// # Errors
    /// * [`CodeError::InvalidChar`] — a character that can never be part of a code
    ///   (including `U`, which the alphabet omits);
    /// * [`CodeError::Length`] — not ten symbols;
    /// * [`CodeError::Check`] — ten symbols, wrong check symbol: a typo.
    ///
    /// None of these inputs can match an issued code, so a server may answer them without
    /// a lookup and without counting them against a rate limit. `Ok` means *well-formed*,
    /// not *issued*.
    pub fn parse(input: &str) -> Result<Self, CodeError> {
        let mut symbols = [0u8; Self::LEN];
        let mut found = 0usize;
        for c in input.chars() {
            if is_ignored(c) {
                continue;
            }
            let value = symbol_value(c).ok_or(CodeError::InvalidChar(c))?;
            if let Some(slot) = symbols.get_mut(found) {
                *slot = value;
            }
            found = found.saturating_add(1);
        }
        if found != Self::LEN {
            return Err(CodeError::Length { found });
        }
        if damm(&symbols) != 0 {
            return Err(CodeError::Check);
        }
        Ok(Self(symbols))
    }

    /// Extracts the code from a claim URL built by [`claim_url`], e.g. what a QR scanner
    /// returns or what a recipient pastes into the code field. The `/k/` path segment is
    /// matched case-insensitively (QR codes may carry the URL in upper case); a query
    /// string or fragment after the code is ignored.
    ///
    /// # Errors
    /// [`CodeError::NotAClaimUrl`] if there is no `/k/<code>` path; otherwise the errors
    /// of [`parse`](Self::parse).
    pub fn from_claim_url(url: &str) -> Result<Self, CodeError> {
        let lower = url.to_ascii_lowercase();
        let start = lower.rfind(CLAIM_PATH).ok_or(CodeError::NotAClaimUrl)? + CLAIM_PATH.len();
        let rest = url.get(start..).ok_or(CodeError::NotAClaimUrl)?;
        let code = rest.split(['?', '#', '/']).next().unwrap_or(rest);
        Self::parse(code)
    }

    /// The ten symbol values (`0..32`), check symbol last.
    #[must_use]
    pub fn symbols(&self) -> [u8; Self::LEN] {
        self.0
    }

    /// The 45-bit payload encoded by the nine data symbols.
    #[must_use]
    pub fn payload(&self) -> u64 {
        self.0.iter().take(Self::DATA_SYMBOLS).fold(0u64, |acc, s| (acc << 5) | u64::from(*s))
    }

    /// The check symbol as a character of [`ALPHABET`](Self::ALPHABET).
    #[must_use]
    pub fn check_symbol(&self) -> char {
        symbol_char(self.0[Self::DATA_SYMBOLS])
    }

    /// The ten symbols without separator, e.g. `"7ZK3M9QXRT"`: nine data symbols and the
    /// check symbol. This is the form used in URLs and storage.
    #[must_use]
    pub fn compact(&self) -> String {
        self.0.iter().map(|s| symbol_char(*s)).collect()
    }

    /// The printed form, two groups of five: `"7ZK3M-9QXRT"`.
    #[must_use]
    pub fn grouped(&self) -> String {
        let mut out = String::with_capacity(Self::LEN + 1);
        for (i, s) in self.0.iter().enumerate() {
            if i == Self::GROUP {
                out.push('-');
            }
            out.push(symbol_char(*s));
        }
        out
    }
}

/// The character for a symbol value.
fn symbol_char(value: u8) -> char {
    char::from(ALPHABET_BYTES[usize::from(value & 0x1f)])
}

impl fmt::Display for GiftCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.grouped())
    }
}

/// Redacted: only the first two symbols are shown, so that codes do not leak through
/// `{:?}` in logs and panic messages.
impl fmt::Debug for GiftCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GiftCode({}{}***-*****)", symbol_char(self.0[0]), symbol_char(self.0[1]))
    }
}

impl FromStr for GiftCode {
    type Err = CodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for GiftCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.compact())
    }
}

impl<'de> Deserialize<'de> for GiftCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Path prefix of claim URLs: `<base>/k/<code>`.
pub const CLAIM_PATH: &str = "/k/";

/// Builds the URL encoded in the QR code and printed as the fallback: base
/// `"https://ebk.example"` gives `"https://ebk.example/k/7ZK3M9QXRT"`.
///
/// Trailing slashes of `base_url` are dropped. The base is otherwise used as given (it is
/// configuration, not user input). The path is short and uses only upper-case letters and
/// digits after `/k/`, so that a QR encoder can upper-case the whole URL and use the
/// compact alphanumeric mode; servers must therefore route `/k/` and `/K/` alike
/// ([`GiftCode::from_claim_url`] does).
#[must_use]
pub fn claim_url(base_url: &str, code: &GiftCode) -> String {
    format!("{}{CLAIM_PATH}{}", base_url.trim_end_matches('/'), code.compact())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Exhaustive proofs over the quasigroup table --------------------------------

    #[test]
    fn table_is_the_documented_field_operation() {
        // Independent bit-by-bit GF(2^5) multiplication, to cross-check the shortcut.
        fn gf_mul(mut a: u8, mut b: u8) -> u8 {
            let mut product = 0u8;
            while b != 0 {
                if b & 1 != 0 {
                    product ^= a;
                }
                a <<= 1;
                if a & 0x20 != 0 {
                    a ^= GF32_POLY;
                }
                b >>= 1;
            }
            product
        }
        // The polynomial is irreducible, hence GF(2^5) is a field: every non-zero element
        // has an inverse, and α generates all 31 non-zero elements.
        for a in 1..32u8 {
            assert_eq!((1..32u8).filter(|b| gf_mul(a, *b) == 1).count(), 1, "inverse of {a}");
        }
        let mut power = 1u8;
        let mut seen = [false; 32];
        for _ in 0..31 {
            power = gf_mul(power, 2);
            assert!(!seen[usize::from(power)], "α is not primitive");
            seen[usize::from(power)] = true;
        }
        assert_eq!(power, 1);
        for a in 0..32u8 {
            for b in 0..32u8 {
                assert_eq!(QUASIGROUP[usize::from(a)][usize::from(b)], gf_mul(2, a ^ b));
                assert!(QUASIGROUP[usize::from(a)][usize::from(b)] < 32);
            }
        }
    }

    #[test]
    fn quasigroup_is_a_latin_square() {
        // Every row and every column is a permutation of 0..32. This is what makes a
        // single wrong symbol change the interim value (rows) and keeps two different
        // interim values different after any further symbol (columns).
        for fixed in 0..32usize {
            let mut row = [false; 32];
            let mut column = [false; 32];
            for other in 0..32usize {
                row[usize::from(QUASIGROUP[fixed][other])] = true;
                column[usize::from(QUASIGROUP[other][fixed])] = true;
            }
            assert!(row.iter().all(|seen| *seen), "row {fixed} is not a permutation");
            assert!(column.iter().all(|seen| *seen), "column {fixed} is not a permutation");
        }
    }

    #[test]
    fn quasigroup_is_weakly_totally_anti_symmetric() {
        // (c ∘ x) ∘ y = (c ∘ y) ∘ x  ⇒  x = y, for all 32³ triples. This is what makes
        // swapping two different adjacent symbols change the interim value.
        let op = |a: u8, b: u8| QUASIGROUP[usize::from(a)][usize::from(b)];
        let mut checked = 0u32;
        for c in 0..32u8 {
            for x in 0..32u8 {
                for y in 0..32u8 {
                    if x != y {
                        assert_ne!(op(op(c, x), y), op(op(c, y), x), "c={c} x={x} y={y}");
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 32 * 32 * 31);
    }

    #[test]
    fn quasigroup_has_a_zero_diagonal() {
        // x ∘ x = 0: the check symbol is the interim value itself.
        for (x, row) in QUASIGROUP.iter().enumerate() {
            assert_eq!(row[x], 0, "{x} ∘ {x}");
        }
    }

    #[test]
    fn single_substitutions_and_adjacent_transpositions_change_the_fold_for_every_state() {
        // The two guarantees, proved at the level of the Damm recurrence rather than of
        // whole codes: whatever the interim value `c` before the error and whatever
        // symbols follow, the final interim value differs.
        let op = |a: u8, b: u8| QUASIGROUP[usize::from(a)][usize::from(b)];
        for c in 0..32u8 {
            for x in 0..32u8 {
                for y in 0..32u8 {
                    if x == y {
                        continue;
                    }
                    // Substitution x -> y right after state c, then any one more symbol.
                    let (good, bad) = (op(c, x), op(c, y));
                    assert_ne!(good, bad);
                    for next in 0..32u8 {
                        assert_ne!(op(good, next), op(bad, next));
                    }
                    // Transposition xy -> yx right after state c, then any one more symbol.
                    let (good, bad) = (op(op(c, x), y), op(op(c, y), x));
                    assert_ne!(good, bad);
                    for next in 0..32u8 {
                        assert_ne!(op(good, next), op(bad, next));
                    }
                }
            }
        }
    }

    // ---- Alphabet and parsing ---------------------------------------------------------

    #[test]
    fn alphabet_is_crockford_base32() {
        assert_eq!(GiftCode::ALPHABET.len(), 32);
        assert_eq!(GiftCode::ALPHABET.as_bytes(), ALPHABET_BYTES);
        for excluded in ['I', 'L', 'O', 'U'] {
            assert!(!GiftCode::ALPHABET.contains(excluded));
        }
        let mut sorted: Vec<char> = GiftCode::ALPHABET.chars().collect();
        sorted.dedup();
        assert_eq!(sorted.len(), 32, "symbols must be distinct");
        assert!(sorted.windows(2).all(|w| w[0] < w[1]), "alphabet must be in ASCII order");
        for (value, c) in GiftCode::ALPHABET.chars().enumerate() {
            assert_eq!(symbol_value(c), Some(value as u8));
            assert_eq!(symbol_value(c.to_ascii_lowercase()), Some(value as u8));
            assert_eq!(symbol_char(value as u8), c);
        }
        assert_eq!(
            (symbol_value('I'), symbol_value('l'), symbol_value('o')),
            (Some(1), Some(1), Some(0))
        );
        assert_eq!(symbol_value('U'), None);
        assert_eq!(symbol_value('u'), None);
        assert_eq!(symbol_value('É'), None);
        assert_eq!((GiftCode::LEN, GiftCode::DATA_SYMBOLS, GiftCode::ENTROPY_BITS), (10, 9, 45));
    }

    #[test]
    fn known_vectors() {
        // All-zero payload: every interim value is 0.
        let zero = GiftCode::from_payload(0).expect("fits");
        assert_eq!(zero.compact(), "0000000000");
        assert_eq!(zero.grouped(), "00000-00000");
        // Payload 1: the last data symbol is 1, the check symbol is 1 ∘-folded: α·1 = 2.
        assert_eq!(GiftCode::from_payload(1).expect("fits").compact(), "0000000012");
        // 0x10 = x⁴; α·x⁴ = x⁵ = x² + 1 = 5.
        assert_eq!(GiftCode::from_payload(0x10).expect("fits").compact(), "00000000G5");
        // The largest payload.
        let max = GiftCode::from_payload((1 << 45) - 1).expect("fits");
        assert_eq!(&max.compact()[..9], "ZZZZZZZZZ");
        assert_eq!(max.payload(), (1 << 45) - 1);
        assert_eq!(GiftCode::from_payload(1 << 45), Err(CodeError::PayloadTooLarge));
        assert_eq!(GiftCode::from_payload(u64::MAX), Err(CodeError::PayloadTooLarge));
    }

    #[test]
    fn random_bytes_map_to_the_top_45_bits() {
        let code =
            GiftCode::from_random_bytes(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xf8]).expect("6 bytes");
        assert_eq!(code.payload(), (1 << 45) - 1);
        // The three low bits of the sixth byte and any further byte are ignored.
        let a = GiftCode::from_random_bytes(&[1, 2, 3, 4, 5, 0b1010_1000]).expect("6 bytes");
        let b =
            GiftCode::from_random_bytes(&[1, 2, 3, 4, 5, 0b1010_1111, 99, 100]).expect("8 bytes");
        assert_eq!(a, b);
        assert_eq!(a.payload(), 0x0102_0304_05a8 >> 3);
        // Every bit of the first 45 matters.
        for bit in 0..45u32 {
            let mut bytes = [0u8; 6];
            bytes[(bit / 8) as usize] = 0x80 >> (bit % 8);
            let code = GiftCode::from_random_bytes(&bytes).expect("6 bytes");
            assert_eq!(code.payload(), 1u64 << (44 - bit), "bit {bit}");
        }
        for len in 0..6 {
            assert_eq!(
                GiftCode::from_random_bytes(&vec![7u8; len]),
                Err(CodeError::NotEnoughRandomness { needed: 6, got: len })
            );
        }
    }

    #[test]
    fn display_debug_and_string_forms() {
        let code = GiftCode::from_payload(0x1234_5678_9abc >> 3).expect("fits");
        let compact = code.compact();
        let grouped = code.grouped();
        assert_eq!(compact.len(), 10);
        assert_eq!(grouped.len(), 11);
        assert_eq!(grouped.replace('-', ""), compact);
        assert_eq!(grouped.find('-'), Some(5));
        assert_eq!(code.to_string(), grouped);
        assert!(compact.chars().all(|c| GiftCode::ALPHABET.contains(c)));
        assert_eq!(compact.chars().last(), Some(code.check_symbol()));
        assert_eq!(compact.parse::<GiftCode>(), Ok(code));
        // Debug must not reveal the code.
        let debug = format!("{code:?}");
        assert!(debug.starts_with(&format!("GiftCode({}", &compact[..2])));
        assert!(!debug.contains(&compact[2..]));
        assert!(!debug.contains(&grouped[3..]));
    }

    #[test]
    fn lenient_parsing() {
        let code = GiftCode::from_payload(0x0f0f_1e2d_3c4b >> 3).expect("fits");
        let compact = code.compact();
        let grouped = code.grouped();
        for typed in [
            compact.clone(),
            grouped.clone(),
            compact.to_lowercase(),
            grouped.to_lowercase(),
            format!("  {grouped}\n"),
            format!("{} {}", &compact[..5], &compact[5..]),
            format!("{}.{}", &compact[..5], &compact[5..]),
            format!("{}_{}", &compact[..5], &compact[5..]),
            format!("{}\u{2013}{}", &compact[..5], &compact[5..]),
            format!("{}\u{2011}{}", &compact[..5], &compact[5..]),
            format!("{}\u{a0}{}", &compact[..5], &compact[5..]),
            compact.chars().map(|c| format!("{c}-")).collect::<String>(),
            compact.chars().map(|c| format!(" {c} ")).collect::<String>(),
            format!("\u{feff}{compact}\u{200b}"),
            compact.replace('0', "O").replace('1', "I"),
            compact.replace('0', "o").replace('1', "l"),
            compact.replace('1', "L"),
        ] {
            assert_eq!(GiftCode::parse(&typed), Ok(code), "typed {typed:?}");
        }
    }

    #[test]
    fn look_alike_letters_decode_to_digits() {
        // Data symbols "011000000": "OIL…" and "oil…" must read the same.
        let code = GiftCode::from_payload((1u64 << 35) | (1u64 << 30)).expect("fits");
        let compact = code.compact();
        assert_eq!(&compact[..3], "011");
        assert_eq!(GiftCode::parse(&format!("OIL{}", &compact[3..])), Ok(code));
        assert_eq!(GiftCode::parse(&format!("oil{}", &compact[3..])), Ok(code));
        assert_eq!(GiftCode::parse(&format!("OLI{}", &compact[3..])), Ok(code));
        // …and the canonical output never contains the look-alikes.
        for payload in [0u64, 1, 0x7fff, (1 << 45) - 1, 0x1234_5678_9abc >> 3] {
            let text = GiftCode::from_payload(payload).expect("fits").grouped();
            assert!(!text.contains(['I', 'L', 'O', 'U']), "{text}");
            assert_eq!(text, text.to_uppercase());
        }
    }

    #[test]
    fn parse_errors_are_specific() {
        let code = GiftCode::from_payload(123_456_789).expect("fits");
        let compact = code.compact();
        assert_eq!(GiftCode::parse(""), Err(CodeError::Length { found: 0 }));
        assert_eq!(GiftCode::parse(" - . _ "), Err(CodeError::Length { found: 0 }));
        assert_eq!(GiftCode::parse(&compact[..9]), Err(CodeError::Length { found: 9 }));
        assert_eq!(GiftCode::parse(&format!("{compact}0")), Err(CodeError::Length { found: 11 }));
        assert_eq!(
            GiftCode::parse(&format!("{compact}{compact}")),
            Err(CodeError::Length { found: 20 })
        );
        assert_eq!(GiftCode::parse("ABCDEFGHJU"), Err(CodeError::InvalidChar('U')));
        assert_eq!(GiftCode::parse("ABCDE/FGHJK"), Err(CodeError::InvalidChar('/')));
        assert_eq!(GiftCode::parse("ABCDÉ-FGHJK"), Err(CodeError::InvalidChar('É')));
        assert_eq!(GiftCode::parse("ＡBCDE-FGHJK"), Err(CodeError::InvalidChar('Ａ')));
        assert_eq!(GiftCode::parse("ABCDE\u{0}FGHJK"), Err(CodeError::InvalidChar('\u{0}')));
        assert_eq!(
            GiftCode::parse("https://ebk.example/k/ABCDEFGHJK"),
            Err(CodeError::InvalidChar(':'))
        );
        // Invalid characters are reported even when the length is wrong too.
        assert_eq!(GiftCode::parse("AB!"), Err(CodeError::InvalidChar('!')));
        // Hostile sizes.
        assert_eq!(
            GiftCode::parse(&"0".repeat(1_000_000)),
            Err(CodeError::Length { found: 1_000_000 })
        );
        assert_eq!(GiftCode::parse(&"-".repeat(1_000_000)), Err(CodeError::Length { found: 0 }));
        // A wrong check symbol is a typo, and says so.
        let wrong_check = if code.check_symbol() == 'Z' { 'Y' } else { 'Z' };
        let typo = format!("{}{wrong_check}", &compact[..9]);
        assert_eq!(GiftCode::parse(&typo), Err(CodeError::Check));
        assert!(CodeError::Check.to_string().contains("mistyped"));
        assert!(CodeError::Length { found: 3 }.to_string().contains("10 characters, found 3"));
    }

    #[test]
    fn claim_urls() {
        let code = GiftCode::from_payload(987_654_321).expect("fits");
        let compact = code.compact();
        assert_eq!(
            claim_url("https://ebk.example", &code),
            format!("https://ebk.example/k/{compact}")
        );
        assert_eq!(
            claim_url("https://ebk.example/", &code),
            format!("https://ebk.example/k/{compact}")
        );
        assert_eq!(
            claim_url("https://ebk.example///", &code),
            format!("https://ebk.example/k/{compact}")
        );
        assert_eq!(
            claim_url("http://localhost:8080/base", &code),
            format!("http://localhost:8080/base/k/{compact}")
        );
        assert_eq!(claim_url("", &code), format!("/k/{compact}"));
        // The URL only uses characters of the QR alphanumeric set once upper-cased.
        let upper = claim_url("https://ebk.example", &code).to_uppercase();
        assert!(upper.chars().all(|c| "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:".contains(c)));

        for url in [
            claim_url("https://ebk.example", &code),
            upper,
            format!("https://ebk.example/k/{compact}?utm=1"),
            format!("https://ebk.example/k/{compact}#top"),
            format!("https://ebk.example/k/{compact}/"),
            format!("https://ebk.example/k/{}", code.grouped()),
            format!("ebk.example/k/{}", compact.to_lowercase()),
            format!("/k/{compact}"),
            format!("https://k.example/k/x/k/{compact}"),
        ] {
            assert_eq!(GiftCode::from_claim_url(&url), Ok(code), "url {url:?}");
        }
        assert_eq!(GiftCode::from_claim_url("https://ebk.example/"), Err(CodeError::NotAClaimUrl));
        assert_eq!(GiftCode::from_claim_url(""), Err(CodeError::NotAClaimUrl));
        assert_eq!(GiftCode::from_claim_url(&compact), Err(CodeError::NotAClaimUrl));
        assert_eq!(
            GiftCode::from_claim_url("https://ebk.example/k/"),
            Err(CodeError::Length { found: 0 })
        );
        assert_eq!(
            GiftCode::from_claim_url("https://ebk.example/k/é"),
            Err(CodeError::InvalidChar('é'))
        );
        assert_eq!(
            GiftCode::from_claim_url(&format!("https://ebk.example/k/{}Z", &compact[..9])).is_err(),
            code.check_symbol() != 'Z'
        );
    }

    #[test]
    fn serialises_as_the_compact_string() {
        let code = GiftCode::from_payload(0x1f_ffff_ffff).expect("fits");
        let json = serde_json::to_string(&code).expect("serialises");
        assert_eq!(json, format!("\"{}\"", code.compact()));
        assert_eq!(serde_json::from_str::<GiftCode>(&json).expect("parses"), code);
        // Deserialisation is as lenient as parsing…
        let grouped = format!("\"{}\"", code.grouped().to_lowercase());
        assert_eq!(serde_json::from_str::<GiftCode>(&grouped).expect("parses"), code);
        // …and as strict about the check symbol.
        let wrong_check = if code.check_symbol() == '0' { '1' } else { '0' };
        let bad = format!("\"{}{wrong_check}\"", &code.compact()[..9]);
        assert!(serde_json::from_str::<GiftCode>(&bad).is_err());
        assert!(serde_json::from_str::<GiftCode>("12345").is_err());
        assert!(serde_json::from_str::<GiftCode>("null").is_err());
    }
}
