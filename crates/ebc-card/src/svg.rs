//! A faithful SVG rendering of a scene page, for live previews in a browser.
//!
//! [`render_page`] draws everything the PDF writer would: paths with their fill rule,
//! strokes and dashes, clipping groups, transforms, images (as `data:` URIs) and glyph
//! runs (as filled outlines, so that no font needs to be available to the viewer).
//!
//! Differences from the PDF, by design: colours are the on-screen approximation of
//! [`ebc_scene::Color::to_srgb_approx`] (not colour-managed; the registration colour is
//! black), text is not selectable, and the whole media box is shown on a white sheet.

use std::fmt::Write as _;
use std::io::Cursor;

use ebc_scene::{
    Color, Document, FillRule, FontId, ImageData, Item, Path, PathSeg, PixelFormat, Stroke,
    StrokeCap, StrokeJoin, Transform, pt_to_mm,
};
use ebc_text::{Font, ShapedGlyph, ShapedText};
use image::ImageEncoder;
use image::codecs::png::PngEncoder;

/// Groups nested deeper than this are not drawn (the PDF writer refuses them too).
const MAX_DEPTH: usize = 24;

/// Renders page `page_index` of `doc` as a standalone SVG document.
///
/// The SVG's user unit is the millimetre and its view box is the page's media box, so
/// coordinates are the page coordinates of the scene (origin at the top-left corner of
/// the trim box). Named layers become `<g data-layer="…">`.
///
/// A page index out of range gives an empty 1 × 1 mm SVG. Items that cannot be drawn (an
/// image or a font that is missing from the document or cannot be read) are skipped.
/// The output is deterministic.
#[must_use]
pub fn render_page(doc: &Document, page_index: usize) -> String {
    let Some(page) = doc.pages.get(page_index) else {
        return r#"<svg xmlns="http://www.w3.org/2000/svg" width="1mm" height="1mm" viewBox="0 0 1 1"/>"#
            .to_owned();
    };
    let media = page.media_box();
    let mut renderer = Renderer { doc, fonts: Vec::new(), out: String::new(), clips: 0 };
    let out = &mut renderer.out;
    let _ = write!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}mm" height="{}mm" viewBox="{} {} {} {}">"#,
        num(media.w),
        num(media.h),
        num(media.x),
        num(media.y),
        num(media.w),
        num(media.h)
    );
    if let Some(label) = &page.label {
        let _ = write!(out, "<title>{}</title>", escape(label));
    }
    let _ = write!(
        out,
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#ffffff"/>"##,
        num(media.x),
        num(media.y),
        num(media.w),
        num(media.h)
    );
    for item in &page.items {
        renderer.item(item, 0);
    }
    renderer.out.push_str("</svg>");
    renderer.out
}

struct Renderer<'a> {
    doc: &'a Document,
    fonts: Vec<(FontId, Option<Font>)>,
    out: String,
    clips: usize,
}

impl Renderer<'_> {
    fn font(&mut self, id: FontId) -> Option<Font> {
        if let Some((_, font)) = self.fonts.iter().find(|(known, _)| *known == id) {
            return font.clone();
        }
        let font = self
            .doc
            .font(id)
            .and_then(|res| Font::from_bytes(res.data.clone(), res.face_index).ok());
        self.fonts.push((id, font.clone()));
        font
    }

    fn item(&mut self, item: &Item, depth: usize) {
        match item {
            Item::Path { path, fill, stroke } => {
                if path.is_empty() || (fill.is_none() && stroke.is_none()) {
                    return;
                }
                let _ = write!(self.out, r#"<path d="{}""#, path_data(path));
                match fill {
                    Some((color, rule)) => {
                        let _ = write!(self.out, r#" fill="{}""#, hex(color));
                        if *rule == FillRule::EvenOdd {
                            self.out.push_str(r#" fill-rule="evenodd""#);
                        }
                    }
                    None => self.out.push_str(r#" fill="none""#),
                }
                if let Some(stroke) = stroke {
                    stroke_attributes(&mut self.out, stroke);
                }
                self.out.push_str("/>");
            }
            Item::Image { image, rect } => {
                let Some(uri) = self.doc.image(*image).and_then(|res| data_uri(&res.data)) else {
                    return;
                };
                let _ = write!(
                    self.out,
                    r#"<image x="{}" y="{}" width="{}" height="{}" preserveAspectRatio="none" href="{uri}"/>"#,
                    num(rect.x),
                    num(rect.y),
                    num(rect.w),
                    num(rect.h)
                );
            }
            Item::Glyphs { font, size_pt, color, origin, glyphs, .. } => {
                let Some(font) = self.font(*font) else { return };
                let run = ShapedText {
                    glyphs: glyphs
                        .iter()
                        .map(|g| ShapedGlyph {
                            id: g.id,
                            x: g.x,
                            y: g.y,
                            advance: 0.0,
                            text: String::new(),
                        })
                        .collect(),
                    width: 0.0,
                    size_pt: *size_pt,
                };
                let outline = run.to_outline(&font, *origin);
                if !outline.is_empty() {
                    let _ = write!(
                        self.out,
                        r#"<path d="{}" fill="{}"/>"#,
                        path_data(&outline),
                        hex(color)
                    );
                }
            }
            Item::Group { transform, clip, layer, items } => {
                if depth >= MAX_DEPTH || items.is_empty() || transform.det().abs() < 1e-12 {
                    return;
                }
                if clip.as_ref().is_some_and(Path::is_empty) {
                    return;
                }
                self.out.push_str("<g");
                if let Some(name) = layer {
                    let _ = write!(self.out, r#" data-layer="{}""#, escape(name));
                }
                if *transform != Transform::IDENTITY {
                    let t = transform;
                    let _ = write!(
                        self.out,
                        r#" transform="matrix({} {} {} {} {} {})""#,
                        num(t.a),
                        num(t.b),
                        num(t.c),
                        num(t.d),
                        num(t.e),
                        num(t.f)
                    );
                }
                match clip {
                    Some(clip) => {
                        // The clip path is in the group's own coordinates: referenced
                        // from the element that carries the transform, it is transformed
                        // with it, as in the scene model.
                        self.clips += 1;
                        let id = self.clips;
                        let _ = write!(
                            self.out,
                            r##" clip-path="url(#clip{id})"><clipPath id="clip{id}"><path d="{}"/></clipPath>"##,
                            path_data(clip)
                        );
                    }
                    None => self.out.push('>'),
                }
                for child in items {
                    self.item(child, depth + 1);
                }
                self.out.push_str("</g>");
            }
        }
    }
}

fn stroke_attributes(out: &mut String, stroke: &Stroke) {
    let _ = write!(
        out,
        r#" stroke="{}" stroke-width="{}""#,
        hex(&stroke.color),
        num(pt_to_mm(stroke.width_pt.max(0.0)))
    );
    if !stroke.dash_mm.is_empty() {
        let dashes: Vec<String> = stroke.dash_mm.iter().map(|d| num(*d)).collect();
        let _ = write!(out, r#" stroke-dasharray="{}""#, dashes.join(" "));
    }
    match stroke.cap {
        StrokeCap::Butt => {}
        StrokeCap::Round => out.push_str(r#" stroke-linecap="round""#),
        StrokeCap::Square => out.push_str(r#" stroke-linecap="square""#),
    }
    match stroke.join {
        StrokeJoin::Miter => {}
        StrokeJoin::Round => out.push_str(r#" stroke-linejoin="round""#),
        StrokeJoin::Bevel => out.push_str(r#" stroke-linejoin="bevel""#),
    }
}

/// A number with at most four decimals, without trailing zeros; non-finite values are 0.
fn num(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_owned();
    }
    let text = format!("{value:.4}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-0" { "0".to_owned() } else { text.to_owned() }
}

fn path_data(path: &Path) -> String {
    let mut d = String::with_capacity(path.segs.len() * 14);
    for seg in &path.segs {
        if !d.is_empty() {
            d.push(' ');
        }
        match *seg {
            PathSeg::MoveTo(p) => {
                let _ = write!(d, "M{} {}", num(p.x), num(p.y));
            }
            PathSeg::LineTo(p) => {
                let _ = write!(d, "L{} {}", num(p.x), num(p.y));
            }
            PathSeg::QuadTo(c, p) => {
                let _ = write!(d, "Q{} {} {} {}", num(c.x), num(c.y), num(p.x), num(p.y));
            }
            PathSeg::CubicTo(c1, c2, p) => {
                let _ = write!(
                    d,
                    "C{} {} {} {} {} {}",
                    num(c1.x),
                    num(c1.y),
                    num(c2.x),
                    num(c2.y),
                    num(p.x),
                    num(p.y)
                );
            }
            PathSeg::Close => d.push('Z'),
        }
    }
    d
}

/// `#rrggbb` for a scene colour (the on-screen approximation).
fn hex(color: &Color) -> String {
    let [r, g, b] = color.to_srgb_approx();
    // Clamped to 0..=255 before the cast.
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(r), byte(g), byte(b))
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// The image as a `data:` URI: JPEG files as they are, raw pixels as PNG (CMYK pixels
/// through the naive on-screen conversion).
fn data_uri(data: &ImageData) -> Option<String> {
    match data {
        ImageData::Jpeg { bytes, .. } => Some(format!("data:image/jpeg;base64,{}", base64(bytes))),
        ImageData::Raw { pixels, width_px, height_px, format } => {
            let (w, h) = (*width_px, *height_px);
            let expected =
                (w as usize).checked_mul(h as usize)?.checked_mul(format.bytes_per_pixel())?;
            if w == 0 || h == 0 || pixels.len() != expected {
                return None;
            }
            let mut png = Vec::new();
            let encoder = PngEncoder::new(Cursor::new(&mut png));
            let result = match format {
                PixelFormat::Gray8 => {
                    encoder.write_image(pixels, w, h, image::ExtendedColorType::L8)
                }
                PixelFormat::Rgb8 => {
                    encoder.write_image(pixels, w, h, image::ExtendedColorType::Rgb8)
                }
                PixelFormat::Rgba8 => {
                    encoder.write_image(pixels, w, h, image::ExtendedColorType::Rgba8)
                }
                PixelFormat::Cmyk8 => {
                    let mut rgb = Vec::with_capacity(pixels.len() / 4 * 3);
                    for px in pixels.chunks_exact(4) {
                        let k = 255 - u32::from(px[3]);
                        for ink in &px[..3] {
                            // 0..=255 by construction.
                            rgb.push(((255 - u32::from(*ink)) * k / 255) as u8);
                        }
                    }
                    encoder.write_image(&rgb, w, h, image::ExtendedColorType::Rgb8)
                }
            };
            result.ok()?;
            Some(format!("data:image/png;base64,{}", base64(&png)))
        }
    }
}

/// Standard base64 with padding.
fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let sextet = |shift: u32| char::from(ALPHABET[((n >> shift) & 63) as usize]);
        out.push(sextet(18));
        out.push(sextet(12));
        out.push(if chunk.len() > 1 { sextet(6) } else { '=' });
        out.push(if chunk.len() > 2 { sextet(0) } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_scene::{Page, Point, Rect, Size};
    use std::sync::Arc;

    #[test]
    fn base64_matches_the_rfc_vectors() {
        let cases = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (plain, encoded) in cases {
            assert_eq!(base64(plain.as_bytes()), encoded);
        }
    }

    #[test]
    fn numbers_are_compact_and_finite() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(-0.00001), "0");
        assert_eq!(num(1.23456), "1.2346");
        assert_eq!(num(-3.5), "-3.5");
        assert_eq!(num(f64::NAN), "0");
        assert_eq!(num(f64::INFINITY), "0");
    }

    fn sample() -> Document {
        let mut doc = Document::default();
        let image = doc.add_image(
            ImageData::Raw {
                pixels: Arc::from(vec![255u8, 0, 0, 0, 255, 0, 0, 0, 255, 9, 9, 9]),
                width_px: 2,
                height_px: 2,
                format: PixelFormat::Rgb8,
            },
            None,
        );
        let mut page = Page::new(Size::new(100.0, 50.0), 3.0, 5.0);
        page.label = Some("out<side>".to_owned());
        page.items.push(Item::filled_rect(
            Rect::new(-3.0, -3.0, 106.0, 56.0),
            Color::cmyk(1.0, 0.0, 0.0, 0.0),
        ));
        let mut triangle = Path::new();
        triangle.move_to(0.0, 0.0).line_to(10.0, 0.0).quad_to(10.0, 10.0, 0.0, 10.0).close();
        page.items.push(Item::Path {
            path: triangle,
            fill: Some((Color::BLACK, FillRule::EvenOdd)),
            stroke: Some(Stroke::dashed(Color::Registration, 0.5, vec![1.0, 2.0])),
        });
        page.items.push(Item::Group {
            transform: Transform::rotate_about(90.0, Point::new(50.0, 25.0)),
            clip: Some(Path::rect(Rect::new(40.0, 15.0, 20.0, 20.0))),
            layer: Some("art\"work".to_owned()),
            items: vec![Item::Image { image, rect: Rect::new(40.0, 15.0, 20.0, 20.0) }],
        });
        doc.pages.push(page);
        doc
    }

    #[test]
    fn renders_paths_groups_clips_and_images() {
        let doc = sample();
        let svg = render_page(&doc, 0);
        assert!(svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" width="116mm" height="66mm" viewBox="-8 -8 116 66">"#), "{svg}");
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("<title>out&lt;side&gt;</title>"));
        assert!(svg.contains(r##"fill="#00ffff""##), "cyan ground: {svg}");
        assert!(svg.contains(r#"fill-rule="evenodd""#));
        assert!(svg.contains(r#"stroke-dasharray="1 2""#));
        assert!(svg.contains("Q10 10 0 10 Z"));
        assert!(svg.contains(r#"data-layer="art&quot;work""#));
        assert!(svg.contains(r##"clip-path="url(#clip1)"><clipPath id="clip1">"##));
        assert!(svg.contains("transform=\"matrix("));
        assert!(svg.contains("href=\"data:image/png;base64,iVBORw0KGgo"));
        assert_eq!(svg.matches("<g").count(), svg.matches("</g>").count());
        // Deterministic.
        assert_eq!(svg, render_page(&doc, 0));
    }

    #[test]
    fn out_of_range_pages_and_broken_resources_do_not_panic() {
        let mut doc = sample();
        assert!(render_page(&doc, 9).starts_with("<svg"));
        // An image id that does not exist, a font that is not a font, a raw image whose
        // buffer is too short: all skipped.
        doc.pages[0].items.push(Item::Image {
            image: ebc_scene::ImageId(99),
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        });
        let font = doc.add_font(Arc::from(vec![1u8, 2, 3]), 0);
        doc.pages[0].items.push(Item::Glyphs {
            font,
            size_pt: 10.0,
            color: Color::BLACK,
            origin: Point::new(0.0, 0.0),
            glyphs: vec![ebc_scene::Glyph { id: 1, x: 0.0, y: 0.0 }],
            glyph_text: vec!["a".to_owned()],
        });
        let short = doc.add_image(
            ImageData::Raw {
                pixels: Arc::from(vec![0u8; 5]),
                width_px: 2,
                height_px: 2,
                format: PixelFormat::Cmyk8,
            },
            None,
        );
        doc.pages[0].items.push(Item::Image { image: short, rect: Rect::new(0.0, 0.0, 1.0, 1.0) });
        let svg = render_page(&doc, 0);
        assert_eq!(svg.matches("<image").count(), 1);
    }
}
