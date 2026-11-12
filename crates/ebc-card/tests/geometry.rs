//! Geometry of the composed pages, measured on the scene document: page boxes, what lies
//! where, safe areas, marks and fold positions.

#![allow(clippy::unwrap_used)]

mod common;

use common::{fonts, request, with_cover};
use ebc_card::inspect::{Inspector, Leaf, LeafKind};
use ebc_card::{NOTES_LAYER, RightsBasis, compose};
use ebc_layout::{
    CardFormat, GUIDES_LAYER, MARKS_LAYER, PrintProfile, Sheet, page_geometry, safe_rect,
};
use ebc_model::Locale;
use ebc_scene::{Document, Item, Rect, Stroke};

const EPS: f64 = 1e-6;

fn is_furniture(leaf: &Leaf) -> bool {
    matches!(leaf.layer.as_deref(), Some(MARKS_LAYER | NOTES_LAYER | GUIDES_LAYER))
}

fn presets_that_fit() -> Vec<(String, CardFormat, PrintProfile)> {
    let mut out = Vec::new();
    for (name, format) in CardFormat::presets() {
        for profile in [
            PrintProfile::upload_eu(),
            PrintProfile::professional_eu(),
            PrintProfile::home(Sheet::A4),
            PrintProfile::home(Sheet::Letter),
            PrintProfile::home(Sheet::A3),
        ] {
            if page_geometry(&format, &profile).is_ok() {
                out.push((format!("{name}/{}", profile.name), format, profile));
            }
        }
    }
    assert!(out.len() >= 14, "{}", out.len());
    out
}

#[test]
fn pages_have_the_boxes_of_the_layout() {
    let fonts = fonts();
    for (label, format, profile) in presets_that_fit() {
        let (doc, _) = compose(&request(format, profile.clone(), Locale::Fr), &fonts).unwrap();
        let geometry = page_geometry(&format, &profile).unwrap();
        assert_eq!(doc.pages.len(), 2, "{label}");
        for (page, name) in doc.pages.iter().zip(["outside", "inside"]) {
            assert_eq!(page.label.as_deref(), Some(name), "{label}");
            assert_eq!(page.trim, format.spread(), "{label}");
            assert_eq!(page.bleed, profile.bleed, "{label}");
            assert_eq!(page.media_box(), geometry.media, "{label}");
        }
        if let Some(sheet) = profile.sheet {
            // Home: the page is the sheet itself, in landscape, the spread centred on it.
            let media = doc.pages[0].media_box();
            let size = sheet.landscape();
            assert!((media.w - size.w).abs() < 1e-9 && (media.h - size.h).abs() < 1e-9, "{label}");
            assert_eq!(media.center(), doc.pages[0].trim_box().center(), "{label}");
        }
    }
}

#[test]
fn inside_content_stays_in_the_safe_area_of_its_panel() {
    let fonts = fonts();
    for (label, format, profile) in presets_that_fit() {
        for locale in [Locale::Fr, Locale::De, Locale::En] {
            let mut req = request(format, profile.clone(), locale);
            req.store_name = Some("Septante (démo)".to_owned());
            req.legal_lines = vec!["EPUB sans DRM. Contact : aide@ebk.example".to_owned()];
            let (doc, _) = compose(&req, &fonts).unwrap();
            let left = safe_rect(format.inside_front(), &profile);
            let right = safe_rect(format.inside_back(), &profile);
            let spine = format.inside_spine();
            let leaves = Inspector::new(&doc).leaves(1);
            let content: Vec<&Leaf> = leaves.iter().filter(|l| !is_furniture(l)).collect();
            assert!(content.len() > 20, "{label}: {}", content.len());
            for leaf in &content {
                let b = leaf.bounds;
                assert!(
                    left.contains_rect(&b, 0.01) || right.contains_rect(&b, 0.01),
                    "{label}/{locale}: {:?} at {b:?} is outside the safe areas {left:?} / {right:?}",
                    leaf.kind
                );
                // Nothing on the inside of the spine, nothing on the creases.
                assert!(b.intersect(&spine.inflate(1.0)).is_none(), "{label}: {b:?}");
            }
            // Both panels are used, and the text is real text.
            assert!(
                content
                    .iter()
                    .any(|l| l.kind == LeafKind::Text && left.contains_rect(&l.bounds, 0.01))
            );
            assert!(
                content
                    .iter()
                    .any(|l| l.kind == LeafKind::Text && right.contains_rect(&l.bounds, 0.01))
            );
        }
    }
}

/// The QR code of the inside page: the white box and the bounds of the dark modules.
fn qr_rects(doc: &Document) -> (Rect, Rect) {
    let items = &doc.pages[1].items;
    let position = items
        .iter()
        .position(
            |item| matches!(item, Item::Path { path, stroke: None, .. } if path.segs.len() > 200),
        )
        .expect("the QR code path");
    let modules = ebc_card::inspect::item_bounds(doc, &items[position]).unwrap();
    let quiet = ebc_card::inspect::item_bounds(doc, &items[position - 1]).unwrap();
    (quiet, modules)
}

#[test]
fn the_qr_code_is_large_black_and_has_its_quiet_zone() {
    let fonts = fonts();
    for (label, format, profile) in presets_that_fit() {
        let req = request(format, profile.clone(), Locale::Fr);
        let (doc, _) = compose(&req, &fonts).unwrap();
        let (quiet, modules) = qr_rects(&doc);
        let safe = safe_rect(format.inside_back(), &profile);
        assert!(safe.contains_rect(&quiet, EPS), "{label}: {quiet:?} in {safe:?}");
        // 25 modules of 1 mm in a 33 mm white box: four modules of quiet zone all round.
        assert!(
            (quiet.w - 33.0).abs() < 1e-9 && (quiet.h - 33.0).abs() < 1e-9,
            "{label}: {quiet:?}"
        );
        assert!(
            (modules.w - 25.0).abs() < 1e-9 && (modules.h - 25.0).abs() < 1e-9,
            "{label}: {modules:?}"
        );
        assert!(modules.w >= ebc_card::QR_MIN_SYMBOL_MM);
        for margin in [
            modules.x - quiet.x,
            quiet.right() - modules.right(),
            modules.y - quiet.y,
            quiet.bottom() - modules.bottom(),
        ] {
            assert!((margin - 4.0).abs() < 1e-9, "{label}: quiet zone of {margin} mm");
        }
        // K only (grey in a home PDF), on paper white.
        let items = &doc.pages[1].items;
        let dark = items.iter().find_map(|item| match item {
            Item::Path { path, fill: Some((color, _)), .. } if path.segs.len() > 200 => {
                Some(*color)
            }
            _ => None,
        });
        let expected = if profile.sheet.is_some() {
            ebc_scene::Color::Gray(0.0)
        } else {
            ebc_scene::Color::BLACK
        };
        assert_eq!(dark, Some(expected), "{label}");
    }
}

#[test]
fn outside_artwork_stays_in_the_bleed_box_and_text_in_the_safe_areas() {
    let fonts = fonts();
    for (label, format, profile) in presets_that_fit() {
        for cover in [false, true] {
            let mut req = request(format, profile.clone(), Locale::Fr);
            if cover {
                req = with_cover(req, RightsBasis::RightsHolderDeclared);
            }
            let (doc, _) = compose(&req, &fonts).unwrap();
            let page = &doc.pages[0];
            let back = safe_rect(format.back(), &profile);
            let front = safe_rect(format.front(), &profile);
            let spine_safe = ebc_layout::spine_safe_rect(&format, &profile);
            // The ground is the first thing painted and fills the bleed box exactly.
            assert_eq!(
                ebc_card::inspect::item_bounds(&doc, &page.items[0]),
                Some(page.bleed_box()),
                "{label}"
            );
            let leaves = Inspector::new(&doc).leaves(0);
            for leaf in leaves.iter().filter(|l| !is_furniture(l)) {
                let b = leaf.bounds;
                assert!(
                    page.bleed_box().contains_rect(&b, EPS),
                    "{label}: {b:?} leaves the bleed box"
                );
                if leaf.kind == LeafKind::Text {
                    assert!(
                        back.contains_rect(&b, 0.01) || front.contains_rect(&b, 0.01),
                        "{label}: text at {b:?} outside the safe areas"
                    );
                }
            }
            // The spine's lettering (outlines) stays between the two safe margins.
            let spine_ink: Vec<&Leaf> = leaves
                .iter()
                .filter(|l| {
                    !is_furniture(l)
                        && l.kind == LeafKind::Path
                        && l.bounds.w < 8.0
                        && l.bounds.x >= format.spine().x
                        && l.bounds.right() <= format.spine().right()
                        && l.bounds.h < 20.0
                })
                .collect();
            assert!(spine_ink.len() > 10, "{label}: the spine carries the title");
            for leaf in spine_ink {
                assert!(spine_safe.contains_rect(&leaf.bounds, 0.01), "{label}: {:?}", leaf.bounds);
            }
        }
    }
}

#[test]
fn marks_and_notes_lie_outside_the_bleed_and_inside_the_sheet() {
    let fonts = fonts();
    for (label, format, profile) in presets_that_fit() {
        let (doc, _) = compose(&request(format, profile.clone(), Locale::Fr), &fonts).unwrap();
        let geometry = page_geometry(&format, &profile).unwrap();
        for index in 0..2 {
            let page = &doc.pages[index];
            let leaves = Inspector::new(&doc).leaves(index);
            let furniture: Vec<&Leaf> = leaves.iter().filter(|l| is_furniture(l)).collect();
            if !profile.marks.any() {
                assert!(furniture.is_empty(), "{label}: the upload profile has no marks");
                continue;
            }
            assert!(furniture.len() >= 12, "{label}: {}", furniture.len());
            for leaf in &furniture {
                assert!(
                    leaf.bounds.intersect(&page.bleed_box()).is_none(),
                    "{label}: {:?} in layer {:?} enters the bleed box",
                    leaf.bounds,
                    leaf.layer
                );
                assert!(
                    page.media_box().contains_rect(&leaf.bounds, EPS),
                    "{label}: {:?}",
                    leaf.bounds
                );
                assert!(
                    geometry.mark_area.contains_rect(&leaf.bounds, 0.05),
                    "{label}: {:?}",
                    leaf.bounds
                );
                if let Some(sheet) = geometry.sheet {
                    assert!(
                        sheet.printable.contains_rect(&leaf.bounds, 0.05),
                        "{label}: unprintable"
                    );
                }
            }
            if profile.sheet.is_some() {
                // The duplex and cutting instructions and the ruler's label.
                let notes: Vec<&&Leaf> =
                    furniture.iter().filter(|l| l.layer.as_deref() == Some(NOTES_LAYER)).collect();
                assert!(notes.len() >= 3, "{label}: {}", notes.len());
            }
        }
    }
}

#[test]
fn fold_marks_sit_on_the_creases_recorded_in_the_document() {
    let fonts = fonts();
    for (label, format, profile) in presets_that_fit() {
        if !profile.marks.fold {
            continue;
        }
        let (doc, _) = compose(&request(format, profile.clone(), Locale::Fr), &fonts).unwrap();
        let recorded: Vec<f64> = doc
            .info
            .custom
            .iter()
            .find(|(k, _)| k == "EBC.FoldXMm")
            .map(|(_, v)| v.split(',').map(|x| x.parse().unwrap()).collect())
            .unwrap();
        assert_eq!(recorded, format.fold_x().to_vec(), "{label}");
        for page in &doc.pages {
            // Dashed vertical strokes of the marks layer: two per crease.
            let mut dashed_x = Vec::new();
            for item in &page.items {
                let Item::Group { layer: Some(layer), items, .. } = item else { continue };
                if layer != MARKS_LAYER {
                    continue;
                }
                for mark in items {
                    if let Item::Path { path, stroke: Some(Stroke { dash_mm, .. }), .. } = mark
                        && !dash_mm.is_empty()
                    {
                        let b = path.control_bounds().unwrap();
                        assert!(b.w.abs() < 1e-12, "{label}: fold marks are vertical");
                        dashed_x.push(b.x);
                    }
                }
            }
            dashed_x.sort_by(f64::total_cmp);
            let [f0, f1] = format.fold_x();
            assert_eq!(dashed_x, [f0, f0, f1, f1], "{label}");
        }
    }
}

#[test]
fn inside_front_is_behind_the_front_cover() {
    // Physical logic: the sheet is turned about the axis parallel to the creases, so the
    // left panel of page 2 is the back of the right panel of page 1 (the front cover).
    let fonts = fonts();
    let format = CardFormat::poche();
    let profile = PrintProfile::home(Sheet::A4);
    let (doc, _) = compose(&request(format, profile.clone(), Locale::Fr), &fonts).unwrap();
    assert_eq!(format.mirror_x(format.front()), format.inside_front());
    let inside = common::page_text(&doc, 1);
    let outside = common::page_text(&doc, 0);
    let left = safe_rect(format.inside_front(), &profile);
    // The gift ("Ce livre est un cadeau", "Pour Bob") is on the left of page 2 …
    let leaves = Inspector::new(&doc).leaves(1);
    let first_text = leaves.iter().find(|l| l.kind == LeafKind::Text).unwrap();
    assert!(left.contains_rect(&first_text.bounds, 0.01));
    assert!(inside.starts_with("Ce livre est un cadeau"), "{inside}");
    assert!(inside.contains("Pour Bob") && inside.contains("De la part d’Alice"));
    // … and the sheet says how to turn it: on the short edge, for this landscape sheet.
    assert!(outside.contains("retourner sur le bord court"), "{outside}");
    assert!(inside.contains("retourner sur le bord court"));
    assert!(outside.contains("Couper sur les traits pleins"));
    assert!(outside.contains("100 mm"));
    assert_eq!(
        page_geometry(&format, &profile).unwrap().duplex_flip(),
        ebc_layout::DuplexFlip::ShortEdge
    );
}
