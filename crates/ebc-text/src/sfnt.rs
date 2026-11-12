//! The font file as this crate sees it: one collection of tables per face.
//!
//! [`Tables::collect`] reads the table directory once. Everything else is built on that
//! collection:
//!
//! * the few fixed-layout fields this crate reads itself (`head`, `hhea`, `maxp`, `OS/2`,
//!   `post`, `name`) — plain big-endian reads at known offsets;
//! * the two views handed to the font libraries ([`Tables::face`] for outlines, metrics
//!   and the character map; [`Tables::shaping_face`] for `harfrust`), which serve tables
//!   from the very same byte ranges.
//!
//! The analysis of this crate (glyph costs, embedding rights) and the libraries
//! therefore can never look at different bytes, whatever a hostile directory says:
//! duplicate tags, overlapping ranges and records pointing outside the file are resolved
//! here, once.
//!
//! Only the tables in [`KNOWN_TABLES`] are visible at all. Bitmap, colour (`COLR`, `SVG `,
//! `sbix`, …) and variable-composite (`VARC`) tables are not: this crate draws outlines
//! only, and those tables would merely give the libraries more to do on hostile input.

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use read_fonts::model::{Blob, Font as Face, Source};
use read_fonts::types::Tag;

use crate::error::FontError;

/// The tables that are read by this crate or shown to the font libraries.
const KNOWN_TABLES: [&[u8; 4]; 30] = [
    b"CFF ", b"CFF2", b"GDEF", b"GPOS", b"GSUB", b"HVAR", b"MVAR", b"OS/2", b"VORG", b"VVAR",
    b"ankr", b"avar", b"cmap", b"feat", b"fvar", b"glyf", b"gvar", b"head", b"hhea", b"hmtx",
    b"kern", b"kerx", b"loca", b"maxp", b"morx", b"name", b"post", b"trak", b"vhea", b"vmtx",
];

/// Tables the shaper never sees: through them it would run charstrings to measure glyphs
/// (fallback mark positioning), once per glyph of the text.
const HIDDEN_FROM_SHAPING: [&[u8; 4]; 2] = [b"CFF ", b"CFF2"];

/// Tables additionally hidden from the shaper of a font whose `glyf` outlines are too
/// expensive to be measured while shaping (see `outline::glyf_must_be_hidden`).
const GLYF_TABLES: [&[u8; 4]; 3] = [b"glyf", b"loca", b"gvar"];

fn be_u16(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at.checked_add(2)?)?;
    Some(u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]))
}

fn be_i16(data: &[u8], at: usize) -> Option<i16> {
    be_u16(data, at).map(|v| v as i16)
}

fn be_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at.checked_add(4)?)?;
    Some(u32::from_be_bytes([*bytes.first()?, *bytes.get(1)?, *bytes.get(2)?, *bytes.get(3)?]))
}

/// A table of the font, as a shareable byte buffer for the font libraries.
struct TableBytes {
    data: Arc<[u8]>,
    range: Range<usize>,
}

impl AsRef<[u8]> for TableBytes {
    fn as_ref(&self) -> &[u8] {
        self.data.get(self.range.clone()).unwrap_or_default()
    }
}

/// The tables of one face of a font file.
#[derive(Debug)]
pub(crate) struct Tables {
    data: Arc<[u8]>,
    /// Byte range of every known table that lies inside the file.
    ranges: BTreeMap<[u8; 4], Range<usize>>,
}

impl Tables {
    /// Reads the table directory of face `face_index` and checks that the three tables
    /// without which nothing works (`head`, `hhea`, `maxp`) are usable.
    ///
    /// The directory is read in file order; a later record replaces an earlier one with
    /// the same tag, and a record pointing outside the file yields no table.
    pub(crate) fn collect(data: &Arc<[u8]>, face_index: u32) -> Result<Self, FontError> {
        let malformed = || FontError::Parse("malformed font".into());
        let unknown_magic = || FontError::Parse("unknown magic".into());
        let is_font_magic = |magic: u32| matches!(magic, 0x0001_0000 | 0x7472_7565 | 0x4F54_544F);
        const COLLECTION: u32 = 0x7474_6366; // "ttcf"

        let mut at = 0usize;
        let magic = be_u32(data, at).ok_or_else(unknown_magic)?;
        if magic == COLLECTION {
            let count = be_u32(data, 8).ok_or_else(malformed)?;
            let offsets_end = usize::try_from(count)
                .ok()
                .and_then(|n| n.checked_mul(4))
                .and_then(|n| n.checked_add(12))
                .filter(|&end| end <= data.len())
                .ok_or_else(malformed)?;
            if face_index >= count {
                return Err(FontError::FaceIndex { index: face_index, count });
            }
            let slot = 12 + 4 * face_index as usize; // inside the offset table: no overflow
            let offset = be_u32(data, slot).ok_or_else(malformed)?;
            at = usize::try_from(offset).map_err(|_| malformed())?;
            // A face lies after the collection header (and inside the file).
            if at < offsets_end || at > data.len() {
                return Err(malformed());
            }
            let magic = be_u32(data, at).ok_or_else(unknown_magic)?;
            if !is_font_magic(magic) {
                return Err(unknown_magic());
            }
        } else if !is_font_magic(magic) {
            return Err(unknown_magic());
        } else if face_index != 0 {
            return Err(FontError::FaceIndex { index: face_index, count: 1 });
        }

        let table_count = usize::from(be_u16(data, at + 4).ok_or_else(malformed)?);
        let records = at + 12;
        let records_end = records + 16 * table_count;
        if records_end > data.len() {
            return Err(malformed());
        }
        let mut ranges = BTreeMap::new();
        for record in (0..table_count).map(|i| records + 16 * i) {
            let Some(tag) = data.get(record..record + 4).and_then(|t| <[u8; 4]>::try_from(t).ok())
            else {
                continue;
            };
            if !KNOWN_TABLES.contains(&&tag) {
                continue;
            }
            let range = be_u32(data, record + 8).zip(be_u32(data, record + 12)).and_then(
                |(offset, length)| {
                    let start = usize::try_from(offset).ok()?;
                    let end = start.checked_add(usize::try_from(length).ok()?)?;
                    (end <= data.len()).then_some(start..end)
                },
            );
            match range {
                Some(range) => ranges.insert(tag, range),
                None => ranges.remove(&tag),
            };
        }

        let tables = Self { data: Arc::clone(data), ranges };
        tables.check_required()?;
        Ok(tables)
    }

    fn check_required(&self) -> Result<(), FontError> {
        let broken =
            |table: &str| FontError::Parse(format!("the {table} table is missing or malformed"));
        // head: 54 bytes, a sane em square, a known `loca` format.
        let head = self.get(b"head").filter(|t| t.len() >= 54).ok_or_else(|| broken("head"))?;
        let units_per_em = be_u16(head, 18).unwrap_or(0);
        let loca_format = be_u16(head, 50).unwrap_or(u16::MAX);
        if !(16..=16384).contains(&units_per_em) || loca_format > 1 {
            return Err(broken("head"));
        }
        self.get(b"hhea").filter(|t| t.len() >= 36).ok_or_else(|| broken("hhea"))?;
        let maxp = self.get(b"maxp").ok_or_else(|| broken("maxp"))?;
        let version = be_u32(maxp, 0).unwrap_or(0);
        let glyphs = be_u16(maxp, 4).unwrap_or(0);
        if !(version == 0x0000_5000 || version == 0x0001_0000) || glyphs == 0 {
            return Err(broken("maxp"));
        }
        Ok(())
    }

    /// The bytes of a table, if the font has it (and it is one of [`KNOWN_TABLES`]).
    pub(crate) fn get(&self, tag: &[u8; 4]) -> Option<&[u8]> {
        self.data.get(self.ranges.get(tag)?.clone())
    }

    pub(crate) fn has(&self, tag: &[u8; 4]) -> bool {
        self.ranges.contains_key(tag)
    }

    /// The font as the font libraries see it for outlines, metrics, variations and the
    /// character map.
    pub(crate) fn face(self: &Arc<Self>) -> Option<Face> {
        self.view(&[])
    }

    /// The font as the *shaper* sees it: without the tables through which a hostile font
    /// could make shaping run glyph programs (`CFF `, `CFF2`; with `hide_glyf` also
    /// `glyf`, `loca` and `gvar`). Substitution, kerning, mark attachment and advances
    /// from `hmtx`/`HVAR` are unaffected.
    pub(crate) fn shaping_face(self: &Arc<Self>, hide_glyf: bool) -> Option<Face> {
        let mut hidden: Vec<&'static [u8; 4]> = HIDDEN_FROM_SHAPING.to_vec();
        if hide_glyf {
            hidden.extend_from_slice(&GLYF_TABLES);
        }
        self.view(&hidden)
    }

    fn view(self: &Arc<Self>, hidden: &[&'static [u8; 4]]) -> Option<Face> {
        let tables = Arc::clone(self);
        let hidden: Vec<[u8; 4]> = hidden.iter().map(|tag| **tag).collect();
        let lookup: Arc<dyn Fn(Tag) -> Option<Blob> + Send + Sync> = Arc::new(move |tag: Tag| {
            let tag = tag.to_be_bytes();
            if hidden.contains(&tag) {
                return None;
            }
            let range = tables.ranges.get(&tag)?.clone();
            let bytes: Arc<dyn AsRef<[u8]> + Send + Sync> =
                Arc::new(TableBytes { data: Arc::clone(&tables.data), range });
            Some(Blob::from(bytes))
        });
        Face::new(Source::from(lookup), 0)
    }

    // -----------------------------------------------------------------------------------
    // head, hhea, maxp (validated by `collect`)
    // -----------------------------------------------------------------------------------

    /// Font units per em, in 16..=16384.
    pub(crate) fn units_per_em(&self) -> u16 {
        self.get(b"head").and_then(|t| be_u16(t, 18)).unwrap_or(1000)
    }

    /// The font bounding box of `head`: `(y_min, y_max)`.
    pub(crate) fn vertical_bounds(&self) -> (i16, i16) {
        let head = self.get(b"head").unwrap_or_default();
        (be_i16(head, 38).unwrap_or(0), be_i16(head, 42).unwrap_or(0))
    }

    /// Ascender, descender and line gap of `hhea`.
    pub(crate) fn hhea_line(&self) -> (i16, i16, i16) {
        let hhea = self.get(b"hhea").unwrap_or_default();
        let read = |at| be_i16(hhea, at).unwrap_or(0);
        (read(4), read(6), read(8))
    }

    /// Number of glyphs (`maxp`), at least 1.
    pub(crate) fn glyph_count(&self) -> u16 {
        self.get(b"maxp").and_then(|t| be_u16(t, 4)).unwrap_or(1)
    }

    // -----------------------------------------------------------------------------------
    // OS/2
    // -----------------------------------------------------------------------------------

    /// `(fsType, version)` read from the raw `OS/2` table, so that a restriction is
    /// honoured even when the rest of the table is too short for its declared version.
    pub(crate) fn fs_type(&self) -> Option<(u16, u16)> {
        let table = self.get(b"OS/2")?;
        Some((be_u16(table, 8)?, be_u16(table, 0)?))
    }

    /// The `OS/2` table, if it is long enough for the version it declares.
    pub(crate) fn os2(&self) -> Option<Os2<'_>> {
        let data = self.get(b"OS/2")?;
        let version = be_u16(data, 0)?;
        let length = match version {
            0 => 78,
            1 => 86,
            2..=4 => 96,
            5 => 100,
            _ => return None,
        };
        (data.len() >= length).then_some(Os2 { data, version })
    }

    // -----------------------------------------------------------------------------------
    // post
    // -----------------------------------------------------------------------------------

    /// `(italic angle in degrees, is fixed pitch)` of a usable `post` table.
    pub(crate) fn post(&self) -> Option<(f32, bool)> {
        let data = self.get(b"post").filter(|t| t.len() >= 32)?;
        let version = be_u32(data, 0)?;
        if !matches!(version, 0x0001_0000 | 0x0002_0000 | 0x0002_5000 | 0x0003_0000 | 0x0004_0000) {
            return None;
        }
        if version == 0x0002_0000 {
            // Version 2 lists a glyph name index per glyph; a table too short for its own
            // list is not a `post` table.
            let names = usize::from(be_u16(data, 32)?);
            data.get(34..34 + 2 * names)?;
        }
        let italic_angle = be_u32(data, 4)? as i32 as f32 / 65536.0;
        Some((italic_angle, be_u32(data, 12)? != 0))
    }

    // -----------------------------------------------------------------------------------
    // name
    // -----------------------------------------------------------------------------------

    /// The records of the `name` table whose string lies inside the table.
    pub(crate) fn names(&self) -> impl Iterator<Item = NameRecord<'_>> {
        let data = self.get(b"name").unwrap_or_default();
        let layout = (|| {
            let version = be_u16(data, 0)?;
            let count = usize::from(be_u16(data, 2)?);
            let storage_offset = usize::from(be_u16(data, 4)?);
            let records = match version {
                0 => 6,
                // Version 1 has language-tag records (4 bytes each) before the names.
                1 => 8 + 4 * usize::from(be_u16(data, 6)?),
                _ => return None,
            };
            let records_end = records + 12 * count;
            let storage = data.get(storage_offset.max(records_end)..)?;
            Some((records, count, storage))
        })();
        let (records, count, storage) = layout.unwrap_or((0, 0, &[]));
        (0..count).filter_map(move |i| {
            let at = records + 12 * i;
            let start = usize::from(be_u16(data, at + 10)?);
            let length = usize::from(be_u16(data, at + 8)?);
            Some(NameRecord {
                platform_id: be_u16(data, at)?,
                encoding_id: be_u16(data, at + 2)?,
                language_id: be_u16(data, at + 4)?,
                name_id: be_u16(data, at + 6)?,
                bytes: storage.get(start..start + length)?,
            })
        })
    }
}

/// A usable `OS/2` table.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Os2<'a> {
    data: &'a [u8],
    version: u16,
}

impl Os2<'_> {
    const ITALIC: u16 = 1 << 0;
    const USE_TYPO_METRICS: u16 = 1 << 7;
    const OBLIQUE: u16 = 1 << 9;

    fn i16_at(&self, at: usize) -> i16 {
        be_i16(self.data, at).unwrap_or(0)
    }

    fn selection(&self) -> u16 {
        be_u16(self.data, 62).unwrap_or(0)
    }

    /// `usWeightClass`.
    pub(crate) fn weight(&self) -> u16 {
        be_u16(self.data, 4).unwrap_or(0)
    }

    /// The italic bit of `fsSelection`.
    pub(crate) fn is_italic(&self) -> bool {
        self.selection() & Self::ITALIC != 0
    }

    /// The oblique bit of `fsSelection` (defined from version 4 on).
    pub(crate) fn is_oblique(&self) -> bool {
        self.version >= 4 && self.selection() & Self::OBLIQUE != 0
    }

    /// The font asks for its typographic line metrics to be used (version 4 on).
    pub(crate) fn use_typo_metrics(&self) -> bool {
        self.version >= 4 && self.selection() & Self::USE_TYPO_METRICS != 0
    }

    /// `sTypoAscender`, `sTypoDescender`, `sTypoLineGap`.
    pub(crate) fn typo_line(&self) -> (i16, i16, i16) {
        (self.i16_at(68), self.i16_at(70), self.i16_at(72))
    }

    /// `usWinAscent` and `usWinDescent`, the latter negated (below the baseline).
    pub(crate) fn win_line(&self) -> (i16, i16) {
        (self.i16_at(74), self.i16_at(76).checked_neg().unwrap_or(i16::MAX))
    }

    /// `sxHeight` (version 2 on).
    pub(crate) fn x_height(&self) -> Option<i16> {
        (self.version >= 2).then(|| be_i16(self.data, 86)).flatten()
    }

    /// `sCapHeight` (version 2 on).
    pub(crate) fn cap_height(&self) -> Option<i16> {
        (self.version >= 2).then(|| be_i16(self.data, 88)).flatten()
    }
}

/// One record of the `name` table.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NameRecord<'a> {
    pub(crate) platform_id: u16,
    pub(crate) encoding_id: u16,
    pub(crate) language_id: u16,
    pub(crate) name_id: u16,
    /// The encoded string.
    pub(crate) bytes: &'a [u8],
}

impl NameRecord<'_> {
    /// Platform ID of the Unicode platform.
    pub(crate) const UNICODE: u16 = 0;
    /// Platform ID of the Macintosh platform.
    pub(crate) const MACINTOSH: u16 = 1;
    /// Platform ID of the Windows platform.
    pub(crate) const WINDOWS: u16 = 3;

    /// `true` when the string is UTF-16BE: the Unicode platform, or the Windows platform
    /// with the symbol (0) or Unicode BMP (1) encoding.
    pub(crate) fn is_unicode(&self) -> bool {
        match self.platform_id {
            Self::UNICODE => true,
            Self::WINDOWS => matches!(self.encoding_id, 0 | 1),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sfnt(tables: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut out = vec![0, 1, 0, 0];
        out.extend_from_slice(&(tables.len() as u16).to_be_bytes());
        out.extend_from_slice(&[0; 6]);
        let mut offset = 12 + 16 * tables.len();
        for (tag, data) in tables {
            out.extend_from_slice(*tag);
            out.extend_from_slice(&[0; 4]);
            out.extend_from_slice(&(offset as u32).to_be_bytes());
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            offset += data.len();
        }
        for (_, data) in tables {
            out.extend_from_slice(data);
        }
        out
    }

    fn head() -> Vec<u8> {
        let mut head = vec![0u8; 54];
        head[18..20].copy_from_slice(&1000u16.to_be_bytes());
        head
    }

    fn maxp(glyphs: u16) -> Vec<u8> {
        let mut maxp = vec![0, 0, 0x50, 0];
        maxp.extend_from_slice(&glyphs.to_be_bytes());
        maxp
    }

    fn collect(bytes: Vec<u8>) -> Result<Tables, FontError> {
        Tables::collect(&Arc::from(bytes), 0)
    }

    #[test]
    fn the_last_record_of_a_tag_wins_and_records_outside_the_file_hide_the_table() {
        let good = [(b"head", head()), (b"hhea", vec![0; 36]), (b"maxp", maxp(3))];
        let tables = collect(sfnt(&good)).expect("usable");
        assert_eq!((tables.units_per_em(), tables.glyph_count()), (1000, 3));

        // Two `maxp` records: the later one is the font's.
        let twice =
            [(b"head", head()), (b"hhea", vec![0; 36]), (b"maxp", maxp(3)), (b"maxp", maxp(7))];
        assert_eq!(collect(sfnt(&twice)).expect("usable").glyph_count(), 7);

        // A later record pointing outside the file removes the table again.
        let mut bytes = sfnt(&twice);
        let last_record = 12 + 16 * 3;
        bytes[last_record + 12..last_record + 16].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(collect(bytes), Err(FontError::Parse(m)) if m.contains("maxp")));

        // Unknown tables are not visible at all.
        let extra = [
            (b"COLR", vec![1, 2, 3]),
            (b"head", head()),
            (b"hhea", vec![0; 36]),
            (b"maxp", maxp(3)),
        ];
        let tables = collect(sfnt(&extra)).expect("usable");
        assert!(tables.get(b"COLR").is_none() && tables.has(b"head"));
    }

    #[test]
    fn the_required_tables_are_checked() {
        let with = |head: Vec<u8>, hhea: Vec<u8>, maxp: Vec<u8>| {
            collect(sfnt(&[(b"head", head), (b"hhea", hhea), (b"maxp", maxp)]))
        };
        assert!(with(head(), vec![0; 36], maxp(1)).is_ok());
        assert!(with(head()[..53].to_vec(), vec![0; 36], maxp(1)).is_err());
        assert!(with(head(), vec![0; 35], maxp(1)).is_err());
        assert!(with(head(), vec![0; 36], maxp(0)).is_err());
        assert!(with(head(), vec![0; 36], vec![0, 2, 0, 0, 0, 1]).is_err(), "maxp version");
        let mut tiny_em = head();
        tiny_em[18..20].copy_from_slice(&15u16.to_be_bytes());
        assert!(with(tiny_em, vec![0; 36], maxp(1)).is_err());
        let mut odd_loca = head();
        odd_loca[50..52].copy_from_slice(&2u16.to_be_bytes());
        assert!(with(odd_loca, vec![0; 36], maxp(1)).is_err());
        assert!(collect(b"wOFF\0\0\0\0".to_vec()).is_err());
        assert!(collect(Vec::new()).is_err());
        assert!(matches!(
            Tables::collect(&Arc::from(sfnt(&[])), 3),
            Err(FontError::FaceIndex { index: 3, count: 1 })
        ));
    }

    #[test]
    fn views_serve_exactly_the_collected_tables() {
        use read_fonts::TableProvider as _;
        // A `cmap` header without subtables, and a (made up) `COLR` table.
        let font = [
            (b"CFF ", vec![9; 8]),
            (b"COLR", vec![0; 14]),
            (b"cmap", vec![0, 0, 0, 0]),
            (b"glyf", vec![7; 4]),
            (b"head", head()),
            (b"hhea", vec![0; 36]),
            (b"loca", vec![0; 6]),
            (b"maxp", maxp(2)),
        ];
        let tables = Arc::new(collect(sfnt(&font)).expect("usable"));
        let seen = |face: &Face| {
            let tables = face.tables();
            (
                tables.head().is_ok(),
                tables.cmap().is_ok(),
                tables.glyf().is_ok(),
                tables.loca(false).is_ok(),
                tables.colr().is_ok(),
            )
        };
        // The complete view: everything this crate knows, and nothing else.
        let face = tables.face().expect("a face");
        assert_eq!((face.units_per_em(), face.num_glyphs()), (1000, 2));
        assert_eq!(seen(&face), (true, true, true, true, false));
        // The shaper never sees charstrings (the shaping view of this font has no
        // outlines at all), and loses `glyf` and `loca` on request.
        assert_eq!(
            seen(&tables.shaping_face(false).expect("a face")),
            (true, true, true, true, false)
        );
        assert_eq!(
            seen(&tables.shaping_face(true).expect("a face")),
            (true, true, false, false, false)
        );
        let outlines = |face: &Face| face.outline_glyphs().underlying_source();
        assert!(outlines(&tables.shaping_face(true).expect("a face")).is_none());
    }

    #[test]
    fn os2_post_and_name_follow_their_versions() {
        let mut os2 = vec![0u8; 96];
        os2[0..2].copy_from_slice(&4u16.to_be_bytes());
        os2[4..6].copy_from_slice(&600u16.to_be_bytes());
        os2[8..10].copy_from_slice(&0x0104u16.to_be_bytes());
        os2[62..64].copy_from_slice(&0x0281u16.to_be_bytes()); // italic, typo metrics, oblique
        os2[76..78].copy_from_slice(&250u16.to_be_bytes());
        os2[88..90].copy_from_slice(&700i16.to_be_bytes());
        let mut post = vec![0u8; 32];
        post[0..4].copy_from_slice(&0x0003_0000u32.to_be_bytes());
        post[4..8].copy_from_slice(&(-12i32 << 16).to_be_bytes());
        post[12..16].copy_from_slice(&1u32.to_be_bytes());
        // name: one Windows record "Ab", one record whose string is out of bounds.
        let mut name = vec![0, 0, 0, 2, 0, 30];
        name.extend_from_slice(&[0, 3, 0, 1, 4, 9, 0, 1, 0, 4, 0, 0]);
        name.extend_from_slice(&[0, 1, 0, 0, 0, 0, 0, 2, 0, 9, 0, 2]);
        name.extend_from_slice(&[0, b'A', 0, b'b']);
        let font = |os2: Vec<u8>, post: Vec<u8>| {
            collect(sfnt(&[
                (b"OS/2", os2),
                (b"head", head()),
                (b"hhea", vec![0; 36]),
                (b"maxp", maxp(2)),
                (b"name", name.clone()),
                (b"post", post),
            ]))
            .expect("usable")
        };

        let tables = font(os2.clone(), post.clone());
        let table = tables.os2().expect("valid");
        assert_eq!(tables.fs_type(), Some((0x0104, 4)));
        assert_eq!((table.weight(), table.is_italic(), table.is_oblique()), (600, true, true));
        assert!(table.use_typo_metrics());
        assert_eq!((table.win_line().1, table.cap_height()), (-250, Some(700)));
        assert_eq!(tables.post(), Some((-12.0, true)));
        let names: Vec<_> = tables.names().collect();
        assert_eq!(names.len(), 1);
        assert_eq!(
            (names[0].name_id, names[0].bytes, names[0].is_unicode()),
            (1, &[0, 65, 0, 98][..], true)
        );

        // Too short for its version: no table, but fsType is still readable.
        let tables = font(os2[..90].to_vec(), post[..31].to_vec());
        assert!(tables.os2().is_none() && tables.post().is_none());
        assert_eq!(tables.fs_type(), Some((0x0104, 4)));
        // Version 1 has no cap height and no oblique or typo-metrics bits.
        let mut old = os2.clone();
        old[0..2].copy_from_slice(&1u16.to_be_bytes());
        let tables = font(old, post);
        let table = tables.os2().expect("valid");
        assert_eq!(
            (table.cap_height(), table.is_oblique(), table.use_typo_metrics()),
            (None, false, false)
        );
    }
}
