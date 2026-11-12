//! The codes people type: the redemption code printed on the card ([`GiftCode`]) and the
//! short-lived code that pairs a phone with an e-reader's browser ([`PairCode`]).
//!
//! Both use the same alphabet of **20 consonants**, `BCDFGHJKLMNPQRSTVWXZ`: letters only,
//! so that no keyboard layer switch is needed on e-ink and phone keyboards (nor Shift on
//! AZERTY), no digit/letter look-alikes exist (0/O, 1/I/L, 5/S, 8/B), and no vowel can
//! spell a word on a gift. Input is case-insensitive and ignores separators.
//!
//! # The check symbol of a gift code
//!
//! A [`GiftCode`] is eleven random data symbols followed by one check symbol, computed
//! with a **Damm-style scheme**: a running state starts at 0, each symbol `s` updates it
//! as `state ← state ∘ s`, and the check symbol is the one symbol that brings the state
//! back to 0.
//!
//! `∘` is a quasigroup operation on the 20 symbol values. A value `v` is read as the pair
//! `(h, l)` with `v = 5·h + l`, `h` in the field GF(4) (polynomials over GF(2) modulo
//! x² + x + 1) and `l` in ℤ/5, and the operation works on each component separately:
//!
//! ```text
//! (h₁, l₁) ∘ (h₂, l₂) = (α·h₁ + h₂ ,  2·l₁ + l₂ mod 5)        α = the element x of GF(4)
//! ```
//!
//! In both components this is `a·x + y` with `a` invertible and `a − 1` invertible too
//! (`2 − 1 = 1` in ℤ/5; `α + 1 ≠ 0` in GF(4)). Hence:
//!
//! 1. it is a **Latin square** — `c ∘ x = c ∘ y ⇒ x = y` and `x ∘ c = y ∘ c ⇒ x = y` — so
//!    a wrong symbol always changes the state and later symbols never undo the change:
//!    **every single-symbol substitution is detected**;
//! 2. it is **totally anti-symmetric** — `(c ∘ x) ∘ y = (c ∘ y) ∘ x ⇒ x = y` (the two
//!    sides differ by `(a − 1)·(x − y)`), and also `x ∘ y = y ∘ x ⇒ x = y` — so **every
//!    transposition of two adjacent symbols is detected**.
//!
//! The same algebra gives two more guarantees: transpositions of symbols **two places
//! apart** (`abc → cba`, the difference is `(a² − 1)·(x − z)`) and adjacent **twin errors**
//! (`aa → bb`, the difference is `(a + 1)·(x − y)`) are always detected. All four
//! properties are verified exhaustively over the quasigroup in this module's tests, and
//! on every position of many random codes in `tests/gift_code.rs`.
//!
//! What is **not** guaranteed: swaps of symbols three or more places apart and jump twin
//! errors (`aca → bcb`) are detected most of the time but not always (the weights of the
//! ℤ/5 component repeat every four positions, those of the GF(4) component every three).
//! Any other random corruption passes with probability 1/20.
//!
//! # Security
//!
//! The check symbol is public arithmetic and adds no entropy. A gift code carries
//! log₂(20¹¹) ≈ 47.5 bits; it is a bearer secret and must be protected by rate limiting
//! on the server — per client **and globally**, because a guess is a guess against every
//! live code. The check symbol lets the server tell a *typo* ([`CodeError::Check`],
//! [`CodeError::Length`], [`CodeError::InvalidChar`]: none of these can match an issued
//! code, so they need not count as guessing attempts) from a *well-formed* code that must
//! be looked up, and counted if it is unknown.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Why a string is not a code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodeError {
    /// Fewer random bytes were supplied than a code needs.
    #[error("a code needs at least {needed} random bytes, got {got}")]
    NotEnoughRandomness {
        /// Bytes required.
        needed: usize,
        /// Bytes supplied.
        got: usize,
    },
    /// A payload outside the range a code can encode.
    #[error("payload is too large for a code")]
    PayloadTooLarge,
    /// The input does not have the right number of symbols once separators are removed.
    #[error("the code has {expected} letters, found {found}")]
    Length {
        /// Symbols a code of this kind has.
        expected: usize,
        /// Symbols found.
        found: usize,
    },
    /// The input contains a character that is neither a symbol nor an ignored separator
    /// (a digit, a vowel, punctuation…).
    #[error("character {0:?} cannot appear in a code (letters {alphabet} only)", alphabet = GiftCode::ALPHABET)]
    InvalidChar(char),
    /// The right number of valid symbols, but the check symbol does not match: almost
    /// certainly a typo. Servers should *not* count this as an attempt to guess a code.
    #[error("the code is mistyped (check letter does not match)")]
    Check,
    /// The URL does not contain a `/k/<code>` path.
    #[error("not a claim URL: no /k/<code> path found")]
    NotAClaimUrl,
}

// ---- Alphabet -------------------------------------------------------------------------

/// The symbol alphabet as bytes. The index of a letter is its symbol value.
const ALPHABET_BYTES: &[u8; 20] = b"BCDFGHJKLMNPQRSTVWXZ";

/// Number of symbols in the alphabet.
const RADIX: u64 = 20;

/// The letter for a symbol value (`0..20`).
fn symbol_char(value: u8) -> char {
    char::from(ALPHABET_BYTES[usize::from(value) % ALPHABET_BYTES.len()])
}

/// Decodes one character to its symbol value, ignoring case.
fn symbol_value(c: char) -> Option<u8> {
    let upper = c.to_ascii_uppercase();
    ALPHABET_BYTES.iter().position(|b| char::from(*b) == upper).map(|i| i as u8)
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

/// Reads exactly `N` symbols from text typed or pasted by a person.
fn parse_symbols<const N: usize>(input: &str) -> Result<[u8; N], CodeError> {
    let mut symbols = [0u8; N];
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
    if found == N { Ok(symbols) } else { Err(CodeError::Length { expected: N, found }) }
}

/// Writes symbols as letters, with a hyphen before every `group`-th symbol (`0`: none).
fn format_symbols(symbols: &[u8], group: usize) -> String {
    let mut out = String::with_capacity(symbols.len() + 3);
    for (i, symbol) in symbols.iter().enumerate() {
        if group != 0 && i != 0 && i % group == 0 {
            out.push('-');
        }
        out.push(symbol_char(*symbol));
    }
    out
}

/// Writes `value` as `N` base-20 digits, most significant first. `value` must be below
/// `20^N`; higher digits are dropped.
fn digits_of<const N: usize>(mut value: u64) -> [u8; N] {
    let mut digits = [0u8; N];
    for slot in digits.iter_mut().rev() {
        *slot = (value % RADIX) as u8;
        value /= RADIX;
    }
    digits
}

/// Reads base-20 digits, most significant first.
fn value_of(digits: &[u8]) -> u64 {
    digits.iter().fold(0u64, |acc, d| acc * RADIX + u64::from(*d))
}

/// Random bytes consumed per sampling attempt.
const CHUNK: usize = 8;

/// Draws a number in `0..modulus` from caller-supplied random bytes.
///
/// The bytes are read in 8-byte chunks. A chunk is accepted when its value falls below the
/// largest multiple of `modulus` that fits in 64 bits (rejection sampling): the result is
/// then exactly uniform. If every complete chunk is rejected — probability `modulus / 2⁶⁴`
/// per chunk, about 10⁻⁵ for a gift code — the last chunk is reduced modulo `modulus`
/// instead of failing, which is biased by at most that same amount.
fn sample_below(bytes: &[u8], modulus: u64) -> Result<u64, CodeError> {
    let mut last: Option<u64> = None;
    let limit = u64::MAX - u64::MAX % modulus; // a multiple of `modulus`
    for chunk in bytes.chunks_exact(CHUNK) {
        let value = chunk.iter().fold(0u64, |acc, b| (acc << 8) | u64::from(*b));
        if value < limit {
            return Ok(value % modulus);
        }
        last = Some(value);
    }
    last.map(|value| value % modulus)
        .ok_or(CodeError::NotEnoughRandomness { needed: CHUNK, got: bytes.len() })
}

// ---- The quasigroup -------------------------------------------------------------------

/// Multiplication by `α` (the element `x`) in GF(4) = GF(2)[x] / (x² + x + 1):
/// `α·0 = 0`, `α·1 = α`, `α·α = α + 1`, `α·(α + 1) = 1`.
const GF4_MUL_ALPHA: [u8; 4] = [0, 2, 3, 1];

/// The quasigroup operation `a ∘ b` on symbol values (see the module documentation).
const fn quasigroup_op(a: u8, b: u8) -> u8 {
    let (ah, al) = (a / 5 % 4, a % 5);
    let (bh, bl) = (b / 5 % 4, b % 5);
    let h = GF4_MUL_ALPHA[ah as usize] ^ bh;
    let l = (2 * al + bl) % 5;
    5 * h + l
}

/// Builds the 20 × 20 operation table at compile time.
const fn build_quasigroup() -> [[u8; 20]; 20] {
    let mut table = [[0u8; 20]; 20];
    let mut a = 0usize;
    while a < 20 {
        let mut b = 0usize;
        while b < 20 {
            table[a][b] = quasigroup_op(a as u8, b as u8);
            b += 1;
        }
        a += 1;
    }
    table
}

/// `QUASIGROUP[a][b] = a ∘ b`.
const QUASIGROUP: [[u8; 20]; 20] = build_quasigroup();

/// Folds symbol values with the Damm recurrence, starting from state 0.
fn damm(symbols: &[u8]) -> u8 {
    symbols
        .iter()
        .fold(0u8, |state, symbol| QUASIGROUP[usize::from(state) % 20][usize::from(*symbol) % 20])
}

/// The one symbol `s` with `state ∘ s = 0`.
fn check_symbol_for(state: u8) -> u8 {
    let row = &QUASIGROUP[usize::from(state) % 20];
    // Each row is a permutation of 0..20, so exactly one column holds 0.
    row.iter().position(|v| *v == 0).unwrap_or(0) as u8
}

// ---- GiftCode -------------------------------------------------------------------------

/// The human-typed redemption code printed on the card.
///
/// Twelve consonants: eleven random data symbols and a check symbol (see the [module
/// documentation](self) for the error-detection guarantees). The canonical forms are
/// [`compact`](Self::compact) (`"BCDFGHJKLMNP"`, used in URLs and storage) and
/// [`grouped`](Self::grouped) (`"BCDF-GHJK-LMNP"`, used in print and by `Display`).
///
/// The code is a bearer secret. To keep it out of logs, `Debug` shows only the first two
/// letters; use `Display`, [`compact`](Self::compact) or [`grouped`](Self::grouped) when
/// the full code is really wanted. It serialises (serde) as its compact string and
/// deserialises with the same leniency as [`parse`](Self::parse).
///
/// ```
/// use ebc_model::{CodeError, GiftCode};
///
/// let random = [0x3a, 0x91, 0x07, 0xee, 0x52, 0xc4, 0x1b, 0x60]; // from a CSPRNG
/// let code = GiftCode::from_random_bytes(&random)?;
/// assert_eq!(code.grouped().len(), 14);
/// // What people type is accepted: lower case, spaces, dots…
/// let typed = code.grouped().to_lowercase().replace('-', " ");
/// assert_eq!(GiftCode::parse(&typed)?, code);
/// // Swapping two neighbouring letters is reported as a typo.
/// let mut typo = code.compact().into_bytes();
/// if typo[4] != typo[5] {
///     typo.swap(4, 5);
///     assert_eq!(GiftCode::parse(&String::from_utf8_lossy(&typo)), Err(CodeError::Check));
/// }
/// # Ok::<(), CodeError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GiftCode([u8; GiftCode::LEN]);

impl GiftCode {
    /// The symbol alphabet: 20 consonants, letters only, no vowels (and no `Y`).
    pub const ALPHABET: &'static str = "BCDFGHJKLMNPQRSTVWXZ";
    /// Number of random data symbols: 20¹¹ ≈ 2⁴⁷·⁵ possible codes.
    pub const DATA_SYMBOLS: usize = 11;
    /// Total number of symbols: data symbols plus one check symbol.
    pub const LEN: usize = 12;
    /// Number of distinct codes: 20¹¹.
    pub const SPACE: u64 = RADIX.pow(Self::DATA_SYMBOLS as u32);
    /// Random bytes [`from_random_bytes`](Self::from_random_bytes) needs.
    pub const MIN_RANDOM_BYTES: usize = CHUNK;
    /// Symbols per printed group (`"XXXX-XXXX-XXXX"`).
    pub const GROUP: usize = 4;

    /// Builds a code from random bytes supplied by the caller, who must take them from a
    /// cryptographically secure generator.
    ///
    /// At least eight bytes are needed; **sixteen or more are recommended**. The bytes are
    /// consumed in 8-byte chunks by rejection sampling, which makes the code exactly
    /// uniform over the 20¹¹ possibilities. Should every chunk be rejected (probability
    /// about 10⁻⁵ per chunk, so 10⁻¹⁰ with sixteen bytes), the last chunk is reduced
    /// modulo 20¹¹ rather than failing; that fallback is biased by at most 10⁻⁵ in
    /// relative terms. The function is deterministic: the same bytes give the same code.
    ///
    /// # Errors
    /// [`CodeError::NotEnoughRandomness`] if fewer than eight bytes are supplied.
    pub fn from_random_bytes(bytes: &[u8]) -> Result<Self, CodeError> {
        sample_below(bytes, Self::SPACE).map(Self::from_payload_unchecked)
    }

    /// Builds the code whose data symbols encode `payload` in base 20 (most significant
    /// symbol first). Meant for tests, fixtures and migrations; real codes come from
    /// [`from_random_bytes`](Self::from_random_bytes).
    ///
    /// # Errors
    /// [`CodeError::PayloadTooLarge`] if `payload` is not below [`SPACE`](Self::SPACE).
    pub fn from_payload(payload: u64) -> Result<Self, CodeError> {
        if payload >= Self::SPACE {
            return Err(CodeError::PayloadTooLarge);
        }
        Ok(Self::from_payload_unchecked(payload))
    }

    fn from_payload_unchecked(payload: u64) -> Self {
        let data: [u8; Self::DATA_SYMBOLS] = digits_of(payload);
        let mut symbols = [0u8; Self::LEN];
        for (slot, digit) in symbols.iter_mut().zip(data) {
            *slot = digit;
        }
        symbols[Self::DATA_SYMBOLS] = check_symbol_for(damm(&data));
        Self(symbols)
    }

    /// Parses a code as typed or pasted by a person.
    ///
    /// Lenient on purpose: case-insensitive; hyphens, dots, underscores, any whitespace,
    /// typographic dashes and invisible formatting characters are ignored wherever they
    /// appear.
    ///
    /// # Errors
    /// * [`CodeError::InvalidChar`] — a character that can never be part of a code (a
    ///   digit, a vowel, punctuation);
    /// * [`CodeError::Length`] — not twelve symbols;
    /// * [`CodeError::Check`] — twelve symbols, wrong check symbol: a typo.
    ///
    /// None of these inputs can match an issued code, so a server may answer them without
    /// a lookup and without counting them against a rate limit. `Ok` means *well-formed*,
    /// not *issued*.
    pub fn parse(input: &str) -> Result<Self, CodeError> {
        let symbols: [u8; Self::LEN] = parse_symbols(input)?;
        if damm(&symbols) != 0 {
            return Err(CodeError::Check);
        }
        Ok(Self(symbols))
    }

    /// Extracts the code from a claim URL built by [`claim_url`] or [`claim_url_qr`], e.g.
    /// what a QR scanner returns or what a recipient pastes into the code field. The
    /// `/k/` path segment is matched case-insensitively; a query string, a fragment or a
    /// further path segment after the code is ignored.
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

    /// Parses what a person typed or pasted into a code field: a code
    /// ([`parse`](Self::parse)) or a whole claim URL, with or without its scheme
    /// (`https://ebk.example/k/…`, `ebk.example/k/…`, `EBK.EXAMPLE/K/…`, `/k/…`).
    ///
    /// The input is first read as a code. Only when that fails with
    /// [`CodeError::InvalidChar`] or [`CodeError::Length`] *and* the input contains a `/`
    /// is it read as a claim URL ([`from_claim_url`](Self::from_claim_url)). A code that is
    /// merely mistyped therefore still reports [`CodeError::Check`], whether it was typed
    /// alone or inside a URL. This is the one function the server's redemption form and
    /// the web app's code field share.
    ///
    /// # Errors
    /// The errors of [`parse`](Self::parse); [`CodeError::NotAClaimUrl`] when the input
    /// holds a `/` but no `/k/<code>` path.
    pub fn parse_typed(input: &str) -> Result<Self, CodeError> {
        match Self::parse(input) {
            Err(CodeError::InvalidChar(_) | CodeError::Length { .. }) if input.contains('/') => {
                Self::from_claim_url(input.trim())
            }
            other => other,
        }
    }

    /// The twelve symbol values (`0..20`), check symbol last.
    #[must_use]
    pub fn symbols(&self) -> [u8; Self::LEN] {
        self.0
    }

    /// The number below [`SPACE`](Self::SPACE) encoded by the eleven data symbols.
    #[must_use]
    pub fn payload(&self) -> u64 {
        value_of(&self.0[..Self::DATA_SYMBOLS])
    }

    /// The check symbol as a letter of [`ALPHABET`](Self::ALPHABET).
    #[must_use]
    pub fn check_symbol(&self) -> char {
        symbol_char(self.0[Self::DATA_SYMBOLS])
    }

    /// The twelve letters without separator, e.g. `"BCDFGHJKLMNP"`. This is the form used
    /// in URLs and storage.
    #[must_use]
    pub fn compact(&self) -> String {
        format_symbols(&self.0, 0)
    }

    /// The printed form, three groups of four: `"BCDF-GHJK-LMNP"`.
    #[must_use]
    pub fn grouped(&self) -> String {
        format_symbols(&self.0, Self::GROUP)
    }

    /// Whether the code happens to contain a letter sequence best not printed on a gift
    /// (a handful of well-known abbreviations such as `KKK` or `WTF`). Consonants cannot
    /// spell words, but they can spell these. An issuer may simply draw another code:
    /// about 1.4 % of codes are flagged.
    #[must_use]
    pub fn has_unfortunate_letters(&self) -> bool {
        const AVOID: [&str; 12] =
            ["KKK", "WTF", "FCK", "FKN", "SHT", "CNT", "DCK", "NGR", "NZS", "BTCH", "PRN", "SLT"];
        let compact = self.compact();
        AVOID.iter().any(|letters| compact.contains(letters))
    }
}

impl fmt::Display for GiftCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.grouped())
    }
}

/// Redacted: only the first two letters are shown, so that codes do not leak through
/// `{:?}` in logs and panic messages.
impl fmt::Debug for GiftCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GiftCode({}{}**-****-****)", symbol_char(self.0[0]), symbol_char(self.0[1]))
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

// ---- PairCode -------------------------------------------------------------------------

/// The short code that pairs a phone with an e-reader's browser: the phone shows it, the
/// recipient types it on the e-reader.
///
/// Six letters of the [`GiftCode`] alphabet, printed `"XXX-XXX"`; 20⁶ = 64 million
/// possibilities and **no check symbol**. It is only safe as a single-use, short-lived
/// secret behind per-client and global failure budgets; that policy belongs to the
/// server. `Debug` is redacted like [`GiftCode`]'s.
///
/// ```
/// use ebc_model::PairCode;
/// let pair = PairCode::from_random_bytes(&[7, 200, 13, 99, 1, 2, 3, 4])?;
/// assert_eq!(pair.grouped().len(), 7);
/// assert_eq!(PairCode::parse(&pair.grouped().to_lowercase())?, pair);
/// # Ok::<(), ebc_model::CodeError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PairCode([u8; PairCode::LEN]);

impl PairCode {
    /// Number of symbols.
    pub const LEN: usize = 6;
    /// Number of distinct pair codes: 20⁶ = 64 000 000.
    pub const SPACE: u64 = RADIX.pow(Self::LEN as u32);
    /// Random bytes [`from_random_bytes`](Self::from_random_bytes) needs.
    pub const MIN_RANDOM_BYTES: usize = CHUNK;
    /// Symbols per printed group (`"XXX-XXX"`).
    pub const GROUP: usize = 3;

    /// Builds a pair code from caller-supplied random bytes (at least eight, from a
    /// cryptographically secure generator), sampled as in
    /// [`GiftCode::from_random_bytes`]; a chunk is rejected with probability below 10⁻¹¹.
    ///
    /// # Errors
    /// [`CodeError::NotEnoughRandomness`] if fewer than eight bytes are supplied.
    pub fn from_random_bytes(bytes: &[u8]) -> Result<Self, CodeError> {
        sample_below(bytes, Self::SPACE).map(|value| Self(digits_of(value)))
    }

    /// Parses a pair code as typed: case-insensitive, separators ignored (see
    /// [`GiftCode::parse`]).
    ///
    /// # Errors
    /// [`CodeError::InvalidChar`] or [`CodeError::Length`]. There is no check symbol, so
    /// every six-letter input is well-formed and must be counted as an attempt.
    pub fn parse(input: &str) -> Result<Self, CodeError> {
        parse_symbols(input).map(Self)
    }

    /// The six symbol values (`0..20`).
    #[must_use]
    pub fn symbols(&self) -> [u8; Self::LEN] {
        self.0
    }

    /// The six letters without separator, e.g. `"BCDFGH"`.
    #[must_use]
    pub fn compact(&self) -> String {
        format_symbols(&self.0, 0)
    }

    /// The displayed form, two groups of three: `"BCD-FGH"`.
    #[must_use]
    pub fn grouped(&self) -> String {
        format_symbols(&self.0, Self::GROUP)
    }
}

impl fmt::Display for PairCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.grouped())
    }
}

/// Redacted: only the first letter is shown.
impl fmt::Debug for PairCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PairCode({}**-***)", symbol_char(self.0[0]))
    }
}

impl FromStr for PairCode {
    type Err = CodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for PairCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.compact())
    }
}

impl<'de> Deserialize<'de> for PairCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

// ---- Claim URLs -----------------------------------------------------------------------

/// Path prefix of claim URLs: `<base>/k/<code>`.
pub const CLAIM_PATH: &str = "/k/";

/// Builds the URL printed in clear on the card: base `"https://ebk.example"` gives
/// `"https://ebk.example/k/BCDFGHJKLMNP"`.
///
/// Trailing slashes of `base_url` are dropped. The base is otherwise used as given (it is
/// configuration, not user input).
#[must_use]
pub fn claim_url(base_url: &str, code: &GiftCode) -> String {
    format!("{}{CLAIM_PATH}{}", base_url.trim_end_matches('/'), code.compact())
}

/// Builds the same URL fully upper-cased, for the QR code:
/// `"HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNP"`.
///
/// Upper case keeps the QR code in *alphanumeric mode* (digits, `A–Z` and
/// ``space $ % * + - . / :``), which needs a smaller symbol than byte mode. That only
/// works — and only gives a URL equivalent to [`claim_url`]'s — when the base is a plain
/// scheme and host without a case-sensitive path or query, and when the server routes
/// `/K/` like `/k/` ([`GiftCode::from_claim_url`] accepts both). Only ASCII letters are
/// changed.
#[must_use]
pub fn claim_url_qr(base_url: &str, code: &GiftCode) -> String {
    claim_url(base_url, code).to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(a: u8, b: u8) -> u8 {
        QUASIGROUP[usize::from(a)][usize::from(b)]
    }

    // ---- Exhaustive proofs over the quasigroup ----------------------------------------

    #[test]
    fn table_is_the_documented_product_of_gf4_and_z5() {
        // Independent GF(4) multiplication: carry-less product reduced by x² + x + 1.
        fn gf4_mul(a: u8, b: u8) -> u8 {
            let mut product = 0u8;
            for bit in 0..2 {
                if b >> bit & 1 == 1 {
                    product ^= a << bit;
                }
            }
            if product & 0b100 != 0 {
                product ^= 0b111;
            }
            product
        }
        // x² + x + 1 is irreducible: every non-zero element has an inverse, and α = x
        // generates the three non-zero elements.
        for a in 1..4u8 {
            assert_eq!((1..4u8).filter(|b| gf4_mul(a, *b) == 1).count(), 1, "inverse of {a}");
        }
        assert_eq!(
            [gf4_mul(2, 1), gf4_mul(2, 2), gf4_mul(2, 3)],
            [2, 3, 1],
            "α, α², α³ = 1 must be the three non-zero elements"
        );
        for v in 0..4u8 {
            assert_eq!(GF4_MUL_ALPHA[usize::from(v)], gf4_mul(2, v));
        }
        for a in 0..20u8 {
            for b in 0..20u8 {
                let h = gf4_mul(2, a / 5) ^ (b / 5);
                let l = (2 * (a % 5) + b % 5) % 5;
                assert_eq!(op(a, b), 5 * h + l, "{a} ∘ {b}");
                assert!(op(a, b) < 20);
            }
        }
    }

    #[test]
    fn quasigroup_is_a_latin_square() {
        // Every row and every column is a permutation of 0..20. Rows: a wrong symbol
        // changes the state. Columns: two different states stay different after any
        // further symbol.
        for fixed in 0..20u8 {
            let mut row = [false; 20];
            let mut column = [false; 20];
            for other in 0..20u8 {
                row[usize::from(op(fixed, other))] = true;
                column[usize::from(op(other, fixed))] = true;
            }
            assert!(row.iter().all(|seen| *seen), "row {fixed} is not a permutation");
            assert!(column.iter().all(|seen| *seen), "column {fixed} is not a permutation");
        }
    }

    #[test]
    fn quasigroup_is_totally_anti_symmetric() {
        // (c ∘ x) ∘ y = (c ∘ y) ∘ x  ⇒  x = y, for all 20³ triples: swapping two
        // different adjacent symbols always changes the state.
        let mut checked = 0u32;
        for c in 0..20u8 {
            for x in 0..20u8 {
                for y in 0..20u8 {
                    if x != y {
                        assert_ne!(op(op(c, x), y), op(op(c, y), x), "c={c} x={x} y={y}");
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 20 * 20 * 19);
        // …and x ∘ y = y ∘ x ⇒ x = y (anti-symmetry proper).
        for x in 0..20u8 {
            for y in 0..20u8 {
                if x != y {
                    assert_ne!(op(x, y), op(y, x), "x={x} y={y}");
                }
            }
        }
    }

    #[test]
    fn substitutions_and_adjacent_transpositions_survive_any_continuation() {
        // The two required guarantees at the level of the recurrence: whatever the state
        // `c` before the error and whatever symbol follows, the states stay different —
        // and so, by induction on the remaining symbols, does the final state.
        for c in 0..20u8 {
            for x in 0..20u8 {
                for y in 0..20u8 {
                    if x == y {
                        continue;
                    }
                    let (good, bad) = (op(c, x), op(c, y));
                    assert_ne!(good, bad, "substitution");
                    let (good2, bad2) = (op(op(c, x), y), op(op(c, y), x));
                    assert_ne!(good2, bad2, "transposition");
                    for next in 0..20u8 {
                        assert_ne!(op(good, next), op(bad, next));
                        assert_ne!(op(good2, next), op(bad2, next));
                    }
                }
            }
        }
    }

    #[test]
    fn jump_transpositions_and_twin_errors_change_the_state() {
        // abc -> cba for every state, every middle symbol and every a ≠ c (20⁴ cases).
        for c in 0..20u8 {
            for x in 0..20u8 {
                for y in 0..20u8 {
                    for z in 0..20u8 {
                        if x != z {
                            assert_ne!(op(op(op(c, x), y), z), op(op(op(c, z), y), x));
                        }
                    }
                }
            }
        }
        // aa -> bb for every state and every a ≠ b.
        for c in 0..20u8 {
            for x in 0..20u8 {
                for y in 0..20u8 {
                    if x != y {
                        assert_ne!(op(op(c, x), x), op(op(c, y), y), "c={c} {x}{x}->{y}{y}");
                    }
                }
            }
        }
    }

    #[test]
    fn documented_limits_of_the_check_are_real() {
        // Honesty check on the module documentation: jump twin errors (aca -> bcb) and
        // swaps three places apart are NOT always detected. If this ever fails, the
        // documentation undersells the scheme and should be updated.
        let mut undetected_jump_twins = 0u32;
        let mut undetected_distance_3 = 0u32;
        for c in 0..20u8 {
            for x in 0..20u8 {
                for y in 0..20u8 {
                    if x == y {
                        continue;
                    }
                    for m in 0..20u8 {
                        if op(op(op(c, x), m), x) == op(op(op(c, y), m), y) {
                            undetected_jump_twins += 1;
                        }
                        let straight = op(op(op(op(c, x), m), m), y);
                        let swapped = op(op(op(op(c, y), m), m), x);
                        if straight == swapped {
                            undetected_distance_3 += 1;
                        }
                    }
                }
            }
        }
        assert!(undetected_jump_twins > 0);
        assert!(undetected_distance_3 > 0);
    }

    #[test]
    fn every_state_has_exactly_one_check_symbol() {
        let mut used = [false; 20];
        for state in 0..20u8 {
            let check = check_symbol_for(state);
            assert_eq!(op(state, check), 0, "state {state}");
            assert_eq!((0..20u8).filter(|s| op(state, *s) == 0).count(), 1);
            used[usize::from(check)] = true;
        }
        // Different states need different check symbols: the check symbol is as
        // unpredictable as the state.
        assert!(used.iter().all(|seen| *seen));
    }

    // ---- Alphabet and sampling --------------------------------------------------------

    #[test]
    fn alphabet_is_twenty_consonants() {
        assert_eq!(GiftCode::ALPHABET, "BCDFGHJKLMNPQRSTVWXZ");
        assert_eq!(GiftCode::ALPHABET.as_bytes(), ALPHABET_BYTES);
        for excluded in "AEIOUY0123456789".chars() {
            assert!(!GiftCode::ALPHABET.contains(excluded), "{excluded}");
            assert_eq!(symbol_value(excluded), None);
            assert_eq!(symbol_value(excluded.to_ascii_lowercase()), None);
        }
        let letters: Vec<char> = GiftCode::ALPHABET.chars().collect();
        assert!(letters.windows(2).all(|w| w[0] < w[1]), "alphabetical and without repeats");
        for (value, c) in GiftCode::ALPHABET.chars().enumerate() {
            assert_eq!(symbol_value(c), Some(value as u8));
            assert_eq!(symbol_value(c.to_ascii_lowercase()), Some(value as u8));
            assert_eq!(symbol_char(value as u8), c);
        }
        assert_eq!(symbol_value('É'), None);
        assert_eq!(symbol_value('Ｂ'), None, "full-width letters are not accepted");
        assert_eq!((GiftCode::LEN, GiftCode::DATA_SYMBOLS, PairCode::LEN), (12, 11, 6));
        assert_eq!(GiftCode::SPACE, 204_800_000_000_000);
        assert_eq!(PairCode::SPACE, 64_000_000);
        // About 47.5 bits (checked at compile time: both sides are constants).
        const { assert!(GiftCode::SPACE > 1 << 47 && GiftCode::SPACE < 1 << 48) };
    }

    #[test]
    fn known_vectors() {
        // Payload 0: all data symbols are B, every state is 0, the check symbol is B.
        let zero = GiftCode::from_payload(0).expect("in range");
        assert_eq!(zero.compact(), "BBBBBBBBBBBB");
        assert_eq!(zero.grouped(), "BBBB-BBBB-BBBB");
        // Payload 1: last data symbol C = (h 0, l 1). State (0, 1); the check symbol must
        // have l with 2·1 + l ≡ 0 (mod 5), i.e. l = 3, h = 0: the letter F.
        assert_eq!(GiftCode::from_payload(1).expect("in range").compact(), "BBBBBBBBBBCF");
        // Payload 5: last data symbol H = (h 1, l 0). State (1, 0); the check symbol
        // needs h = α·1 = 2, l = 0: value 10, the letter N.
        assert_eq!(GiftCode::from_payload(5).expect("in range").compact(), "BBBBBBBBBBHN");
        // The largest payload: eleven Z.
        let max = GiftCode::from_payload(GiftCode::SPACE - 1).expect("in range");
        assert_eq!(&max.compact()[..11], "ZZZZZZZZZZZ");
        assert_eq!(max.payload(), GiftCode::SPACE - 1);
        assert_eq!(GiftCode::from_payload(GiftCode::SPACE), Err(CodeError::PayloadTooLarge));
        assert_eq!(GiftCode::from_payload(u64::MAX), Err(CodeError::PayloadTooLarge));
    }

    #[test]
    fn sampling_is_rejection_then_reduction() {
        // An accepted chunk: the value itself, reduced.
        assert_eq!(sample_below(&[0, 0, 0, 0, 0, 0, 0, 7], 20), Ok(7));
        assert_eq!(sample_below(&[0, 0, 0, 0, 0, 0, 1, 0], 20), Ok(256 % 20));
        // Bytes beyond the first accepted chunk are ignored, as is an incomplete tail.
        assert_eq!(sample_below(&[0, 0, 0, 0, 0, 0, 0, 7, 9, 9, 9], 20), Ok(7));
        assert_eq!(sample_below(&[0, 0, 0, 0, 0, 0, 0, 7, 1, 1, 1, 1, 1, 1, 1, 1], 20), Ok(7));
        // A rejected chunk (above the largest multiple of the modulus) falls through to
        // the next one…
        let modulus = GiftCode::SPACE;
        let limit = u64::MAX - u64::MAX % modulus;
        let rejected = limit.to_be_bytes();
        let mut two_chunks = rejected.to_vec();
        two_chunks.extend_from_slice(&42u64.to_be_bytes());
        assert_eq!(sample_below(&two_chunks, modulus), Ok(42));
        // …and, when there is none, is reduced rather than refused.
        assert_eq!(sample_below(&rejected, modulus), Ok(limit % modulus));
        assert_eq!(sample_below(&u64::MAX.to_be_bytes(), modulus), Ok(u64::MAX % modulus));
        // The acceptance region is a whole number of copies of 0..modulus.
        assert_eq!(limit % modulus, 0);
        assert!(u64::MAX - limit < modulus);
        for len in 0..8 {
            assert_eq!(
                sample_below(&vec![1u8; len], modulus),
                Err(CodeError::NotEnoughRandomness { needed: 8, got: len })
            );
        }
    }

    #[test]
    fn random_bytes_give_the_documented_code() {
        let bytes = 123_456_789_012_345u64.to_be_bytes();
        let code = GiftCode::from_random_bytes(&bytes).expect("8 bytes");
        assert_eq!(code.payload(), 123_456_789_012_345);
        assert_eq!(GiftCode::from_payload(123_456_789_012_345), Ok(code));
        // Values above 20¹¹ wrap around.
        let wrapped = (GiftCode::SPACE + 5).to_be_bytes();
        assert_eq!(GiftCode::from_random_bytes(&wrapped).expect("8 bytes").payload(), 5);
        // Deterministic, and insensitive to extra bytes once a chunk is accepted.
        let mut longer = bytes.to_vec();
        longer.extend_from_slice(&[0xff; 9]);
        assert_eq!(GiftCode::from_random_bytes(&longer), Ok(code));
        assert_eq!(
            GiftCode::from_random_bytes(&[1, 2, 3, 4, 5, 6, 7]),
            Err(CodeError::NotEnoughRandomness { needed: 8, got: 7 })
        );
        assert_eq!(
            GiftCode::from_random_bytes(&[]),
            Err(CodeError::NotEnoughRandomness { needed: 8, got: 0 })
        );
    }

    // ---- Text forms and parsing -------------------------------------------------------

    #[test]
    fn display_debug_and_string_forms() {
        let code = GiftCode::from_payload(98_765_432_109_876).expect("in range");
        let compact = code.compact();
        let grouped = code.grouped();
        assert_eq!(compact.len(), 12);
        assert_eq!(grouped.len(), 14);
        assert_eq!(grouped.replace('-', ""), compact);
        assert_eq!(grouped.match_indices('-').map(|(i, _)| i).collect::<Vec<_>>(), [4, 9]);
        assert_eq!(code.to_string(), grouped);
        assert!(compact.chars().all(|c| GiftCode::ALPHABET.contains(c)));
        assert_eq!(compact.chars().last(), Some(code.check_symbol()));
        assert_eq!(compact.parse::<GiftCode>(), Ok(code));
        assert_eq!(code.symbols().len(), 12);
        // Debug must not reveal the code.
        let debug = format!("{code:?}");
        assert!(debug.starts_with(&format!("GiftCode({}", &compact[..2])));
        assert!(!debug.contains(&compact[2..]));
        assert!(!debug.contains(&grouped[3..]));
    }

    #[test]
    fn lenient_parsing() {
        let code = GiftCode::from_payload(11_223_344_556_677).expect("in range");
        let compact = code.compact();
        let grouped = code.grouped();
        for typed in [
            compact.clone(),
            grouped.clone(),
            compact.to_lowercase(),
            grouped.to_lowercase(),
            format!("  {grouped}\n"),
            grouped.replace('-', " "),
            grouped.replace('-', "."),
            grouped.replace('-', "_"),
            grouped.replace('-', "\u{2013}"),
            grouped.replace('-', "\u{2011}"),
            grouped.replace('-', "\u{a0}"),
            compact.chars().map(|c| format!("{c}-")).collect::<String>(),
            compact.chars().map(|c| format!(" {c} ")).collect::<String>(),
            format!("\u{feff}{compact}\u{200b}"),
            format!("{}\t{}", &compact[..6], &compact[6..]),
        ] {
            assert_eq!(GiftCode::parse(&typed), Ok(code), "typed {typed:?}");
        }
    }

    #[test]
    fn parse_errors_are_specific() {
        let code = GiftCode::from_payload(123_456_789).expect("in range");
        let compact = code.compact();
        let length = |found| Err(CodeError::Length { expected: 12, found });
        assert_eq!(GiftCode::parse(""), length(0));
        assert_eq!(GiftCode::parse(" - . _ "), length(0));
        assert_eq!(GiftCode::parse(&compact[..11]), length(11));
        assert_eq!(GiftCode::parse(&format!("{compact}B")), length(13));
        assert_eq!(GiftCode::parse(&format!("{compact}{compact}")), length(24));
        // Vowels, digits and anything else are named.
        assert_eq!(GiftCode::parse("BCDF-GHJK-LMNA"), Err(CodeError::InvalidChar('A')));
        assert_eq!(GiftCode::parse("bcdf-ghjk-lmny"), Err(CodeError::InvalidChar('y')));
        assert_eq!(GiftCode::parse("BCDF-GHJK-LMN0"), Err(CodeError::InvalidChar('0')));
        assert_eq!(GiftCode::parse("BCDF/GHJK/LMNP"), Err(CodeError::InvalidChar('/')));
        assert_eq!(GiftCode::parse("BCDÉ-GHJK-LMNP"), Err(CodeError::InvalidChar('É')));
        assert_eq!(GiftCode::parse("ＢCDF-GHJK-LMNP"), Err(CodeError::InvalidChar('Ｂ')));
        assert_eq!(GiftCode::parse("BCDF\u{0}GHJKLMNP"), Err(CodeError::InvalidChar('\u{0}')));
        assert_eq!(
            GiftCode::parse("https://ebk.example/k/BCDFGHJKLMNP"),
            Err(CodeError::InvalidChar(':'))
        );
        // Invalid characters are reported even when the length is wrong too.
        assert_eq!(GiftCode::parse("BC!"), Err(CodeError::InvalidChar('!')));
        // Hostile sizes.
        assert_eq!(GiftCode::parse(&"B".repeat(1_000_000)), length(1_000_000));
        assert_eq!(GiftCode::parse(&"-".repeat(1_000_000)), length(0));
        // A wrong check symbol is a typo, and says so.
        let wrong_check = if code.check_symbol() == 'Z' { 'X' } else { 'Z' };
        let typo = format!("{}{wrong_check}", &compact[..11]);
        assert_eq!(GiftCode::parse(&typo), Err(CodeError::Check));
        assert!(CodeError::Check.to_string().contains("mistyped"));
        assert_eq!(
            CodeError::Length { expected: 12, found: 3 }.to_string(),
            "the code has 12 letters, found 3"
        );
        assert!(CodeError::InvalidChar('0').to_string().contains("BCDFGHJKLMNPQRSTVWXZ"));
    }

    #[test]
    fn claim_urls() {
        let code = GiftCode::from_payload(987_654_321).expect("in range");
        let compact = code.compact();
        let expected = format!("https://ebk.example/k/{compact}");
        assert_eq!(claim_url("https://ebk.example", &code), expected);
        assert_eq!(claim_url("https://ebk.example/", &code), expected);
        assert_eq!(claim_url("https://ebk.example///", &code), expected);
        assert_eq!(
            claim_url("http://localhost:8080/base", &code),
            format!("http://localhost:8080/base/k/{compact}")
        );
        assert_eq!(claim_url("", &code), format!("/k/{compact}"));

        // The QR form is the same URL in upper case, entirely within the QR alphanumeric
        // character set.
        let qr = claim_url_qr("https://ebk.example", &code);
        assert_eq!(qr, format!("HTTPS://EBK.EXAMPLE/K/{compact}"));
        assert_eq!(qr.to_lowercase(), expected.to_lowercase());
        assert!(qr.chars().all(|c| "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:".contains(c)));
        assert_eq!(
            claim_url_qr("http://localhost:8080/", &code),
            format!("HTTP://LOCALHOST:8080/K/{compact}")
        );
        // Only ASCII is upper-cased.
        assert_eq!(
            claim_url_qr("https://é.example", &code),
            format!("HTTPS://é.EXAMPLE/K/{compact}")
        );

        for url in [
            expected.clone(),
            qr,
            format!("https://ebk.example/k/{compact}?utm=1"),
            format!("https://ebk.example/k/{compact}#top"),
            format!("https://ebk.example/k/{compact}/"),
            format!("https://ebk.example/k/{compact}?device=kobo"),
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
            Err(CodeError::Length { expected: 12, found: 0 })
        );
        assert_eq!(
            GiftCode::from_claim_url("https://ebk.example/k/é"),
            Err(CodeError::InvalidChar('é'))
        );
    }

    #[test]
    fn typed_input_is_a_code_or_a_claim_url_with_or_without_scheme() {
        let code = GiftCode::from_payload(123_456_789_012).expect("in range");
        let compact = code.compact();
        let lower = compact.to_lowercase();
        let spaced = code.grouped().replace('-', " ");
        for input in [
            compact.clone(),
            lower.clone(),
            spaced.clone(),
            format!("  {}  ", code.grouped()),
            format!("https://ebk.example/k/{compact}"),
            format!("ebk.example/k/{compact}"),
            format!("EBK.EXAMPLE/K/{compact}"),
            format!("HTTPS://EBK.EXAMPLE/K/{compact}"),
            format!("/k/{compact}"),
            format!("/K/{lower}"),
            format!("ebk.example/k/{lower}"),
            format!("  ebk.example/k/{spaced}  "),
            format!("https://ebk.example/k/{}?device=kobo#x", code.grouped()),
            format!("http://localhost:8080/base/k/{compact}/"),
        ] {
            assert_eq!(GiftCode::parse_typed(&input), Ok(code), "input {input:?}");
        }

        // Typo versus not-a-code semantics are kept, inside a URL too.
        let mut typo = compact.clone().into_bytes();
        typo.swap(0, 1);
        let typo = String::from_utf8(typo).expect("ascii");
        if typo != compact {
            assert_eq!(GiftCode::parse_typed(&typo), Err(CodeError::Check));
            assert_eq!(
                GiftCode::parse_typed(&format!("ebk.example/k/{typo}")),
                Err(CodeError::Check)
            );
        }
        assert_eq!(
            GiftCode::parse_typed("ebk.example/k/BCDF"),
            Err(CodeError::Length { expected: 12, found: 4 })
        );
        assert_eq!(GiftCode::parse_typed("ebk.example/"), Err(CodeError::NotAClaimUrl));
        assert_eq!(GiftCode::parse_typed("ebk.example/k/é"), Err(CodeError::InvalidChar('é')));
        // Without a `/`, the errors of `parse` are unchanged.
        assert_eq!(GiftCode::parse_typed("ebk.example"), Err(CodeError::InvalidChar('e')));
        assert_eq!(
            GiftCode::parse_typed("BCDF"),
            Err(CodeError::Length { expected: 12, found: 4 })
        );
        assert_eq!(GiftCode::parse_typed(""), Err(CodeError::Length { expected: 12, found: 0 }));

        // Hostile input never panics.
        for input in ["/", "//k//", "/k/", "/k/\u{0}", "é/k/é", "/k/%%%", &"/k/".repeat(500)] {
            assert!(GiftCode::parse_typed(input).is_err(), "{input:?}");
        }
    }

    #[test]
    fn gift_code_serialises_as_the_compact_string() {
        let code = GiftCode::from_payload(55_555_555_555).expect("in range");
        let json = serde_json::to_string(&code).expect("serialises");
        assert_eq!(json, format!("\"{}\"", code.compact()));
        assert_eq!(serde_json::from_str::<GiftCode>(&json).expect("parses"), code);
        // Deserialisation is as lenient as parsing…
        let grouped = format!("\"{}\"", code.grouped().to_lowercase());
        assert_eq!(serde_json::from_str::<GiftCode>(&grouped).expect("parses"), code);
        // …and as strict about the check symbol.
        let wrong_check = if code.check_symbol() == 'B' { 'C' } else { 'B' };
        let bad = format!("\"{}{wrong_check}\"", &code.compact()[..11]);
        assert!(serde_json::from_str::<GiftCode>(&bad).is_err());
        assert!(serde_json::from_str::<GiftCode>("12345").is_err());
        assert!(serde_json::from_str::<GiftCode>("null").is_err());
    }

    #[test]
    fn unfortunate_letters_are_flagged() {
        // Build codes around a given data prefix; the check symbol follows automatically.
        let with_prefix = |prefix: &str| {
            let mut payload = 0u64;
            for i in 0..GiftCode::DATA_SYMBOLS {
                let c = prefix.chars().nth(i).unwrap_or('B');
                payload = payload * 20 + u64::from(symbol_value(c).expect("alphabet letter"));
            }
            GiftCode::from_payload(payload).expect("in range")
        };
        assert!(with_prefix("KKK").has_unfortunate_letters());
        assert!(with_prefix("BBWTF").has_unfortunate_letters());
        assert!(with_prefix("BBBBBBBBTCH").has_unfortunate_letters());
        assert!(!with_prefix("BCDFGHJKLMN").has_unfortunate_letters());
        assert!(!GiftCode::from_payload(0).expect("in range").has_unfortunate_letters());
    }

    // ---- PairCode ---------------------------------------------------------------------

    #[test]
    fn pair_code_forms_and_parsing() {
        let pair = PairCode::from_random_bytes(&12_345_678u64.to_be_bytes()).expect("8 bytes");
        let compact = pair.compact();
        let grouped = pair.grouped();
        assert_eq!(compact.len(), 6);
        assert_eq!(grouped.len(), 7);
        assert_eq!(grouped.find('-'), Some(3));
        assert_eq!(pair.to_string(), grouped);
        assert!(compact.chars().all(|c| GiftCode::ALPHABET.contains(c)));
        assert_eq!(value_of(&pair.symbols()), 12_345_678);
        for typed in [
            compact.clone(),
            grouped.clone(),
            grouped.to_lowercase(),
            format!(" {} {} ", &compact[..3], &compact[3..]),
            compact.chars().map(|c| format!("{c}.")).collect::<String>(),
        ] {
            assert_eq!(PairCode::parse(&typed), Ok(pair), "typed {typed:?}");
            assert_eq!(typed.parse::<PairCode>(), Ok(pair));
        }
        // No check symbol: every six-letter string is a pair code.
        assert!(PairCode::parse("BBBBBB").is_ok());
        assert!(PairCode::parse("zzz-zzz").is_ok());
        assert_eq!(PairCode::parse("BBBBB"), Err(CodeError::Length { expected: 6, found: 5 }));
        assert_eq!(PairCode::parse("BBBBBBB"), Err(CodeError::Length { expected: 6, found: 7 }));
        assert_eq!(PairCode::parse("BBB-BB1"), Err(CodeError::InvalidChar('1')));
        assert_eq!(PairCode::parse("BBB-BBA"), Err(CodeError::InvalidChar('A')));
        assert_eq!(PairCode::parse(""), Err(CodeError::Length { expected: 6, found: 0 }));
        assert_eq!(
            PairCode::from_random_bytes(&[0; 7]),
            Err(CodeError::NotEnoughRandomness { needed: 8, got: 7 })
        );
        // Smallest and largest.
        assert_eq!(PairCode::from_random_bytes(&[0; 8]).expect("8 bytes").compact(), "BBBBBB");
        let last = (PairCode::SPACE - 1).to_be_bytes();
        assert_eq!(PairCode::from_random_bytes(&last).expect("8 bytes").compact(), "ZZZZZZ");
        // Debug is redacted; serde uses the compact string.
        let debug = format!("{pair:?}");
        assert!(debug.starts_with("PairCode(") && !debug.contains(&compact[1..]));
        let json = serde_json::to_string(&pair).expect("serialises");
        assert_eq!(json, format!("\"{compact}\""));
        assert_eq!(serde_json::from_str::<PairCode>(&json).expect("parses"), pair);
        assert!(serde_json::from_str::<PairCode>("\"BBB\"").is_err());
    }
}
