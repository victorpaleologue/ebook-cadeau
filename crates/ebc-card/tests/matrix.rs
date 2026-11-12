//! Every preset format × every built-in profile × every locale composes into a PDF that
//! passes the independent preflight with zero errors — or is refused with a typed layout
//! error when the card does not fit the home sheet. Also: byte-for-byte determinism and
//! the document information.

#![allow(clippy::unwrap_used)]

mod common;

use common::{
    errors, expectations, fonts, poche, request, warnings, with_cover, without_crease_advice,
};
use ebc_card::{CardError, RightsBasis, compose, render_pdf};
use ebc_layout::{CardFormat, LayoutError, PrintProfile, Sheet};
use ebc_model::Locale;
use ebc_preflight::check;
use ebc_scene::PdfStandard;

#[test]
fn every_format_profile_and_locale_passes_preflight() {
    let fonts = fonts();
    let mut composed = 0;
    let mut refused = Vec::new();
    let mut preflight_warnings = std::collections::BTreeSet::new();
    for (format_name, format) in CardFormat::presets() {
        for profile_name in PrintProfile::NAMES {
            let profile = PrintProfile::by_name(profile_name).unwrap();
            for locale in Locale::ALL {
                let req = request(format, profile.clone(), locale);
                let label = format!("{format_name} / {profile_name} / {locale}");
                match render_pdf(&req, &fonts) {
                    Ok((pdf, card_warnings)) => {
                        let rest = without_crease_advice(&card_warnings);
                        assert!(rest.is_empty(), "{label}: {card_warnings:?}");
                        // Lamination advice only for professional files.
                        if profile.standard == PdfStandard::Plain {
                            assert!(card_warnings.is_empty(), "{label}: {card_warnings:?}");
                        }
                        let report = check(&pdf, &expectations(&req, false)).unwrap();
                        assert!(errors(&report).is_empty(), "{label}: {:#?}", errors(&report));
                        assert_eq!(report.pages.len(), 2, "{label}");
                        // Heavy ink at a crease is advice the card gives too: the referee
                        // and the composer agree (the composer also flags dark K-only
                        // grounds, which the referee, judging ink alone, does not).
                        if report.has(ebc_preflight::codes::INK_HEAVY_AT_CREASE) {
                            assert_ne!(rest.len(), card_warnings.len(), "{label}: no advice");
                        }
                        for w in warnings(&report) {
                            if !w.starts_with(ebc_preflight::codes::INK_HEAVY_AT_CREASE) {
                                preflight_warnings.insert(w);
                            }
                        }
                        // The QR code is found by the referee, with printable modules.
                        let qr = &report.pages[1].qr_candidates;
                        assert_eq!(qr.len(), 1, "{label}: {qr:?}");
                        assert!(qr[0].module_mm >= 0.5 && qr[0].side_mm >= 20.0, "{label}: {qr:?}");
                        composed += 1;
                    }
                    Err(CardError::Layout(
                        LayoutError::DoesNotFit { .. } | LayoutError::MarksDoNotFit { .. },
                    )) => {
                        assert!(profile.sheet.is_some(), "{label}: only a sheet can be too small");
                        refused.push(label);
                    }
                    Err(other) => panic!("{label}: {other}"),
                }
            }
        }
    }
    eprintln!("{composed} cards composed; refused because the sheet is too small: {refused:?}");
    eprintln!("preflight warnings seen: {preflight_warnings:#?}");
    // 4 formats × 6 profiles × 7 locales. Press profiles always work; the poche and A6
    // fit every sheet; the digest and A5 need a sheet larger than A4 or Letter.
    assert_eq!(composed + refused.len(), 168);
    assert!(composed >= 4 * 2 * 7 + 2 * 4 * 7, "{composed}");
    assert!(refused.iter().all(|label| label.starts_with("digest") || label.starts_with("a5")));
    assert!(refused.iter().any(|label| label.contains("home_a4")));
    assert!(preflight_warnings.is_empty(), "{preflight_warnings:#?}");
}

#[test]
fn cards_with_a_cover_pass_preflight_on_both_routes() {
    let fonts = fonts();
    for profile in [
        PrintProfile::upload_eu(),
        PrintProfile::professional_eu(),
        PrintProfile::home(Sheet::A4),
        PrintProfile::home(Sheet::Letter),
    ] {
        let req = with_cover(poche(profile.clone()), RightsBasis::RightsHolderDeclared);
        let (pdf, _) = render_pdf(&req, &fonts).unwrap();
        let report = check(&pdf, &expectations(&req, true)).unwrap();
        assert!(errors(&report).is_empty(), "{}: {:#?}", profile.name, errors(&report));
        assert_eq!(report.pages[0].images.len(), 1);
        let image = &report.pages[0].images[0];
        assert!(image.min_ppi() >= 300.0, "{image:?}");
        assert!(
            (image.ppi_x - image.ppi_y).abs() / image.ppi_x < 0.001,
            "uniform scale: {image:?}"
        );
    }
}

#[test]
fn output_is_deterministic() {
    let fonts = fonts();
    for req in [
        poche(PrintProfile::professional_eu()),
        poche(PrintProfile::home(Sheet::A4)),
        with_cover(poche(PrintProfile::upload_eu()), RightsBasis::PrintLicence),
        with_cover(poche(PrintProfile::home(Sheet::Letter)), RightsBasis::UserPrivateCopy),
    ] {
        let (first, first_warnings) = render_pdf(&req, &fonts).unwrap();
        let (second, second_warnings) = render_pdf(&req, &fonts).unwrap();
        assert!(first == second, "{}: the two renderings differ", req.profile.name);
        assert_eq!(first_warnings, second_warnings);
        let (a, _) = compose(&req, &fonts).unwrap();
        let (b, _) = compose(&req, &fonts).unwrap();
        assert_eq!(a, b);
    }
}

#[test]
fn document_information() {
    let fonts = fonts();
    let req = poche(PrintProfile::professional_eu());
    let (doc, _) = compose(&req, &fonts).unwrap();
    assert_eq!(doc.info.title, "Les Vagues du soir — eBook Cadeau");
    assert_eq!(doc.info.author.as_deref(), Some("Jeanne Roy"));
    assert_eq!(doc.info.language.as_deref(), Some("fr-FR"));
    assert_eq!(doc.info.created_unix, Some(common::CREATED_UNIX));
    assert_eq!(doc.info.creator, Some(format!("ebc-card {}", env!("CARGO_PKG_VERSION"))));
    let get = |key: &str| doc.info.custom.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
    assert_eq!(get("EBC.CoverWidthMm"), Some("110"));
    assert_eq!(get("EBC.CoverHeightMm"), Some("178"));
    assert_eq!(get("EBC.SpineWidthMm"), Some("10"));
    assert_eq!(get("EBC.FoldXMm"), Some("110,120"));
    assert_eq!(get("EBC.Profile"), Some("professional_eu"));
    assert_eq!(get("EBC.Format"), Some("poche"));
    assert_eq!(get("EBC.RightsBasis"), Some("none"));
    assert_eq!(get("EBC.QrModuleMm"), Some("1.000"));
    // The redemption code is printed on the card but never written to the metadata.
    let code = common::code();
    let meta = format!("{:?}", doc.info);
    assert!(!meta.contains(&code.compact()) && !meta.contains(&code.grouped()), "{meta}");
    assert_eq!(doc.standard, PdfStandard::X4);
    let intent = doc.output_intent.as_ref().unwrap();
    assert_eq!((intent.condition_identifier.as_str(), intent.components), ("FOGRA39", 4));

    // A custom format is named as such; a home sheet has no output intent; the locale
    // sets the language.
    let mut custom = request(
        CardFormat::custom(ebc_scene::Size::new(100.0, 150.0), 8.0).unwrap(),
        PrintProfile::home(Sheet::A4),
        Locale::De,
    );
    custom.created_unix = None;
    let (doc, _) = compose(&custom, &fonts).unwrap();
    assert!(doc.info.custom.contains(&("EBC.Format".to_owned(), "custom".to_owned())));
    assert!(doc.output_intent.is_none() && doc.standard == PdfStandard::Plain);
    assert_eq!(doc.info.language.as_deref(), Some("de-DE"));
    assert_eq!(doc.info.created_unix, None);
}

#[test]
fn the_default_output_profile_is_a_printer_profile_and_others_are_reported() {
    use ebc_card::{CardWarning, OutputProfile};
    let fonts = fonts();
    let mut req = poche(PrintProfile::upload_eu());
    let (pdf, card_warnings) = render_pdf(&req, &fonts).unwrap();
    assert!(without_crease_advice(&card_warnings).is_empty(), "{card_warnings:?}");
    let report = check(&pdf, &expectations(&req, false)).unwrap();
    assert!(!report.has(ebc_preflight::codes::OUTPUT_INTENT_INVALID), "{:?}", report.codes());
    let intent = report.document.output_intent.as_ref().unwrap();
    assert_eq!(intent.identifier.as_deref(), Some("FOGRA39"));
    assert!(intent.profile_embedded);

    // The CC0 fallback: a test aid. The composer flags it (as an input-class profile
    // without B2A tables it cannot be converted to either: colours are not managed),
    // and the referee refuses it as the output intent of a PDF/X file (check A6: a
    // scanner profile is not a printing condition).
    req.output_profile = Some(OutputProfile::bundled_cc0_fallback().unwrap());
    let (pdf, card_warnings) = render_pdf(&req, &fonts).unwrap();
    assert_eq!(without_crease_advice(&card_warnings), [CardWarning::FallbackOutputProfile]);
    // The same file named as the caller's own profile: not managed, and said so.
    let mut own_cc0 = req.clone();
    own_cc0.output_profile = Some(
        OutputProfile::new(
            OutputProfile::bundled_cc0_fallback().unwrap().icc().clone(),
            "CGATS TR 001",
            "x",
        )
        .unwrap(),
    );
    let (_, own_warnings) = compose(&own_cc0, &fonts).unwrap();
    assert!(own_warnings.iter().any(|w| matches!(w, CardWarning::ColourNotManaged { .. })));
    assert_eq!(
        without_crease_advice(&card_warnings)[0].to_string(),
        "fallback output profile — supply FOGRA39/FOGRA51 for European printers"
    );
    let report = check(&pdf, &expectations(&req, false)).unwrap();
    assert_eq!(
        report.errors().map(|f| f.code).collect::<Vec<_>>(),
        [ebc_preflight::codes::OUTPUT_INTENT_INVALID],
        "{:#?}",
        errors(&report)
    );

    // A caller's own profile is embedded under the names the caller gives.
    let own = OutputProfile::new(
        OutputProfile::bundled_fogra39().unwrap().icc().clone(),
        "FOGRA51",
        "PSO Coated v3 (stand-in for the test)",
    )
    .unwrap();
    req.output_profile = Some(own);
    let (doc, card_warnings) = compose(&req, &fonts).unwrap();
    assert!(without_crease_advice(&card_warnings).is_empty(), "{card_warnings:?}");
    assert_eq!(doc.output_intent.unwrap().condition_identifier, "FOGRA51");

    // PDF/X needs a creation time; the composer does not read the clock.
    req.created_unix = None;
    assert!(matches!(compose(&req, &fonts), Err(CardError::MissingCreationDate)));
}
