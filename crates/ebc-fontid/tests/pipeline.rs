//! The whole pipeline on synthetic covers drawn with the bundled fonts: detection,
//! labelling, case handling, identification, re-ranking and the evaluation harness.
//!
//! Covers are never the index glyphs in disguise: titles are shaped as words (kerning,
//! ligatures), drawn at other sizes than the 96 px/em reference glyphs, supersampled
//! with a random sub-pixel offset and, in the degraded tests, rotated, blurred, made
//! noisy and JPEG-compressed.

mod common;

use common::{bundled_fonts, clean_spec, cover, degraded_spec, fresh_index, is};
use ebc_fontid::eval::{Condition, EvalOptions, evaluate, evaluate_holdout, is_detected};
use ebc_fontid::synth::{Background, Degradation};
use ebc_fontid::{
    CaseUsed, FontIndex, IndexEntry, bundled_index, find_text_regions, find_title_regions,
    identify, identify_detailed, rerank, rerank_detailed,
};
use ebc_text::Font;

const TITLES: [&str; 4] =
    ["Le Rouge et le Noir", "Bonjour tristesse", "Wuthering Heights", "Germinal"];

#[test]
fn clean_covers_are_identified_to_the_exact_face() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    let (mut face, mut family, mut confident, mut n) = (0, 0, 0, 0);
    for (i, (meta, _)) in fonts.iter().enumerate() {
        for (k, cap) in [31.0f32, 47.0].into_iter().enumerate() {
            let title = TITLES[(i + k) % TITLES.len()];
            let c = cover(&fonts, i, &clean_spec(&[title], cap, (i * 7 + k) as u64));
            let id = identify_detailed(&c.image, &c.title_region, Some(title), &index);
            assert_eq!(id.matches.len(), 8);
            assert!(id.labelled_glyphs >= 6, "{meta:?} {title}: {id:?}");
            assert!((id.text_coverage - 1.0).abs() < 1e-6);
            assert!(id.matches.windows(2).all(|p| p[0].score >= p[1].score));
            assert!(id.matches.iter().all(|m| (0.0..=1.0).contains(&m.score)));
            n += 1;
            face += usize::from(is(&id.matches[0].entry, meta));
            family += usize::from(id.matches[0].entry.family == meta.family);
            confident += usize::from(id.confident && id.matches[0].entry.family == meta.family);
            assert!(
                !id.confident || id.matches[0].entry.family == meta.family,
                "confidently wrong"
            );
        }
    }
    // Measured: 16/16 for all three. The floor leaves room for one miss.
    assert!(family * 16 >= n * 15, "family top-1 {family}/{n}");
    assert!(face * 16 >= n * 14, "face top-1 {face}/{n}");
    assert!(confident * 16 >= n * 12, "confident and right {confident}/{n}");
}

#[test]
fn degraded_covers_degrade_gracefully() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    let (mut family, mut top3, mut n) = (0, 0, 0);
    for (i, (meta, _)) in fonts.iter().enumerate() {
        let title = TITLES[i % TITLES.len()];
        let c = cover(&fonts, i, &degraded_spec(&[title], 38.0, i as u64 + 40));
        let id = identify_detailed(&c.image, &c.title_region, Some(title), &index);
        n += 1;
        family += usize::from(id.matches.first().is_some_and(|m| m.entry.family == meta.family));
        top3 += usize::from(id.matches.iter().take(3).any(|m| is(&m.entry, meta)));
    }
    // Measured: 8/8 and 8/8.
    assert!(family * 8 >= n * 6, "family top-1 {family}/{n} on degraded covers");
    assert!(top3 * 8 >= n * 6, "face in top-3 {top3}/{n} on degraded covers");
}

#[test]
fn the_case_on_the_cover_is_inferred() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    // Spectral Regular: a font with distinct upper and lower case.
    let spectral = fonts.iter().position(|(m, _)| m.family == "Spectral").expect("bundled");
    let catalogue = "Le Rouge et le Noir";
    for (shown, expected) in [
        ("Le Rouge et le Noir", CaseUsed::AsGiven),
        ("LE ROUGE ET LE NOIR", CaseUsed::Upper),
        ("le rouge et le noir", CaseUsed::Lower),
        ("Le Rouge Et Le Noir", CaseUsed::Title),
    ] {
        let c = cover(&fonts, spectral, &clean_spec(&[shown], 40.0, 5));
        let id = identify_detailed(&c.image, &c.title_region, Some(catalogue), &index);
        assert_eq!(id.case_used, expected, "cover shows {shown:?}");
        assert_eq!(id.labelled_glyphs, 15);
        assert_eq!(id.matches[0].entry.family, "Spectral", "cover shows {shown:?}");
    }
    // A catalogue title in capitals, set in lower case on the cover.
    let c = cover(&fonts, spectral, &clean_spec(&["germinal"], 40.0, 6));
    let id = identify_detailed(&c.image, &c.title_region, Some("GERMINAL"), &index);
    assert_eq!(id.case_used, CaseUsed::Lower);
}

#[test]
fn accents_punctuation_and_several_lines_are_labelled() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    // "L'Étranger": 9 letters, an apostrophe and an accent that must not count.
    let c = cover(&fonts, 0, &clean_spec(&["L'Étranger"], 44.0, 2));
    let id = identify_detailed(&c.image, &c.title_region, Some("L'Étranger"), &index);
    assert_eq!((id.glyphs, id.labelled_glyphs, id.lines), (9, 9, 1), "{id:?}");
    assert!(is(&id.matches[0].entry, &fonts[0].0));

    // Three lines; "Où es-tu ?" has a hyphen, a question mark and a grave accent.
    let lines = ["Voyage au bout", "de la nuit", "Où es-tu ?"];
    let title = lines.join(" ");
    let c = cover(&fonts, 3, &clean_spec(&lines, 34.0, 4));
    let id = identify_detailed(&c.image, &c.title_region, Some(&title), &index);
    assert_eq!(id.lines, 3);
    assert_eq!(id.labelled_glyphs, 12 + 8 + 7, "{id:?}");
    assert!(is(&id.matches[0].entry, &fonts[3].0));

    // A box around the middle line only: part of the title, still labelled.
    let mut middle = c.title_region;
    middle.y += middle.h * 4 / 11;
    middle.h = middle.h * 3 / 11;
    let id = identify_detailed(&c.image, &middle, Some(&title), &index);
    assert_eq!(id.lines, 1, "{id:?}");
    assert_eq!(id.labelled_glyphs, 8, "{id:?}");
    assert!(id.text_coverage > 0.25 && id.text_coverage < 0.35, "{}", id.text_coverage);
}

#[test]
fn without_the_title_the_fallback_is_used_and_never_confident() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    let mut family = 0;
    for (i, (meta, _)) in fonts.iter().enumerate() {
        let c = cover(&fonts, i, &clean_spec(&["Wuthering Heights"], 42.0, i as u64));
        let id = identify_detailed(&c.image, &c.title_region, None, &index);
        assert_eq!((id.labelled_glyphs, id.case_used), (0, CaseUsed::Unknown));
        assert!(!id.confident);
        assert!(id.matches.iter().all(|m| m.score <= 0.8 + 1e-6), "fallback scores are capped");
        family += usize::from(id.matches[0].entry.family == meta.family);
        // A text that does not fit the glyphs falls back too, instead of mislabelling.
        let wrong = identify_detailed(
            &c.image,
            &c.title_region,
            Some("Completely unrelated words"),
            &index,
        );
        assert_eq!(wrong.labelled_glyphs, 0, "{wrong:?}");
        assert_eq!(wrong.matches[0].entry, id.matches[0].entry);
    }
    assert!(family >= 7, "family top-1 without the title: {family}/8");
}

#[test]
fn the_detector_finds_the_title_with_and_without_the_prior() {
    let fonts = bundled_fonts();
    let (mut with_prior, mut plain, mut n) = (0, 0, 0);
    for i in 0..fonts.len() {
        for (k, make) in [clean_spec, degraded_spec].into_iter().enumerate() {
            let title = TITLES[(i + k) % TITLES.len()];
            let c = cover(&fonts, i, &make(&[title], 36.0 + 9.0 * k as f32, (i + 11 * k) as u64));
            n += 1;
            let regions = find_title_regions(&c.image, title);
            assert!(regions.len() <= 12);
            assert!(regions.windows(2).all(|p| p[0].score >= p[1].score));
            assert!(
                regions.iter().all(|r| r.w > 0 && r.h > 0 && r.x + r.w <= 420 && r.y + r.h <= 630)
            );
            with_prior +=
                usize::from(regions.first().is_some_and(|r| is_detected(r, &c.title_region)));
            let regions = find_text_regions(&c.image);
            plain += usize::from(regions.first().is_some_and(|r| is_detected(r, &c.title_region)));
        }
    }
    // Measured: 16/16 and 16/16.
    assert!(with_prior * 16 >= n * 14, "title found {with_prior}/{n} with the prior");
    assert!(plain * 16 >= n * 13, "title found {plain}/{n} without the prior");
}

#[test]
fn the_prior_picks_the_title_over_larger_text() {
    let fonts = bundled_fonts();
    // The "title" of the cover is a big word; the text we look for is the small line
    // that the synthesiser calls the author.
    let c = cover(&fonts, 0, &clean_spec(&["Mémoires"], 60.0, 9));
    let plain = find_text_regions(&c.image);
    assert!(is_detected(&plain[0], &c.title_region), "biggest text first without a prior");
    let found = find_title_regions(&c.image, "Camille Vasseur");
    assert!(found[0].y > 450, "the author line is near the bottom: {:?}", found[0]);
    assert!(!is_detected(&found[0], &c.title_region));
}

#[test]
fn rerank_confirms_the_face_and_fits_letter_spacing() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    let candidates: Vec<(IndexEntry, Font)> =
        index.entries().iter().cloned().zip(fonts.iter().map(|(_, f)| f.clone())).collect();
    let mut right = 0;
    for (i, (meta, _)) in fonts.iter().enumerate() {
        let title = TITLES[i % TITLES.len()];
        let mut spec = degraded_spec(&[title], 40.0, i as u64);
        spec.letter_spacing_em = if i % 2 == 0 { 0.12 } else { 0.0 };
        let c = cover(&fonts, i, &spec);
        let rr = rerank_detailed(&c.image, &c.title_region, title, &candidates);
        assert_eq!(rr.matches.len(), 8);
        assert!(rr.matches.windows(2).all(|p| p[0].score >= p[1].score));
        assert!(rr.matches.iter().all(|m| (0.0..=1.0).contains(&m.score)));
        right += usize::from(is(&rr.matches[0].entry, meta));
        // The true face must clearly beat every other family after rendering.
        if is(&rr.matches[0].entry, meta) {
            assert!(rr.family_margin > 0.05, "{meta:?}: margin {}", rr.family_margin);
        }
    }
    // Measured: 8/8.
    assert!(right >= 7, "rerank top-1 {right}/8");

    // A subset of candidates: only those are returned.
    let c = cover(&fonts, 4, &clean_spec(&["Germinal"], 40.0, 1));
    let two = rerank(&c.image, &c.title_region, "Germinal", &candidates[3..5]);
    assert_eq!(two.len(), 2);
    assert!(is(&two[0].entry, &fonts[4].0));
    // A text that cannot be on the cover: no labels, descriptor-only scores, no panic.
    let odd = rerank_detailed(&c.image, &c.title_region, "   ", &candidates);
    assert_eq!(odd.labelled_glyphs, 0);
    assert_eq!(odd.matches.len(), 8);
    // An entry described outside any index works as a candidate.
    let alone = vec![(IndexEntry::from_font(fonts[4].0.clone(), &fonts[4].1), fonts[4].1.clone())];
    let single = rerank(&c.image, &c.title_region, "Germinal", &alone);
    assert_eq!(single.len(), 1);
    assert!(single[0].score > 0.8, "{}", single[0].score);
}

#[test]
fn results_are_deterministic_and_the_embedded_index_agrees_with_a_fresh_one() {
    let fonts = bundled_fonts();
    let fresh = fresh_index(&fonts);
    let embedded = bundled_index().expect("embedded index parses");
    let c = cover(&fonts, 5, &degraded_spec(&["Bonjour tristesse"], 42.0, 3));
    let a = identify_detailed(&c.image, &c.title_region, Some("Bonjour tristesse"), &fresh);
    let b = identify_detailed(&c.image, &c.title_region, Some("Bonjour tristesse"), &fresh);
    assert_eq!(a, b);
    let e = identify(&c.image, &c.title_region, Some("Bonjour tristesse"), &embedded);
    assert!(is(&e[0].entry, &fonts[5].0));
    assert_eq!(e[0].entry.family, a.matches[0].entry.family);
    assert!(
        (e[0].score - a.matches[0].score).abs() < 0.05,
        "{} vs {}",
        e[0].score,
        a.matches[0].score
    );
    // An empty index has nothing to say.
    assert!(identify(&c.image, &c.title_region, Some("x"), &FontIndex::default()).is_empty());
}

#[test]
fn light_text_and_small_rotations_are_reported() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    let mut spec = clean_spec(&["Wuthering Heights"], 36.0, 1);
    assert!(spec.light_on_dark);
    spec.degradation = Degradation { rotation_degrees: 2.0, ..Degradation::default() };
    spec.background = Background::Gradient;
    let c = cover(&fonts, 1, &spec);
    let id = identify_detailed(&c.image, &c.title_region, Some("Wuthering Heights"), &index);
    assert!(id.light_on_dark);
    assert!((id.skew_degrees - 2.0).abs() < 0.7, "skew {}", id.skew_degrees);
    assert!(is(&id.matches[0].entry, &fonts[1].0), "{:?}", id.matches[0]);
}

#[test]
fn the_evaluation_harness_meets_its_floors_on_the_bundled_fonts() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts);
    let opts = EvalOptions { cases: 16, width: 400, height: 600, ..EvalOptions::default() };
    let rows = evaluate(&fonts, &index, &[Condition::clean(), Condition::typical()], &opts);
    let (clean, typical) = (&rows[0], &rows[1]);
    assert_eq!((clean.cases, typical.cases), (16, 16));
    // Measured with these options: clean 16/16 everywhere; typical 16/16 family.
    assert!(clean.id_family >= 15, "clean family top-1 {}/16", clean.id_family);
    assert!(clean.id_top1 >= 14, "clean face top-1 {}/16", clean.id_top1);
    assert!(clean.rr_family >= 15, "clean family top-1 after rerank {}/16", clean.rr_family);
    assert!(clean.detected >= 14, "clean detection {}/16", clean.detected);
    assert!(clean.auto_family >= 14, "clean end-to-end family {}/16", clean.auto_family);
    assert!(typical.id_family >= 12, "typical family top-1 {}/16", typical.id_family);
    assert!(typical.id_top3 >= 12, "typical face top-3 {}/16", typical.id_top3);
    assert!(typical.detected >= 12, "typical detection {}/16", typical.detected);
    assert_eq!(clean.confident, clean.confident_right, "confident implies the right family");
    assert_eq!(typical.confident, typical.confident_right);
    // The same seed gives the same counts.
    let again = evaluate(&fonts, &index, &[Condition::clean()], &opts);
    assert_eq!(&again[0], clean);

    // Hold-out: without the true face, a sibling of the family is usually nearest.
    let holdout = evaluate_holdout(&fonts, &index, &Condition::clean(), &opts);
    assert_eq!(holdout.cases, 12, "Atkinson and Spectral faces have siblings");
    assert!(holdout.sibling_top3 >= holdout.sibling_top1);
    assert!(
        holdout.sibling_top1 as f32 >= 1.5 * holdout.chance * holdout.cases as f32,
        "sibling top-1 {}/{} vs chance {:.2}",
        holdout.sibling_top1,
        holdout.cases,
        holdout.chance
    );
}
