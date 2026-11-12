//! From a format and a profile to the boxes of a PDF page.

use ebc_scene::{Page, Rect, Size};

use crate::error::LayoutError;
use crate::format::{CardFormat, Face};
use crate::profile::PrintProfile;

/// Slack allowed when testing whether a spread fits a sheet, in millimetres, so that a
/// spread that fits *exactly* is not rejected because of floating-point rounding.
const FIT_EPSILON_MM: f64 = 1e-6;

/// The duplex ("print on both sides") setting with which the inside page lands upright
/// behind the outside page.
///
/// The card is laid out for a sheet that is turned over about the axis parallel to its
/// creases — like the page of a book. Print dialogs name that by the edge of the *paper*
/// the turn is parallel to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DuplexFlip {
    /// "Flip on short edge" (*retourner sur le bord court*): the creases are parallel to
    /// the short edge of the paper, i.e. the sheet is used in landscape. This is the case
    /// of every preset — and it is **not** the usual print-dialog default.
    ShortEdge,
    /// "Flip on long edge" (*retourner sur le bord long*): the creases are parallel to the
    /// long edge of the paper, i.e. the sheet is used in portrait (tall custom cards only).
    LongEdge,
}

impl DuplexFlip {
    const fn for_landscape(landscape: bool) -> Self {
        if landscape { Self::ShortEdge } else { Self::LongEdge }
    }
}

/// Where the physical sheet lies around the spread, for sheet-based (home) profiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SheetPlacement {
    /// The whole sheet in page coordinates, in the chosen orientation, with the spread
    /// centred on it (so its `x` and `y` are negative).
    pub sheet: Rect,
    /// The part of the sheet the printer can print on (`sheet` shrunk by the profile's
    /// `printable_margin`).
    pub printable: Rect,
    /// `true` when the sheet is used with its long edge horizontal. The sheet always takes
    /// the orientation of the spread: landscape when the spread is at least as wide as it
    /// is high (every preset), portrait for a spread that is taller than wide — see
    /// [`page_geometry`].
    pub landscape: bool,
    /// `true` when the page's media box equals the sheet. It can only do so when the
    /// spread's left/right and top/bottom margins on the sheet are equal — see
    /// [`new_page`] for why.
    pub media_is_sheet: bool,
}

impl SheetPlacement {
    /// The duplex setting this sheet must be printed with:
    /// [`DuplexFlip::ShortEdge`] in landscape, [`DuplexFlip::LongEdge`] in portrait.
    ///
    /// With the other setting the trim boxes of the two sides still coincide (the spread
    /// is centred and symmetric), so nothing looks wrong on the uncut sheet — but the
    /// inside is upside down and behind the wrong cover. The composer must therefore print
    /// this instruction on the sheet (e.g. in a [`caption_areas`](crate::caption_areas)
    /// rectangle), next to "print at 100 %".
    #[must_use]
    pub const fn duplex_flip(&self) -> DuplexFlip {
        DuplexFlip::for_landscape(self.landscape)
    }
}

/// The boxes of a card page, in page coordinates (millimetres, origin at the top-left of
/// the trim box, y down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageGeometry {
    /// The trim box: the spread, at the origin.
    pub trim: Rect,
    /// The bleed box: `trim` grown by the profile's bleed.
    pub bleed: Rect,
    /// The media box of the [`Page`] returned by [`new_page`].
    pub media: Rect,
    /// The rectangle marks must stay within: the media box for press profiles, and for
    /// sheet-based profiles the media box shrunk by the printable margin.
    pub mark_area: Rect,
    /// Bleed on every side, in millimetres (`Page::bleed`).
    pub bleed_mm: f64,
    /// Slug on every side, in millimetres (`Page::slug`). For sheet-based profiles it is
    /// derived from the sheet, not taken from the profile.
    pub slug_mm: f64,
    /// How far from the trim edge marks may reach, in millimetres, on every side
    /// (`mark_area` is `trim` grown by this).
    pub mark_reach_mm: f64,
    /// The physical sheet, for sheet-based profiles.
    pub sheet: Option<SheetPlacement>,
}

impl PageGeometry {
    /// Width of the band between the bleed edge and the limit of the mark area, i.e. the
    /// room available for marks on each side, in millimetres.
    #[must_use]
    pub fn mark_band_mm(&self) -> f64 {
        (self.mark_reach_mm - self.bleed_mm).max(0.0)
    }

    /// The duplex setting with which page 2 (inside) lands upright behind page 1
    /// (outside) when this page is printed on both sides of one sheet:
    /// [`DuplexFlip::ShortEdge`] for a landscape page, [`DuplexFlip::LongEdge`] for a
    /// portrait one. A square page counts as landscape.
    ///
    /// The page has the orientation of the spread, and so has the sheet of a sheet-based
    /// profile, so this agrees with [`SheetPlacement::duplex_flip`]. For a press profile
    /// the printer imposes the pages and the value only matters for an office-printer
    /// proof.
    #[must_use]
    pub fn duplex_flip(&self) -> DuplexFlip {
        DuplexFlip::for_landscape(self.media.w >= self.media.h)
    }
}

/// Computes the page boxes for a format printed with a profile.
///
/// # Orientation of the sheet
///
/// A sheet-based profile uses its sheet in the orientation of the spread: landscape when
/// the spread is at least as wide as it is high, portrait otherwise. This is the
/// orientation that leaves the larger margin on the tighter side (so cut and fold marks
/// have the most room), and the only one worth trying: a spread that does not fit the
/// sheet this way round does not fit it the other way round either. It also means that
/// the page, the sheet and the spread always have the same orientation, so a print dialog
/// that rotates the page automatically puts it on the paper the way the layout assumed.
///
/// # Errors
/// * [`LayoutError::CoverOutOfRange`] / [`LayoutError::SpineOutOfRange`] — invalid format;
/// * [`LayoutError::InvalidProfile`] / [`LayoutError::InvalidSheet`] — invalid profile;
/// * [`LayoutError::DoesNotFit`] — the profile has a sheet and the spread plus bleed plus
///   printable margin is larger than the sheet in both orientations.
///
/// A card that fits may still leave no room for its cut and fold marks; ask
/// [`check_marks_fit`](crate::check_marks_fit).
pub fn page_geometry(
    format: &CardFormat,
    profile: &PrintProfile,
) -> Result<PageGeometry, LayoutError> {
    format.validate()?;
    profile.validate()?;
    let spread = format.spread();
    let bleed = profile.bleed;

    let (slug, reach, sheet) = match profile.sheet {
        None => (profile.slug, bleed + profile.slug, None),
        Some(sheet) => {
            let margin = bleed + profile.printable_margin;
            let needed = Size::new(spread.w + 2.0 * margin, spread.h + 2.0 * margin);
            // The sheet takes the orientation of the spread. No need to try the other
            // one: if `w >= h` fits a portrait sheet (short edge S, long edge L), then
            // `w <= S <= L` and `h <= w <= S`, so it fits the landscape sheet too — and
            // symmetrically for a tall spread.
            let landscape = spread.w >= spread.h;
            let size = if landscape { sheet.landscape() } else { sheet.portrait() };
            if needed.w > size.w + FIT_EPSILON_MM || needed.h > size.h + FIT_EPSILON_MM {
                return Err(LayoutError::DoesNotFit { needed, sheet: size });
            }
            // The spread is centred: equal margins left/right and top/bottom.
            let mx = (size.w - spread.w) / 2.0;
            let my = (size.h - spread.h) / 2.0;
            // `ebc_scene::Page` has ONE slug for all four sides, so the media box can only
            // follow the tighter of the two margins (see `new_page`).
            let m = mx.min(my);
            let slug = (m - bleed).max(0.0);
            let reach = (m - profile.printable_margin).max(bleed);
            let sheet_rect = Rect::new(-mx, -my, size.w, size.h);
            let placement = SheetPlacement {
                sheet: sheet_rect,
                printable: sheet_rect.inflate(-profile.printable_margin),
                landscape,
                media_is_sheet: (mx - my).abs() <= FIT_EPSILON_MM,
            };
            (slug, reach, Some(placement))
        }
    };

    // Derive the boxes from the very `Page` the caller gets, so they agree bit for bit.
    let page = Page::new(spread, bleed, slug);
    let trim = page.trim_box();
    let media = page.media_box();
    // Guard against a last-bit overshoot of `trim.inflate(reach)` beyond the media box.
    let mark_area = trim.inflate(reach).intersect(&media).unwrap_or(trim);
    Ok(PageGeometry {
        trim,
        bleed: page.bleed_box(),
        media,
        mark_area,
        bleed_mm: bleed,
        slug_mm: slug,
        mark_reach_mm: reach,
        sheet,
    })
}

/// Creates the empty [`Page`] for one face of the card. Both faces use the same page.
///
/// * **Press profiles** (`profile.sheet == None`): `trim` = the spread, `bleed` and
///   `slug` from the profile, hence `TrimBox` = spread, `BleedBox` = spread + bleed,
///   `MediaBox` = spread + bleed + slug on every side.
/// * **Sheet-based (home) profiles**: `trim` = the spread and `bleed` from the profile
///   again, and the slug is chosen so that the media box is the **largest box centred on
///   the spread that fits the sheet**, the sheet being used in the orientation of the
///   spread (see [`page_geometry`]).
///
/// # Known deviation: the media box is not always the whole sheet
///
/// `docs/architecture.md` wants `MediaBox` = the sheet for home profiles. This function
/// **cannot deliver that** with today's [`ebc_scene::Page`], which describes the media box
/// with a single `slug` applied to all four sides: it can only express a sheet whose
/// left/right margin around the spread equals its top/bottom margin. A 230 × 178 mm poche
/// spread centred on A4 landscape has 33.5 mm left and right but 16 mm above and below.
/// The returned page therefore has a media box that equals the sheet in the tighter
/// direction and is narrower than it in the other — 262 × 210 mm instead of 297 × 210 mm
/// for poche on A4, 267.9 × 215.9 mm instead of 279.4 × 215.9 mm on US Letter. Only when
/// both margins are equal is it exactly the sheet ([`SheetPlacement::media_is_sheet`]).
///
/// What this costs: the two sides only register if the spread is centred on the physical
/// paper. With an off-size page that is left to the PDF viewer and the printer driver. A
/// viewer that centres a smaller page on the paper at "actual size" gives the intended
/// result; one that anchors it to a corner shifts each side by half the missing width
/// (17.5 mm for poche on A4) in opposite directions once the sheet is turned, 35 mm
/// between the two sides. This has not been tested on real viewers or printers. Marks are kept inside the printable area
/// of the real sheet either way.
///
/// The fix is outside this crate: [`ebc_scene::Page`] needs an explicit media box (or a
/// slug per side) and `ebc-pdf` has to write it; `new_page` then only has to pass on
/// [`SheetPlacement::sheet`], which [`page_geometry`] already reports. Until then a host
/// that can set the `MediaBox` itself should use that rectangle.
///
/// # Errors
/// As [`page_geometry`]; notably [`LayoutError::DoesNotFit`] when the spread plus bleed
/// plus printable margin exceeds the sheet (e.g. A5 covers on A4).
pub fn new_page(format: &CardFormat, profile: &PrintProfile) -> Result<Page, LayoutError> {
    let g = page_geometry(format, profile)?;
    Ok(Page::new(g.trim.size(), g.bleed_mm, g.slug_mm))
}

/// [`new_page`] with the page labelled `"outside"` or `"inside"` ([`Face::label`]).
/// The geometry of the two pages is identical.
///
/// # Errors
/// As [`new_page`].
pub fn new_page_for(
    face: Face,
    format: &CardFormat,
    profile: &PrintProfile,
) -> Result<Page, LayoutError> {
    let mut page = new_page(format, profile)?;
    page.label = Some(face.label().to_owned());
    Ok(page)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::profile::Sheet;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn rect_close(a: Rect, b: Rect) -> bool {
        close(a.x, b.x) && close(a.y, b.y) && close(a.w, b.w) && close(a.h, b.h)
    }

    #[test]
    fn professional_page_boxes_for_the_default_card() {
        let page = new_page(&CardFormat::poche(), &PrintProfile::professional_eu()).unwrap();
        assert_eq!(page.trim, Size::new(230.0, 178.0));
        assert_eq!(page.bleed, 3.0);
        assert_eq!(page.slug, 10.0);
        assert_eq!(page.trim_box(), Rect::new(0.0, 0.0, 230.0, 178.0));
        assert_eq!(page.bleed_box(), Rect::new(-3.0, -3.0, 236.0, 184.0));
        assert_eq!(page.media_box(), Rect::new(-13.0, -13.0, 256.0, 204.0));
        assert!(page.items.is_empty());
    }

    #[test]
    fn labelled_pages_differ_only_by_their_label() {
        let (f, p) = (CardFormat::poche(), PrintProfile::home(Sheet::Letter));
        let plain = new_page(&f, &p).unwrap();
        for face in Face::BOTH {
            let mut page = new_page_for(face, &f, &p).unwrap();
            assert_eq!(page.label.as_deref(), Some(face.label()));
            page.label = None;
            assert_eq!(page, plain);
        }
        assert!(new_page_for(Face::Inside, &CardFormat::a5(), &p).is_err());
    }

    #[test]
    fn upload_page_is_trim_plus_bleed() {
        let page = new_page(&CardFormat::a5(), &PrintProfile::upload_eu()).unwrap();
        assert_eq!(page.media_box(), page.bleed_box());
        assert_eq!(page.media_box(), Rect::new(-3.0, -3.0, 312.0, 216.0));
    }

    #[test]
    fn poche_on_a4_is_centred_and_full_height() {
        let (f, p) = (CardFormat::poche(), PrintProfile::home(Sheet::A4));
        let g = page_geometry(&f, &p).unwrap();
        let page = new_page(&f, &p).unwrap();
        assert_eq!(page.media_box(), g.media);
        assert_eq!(page.trim_box(), Rect::new(0.0, 0.0, 230.0, 178.0));
        assert_eq!(page.bleed_box(), Rect::new(-2.0, -2.0, 234.0, 182.0));
        let s = g.sheet.unwrap();
        assert!(s.landscape);
        assert_eq!(s.duplex_flip(), DuplexFlip::ShortEdge);
        assert!(rect_close(s.sheet, Rect::new(-33.5, -16.0, 297.0, 210.0)));
        // 6.35 mm unprintable on every side: 284.3 x 197.3 mm printable.
        assert!(rect_close(s.printable, Rect::new(-27.15, -9.65, 284.3, 197.3)));
        // KNOWN DEVIATION from docs/architecture.md (see `new_page`): the page is not the
        // whole sheet. Top/bottom margin (16 mm) is the tighter one: the media box has
        // the sheet's full height and is centred on the same point, but is narrower.
        assert!(!s.media_is_sheet);
        assert!(rect_close(g.media, Rect::new(-16.0, -16.0, 262.0, 210.0)));
        assert!(close(g.media.h, s.sheet.h));
        assert!(close(g.media.center().x, s.sheet.center().x));
        assert!(close(g.media.center().y, s.sheet.center().y));
        assert!(s.sheet.contains_rect(&g.media, 1e-9));
        assert!(close(page.slug, 14.0));
        // Marks may reach 9.65 mm from the trim: the printable limit above and below.
        assert!(close(g.mark_reach_mm, 9.65));
        assert!(close(g.mark_band_mm(), 7.65));
        assert!(s.printable.contains_rect(&g.mark_area, 1e-9));
    }

    #[test]
    fn poche_on_letter_is_centred_and_full_height() {
        let (f, p) = (CardFormat::poche(), PrintProfile::home(Sheet::Letter));
        let g = page_geometry(&f, &p).unwrap();
        let s = g.sheet.unwrap();
        assert!(s.landscape);
        assert!(rect_close(s.sheet, Rect::new(-24.7, -18.95, 279.4, 215.9)));
        assert!(rect_close(g.media, Rect::new(-18.95, -18.95, 267.9, 215.9)));
        assert!(close(g.media.h, 215.9));
        assert!(s.sheet.contains_rect(&g.media, 1e-9));
        assert!(close(g.mark_reach_mm, 12.6));
        // A quarter-inch frame on Letter: 266.7 x 203.2 mm printable.
        assert!(close(s.printable.w, 266.7) && close(s.printable.h, 203.2));
    }

    #[test]
    fn media_box_is_exactly_the_sheet_when_margins_are_equal() {
        // 210 x 123 mm spread on A4 landscape: 43.5 mm on every side.
        let f = CardFormat::custom(Size::new(100.0, 123.0), 10.0).unwrap();
        let g = page_geometry(&f, &PrintProfile::home(Sheet::A4)).unwrap();
        let s = g.sheet.unwrap();
        assert!(s.media_is_sheet);
        assert!(rect_close(g.media, s.sheet));
        assert!(rect_close(g.media, Rect::new(-43.5, -43.5, 297.0, 210.0)));
        // Same on Letter landscape (279.4 x 215.9): spread 179.4 x 115.9, 50 mm all round.
        let f = CardFormat::custom(Size::new(84.7, 115.9), 10.0).unwrap();
        let g = page_geometry(&f, &PrintProfile::home(Sheet::Letter)).unwrap();
        assert!(g.sheet.unwrap().media_is_sheet);
        assert!(rect_close(g.media, Rect::new(-50.0, -50.0, 279.4, 215.9)));
    }

    #[test]
    fn a5_and_digest_do_not_fit_a4_or_letter() {
        for sheet in [Sheet::A4, Sheet::Letter] {
            let p = PrintProfile::home(sheet);
            for f in [CardFormat::a5(), CardFormat::digest()] {
                match new_page(&f, &p) {
                    Err(LayoutError::DoesNotFit { needed, sheet: s }) => {
                        assert_eq!(s, sheet.landscape());
                        // 2 mm bleed + 6.35 mm unprintable, twice.
                        assert!(close(needed.w, f.spread().w + 16.7));
                        assert!(close(needed.h, f.spread().h + 16.7));
                    }
                    other => panic!("{f:?} on {sheet:?}: {other:?}"),
                }
            }
            // The smaller presets do fit.
            assert!(new_page(&CardFormat::poche(), &p).is_ok());
            assert!(new_page(&CardFormat::a6(), &p).is_ok());
        }
        let e = new_page(&CardFormat::a5(), &PrintProfile::home(Sheet::A4)).unwrap_err();
        assert_eq!(
            e.to_string(),
            "the card needs 322.7 x 226.7 mm (spread + bleed + printable margin) \
             but the sheet is only 297.0 x 210.0 mm"
        );
    }

    #[test]
    fn every_preset_fits_a3_and_sra3() {
        for sheet in [Sheet::A3, Sheet::Sra3] {
            for (name, f) in CardFormat::presets() {
                let g = page_geometry(&f, &PrintProfile::home(sheet)).unwrap();
                assert!(g.sheet.unwrap().landscape, "{name}");
            }
        }
    }

    #[test]
    fn exact_fit_is_accepted_and_one_micron_more_is_not() {
        // spread 283 x 196 + 2 * (2 + 5) = 297 x 210 exactly.
        let f = CardFormat::custom(Size::new(136.5, 196.0), 10.0).unwrap();
        let mut p = PrintProfile::home(Sheet::A4);
        p.printable_margin = 5.0;
        let g = page_geometry(&f, &p).unwrap();
        // No room at all for marks, but the page is valid...
        assert!(g.mark_band_mm() < 1e-9);
        assert!(g.media.contains_rect(&g.bleed, 1e-9));
        // ...and `check_marks_fit` is how a caller finds out.
        assert!(matches!(crate::check_marks_fit(&f, &p), Err(LayoutError::MarksDoNotFit { .. })));
        let wider = CardFormat::custom(Size::new(136.5005, 196.0), 10.0).unwrap();
        assert!(matches!(new_page(&wider, &p), Err(LayoutError::DoesNotFit { .. })));
        let taller = CardFormat::custom(Size::new(136.5, 196.001), 10.0).unwrap();
        assert!(matches!(new_page(&taller, &p), Err(LayoutError::DoesNotFit { .. })));
    }

    #[test]
    fn the_sheet_takes_the_orientation_of_the_spread() {
        let home = PrintProfile::home(Sheet::A4);
        // A tall, narrow card: spread 110 x 260 does not fit A4 landscape (210 high) but
        // fits A4 portrait.
        let f = CardFormat::custom(Size::new(50.0, 260.0), 10.0).unwrap();
        let g = page_geometry(&f, &home).unwrap();
        let s = g.sheet.unwrap();
        assert!(!s.landscape);
        assert!(rect_close(s.sheet, Rect::new(-50.0, -18.5, 210.0, 297.0)));
        assert!(close(g.media.h, 297.0));

        // Spread 110 x 195: it would fit A4 landscape too, with 7.5 mm above and below
        // and no room for a single mark. In portrait it has 50 mm on every side.
        let f = CardFormat::custom(Size::new(50.0, 195.0), 10.0).unwrap();
        let g = page_geometry(&f, &home).unwrap();
        let s = g.sheet.unwrap();
        assert!(!s.landscape);
        assert!(rect_close(s.sheet, Rect::new(-50.0, -51.0, 210.0, 297.0)));
        assert!(close(g.mark_reach_mm, 50.0 - 6.35));
        assert_eq!(crate::check_marks_fit(&f, &home), Ok(()));

        // Wider than tall, by however little: landscape. A square spread counts as wide.
        for (cover, landscape) in [
            (Size::new(95.5, 200.0), true),  // 201 x 200
            (Size::new(95.0, 200.0), true),  // 200 x 200
            (Size::new(94.5, 200.0), false), // 199 x 200
        ] {
            let f = CardFormat::custom(cover, 10.0).unwrap();
            let g = page_geometry(&f, &PrintProfile::home(Sheet::A3)).unwrap();
            assert_eq!(g.sheet.unwrap().landscape, landscape, "{cover:?}");
        }
    }

    #[test]
    fn duplex_flip_follows_the_orientation() {
        // Every preset is wider than tall: landscape sheet, creases parallel to its short
        // edge, so the sheet turns like a book page on "flip on short edge".
        for (name, f) in CardFormat::presets() {
            let g = page_geometry(&f, &PrintProfile::home(Sheet::A3)).unwrap();
            assert_eq!(g.sheet.unwrap().duplex_flip(), DuplexFlip::ShortEdge, "{name}");
            assert_eq!(g.duplex_flip(), DuplexFlip::ShortEdge, "{name}");
            // A press page has no sheet but the same answer for an office-printer proof.
            let g = page_geometry(&f, &PrintProfile::professional_eu()).unwrap();
            assert!(g.sheet.is_none());
            assert_eq!(g.duplex_flip(), DuplexFlip::ShortEdge, "{name}");
        }
        // A tall card on a portrait sheet: the creases run along the long edge.
        let tall = CardFormat::custom(Size::new(50.0, 195.0), 10.0).unwrap();
        for profile in [PrintProfile::home(Sheet::A4), PrintProfile::upload_eu()] {
            let g = page_geometry(&tall, &profile).unwrap();
            assert_eq!(g.duplex_flip(), DuplexFlip::LongEdge);
            if let Some(s) = g.sheet {
                assert!(!s.landscape);
                assert_eq!(s.duplex_flip(), DuplexFlip::LongEdge);
            }
        }
    }

    #[test]
    fn page_sheet_and_spread_always_have_the_same_orientation() {
        // Otherwise a print dialog that auto-rotates the (off-size) page would put it on
        // the paper the other way round and the duplex instruction would be wrong.
        for sheet in [Sheet::A4, Sheet::Letter, Sheet::A3, Sheet::Sra3] {
            for w in (50..=300).step_by(25) {
                for h in (50..=300).step_by(25) {
                    let f =
                        CardFormat::custom(Size::new(f64::from(w), f64::from(h)), 10.0).unwrap();
                    let Ok(g) = page_geometry(&f, &PrintProfile::home(sheet)) else { continue };
                    let s = g.sheet.unwrap();
                    let spread = f.spread();
                    assert_eq!(s.landscape, spread.w >= spread.h, "{f:?} on {sheet:?}");
                    assert_eq!(s.landscape, g.media.w >= g.media.h, "{f:?} on {sheet:?}");
                    assert_eq!(s.landscape, s.sheet.w >= s.sheet.h, "{f:?} on {sheet:?}");
                    assert_eq!(s.duplex_flip(), g.duplex_flip(), "{f:?} on {sheet:?}");
                }
            }
        }
    }

    #[test]
    fn a_card_that_does_not_fit_reports_the_sheet_the_way_it_was_tried() {
        // Spread 110 x 295 on A4: tried in portrait, 311.7 mm needed of 297.
        let f = CardFormat::custom(Size::new(50.0, 295.0), 10.0).unwrap();
        match page_geometry(&f, &PrintProfile::home(Sheet::A4)) {
            Err(LayoutError::DoesNotFit { needed, sheet }) => {
                assert_eq!(sheet, Size::new(210.0, 297.0));
                assert!(close(needed.w, 126.7) && close(needed.h, 311.7));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn custom_sheet_orientation_does_not_matter() {
        let f = CardFormat::poche();
        let a = page_geometry(&f, &PrintProfile::home(Sheet::Custom(Size::new(300.0, 200.0))));
        let b = page_geometry(&f, &PrintProfile::home(Sheet::Custom(Size::new(200.0, 300.0))));
        assert_eq!(a, b);
        assert!(a.unwrap().sheet.unwrap().landscape);
    }

    #[test]
    fn invalid_inputs_are_errors_not_panics() {
        let bad_format = CardFormat { cover: Size::new(f64::NAN, 178.0), spine_width: 10.0 };
        assert!(matches!(
            new_page(&bad_format, &PrintProfile::professional_eu()),
            Err(LayoutError::CoverOutOfRange { .. })
        ));
        let mut bad_profile = PrintProfile::professional_eu();
        bad_profile.bleed = f64::INFINITY;
        assert!(matches!(
            new_page(&CardFormat::poche(), &bad_profile),
            Err(LayoutError::InvalidProfile { field: "bleed", .. })
        ));
        let p = PrintProfile::home(Sheet::Custom(Size::new(f64::NAN, 100.0)));
        assert!(matches!(
            new_page(&CardFormat::poche(), &p),
            Err(LayoutError::InvalidSheet { .. })
        ));
        // A sheet smaller than the card.
        let p = PrintProfile::home(Sheet::Custom(Size::new(10.0, 10.0)));
        assert!(matches!(new_page(&CardFormat::poche(), &p), Err(LayoutError::DoesNotFit { .. })));
    }

    #[test]
    fn press_profile_with_a_sheet_is_imposed_one_up() {
        // A press profile centred on SRA3: marks keep the registration colour and the
        // sheet decides the slug.
        let mut p = PrintProfile::professional_eu();
        p.sheet = Some(Sheet::Sra3);
        let g = page_geometry(&CardFormat::a5(), &p).unwrap();
        // 450 x 320 sheet, 306 x 210 spread: margins 72 and 55 mm.
        assert!(close(g.slug_mm, 52.0));
        assert!(close(g.media.h, 320.0));
        assert!(close(g.mark_reach_mm, 55.0));
    }
}
