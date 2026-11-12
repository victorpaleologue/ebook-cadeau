//! Font loading: names, licence, embedding permission, metrics and variable-font instances.

use std::fmt;
use std::sync::Arc;

use ebc_scene::MM_PER_PT;
use ttf_parser::{Tag, name_id};

use crate::error::FontError;
use crate::guard::contain;

/// Millimetres per font unit at `size_pt` points — the one conversion every measurement
/// in this crate goes through, so that metrics, shaping and fitting agree to the last bit.
pub(crate) fn mm_per_unit(units_per_em: u16, size_pt: f64) -> f64 {
    size_pt * MM_PER_PT / f64::from(units_per_em.max(1))
}

/// What the font vendor allows regarding embedding the font in documents, decoded from
/// the usage-permission bits (0–3) of the `OS/2` table's `fsType` field.
///
/// The two independent modifier bits (*no subsetting*, *bitmap embedding only*) are
/// reported separately as [`EmbeddingFlags`]. The product rules built on both are
/// [`Font::allows_bundling`], [`Font::allows_subset_embedding`] and
/// [`Font::allows_outlines`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddingPermission {
    /// `fsType & 0x000F == 0`: the font may be embedded and permanently installed.
    Installable,
    /// Bit `0x0004`: the font may be embedded for viewing and printing the document only.
    /// This is enough for a print-ready PDF.
    PreviewAndPrint,
    /// Bit `0x0008`: the font may be embedded and the document may be edited with it.
    Editable,
    /// Bit `0x0002`: the font must not be embedded, subset or otherwise shared without
    /// the explicit permission of its owner. Draw nothing with it in a PDF.
    Restricted,
}

impl EmbeddingPermission {
    /// `fsType` bit: restricted-licence embedding.
    pub const FS_RESTRICTED: u16 = 0x0002;
    /// `fsType` bit: preview & print embedding.
    pub const FS_PREVIEW_AND_PRINT: u16 = 0x0004;
    /// `fsType` bit: editable embedding.
    pub const FS_EDITABLE: u16 = 0x0008;
    /// `fsType` bit: the font may not be subset before embedding.
    pub const FS_NO_SUBSETTING: u16 = 0x0100;
    /// `fsType` bit: only the bitmaps of the font may be embedded.
    pub const FS_BITMAP_ONLY: u16 = 0x0200;

    /// Decodes the usage permission of an `fsType` value with the rule of `OS/2`
    /// versions 0–2: when several permission bits are set, the **least restrictive** one
    /// wins (*editable* over *preview & print* over *restricted*).
    ///
    /// ```
    /// use ebc_text::EmbeddingPermission as P;
    /// assert_eq!(P::from_fs_type(0x0000), P::Installable);
    /// assert_eq!(P::from_fs_type(0x0002), P::Restricted);
    /// assert_eq!(P::from_fs_type(0x0004), P::PreviewAndPrint);
    /// assert_eq!(P::from_fs_type(0x0008), P::Editable);
    /// assert_eq!(P::from_fs_type(0x0104), P::PreviewAndPrint); // + "no subsetting" flag
    /// ```
    #[must_use]
    pub const fn from_fs_type(fs_type: u16) -> Self {
        if fs_type & Self::FS_EDITABLE != 0 {
            Self::Editable
        } else if fs_type & Self::FS_PREVIEW_AND_PRINT != 0 {
            Self::PreviewAndPrint
        } else if fs_type & Self::FS_RESTRICTED != 0 {
            Self::Restricted
        } else {
            // Bit 0 is reserved and carries no restriction.
            Self::Installable
        }
    }

    /// Decodes the usage permission of an `fsType` value for a given `OS/2` table version.
    ///
    /// From version 3 on, the permission bits are mutually exclusive. A font that sets
    /// several of them anyway is malformed; this function then takes the **most
    /// restrictive** bit (the legally conservative reading). Versions 0–2 use
    /// [`from_fs_type`](Self::from_fs_type).
    #[must_use]
    pub const fn from_os2(fs_type: u16, os2_version: u16) -> Self {
        if os2_version <= 2 {
            return Self::from_fs_type(fs_type);
        }
        if fs_type & Self::FS_RESTRICTED != 0 {
            Self::Restricted
        } else if fs_type & Self::FS_PREVIEW_AND_PRINT != 0 {
            Self::PreviewAndPrint
        } else if fs_type & Self::FS_EDITABLE != 0 {
            Self::Editable
        } else {
            Self::Installable
        }
    }

    /// `true` unless the permission is [`Restricted`](Self::Restricted): the font may be
    /// embedded in a document that is going to be viewed and printed.
    #[must_use]
    pub const fn allows_embedding(self) -> bool {
        !matches!(self, Self::Restricted)
    }
}

/// The modifier bits of `fsType` that come on top of the [`EmbeddingPermission`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EmbeddingFlags {
    /// Bit `0x0100`: the font must be embedded whole, never as a subset.
    pub no_subsetting: bool,
    /// Bit `0x0200`: only bitmaps contained in the font may be embedded; its outlines
    /// may not. For an outline-only font this means "do not embed".
    pub bitmap_only: bool,
}

impl EmbeddingFlags {
    /// Decodes the modifier bits of an `fsType` value.
    #[must_use]
    pub const fn from_fs_type(fs_type: u16) -> Self {
        Self {
            no_subsetting: fs_type & EmbeddingPermission::FS_NO_SUBSETTING != 0,
            bitmap_only: fs_type & EmbeddingPermission::FS_BITMAP_ONLY != 0,
        }
    }
}

/// Global metrics of a font (of the selected instance, for variable fonts), in font units
/// with **y pointing up** as in the font file: `ascender` is positive, `descender` negative.
///
/// The `*_mm` helpers are plain unit conversions for a given font size in points and keep
/// the sign of the field (`descender_mm` is negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetrics {
    /// Font units per em (16..=16384).
    pub units_per_em: u16,
    /// Distance from the baseline to the top of the line box (positive). This is a
    /// line-spacing value, not a bound on the ink: accented capitals can rise above it.
    pub ascender: i16,
    /// Distance from the baseline to the bottom of the line box (negative).
    pub descender: i16,
    /// Extra space the font recommends between two lines.
    pub line_gap: i16,
    /// Height of flat capital letters such as "H". Taken from `OS/2.sCapHeight`, else
    /// measured on the outline of "H", "I", "X" or "E", else assumed to be 0.7 em.
    pub cap_height: i16,
    /// Height of flat lower-case letters such as "x". Taken from `OS/2.sxHeight`, else
    /// measured on the outline of "x", "z" or "v", else assumed to be 0.5 em.
    pub x_height: i16,
}

impl FontMetrics {
    /// Converts a length in font units to millimetres at `size_pt`.
    #[must_use]
    pub fn units_to_mm(&self, units: f64, size_pt: f64) -> f64 {
        units * self.mm_per_unit(size_pt)
    }

    /// Millimetres per font unit at `size_pt`.
    #[must_use]
    pub fn mm_per_unit(&self, size_pt: f64) -> f64 {
        mm_per_unit(self.units_per_em, size_pt)
    }

    /// Ascender in millimetres at `size_pt` (positive: above the baseline).
    #[must_use]
    pub fn ascender_mm(&self, size_pt: f64) -> f64 {
        self.units_to_mm(f64::from(self.ascender), size_pt)
    }

    /// Descender in millimetres at `size_pt` (**negative**: below the baseline).
    #[must_use]
    pub fn descender_mm(&self, size_pt: f64) -> f64 {
        self.units_to_mm(f64::from(self.descender), size_pt)
    }

    /// Recommended extra line spacing in millimetres at `size_pt`.
    #[must_use]
    pub fn line_gap_mm(&self, size_pt: f64) -> f64 {
        self.units_to_mm(f64::from(self.line_gap), size_pt)
    }

    /// Capital height in millimetres at `size_pt`.
    #[must_use]
    pub fn cap_height_mm(&self, size_pt: f64) -> f64 {
        self.units_to_mm(f64::from(self.cap_height), size_pt)
    }

    /// x-height in millimetres at `size_pt`.
    #[must_use]
    pub fn x_height_mm(&self, size_pt: f64) -> f64 {
        self.units_to_mm(f64::from(self.x_height), size_pt)
    }

    /// Default baseline-to-baseline distance in millimetres at `size_pt`:
    /// `ascender - descender + line_gap`.
    #[must_use]
    pub fn line_height_mm(&self, size_pt: f64) -> f64 {
        let units = f64::from(self.ascender) - f64::from(self.descender) + f64::from(self.line_gap);
        self.units_to_mm(units, size_pt)
    }

    /// The font size in points at which capital letters are `cap_height_mm` millimetres
    /// tall. Returns `0.0` if the font reports no capital height.
    #[must_use]
    pub fn size_for_cap_height(&self, cap_height_mm: f64) -> f64 {
        let per_pt = self.cap_height_mm(1.0);
        if per_pt > 0.0 { cap_height_mm / per_pt } else { 0.0 }
    }
}

/// One design axis of a variable font (`fvar` table).
#[derive(Debug, Clone, PartialEq)]
pub struct VariationAxis {
    /// Four-character axis tag, e.g. `"wght"`, `"wdth"`, `"opsz"`.
    pub tag: String,
    /// Human-readable axis name from the `name` table, if any.
    pub name: Option<String>,
    /// Smallest value of the axis.
    pub min: f32,
    /// Value of the default instance.
    pub default: f32,
    /// Largest value of the axis.
    pub max: f32,
    /// The font asks user interfaces not to show this axis.
    pub hidden: bool,
}

/// The kind of glyph outlines a font contains (decides how a PDF writer embeds it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutlineFormat {
    /// Quadratic TrueType outlines (`glyf` table).
    TrueType,
    /// Cubic PostScript outlines (`CFF ` table).
    Cff,
    /// Cubic variable PostScript outlines (`CFF2` table).
    Cff2,
    /// No scalable outlines (bitmap, SVG or colour-only font).
    None,
}

/// Facts about the font file that do not depend on the selected variation instance.
#[derive(Debug)]
struct StaticInfo {
    family: String,
    style: String,
    full: String,
    postscript: Option<String>,
    licence_description: Option<String>,
    licence_url: Option<String>,
    /// `(fsType, OS/2 version)`, when the font has an `OS/2` table.
    os2: Option<(u16, u16)>,
    axes: Vec<(Tag, VariationAxis)>,
    glyph_count: u16,
    weight: u16,
    italic: bool,
    monospaced: bool,
    outlines: OutlineFormat,
}

#[derive(Debug)]
struct Inner {
    data: Arc<[u8]>,
    face_index: u32,
    info: Arc<StaticInfo>,
    /// Axis settings that differ from the default instance, in user-space units,
    /// clamped to the axis range and sorted by tag.
    variations: Vec<(Tag, f32)>,
    metrics: FontMetrics,
}

/// A parsed font face. Cloning is cheap (reference counted); the type is `Send + Sync`.
///
/// The font data is validated once by [`Font::from_bytes`]; afterwards no method
/// returns an error. Fonts are untrusted input: tables that turn out to be malformed
/// later simply contribute nothing (no glyph, no kerning, …), and a panic inside the
/// font libraries is contained wherever the platform can unwind (see the crate docs).
///
/// ```
/// # fn demo(bytes: Vec<u8>) -> Result<(), ebc_text::FontError> {
/// let font = ebc_text::Font::from_bytes(bytes, 0)?;
/// if !font.embedding().allows_embedding() {
///     // fall back to a bundled OFL font
/// }
/// let cap_mm = font.metrics().cap_height_mm(14.0);
/// # let _ = cap_mm; Ok(()) }
/// ```
#[derive(Clone)]
pub struct Font {
    inner: Arc<Inner>,
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Font")
            .field("family", &self.inner.info.family)
            .field("style", &self.inner.info.style)
            .field("face_index", &self.inner.face_index)
            .field("variations", &self.variations())
            .field("bytes", &self.inner.data.len())
            .finish()
    }
}

impl PartialEq for Font {
    /// Two fonts are equal when they refer to the same face of the same font data with
    /// the same variation settings.
    fn eq(&self, other: &Self) -> bool {
        self.inner.face_index == other.inner.face_index
            && self.inner.variations == other.inner.variations
            && (Arc::ptr_eq(&self.inner.data, &other.inner.data)
                || self.inner.data == other.inner.data)
    }
}

impl Font {
    /// Parses a TrueType/OpenType font (`.ttf`, `.otf`) or one face of a collection
    /// (`.ttc`, `.otc`). `face_index` is 0 for plain font files.
    ///
    /// WOFF/WOFF2 containers are not supported — decompress them first.
    ///
    /// # Errors
    /// [`FontError::FaceIndex`] if the collection has no such face, [`FontError::Parse`]
    /// for anything that is not a usable font. Malformed data is an error, not a panic
    /// (see "Untrusted fonts" in the crate documentation).
    pub fn from_bytes(data: impl Into<Arc<[u8]>>, face_index: u32) -> Result<Self, FontError> {
        let data: Arc<[u8]> = data.into();
        check_collection_header(&data, face_index)?;
        let (info, metrics) = contain(|| {
            let face = ttf_parser::Face::parse(&data, face_index).map_err(|e| match e {
                ttf_parser::FaceParsingError::FaceIndexOutOfBounds => FontError::FaceIndex {
                    index: face_index,
                    count: ttf_parser::fonts_in_collection(&data).unwrap_or(1),
                },
                other => FontError::Parse(other.to_string()),
            })?;
            Ok((static_info(&face), metrics_of(&face)))
        })
        .unwrap_or_else(|| Err(FontError::Parse("the font parser gave up on this data".into())))?;
        Ok(Self {
            inner: Arc::new(Inner {
                data,
                face_index,
                info: Arc::new(info),
                variations: Vec::new(),
                metrics,
            }),
        })
    }

    /// The complete font file this face was loaded from.
    #[must_use]
    pub fn data(&self) -> &Arc<[u8]> {
        &self.inner.data
    }

    /// Index of the face inside a font collection (0 for plain font files).
    #[must_use]
    pub fn face_index(&self) -> u32 {
        self.inner.face_index
    }

    /// The family name, e.g. `"Spectral"`: the typographic family (name ID 16) when the
    /// font has one, else the legacy family (name ID 1). Empty if the font has no names.
    #[must_use]
    pub fn family_name(&self) -> String {
        self.inner.info.family.clone()
    }

    /// The style within the family, e.g. `"SemiBold"` or `"Italic"` (name ID 17, else 2).
    /// Empty if the font has no names.
    #[must_use]
    pub fn style_name(&self) -> String {
        self.inner.info.style.clone()
    }

    /// The full name, e.g. `"Spectral SemiBold"` (name ID 4, else family + style).
    #[must_use]
    pub fn full_name(&self) -> String {
        self.inner.info.full.clone()
    }

    /// The PostScript name, e.g. `"Spectral-SemiBold"` (name ID 6).
    #[must_use]
    pub fn postscript_name(&self) -> Option<String> {
        self.inner.info.postscript.clone()
    }

    /// The licence description stored in the font (name ID 13).
    #[must_use]
    pub fn licence_description(&self) -> Option<String> {
        self.inner.info.licence_description.clone()
    }

    /// The URL of the full licence text stored in the font (name ID 14).
    #[must_use]
    pub fn licence_url(&self) -> Option<String> {
        self.inner.info.licence_url.clone()
    }

    /// The version string stored in the font, e.g. `"Version 2.001"` (name ID 5).
    #[must_use]
    pub fn version(&self) -> Option<String> {
        self.name(name_id::VERSION)
    }

    /// Any string of the `name` table by its name ID (0 = copyright, 5 = version,
    /// 9 = designer, …), preferring an English, Unicode-encoded record.
    #[must_use]
    pub fn name(&self, name_id: u16) -> Option<String> {
        contain(|| best_name(&self.tt_face()?, name_id)).flatten()
    }

    /// The raw `OS/2.fsType` value, or `None` when the font has no `OS/2` table.
    #[must_use]
    pub fn fs_type(&self) -> Option<u16> {
        self.inner.info.os2.map(|(fs_type, _)| fs_type)
    }

    /// The embedding permission from `OS/2.fsType` (see
    /// [`EmbeddingPermission::from_os2`]).
    ///
    /// A font **without an `OS/2` table** states nothing about embedding. Its
    /// permission is unknown, and — the project's rule being to refuse what it cannot
    /// verify (`docs/research/08-legal-fonts-and-genai.md` §4.3) — it is reported as
    /// [`EmbeddingPermission::Restricted`]. [`fs_type`](Self::fs_type) returns `None`
    /// in that case, for callers that need to tell "forbidden" from "unknown".
    #[must_use]
    pub fn embedding(&self) -> EmbeddingPermission {
        match self.inner.info.os2 {
            Some((fs_type, version)) => EmbeddingPermission::from_os2(fs_type, version),
            None => EmbeddingPermission::Restricted,
        }
    }

    /// The *no subsetting* and *bitmap only* modifier bits of `OS/2.fsType` (both
    /// `false` when the font has no `OS/2` table).
    #[must_use]
    pub fn embedding_flags(&self) -> EmbeddingFlags {
        EmbeddingFlags::from_fs_type(self.fs_type().unwrap_or(0))
    }

    /// The rule for fonts **shipped with the product or listed in its font index**:
    /// `fsType` must be exactly 0 (installable embedding, no modifier bit).
    #[must_use]
    pub fn allows_bundling(&self) -> bool {
        self.fs_type() == Some(0)
    }

    /// `true` when a subset of this font may be embedded in a PDF meant for printing:
    /// the font has an `OS/2` table, the permission is not *restricted*, and neither
    /// the *no subsetting* nor the *bitmap only* bit is set.
    #[must_use]
    pub fn allows_subset_embedding(&self) -> bool {
        let flags = self.embedding_flags();
        self.embedding().allows_embedding() && !flags.no_subsetting && !flags.bitmap_only
    }

    /// The rule for a **font supplied by the user**: `true` when its text may be drawn
    /// as vector outlines ([`ShapedText::to_outline`]) — the font has an `OS/2` table
    /// and sets neither the *restricted* nor the *bitmap only* bit. (*No subsetting*
    /// does not matter here: outlines embed no font program at all.)
    ///
    /// This reads the font's own flags only. `fsType` states the foundry's intent; the
    /// licence text prevails, and outlining does not cure a missing licence.
    ///
    /// [`ShapedText::to_outline`]: crate::ShapedText::to_outline
    #[must_use]
    pub fn allows_outlines(&self) -> bool {
        self.embedding().allows_embedding() && !self.embedding_flags().bitmap_only
    }

    /// Global metrics of this face (of the selected instance for variable fonts).
    #[must_use]
    pub fn metrics(&self) -> FontMetrics {
        self.inner.metrics
    }

    /// Number of glyphs in the font.
    #[must_use]
    pub fn glyph_count(&self) -> u16 {
        self.inner.info.glyph_count
    }

    /// The kind of outlines the font contains.
    #[must_use]
    pub fn outline_format(&self) -> OutlineFormat {
        self.inner.info.outlines
    }

    /// The weight class from `OS/2.usWeightClass` (400 = regular, 700 = bold; 400 when
    /// the font has no `OS/2` table).
    #[must_use]
    pub fn weight(&self) -> u16 {
        self.inner.info.weight
    }

    /// `true` for italic or oblique faces.
    #[must_use]
    pub fn is_italic(&self) -> bool {
        self.inner.info.italic
    }

    /// `true` when the font declares itself monospaced.
    #[must_use]
    pub fn is_monospaced(&self) -> bool {
        self.inner.info.monospaced
    }

    /// `true` when the font has variation axes (`fvar` table).
    #[must_use]
    pub fn is_variable(&self) -> bool {
        !self.inner.info.axes.is_empty()
    }

    /// The variation axes of the font; empty for static fonts.
    #[must_use]
    pub fn axes(&self) -> Vec<VariationAxis> {
        self.inner.info.axes.iter().map(|(_, axis)| axis.clone()).collect()
    }

    /// The position of this instance on every axis, as `(tag, value)` in axis order.
    /// Axes that were not set explicitly report their default. Empty for static fonts.
    #[must_use]
    pub fn variations(&self) -> Vec<(String, f32)> {
        self.inner
            .info
            .axes
            .iter()
            .map(|(tag, axis)| {
                let value = self
                    .inner
                    .variations
                    .iter()
                    .find(|(t, _)| t == tag)
                    .map_or(axis.default, |(_, v)| *v);
                (axis.tag.clone(), value)
            })
            .collect()
    }

    /// Returns the same font at another position of its design space, e.g.
    /// `font.with_variations(&[("wght", 700.0)])`. Measuring, shaping, outlining and
    /// rasterising all use the selected instance.
    ///
    /// Values are clamped to the axis range; unknown axes, malformed tags and
    /// non-finite values are ignored; settings accumulate over successive calls. On a
    /// static font this returns an unchanged clone.
    ///
    /// Note that [`ShapedText::to_glyph_item`] refers to the font *file*: a PDF viewer
    /// would draw the default instance. Use [`ShapedText::to_outline`] for instances.
    ///
    /// [`ShapedText::to_glyph_item`]: crate::ShapedText::to_glyph_item
    /// [`ShapedText::to_outline`]: crate::ShapedText::to_outline
    #[must_use]
    pub fn with_variations(&self, axes: &[(&str, f32)]) -> Self {
        if !self.is_variable() {
            return self.clone();
        }
        let mut variations = self.inner.variations.clone();
        for &(tag, value) in axes {
            let Some(tag) = parse_tag(tag) else { continue };
            let Some((_, axis)) = self.inner.info.axes.iter().find(|(t, _)| *t == tag) else {
                continue;
            };
            if !value.is_finite() {
                continue;
            }
            let value = value.clamp(axis.min, axis.max);
            // Only non-default positions are stored, so that equal instances compare equal.
            variations.retain(|(t, _)| *t != tag);
            if value != axis.default {
                variations.push((tag, value));
            }
        }
        variations.sort_by_key(|(tag, _)| tag.0);
        if variations == self.inner.variations {
            return self.clone();
        }
        let metrics = contain(|| {
            parse_face(&self.inner.data, self.inner.face_index, &variations)
                .map(|face| metrics_of(&face))
        })
        .flatten()
        .unwrap_or(self.inner.metrics);
        Self {
            inner: Arc::new(Inner {
                data: Arc::clone(&self.inner.data),
                face_index: self.inner.face_index,
                info: Arc::clone(&self.inner.info),
                variations,
                metrics,
            }),
        }
    }

    /// The glyph the font maps `c` to, or `None` if the character is not covered.
    #[must_use]
    pub fn glyph_id(&self, c: char) -> Option<u16> {
        contain(|| {
            let face = self.tt_face()?;
            face.glyph_index(c).map(|g| g.0).filter(|&g| g != 0)
        })
        .flatten()
    }

    /// The characters of `text` that this font cannot draw, deduplicated, in order of
    /// first appearance. Shaping such text yields `.notdef` boxes ("tofu"), so callers
    /// should pick another font when this is not empty.
    ///
    /// A character counts as drawable when the font maps it to a glyph, or when the
    /// shaper can stand in for it (a canonical decomposition such as `é` → `e` + `´`,
    /// a non-breaking variant of a space or hyphen). Control characters and invisible
    /// format characters are never reported.
    #[must_use]
    pub fn missing_chars(&self, text: &str) -> Vec<char> {
        let mut seen = std::collections::HashSet::new();
        let distinct: Vec<char> = text.chars().filter(|&c| seen.insert(c)).collect();
        contain(|| {
            let face = self.hb_face()?;
            Some(
                distinct
                    .iter()
                    .copied()
                    .filter(|&c| !crate::shape::is_drawable(&face, c))
                    .collect::<Vec<char>>(),
            )
        })
        .flatten()
        // A font the shaper cannot work with draws nothing at all.
        .unwrap_or_else(|| {
            distinct.into_iter().filter(|&c| !crate::shape::is_set_as_space(c)).collect()
        })
    }

    /// Parses the face with the variation settings applied. Cheap (table directory only).
    pub(crate) fn tt_face(&self) -> Option<ttf_parser::Face<'_>> {
        parse_face(&self.inner.data, self.inner.face_index, &self.inner.variations)
    }

    /// Parses the face for shaping, with the variation settings applied.
    pub(crate) fn hb_face(&self) -> Option<rustybuzz::Face<'_>> {
        // `from_face` takes the `ttf_parser::Face` of the crate version rustybuzz was
        // built with; this line stops compiling if the two versions ever diverge.
        Some(rustybuzz::Face::from_face(self.tt_face()?))
    }
}

/// Rejects font collections whose header promises more faces than the file can hold,
/// before the parser sees them (it would compute the size of the offset table first).
fn check_collection_header(data: &[u8], face_index: u32) -> Result<(), FontError> {
    if data.get(..4) != Some(b"ttcf".as_slice()) {
        return Ok(());
    }
    let malformed = || FontError::Parse("malformed font collection header".into());
    let count_bytes: [u8; 4] =
        data.get(8..12).and_then(|b| b.try_into().ok()).ok_or_else(malformed)?;
    let count = u32::from_be_bytes(count_bytes);
    let table_end = u64::from(count) * 4 + 12;
    if table_end > data.len() as u64 {
        return Err(malformed());
    }
    if face_index >= count {
        return Err(FontError::FaceIndex { index: face_index, count });
    }
    Ok(())
}

fn parse_face<'a>(
    data: &'a [u8],
    face_index: u32,
    variations: &[(Tag, f32)],
) -> Option<ttf_parser::Face<'a>> {
    let mut face = ttf_parser::Face::parse(data, face_index).ok()?;
    for &(tag, value) in variations {
        let _ = face.set_variation(tag, value);
    }
    Some(face)
}

/// Parses a 1–4 character ASCII tag, padding short tags with spaces.
fn parse_tag(tag: &str) -> Option<Tag> {
    let bytes = tag.as_bytes();
    if bytes.is_empty() || bytes.len() > 4 || !bytes.iter().all(|b| (0x20..0x7F).contains(b)) {
        return None;
    }
    Some(Tag::from_bytes_lossy(bytes))
}

fn tag_to_string(tag: Tag) -> String {
    tag.to_bytes()
        .iter()
        .map(|&b| if (0x20..0x7F).contains(&b) { b as char } else { '?' })
        .collect()
}

fn static_info(face: &ttf_parser::Face<'_>) -> StaticInfo {
    let name = |id: u16| best_name(face, id);
    let legacy_family = name(name_id::FAMILY);
    let legacy_style = name(name_id::SUBFAMILY);
    let family =
        name(name_id::TYPOGRAPHIC_FAMILY).or_else(|| legacy_family.clone()).unwrap_or_default();
    let style =
        name(name_id::TYPOGRAPHIC_SUBFAMILY).or_else(|| legacy_style.clone()).unwrap_or_default();
    let full = name(name_id::FULL_NAME).unwrap_or_else(|| {
        let family = legacy_family.as_deref().unwrap_or(&family);
        let style = legacy_style.as_deref().unwrap_or(&style);
        if style.is_empty() || style.eq_ignore_ascii_case("regular") {
            family.to_owned()
        } else {
            format!("{family} {style}").trim().to_owned()
        }
    });

    // Read fsType from the raw table so that a restriction is honoured even when the
    // rest of the table is too short for its declared version. The directory is
    // scanned linearly (it may be unsorted); like ttf-parser, the last record wins.
    let raw = face.raw_face();
    let os2 = raw
        .table_records
        .into_iter()
        .filter(|record| record.tag == Tag::from_bytes(b"OS/2"))
        .last()
        .and_then(|record| {
            let start = usize::try_from(record.offset).ok()?;
            let end = start.checked_add(usize::try_from(record.length).ok()?)?;
            let table = raw.data.get(start..end)?;
            let version = u16::from_be_bytes([*table.first()?, *table.get(1)?]);
            let fs_type = u16::from_be_bytes([*table.get(8)?, *table.get(9)?]);
            Some((fs_type, version))
        });

    let axes = face
        .variation_axes()
        .into_iter()
        .map(|a| {
            let axis = VariationAxis {
                tag: tag_to_string(a.tag),
                name: best_name(face, a.name_id),
                min: a.min_value,
                default: a.def_value,
                max: a.max_value,
                hidden: a.hidden,
            };
            (a.tag, axis)
        })
        .collect();

    let tables = face.tables();
    let outlines = if tables.glyf.is_some() {
        OutlineFormat::TrueType
    } else if tables.cff.is_some() {
        OutlineFormat::Cff
    } else if tables.cff2.is_some() {
        OutlineFormat::Cff2
    } else {
        OutlineFormat::None
    };

    StaticInfo {
        family,
        style,
        full,
        postscript: name(name_id::POST_SCRIPT_NAME),
        licence_description: name(name_id::LICENSE),
        licence_url: name(name_id::LICENSE_URL),
        os2,
        axes,
        glyph_count: face.number_of_glyphs(),
        weight: face.weight().to_number(),
        italic: face.is_italic() || face.is_oblique(),
        monospaced: face.is_monospaced(),
        outlines,
    }
}

fn metrics_of(face: &ttf_parser::Face<'_>) -> FontMetrics {
    let units_per_em = face.units_per_em();
    let em = f64::from(units_per_em);
    let fraction_of_em = |f: f64| -> i16 {
        // units_per_em <= 16384, so the product always fits an i16.
        i16::try_from((em * f).round() as i64).unwrap_or(i16::MAX)
    };
    let top_of = |chars: &[char]| -> Option<i16> {
        chars
            .iter()
            .filter_map(|&c| face.glyph_index(c))
            .filter_map(|g| face.glyph_bounding_box(g))
            .map(|bbox| bbox.y_max)
            .find(|&top| top > 0)
    };

    let mut ascender = face.ascender();
    let mut descender = face.descender();
    if ascender == 0 && descender == 0 {
        // No usable vertical metrics at all: use the font bounding box, else 0.8/-0.2 em.
        let bbox = face.global_bounding_box();
        if bbox.y_max > 0 && bbox.y_min <= 0 {
            ascender = bbox.y_max;
            descender = bbox.y_min;
        } else {
            ascender = fraction_of_em(0.8);
            descender = -fraction_of_em(0.2);
        }
    }

    let cap_height = face
        .capital_height()
        .filter(|&v| v > 0)
        .or_else(|| top_of(&['H', 'I', 'X', 'E']))
        .unwrap_or_else(|| fraction_of_em(0.7));
    let x_height = face
        .x_height()
        .filter(|&v| v > 0)
        .or_else(|| top_of(&['x', 'z', 'v']))
        .unwrap_or_else(|| fraction_of_em(0.5));

    FontMetrics {
        units_per_em,
        ascender,
        descender,
        line_gap: face.line_gap(),
        cap_height,
        x_height,
    }
}

/// Picks the best record for `id` in the `name` table and decodes it.
///
/// Preference: Windows/Unicode US English, any Unicode record in English, any Unicode
/// record, Macintosh Roman English, any Macintosh Roman record.
fn best_name(face: &ttf_parser::Face<'_>, id: u16) -> Option<String> {
    use ttf_parser::PlatformId;
    const WINDOWS_EN_US: u16 = 0x0409;

    let names = face.names();
    let mut best: Option<(u8, String)> = None;
    // Indexed access: the iterator of `Names` stops at the first malformed record.
    for record in (0..names.len()).filter_map(|i| names.get(i)) {
        if record.name_id != id {
            continue;
        }
        let rank = match record.platform_id {
            PlatformId::Windows if record.is_unicode() => {
                if record.language_id == WINDOWS_EN_US {
                    0
                } else if record.language_id & 0x03FF == 0x0009 {
                    1 // another English locale
                } else {
                    3
                }
            }
            PlatformId::Unicode => 2,
            PlatformId::Macintosh if record.encoding_id == 0 => {
                if record.language_id == 0 {
                    4
                } else {
                    5
                }
            }
            _ => continue,
        };
        if best.as_ref().is_some_and(|(r, _)| *r <= rank) {
            continue;
        }
        let decoded = if rank >= 4 {
            Some(decode_mac_roman(record.name))
        } else {
            decode_utf16_be(record.name)
        };
        if let Some(text) = decoded.map(|s| clean_name(&s)).filter(|s| !s.is_empty()) {
            best = Some((rank, text));
        }
    }
    best.map(|(_, text)| text)
}

fn clean_name(raw: &str) -> String {
    raw.chars().filter(|&c| c != '\0').collect::<String>().trim().to_owned()
}

fn decode_utf16_be(bytes: &[u8]) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units = bytes.chunks_exact(2).map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
    Some(char::decode_utf16(units).map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER)).collect())
}

/// Mac OS Roman, upper half (0x80..=0xFF).
const MAC_ROMAN_HIGH: [char; 128] = [
    'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è', // 0x80
    'ê', 'ë', 'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü', // 0x90
    '†', '°', '¢', '£', '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø', // 0xA0
    '∞', '±', '≤', '≥', '¥', 'µ', '∂', '∑', '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø', // 0xB0
    '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«', '»', '…', '\u{a0}', 'À', 'Ã', 'Õ', 'Œ',
    'œ', // 0xC0
    '–', '—', '“', '”', '‘', '’', '÷', '◊', 'ÿ', 'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ', // 0xD0
    '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í', 'Î', 'Ï', 'Ì', 'Ó', 'Ô', // 0xE0
    '\u{f8ff}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚', '¸', '˝', '˛',
    'ˇ', // 0xF0
];

fn decode_mac_roman(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| if b < 0x80 { b as char } else { MAC_ROMAN_HIGH[usize::from(b - 0x80)] })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_type_single_bits() {
        use EmbeddingPermission as P;
        assert_eq!(P::from_fs_type(0), P::Installable);
        assert_eq!(P::from_fs_type(0x0002), P::Restricted);
        assert_eq!(P::from_fs_type(0x0004), P::PreviewAndPrint);
        assert_eq!(P::from_fs_type(0x0008), P::Editable);
        // Reserved bit 0 alone carries no restriction.
        assert_eq!(P::from_fs_type(0x0001), P::Installable);
        // Modifier bits do not change the permission.
        assert_eq!(P::from_fs_type(0x0300), P::Installable);
        assert_eq!(P::from_fs_type(0x0302), P::Restricted);
    }

    #[test]
    fn fs_type_conflicting_bits_depend_on_the_table_version() {
        use EmbeddingPermission as P;
        // Versions 0-2: least restrictive wins.
        for version in 0..=2 {
            assert_eq!(P::from_os2(0x0006, version), P::PreviewAndPrint);
            assert_eq!(P::from_os2(0x000A, version), P::Editable);
            assert_eq!(P::from_os2(0x000E, version), P::Editable);
            assert_eq!(P::from_os2(0x000C, version), P::Editable);
        }
        // Version 3+: bits are exclusive; a malformed mix is read conservatively.
        for version in 3..=5 {
            assert_eq!(P::from_os2(0x0006, version), P::Restricted);
            assert_eq!(P::from_os2(0x000A, version), P::Restricted);
            assert_eq!(P::from_os2(0x000E, version), P::Restricted);
            assert_eq!(P::from_os2(0x000C, version), P::PreviewAndPrint);
            // Well-formed values are the same in every version.
            assert_eq!(P::from_os2(0x0000, version), P::Installable);
            assert_eq!(P::from_os2(0x0002, version), P::Restricted);
            assert_eq!(P::from_os2(0x0004, version), P::PreviewAndPrint);
            assert_eq!(P::from_os2(0x0008, version), P::Editable);
        }
    }

    #[test]
    fn fs_type_exhaustive_consistency() {
        use EmbeddingPermission as P;
        for fs_type in 0..=u16::MAX {
            let lenient = P::from_fs_type(fs_type);
            let strict = P::from_os2(fs_type, 4);
            // Installable exactly when no permission bit (1..=3) is set.
            assert_eq!(lenient == P::Installable, fs_type & 0x000E == 0, "{fs_type:#06x}");
            assert_eq!(strict == P::Installable, fs_type & 0x000E == 0, "{fs_type:#06x}");
            // The strict reading never allows more than the lenient one.
            if lenient == P::Restricted {
                assert_eq!(strict, P::Restricted);
            }
            if fs_type & 0x0002 != 0 {
                assert!(!strict.allows_embedding());
            }
            let flags = EmbeddingFlags::from_fs_type(fs_type);
            assert_eq!(flags.no_subsetting, fs_type & 0x0100 != 0);
            assert_eq!(flags.bitmap_only, fs_type & 0x0200 != 0);
        }
    }

    #[test]
    fn mac_roman_decoding() {
        assert_eq!(decode_mac_roman(b"Caf\x8e \xd2x\xd3 \xa9"), "Café “x” ©");
        // Every byte decodes to exactly one char.
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(decode_mac_roman(&all).chars().count(), 256);
    }

    #[test]
    fn utf16_decoding_is_lossy_but_total() {
        assert_eq!(decode_utf16_be(&[0x00, 0x41, 0x00, 0xE9]).as_deref(), Some("Aé"));
        // Unpaired surrogate becomes U+FFFD.
        assert_eq!(decode_utf16_be(&[0xD8, 0x00, 0x00, 0x41]).as_deref(), Some("\u{fffd}A"));
        // Odd length is not UTF-16.
        assert_eq!(decode_utf16_be(&[0x00, 0x41, 0x00]), None);
        assert_eq!(clean_name("  A\0b \n"), "Ab");
    }

    #[test]
    fn tags() {
        assert_eq!(parse_tag("wght"), Some(Tag::from_bytes(b"wght")));
        assert_eq!(parse_tag("wd"), Some(Tag::from_bytes(b"wd  ")));
        assert_eq!(parse_tag(""), None);
        assert_eq!(parse_tag("weight"), None);
        assert_eq!(parse_tag("wgh\u{e9}"), None);
        assert_eq!(tag_to_string(Tag::from_bytes(b"opsz")), "opsz");
        assert_eq!(tag_to_string(Tag(0x0001_0203)), "????");
    }

    #[test]
    fn metric_conversions() {
        let m = FontMetrics {
            units_per_em: 1000,
            ascender: 800,
            descender: -200,
            line_gap: 100,
            cap_height: 700,
            x_height: 500,
        };
        // 72 pt = 1 inch = 25.4 mm per em.
        assert!((m.ascender_mm(72.0) - 0.8 * 25.4).abs() < 1e-9);
        assert!((m.descender_mm(72.0) + 0.2 * 25.4).abs() < 1e-9);
        assert!((m.cap_height_mm(72.0) - 0.7 * 25.4).abs() < 1e-9);
        assert!((m.x_height_mm(72.0) - 0.5 * 25.4).abs() < 1e-9);
        assert!((m.line_gap_mm(72.0) - 0.1 * 25.4).abs() < 1e-9);
        assert!((m.line_height_mm(72.0) - 1.1 * 25.4).abs() < 1e-9);
        assert!((m.units_to_mm(1000.0, 36.0) - 12.7).abs() < 1e-9);
        // Round trip: the size that gives 6 mm capitals gives 6 mm capitals.
        let size = m.size_for_cap_height(6.0);
        assert!((m.cap_height_mm(size) - 6.0).abs() < 1e-9);
        let flat = FontMetrics { cap_height: 0, ..m };
        assert_eq!(flat.size_for_cap_height(6.0), 0.0);
    }
}
