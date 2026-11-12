//! Advice for dark or heavy ink across the creases of the outside.
//!
//! The card is creased twice, either side of the spine. Heavy ink coverage on a crease
//! cracks when the board is folded, and a dark colour shows the white fibres of the crack
//! (`docs/research/03-print-requirements.md` §4.3 and §6.1; one French printer excludes
//! torn folds under heavy ink from complaints). Industry practice is not to give dark
//! covers up but to finish them so that the crease holds: **matt or soft-touch
//! lamination** (a film that bridges the crease), **paper grain parallel to the fold**
//! (board folds cleanly with the grain and cracks across it), and no hard colour break on
//! the fold itself — the card's one-ground design already avoids that.
//!
//! So when what is printed within [`CREASE_BAND_MM`] of a crease on the outside is heavy
//! (above [`DARK_CREASE_INK_PERCENT`] of total ink) or dark (L\* below
//! [`DARK_CREASE_LSTAR`]), the composer returns
//! [`CardWarning::DarkAcrossCreases`](crate::CardWarning::DarkAcrossCreases). Only for
//! professional (PDF/X) files: the advice is about the order placed with a printer.
//!
//! # What is measured
//!
//! The leaves of the outside page that intersect the band around each crease, inside the
//! trim height, in painting order; one that covers the whole band hides those under it.
//! Flat fills count with their colour as written; CMYK images (the converted cover when
//! it reaches the crease, image spines) with the pixels of the columns inside the band —
//! the 95th percentile of their total ink, so that a few dark pixels do not decide. L\* is
//! measured through the output profile's `A2B1` table (relative colorimetric: paper is
//! L\* 100) when the transform has one, else from the naive device colour. Images are
//! assumed axis-aligned (the composer never rotates them); a CMYK JPEG cover passed
//! through unchanged is not decoded and does not count.

use ebc_color::{CmykProfile, Lab, RenderingIntent, from_scene_color};
use ebc_scene::{Color, Document, ImageData, ImageId, Item, PixelFormat, Rect, Transform};
use ebc_text::outline_bounds;

use crate::error::CardWarning;
use crate::inspect::transform_rect;

/// Total ink, in per cent, above which a colour on a crease is heavy enough to crack.
///
/// A house value between the K 100 alone that research 03 (§4.3) accepts for a black
/// crossing a crease (100 %) and the 240–300 % of rich blacks and dark builds: it catches
/// any four-colour dark build and leaves mid-tones (a mid blue is about 120–160 %) alone.
pub const DARK_CREASE_INK_PERCENT: f32 = 180.0;
/// CIELAB lightness below which a colour on a crease is dark enough for a crack to show.
///
/// A house value: below about L\* 35 a colour reads as dark (navy, burgundy, forest
/// green, K 100 at L\* 17), and the white board fibres of a crack stand out on it.
pub const DARK_CREASE_LSTAR: f32 = 35.0;
/// Half-width of the band around a crease that is examined, in millimetres: the 2 mm of
/// the "no colour boundary near a crease" rule (research 03 §1.2) and of the preflight's
/// own check.
pub const CREASE_BAND_MM: f64 = 2.0;
/// Percentile of the band's pixels that stands for an image.
const IMAGE_PERCENTILE: f64 = 0.95;

/// What is painted in a crease band.
#[derive(Debug, Clone, Copy)]
enum Mark {
    Flat(Color),
    /// An image, with its full (unclipped) rectangle on the page.
    Image(ImageId, Rect),
}

/// The heaviest ink and the darkest lightness found in a band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Heaviness {
    pub(crate) ink_percent: f32,
    pub(crate) lightness: f32,
}

impl Heaviness {
    fn merge(self, other: Self) -> Self {
        Self {
            ink_percent: self.ink_percent.max(other.ink_percent),
            lightness: self.lightness.min(other.lightness),
        }
    }

    fn is_dark(self) -> bool {
        self.ink_percent > DARK_CREASE_INK_PERCENT || self.lightness < DARK_CREASE_LSTAR
    }
}

/// The advice for page `page` of `doc`, whose creases are at `creases` (x, page
/// millimetres) over a trim `height` from y = 0; `None` when nothing dark or heavy
/// reaches a crease.
pub(crate) fn advice(
    doc: &Document,
    page: usize,
    creases: &[f64],
    height: f64,
    profile: Option<&CmykProfile>,
) -> Option<CardWarning> {
    let page = doc.pages.get(page)?;
    let mut found: Option<Heaviness> = None;
    for &x in creases {
        let band = Rect::new(x - CREASE_BAND_MM, 0.0, 2.0 * CREASE_BAND_MM, height);
        let mut marks: Vec<(Mark, Rect)> = Vec::new();
        for item in &page.items {
            collect(item, Transform::IDENTITY, None, band, &mut marks);
        }
        for (mark, visible) in &marks {
            if let Some(h) = measure(doc, *mark, visible, profile) {
                found = Some(found.map_or(h, |f| f.merge(h)));
            }
        }
    }
    let found = found?;
    found.is_dark().then_some(CardWarning::DarkAcrossCreases {
        max_ink_percent: (found.ink_percent * 10.0).round() / 10.0,
        min_lightness: (found.lightness * 10.0).round() / 10.0,
    })
}

/// Collects, in painting order, what `item` paints inside `band`; a mark that covers the
/// whole band removes those collected before it.
fn collect(
    item: &Item,
    ctm: Transform,
    clip: Option<Rect>,
    band: Rect,
    out: &mut Vec<(Mark, Rect)>,
) {
    let (mark, bounds) = match item {
        Item::Group { transform, clip: group_clip, items, .. } => {
            let ctm = ctm.then_inner(*transform);
            let mut clip = clip;
            if let Some(path) = group_clip {
                let Some(b) = outline_bounds(path).or_else(|| path.control_bounds()) else {
                    return;
                };
                let b = transform_rect(&ctm, b);
                clip = match clip {
                    Some(c) => match c.intersect(&b) {
                        Some(both) => Some(both),
                        None => return,
                    },
                    None => Some(b),
                };
            }
            for child in items {
                collect(child, ctm, clip, band, out);
            }
            return;
        }
        Item::Path { path, fill: Some((color, _)), .. } => {
            let Some(b) = outline_bounds(path).or_else(|| path.control_bounds()) else { return };
            (Mark::Flat(*color), transform_rect(&ctm, b))
        }
        Item::Image { image, rect } => {
            let full = transform_rect(&ctm, *rect);
            (Mark::Image(*image, full), full)
        }
        // Strokes and text never fill a crease band.
        Item::Path { .. } | Item::Glyphs { .. } => return,
    };
    let visible = match clip {
        Some(c) => match bounds.intersect(&c) {
            Some(v) => v,
            None => return,
        },
        None => bounds,
    };
    let Some(inside) = visible.intersect(&band) else { return };
    if inside.w <= 1e-6 || inside.h <= 1e-6 {
        return;
    }
    if inside.contains_rect(&band, 1e-6) {
        out.clear();
    }
    out.push((mark, inside));
}

/// Total ink (per cent) and L\* of a colour as written.
fn flat(color: Color, profile: Option<&CmykProfile>) -> Option<Heaviness> {
    let ink = match color {
        Color::Gray(level) => (1.0 - level.clamp(0.0, 1.0)) * 100.0,
        other => other.total_ink_percent()?,
    };
    let lightness = match (color, profile) {
        (Color::Cmyk { c, m, y, k }, Some(profile)) => {
            let cmyk = [c, m, y, k].map(f64::from);
            profile.cmyk_to_lab(cmyk, RenderingIntent::RelativeColorimetric)[0]
        }
        _ => Lab::from_rgb8(from_scene_color(&color)).l,
    };
    // L* in 0–100: the narrowing is a rounding.
    #[allow(clippy::cast_possible_truncation)]
    let lightness = lightness as f32;
    Some(Heaviness { ink_percent: ink, lightness })
}

fn measure(
    doc: &Document,
    mark: Mark,
    inside: &Rect,
    profile: Option<&CmykProfile>,
) -> Option<Heaviness> {
    let (id, full) = match mark {
        Mark::Flat(color) => return flat(color, profile),
        Mark::Image(id, full) => (id, full),
    };
    let ImageData::Raw { pixels, width_px, height_px, format: PixelFormat::Cmyk8 } =
        &doc.image(id)?.data
    else {
        return None;
    };
    let (w, h) = (*width_px as usize, *height_px as usize);
    if w == 0 || h == 0 || full.w <= 0.0 || full.h <= 0.0 || pixels.len() < w * h * 4 {
        return None;
    }
    // The pixel columns and rows under `inside`.
    let span = |from: f64, to: f64, origin: f64, size: f64, count: usize| {
        let scale = count as f64 / size;
        // Clamped to 0..=count: the conversions cannot overflow.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let lo = (((from - origin) * scale).floor().max(0.0) as usize).min(count);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let hi = (((to - origin) * scale).ceil().max(0.0) as usize).min(count);
        lo..hi.max(lo)
    };
    let cols = span(inside.x, inside.right(), full.x, full.w, w);
    let rows = span(inside.y, inside.bottom(), full.y, full.h, h);
    let mut inks: Vec<(u32, usize)> = Vec::with_capacity(cols.len() * rows.len());
    for row in rows {
        for col in cols.clone() {
            let at = (row * w + col) * 4;
            let px = pixels.get(at..at + 4)?;
            inks.push((px.iter().map(|&v| u32::from(v)).sum(), at));
        }
    }
    if inks.is_empty() {
        return None;
    }
    inks.sort_unstable();
    // A rank in 0..len: the conversion is exact enough.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rank = ((inks.len() - 1) as f64 * IMAGE_PERCENTILE).round() as usize;
    let (_, at) = inks[rank.min(inks.len() - 1)];
    let px = pixels.get(at..at + 4)?;
    let [c, m, y, k] = [px[0], px[1], px[2], px[3]].map(|v| f32::from(v) / 255.0);
    flat(Color::cmyk(c, m, y, k), profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_scene::{Page, Path, Size};

    fn doc(items: Vec<Item>, images: Vec<ImageData>) -> Document {
        let mut doc = Document::default();
        for data in images {
            doc.add_image(data, None);
        }
        let mut page = Page::new(Size::new(230.0, 178.0), 3.0, 0.0);
        page.items = items;
        doc.pages.push(page);
        doc
    }

    fn ground(color: Color) -> Item {
        Item::filled_rect(Rect::new(-3.0, -3.0, 236.0, 184.0), color)
    }

    #[test]
    fn heavy_and_dark_grounds_are_advised_light_ones_are_not() {
        let creases = [110.0, 120.0];
        let navy = Color::cmyk(0.75, 0.64, 0.24, 0.77);
        let advice_for =
            |color| advice(&doc(vec![ground(color)], vec![]), 0, &creases, 178.0, None);
        match advice_for(navy) {
            Some(CardWarning::DarkAcrossCreases { max_ink_percent, min_lightness }) => {
                assert!((max_ink_percent - 240.0).abs() < 0.1, "{max_ink_percent}");
                assert!(min_lightness < 35.0, "{min_lightness}");
            }
            other => panic!("{other:?}"),
        }
        // K 100 alone: light on ink, but dark.
        assert!(advice_for(Color::BLACK).is_some());
        // Cream and a mid tone: nothing to say.
        assert_eq!(advice_for(Color::cmyk(0.05, 0.08, 0.2, 0.0)), None);
        assert_eq!(advice_for(Color::cmyk(0.6, 0.3, 0.0, 0.0)), None);
        assert_eq!(advice_for(Color::WHITE), None);
        // No page, no advice.
        assert_eq!(advice(&Document::default(), 0, &creases, 178.0, None), None);
    }

    #[test]
    fn what_covers_the_band_hides_what_is_under_it() {
        let creases = [110.0];
        // A dark ground under a light band that covers the crease ± 2 mm: nothing dark
        // shows on the crease.
        let band = Item::filled_rect(Rect::new(105.0, -3.0, 20.0, 184.0), Color::WHITE);
        let d = doc(vec![ground(Color::BLACK), band], vec![]);
        assert_eq!(advice(&d, 0, &creases, 178.0, None), None);
        // A narrower band does not hide it.
        let narrow = Item::filled_rect(Rect::new(109.0, -3.0, 2.0, 184.0), Color::WHITE);
        let d = doc(vec![ground(Color::BLACK), narrow], vec![]);
        assert!(advice(&d, 0, &creases, 178.0, None).is_some());
        // Marks outside the trim height and strokes do not count.
        let tick = Item::filled_rect(Rect::new(109.0, -10.0, 2.0, 5.0), Color::BLACK);
        let line = Item::stroke(
            Path::line(ebc_scene::Point::new(110.0, 0.0), ebc_scene::Point::new(110.0, 178.0)),
            ebc_scene::Stroke::solid(Color::BLACK, 1.0),
        );
        let d = doc(vec![ground(Color::WHITE), tick, line], vec![]);
        assert_eq!(advice(&d, 0, &creases, 178.0, None), None);
    }

    #[test]
    fn images_count_by_the_pixels_next_to_the_crease() {
        // A 20 × 10 px CMYK image over x 100..120 mm: left half heavy, right half white.
        let (w, h) = (20u32, 10u32);
        let mut pixels = Vec::new();
        for _ in 0..h {
            for x in 0..w {
                pixels.extend_from_slice(if x < 10 {
                    &[200, 180, 150, 230]
                } else {
                    &[0, 0, 0, 0]
                });
            }
        }
        let data = ImageData::Raw {
            pixels: pixels.into(),
            width_px: w,
            height_px: h,
            format: PixelFormat::Cmyk8,
        };
        let image = Item::Image { image: ImageId(0), rect: Rect::new(100.0, 0.0, 20.0, 178.0) };
        let d = doc(vec![ground(Color::WHITE), image], vec![data.clone()]);
        // Crease at 104 mm: heavy pixels only.
        let Some(CardWarning::DarkAcrossCreases { max_ink_percent, .. }) =
            advice(&d, 0, &[104.0], 178.0, None)
        else {
            panic!("heavy pixels on the crease")
        };
        assert!(max_ink_percent > 290.0, "{max_ink_percent}");
        // Crease at 118 mm: white pixels only.
        assert_eq!(advice(&d, 0, &[118.0], 178.0, None), None);
        // A clipped image counts only where it is visible.
        let clipped = Item::clipped(
            Rect::new(110.0, 0.0, 10.0, 178.0),
            vec![Item::Image { image: ImageId(0), rect: Rect::new(100.0, 0.0, 20.0, 178.0) }],
        );
        let d = doc(vec![ground(Color::WHITE), clipped], vec![data]);
        assert_eq!(advice(&d, 0, &[110.0], 178.0, None), None);
        // A hostile image record (too few pixels) is ignored, not trusted.
        let broken = ImageData::Raw {
            pixels: vec![255u8; 8].into(),
            width_px: 20,
            height_px: 10,
            format: PixelFormat::Cmyk8,
        };
        let image = Item::Image { image: ImageId(0), rect: Rect::new(100.0, 0.0, 20.0, 178.0) };
        let d = doc(vec![image], vec![broken]);
        assert_eq!(advice(&d, 0, &[104.0], 178.0, None), None);
    }
}
