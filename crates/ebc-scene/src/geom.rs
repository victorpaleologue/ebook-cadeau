//! Points, sizes, rectangles and affine transforms, in millimetres.

/// Millimetres in one inch.
pub const MM_PER_INCH: f64 = 25.4;
/// PostScript/PDF points in one millimetre (1 pt = 1/72 in).
pub const PT_PER_MM: f64 = 72.0 / MM_PER_INCH;
/// Millimetres in one PostScript/PDF point.
pub const MM_PER_PT: f64 = MM_PER_INCH / 72.0;

/// Converts millimetres to PDF points.
#[must_use]
pub fn mm_to_pt(mm: f64) -> f64 {
    mm * PT_PER_MM
}

/// Converts PDF points to millimetres.
#[must_use]
pub fn pt_to_mm(pt: f64) -> f64 {
    pt * MM_PER_PT
}

/// A point on the page, in millimetres (origin: top-left of the trim box, y down).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    /// Horizontal position, growing to the right.
    pub x: f64,
    /// Vertical position, growing downwards.
    pub y: f64,
}

impl Point {
    /// Creates a point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Returns this point moved by `(dx, dy)`.
    #[must_use]
    pub fn offset(self, dx: f64, dy: f64) -> Self {
        Self { x: self.x + dx, y: self.y + dy }
    }
}

/// A width and a height, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Size {
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl Size {
    /// Creates a size.
    #[must_use]
    pub const fn new(w: f64, h: f64) -> Self {
        Self { w, h }
    }

    /// Width divided by height.
    #[must_use]
    pub fn aspect(self) -> f64 {
        self.w / self.h
    }
}

/// An axis-aligned rectangle, in millimetres. `(x, y)` is the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width (non-negative).
    pub w: f64,
    /// Height (non-negative).
    pub h: f64,
}

impl Rect {
    /// Creates a rectangle from its top-left corner and size.
    #[must_use]
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Creates a rectangle from two opposite corners given in any order.
    #[must_use]
    pub fn from_corners(a: Point, b: Point) -> Self {
        let (x0, x1) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (y0, y1) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        Self { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
    }

    /// Right edge.
    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    /// Bottom edge.
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    /// Centre point.
    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Size of the rectangle.
    #[must_use]
    pub fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }

    /// Grows the rectangle by `d` on every side (shrinks if `d` is negative).
    /// The result never has a negative width or height.
    #[must_use]
    pub fn inflate(&self, d: f64) -> Self {
        self.inflate_xy(d, d)
    }

    /// Grows the rectangle by `dx` on the left and right and `dy` on the top and bottom.
    #[must_use]
    pub fn inflate_xy(&self, dx: f64, dy: f64) -> Self {
        let w = (self.w + 2.0 * dx).max(0.0);
        let h = (self.h + 2.0 * dy).max(0.0);
        let cx = self.x + self.w / 2.0;
        let cy = self.y + self.h / 2.0;
        Self { x: cx - w / 2.0, y: cy - h / 2.0, w, h }
    }

    /// Returns `true` if `p` lies inside the rectangle (edges included).
    #[must_use]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }

    /// Returns `true` if `other` lies entirely inside this rectangle, with tolerance `eps`.
    #[must_use]
    pub fn contains_rect(&self, other: &Rect, eps: f64) -> bool {
        other.x >= self.x - eps
            && other.y >= self.y - eps
            && other.right() <= self.right() + eps
            && other.bottom() <= self.bottom() + eps
    }

    /// Intersection of two rectangles, or `None` if they do not overlap.
    #[must_use]
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = self.right().min(other.right());
        let y1 = self.bottom().min(other.bottom());
        (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
    }

    /// Smallest rectangle containing both rectangles.
    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = self.right().max(other.right());
        let y1 = self.bottom().max(other.bottom());
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }
}

/// A 2-D affine transform mapping `(x, y)` to `(a·x + c·y + e, b·x + d·y + f)`.
///
/// This is the same component order as the PDF `cm` operator and the SVG/Canvas
/// `matrix(a, b, c, d, e, f)`. Because page y points *down*, a positive rotation
/// angle turns shapes **clockwise** on the page.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    /// x scale / cos.
    pub a: f64,
    /// y shear / sin.
    pub b: f64,
    /// x shear / -sin.
    pub c: f64,
    /// y scale / cos.
    pub d: f64,
    /// x translation (mm).
    pub e: f64,
    /// y translation (mm).
    pub f: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// The identity transform.
    pub const IDENTITY: Self = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };

    /// A translation by `(dx, dy)` millimetres.
    #[must_use]
    pub const fn translate(dx: f64, dy: f64) -> Self {
        Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: dx, f: dy }
    }

    /// A scale about the origin.
    #[must_use]
    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self { a: sx, b: 0.0, c: 0.0, d: sy, e: 0.0, f: 0.0 }
    }

    /// A rotation about the origin by `degrees`, clockwise on the page.
    #[must_use]
    pub fn rotate(degrees: f64) -> Self {
        let (s, c) = degrees.to_radians().sin_cos();
        Self { a: c, b: s, c: -s, d: c, e: 0.0, f: 0.0 }
    }

    /// A rotation about `center` by `degrees`, clockwise on the page.
    #[must_use]
    pub fn rotate_about(degrees: f64, center: Point) -> Self {
        Self::translate(center.x, center.y)
            .then_inner(Self::rotate(degrees))
            .then_inner(Self::translate(-center.x, -center.y))
    }

    /// Composes two transforms: the result applies `inner` **first**, then `self`.
    ///
    /// `t.then_inner(u).apply(p) == t.apply(u.apply(p))`.
    #[must_use]
    pub fn then_inner(self, inner: Self) -> Self {
        Self {
            a: self.a * inner.a + self.c * inner.b,
            b: self.b * inner.a + self.d * inner.b,
            c: self.a * inner.c + self.c * inner.d,
            d: self.b * inner.c + self.d * inner.d,
            e: self.a * inner.e + self.c * inner.f + self.e,
            f: self.b * inner.e + self.d * inner.f + self.f,
        }
    }

    /// Applies the transform to a point.
    #[must_use]
    pub fn apply(&self, p: Point) -> Point {
        Point::new(self.a * p.x + self.c * p.y + self.e, self.b * p.x + self.d * p.y + self.f)
    }

    /// Determinant of the linear part. Zero means the transform is not invertible.
    #[must_use]
    pub fn det(&self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    /// The inverse transform, or `None` if this transform is singular.
    #[must_use]
    pub fn invert(&self) -> Option<Self> {
        let det = self.det();
        if det.abs() < 1e-12 {
            return None;
        }
        let a = self.d / det;
        let b = -self.b / det;
        let c = -self.c / det;
        let d = self.a / det;
        let e = -(a * self.e + c * self.f);
        let f = -(b * self.e + d * self.f);
        Some(Self { a, b, c, d, e, f })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9
    }

    #[test]
    fn unit_conversion_round_trips() {
        assert!((mm_to_pt(25.4) - 72.0).abs() < 1e-12);
        assert!((pt_to_mm(mm_to_pt(123.456)) - 123.456).abs() < 1e-12);
    }

    #[test]
    fn rect_basics() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.right(), 40.0);
        assert_eq!(r.bottom(), 60.0);
        assert_eq!(r.center(), Point::new(25.0, 40.0));
        assert_eq!(r.inflate(3.0), Rect::new(7.0, 17.0, 36.0, 46.0));
        assert_eq!(r.inflate(-100.0).w, 0.0);
        assert!(r.contains(Point::new(10.0, 20.0)));
        assert!(!r.contains(Point::new(9.9, 20.0)));
        assert!(r.inflate(3.0).contains_rect(&r, 0.0));
        assert_eq!(
            Rect::from_corners(Point::new(5.0, 9.0), Point::new(1.0, 2.0)),
            Rect::new(1.0, 2.0, 4.0, 7.0)
        );
    }

    #[test]
    fn rect_intersection_and_union() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        assert_eq!(a.intersect(&b), Some(Rect::new(5.0, 5.0, 5.0, 5.0)));
        assert_eq!(a.union(&b), Rect::new(0.0, 0.0, 15.0, 15.0));
        assert_eq!(a.intersect(&Rect::new(20.0, 20.0, 1.0, 1.0)), None);
    }

    #[test]
    fn rotation_is_clockwise_on_a_y_down_page() {
        // +x axis rotated by 90 degrees clockwise (y down) points to +y (down).
        let p = Transform::rotate(90.0).apply(Point::new(1.0, 0.0));
        assert!(close(p, Point::new(0.0, 1.0)), "{p:?}");
    }

    #[test]
    fn composition_order() {
        let t = Transform::translate(10.0, 0.0);
        let s = Transform::scale(2.0, 2.0);
        // scale first, then translate
        assert!(close(t.then_inner(s).apply(Point::new(1.0, 1.0)), Point::new(12.0, 2.0)));
        // translate first, then scale
        assert!(close(s.then_inner(t).apply(Point::new(1.0, 1.0)), Point::new(22.0, 2.0)));
    }

    #[test]
    fn rotate_about_keeps_center_fixed() {
        let c = Point::new(7.0, -3.0);
        let t = Transform::rotate_about(37.0, c);
        assert!(close(t.apply(c), c));
    }

    #[test]
    fn inverse_round_trips() {
        let t = Transform::translate(3.0, 4.0)
            .then_inner(Transform::rotate(30.0))
            .then_inner(Transform::scale(2.0, 0.5));
        let inv = t.invert().expect("invertible");
        let p = Point::new(1.25, -8.5);
        assert!(close(inv.apply(t.apply(p)), p));
        assert!(Transform::scale(0.0, 1.0).invert().is_none());
    }
}
