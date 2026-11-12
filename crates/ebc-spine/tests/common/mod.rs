//! Shared helpers of the integration tests: bundled fonts, procedurally generated covers
//! and spines, and path inspection. No file other than the fonts in `assets/fonts` is
//! read; every image is generated here.
#![allow(dead_code)]

use ebc_i18n::SpineDirection;
use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::{BookMeta, Contributor};
use ebc_scene::{Color, Item, Path, PathSeg, Pixmap, Rect};
use ebc_spine::{SpineDesign, SpineRequest, SpineSource};
use ebc_text::{Font, outline_bounds};

/// Loads one of the fonts bundled in `assets/fonts`.
pub fn font(file: &str) -> Font {
    let path = format!("{}/../../assets/fonts/{file}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    Font::from_bytes(bytes, 0).expect("a bundled font parses")
}

pub fn serif() -> Font {
    font("Spectral-SemiBold.ttf")
}

pub fn serif_regular() -> Font {
    font("Spectral-Regular.ttf")
}

pub fn sans() -> Font {
    font("AtkinsonHyperlegible-Bold.ttf")
}

pub fn caps() -> Font {
    font("BebasNeue-Regular.ttf")
}

/// A book with a title and authors.
pub fn book(title: &str, authors: &[&str]) -> BookMeta {
    let mut book = BookMeta::titled(title);
    book.language = Some("fr".to_owned());
    book.contributors = authors.iter().map(|name| Contributor::author(*name)).collect();
    book
}

/// The spine of the *poche* card (10 × 178 mm) with the professional profile: the
/// rectangle including bleed, the bleed and the safe margin along the creases.
pub fn poche() -> (Rect, f64, f64) {
    let format = CardFormat::poche();
    let profile = PrintProfile::professional_eu();
    let rect = format.outside().with_bleed(profile.bleed).spine;
    (rect, profile.bleed, profile.spine_safe_margin)
}

/// A request for the *poche* spine.
pub fn request<'a>(
    book: &'a BookMeta,
    cover: Option<&'a Pixmap>,
    title_font: &'a Font,
    author_font: &'a Font,
    direction: SpineDirection,
) -> SpineRequest<'a> {
    let (rect, bleed, safe_margin) = poche();
    SpineRequest {
        book,
        cover,
        source: SpineSource::Synthesize,
        rect,
        bleed,
        title_font,
        author_font,
        direction,
        safe_margin,
    }
}

// ----- generated images ------------------------------------------------------------------

/// A deterministic pseudo-random generator (xorshift), so that "busy" images are the
/// same on every run.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn below(&mut self, n: u32) -> u32 {
        (self.next() % u64::from(n.max(1))) as u32
    }
}

pub const COVER_W: u32 = 330;
pub const COVER_H: u32 = 534;

/// A cover in one flat colour.
pub fn flat_cover(rgb: [u8; 3]) -> Pixmap {
    Pixmap::filled(COVER_W, COVER_H, [rgb[0], rgb[1], rgb[2], 255])
}

pub fn fill(img: &mut Pixmap, x0: u32, y0: u32, x1: u32, y1: u32, rgb: [u8; 3]) {
    for y in y0..y1.min(img.height()) {
        for x in x0..x1.min(img.width()) {
            img.set(x, y, [rgb[0], rgb[1], rgb[2], 255]);
        }
    }
}

/// A "photographic" cover: a dark blue ground (about 45 % of the surface once the
/// blocks are drawn) covered with many blocks of strongly different colours, right up to
/// the edges.
pub fn busy_cover(seed: u64) -> Pixmap {
    let mut img = flat_cover([24, 38, 92]);
    let mut rng = Rng::new(seed);
    let colours: [[u8; 3]; 6] =
        [[214, 48, 49], [253, 203, 110], [0, 184, 148], [240, 240, 240], [9, 9, 20], [232, 67, 147]];
    for _ in 0..160 {
        let w = 12 + rng.below(50);
        let h = 12 + rng.below(50);
        let x = rng.below(COVER_W);
        let y = rng.below(COVER_H);
        let colour = colours[rng.below(colours.len() as u32) as usize];
        fill(&mut img, x.saturating_sub(w / 2), y.saturating_sub(h / 2), x + w / 2, y + h / 2, colour);
    }
    img
}

/// A busy picture inside a uniform frame `frame` pixels thick.
pub fn framed_cover(frame_rgb: [u8; 3], frame: u32, seed: u64) -> Pixmap {
    let inner = busy_cover(seed);
    let mut img = flat_cover(frame_rgb);
    for y in frame..COVER_H - frame {
        for x in frame..COVER_W - frame {
            let p = inner.get(x, y).expect("inside the cover");
            img.set(x, y, p);
        }
    }
    img
}

/// A cover whose left part is a plain band and whose right part is busy.
pub fn banded_cover(band_rgb: [u8; 3], band_px: u32, seed: u64) -> Pixmap {
    let mut img = busy_cover(seed);
    fill(&mut img, 0, 0, band_px, COVER_H, band_rgb);
    img
}

/// Pixels per millimetre of the generated spine images (300 ppi).
pub const SPINE_PX_PER_MM: f64 = 300.0 / 25.4;

/// An upright image of a real spine at trim size: `width_mm` wide, for a book
/// `height_mm` high, in the `ground` colour, with a line of "lettering" along it — dark
/// bars `text_mm` across, centred — imitating a rotated title.
pub fn spine_image(width_mm: f64, height_mm: f64, ground: [u8; 3], text_mm: f64) -> Pixmap {
    let w = (width_mm * SPINE_PX_PER_MM).round().max(1.0) as u32;
    let h = (height_mm * SPINE_PX_PER_MM).round().max(1.0) as u32;
    let mut img = Pixmap::filled(w, h, [ground[0], ground[1], ground[2], 255]);
    let band = (text_mm * SPINE_PX_PER_MM).round() as u32;
    let x0 = (w.saturating_sub(band)) / 2;
    letters(&mut img, x0, x0 + band, h / 5, 4 * h / 5);
    img
}

/// Draws bars imitating letters set along the spine, in the columns `x0..x1`.
pub fn letters(img: &mut Pixmap, x0: u32, x1: u32, y0: u32, y1: u32) {
    let pitch = ((x1 - x0) * 3 / 4).max(4);
    let mut y = y0;
    while y + pitch / 2 < y1 {
        fill(img, x0, y, x1, y + pitch / 2, [20, 20, 20]);
        y += pitch;
    }
}

// ----- inspecting a design ---------------------------------------------------------------

/// The exact bounds of everything a list of items paints (images and paths; groups are
/// not produced by the crate).
pub fn painted_bounds(items: &[Item]) -> Vec<Rect> {
    items
        .iter()
        .map(|item| match item {
            Item::Path { path, .. } => outline_bounds(path).expect("a path with segments"),
            Item::Image { rect, .. } => *rect,
            other => panic!("unexpected item {other:?}"),
        })
        .collect()
}

/// The path and fill colour of item `index`.
pub fn filled_path(design: &SpineDesign, index: usize) -> (&Path, Color) {
    match &design.items[index] {
        Item::Path { path, fill: Some((color, _)), stroke: None } => (path, *color),
        other => panic!("item {index} is not a filled path: {other:?}"),
    }
}

/// The bounds of the first contour of a path — the first stroke of its first letter.
pub fn first_contour_bounds(path: &Path) -> Rect {
    let mut first = Path::new();
    for (i, seg) in path.segs.iter().enumerate() {
        if i > 0 && matches!(seg, PathSeg::MoveTo(_)) {
            break;
        }
        first.segs.push(*seg);
    }
    outline_bounds(&first).expect("a contour")
}

/// The bounds of the last contour of a path.
pub fn last_contour_bounds(path: &Path) -> Rect {
    let start = path
        .segs
        .iter()
        .rposition(|seg| matches!(seg, PathSeg::MoveTo(_)))
        .expect("a path starts with a move");
    let last = Path { segs: path.segs[start..].to_vec() };
    outline_bounds(&last).expect("a contour")
}

pub fn has_note(design: &SpineDesign, wanted: impl Fn(&ebc_spine::SpineNote) -> bool) -> bool {
    design.details.iter().any(wanted)
}
