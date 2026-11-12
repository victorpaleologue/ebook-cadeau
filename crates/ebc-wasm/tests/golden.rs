//! The golden cards, natively: three fixed requests (`tests/fixtures/golden-*.json`)
//! must render, deterministically, to well-formed print files.
//!
//! The same fixtures are rendered by the WebAssembly build in
//! `web/tests/smoke.test.mjs`, which compares their SHA-256 with the digests the
//! `golden-digests` example printed when the package was built
//! (`scripts/build-wasm.sh`): that is how "the browser writes the very bytes the native
//! library writes" is checked, without committing digests that every change to a
//! sibling crate would invalidate.

#[path = "common/golden.rs"]
mod golden;

use golden::{CASES, digests, render, sha256_hex};

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack.windows(needle.len()).filter(|window| *window == needle).count()
}

#[test]
fn golden_cards_render_deterministically() {
    let first = digests();
    assert_eq!(first, digests(), "the same request must give the same bytes");
    assert_eq!(first.lines().count(), CASES.len());
    for line in first.lines() {
        let (digest, name) = line.split_once("  ").expect("sha256sum format");
        assert_eq!(digest.len(), 64, "{name}");
        assert!(CASES.iter().any(|(case, _, _)| *case == name));
    }
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn golden_cards_are_what_they_claim() {
    // Typographic, home A4: a plain two-page PDF without any image or output intent.
    let typographic = render("typographic", false, false);
    assert!(typographic.pdf.starts_with(b"%PDF-1."));
    assert!(typographic.pdf.ends_with(b"%%EOF\n") || typographic.pdf.ends_with(b"%%EOF"));
    assert_eq!(count(&typographic.pdf, b"/Subtype /Image"), 0);
    assert_eq!(count(&typographic.pdf, b"/OutputIntent"), 0);
    assert_eq!(typographic.warnings_json, "[]");

    // A declared cover: one image, and a credit line on file.
    let cover = render("cover", true, false);
    assert!(count(&cover.pdf, b"/Subtype /Image") >= 1);
    assert!(!cover.warnings_json.contains("cover_without_rights"));

    // PDF/X-4 for an online printer: an output intent with the profile handed over.
    let pdfx = render("pdfx", false, true);
    assert!(count(&pdfx.pdf, b"/OutputIntent") >= 1);
    assert!(count(&pdfx.pdf, b"GTS_PDFX") >= 1);
    assert!(!pdfx.warnings_json.contains("fallback_output_profile"));
}
