//! Validated geometry of one spine: the panel with and without bleed, the band across
//! the spine and the stretch along it in which text may be set.

use ebc_scene::Rect;

use crate::error::SpineError;
use crate::{SpineOptions, SpineRequest};

/// Widest spine accepted, in millimetres (`ebc-layout` stops at 40 mm).
pub(crate) const MAX_SPINE_WIDTH_MM: f64 = 200.0;
/// Tallest spine accepted (bleed included), in millimetres (`ebc-layout` stops at 300 mm).
pub(crate) const MAX_SPINE_HEIGHT_MM: f64 = 2_000.0;

/// The measurements every part of the designer works from. All lengths in millimetres,
/// in page coordinates (origin top-left of the trim box, y down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Geometry {
    /// The spine panel extended by the bleed at the top and bottom (what the background
    /// must cover).
    pub(crate) rect: Rect,
    /// The spine panel at trim size.
    pub(crate) trim: Rect,
    /// Bleed at the top and at the bottom.
    pub(crate) bleed: f64,
    /// Width of the spine, crease to crease.
    pub(crate) width: f64,
    /// The x coordinate of the centre line of the spine.
    pub(crate) center_x: f64,
    /// Width of the band between the two safe margins (0 when the margins meet).
    pub(crate) band: f64,
    /// The y coordinate where the live length starts (top).
    pub(crate) live_top: f64,
    /// The y coordinate where the live length ends (bottom).
    pub(crate) live_bottom: f64,
}

impl Geometry {
    /// Validates the request's geometry.
    pub(crate) fn new(req: &SpineRequest<'_>, opts: &SpineOptions) -> Result<Self, SpineError> {
        let r = req.rect;
        let bad = |what: &str| Err(SpineError::InvalidGeometry(what.to_owned()));
        if !(r.x.is_finite() && r.y.is_finite() && r.w.is_finite() && r.h.is_finite()) {
            return bad("the spine rectangle has a coordinate that is not a finite number");
        }
        if !(r.w > 0.0 && r.h > 0.0) {
            return bad("the spine rectangle is empty");
        }
        if r.w > MAX_SPINE_WIDTH_MM || r.h > MAX_SPINE_HEIGHT_MM {
            return bad("the spine rectangle is larger than any printable card");
        }
        if r.x.abs() > 1.0e6 || r.y.abs() > 1.0e6 {
            return bad("the spine rectangle is absurdly far from the page origin");
        }
        if !(req.bleed.is_finite() && req.bleed >= 0.0) {
            return bad("the bleed is negative or not a finite number");
        }
        if 2.0 * req.bleed >= r.h {
            return bad("the bleed leaves no spine between the top and bottom trim edges");
        }
        if !(req.safe_margin.is_finite() && req.safe_margin >= 0.0) {
            return bad("the safe margin is negative or not a finite number");
        }
        let trim = Rect::new(r.x, r.y + req.bleed, r.w, r.h - 2.0 * req.bleed);
        let band = (r.w - 2.0 * req.safe_margin).max(0.0);
        let end = opts.end_margin.min(trim.h / 2.0);
        Ok(Self {
            rect: r,
            trim,
            bleed: req.bleed,
            width: r.w,
            center_x: r.x + r.w / 2.0,
            band,
            live_top: trim.y + end,
            live_bottom: trim.bottom() - end,
        })
    }

    /// Length available to the text along the spine.
    pub(crate) fn live_len(&self) -> f64 {
        (self.live_bottom - self.live_top).max(0.0)
    }

    /// The rectangle in which every bit of ink of the text must lie: the band between
    /// the safe margins, between the head and foot margins.
    pub(crate) fn text_area(&self) -> Rect {
        Rect::new(self.center_x - self.band / 2.0, self.live_top, self.band, self.live_len())
    }
}
