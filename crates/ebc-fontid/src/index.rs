//! The font index: per font, the descriptor of every character of a reference set,
//! plus a compact, versioned binary serialisation.

use std::fmt;
use std::sync::Arc;

use ebc_text::{Font, ShapeOptions, try_rasterize};

use crate::descriptor::{BINS, CELLS, FEATURE_LEN, Features, SCALARS, describe, remap_coverage};
use crate::error::FontIdError;
use crate::gray::Gray;

/// The characters every font is described by: `A–Z`, `a–z`, `0–9` and `& ! ?`.
///
/// Small punctuation is left out on purpose: once normalised, a full stop or a hyphen
/// looks the same in almost every typeface. Accented letters are matched through their
/// base letter (the accent is cut off before comparing).
pub const REFERENCE_CHARS: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789&!?";

/// Resolution at which reference glyphs are rendered, in pixels per em.
pub(crate) const REF_PX_PER_EM: f32 = 96.0;
/// Geometry bytes stored after the features of a reference glyph.
pub(crate) const GEOM: usize = 3;
/// Bytes of one reference record: features + geometry.
pub(crate) const RECORD_LEN: usize = FEATURE_LEN + GEOM;
/// Geometry: ink height in em, `byte / 170`.
pub(crate) const G_HEIGHT: usize = FEATURE_LEN;
/// Geometry: ink width in em, `byte / 170`.
pub(crate) const G_WIDTH: usize = FEATURE_LEN + 1;
/// Geometry: position of the ink's bottom edge below the baseline in em, `byte / 255 - 0.5`.
pub(crate) const G_BOTTOM: usize = FEATURE_LEN + 2;
const GEOM_SIZE_SCALE: f32 = 170.0;

const MAGIC: &[u8; 8] = b"EBCFIDX\0";
const HEADER_LEN: usize = 20;
const MAX_CHARS: usize = 4096;

/// Metadata of a font to be indexed (see [`FontIndex::build`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexEntryMeta {
    /// Family name, e.g. `"Spectral"`.
    pub family: String,
    /// Style within the family, e.g. `"SemiBold Italic"`.
    pub style: String,
    /// Licence, preferably an SPDX identifier such as `"OFL-1.1"`.
    pub licence: String,
    /// Where the font file can be fetched from (URL or application-relative path).
    pub source_url: String,
}

impl IndexEntryMeta {
    /// Metadata read from the font's own name table. The licence is the font's licence
    /// URL or description when it has one, else `"unknown"`.
    #[must_use]
    pub fn from_font(font: &Font, source_url: impl Into<String>) -> Self {
        let licence = font
            .licence_url()
            .or_else(|| font.licence_description())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        Self {
            family: font.family_name(),
            style: font.style_name(),
            licence,
            source_url: source_url.into(),
        }
    }
}

/// The reference character set of an index and the slot of each character.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CharTable {
    chars: Vec<char>,
    sorted: Vec<(char, u16)>,
}

impl CharTable {
    /// Builds the table; `None` if a character repeats or there are too many.
    pub(crate) fn new(chars: Vec<char>) -> Option<Self> {
        if chars.len() > MAX_CHARS {
            return None;
        }
        let mut sorted: Vec<(char, u16)> =
            chars.iter().enumerate().map(|(i, &c)| (c, i as u16)).collect();
        sorted.sort_unstable();
        if sorted.windows(2).any(|p| p[0].0 == p[1].0) {
            return None;
        }
        Some(Self { chars, sorted })
    }

    pub(crate) fn reference() -> Arc<Self> {
        // The constant has no duplicates and 65 characters, so this cannot fail; the
        // fallback keeps the function total anyway.
        Arc::new(
            Self::new(REFERENCE_CHARS.chars().collect())
                .unwrap_or(Self { chars: Vec::new(), sorted: Vec::new() }),
        )
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline]
    pub(crate) fn chars(&self) -> &[char] {
        &self.chars
    }

    #[inline]
    pub(crate) fn slot(&self, ch: char) -> Option<usize> {
        self.sorted
            .binary_search_by_key(&ch, |&(c, _)| c)
            .ok()
            .map(|i| usize::from(self.sorted[i].1))
    }
}

/// The descriptors of one face: a record per character of the table (absent characters
/// have no record).
#[derive(PartialEq, Eq)]
pub(crate) struct FaceDescriptor {
    table: Arc<CharTable>,
    present: Vec<bool>,
    /// `table.len() * RECORD_LEN` bytes; zero where the character is absent.
    records: Vec<u8>,
}

impl FaceDescriptor {
    /// Renders and describes every character of `table` that `font` can draw.
    pub(crate) fn from_font(font: &Font, table: &Arc<CharTable>) -> Self {
        let n = table.len();
        let mut present = vec![false; n];
        let mut records = vec![0u8; n * RECORD_LEN];
        for (slot, &ch) in table.chars().iter().enumerate() {
            if let Some(record) = reference_record(font, ch) {
                present[slot] = true;
                records[slot * RECORD_LEN..(slot + 1) * RECORD_LEN].copy_from_slice(&record);
            }
        }
        Self { table: Arc::clone(table), present, records }
    }

    /// The record of `ch`, if the face has that character.
    #[inline]
    pub(crate) fn record(&self, ch: char) -> Option<&[u8]> {
        self.record_at(self.table.slot(ch)?)
    }

    /// The record in `slot`, if present.
    #[inline]
    pub(crate) fn record_at(&self, slot: usize) -> Option<&[u8]> {
        if *self.present.get(slot)? {
            self.records.get(slot * RECORD_LEN..(slot + 1) * RECORD_LEN)
        } else {
            None
        }
    }

    /// Number of characters this face has a record for.
    pub(crate) fn coverage(&self) -> usize {
        self.present.iter().filter(|&&p| p).count()
    }

    /// The characters of the table this face is described over.
    pub(crate) fn chars(&self) -> &[char] {
        self.table.chars()
    }

    /// The mean record per character of `table` over the given faces (looked up by
    /// character, so the faces may have been described over other tables).
    pub(crate) fn centroid<'a>(
        table: &Arc<CharTable>,
        faces: impl Iterator<Item = &'a FaceDescriptor> + Clone,
    ) -> Self {
        let n = table.len();
        let mut present = vec![false; n];
        let mut records = vec![0u8; n * RECORD_LEN];
        let mut sums = [0u32; RECORD_LEN];
        for (slot, &ch) in table.chars().iter().enumerate() {
            sums.iter_mut().for_each(|s| *s = 0);
            let mut count = 0u32;
            for face in faces.clone() {
                let record = if Arc::ptr_eq(&face.table, table) {
                    face.record_at(slot)
                } else {
                    face.record(ch)
                };
                if let Some(record) = record {
                    count += 1;
                    for (s, &b) in sums.iter_mut().zip(record) {
                        *s += u32::from(b);
                    }
                }
            }
            if count > 0 {
                present[slot] = true;
                for (o, s) in records[slot * RECORD_LEN..(slot + 1) * RECORD_LEN].iter_mut().zip(&sums) {
                    *o = ((s + count / 2) / count) as u8;
                }
            }
        }
        Self { table: Arc::clone(table), present, records }
    }
}

/// Describes `ch` as drawn by `font`: features of the rendered glyph and its geometry in
/// em units. `None` if the font does not have the character or draws nothing for it.
pub(crate) fn reference_record(font: &Font, ch: char) -> Option<[u8; RECORD_LEN]> {
    font.glyph_id(ch)?;
    let mut buf = [0u8; 4];
    let text: &str = ch.encode_utf8(&mut buf);
    let raster = try_rasterize(font, text, REF_PX_PER_EM, &ShapeOptions::default()).ok()?;
    let mask = raster.mask;
    let (w, h) = (mask.width as usize, mask.height as usize);
    if w == 0 || h == 0 || mask.data.len() < w * h {
        return None;
    }
    let soft = Gray { w, h, px: mask.data[..w * h].iter().map(|&v| remap_coverage(v)).collect() };
    let (crop, (_, _, _, y1)) = core_crop(&soft)?;
    let Features(features) = describe(&crop)?;
    let mut record = [0u8; RECORD_LEN];
    record[..FEATURE_LEN].copy_from_slice(&features);
    let size_byte = |px: usize| {
        (px as f32 / REF_PX_PER_EM * GEOM_SIZE_SCALE + 0.5).clamp(0.0, 255.0) as u8
    };
    record[G_HEIGHT] = size_byte(crop.h);
    record[G_WIDTH] = size_byte(crop.w);
    let bottom_em = (raster.top as f32 + y1 as f32) / REF_PX_PER_EM;
    record[G_BOTTOM] = ((bottom_em + 0.5) * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    Some(record)
}

/// Geometry of a reference record in em: `(height, width, bottom below baseline)`.
#[inline]
pub(crate) fn record_geometry(record: &[u8]) -> (f32, f32, f32) {
    (
        f32::from(record[G_HEIGHT]) / GEOM_SIZE_SCALE,
        f32::from(record[G_WIDTH]) / GEOM_SIZE_SCALE,
        f32::from(record[G_BOTTOM]) / 255.0 - 0.5,
    )
}

/// Cuts a coverage image down to its *core* — the bounding box of the pixels that are at
/// least half covered — and clears everything farther than one pixel from a core pixel.
/// Returns the crop and the box `(x0, y0, x1, y1)` (exclusive ends) in `soft`.
pub(crate) fn core_crop(soft: &Gray) -> Option<(Gray, (usize, usize, usize, usize))> {
    let (w, h) = (soft.w, soft.h);
    let core = |x: usize, y: usize| soft.px[y * w + x] >= 128;
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
    for y in 0..h {
        for x in 0..w {
            if core(x, y) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x0 == usize::MAX {
        return None;
    }
    let (cw, ch) = (x1 - x0, y1 - y0);
    let mut px = vec![0u8; cw * ch];
    for y in y0..y1 {
        for x in x0..x1 {
            let near_core = (y.saturating_sub(1)..=(y + 1).min(h - 1))
                .any(|ny| (x.saturating_sub(1)..=(x + 1).min(w - 1)).any(|nx| core(nx, ny)));
            if near_core {
                px[(y - y0) * cw + (x - x0)] = soft.px[y * w + x];
            }
        }
    }
    Some((Gray { w: cw, h: ch, px }, (x0, y0, x1, y1)))
}

/// One font of a [`FontIndex`]: its metadata and (privately) its glyph descriptors.
///
/// Cloning is cheap: the descriptors are shared.
#[derive(Clone, PartialEq, Eq)]
pub struct IndexEntry {
    /// Family name, e.g. `"Spectral"`.
    pub family: String,
    /// Style within the family, e.g. `"SemiBold"`.
    pub style: String,
    /// Licence of the font (SPDX identifier or URL).
    pub licence: String,
    /// Where the font file can be fetched from.
    pub source_url: String,
    pub(crate) descriptor: Arc<FaceDescriptor>,
}

impl IndexEntry {
    /// Describes a single font outside any index — what [`rerank`](crate::rerank) needs
    /// for candidates that were not found through a [`FontIndex`].
    #[must_use]
    pub fn from_font(meta: IndexEntryMeta, font: &Font) -> Self {
        Self::with_table(meta, font, &CharTable::reference())
    }

    fn with_table(meta: IndexEntryMeta, font: &Font, table: &Arc<CharTable>) -> Self {
        Self {
            family: meta.family,
            style: meta.style,
            licence: meta.licence,
            source_url: meta.source_url,
            descriptor: Arc::new(FaceDescriptor::from_font(font, table)),
        }
    }

    /// Number of reference characters this font can draw (out of
    /// [`REFERENCE_CHARS`]`.chars().count()` for an index built by this version).
    #[must_use]
    pub fn coverage(&self) -> usize {
        self.descriptor.coverage()
    }

    /// The metadata of this entry.
    #[must_use]
    pub fn meta(&self) -> IndexEntryMeta {
        IndexEntryMeta {
            family: self.family.clone(),
            style: self.style.clone(),
            licence: self.licence.clone(),
            source_url: self.source_url.clone(),
        }
    }
}

impl fmt::Debug for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexEntry")
            .field("family", &self.family)
            .field("style", &self.style)
            .field("licence", &self.licence)
            .field("source_url", &self.source_url)
            .field("coverage", &self.coverage())
            .finish()
    }
}

/// An index of fonts to identify against.
///
/// Build it once from font files with [`FontIndex::build`] (see the `build-index`
/// example), ship it as bytes ([`FontIndex::to_bytes`]) and load it in the browser with
/// [`FontIndex::from_bytes`]. It holds no font data, only descriptors: about 9 KB per
/// font.
#[derive(Clone, PartialEq, Eq)]
pub struct FontIndex {
    table: Arc<CharTable>,
    entries: Vec<IndexEntry>,
    /// Per character slot, the mean record over the entries that have it.
    centroids: Arc<FaceDescriptor>,
}

impl fmt::Debug for FontIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontIndex")
            .field("chars", &self.table.len())
            .field("entries", &self.entries)
            .finish()
    }
}

impl Default for FontIndex {
    fn default() -> Self {
        Self::from_entries(CharTable::reference(), Vec::new())
    }
}

impl FontIndex {
    /// Version of the binary format written by [`to_bytes`](Self::to_bytes).
    pub const FORMAT_VERSION: u16 = 1;

    /// Describes every font: each character of [`REFERENCE_CHARS`] the font can draw is
    /// rendered with [`ebc_text::try_rasterize`] and reduced to a descriptor.
    ///
    /// Entries keep the order of `fonts`. A font that draws none of the reference
    /// characters still gets an entry (it will never match). Takes about 10 ms per font.
    #[must_use]
    pub fn build(fonts: &[(IndexEntryMeta, Font)]) -> Self {
        let table = CharTable::reference();
        let entries = fonts
            .iter()
            .map(|(meta, font)| IndexEntry::with_table(meta.clone(), font, &table))
            .collect();
        Self::from_entries(table, entries)
    }

    fn from_entries(table: Arc<CharTable>, entries: Vec<IndexEntry>) -> Self {
        let centroids =
            Arc::new(FaceDescriptor::centroid(&table, entries.iter().map(|e| &*e.descriptor)));
        Self { table, entries, centroids }
    }

    /// The fonts of the index, in the order they were given to [`build`](Self::build).
    #[must_use]
    pub fn entries(&self) -> &[IndexEntry] {
        &self.entries
    }

    /// Number of fonts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` if the index holds no font.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The reference characters of this index.
    #[must_use]
    pub fn chars(&self) -> &[char] {
        self.table.chars()
    }

    pub(crate) fn centroids(&self) -> &FaceDescriptor {
        &self.centroids
    }

    /// Serialises the index.
    ///
    /// Layout (little-endian): magic `"EBCFIDX\0"`, `u16` version, four layout bytes
    /// (histogram cells per side, orientation bins, scalar count, geometry count), `u16`
    /// character count, `u32` entry count, the characters as `u32` code points, then per
    /// entry four length-prefixed (`u16`) UTF-8 strings (family, style, licence, source
    /// URL), a presence bitmap and one record per present character; finally a `u32`
    /// FNV-1a checksum of everything before it. The output is deterministic.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let n = self.table.len();
        let bitmap_len = n.div_ceil(8);
        let mut out = Vec::with_capacity(
            HEADER_LEN + n * 4 + self.entries.len() * (64 + bitmap_len + n * RECORD_LEN) + 4,
        );
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&Self::FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&[CELLS as u8, BINS as u8, SCALARS as u8, GEOM as u8]);
        out.extend_from_slice(&(n as u16).to_le_bytes());
        out.extend_from_slice(&(self.entries.len().min(u32::MAX as usize) as u32).to_le_bytes());
        for &ch in self.table.chars() {
            out.extend_from_slice(&u32::from(ch).to_le_bytes());
        }
        for entry in self.entries.iter().take(u32::MAX as usize) {
            for s in [&entry.family, &entry.style, &entry.licence, &entry.source_url] {
                let bytes = truncate_utf8(s, usize::from(u16::MAX));
                out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
                out.extend_from_slice(bytes);
            }
            let mut bitmap = vec![0u8; bitmap_len];
            for slot in 0..n {
                if entry.descriptor.record_at(slot).is_some() {
                    bitmap[slot / 8] |= 1 << (slot % 8);
                }
            }
            out.extend_from_slice(&bitmap);
            for slot in 0..n {
                if let Some(record) = entry.descriptor.record_at(slot) {
                    out.extend_from_slice(record);
                }
            }
        }
        let checksum = fnv1a(&out);
        out.extend_from_slice(&checksum.to_le_bytes());
        out
    }

    /// Parses an index written by [`to_bytes`](Self::to_bytes).
    ///
    /// The data is treated as untrusted: every length is checked against the remaining
    /// input before anything is allocated or read.
    ///
    /// # Errors
    /// [`FontIdError::BadMagic`], [`FontIdError::UnsupportedVersion`],
    /// [`FontIdError::UnsupportedLayout`], [`FontIdError::Truncated`],
    /// [`FontIdError::Checksum`] or [`FontIdError::Corrupt`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FontIdError> {
        let mut r = Reader { bytes, pos: 0 };
        if r.take(MAGIC.len()).map_err(|_| FontIdError::BadMagic)? != MAGIC {
            return Err(FontIdError::BadMagic);
        }
        let version = r.u16()?;
        if version != Self::FORMAT_VERSION {
            return Err(FontIdError::UnsupportedVersion {
                found: version,
                supported: Self::FORMAT_VERSION,
            });
        }
        let layout = r.take(4)?;
        if layout != [CELLS as u8, BINS as u8, SCALARS as u8, GEOM as u8] {
            return Err(FontIdError::UnsupportedLayout);
        }
        let n_chars = usize::from(r.u16()?);
        let n_entries = r.u32()? as usize;
        if n_chars > MAX_CHARS {
            return Err(FontIdError::Corrupt("too many reference characters"));
        }

        // The checksum covers everything but its own four bytes.
        let Some(body_len) = bytes.len().checked_sub(4).filter(|&l| l >= r.pos) else {
            return Err(FontIdError::Truncated { offset: bytes.len(), needed: 4 });
        };
        let stored = u32::from_le_bytes([
            bytes[body_len],
            bytes[body_len + 1],
            bytes[body_len + 2],
            bytes[body_len + 3],
        ]);
        if fnv1a(&bytes[..body_len]) != stored {
            return Err(FontIdError::Checksum);
        }
        let mut r = Reader { bytes: &bytes[..body_len], pos: r.pos };

        let mut chars = Vec::with_capacity(n_chars);
        for _ in 0..n_chars {
            let code = r.u32()?;
            chars.push(char::from_u32(code).ok_or(FontIdError::Corrupt("invalid character"))?);
        }
        let table =
            Arc::new(CharTable::new(chars).ok_or(FontIdError::Corrupt("repeated character"))?);

        let bitmap_len = n_chars.div_ceil(8);
        // Every entry takes at least four string lengths and a bitmap.
        let min_entry = 8 + bitmap_len;
        if n_entries > r.remaining() / min_entry.max(1) {
            return Err(FontIdError::Truncated {
                offset: r.pos,
                needed: n_entries.saturating_mul(min_entry),
            });
        }
        let mut entries = Vec::with_capacity(n_entries);
        for _ in 0..n_entries {
            let family = r.string()?;
            let style = r.string()?;
            let licence = r.string()?;
            let source_url = r.string()?;
            let bitmap = r.take(bitmap_len)?;
            if n_chars % 8 != 0 && bitmap_len > 0 && bitmap[bitmap_len - 1] >> (n_chars % 8) != 0 {
                return Err(FontIdError::Corrupt("presence bits beyond the character set"));
            }
            let present: Vec<bool> =
                (0..n_chars).map(|slot| bitmap[slot / 8] >> (slot % 8) & 1 == 1).collect();
            let n_present = present.iter().filter(|&&p| p).count();
            let data = r.take(n_present * RECORD_LEN)?;
            let mut records = vec![0u8; n_chars * RECORD_LEN];
            let mut next = 0;
            for (slot, &p) in present.iter().enumerate() {
                if p {
                    records[slot * RECORD_LEN..(slot + 1) * RECORD_LEN]
                        .copy_from_slice(&data[next..next + RECORD_LEN]);
                    next += RECORD_LEN;
                }
            }
            entries.push(IndexEntry {
                family,
                style,
                licence,
                source_url,
                descriptor: Arc::new(FaceDescriptor {
                    table: Arc::clone(&table),
                    present,
                    records,
                }),
            });
        }
        if r.remaining() != 0 {
            return Err(FontIdError::Corrupt("trailing bytes after the last entry"));
        }
        Ok(Self::from_entries(table, entries))
    }
}

/// The longest prefix of `s` that fits in `max` bytes and ends on a character boundary.
fn truncate_utf8(s: &str, max: usize) -> &[u8] {
    if s.len() <= max {
        return s.as_bytes();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s.as_bytes()[..end]
}

/// 32-bit FNV-1a.
fn fnv1a(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for &b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// Bounds-checked cursor over untrusted bytes.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], FontIdError> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&end| end <= self.bytes.len())
            .ok_or(FontIdError::Truncated { offset: self.pos, needed: n })?;
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16, FontIdError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, FontIdError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn string(&mut self) -> Result<String, FontIdError> {
        let len = usize::from(self.u16()?);
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| FontIdError::Corrupt("string is not valid UTF-8"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_table_maps_characters_to_slots() {
        let table = CharTable::reference();
        assert_eq!(table.len(), 65);
        assert_eq!(table.slot('A'), Some(0));
        assert_eq!(table.slot('a'), Some(26));
        assert_eq!(table.slot('0'), Some(52));
        assert_eq!(table.slot('?'), Some(64));
        assert_eq!(table.slot('é'), None);
        assert!(CharTable::new(vec!['a', 'b', 'a']).is_none());
        assert!(CharTable::new(vec![]).is_some());
    }

    #[test]
    fn core_crop_keeps_a_one_pixel_halo_and_drops_the_rest() {
        #[rustfmt::skip]
        let px = vec![
            90,  0,   0,   0,  0,
            0,   60, 200, 60,  0,
            0,   60, 255, 60, 90,
            0,   0,   60,  0,  0,
        ];
        let (crop, bbox) = core_crop(&Gray { w: 5, h: 4, px }).expect("has a core");
        assert_eq!(bbox, (2, 1, 3, 3));
        assert_eq!((crop.w, crop.h), (1, 2));
        assert_eq!(crop.px, vec![200, 255]);
        assert!(core_crop(&Gray::filled(3, 3, 100)).is_none());
    }

    #[test]
    fn empty_index_round_trips() {
        let index = FontIndex::default();
        assert!(index.is_empty());
        assert_eq!(index.chars().len(), 65);
        let bytes = index.to_bytes();
        assert_eq!(bytes.len(), HEADER_LEN + 65 * 4 + 4);
        assert_eq!(FontIndex::from_bytes(&bytes).expect("round trip"), index);
    }

    #[test]
    fn utf8_truncation_respects_boundaries() {
        assert_eq!(truncate_utf8("abc", 10), b"abc");
        assert_eq!(truncate_utf8("aé", 2), b"a");
        assert_eq!(truncate_utf8("aé", 3), "aé".as_bytes());
        assert_eq!(truncate_utf8("é", 0), b"");
    }

    #[test]
    fn fnv_reference_values() {
        assert_eq!(fnv1a(b""), 0x811c_9dc5);
        assert_eq!(fnv1a(b"a"), 0xe40c_292c);
        assert_eq!(fnv1a(b"foobar"), 0xbf9c_f968);
    }
}
