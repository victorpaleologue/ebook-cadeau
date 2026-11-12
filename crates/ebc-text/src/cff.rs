//! Bounded outlines for PostScript-flavoured fonts (`CFF ` and `CFF2` tables).
//!
//! # Why this module exists
//!
//! A Type 2 charstring may call subroutines, which may call subroutines, ten levels
//! deep. Nothing in the format limits how often: a font of 750 bytes whose subroutines
//! each call the next one eight times asks for 8^10 ≈ 10^9 calls. `read-fonts`, whose
//! interpreter draws the glyphs, stops a charstring after a million operators — a bound
//! per glyph, which a long text multiplies, and which says nothing about how much of
//! the budget of one call (see `outline::MAX_RUN_COST`) a glyph has used.
//!
//! # What it does
//!
//! The interpreter of `read-fonts` obtains every subroutine (and the two glyphs of a
//! `seac` accent composition) from a context object. This module supplies that context
//! and **meters** it: the glyph's own charstring and every subroutine body handed out
//! are charged, byte for byte, to a budget of at most [`MAX_CHARSTRING_STEPS`]; once the
//! budget is used up, the next request is refused and the interpreter stops. Since the
//! interpreter consumes at least one byte per operator or operand it executes, it can
//! never do more work than the bytes it was given — by construction, whatever the font
//! contains. A glyph that exceeds the budget, or whose charstring is malformed, has no
//! outline.
//!
//! The Private DICT of a glyph's font dictionary (which names its local subroutines) is
//! read once per [`CffOutlines`] and charged by its size.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Range;

use read_fonts::model::Error;
use read_fonts::model::glyph::outline::PathSink;
use read_fonts::model::variation::NormalizedCoord;
use read_fonts::ps::FontMatrix;
use read_fonts::ps::cff::blend::BlendState;
use read_fonts::ps::cff::dict::{self, Entry};
use read_fonts::ps::cff::index::Index;
use read_fonts::ps::cff::{CffFontRef, Subfont};
use read_fonts::ps::charstring::{
    self, CharstringContext, CharstringKind, CommandSink, TransformSink,
};
use read_fonts::tables::{cff::Cff, cff2::Cff2};
use read_fonts::types::Fixed;
use read_fonts::types::GlyphId;
use read_fonts::{FontData, FontRead};

/// Most charstring bytes interpreted for one glyph (subroutine bodies included, each
/// time they are called). Real glyphs need a few hundred to a few thousand.
pub(crate) const MAX_CHARSTRING_STEPS: u32 = 1 << 18;

/// Why a glyph could not be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    /// The charstring (or a table it needs) is malformed or uses something unsupported.
    Invalid,
    /// The step budget ran out.
    TooComplex,
}

/// Takes `cost` steps from `steps`; on failure the budget is left empty.
fn charge(steps: &mut u32, cost: usize) -> Result<(), Stop> {
    match u32::try_from(cost).ok().and_then(|cost| steps.checked_sub(cost)) {
        Some(left) => {
            *steps = left;
            Ok(())
        }
        None => {
            *steps = 0;
            Err(Stop::TooComplex)
        }
    }
}

/// The cost of reading a DICT of `len` bytes.
const fn dict_cost(len: usize) -> usize {
    len / 8 + 1
}

/// What a glyph's font dictionary contributes to its charstring.
#[derive(Clone)]
struct Private<'a> {
    subfont: Subfont,
    local_subrs: Index<'a>,
}

/// The charstrings of a `CFF ` or `CFF2` table, drawn under a budget.
pub(crate) struct CffOutlines<'a> {
    font: CffFontRef<'a>,
    cff2: bool,
    /// Private DICT of a font without Font DICTs (a plain, non-CID `CFF ` font).
    private: Option<Range<usize>>,
    /// The Font DICTs of a CID-keyed `CFF ` font or of a `CFF2` font.
    font_dicts: Option<Index<'a>>,
    /// The font dictionaries looked up so far (`None`: unusable).
    privates: BTreeMap<u16, Option<Private<'a>>>,
}

impl<'a> CffOutlines<'a> {
    /// Parses a `CFF ` table (version 1) of a font with `units_per_em` units per em.
    pub(crate) fn parse_cff(data: &'a [u8], units_per_em: u16) -> Option<Self> {
        let font = CffFontRef::new_cff(data, 0, Some(i32::from(units_per_em))).ok()?;
        let top_dict = Cff::read(FontData::new(data)).ok()?.top_dicts().get(0)?;
        Some(Self::new(font, false, data, top_dict))
    }

    /// Parses a `CFF2` table of a font with `units_per_em` units per em.
    pub(crate) fn parse_cff2(data: &'a [u8], units_per_em: u16) -> Option<Self> {
        let font = CffFontRef::new_cff2(data, Some(i32::from(units_per_em))).ok()?;
        let top_dict = Cff2::read(FontData::new(data)).ok()?.top_dict_data();
        Some(Self::new(font, true, data, top_dict))
    }

    fn new(font: CffFontRef<'a>, cff2: bool, data: &'a [u8], top_dict: &'a [u8]) -> Self {
        let mut private = None;
        let mut font_dicts = None;
        for entry in dict::entries(top_dict, None).filter_map(Result::ok) {
            match entry {
                Entry::PrivateDictRange(range) => private = Some(range),
                Entry::FdArrayOffset(offset) => {
                    font_dicts = data.get(offset..).and_then(|d| Index::new(d, cff2).ok());
                }
                _ => {}
            }
        }
        Self { font, cff2, private, font_dicts, privates: BTreeMap::new() }
    }

    /// The font dictionary that applies to `glyph`. It is read once per font dictionary;
    /// reading its DICTs is charged to `steps` by their size.
    fn private(
        &mut self,
        glyph: GlyphId,
        coords: &[NormalizedCoord],
        steps: &mut u32,
    ) -> Result<Private<'a>, Stop> {
        let index = self.font.subfont_index(glyph).ok_or(Stop::Invalid)?;
        if let Some(known) = self.privates.get(&index) {
            return known.clone().ok_or(Stop::Invalid);
        }
        let data = self.font.data();
        // The size of what `CffFontRef::subfont` is about to parse.
        let range = match &self.font_dicts {
            Some(font_dicts) => {
                let font_dict = font_dicts.get(usize::from(index)).ok_or(Stop::Invalid)?;
                charge(steps, dict_cost(font_dict.len()))?;
                dict::entries(font_dict, None).filter_map(Result::ok).find_map(
                    |entry| match entry {
                        Entry::PrivateDictRange(range) => Some(range),
                        _ => None,
                    },
                )
            }
            None => self.private.clone(),
        };
        if let Some(range) = &range {
            charge(steps, dict_cost(range.len().min(data.len())))?;
        }
        let private = self.font.subfont(index, coords).ok().and_then(|subfont| {
            let local_subrs = match subfont.subrs_offset() {
                0 => Index::Empty,
                offset => Index::new(data.get(usize::try_from(offset).ok()?..)?, self.cff2).ok()?,
            };
            Some(Private { subfont, local_subrs })
        });
        self.privates.insert(index, private.clone());
        private.ok_or(Stop::Invalid)
    }

    /// Draws `glyph` into `sink`, in font units, at the variation position `coords`;
    /// `steps` is the remaining budget (at most [`MAX_CHARSTRING_STEPS`] are used) and
    /// is reduced by what was used.
    ///
    /// On an error, `sink` may have received part of the outline.
    pub(crate) fn outline(
        &mut self,
        glyph: u16,
        coords: &'a [NormalizedCoord],
        steps: &mut u32,
        mut sink: &mut dyn PathSink,
    ) -> Result<(), Stop> {
        let glyph = GlyphId::new(u32::from(glyph));
        let charstrings = self.font.charstrings().clone();
        let body = charstrings.get(glyph.to_u32() as usize).ok_or(Stop::Invalid)?;
        charge(steps, body.len() + 1)?;
        let private = self.private(glyph, coords, steps)?;

        let blend = self.font.var_store().and_then(|store| {
            BlendState::new(store.clone(), coords, private.subfont.vs_index()).ok()
        });
        let global_subrs = self.font.global_subrs().clone();
        let context = Metered {
            inner: (self.font.data(), &charstrings, &global_subrs, &private.local_subrs),
            // One charset search per accent composition, twice.
            seac_cost: charstrings.count() as usize / 2 + 64,
            left: Cell::new(*steps),
            exhausted: Cell::new(false),
        };
        // The commands are forwarded as the charstring gives them: explicit closing lines
        // and zero-length segments included (the library's own drawing filters those).
        let transform = self.font.transform(&private.subfont, None);
        let result = if transform.matrix == FontMatrix::IDENTITY && transform.scale.is_none() {
            // The usual case: font units as they are, fractions included.
            charstring::evaluate(&context, blend, body, &mut Exact(sink))
        } else {
            // An unusual font matrix: applied by the library, which works in whole font
            // units when no size is given.
            let mut transformed = TransformSink::new(&mut sink, transform);
            charstring::evaluate(&context, blend, body, &mut transformed)
        };
        *steps = context.left.get();
        match result {
            Ok(_) => Ok(()),
            Err(_) if context.exhausted.get() => Err(Stop::TooComplex),
            Err(_) => Err(Stop::Invalid),
        }
    }
}

/// Passes the interpreter's 16.16 fixed-point coordinates on without rounding them.
struct Exact<'s>(&'s mut dyn PathSink);

impl CommandSink for Exact<'_> {
    fn move_to(&mut self, x: Fixed, y: Fixed) {
        self.0.move_to(x.to_f32(), y.to_f32());
    }

    fn line_to(&mut self, x: Fixed, y: Fixed) {
        self.0.line_to(x.to_f32(), y.to_f32());
    }

    fn curve_to(&mut self, cx0: Fixed, cy0: Fixed, cx1: Fixed, cy1: Fixed, x: Fixed, y: Fixed) {
        self.0.curve_to(
            cx0.to_f32(),
            cy0.to_f32(),
            cx1.to_f32(),
            cy1.to_f32(),
            x.to_f32(),
            y.to_f32(),
        );
    }

    fn close(&mut self) {
        self.0.close();
    }
}

/// Hands the interpreter its subroutines, charging each body to the budget.
struct Metered<'c, 'a> {
    /// The context of `read-fonts`: table, CharStrings, global and local subroutines.
    inner: (&'a [u8], &'c Index<'a>, &'c Index<'a>, &'c Index<'a>),
    seac_cost: usize,
    left: Cell<u32>,
    exhausted: Cell<bool>,
}

impl Metered<'_, '_> {
    fn charge(&self, cost: usize) -> Result<(), Error> {
        let mut left = self.left.get();
        let charged = charge(&mut left, cost);
        self.left.set(left);
        charged.map_err(|_| {
            self.exhausted.set(true);
            Error::LimitExceeded
        })
    }
}

impl CharstringContext for Metered<'_, '_> {
    fn kind(&self) -> CharstringKind {
        CharstringKind::Type2
    }

    fn seac_components(&self, base_code: i32, accent_code: i32) -> Result<[&[u8]; 2], Error> {
        self.charge(self.seac_cost)?;
        let parts = self.inner.seac_components(base_code, accent_code)?;
        self.charge(parts[0].len() + parts[1].len() + 2)?;
        Ok(parts)
    }

    fn global_subr(&self, index: i32) -> Result<&[u8], Error> {
        let body = self.inner.global_subr(index)?;
        self.charge(body.len() + 1)?;
        Ok(body)
    }

    fn subr(&self, index: i32) -> Result<&[u8], Error> {
        let body = self.inner.subr(index)?;
        self.charge(body.len() + 1)?;
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use read_fonts::types::F2Dot14;

    use super::*;

    /// Collects an outline as text, e.g. `M 0 0 L 10 0 Z`.
    #[derive(Default)]
    struct Trace(String);

    impl PathSink for Trace {
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

    /// The outline of a glyph as `read-fonts` draws it on its own (unmetered), and as
    /// this module draws it.
    fn both(table: &[u8], glyph: u16) -> (Option<String>, Option<String>) {
        let reference = {
            let mut trace = Trace::default();
            CffFontRef::new_cff(table, 0, Some(1000))
                .ok()
                .and_then(|font| {
                    let id = GlyphId::new(u32::from(glyph));
                    let subfont = font.subfont(font.subfont_index(id)?, &[]).ok()?;
                    font.draw(&subfont, id, &[], None, &mut trace).ok()
                })
                .map(|_| trace.0)
        };
        let ours = {
            let mut trace = Trace::default();
            let mut steps = MAX_CHARSTRING_STEPS;
            CffOutlines::parse_cff(table, 1000)
                .and_then(|mut font| font.outline(glyph, &[], &mut steps, &mut trace).ok())
                .map(|()| trace.0)
        };
        (reference, ours)
    }

    #[test]
    fn metered_glyphs_draw_exactly_what_the_reference_draws() {
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
            assert!(reference.as_deref().is_some_and(|r| !r.is_empty()), "glyph {glyph}");
            assert_eq!(ours, reference, "glyph {glyph}");
        }
        // The values, not only the agreement: "plain" is a move and a horizontal line;
        // "nested" runs through both global and both local subroutines.
        assert_eq!(both(&table, 1).1.as_deref(), Some("M 100 200 L 400 200 Z "));
        assert_eq!(
            both(&table, 2).1.as_deref(),
            Some(
                "M 10 20 L 60 80 C 70 100 100 140 150 80 L 220 0 L 270 60 \
                 C 280 80 310 120 360 60 L 355 60 Z "
            )
        );
        assert_eq!(both(&table, 0), (Some(String::new()), Some(String::new())), "an empty glyph");
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
        let mut font = CffOutlines::parse_cff(&table, 1000).expect("parses");

        let mut steps = MAX_CHARSTRING_STEPS;
        let mut trace = Trace::default();
        assert_eq!(font.outline(1, &[], &mut steps, &mut trace), Err(Stop::TooComplex));
        assert_eq!(steps, 0, "the whole budget was used, and no more");
        // A budget that the glyph's own charstring already exceeds.
        let mut steps = 5;
        assert_eq!(font.outline(1, &[], &mut steps, &mut trace), Err(Stop::TooComplex));
        assert_eq!(steps, 0);

        // Recursion is cut at the nesting limit, cheaply.
        let recursive: Vec<u8> = vec![32, CALLGSUBR, 11];
        let glyph: Vec<u8> = vec![32, CALLGSUBR, 14];
        let table = cff(&[&[14], &glyph], &[&recursive], &[]);
        let mut font = CffOutlines::parse_cff(&table, 1000).expect("parses");
        let mut steps = MAX_CHARSTRING_STEPS;
        assert_eq!(font.outline(1, &[], &mut steps, &mut trace), Err(Stop::Invalid));
        assert!(MAX_CHARSTRING_STEPS - steps < 100);
    }

    #[test]
    fn the_budget_is_charged_by_the_bytes_handed_to_the_interpreter() {
        const CALLGSUBR: u8 = 29;
        // The glyph (5 bytes) calls global 0 (one byte: return) twice; the Private DICT
        // is 2 bytes.
        let subr = vec![11];
        let glyph = vec![32, CALLGSUBR, 32, CALLGSUBR, 14];
        let table = cff(&[&[14], &glyph], &[&subr], &[]);
        let mut font = CffOutlines::parse_cff(&table, 1000).expect("parses");
        let mut steps = 1_000;
        let _ = font.outline(1, &[], &mut steps, &mut Trace::default());
        // Charstring 5 + 1, Private DICT 1, two calls of 1 + 1.
        assert_eq!(1_000 - steps, 6 + 1 + 4);
        // The font dictionary is read once: a second glyph does not pay for it again.
        let mut steps = 1_000;
        let _ = font.outline(1, &[], &mut steps, &mut Trace::default());
        assert_eq!(1_000 - steps, 6 + 4);
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
        let drawn = ours.as_deref().expect("accent compositions are supported");
        // The accent, moved by (40, 700), then the base glyph.
        assert_eq!(drawn, "M 45 705 L 55 705 Z M 0 0 L 100 0 Z ");
        assert_eq!(ours, reference);
        // The composition costs more than its three charstrings: the charset is searched.
        let mut font = CffOutlines::parse_cff(&table, 1000).expect("parses");
        let mut steps = 40;
        assert_eq!(font.outline(1, &[], &mut steps, &mut Trace::default()), Err(Stop::TooComplex));
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
        // Where the blended values are whole numbers, the outline is what the library
        // draws on its own.
        for coordinate in [0.0_f32, 1.0] {
            let coordinates = [F2Dot14::from_f32(coordinate)];
            let mut reference = Trace::default();
            let font = CffFontRef::new_cff2(&table, Some(1000)).expect("parses");
            let subfont = font.subfont(0, &coordinates).expect("a font dictionary");
            font.draw(&subfont, GlyphId::new(1), &coordinates, None, &mut reference)
                .expect("the reference must draw the glyph");
            let mut ours = Trace::default();
            let mut steps = MAX_CHARSTRING_STEPS;
            let mut font = CffOutlines::parse_cff2(&table, 1000).expect("parses");
            assert_eq!(font.outline(1, &coordinates, &mut steps, &mut ours), Ok(()));
            assert_eq!(ours.0, reference.0, "at {coordinate}");
        }
        // Half way, the deltas of 5 and -6 give half units — which are kept (the
        // library's own drawing truncates coordinates to whole font units).
        let coordinates = [F2Dot14::from_f32(0.5)];
        let mut ours = Trace::default();
        let mut steps = MAX_CHARSTRING_STEPS;
        let mut font = CffOutlines::parse_cff2(&table, 1000).expect("parses");
        assert_eq!(font.outline(1, &coordinates, &mut steps, &mut ours), Ok(()));
        assert_eq!(ours.0, "M 105 210 L 137.5 247 L 67.5 247 L 100 284 L 109 284 Z ");
        // At the region's peak the deltas apply in full: the first point is (110, 220).
        let coordinates = [F2Dot14::from_f32(1.0)];
        let mut trace = Trace::default();
        let mut steps = MAX_CHARSTRING_STEPS;
        let mut font = CffOutlines::parse_cff2(&table, 1000).expect("parses");
        let _ = font.outline(1, &coordinates, &mut steps, &mut trace);
        assert!(trace.0.starts_with("M 110 220 L 145 254 "), "{}", trace.0);
    }

    #[test]
    fn garbage_tables_are_rejected_without_panicking() {
        assert!(CffOutlines::parse_cff(&[], 1000).is_none());
        assert!(CffOutlines::parse_cff(&[1, 0, 4, 1], 1000).is_none());
        assert!(CffOutlines::parse_cff2(&[2, 0, 5, 0, 0], 1000).is_none());
        let table = cff(&[&[14], &[32, 10, 14]], &[], &[&[11]]);
        let mut state = 0x243F_6A88_85A3_08D3u64;
        for _ in 0..3_000 {
            let mut broken = table.clone();
            for _ in 0..3 {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let at = (state >> 33) as usize % broken.len();
                broken[at] = (state >> 24) as u8;
            }
            for cff2 in [false, true] {
                let parsed = if cff2 {
                    CffOutlines::parse_cff2(&broken, 1000)
                } else {
                    CffOutlines::parse_cff(&broken, 1000)
                };
                if let Some(mut font) = parsed {
                    for glyph in 0..3 {
                        let mut steps = 10_000;
                        let _ = font.outline(glyph, &[], &mut steps, &mut Trace::default());
                        assert!(steps <= 10_000);
                    }
                }
            }
        }
    }
}
