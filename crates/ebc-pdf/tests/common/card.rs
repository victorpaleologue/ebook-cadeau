//! A complete "card-like" sample document: the outside spread (back | spine | front)
//! and the inside spread (instructions, QR-like grid, code), built only from scene items.

use std::sync::Arc;

use ebc_scene::{
    Color, Document, FontId, ImageData, Item, Page, Path, PdfStandard, PixelFormat, Point, Rect,
    Size, Stroke, Transform,
};

use super::{base_doc, fixture, font, jpeg_cmyk_with, jpeg_rgb, text_item};

/// Finished size of the spread: two 110 mm panels and a 10 mm spine.
pub(crate) const TRIM: Size = Size::new(230.0, 178.0);
pub(crate) const BLEED: f64 = 3.0;
pub(crate) const SLUG: f64 = 7.0;
/// Distance from the media box to the trim box.
pub(crate) const MARGIN: f64 = BLEED + SLUG;
pub(crate) const SPINE: Rect = Rect::new(110.0, 0.0, 10.0, 178.0);
/// The front panel including its bleed; the cover image is clipped to it.
pub(crate) const FRONT: Rect = Rect::new(120.0, -3.0, 113.0, 184.0);
/// Where the (larger) cover image is placed before clipping.
pub(crate) const COVER_IMAGE: Rect = Rect::new(107.5, -3.0, 138.0, 184.0);
pub(crate) const COVER_PX: (u32, u32) = (300, 400);

pub(crate) const QR_ORIGIN: Point = Point::new(140.0, 45.0);
pub(crate) const QR_MODULE: f64 = 1.2;
pub(crate) const QR_SIZE: u32 = 25;
/// The red rectangle on the inside.
pub(crate) const STAMP: Rect = Rect::new(20.0, 120.0, 30.0, 15.0);
/// Centre and radius of the coloured disc on the inside.
pub(crate) const DISC: (Point, f64) = (Point::new(75.0, 127.5), 7.5);

pub(crate) const SPINE_TITLE: &str = "L'ÉTRANGER";
pub(crate) const SPINE_AUTHOR: &str = "ALBERT CAMUS";
pub(crate) const OUTSIDE_TEXT: [&str; 5] = [
    "Un livre numérique à offrir",
    "Ouvrez la carte : le code est à l'intérieur.",
    "Fonctionne sur liseuse, tablette et téléphone.",
    "L'Étranger",
    "Albert Camus",
];
pub(crate) const INSIDE_TEXT: [&str; 7] = [
    "Un livre vous attend",
    "1. Scannez le code avec votre téléphone",
    "2. Ou saisissez-le sur votre liseuse : ebk.example",
    "3. Bonne lecture !",
    "Pour Zoé — Joyeux Noël, de la part de Mémé",
    "K7Q2M-9XTRD",
    "https://ebk.example/k/K7Q2M9XTRD",
];

/// Whether module `(x, y)` of the QR-like grid is dark: three finder patterns plus a
/// deterministic scatter.
pub(crate) fn qr_module(x: u32, y: u32) -> bool {
    let finder = |fx: u32, fy: u32| -> Option<bool> {
        let (dx, dy) = (x.checked_sub(fx)?, y.checked_sub(fy)?);
        (dx < 7 && dy < 7).then(|| {
            let ring = dx.min(6 - dx).min(dy).min(6 - dy);
            ring != 1
        })
    };
    finder(0, 0)
        .or_else(|| finder(QR_SIZE - 7, 0))
        .or_else(|| finder(0, QR_SIZE - 7))
        .unwrap_or_else(|| (x * 7 + y * 13 + x * y) % 3 == 0)
}

/// The dark modules as one path of merged horizontal runs.
fn qr_path() -> Path {
    let mut path = Path::new();
    for y in 0..QR_SIZE {
        let mut x = 0;
        while x < QR_SIZE {
            if !qr_module(x, y) {
                x += 1;
                continue;
            }
            let start = x;
            while x < QR_SIZE && qr_module(x, y) {
                x += 1;
            }
            path.push_rect(Rect::new(
                QR_ORIGIN.x + f64::from(start) * QR_MODULE,
                QR_ORIGIN.y + f64::from(y) * QR_MODULE,
                f64::from(x - start) * QR_MODULE,
                QR_MODULE,
            ));
        }
    }
    path
}

/// Colour of the generated cover at pixel `(x, y)`: a teal-to-orange gradient with a
/// pale disc.
pub(crate) fn cover_rgb(x: u32, y: u32) -> [u8; 3] {
    let (dx, dy) = (f64::from(x) - 150.0, f64::from(y) - 150.0);
    if dx * dx + dy * dy <= 60.0 * 60.0 {
        return [250, 245, 235];
    }
    let t = f64::from(y) / 399.0;
    let mix = |a: f64, b: f64| (a + (b - a) * t).round() as u8;
    [mix(20.0, 240.0), mix(110.0, 140.0), mix(120.0, 40.0)]
}

/// The same artwork as ink values (C, M, Y, K; 255 = 100 %) for PDF/X-1a.
pub(crate) fn cover_cmyk(x: u16, y: u16) -> [u8; 4] {
    let (dx, dy) = (f64::from(x) - 150.0, f64::from(y) - 150.0);
    if dx * dx + dy * dy <= 60.0 * 60.0 {
        return [0, 5, 15, 0];
    }
    let t = f64::from(y) / 399.0;
    let mix = |a: f64, b: f64| (a + (b - a) * t).round() as u8;
    [mix(220.0, 0.0), mix(40.0, 120.0), mix(80.0, 230.0), mix(30.0, 0.0)]
}

fn crop_and_fold_marks() -> Item {
    let registration = Stroke::solid(Color::Registration, 0.25);
    let dashed = Stroke::dashed(Color::Registration, 0.25, vec![1.0, 0.5]);
    let (w, h) = (TRIM.w, TRIM.h);
    let line = |x0: f64, y0: f64, x1: f64, y1: f64, stroke: &Stroke| {
        Item::stroke(Path::line(Point::new(x0, y0), Point::new(x1, y1)), stroke.clone())
    };
    let mut items = Vec::new();
    for x in [0.0, w] {
        items.push(line(x, -9.0, x, -4.0, &registration));
        items.push(line(x, h + 4.0, x, h + 9.0, &registration));
    }
    for y in [0.0, h] {
        items.push(line(-9.0, y, -4.0, y, &registration));
        items.push(line(w + 4.0, y, w + 9.0, y, &registration));
    }
    for x in [SPINE.x, SPINE.x + SPINE.w] {
        items.push(line(x, -9.0, x, -4.0, &dashed));
        items.push(line(x, h + 4.0, x, h + 9.0, &dashed));
    }
    Item::layer("marks", items)
}

struct Fonts {
    sans: (FontId, Arc<[u8]>),
    serif_bold: (FontId, Arc<[u8]>),
    serif_italic: (FontId, Arc<[u8]>),
    display: (FontId, Arc<[u8]>),
    code: (FontId, Arc<[u8]>),
    mono_cff: (FontId, Arc<[u8]>),
}

fn register_fonts(doc: &mut Document) -> Fonts {
    let mut add = |data: Arc<[u8]>| (doc.add_font(data.clone(), 0), data);
    Fonts {
        sans: add(font("AtkinsonHyperlegible-Regular.ttf")),
        serif_bold: add(font("Spectral-SemiBold.ttf")),
        serif_italic: add(font("Spectral-Italic.ttf")),
        display: add(font("BebasNeue-Regular.ttf")),
        code: add(font("CourierPrime-Bold.ttf")),
        mono_cff: add(fixture("SourceCodePro-Regular.otf")),
    }
}

fn text(font: &(FontId, Arc<[u8]>), s: &str, size_pt: f64, x: f64, y: f64, color: Color) -> Item {
    text_item(font.0, &font.1, s, size_pt, Point::new(x, y), color)
}

fn outside(doc: &mut Document, fonts: &Fonts) -> Page {
    let rgb_allowed = doc.standard != PdfStandard::X1a2001;
    let cover = if rgb_allowed {
        doc.add_image(jpeg_rgb(COVER_PX.0, COVER_PX.1, cover_rgb), None)
    } else {
        doc.add_image(jpeg_cmyk_with(300, 400, cover_cmyk, true), None)
    };

    let mut page = Page::new(TRIM, BLEED, SLUG);
    page.label = Some("outside".into());
    let paper = Color::cmyk(0.02, 0.03, 0.10, 0.0);
    let navy = Color::cmyk(0.85, 0.55, 0.25, 0.35);

    // Background to the bleed edge, back panel copy.
    page.items.push(Item::filled_rect(page.bleed_box(), paper));
    page.items.push(text(&fonts.serif_bold, OUTSIDE_TEXT[0], 16.0, 12.0, 30.0, Color::BLACK));
    page.items.push(text(&fonts.sans, OUTSIDE_TEXT[1], 9.0, 12.0, 40.0, Color::BLACK));
    page.items.push(text(&fonts.sans, OUTSIDE_TEXT[2], 9.0, 12.0, 45.0, Color::gray(0.35)));

    // Front: the cover fills the panel ("cover" fit) and is clipped to it.
    page.items.push(Item::clipped(FRONT, vec![Item::Image { image: cover, rect: COVER_IMAGE }]));
    page.items.push(text(&fonts.serif_bold, OUTSIDE_TEXT[3], 26.0, 130.0, 125.0, Color::WHITE));
    page.items.push(text(&fonts.serif_italic, OUTSIDE_TEXT[4], 14.0, 130.0, 137.0, Color::WHITE));

    // Spine: a band to the bleed edge and the title set top-to-bottom. The group turns
    // its content a quarter turn clockwise around the point where the baseline starts.
    page.items
        .push(Item::filled_rect(Rect::new(SPINE.x, -BLEED, SPINE.w, TRIM.h + 2.0 * BLEED), navy));
    page.items.push(Item::group(
        Transform::translate(112.6, 12.0).then_inner(Transform::rotate(90.0)),
        vec![
            text(&fonts.display, SPINE_TITLE, 20.0, 0.0, 0.0, Color::WHITE),
            text(&fonts.display, SPINE_AUTHOR, 11.0, 118.0, -1.0, Color::cmyk(0.0, 0.1, 0.4, 0.0)),
        ],
    ));

    page.items.push(crop_and_fold_marks());
    page
}

fn inside(doc: &mut Document, fonts: &Fonts) -> Page {
    let standard = doc.standard;
    let mut page = Page::new(TRIM, BLEED, SLUG);
    page.label = Some("inside".into());

    page.items.push(text(&fonts.serif_bold, INSIDE_TEXT[0], 18.0, 15.0, 35.0, Color::BLACK));
    for (i, line) in INSIDE_TEXT[1..4].iter().enumerate() {
        page.items.push(text(&fonts.sans, line, 10.0, 15.0, 50.0 + 7.0 * i as f64, Color::BLACK));
    }
    page.items.push(text(&fonts.serif_italic, INSIDE_TEXT[4], 12.0, 15.0, 90.0, Color::gray(0.25)));

    // QR-like symbol: white quiet zone, K-only modules as a single path.
    let side = f64::from(QR_SIZE) * QR_MODULE;
    let quiet = 4.0 * QR_MODULE;
    page.items.push(Item::filled_rect(
        Rect::new(QR_ORIGIN.x - quiet, QR_ORIGIN.y - quiet, side + 2.0 * quiet, side + 2.0 * quiet),
        Color::WHITE,
    ));
    page.items.push(Item::fill(qr_path(), Color::BLACK));
    page.items.push(text(&fonts.code, INSIDE_TEXT[5], 16.0, 140.0, 90.0, Color::BLACK));
    page.items.push(text(&fonts.mono_cff, INSIDE_TEXT[6], 7.0, 140.0, 96.0, Color::gray(0.3)));

    // A red block, a coloured disc (RGB where the standard allows it), a grey circle
    // outline with a thick dashed stroke.
    page.items.push(Item::filled_rect(STAMP, Color::cmyk(0.0, 1.0, 1.0, 0.0)));
    let blue = if standard == PdfStandard::X1a2001 {
        Color::cmyk(1.0, 0.6, 0.0, 0.0)
    } else {
        Color::rgb8(0, 102, 204)
    };
    page.items.push(Item::fill(Path::circle(DISC.0, DISC.1), blue));
    page.items.push(Item::stroke(
        Path::circle(Point::new(95.0, 127.5), 6.0),
        Stroke::dashed(Color::gray(0.5), 1.5, vec![2.0, 1.0]),
    ));

    // A translucent badge where live transparency is allowed.
    if matches!(standard, PdfStandard::X4 | PdfStandard::Plain) {
        let mut rgba = Vec::new();
        for y in 0..16u32 {
            for x in 0..16u32 {
                rgba.extend_from_slice(&[230, 160, 20, ((x + y) * 8) as u8]);
            }
        }
        let badge = doc.add_image(
            ImageData::Raw {
                pixels: Arc::from(rgba),
                width_px: 16,
                height_px: 16,
                format: PixelFormat::Rgba8,
            },
            None,
        );
        page.items.push(Item::Image { image: badge, rect: Rect::new(15.0, 145.0, 20.0, 20.0) });
    }

    // Fold guides on their own layer, then the printer's marks.
    let guide = Stroke::dashed(Color::gray(0.6), 0.5, vec![2.0, 1.0]);
    page.items.push(Item::layer(
        "guides",
        [SPINE.x, SPINE.x + SPINE.w]
            .into_iter()
            .map(|x| {
                Item::stroke(Path::line(Point::new(x, 0.0), Point::new(x, TRIM.h)), guide.clone())
            })
            .collect(),
    ));
    page.items.push(crop_and_fold_marks());
    page
}

/// The complete two-page sample card for `standard`.
pub(crate) fn document(standard: PdfStandard) -> Document {
    let mut doc = base_doc(standard);
    doc.info.subject = Some("Carte cadeau imprimable".into());
    doc.info.keywords = vec!["eBook".into(), "cadeau".into()];
    doc.info.custom = vec![
        ("EBC.Format".into(), "poche".into()),
        ("EBC.SpineWidthMm".into(), "10".into()),
        ("EBC.FoldXMm".into(), "110,120".into()),
    ];
    let fonts = register_fonts(&mut doc);
    let outside = outside(&mut doc, &fonts);
    let inside = inside(&mut doc, &fonts);
    doc.pages = vec![outside, inside];
    doc
}
