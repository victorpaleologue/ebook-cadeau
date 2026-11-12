//! The card itself: two cover panels and a spine, on both faces of the sheet.

use ebc_scene::{Point, Rect, Size};

use crate::error::LayoutError;

/// One face of the printed sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// The jacket side: **back cover | spine | front cover** (page 1).
    Outside,
    /// The reverse side: **inside-front | inside-spine | inside-back** (page 2).
    Inside,
}

/// The three panels of one face of the card, left to right, in page coordinates
/// (millimetres, origin at the top-left corner of the trim box, y down).
///
/// What `left` and `right` *are* depends on the face — see [`CardFormat::outside`] and
/// [`CardFormat::inside`]. The panels tile the spread exactly: `left.right() == spine.x`,
/// `spine.right() == right.x`, and `right.right()` is the spread width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Panels {
    /// The left-hand cover panel.
    pub left: Rect,
    /// The spine, between the two creases.
    pub spine: Rect,
    /// The right-hand cover panel.
    pub right: Rect,
}

impl Panels {
    /// The panels as `[left, spine, right]`.
    #[must_use]
    pub fn to_array(&self) -> [Rect; 3] {
        [self.left, self.spine, self.right]
    }

    /// The rectangle covered by the three panels together (the trim box of the spread).
    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.left.union(&self.spine).union(&self.right)
    }

    /// The x coordinate of the vertical centre line of each panel, as `[left, spine, right]`.
    ///
    /// Titles are centred on these lines; the spine's is where a spine title's baseline
    /// box is centred.
    #[must_use]
    pub fn center_x(&self) -> [f64; 3] {
        [self.left.center().x, self.spine.center().x, self.right.center().x]
    }

    /// The panels extended into the bleed on their **outer** edges only: the left panel
    /// grows to the left, top and bottom, the spine to the top and bottom, the right panel
    /// to the right, top and bottom. Shared edges (the creases) do not move, so the three
    /// rectangles still tile, now covering the whole bleed box.
    ///
    /// This is the rectangle a full-bleed background or cover image must fill (and be
    /// clipped to). A negative or non-finite `bleed` is treated as zero.
    #[must_use]
    pub fn with_bleed(&self, bleed: f64) -> Self {
        let b = if bleed.is_finite() { bleed.max(0.0) } else { 0.0 };
        let grow_v = |r: Rect| Rect::new(r.x, r.y - b, r.w, r.h + 2.0 * b);
        let l = grow_v(self.left);
        let r = grow_v(self.right);
        Self {
            left: Rect::new(l.x - b, l.y, l.w + b, l.h),
            spine: grow_v(self.spine),
            right: Rect::new(r.x, r.y, r.w + b, r.h),
        }
    }
}

/// The finished size of the card: one cover panel and the width of the spine.
///
/// The flat (unfolded) card — the *spread* — is `cover | spine | cover`, so it is
/// `2 × cover.w + spine_width` wide and `cover.h` high. Both creases run top to bottom.
///
/// The fields are public so that a format can be tweaked in place; anything that turns a
/// format into geometry on a page ([`new_page`](crate::new_page), [`marks`](crate::marks),
/// …) re-validates it, so an out-of-range value never produces a broken page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardFormat {
    /// Size of one cover panel (front or back), in millimetres.
    pub cover: Size,
    /// Distance between the two creases, in millimetres.
    pub spine_width: f64,
}

impl Default for CardFormat {
    /// [`CardFormat::poche`]: the only common book trim that prints on A4 and US Letter
    /// at 100 % with bleed and marks.
    fn default() -> Self {
        Self::poche()
    }
}

impl CardFormat {
    /// The default spine width: 10 mm, wide enough for a title and a stock option of
    /// "double crease" folder products at European online printers.
    pub const SPINE_DEFAULT_MM: f64 = 10.0;
    /// Smallest accepted cover width or height.
    pub const COVER_MIN_MM: f64 = 50.0;
    /// Largest accepted cover width or height.
    pub const COVER_MAX_MM: f64 = 300.0;
    /// Smallest accepted spine width. Below about 5 mm a spine cannot carry a title
    /// (see [`CardFormat::spine_can_carry_text`]), below 3 mm two creases merge into one.
    pub const SPINE_MIN_MM: f64 = 3.0;
    /// Largest accepted spine width.
    pub const SPINE_MAX_MM: f64 = 40.0;
    /// Printers' consensus threshold below which no text should be set on a spine
    /// (4–6 mm depending on the printer; 5 mm is the usual figure).
    pub const SPINE_TEXT_MIN_MM: f64 = 5.0;
    /// Names accepted by [`CardFormat::by_name`], in the order of [`CardFormat::presets`].
    pub const PRESET_NAMES: [&'static str; 4] = ["poche", "a6", "digest", "a5"];

    const fn preset(w: f64, h: f64) -> Self {
        Self { cover: Size::new(w, h), spine_width: Self::SPINE_DEFAULT_MM }
    }

    /// French *poche* (mass-market paperback): 110 × 178 mm covers, 10 mm spine.
    /// Spread 230 × 178 mm — fits A4 and US Letter landscape with bleed and marks.
    #[must_use]
    pub const fn poche() -> Self {
        Self::preset(110.0, 178.0)
    }

    /// ISO A6: 105 × 148 mm covers, 10 mm spine. Spread 220 × 148 mm.
    #[must_use]
    pub const fn a6() -> Self {
        Self::preset(105.0, 148.0)
    }

    /// *Digest* / French *broché* trade size: 130 × 200 mm covers, 10 mm spine.
    /// Spread 270 × 200 mm — needs A3/SRA3 or a professional printer.
    #[must_use]
    pub const fn digest() -> Self {
        Self::preset(130.0, 200.0)
    }

    /// ISO A5: 148 × 210 mm covers, 10 mm spine. Spread 306 × 210 mm — wider than A4.
    #[must_use]
    pub const fn a5() -> Self {
        Self::preset(148.0, 210.0)
    }

    /// The presets with their names, smallest sheet requirement first.
    #[must_use]
    pub const fn presets() -> [(&'static str, Self); 4] {
        [("poche", Self::poche()), ("a6", Self::a6()), ("digest", Self::digest()), ("a5", Self::a5())]
    }

    /// Looks a preset up by name (case-insensitive, surrounding whitespace ignored).
    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::presets().into_iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, f)| f)
    }

    /// A custom format. Each cover dimension must be within
    /// [`COVER_MIN_MM`](Self::COVER_MIN_MM)`..=`[`COVER_MAX_MM`](Self::COVER_MAX_MM) and the
    /// spine within [`SPINE_MIN_MM`](Self::SPINE_MIN_MM)`..=`[`SPINE_MAX_MM`](Self::SPINE_MAX_MM).
    ///
    /// # Errors
    /// [`LayoutError::CoverOutOfRange`] or [`LayoutError::SpineOutOfRange`]; NaN and
    /// infinities are out of range.
    pub fn custom(cover: Size, spine_width: f64) -> Result<Self, LayoutError> {
        let format = Self { cover, spine_width };
        format.validate()?;
        Ok(format)
    }

    /// The same covers with another spine width.
    ///
    /// # Errors
    /// As [`CardFormat::custom`].
    pub fn with_spine(self, spine_width: f64) -> Result<Self, LayoutError> {
        Self::custom(self.cover, spine_width)
    }

    /// Checks the ranges documented on [`CardFormat::custom`].
    ///
    /// # Errors
    /// [`LayoutError::CoverOutOfRange`] or [`LayoutError::SpineOutOfRange`].
    pub fn validate(&self) -> Result<(), LayoutError> {
        let cover_range = Self::COVER_MIN_MM..=Self::COVER_MAX_MM;
        for (dimension, value) in [("width", self.cover.w), ("height", self.cover.h)] {
            // `contains` is false for NaN, so NaN is rejected here too.
            if !cover_range.contains(&value) {
                return Err(LayoutError::CoverOutOfRange {
                    dimension,
                    value,
                    min: Self::COVER_MIN_MM,
                    max: Self::COVER_MAX_MM,
                });
            }
        }
        if !(Self::SPINE_MIN_MM..=Self::SPINE_MAX_MM).contains(&self.spine_width) {
            return Err(LayoutError::SpineOutOfRange {
                value: self.spine_width,
                min: Self::SPINE_MIN_MM,
                max: Self::SPINE_MAX_MM,
            });
        }
        Ok(())
    }

    /// Whether the spine is wide enough to carry a title
    /// (≥ [`SPINE_TEXT_MIN_MM`](Self::SPINE_TEXT_MIN_MM)).
    #[must_use]
    pub fn spine_can_carry_text(&self) -> bool {
        self.spine_width >= Self::SPINE_TEXT_MIN_MM
    }

    /// Size of the flat card: `2 × cover.w + spine_width` by `cover.h`. This is the trim
    /// size of both pages.
    #[must_use]
    pub fn spread(&self) -> Size {
        // Same association as the right panel's right edge, so the panels tile exactly.
        Size::new(self.cover.w + self.spine_width + self.cover.w, self.cover.h)
    }

    /// The trim box of the spread in page coordinates (at the origin).
    #[must_use]
    pub fn trim_rect(&self) -> Rect {
        let s = self.spread();
        Rect::new(0.0, 0.0, s.w, s.h)
    }

    fn panel_rects(&self) -> Panels {
        let (cw, sw, h) = (self.cover.w, self.spine_width, self.cover.h);
        Panels {
            left: Rect::new(0.0, 0.0, cw, h),
            spine: Rect::new(cw, 0.0, sw, h),
            right: Rect::new(cw + sw, 0.0, cw, h),
        }
    }

    /// Panels of the **outside** face (page 1), for a left-to-right book:
    /// `left` = **back cover**, `spine`, `right` = **front cover**.
    #[must_use]
    pub fn outside(&self) -> Panels {
        self.panel_rects()
    }

    /// Panels of the **inside** face (page 2), i.e. the reverse of the sheet after it has
    /// been flipped about its vertical axis:
    /// `left` = **inside-front** (the back of the front cover), `spine` = inside of the
    /// spine, `right` = **inside-back** (the back of the back cover).
    ///
    /// Because the spread is symmetric the rectangles are numerically those of
    /// [`outside`](Self::outside), but each one sits behind the *mirrored* outside panel:
    /// `inside().left` is printed on the back of `outside().right`, and vice versa
    /// (see [`mirror_x`](Self::mirror_x)).
    #[must_use]
    pub fn inside(&self) -> Panels {
        self.panel_rects()
    }

    /// Panels of the given face.
    #[must_use]
    pub fn panels(&self, face: Face) -> Panels {
        match face {
            Face::Outside => self.outside(),
            Face::Inside => self.inside(),
        }
    }

    /// The back cover (outside face, left).
    #[must_use]
    pub fn back(&self) -> Rect {
        self.outside().left
    }

    /// The spine (outside face, centre).
    #[must_use]
    pub fn spine(&self) -> Rect {
        self.outside().spine
    }

    /// The front cover (outside face, right).
    #[must_use]
    pub fn front(&self) -> Rect {
        self.outside().right
    }

    /// The inside of the front cover (inside face, left): the first thing the recipient
    /// sees when opening the card.
    #[must_use]
    pub fn inside_front(&self) -> Rect {
        self.inside().left
    }

    /// The inside of the spine (inside face, centre).
    #[must_use]
    pub fn inside_spine(&self) -> Rect {
        self.inside().spine
    }

    /// The inside of the back cover (inside face, right).
    #[must_use]
    pub fn inside_back(&self) -> Rect {
        self.inside().right
    }

    /// The x coordinates of the two creases, left to right. They are the same on both
    /// faces.
    #[must_use]
    pub fn fold_x(&self) -> [f64; 2] {
        [self.cover.w, self.cover.w + self.spine_width]
    }

    /// The two crease lines across the spread, each as `(top, bottom)` end points on the
    /// trim edges.
    #[must_use]
    pub fn fold_lines(&self) -> [(Point, Point); 2] {
        let h = self.cover.h;
        self.fold_x().map(|x| (Point::new(x, 0.0), Point::new(x, h)))
    }

    /// Maps a rectangle on one face to the rectangle that lies directly behind it on the
    /// other face (a mirror about the vertical centre line of the spread). Useful to keep
    /// something on the inside away from what shows through or is embossed on the outside,
    /// and to reason about duplex alignment.
    #[must_use]
    pub fn mirror_x(&self, r: Rect) -> Rect {
        Rect::new(self.spread().w - r.right(), r.y, r.w, r.h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_the_documented_sizes() {
        assert_eq!(CardFormat::poche().cover, Size::new(110.0, 178.0));
        assert_eq!(CardFormat::a6().cover, Size::new(105.0, 148.0));
        assert_eq!(CardFormat::digest().cover, Size::new(130.0, 200.0));
        assert_eq!(CardFormat::a5().cover, Size::new(148.0, 210.0));
        for (name, f) in CardFormat::presets() {
            assert_eq!(f.spine_width, 10.0, "{name}");
            assert_eq!(f.validate(), Ok(()), "{name}");
            assert_eq!(CardFormat::by_name(name), Some(f));
            assert!(f.spine_can_carry_text());
        }
        assert_eq!(CardFormat::PRESET_NAMES.len(), CardFormat::presets().len());
        for (n, (m, _)) in CardFormat::PRESET_NAMES.iter().zip(CardFormat::presets()) {
            assert_eq!(*n, m);
        }
        assert_eq!(CardFormat::default(), CardFormat::poche());
    }

    #[test]
    fn by_name_is_lenient_about_case_and_space_but_not_about_names() {
        assert_eq!(CardFormat::by_name("  POCHE "), Some(CardFormat::poche()));
        assert_eq!(CardFormat::by_name("A5"), Some(CardFormat::a5()));
        assert_eq!(CardFormat::by_name(""), None);
        assert_eq!(CardFormat::by_name("a4"), None);
        assert_eq!(CardFormat::by_name("poche\0"), None);
    }

    #[test]
    fn spread_and_folds_of_the_default_card() {
        let f = CardFormat::poche();
        assert_eq!(f.spread(), Size::new(230.0, 178.0));
        assert_eq!(f.fold_x(), [110.0, 120.0]);
        assert_eq!(f.back(), Rect::new(0.0, 0.0, 110.0, 178.0));
        assert_eq!(f.spine(), Rect::new(110.0, 0.0, 10.0, 178.0));
        assert_eq!(f.front(), Rect::new(120.0, 0.0, 110.0, 178.0));
        assert_eq!(f.inside_front(), Rect::new(0.0, 0.0, 110.0, 178.0));
        assert_eq!(f.inside_spine(), f.spine());
        assert_eq!(f.inside_back(), Rect::new(120.0, 0.0, 110.0, 178.0));
        assert_eq!(f.trim_rect(), Rect::new(0.0, 0.0, 230.0, 178.0));
        let [a, b] = f.fold_lines();
        assert_eq!(a, (Point::new(110.0, 0.0), Point::new(110.0, 178.0)));
        assert_eq!(b, (Point::new(120.0, 0.0), Point::new(120.0, 178.0)));
    }

    #[test]
    fn custom_accepts_the_limits_and_rejects_just_beyond() {
        let ok = |w, h, s| CardFormat::custom(Size::new(w, h), s);
        assert!(ok(50.0, 50.0, 3.0).is_ok());
        assert!(ok(300.0, 300.0, 40.0).is_ok());
        assert!(matches!(
            ok(49.999, 100.0, 10.0),
            Err(LayoutError::CoverOutOfRange { dimension: "width", .. })
        ));
        assert!(matches!(
            ok(100.0, 300.001, 10.0),
            Err(LayoutError::CoverOutOfRange { dimension: "height", .. })
        ));
        assert!(matches!(ok(100.0, 100.0, 2.999), Err(LayoutError::SpineOutOfRange { .. })));
        assert!(matches!(ok(100.0, 100.0, 40.001), Err(LayoutError::SpineOutOfRange { .. })));
    }

    #[test]
    fn custom_rejects_hostile_numbers() {
        let bad = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -110.0, 0.0, -0.0, f64::MAX];
        for v in bad {
            assert!(CardFormat::custom(Size::new(v, 178.0), 10.0).is_err(), "w = {v}");
            assert!(CardFormat::custom(Size::new(110.0, v), 10.0).is_err(), "h = {v}");
            assert!(CardFormat::custom(Size::new(110.0, 178.0), v).is_err(), "spine = {v}");
            assert!(CardFormat::poche().with_spine(v).is_err());
        }
        // Width is reported before height, height before spine.
        let all_bad = CardFormat { cover: Size::new(f64::NAN, f64::NAN), spine_width: f64::NAN };
        assert!(matches!(
            all_bad.validate(),
            Err(LayoutError::CoverOutOfRange { dimension: "width", .. })
        ));
    }

    #[test]
    fn errors_have_readable_messages() {
        let e = CardFormat::custom(Size::new(20.0, 178.0), 10.0).unwrap_err();
        assert_eq!(e.to_string(), "cover width of 20 mm is outside the supported range 50..=300 mm");
        let e = CardFormat::poche().with_spine(1.0).unwrap_err();
        assert_eq!(e.to_string(), "spine width of 1 mm is outside the supported range 3..=40 mm");
    }

    #[test]
    fn spine_text_threshold() {
        assert!(!CardFormat::poche().with_spine(4.9).unwrap().spine_can_carry_text());
        assert!(CardFormat::poche().with_spine(5.0).unwrap().spine_can_carry_text());
    }

    #[test]
    fn with_bleed_extends_outer_edges_only() {
        let f = CardFormat::poche();
        let p = f.outside().with_bleed(3.0);
        assert_eq!(p.left, Rect::new(-3.0, -3.0, 113.0, 184.0));
        assert_eq!(p.spine, Rect::new(110.0, -3.0, 10.0, 184.0));
        assert_eq!(p.right, Rect::new(120.0, -3.0, 113.0, 184.0));
        assert_eq!(p.bounds(), Rect::new(-3.0, -3.0, 236.0, 184.0));
        // Hostile bleed values leave the panels alone.
        for b in [f64::NAN, f64::INFINITY, -5.0] {
            assert_eq!(f.outside().with_bleed(b), f.outside(), "bleed {b}");
        }
    }

    #[test]
    fn center_lines() {
        assert_eq!(CardFormat::poche().outside().center_x(), [55.0, 115.0, 175.0]);
    }

    #[test]
    fn mirror_is_an_involution_and_swaps_the_covers() {
        let f = CardFormat::custom(Size::new(123.4, 201.7), 7.3).unwrap();
        let r = Rect::new(12.5, 20.0, 30.0, 40.0);
        let back = f.mirror_x(f.mirror_x(r));
        assert!((back.x - r.x).abs() < 1e-9 && back.y == r.y && back.w == r.w && back.h == r.h);
        let m = f.mirror_x(f.back());
        assert!((m.x - f.front().x).abs() < 1e-9 && m.w == f.front().w);
    }
}
