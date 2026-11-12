//! What goes behind the title: a flat colour computed from the cover (the default), or —
//! only on explicit request — an image continuing the cover's binding edge.

use ebc_color::{
    Edge, Rgb8, UNIFORM_EDGE_THRESHOLD, contrast_ratio, edge_palette, edge_uniformity,
    has_uniform_border, palette,
};
use ebc_scene::{MM_PER_INCH, Pixmap};

use crate::BackgroundKind;
use crate::geometry::Geometry;
use crate::raster::{Grid, Raster, downscale};

/// Thickness of the strip examined along the binding edge, as a fraction of the cover
/// width (research note `spine-design.adaptation.md`, step S3: 1 % skipped + 3 %).
pub(crate) const EDGE_STRIP_FRACTION: f32 = 0.04;
/// Number of swatches looked at when the binding edge is busy (step S6: "the cover's
/// five largest swatches").
const PALETTE_SIZE: usize = 5;
/// A swatch must cover at least this share of the cover to be a candidate ground.
const MIN_SWATCH_SHARE: f32 = 0.10;

/// Standard deviation of the blur across the spine, in millimetres (step S6b).
const BLUR_ACROSS_MM: f64 = 2.5;
/// Standard deviation of the blur along the spine, in millimetres. The note proposes
/// 1 mm; 3 mm is used so that no shape of the artwork stays recognisable.
const BLUR_ALONG_MM: f64 = 3.0;
/// How far the continuation is pulled towards its mean colour (step S6b: 35 %).
const PULL_TO_MEAN: f32 = 0.35;
/// Size of the working grid of the continuation image.
const GRID_W: usize = 32;
const GRID_H: usize = 512;
/// Upper bound on the pixels of a generated image.
const MAX_IMAGE_PIXELS: f64 = 8.0e6;

/// The binding edge of a front cover (left-to-right books only; see the crate docs).
const BINDING_EDGE: Edge = Edge::Left;

/// The mean colour of the strip along the binding edge.
fn edge_mean(cover: &Pixmap) -> Option<Rgb8> {
    edge_palette(cover, BINDING_EDGE, EDGE_STRIP_FRACTION, 1).first().map(|s| s.color)
}

/// The swatch covering the largest share of the cover.
pub(crate) fn dominant(cover: &Pixmap) -> Rgb8 {
    palette(cover, PALETTE_SIZE).first().map_or(Rgb8::WHITE, |s| s.color)
}

/// The flat ground [`BackgroundStyle::Auto`](crate::BackgroundStyle::Auto) uses for a
/// cover. Never samples pixels onto the spine: the result is one colour.
///
/// 1. The strip along the binding edge is near-uniform → its mean colour.
/// 2. The cover has a uniform frame on all four sides → the frame's colour.
/// 3. Otherwise (busy edge) → among the cover's five largest swatches that cover at
///    least 10 % of it, the one nearest (ΔE\*ab) to the mean colour of the binding edge;
///    the largest swatch if none qualifies.
pub(crate) fn auto_flat(cover: &Pixmap) -> (Rgb8, BackgroundKind) {
    let mean = edge_mean(cover);
    if edge_uniformity(cover, BINDING_EDGE, EDGE_STRIP_FRACTION) >= UNIFORM_EDGE_THRESHOLD {
        if let Some(colour) = mean {
            return (colour, BackgroundKind::EdgeColour);
        }
    }
    if let Some(colour) = has_uniform_border(cover) {
        return (colour, BackgroundKind::BorderColour);
    }
    let swatches = palette(cover, PALETTE_SIZE);
    let largest = swatches.first().map_or(Rgb8::WHITE, |s| s.color);
    let Some(mean) = mean else { return (largest, BackgroundKind::DominantColour) };
    let target = mean.to_lab();
    let mut best: Option<(f64, Rgb8)> = None;
    for swatch in swatches.iter().filter(|s| s.share >= MIN_SWATCH_SHARE) {
        let distance = swatch.color.to_lab().delta_e76(target);
        // Strict comparison: on a tie the larger swatch (listed first) wins.
        if best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, swatch.color));
        }
    }
    (best.map_or(largest, |(_, colour)| colour), BackgroundKind::DominantColour)
}

/// White or black text for a flat ground: white when it reaches `min_ratio` (white on
/// mid-tones and darks reads better than black at the same WCAG ratio), else black when
/// it does, else whichever contrasts more.
pub(crate) fn auto_text_colour(ground: Rgb8, min_ratio: f64) -> Rgb8 {
    let white = contrast_ratio(Rgb8::WHITE, ground);
    let black = contrast_ratio(Rgb8::BLACK, ground);
    if white >= min_ratio {
        Rgb8::WHITE
    } else if black >= min_ratio || black > white {
        Rgb8::BLACK
    } else {
        Rgb8::WHITE
    }
}

/// A generated continuation of the cover's binding edge.
#[derive(Debug, Clone)]
pub(crate) struct Continuation {
    /// The pixels, covering the spine panel **including** the bleed at top and bottom.
    pub(crate) raster: Raster,
    /// Mean colour of the image.
    pub(crate) mean: Rgb8,
    /// Its darkest region.
    pub(crate) darkest: Rgb8,
    /// Its lightest region.
    pub(crate) lightest: Rgb8,
}

/// Builds the opt-in "edge continuation": the strip of the cover next to the binding
/// edge, as wide as the spine, mirrored about the fold, strongly blurred and pulled
/// towards its mean colour. The cover is assumed to fill the height of the panel.
pub(crate) fn continuation(cover: &Pixmap, geom: &Geometry, ppi: f64) -> Continuation {
    let px_per_mm = f64::from(cover.height()) / geom.trim.h;
    let strip = ((geom.width * px_per_mm).round().max(1.0) as u32).min(cover.width());
    let small = downscale(cover, 0, strip, GRID_W, GRID_H);
    let mut grid = Grid::mirrored(&small);
    grid.blur(
        BLUR_ACROSS_MM / geom.width * grid.w as f64,
        BLUR_ALONG_MM / geom.trim.h * grid.h as f64,
    );
    let mean = grid.mean();
    grid.pull_towards(mean, PULL_TO_MEAN);
    let (darkest, lightest) = grid.extremes();

    // Output size at the requested resolution, bounded.
    let mut px_per_mm = ppi / MM_PER_INCH;
    let wanted = geom.width * px_per_mm * geom.rect.h * px_per_mm;
    if wanted > MAX_IMAGE_PIXELS {
        px_per_mm *= (MAX_IMAGE_PIXELS / wanted).sqrt();
    }
    let out_w = (geom.width * px_per_mm).ceil().max(1.0) as usize;
    let out_h = (geom.trim.h * px_per_mm).ceil().max(1.0) as usize;
    let bleed = (geom.bleed * px_per_mm).ceil().max(0.0) as usize;
    let raster = grid.upscale(out_w, out_h).extend_rows(bleed, bleed);
    let mean = raster.mean();
    let rgb = |p: [u8; 3]| Rgb8::new(p[0], p[1], p[2]);
    Continuation { raster, mean, darkest: rgb(darkest), lightest: rgb(lightest) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_on_mid_tones_and_darks_black_on_lights() {
        assert_eq!(auto_text_colour(Rgb8::new(20, 30, 80), 4.5), Rgb8::WHITE);
        assert_eq!(auto_text_colour(Rgb8::new(250, 245, 230), 4.5), Rgb8::BLACK);
        // #767676: white reaches 4.54:1 and is preferred although black reaches 4.62:1.
        assert_eq!(auto_text_colour(Rgb8::new(0x76, 0x76, 0x76), 4.5), Rgb8::WHITE);
        assert_eq!(auto_text_colour(Rgb8::new(0x80, 0x80, 0x80), 4.5), Rgb8::BLACK);
        // An unreachable target: the better of the two.
        assert_eq!(auto_text_colour(Rgb8::new(0x80, 0x80, 0x80), 20.0), Rgb8::BLACK);
        assert_eq!(auto_text_colour(Rgb8::new(0x60, 0x60, 0x60), 20.0), Rgb8::WHITE);
        // Whatever the ground, the default threshold is met.
        for v in (0..=255).step_by(5) {
            let ground = Rgb8::new(v, v, 255 - v);
            let text = auto_text_colour(ground, 4.5);
            assert!(contrast_ratio(text, ground) >= 4.5, "{ground:?}");
        }
    }
}
