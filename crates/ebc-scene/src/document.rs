//! The document model handed to the PDF writer: pages, drawable items and resources.

use std::sync::Arc;

use crate::color::Color;
use crate::geom::{Point, Rect, Size, Transform};
use crate::path::{FillRule, Path};

/// Identifies an [`ImageResource`] within a [`Document`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(pub u32);

/// Identifies a [`FontResource`] within a [`Document`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(pub u32);

/// Shape of the ends of open stroked lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeCap {
    /// Squared off at the end point.
    #[default]
    Butt,
    /// Semicircle around the end point.
    Round,
    /// Square projecting half the line width beyond the end point.
    Square,
}

/// Shape of the corners of stroked lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeJoin {
    /// Sharp corner.
    #[default]
    Miter,
    /// Rounded corner.
    Round,
    /// Flattened corner.
    Bevel,
}

/// How a path outline is stroked.
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// Stroke colour.
    pub color: Color,
    /// Line width in **points** (print convention: hairline marks are 0.25 pt).
    pub width_pt: f64,
    /// Dash pattern as alternating on/off lengths in millimetres; empty for a solid line.
    pub dash_mm: Vec<f64>,
    /// Line cap.
    pub cap: StrokeCap,
    /// Line join.
    pub join: StrokeJoin,
}

impl Stroke {
    /// A solid stroke with butt caps and miter joins.
    #[must_use]
    pub fn solid(color: Color, width_pt: f64) -> Self {
        Self { color, width_pt, dash_mm: Vec::new(), cap: StrokeCap::Butt, join: StrokeJoin::Miter }
    }

    /// A dashed stroke; `dash_mm` alternates on/off lengths in millimetres.
    #[must_use]
    pub fn dashed(color: Color, width_pt: f64, dash_mm: Vec<f64>) -> Self {
        Self { color, width_pt, dash_mm, cap: StrokeCap::Butt, join: StrokeJoin::Miter }
    }
}

/// One positioned glyph of a [`Item::Glyphs`] run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    /// Glyph index in the font (not a Unicode code point).
    pub id: u16,
    /// Horizontal offset of the glyph origin from the run origin, in millimetres.
    pub x: f64,
    /// Vertical offset of the glyph origin from the run baseline, in millimetres
    /// (positive = lower on the page).
    pub y: f64,
}

/// A drawable element of a page. Items are painted in order (later items on top).
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A filled and/or stroked vector path.
    Path {
        /// The outline, in page millimetres.
        path: Path,
        /// Fill colour and rule, if the path is filled.
        fill: Option<(Color, FillRule)>,
        /// Stroke style, if the outline is stroked.
        stroke: Option<Stroke>,
    },
    /// A raster image scaled to fill `rect` exactly (the caller handles aspect ratio,
    /// usually by wrapping the item in a clipped [`Item::Group`]).
    Image {
        /// Which image to draw.
        image: ImageId,
        /// Destination rectangle in page millimetres.
        rect: Rect,
    },
    /// A run of already-shaped glyphs set on one baseline, drawn as real (embedded-font) text.
    Glyphs {
        /// Which font the glyph ids refer to.
        font: FontId,
        /// Font size in points.
        size_pt: f64,
        /// Text colour.
        color: Color,
        /// Position of the run origin on the baseline, in page millimetres.
        origin: Point,
        /// The glyphs, positioned relative to `origin`.
        glyphs: Vec<Glyph>,
        /// The source text of the run, used for the PDF `ToUnicode` map / text extraction.
        /// `glyph_text[i]` is the text that glyph `i` stands for (empty for the trailing
        /// parts of a ligature or for marks merged into a previous glyph).
        glyph_text: Vec<String>,
    },
    /// A group of items with a transform and an optional clip path applied to all of them.
    Group {
        /// Transform applied to the children (and to `clip`).
        transform: Transform,
        /// Optional clip path, in the group's (pre-transform) coordinates.
        clip: Option<Path>,
        /// Optional name, emitted as a PDF layer (optional content group) when the writer
        /// is asked to keep layers, e.g. `"marks"`, `"artwork"`, `"guides"`.
        layer: Option<String>,
        /// The grouped items.
        items: Vec<Item>,
    },
}

impl Item {
    /// A filled path with the non-zero winding rule and no stroke.
    #[must_use]
    pub fn fill(path: Path, color: Color) -> Self {
        Self::Path { path, fill: Some((color, FillRule::NonZero)), stroke: None }
    }

    /// A stroked path with no fill.
    #[must_use]
    pub fn stroke(path: Path, stroke: Stroke) -> Self {
        Self::Path { path, fill: None, stroke: Some(stroke) }
    }

    /// A filled rectangle.
    #[must_use]
    pub fn filled_rect(rect: Rect, color: Color) -> Self {
        Self::fill(Path::rect(rect), color)
    }

    /// A plain group with a transform, no clip and no layer name.
    #[must_use]
    pub fn group(transform: Transform, items: Vec<Item>) -> Self {
        Self::Group { transform, clip: None, layer: None, items }
    }

    /// A group clipped to a rectangle.
    #[must_use]
    pub fn clipped(clip: Rect, items: Vec<Item>) -> Self {
        Self::Group {
            transform: Transform::IDENTITY,
            clip: Some(Path::rect(clip)),
            layer: None,
            items,
        }
    }

    /// A named layer (e.g. `"marks"`).
    #[must_use]
    pub fn layer(name: impl Into<String>, items: Vec<Item>) -> Self {
        Self::Group { transform: Transform::IDENTITY, clip: None, layer: Some(name.into()), items }
    }
}

/// Pixel layout of an uncompressed [`ImageData::Raw`] image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 1 byte per pixel, 0 = black.
    Gray8,
    /// 3 bytes per pixel, sRGB.
    Rgb8,
    /// 4 bytes per pixel, sRGB + non-premultiplied alpha.
    Rgba8,
    /// 4 bytes per pixel, C M Y K with 255 = 100 % ink.
    Cmyk8,
}

impl PixelFormat {
    /// Bytes per pixel.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Gray8 => 1,
            Self::Rgb8 => 3,
            Self::Rgba8 | Self::Cmyk8 => 4,
        }
    }
}

/// The pixel data of an image resource.
#[derive(Debug, Clone, PartialEq)]
pub enum ImageData {
    /// A complete baseline or progressive JPEG file, embedded as-is (`DCTDecode`).
    Jpeg {
        /// The JPEG file bytes.
        bytes: Arc<[u8]>,
        /// Width in pixels.
        width_px: u32,
        /// Height in pixels.
        height_px: u32,
        /// Number of colour components: 1 (grey), 3 (YCbCr/RGB) or 4 (CMYK/YCCK).
        components: u8,
    },
    /// Uncompressed pixels, compressed losslessly by the writer.
    Raw {
        /// Row-major pixel bytes, `width_px * height_px * format.bytes_per_pixel()` long.
        pixels: Arc<[u8]>,
        /// Width in pixels.
        width_px: u32,
        /// Height in pixels.
        height_px: u32,
        /// Pixel layout.
        format: PixelFormat,
    },
}

impl ImageData {
    /// Pixel dimensions `(width, height)`.
    #[must_use]
    pub fn dimensions_px(&self) -> (u32, u32) {
        match self {
            Self::Jpeg { width_px, height_px, .. } | Self::Raw { width_px, height_px, .. } => {
                (*width_px, *height_px)
            }
        }
    }

    /// Effective resolution in pixels per inch when the image is drawn at `size` millimetres,
    /// as `(horizontal, vertical)`.
    #[must_use]
    pub fn effective_ppi(&self, size: Size) -> (f64, f64) {
        let (w, h) = self.dimensions_px();
        (
            f64::from(w) / (size.w / crate::geom::MM_PER_INCH),
            f64::from(h) / (size.h / crate::geom::MM_PER_INCH),
        )
    }
}

/// An image that pages can draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageResource {
    /// Identifier referenced by [`Item::Image`].
    pub id: ImageId,
    /// The pixel data.
    pub data: ImageData,
    /// Optional embedded ICC profile describing the image's colour space
    /// (when absent, RGB data is treated as sRGB and CMYK data as the output intent).
    pub icc_profile: Option<Arc<[u8]>>,
}

/// A font that pages can set glyphs in. The writer embeds a subset of it.
#[derive(Debug, Clone, PartialEq)]
pub struct FontResource {
    /// Identifier referenced by [`Item::Glyphs`].
    pub id: FontId,
    /// The complete TrueType/OpenType font file.
    pub data: Arc<[u8]>,
    /// Face index inside a font collection (0 for plain `.ttf`/`.otf` files).
    pub face_index: u32,
}

/// The PDF/X conformance level the writer should target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfStandard {
    /// Plain PDF 1.7 with page boxes; no PDF/X claims.
    Plain,
    /// PDF/X-1a:2001 — CMYK/spot only, no transparency, PDF 1.3.
    X1a2001,
    /// PDF/X-3:2002 — colour-managed (ICC-tagged RGB allowed), no transparency, PDF 1.3.
    X3_2002,
    /// PDF/X-4 — colour-managed, live transparency and layers allowed, PDF 1.6.
    #[default]
    X4,
}

/// The intended printing condition (PDF `OutputIntent`).
#[derive(Debug, Clone, PartialEq)]
pub struct OutputIntent {
    /// Identifier of the characterisation, e.g. `"FOGRA39"` or `"FOGRA51"`.
    pub condition_identifier: String,
    /// Human-readable description, e.g. `"ISO Coated v2 (ECI)"`.
    pub info: String,
    /// Registry where the identifier is defined, normally `"http://www.color.org"`.
    pub registry_name: String,
    /// The destination ICC profile to embed. Required for PDF/X-4; optional for
    /// PDF/X-1a and PDF/X-3 when the condition is in the ICC registry.
    pub icc_profile: Option<Arc<[u8]>>,
    /// Number of colour components of `icc_profile` (4 for CMYK).
    pub components: u8,
}

/// Document information (PDF Info dictionary / XMP).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocInfo {
    /// Document title (required by PDF/X).
    pub title: String,
    /// Author.
    pub author: Option<String>,
    /// Subject.
    pub subject: Option<String>,
    /// Keywords.
    pub keywords: Vec<String>,
    /// The application that composed the document.
    pub creator: Option<String>,
    /// Creation time as seconds since the Unix epoch (UTC). Passing it explicitly keeps
    /// output byte-for-byte reproducible; `None` omits the date.
    pub created_unix: Option<i64>,
    /// BCP 47 language tag of the document's text, e.g. `"fr-FR"`.
    pub language: Option<String>,
    /// Extra key/value pairs written to the PDF Info dictionary, e.g.
    /// `("EBC.SpineWidthMm", "10")`. Keys must be ASCII without spaces. Used to carry the
    /// card geometry so that preflight tools can verify fold positions.
    pub custom: Vec<(String, String)>,
}

/// One page (one side of one printed sheet).
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// Finished (cut) size. Page coordinates start at its top-left corner.
    pub trim: Size,
    /// Bleed beyond the trim on every side, in millimetres (`BleedBox = trim + bleed`).
    pub bleed: f64,
    /// Extra margin beyond the bleed, reserved for printer's marks
    /// (`MediaBox = trim + bleed + slug`). Zero when no marks are drawn. It applies to all
    /// four sides, unless [`slug_v`](Self::slug_v) is set: then to the left and right only.
    pub slug: f64,
    /// Extra margin beyond the bleed at the **top and bottom**, when it differs from
    /// [`slug`](Self::slug); `None` means the same as `slug`.
    ///
    /// This is what lets a page be a sheet of paper with the trim centred on it: a
    /// 230 × 178 mm trim centred on A4 landscape (297 × 210 mm) with a 2 mm bleed has
    /// `slug = 31.5` and `slug_v = Some(14.0)`. The trim stays centred in the media box;
    /// off-centre placements cannot be expressed.
    pub slug_v: Option<f64>,
    /// The page content, painted in order.
    pub items: Vec<Item>,
    /// Optional human-readable name, e.g. `"outside"` or `"inside"`.
    pub label: Option<String>,
}

impl Page {
    /// Creates an empty page with the same slug on all four sides (`slug_v` is `None`).
    #[must_use]
    pub fn new(trim: Size, bleed: f64, slug: f64) -> Self {
        Self { trim, bleed, slug, slug_v: None, items: Vec::new(), label: None }
    }

    /// The trim box in page coordinates (always at the origin).
    #[must_use]
    pub fn trim_box(&self) -> Rect {
        Rect::new(0.0, 0.0, self.trim.w, self.trim.h)
    }

    /// The bleed box in page coordinates.
    #[must_use]
    pub fn bleed_box(&self) -> Rect {
        self.trim_box().inflate(self.bleed)
    }

    /// The media box (full sheet incl. marks area) in page coordinates: the trim box
    /// grown by `bleed + slug` to the left and right and by `bleed + slug_v` (or
    /// `bleed + slug` when `slug_v` is `None`) at the top and bottom.
    #[must_use]
    pub fn media_box(&self) -> Rect {
        let slug_v = self.slug_v.unwrap_or(self.slug);
        self.trim_box().inflate_xy(self.bleed + self.slug, self.bleed + slug_v)
    }
}

/// A complete printable document.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    /// Document information.
    pub info: DocInfo,
    /// Target conformance level.
    pub standard: PdfStandard,
    /// Intended printing condition. Required for every PDF/X level.
    pub output_intent: Option<OutputIntent>,
    /// The pages, in order.
    pub pages: Vec<Page>,
    /// Images referenced by the pages.
    pub images: Vec<ImageResource>,
    /// Fonts referenced by the pages.
    pub fonts: Vec<FontResource>,
}

impl Document {
    /// Registers an image and returns its id.
    pub fn add_image(&mut self, data: ImageData, icc_profile: Option<Arc<[u8]>>) -> ImageId {
        let id = ImageId(u32::try_from(self.images.len()).unwrap_or(u32::MAX));
        self.images.push(ImageResource { id, data, icc_profile });
        id
    }

    /// Registers a font and returns its id.
    pub fn add_font(&mut self, data: Arc<[u8]>, face_index: u32) -> FontId {
        let id = FontId(u32::try_from(self.fonts.len()).unwrap_or(u32::MAX));
        self.fonts.push(FontResource { id, data, face_index });
        id
    }

    /// Looks up an image resource.
    #[must_use]
    pub fn image(&self, id: ImageId) -> Option<&ImageResource> {
        self.images.iter().find(|i| i.id == id)
    }

    /// Looks up a font resource.
    #[must_use]
    pub fn font(&self, id: FontId) -> Option<&FontResource> {
        self.fonts.iter().find(|f| f.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_boxes_nest() {
        let page = Page::new(Size::new(230.0, 178.0), 3.0, 7.0);
        assert_eq!(page.trim_box(), Rect::new(0.0, 0.0, 230.0, 178.0));
        assert_eq!(page.bleed_box(), Rect::new(-3.0, -3.0, 236.0, 184.0));
        assert_eq!(page.media_box(), Rect::new(-10.0, -10.0, 250.0, 198.0));
        assert!(page.media_box().contains_rect(&page.bleed_box(), 0.0));
        assert!(page.bleed_box().contains_rect(&page.trim_box(), 0.0));
    }

    #[test]
    fn a_vertical_slug_makes_the_media_box_a_sheet_around_a_centred_trim() {
        // The poche spread centred on A4 landscape, 2 mm bleed: 33.5 mm of sheet left and
        // right of the trim, 16 mm above and below.
        let mut page = Page::new(Size::new(230.0, 178.0), 2.0, 31.5);
        assert_eq!(page.slug_v, None);
        // One slug: the same margin on all four sides, so not A4.
        assert_eq!(page.media_box(), Rect::new(-33.5, -33.5, 297.0, 245.0));
        page.slug_v = Some(14.0);
        assert_eq!(page.media_box(), Rect::new(-33.5, -16.0, 297.0, 210.0));
        // Trim and bleed do not depend on the slugs, and the trim stays centred.
        assert_eq!(page.trim_box(), Rect::new(0.0, 0.0, 230.0, 178.0));
        assert_eq!(page.bleed_box(), Rect::new(-2.0, -2.0, 234.0, 182.0));
        assert_eq!(page.media_box().center(), page.trim_box().center());
        assert!(page.media_box().contains_rect(&page.bleed_box(), 0.0));
        // `Some(slug)` and `None` are the same page geometry.
        page.slug_v = Some(31.5);
        assert_eq!(page.media_box(), Page::new(page.trim, 2.0, 31.5).media_box());
        // A vertical slug only: the bleed box touches the media box left and right.
        let mut page = Page::new(Size::new(100.0, 50.0), 3.0, 0.0);
        page.slug_v = Some(10.0);
        assert_eq!(page.media_box(), Rect::new(-3.0, -13.0, 106.0, 76.0));
    }

    #[test]
    fn resources_get_sequential_ids() {
        let mut doc = Document::default();
        let px: Arc<[u8]> = Arc::from(vec![0u8; 3]);
        let a = doc.add_image(
            ImageData::Raw {
                pixels: px.clone(),
                width_px: 1,
                height_px: 1,
                format: PixelFormat::Rgb8,
            },
            None,
        );
        let b = doc.add_image(
            ImageData::Raw { pixels: px, width_px: 1, height_px: 1, format: PixelFormat::Rgb8 },
            None,
        );
        assert_eq!((a, b), (ImageId(0), ImageId(1)));
        assert!(doc.image(b).is_some());
        assert!(doc.image(ImageId(9)).is_none());
        let f = doc.add_font(Arc::from(vec![1u8, 2, 3]), 0);
        assert_eq!(f, FontId(0));
        assert!(doc.font(f).is_some());
    }

    #[test]
    fn effective_resolution() {
        let img = ImageData::Jpeg {
            bytes: Arc::from(vec![]),
            width_px: 1299,
            height_px: 2102,
            components: 3,
        };
        let (x, y) = img.effective_ppi(Size::new(110.0, 178.0));
        assert!((x - 300.0).abs() < 0.2, "{x}");
        assert!((y - 300.0).abs() < 0.2, "{y}");
    }
}
