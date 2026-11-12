//! Vector paths.

use crate::geom::{Point, Rect, Transform};

/// One segment of a [`Path`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PathSeg {
    /// Starts a new sub-path at the given point.
    MoveTo(Point),
    /// Straight line to the given point.
    LineTo(Point),
    /// Quadratic Bézier curve: control point, end point.
    QuadTo(Point, Point),
    /// Cubic Bézier curve: first control point, second control point, end point.
    CubicTo(Point, Point, Point),
    /// Closes the current sub-path with a straight line to its start.
    Close,
}

/// How the inside of a path is determined when filling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FillRule {
    /// Non-zero winding rule (the PDF `f` operator).
    #[default]
    NonZero,
    /// Even-odd rule (the PDF `f*` operator).
    EvenOdd,
}

/// A vector path made of one or more sub-paths, in page millimetres.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path {
    /// The segments, in drawing order.
    pub segs: Vec<PathSeg>,
}

impl Path {
    /// Creates an empty path.
    #[must_use]
    pub const fn new() -> Self {
        Self { segs: Vec::new() }
    }

    /// Returns `true` if the path has no segments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segs.is_empty()
    }

    /// Starts a new sub-path.
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.segs.push(PathSeg::MoveTo(Point::new(x, y)));
        self
    }

    /// Adds a straight line.
    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.segs.push(PathSeg::LineTo(Point::new(x, y)));
        self
    }

    /// Adds a quadratic Bézier curve.
    pub fn quad_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) -> &mut Self {
        self.segs.push(PathSeg::QuadTo(Point::new(cx, cy), Point::new(x, y)));
        self
    }

    /// Adds a cubic Bézier curve.
    pub fn cubic_to(
        &mut self,
        c1x: f64,
        c1y: f64,
        c2x: f64,
        c2y: f64,
        x: f64,
        y: f64,
    ) -> &mut Self {
        self.segs.push(PathSeg::CubicTo(
            Point::new(c1x, c1y),
            Point::new(c2x, c2y),
            Point::new(x, y),
        ));
        self
    }

    /// Closes the current sub-path.
    pub fn close(&mut self) -> &mut Self {
        self.segs.push(PathSeg::Close);
        self
    }

    /// Appends a closed rectangle sub-path.
    pub fn push_rect(&mut self, r: Rect) -> &mut Self {
        self.move_to(r.x, r.y)
            .line_to(r.right(), r.y)
            .line_to(r.right(), r.bottom())
            .line_to(r.x, r.bottom())
            .close()
    }

    /// A closed rectangle.
    #[must_use]
    pub fn rect(r: Rect) -> Self {
        let mut p = Self::new();
        p.push_rect(r);
        p
    }

    /// An open straight line from `a` to `b`.
    #[must_use]
    pub fn line(a: Point, b: Point) -> Self {
        let mut p = Self::new();
        p.move_to(a.x, a.y).line_to(b.x, b.y);
        p
    }

    /// A closed circle approximated by four cubic Bézier curves.
    #[must_use]
    pub fn circle(center: Point, radius: f64) -> Self {
        // Standard kappa for a quarter-circle approximation.
        const K: f64 = 0.552_284_749_830_793_4;
        let (cx, cy, r, k) = (center.x, center.y, radius, radius * K);
        let mut p = Self::new();
        p.move_to(cx + r, cy)
            .cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r)
            .cubic_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy)
            .cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r)
            .cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy)
            .close();
        p
    }

    /// Appends all segments of `other`.
    pub fn extend(&mut self, other: &Path) -> &mut Self {
        self.segs.extend_from_slice(&other.segs);
        self
    }

    /// Returns a copy of the path with every point mapped through `t`.
    #[must_use]
    pub fn transformed(&self, t: &Transform) -> Self {
        let segs = self
            .segs
            .iter()
            .map(|s| match *s {
                PathSeg::MoveTo(p) => PathSeg::MoveTo(t.apply(p)),
                PathSeg::LineTo(p) => PathSeg::LineTo(t.apply(p)),
                PathSeg::QuadTo(c, p) => PathSeg::QuadTo(t.apply(c), t.apply(p)),
                PathSeg::CubicTo(c1, c2, p) => {
                    PathSeg::CubicTo(t.apply(c1), t.apply(c2), t.apply(p))
                }
                PathSeg::Close => PathSeg::Close,
            })
            .collect();
        Self { segs }
    }

    /// Bounding box of all on-curve and control points (a conservative bound for curves),
    /// or `None` for an empty path.
    #[must_use]
    pub fn control_bounds(&self) -> Option<Rect> {
        let mut pts = self.segs.iter().flat_map(|s| {
            let (a, b, c) = match *s {
                PathSeg::MoveTo(p) | PathSeg::LineTo(p) => (Some(p), None, None),
                PathSeg::QuadTo(c, p) => (Some(c), Some(p), None),
                PathSeg::CubicTo(c1, c2, p) => (Some(c1), Some(c2), Some(p)),
                PathSeg::Close => (None, None, None),
            };
            [a, b, c].into_iter().flatten()
        });
        let first = pts.next()?;
        let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
        for p in pts {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        Some(Rect::new(x0, y0, x1 - x0, y1 - y0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_path_has_five_segments_and_right_bounds() {
        let r = Rect::new(1.0, 2.0, 3.0, 4.0);
        let p = Path::rect(r);
        assert_eq!(p.segs.len(), 5);
        assert_eq!(p.control_bounds(), Some(r));
    }

    #[test]
    fn circle_bounds() {
        let b = Path::circle(Point::new(10.0, 10.0), 2.0).control_bounds().expect("non-empty");
        assert!((b.x - 8.0).abs() < 1e-9 && (b.w - 4.0).abs() < 1e-9);
    }

    #[test]
    fn transform_moves_every_point() {
        let p = Path::line(Point::new(0.0, 0.0), Point::new(1.0, 0.0));
        let q = p.transformed(&Transform::translate(5.0, 5.0));
        assert_eq!(q.control_bounds(), Some(Rect::new(5.0, 5.0, 1.0, 0.0)));
        assert!(Path::new().control_bounds().is_none());
    }
}
