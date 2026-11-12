//! The page tree (with inherited attributes) and the page boxes.

use std::collections::HashSet;

use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::codes;
use crate::expect::{BLEED_TOLERANCE_MM, Expectations, NESTING_TOLERANCE_MM, TRIM_TOLERANCE_MM};
use crate::pdf::{PT_PER_MM, RectPt, as_dict, get_num, int, resolve, resolve_with_id};
use crate::report::{BleedMm, BoxMm, round3};
use crate::sink::Sink;

/// Pages analysed per document.
pub(crate) const MAX_PAGES: usize = 2_000;
/// Depth of the page tree.
const MAX_TREE_DEPTH: usize = 64;

/// Attributes a page inherits from its ancestors in the page tree.
#[derive(Clone, Copy, Default)]
struct Inherited<'a> {
    media_box: Option<&'a Object>,
    crop_box: Option<&'a Object>,
    resources: Option<&'a Object>,
    rotate: Option<&'a Object>,
}

impl<'a> Inherited<'a> {
    fn updated(self, doc: &'a Document, node: &'a Dictionary) -> Self {
        let own = |key: &[u8], inherited: Option<&'a Object>| match node.get(key) {
            Ok(value) if resolve(doc, value).is_some() => Some(value),
            _ => inherited,
        };
        Self {
            media_box: own(b"MediaBox", self.media_box),
            crop_box: own(b"CropBox", self.crop_box),
            resources: own(b"Resources", self.resources),
            rotate: own(b"Rotate", self.rotate),
        }
    }
}

/// A leaf of the page tree with its inherited attributes resolved.
pub(crate) struct PageNode<'a> {
    pub(crate) dict: &'a Dictionary,
    pub(crate) resources: Option<&'a Dictionary>,
    media_box: Option<&'a Object>,
    crop_box: Option<&'a Object>,
    rotate: Option<&'a Object>,
}

/// Walks the page tree in document order. Cycles, shared nodes and excessive depth are
/// reported and skipped; the walk is iterative and visits every indirect node at most once.
pub(crate) fn collect<'a>(
    doc: &'a Document,
    catalog: &'a Dictionary,
    sink: &mut Sink,
) -> Vec<PageNode<'a>> {
    let mut pages = Vec::new();
    let Ok(root) = catalog.get(b"Pages") else {
        sink.error(codes::PAGE_TREE_INVALID, None, "the catalog has no Pages entry");
        return pages;
    };
    let mut visited: HashSet<ObjectId> = HashSet::new();
    // Each frame: the kids still to visit (in reverse order) and what they inherit.
    let mut stack: Vec<(Vec<&'a Object>, Inherited<'a>)> = vec![(vec![root], Inherited::default())];
    while let Some((kids, inherited)) = stack.last_mut() {
        let Some(node) = kids.pop() else {
            stack.pop();
            continue;
        };
        let inherited = *inherited;
        let (id, resolved) = resolve_with_id(doc, node);
        let Some(Object::Dictionary(dict)) = resolved else {
            sink.error(codes::PAGE_TREE_INVALID, None, "a page tree node is not a dictionary");
            continue;
        };
        if let Some(id) = id
            && !visited.insert(id)
        {
            sink.error(
                codes::PAGE_TREE_INVALID,
                None,
                format!(
                    "page tree node {} {} is reachable more than once (cycle or shared node)",
                    id.0, id.1
                ),
            );
            continue;
        }
        let inherited = inherited.updated(doc, dict);
        let kids_entry = dict.get(b"Kids").ok().and_then(|kids| resolve(doc, kids));
        let is_tree_node = match dict.get(b"Type").ok().and_then(|t| resolve(doc, t)) {
            Some(Object::Name(name)) if name == b"Pages" => true,
            Some(Object::Name(name)) if name == b"Page" => false,
            _ => kids_entry.is_some(),
        };
        if is_tree_node {
            let Some(Object::Array(children)) = kids_entry else {
                sink.error(codes::PAGE_TREE_INVALID, None, "a Pages node has no Kids array");
                continue;
            };
            if stack.len() >= MAX_TREE_DEPTH {
                sink.error(codes::PAGE_TREE_INVALID, None, "the page tree is nested too deeply");
                continue;
            }
            stack.push((children.iter().rev().collect(), inherited));
        } else {
            if pages.len() >= MAX_PAGES {
                sink.error(
                    codes::LIMIT_EXCEEDED,
                    None,
                    format!(
                        "the document has more than {MAX_PAGES} pages; the rest are not analysed"
                    ),
                );
                break;
            }
            pages.push(PageNode {
                dict,
                resources: inherited.resources.and_then(|r| as_dict(doc, r)),
                media_box: inherited.media_box,
                crop_box: inherited.crop_box,
                rotate: inherited.rotate,
            });
        }
    }
    pages
}

/// The boxes of one page, in points of default user space, plus what scales them.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PageBoxes {
    pub(crate) media: Option<RectPt>,
    pub(crate) crop: Option<RectPt>,
    pub(crate) bleed: Option<RectPt>,
    pub(crate) trim: Option<RectPt>,
    pub(crate) art: Option<RectPt>,
    pub(crate) user_unit: f64,
    pub(crate) rotate: i32,
}

impl PageBoxes {
    /// Converts a box to millimetres.
    pub(crate) fn to_mm(self, rect: RectPt) -> BoxMm {
        let k = self.user_unit / PT_PER_MM;
        BoxMm { x0: rect.x0 * k, y0: rect.y0 * k, x1: rect.x1 * k, y1: rect.y1 * k }
    }

    /// The area that is printed next to or inside the finished piece: the bleed box, else
    /// the trim box.
    pub(crate) fn printed_area(&self) -> Option<RectPt> {
        self.bleed.or(self.trim)
    }

    /// Trim size in millimetres.
    pub(crate) fn trim_size_mm(&self) -> Option<(f64, f64)> {
        self.trim.map(|trim| {
            let mm = self.to_mm(trim);
            (mm.width(), mm.height())
        })
    }

    /// Media size in millimetres.
    pub(crate) fn media_size_mm(&self) -> Option<(f64, f64)> {
        self.media.map(|media| {
            let mm = self.to_mm(media);
            (mm.width(), mm.height())
        })
    }
}

/// Reads the boxes of a page, reporting malformed ones.
pub(crate) fn read_boxes(
    doc: &Document,
    node: &PageNode<'_>,
    page: u32,
    sink: &mut Sink,
) -> PageBoxes {
    let mut read = |name: &str, object: Option<&Object>| -> Option<RectPt> {
        let object = object?;
        resolve(doc, object)?;
        match RectPt::from_object(doc, object) {
            Some(rect) if rect.width() > 0.0 && rect.height() > 0.0 => Some(rect),
            Some(_) => {
                sink.error(
                    codes::BOX_INVALID,
                    Some(page),
                    format!("{name} is empty (zero width or height)"),
                );
                None
            }
            None => {
                sink.error(
                    codes::BOX_INVALID,
                    Some(page),
                    format!("{name} is not a rectangle of four finite numbers"),
                );
                None
            }
        }
    };
    let media = read("MediaBox", node.media_box);
    let crop = read("CropBox", node.crop_box);
    let bleed = read("BleedBox", node.dict.get(b"BleedBox").ok());
    let trim = read("TrimBox", node.dict.get(b"TrimBox").ok());
    let art = read("ArtBox", node.dict.get(b"ArtBox").ok());

    let user_unit = match get_num(doc, node.dict, b"UserUnit") {
        Some(unit) if unit > 0.0 && unit <= 75_000.0 => {
            if (unit - 1.0).abs() > 1.0e-9 {
                sink.warning(
                    codes::PAGE_USER_UNIT,
                    Some(page),
                    format!("the page uses UserUnit {unit}; many print workflows ignore it"),
                );
            }
            unit
        }
        Some(unit) => {
            sink.error(codes::BOX_INVALID, Some(page), format!("UserUnit {unit} is not valid"));
            1.0
        }
        None => 1.0,
    };
    let rotate = node
        .rotate
        .and_then(|r| resolve(doc, r))
        .and_then(int)
        .map_or(0, |r| i32::try_from(r.rem_euclid(360)).unwrap_or(0));
    if rotate != 0 {
        sink.warning(
            codes::PAGE_ROTATED,
            Some(page),
            format!("the page is rotated by {rotate}° with the Rotate key; supply it unrotated"),
        );
    }
    PageBoxes { media, crop, bleed, trim, art, user_unit, rotate }
}

/// What the box checks computed for the page summary.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BoxMeasures {
    pub(crate) trim_mm: Option<(f64, f64)>,
    pub(crate) bleed_mm: Option<BleedMm>,
}

/// Checks presence, nesting, bleed and trim size.
pub(crate) fn check_boxes(
    boxes: &PageBoxes,
    expect: &Expectations,
    page: u32,
    sink: &mut Sink,
) -> BoxMeasures {
    let mut measures = BoxMeasures::default();
    let Some(media) = boxes.media else {
        sink.error(codes::BOX_MEDIA_MISSING, Some(page), "the page has no valid MediaBox");
        return measures;
    };
    let media_mm = boxes.to_mm(media);
    let Some(trim) = boxes.trim else {
        let hint = if boxes.art.is_some() {
            " (it has an ArtBox, which printers do not use for cutting)"
        } else {
            ""
        };
        sink.error(
            codes::BOX_TRIM_MISSING,
            Some(page),
            format!("the page has no TrimBox: the finished size is undefined{hint}"),
        );
        return measures;
    };
    let trim_mm = boxes.to_mm(trim);
    measures.trim_mm = Some((round3(trim_mm.width()), round3(trim_mm.height())));
    if boxes.art.is_some() {
        sink.warning(
            codes::BOX_TRIM_AND_ART,
            Some(page),
            "the page has both a TrimBox and an ArtBox; PDF/X allows only one of them",
        );
    }

    // Without a BleedBox, the box that clips the output takes its place (ISO 32000-1, 14.11.2).
    let (bleed_source, effective_bleed) = match (boxes.bleed, boxes.crop) {
        (Some(bleed), _) => ("BleedBox", bleed),
        (None, Some(crop)) => ("CropBox", crop),
        (None, None) => ("MediaBox", media),
    };
    let bleed_mm = boxes.to_mm(effective_bleed);
    if boxes.bleed.is_none() && expect.bleed_mm > 0.0 {
        sink.warning(
            codes::BOX_BLEED_MISSING,
            Some(page),
            format!("the page has no BleedBox; the {bleed_source} is used to measure the bleed"),
        );
    }

    let mut nested = true;
    if !bleed_mm.contains(&trim_mm, NESTING_TOLERANCE_MM) {
        nested = false;
        sink.error(
            codes::BOX_NOT_NESTED,
            Some(page),
            format!("the TrimBox extends beyond the {bleed_source}"),
        );
    }
    if !media_mm.contains(&bleed_mm, NESTING_TOLERANCE_MM) {
        sink.error(
            codes::BOX_NOT_NESTED,
            Some(page),
            format!("the {bleed_source} extends beyond the MediaBox"),
        );
    }
    if let (Some(crop), Some(bleed)) = (boxes.crop, boxes.bleed)
        && !boxes.to_mm(crop).contains(&boxes.to_mm(bleed), NESTING_TOLERANCE_MM)
    {
        sink.warning(
            codes::BOX_CROP_CLIPS_BLEED,
            Some(page),
            "the CropBox is smaller than the BleedBox: viewers and some RIPs clip the bleed",
        );
    }

    let margins = BleedMm {
        left: round3(trim_mm.x0 - bleed_mm.x0),
        right: round3(bleed_mm.x1 - trim_mm.x1),
        top: round3(bleed_mm.y1 - trim_mm.y1),
        bottom: round3(trim_mm.y0 - bleed_mm.y0),
    };
    measures.bleed_mm = Some(margins);
    if nested && expect.bleed_mm > 0.0 && margins.min() < expect.bleed_mm - BLEED_TOLERANCE_MM {
        sink.error(
            codes::BOX_BLEED_TOO_SMALL,
            Some(page),
            format!(
                "bleed is {:.2} mm left, {:.2} mm right, {:.2} mm top, {:.2} mm bottom; at least {:.2} mm is expected on every side",
                margins.left, margins.right, margins.top, margins.bottom, expect.bleed_mm
            ),
        );
    }

    if let Some((expected_w, expected_h)) = expect.trim_mm {
        let (w, h) = (trim_mm.width(), trim_mm.height());
        if (w - expected_w).abs() > TRIM_TOLERANCE_MM || (h - expected_h).abs() > TRIM_TOLERANCE_MM
        {
            let swapped = (w - expected_h).abs() <= TRIM_TOLERANCE_MM
                && (h - expected_w).abs() <= TRIM_TOLERANCE_MM;
            let hint = if swapped { " (width and height are swapped)" } else { "" };
            sink.error(
                codes::BOX_TRIM_SIZE,
                Some(page),
                format!("the trim size is {w:.2} × {h:.2} mm, expected {expected_w:.2} × {expected_h:.2} mm{hint}"),
            );
        }
    }
    measures
}

/// Warns when artwork reaches the trim edge without extending into the bleed.
///
/// `painted` is the bounding box of everything painted on the page (in points), so this is
/// a coarse test: it cannot see an object that stops at the trim line next to another one
/// that bleeds properly, nor artwork that is clipped at the trim line.
pub(crate) fn check_bleed_painted(
    boxes: &PageBoxes,
    painted: Option<RectPt>,
    expect: &Expectations,
    page: u32,
    sink: &mut Sink,
) {
    /// How close to the trim line artwork must come to count as reaching it, in millimetres.
    const REACH_MM: f64 = 0.1;
    /// Shortfall of bleed that is tolerated, in millimetres.
    const SLACK_MM: f64 = 0.2;
    let (Some(trim), Some(painted), Some(media)) = (boxes.trim, painted, boxes.media) else {
        return;
    };
    if expect.bleed_mm <= 0.0 {
        return;
    }
    let outer = boxes.bleed.or(boxes.crop).unwrap_or(media);
    let pt_per_mm = PT_PER_MM / boxes.user_unit;
    // Per side: how far the artwork extends beyond the trim line (negative: it stops short
    // of it), and how much bleed the page offers there.
    let sides = [
        ("left", trim.x0 - painted.x0, trim.x0 - outer.x0),
        ("right", painted.x1 - trim.x1, outer.x1 - trim.x1),
        ("bottom", trim.y0 - painted.y0, trim.y0 - outer.y0),
        ("top", painted.y1 - trim.y1, outer.y1 - trim.y1),
    ];
    let mut short: Vec<String> = Vec::new();
    for (side, extends_pt, available_pt) in sides {
        let extends = extends_pt / pt_per_mm;
        let needed = expect.bleed_mm.min(available_pt / pt_per_mm) - SLACK_MM;
        if extends >= -REACH_MM && extends < needed {
            short.push(format!("{side} ({:.1} mm)", extends.max(0.0)));
        }
    }
    if !short.is_empty() {
        sink.warning(
            codes::BLEED_NOT_PAINTED,
            Some(page),
            format!(
                "artwork reaches the trim edge but does not fill the bleed at the {}: \
                 a white sliver may show after cutting",
                short.join(", ")
            ),
        );
    }
}

/// Checks that all pages have the same trim size and the same media size. Each box is
/// compared with the first page that has one, so a page with a missing box (reported on its
/// own) does not make every other page look different.
pub(crate) fn check_same_size(all: &[PageBoxes], sink: &mut Sink) {
    let differs = |a: (f64, f64), b: (f64, f64)| {
        (a.0 - b.0).abs() > TRIM_TOLERANCE_MM || (a.1 - b.1).abs() > TRIM_TOLERANCE_MM
    };
    type SizeOf = fn(&PageBoxes) -> Option<(f64, f64)>;
    let kinds: [(&str, SizeOf); 2] =
        [("trim", PageBoxes::trim_size_mm), ("media", PageBoxes::media_size_mm)];
    for (kind, size_of) in kinds {
        let Some((reference_index, reference)) = all
            .iter()
            .enumerate()
            .find_map(|(index, boxes)| size_of(boxes).map(|size| (index, size)))
        else {
            continue;
        };
        for (index, boxes) in all.iter().enumerate().skip(reference_index + 1) {
            let Some(size) = size_of(boxes) else { continue };
            if differs(size, reference) {
                sink.error(
                    codes::PAGE_SIZE_MISMATCH,
                    u32::try_from(index + 1).ok(),
                    format!(
                        "the {kind} size is {:.2} × {:.2} mm but page {} has {:.2} × {:.2} mm",
                        size.0,
                        size.1,
                        reference_index + 1,
                        reference.0,
                        reference.1
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use lopdf::dictionary;

    use super::*;
    use crate::report::Severity;

    fn mm(v: f64) -> f64 {
        v * PT_PER_MM
    }

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> RectPt {
        RectPt { x0: mm(x0), y0: mm(y0), x1: mm(x1), y1: mm(y1) }
    }

    fn card_boxes() -> PageBoxes {
        PageBoxes {
            media: Some(rect(0.0, 0.0, 236.0, 184.0)),
            crop: None,
            bleed: Some(rect(0.0, 0.0, 236.0, 184.0)),
            trim: Some(rect(3.0, 3.0, 233.0, 181.0)),
            art: None,
            user_unit: 1.0,
            rotate: 0,
        }
    }

    fn codes_of(boxes: &PageBoxes, expect: &Expectations) -> Vec<(&'static str, Severity)> {
        let mut sink = Sink::default();
        check_boxes(boxes, expect, 1, &mut sink);
        sink.into_sorted().into_iter().map(|f| (f.code, f.severity)).collect()
    }

    #[test]
    fn a_correct_card_page_has_no_box_findings() {
        let expect = Expectations::professional().with_trim(230.0, 178.0);
        let mut sink = Sink::default();
        let measures = check_boxes(&card_boxes(), &expect, 1, &mut sink);
        assert!(sink.into_sorted().is_empty());
        assert_eq!(measures.trim_mm, Some((230.0, 178.0)));
        let bleed = measures.bleed_mm.unwrap();
        assert_eq!((bleed.left, bleed.right, bleed.top, bleed.bottom), (3.0, 3.0, 3.0, 3.0));
    }

    #[test]
    fn bleed_is_measured_per_side() {
        let expect = Expectations::professional();
        // 3 mm everywhere except 1 mm at the top.
        let mut boxes = card_boxes();
        boxes.bleed = Some(rect(0.0, 0.0, 236.0, 182.0));
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_BLEED_TOO_SMALL, Severity::Error)]);
        // 2.96 mm is within tolerance of 3 mm, 2.9 mm is not.
        boxes.bleed = Some(rect(0.04, 0.04, 235.96, 183.96));
        assert!(codes_of(&boxes, &expect).is_empty());
        boxes.bleed = Some(rect(0.1, 0.0, 236.0, 184.0));
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_BLEED_TOO_SMALL, Severity::Error)]);
        // No bleed expected: nothing to complain about.
        boxes.bleed = Some(rect(3.0, 3.0, 233.0, 181.0));
        assert!(codes_of(&boxes, &Expectations::home()).is_empty());
    }

    #[test]
    fn missing_boxes() {
        let expect = Expectations::professional();
        let mut boxes = card_boxes();
        boxes.trim = None;
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_TRIM_MISSING, Severity::Error)]);
        boxes.media = None;
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_MEDIA_MISSING, Severity::Error)]);
        // Without a BleedBox the MediaBox stands in: a warning, and the bleed is still measured.
        let mut boxes = card_boxes();
        boxes.bleed = None;
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_BLEED_MISSING, Severity::Warning)]);
        boxes.media = Some(rect(2.0, 2.0, 234.0, 182.0));
        assert_eq!(
            codes_of(&boxes, &expect),
            vec![
                (codes::BOX_BLEED_MISSING, Severity::Warning),
                (codes::BOX_BLEED_TOO_SMALL, Severity::Error)
            ]
        );
    }

    #[test]
    fn nesting_violations() {
        let expect = Expectations::professional();
        // Trim pokes out of the bleed box: reported once, without a second bleed finding.
        let mut boxes = card_boxes();
        boxes.trim = Some(rect(-1.0, 3.0, 233.0, 181.0));
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_NOT_NESTED, Severity::Error)]);
        // Bleed box larger than the media box.
        let mut boxes = card_boxes();
        boxes.bleed = Some(rect(-2.0, -2.0, 238.0, 186.0));
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_NOT_NESTED, Severity::Error)]);
        // A crop box that cuts into the bleed.
        let mut boxes = card_boxes();
        boxes.crop = Some(rect(3.0, 3.0, 233.0, 181.0));
        assert_eq!(
            codes_of(&boxes, &expect),
            vec![(codes::BOX_CROP_CLIPS_BLEED, Severity::Warning)]
        );
        // Trim and art box together.
        let mut boxes = card_boxes();
        boxes.art = boxes.trim;
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_TRIM_AND_ART, Severity::Warning)]);
    }

    #[test]
    fn trim_size_tolerance_and_user_unit() {
        let expect = Expectations::professional().with_trim(230.0, 178.0);
        let mut boxes = card_boxes();
        boxes.trim = Some(rect(3.0, 3.0, 233.04, 181.04));
        assert!(codes_of(&boxes, &expect).is_empty());
        boxes.trim = Some(rect(3.0, 3.0, 232.8, 181.0));
        assert_eq!(codes_of(&boxes, &expect), vec![(codes::BOX_TRIM_SIZE, Severity::Error)]);
        // Portrait instead of landscape: the message says so.
        let swapped = PageBoxes {
            media: Some(rect(0.0, 0.0, 184.0, 236.0)),
            bleed: Some(rect(0.0, 0.0, 184.0, 236.0)),
            trim: Some(rect(3.0, 3.0, 181.0, 233.0)),
            ..card_boxes()
        };
        let mut sink = Sink::default();
        check_boxes(&swapped, &expect, 1, &mut sink);
        let findings = sink.into_sorted();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("swapped"), "{}", findings[0].message);
        // UserUnit scales everything: half-size boxes with UserUnit 2 are the same card.
        let half =
            |r: RectPt| RectPt { x0: r.x0 / 2.0, y0: r.y0 / 2.0, x1: r.x1 / 2.0, y1: r.y1 / 2.0 };
        let base = card_boxes();
        let scaled = PageBoxes {
            media: base.media.map(half),
            bleed: base.bleed.map(half),
            trim: base.trim.map(half),
            user_unit: 2.0,
            ..base
        };
        assert!(codes_of(&scaled, &expect).is_empty());
    }

    #[test]
    fn artwork_must_fill_the_bleed_where_it_reaches_the_trim() {
        let expect = Expectations::professional();
        let boxes = card_boxes();
        let warn = |painted: RectPt| {
            let mut sink = Sink::default();
            check_bleed_painted(&boxes, Some(painted), &expect, 1, &mut sink);
            sink.into_sorted()
                .into_iter()
                .map(|f| (f.code, f.severity, f.message))
                .collect::<Vec<_>>()
        };
        // Full bleed, and artwork with a white margin all round: fine.
        assert!(warn(rect(0.0, 0.0, 236.0, 184.0)).is_empty());
        assert!(warn(rect(20.0, 20.0, 200.0, 150.0)).is_empty());
        // Exactly to the trim line on every side.
        let got = warn(rect(3.0, 3.0, 233.0, 181.0));
        assert_eq!(got.len(), 1);
        assert_eq!((got[0].0, got[0].1), (codes::BLEED_NOT_PAINTED, Severity::Warning));
        for side in ["left (0.0 mm)", "right (0.0 mm)", "bottom (0.0 mm)", "top (0.0 mm)"] {
            assert!(got[0].2.contains(side), "{}", got[0].2);
        }
        // One millimetre into the bleed on the right only.
        let got = warn(rect(0.0, 0.0, 234.0, 184.0));
        assert_eq!(got.len(), 1);
        assert!(got[0].2.contains("right (1.0 mm)") && !got[0].2.contains("left"), "{}", got[0].2);
        // 2.9 mm is within the slack.
        assert!(warn(rect(0.1, 0.1, 235.9, 183.9)).is_empty());
        // When the page offers less bleed than expected (reported elsewhere), filling what
        // there is suffices.
        let mut narrow = card_boxes();
        narrow.bleed = Some(rect(2.0, 2.0, 234.0, 182.0));
        let mut sink = Sink::default();
        check_bleed_painted(&narrow, Some(rect(2.0, 2.0, 234.0, 182.0)), &expect, 1, &mut sink);
        assert!(sink.into_sorted().is_empty());
        // No bleed expected, nothing painted, or no trim box: nothing to say.
        let mut sink = Sink::default();
        check_bleed_painted(
            &boxes,
            Some(rect(3.0, 3.0, 233.0, 181.0)),
            &Expectations::home(),
            1,
            &mut sink,
        );
        check_bleed_painted(&boxes, None, &expect, 1, &mut sink);
        let mut no_trim = card_boxes();
        no_trim.trim = None;
        check_bleed_painted(&no_trim, Some(rect(3.0, 3.0, 233.0, 181.0)), &expect, 1, &mut sink);
        assert!(sink.into_sorted().is_empty());
    }

    #[test]
    fn pages_must_share_one_size() {
        let a = card_boxes();
        let mut b = card_boxes();
        b.trim = Some(rect(3.0, 3.0, 223.0, 181.0));
        let mut sink = Sink::default();
        check_same_size(&[a, a, b], &mut sink);
        let findings = sink.into_sorted();
        assert_eq!(findings.len(), 1);
        assert_eq!((findings[0].code, findings[0].page), (codes::PAGE_SIZE_MISMATCH, Some(3)));
        assert!(findings[0].message.starts_with("the trim size is 220.00 × 178.00 mm but page 1"));
        // A page without a trim box is not compared by its media box against other trims,
        // and the remaining pages are compared among themselves.
        let mut no_trim = card_boxes();
        no_trim.trim = None;
        let mut sink = Sink::default();
        check_same_size(&[no_trim, a, a], &mut sink);
        assert!(sink.into_sorted().is_empty());
        let mut sink = Sink::default();
        check_same_size(&[no_trim, a, b], &mut sink);
        let findings = sink.into_sorted();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("but page 2 has"), "{}", findings[0].message);
        // A different sheet with the same trim is a mismatch too.
        let mut bigger_sheet = card_boxes();
        bigger_sheet.media = Some(rect(-10.0, -10.0, 246.0, 194.0));
        let mut sink = Sink::default();
        check_same_size(&[a, bigger_sheet], &mut sink);
        let findings = sink.into_sorted();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.starts_with("the media size is 256.00 × 204.00 mm"));
        let mut sink = Sink::default();
        check_same_size(&[], &mut sink);
        check_same_size(&[a], &mut sink);
        assert!(sink.into_sorted().is_empty());
    }

    #[test]
    fn page_tree_walk_handles_inheritance_cycles_and_junk() {
        let mut doc = Document::with_version("1.6");
        let root = doc.new_object_id();
        let inner = doc.new_object_id();
        let p1 = doc.add_object(dictionary! { "Type" => "Page", "Parent" => inner });
        let p2 = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => inner,
            "MediaBox" => vec![0.into(), 0.into(), 10.into(), 10.into()],
            "Rotate" => 90,
        });
        let p3 = doc.add_object(dictionary! { "Parent" => root });
        doc.objects.insert(
            inner,
            Object::Dictionary(dictionary! {
                "Type" => "Pages", "Parent" => root, "Count" => 2,
                // A cycle back to the root, a dangling reference and a number, between pages.
                "Kids" => vec![p1.into(), root.into(), p2.into(), Object::Reference((999, 0)), 7.into()],
                "Rotate" => 180,
            }),
        );
        doc.objects.insert(
            root,
            Object::Dictionary(dictionary! {
                "Type" => "Pages", "Count" => 3,
                "Kids" => vec![inner.into(), p3.into(), p1.into()],
                "MediaBox" => vec![0.into(), 0.into(), 100.into(), 200.into()],
                "Resources" => dictionary! { "Font" => dictionary! {} },
            }),
        );
        let catalog = dictionary! { "Type" => "Catalog", "Pages" => root };
        let mut sink = Sink::default();
        let pages = collect(&doc, &catalog, &mut sink);
        assert_eq!(pages.len(), 3, "p1, p2, p3 in order; p1 is not visited twice");
        let mut box_sink = Sink::default();
        let b1 = read_boxes(&doc, &pages[0], 1, &mut box_sink);
        let b2 = read_boxes(&doc, &pages[1], 2, &mut box_sink);
        let b3 = read_boxes(&doc, &pages[2], 3, &mut box_sink);
        assert_eq!(b1.media.map(|m| (m.width(), m.height())), Some((100.0, 200.0)));
        assert_eq!(b1.rotate, 180);
        assert_eq!(b2.media.map(|m| (m.width(), m.height())), Some((10.0, 10.0)));
        assert_eq!(b2.rotate, 90);
        assert_eq!(b3.rotate, 0);
        assert!(pages.iter().all(|p| p.resources.is_some()));
        let findings = sink.into_sorted();
        assert!(findings.iter().all(|f| f.code == codes::PAGE_TREE_INVALID));
        // The cycle to the root, the two kids that are not dictionaries (one message), and
        // the shared page p1.
        assert_eq!(findings.len(), 3, "{findings:?}");
    }

    #[test]
    fn deep_page_trees_are_cut_off() {
        let mut doc = Document::with_version("1.6");
        let mut child = doc.add_object(dictionary! { "Type" => "Page" });
        for _ in 0..500 {
            child = doc.add_object(dictionary! { "Type" => "Pages", "Kids" => vec![child.into()] });
        }
        let catalog = dictionary! { "Pages" => child };
        let mut sink = Sink::default();
        let pages = collect(&doc, &catalog, &mut sink);
        assert!(pages.is_empty());
        let findings = sink.into_sorted();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("too deeply"));
    }

    #[test]
    fn malformed_boxes_are_reported() {
        let mut doc = Document::with_version("1.6");
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), 100.into(), 100.into()],
            "TrimBox" => vec![0.into(), 0.into(), 100.into()],
            "BleedBox" => vec![5.into(), 5.into(), 5.into(), 50.into()],
            "ArtBox" => Object::string_literal("nope"),
            "UserUnit" => -3,
            "Rotate" => 450,
        });
        let pages_id =
            doc.add_object(dictionary! { "Type" => "Pages", "Kids" => vec![page.into()] });
        let catalog = dictionary! { "Pages" => pages_id };
        let mut sink = Sink::default();
        let pages = collect(&doc, &catalog, &mut sink);
        let boxes = read_boxes(&doc, &pages[0], 1, &mut sink);
        assert!(
            boxes.media.is_some()
                && boxes.trim.is_none()
                && boxes.bleed.is_none()
                && boxes.art.is_none()
        );
        assert_eq!(boxes.user_unit, 1.0);
        assert_eq!(boxes.rotate, 90);
        let codes: Vec<_> = sink.into_sorted().into_iter().map(|f| f.code).collect();
        assert_eq!(
            codes,
            vec![
                codes::BOX_INVALID,
                codes::BOX_INVALID,
                codes::BOX_INVALID,
                codes::BOX_INVALID,
                codes::PAGE_ROTATED
            ]
        );
    }
}
