//! Hostile and degenerate input: nothing panics; what cannot be honoured is a typed
//! error, what can be degraded is a warning.

#![allow(clippy::unwrap_used)]

mod common;

use common::{errors, expectations, fonts, page_text, poche, request};
use ebc_card::{CardError, CardWarning, TextField, compose, preview_svg, render_pdf};
use ebc_layout::{CardFormat, LayoutError, PrintProfile, Sheet};
use ebc_model::{Contributor, Locale};
use ebc_preflight::check;
use ebc_scene::Size;

fn clamped(warnings: &[CardWarning], field: TextField) -> bool {
    warnings.contains(&CardWarning::TextClamped { field })
}

fn missing(warnings: &[CardWarning], field: TextField) -> Option<&str> {
    warnings.iter().find_map(|w| match w {
        CardWarning::MissingGlyphs { field: f, characters } if *f == field => {
            Some(characters.as_str())
        }
        _ => None,
    })
}

fn profiles() -> [PrintProfile; 2] {
    [PrintProfile::upload_eu(), PrintProfile::home(Sheet::A4)]
}

#[test]
fn very_long_texts_are_cut_with_a_warning_and_still_print() {
    let fonts = fonts();
    let sentence = "Il était une fois une phrase qui ne voulait pas finir, ";
    let long: String = sentence.chars().cycle().take(2_000).collect();
    let unbroken = "W".repeat(2_000);
    for profile in profiles() {
        for text in [&long, &unbroken] {
            let mut req = poche(profile.clone());
            req.book.title = text.clone();
            req.book.subtitle = Some(text.clone());
            req.book.publisher = Some(text.clone());
            req.book.contributors = vec![Contributor::author(text.clone())];
            // 2 000 characters of running text just fit the panels at the smallest
            // size; three times as much cannot.
            req.message = Some(text.repeat(3));
            req.back_text = Some(text.repeat(3));
            req.giver = Some(text.clone());
            req.recipient = Some(text.clone());
            req.store_name = Some(text.clone());
            req.display_url = text.clone();
            req.help_url = Some(text.clone());
            req.cover_credit = Some(text.clone());
            req.legal_lines = vec![text.clone(); 40];
            let (pdf, warnings) = render_pdf(&req, &fonts).unwrap();
            for field in [
                TextField::Title,
                TextField::Message,
                TextField::BackText,
                TextField::Giver,
                TextField::Recipient,
                TextField::Legal,
            ] {
                assert!(clamped(&warnings, field), "{field} was not reported: {warnings:?}");
            }
            let report = check(&pdf, &expectations(&req, false)).unwrap();
            assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
            // The code is never cut, whatever surrounds it, and nothing spills out of
            // the safe areas.
            let (doc, _) = compose(&req, &fonts).unwrap();
            assert!(page_text(&doc, 1).contains(&req.code.grouped()));
            for (index, panels) in [(0, req.format.outside()), (1, req.format.inside())] {
                let left = ebc_layout::safe_rect(panels.left, &req.profile);
                let right = ebc_layout::safe_rect(panels.right, &req.profile);
                for leaf in ebc_card::inspect::Inspector::new(&doc).leaves(index) {
                    if leaf.kind == ebc_card::inspect::LeafKind::Text && leaf.layer.is_none() {
                        let b = leaf.bounds;
                        assert!(
                            left.contains_rect(&b, 0.01) || right.contains_rect(&b, 0.01),
                            "page {index}: text at {b:?} leaves the safe areas"
                        );
                    }
                }
            }
            // A 2 000-character message alone is set small rather than cut, without
            // overflowing either.
            let mut req = poche(req.profile.clone());
            req.message = Some(text.clone());
            let (doc, _) = compose(&req, &fonts).unwrap();
            let left = ebc_layout::safe_rect(req.format.inside_front(), &req.profile);
            let runs = ebc_card::inspect::Inspector::new(&doc).leaves(1);
            assert!(runs.iter().filter(|l| left.contains_rect(&l.bounds, 0.01)).count() > 10);
        }
    }
}

#[test]
fn characters_the_fonts_lack_are_left_out_and_reported() {
    let fonts = fonts();
    for profile in profiles() {
        let mut req = poche(profile);
        req.book.title = "Les Vagues 🌊 du soir 夜の波".to_owned();
        req.message =
            Some("Bonne lecture 😀👍🏽 ! 読書を楽しんで\nשלום עולם — مرحبا بالعالم".to_owned());
        req.giver = Some("Алиса 👩‍👩‍👧".to_owned());
        req.recipient = Some("Bob\u{200d}\u{fe0f}".to_owned());
        let (pdf, warnings) = render_pdf(&req, &fonts).unwrap();
        let title = missing(&warnings, TextField::Title).expect("title warning");
        assert!(title.contains("U+1F30A") && title.contains("U+591C"), "{title}");
        let message = missing(&warnings, TextField::Message).expect("message warning");
        assert!(message.contains("U+1F600") && message.contains("U+05E9"), "{message}");
        assert!(missing(&warnings, TextField::Giver).is_some());
        let (doc, _) = compose(&req, &fonts).unwrap();
        // What can be drawn is drawn; no `.notdef` glyph is ever placed.
        let text = format!("{}\n{}", page_text(&doc, 0), page_text(&doc, 1));
        assert!(text.contains("Les Vagues du soir"), "{text}");
        assert!(text.contains("Bonne lecture !"), "{text}");
        fn no_notdef(item: &ebc_scene::Item) {
            match item {
                ebc_scene::Item::Glyphs { glyphs, .. } => {
                    assert!(glyphs.iter().all(|g| g.id != 0), "a .notdef glyph was placed");
                }
                ebc_scene::Item::Group { items, .. } => items.iter().for_each(no_notdef),
                _ => {}
            }
        }
        doc.pages.iter().flat_map(|p| &p.items).for_each(no_notdef);
        let report = check(&pdf, &expectations(&req, false)).unwrap();
        assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
    }
}

#[test]
fn a_title_the_fonts_cannot_draw_at_all_still_gives_a_card() {
    let fonts = fonts();
    let mut req = poche(PrintProfile::home(Sheet::A4));
    req.book.title = "吾輩は猫である".to_owned();
    req.book.contributors = vec![Contributor::author("夏目漱石")];
    let (pdf, warnings) = render_pdf(&req, &fonts).unwrap();
    assert!(missing(&warnings, TextField::Title).is_some());
    assert!(warnings.iter().any(|w| matches!(w, CardWarning::Spine(_))), "{warnings:?}");
    assert!(pdf.starts_with(b"%PDF-"));
    // The metadata keeps the real title.
    let (doc, _) = compose(&req, &fonts).unwrap();
    assert!(doc.info.title.starts_with("吾輩は猫である"));
}

#[test]
fn empty_and_blank_strings() {
    let fonts = fonts();
    // A blank title is the one thing that cannot be degraded.
    for title in ["", "   ", "\n\t"] {
        let mut req = poche(PrintProfile::upload_eu());
        req.book.title = title.to_owned();
        assert!(matches!(compose(&req, &fonts), Err(CardError::EmptyTitle)), "{title:?}");
    }
    // Everything else may be empty: it is simply not printed.
    let mut req = poche(PrintProfile::home(Sheet::A4));
    req.book.subtitle = Some(String::new());
    req.book.publisher = Some("  ".to_owned());
    req.book.contributors = vec![Contributor::author("")];
    req.giver = Some(String::new());
    req.recipient = Some(" \n ".to_owned());
    req.message = Some("\n\n".to_owned());
    req.back_text = Some(String::new());
    req.store_name = Some(String::new());
    req.help_url = Some(String::new());
    req.cover_credit = Some(String::new());
    req.legal_lines = vec![String::new(), "  ".to_owned()];
    req.display_url = String::new();
    let (doc, warnings) = compose(&req, &fonts).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let inside = page_text(&doc, 1);
    assert!(!inside.contains("Pour ") || inside.contains("Pour le lire"), "{inside}");
    assert!(!inside.contains("De la part"), "{inside}");
    // Without an address the host of the claim URL is printed.
    assert!(inside.contains("ebk.example"), "{inside}");
    // Steps of the open flow, not the store's.
    assert!(inside.contains("Choisir où lire"));
}

#[test]
fn control_characters_markup_and_placeholders_are_plain_text() {
    let fonts = fonts();
    let mut req = poche(PrintProfile::upload_eu());
    req.book.title = "Les\u{0}Vagues\r\n\tdu\u{1b}[31m soir".to_owned();
    req.giver = Some("{name} {url} {store} %s <b>Alice</b>".to_owned());
    req.recipient = Some("Bob\u{202e}evil\u{feff}".to_owned());
    req.message = Some("ligne 1\r\nligne 2\u{2028}ligne 3\u{0007}".to_owned());
    req.store_name = Some("{store} & Co <script>".to_owned());
    let (pdf, _) = render_pdf(&req, &fonts).unwrap();
    let (doc, _) = compose(&req, &fonts).unwrap();
    let outside = page_text(&doc, 0);
    let inside = page_text(&doc, 1);
    assert!(inside.contains("{name} {url} {store} %s <b>Alice</b>"), "{inside}");
    assert!(inside.replace('\n', " ").contains("{store} & Co <script>"), "{inside}");
    assert!(inside.contains("ligne 1\nligne 2\nligne 3"), "{inside}");
    assert!(!outside.contains('\u{0}') && !outside.contains('\u{1b}'));
    let report = check(&pdf, &expectations(&req, false)).unwrap();
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
    // The preview does not choke on any of it either.
    let svgs = preview_svg(&req, &fonts).unwrap();
    assert_eq!(svgs.len(), 2);
    assert!(svgs.iter().all(|svg| svg.starts_with("<svg") && svg.ends_with("</svg>")));
}

#[test]
fn unusable_claim_urls_are_typed_errors_and_odd_ones_are_reported() {
    let fonts = fonts();
    for url in ["", "javascript:alert(1)", "ebk.example/k/x", "https://", "https://a b/k/c"] {
        let mut req = poche(PrintProfile::upload_eu());
        req.claim_url = url.to_owned();
        assert!(matches!(compose(&req, &fonts), Err(CardError::InvalidClaimUrl(_))), "{url:?}");
    }
    // A store's own redemption link: encoded as given, with a note.
    let mut req = poche(PrintProfile::upload_eu());
    req.claim_url = "https://store.example/Gifts/redeem?c=AbCdEf".to_owned();
    let (_, warnings) = compose(&req, &fonts).unwrap();
    assert!(warnings.contains(&CardWarning::QrPayloadNotUpperCased));
    // A very long link needs a dense symbol: reported, the modules stay printable or the
    // warning says they are not.
    req.claim_url = format!("https://store.example/redeem?token={}", "aZ9".repeat(120));
    let (pdf, warnings) = render_pdf(&req, &fonts).unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, CardWarning::QrVersionHigh { version } if *version > 10))
    );
    assert!(warnings.iter().any(|w| matches!(
        w,
        CardWarning::QrModuleSmall { .. } | CardWarning::QrModuleBelowMinimum { .. }
    )));
    assert!(pdf.starts_with(b"%PDF-"));
    // Longer than any QR code can hold: a typed error.
    req.claim_url = format!("https://store.example/{}", "x".repeat(8_000));
    assert!(matches!(compose(&req, &fonts), Err(CardError::Qr(_))));
}

#[test]
fn unusable_formats_and_profiles_are_typed_errors() {
    let fonts = fonts();
    let mut req = poche(PrintProfile::upload_eu());
    req.format.cover = Size::new(f64::NAN, 178.0);
    assert!(matches!(compose(&req, &fonts), Err(CardError::Layout(_))));
    req.format = CardFormat::poche();
    req.format.spine_width = -4.0;
    assert!(matches!(compose(&req, &fonts), Err(CardError::Layout(_))));
    req.format = CardFormat::poche();
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        let mut req = req.clone();
        req.profile.bleed = bad;
        assert!(matches!(compose(&req, &fonts), Err(CardError::Layout(_))), "bleed {bad}");
        let mut req = poche(PrintProfile::upload_eu());
        req.profile.safe_margin = bad;
        assert!(matches!(compose(&req, &fonts), Err(CardError::Layout(_))), "safe margin {bad}");
    }
    // A card that does not fit its sheet is refused, never scaled.
    let req = request(CardFormat::a5(), PrintProfile::home(Sheet::A4), Locale::Fr);
    assert!(matches!(
        compose(&req, &fonts),
        Err(CardError::Layout(LayoutError::DoesNotFit { .. }))
    ));
}

#[test]
fn small_and_odd_custom_formats_never_panic() {
    let fonts = fonts();
    let mut composed = 0;
    for (w, h, spine) in [
        (50.0, 50.0, 3.0),
        (50.0, 300.0, 3.0),
        (300.0, 50.0, 40.0),
        (60.0, 90.0, 5.0),
        (70.0, 100.0, 6.0),
        (80.0, 120.0, 8.0),
        (90.0, 140.0, 10.0),
        (100.0, 100.0, 10.0),
        (148.0, 210.0, 12.0),
        (300.0, 300.0, 40.0),
    ] {
        let format = CardFormat::custom(Size::new(w, h), spine).unwrap();
        for locale in [Locale::Fr, Locale::De] {
            let mut req = request(format, PrintProfile::upload_eu(), locale);
            req.store_name = Some("Grande Enseigne (démo)".to_owned());
            match render_pdf(&req, &fonts) {
                Ok((pdf, warnings)) => {
                    composed += 1;
                    let report = check(&pdf, &expectations(&req, false)).unwrap();
                    assert!(errors(&report).is_empty(), "{w}x{h}: {:#?}", errors(&report));
                    let (doc, _) = compose(&req, &fonts).unwrap();
                    assert!(page_text(&doc, 1).contains(&req.code.grouped()), "{w}x{h}");
                    if w < 100.0 || h < 150.0 {
                        assert!(!warnings.is_empty(), "{w}x{h}: a tight card says what it cut");
                    }
                }
                Err(CardError::FormatTooSmall { needed_mm, available_mm, .. }) => {
                    assert!(needed_mm > available_mm, "{w}x{h}");
                    assert!(w < 80.0 || h < 120.0, "{w}x{h} is large enough for a card");
                }
                Err(other) => panic!("{w}x{h}: {other}"),
            }
        }
    }
    assert!(composed >= 8, "{composed}");
}

#[test]
fn every_locale_and_the_store_variant_print_their_own_strings() {
    let fonts = fonts();
    for locale in Locale::ALL {
        let mut req = request(CardFormat::poche(), PrintProfile::home(Sheet::A4), locale);
        let (doc, warnings) = compose(&req, &fonts).unwrap();
        assert!(warnings.is_empty(), "{locale}: {warnings:?}");
        let inside = page_text(&doc, 1);
        for msg in [ebc_i18n::Msg::CardHeadline, ebc_i18n::Msg::CardHowTo, ebc_i18n::Msg::CodeLabel]
        {
            assert!(inside.contains(ebc_i18n::t(locale, msg)), "{locale}: {msg:?}");
        }
        // The three steps are numbered, and set at the 12 pt they are designed for.
        let steps: Vec<_> = common::text_runs(&doc, 1)
            .into_iter()
            .filter(|(text, _)| ["1", "2", "3"].contains(&text.as_str()))
            .collect();
        assert_eq!(steps.len(), 3, "{locale}");
        assert!(steps.iter().all(|(_, size)| *size == 12.0), "{locale}: {steps:?}");
        // The address in lower case, the code grouped at 16 pt.
        assert!(inside.contains("ebk.example"));
        let code = common::text_runs(&doc, 1)
            .into_iter()
            .find(|(text, _)| *text == req.code.grouped())
            .expect("the grouped code");
        assert_eq!(code.1, 16.0, "{locale}");
        // No stored-value wording slipped in through the composer.
        assert!(!inside.to_lowercase().contains("qr"), "{locale}: the card never says QR");

        req.store_name = Some("Septante (démo)".to_owned());
        req.display_url = "EBK.Example".to_owned();
        let (doc, _) = compose(&req, &fonts).unwrap();
        let inside = page_text(&doc, 1);
        let outside = page_text(&doc, 0);
        assert!(inside.replace('\n', " ").contains("Septante (démo)"), "{locale}: {inside}");
        assert!(outside.contains("Septante (démo) ·"), "{locale}: imprint names the store");
        assert!(inside.contains("ebk.example") && !inside.contains("EBK.Example"));
    }
}
