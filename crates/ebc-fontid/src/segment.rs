//! Analysis of one text region: ink extraction, deskewing, glyph segmentation.

use ebc_scene::Pixmap;

use crate::TextRegion;
use crate::cc::{Comp, label_components, local_contrast_masks, median, theil_sen_slope};
use crate::descriptor::remap_coverage;
use crate::gray::{Channel, Gray, Integral, noise_sigma, otsu};

/// Largest height of the analysed crop, in pixels (larger crops are scaled down).
const MAX_CROP_H: usize = 480;
/// Largest width of the analysed crop, in pixels.
const MAX_CROP_W: usize = 2400;
/// Largest number of connected components considered in a region.
const MAX_COMPONENTS: usize = 4000;

/// A glyph: one connected component, or a base component with its detached marks
/// (accents, the dot of an i, the dot of an exclamation mark).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Glyph {
    /// Indices into [`Analysis::comps`] of all parts.
    pub(crate) parts: Vec<u32>,
    /// Index of the largest part (the base letter).
    pub(crate) main: u32,
    /// Letter-sized (as opposed to small punctuation).
    pub(crate) tall: bool,
    /// Union box of all parts: left, top (inclusive), right, bottom (exclusive).
    pub(crate) x0: u32,
    pub(crate) y0: u32,
    pub(crate) x1: u32,
    pub(crate) y1: u32,
}

/// A line of text: glyphs from left to right.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Line {
    pub(crate) glyphs: Vec<Glyph>,
}

impl Line {
    /// Number of letter-sized glyphs.
    pub(crate) fn tall_count(&self) -> usize {
        self.glyphs.iter().filter(|g| g.tall).count()
    }

    /// All component indices of the line.
    pub(crate) fn all_parts(&self) -> Vec<u32> {
        self.glyphs.iter().flat_map(|g| g.parts.iter().copied()).collect()
    }
}

/// The result of analysing a region.
#[derive(Debug, Clone)]
pub(crate) struct Analysis {
    /// Ink coverage (255 = ink), deskewed and at the analysis scale.
    pub(crate) soft: Gray,
    /// Component label per pixel of `soft` (0 = none, `i + 1` = `comps[i]`).
    pub(crate) labels: Vec<u32>,
    /// Connected components of the binarised coverage.
    pub(crate) comps: Vec<Comp>,
    /// Text lines, top to bottom.
    pub(crate) lines: Vec<Line>,
    /// Rotation that was removed, in degrees (positive = text ran downhill to the right).
    pub(crate) skew_degrees: f32,
    /// `true` when the text is lighter than its background.
    pub(crate) light_on_dark: bool,
}

impl Analysis {
    /// Number of letter-sized glyphs in all lines.
    pub(crate) fn tall_count(&self) -> usize {
        self.lines.iter().map(Line::tall_count).sum()
    }

    /// Cuts the given components out of the coverage map: the union box of the
    /// components, with every pixel that belongs to another component (or is farther
    /// than one pixel from these) cleared. Returns the crop and its box
    /// `(x0, y0, x1, y1)`.
    pub(crate) fn crop_parts(&self, parts: &[u32]) -> Option<(Gray, (u32, u32, u32, u32))> {
        let mut bbox: Option<(u32, u32, u32, u32)> = None;
        for &p in parts {
            let c = self.comps.get(p as usize)?;
            bbox = Some(match bbox {
                None => (c.x0, c.y0, c.x1, c.y1),
                Some((x0, y0, x1, y1)) => (x0.min(c.x0), y0.min(c.y0), x1.max(c.x1), y1.max(c.y1)),
            });
        }
        let (x0, y0, x1, y1) = bbox?;
        let (w, h) = (self.soft.w, self.soft.h);
        let own = |label: u32| label != 0 && parts.contains(&(label - 1));
        let (cw, ch) = ((x1 - x0) as usize, (y1 - y0) as usize);
        let mut px = vec![0u8; cw * ch];
        for y in y0 as usize..y1 as usize {
            for x in x0 as usize..x1 as usize {
                let label = self.labels[y * w + x];
                let keep = if label != 0 {
                    own(label)
                } else {
                    (y.saturating_sub(1)..=(y + 1).min(h - 1)).any(|ny| {
                        (x.saturating_sub(1)..=(x + 1).min(w - 1))
                            .any(|nx| own(self.labels[ny * w + nx]))
                    })
                };
                if keep {
                    px[(y - y0 as usize) * cw + (x - x0 as usize)] = self.soft.px[y * w + x];
                }
            }
        }
        Some((Gray { w: cw, h: ch, px }, (x0, y0, x1, y1)))
    }
}

/// Analyses `region` of `img`. `None` when the region is empty, lies outside the image
/// or contains nothing that looks like text.
pub(crate) fn analyse(img: &Pixmap, region: &TextRegion) -> Option<Analysis> {
    let (iw, ih) = (img.width(), img.height());
    let (rx0, ry0) = (region.x.min(iw), region.y.min(ih));
    let rx1 = region.x.saturating_add(region.w).min(iw);
    let ry1 = region.y.saturating_add(region.h).min(ih);
    if rx1 <= rx0 || ry1 <= ry0 {
        return None;
    }
    // A margin gives the background estimate something to hold on to and keeps letters
    // of a tightly drawn box from touching the border.
    let margin = ((ry1 - ry0) as f32 * 0.15).round().clamp(2.0, 48.0) as u32;
    let (cx0, cy0) = (rx0.saturating_sub(margin), ry0.saturating_sub(margin));
    let (cx1, cy1) = (rx1.saturating_add(margin).min(iw), ry1.saturating_add(margin).min(ih));
    let rect = (cx0, cy0, cx1 - cx0, cy1 - cy0);

    // Huge regions are read block-averaged, so that no full-size buffer is ever built.
    let k = (rect.3 / MAX_CROP_H as u32).max(rect.2 / MAX_CROP_W as u32).max(1);
    let read = |channel| Gray::from_pixmap_rect_reduced(img, rect.0, rect.1, rect.2, rect.3, channel, k);
    let mut gray = read(Channel::Luma)?;
    // Coloured text on a coloured ground of the same lightness: use the colour channel
    // that separates best instead.
    if class_separation(&gray) < 40.0 {
        let mut best = class_separation(&gray);
        for channel in [Channel::Red, Channel::Green, Channel::Blue] {
            if let Some(candidate) = read(channel) {
                let sep = class_separation(&candidate);
                if sep > best + 8.0 {
                    best = sep;
                    gray = candidate;
                }
            }
        }
    }
    let shrink = (MAX_CROP_H as f32 / gray.h as f32).min(MAX_CROP_W as f32 / gray.w as f32);
    if shrink < 1.0 {
        let nw = ((gray.w as f32 * shrink).round() as usize).max(1);
        let nh = ((gray.h as f32 * shrink).round() as usize).max(1);
        gray = gray.resize(nw, nh);
    }

    let first = segment(&gray, None)?;
    let tall_heights: Vec<f32> = first
        .lines
        .iter()
        .flat_map(|l| l.glyphs.iter())
        .filter(|g| g.tall)
        .map(|g| first.comps[g.main as usize].h() as f32)
        .collect();
    if tall_heights.is_empty() {
        return None;
    }
    let med_h = median(&mut tall_heights.clone());
    let skew = estimate_skew(&first);

    // Second pass on a deskewed crop at a comfortable glyph size.
    let scale = if med_h < 30.0 {
        (40.0 / med_h).min(3.0)
    } else if med_h > 90.0 {
        72.0 / med_h
    } else {
        1.0
    };
    if scale == 1.0 && skew == 0.0 {
        return Some(first);
    }
    let mut second = gray;
    if scale != 1.0 {
        let nw = ((second.w as f32 * scale).round() as usize).clamp(1, MAX_CROP_W * 2);
        let nh = ((second.h as f32 * scale).round() as usize).clamp(1, MAX_CROP_H * 3);
        second = second.resize(nw, nh);
    }
    if skew != 0.0 {
        second = second.rotated(-skew.to_radians());
    }
    match segment(&second, Some(first.light_on_dark)) {
        Some(mut a) if a.tall_count() > 0 => {
            a.skew_degrees = skew;
            Some(a)
        }
        _ => Some(first),
    }
}

/// Distance between the means of the two Otsu classes of an image (0 for a flat image).
fn class_separation(g: &Gray) -> f32 {
    let mut hist = [0u32; 256];
    for &v in &g.px {
        hist[usize::from(v)] += 1;
    }
    otsu(&hist).map_or(0.0, |(_, dark, light)| light - dark)
}

/// Skew of the longest line in degrees, from a robust fit through the bottoms of its
/// letters. 0 when it cannot be measured or is negligible.
fn estimate_skew(a: &Analysis) -> f32 {
    let Some(line) = a.lines.iter().max_by_key(|l| l.tall_count()) else { return 0.0 };
    let points: Vec<(f32, f32)> = line
        .glyphs
        .iter()
        .filter(|g| g.tall)
        .map(|g| {
            let c = &a.comps[g.main as usize];
            (c.cx(), c.y1 as f32)
        })
        .collect();
    if points.len() < 4 {
        return 0.0;
    }
    let extent = points.iter().map(|p| p.0).fold(f32::MIN, f32::max)
        - points.iter().map(|p| p.0).fold(f32::MAX, f32::min);
    let mut heights: Vec<f32> =
        line.glyphs.iter().filter(|g| g.tall).map(|g| (g.y1 - g.y0) as f32).collect();
    if extent < 3.0 * median(&mut heights) {
        return 0.0;
    }
    let degrees = theil_sen_slope(&points).atan().to_degrees();
    if degrees.abs() < 0.4 || degrees.abs() > 10.0 { 0.0 } else { degrees }
}

/// One segmentation pass over a grey crop.
fn segment(gray: &Gray, polarity: Option<bool>) -> Option<Analysis> {
    let (w, h) = (gray.w, gray.h);
    if w < 4 || h < 4 {
        return None;
    }
    let c_min = (3.5 * noise_sigma(gray)).clamp(12.0, 48.0);
    let window = (w.min(h) | 1).max(15);
    let (dark, light) = local_contrast_masks(gray, window, 0.3, c_min);
    let light_on_dark = match polarity {
        Some(p) => p,
        None => choose_polarity(gray, &dark, &light)?,
    };
    // From here on ink is dark.
    let oriented = if light_on_dark { gray.inverted() } else { gray.clone() };
    let mask = if light_on_dark { light } else { dark };
    let soft = refine(&oriented, &mask, window)?;

    let binary: Vec<u8> = soft.px.iter().map(|&v| u8::from(v >= 128)).collect();
    let (labels, comps) = label_components(&binary, w, h);
    if comps.is_empty() || comps.len() > MAX_COMPONENTS {
        return None;
    }
    let lines = build_lines(&comps, w, h);
    if lines.is_empty() {
        return None;
    }
    Some(Analysis { soft, labels, comps, lines, skew_degrees: 0.0, light_on_dark })
}

/// Decides whether the text is light on dark. The ink is the class whose grey level is
/// far from the level of the crop's border; when the border is inconclusive, the
/// polarity whose components look more like letters wins.
fn choose_polarity(gray: &Gray, dark: &[u8], light: &[u8]) -> Option<bool> {
    let (w, h) = (gray.w, gray.h);
    let mut border: Vec<f32> = Vec::with_capacity(2 * (w + h));
    for x in 0..w {
        border.push(f32::from(gray.px[x]));
        border.push(f32::from(gray.px[(h - 1) * w + x]));
    }
    for y in 0..h {
        border.push(f32::from(gray.px[y * w]));
        border.push(f32::from(gray.px[y * w + w - 1]));
    }
    let border_level = median(&mut border);
    let mean_of = |mask: &[u8]| -> Option<f32> {
        let (mut sum, mut n) = (0u64, 0u64);
        for (&m, &v) in mask.iter().zip(&gray.px) {
            if m != 0 {
                sum += u64::from(v);
                n += 1;
            }
        }
        (n > 0).then(|| sum as f32 / n as f32)
    };
    let d_dark = mean_of(dark).map(|m| (m - border_level).abs());
    let d_light = mean_of(light).map(|m| (m - border_level).abs());
    match (d_dark, d_light) {
        (None, None) => None,
        (Some(_), None) => Some(false),
        (None, Some(_)) => Some(true),
        (Some(dd), Some(dl)) => {
            if dd.max(dl) < 6.0 {
                None
            } else if dd > 1.5 * dl {
                Some(false)
            } else if dl > 1.5 * dd {
                Some(true)
            } else {
                let score = |mask: &[u8]| -> f32 {
                    let (_, comps) = label_components(mask, w, h);
                    letter_like(&comps, w, h)
                        .iter()
                        .map(|&i| (comps[i].w() * comps[i].h()) as f32)
                        .sum()
                };
                Some(score(light) > score(dark))
            }
        }
    }
}

/// Turns a first ink mask into a coverage map: the background level is estimated
/// locally from non-ink pixels, the ink level from the cores of the strokes, and every
/// pixel is placed between the two. `g` must have dark ink.
fn refine(g: &Gray, mask: &[u8], window: usize) -> Option<Gray> {
    let (w, h) = (g.w, g.h);
    let at = |x: usize, y: usize| mask[y * w + x] != 0;
    let mut background = vec![0u8; w * h];
    let mut core = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let (xa, xb) = (x.saturating_sub(1), (x + 1).min(w - 1));
            let (ya, yb) = (y.saturating_sub(1), (y + 1).min(h - 1));
            let near_ink = (ya..=yb).any(|ny| (xa..=xb).any(|nx| at(nx, ny)));
            background[y * w + x] = u8::from(!near_ink);
            let inside = at(x, y) && at(xa, y) && at(xb, y) && at(x, ya) && at(x, yb);
            core[y * w + x] = u8::from(inside);
        }
    }
    let bg = Integral::new(g, Some(&background));
    let ink = Integral::new(g, Some(&core));
    let far = w.max(h);
    let (n_bg, global_bg, _) = bg.window(w / 2, h / 2, far);
    if n_bg < 1.0 {
        return None;
    }
    let (n_core, core_mean, _) = ink.window(w / 2, h / 2, far);
    let global_ink = if n_core >= 20.0 {
        core_mean
    } else {
        // Hairline strokes have no core: take the darkest third of the ink pixels.
        let mut values: Vec<u8> =
            mask.iter().zip(&g.px).filter(|(m, _)| **m != 0).map(|(_, &v)| v).collect();
        if values.is_empty() {
            return None;
        }
        values.sort_unstable();
        let n = (values.len() / 3).max(1);
        values[..n].iter().map(|&v| f32::from(v)).sum::<f32>() / n as f32
    };

    let r = (window / 2).max(1);
    let min_bg = ((window * window) as f32 * 0.02).max(8.0);
    let mut px = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let (nb, mb, _) = bg.window(x, y, r);
            let level_bg = if nb >= min_bg { mb } else { global_bg };
            let (ni, mi, _) = ink.window(x, y, r);
            let level_ink = if ni >= 30.0 { mi } else { global_ink };
            let range = level_bg - level_ink;
            if range < 6.0 {
                continue;
            }
            let v = (level_bg - f32::from(g.px[y * w + x])) / range;
            px[y * w + x] = remap_coverage((v * 255.0 + 0.5).clamp(0.0, 255.0) as u8);
        }
    }
    Some(Gray { w, h, px })
}

/// Indices of the components that could be letters: clear of the border, not a rule or
/// a frame, and at least 0.42 of the height of the typical tall component.
pub(crate) fn letter_like(comps: &[Comp], w: usize, h: usize) -> Vec<usize> {
    let usable = usable_components(comps, w, h);
    let Some(reference) = reference_height(comps, &usable) else { return Vec::new() };
    usable.into_iter().filter(|&i| comps[i].h() as f32 >= 0.42 * reference).collect()
}

/// Components that are neither specks, nor touching the border, nor rules/frames.
fn usable_components(comps: &[Comp], w: usize, h: usize) -> Vec<usize> {
    comps
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            let touches = c.x0 == 0 || c.y0 == 0 || c.x1 as usize >= w || c.y1 as usize >= h;
            let (cw, ch) = (c.w() as f32, c.h() as f32);
            let rule = (cw > 0.6 * w as f32 && cw > 10.0 * ch)
                || (ch > 0.9 * h as f32 && ch > 10.0 * cw)
                || cw > 8.0 * ch.max(4.0);
            !touches && !rule && c.area >= 3 && c.w().max(c.h()) >= 2
        })
        .map(|(i, _)| i)
        .collect()
}

/// The typical height of tall components: the median of the taller half of all usable
/// components that are at least 5 pixels in one dimension.
fn reference_height(comps: &[Comp], usable: &[usize]) -> Option<f32> {
    let mut heights: Vec<f32> = usable
        .iter()
        .map(|&i| &comps[i])
        .filter(|c| c.w().max(c.h()) >= 5)
        .map(|c| c.h() as f32)
        .collect();
    if heights.is_empty() {
        return None;
    }
    heights.sort_by(f32::total_cmp);
    let half = heights.len() / 2;
    Some(median(&mut heights[half..]))
}

/// Groups components into lines of glyphs.
fn build_lines(comps: &[Comp], w: usize, h: usize) -> Vec<Line> {
    let usable = usable_components(comps, w, h);
    let Some(reference) = reference_height(comps, &usable) else { return Vec::new() };
    let floor = (0.06 * reference).max(2.0);
    let (mut tall, mut small): (Vec<usize>, Vec<usize>) = (Vec::new(), Vec::new());
    for &i in &usable {
        let c = &comps[i];
        if c.h() as f32 >= 0.42 * reference {
            tall.push(i);
        } else if (c.w().max(c.h()) as f32) >= floor {
            small.push(i);
        }
    }
    if tall.is_empty() {
        return Vec::new();
    }

    // --- tall components → lines, by vertical overlap -------------------------------
    struct Band {
        y0: f32,
        y1: f32,
        members: Vec<usize>,
    }
    tall.sort_by(|&a, &b| comps[a].cy().total_cmp(&comps[b].cy()).then(a.cmp(&b)));
    let mut bands: Vec<Band> = Vec::new();
    for &i in &tall {
        let c = &comps[i];
        let (cy0, cy1) = (c.y0 as f32, c.y1 as f32);
        let mut best: Option<(usize, f32)> = None;
        for (bi, band) in bands.iter().enumerate() {
            let overlap = cy1.min(band.y1) - cy0.max(band.y0);
            let ratio = overlap / (cy1 - cy0).min(band.y1 - band.y0).max(1.0);
            if ratio >= 0.5 && best.is_none_or(|(_, r)| ratio > r) {
                best = Some((bi, ratio));
            }
        }
        match best {
            Some((bi, _)) => {
                let band = &mut bands[bi];
                band.y0 = band.y0.min(cy0);
                band.y1 = band.y1.max(cy1);
                band.members.push(i);
            }
            None => bands.push(Band { y0: cy0, y1: cy1, members: vec![i] }),
        }
    }
    bands.sort_by(|a, b| a.y0.total_cmp(&b.y0));

    // --- glyphs: tall components, with detached marks merged in ------------------------
    let mut lines: Vec<Vec<Glyph>> = bands
        .iter()
        .map(|band| {
            band.members
                .iter()
                .map(|&i| {
                    let c = &comps[i];
                    Glyph {
                        parts: vec![i as u32],
                        main: i as u32,
                        tall: true,
                        x0: c.x0,
                        y0: c.y0,
                        x1: c.x1,
                        y1: c.y1,
                    }
                })
                .collect()
        })
        .collect();

    let mut loose: Vec<(usize, usize)> = Vec::new(); // (line, component)
    for &s in &small {
        let sc = &comps[s];
        let (sw, sh) = (sc.w() as f32, sc.h() as f32);
        // A mark sits entirely above or below its base and overlaps it horizontally.
        let mut best: Option<(usize, usize, f32)> = None;
        for (li, line) in lines.iter().enumerate() {
            for (gi, glyph) in line.iter().enumerate() {
                let base = &comps[glyph.main as usize];
                let overlap = sc.x1.min(base.x1) as f32 - sc.x0.max(base.x0) as f32;
                if overlap < 0.5 * sw {
                    continue;
                }
                let above = (sc.y1 as f32) <= base.y0 as f32 + 0.15 * sh;
                let below = (sc.y0 as f32) >= base.y1 as f32 - 0.15 * sh;
                let gap = if above {
                    base.y0 as f32 - sc.y1 as f32
                } else if below {
                    sc.y0 as f32 - base.y1 as f32
                } else {
                    continue;
                };
                if gap <= 0.6 * reference && best.is_none_or(|(_, _, g)| gap < g) {
                    best = Some((li, gi, gap));
                }
            }
        }
        if let Some((li, gi, _)) = best {
            let glyph = &mut lines[li][gi];
            glyph.parts.push(s as u32);
            glyph.x0 = glyph.x0.min(sc.x0);
            glyph.y0 = glyph.y0.min(sc.y0);
            glyph.x1 = glyph.x1.max(sc.x1);
            glyph.y1 = glyph.y1.max(sc.y1);
            continue;
        }
        // Otherwise: punctuation of the line whose band it lies in.
        let cy = sc.cy();
        let slack = 0.35 * reference;
        let line = bands
            .iter()
            .enumerate()
            .filter(|(_, b)| cy >= b.y0 - slack && cy <= b.y1 + slack)
            .min_by(|(_, a), (_, b)| {
                let da = (cy - 0.5 * (a.y0 + a.y1)).abs();
                let db = (cy - 0.5 * (b.y0 + b.y1)).abs();
                da.total_cmp(&db)
            })
            .map(|(li, _)| li);
        if let Some(li) = line {
            loose.push((li, s));
        }
    }
    // Stacked small components (colon, semicolon, equals sign) form one glyph.
    loose.sort_by_key(|&(li, s)| (li, comps[s].x0, s));
    for (li, s) in loose {
        let sc = &comps[s];
        let merged = lines[li].iter_mut().filter(|g| !g.tall).find(|g| {
            let overlap = sc.x1.min(g.x1) as f32 - sc.x0.max(g.x0) as f32;
            overlap >= 0.5 * (sc.w().min(g.x1 - g.x0)) as f32
        });
        match merged {
            Some(glyph) => {
                if sc.area > comps[glyph.main as usize].area {
                    glyph.main = s as u32;
                }
                glyph.parts.push(s as u32);
                glyph.x0 = glyph.x0.min(sc.x0);
                glyph.y0 = glyph.y0.min(sc.y0);
                glyph.x1 = glyph.x1.max(sc.x1);
                glyph.y1 = glyph.y1.max(sc.y1);
            }
            None => lines[li].push(Glyph {
                parts: vec![s as u32],
                main: s as u32,
                tall: false,
                x0: sc.x0,
                y0: sc.y0,
                x1: sc.x1,
                y1: sc.y1,
            }),
        }
    }

    lines
        .into_iter()
        .map(|mut glyphs| {
            glyphs.sort_by_key(|g| (comps[g.main as usize].x0, g.main));
            Line { glyphs }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comp(x0: u32, y0: u32, x1: u32, y1: u32) -> Comp {
        let area = (x1 - x0) * (y1 - y0);
        Comp { x0, y0, x1, y1, area, perimeter: 2 * (x1 - x0 + y1 - y0) }
    }

    #[test]
    fn marks_merge_with_their_base_and_punctuation_stays_separate() {
        // "É i . T" on one line: E + accent, i stem + dot, a full stop, T.
        let comps = vec![
            comp(10, 30, 40, 80),  // 0: E
            comp(18, 18, 30, 26),  // 1: acute accent above E
            comp(50, 45, 58, 80),  // 2: i stem
            comp(50, 32, 58, 40),  // 3: i dot
            comp(66, 72, 74, 80),  // 4: full stop
            comp(82, 30, 112, 80), // 5: T
        ];
        let lines = build_lines(&comps, 200, 100);
        assert_eq!(lines.len(), 1);
        let glyphs = &lines[0].glyphs;
        assert_eq!(glyphs.len(), 4);
        assert_eq!(glyphs[0].parts, vec![0, 1]);
        assert_eq!((glyphs[0].y0, glyphs[0].y1), (18, 80));
        assert_eq!(glyphs[0].main, 0);
        assert_eq!(glyphs[1].parts, vec![2, 3]);
        assert!(glyphs[0].tall && glyphs[1].tall);
        assert_eq!(glyphs[2].parts, vec![4]);
        assert!(!glyphs[2].tall);
        assert_eq!(glyphs[3].parts, vec![5]);
        assert_eq!(lines[0].tall_count(), 3);
    }

    #[test]
    fn colon_parts_form_one_small_glyph() {
        let comps = vec![
            comp(10, 20, 40, 70), // A
            comp(48, 40, 54, 46), // upper dot
            comp(48, 64, 54, 70), // lower dot
            comp(62, 20, 92, 70), // B
        ];
        let lines = build_lines(&comps, 120, 90);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].glyphs.len(), 3);
        assert_eq!(lines[0].glyphs[1].parts.len(), 2);
        assert!(!lines[0].glyphs[1].tall);
    }

    #[test]
    fn stacked_lines_are_separated_and_ordered() {
        let comps = vec![
            comp(10, 110, 40, 160), // second line
            comp(50, 110, 80, 160),
            comp(10, 20, 40, 70), // first line
            comp(50, 35, 80, 70), // x-height letter on the first line
            comp(90, 35, 120, 85), // letter with a descender on the first line
        ];
        let lines = build_lines(&comps, 200, 200);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].tall_count(), 3);
        assert_eq!(lines[1].tall_count(), 2);
        assert_eq!(lines[0].glyphs[0].main, 2);
    }

    #[test]
    fn border_components_rules_and_specks_are_ignored() {
        let comps = vec![
            comp(0, 20, 30, 70),     // touches the left border
            comp(40, 20, 70, 70),    // letter
            comp(80, 20, 110, 70),   // letter
            comp(20, 90, 180, 93),   // horizontal rule
            comp(120, 40, 121, 41),  // speck
        ];
        let lines = build_lines(&comps, 200, 100);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].glyphs.len(), 2);
        assert!(build_lines(&[], 10, 10).is_empty());
    }

    #[test]
    fn regions_outside_the_image_or_empty_yield_nothing() {
        let img = Pixmap::filled(40, 30, [255, 255, 255, 255]);
        let region = |x, y, w, h| TextRegion { x, y, w, h, score: 1.0 };
        assert!(analyse(&img, &region(40, 0, 10, 10)).is_none());
        assert!(analyse(&img, &region(0, 0, 0, 10)).is_none());
        assert!(analyse(&img, &region(u32::MAX, u32::MAX, u32::MAX, u32::MAX)).is_none());
        // A blank region has no text.
        assert!(analyse(&img, &region(0, 0, 40, 30)).is_none());
    }

    #[test]
    fn dark_and_light_bars_are_found_with_the_right_polarity() {
        for (bg, ink, light_on_dark) in [(230u8, 30u8, false), (25, 220, true)] {
            let mut img = Pixmap::filled(200, 80, [bg, bg, bg, 255]);
            // Four "letters": bars 12 px wide, 40 px tall.
            for k in 0..4u32 {
                for y in 20..60 {
                    for x in 30 + k * 40..42 + k * 40 {
                        img.set(x, y, [ink, ink, ink, 255]);
                    }
                }
            }
            let a = analyse(&img, &TextRegion { x: 20, y: 14, w: 160, h: 52, score: 1.0 })
                .expect("text found");
            assert_eq!(a.light_on_dark, light_on_dark);
            assert_eq!(a.lines.len(), 1);
            assert_eq!(a.lines[0].tall_count(), 4);
            let (crop, _) = a.crop_parts(&a.lines[0].glyphs[0].parts).expect("crop");
            assert!(crop.px.iter().all(|&v| v == 255), "a solid bar is fully covered");
        }
    }
}
