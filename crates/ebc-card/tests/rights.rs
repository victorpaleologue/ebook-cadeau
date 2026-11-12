//! The rights gate and the cover-integrity rules of
//! `docs/research/07-legal-covers-and-reseller-rights.md` (implications 1–14).

#![allow(clippy::unwrap_used)]

mod common;

use common::{art, errors, expectations, fonts, images, page_text, poche, text_runs, with_cover};
use ebc_card::inspect::{Inspector, LeafKind};
use ebc_card::{
    CardError, CardWarning, CoverInput, MIN_COVER_INSET_MM, PrintRoute, REDUCED_COVER_SHARE,
    RightsBasis, SpineChoice, compose, render_pdf,
};
use ebc_layout::{CardFormat, GUIDES_LAYER, PrintProfile, Sheet, safe_rect};
use ebc_model::Locale;
use ebc_preflight::check;
use ebc_scene::{Document, Item, PdfStandard, Rect};
use ebc_spine::SpineNote;

fn home() -> PrintProfile {
    PrintProfile::home(Sheet::A4)
}

fn pro() -> PrintProfile {
    PrintProfile::upload_eu()
}

fn has_warning(warnings: &[CardWarning], wanted: &CardWarning) -> bool {
    warnings.contains(wanted)
}

/// Asserts that the single cover image of the outside page is scaled uniformly (to
/// 0.1 %), is not rotated or transformed, and returns its rectangle and clip.
fn cover_placement(doc: &Document) -> (Rect, Option<Rect>) {
    let found = images(doc, 0);
    assert_eq!(found.len(), 1, "exactly one image on the outside page");
    assert!(images(doc, 1).is_empty(), "no image on the inside page");
    let (id, rect, clip) = found[0];
    let (w, h) = doc.image(id).unwrap().data.dimensions_px();
    let (sx, sy) = (rect.w / f64::from(w), rect.h / f64::from(h));
    assert!((sx / sy - 1.0).abs() < 0.001, "non-uniform scale: {sx} vs {sy}");
    (rect, clip)
}

#[test]
fn a_cover_without_a_rights_basis_never_reaches_the_file() {
    let fonts = fonts();
    for profile in [home(), pro()] {
        // Real pixels with basis `none`: ignored. Garbage with basis `none`: not even
        // decoded.
        for cover in [
            CoverInput::Pixels { pixmap: common::cover_pixmap(), rights: RightsBasis::None },
            CoverInput::Encoded { bytes: b"not an image".to_vec(), rights: RightsBasis::None },
        ] {
            let mut req = poche(profile.clone());
            req.cover = cover;
            let (doc, warnings) = compose(&req, &fonts).unwrap();
            assert!(doc.images.is_empty());
            assert!(images(&doc, 0).is_empty());
            assert_eq!(warnings, [CardWarning::CoverWithoutRights]);
            assert!(doc.info.custom.contains(&("EBC.RightsBasis".to_owned(), "none".to_owned())));
            // The typographic card: title, subtitle, author and publisher as text.
            let text = page_text(&doc, 0);
            for expected in
                ["Les Vagues du soir", "Roman", "Jeanne Roy", "Éditions du Lac (fictif)"]
            {
                assert!(text.contains(expected), "{expected} missing from {text}");
            }
            assert!(!text.contains('©'), "no cover, no credit line: {text}");
        }
    }
    // No image at all: the same card, without a warning.
    let (doc, warnings) = compose(&poche(pro()), &fonts).unwrap();
    assert!(doc.images.is_empty() && warnings.is_empty());
}

#[test]
fn the_typographic_card_carries_no_blurb_logo_or_barcode() {
    let fonts = fonts();
    let mut req = poche(pro());
    req.book.description = Some("UN BLURB DE L'ÉDITEUR qui ne doit pas être imprimé.".to_owned());
    req.book.isbn13 = "9782070360024".parse().ok();
    let (doc, _) = compose(&req, &fonts).unwrap();
    let text = format!("{}\n{}", page_text(&doc, 0), page_text(&doc, 1));
    assert!(!text.contains("BLURB"), "{text}");
    assert!(
        !text.contains("9782070360024") && !text.contains("978-2"),
        "no ISBN digits, no barcode"
    );
    assert!(doc.images.is_empty());
    // Nothing on the back panel looks like a barcode: no run of many thin filled bars.
    let back = CardFormat::poche().back();
    let bars = Inspector::new(&doc)
        .leaves(0)
        .into_iter()
        .filter(|l| {
            l.kind == LeafKind::Path
                && back.contains_rect(&l.bounds, 0.0)
                && l.bounds.w < 1.0
                && l.bounds.h > 8.0
        })
        .count();
    assert_eq!(bars, 0);
}

#[test]
fn the_print_route_gate() {
    let fonts = fonts();
    assert_eq!(ebc_card::rights::route_of(&home()), PrintRoute::HomePrint);
    assert_eq!(ebc_card::rights::route_of(&pro()), PrintRoute::ProfessionalOrder);
    for basis in RightsBasis::ALL {
        // Home print: every basis except `none` puts the cover in the file.
        let (doc, warnings) = compose(&with_cover(poche(home()), basis), &fonts).unwrap();
        assert_eq!(doc.images.len(), usize::from(basis != RightsBasis::None), "{basis}");
        assert_eq!(
            has_warning(&warnings, &CardWarning::OwnPrinterOnly),
            basis == RightsBasis::UserPrivateCopy,
            "{basis}"
        );
        assert!(
            doc.info.custom.contains(&("EBC.RightsBasis".to_owned(), basis.as_str().to_owned()))
        );

        // Professional print: only three bases; the others are typed errors.
        for profile in [PrintProfile::upload_eu(), PrintProfile::professional_eu()] {
            let result = compose(&with_cover(poche(profile), basis), &fonts);
            match basis {
                RightsBasis::None => {
                    let (doc, warnings) = result.unwrap();
                    assert!(doc.images.is_empty());
                    assert_eq!(warnings, [CardWarning::CoverWithoutRights]);
                }
                RightsBasis::PrintLicence
                | RightsBasis::RightsHolderDeclared
                | RightsBasis::PublicDomain => {
                    let (doc, warnings) = result.unwrap();
                    assert_eq!(doc.images.len(), 1, "{basis}");
                    // Converted to the output intent's CMYK (§3.4), not left as sRGB.
                    assert!(!has_warning(&warnings, &CardWarning::CoverNotConverted));
                    assert!(matches!(
                        doc.images[0].data,
                        ebc_scene::ImageData::Raw { format: ebc_scene::PixelFormat::Cmyk8, .. }
                    ));
                }
                RightsBasis::PromoAllMedia | RightsBasis::UserPrivateCopy => {
                    let Err(CardError::RightsRouteRefused { basis: refused, route }) = result
                    else {
                        panic!("{basis} must be refused on a professional profile")
                    };
                    assert_eq!((refused, route), (basis, PrintRoute::ProfessionalOrder));
                }
            }
        }
    }
    // The refusal is decided before the image is even looked at.
    let mut req = poche(pro());
    req.cover =
        CoverInput::Encoded { bytes: b"garbage".to_vec(), rights: RightsBasis::UserPrivateCopy };
    assert!(matches!(compose(&req, &fonts), Err(CardError::RightsRouteRefused { .. })));
}

#[test]
fn by_default_the_cover_is_whole_inset_and_letterboxed_on_a_flat_ground() {
    let fonts = fonts();
    let format = CardFormat::poche();
    for profile in [home(), pro()] {
        let front = format.front();
        let safe = safe_rect(front, &profile);
        // Portrait (the panel's own proportions, more or less), landscape, square, very
        // tall: always the whole image, inside the safe area, never clipped.
        for (w, h) in [(620, 1004), (1000, 620), (800, 800), (300, 1500)] {
            let mut req = poche(profile.clone());
            req.cover = CoverInput::Pixels {
                pixmap: art::cover(w, h, false, false),
                rights: RightsBasis::RightsHolderDeclared,
            };
            let (doc, _) = compose(&req, &fonts).unwrap();
            let (rect, clip) = cover_placement(&doc);
            assert_eq!(clip, None, "{w}x{h}: no clip means no crop");
            assert!(safe.contains_rect(&rect, 1e-9), "{w}x{h}: {rect:?} in {safe:?}");
            // At least 3 mm inside the trim on the outer edges, and off the crease.
            assert!(rect.y - front.y >= MIN_COVER_INSET_MM - 1e-9);
            assert!(front.bottom() - rect.bottom() >= MIN_COVER_INSET_MM - 1e-9);
            assert!(front.right() - rect.right() >= MIN_COVER_INSET_MM - 1e-9);
            assert!(rect.x - front.x >= MIN_COVER_INSET_MM - 1e-9);
            // Letterboxed: it touches the box in one direction and is centred in the other.
            let touches_w = (rect.w - safe.w).abs() < 1e-9;
            let touches_h = (rect.h - safe.h).abs() < 1e-9;
            assert!(touches_w || touches_h, "{w}x{h}: {rect:?}");
            assert!((rect.center().x - safe.center().x).abs() < 1e-9);
            assert!((rect.center().y - safe.center().y).abs() < 1e-9);
            // The pixels are embedded as they are: same pixel dimensions as the input.
            assert_eq!(doc.images[0].data.dimensions_px(), (w, h));
            // The bleed is filled with one flat colour: the first item is a plain filled
            // rectangle over the whole bleed box, and the image is a top-level item (no
            // group, hence no transform and no clip).
            let page = &doc.pages[0];
            assert!(matches!(&page.items[0], Item::Path { fill: Some(_), stroke: None, .. }));
            assert_eq!(
                ebc_card::inspect::item_bounds(&doc, &page.items[0]),
                Some(page.bleed_box())
            );
            assert!(page.items.iter().any(|i| matches!(i, Item::Image { .. })));
        }
    }
}

#[test]
fn a_profile_with_a_small_safe_margin_still_keeps_the_cover_3_mm_inside() {
    let fonts = fonts();
    let mut profile = pro();
    profile.safe_margin = 1.0;
    let (doc, _) = compose(&with_cover(poche(profile), RightsBasis::PrintLicence), &fonts).unwrap();
    let (rect, _) = cover_placement(&doc);
    let front = CardFormat::poche().front();
    assert!(front.inflate(-MIN_COVER_INSET_MM).contains_rect(&rect, 1e-9), "{rect:?}");
}

#[test]
fn full_bleed_only_when_the_trim_is_authorised() {
    let fonts = fonts();
    let format = CardFormat::poche();
    for profile in [home(), pro()] {
        let mut req = with_cover(poche(profile.clone()), RightsBasis::PrintLicence);
        req.trim_authorised = true;
        let (doc, _) = compose(&req, &fonts).unwrap();
        let (rect, clip) = cover_placement(&doc);
        let clip = clip.expect("a full-bleed cover is clipped to the panel and its bleed");
        // The clip is the front panel grown into the bleed on its three outer edges …
        let b = profile.bleed;
        let front = format.front();
        assert_eq!(clip, Rect::new(front.x, -b, front.w + b, front.h + 2.0 * b));
        // … the image covers it, and its spine-side edge lies exactly on the crease.
        assert!(rect.contains_rect(&clip, 1e-9), "{rect:?} covers {clip:?}");
        assert_eq!(rect.x, format.fold_x()[1]);
        // Without the authorisation the very same request is letterboxed.
        req.trim_authorised = false;
        let (doc, _) = compose(&req, &fonts).unwrap();
        assert_eq!(cover_placement(&doc).1, None);
    }
    // A cover with very different proportions: a warning says how much is lost.
    let mut req = poche(pro());
    req.cover = CoverInput::Pixels {
        pixmap: art::cover(1000, 620, false, false),
        rights: RightsBasis::PrintLicence,
    };
    req.trim_authorised = true;
    let (_, warnings) = compose(&req, &fonts).unwrap();
    assert!(
        warnings.iter().any(|w| matches!(w, CardWarning::HeavyCrop { dimension: "width", percent } if *percent > 50.0)),
        "{warnings:?}"
    );
}

#[test]
fn promo_all_media_gives_the_reduced_size_layout() {
    let fonts = fonts();
    let format = CardFormat::poche();
    let front = format.front();
    for trim_authorised in [false, true] {
        let mut req = with_cover(poche(home()), RightsBasis::PromoAllMedia);
        req.trim_authorised = trim_authorised; // never widens a promotional licence
        let (doc, _) = compose(&req, &fonts).unwrap();
        let (rect, clip) = cover_placement(&doc);
        assert_eq!(clip, None);
        assert!(rect.w <= front.w * REDUCED_COVER_SHARE + 1e-9, "{rect:?}");
        assert!(rect.h <= front.h * REDUCED_COVER_SHARE + 1e-9, "{rect:?}");
        assert!(rect.w * rect.h <= 0.36 * front.w * front.h + 1e-9);
        assert!(safe_rect(front, &req.profile).contains_rect(&rect, 1e-9));
        // The title, the author and the publisher are retyped on the same panel, below
        // the cover, in the safe area.
        let below: Vec<_> = Inspector::new(&doc)
            .leaves(0)
            .into_iter()
            .filter(|l| l.kind == LeafKind::Text && l.layer.is_none() && l.bounds.x >= front.x)
            .collect();
        assert!(below.len() >= 3, "{}", below.len());
        for leaf in &below {
            assert!(leaf.bounds.y >= rect.bottom(), "{:?} is not below the cover", leaf.bounds);
            assert!(safe_rect(front, &req.profile).contains_rect(&leaf.bounds, 0.01));
        }
        let text = page_text(&doc, 0);
        assert!(text.matches("Les Vagues du soir").count() >= 1 && text.contains("Jeanne Roy"));
    }
}

#[test]
fn nothing_is_painted_over_the_cover() {
    let fonts = fonts();
    let mut cases = Vec::new();
    for profile in [home(), pro(), PrintProfile::professional_eu()] {
        for trim_authorised in [false, true] {
            for spine in [SpineChoice::Typeset, SpineChoice::CoverColour] {
                let mut req = with_cover(poche(profile.clone()), RightsBasis::RightsHolderDeclared);
                req.trim_authorised = trim_authorised;
                req.spine = spine;
                cases.push(req);
            }
        }
    }
    cases.push(with_cover(poche(home()), RightsBasis::PromoAllMedia));
    let mut guides = with_cover(poche(pro()), RightsBasis::PublicDomain);
    guides.guides = true;
    cases.push(guides);
    for req in cases {
        let (doc, _) = compose(&req, &fonts).unwrap();
        let (rect, clip) = cover_placement(&doc);
        let visible = clip.unwrap_or(rect);
        let leaves = Inspector::new(&doc).leaves(0);
        let index = leaves.iter().position(|l| l.kind == LeafKind::Image).unwrap();
        assert!(index >= 1, "the ground lies under the cover");
        for leaf in &leaves[index + 1..] {
            if leaf.layer.as_deref() == Some(GUIDES_LAYER) {
                continue; // the proofing overlay is not artwork
            }
            assert!(
                leaf.bounds.intersect(&visible.inflate(-1e-6)).is_none(),
                "{}: {:?} at {:?} overlaps the cover {visible:?}",
                req.profile.name,
                leaf.kind,
                leaf.bounds
            );
        }
        // Under the cover there is only flat ground: plain filled rectangles.
        for leaf in &leaves[..index] {
            if leaf.bounds.intersect(&visible.inflate(-1e-6)).is_some() {
                assert_eq!(leaf.kind, LeafKind::Path);
                assert!(leaf.bounds.w > 2.0 && leaf.bounds.h > 100.0, "{:?}", leaf.bounds);
            }
        }
    }
}

#[test]
fn a_printed_cover_gets_a_credit_line_of_at_least_6_pt() {
    let fonts = fonts();
    let credit_of = |doc: &Document| -> Option<(String, f64)> {
        text_runs(doc, 0).into_iter().find(|(text, _)| text.starts_with("Couverture"))
    };
    let back = CardFormat::poche().back();

    // Default: "Couverture : © {publisher}", on the back panel.
    let (doc, warnings) =
        compose(&with_cover(poche(home()), RightsBasis::RightsHolderDeclared), &fonts).unwrap();
    let (text, size) = credit_of(&doc).expect("a credit line");
    assert_eq!(text, "Couverture\u{a0}: ©\u{a0}Éditions du Lac (fictif)");
    assert!(size >= 6.0, "{size} pt");
    assert!(!has_warning(&warnings, &CardWarning::CoverCreditMissing));
    let leaves = Inspector::new(&doc).leaves(0);
    let smallest = leaves
        .iter()
        .filter(|l| l.kind == LeafKind::Text && back.contains_rect(&l.bounds, 0.0))
        .count();
    assert!(smallest >= 5, "product line, tagline, address, imprint, credit");

    // A credit supplied by the caller is printed as given.
    let mut req = with_cover(poche(pro()), RightsBasis::PrintLicence);
    req.cover_credit = Some("© Jeanne Roy / Atelier du Lac".to_owned());
    let (doc, _) = compose(&req, &fonts).unwrap();
    let (text, size) = credit_of(&doc).unwrap();
    assert_eq!(text, "Couverture\u{a0}: © Jeanne Roy / Atelier du Lac");
    assert!(size >= 6.0);

    // Public domain: no © claim. Other languages: translated.
    let mut req = with_cover(poche(home()), RightsBasis::PublicDomain);
    req.locale = Locale::En;
    let (doc, _) = compose(&req, &fonts).unwrap();
    assert!(page_text(&doc, 0).contains("Cover: public domain"));

    // Neither a credit nor a publisher: a generic line and a warning to record.
    let mut req = with_cover(poche(home()), RightsBasis::UserPrivateCopy);
    req.book.publisher = None;
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert!(has_warning(&warnings, &CardWarning::CoverCreditMissing));
    assert_eq!(credit_of(&doc).unwrap().0, "Couverture\u{a0}: tous droits réservés");

    // White text on a dark ground is never smaller than 8 pt.
    let mut req = with_cover(poche(pro()), RightsBasis::PrintLicence);
    req.ground = Some(ebc_color::Rgb8::new(20, 30, 60));
    let (doc, _) = compose(&req, &fonts).unwrap();
    assert!(credit_of(&doc).unwrap().1 >= 8.0);
    assert!(text_runs(&doc, 0).iter().all(|(_, size)| *size >= 8.0));
}

#[test]
fn the_default_spine_takes_nothing_from_the_cover() {
    let fonts = fonts();
    let (doc, warnings) =
        compose(&with_cover(poche(pro()), RightsBasis::PrintLicence), &fonts).unwrap();
    // One image only (the cover): the spine is flat colour and outlines.
    assert_eq!(doc.images.len(), 1);
    assert!(
        !warnings.iter().any(|w| matches!(w, CardWarning::Spine(SpineNote::AuthorisationRequired)))
    );
    // The flat ground is one colour across back, spine and front: every large filled
    // rectangle of the page has the colour of the first.
    let page = &doc.pages[0];
    let Item::Path { fill: Some((ground, _)), .. } = &page.items[0] else { panic!("ground") };
    for item in &page.items[1..] {
        if let Item::Path { path, fill: Some((color, _)), stroke: None } = item
            && path.segs.len() == 5
            && path.control_bounds().is_some_and(|b| b.h > 150.0)
        {
            assert_eq!(color, ground, "the spine's ground is the card's ground");
        }
    }
}

#[test]
fn spine_styles_that_adapt_artwork_need_authorisation_and_a_licence() {
    let fonts = fonts();
    let spine_image = art::cover(120, 2008, false, false);
    let adapting = || {
        [
            SpineChoice::EdgeContinuation,
            SpineChoice::Provided(spine_image.clone()),
            SpineChoice::FullWrap {
                image: art::cover(2600, 2008, false, false),
                spine_px: (1240, 1360),
            },
        ]
    };
    for spine in adapting() {
        let name = spine.name();
        assert!(spine.requires_authorisation());
        for basis in RightsBasis::ALL {
            for authorised in [false, true] {
                let mut req = with_cover(poche(home()), basis);
                req.spine = spine.clone();
                req.adaptation_authorised = authorised;
                let allowed = authorised
                    && matches!(
                        basis,
                        RightsBasis::PrintLicence | RightsBasis::RightsHolderDeclared
                    );
                match compose(&req, &fonts) {
                    Ok((doc, warnings)) => {
                        assert!(allowed, "{name} / {basis} / {authorised} must be refused");
                        assert!(doc.pages.len() == 2 && !warnings.is_empty());
                    }
                    Err(CardError::AdaptationNotAuthorised { style, basis: b, authorised: a }) => {
                        assert!(!allowed, "{name} / {basis} / {authorised} must be accepted");
                        assert_eq!((style, a), (name, authorised));
                        // A cover without rights counts as no cover at all.
                        assert_eq!(b, basis);
                    }
                    Err(other) => panic!("{name} / {basis} / {authorised}: {other}"),
                }
            }
        }
        // Without any cover there is nothing that could be licensed.
        let mut req = poche(home());
        req.spine = spine;
        req.adaptation_authorised = true;
        assert!(matches!(
            compose(&req, &fonts),
            Err(CardError::AdaptationNotAuthorised {
                basis: RightsBasis::None,
                authorised: true,
                ..
            })
        ));
    }

    // Authorised and licensed: the continuation is an image, and the note says so.
    let mut req = with_cover(poche(home()), RightsBasis::PrintLicence);
    req.spine = SpineChoice::EdgeContinuation;
    req.adaptation_authorised = true;
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert_eq!(doc.images.len(), 2, "the cover and the spine's background");
    assert!(
        warnings.contains(&CardWarning::Spine(SpineNote::AuthorisationRequired)),
        "{warnings:?}"
    );
    let (pdf, _) = render_pdf(&req, &fonts).unwrap();
    let report = check(&pdf, &expectations(&req, true)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
}

#[test]
fn a_flat_colour_computed_from_the_cover_needs_no_authorisation() {
    let fonts = fonts();
    let mut req = with_cover(poche(pro()), RightsBasis::PublicDomain);
    req.spine = SpineChoice::CoverColour;
    // By default the ground already is the cover's colour; a white ground set by the
    // caller shows the spine and back in their own colour.
    req.ground = Some(ebc_color::Rgb8::WHITE);
    assert!(!req.spine.requires_authorisation());
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert_eq!(doc.images.len(), 1, "still no pixels of the cover on the spine");
    assert!(
        !warnings.iter().any(|w| matches!(w, CardWarning::Spine(SpineNote::AuthorisationRequired)))
    );
    // Back and spine take the computed colour; it continues 2 mm past the front crease
    // and stops short of the cover.
    let page = &doc.pages[0];
    let Item::Path { path, fill: Some((colour, _)), .. } = &page.items[1] else {
        panic!("back ground")
    };
    let area = path.control_bounds().unwrap();
    let Item::Path { fill: Some((ground, _)), .. } = &page.items[0] else { panic!("ground") };
    assert_ne!(colour, ground);
    assert_eq!(area.x, page.bleed_box().x);
    assert!((area.right() - (CardFormat::poche().fold_x()[1] + 2.0)).abs() < 1e-9, "{area:?}");
    let (rect, _) = cover_placement(&doc);
    assert!(rect.x - area.right() >= 1.0, "the spine colour never reaches the cover");
    let (pdf, _) = render_pdf(&req, &fonts).unwrap();
    let report = check(&pdf, &expectations(&req, true)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));

    // Asked for without a printable cover: the default spine, and a warning.
    let mut req = poche(pro());
    req.spine = SpineChoice::CoverColour;
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert!(doc.images.is_empty());
    assert_eq!(warnings, [CardWarning::SpineChoiceIgnored { style: "cover_colour" }]);
}

#[test]
fn pdf_x_1a_carries_a_converted_cover_but_never_an_rgb_one() {
    let fonts = fonts();
    let mut profile = pro();
    profile.standard = PdfStandard::X1a2001;
    // Converted to CMYK through the output profile: a valid PDF/X-1a file.
    let req = with_cover(poche(profile.clone()), RightsBasis::PrintLicence);
    let (pdf, _) = render_pdf(&req, &fonts).unwrap();
    let report = check(&pdf, &expectations(&req, false)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
    // With an output profile the colour-management module cannot read, the cover could
    // only stay RGB, which PDF/X-1a forbids (even when the profile allows RGB).
    let mut rgb = req.clone();
    rgb.profile.allow_rgb = true;
    rgb.output_profile = Some(
        ebc_card::OutputProfile::new(
            ebc_card::OutputProfile::bundled_cc0_fallback().unwrap().icc().clone(),
            "CGATS TR 001",
            "test",
        )
        .unwrap(),
    );
    assert!(matches!(
        render_pdf(&rgb, &fonts),
        Err(CardError::CoverColourSpace { standard: PdfStandard::X1a2001 })
    ));
    for standard in [PdfStandard::X1a2001, PdfStandard::X3_2002] {
        profile.standard = standard;
        let req = poche(profile.clone());
        let (pdf, warnings) = render_pdf(&req, &fonts).unwrap();
        assert!(warnings.is_empty());
        let report = check(&pdf, &expectations(&req, false)).unwrap();
        assert!(errors(&report).is_empty(), "{standard:?}: {:#?}", errors(&report));
    }
}
