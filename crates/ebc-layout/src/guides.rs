//! Safe areas, a non-printing "guides" overlay and the geometry facts recorded in the PDF.

use ebc_scene::{Color, Item, Path, Rect, Stroke};

use crate::format::{CardFormat, Panels};
use crate::page::page_geometry;
use crate::profile::PrintProfile;

/// Name of the layer returned by [`guides`].
pub const GUIDES_LAYER: &str = "guides";
/// Stroke weight of the guide lines, in points.
const GUIDE_WEIGHT_PT: f64 = 0.25;
/// Dash pattern (on, off — millimetres) of the fold and safe-area guides.
const GUIDE_DASH_MM: [f64; 2] = [2.0, 1.0];

/// A margin that is safe to subtract: negative and non-finite values count as zero.
fn sane(margin: f64) -> f64 {
    if margin.is_finite() { margin.max(0.0) } else { 0.0 }
}

/// The part of a cover panel where text and other important content may go: the panel
/// shrunk by the profile's `safe_margin` on all four sides — away from the trim edges,
/// where the cut may drift by a millimetre or two, and away from the crease.
///
/// For the spine use [`spine_safe_rect`], which applies the tighter `spine_safe_margin`
/// across the spine. The result never has a negative size: a margin larger than half the
/// panel collapses it to its centre.
#[must_use]
pub fn safe_rect(panel: Rect, profile: &PrintProfile) -> Rect {
    panel.inflate(-sane(profile.safe_margin))
}

/// The part of the spine where the title may go: `spine_safe_margin` away from each
/// crease (so that a crease that drifts does not fold through the lettering) and
/// `safe_margin` away from the top and bottom trim edges.
///
/// With the default 10 mm spine and the built-in profiles (2 mm from each crease) this is
/// a 6 mm wide band.
#[must_use]
pub fn spine_safe_rect(format: &CardFormat, profile: &PrintProfile) -> Rect {
    format.spine().inflate_xy(-sane(profile.spine_safe_margin), -sane(profile.safe_margin))
}

/// The safe areas of the three panels of either face: [`safe_rect`] for the two covers
/// and [`spine_safe_rect`] for the spine.
#[must_use]
pub fn safe_panels(format: &CardFormat, profile: &PrintProfile) -> Panels {
    let p = format.outside();
    Panels {
        left: safe_rect(p.left, profile),
        spine: spine_safe_rect(format, profile),
        right: safe_rect(p.right, profile),
    }
}

/// A proofing overlay as a layer named [`GUIDES_LAYER`] (`"guides"`): the bleed box
/// (cyan), the trim box (magenta), the two creases (magenta, dashed) and the three safe
/// areas (green, dashed), all as 0.25 pt strokes.
///
/// **Not for production.** The lines run across the artwork; add the layer only to a
/// proof or on-screen preview, on top of everything else, and let the PDF writer mark it
/// as non-printing (or drop it) for the print file. Colours are plain CMYK so the overlay
/// is legal in every PDF/X flavour.
///
/// The layer is empty when [`new_page`](crate::new_page) would fail.
#[must_use]
pub fn guides(format: &CardFormat, profile: &PrintProfile) -> Item {
    let Ok(g) = page_geometry(format, profile) else {
        return Item::layer(GUIDES_LAYER, Vec::new());
    };
    let cyan = Color::cmyk(1.0, 0.0, 0.0, 0.0);
    let magenta = Color::cmyk(0.0, 1.0, 0.0, 0.0);
    let green = Color::cmyk(1.0, 0.0, 1.0, 0.0);
    let solid = |c| Stroke::solid(c, GUIDE_WEIGHT_PT);
    let dashed = |c| Stroke::dashed(c, GUIDE_WEIGHT_PT, GUIDE_DASH_MM.to_vec());

    let mut folds = Path::new();
    for (top, bottom) in format.fold_lines() {
        folds.move_to(top.x, top.y).line_to(bottom.x, bottom.y);
    }
    let mut safe = Path::new();
    for r in safe_panels(format, profile).to_array() {
        safe.push_rect(r);
    }
    Item::layer(
        GUIDES_LAYER,
        vec![
            Item::stroke(Path::rect(g.bleed), solid(cyan)),
            Item::stroke(Path::rect(g.trim), solid(magenta)),
            Item::stroke(folds, dashed(magenta)),
            Item::stroke(safe, dashed(green)),
        ],
    )
}

/// Formats a length for an Info-dictionary value: at most three decimals, no trailing
/// zeros, `.` as the decimal separator whatever the locale.
fn mm(value: f64) -> String {
    let s = format!("{value:.3}");
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_owned() } else { s }
}

/// The card geometry as key/value pairs for the PDF Info dictionary
/// ([`ebc_scene::DocInfo::custom`]), so that a preflight tool can check that the creases
/// are where the file says they are:
///
/// | key | value (example) |
/// |---|---|
/// | `EBC.CoverWidthMm` | `110` |
/// | `EBC.CoverHeightMm` | `178` |
/// | `EBC.SpineWidthMm` | `10` |
/// | `EBC.FoldXMm` | `110,120` — distances of the creases from the left trim edge |
/// | `EBC.BleedMm` | `3` |
/// | `EBC.Profile` | `professional_eu` |
///
/// Lengths are in millimetres with at most three decimals. Keys are ASCII without spaces.
#[must_use]
pub fn info_entries(format: &CardFormat, profile: &PrintProfile) -> Vec<(String, String)> {
    let [f0, f1] = format.fold_x();
    [
        ("EBC.CoverWidthMm", mm(format.cover.w)),
        ("EBC.CoverHeightMm", mm(format.cover.h)),
        ("EBC.SpineWidthMm", mm(format.spine_width)),
        ("EBC.FoldXMm", format!("{},{}", mm(f0), mm(f1))),
        ("EBC.BleedMm", mm(profile.bleed)),
        ("EBC.Profile", profile.name.clone()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::profile::Sheet;
    use ebc_scene::Size;

    #[test]
    fn safe_areas_of_the_default_card() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        // 5 mm from the trim and from the crease: the 100 x 168 mm cover box.
        assert_eq!(safe_rect(f.front(), &p), Rect::new(125.0, 5.0, 100.0, 168.0));
        assert_eq!(safe_rect(f.back(), &p), Rect::new(5.0, 5.0, 100.0, 168.0));
        // 2 mm from each crease, 5 mm from top and bottom: a 6 mm text band.
        assert_eq!(spine_safe_rect(&f, &p), Rect::new(112.0, 5.0, 6.0, 168.0));
        let s = safe_panels(&f, &p);
        assert_eq!(s.left, safe_rect(f.back(), &p));
        assert_eq!(s.spine, spine_safe_rect(&f, &p));
        assert_eq!(s.right, safe_rect(f.front(), &p));
        // Home: the same margins.
        let h = PrintProfile::home(Sheet::A4);
        assert_eq!(spine_safe_rect(&f, &h), Rect::new(112.0, 5.0, 6.0, 168.0));
        // A printer's data sheet with other margins is honoured as given.
        let mut tight = p.clone();
        (tight.safe_margin, tight.spine_safe_margin) = (4.0, 1.5);
        assert_eq!(safe_rect(f.front(), &tight), Rect::new(124.0, 4.0, 102.0, 170.0));
        assert_eq!(spine_safe_rect(&f, &tight), Rect::new(111.5, 4.0, 7.0, 170.0));
    }

    #[test]
    fn safe_rect_never_goes_inside_out_and_ignores_hostile_margins() {
        let mut p = PrintProfile::professional_eu();
        let panel = Rect::new(10.0, 20.0, 30.0, 40.0);
        p.safe_margin = 1000.0;
        let r = safe_rect(panel, &p);
        assert_eq!((r.w, r.h), (0.0, 0.0));
        assert_eq!(r.center(), panel.center());
        for m in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -4.0] {
            p.safe_margin = m;
            p.spine_safe_margin = m;
            // Never grows the panel, never produces NaN.
            assert_eq!(safe_rect(panel, &p), panel, "margin {m}");
            assert_eq!(spine_safe_rect(&CardFormat::poche(), &p), CardFormat::poche().spine());
        }
        // A 3 mm spine with a 2 mm spine margin collapses to its centre line.
        // Built directly: a 3 mm spine no longer validates, but the helper must cope.
        let thin = CardFormat { spine_width: 3.0, ..CardFormat::poche() };
        let r = spine_safe_rect(&thin, &PrintProfile::home(Sheet::A4));
        assert_eq!(r.w, 0.0);
        assert_eq!(r.center().x, thin.spine().center().x);
    }

    #[test]
    fn guides_layer_outlines_the_boxes() {
        let (f, p) = (CardFormat::poche(), PrintProfile::professional_eu());
        let Item::Group { layer, items, .. } = guides(&f, &p) else { panic!() };
        assert_eq!(layer.as_deref(), Some("guides"));
        assert_eq!(items.len(), 4);
        let bounds: Vec<Rect> = items
            .iter()
            .map(|i| match i {
                Item::Path { path, fill: None, stroke: Some(s) } => {
                    assert!(matches!(s.color, Color::Cmyk { .. }), "guides are CMYK");
                    path.control_bounds().unwrap()
                }
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(bounds[0], Rect::new(-3.0, -3.0, 236.0, 184.0)); // bleed
        assert_eq!(bounds[1], Rect::new(0.0, 0.0, 230.0, 178.0)); // trim
        assert_eq!(bounds[2], Rect::new(110.0, 0.0, 10.0, 178.0)); // creases
        assert_eq!(bounds[3], Rect::new(5.0, 5.0, 220.0, 168.0)); // safe areas
        // Invalid input gives an empty layer.
        let bad = CardFormat { cover: Size::new(0.0, 0.0), spine_width: 0.0 };
        assert_eq!(guides(&bad, &p), Item::layer("guides", vec![]));
    }

    #[test]
    fn info_entries_describe_the_geometry() {
        let e = info_entries(&CardFormat::poche(), &PrintProfile::professional_eu());
        let get = |k: &str| e.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
        assert_eq!(get("EBC.SpineWidthMm"), Some("10"));
        assert_eq!(get("EBC.FoldXMm"), Some("110,120"));
        assert_eq!(get("EBC.CoverWidthMm"), Some("110"));
        assert_eq!(get("EBC.CoverHeightMm"), Some("178"));
        assert_eq!(get("EBC.BleedMm"), Some("3"));
        assert_eq!(get("EBC.Profile"), Some("professional_eu"));
        for (k, _) in &e {
            assert!(k.is_ascii() && !k.contains(' ') && k.starts_with("EBC."), "{k}");
        }
        // Fractions keep up to three decimals and lose trailing zeros.
        let f = CardFormat::custom(Size::new(107.95, 177.8), 9.5254).unwrap();
        let e = info_entries(&f, &PrintProfile::home(Sheet::Letter));
        let get = |k: &str| e.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
        assert_eq!(get("EBC.CoverWidthMm"), Some("107.95"));
        assert_eq!(get("EBC.SpineWidthMm"), Some("9.525"));
        assert_eq!(get("EBC.FoldXMm"), Some("107.95,117.475"));
        assert_eq!(get("EBC.Profile"), Some("home_letter"));
    }

    #[test]
    fn number_formatting() {
        assert_eq!(mm(10.0), "10");
        assert_eq!(mm(100.0), "100");
        assert_eq!(mm(0.0), "0");
        assert_eq!(mm(1.5), "1.5");
        assert_eq!(mm(3.175), "3.175");
        assert_eq!(mm(0.0004), "0");
        // Never panics on junk.
        assert_eq!(mm(f64::NAN), "NaN");
        assert_eq!(mm(f64::INFINITY), "inf");
    }
}
