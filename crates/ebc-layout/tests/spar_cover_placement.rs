//! Sparring tests: the documented recipe for a full-bleed cover image.
//!
//! The crate docs (`lib.rs` "Typical use", `place_image`, `Panels::with_bleed`) tell the
//! composer to do this:
//!
//! ```text
//! let front = format.outside().with_bleed(profile.bleed).right;   // bleed on 3 sides only
//! let image_rect = place_image(px, front, Fit::Cover);            // centred in `front`
//! ```
//!
//! `front` is the panel grown on its three *outer* edges, so its centre is `bleed / 2` to
//! the right of the panel's centre, and `place_image` centres the image on it. After
//! trimming, the card shows a crop of the cover that is shifted by `bleed / 2` towards the
//! fore-edge: the fold cuts almost nothing off the cover while the fore-edge loses the
//! whole bleed. A cover with a border or frame (most French literary covers) comes out
//! visibly lopsided.

#![allow(clippy::unwrap_used)]

use ebc_layout::{CardFormat, Fit, PrintProfile, Sheet, place_image};
use ebc_scene::Rect;

/// How much of the image is hidden on each side of the *trimmed* panel: `(left, right)`.
fn hidden(panel: Rect, placed: Rect) -> (f64, f64) {
    (panel.x - placed.x, placed.right() - panel.right())
}

/// A cover scan with the same proportions as the panel it goes on, placed by the
/// documented recipe, must sit centred on that panel: what the fold hides on one side
/// equals what the trim removes on the other.
#[test]
fn the_documented_recipe_centres_the_cover_on_its_panel() {
    let mut wrong = Vec::new();
    for (name, format) in CardFormat::presets() {
        for profile in [PrintProfile::professional_eu(), PrintProfile::home(Sheet::A3)] {
            let bled = format.outside().with_bleed(profile.bleed);
            for (which, panel, target) in
                [("front", format.front(), bled.right), ("back", format.back(), bled.left)]
            {
                // Pixel dimensions in the panel's own proportions (10 px per mm).
                let px = ((panel.w * 10.0) as u32, (panel.h * 10.0) as u32);
                let placed = place_image(px, target, Fit::Cover);
                assert!(placed.contains_rect(&target, 1e-9), "{name} {which}: bleed not covered");
                let (left, right) = hidden(panel, placed);
                let off_centre = placed.center().x - panel.center().x;
                if (left - right).abs() > 0.1 {
                    wrong.push(format!(
                        "{name} {which} ({}): hides {left:.2} mm on the left and {right:.2} mm \
                         on the right; the cover is {off_centre:+.2} mm off the panel's centre",
                        profile.name
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the recipe from the crate docs does not centre the cover:\n{}",
        wrong.join("\n")
    );
}

/// The visible consequence on the default card: a cover with a frame drawn 6 mm inside
/// its edges. On the finished card the frame should be the same distance from the fold
/// as from the fore-edge.
#[test]
fn a_framed_cover_keeps_equal_margins_on_the_finished_poche_card() {
    let format = CardFormat::poche();
    let profile = PrintProfile::professional_eu();
    let panel = format.front();
    let target = format.outside().with_bleed(profile.bleed).right;
    // 1100 x 1780 px scan of a 110 x 178 mm cover; frame 60 px (6 mm) inside each edge.
    let (px_w, px_h, frame_px) = (1100u32, 1780u32, 60.0);
    let placed = place_image((px_w, px_h), target, Fit::Cover);
    let mm_per_px = placed.w / f64::from(px_w);
    let frame_left = placed.x + frame_px * mm_per_px;
    let frame_right = placed.right() - frame_px * mm_per_px;
    let margin_at_fold = frame_left - panel.x;
    let margin_at_fore_edge = panel.right() - frame_right;
    assert!(
        (margin_at_fold - margin_at_fore_edge).abs() < 0.5,
        "frame is {margin_at_fold:.2} mm from the fold but {margin_at_fore_edge:.2} mm from \
         the fore-edge after trimming"
    );
}
