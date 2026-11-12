//! Spar review (print domain): failing tests that pin print defects of the composed card.
//! Intended location: `crates/ebc-card/tests/spar_print_card.rs` (uses `tests/common`).
#![allow(clippy::unwrap_used)]

mod common;

use common::{errors, expectations, fonts, poche, with_cover};
use ebc_card::{RightsBasis, SpineChoice, compose, render_pdf};
use ebc_layout::{CardFormat, PrintProfile, Sheet, page_geometry};
use ebc_preflight::check;
use ebc_scene::{Color, Item, Rect};

/// 03-print-requirements.md §2.2 rule 1, §10.1 B4: an upload file is exactly trim + bleed
/// with no printer's marks. Preflight run for the upload profile must refuse a file that
/// carries crop/fold marks in the registration colour and a 10 mm slug.
#[test]
fn upload_preflight_refuses_a_file_with_marks_and_slug() {
    let marked = poche(PrintProfile::professional_eu());
    let (pdf, _) = render_pdf(&marked, &fonts()).unwrap();
    // Expectations an upload order would use (same trim, spine, bleed).
    let upload = poche(PrintProfile::upload_eu());
    let report = check(&pdf, &expectations(&upload, false)).unwrap();
    assert!(
        !errors(&report).is_empty(),
        "a 256 x 204 mm page with registration-colour marks passed the upload preflight"
    );
}

/// §3.2 / §10.1 C3: no optional content in professional output. A proof with the guides
/// overlay is still labelled PDF/X-4, and its guides layer is ON by default (no /OFF in
/// /D, no print usage), so a RIP prints cyan/magenta/green lines across the live art.
#[test]
fn a_pdfx_file_never_carries_a_visible_guides_layer() {
    let mut req = with_cover(poche(PrintProfile::upload_eu()), RightsBasis::PublicDomain);
    req.guides = true;
    let (pdf, _) = render_pdf(&req, &fonts()).unwrap();
    let has = |needle: &[u8]| pdf.windows(needle.len()).any(|w| w == needle);
    if has(b"/GTS_PDFX") && has(b"/OCProperties") {
        assert!(has(b"/OFF"), "PDF/X file with an optional-content guides layer that prints");
    }
    let report = check(&pdf, &expectations(&req, true)).unwrap();
    assert!(!errors(&report).is_empty(), "preflight passes a PDF/X-4 with a guides layer");
}

/// §3.2 content subset, §3.4, implication 23: a professional cover card is CMYK/grey only
/// unless the profile allows RGB; otherwise the professional output is refused.
#[test]
fn professional_cover_card_passes_its_own_cmyk_preflight() {
    let req = with_cover(poche(PrintProfile::upload_eu()), RightsBasis::RightsHolderDeclared);
    let (pdf, _) = render_pdf(&req, &fonts()).unwrap();
    let report = check(&pdf, &expectations(&req, false)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
}

/// §4.4 and implication 27: outlined text, and reversed text, 8 pt or more. The spine
/// author is set at 7 pt white on the ground with only a note.
#[test]
fn spine_never_sets_reversed_or_outlined_text_below_8_pt() {
    let mut req = with_cover(poche(PrintProfile::upload_eu()), RightsBasis::RightsHolderDeclared);
    req.book.title =
        "Les Vagues du soir et autres nouvelles de la mer du Nord, suivies de poèmes".to_owned();
    req.book.contributors = vec![ebc_model::Contributor::author("Marie-Hélène de La Tour-Dupont")];
    let (_doc, warnings) = compose(&req, &fonts()).unwrap();
    let small: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, ebc_card::CardWarning::Spine(ebc_spine::SpineNote::SmallReversedText { .. })))
        .collect();
    assert!(small.is_empty(), "{small:?}");
}

fn walk<'a>(items: &'a [Item], out: &mut Vec<&'a Item>) {
    for item in items {
        out.push(item);
        if let Item::Group { items, .. } = item {
            walk(items, out);
        }
    }
}

/// §6.4 rule 3 and §10.1 B8: a spine with its own colour runs 2 mm past each crease and
/// no colour boundary falls within 2 mm of a crease. With the edge-continuation spine the
/// spine image stops exactly on both creases and a 2 mm band of a third colour is
/// painted on the front panel (x 120..122 on the poche).
#[test]
fn edge_continuation_spine_paints_no_stripe_on_the_front_panel() {
    let mut req = with_cover(poche(PrintProfile::home(Sheet::A4)), RightsBasis::RightsHolderDeclared);
    req.spine = SpineChoice::EdgeContinuation;
    req.adaptation_authorised = true;
    let (doc, _) = compose(&req, &fonts()).unwrap();
    let geometry = page_geometry(&CardFormat::poche(), &req.profile).unwrap();
    let mut all = Vec::new();
    walk(&doc.pages[0].items, &mut all);
    let ground = all
        .iter()
        .find_map(|i| match i {
            Item::Path { fill: Some((c, _)), .. } => Some(*c),
            _ => None,
        })
        .unwrap();
    let crease_b = 120.0;
    let stripe: Vec<(Rect, Color)> = all
        .iter()
        .filter_map(|i| match i {
            Item::Path { path, fill: Some((c, _)), .. } if *c != ground => {
                let b = path.control_bounds()?;
                (b.x <= geometry.bleed.x + 1e-6 && b.right() > crease_b + 0.5).then_some((b, *c))
            }
            _ => None,
        })
        .collect();
    assert!(stripe.is_empty(), "flat fills other than the ground reach onto the front panel: {stripe:?}");
}
