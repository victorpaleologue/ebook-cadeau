//! Writes sample cards for every preset into `target/tmp/ebc-card-samples/` — and checks
//! each of them with the independent preflight, so that the samples are also a test.
//!
//! The book, the people, the store and the address are invented; the cover is an
//! own-made drawing (`tests/common/art.rs`).

#![allow(clippy::unwrap_used)]

mod common;

use common::{errors, expectations, fonts, request, sample_dir, with_cover};
use ebc_card::{CardRequest, RightsBasis, SpineChoice, preview_svg, render_pdf};
use ebc_layout::{CardFormat, PrintProfile, page_geometry};
use ebc_model::Locale;
use ebc_preflight::check;

fn write(name: &str, req: &CardRequest) -> usize {
    let (pdf, _) = render_pdf(req, &fonts()).unwrap();
    let mut expect = expectations(req, false);
    if req.guides {
        // A proof is not a print file: a plain PDF with a layer, no PDF/X label.
        assert!(!pdf.windows(9).any(|w| w == b"/GTS_PDFX"), "{name}: a proof claims PDF/X");
        expect.require_output_intent = false;
    }
    let report = check(&pdf, &expect).unwrap();
    assert!(errors(&report).is_empty(), "{name}: {:#?}", errors(&report));
    std::fs::write(sample_dir().join(format!("{name}.pdf")), &pdf).unwrap();
    pdf.len()
}

#[test]
fn write_a_sample_for_every_preset() {
    let mut written = Vec::new();
    for (format_name, format) in CardFormat::presets() {
        for profile_name in ["upload_eu", "professional_eu", "home_a4", "home_letter", "home_a3"] {
            let profile = PrintProfile::by_name(profile_name).unwrap();
            if page_geometry(&format, &profile).is_err() {
                continue; // the digest and A5 cards do not fit an A4 or Letter sheet
            }
            let typographic = request(format, profile.clone(), Locale::Fr);
            let name = format!("{format_name}-{profile_name}-typographic");
            written.push((name.clone(), write(&name, &typographic)));
            let cover = with_cover(typographic, RightsBasis::RightsHolderDeclared);
            let name = format!("{format_name}-{profile_name}-cover");
            written.push((name.clone(), write(&name, &cover)));
        }
    }
    assert!(written.len() >= 28, "{}", written.len());

    // The variants of the rights rules, on the default card.
    let poche = |profile: &str, locale| {
        request(CardFormat::poche(), PrintProfile::by_name(profile).unwrap(), locale)
    };
    let mut reduced = with_cover(poche("home_a4", Locale::Fr), RightsBasis::PromoAllMedia);
    reduced.cover_credit = Some("© Éditions du Lac (fictif) / J. Roy".to_owned());
    written.push((
        "poche-home_a4-promo-reduced".to_owned(),
        write("poche-home_a4-promo-reduced", &reduced),
    ));

    let mut full_bleed =
        with_cover(poche("professional_eu", Locale::Fr), RightsBasis::PrintLicence);
    full_bleed.trim_authorised = true;
    written.push((
        "poche-professional_eu-full-bleed".to_owned(),
        write("poche-professional_eu-full-bleed", &full_bleed),
    ));

    let mut colour = with_cover(poche("upload_eu", Locale::Fr), RightsBasis::PrintLicence);
    colour.spine = SpineChoice::CoverColour;
    written.push((
        "poche-upload_eu-cover-colour-spine".to_owned(),
        write("poche-upload_eu-cover-colour-spine", &colour),
    ));

    let mut continuation =
        with_cover(poche("home_a4", Locale::Fr), RightsBasis::RightsHolderDeclared);
    continuation.spine = SpineChoice::EdgeContinuation;
    continuation.adaptation_authorised = true;
    written.push((
        "poche-home_a4-edge-continuation-authorised".to_owned(),
        write("poche-home_a4-edge-continuation-authorised", &continuation),
    ));

    let mut store = poche("home_a4", Locale::En);
    store.store_name = Some("Septante (demo — not affiliated)".to_owned());
    store.message = None;
    store.legal_lines = vec!["EPUB, to read with a Septante account.".to_owned()];
    store.help_url = Some("ebk.example/help".to_owned());
    store.expires_unix = Some(1_830_000_000);
    written.push((
        "poche-home_a4-en-store-handwriting".to_owned(),
        write("poche-home_a4-en-store-handwriting", &store),
    ));

    let mut proof = with_cover(poche("upload_eu", Locale::Fr), RightsBasis::PublicDomain);
    proof.guides = true;
    written.push((
        "poche-upload_eu-proof-with-guides".to_owned(),
        write("poche-upload_eu-proof-with-guides", &proof),
    ));

    for locale in Locale::ALL {
        let name = format!("poche-home_a4-typographic-{locale}");
        written.push((name.clone(), write(&name, &poche("home_a4", locale))));
    }

    // SVG previews of one card, next to the PDFs.
    let svgs = preview_svg(
        &with_cover(poche("home_a4", Locale::Fr), RightsBasis::RightsHolderDeclared),
        &fonts(),
    )
    .unwrap();
    for (svg, name) in svgs.iter().zip(["preview-outside.svg", "preview-inside.svg"]) {
        std::fs::write(sample_dir().join(name), svg).unwrap();
    }

    for (name, size) in &written {
        eprintln!("{:>9} bytes  {name}.pdf", size);
    }
    eprintln!("{} samples in {}", written.len(), sample_dir().display());
}
