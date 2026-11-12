//! Adapting a provided spine image (or the spine columns of a full wraparound cover) to
//! the spine of the card.
//!
//! Decision ladder — `w'` is the width of the image once it is scaled **uniformly** so
//! that its height equals the trimmed height of the spine, `W` the width of the spine
//! (10 mm by default; the figures in brackets are for `W` = 10 mm):
//!
//! | # | Condition | Outcome |
//! |---|---|---|
//! | 0 | fewer than `min_provided_ppi` pixels per inch along the spine | incompatible |
//! | 0 | no text found on the image | incompatible |
//! | 0 | `w'` < 0.5 `W` (< 5 mm) | incompatible: much too narrow |
//! | 1 | 0.92 `W` ≤ `w'` ≤ 1.08 `W` (9.2–10.8 mm) | **stretched** across by at most 8 % |
//! | 2 | 0.5 `W` ≤ `w'` < 0.92 `W`, both long sides plain | **padded** on both sides with the side colours |
//! | 2 | same, sides not plain | incompatible |
//! | 3 | `w'` > 1.08 `W`, sides plain and content ≤ `W` − 2 × safe margin | **cropped**, centred on the content |
//! | 4 | 1.08 `W` < `w'` ≤ 1.3 `W` (≤ 13 mm) otherwise | **shrunk** uniformly to the width; head and foot filled with the image's end colours |
//! | 5 | `w'` > 1.3 `W` otherwise | incompatible: too wide |
//!
//! After 1–4 the text must still be legible: the widest band of text across the spine
//! must be at least [`MIN_TEXT_ACROSS_MM`] at final size, else the image is
//! incompatible. "Incompatible" means the caller typesets a spine instead.
//!
//! The thresholds come from `docs/research/notes/spine-design.adaptation.md` §4, where
//! they are marked as engineering defaults checked on synthetic images only. The text
//! detector is a heuristic (ink share per column, then dark/light transitions along the
//! spine); it is not a text recogniser.

use ebc_color::Rgb8;
use ebc_scene::{MM_PER_INCH, Pixmap, Rect};

use crate::geometry::Geometry;
use crate::raster::{Raster, downscale, rgb_distance};
use crate::{Adaptation, Incompatibility};

/// Stretch across the spine tolerated without the type looking condensed or expanded.
const MAX_STRETCH: f64 = 0.08;
/// Narrowest scaled spine that is adapted (relative to the spine width).
const MIN_RELATIVE_WIDTH: f64 = 0.5;
/// Widest scaled spine that is shrunk to fit (relative to the spine width).
const MAX_SHRINK_RELATIVE_WIDTH: f64 = 1.3;
/// Smallest extent across the spine of the widest text band, at final size, for the
/// text to count as legible: the capital height of 7 pt type.
pub(crate) const MIN_TEXT_ACROSS_MM: f64 = 1.7;

/// Working size of the analysis raster.
const ANALYSIS_W: usize = 256;
const ANALYSIS_H: usize = 1024;
/// Largest raster registered in the document.
const MAX_OUTPUT_W: usize = 4096;
const MAX_OUTPUT_H: usize = 16384;

/// Sum of absolute channel differences below which a pixel is "the background colour"
/// (absorbs JPEG noise).
const BACKGROUND_TOLERANCE: u32 = 36;
/// Sum of absolute channel differences above which a pixel is "ink".
const INK_DISTANCE: u32 = 90;
/// Share of a column that must be background for the column to be background.
const BACKGROUND_COVERAGE: f64 = 0.98;
/// Share of a column that must be ink for the column to belong to a text band.
const TEXT_COLUMN_INK: f64 = 0.01;
/// Dark/light transitions along the spine that a text band must show (three strokes).
const MIN_TRANSITIONS: usize = 6;

/// What the image looks like, measured on a reduced copy.
#[derive(Debug, Clone)]
struct Analysis {
    /// Width of the reduced copy, in its pixels.
    width: usize,
    /// Colour of the left long side if it is plain.
    left: Option<[u8; 3]>,
    /// Colour of the right long side if it is plain.
    right: Option<[u8; 3]>,
    /// Columns `start..end` that hold something other than background; `None` when
    /// the whole image is background.
    content: Option<(usize, usize)>,
    /// Width, in columns, of the widest band of text; `None` when no text was found.
    text_band: Option<usize>,
}

/// Per-channel median of a set of pixels.
fn median(pixels: &[[u8; 3]]) -> [u8; 3] {
    let mut out = [0u8; 3];
    let mut channel: Vec<u8> = Vec::with_capacity(pixels.len());
    for (c, slot) in out.iter_mut().enumerate() {
        channel.clear();
        channel.extend(pixels.iter().map(|p| p[c]));
        channel.sort_unstable();
        *slot = channel.get(channel.len() / 2).copied().unwrap_or(255);
    }
    out
}

fn column(raster: &Raster, x: usize) -> Vec<[u8; 3]> {
    (0..raster.h).map(|y| raster.px(x, y)).collect()
}

/// Share of the column `x` within the background tolerance of `colour`.
fn coverage(raster: &Raster, x: usize, colour: [u8; 3]) -> f64 {
    let hits =
        (0..raster.h).filter(|&y| rgb_distance(raster.px(x, y), colour) <= BACKGROUND_TOLERANCE);
    hits.count() as f64 / raster.h as f64
}

/// The colour of column `x` if the column is plain.
fn plain_column(raster: &Raster, x: usize) -> Option<[u8; 3]> {
    let colour = median(&column(raster, x));
    (coverage(raster, x, colour) >= BACKGROUND_COVERAGE).then_some(colour)
}

fn analyse(small: &Raster) -> Analysis {
    let w = small.w;
    let left = plain_column(small, 0);
    let right = plain_column(small, w - 1);

    // Background margins, scanned inwards from each plain side.
    let content = match (left, right) {
        (Some(l), Some(r)) => {
            let start = (0..w).find(|&x| coverage(small, x, l) < BACKGROUND_COVERAGE);
            match start {
                None => None,
                Some(start) => {
                    let end = (start..w)
                        .rev()
                        .find(|&x| coverage(small, x, r) < BACKGROUND_COVERAGE)
                        .map_or(start + 1, |x| x + 1);
                    Some((start, end))
                }
            }
        }
        _ => Some((0, w)),
    };

    // Ink is whatever differs clearly from the background colours (the side colours,
    // or the overall median colour when the sides are not plain).
    let overall = {
        let step = (small.w * small.h / 4096).max(1);
        let sample: Vec<[u8; 3]> = (0..small.w * small.h)
            .step_by(step)
            .map(|i| small.px(i % small.w, i / small.w))
            .collect();
        median(&sample)
    };
    let grounds = [left.unwrap_or(overall), right.unwrap_or(overall)];
    let is_ink = |x: usize, y: usize| {
        let p = small.px(x, y);
        grounds.iter().all(|&g| rgb_distance(p, g) > INK_DISTANCE)
    };
    let ink_share =
        |x: usize| (0..small.h).filter(|&y| is_ink(x, y)).count() as f64 / small.h as f64;
    let transitions = |x: usize| {
        let mut count = 0usize;
        let mut previous = is_ink(x, 0);
        for y in 1..small.h {
            let current = is_ink(x, y);
            if current != previous {
                count += 1;
            }
            previous = current;
        }
        count
    };

    let mut text_band: Option<usize> = None;
    let mut run_start: Option<usize> = None;
    for x in 0..=w {
        let texty = x < w && ink_share(x) >= TEXT_COLUMN_INK;
        match (run_start, texty) {
            (None, true) => run_start = Some(x),
            (Some(start), false) => {
                run_start = None;
                let len = x - start;
                let probes = [start + len / 4, start + len / 2, start + (3 * len) / 4];
                let busiest = probes.iter().map(|&p| transitions(p.min(w - 1))).max().unwrap_or(0);
                if busiest >= MIN_TRANSITIONS && text_band.is_none_or(|best| len > best) {
                    text_band = Some(len);
                }
            }
            _ => {}
        }
    }

    Analysis { width: w, left, right, content, text_band }
}

/// A flat fill that goes under the image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Fill {
    pub(crate) rect: Rect,
    pub(crate) colour: Rgb8,
}

/// A provided spine fitted to the panel.
#[derive(Debug, Clone)]
pub(crate) struct Adapted {
    /// Flat fills, painted first, in order. Together with the image they cover the
    /// whole panel including the bleed.
    pub(crate) fills: Vec<Fill>,
    /// The pixels to register in the document.
    pub(crate) raster: Raster,
    /// Where the image goes.
    pub(crate) image_rect: Rect,
    /// Representative background colour.
    pub(crate) background: Rgb8,
    /// What was done to the image.
    pub(crate) how: Adaptation,
}

/// The result of trying to adapt a provided spine.
#[derive(Debug, Clone)]
pub(crate) enum Outcome {
    /// The image can be used.
    Adapted(Adapted),
    /// It cannot; `colour` is its plain background colour, if it has one, to be reused
    /// as the ground of the typeset spine.
    Incompatible { reason: Incompatibility, colour: Option<Rgb8> },
}

fn rgb(p: [u8; 3]) -> Rgb8 {
    Rgb8::new(p[0], p[1], p[2])
}

/// Tries to fit the columns `x0..x1` of `img` (a spine at trim height, without bleed)
/// to the panel described by `geom`. See the module documentation for the rules.
pub(crate) fn adapt(
    img: &Pixmap,
    (x0, x1): (u32, u32),
    geom: &Geometry,
    safe_margin: f64,
    min_ppi: f64,
) -> Outcome {
    let source_w = f64::from(x1 - x0);
    let source_h = f64::from(img.height());
    let mm_per_px = geom.trim.h / source_h;
    let scaled_width = source_w * mm_per_px;
    let relative = scaled_width / geom.width;

    let small = downscale(img, x0, x1, ANALYSIS_W, ANALYSIS_H);
    let analysis = analyse(&small);
    let colour = match (analysis.left, analysis.right) {
        (Some(l), Some(_)) => Some(rgb(l)),
        _ => None,
    };
    let incompatible = |reason: Incompatibility| Outcome::Incompatible { reason, colour };

    let ppi = source_h / (geom.trim.h / MM_PER_INCH);
    if ppi < min_ppi {
        return incompatible(Incompatibility::LowResolution { ppi });
    }
    let Some(text_band) = analysis.text_band else {
        return incompatible(Incompatibility::NoText);
    };
    if relative < MIN_RELATIVE_WIDTH {
        return incompatible(Incompatibility::TooNarrow { width_mm: scaled_width });
    }
    let sides_plain = analysis.left.is_some() && analysis.right.is_some();
    // Extents measured on the reduced copy, as fractions of the source width.
    let fraction = |columns: usize| columns as f64 / analysis.width as f64;

    // (columns of the source to keep, where the image goes, what was done, the width in
    // millimetres that the full source width takes at final size)
    let trim = geom.trim;
    let (keep, image_rect, how, final_source_width) = if (relative - 1.0).abs() <= MAX_STRETCH {
        let how = Adaptation::Stretched { percent: (1.0 / relative - 1.0) * 100.0 };
        ((x0, x1), Rect::new(trim.x, trim.y, trim.w, trim.h), how, geom.width)
    } else if relative < 1.0 {
        if !sides_plain {
            return incompatible(Incompatibility::BusySides { width_mm: scaled_width });
        }
        let rect = Rect::new(geom.center_x - scaled_width / 2.0, trim.y, scaled_width, trim.h);
        let how = Adaptation::Padded { added_mm: geom.width - scaled_width };
        ((x0, x1), rect, how, scaled_width)
    } else {
        let content_mm = analysis.content.map_or(0.0, |(s, e)| fraction(e - s) * scaled_width);
        let room = geom.width - 2.0 * safe_margin;
        if sides_plain && content_mm <= room {
            // Keep `W` millimetres of the source, centred on the content.
            let keep_px = (geom.width / mm_per_px).round().clamp(1.0, source_w);
            let centre = analysis
                .content
                .map_or(source_w / 2.0, |(s, e)| fraction(s + e) / 2.0 * source_w);
            let start = (centre - keep_px / 2.0).round().clamp(0.0, source_w - keep_px);
            let first = x0 + start as u32;
            let last = (first + keep_px as u32).min(x1);
            let how = Adaptation::Cropped { removed_mm: scaled_width - geom.width };
            ((first, last), Rect::new(trim.x, trim.y, trim.w, trim.h), how, scaled_width)
        } else if relative <= MAX_SHRINK_RELATIVE_WIDTH {
            let height = trim.h / relative;
            let rect = Rect::new(trim.x, trim.y + (trim.h - height) / 2.0, trim.w, height);
            let how = Adaptation::Shrunk { percent: (1.0 - 1.0 / relative) * 100.0 };
            ((x0, x1), rect, how, geom.width)
        } else {
            return incompatible(Incompatibility::TooWide { width_mm: scaled_width });
        }
    };

    let text_mm = fraction(text_band) * final_source_width;
    if text_mm < MIN_TEXT_ACROSS_MM {
        return incompatible(Incompatibility::TextTooSmall { height_mm: text_mm });
    }

    let raster = downscale(img, keep.0, keep.1, MAX_OUTPUT_W, MAX_OUTPUT_H);
    let end_rows = (raster.h / 100).max(1);
    let head = raster.rows_mean(0, end_rows);
    let foot = raster.rows_mean(raster.h - end_rows, raster.h);
    let background = colour.unwrap_or_else(|| raster.mean());

    // Head and foot colours first, each over its half of the panel (bleed included);
    // then, for a padded image, the side colours over the trimmed height. The image
    // goes on top, so no seam can show between two fills.
    let panel = geom.rect;
    let middle = trim.y + trim.h / 2.0;
    let mut fills = vec![
        Fill { rect: Rect::new(panel.x, panel.y, panel.w, middle - panel.y), colour: head },
        Fill { rect: Rect::new(panel.x, middle, panel.w, panel.bottom() - middle), colour: foot },
    ];
    if let (Adaptation::Padded { .. }, Some(l), Some(r)) = (how, analysis.left, analysis.right) {
        let half = panel.w / 2.0;
        fills.push(Fill { rect: Rect::new(panel.x, trim.y, half, trim.h), colour: rgb(l) });
        fills.push(Fill { rect: Rect::new(geom.center_x, trim.y, half, trim.h), colour: rgb(r) });
    }

    Outcome::Adapted(Adapted { fills, raster, image_rect, background, how })
}
