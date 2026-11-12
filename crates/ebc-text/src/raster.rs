//! Rasterising shaped text to an 8-bit coverage mask (feature `raster`).

use ebc_scene::{Path, PathSeg};

use crate::error::FontError;
use crate::font::Font;
use crate::guard::contain;
use crate::outline::Refusal;
use crate::shape::{ShapeOptions, append_outlines, shape_units};

/// Largest width or height of a [`Mask`], in pixels (before cropping to the ink). The
/// rasteriser works in 24.8 fixed point on 32-bit integers; this bound keeps every
/// intermediate product in range.
pub const MAX_MASK_SIDE: u32 = 8_192;
/// Largest number of pixels of a [`Mask`] (before cropping to the ink).
pub const MAX_MASK_PIXELS: u64 = 1 << 24;

/// An 8-bit coverage mask: `data[y * width + x]` is how much of pixel `(x, y)` is covered
/// by ink, from 0 (none) to 255 (all). Rows run top to bottom.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mask {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height` coverage values, row-major.
    pub data: Vec<u8>,
}

impl Mask {
    /// `true` if the mask has no pixels (nothing was drawn).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Coverage of pixel `(x, y)`; 0 outside the mask.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let index = y as usize * self.width as usize + x as usize;
        self.data.get(index).copied().unwrap_or(0)
    }

    /// One row of coverage values; empty if `y` is out of range.
    #[must_use]
    pub fn row(&self, y: u32) -> &[u8] {
        if y >= self.height {
            return &[];
        }
        let start = y as usize * self.width as usize;
        self.data.get(start..start + self.width as usize).unwrap_or(&[])
    }
}

/// A rasterised run of text: the [`Mask`], cropped to the ink, and where it sits
/// relative to the run origin (the start of the baseline).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RasterizedText {
    /// The coverage mask, cropped to the smallest pixel rectangle containing the ink.
    pub mask: Mask,
    /// Pixel column of the mask's left edge relative to the run origin (the run origin
    /// sits on a pixel corner). Usually a small positive number: the left side bearing.
    pub left: i32,
    /// Pixel row of the mask's top edge relative to the baseline, y down — negative
    /// for anything that rises above the baseline. The baseline lies between pixel rows
    /// `-top - 1` and `-top` of the mask.
    pub top: i32,
    /// Advance width of the run in pixels (what [`shape`](crate::shape) reports as
    /// `width`, at this resolution).
    pub advance: f32,
}

/// Shapes `text` and renders it, anti-aliased, at `px_per_em` pixels per em.
///
/// The result is cropped to the ink: its first and last rows and columns all contain
/// ink. Glyphs are placed exactly where shaping puts them (kerning, mark attachment,
/// letter spacing, sub-pixel positions); nothing is hinted or snapped, so the output is
/// deterministic across platforms.
///
/// Returns an empty mask when there is nothing to draw (empty or blank text) and also
/// when the request is invalid or too large — use [`try_rasterize`] to tell these apart
/// and to learn where the mask sits relative to the baseline.
#[must_use]
pub fn rasterize(font: &Font, text: &str, px_per_em: f32, opts: &ShapeOptions) -> Mask {
    try_rasterize(font, text, px_per_em, opts).map(|r| r.mask).unwrap_or_default()
}

/// Like [`rasterize`], but reports errors and returns the placement of the mask.
///
/// # Errors
/// * [`FontError::InvalidRasterSize`] — `px_per_em` is not a positive finite number;
/// * [`FontError::RasterTooLarge`] — the text would exceed [`MAX_MASK_SIDE`] pixels in
///   one direction or [`MAX_MASK_PIXELS`] in total;
/// * [`FontError::NonFiniteOutline`] — the font produced unusable coordinates;
/// * [`FontError::OutlineTooComplex`] — a glyph of the text is a "glyph bomb", or the
///   text as a whole is beyond the outline budget (see the crate documentation);
/// * [`FontError::Malformed`] — the font libraries gave up on the font data.
pub fn try_rasterize(
    font: &Font,
    text: &str,
    px_per_em: f32,
    opts: &ShapeOptions,
) -> Result<RasterizedText, FontError> {
    if !(px_per_em.is_finite() && px_per_em > 0.0) {
        return Err(FontError::InvalidRasterSize(px_per_em.to_string()));
    }
    contain(|| rasterize_run(font, text, px_per_em, opts)).unwrap_or(Err(FontError::Malformed))
}

fn rasterize_run(
    font: &Font,
    text: &str,
    px_per_em: f32,
    opts: &ShapeOptions,
) -> Result<RasterizedText, FontError> {
    let Some(face) = font.hb_face() else { return Ok(RasterizedText::default()) };
    let run = shape_units(&face, text, opts);
    let scale = f64::from(px_per_em) / f64::from(run.units_per_em.max(1));
    let advance = (run.width * scale) as f32;

    // Outlines in pixel coordinates relative to the run origin, y down.
    let mut path = Path::new();
    let Some(mut outliner) = font.outliner() else { return Ok(RasterizedText::default()) };
    let placed = run.glyphs.iter().map(|g| (g.id, (g.pen + g.dx) * scale, -g.dy * scale));
    append_outlines(&mut outliner, placed, scale, &mut path);
    if let Some(Refusal::GlyphTooComplex(glyph) | Refusal::BudgetExhausted(glyph)) =
        outliner.refusal()
    {
        return Err(FontError::OutlineTooComplex { glyph });
    }
    let empty = RasterizedText { mask: Mask::default(), left: 0, top: 0, advance };
    if path.is_empty() {
        return Ok(empty);
    }

    // Pixel-aligned frame around every on-curve and control point.
    let bounds = path.control_bounds().ok_or(FontError::NonFiniteOutline)?;
    let (x0, y0) = (bounds.x.floor(), bounds.y.floor());
    let (x1, y1) = (bounds.right().ceil(), bounds.bottom().ceil());
    if ![x0, y0, x1, y1].iter().all(|v| v.is_finite()) || !points_are_finite(&path) {
        return Err(FontError::NonFiniteOutline);
    }
    let (frame_w, frame_h) = ((x1 - x0).max(1.0), (y1 - y0).max(1.0));
    let side = f64::from(MAX_MASK_SIDE);
    if frame_w > side || frame_h > side || frame_w * frame_h > MAX_MASK_PIXELS as f64 {
        return Err(FontError::RasterTooLarge {
            width: frame_w.min(u64::MAX as f64) as u64,
            height: frame_h.min(u64::MAX as f64) as u64,
        });
    }
    // In range by the checks above (<= MAX_MASK_SIDE).
    let (width, height) = (frame_w as u32, frame_h as u32);

    // Every coordinate handed to the rasteriser now lies in 0..=MAX_MASK_SIDE.
    let commands: Vec<zeno::Command> = path
        .segs
        .iter()
        .map(|seg| {
            let v = |p: ebc_scene::Point| zeno::Vector::new((p.x - x0) as f32, (p.y - y0) as f32);
            match *seg {
                PathSeg::MoveTo(p) => zeno::Command::MoveTo(v(p)),
                PathSeg::LineTo(p) => zeno::Command::LineTo(v(p)),
                PathSeg::QuadTo(c, p) => zeno::Command::QuadTo(v(c), v(p)),
                PathSeg::CubicTo(c1, c2, p) => zeno::Command::CurveTo(v(c1), v(c2), v(p)),
                PathSeg::Close => zeno::Command::Close,
            }
        })
        .collect();
    let mut frame = vec![0u8; width as usize * height as usize];
    zeno::Mask::new(commands.as_slice())
        .style(zeno::Fill::NonZero)
        .size(width, height)
        .render_into(&mut frame, None);

    let Some((crop_x, crop_y, crop_w, crop_h)) = ink_box(&frame, width, height) else {
        return Ok(empty);
    };
    let mut data = Vec::with_capacity(crop_w as usize * crop_h as usize);
    for row in crop_y..crop_y + crop_h {
        let start = row as usize * width as usize + crop_x as usize;
        data.extend_from_slice(&frame[start..start + crop_w as usize]);
    }
    Ok(RasterizedText {
        mask: Mask { width: crop_w, height: crop_h, data },
        // |x0|, |y0| can be large only if the frame is far from the origin, which the
        // saturating float-to-int conversion handles.
        left: (x0 as i64 + i64::from(crop_x)).clamp(i64::from(i32::MIN), i64::from(i32::MAX))
            as i32,
        top: (y0 as i64 + i64::from(crop_y)).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        advance,
    })
}

fn points_are_finite(path: &Path) -> bool {
    let ok = |p: &ebc_scene::Point| p.x.is_finite() && p.y.is_finite();
    path.segs.iter().all(|seg| match seg {
        PathSeg::MoveTo(p) | PathSeg::LineTo(p) => ok(p),
        PathSeg::QuadTo(c, p) => ok(c) && ok(p),
        PathSeg::CubicTo(c1, c2, p) => ok(c1) && ok(c2) && ok(p),
        PathSeg::Close => true,
    })
}

/// The smallest rectangle `(x, y, w, h)` containing every non-zero pixel.
fn ink_box(frame: &[u8], width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let w = width as usize;
    let row_has_ink = |y: u32| frame[y as usize * w..(y as usize + 1) * w].iter().any(|&v| v != 0);
    let top = (0..height).find(|&y| row_has_ink(y))?;
    let bottom = (top..height).rev().find(|&y| row_has_ink(y))?;
    let (mut left, mut right) = (width, 0);
    for y in top..=bottom {
        let row = &frame[y as usize * w..(y as usize + 1) * w];
        if let Some(first) = row.iter().position(|&v| v != 0) {
            left = left.min(first as u32);
        }
        if let Some(last) = row.iter().rposition(|&v| v != 0) {
            right = right.max(last as u32);
        }
    }
    (left <= right).then(|| (left, top, right - left + 1, bottom - top + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ink_box_finds_the_tight_rectangle() {
        #[rustfmt::skip]
        let frame = [
            0, 0, 0, 0, 0,
            0, 0, 9, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 7, 0,
            0, 0, 0, 0, 0,
        ];
        assert_eq!(ink_box(&frame, 5, 5), Some((1, 1, 3, 3)));
        assert_eq!(ink_box(&[0; 6], 3, 2), None);
        assert_eq!(ink_box(&[5], 1, 1), Some((0, 0, 1, 1)));
        assert_eq!(ink_box(&[], 0, 0), None);
    }

    #[test]
    fn mask_accessors_are_total() {
        let mask = Mask { width: 2, height: 2, data: vec![1, 2, 3, 4] };
        assert_eq!(mask.get(1, 0), 2);
        assert_eq!(mask.get(0, 1), 3);
        assert_eq!(mask.get(2, 0), 0);
        assert_eq!(mask.get(0, 2), 0);
        assert_eq!(mask.row(1), [3, 4]);
        assert!(mask.row(2).is_empty());
        assert!(!mask.is_empty());
        // An inconsistent, hand-built mask must not panic either.
        let broken = Mask { width: 4, height: 4, data: vec![1] };
        assert_eq!(broken.get(3, 3), 0);
        assert!(broken.row(3).is_empty());
        assert!(Mask::default().is_empty());
    }
}
