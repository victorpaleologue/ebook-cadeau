//! The few facts the PDF writer needs from a font file, read straight from its tables.
//!
//! Embedding a font takes its PostScript name, its `OS/2.fsType` licence bits, the kind
//! of outlines it has, the advance widths of the glyphs in use and a dozen numbers for
//! the font descriptor — all of them fields at fixed offsets of `head`, `hhea`, `maxp`,
//! `hmtx`, `OS/2`, `post` and `name`. They are read here with bounds-checked big-endian
//! reads; nothing in this module interprets glyph data (subsetting is left to the
//! `subsetter` crate) and no table is trusted to be as long as it claims.
//!
//! The table directory is read in file order: a later record replaces an earlier one
//! with the same tag, and a record pointing outside the file yields no table.

use std::collections::BTreeMap;

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

/// One face of a font file.
#[derive(Debug)]
pub(crate) struct Face<'a> {
    tables: BTreeMap<[u8; 4], &'a [u8]>,
    units_per_em: u16,
    glyph_count: u16,
}

/// One record of the `name` table.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NameRecord<'a> {
    pub(crate) platform_id: u16,
    pub(crate) encoding_id: u16,
    pub(crate) name_id: u16,
    /// The encoded string.
    pub(crate) bytes: &'a [u8],
}

impl NameRecord<'_> {
    /// Platform ID of the Macintosh platform.
    pub(crate) const MACINTOSH: u16 = 1;

    /// The string of a UTF-16BE record (the Unicode platform, or the Windows platform
    /// with the symbol or Unicode BMP encoding); `None` for other encodings and for
    /// ill-formed UTF-16.
    pub(crate) fn to_string(&self) -> Option<String> {
        let unicode = match self.platform_id {
            0 => true,
            3 => matches!(self.encoding_id, 0 | 1),
            _ => false,
        };
        if !unicode {
            return None;
        }
        let units: Vec<u16> =
            self.bytes.chunks_exact(2).map(|pair| u16::from_be_bytes([pair[0], pair[1]])).collect();
        String::from_utf16(&units).ok()
    }
}

/// Reads the table directory of face `face_index` (0 for a plain font file).
///
/// # Errors
/// A short description of what is wrong with the data.
fn directory(data: &[u8], face_index: u32) -> Result<BTreeMap<[u8; 4], &[u8]>, &'static str> {
    {
        const MALFORMED: &str = "malformed font";
        const UNKNOWN_MAGIC: &str = "unknown magic";
        const NO_FACE: &str = "face index is out of bounds";
        const COLLECTION: u32 = 0x7474_6366; // "ttcf"
        let is_font_magic = |magic: u32| matches!(magic, 0x0001_0000 | 0x7472_7565 | 0x4F54_544F);

        let mut at = 0usize;
        let magic = be_u32(data, at).ok_or(UNKNOWN_MAGIC)?;
        if magic == COLLECTION {
            let count = be_u32(data, 8).ok_or(MALFORMED)?;
            let offsets_end = usize::try_from(count)
                .ok()
                .and_then(|n| n.checked_mul(4))
                .and_then(|n| n.checked_add(12))
                .filter(|&end| end <= data.len())
                .ok_or(MALFORMED)?;
            if face_index >= count {
                return Err(NO_FACE);
            }
            let slot = 12 + 4 * face_index as usize; // inside the offset table: no overflow
            at = be_u32(data, slot).and_then(|o| usize::try_from(o).ok()).ok_or(MALFORMED)?;
            // A face lies after the collection header (and inside the file).
            if at < offsets_end || at > data.len() {
                return Err(MALFORMED);
            }
            let magic = be_u32(data, at).ok_or(UNKNOWN_MAGIC)?;
            if !is_font_magic(magic) {
                return Err(UNKNOWN_MAGIC);
            }
        } else if !is_font_magic(magic) {
            return Err(UNKNOWN_MAGIC);
        } else if face_index != 0 {
            return Err(NO_FACE);
        }

        let table_count = usize::from(be_u16(data, at + 4).ok_or(MALFORMED)?);
        let records = at + 12;
        if records + 16 * table_count > data.len() {
            return Err(MALFORMED);
        }
        let mut tables = BTreeMap::new();
        for record in (0..table_count).map(|i| records + 16 * i) {
            let Some(tag) = data.get(record..record + 4).and_then(|t| <[u8; 4]>::try_from(t).ok())
            else {
                continue;
            };
            let table = be_u32(data, record + 8).zip(be_u32(data, record + 12)).and_then(
                |(offset, length)| {
                    let start = usize::try_from(offset).ok()?;
                    let end = start.checked_add(usize::try_from(length).ok()?)?;
                    data.get(start..end)
                },
            );
            match table {
                Some(table) => tables.insert(tag, table),
                None => tables.remove(&tag),
            };
        }
        Ok(tables)
    }
}

/// The bytes of one table of a single-font file, whatever else the file contains (the
/// output of the subsetter is read with this).
pub(crate) fn table_of<'a>(data: &'a [u8], tag: &[u8; 4]) -> Option<&'a [u8]> {
    directory(data, 0).ok()?.get(tag).copied()
}

impl<'a> Face<'a> {
    /// Reads the table directory of face `face_index` (0 for a plain font file) and
    /// checks the tables without which a font is not a font: `head`, `hhea`, `maxp`.
    ///
    /// # Errors
    /// A short description of what is wrong with the data.
    pub(crate) fn parse(data: &'a [u8], face_index: u32) -> Result<Self, &'static str> {
        let tables = directory(data, face_index)?;
        // head: 54 bytes, a sane em square, a known `loca` format.
        let head = tables
            .get(b"head")
            .filter(|t| t.len() >= 54)
            .ok_or("the head table is missing or malformed")?;
        let units_per_em = be_u16(head, 18).unwrap_or(0);
        if !(16..=16384).contains(&units_per_em) || be_u16(head, 50).unwrap_or(u16::MAX) > 1 {
            return Err("the head table is missing or malformed");
        }
        if tables.get(b"hhea").is_none_or(|t| t.len() < 36) {
            return Err("the hhea table is missing or malformed");
        }
        let maxp = tables.get(b"maxp").ok_or("the maxp table is missing or malformed")?;
        let version = be_u32(maxp, 0).unwrap_or(0);
        let glyph_count = be_u16(maxp, 4).unwrap_or(0);
        if !(version == 0x0000_5000 || version == 0x0001_0000) || glyph_count == 0 {
            return Err("the maxp table is missing or malformed");
        }
        Ok(Self { tables, units_per_em, glyph_count })
    }

    /// The bytes of a table.
    pub(crate) fn table(&self, tag: &[u8; 4]) -> Option<&'a [u8]> {
        self.tables.get(tag).copied()
    }

    /// Font units per em, in 16..=16384.
    pub(crate) fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// Number of glyphs, at least 1.
    pub(crate) fn glyph_count(&self) -> u16 {
        self.glyph_count
    }

    /// `true` if the font has TrueType outlines (`glyf` with its `loca`).
    pub(crate) fn has_glyf(&self) -> bool {
        self.table(b"glyf").is_some() && self.table(b"loca").is_some()
    }

    /// `true` if the font has a `CFF ` table (version 1).
    pub(crate) fn has_cff(&self) -> bool {
        self.table(b"CFF ").is_some_and(|table| table.first() == Some(&1))
    }

    /// The advance width of a glyph in font units (`hmtx`); `None` if the font does
    /// not state one.
    pub(crate) fn advance(&self, glyph: u16) -> Option<u16> {
        let hmtx = self.table(b"hmtx")?;
        // The first `metrics` glyphs have an advance and a bearing (4 bytes); the last
        // advance applies to all remaining glyphs.
        let metrics = be_u16(self.table(b"hhea")?, 34).filter(|&n| n > 0)?;
        hmtx.get(..4 * usize::from(metrics))?;
        if glyph >= metrics.max(self.glyph_count) {
            return None;
        }
        be_u16(hmtx, 4 * usize::from(glyph.min(metrics - 1)))
    }

    /// The font bounding box of `head`: `[x_min, y_min, x_max, y_max]`.
    pub(crate) fn bounding_box(&self) -> [i16; 4] {
        let head = self.table(b"head").unwrap_or_default();
        [36, 38, 40, 42].map(|at| be_i16(head, at).unwrap_or(0))
    }

    /// The `OS/2` table, if it is long enough for the version it declares.
    fn os2(&self) -> Option<(&'a [u8], u16)> {
        let data = self.table(b"OS/2")?;
        let version = be_u16(data, 0)?;
        let length = match version {
            0 => 78,
            1 => 86,
            2..=4 => 96,
            5 => 100,
            _ => return None,
        };
        (data.len() >= length).then_some((data, version))
    }

    /// Ascender and descender of the horizontal line, chosen as FreeType does: the
    /// typographic ones when the font asks for them, else those of `hhea`, else (where
    /// `hhea` has none) the typographic or the Windows ones.
    fn line(&self) -> (i16, i16) {
        let hhea = self.table(b"hhea").unwrap_or_default();
        let mut ascender = be_i16(hhea, 4).unwrap_or(0);
        let mut descender = be_i16(hhea, 6).unwrap_or(0);
        let Some((os2, version)) = self.os2() else { return (ascender, descender) };
        let read = |at| be_i16(os2, at).unwrap_or(0);
        let (typo_ascender, typo_descender) = (read(68), read(70));
        const USE_TYPO_METRICS: u16 = 1 << 7;
        if version >= 4 && be_u16(os2, 62).unwrap_or(0) & USE_TYPO_METRICS != 0 {
            return (typo_ascender, typo_descender);
        }
        if ascender == 0 {
            ascender = if typo_ascender != 0 { typo_ascender } else { read(74) };
        }
        if descender == 0 {
            // `usWinDescent` is a distance below the baseline.
            let win_descender = read(76).checked_neg().unwrap_or(i16::MAX);
            descender = if typo_descender != 0 { typo_descender } else { win_descender };
        }
        (ascender, descender)
    }

    /// The ascender of the horizontal line (positive).
    pub(crate) fn ascender(&self) -> i16 {
        self.line().0
    }

    /// The descender of the horizontal line (negative).
    pub(crate) fn descender(&self) -> i16 {
        self.line().1
    }

    /// `OS/2.sCapHeight` (version 2 on).
    pub(crate) fn cap_height(&self) -> Option<i16> {
        let (os2, version) = self.os2()?;
        (version >= 2).then(|| be_i16(os2, 88)).flatten()
    }

    /// `OS/2.usWeightClass`; 400 for a font without a usable `OS/2` table.
    pub(crate) fn weight(&self) -> u16 {
        self.os2().map_or(400, |(os2, _)| be_u16(os2, 4).unwrap_or(0))
    }

    /// `(italic angle in degrees, is fixed pitch)` of a usable `post` table.
    fn post(&self) -> Option<(f32, bool)> {
        let data = self.table(b"post").filter(|t| t.len() >= 32)?;
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

    /// The italic angle in degrees (`post`), 0 for upright fonts.
    pub(crate) fn italic_angle(&self) -> f32 {
        self.post().map_or(0.0, |(angle, _)| angle)
    }

    /// `true` when the font declares itself monospaced (`post`).
    pub(crate) fn is_monospaced(&self) -> bool {
        self.post().is_some_and(|(_, fixed_pitch)| fixed_pitch)
    }

    /// `true` for a font whose `OS/2` style is italic, or that is slanted.
    pub(crate) fn is_italic(&self) -> bool {
        const ITALIC: u16 = 1 << 0;
        self.os2().is_some_and(|(os2, _)| be_u16(os2, 62).unwrap_or(0) & ITALIC != 0)
            || self.italic_angle() != 0.0
    }

    /// The records of the `name` table, up to the first one whose string does not lie
    /// inside the table.
    pub(crate) fn names(&self) -> impl Iterator<Item = NameRecord<'a>> {
        let data = self.table(b"name").unwrap_or_default();
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
        (0..count).map_while(move |i| {
            let at = records + 12 * i;
            let start = usize::from(be_u16(data, at + 10)?);
            let length = usize::from(be_u16(data, at + 8)?);
            Some(NameRecord {
                platform_id: be_u16(data, at)?,
                encoding_id: be_u16(data, at + 2)?,
                name_id: be_u16(data, at + 6)?,
                bytes: storage.get(start..start + length)?,
            })
        })
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
        head[18..20].copy_from_slice(&2048u16.to_be_bytes());
        for (at, v) in [(36, -100i16), (38, -400), (40, 2100), (42, 1900)] {
            head[at..at + 2].copy_from_slice(&v.to_be_bytes());
        }
        head
    }

    fn hhea(ascender: i16, descender: i16, metrics: u16) -> Vec<u8> {
        let mut hhea = vec![0u8; 36];
        hhea[4..6].copy_from_slice(&ascender.to_be_bytes());
        hhea[6..8].copy_from_slice(&descender.to_be_bytes());
        hhea[34..36].copy_from_slice(&metrics.to_be_bytes());
        hhea
    }

    fn maxp(glyphs: u16) -> Vec<u8> {
        let mut maxp = vec![0, 0, 0x50, 0];
        maxp.extend_from_slice(&glyphs.to_be_bytes());
        maxp
    }

    fn os2(version: u16, selection: u16, typo: (i16, i16), win: (u16, u16)) -> Vec<u8> {
        let mut os2 = vec![0u8; if version >= 2 { 96 } else { 86 }];
        os2[0..2].copy_from_slice(&version.to_be_bytes());
        os2[4..6].copy_from_slice(&700u16.to_be_bytes());
        os2[62..64].copy_from_slice(&selection.to_be_bytes());
        os2[68..70].copy_from_slice(&typo.0.to_be_bytes());
        os2[70..72].copy_from_slice(&typo.1.to_be_bytes());
        os2[74..76].copy_from_slice(&win.0.to_be_bytes());
        os2[76..78].copy_from_slice(&win.1.to_be_bytes());
        if version >= 2 {
            os2[88..90].copy_from_slice(&1400i16.to_be_bytes());
        }
        os2
    }

    #[test]
    fn directory_and_required_tables() {
        let good = sfnt(&[(b"head", head()), (b"hhea", hhea(1, -1, 1)), (b"maxp", maxp(3))]);
        let face = Face::parse(&good, 0).expect("usable");
        assert_eq!((face.units_per_em(), face.glyph_count()), (2048, 3));
        assert_eq!(face.bounding_box(), [-100, -400, 2100, 1900]);
        assert!(!face.has_glyf() && !face.has_cff());
        assert_eq!(Face::parse(&good, 1).expect_err("one face"), "face index is out of bounds");

        // The later of two records with the same tag is the font's; one that points
        // outside the file hides the table.
        let twice =
            [(b"head", head()), (b"hhea", hhea(1, -1, 1)), (b"maxp", maxp(3)), (b"maxp", maxp(9))];
        let mut bytes = sfnt(&twice);
        assert_eq!(Face::parse(&bytes, 0).expect("usable").glyph_count(), 9);
        let last_record = 12 + 16 * 3;
        bytes[last_record + 12..last_record + 16].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            Face::parse(&bytes, 0).expect_err("no maxp"),
            "the maxp table is missing or malformed"
        );

        for garbage in [&b""[..], b"OTTO", b"wOF2....", &[0u8; 64], &good[..40]] {
            assert!(Face::parse(garbage, 0).is_err());
        }
        let no_glyphs = sfnt(&[(b"head", head()), (b"hhea", hhea(1, -1, 1)), (b"maxp", maxp(0))]);
        assert!(Face::parse(&no_glyphs, 0).is_err());
        // A single table can be read from a file that is not a complete font.
        assert_eq!(table_of(&no_glyphs, b"maxp"), Some(&maxp(0)[..]));
        assert_eq!(table_of(&no_glyphs, b"CFF "), None);
        assert_eq!(table_of(b"not a font", b"maxp"), None);
        let mut odd_em = head();
        odd_em[18..20].copy_from_slice(&0u16.to_be_bytes());
        let no_em = sfnt(&[(b"head", odd_em), (b"hhea", hhea(1, -1, 1)), (b"maxp", maxp(1))]);
        assert_eq!(
            Face::parse(&no_em, 0).expect_err("no em"),
            "the head table is missing or malformed"
        );
    }

    #[test]
    fn advances_follow_the_number_of_metrics() {
        // Two full metrics (advance, bearing) for five glyphs: the last advance repeats.
        let hmtx = vec![1, 0, 0, 0, 2, 0, 0, 0, 0, 7, 0, 7, 0, 7];
        let font = |metrics: u16, hmtx: Vec<u8>| {
            sfnt(&[
                (b"head", head()),
                (b"hhea", hhea(1, -1, metrics)),
                (b"hmtx", hmtx),
                (b"maxp", maxp(5)),
            ])
        };
        let bytes = font(2, hmtx.clone());
        let face = Face::parse(&bytes, 0).expect("usable");
        let advances: Vec<_> = (0..7).map(|glyph| face.advance(glyph)).collect();
        assert_eq!(advances, [Some(256), Some(512), Some(512), Some(512), Some(512), None, None]);
        // No metrics at all, or a table shorter than its metrics: no advances.
        assert_eq!(Face::parse(&font(0, hmtx.clone()), 0).expect("usable").advance(0), None);
        assert_eq!(Face::parse(&font(4, hmtx), 0).expect("usable").advance(0), None);
    }

    #[test]
    fn line_metrics_are_chosen_like_freetype() {
        let font = |hhea_line: (i16, i16), os2: Option<Vec<u8>>| {
            let mut tables = vec![
                (b"head", head()),
                (b"hhea", hhea(hhea_line.0, hhea_line.1, 1)),
                (b"maxp", maxp(1)),
            ];
            if let Some(os2) = os2 {
                tables.insert(0, (b"OS/2", os2));
            }
            sfnt(&tables)
        };
        let line = |bytes: &[u8]| {
            let face = Face::parse(bytes, 0).expect("usable");
            (face.ascender(), face.descender())
        };
        // hhea, unless the font asks for its typographic metrics (version 4 on).
        assert_eq!(line(&font((1900, -500), None)), (1900, -500));
        let typo = os2(4, 0, (1500, -400), (2000, 600));
        assert_eq!(line(&font((1900, -500), Some(typo))), (1900, -500));
        let asks = os2(4, 1 << 7, (1500, -400), (2000, 600));
        assert_eq!(line(&font((1900, -500), Some(asks))), (1500, -400));
        let too_old = os2(3, 1 << 7, (1500, -400), (2000, 600));
        assert_eq!(line(&font((1900, -500), Some(too_old))), (1900, -500));
        // An empty hhea line falls back to the typographic, then to the Windows metrics.
        let typo = os2(4, 0, (1500, -400), (2000, 600));
        assert_eq!(line(&font((0, 0), Some(typo))), (1500, -400));
        let win_only = os2(4, 0, (0, 0), (2000, 600));
        assert_eq!(line(&font((0, 0), Some(win_only))), (2000, -600));

        let bytes = font((1900, -500), Some(os2(4, 1, (0, 0), (0, 0))));
        let face = Face::parse(&bytes, 0).expect("usable");
        assert_eq!((face.cap_height(), face.weight(), face.is_italic()), (Some(1400), 700, true));
        let bytes = font((1900, -500), Some(os2(1, 0, (0, 0), (0, 0))));
        let face = Face::parse(&bytes, 0).expect("usable");
        assert_eq!((face.cap_height(), face.weight(), face.is_italic()), (None, 700, false));
        // A table too short for its version is no table at all.
        let bytes = font((1900, -500), Some(os2(4, 1, (0, 0), (0, 0))[..90].to_vec()));
        let face = Face::parse(&bytes, 0).expect("usable");
        assert_eq!((face.cap_height(), face.weight(), face.is_italic()), (None, 400, false));
    }

    #[test]
    fn post_and_names() {
        let mut post = vec![0u8; 32];
        post[0..4].copy_from_slice(&0x0003_0000u32.to_be_bytes());
        post[4..8].copy_from_slice(&((-9i32 << 16) - 0x8000).to_be_bytes());
        post[12..16].copy_from_slice(&1u32.to_be_bytes());
        // name: a Windows record "Ab", a Macintosh record "Cd", then one out of bounds
        // and one that would be fine.
        let mut name = vec![0, 0, 0, 4, 0, 54];
        name.extend_from_slice(&[0, 3, 0, 1, 4, 9, 0, 6, 0, 4, 0, 0]);
        name.extend_from_slice(&[0, 1, 0, 0, 0, 0, 0, 4, 0, 2, 0, 4]);
        name.extend_from_slice(&[0, 3, 0, 1, 4, 9, 0, 1, 0, 9, 0, 9]);
        name.extend_from_slice(&[0, 3, 0, 1, 4, 9, 0, 2, 0, 2, 0, 0]);
        name.extend_from_slice(&[0, b'A', 0, b'b', b'C', b'd']);
        let bytes = sfnt(&[
            (b"head", head()),
            (b"hhea", hhea(1, -1, 1)),
            (b"maxp", maxp(1)),
            (b"name", name),
            (b"post", post.clone()),
        ]);
        let face = Face::parse(&bytes, 0).expect("usable");
        assert_eq!(
            (face.italic_angle(), face.is_monospaced(), face.is_italic()),
            (-9.5, true, true)
        );
        let names: Vec<_> = face.names().collect();
        assert_eq!(names.len(), 2, "the listing stops at the record that is out of bounds");
        assert_eq!((names[0].name_id, names[0].to_string().as_deref()), (6, Some("Ab")));
        assert_eq!(
            (names[1].platform_id, names[1].bytes, names[1].to_string()),
            (1, &b"Cd"[..], None)
        );

        // An unknown version, or a version 2 table without its glyph list: no `post`.
        post[0..4].copy_from_slice(&0x0002_0000u32.to_be_bytes());
        let bytes = sfnt(&[
            (b"head", head()),
            (b"hhea", hhea(1, -1, 1)),
            (b"maxp", maxp(1)),
            (b"post", post),
        ]);
        let face = Face::parse(&bytes, 0).expect("usable");
        assert_eq!(
            (face.italic_angle(), face.is_monospaced(), face.is_italic()),
            (0.0, false, false)
        );
        assert_eq!(face.names().count(), 0);
    }
}
