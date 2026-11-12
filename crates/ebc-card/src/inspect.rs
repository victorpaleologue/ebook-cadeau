//! Measuring a composed [`Document`]: the bounding boxes of items as they are painted.
//!
//! The composer uses this to verify its own output (nothing may be painted over the
//! cover), and tests and tools can use it to check that text stays inside safe areas.

use ebc_scene::{Document, FontId, Item, Point, Rect, Transform, pt_to_mm};
use ebc_text::{Font, ShapedGlyph, ShapedText, outline_bounds};

/// Computes the bounding boxes of the items of one document, parsing each font at most
/// once.
#[derive(Debug)]
pub struct Inspector<'a> {
    doc: &'a Document,
    fonts: Vec<(FontId, Option<Font>)>,
}

impl<'a> Inspector<'a> {
    /// An inspector for `doc`.
    #[must_use]
    pub fn new(doc: &'a Document) -> Self {
        Self { doc, fonts: Vec::new() }
    }

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

    /// The bounding box of what `item` paints, in the coordinates the item is placed in
    /// (page millimetres for a top-level item); `None` when it paints nothing.
    ///
    /// * Paths: the exact box of the outline, grown by half the stroke width when stroked.
    /// * Images: the destination rectangle.
    /// * Glyph runs: the exact box of the ink of the glyphs (not the advance box).
    /// * Groups: the union of the children, cut to the clip path's box if there is one,
    ///   then mapped through the group's transform.
    pub fn bounds(&mut self, item: &Item) -> Option<Rect> {
        match item {
            Item::Path { path, fill, stroke } => {
                let bounds = outline_bounds(path).or_else(|| path.control_bounds())?;
                match stroke {
                    Some(s) => Some(bounds.inflate(pt_to_mm(s.width_pt.max(0.0)) / 2.0)),
                    None if fill.is_some() => Some(bounds),
                    None => None,
                }
            }
            Item::Image { rect, .. } => Some(*rect),
            Item::Glyphs { font, size_pt, origin, glyphs, .. } => {
                let font = self.font(*font)?;
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
                run.ink_bounds(&font, *origin)
            }
            Item::Group { transform, clip, items, .. } => {
                let mut bounds: Option<Rect> = None;
                for child in items {
                    if let Some(b) = self.bounds(child) {
                        bounds = Some(bounds.map_or(b, |acc| acc.union(&b)));
                    }
                }
                let mut bounds = bounds?;
                if let Some(clip) = clip {
                    let clip = outline_bounds(clip).or_else(|| clip.control_bounds())?;
                    bounds = bounds.intersect(&clip)?;
                }
                Some(transform_rect(transform, bounds))
            }
        }
    }

    /// The bounding boxes of every leaf item of a page (groups are descended into, with
    /// their clips and transforms applied), in painting order, each with the name of the
    /// innermost layer it belongs to.
    pub fn leaves(&mut self, page_index: usize) -> Vec<Leaf> {
        let mut out = Vec::new();
        if let Some(page) = self.doc.pages.get(page_index) {
            for item in &page.items {
                self.collect(item, Transform::IDENTITY, None, None, &mut out);
            }
        }
        out
    }

    fn collect(
        &mut self,
        item: &Item,
        ctm: Transform,
        clip: Option<Rect>,
        layer: Option<&str>,
        out: &mut Vec<Leaf>,
    ) {
        if let Item::Group { transform, clip: group_clip, layer: name, items } = item {
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
            let layer = name.as_deref().or(layer);
            for child in items {
                self.collect(child, ctm, clip, layer, out);
            }
            return;
        }
        let Some(bounds) = self.bounds(item) else { return };
        let bounds = transform_rect(&ctm, bounds);
        let bounds = match clip {
            Some(c) => match bounds.intersect(&c) {
                Some(visible) => visible,
                None => return,
            },
            None => bounds,
        };
        let kind = match item {
            Item::Path { .. } => LeafKind::Path,
            Item::Image { .. } => LeafKind::Image,
            Item::Glyphs { .. } => LeafKind::Text,
            Item::Group { .. } => return,
        };
        out.push(Leaf { bounds, kind, layer: layer.map(str::to_owned) });
    }
}

/// What a [`Leaf`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafKind {
    /// A filled or stroked path (grounds, rules, the QR code, outlined text, marks).
    Path,
    /// A raster image.
    Image,
    /// A run of embedded-font text.
    Text,
}

/// One painted item of a page, flattened out of its groups.
#[derive(Debug, Clone, PartialEq)]
pub struct Leaf {
    /// Its visible bounding box in page millimetres.
    pub bounds: Rect,
    /// What it is.
    pub kind: LeafKind,
    /// The innermost named layer it belongs to, e.g. `"marks"`.
    pub layer: Option<String>,
}

/// The bounding box of what `item` paints (see [`Inspector::bounds`]).
#[must_use]
pub fn item_bounds(doc: &Document, item: &Item) -> Option<Rect> {
    Inspector::new(doc).bounds(item)
}

/// The axis-aligned box of a rectangle mapped through a transform.
fn transform_rect(t: &Transform, r: Rect) -> Rect {
    let corners = [
        t.apply(Point::new(r.x, r.y)),
        t.apply(Point::new(r.right(), r.y)),
        t.apply(Point::new(r.x, r.bottom())),
        t.apply(Point::new(r.right(), r.bottom())),
    ];
    let (mut x0, mut y0, mut x1, mut y1) = (corners[0].x, corners[0].y, corners[0].x, corners[0].y);
    for p in &corners[1..] {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_scene::{Color, Path, Stroke};

    #[test]
    fn paths_images_and_groups() {
        let doc = Document::default();
        let mut inspector = Inspector::new(&doc);
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(inspector.bounds(&Item::filled_rect(rect, Color::BLACK)), Some(rect));
        let stroked = Item::stroke(Path::rect(rect), Stroke::solid(Color::BLACK, 72.0 / 25.4));
        assert_eq!(inspector.bounds(&stroked), Some(rect.inflate(0.5)));
        // A path that is neither filled nor stroked paints nothing.
        let nothing = Item::Path { path: Path::rect(rect), fill: None, stroke: None };
        assert_eq!(inspector.bounds(&nothing), None);

        let clipped = Item::clipped(
            Rect::new(0.0, 0.0, 20.0, 100.0),
            vec![Item::filled_rect(rect, Color::BLACK)],
        );
        assert_eq!(inspector.bounds(&clipped), Some(Rect::new(10.0, 20.0, 10.0, 40.0)));

        let moved = Item::group(
            Transform::translate(5.0, -5.0),
            vec![Item::filled_rect(rect, Color::BLACK)],
        );
        assert_eq!(inspector.bounds(&moved), Some(Rect::new(15.0, 15.0, 30.0, 40.0)));
        assert_eq!(item_bounds(&doc, &Item::group(Transform::IDENTITY, Vec::new())), None);
    }

    #[test]
    fn leaves_carry_their_layer_and_clip() {
        let mut doc = Document::default();
        let mut page = ebc_scene::Page::new(ebc_scene::Size::new(100.0, 100.0), 0.0, 0.0);
        page.items.push(Item::filled_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::BLACK));
        page.items.push(Item::layer(
            "marks",
            vec![Item::clipped(
                Rect::new(0.0, 0.0, 5.0, 5.0),
                vec![Item::filled_rect(Rect::new(2.0, 2.0, 10.0, 10.0), Color::BLACK)],
            )],
        ));
        doc.pages.push(page);
        let leaves = Inspector::new(&doc).leaves(0);
        assert_eq!(leaves.len(), 2);
        assert_eq!(leaves[0].layer, None);
        assert_eq!(leaves[1].layer.as_deref(), Some("marks"));
        assert_eq!(leaves[1].bounds, Rect::new(2.0, 2.0, 3.0, 3.0));
        assert!(Inspector::new(&doc).leaves(7).is_empty());
    }
}
