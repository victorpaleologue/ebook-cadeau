//! Glyph outlines with a bound on the work a font can ask for.
//!
//! Font files are hostile input, and two outline formats let a tiny file describe an
//! astronomically large drawing:
//!
//! * **TrueType composites** (`glyf`): a glyph may be made of other glyphs, which may be
//!   made of other glyphs… A 1 kB font with 18 levels of two components each stands for
//!   2^18 copies of its last glyph; with empty leaves nothing is ever drawn, yet every
//!   component is visited.
//! * **Type 2 subroutines** (`CFF `, `CFF2`): the same with nested subroutine calls.
//!
//! `ttf-parser` limits the nesting depth of both but not the fan-out, so this module
//! stands between the crate and `ttf_parser::Face::outline_glyph`:
//!
//! * for `glyf` fonts the *expanded size* of a glyph — one unit per component reference
//!   and per point, summed over the whole component tree — is computed first, exactly and
//!   in linear time (each glyph is looked at once); a glyph above [`MAX_GLYPH_COST`] is
//!   not drawn;
//! * for `CFF `/`CFF2` fonts the charstring is flattened under a budget (see
//!   [`crate::cff`]) and `ttf-parser` only ever sees subroutine-free charstrings;
//! * every [`Outliner`] — one per `to_outline`, `rasterize` or font load — has an overall
//!   budget of [`MAX_RUN_COST`], so that a huge text cannot multiply a merely expensive
//!   glyph into minutes of work or gigabytes of path data.
//!
//! All faces used for outlines and for shaping are built from one collection of the
//! font's tables ([`collect_tables`]), which is also what the analysis reads: there is no
//! way for the analysis and the parser to look at different bytes.

use std::collections::BTreeMap;
use std::num::NonZeroU16;

use ttf_parser::{Face, GlyphId, OutlineBuilder, RawFace, RawFaceTables, Rect, Tag};

use crate::cff::{CffFont, MAX_CHARSTRING_STEPS, Stop};

/// Largest expanded size of one TrueType glyph: component references plus points over
/// its whole component tree. A glyph cannot legitimately have more than 65 535 points
/// (point numbers are 16-bit).
pub(crate) const MAX_GLYPH_COST: u32 = 1 << 16;

/// Largest glyph cost at which glyph outlines stay available to the *shaper* of a
/// variable font without `HVAR`/`VVAR` tables — the one situation in which shaping
/// itself outlines glyphs (to derive advances), once per glyph of the text.
pub(crate) const MAX_SHAPING_GLYPH_COST: u32 = 1 << 12;

/// Overall budget of one [`Outliner`], in the units of [`MAX_GLYPH_COST`] (points and
/// component references) or charstring steps: about 20 000 average letters, or 3 000
/// complex ideographs, per call — and at most some 70 MB of path data.
pub(crate) const MAX_RUN_COST: u64 = 1 << 20;

/// Most path segments one glyph may contribute (a contour of `n` points yields at most
/// `n + 2` segments, so this is beyond any real glyph).
pub(crate) const MAX_GLYPH_SEGMENTS: usize = 2 * 65_536;

/// Why a glyph was left out of an outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// The glyph on its own is beyond [`MAX_GLYPH_COST`] / the charstring budget.
    GlyphTooComplex(u16),
    /// The glyph would have been fine, but the run's overall budget is used up.
    BudgetExhausted(u16),
}

/// Gathers the tables of a face exactly as `ttf_parser::Face::parse` does: the directory
/// is read in file order, a later record replaces an earlier one with the same tag, and
/// a record pointing outside the file yields no table.
pub(crate) fn collect_tables<'a>(raw: &RawFace<'a>) -> RawFaceTables<'a> {
    let mut tables = RawFaceTables::default();
    for record in raw.table_records {
        let Ok(start) = usize::try_from(record.offset) else { continue };
        let Some(end) = usize::try_from(record.length).ok().and_then(|n| start.checked_add(n))
        else {
            continue;
        };
        let data = raw.data.get(start..end);
        match &record.tag.to_bytes() {
            b"bdat" => tables.bdat = data,
            b"bloc" => tables.bloc = data,
            b"CBDT" => tables.cbdt = data,
            b"CBLC" => tables.cblc = data,
            b"CFF " => tables.cff = data,
            b"CFF2" => tables.cff2 = data,
            b"COLR" => tables.colr = data,
            b"CPAL" => tables.cpal = data,
            b"EBDT" => tables.ebdt = data,
            b"EBLC" => tables.eblc = data,
            b"GDEF" => tables.gdef = data,
            b"GPOS" => tables.gpos = data,
            b"GSUB" => tables.gsub = data,
            b"MATH" => tables.math = data,
            b"HVAR" => tables.hvar = data,
            b"MVAR" => tables.mvar = data,
            b"OS/2" => tables.os2 = data,
            b"SVG " => tables.svg = data,
            b"VORG" => tables.vorg = data,
            b"VVAR" => tables.vvar = data,
            b"ankr" => tables.ankr = data,
            b"avar" => tables.avar = data,
            b"cmap" => tables.cmap = data,
            b"feat" => tables.feat = data,
            b"fvar" => tables.fvar = data,
            b"glyf" => tables.glyf = data,
            b"gvar" => tables.gvar = data,
            b"head" => tables.head = data.unwrap_or_default(),
            b"hhea" => tables.hhea = data.unwrap_or_default(),
            b"hmtx" => tables.hmtx = data,
            b"kern" => tables.kern = data,
            b"kerx" => tables.kerx = data,
            b"loca" => tables.loca = data,
            b"maxp" => tables.maxp = data.unwrap_or_default(),
            b"morx" => tables.morx = data,
            b"name" => tables.name = data,
            b"post" => tables.post = data,
            b"sbix" => tables.sbix = data,
            b"STAT" => tables.stat = data,
            b"trak" => tables.trak = data,
            b"vhea" => tables.vhea = data,
            b"vmtx" => tables.vmtx = data,
            _ => {}
        }
    }
    tables
}

/// Builds a face from collected tables and applies variation settings.
pub(crate) fn face_from_tables<'a>(
    tables: RawFaceTables<'a>,
    variations: &[(Tag, f32)],
) -> Option<Face<'a>> {
    let mut face = Face::from_raw_tables(tables).ok()?;
    for &(tag, value) in variations {
        let _ = face.set_variation(tag, value);
    }
    Some(face)
}

/// The tables a face needs for *shaping*, without those through which a font can make
/// the shaper do unbounded work:
///
/// * `CFF `, `CFF2`: the shaper only looks at them to derive advances of variable fonts
///   that lack `HVAR` (which the specification requires for `CFF2`), by outlining;
/// * `COLR`: only used for the extents of colour glyphs in fallback mark positioning, by
///   walking the paint graph;
/// * `glyf`, `loca`, `gvar` when `strip_glyf` is set (see [`glyf_must_be_hidden`]).
///
/// Substitution, kerning, mark attachment and advances from `hmtx`/`HVAR` are unaffected.
pub(crate) fn shaping_tables(mut tables: RawFaceTables<'_>, strip_glyf: bool) -> RawFaceTables<'_> {
    tables.cff = None;
    tables.cff2 = None;
    tables.colr = None;
    if strip_glyf {
        tables.glyf = None;
        tables.loca = None;
        tables.gvar = None;
    }
    tables
}

/// Whether the shaper must not see the `glyf` outlines of a font whose most expensive
/// glyph costs `max_glyph_cost`.
pub(crate) fn glyf_must_be_hidden(tables: &RawFaceTables<'_>, max_glyph_cost: u32) -> bool {
    let outlines_while_shaping =
        tables.fvar.is_some() && tables.hvar.is_none() && tables.vvar.is_none();
    max_glyph_cost > MAX_GLYPH_COST
        || (outlines_while_shaping && max_glyph_cost > MAX_SHAPING_GLYPH_COST)
}

// ---------------------------------------------------------------------------------------
// TrueType: the expanded size of composite glyphs
// ---------------------------------------------------------------------------------------

/// Computes glyph costs from the `glyf` and `loca` tables, memoised.
pub(crate) struct GlyfCosts<'a> {
    loca: ttf_parser::loca::Table<'a>,
    glyf: &'a [u8],
    /// Cost per glyph; [`Self::IN_PROGRESS`] while its components are being expanded.
    memo: BTreeMap<u16, u32>,
}

/// A glyph whose components are being added up.
struct Frame {
    glyph: u16,
    components: Vec<u16>,
    next: usize,
    cost: u32,
}

impl<'a> GlyfCosts<'a> {
    /// Marks a glyph on the expansion stack. Real costs are at least 1.
    const IN_PROGRESS: u32 = 0;

    /// `None` for a font without usable `glyf`/`loca` tables.
    pub(crate) fn new(tables: &RawFaceTables<'a>) -> Option<Self> {
        // The same construction as in `ttf_parser::Face`, from the same bytes.
        let head = ttf_parser::head::Table::parse(tables.head)?;
        let maxp = ttf_parser::maxp::Table::parse(tables.maxp)?;
        let loca = ttf_parser::loca::Table::parse(
            maxp.number_of_glyphs,
            head.index_to_location_format,
            tables.loca?,
        )?;
        Some(Self { loca, glyf: tables.glyf?, memo: BTreeMap::new() })
    }

    /// The largest cost of any of the font's first `glyph_count` glyphs.
    pub(crate) fn max_cost(&mut self, glyph_count: NonZeroU16) -> u32 {
        (0..glyph_count.get()).map(|glyph| self.cost(glyph)).max().unwrap_or(1)
    }

    /// The expanded size of a glyph: 1 for the glyph itself, plus its points, plus the
    /// cost of each of its components — an upper bound of the work `ttf-parser` does to
    /// outline it. `u32::MAX` (saturated) for a glyph that contains itself.
    pub(crate) fn cost(&mut self, glyph: u16) -> u32 {
        if let Some(&known) = self.memo.get(&glyph) {
            return if known == Self::IN_PROGRESS { u32::MAX } else { known };
        }
        // Depth-first over the component graph with an explicit stack: a chain of
        // components can be as long as the font has glyphs.
        let mut stack = vec![self.enter(glyph)];
        let mut result = u32::MAX;
        while let Some(top) = stack.last_mut() {
            if let Some(&component) = top.components.get(top.next) {
                top.next += 1;
                match self.memo.get(&component) {
                    // A cycle. `ttf-parser` gives up on such glyphs at depth 32.
                    Some(&Self::IN_PROGRESS) => top.cost = u32::MAX,
                    Some(&known) => top.cost = top.cost.saturating_add(known),
                    None => {
                        let frame = self.enter(component);
                        stack.push(frame);
                    }
                }
                continue;
            }
            let (finished, cost) = (top.glyph, top.cost);
            stack.pop();
            self.memo.insert(finished, cost);
            match stack.last_mut() {
                Some(parent) => parent.cost = parent.cost.saturating_add(cost),
                None => result = cost,
            }
        }
        result
    }

    /// Reads a glyph's header: its own cost and the glyphs it is composed of.
    fn enter(&mut self, glyph: u16) -> Frame {
        self.memo.insert(glyph, Self::IN_PROGRESS);
        let mut frame = Frame { glyph, components: Vec::new(), next: 0, cost: 1 };
        let Some(data) = self.loca.glyph_range(GlyphId(glyph)).and_then(|r| self.glyf.get(r))
        else {
            return frame; // an empty glyph
        };
        let u16_at = |at: usize| data.get(at..at + 2).map(|b| u16::from_be_bytes([b[0], b[1]]));
        let Some(contours) = u16_at(0).map(|v| v as i16) else { return frame };
        if contours > 0 {
            // A simple glyph: the last contour end point is the number of points − 1.
            let last_end = u16_at(10 + 2 * (usize::from(contours as u16) - 1));
            frame.cost += last_end.map_or(0, |end| u32::from(end) + 1);
        } else if contours < 0 {
            // A composite glyph. This walks the component records exactly like
            // `ttf_parser`'s `CompositeGlyphIter`, so that both see the same components.
            let mut at = 10usize;
            while let (Some(flags), Some(component)) = (u16_at(at), u16_at(at + 2)) {
                at += 4;
                if flags & 0x0002 != 0 {
                    at += if flags & 0x0001 != 0 { 4 } else { 2 }; // offsets: words or bytes
                }
                at += if flags & 0x0080 != 0 {
                    8 // a 2 x 2 matrix
                } else if flags & 0x0040 != 0 {
                    4 // x and y scales
                } else if flags & 0x0008 != 0 {
                    2 // one scale
                } else {
                    0
                };
                if at > data.len() {
                    break; // a truncated record is not a component
                }
                frame.components.push(component);
                if flags & 0x0020 == 0 {
                    break; // no more components
                }
            }
            frame.cost = frame.cost.saturating_add(frame.components.len() as u32);
        }
        frame
    }
}

// ---------------------------------------------------------------------------------------
// The outliner
// ---------------------------------------------------------------------------------------

enum Outlines<'a> {
    TrueType(GlyfCosts<'a>),
    PostScript(Box<CffFont<'a>>),
    None,
}

/// Draws glyph outlines of one face under a budget; see the module documentation.
pub(crate) struct Outliner<'a> {
    /// A face without `CFF `/`CFF2` tables: `ttf-parser` never runs a font's charstring.
    face: Face<'a>,
    outlines: Outlines<'a>,
    remaining: u64,
    refusal: Option<Refusal>,
}

impl<'a> Outliner<'a> {
    /// `None` if the tables do not make a face.
    pub(crate) fn new(tables: RawFaceTables<'a>, variations: &[(Tag, f32)]) -> Option<Self> {
        // The precedence of `ttf_parser::Face::outline_glyph`: glyf, then CFF, then CFF2.
        let outlines = if let Some(costs) = GlyfCosts::new(&tables) {
            Outlines::TrueType(costs)
        } else if let Some(font) = tables.cff.and_then(CffFont::parse_cff) {
            Outlines::PostScript(Box::new(font))
        } else if let Some(font) = tables.cff2.and_then(CffFont::parse_cff2) {
            Outlines::PostScript(Box::new(font))
        } else {
            Outlines::None
        };
        let mut tables = tables;
        tables.cff = None;
        tables.cff2 = None;
        let face = face_from_tables(tables, variations)?;
        Some(Self { face, outlines, remaining: MAX_RUN_COST, refusal: None })
    }

    /// The face the outlines are taken from (for metrics and character mapping).
    pub(crate) fn face(&self) -> &Face<'a> {
        &self.face
    }

    /// The first glyph that was refused, if any, and why.
    pub(crate) fn refusal(&self) -> Option<Refusal> {
        self.refusal
    }

    /// Sends the outline of `glyph`, in font units, to `builder` and returns its bounding
    /// box. `None` if the glyph has no outline, is malformed, or was refused (see
    /// [`refusal`](Self::refusal)); `builder` may then have received part of an outline.
    pub(crate) fn outline(&mut self, glyph: u16, builder: &mut dyn OutlineBuilder) -> Option<Rect> {
        match &mut self.outlines {
            Outlines::None => None,
            Outlines::TrueType(costs) => {
                let cost = costs.cost(glyph);
                if cost > MAX_GLYPH_COST {
                    self.refusal.get_or_insert(Refusal::GlyphTooComplex(glyph));
                    return None;
                }
                if u64::from(cost) > self.remaining {
                    self.refusal.get_or_insert(Refusal::BudgetExhausted(glyph));
                    return None;
                }
                self.remaining -= u64::from(cost);
                self.face.outline_glyph(GlyphId(glyph), builder)
            }
            Outlines::PostScript(font) => {
                let allowed =
                    u32::try_from(self.remaining).unwrap_or(u32::MAX).min(MAX_CHARSTRING_STEPS);
                let mut steps = allowed;
                let outcome =
                    font.outline(glyph, self.face.variation_coordinates(), &mut steps, builder);
                // One unit per glyph on top of the steps, so that glyphs without any
                // operator are not free.
                let used = u64::from(allowed - steps) + 1;
                self.remaining = self.remaining.saturating_sub(used);
                match outcome {
                    Ok(rect) => rect,
                    Err(Stop::Invalid) => None,
                    Err(Stop::TooComplex) => {
                        let refusal = if allowed < MAX_CHARSTRING_STEPS {
                            Refusal::BudgetExhausted(glyph)
                        } else {
                            Refusal::GlyphTooComplex(glyph)
                        };
                        self.refusal.get_or_insert(refusal);
                        None
                    }
                }
            }
        }
    }

    /// The bounding box of a glyph's outline in font units (the outline is computed and
    /// discarded).
    pub(crate) fn bounding_box(&mut self, glyph: u16) -> Option<Rect> {
        struct Discard;
        impl OutlineBuilder for Discard {
            fn move_to(&mut self, _: f32, _: f32) {}
            fn line_to(&mut self, _: f32, _: f32) {}
            fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
            fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
            fn close(&mut self) {}
        }
        self.outline(glyph, &mut Discard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `loca` (long format) and `glyf` for the given glyph programs.
    fn glyf_tables(glyphs: &[Vec<u8>]) -> (Vec<u8>, Vec<u8>) {
        let (mut loca, mut glyf) = (Vec::new(), Vec::new());
        for glyph in glyphs {
            loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
            glyf.extend_from_slice(glyph); // unpadded: the long format allows any offset
        }
        loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
        (loca, glyf)
    }

    fn composite(components: &[u16]) -> Vec<u8> {
        let mut glyph = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0];
        for (i, component) in components.iter().enumerate() {
            let more = if i + 1 < components.len() { 0x20 } else { 0 };
            glyph.extend_from_slice(&[0, 0x02 | more]);
            glyph.extend_from_slice(&component.to_be_bytes());
            glyph.extend_from_slice(&[0, 0]);
        }
        glyph
    }

    /// A simple glyph header claiming `points` points in one contour (no point data:
    /// the cost only reads the header).
    fn simple(points: u16) -> Vec<u8> {
        let mut glyph = vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        glyph.extend_from_slice(&(points - 1).to_be_bytes());
        glyph
    }

    fn costs_of(glyphs: &[Vec<u8>]) -> Vec<u32> {
        let (loca, glyf) = glyf_tables(glyphs);
        let mut head = vec![0u8; 54];
        head[18..20].copy_from_slice(&1000u16.to_be_bytes());
        head[50..52].copy_from_slice(&1u16.to_be_bytes()); // long loca
        let mut maxp = vec![0, 0, 0x50, 0];
        maxp.extend_from_slice(&(glyphs.len() as u16).to_be_bytes());
        let tables = RawFaceTables {
            head: &head,
            maxp: &maxp,
            loca: Some(&loca),
            glyf: Some(&glyf),
            ..RawFaceTables::default()
        };
        let mut costs = GlyfCosts::new(&tables).expect("usable tables");
        (0..glyphs.len() as u16).map(|glyph| costs.cost(glyph)).collect()
    }

    #[test]
    fn costs_add_up_over_the_component_tree() {
        let costs = costs_of(&[
            Vec::new(),            // 0: empty
            simple(4),             // 1: 1 + 4 points
            composite(&[1, 1, 0]), // 2: 1 + 3 references + 5 + 5 + 1
            composite(&[2, 2]),    // 3: 1 + 2 + 15 + 15
            composite(&[3, 9, 1]), // 4: 1 + 3 + 33 + 1 (no such glyph) + 5
        ]);
        assert_eq!(costs, [1, 5, 15, 33, 43]);
    }

    #[test]
    fn exponential_trees_cost_linear_time_and_saturate() {
        // 40 levels of fan-out 4 over an empty leaf: 4^40 references.
        let mut glyphs = vec![Vec::new()];
        for level in 1..=40u16 {
            glyphs.push(composite(&[level - 1; 4]));
        }
        let costs = costs_of(&glyphs);
        assert_eq!(costs[1], 1 + 4 + 4);
        assert_eq!(costs[2], 1 + 4 + 4 * 9);
        assert_eq!(costs[40], u32::MAX);
        assert!(costs[8] > MAX_GLYPH_COST);
    }

    #[test]
    fn cycles_and_very_long_chains_are_handled_without_recursion() {
        // 0 -> 1 -> 2 -> 0, and 3 -> 0.
        let costs = costs_of(&[composite(&[1]), composite(&[2]), composite(&[0]), composite(&[0])]);
        assert_eq!(costs, [u32::MAX; 4]);
        // A chain through 60 000 glyphs is fine (and cheap).
        let mut glyphs = vec![simple(3)];
        for glyph in 1..60_000u16 {
            glyphs.push(composite(&[glyph - 1]));
        }
        let costs = costs_of(&glyphs);
        assert_eq!(costs[0], 4);
        assert_eq!(costs[59_999], 4 + 2 * 59_999);
    }

    #[test]
    fn truncated_and_odd_component_records_do_not_panic() {
        let mut odd = composite(&[1, 1]);
        odd.truncate(odd.len() - 1); // the last record lacks a byte
        let words = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03, 0, 1, 0, 0, 0, 0];
        let no_offsets = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x20, 0, 1, 0, 0x00, 0, 1];
        let costs =
            costs_of(&[Vec::new(), simple(2), odd, words, no_offsets, vec![0xFF], vec![0, 5]]);
        assert_eq!(costs[2], 1 + 1 + 3, "only the complete record counts");
        assert_eq!(costs[3], 1 + 1 + 3);
        assert_eq!(costs[4], 1 + 2 + 3 + 3, "point-matching arguments are read as a record");
        assert_eq!((costs[5], costs[6]), (1, 1));
    }
}
