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
//! use ebc_layout::{CardFormat, Fit, PrintProfile, marks, new_page, place_image, safe_rect};
//!
//! let format = CardFormat::poche();                 // 110 x 178 mm covers, 10 mm spine
//! let profile = PrintProfile::professional_eu();    // 3 mm bleed, marks, PDF/X-4
//!
//! let mut outside = new_page(&format, &profile)?;   // TrimBox 230 x 178 mm
//! assert_eq!(outside.media_box().w, 230.0 + 2.0 * (3.0 + 10.0));
//!
//! // Where the cover image goes: the front panel plus bleed on its outer edges.
//! let front = format.outside().with_bleed(profile.bleed).right;
//! let image_rect = place_image((1400, 2200), front, Fit::Cover);
//! assert!(image_rect.contains_rect(&front, 1e-9));
//!
//! // Where text may go on the back cover.
//! let text_area = safe_rect(format.back(), &profile);
//! assert_eq!(text_area.x, 4.0);
//!
//! // ... add artwork to `outside.items`, then the marks on top:
//! outside.items.push(marks(&format, &profile));
//! # Ok::<(), ebc_layout::LayoutError>(())
//! ```
//!
//! # Profiles
//!
//! * [`PrintProfile::professional_eu`] — page = trim + 3 mm bleed + 10 mm slug with crop
//!   marks, fold marks and registration targets in the registration colour.
//! * [`PrintProfile::upload_eu`] — the same without marks or slug, for web-to-print
//!   portals that reject marks.
//! * [`PrintProfile::home`] — the spread centred on an A4/Letter/… sheet with light grey
//!   cut and fold guides kept inside the printable area. See [`new_page`] for the one
//!   thing [`ebc_scene::Page`] cannot express there.
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
pub use image::{Fit, effective_ppi, place_image, place_image_aligned};
pub use marks::{
    CAPTION_MIN_HEIGHT_MM, COLOR_BAR_PATCHES, FOLD_GUIDE_DASH_MM, FOLD_GUIDE_MAX_MM,
    FURNITURE_BAND_MAX_MM, MARKS_LAYER, MIN_MARK_LENGTH_MM, ScaleRuler, caption_areas, color_bar,
    crop_marks, fold_marks, marks, registration_target_centers, registration_targets, scale_ruler,
};
pub use page::{PageGeometry, SheetPlacement, new_page, new_page_for, page_geometry};
pub use profile::{
    GUIDE_GRAY_LEVEL, MAX_MARGIN_MM, MAX_MARK_WEIGHT_PT, MAX_SHEET_MM, MarkStyle, Marks,
    PrintProfile, Resolution, Sheet,
};
