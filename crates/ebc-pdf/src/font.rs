//! Font embedding: licence check, subsetting, and the Type 0 / CIDFont object graph.
//!
//! Every font is written as a composite (Type 0) font with the `Identity-H` encoding, so
//! content streams address glyphs by two-byte CIDs:
//!
//! * TrueType (`glyf`) outlines become a `CIDFontType2` with a `FontFile2` program and an
//!   identity `CIDToGIDMap`;
//! * CFF outlines become a `CIDFontType0` with a bare CFF `FontFile3` program
//!   (`/Subtype /CIDFontType0C`).
//!
//! Fonts are subset to the glyphs the document uses (the subsetter renumbers glyphs, and
//! the new glyph id is the CID). Fonts whose `OS/2.fsType` forbids subsetting are embedded
//! whole; fonts whose `fsType` forbids embedding are refused.

use std::borrow::Cow;
use std::collections::BTreeMap;

use ebc_scene::FontResource;
use pdf_writer::types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Chunk, Filter, Finish, Name, Rect, Ref, Str};
use subsetter::GlyphRemapper;
use ttf_parser::{Face, GlyphId, RawFace, Tag, name_id};

use crate::error::{EmbeddingRestriction, PdfError};
use crate::md5;
use crate::writer::{RefAllocator, deflate, stream_len};

/// Longest text (in UTF-16 code units) a single glyph may map to in the `ToUnicode` CMap.
/// Longer texts are truncated; a real ligature or cluster never comes close.
const MAX_TO_UNICODE_UNITS: usize = 256;

/// The glyphs of one font that the document uses.
#[derive(Debug, Default)]
pub(crate) struct FontUsage {
    /// Glyph id → the text it stands for (first non-empty occurrence wins; empty when the
    /// glyph was only ever seen without text).
    pub(crate) glyphs: BTreeMap<u16, String>,
}

impl FontUsage {
    /// Records one use of `glyph` standing for `text`.
    pub(crate) fn record(&mut self, glyph: u16, text: &str) {
        let slot = self.glyphs.entry(glyph).or_default();
        if slot.is_empty() && !text.is_empty() {
            let mut units = 0;
            for c in text.chars() {
                units += c.len_utf16();
                if units > MAX_TO_UNICODE_UNITS {
                    break;
                }
                slot.push(c);
            }
        }
    }
}

/// Which kind of outlines the font program holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outlines {
    /// TrueType outlines (`glyf`).
    Glyf,
    /// Compact Font Format outlines (`CFF `).
    Cff,
}

/// Licence flags from `OS/2.fsType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct EmbeddingRights {
    /// Embedding is not allowed (restricted licence, or bitmap-only).
    pub(crate) forbidden: Option<EmbeddingRestriction>,
    /// The font may only be embedded whole.
    pub(crate) no_subsetting: bool,
}

impl EmbeddingRights {
    /// Interprets a raw `OS/2` table. A font without the table carries no restriction.
    pub(crate) fn from_os2(os2: Option<&[u8]>) -> Self {
        let Some(table) = os2 else { return Self::default() };
        let (Some(version), Some(fs_type)) = (read_u16(table, 0), read_u16(table, 8)) else {
            return Self::default();
        };
        let level = fs_type & 0x000E;
        let restricted = if level == 0 {
            false
        } else if version <= 2 {
            // Up to version 2 several bits may be set and the most permissive one wins.
            level & 0x000C == 0
        } else {
            // From version 3 the bits are mutually exclusive; when a font sets the
            // restricted bit together with others we stay on the safe side.
            level & 0x0002 != 0
        };
        // The no-subsetting and bitmap-only bits were introduced with version 2.
        let no_subsetting = version >= 2 && fs_type & 0x0100 != 0;
        let bitmap_only = version >= 2 && fs_type & 0x0200 != 0;
        let forbidden = if restricted {
            Some(EmbeddingRestriction::Restricted)
        } else if bitmap_only {
            Some(EmbeddingRestriction::BitmapOnly)
        } else {
            None
        };
        Self { forbidden, no_subsetting }
    }
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*data.get(offset)?, *data.get(offset + 1)?]))
}

/// How glyph ids of the original font map to CIDs of the embedded font.
#[derive(Debug)]
enum CidMap {
    /// The font is embedded whole: CID = glyph id.
    Identity,
    /// The font is subset: original glyph id → new glyph id (= CID).
    Subset(BTreeMap<u16, u16>),
}

/// Metrics for the font descriptor, in thousandths of an em.
#[derive(Debug, Clone, Copy)]
struct Descriptor {
    /// Bits of the PDF font `Flags` entry.
    flags: u32,
    bbox: [f32; 4],
    italic_angle: f32,
    ascent: f32,
    descent: f32,
    cap_height: f32,
    stem_v: f32,
}

/// A font ready to be written to the PDF.
#[derive(Debug)]
pub(crate) struct EmbeddedFont {
    /// Name of the font in page resource dictionaries, e.g. `F0`.
    resource_name: Vec<u8>,
    /// PostScript name, with a subset tag when the font is subset.
    base_name: String,
    outlines: Outlines,
    /// The font program (`FontFile2` or `FontFile3` data, uncompressed).
    program: Vec<u8>,
    cids: CidMap,
    /// CID → advance width in thousandths of an em, exactly as written to `/W`.
    widths: BTreeMap<u16, f32>,
    /// CID → text, for the `ToUnicode` CMap.
    to_unicode: Vec<(u16, String)>,
    descriptor: Descriptor,
}

impl EmbeddedFont {
    /// Checks the licence flags of `resource`, subsets it to `usage` and gathers everything
    /// needed to write the font. `index` numbers the fonts of the document.
    pub(crate) fn prepare(
        resource: &FontResource,
        usage: &FontUsage,
        index: usize,
    ) -> Result<Self, PdfError> {
        let id = resource.id;
        let invalid = |reason: String| PdfError::InvalidFont { font: id, reason };
        let data: &[u8] = &resource.data;

        let face = Face::parse(data, resource.face_index).map_err(|e| invalid(e.to_string()))?;
        let units_per_em = f64::from(face.units_per_em());
        if units_per_em <= 0.0 {
            return Err(invalid("units per em is zero".into()));
        }
        let glyph_count = face.number_of_glyphs();
        if let Some(&glyph) = usage.glyphs.keys().find(|&&g| g >= glyph_count) {
            return Err(PdfError::InvalidGlyph { font: id, glyph, glyph_count });
        }

        let ps_name = postscript_name(&face);
        let rights = EmbeddingRights::from_os2(face.raw_face().table(Tag::from_bytes(b"OS/2")));
        if let Some(restriction) = rights.forbidden {
            return Err(PdfError::FontEmbeddingForbidden { font: id, name: ps_name, restriction });
        }

        let outlines = if face.tables().glyf.is_some() {
            Outlines::Glyf
        } else if face.tables().cff.is_some() {
            Outlines::Cff
        } else {
            return Err(invalid(
                "the font has neither TrueType (glyf) nor CFF outlines; CFF2 and bitmap fonts \
                 are not supported"
                    .into(),
            ));
        };

        let is_collection = data.starts_with(b"ttcf");
        let (program, cids, whole) =
            if rights.no_subsetting && outlines == Outlines::Glyf && !is_collection {
                // The licence asks for the complete font: embed the file as it is.
                (data.to_vec(), CidMap::Identity, true)
            } else {
                let mut remapper = GlyphRemapper::new();
                let mut map = BTreeMap::new();
                if rights.no_subsetting {
                    // Keep every glyph (new id = old id); only the container is rewritten.
                    for glyph in 0..glyph_count {
                        map.insert(glyph, remapper.remap(glyph));
                    }
                } else {
                    map.insert(0, 0);
                    for &glyph in usage.glyphs.keys() {
                        map.insert(glyph, remapper.remap(glyph));
                    }
                }
                let subset = subsetter::subset(data, resource.face_index, &remapper)
                    .map_err(|e| invalid(format!("subsetting failed: {e}")))?;
                let program = match outlines {
                    Outlines::Glyf => subset,
                    Outlines::Cff => RawFace::parse(&subset, 0)
                        .ok()
                        .and_then(|raw| raw.table(Tag::from_bytes(b"CFF ")))
                        .map(<[u8]>::to_vec)
                        .ok_or_else(|| invalid("the subset font has no CFF table".into()))?,
                };
                (program, CidMap::Subset(map), rights.no_subsetting)
            };

        let scale = 1000.0 / units_per_em;
        let mut widths = BTreeMap::new();
        let mut to_unicode = Vec::new();
        for (&glyph, text) in &usage.glyphs {
            let cid = match &cids {
                CidMap::Identity => glyph,
                CidMap::Subset(map) => *map
                    .get(&glyph)
                    .ok_or_else(|| invalid("internal error: glyph missing from subset".into()))?,
            };
            let advance = f64::from(face.glyph_hor_advance(GlyphId(glyph)).unwrap_or(0));
            widths.insert(cid, (advance * scale) as f32);
            if !text.is_empty() {
                to_unicode.push((cid, text.clone()));
            }
        }
        to_unicode.sort_by_key(|(cid, _)| *cid);

        let base_name = if whole {
            ps_name
        } else {
            format!("{}+{}", subset_tag(data, resource.face_index, index, usage), ps_name)
        };

        Ok(Self {
            resource_name: format!("F{index}").into_bytes(),
            base_name,
            outlines,
            program,
            cids,
            widths,
            to_unicode,
            descriptor: descriptor(&face, scale),
        })
    }

    /// The name under which pages reference the font.
    pub(crate) fn resource_name(&self) -> Name<'_> {
        Name(&self.resource_name)
    }

    /// The CID that shows `glyph`, if the glyph is part of the embedded font.
    pub(crate) fn cid(&self, glyph: u16) -> Option<u16> {
        match &self.cids {
            CidMap::Identity => self.widths.contains_key(&glyph).then_some(glyph),
            CidMap::Subset(map) => map.get(&glyph).copied(),
        }
    }

    /// Advance width of `cid` in thousandths of an em, exactly as declared in the PDF.
    pub(crate) fn width(&self, cid: u16) -> f32 {
        self.widths.get(&cid).copied().unwrap_or(0.0)
    }

    /// Writes the font objects and returns the reference of the Type 0 font dictionary.
    pub(crate) fn write(
        &self,
        chunk: &mut Chunk,
        refs: &mut RefAllocator,
        compress: bool,
    ) -> Result<Ref, PdfError> {
        let type0_ref = refs.next()?;
        let cid_ref = refs.next()?;
        let descriptor_ref = refs.next()?;
        let file_ref = refs.next()?;

        let base_name = Name(self.base_name.as_bytes());
        let to_unicode_ref = if self.to_unicode.is_empty() { None } else { Some(refs.next()?) };

        // Type 0 (composite) font.
        let type0_name: Cow<'_, [u8]> = match self.outlines {
            Outlines::Glyf => Cow::Borrowed(self.base_name.as_bytes()),
            Outlines::Cff => Cow::Owned(format!("{}-Identity-H", self.base_name).into_bytes()),
        };
        let mut type0 = chunk.type0_font(type0_ref);
        type0
            .base_font(Name(&type0_name))
            .encoding_predefined(Name(b"Identity-H"))
            .descendant_font(cid_ref);
        if let Some(to_unicode_ref) = to_unicode_ref {
            type0.to_unicode(to_unicode_ref);
        }
        type0.finish();

        // Descendant CID font.
        let mut cid_font = chunk.cid_font(cid_ref);
        cid_font
            .subtype(match self.outlines {
                Outlines::Glyf => CidFontType::Type2,
                Outlines::Cff => CidFontType::Type0,
            })
            .base_font(base_name)
            .system_info(SystemInfo {
                registry: Str(b"Adobe"),
                ordering: Str(b"Identity"),
                supplement: 0,
            })
            .font_descriptor(descriptor_ref);
        if !self.widths.is_empty() {
            let mut w = cid_font.widths();
            let mut run_start = 0u16;
            let mut run: Vec<f32> = Vec::new();
            for (&cid, &width) in &self.widths {
                // Runs are capped so that no array exceeds what PDF 1.3 consumers support.
                let contiguous = !run.is_empty()
                    && run.len() < 4_000
                    && usize::from(cid) == usize::from(run_start) + run.len();
                if !contiguous && !run.is_empty() {
                    w.consecutive(run_start, run.drain(..));
                }
                if run.is_empty() {
                    run_start = cid;
                }
                run.push(width);
            }
            if !run.is_empty() {
                w.consecutive(run_start, run.drain(..));
            }
        }
        if self.outlines == Outlines::Glyf {
            cid_font.cid_to_gid_map_predefined(Name(b"Identity"));
        }
        cid_font.finish();

        // Font descriptor.
        let d = &self.descriptor;
        let mut descriptor = chunk.font_descriptor(descriptor_ref);
        descriptor
            .name(base_name)
            .flags(FontFlags::from_bits_truncate(d.flags))
            .bbox(Rect::new(d.bbox[0], d.bbox[1], d.bbox[2], d.bbox[3]))
            .italic_angle(d.italic_angle)
            .ascent(d.ascent)
            .descent(d.descent)
            .cap_height(d.cap_height)
            .stem_v(d.stem_v);
        match self.outlines {
            Outlines::Glyf => descriptor.font_file2(file_ref),
            Outlines::Cff => descriptor.font_file3(file_ref),
        };
        descriptor.finish();

        // Font program.
        let data: Cow<'_, [u8]> = if compress {
            Cow::Owned(deflate(&self.program))
        } else {
            Cow::Borrowed(&self.program)
        };
        stream_len(data.len(), "an embedded font")?;
        let length1 = stream_len(self.program.len(), "an embedded font")?;
        let mut file = chunk.stream(file_ref, &data);
        if compress {
            file.filter(Filter::FlateDecode);
        }
        match self.outlines {
            Outlines::Glyf => file.pair(Name(b"Length1"), length1),
            Outlines::Cff => file.pair(Name(b"Subtype"), Name(b"CIDFontType0C")),
        };
        file.finish();

        // ToUnicode CMap.
        if let Some(to_unicode_ref) = to_unicode_ref {
            let mut cmap = UnicodeCmap::<u16>::new(
                Name(b"Adobe-Identity-UCS"),
                SystemInfo { registry: Str(b"Adobe"), ordering: Str(b"UCS"), supplement: 0 },
            );
            for (cid, text) in &self.to_unicode {
                cmap.pair_with_multiple(*cid, text.chars());
            }
            let raw = cmap.finish();
            let data: Cow<'_, [u8]> =
                if compress { Cow::Owned(deflate(&raw)) } else { Cow::Borrowed(raw.as_slice()) };
            stream_len(data.len(), "a ToUnicode CMap")?;
            let mut stream = chunk.cmap(to_unicode_ref, &data);
            if compress {
                stream.filter(Filter::FlateDecode);
            }
            stream.finish();
        }

        Ok(type0_ref)
    }

    /// `true` when the complete font (every glyph) is embedded, i.e. the name carries
    /// no subset tag.
    #[cfg(test)]
    pub(crate) fn is_whole(&self) -> bool {
        !self.base_name.contains('+')
    }

    /// The `BaseFont` name.
    #[cfg(test)]
    pub(crate) fn base_name(&self) -> &str {
        &self.base_name
    }
}

/// Six upper-case letters identifying a subset, derived from the font and its glyph set.
fn subset_tag(data: &[u8], face_index: u32, index: usize, usage: &FontUsage) -> String {
    let mut hasher = md5::Md5::new();
    hasher.update(data).update(&face_index.to_be_bytes()).update(&(index as u64).to_be_bytes());
    for glyph in usage.glyphs.keys() {
        hasher.update(&glyph.to_be_bytes());
    }
    hasher.finish()[..6].iter().map(|b| char::from(b'A' + b % 26)).collect()
}

/// The font's PostScript name, reduced to characters that are safe in a PDF name.
fn postscript_name(face: &Face<'_>) -> String {
    let mut best: Option<(u8, String)> = None;
    for name in face.names() {
        let rank = match name.name_id {
            name_id::POST_SCRIPT_NAME => 0,
            name_id::FULL_NAME => 1,
            name_id::FAMILY => 2,
            _ => continue,
        };
        if best.as_ref().is_some_and(|(r, _)| *r <= rank) {
            continue;
        }
        let text = name.to_string().or_else(|| {
            // Macintosh Roman names: accept the ASCII subset.
            (name.platform_id == ttf_parser::PlatformId::Macintosh && name.name.is_ascii())
                .then(|| String::from_utf8_lossy(name.name).into_owned())
        });
        if let Some(text) = text {
            let clean = sanitize_name(&text);
            if !clean.is_empty() {
                best = Some((rank, clean));
            }
        }
    }
    best.map_or_else(|| "Font".to_owned(), |(_, name)| name)
}

fn sanitize_name(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .take(63)
        .collect()
}

fn descriptor(face: &Face<'_>, scale: f64) -> Descriptor {
    let scaled = |v: i16| (f64::from(v) * scale) as f32;
    let bbox = face.global_bounding_box();
    let ascent = scaled(face.ascender());
    let cap_height = face.capital_height().filter(|h| *h > 0).map_or(ascent, scaled);
    let italic_angle = face.italic_angle();
    let italic_angle = if italic_angle.is_finite() { italic_angle } else { 0.0 };

    let mut flags = FontFlags::SYMBOLIC;
    if face.is_monospaced() {
        flags |= FontFlags::FIXED_PITCH;
    }
    if face.is_italic() || italic_angle != 0.0 {
        flags |= FontFlags::ITALIC;
    }

    // The dominant vertical stem width is not stored in OpenType fonts; this is the
    // customary estimate from the weight class.
    let weight = f32::from(face.weight().to_number().clamp(1, 1000));
    let stem_v = (10.0 + 0.244 * (weight - 50.0)).max(10.0);

    Descriptor {
        flags: flags.bits(),
        bbox: [scaled(bbox.x_min), scaled(bbox.y_min), scaled(bbox.x_max), scaled(bbox.y_max)],
        italic_angle,
        ascent,
        descent: scaled(face.descender()),
        cap_height,
        stem_v,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ebc_scene::FontId;

    use super::*;

    fn font_bytes(name: &str) -> Vec<u8> {
        let path = format!("{}/../../assets/fonts/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(path).expect("font asset")
    }

    fn os2(version: u16, fs_type: u16) -> Vec<u8> {
        let mut table = vec![0u8; 78];
        table[0..2].copy_from_slice(&version.to_be_bytes());
        table[8..10].copy_from_slice(&fs_type.to_be_bytes());
        table
    }

    fn rights(version: u16, fs_type: u16) -> EmbeddingRights {
        EmbeddingRights::from_os2(Some(&os2(version, fs_type)))
    }

    #[test]
    fn fs_type_interpretation() {
        let free = EmbeddingRights { forbidden: None, no_subsetting: false };
        assert_eq!(EmbeddingRights::from_os2(None), free);
        assert_eq!(EmbeddingRights::from_os2(Some(&[0, 3])), free, "truncated table");
        assert_eq!(rights(4, 0x0000), free, "installable");
        assert_eq!(rights(4, 0x0004), free, "preview & print");
        assert_eq!(rights(4, 0x0008), free, "editable");
        assert_eq!(rights(4, 0x0002).forbidden, Some(EmbeddingRestriction::Restricted));
        assert_eq!(rights(1, 0x0002).forbidden, Some(EmbeddingRestriction::Restricted));
        // Old fonts: the most permissive bit wins. New fonts: stay conservative.
        assert_eq!(rights(2, 0x0006).forbidden, None);
        assert_eq!(rights(3, 0x0006).forbidden, Some(EmbeddingRestriction::Restricted));
        // No-subsetting and bitmap-only exist from version 2.
        assert_eq!(rights(4, 0x0100), EmbeddingRights { forbidden: None, no_subsetting: true });
        assert_eq!(rights(1, 0x0100), free);
        assert_eq!(rights(4, 0x0200).forbidden, Some(EmbeddingRestriction::BitmapOnly));
        assert_eq!(rights(4, 0x0204).forbidden, Some(EmbeddingRestriction::BitmapOnly));
        assert_eq!(rights(1, 0x0200).forbidden, None);
        assert_eq!(
            rights(4, 0x0108),
            EmbeddingRights { forbidden: None, no_subsetting: true },
            "editable, whole font only"
        );
    }

    #[test]
    fn usage_keeps_the_first_text_and_truncates_absurd_ones() {
        let mut usage = FontUsage::default();
        usage.record(5, "");
        usage.record(5, "fi");
        usage.record(5, "x");
        assert_eq!(usage.glyphs[&5], "fi");
        usage.record(6, &"é".repeat(1000));
        assert_eq!(usage.glyphs[&6].chars().count(), MAX_TO_UNICODE_UNITS);
        usage.record(7, &"🎁".repeat(1000));
        assert_eq!(usage.glyphs[&7].chars().count(), MAX_TO_UNICODE_UNITS / 2);
    }

    #[test]
    fn subset_is_much_smaller_and_maps_glyphs_to_dense_cids() {
        let data = font_bytes("Spectral-Regular.ttf");
        let face = Face::parse(&data, 0).expect("valid font");
        let mut usage = FontUsage::default();
        let mut glyphs = Vec::new();
        for c in "Étranger".chars() {
            let glyph = face.glyph_index(c).expect("glyph").0;
            usage.record(glyph, &c.to_string());
            glyphs.push(glyph);
        }
        let resource = FontResource { id: FontId(7), data: Arc::from(data.clone()), face_index: 0 };
        let font = EmbeddedFont::prepare(&resource, &usage, 3).expect("embeddable");

        assert!(!font.is_whole());
        assert!(font.program.len() < data.len() / 10, "{} bytes", font.program.len());
        assert_eq!(font.resource_name, b"F3");
        let (tag, name) = font.base_name().split_once('+').expect("subset tag");
        assert_eq!(tag.len(), 6);
        assert!(tag.bytes().all(|b| b.is_ascii_uppercase()));
        assert_eq!(name, "Spectral-Regular");

        // CIDs are 1..=n in glyph-id order; unused glyphs have none.
        let mut sorted = glyphs.clone();
        sorted.sort_unstable();
        sorted.dedup();
        for (i, glyph) in sorted.iter().enumerate() {
            assert_eq!(font.cid(*glyph), Some(u16::try_from(i + 1).expect("small")));
        }
        assert_eq!(font.cid(0), Some(0));
        let unused = face.glyph_index('Z').expect("glyph").0;
        assert_eq!(font.cid(unused), None);

        // Widths are the font's advances in thousandths of an em.
        let upm = f32::from(face.units_per_em());
        for glyph in &sorted {
            let cid = font.cid(*glyph).expect("mapped");
            let advance = f32::from(face.glyph_hor_advance(GlyphId(*glyph)).expect("advance"));
            assert!((font.width(cid) - advance * 1000.0 / upm).abs() < 1e-3);
        }
        assert_eq!(font.to_unicode.len(), sorted.len());

        // The subset is itself a parseable font with exactly the kept glyphs (+ .notdef).
        let subset = Face::parse(&font.program, 0).expect("subset parses");
        assert!(usize::from(subset.number_of_glyphs()) > sorted.len());
        assert!(usize::from(subset.number_of_glyphs()) <= sorted.len() + 8);
    }

    #[test]
    fn subset_tags_differ_between_glyph_sets_and_are_stable() {
        let data = font_bytes("AtkinsonHyperlegible-Regular.ttf");
        let mut a = FontUsage::default();
        a.record(10, "a");
        let mut b = FontUsage::default();
        b.record(11, "b");
        assert_eq!(subset_tag(&data, 0, 0, &a), subset_tag(&data, 0, 0, &a));
        assert_ne!(subset_tag(&data, 0, 0, &a), subset_tag(&data, 0, 0, &b));
        assert_ne!(subset_tag(&data, 0, 0, &a), subset_tag(&data, 0, 1, &a));
    }

    #[test]
    fn invalid_glyph_ids_and_garbage_fonts_are_errors() {
        let data = font_bytes("CourierPrime-Bold.ttf");
        let count = Face::parse(&data, 0).expect("valid").number_of_glyphs();
        let resource = FontResource { id: FontId(1), data: Arc::from(data), face_index: 0 };
        let mut usage = FontUsage::default();
        usage.record(count, "x");
        assert_eq!(
            EmbeddedFont::prepare(&resource, &usage, 0).expect_err("glyph id out of range"),
            PdfError::InvalidGlyph { font: FontId(1), glyph: count, glyph_count: count }
        );

        let mut usage = FontUsage::default();
        usage.record(1, "x");
        for garbage in [&b""[..], b"not a font", &[0u8; 64], b"OTTO\0\x01"] {
            let resource =
                FontResource { id: FontId(2), data: Arc::from(garbage.to_vec()), face_index: 0 };
            assert!(matches!(
                EmbeddedFont::prepare(&resource, &usage, 0),
                Err(PdfError::InvalidFont { font: FontId(2), .. })
            ));
        }
        // A face index that does not exist.
        let resource = FontResource {
            id: FontId(3),
            data: Arc::from(font_bytes("CourierPrime-Bold.ttf")),
            face_index: 5,
        };
        assert!(matches!(
            EmbeddedFont::prepare(&resource, &usage, 0),
            Err(PdfError::InvalidFont { .. })
        ));
    }

    #[test]
    fn name_sanitising() {
        assert_eq!(sanitize_name("Spectral Semi/Bold (v2) #1"), "SpectralSemiBoldv21");
        assert_eq!(sanitize_name("Été-Regular_1.0"), "t-Regular_1.0");
        assert_eq!(sanitize_name(&"x".repeat(200)).len(), 63);
    }
}
