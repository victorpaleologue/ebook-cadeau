//! A small, strictly bounded tokenizer for PDF content streams.
//!
//! `lopdf` ships a content parser, but it silently drops whatever follows the first
//! construct it cannot parse. A preflight must not approve a page whose tail it never read,
//! so this lexer reports syntax errors instead. It is iterative (no recursion), linear in
//! the input and keeps only what the interpreter needs: numbers, names and operators.
//! Strings, arrays and dictionaries are skipped as opaque operands.

use std::collections::VecDeque;

/// An operand preceding an operator.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Operand {
    /// An integer or real number.
    Number(f64),
    /// A name, without the leading slash, with `#xx` escapes decoded.
    Name(Vec<u8>),
    /// A string, array, dictionary, boolean or null (content not needed).
    Other,
}

/// The colour space entry of an inline image.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum InlineColorSpace {
    /// No `CS` entry.
    #[default]
    None,
    /// A name: a device space, its abbreviation, or a named resource.
    Name(Vec<u8>),
    /// `[/Indexed base …]` with the name of the base space (empty when it is not a name).
    Indexed(Vec<u8>),
    /// Any other array.
    Other,
}

/// The dictionary of an inline image (`BI … ID … EI`), as far as the checks need it.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct InlineImage {
    pub(crate) width: Option<i64>,
    pub(crate) height: Option<i64>,
    pub(crate) bits_per_component: Option<i64>,
    pub(crate) color_space: InlineColorSpace,
    pub(crate) image_mask: bool,
    pub(crate) filters: Vec<Vec<u8>>,
}

/// One step of a content stream.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token<'a> {
    /// An operator; its operands are available from [`Lexer::operands`].
    Operator(&'a [u8]),
    /// A complete inline image.
    InlineImage(InlineImage),
}

/// A syntax error, with the byte offset where it was detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LexError {
    pub(crate) offset: usize,
    pub(crate) what: &'static str,
}

/// Operands kept before an operator. Real operators take at most six (plus colour
/// components for `DeviceN`, at most 32); older ones are dropped.
const MAX_OPERANDS: usize = 64;
/// Maximum nesting of arrays and dictionaries inside an operand.
const MAX_NESTING: usize = 64;
/// Maximum number of entries read from an inline image dictionary.
const MAX_INLINE_ENTRIES: usize = 64;

pub(crate) struct Lexer<'a> {
    data: &'a [u8],
    pos: usize,
    operands: VecDeque<Operand>,
}

const fn is_whitespace(b: u8) -> bool {
    matches!(b, 0 | 9 | 10 | 12 | 13 | 32)
}

const fn is_delimiter(b: u8) -> bool {
    matches!(b, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

const fn is_regular(b: u8) -> bool {
    !is_whitespace(b) && !is_delimiter(b)
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0, operands: VecDeque::with_capacity(MAX_OPERANDS) }
    }

    /// The operands of the operator returned by the last call to [`Lexer::next_token`].
    pub(crate) fn operands(&self) -> &VecDeque<Operand> {
        &self.operands
    }

    /// The last `N` operands, when there are at least `N` and all are numbers.
    pub(crate) fn numbers<const N: usize>(&self) -> Option<[f64; N]> {
        let len = self.operands.len();
        let start = len.checked_sub(N)?;
        let mut out = [0.0_f64; N];
        for (slot, operand) in out.iter_mut().zip(self.operands.range(start..)) {
            match operand {
                Operand::Number(value) => *slot = *value,
                _ => return None,
            }
        }
        Some(out)
    }

    /// The last operand, when it is a name.
    pub(crate) fn last_name(&self) -> Option<&[u8]> {
        match self.operands.back()? {
            Operand::Name(name) => Some(name.as_slice()),
            _ => None,
        }
    }

    /// Reads up to the next operator (or inline image). `Ok(None)` at the end of the stream.
    pub(crate) fn next_token(&mut self) -> Result<Option<Token<'a>>, LexError> {
        self.operands.clear();
        loop {
            self.skip_whitespace_and_comments();
            let Some(&byte) = self.data.get(self.pos) else {
                if self.operands.is_empty() {
                    return Ok(None);
                }
                return Err(self.error("operands without an operator at the end of the stream"));
            };
            match byte {
                b'/' => {
                    let name = self.read_name();
                    self.push(Operand::Name(name));
                }
                b'(' => {
                    self.skip_literal_string()?;
                    self.push(Operand::Other);
                }
                b'<' | b'[' => {
                    self.skip_composite()?;
                    self.push(Operand::Other);
                }
                b')' | b'>' | b']' | b'{' | b'}' => {
                    return Err(self.error("unexpected delimiter"));
                }
                _ => {
                    let word = self.read_regular();
                    if let Some(value) = parse_number(word) {
                        self.push(Operand::Number(value));
                    } else if matches!(word, b"true" | b"false" | b"null") {
                        self.push(Operand::Other);
                    } else if word == b"BI" {
                        let image = self.read_inline_image()?;
                        return Ok(Some(Token::InlineImage(image)));
                    } else {
                        return Ok(Some(Token::Operator(word)));
                    }
                }
            }
        }
    }

    fn push(&mut self, operand: Operand) {
        if self.operands.len() == MAX_OPERANDS {
            self.operands.pop_front();
        }
        self.operands.push_back(operand);
    }

    fn error(&self, what: &'static str) -> LexError {
        LexError { offset: self.pos, what }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(&byte) = self.data.get(self.pos) {
            if is_whitespace(byte) {
                self.pos += 1;
            } else if byte == b'%' {
                while let Some(&b) = self.data.get(self.pos) {
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    /// Reads a run of regular characters starting at the current position (at least one byte
    /// is consumed even if it is not regular, so the lexer always makes progress).
    fn read_regular(&mut self) -> &'a [u8] {
        let start = self.pos;
        while self.data.get(self.pos).is_some_and(|&b| is_regular(b)) {
            self.pos += 1;
        }
        if self.pos == start {
            self.pos = (start + 1).min(self.data.len());
        }
        &self.data[start..self.pos]
    }

    /// Reads a name; the current byte is the slash.
    fn read_name(&mut self) -> Vec<u8> {
        self.pos += 1;
        let start = self.pos;
        while self.data.get(self.pos).is_some_and(|&b| is_regular(b)) {
            self.pos += 1;
        }
        decode_name(&self.data[start..self.pos])
    }

    /// Skips `( … )` with nested parentheses and backslash escapes.
    fn skip_literal_string(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        let mut depth = 0_usize;
        while let Some(&byte) = self.data.get(self.pos) {
            self.pos += 1;
            match byte {
                b'\\' => self.pos += 1,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        self.pos = self.data.len();
        Err(LexError { offset: start, what: "unterminated string" })
    }

    /// Skips `< … >`; the current byte is the opening angle bracket.
    fn skip_hex_string(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        self.pos += 1;
        while let Some(&byte) = self.data.get(self.pos) {
            self.pos += 1;
            if byte == b'>' {
                return Ok(());
            }
        }
        Err(LexError { offset: start, what: "unterminated hexadecimal string" })
    }

    /// Skips one array, dictionary or hexadecimal string, including everything nested in it.
    fn skip_composite(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        // Closers still expected: b']' for arrays, b'>' for dictionaries.
        let mut open: Vec<u8> = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            let Some(&byte) = self.data.get(self.pos) else {
                return Err(LexError { offset: start, what: "unterminated array or dictionary" });
            };
            match byte {
                b'[' => {
                    open.push(b']');
                    self.pos += 1;
                }
                b'<' if self.data.get(self.pos + 1) == Some(&b'<') => {
                    open.push(b'>');
                    self.pos += 2;
                }
                b'<' => self.skip_hex_string()?,
                b']' => {
                    if open.pop() != Some(b']') {
                        return Err(self.error("unbalanced ]"));
                    }
                    self.pos += 1;
                }
                b'>' => {
                    if self.data.get(self.pos + 1) != Some(&b'>') || open.pop() != Some(b'>') {
                        return Err(self.error("unbalanced >"));
                    }
                    self.pos += 2;
                }
                b'(' => self.skip_literal_string()?,
                b'/' => {
                    self.read_name();
                }
                b')' | b'{' | b'}' => return Err(self.error("unexpected delimiter")),
                _ => {
                    self.read_regular();
                }
            }
            if open.is_empty() {
                return Ok(());
            }
            if open.len() > MAX_NESTING {
                return Err(self.error("arrays or dictionaries nested too deeply"));
            }
        }
    }

    /// Parses the remainder of an inline image; `BI` has just been consumed.
    fn read_inline_image(&mut self) -> Result<InlineImage, LexError> {
        let start = self.pos;
        let mut image = InlineImage::default();
        let mut entries = 0_usize;
        loop {
            self.skip_whitespace_and_comments();
            let Some(&byte) = self.data.get(self.pos) else {
                return Err(LexError { offset: start, what: "inline image without ID" });
            };
            if byte != b'/' {
                let word = self.read_regular();
                if word == b"ID" {
                    break;
                }
                return Err(self.error("expected a key or ID in an inline image"));
            }
            entries += 1;
            if entries > MAX_INLINE_ENTRIES {
                return Err(self.error("too many entries in an inline image"));
            }
            let key = self.read_name();
            self.skip_whitespace_and_comments();
            let value = self.read_inline_value()?;
            match (key.as_slice(), value) {
                (b"W" | b"Width", InlineValue::Number(n)) => image.width = whole(n),
                (b"H" | b"Height", InlineValue::Number(n)) => image.height = whole(n),
                (b"BPC" | b"BitsPerComponent", InlineValue::Number(n)) => {
                    image.bits_per_component = whole(n);
                }
                (b"IM" | b"ImageMask", InlineValue::Bool(b)) => image.image_mask = b,
                (b"CS" | b"ColorSpace", InlineValue::Name(name)) => {
                    image.color_space = InlineColorSpace::Name(name);
                }
                (b"CS" | b"ColorSpace", InlineValue::Array(items)) => {
                    image.color_space = match items.as_slice() {
                        [InlineValue::Name(kind), rest @ ..]
                            if matches!(kind.as_slice(), b"Indexed" | b"I") =>
                        {
                            match rest.first() {
                                Some(InlineValue::Name(base)) => {
                                    InlineColorSpace::Indexed(base.clone())
                                }
                                _ => InlineColorSpace::Indexed(Vec::new()),
                            }
                        }
                        _ => InlineColorSpace::Other,
                    };
                }
                (b"F" | b"Filter", InlineValue::Name(name)) => image.filters = vec![name],
                (b"F" | b"Filter", InlineValue::Array(items)) => {
                    image.filters = items
                        .into_iter()
                        .filter_map(|item| match item {
                            InlineValue::Name(name) => Some(name),
                            _ => None,
                        })
                        .collect();
                }
                _ => {}
            }
        }
        // One white-space character separates ID from the data (CR LF counts as one).
        match self.data.get(self.pos) {
            Some(b'\r') if self.data.get(self.pos + 1) == Some(&b'\n') => self.pos += 2,
            Some(&b) if is_whitespace(b) => self.pos += 1,
            _ => {}
        }
        self.skip_inline_data(&image, start)?;
        Ok(image)
    }

    /// Positions the lexer after the `EI` that ends the inline image data.
    fn skip_inline_data(&mut self, image: &InlineImage, start: usize) -> Result<(), LexError> {
        // Unfiltered data has a known length: prefer it, since raw samples may contain "EI".
        if let Some(length) = raw_inline_length(image)
            && let Some(end) = self.pos.checked_add(length).filter(|&end| end <= self.data.len())
        {
            let mut after = end;
            while self.data.get(after).is_some_and(|&b| is_whitespace(b)) {
                after += 1;
            }
            if self.data[after..].starts_with(b"EI")
                && self.data.get(after + 2).is_none_or(|&b| !is_regular(b))
            {
                self.pos = after + 2;
                return Ok(());
            }
        }
        // Otherwise scan for EI between white space.
        let mut i = self.pos;
        while i + 1 < self.data.len() {
            if self.data[i] == b'E'
                && self.data[i + 1] == b'I'
                && (i == self.pos || is_whitespace(self.data[i - 1]))
                && self.data.get(i + 2).is_none_or(|&b| !is_regular(b))
            {
                self.pos = i + 2;
                return Ok(());
            }
            i += 1;
        }
        self.pos = self.data.len();
        Err(LexError { offset: start, what: "inline image without EI" })
    }

    /// Reads one value of an inline image dictionary.
    fn read_inline_value(&mut self) -> Result<InlineValue, LexError> {
        let Some(&byte) = self.data.get(self.pos) else {
            return Err(self.error("inline image entry without a value"));
        };
        match byte {
            b'/' => Ok(InlineValue::Name(self.read_name())),
            b'(' => {
                self.skip_literal_string()?;
                Ok(InlineValue::Other)
            }
            b'[' => self.read_inline_array(),
            b'<' => {
                self.skip_composite()?;
                Ok(InlineValue::Other)
            }
            b')' | b'>' | b']' | b'{' | b'}' => Err(self.error("unexpected delimiter")),
            _ => {
                let word = self.read_regular();
                Ok(match word {
                    b"true" => InlineValue::Bool(true),
                    b"false" => InlineValue::Bool(false),
                    _ => parse_number(word).map_or(InlineValue::Other, InlineValue::Number),
                })
            }
        }
    }

    /// Reads a flat array of an inline image dictionary: names and numbers are kept (up to
    /// a few), nested values are skipped.
    fn read_inline_array(&mut self) -> Result<InlineValue, LexError> {
        const MAX_ITEMS: usize = 16;
        let start = self.pos;
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            let Some(&byte) = self.data.get(self.pos) else {
                return Err(LexError { offset: start, what: "unterminated array" });
            };
            let item = match byte {
                b']' => {
                    self.pos += 1;
                    return Ok(InlineValue::Array(items));
                }
                b'/' => InlineValue::Name(self.read_name()),
                b'(' => {
                    self.skip_literal_string()?;
                    InlineValue::Other
                }
                b'[' | b'<' => {
                    self.skip_composite()?;
                    InlineValue::Other
                }
                b')' | b'>' | b'{' | b'}' => return Err(self.error("unexpected delimiter")),
                _ => {
                    let word = self.read_regular();
                    parse_number(word).map_or(InlineValue::Other, InlineValue::Number)
                }
            };
            if items.len() < MAX_ITEMS {
                items.push(item);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InlineValue {
    Number(f64),
    Name(Vec<u8>),
    Bool(bool),
    Array(Vec<InlineValue>),
    Other,
}

fn whole(value: f64) -> Option<i64> {
    // Truncation is safe: the value is whole and far inside the i64 range.
    #[allow(clippy::cast_possible_truncation)]
    (value.fract() == 0.0 && value.abs() < 1.0e12).then_some(value as i64)
}

/// Byte length of the data of an unfiltered inline image, when it can be computed.
fn raw_inline_length(image: &InlineImage) -> Option<usize> {
    if !image.filters.is_empty() {
        return None;
    }
    let width = usize::try_from(image.width?).ok()?;
    let height = usize::try_from(image.height?).ok()?;
    let (components, bits) = if image.image_mask {
        (1_usize, 1_usize)
    } else {
        let bits = usize::try_from(image.bits_per_component?).ok()?;
        let components = match &image.color_space {
            InlineColorSpace::Name(name) => match name.as_slice() {
                b"G" | b"DeviceGray" | b"CalGray" => 1,
                b"RGB" | b"DeviceRGB" | b"CalRGB" => 3,
                b"CMYK" | b"DeviceCMYK" => 4,
                _ => return None,
            },
            InlineColorSpace::Indexed(_) => 1,
            InlineColorSpace::None | InlineColorSpace::Other => return None,
        };
        (components, bits)
    };
    let row_bits = width.checked_mul(components)?.checked_mul(bits)?;
    row_bits.div_ceil(8).checked_mul(height)
}

/// Parses a PDF number: optional signs, digits with at most one decimal point.
fn parse_number(word: &[u8]) -> Option<f64> {
    const MAX_LEN: usize = 64;
    if word.is_empty() || word.len() > MAX_LEN {
        return None;
    }
    let digits_start = word.iter().position(|&b| b != b'+' && b != b'-')?;
    let (signs, body) = word.split_at(digits_start);
    if body.is_empty() {
        return None;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for &b in body {
        match b {
            b'0'..=b'9' => seen_digit = true,
            b'.' if !seen_dot => seen_dot = true,
            _ => return None,
        }
    }
    if !seen_digit {
        return None;
    }
    let text = std::str::from_utf8(body).ok()?;
    let magnitude: f64 = text.parse().ok()?;
    if !magnitude.is_finite() {
        return None;
    }
    // Producers have been seen writing "--5"; like viewers, read the first sign only.
    Some(if signs.first() == Some(&b'-') { -magnitude } else { magnitude })
}

/// Decodes the `#xx` escapes of a name.
fn decode_name(raw: &[u8]) -> Vec<u8> {
    if !raw.contains(&b'#') {
        return raw.to_vec();
    }
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'#'
            && let (Some(hi), Some(lo)) = (
                raw.get(i + 1).and_then(|&b| hex_value(b)),
                raw.get(i + 2).and_then(|&b| hex_value(b)),
            )
        {
            out.push(hi * 16 + lo);
            i += 3;
            continue;
        }
        out.push(raw[i]);
        i += 1;
    }
    out
}

const fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Collects `(operator, numeric operands)` pairs; inline images appear as `"BI"`.
    fn ops(data: &[u8]) -> Result<Vec<(String, Vec<f64>)>, LexError> {
        let mut lexer = Lexer::new(data);
        let mut out = Vec::new();
        while let Some(token) = lexer.next_token()? {
            let numbers = lexer
                .operands()
                .iter()
                .filter_map(|o| match o {
                    Operand::Number(n) => Some(*n),
                    _ => None,
                })
                .collect();
            let name = match token {
                Token::Operator(op) => String::from_utf8_lossy(op).into_owned(),
                Token::InlineImage(_) => "BI".to_string(),
            };
            out.push((name, numbers));
        }
        Ok(out)
    }

    #[test]
    fn reads_operators_and_numbers() {
        let got = ops(b"q 1 0 0 -1.5 +.25 -3. cm\n0.1 .2 0.3 0.4 k % comment ) (\nf* Q").unwrap();
        assert_eq!(
            got,
            vec![
                ("q".to_string(), vec![]),
                ("cm".to_string(), vec![1.0, 0.0, 0.0, -1.5, 0.25, -3.0]),
                ("k".to_string(), vec![0.1, 0.2, 0.3, 0.4]),
                ("f*".to_string(), vec![]),
                ("Q".to_string(), vec![]),
            ]
        );
    }

    #[test]
    fn strings_arrays_and_dictionaries_are_opaque_operands() {
        let data = b"BT /F1 12 Tf [(a\\)b) -120 <0041> [1 [2]] ] TJ (x(y)z) Tj \
                     /OC << /A [1 2] /B << /C (d) >> >> BDC ET";
        let mut lexer = Lexer::new(data);
        let mut seen = Vec::new();
        while let Some(Token::Operator(op)) = lexer.next_token().unwrap() {
            seen.push((String::from_utf8_lossy(op).into_owned(), lexer.operands().len()));
        }
        assert_eq!(
            seen,
            vec![
                ("BT".to_string(), 0),
                ("Tf".to_string(), 2),
                ("TJ".to_string(), 1),
                ("Tj".to_string(), 1),
                ("BDC".to_string(), 2),
                ("ET".to_string(), 0),
            ]
        );
    }

    #[test]
    fn names_decode_hash_escapes() {
        let mut lexer = Lexer::new(b"/A#20B#2F /Plain /Bad#zz gs");
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Operator(b"gs")));
        let names: Vec<&Operand> = lexer.operands().iter().collect();
        assert_eq!(
            names,
            vec![
                &Operand::Name(b"A B/".to_vec()),
                &Operand::Name(b"Plain".to_vec()),
                &Operand::Name(b"Bad#zz".to_vec()),
            ]
        );
        assert_eq!(lexer.last_name(), Some(&b"Bad#zz"[..]));
    }

    #[test]
    fn numbers_helper_needs_enough_numeric_operands() {
        let mut lexer = Lexer::new(b"1 2 3 4 re /N 5 re 1 2 re");
        lexer.next_token().unwrap();
        assert_eq!(lexer.numbers::<4>(), Some([1.0, 2.0, 3.0, 4.0]));
        assert_eq!(lexer.numbers::<2>(), Some([3.0, 4.0]));
        lexer.next_token().unwrap();
        assert_eq!(lexer.numbers::<2>(), None);
        lexer.next_token().unwrap();
        assert_eq!(lexer.numbers::<4>(), None);
    }

    #[test]
    fn syntax_errors_are_reported_not_swallowed() {
        for (data, what) in [
            (&b"q (never closed"[..], "unterminated string"),
            (b"[1 2 3", "unterminated array or dictionary"),
            (b"<< /A 1 ", "unterminated array or dictionary"),
            (b"<0A0B", "unterminated hexadecimal string"),
            (b"1 0 0 rg ) f", "unexpected delimiter"),
            (b"q ] Q", "unexpected delimiter"),
            (b"[ << ] >> TJ", "unbalanced ]"),
            (b"[ > ] TJ", "unbalanced >"),
            (b"0 0 m 10 10", "operands without an operator at the end of the stream"),
            (b"BI /W 2 /H 2", "inline image without ID"),
            (b"BI /W 2 /H 2 /BPC 8 /CS /G ID abcd", "inline image without EI"),
            (b"BI 12 ID x EI", "expected a key or ID in an inline image"),
        ] {
            let err = ops(data).expect_err(&String::from_utf8_lossy(data));
            assert_eq!(err.what, what, "{}", String::from_utf8_lossy(data));
        }
    }

    #[test]
    fn deep_nesting_is_rejected_without_recursion() {
        let mut data = vec![b'['; 100_000];
        data.extend(std::iter::repeat_n(b']', 100_000));
        data.extend_from_slice(b" TJ");
        let err = ops(&data).unwrap_err();
        assert_eq!(err.what, "arrays or dictionaries nested too deeply");
        // 60 levels are fine.
        let mut ok = vec![b'['; 60];
        ok.extend(std::iter::repeat_n(b']', 60));
        ok.extend_from_slice(b" TJ");
        assert_eq!(ops(&ok).unwrap().len(), 1);
    }

    #[test]
    fn operand_flood_keeps_only_the_last_operands() {
        let mut data = Vec::new();
        for i in 0..10_000 {
            data.extend_from_slice(format!("{i} ").as_bytes());
        }
        data.extend_from_slice(b"re");
        let mut lexer = Lexer::new(&data);
        lexer.next_token().unwrap();
        assert_eq!(lexer.operands().len(), MAX_OPERANDS);
        assert_eq!(lexer.numbers::<2>(), Some([9998.0, 9999.0]));
    }

    #[test]
    fn inline_image_with_raw_data_containing_ei() {
        // 4x1 grey pixels whose bytes spell " EI " — the declared size must win.
        let data = b"q BI /W 4 /H 1 /BPC 8 /CS /G ID  EI \nEI Q 1 g";
        let mut lexer = Lexer::new(data);
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Operator(b"q")));
        let Some(Token::InlineImage(image)) = lexer.next_token().unwrap() else {
            panic!("expected an inline image");
        };
        assert_eq!(
            (image.width, image.height, image.bits_per_component),
            (Some(4), Some(1), Some(8))
        );
        assert_eq!(image.color_space, InlineColorSpace::Name(b"G".to_vec()));
        assert!(image.filters.is_empty());
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Operator(b"Q")));
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Operator(b"g")));
        assert_eq!(lexer.next_token().unwrap(), None);
    }

    #[test]
    fn inline_image_with_filter_scans_for_ei() {
        let data = b"BI /W 10 /H 10 /BPC 8 /CS [/I /RGB 1 <000000ffffff>] /F [/A85 /Fl] \
                     /DP [null << /Predictor 12 >>] /IM false ID Gb\"0~> \nEI\nn";
        let mut lexer = Lexer::new(data);
        let Some(Token::InlineImage(image)) = lexer.next_token().unwrap() else {
            panic!("expected an inline image");
        };
        assert_eq!(image.color_space, InlineColorSpace::Indexed(b"RGB".to_vec()));
        assert_eq!(image.filters, vec![b"A85".to_vec(), b"Fl".to_vec()]);
        assert!(!image.image_mask);
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Operator(b"n")));
    }

    #[test]
    fn image_mask_length_is_one_bit_per_pixel() {
        // 9x2 mask = 2 bytes per row = 4 bytes of data.
        let data = b"BI /IM true /W 9 /H 2 ID \xff\x80\xff\x80 EI f";
        let mut lexer = Lexer::new(data);
        let Some(Token::InlineImage(image)) = lexer.next_token().unwrap() else {
            panic!("expected an inline image");
        };
        assert!(image.image_mask);
        assert_eq!(lexer.next_token().unwrap(), Some(Token::Operator(b"f")));
    }

    #[test]
    fn odd_numbers() {
        assert_eq!(parse_number(b"--5"), Some(-5.0));
        assert_eq!(parse_number(b"+7"), Some(7.0));
        assert_eq!(parse_number(b"."), None);
        assert_eq!(parse_number(b"-"), None);
        assert_eq!(parse_number(b"1.2.3"), None);
        assert_eq!(parse_number(b"1e5"), None);
        assert_eq!(parse_number(b"12abc"), None);
        assert_eq!(parse_number(&[b'9'; 65]), None);
        assert_eq!(parse_number(&[b'9'; 60]).map(f64::is_finite), Some(true));
    }

    #[test]
    fn arbitrary_bytes_never_panic_and_always_terminate() {
        // A deterministic pseudo-random walk over byte soup, biased towards delimiters.
        let alphabet = b"()<>[]{}/% \n\r\t0123456789.-+qQBIDEfgkreTJ#\\\x00\xff";
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        for round in 0..2_000 {
            let len = 1 + (round % 97);
            let mut data = Vec::with_capacity(len);
            for _ in 0..len {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                data.push(alphabet[(state % alphabet.len() as u64) as usize]);
            }
            let mut lexer = Lexer::new(&data);
            let mut steps = 0;
            while let Ok(Some(_)) = lexer.next_token() {
                steps += 1;
                assert!(steps <= data.len(), "lexer must consume input on every token");
            }
        }
    }
}
