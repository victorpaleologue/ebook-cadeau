//! Print geometry for the eBook Cadeau card: formats with a spine, panels, bleed and safe
//! areas, page boxes, printer's marks, and one-up placement on a home-printer sheet.
//!
//! The card is a flat sheet — the *spread* — creased twice so that it wraps like a book
//! jacket:
//!
//! ```text
//!            fold_x[0]  fold_x[1]
//!   0            |          |                spread.w
//!   +------------+----------+------------+   y = 0      outside (page 1), left to right:
//!   |            |          |            |                back cover | spine | front cover
//!   |    left    |  spine   |   right    |
//!   |            |          |            |              inside (page 2), left to right:
//!   +------------+----------+------------+   y = h        inside-front | spine | inside-back
//! ```
//!
//! Everything is in **millimetres**, in the page coordinate system of [`ebc_scene`]:
//! origin at the top-left corner of the trim box, x to the right, y down. Artwork that
//! bleeds and all printer's marks therefore have negative coordinates or coordinates
//! beyond the spread.
//!
//! # Typical use
//!
//! ```
//! use ebc_layout::{
//!     CardFormat, Fit, PrintProfile, check_marks_fit, marks, new_page, place_image,
//!     place_image_bleed, safe_rect,
//! };
//!
//! let format = CardFormat::poche();                 // 110 x 178 mm covers, 10 mm spine
//! let profile = PrintProfile::professional_eu();    // 3 mm bleed, marks, PDF/X-4
//!
//! let mut outside = new_page(&format, &profile)?;   // TrimBox 230 x 178 mm
//! assert_eq!(outside.media_box().w, 230.0 + 2.0 * (3.0 + 10.0));
//! check_marks_fit(&format, &profile)?;              // the cut and fold marks have room
//!
//! // Where text — and, by default, the cover image — may go: 5 mm inside the panel.
//! let safe = safe_rect(format.front(), &profile);
//! assert_eq!((safe.x, safe.w, safe.h), (125.0, 100.0, 168.0));
//! // The whole cover, uncropped, letterboxed in that box on a flat ground.
//! let image_rect = place_image((1400, 2200), safe, Fit::Contain);
//! assert!(safe.contains_rect(&image_rect, 1e-9));
//!
//! // A full-bleed cover instead (only when the cover may be cropped): centred on the
//! // panel, clipped to the panel plus bleed on its three outer edges.
//! let clip = format.outside().with_bleed(profile.bleed).right;
//! let image_rect = place_image_bleed((1400, 2200), format.front(), profile.bleed);
//! assert!(image_rect.contains_rect(&clip, 1e-9));
//! assert_eq!(image_rect.center().x, format.front().center().x);
//!
//! // ... add artwork to `outside.items`, then the marks on top:
//! outside.items.push(marks(&format, &profile));
//! # Ok::<(), ebc_layout::LayoutError>(())
//! ```
//!
//! # Placing the cover
//!
//! * **Default — the whole cover, uncropped.** Letterbox it in the safe area of its panel
//!   (5 mm inside the trim and the crease) on a flat ground that fills the bleed:
//!   `place_image(px, safe_rect(format.front(), &profile), Fit::Contain)`. Nothing of the
//!   cover is cut off and nothing of it lies in the bleed, which is what
//!   `docs/research/03-print-requirements.md` (implication 14) asks for unless cropping
//!   the cover has been authorised.
//! * **Full bleed, centred — only when the cover may be cropped.** Use
//!   [`place_image_bleed`] and clip to the panel grown into the bleed on its outer edges
//!   ([`Panels::with_bleed`]). The crop is then symmetric about the panel: a frame or a
//!   centred title stays centred on the finished card. Do **not** pass the `with_bleed`
//!   rectangle to [`place_image`]: that rectangle is not centred on the panel (it grows on
//!   three sides only), so the cover would end up half a bleed off-centre, losing next to
//!   nothing at the fold and the whole bleed at the fore-edge.
//! * **Full bleed, nothing lost at the fold.** If the cover's spine-side edge has to lie
//!   exactly on the crease (the reading of implication 14 for an authorised crop), say so
//!   explicitly: `place_image_aligned(px, clip, Fit::Cover, 0.0, 0.5)` for the front
//!   cover (`1.0` for the back), `clip` being the `with_bleed` rectangle. The price is the
//!   one above — the artwork then sits about half a bleed towards the fore-edge, which
//!   shows on a cover with a frame — but here it is a decision, not an accident.
//!
//! # Profiles
//!
//! * [`PrintProfile::upload_eu`] — page = trim + 3 mm bleed, no marks: what web-to-print
//!   portals want (most of them reject marks).
//! * [`PrintProfile::professional_eu`] — the same plus a 10 mm slug with solid crop marks
//!   and dashed fold marks in the registration colour, for printers working from a quote.
//! * [`PrintProfile::home`] — the spread centred on an A4/Letter/… sheet with light grey
//!   cut and fold guides and a scale ruler, all kept inside the printable area. Two
//!   things the composer must do for it: print the duplex instruction
//!   ([`SheetPlacement::duplex_flip`] — "flip on the short edge" for every preset), and
//!   call [`check_marks_fit`]. The PDF page is the sheet itself (`MediaBox` = A4
//!   landscape for the default card), with the spread centred on it — see [`new_page`].
//!
//! # Seeing it
//!
//! `cargo run -p ebc-layout --example layout_svg -- poche home_a4 out.svg` draws the page
//! boxes, the [`guides`] overlay and the [`marks`] of any preset and profile as an SVG.
//!
//! # Not implemented
//!
//! Multi-up imposition (several cards per sheet), creep, and work-and-turn layouts: the
//! only "imposition" here is one card centred on one sheet.

#![forbid(unsafe_code)]

mod error;
mod format;
mod guides;
mod image;
mod marks;
mod page;
mod profile;

pub use error::LayoutError;
pub use format::{CardFormat, Face, Panels};
pub use guides::{GUIDES_LAYER, guides, info_entries, safe_panels, safe_rect, spine_safe_rect};
pub use image::{Fit, effective_ppi, place_image, place_image_aligned, place_image_bleed};
pub use marks::{
    CAPTION_MIN_HEIGHT_MM, COLOR_BAR_PATCHES, FOLD_GUIDE_MAX_MM, FOLD_MARK_DASH_MM,
    FURNITURE_BAND_MAX_MM, MARK_BLEED_GAP_MM, MARKS_LAYER, MIN_MARK_LENGTH_MM, ScaleRuler,
    caption_areas, check_marks_fit, color_bar, crop_marks, fold_marks, marks,
    registration_target_centers, registration_targets, scale_ruler,
};
pub use page::{DuplexFlip, PageGeometry, SheetPlacement, new_page, new_page_for, page_geometry};
pub use profile::{
    GUIDE_GRAY_LEVEL, HOME_PRINTABLE_MARGIN_MM, MAX_MARGIN_MM, MAX_MARK_WEIGHT_PT, MAX_SHEET_MM,
    MarkStyle, Marks, PrintProfile, Resolution, Sheet,
};
