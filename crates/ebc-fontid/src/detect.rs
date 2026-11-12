//! Finding lines and blocks of text on a whole cover.

use ebc_scene::Pixmap;

use crate::cc::{Comp, label_components, local_contrast_masks, median, theil_sen_slope};
use crate::gray::{Channel, Gray, noise_sigma};
use crate::index::{CharTable, FaceDescriptor};
use crate::label::{CaseUsed, apply_case, match_text_structure, observe};
use crate::segment::analyse;

/// A rectangle of an image that contains text, in pixels (origin top-left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextRegion {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub w: u32,
    /// Height.
    pub h: u32,
    /// How likely the region is the wanted text, in `0..=1` (higher is likelier).
    /// Regions drawn by a user can carry any value; it is not used by
    /// [`identify`](crate::identify).
    pub score: f32,
}

/// Longest side of the image at the finer of the two working scales, in pixels.
const WORK_SIDE: usize = 960;
/// Most components per polarity and scale that are grouped (the largest are kept).
const MAX_CANDIDATES: usize = 2500;
/// Most regions returned.
const MAX_REGIONS: usize = 12;
/// Most candidate regions that are segmented to check them against the known title.
const MAX_CHECKED: usize = 14;
/// Smallest letter height at a working scale, in pixels.
const MIN_LETTER: u32 = 7;

/// A candidate region in coordinates of the original image.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    /// Typical letter height.
    letter: f32,
    /// Number of letter-sized components.
    letters: usize,
    /// Number of lines.
    lines: usize,
    light: bool,
    score: f32,
}

impl Candidate {
    fn area(&self) -> f32 {
        (self.x1 - self.x0).max(0.0) * (self.y1 - self.y0).max(0.0)
    }

    fn intersection(&self, other: &Self) -> f32 {
        let w = self.x1.min(other.x1) - self.x0.max(other.x0);
        let h = self.y1.min(other.y1) - self.y0.max(other.y0);
        w.max(0.0) * h.max(0.0)
    }

    fn region(&self, img_w: u32, img_h: u32) -> TextRegion {
        let pad = (0.08 * self.letter).max(1.0);
        let x0 = (self.x0 - pad).floor().clamp(0.0, img_w as f32) as u32;
        let y0 = (self.y0 - pad).floor().clamp(0.0, img_h as f32) as u32;
        let x1 = (self.x1 + pad).ceil().clamp(0.0, img_w as f32) as u32;
        let y1 = (self.y1 + pad).ceil().clamp(0.0, img_h as f32) as u32;
        TextRegion {
            x: x0,
            y: y0,
            w: x1.saturating_sub(x0),
            h: y1.saturating_sub(y0),
            score: self.score.clamp(0.0, 1.0),
        }
    }
}

/// Finds lines and blocks of text in `img`, most title-like first (large, regular,
/// well-aligned lettering scores highest). At most 12 regions.
///
/// This is a classical detector — local-contrast binarisation in both polarities at two
/// scales, connected components, grouping by height, stroke width and alignment. It
/// works on lettering that contrasts with a reasonably calm background and reads the
/// luminance only; on photographic or textured covers it can miss the title or return
/// clutter, so a user interface should let the user adjust the box. An image without
/// text (or without pixels) yields an empty list.
#[must_use]
pub fn find_text_regions(img: &Pixmap) -> Vec<TextRegion> {
    detect(img, None)
}

/// Like [`find_text_regions`], using the known title as a prior: regions whose lines
/// hold exactly the letters of the title (of consecutive words of it, for a title set on
/// several lines) come first. The case of `known_title` does not matter.
#[must_use]
pub fn find_title_regions(img: &Pixmap, known_title: &str) -> Vec<TextRegion> {
    let title = known_title.trim();
    detect(img, (!title.is_empty()).then_some(title))
}

fn detect(img: &Pixmap, title: Option<&str>) -> Vec<TextRegion> {
    let (iw, ih) = (img.width(), img.height());
    if iw < 8 || ih < 8 {
        return Vec::new();
    }
    let longest = iw.max(ih) as usize;
    let reduce = (longest / (2 * WORK_SIDE)).max(1) as u32;
    let Some(base) = Gray::from_pixmap_rect_reduced(img, 0, 0, iw, ih, Channel::Luma, reduce)
    else {
        return Vec::new();
    };

    let mut candidates: Vec<Candidate> = Vec::new();
    let fine = (WORK_SIDE as f32 / base.w.max(base.h) as f32).min(1.0);
    for scale in [fine, fine * 0.5] {
        let w = ((base.w as f32 * scale).round() as usize).max(1);
        let h = ((base.h as f32 * scale).round() as usize).max(1);
        if w.min(h) < 48 {
            continue;
        }
        let gray = base.resize(w, h);
        // Working pixel → original pixel.
        let (ux, uy) = (iw as f32 / w as f32, ih as f32 / h as f32);
        let c_min = (3.5 * noise_sigma(&gray)).clamp(12.0, 48.0);
        let window = ((w.max(h) / 10) | 1).clamp(31, 65);
        let (dark, light) = local_contrast_masks(&gray, window, 0.3, c_min);
        for (mask, is_light) in [(&dark, false), (&light, true)] {
            let (labels, comps) = label_components(mask, w, h);
            let lines = find_lines(&gray, &labels, &comps, is_light, c_min);
            let start = candidates.len();
            for line in &lines {
                candidates.push(Candidate {
                    x0: line.x0 * ux,
                    y0: line.y0 * uy,
                    x1: line.x1 * ux,
                    y1: line.y1 * uy,
                    letter: line.letter * uy,
                    letters: line.letters,
                    lines: 1,
                    light: is_light,
                    score: line.quality,
                });
            }
            let singles: Vec<Candidate> = candidates[start..].to_vec();
            candidates.extend(blocks(&singles));
        }
    }
    if candidates.is_empty() {
        return Vec::new();
    }

    // Base score: quality of the lettering and its size relative to the cover.
    for c in &mut candidates {
        let size = (c.letter / (0.11 * ih as f32)).clamp(0.0, 1.0).sqrt();
        let count = (c.letters as f32 / 5.0).min(1.0);
        c.score = 0.5 * size + 0.2 * count + 0.3 * c.score;
    }
    let min_letters = if title.is_some() { 2 } else { 3 };
    candidates.retain(|c| c.letters >= min_letters);
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept = suppress(&candidates);

    if let Some(title) = title {
        let empty = FaceDescriptor::centroid(&CharTable::reference(), std::iter::empty());
        for (rank, c) in kept.iter_mut().enumerate() {
            let prior = if rank < MAX_CHECKED {
                title_prior(img, &c.region(iw, ih), title, &empty)
            } else {
                0.0
            };
            c.score *= 0.4 + 0.6 * prior;
        }
        kept.sort_by(|a, b| b.score.total_cmp(&a.score));
    }
    kept.truncate(MAX_REGIONS);
    kept.iter().map(|c| c.region(iw, ih)).filter(|r| r.w > 0 && r.h > 0).collect()
}

/// How well the letters of the title fit the lines of a region, in `0..=1`: 1 when the
/// region's lines hold exactly the letters of the whole title, 0.6 when they do nearly
/// (touching or broken letters), and a small value for a part of the title.
fn title_prior(img: &Pixmap, region: &TextRegion, title: &str, no_shapes: &FaceDescriptor) -> f32 {
    let Some(analysis) = analyse(img, region) else { return 0.0 };
    let obs = observe(&analysis, false);
    let mut best = 0.0f32;
    let mut tried: Vec<String> = Vec::new();
    for case in [CaseUsed::AsGiven, CaseUsed::Upper] {
        let cased = apply_case(title, case);
        if tried.contains(&cased) {
            continue;
        }
        if let Some((coverage, exact, matched_lines)) =
            match_text_structure(&obs, &cased, no_shapes)
        {
            // Only the whole title with the right letter count on every line is strong
            // evidence; a part of it fits other text too easily.
            let all_lines = matched_lines == obs.lines.len();
            let p = if coverage >= 0.999 && all_lines {
                if exact { 1.0 } else { 0.6 }
            } else {
                0.3 * coverage * (matched_lines as f32 / obs.lines.len().max(1) as f32)
            };
            best = best.max(p);
        }
        tried.push(cased);
    }
    best
}

/// Removes duplicates: the same text found at the other scale, and the "ghost" lines
/// that the counters of letters form in the opposite polarity. `sorted` is best first.
fn suppress(sorted: &[Candidate]) -> Vec<Candidate> {
    let mut kept: Vec<Candidate> = Vec::new();
    'next: for c in sorted {
        for k in &kept {
            let inter = c.intersection(k);
            if inter <= 0.0 {
                continue;
            }
            let iou = inter / (c.area() + k.area() - inter).max(1.0);
            let same_layout = c.light == k.light && c.lines == k.lines;
            if (same_layout && iou >= 0.5) || iou >= 0.8 {
                continue 'next;
            }
            if c.light != k.light && inter / c.area().max(1.0) >= 0.6 {
                continue 'next;
            }
        }
        kept.push(*c);
        if kept.len() >= 4 * MAX_REGIONS {
            break;
        }
    }
    kept
}

/// A line of letter-like components at a working scale.
#[derive(Debug, Clone, Copy)]
struct LineBox {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    letter: f32,
    letters: usize,
    /// Regularity of stroke widths and alignment, in `0..=1`.
    quality: f32,
}

/// Could this component be a letter (or two touching letters)?
fn letter_like(c: &Comp, w: usize, h: usize) -> bool {
    let (cw, ch) = (c.w(), c.h());
    if ch < MIN_LETTER || c.area < 12 {
        return false;
    }
    if c.x0 == 0 || c.y0 == 0 || c.x1 as usize >= w || c.y1 as usize >= h {
        return false;
    }
    // Up to six times as wide as tall: a few touching letters still count.
    if ch as usize * 2 > h || cw as usize * 2 > w || cw > ch * 6 || ch > cw * 14 {
        return false;
    }
    let fill = c.area as f32 / (cw * ch) as f32;
    if fill < 0.08 {
        return false;
    }
    // Solid blocks are letters only when they are bars (I, l, 1).
    if (fill > 0.92 && cw * 2 > ch) || (cw > ch * 3 && fill > 0.75) {
        return false;
    }
    c.stroke_width() <= 0.5 * ch as f32
}

/// Groups letter-like components into lines. `labels` and `comps` describe the ink mask
/// of one polarity of `gray` (`light`: the ink is lighter than its surroundings).
fn find_lines(
    gray: &Gray,
    labels: &[u32],
    comps: &[Comp],
    light: bool,
    c_min: f32,
) -> Vec<LineBox> {
    let (w, h) = (gray.w, gray.h);
    let mut order: Vec<usize> =
        (0..comps.len()).filter(|&i| letter_like(&comps[i], w, h)).collect();
    if order.len() > MAX_CANDIDATES {
        order.sort_by(|&a, &b| comps[b].area.cmp(&comps[a].area));
        order.truncate(MAX_CANDIDATES);
    }
    order.sort_by_key(|&i| (comps[i].x0, comps[i].y0));
    let letters: Vec<&Comp> = order.iter().map(|&i| &comps[i]).collect();
    let n = letters.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let max_gap = h as f32 * 0.5;
    for i in 0..n {
        let a = letters[i];
        let (ha, sa) = (a.h() as f32, a.stroke_width());
        for (j, &b) in letters.iter().enumerate().skip(i + 1) {
            let gap = b.x0 as f32 - a.x1 as f32;
            if gap > max_gap {
                break;
            }
            let hb = b.h() as f32;
            let (tall, short) = (ha.max(hb), ha.min(hb));
            if gap > 1.6 * tall || tall > 2.3 * short {
                continue;
            }
            let overlap = a.y1.min(b.y1) as f32 - a.y0.max(b.y0) as f32;
            if overlap < 0.5 * short {
                continue;
            }
            let sb = b.stroke_width();
            if sa.max(sb) > 2.5 * sa.min(sb) {
                continue;
            }
            let (ra, rb) = (find(&mut parent, i), find(&mut parent, j));
            if ra != rb {
                parent[ra] = rb;
            }
        }
    }
    // Second stage: words on one line whose gap is wide for their letters (letter-spaced
    // or monospaced lower case) are joined by the size of the words instead.
    {
        // (root, x0, y0, x1, y1, reference height) of every group so far.
        let mut words: Vec<(usize, f32, f32, f32, f32, f32)> = Vec::new();
        let mut members: Vec<Vec<usize>> = vec![Vec::new(); n];
        for i in 0..n {
            let root = find(&mut parent, i);
            members[root].push(i);
        }
        for (root, group) in members.iter().enumerate().filter(|(_, g)| !g.is_empty()) {
            let mut heights: Vec<f32> = group.iter().map(|&i| letters[i].h() as f32).collect();
            heights.sort_by(f32::total_cmp);
            let reference = median(&mut heights[group.len() / 2..]);
            let x0 = group.iter().map(|&i| letters[i].x0).min().unwrap_or(0) as f32;
            let y0 = group.iter().map(|&i| letters[i].y0).min().unwrap_or(0) as f32;
            let x1 = group.iter().map(|&i| letters[i].x1).max().unwrap_or(0) as f32;
            let y1 = group.iter().map(|&i| letters[i].y1).max().unwrap_or(0) as f32;
            words.push((root, x0, y0, x1, y1, reference));
        }
        words.sort_by(|a, b| a.1.total_cmp(&b.1));
        for i in 0..words.len() {
            let a = words[i];
            for b in &words[i + 1..] {
                let gap = b.1 - a.3;
                let tall = a.5.max(b.5);
                if gap > 1.3 * tall {
                    if gap > max_gap {
                        break;
                    }
                    continue;
                }
                let overlap = a.4.min(b.4) - a.2.max(b.2);
                if tall > 1.8 * a.5.min(b.5) || overlap < 0.6 * (a.4 - a.2).min(b.4 - b.2) {
                    continue;
                }
                let (ra, rb) = (find(&mut parent, a.0), find(&mut parent, b.0));
                if ra != rb {
                    parent[ra] = rb;
                }
            }
        }
    }
    let mut groups: Vec<Vec<usize>> = vec![Vec::new(); n];
    // Group of every component (`u32::MAX`: none), to sum grey levels per group.
    let mut group_of = vec![u32::MAX; comps.len()];
    for i in 0..n {
        let root = find(&mut parent, i);
        groups[root].push(i);
        group_of[order[i]] = root as u32;
    }
    let mut level_sum = vec![0u64; n];
    let mut level_count = vec![0u32; n];
    for (&label, &v) in labels.iter().zip(&gray.px) {
        if label == 0 {
            continue;
        }
        let group = group_of[label as usize - 1];
        if group != u32::MAX {
            level_sum[group as usize] += u64::from(v);
            level_count[group as usize] += 1;
        }
    }

    let mut lines = Vec::new();
    for (root, members) in groups.into_iter().enumerate().filter(|(_, g)| g.len() >= 2) {
        let mut heights: Vec<f32> = members.iter().map(|&i| letters[i].h() as f32).collect();
        heights.sort_by(f32::total_cmp);
        let reference = median(&mut heights[members.len() / 2..]);
        let tall: Vec<&Comp> = members
            .iter()
            .map(|&i| letters[i])
            .filter(|c| c.h() as f32 >= 0.42 * reference)
            .collect();
        if tall.len() < 2 {
            continue;
        }
        // Stroke-width regularity.
        let strokes: Vec<f32> = tall.iter().map(|c| c.stroke_width()).collect();
        let mean = strokes.iter().sum::<f32>() / strokes.len() as f32;
        let var =
            strokes.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / strokes.len() as f32;
        let cv = var.sqrt() / mean.max(1e-3);
        // Alignment of the bottoms on a (possibly slanted) baseline.
        let points: Vec<(f32, f32)> = tall.iter().map(|c| (c.cx(), c.y1 as f32)).collect();
        let slope = if points.len() >= 4 { theil_sen_slope(&points) } else { 0.0 };
        let mut offsets: Vec<f32> = points.iter().map(|p| p.1 - slope * p.0).collect();
        let baseline = median(&mut offsets);
        let mut deviations: Vec<f32> = offsets.iter().map(|o| (o - baseline).abs()).collect();
        let mad = median(&mut deviations) / reference.max(1.0);
        let quality = 0.5 / (1.0 + 3.0 * cv) + 0.5 / (1.0 + 6.0 * mad);

        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for c in &tall {
            x0 = x0.min(c.x0);
            y0 = y0.min(c.y0);
            x1 = x1.max(c.x1);
            y1 = y1.max(c.y1);
        }
        // Real lettering differs from what lies just around the line. The halo that
        // dark text leaves in the light mask (and vice versa) and the counters of
        // letters do not: they have the level of the background.
        let pad = (0.35 * reference).round().max(2.0) as u32;
        let ring =
            ring_level(gray, x0.saturating_sub(pad), y0.saturating_sub(pad), x1 + pad, y1 + pad);
        let ink = level_sum[root] as f32 / level_count[root].max(1) as f32;
        let contrast = if light { ink - ring } else { ring - ink };
        if contrast < c_min.max(20.0) {
            continue;
        }
        lines.push(LineBox {
            x0: x0 as f32,
            y0: y0 as f32,
            x1: x1 as f32,
            y1: y1 as f32,
            letter: reference,
            letters: tall.len(),
            quality,
        });
    }
    lines
}

/// Median grey level on the outline of a rectangle (clamped to the image).
fn ring_level(gray: &Gray, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let (w, h) = (gray.w, gray.h);
    let (x0, y0) = ((x0 as usize).min(w - 1), (y0 as usize).min(h - 1));
    let (x1, y1) = ((x1 as usize).clamp(x0 + 1, w), (y1 as usize).clamp(y0 + 1, h));
    let mut hist = [0u32; 256];
    let mut n = 0u32;
    for x in x0..x1 {
        hist[usize::from(gray.px[y0 * w + x])] += 1;
        hist[usize::from(gray.px[(y1 - 1) * w + x])] += 1;
        n += 2;
    }
    for y in y0..y1 {
        hist[usize::from(gray.px[y * w + x0])] += 1;
        hist[usize::from(gray.px[y * w + x1 - 1])] += 1;
        n += 2;
    }
    let mut acc = 0u32;
    for (level, &count) in hist.iter().enumerate() {
        acc += count;
        if acc * 2 >= n {
            return level as f32;
        }
    }
    255.0
}

/// Blocks of two to four stacked lines of similar letter height.
fn blocks(lines: &[Candidate]) -> Vec<Candidate> {
    let mut order: Vec<usize> = (0..lines.len()).collect();
    order.sort_by(|&a, &b| lines[a].y0.total_cmp(&lines[b].y0));
    // For each line, the nearest compatible line below it.
    let below = |i: usize| -> Option<usize> {
        let a = &lines[i];
        order
            .iter()
            .copied()
            .filter(|&j| j != i)
            .filter(|&j| {
                let b = &lines[j];
                let gap = b.y0 - a.y1;
                let tall = a.letter.max(b.letter);
                let overlap = a.x1.min(b.x1) - a.x0.max(b.x0);
                b.y0 > a.y0
                    && gap >= -0.3 * tall
                    && gap <= 1.3 * tall
                    && tall <= 1.67 * a.letter.min(b.letter)
                    && overlap > 0.0
            })
            .min_by(|&p, &q| lines[p].y0.total_cmp(&lines[q].y0))
    };
    let mut out = Vec::new();
    for start in 0..lines.len() {
        let mut block = lines[start];
        let mut last = start;
        let mut quality = lines[start].score;
        for _ in 0..3 {
            let Some(next) = below(last) else { break };
            let b = &lines[next];
            block.x0 = block.x0.min(b.x0);
            block.y0 = block.y0.min(b.y0);
            block.x1 = block.x1.max(b.x1);
            block.y1 = block.y1.max(b.y1);
            block.letters += b.letters;
            block.lines += 1;
            block.letter = block.letter.max(b.letter);
            quality += b.score;
            last = next;
            let mut c = block;
            c.score = quality / c.lines as f32;
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bars(img: &mut Pixmap, x: u32, y: u32, n: u32, h: u32, ink: [u8; 4]) {
        let w = (h / 4).max(2);
        for k in 0..n {
            for yy in y..y + h {
                for xx in x + k * (w * 3)..x + k * (w * 3) + w {
                    img.set(xx, yy, ink);
                }
            }
        }
    }

    #[test]
    fn rows_of_bars_are_found_largest_first_in_both_polarities() {
        let mut img = Pixmap::filled(300, 400, [240, 240, 240, 255]);
        bars(&mut img, 40, 60, 8, 40, [20, 20, 20, 255]);
        bars(&mut img, 40, 300, 10, 14, [20, 20, 20, 255]);
        let regions = find_text_regions(&img);
        assert!(regions.len() >= 2, "{regions:?}");
        let r = regions[0];
        assert!(r.y <= 60 && r.y + r.h >= 100 && r.y > 30 && r.y + r.h < 130, "{r:?}");
        assert!(r.x <= 40 && r.x + r.w >= 40 + 7 * 30 + 10, "{r:?}");
        assert!(regions.windows(2).all(|p| p[0].score >= p[1].score));
        assert!(regions.iter().all(|r| (0.0..=1.0).contains(&r.score)));

        let mut dark = Pixmap::filled(300, 400, [15, 15, 40, 255]);
        bars(&mut dark, 40, 200, 8, 40, [250, 240, 200, 255]);
        let regions = find_text_regions(&dark);
        assert!(!regions.is_empty());
        assert!(regions[0].y <= 200 && regions[0].y + regions[0].h >= 240);
    }

    #[test]
    fn images_without_text_yield_nothing() {
        assert!(find_text_regions(&Pixmap::filled(200, 300, [255, 255, 255, 255])).is_empty());
        assert!(find_text_regions(&Pixmap::filled(200, 300, [0, 0, 0, 255])).is_empty());
        assert!(find_text_regions(&Pixmap::filled(3, 3, [0, 0, 0, 255])).is_empty());
        assert!(find_text_regions(&Pixmap::filled(1, 5000, [9, 9, 9, 255])).is_empty());
        assert!(
            find_title_regions(&Pixmap::filled(200, 300, [90, 90, 90, 255]), "Title").is_empty()
        );
        // One isolated blob is not text.
        let mut img = Pixmap::filled(200, 200, [255, 255, 255, 255]);
        bars(&mut img, 90, 90, 1, 30, [0, 0, 0, 255]);
        assert!(find_text_regions(&img).is_empty());
    }

    #[test]
    fn the_title_prior_prefers_the_line_with_the_right_letter_count() {
        let mut img = Pixmap::filled(400, 500, [240, 240, 240, 255]);
        // A big line of 9 bars and a smaller one of 5.
        bars(&mut img, 30, 60, 9, 44, [20, 20, 20, 255]);
        bars(&mut img, 30, 300, 5, 30, [20, 20, 20, 255]);
        let plain = find_text_regions(&img);
        assert!(plain[0].y < 150, "without a title the big line wins: {plain:?}");
        let with_title = find_title_regions(&img, "Titre");
        assert!(with_title[0].y > 250, "five letters: the small line wins: {with_title:?}");
        // A blank title falls back to the plain detector.
        assert_eq!(find_title_regions(&img, "   "), plain);
    }
}
