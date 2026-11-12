//! Bounded outlines for PostScript-flavoured fonts (`CFF ` and `CFF2` tables).
//!
//! # Why this module exists
//!
//! A Type 2 charstring may call subroutines, which may call subroutines, ten levels
//! deep. Nothing in the format limits how often: a font of 750 bytes whose subroutines
//! each call the next one eight times asks for 8^10 ≈ 10^9 calls, and `ttf-parser`
//! (which this crate otherwise relies on) executes them all — it limits the nesting
//! depth but not the amount of work. Drawing one letter, or merely loading the font
//! (the capital height is measured on "H"), then hangs.
//!
//! # What it does
//!
//! The glyph's charstring is **flattened** by a small interpreter of our own: every
//! subroutine call is replaced by the subroutine's body, under a budget of
//! [`MAX_CHARSTRING_STEPS`] operators and operands and [`MAX_FLAT_BYTES`] of output. The
//! flattened charstring — which contains no call at all — is then wrapped in a minimal
//! synthetic CFF/CFF2 table *without any subroutine* and handed to `ttf-parser`, which
//! still does all the path construction (and CFF2 blending) but, by construction, cannot
//! do more work than the flattened charstring is long. A glyph that exceeds the budget,
//! or whose charstring is malformed, has no outline.
//!
//! The flattener only has to be *correct for well-formed fonts*: whatever it gets wrong
//! on a hostile font can change what is drawn, never how long it takes.
//!
//! `seac` (the deprecated "accented character" form of `endchar`) is resolved here as
//! well, one level deep: base and accent glyphs are flattened and drawn separately.

use std::collections::BTreeMap;

use ttf_parser::{GlyphId, NormalizedCoordinate, OutlineBuilder, Rect};

/// Most operators and operands interpreted while flattening one glyph (subroutine
/// bodies included). Real glyphs need a few hundred to a few thousand.
pub(crate) const MAX_CHARSTRING_STEPS: u32 = 1 << 18;
/// Longest flattened charstring, in bytes.
pub(crate) const MAX_FLAT_BYTES: usize = 1 << 18;
/// Subroutine nesting limit of the Type 2 charstring format.
const NESTING_LIMIT: u8 = 10;

/// Why a glyph could not be flattened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    /// The charstring (or a table it needs) is malformed or uses something unsupported.
    Invalid,
    /// The step or size budget ran out.
    TooComplex,
}

// ---------------------------------------------------------------------------------------
// Low-level readers
// ---------------------------------------------------------------------------------------

fn be_u16(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at.checked_add(2)?)?;
    Some(u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]))
}

fn be_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at.checked_add(4)?)?;
    Some(u32::from_be_bytes([*bytes.first()?, *bytes.get(1)?, *bytes.get(2)?, *bytes.get(3)?]))
}

/// A CFF `INDEX`: an array of variable-sized objects.
#[derive(Debug, Clone, Copy, Default)]
struct Index<'a> {
    count: u32,
    offset_size: usize,
    /// `count + 1` offsets of `offset_size` bytes each.
    offsets: &'a [u8],
    /// The data the offsets point into; offset 1 is its first byte.
    objects: &'a [u8],
}

impl<'a> Index<'a> {
    /// Parses the INDEX starting at `at`. The count is 16 bits wide in CFF, 32 in CFF2.
    /// Returns the INDEX and the position just after it.
    fn parse(data: &'a [u8], at: usize, wide_count: bool) -> Option<(Self, usize)> {
        let (count, mut at) = if wide_count {
            (be_u32(data, at)?, at.checked_add(4)?)
        } else {
            (u32::from(be_u16(data, at)?), at.checked_add(2)?)
        };
        if count == 0 {
            return Some((Self::default(), at));
        }
        let offset_size = usize::from(*data.get(at)?);
        at += 1;
        if !(1..=4).contains(&offset_size) {
            return None;
        }
        let table_len = usize::try_from(count).ok()?.checked_add(1)?.checked_mul(offset_size)?;
        let offsets = data.get(at..at.checked_add(table_len)?)?;
        at += table_len;
        let index = Self { count, offset_size, offsets, objects: data.get(at..)? };
        let end = at.checked_add(index.offset(count)?.checked_sub(1)?)?;
        (end <= data.len()).then_some((index, end))
    }

    fn offset(&self, i: u32) -> Option<usize> {
        let at = usize::try_from(i).ok()?.checked_mul(self.offset_size)?;
        let bytes = self.offsets.get(at..at.checked_add(self.offset_size)?)?;
        Some(bytes.iter().fold(0usize, |acc, &b| (acc << 8) | usize::from(b)))
    }

    fn get(&self, i: u32) -> Option<&'a [u8]> {
        if i >= self.count {
            return None;
        }
        let start = self.offset(i)?.checked_sub(1)?;
        let end = self.offset(i.checked_add(1)?)?.checked_sub(1)?;
        self.objects.get(start..end)
    }
}

/// Walks a DICT and calls `visit(operator, operands)` for every entry. Two-byte operators
/// are reported as `1200 + second byte`; real-number operands as NaN (none of the
/// entries read here uses them).
fn for_each_dict_entry(data: &[u8], mut visit: impl FnMut(u16, &[f64])) {
    let mut operands: Vec<f64> = Vec::new();
    let mut at = 0usize;
    while let Some(&b0) = data.get(at) {
        at += 1;
        match b0 {
            0..=27 => {
                let operator = if b0 == 12 {
                    let Some(&b1) = data.get(at) else { return };
                    at += 1;
                    1200 + u16::from(b1)
                } else {
                    u16::from(b0)
                };
                visit(operator, &operands);
                operands.clear();
            }
            28 => {
                let Some(v) = be_u16(data, at) else { return };
                operands.push(f64::from(v as i16));
                at += 2;
            }
            29 => {
                let Some(v) = be_u32(data, at) else { return };
                operands.push(f64::from(v as i32));
                at += 4;
            }
            30 => {
                // A real number: nibbles up to and including the terminator 0xf.
                loop {
                    let Some(&b) = data.get(at) else { return };
                    at += 1;
                    if b >> 4 == 0xf || b & 0xf == 0xf {
                        break;
                    }
                }
                operands.push(f64::NAN);
            }
            32..=246 => operands.push(f64::from(b0) - 139.0),
            247..=254 => {
                let Some(&b1) = data.get(at) else { return };
                at += 1;
                operands.push(two_byte_number(b0, b1));
            }
            _ => return, // reserved
        }
        // CFF2 allows up to 513 operands (blend); anything longer is not a DICT.
        if operands.len() > 513 {
            return;
        }
    }
}

/// The value of a two-byte number of a charstring or DICT (`b0` in 247..=254).
fn two_byte_number(b0: u8, b1: u8) -> f64 {
    if b0 < 251 {
        (f64::from(b0) - 247.0) * 256.0 + f64::from(b1) + 108.0
    } else {
        -(f64::from(b0) - 251.0) * 256.0 - f64::from(b1) - 108.0
    }
}

/// A DICT operand used as an offset or size: a non-negative integer.
fn as_offset(value: f64) -> Option<usize> {
    let ok = value >= 0.0 && value <= f64::from(u32::MAX) && value.fract() == 0.0;
    ok.then_some(value as usize)
}

/// A charstring operand used as an index or code: an integer in the `i32` range.
fn as_integer(value: f64) -> Option<i64> {
    let ok = value.fract() == 0.0 && value.abs() <= f64::from(i32::MAX);
    ok.then_some(value as i64)
}

// ---------------------------------------------------------------------------------------
// The font-level structures
// ---------------------------------------------------------------------------------------

/// Standard-encoding codes with a glyph: `(first code, last code, first string id)`
/// (Adobe Technical Note #5176, appendix B). Used by `seac` only.
const STANDARD_ENCODING_RANGES: [(u8, u8, u16); 14] = [
    (32, 126, 1),
    (161, 175, 96),
    (177, 180, 111),
    (182, 189, 115),
    (191, 191, 123),
    (193, 200, 124),
    (202, 203, 132),
    (205, 208, 134),
    (225, 225, 138),
    (227, 227, 139),
    (232, 235, 140),
    (241, 241, 144),
    (245, 245, 145),
    (248, 251, 146),
];

fn standard_encoding_sid(code: u8) -> u16 {
    STANDARD_ENCODING_RANGES
        .iter()
        .find(|&&(first, last, _)| (first..=last).contains(&code))
        .map_or(0, |&(first, _, sid)| sid + u16::from(code - first))
}

/// How glyph ids map to string ids (CFF only; needed for `seac`).
#[derive(Debug, Clone, Copy)]
enum Charset {
    /// The predefined ISOAdobe charset: glyph id = string id.
    IsoAdobe,
    /// The predefined Expert charsets, which contain no standard-encoding glyph.
    Expert,
    /// A charset table at this offset.
    Table(usize),
}

/// The parts of a `CFF ` or `CFF2` table needed to flatten charstrings.
#[derive(Debug, Clone)]
pub(crate) struct CffFont<'a> {
    data: &'a [u8],
    cff2: bool,
    global_subrs: Index<'a>,
    char_strings: Index<'a>,
    /// Private DICT `(offset, size)` of a non-CID CFF font.
    private: Option<(usize, usize)>,
    fd_array: Option<Index<'a>>,
    fd_select: Option<usize>,
    charset: Charset,
    /// Offset of the CFF2 variation store (its 16-bit length field).
    vstore: Option<usize>,
    /// The local subroutines of the Font DICTs looked up so far.
    locals: BTreeMap<u32, Option<Index<'a>>>,
}

impl<'a> CffFont<'a> {
    /// Parses a `CFF ` table (version 1).
    pub(crate) fn parse_cff(data: &'a [u8]) -> Option<Self> {
        if *data.first()? != 1 {
            return None;
        }
        let header_size = usize::from(*data.get(2)?);
        let (_names, at) = Index::parse(data, header_size, false)?;
        let (top_dicts, at) = Index::parse(data, at, false)?;
        let (_strings, at) = Index::parse(data, at, false)?;
        let (global_subrs, _) = Index::parse(data, at, false)?;

        let mut font = Self::empty(data, false, global_subrs);
        let mut char_strings = None;
        for_each_dict_entry(top_dicts.get(0)?, |operator, operands| match (operator, operands) {
            (15, [offset]) => {
                font.charset = match as_offset(*offset) {
                    Some(0) | None => Charset::IsoAdobe,
                    Some(1 | 2) => Charset::Expert,
                    Some(offset) => Charset::Table(offset),
                };
            }
            (17, [offset]) => char_strings = as_offset(*offset),
            (18, [size, offset]) => font.private = as_offset(*offset).zip(as_offset(*size)),
            (1236, [offset]) => {
                font.fd_array = as_offset(*offset)
                    .and_then(|at| Index::parse(data, at, false))
                    .map(|(index, _)| index);
            }
            (1237, [offset]) => font.fd_select = as_offset(*offset),
            _ => {}
        });
        font.char_strings = Index::parse(data, char_strings.filter(|&at| at > 0)?, false)?.0;
        Some(font)
    }

    /// Parses a `CFF2` table.
    pub(crate) fn parse_cff2(data: &'a [u8]) -> Option<Self> {
        if *data.first()? != 2 {
            return None;
        }
        let header_size = usize::from(*data.get(2)?).max(5);
        let top_dict_len = usize::from(be_u16(data, 3)?);
        let top_dict_end = header_size.checked_add(top_dict_len)?;
        let top_dict = data.get(header_size..top_dict_end)?;
        let (global_subrs, _) = Index::parse(data, top_dict_end, true)?;

        let mut font = Self::empty(data, true, global_subrs);
        let mut char_strings = None;
        for_each_dict_entry(top_dict, |operator, operands| match (operator, operands) {
            (17, [offset]) => char_strings = as_offset(*offset),
            (24, [offset]) => font.vstore = as_offset(*offset),
            (1236, [offset]) => {
                font.fd_array = as_offset(*offset)
                    .and_then(|at| Index::parse(data, at, true))
                    .map(|(index, _)| index);
            }
            (1237, [offset]) => font.fd_select = as_offset(*offset),
            _ => {}
        });
        font.char_strings = Index::parse(data, char_strings.filter(|&at| at > 0)?, true)?.0;
        Some(font)
    }

    fn empty(data: &'a [u8], cff2: bool, global_subrs: Index<'a>) -> Self {
        Self {
            data,
            cff2,
            global_subrs,
            char_strings: Index::default(),
            private: None,
            fd_array: None,
            fd_select: None,
            charset: Charset::IsoAdobe,
            vstore: None,
            locals: BTreeMap::new(),
        }
    }

    /// The Font DICT a glyph belongs to (`FDSelect`); 0 when there is no `FDSelect`.
    fn font_dict_of(&self, glyph: u16) -> Option<u32> {
        let Some(at) = self.fd_select else { return Some(0) };
        let data = self.data;
        let glyph = u32::from(glyph);
        // Formats 3 and 4: `count` ranges of (first glyph, font dict), sorted by first
        // glyph and closed by a sentinel glyph id. `first(count)` is the sentinel.
        let in_ranges = |count: usize,
                         first: &dyn Fn(usize) -> Option<u32>,
                         font_dict: &dyn Fn(usize) -> Option<u32>| {
            // Binary search for the last range that starts at or before the glyph.
            let (mut low, mut high) = (0usize, count);
            while low < high {
                let middle = low + (high - low) / 2;
                if first(middle)? <= glyph {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            let range = low.checked_sub(1)?;
            (glyph < first(range + 1)?).then(|| font_dict(range)).flatten()
        };
        match *data.get(at)? {
            0 => Some(u32::from(*data.get(at.checked_add(1)?.checked_add(glyph as usize)?)?)),
            3 => {
                let entry = |i: usize| at.checked_add(3)?.checked_add(i.checked_mul(3)?);
                in_ranges(
                    usize::from(be_u16(data, at.checked_add(1)?)?),
                    &|i| be_u16(data, entry(i)?).map(u32::from),
                    &|i| data.get(entry(i)?.checked_add(2)?).map(|&fd| u32::from(fd)),
                )
            }
            4 => {
                let entry = |i: usize| at.checked_add(5)?.checked_add(i.checked_mul(6)?);
                in_ranges(
                    usize::try_from(be_u32(data, at.checked_add(1)?)?).ok()?,
                    &|i| be_u32(data, entry(i)?),
                    &|i| be_u16(data, entry(i)?.checked_add(4)?).map(u32::from),
                )
            }
            _ => None,
        }
    }

    /// The local subroutines that apply to a glyph, if it has any. They are looked up
    /// once per Font DICT; reading the DICTs is charged to `steps` by their size.
    fn local_subrs(&mut self, glyph: u16, steps: &mut u32) -> Result<Option<Index<'a>>, Stop> {
        let Some(font_dict) = self.font_dict_of(glyph) else { return Ok(None) };
        if let Some(&known) = self.locals.get(&font_dict) {
            return Ok(known);
        }
        let data = self.data;
        let mut charge = |dict: &[u8]| {
            let cost = u32::try_from(dict.len() / 8 + 1).unwrap_or(u32::MAX);
            *steps = steps.checked_sub(cost).ok_or(Stop::TooComplex)?;
            Ok(())
        };
        let private = match self.fd_array {
            Some(fd_array) => match fd_array.get(font_dict) {
                Some(font_dict) => {
                    charge(font_dict)?;
                    let mut private = None;
                    for_each_dict_entry(font_dict, |operator, operands| {
                        if let (18, [size, offset]) = (operator, operands) {
                            private = as_offset(*offset).zip(as_offset(*size));
                        }
                    });
                    private
                }
                None => None,
            },
            None => self.private,
        };
        let mut subrs = None;
        if let Some((at, len)) = private {
            let end = at.saturating_add(len);
            if let Some(private) = data.get(at..end) {
                charge(private)?;
                let mut offset = None;
                for_each_dict_entry(private, |operator, operands| {
                    if let (19, [value]) = (operator, operands) {
                        offset = as_offset(*value);
                    }
                });
                // "The local subroutines offset is relative to the beginning of the
                // Private DICT data."
                subrs = offset
                    .and_then(|offset| at.checked_add(offset))
                    .and_then(|at| Index::parse(data, at, self.cff2))
                    .map(|(index, _)| index);
            }
        }
        self.locals.insert(font_dict, subrs);
        Ok(subrs)
    }

    /// The glyph that the standard-encoding `code` of a `seac` refers to. Searching the
    /// charset is charged to `steps`.
    fn seac_glyph(&self, code: f64, steps: &mut u32) -> Result<u16, Stop> {
        let code = as_integer(code).and_then(|v| u8::try_from(v).ok()).ok_or(Stop::Invalid)?;
        let sid = standard_encoding_sid(code);
        if sid == 0 {
            return Ok(0);
        }
        match self.charset {
            Charset::IsoAdobe => Ok(sid),
            Charset::Expert => Err(Stop::Invalid),
            Charset::Table(at) => self.glyph_of_sid(at, sid, steps),
        }
    }

    /// Looks a string id up in a charset table (formats 0, 1 and 2). Glyph 0 is implicit.
    fn glyph_of_sid(&self, at: usize, sid: u16, steps: &mut u32) -> Result<u16, Stop> {
        let data = self.data;
        let glyphs = self.char_strings.count.min(u32::from(u16::MAX));
        let (format, entry_len) = match data.get(at) {
            Some(0) => (0, 2),
            Some(1) => (1, 3),
            Some(2) => (2, 4),
            _ => return Err(Stop::Invalid),
        };
        let mut entry = at + 1; // `at` is inside the table
        let mut glyph = 1u32;
        while glyph < glyphs {
            *steps = steps.checked_sub(1).ok_or(Stop::TooComplex)?;
            let first = u32::from(be_u16(data, entry).ok_or(Stop::Invalid)?);
            let left = match format {
                0 => Some(0),
                1 => data.get(entry + 2).map(|&n| u32::from(n)),
                _ => be_u16(data, entry + 2).map(u32::from),
            };
            let left = left.ok_or(Stop::Invalid)?;
            if (first..=first + left).contains(&u32::from(sid)) {
                return u16::try_from(glyph + (u32::from(sid) - first)).map_err(|_| Stop::Invalid);
            }
            glyph += left + 1;
            entry += entry_len; // bounded by the table: the reads above have succeeded
        }
        Err(Stop::Invalid)
    }

    /// The raw CFF2 variation store, with its 16-bit length prefix.
    fn vstore_bytes(&self) -> Option<&'a [u8]> {
        let at = self.vstore?;
        let len = usize::from(be_u16(self.data, at)?);
        let end = at.checked_add(2)?.checked_add(len)?.min(self.data.len());
        self.data.get(at..end)
    }

    /// Number of variation regions of the CFF2 `ItemVariationData` number `vsindex` —
    /// the number of deltas per value that a `blend` operator consumes.
    fn region_count(&self, vsindex: u16) -> Option<usize> {
        // ItemVariationStore: format, region list offset (u32), data count, data offsets.
        let store = self.vstore_bytes()?.get(2..)?;
        if vsindex >= be_u16(store, 6)? {
            return None;
        }
        let subtable = usize::try_from(be_u32(store, 8 + 4 * usize::from(vsindex))?).ok()?;
        // ItemVariationData: item count, word delta count, region index count.
        Some(usize::from(be_u16(store, subtable.checked_add(4)?)?))
    }

    /// Outlines `glyph` into `builder` after flattening its charstring; `steps` is the
    /// remaining budget (see [`MAX_CHARSTRING_STEPS`]) and is reduced by what was used.
    ///
    /// Returns the bounding box, or `Ok(None)` for a glyph that draws nothing.
    pub(crate) fn outline(
        &mut self,
        glyph: u16,
        coordinates: &[NormalizedCoordinate],
        steps: &mut u32,
        builder: &mut dyn OutlineBuilder,
    ) -> Result<Option<Rect>, Stop> {
        let flat = self.flatten(glyph, steps)?;
        let own = self.outline_flat(&flat.charstring, coordinates, builder);
        let Some(seac) = flat.seac else { return Ok(own) };

        // An accented character: the base glyph as it is, the accent moved by (dx, dy).
        let base = self.seac_glyph(seac.base, steps)?;
        let accent = self.seac_glyph(seac.accent, steps)?;
        let base = self.flatten(base, steps)?;
        let accent = self.flatten(accent, steps)?;
        if base.seac.is_some() || accent.seac.is_some() {
            return Err(Stop::Invalid); // nested accents: not supported
        }
        let base_box = self.outline_flat(&base.charstring, coordinates, builder);
        let mut moved = Translated { inner: builder, dx: seac.dx as f32, dy: seac.dy as f32 };
        let accent_box = self.outline_flat(&accent.charstring, coordinates, &mut moved).map(|r| {
            let shift = |v: i16, d: f64| (f64::from(v) + d).clamp(-32768.0, 32767.0) as i16;
            Rect {
                x_min: shift(r.x_min, seac.dx),
                y_min: shift(r.y_min, seac.dy),
                x_max: shift(r.x_max, seac.dx),
                y_max: shift(r.y_max, seac.dy),
            }
        });
        Ok([own, base_box, accent_box].into_iter().flatten().reduce(|a, b| Rect {
            x_min: a.x_min.min(b.x_min),
            y_min: a.y_min.min(b.y_min),
            x_max: a.x_max.max(b.x_max),
            y_max: a.y_max.max(b.y_max),
        }))
    }

    /// Replaces every subroutine call of a glyph's charstring by the subroutine's body.
    fn flatten(&mut self, glyph: u16, steps: &mut u32) -> Result<Flat, Stop> {
        let body = self.char_strings.get(u32::from(glyph)).ok_or(Stop::Invalid)?;
        let locals = self.local_subrs(glyph, steps)?;
        let mut flattener = Flattener {
            font: self,
            locals,
            out: Vec::new(),
            stack: Vec::new(),
            stems: 0,
            have_width: false,
            // `ttf-parser` starts every CFF2 charstring with the regions of item 0.
            regions: if self.cff2 { self.region_count(0).ok_or(Stop::Invalid)? } else { 0 },
            steps_left: *steps,
            seac: None,
            ended: false,
        };
        let result = flattener.run(body, 0);
        *steps = flattener.steps_left;
        result?;
        if !self.cff2 && !flattener.ended {
            return Err(Stop::Invalid); // a CFF charstring must finish with endchar
        }
        Ok(Flat { charstring: flattener.out, seac: flattener.seac })
    }

    /// Lets `ttf-parser` draw a flattened charstring from a synthetic table that has no
    /// subroutines, no usable `seac` charset and exactly one glyph.
    fn outline_flat(
        &self,
        charstring: &[u8],
        coordinates: &[NormalizedCoordinate],
        builder: &mut dyn OutlineBuilder,
    ) -> Option<Rect> {
        if self.cff2 {
            let table = synthetic_cff2(charstring, self.vstore_bytes());
            ttf_parser::cff2::Table::parse(&table)?.outline(coordinates, GlyphId(0), builder).ok()
        } else {
            let table = synthetic_cff(charstring);
            ttf_parser::cff::Table::parse(&table)?.outline(GlyphId(0), builder).ok()
        }
    }
}

/// A minimal `CFF ` table holding one charstring: no subroutines, no Private DICT, and
/// the predefined *Expert* charset, in which no `seac` code resolves to a glyph.
fn synthetic_cff(charstring: &[u8]) -> Vec<u8> {
    const TOP_DICT_LEN: u8 = 12;
    let mut table = vec![1, 0, 4, 1]; // header: major, minor, header size, offset size
    table.extend_from_slice(&[0, 1, 1, 1, 2, b'A']); // Name INDEX: one name, "A"
    table.extend_from_slice(&[0, 1, 1, 1, 1 + TOP_DICT_LEN]); // Top DICT INDEX: one DICT
    // The CharStrings INDEX follows the Top DICT, the String INDEX and the Global Subr
    // INDEX (both empty: two bytes each).
    let char_strings_at = table.len() + usize::from(TOP_DICT_LEN) + 4;
    table.push(29);
    table.extend_from_slice(&1u32.to_be_bytes());
    table.push(15); // charset: 1 = Expert
    table.push(29);
    table.extend_from_slice(&(char_strings_at as u32).to_be_bytes());
    table.push(17); // CharStrings
    table.extend_from_slice(&[0, 0, 0, 0]); // String INDEX, Global Subr INDEX
    table.extend_from_slice(&[0, 1, 4]); // CharStrings INDEX: one object, 4-byte offsets
    table.extend_from_slice(&1u32.to_be_bytes());
    table.extend_from_slice(&(charstring.len() as u32 + 1).to_be_bytes());
    table.extend_from_slice(charstring);
    table
}

/// A minimal `CFF2` table holding one charstring, no subroutines, and a copy of the
/// font's variation store (needed by `blend`).
fn synthetic_cff2(charstring: &[u8], vstore: Option<&[u8]>) -> Vec<u8> {
    const HEADER_LEN: usize = 5;
    let vstore = vstore.unwrap_or_default();
    let top_dict_len: usize = if vstore.is_empty() { 6 } else { 12 };
    // Header, Top DICT, Global Subr INDEX (a zero count), variation store, CharStrings.
    let vstore_at = HEADER_LEN + top_dict_len + 4;
    let char_strings_at = vstore_at + vstore.len();
    let mut table = vec![2, 0, HEADER_LEN as u8, 0, top_dict_len as u8];
    table.push(29);
    table.extend_from_slice(&(char_strings_at as u32).to_be_bytes());
    table.push(17); // CharStrings
    if !vstore.is_empty() {
        table.push(29);
        table.extend_from_slice(&(vstore_at as u32).to_be_bytes());
        table.push(24); // vstore
    }
    table.extend_from_slice(&[0, 0, 0, 0]); // Global Subr INDEX
    table.extend_from_slice(vstore);
    table.extend_from_slice(&[0, 0, 0, 1, 4]); // CharStrings INDEX: one object
    table.extend_from_slice(&1u32.to_be_bytes());
    table.extend_from_slice(&(charstring.len() as u32 + 1).to_be_bytes());
    table.extend_from_slice(charstring);
    table
}

/// Forwards an outline moved by `(dx, dy)` font units.
struct Translated<'b> {
    inner: &'b mut dyn OutlineBuilder,
    dx: f32,
    dy: f32,
}

impl OutlineBuilder for Translated<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.inner.move_to(x + self.dx, y + self.dy);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.inner.line_to(x + self.dx, y + self.dy);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.inner.quad_to(x1 + self.dx, y1 + self.dy, x + self.dx, y + self.dy);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (dx, dy) = (self.dx, self.dy);
        self.inner.curve_to(x1 + dx, y1 + dy, x2 + dx, y2 + dy, x + dx, y + dy);
    }

    fn close(&mut self) {
        self.inner.close();
    }
}

// ---------------------------------------------------------------------------------------
// The flattener
// ---------------------------------------------------------------------------------------

/// A flattened charstring, and the accent composition its `endchar` asked for.
struct Flat {
    charstring: Vec<u8>,
    seac: Option<Seac>,
}

/// The operands of a `seac`-style `endchar`: accent offset and standard-encoding codes.
struct Seac {
    dx: f64,
    dy: f64,
    base: f64,
    accent: f64,
}

/// A number on the argument stack.
struct Operand {
    value: f64,
    /// Where its literal starts in the output. `None` once a `blend` has replaced the
    /// literal by a computed value, which can then not be taken back out of the output.
    at: Option<usize>,
}

/// The operators that only consume their operands (drawing and flex operators aside from
/// the ones handled individually).
const fn is_path_operator(op: u8) -> bool {
    matches!(op, 5..=8 | 24..=27 | 30 | 31)
}

struct Flattener<'f, 'a> {
    font: &'f CffFont<'a>,
    locals: Option<Index<'a>>,
    /// The flattened charstring. The operands on `stack` are always its last tokens: every
    /// operator that is written also clears the stack (`blend` aside, see [`Operand`]).
    out: Vec<u8>,
    stack: Vec<Operand>,
    /// Number of stem hints declared so far: decides the length of hint masks.
    stems: u32,
    /// CFF only: the glyph's width operand has been seen.
    have_width: bool,
    /// CFF2 only: number of regions of the current `vsindex`.
    regions: usize,
    steps_left: u32,
    seac: Option<Seac>,
    /// CFF only: `endchar` was executed.
    ended: bool,
}

impl<'a> Flattener<'_, 'a> {
    fn emit(&mut self, bytes: &[u8]) -> Result<(), Stop> {
        if self.out.len() + bytes.len() > MAX_FLAT_BYTES {
            return Err(Stop::TooComplex);
        }
        self.out.extend_from_slice(bytes);
        Ok(())
    }

    /// Pushes a number and copies its literal to the output.
    fn push(&mut self, value: f64, literal: &[u8]) -> Result<(), Stop> {
        let limit = if self.font.cff2 { 513 } else { 48 };
        if self.stack.len() >= limit {
            return Err(Stop::Invalid);
        }
        self.stack.push(Operand { value, at: Some(self.out.len()) });
        self.emit(literal)
    }

    /// Writes an operator that consumes the whole stack.
    fn operator(&mut self, bytes: &[u8]) -> Result<(), Stop> {
        self.stack.clear();
        self.emit(bytes)
    }

    /// Takes the top operand back out of the output (it must still be a literal there).
    fn take_literal(&mut self) -> Result<f64, Stop> {
        let operand = self.stack.pop().ok_or(Stop::Invalid)?;
        self.out.truncate(operand.at.ok_or(Stop::Invalid)?);
        Ok(operand.value)
    }

    fn run(&mut self, body: &'a [u8], depth: u8) -> Result<(), Stop> {
        let cff2 = self.font.cff2;
        let mut at = 0usize;
        while let Some(&op) = body.get(at) {
            self.steps_left = self.steps_left.checked_sub(1).ok_or(Stop::TooComplex)?;
            // A number literal of `len` bytes starting with `op`.
            let start = at;
            let number = |len: usize| body.get(start..start + len).ok_or(Stop::Invalid);
            at += 1;
            match op {
                28 => {
                    let literal = number(3)?;
                    let value = be_u16(literal, 1).ok_or(Stop::Invalid)? as i16;
                    self.push(f64::from(value), literal)?;
                    at += 2;
                }
                32..=246 => self.push(f64::from(op) - 139.0, number(1)?)?,
                247..=254 => {
                    let literal = number(2)?;
                    let b1 = *literal.get(1).ok_or(Stop::Invalid)?;
                    self.push(two_byte_number(op, b1), literal)?;
                    at += 1;
                }
                255 => {
                    let literal = number(5)?;
                    let fixed = be_u32(literal, 1).ok_or(Stop::Invalid)? as i32;
                    self.push(f64::from(fixed) / 65536.0, literal)?;
                    at += 4;
                }
                // hstem, vstem, hstemhm, vstemhm
                1 | 3 | 18 | 23 => {
                    self.count_stems();
                    self.operator(&[op])?;
                }
                // hintmask, cntrmask: followed by one bit per stem declared so far.
                19 | 20 => {
                    self.count_stems();
                    self.operator(&[op])?;
                    let mask_len = usize::try_from(self.stems.div_ceil(8)).unwrap_or(usize::MAX);
                    if mask_len > MAX_FLAT_BYTES {
                        return Err(Stop::TooComplex);
                    }
                    let mask = body.get(at..).unwrap_or_default();
                    let mask = mask.get(..mask_len).unwrap_or(mask);
                    self.emit(mask)?;
                    // A mask cut short by the end of the body is padded, so that what
                    // follows in the output is still read as operators.
                    let padding = vec![0; mask_len - mask.len()];
                    self.emit(&padding)?;
                    at += mask.len();
                }
                // vmoveto, rmoveto, hmoveto: one operand too many is the glyph's width.
                4 | 21 | 22 => {
                    if self.stack.len() == if op == 21 { 3 } else { 2 } {
                        self.have_width = true;
                    }
                    self.operator(&[op])?;
                }
                _ if is_path_operator(op) => self.operator(&[op])?,
                // The flex operators; every other two-byte operator is unsupported by
                // the outline code, so the glyph could not be drawn anyway.
                12 => {
                    let op2 = *body.get(at).ok_or(Stop::Invalid)?;
                    at += 1;
                    if !(34..=37).contains(&op2) {
                        return Err(Stop::Invalid);
                    }
                    self.operator(&[12, op2])?;
                }
                // callsubr, callgsubr
                10 | 29 => {
                    if depth >= NESTING_LIMIT {
                        return Err(Stop::Invalid);
                    }
                    let subrs = if op == 10 {
                        self.locals.ok_or(Stop::Invalid)?
                    } else {
                        self.font.global_subrs
                    };
                    let bias = match subrs.count {
                        0..1240 => 107,
                        1240..33900 => 1131,
                        _ => 32768,
                    };
                    let index = as_integer(self.take_literal()?).ok_or(Stop::Invalid)? + bias;
                    let index = u32::try_from(index).map_err(|_| Stop::Invalid)?;
                    self.run(subrs.get(index).ok_or(Stop::Invalid)?, depth + 1)?;
                    if self.ended {
                        return Ok(());
                    }
                }
                // return
                11 if !cff2 => return Ok(()),
                // endchar, possibly with the four operands of an accented character
                14 if !cff2 => {
                    let operands = self.stack.len();
                    if operands == 4 || (operands == 5 && !self.have_width) {
                        let accent = self.take_literal()?;
                        let base = self.take_literal()?;
                        let dy = self.take_literal()?;
                        let dx = self.take_literal()?;
                        self.seac = Some(Seac { dx, dy, base, accent });
                    }
                    self.operator(&[op])?;
                    self.ended = true;
                    return Ok(());
                }
                // vsindex: selects the variation data that `blend` uses
                15 if cff2 => {
                    let [index] = self.stack.as_slice() else { return Err(Stop::Invalid) };
                    let index = as_integer(index.value)
                        .and_then(|v| u16::try_from(v).ok())
                        .ok_or(Stop::Invalid)?;
                    self.regions = self.font.region_count(index).ok_or(Stop::Invalid)?;
                    self.operator(&[op])?;
                }
                // blend: n values, n x regions deltas and n itself become n values
                16 if cff2 => {
                    let n = self.stack.pop().ok_or(Stop::Invalid)?.value;
                    let n = as_integer(n).and_then(|v| usize::try_from(v).ok());
                    let n = n.ok_or(Stop::Invalid)?;
                    let deltas = n.checked_mul(self.regions).ok_or(Stop::Invalid)?;
                    let needed = deltas.checked_add(n).ok_or(Stop::Invalid)?;
                    if self.stack.len() < needed {
                        return Err(Stop::Invalid);
                    }
                    self.stack.truncate(self.stack.len() - deltas);
                    self.stack.iter_mut().for_each(|operand| operand.at = None);
                    self.emit(&[op])?;
                }
                _ => return Err(Stop::Invalid), // reserved
            }
        }
        Ok(())
    }

    /// Accounts for the stems declared by the operands of a stem or mask operator: two
    /// operands per stem; in CFF an odd operand out is the glyph's width.
    fn count_stems(&mut self) {
        let operands = self.stack.len();
        if !self.font.cff2 && operands % 2 == 1 {
            self.have_width = true;
        }
        self.stems = self.stems.saturating_add(u32::try_from(operands / 2).unwrap_or(u32::MAX));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collects an outline as text, e.g. `M 0 0 L 10 0 Z`.
    #[derive(Default)]
    struct Trace(String);

    impl OutlineBuilder for Trace {
        fn move_to(&mut self, x: f32, y: f32) {
            self.0.push_str(&format!("M {x} {y} "));
        }
        fn line_to(&mut self, x: f32, y: f32) {
            self.0.push_str(&format!("L {x} {y} "));
        }
        fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
            self.0.push_str(&format!("Q {x1} {y1} {x} {y} "));
        }
        fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
            self.0.push_str(&format!("C {x1} {y1} {x2} {y2} {x} {y} "));
        }
        fn close(&mut self) {
            self.0.push_str("Z ");
        }
    }

    fn index(items: &[&[u8]]) -> Vec<u8> {
        let mut out = (items.len() as u16).to_be_bytes().to_vec();
        if items.is_empty() {
            return out;
        }
        out.push(2);
        let mut offset = 1u16;
        out.extend_from_slice(&offset.to_be_bytes());
        for item in items {
            offset += item.len() as u16;
            out.extend_from_slice(&offset.to_be_bytes());
        }
        for item in items {
            out.extend_from_slice(item);
        }
        out
    }

    /// A CFF table with the given charstrings, global subroutines and local subroutines.
    fn cff(char_strings: &[&[u8]], globals: &[&[u8]], locals: &[&[u8]]) -> Vec<u8> {
        const TOP_DICT_LEN: usize = 17;
        let names = index(&[b"T"]);
        let globals = index(globals);
        let char_strings = index(char_strings);
        // Header, Name INDEX, Top DICT INDEX, String INDEX (empty), Global Subr INDEX,
        // CharStrings INDEX, Private DICT (two bytes), Local Subr INDEX.
        let char_strings_at = 4 + names.len() + (2 + 1 + 4 + TOP_DICT_LEN) + 2 + globals.len();
        let private_at = char_strings_at + char_strings.len();
        let mut top = vec![29];
        top.extend_from_slice(&(char_strings_at as u32).to_be_bytes());
        top.push(17); // CharStrings
        top.push(29);
        top.extend_from_slice(&2u32.to_be_bytes());
        top.push(29);
        top.extend_from_slice(&(private_at as u32).to_be_bytes());
        top.push(18); // Private: size, offset
        assert_eq!(top.len(), TOP_DICT_LEN);
        let mut table = vec![1, 0, 4, 1];
        table.extend_from_slice(&names);
        table.extend_from_slice(&index(&[&top]));
        table.extend_from_slice(&index(&[]));
        table.extend_from_slice(&globals);
        assert_eq!(table.len(), char_strings_at);
        table.extend_from_slice(&char_strings);
        table.extend_from_slice(&[141, 19]); // Private DICT: Subrs at offset 2
        table.extend_from_slice(&index(locals));
        table
    }

    /// A CFF2 INDEX (32-bit count).
    fn wide_index(items: &[&[u8]]) -> Vec<u8> {
        let narrow = index(items);
        [&[0, 0][..], &narrow].concat()
    }

    /// A CFF2 table with one variation axis and one region (peak at +1.0), the given
    /// charstrings, global subroutines and local subroutines.
    fn cff2(char_strings: &[&[u8]], globals: &[&[u8]], locals: &[&[u8]]) -> Vec<u8> {
        const TOP_DICT_LEN: usize = 19;
        let globals = wide_index(globals);
        let char_strings = wide_index(char_strings);
        // ItemVariationStore: one region list (1 axis, 1 region), one ItemVariationData.
        let mut store = vec![0, 1, 0, 0, 0, 12, 0, 1, 0, 0, 0, 22];
        store.extend_from_slice(&[0, 1, 0, 1, 0, 0, 0x40, 0, 0x40, 0]); // 0.0, 1.0, 1.0
        store.extend_from_slice(&[0, 0, 0, 0, 0, 1, 0, 0]);
        let vstore = [&(store.len() as u16).to_be_bytes()[..], &store].concat();

        let char_strings_at = 5 + TOP_DICT_LEN + globals.len();
        let vstore_at = char_strings_at + char_strings.len();
        let fd_array_at = vstore_at + vstore.len();
        // FDArray: one Font DICT of 11 bytes (Private: size 2, offset).
        let private_at = fd_array_at + (4 + 1 + 4 + 11);
        let mut top = vec![29];
        top.extend_from_slice(&(char_strings_at as u32).to_be_bytes());
        top.push(17); // CharStrings
        top.push(29);
        top.extend_from_slice(&(vstore_at as u32).to_be_bytes());
        top.push(24); // vstore
        top.push(29);
        top.extend_from_slice(&(fd_array_at as u32).to_be_bytes());
        top.extend_from_slice(&[12, 36]); // FDArray
        assert_eq!(top.len(), TOP_DICT_LEN);
        let mut font_dict = vec![29];
        font_dict.extend_from_slice(&2u32.to_be_bytes());
        font_dict.push(29);
        font_dict.extend_from_slice(&(private_at as u32).to_be_bytes());
        font_dict.push(18);

        let mut table = vec![2, 0, 5, 0, TOP_DICT_LEN as u8];
        table.extend_from_slice(&top);
        table.extend_from_slice(&globals);
        assert_eq!(table.len(), char_strings_at);
        table.extend_from_slice(&char_strings);
        table.extend_from_slice(&vstore);
        table.extend_from_slice(&wide_index(&[&font_dict]));
        assert_eq!(table.len(), private_at);
        table.extend_from_slice(&[141, 19]); // Private DICT: Subrs at offset 2
        table.extend_from_slice(&wide_index(locals));
        table
    }

    fn num(v: i16) -> [u8; 3] {
        let b = v.to_be_bytes();
        [28, b[0], b[1]]
    }

    fn concat(parts: &[&[u8]]) -> Vec<u8> {
        parts.concat()
    }

    fn both(table: &[u8], glyph: u16) -> (Option<String>, Option<String>) {
        let reference = {
            let mut trace = Trace::default();
            ttf_parser::cff::Table::parse(table)
                .and_then(|t| t.outline(GlyphId(glyph), &mut trace).ok())
                .map(|rect| format!("{}{rect:?}", trace.0))
        };
        let ours = {
            let mut trace = Trace::default();
            let mut steps = MAX_CHARSTRING_STEPS;
            CffFont::parse_cff(table)
                .and_then(|mut font| {
                    font.outline(glyph, &[], &mut steps, &mut trace).ok().flatten()
                })
                .map(|rect| format!("{}{rect:?}", trace.0))
        };
        (reference, ours)
    }

    #[test]
    fn standard_encoding_matches_the_specification() {
        assert_eq!(standard_encoding_sid(b' '), 1);
        assert_eq!(standard_encoding_sid(b'A'), 34);
        assert_eq!(standard_encoding_sid(b'~'), 95);
        assert_eq!(standard_encoding_sid(127), 0);
        assert_eq!(standard_encoding_sid(161), 96); // exclamdown
        assert_eq!(standard_encoding_sid(194), 125); // acute
        assert_eq!(standard_encoding_sid(208), 137); // emdash
        assert_eq!(standard_encoding_sid(251), 149); // germandbls
        assert_eq!(standard_encoding_sid(255), 0);
        assert_eq!((0..=255u8).filter(|&c| standard_encoding_sid(c) != 0).count(), 149);
    }

    #[test]
    fn flattened_glyphs_draw_exactly_what_the_reference_draws() {
        const RMOVETO: u8 = 21;
        const RLINETO: u8 = 5;
        const HLINETO: u8 = 6;
        const RRCURVETO: u8 = 8;
        const CALLSUBR: u8 = 10;
        const RETURN: u8 = 11;
        const ENDCHAR: u8 = 14;
        const HSTEMHM: u8 = 18;
        const HINTMASK: u8 = 19;
        const CALLGSUBR: u8 = 29;
        // Index bias 107: subroutine `i` is called with the operand `i - 107`.
        let call = |i: i16, op: u8| concat(&[&num(i - 107), &[op]]);

        // Global 0 draws a line and calls global 1; global 1 draws a curve.
        let g0 = concat(&[&num(50), &num(60), &[RLINETO], &call(1, CALLGSUBR), &[RETURN]]);
        let g1 = concat(&[
            &num(10),
            &num(20),
            &num(30),
            &num(40),
            &num(50),
            &num(-60),
            &[RRCURVETO, RETURN],
        ]);
        // Local 0 pushes two operands for its caller's operator; local 1 ends the glyph.
        let l0 = concat(&[&num(70), &num(-80), &[RETURN]]);
        let l1 = concat(&[&num(-5), &[HLINETO, ENDCHAR]]);

        let plain = concat(&[&num(100), &num(200), &[RMOVETO], &num(300), &[HLINETO, ENDCHAR]]);
        let nested = concat(&[
            &num(500), // the width
            &num(10),
            &num(20),
            &[RMOVETO],
            &call(0, CALLGSUBR),
            &call(0, CALLSUBR),
            &[RLINETO],
            &call(0, CALLGSUBR),
            &call(1, CALLSUBR),
        ]);
        // Three stems (one declared by the mask operator itself): a one-byte mask.
        let hinted = concat(&[
            &num(10),
            &num(20),
            &num(30),
            &num(40),
            &[HSTEMHM],
            &num(1),
            &num(2),
            &[HINTMASK, 0b1010_0000],
            &num(7),
            &num(8),
            &[RMOVETO],
            &call(0, CALLSUBR),
            &[RLINETO, HINTMASK, 0b0100_0000],
            &num(90),
            &[HLINETO, ENDCHAR],
        ]);
        let empty = vec![ENDCHAR];
        let table = cff(&[&empty, &plain, &nested, &hinted], &[&g0, &g1], &[&l0, &l1]);

        for glyph in 1..=3 {
            let (reference, ours) = both(&table, glyph);
            assert!(reference.is_some(), "glyph {glyph}: the reference must draw it");
            assert_eq!(ours, reference, "glyph {glyph}");
        }
        assert_eq!(both(&table, 0), (None, None), "an empty glyph has no outline");
        assert_eq!(both(&table, 9), (None, None), "no such glyph");
    }

    #[test]
    fn subroutine_bombs_stop_at_the_budget() {
        const CALLGSUBR: u8 = 29;
        // Ten levels, each calling the next eight times: 8^10 calls.
        let mut subrs: Vec<Vec<u8>> = Vec::new();
        for level in 0..10u8 {
            let mut body = Vec::new();
            if level < 9 {
                for _ in 0..8 {
                    body.extend_from_slice(&[level + 1 + 32, CALLGSUBR]);
                }
            }
            body.push(11);
            subrs.push(body);
        }
        let glyph: Vec<u8> = [[32, CALLGSUBR]; 8].concat().into_iter().chain([14]).collect();
        let subrs: Vec<&[u8]> = subrs.iter().map(Vec::as_slice).collect();
        let table = cff(&[&[14], &glyph], &subrs, &[]);
        let mut font = CffFont::parse_cff(&table).expect("parses");

        let mut steps = MAX_CHARSTRING_STEPS;
        let mut trace = Trace::default();
        assert_eq!(font.outline(1, &[], &mut steps, &mut trace), Err(Stop::TooComplex));
        assert_eq!(steps, 0, "the whole budget was used, and no more");

        // Recursion is cut at the nesting limit, cheaply.
        let recursive: Vec<u8> = vec![32, CALLGSUBR, 11];
        let glyph: Vec<u8> = vec![32, CALLGSUBR, 14];
        let table = cff(&[&[14], &glyph], &[&recursive], &[]);
        let mut font = CffFont::parse_cff(&table).expect("parses");
        let mut steps = MAX_CHARSTRING_STEPS;
        assert_eq!(font.outline(1, &[], &mut steps, &mut trace), Err(Stop::Invalid));
        assert!(MAX_CHARSTRING_STEPS - steps < 100);
    }

    #[test]
    fn accented_characters_are_composed_from_base_and_accent() {
        const RMOVETO: u8 = 21;
        const HLINETO: u8 = 6;
        const ENDCHAR: u8 = 14;
        // ISOAdobe charset: glyph id = string id; "A" is 34, "acute" (code 194) is 125.
        let mut glyphs: Vec<Vec<u8>> = vec![vec![ENDCHAR]; 126];
        glyphs[34] = concat(&[&num(0), &num(0), &[RMOVETO], &num(100), &[HLINETO, ENDCHAR]]);
        glyphs[125] = concat(&[&num(5), &num(5), &[RMOVETO], &num(10), &[HLINETO, ENDCHAR]]);
        // Glyph 1: width, adx, ady, bchar, achar, endchar.
        glyphs[1] = concat(&[&num(600), &num(40), &num(700), &num(65), &num(194), &[ENDCHAR]]);
        let glyphs: Vec<&[u8]> = glyphs.iter().map(Vec::as_slice).collect();
        let table = cff(&glyphs, &[], &[]);

        let (reference, ours) = both(&table, 1);
        let drawn = reference.as_deref().expect("the reference supports seac");
        assert!(drawn.starts_with("M 0 0 L 100 0 Z M 45 705 L 55 705 Z "), "{drawn}");
        assert!(drawn.ends_with("x_max: 100, y_max: 705 }"), "{drawn}");
        assert_eq!(ours, reference);
    }

    #[test]
    fn cff2_glyphs_with_blends_and_subroutines_match_the_reference() {
        const RMOVETO: u8 = 21;
        const RLINETO: u8 = 5;
        const HLINETO: u8 = 6;
        const CALLSUBR: u8 = 10;
        const BLEND: u8 = 16;
        const CALLGSUBR: u8 = 29;
        let call = |i: i16, op: u8| concat(&[&num(i - 107), &[op]]);
        // One region: every blended value is followed by one delta.
        let g0 = concat(&[&num(30), &num(40), &num(5), &num(-6), &num(2), &[BLEND, RLINETO]]);
        let l0 = concat(&[&num(-70), &[HLINETO], &call(0, CALLGSUBR)]);
        let glyph = concat(&[
            &num(100),
            &num(200),
            &num(10), // delta of 100
            &num(20), // delta of 200
            &num(2),
            &[BLEND, RMOVETO],
            &call(0, CALLGSUBR),
            &call(0, CALLSUBR),
            &num(9),
            &[HLINETO],
        ]);
        let table = cff2(&[&[], &glyph], &[&g0], &[&l0]);
        for coordinate in [0.0_f32, 0.5, 1.0] {
            let coordinates = [NormalizedCoordinate::from(coordinate)];
            let mut reference = Trace::default();
            let expected = ttf_parser::cff2::Table::parse(&table)
                .and_then(|t| t.outline(&coordinates, GlyphId(1), &mut reference).ok());
            assert!(expected.is_some(), "the reference must draw the glyph");
            let mut ours = Trace::default();
            let mut steps = MAX_CHARSTRING_STEPS;
            let mut font = CffFont::parse_cff2(&table).expect("parses");
            let rect = font.outline(1, &coordinates, &mut steps, &mut ours);
            assert_eq!(rect, Ok(expected), "at {coordinate}");
            assert_eq!(ours.0, reference.0, "at {coordinate}");
        }
        // At the region's peak the deltas apply in full: the first point is (110, 220).
        let mut trace = Trace::default();
        let mut steps = MAX_CHARSTRING_STEPS;
        let mut font = CffFont::parse_cff2(&table).expect("parses");
        let _ = font.outline(1, &[NormalizedCoordinate::from(1.0)], &mut steps, &mut trace);
        assert!(trace.0.starts_with("M 110 220 L 145 254 "), "{}", trace.0);
    }

    #[test]
    fn font_dict_selection_and_charset_lookups() {
        let with_fd_select = |data: &'static [u8]| {
            let mut font = CffFont::empty(data, false, Index::default());
            font.fd_select = Some(0);
            font
        };
        // Format 0: one font dict per glyph.
        let font = with_fd_select(&[0, 1, 2, 3]);
        assert_eq!([0, 2, 3].map(|g| font.font_dict_of(g)), [Some(1), Some(3), None]);
        // Format 3: glyphs 0..10 -> 5, 10..20 -> 7, sentinel 20.
        let font = with_fd_select(&[3, 0, 2, 0, 0, 5, 0, 10, 7, 0, 20]);
        let picks = [0, 9, 10, 19, 20, 500].map(|g| font.font_dict_of(g));
        assert_eq!(picks, [Some(5), Some(5), Some(7), Some(7), None, None]);
        // Format 4 (CFF2): the same with wide fields.
        let font =
            with_fd_select(&[4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 5, 0, 0, 0, 10, 0, 7, 0, 0, 0, 20]);
        let picks = [0, 9, 10, 19, 20].map(|g| font.font_dict_of(g));
        assert_eq!(picks, [Some(5), Some(5), Some(7), Some(7), None]);
        // Unknown formats, truncated tables and offsets outside the table select nothing.
        assert_eq!(with_fd_select(&[9, 0, 0]).font_dict_of(0), None);
        assert_eq!(with_fd_select(&[3, 0, 2, 0, 0, 5]).font_dict_of(3), None);
        let mut nowhere = with_fd_select(&[3]);
        nowhere.fd_select = Some(usize::MAX);
        assert_eq!(nowhere.font_dict_of(0), None);
        // No FDSelect at all: everything belongs to font dict 0.
        assert_eq!(CffFont::empty(&[], true, Index::default()).font_dict_of(77), Some(0));

        // Charsets: glyph 0 is implicit, the table starts with glyph 1.
        let charset = |data: &'static [u8]| {
            let mut font = CffFont::empty(data, false, Index::default());
            font.char_strings.count = 6;
            font
        };
        let lookup = |font: &CffFont<'_>, sid: u16| font.glyph_of_sid(0, sid, &mut 1_000);
        let format0 = charset(&[0, 0, 40, 0, 34, 0, 99, 0, 7, 0, 8]);
        assert_eq!(lookup(&format0, 34), Ok(2));
        assert_eq!(lookup(&format0, 8), Ok(5));
        assert_eq!(lookup(&format0, 35), Err(Stop::Invalid));
        let format1 = charset(&[1, 0, 34, 3, 0, 90, 0]); // sids 34..=37, then 90
        assert_eq!([34, 37, 90].map(|sid| lookup(&format1, sid)), [Ok(1), Ok(4), Ok(5)]);
        let format2 = charset(&[2, 0, 34, 0, 4]); // sids 34..=38
        assert_eq!(lookup(&format2, 38), Ok(5));
        assert_eq!(lookup(&format2, 39), Err(Stop::Invalid));
        // The search is charged to the budget.
        assert_eq!(format0.glyph_of_sid(0, 8, &mut 3), Err(Stop::TooComplex));
        assert_eq!(charset(&[7]).glyph_of_sid(0, 1, &mut 9), Err(Stop::Invalid));
    }

    #[test]
    fn garbage_tables_are_rejected_without_panicking() {
        assert!(CffFont::parse_cff(&[]).is_none());
        assert!(CffFont::parse_cff(&[1, 0, 4, 1]).is_none());
        assert!(CffFont::parse_cff2(&[2, 0, 5, 0, 0]).is_none());
        let table = cff(&[&[14], &[32, 10, 14]], &[], &[&[11]]);
        let mut state = 0x243F_6A88_85A3_08D3u64;
        for _ in 0..3_000 {
            let mut broken = table.clone();
            for _ in 0..3 {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let at = (state >> 33) as usize % broken.len();
                broken[at] = (state >> 24) as u8;
            }
            for parse in [CffFont::parse_cff, CffFont::parse_cff2] {
                if let Some(mut font) = parse(&broken) {
                    for glyph in 0..3 {
                        let mut steps = 10_000;
                        let _ = font.outline(glyph, &[], &mut steps, &mut Trace::default());
                    }
                }
            }
        }
    }
}
