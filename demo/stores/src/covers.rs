//! Typographic covers drawn by the store itself: an SVG per book, and PNG renditions for
//! print and for the EPUB.
//!
//! The six designs are deliberately different from each other (ground colour, framed or
//! not, serif / sans / display title) because they double as test inputs for the spine
//! generator of eBook Cadeau. The PNG renditions are rasterised with `resvg` using the
//! openly licensed fonts bundled in `assets/fonts` (SIL OFL), never system fonts, so the
//! output is identical on every machine. Browsers showing the SVG fall back to a generic
//! serif or sans-serif face when those fonts are not installed.

use std::sync::{Arc, OnceLock};

use resvg::{tiny_skia, usvg};

use crate::catalogue::Book;
use crate::error::StoreError;

/// Width of the cover artwork in SVG user units (one unit = 0.1 mm of a 110 mm cover).
pub const SVG_WIDTH: u32 = 1100;
/// Height of the cover artwork in SVG user units (178 mm).
pub const SVG_HEIGHT: u32 = 1780;
/// Pixel width of the print rendition served at `/covers/{id}.png` (1430 × 2314 px, about
/// 330 ppi on a 110 × 178 mm cover).
pub const PRINT_WIDTH_PX: u32 = 1430;
/// Pixel width of the small rendition embedded in EPUBs.
pub const THUMB_WIDTH_PX: u32 = 550;

const SPECTRAL_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Spectral-Regular.ttf");
const SPECTRAL_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Spectral-SemiBold.ttf");
const SPECTRAL_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/Spectral-Italic.ttf");
const ATKINSON_REGULAR: &[u8] =
    include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Regular.ttf");
const ATKINSON_BOLD: &[u8] = include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Bold.ttf");
const BEBAS_NEUE: &[u8] = include_bytes!("../../../assets/fonts/BebasNeue-Regular.ttf");

/// The typeface of the title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TitleFace {
    /// Spectral SemiBold (serif).
    Serif,
    /// Spectral Italic (serif).
    SerifItalic,
    /// Atkinson Hyperlegible Bold (sans-serif).
    Sans,
    /// Bebas Neue (condensed display capitals).
    Display,
}

impl TitleFace {
    /// `font-family`, `font-weight` and `font-style` attribute values.
    const fn attributes(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Serif => (SERIF_STACK, "600", "normal"),
            Self::SerifItalic => (SERIF_STACK, "400", "italic"),
            Self::Sans => (SANS_STACK, "700", "normal"),
            Self::Display => ("'Bebas Neue', Impact, 'Arial Narrow', sans-serif", "400", "normal"),
        }
    }

    /// Whether the design is a serif one (the author line follows the title's family).
    const fn is_serif(self) -> bool {
        matches!(self, Self::Serif | Self::SerifItalic)
    }
}

const SERIF_STACK: &str = "Spectral, Georgia, 'Times New Roman', serif";
const SANS_STACK: &str = "'Atkinson Hyperlegible', Helvetica, Arial, sans-serif";

/// A frame drawn inside the cover's edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// No frame: the ground runs to the edge.
    None,
    /// One rule.
    Single,
    /// A thick and a thin rule.
    Double,
}

/// The decorative element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ornament {
    /// A short rule under the title.
    Rule,
    /// A full-width band that bleeds off both edges.
    Band,
    /// Three small diamonds above the title.
    Diamonds,
    /// A cluster of discs (petals).
    Discs,
    /// A large disc with a bite taken out of it.
    Eclipse,
}

/// Everything that distinguishes one cover from another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverDesign {
    /// Ground colour, `#rrggbb`.
    pub ground: &'static str,
    /// Colour of the title and author.
    pub ink: &'static str,
    /// Colour of the frame and ornament.
    pub accent: &'static str,
    /// Frame style.
    pub frame: Frame,
    /// Title typeface.
    pub title_face: TitleFace,
    /// The title, split into lines by hand.
    pub title_lines: &'static [&'static str],
    /// Title size in SVG user units.
    pub title_size: u32,
    /// Decorative element.
    pub ornament: Ornament,
}

/// Escapes text for use in XML/XHTML content and double-quoted attributes. Characters that
/// XML 1.0 forbids (most C0 controls) are dropped.
#[must_use]
pub fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if c < ' ' || c == '\u{fffe}' || c == '\u{ffff}' => {}
            c => out.push(c),
        }
    }
    out
}

/// Draws the cover of `book` as a standalone SVG document.
#[must_use]
pub fn svg(book: &Book) -> String {
    let d = &book.cover;
    let (w, h) = (SVG_WIDTH, SVG_HEIGHT);
    let mut s = String::with_capacity(2048);
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" \
         height=\"{h}\" role=\"img\" aria-label=\"{label}\">\n<title>{label}</title>\n",
        label = xml_escape(&format!("{} — {}", book.full_title(), book.author)),
    ));
    s.push_str(&format!("<rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/>\n", d.ground));

    match d.frame {
        Frame::None => {}
        Frame::Single => s.push_str(&frame_rect(55, 6, d.accent)),
        Frame::Double => {
            s.push_str(&frame_rect(50, 10, d.accent));
            s.push_str(&frame_rect(78, 3, d.accent));
        }
    }

    // The title block is centred on y = 700; lines are set 1.08 em apart.
    let line_height = d.title_size * 108 / 100;
    let lines = u32::try_from(d.title_lines.len()).unwrap_or(1).max(1);
    let block = line_height * (lines - 1);
    let first_baseline = 700u32.saturating_sub(block / 2) + d.title_size * 35 / 100;
    let (family, weight, style) = d.title_face.attributes();
    let mut baseline = first_baseline;
    for line in d.title_lines {
        s.push_str(&format!(
            "<text x=\"550\" y=\"{baseline}\" text-anchor=\"middle\" font-family=\"{family}\" \
             font-weight=\"{weight}\" font-style=\"{style}\" font-size=\"{size}\" \
             fill=\"{ink}\">{text}</text>\n",
            size = d.title_size,
            ink = d.ink,
            text = xml_escape(line),
        ));
        baseline += line_height;
    }
    let title_top = first_baseline.saturating_sub(d.title_size);
    let title_bottom = baseline - line_height + d.title_size * 30 / 100;

    match d.ornament {
        Ornament::Rule => s.push_str(&format!(
            "<rect x=\"430\" y=\"{y}\" width=\"240\" height=\"8\" fill=\"{c}\"/>\n",
            y = title_bottom + 70,
            c = d.accent
        )),
        Ornament::Band => s.push_str(&format!(
            "<rect x=\"0\" y=\"{y}\" width=\"{w}\" height=\"46\" fill=\"{c}\"/>\n\
             <rect x=\"0\" y=\"{y2}\" width=\"{w}\" height=\"8\" fill=\"{c}\"/>\n",
            y = title_bottom + 80,
            y2 = title_bottom + 146,
            c = d.accent
        )),
        Ornament::Diamonds => {
            let cy = title_top.saturating_sub(90);
            for cx in [470u32, 550, 630] {
                s.push_str(&format!(
                    "<path d=\"M{cx} {top} L{right} {cy} L{cx} {bottom} L{left} {cy} Z\" \
                     fill=\"{c}\"/>\n",
                    top = cy.saturating_sub(26),
                    bottom = cy + 26,
                    left = cx - 18,
                    right = cx + 18,
                    c = d.accent
                ));
            }
        }
        Ornament::Discs => {
            let cy = title_top.saturating_sub(190);
            for (dx, dy) in [(0i32, -78i32), (74, -24), (46, 63), (-46, 63), (-74, -24)] {
                s.push_str(&format!(
                    "<circle cx=\"{cx}\" cy=\"{y}\" r=\"52\" fill=\"{c}\"/>\n",
                    cx = 550 + dx,
                    y = i64::from(cy) + i64::from(dy),
                    c = d.accent
                ));
            }
            s.push_str(&format!(
                "<circle cx=\"550\" cy=\"{cy}\" r=\"34\" fill=\"{c}\"/>\n",
                c = d.ink
            ));
        }
        Ornament::Eclipse => s.push_str(&format!(
            "<circle cx=\"550\" cy=\"1180\" r=\"230\" fill=\"{c}\"/>\n\
             <circle cx=\"640\" cy=\"1110\" r=\"170\" fill=\"{g}\"/>\n",
            c = d.accent,
            g = d.ground
        )),
    }

    let author_family = if d.title_face.is_serif() { SERIF_STACK } else { SANS_STACK };
    // The eclipse fills the lower half, so that design sets the author right under the
    // title; the others set it in the lower third.
    let author_y = if d.ornament == Ornament::Eclipse { title_bottom + 110 } else { 1400 };
    s.push_str(&format!(
        "<text x=\"550\" y=\"{author_y}\" text-anchor=\"middle\" font-family=\"{author_family}\" \
         font-weight=\"400\" font-size=\"66\" fill=\"{ink}\">{author}</text>\n",
        ink = d.ink,
        author = xml_escape(book.author),
    ));
    if let Some(sub) = book.subtitle {
        s.push_str(&format!(
            "<text x=\"550\" y=\"{y}\" text-anchor=\"middle\" font-family=\"{author_family}\" \
             font-weight=\"400\" font-size=\"48\" fill=\"{ink}\">{sub}</text>\n",
            y = author_y + 90,
            ink = d.ink,
            sub = xml_escape(sub),
        ));
    }
    s.push_str(&format!(
        "<text x=\"550\" y=\"1640\" text-anchor=\"middle\" font-family=\"{SANS_STACK}\" \
         font-weight=\"400\" font-size=\"30\" letter-spacing=\"6\" \
         fill=\"{ink}\">ÉDITION DE DÉMONSTRATION</text>\n",
        ink = d.ink,
    ));
    s.push_str("</svg>\n");
    s
}

fn frame_rect(inset: u32, stroke: u32, color: &str) -> String {
    format!(
        "<rect x=\"{inset}\" y=\"{inset}\" width=\"{w}\" height=\"{h}\" fill=\"none\" \
         stroke=\"{color}\" stroke-width=\"{stroke}\"/>\n",
        w = SVG_WIDTH - 2 * inset,
        h = SVG_HEIGHT - 2 * inset,
    )
}

/// The font database used for rasterising: only the bundled OFL fonts.
fn font_database() -> Arc<usvg::fontdb::Database> {
    static DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        for font in [
            SPECTRAL_REGULAR,
            SPECTRAL_SEMIBOLD,
            SPECTRAL_ITALIC,
            ATKINSON_REGULAR,
            ATKINSON_BOLD,
            BEBAS_NEUE,
        ] {
            db.load_font_data(font.to_vec());
        }
        db.set_serif_family("Spectral");
        db.set_sans_serif_family("Atkinson Hyperlegible");
        Arc::new(db)
    })
    .clone()
}

/// Rasterises an SVG produced by [`svg`] to an opaque 8-bit RGB PNG `width_px` wide (the
/// height follows the aspect ratio).
///
/// # Errors
/// [`StoreError::Cover`] when the SVG cannot be parsed, the size is unusable, or encoding
/// fails.
pub fn render_png(svg_source: &str, width_px: u32) -> Result<Vec<u8>, StoreError> {
    let options = usvg::Options { fontdb: font_database(), ..usvg::Options::default() };
    let tree = usvg::Tree::from_str(svg_source, &options)
        .map_err(|e| StoreError::Cover(format!("invalid SVG: {e}")))?;
    let size = tree.size();
    if width_px == 0 || width_px > 8_000 || size.width() <= 0.0 {
        return Err(StoreError::Cover(format!("unusable target width {width_px}")));
    }
    #[allow(clippy::cast_precision_loss)]
    let scale = width_px as f32 / size.width();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height_px = (size.height() * scale).round().clamp(1.0, 16_000.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width_px, height_px)
        .ok_or_else(|| StoreError::Cover("could not allocate the pixmap".to_owned()))?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    // The artwork covers the whole canvas with an opaque ground, so alpha is 255 everywhere
    // and the premultiplied RGBA buffer can be flattened to RGB by dropping the alpha byte.
    // An opaque RGB file is what print workflows expect (no transparency to flatten).
    let rgb: Vec<u8> = pixmap.data().chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]).collect();
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width_px, height_px);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| StoreError::Cover(format!("PNG header: {e}")))?;
        writer
            .write_image_data(&rgb)
            .map_err(|e| StoreError::Cover(format!("PNG data: {e}")))?;
        writer.finish().map_err(|e| StoreError::Cover(format!("PNG end: {e}")))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::catalogue;

    fn decode(png_bytes: &[u8]) -> (png::OutputInfo, Vec<u8>) {
        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        buf.truncate(info.buffer_size());
        (info, buf)
    }

    fn pixel(info: &png::OutputInfo, data: &[u8], x: u32, y: u32) -> [u8; 3] {
        let i = (y as usize * info.width as usize + x as usize) * 3;
        [data[i], data[i + 1], data[i + 2]]
    }

    fn parse_hex(color: &str) -> [u8; 3] {
        let v = crate::crypto::unhex(color.trim_start_matches('#')).unwrap();
        [v[0], v[1], v[2]]
    }

    #[test]
    fn xml_escape_handles_markup_and_controls() {
        assert_eq!(xml_escape("a<b>&\"c'"), "a&lt;b&gt;&amp;&quot;c&#39;");
        assert_eq!(xml_escape("x\u{0}\u{1b}y\u{ffff}"), "xy");
        assert_eq!(xml_escape("ligne\nsuivante\té"), "ligne\nsuivante\té");
    }

    #[test]
    fn svgs_are_well_formed_and_distinct() {
        let mut grounds = HashSet::new();
        let mut faces = HashSet::new();
        let mut framed = 0;
        for book in catalogue::books() {
            let source = svg(book);
            let options = usvg::Options { fontdb: font_database(), ..usvg::Options::default() };
            let tree = usvg::Tree::from_str(&source, &options).expect("valid SVG");
            assert_eq!(tree.size().width(), 1100.0);
            assert_eq!(tree.size().height(), 1780.0);
            assert!(source.contains(&xml_escape(book.author)));
            assert!(source.contains("ÉDITION DE DÉMONSTRATION"));
            grounds.insert(book.cover.ground);
            faces.insert(book.cover.title_face);
            framed += usize::from(book.cover.frame != Frame::None);
        }
        assert_eq!(grounds.len(), 6, "every cover has its own ground colour");
        assert!(faces.len() >= 3, "serif, sans and display titles are all represented");
        assert!((2..=4).contains(&framed), "some covers are framed and some are not");
    }

    #[test]
    fn thumbnails_render_with_the_right_ground_and_visible_ink() {
        for book in catalogue::books() {
            let png_bytes = render_png(&svg(book), THUMB_WIDTH_PX).unwrap();
            let (info, data) = decode(&png_bytes);
            assert_eq!((info.width, info.height), (550, 890));
            assert_eq!(info.color_type, png::ColorType::Rgb, "opaque RGB, no alpha channel");
            assert_eq!(info.bit_depth, png::BitDepth::Eight);
            let ground = parse_hex(book.cover.ground);
            assert_eq!(pixel(&info, &data, 2, 2), ground, "{}", book.id);
            assert_eq!(pixel(&info, &data, 547, 887), ground, "{}", book.id);
            // The title must actually have been drawn with the bundled fonts: a fair
            // number of pixels in the title area take the ink colour.
            let ink = parse_hex(book.cover.ink);
            let ink_pixels = (200..500)
                .flat_map(|y| (30..520).map(move |x| (x, y)))
                .filter(|&(x, y)| pixel(&info, &data, x, y) == ink)
                .count();
            assert!(ink_pixels > 1_500, "{}: only {ink_pixels} ink pixels", book.id);
        }
    }

    #[test]
    fn titles_stay_inside_the_safe_area() {
        // Nothing but the ground (or a frame / bleeding band) may touch the outer 4 % of
        // the width: a title that overflowed would be cut at the trim.
        for book in catalogue::books() {
            let png_bytes = render_png(&svg(book), THUMB_WIDTH_PX).unwrap();
            let (info, data) = decode(&png_bytes);
            let ground = parse_hex(book.cover.ground);
            let accent = parse_hex(book.cover.accent);
            for y in 0..info.height {
                for x in (0..12).chain(538..550) {
                    let p = pixel(&info, &data, x, y);
                    let is_ink = p == parse_hex(book.cover.ink) && p != accent && p != ground;
                    assert!(!is_ink, "{}: title ink at the edge ({x},{y})", book.id);
                }
            }
        }
    }

    #[test]
    fn print_rendition_is_large_enough_and_deterministic() {
        let book = catalogue::find("candide").unwrap();
        let a = render_png(&svg(book), PRINT_WIDTH_PX).unwrap();
        let b = render_png(&svg(book), PRINT_WIDTH_PX).unwrap();
        assert_eq!(a, b, "same input, same bytes");
        let (info, data) = decode(&a);
        assert!(info.width >= 1300 && info.height >= 2100, "{}x{}", info.width, info.height);
        assert_eq!((info.width, info.height), (1430, 2314));
        // The double frame is drawn in the accent colour: 50 units * 1.3 = 65 px in.
        assert_eq!(pixel(&info, &data, 68, 1000), parse_hex(book.cover.accent));
    }

    #[test]
    fn render_rejects_garbage_without_panicking() {
        for bad in ["", "<svg", "not xml at all", "<svg xmlns='http://www.w3.org/2000/svg'/>"] {
            // Either an error or (for the empty-but-valid document) a small image: no panic.
            let _ = render_png(bad, 100);
        }
        assert!(render_png("<svg", 100).is_err());
        let ok = svg(catalogue::find("candide").unwrap());
        assert!(render_png(&ok, 0).is_err());
        assert!(render_png(&ok, 1_000_000).is_err());
    }
}
