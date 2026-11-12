//! The printed card, end to end: the store's order page offers the real card PDF (composed
//! by `ebc-card` inside the server); the test looks at it the way its recipient would —
//! the QR code of the inside page is rendered with `pdftoppm` and decoded with `zbarimg`,
//! the printed code and address are read with `pdftotext` — and redeems the gift with
//! what is printed, on a simulated e-reader.
//!
//! The external tools are optional: where one is missing the corresponding check is
//! skipped with a note on stderr.

use std::path::{Path, PathBuf};
use std::process::Command;

use ebc_demo_ereaders::{DeviceProfile, FileFormat};
use ebc_demo_integration::{T0, World, shop};
use ebc_model::GiftCode;

use crate::support::{
    assert_lint_clean, browser, buy_candide, gift_id, printed_code, sha256_hex, stamped_epub,
};

/// Whether an external tool can be run (`tool -v` or `tool --version` starts).
fn has_tool(tool: &str) -> bool {
    let found =
        ["-v", "--version"].iter().any(|flag| Command::new(tool).arg(flag).output().is_ok());
    if !found {
        eprintln!("NOTE: `{tool}` is not installed — the check that needs it is SKIPPED");
    }
    found
}

/// A scratch directory of this test under `target/tmp`.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("card-journey").join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The text of one page of a PDF (`pdftotext`), or `None` without the tool.
fn page_text(pdf: &Path, page: u32) -> Option<String> {
    if !has_tool("pdftotext") {
        return None;
    }
    let page = page.to_string();
    let output = Command::new("pdftotext")
        .args(["-f", &page, "-l", &page, "-layout"])
        .arg(pdf)
        .arg("-")
        .output()
        .unwrap();
    assert!(output.status.success(), "pdftotext: {}", String::from_utf8_lossy(&output.stderr));
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The payloads of the QR codes on one page of a PDF (`pdftoppm` at 300 dpi, then
/// `zbarimg`), or `None` without the tools.
fn qr_payloads(pdf: &Path, page: u32, dir: &Path) -> Option<Vec<String>> {
    if !has_tool("pdftoppm") || !has_tool("zbarimg") {
        return None;
    }
    let page_arg = page.to_string();
    let prefix = dir.join(format!("page-{page}"));
    let rendered = Command::new("pdftoppm")
        .args(["-r", "300", "-f", &page_arg, "-l", &page_arg, "-png", "-singlefile"])
        .arg(pdf)
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(rendered.status.success(), "pdftoppm: {}", String::from_utf8_lossy(&rendered.stderr));
    let image = prefix.with_extension("png");
    assert!(image.exists(), "pdftoppm wrote no {}", image.display());
    let decoded = Command::new("zbarimg").args(["--raw", "-q"]).arg(&image).output().unwrap();
    // zbarimg exits with 4 when it finds no symbol.
    let text = String::from_utf8_lossy(&decoded.stdout);
    Some(text.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_owned).collect())
}

/// Every `XXXX-XXXX-XXXX` gift code found in a text (check letter verified).
fn codes_in(text: &str) -> Vec<GiftCode> {
    let mut codes: Vec<GiftCode> = text
        .split(|c: char| !(c.is_ascii_alphabetic() || c == '-'))
        .filter(|word| word.len() == 14 && word.matches('-').count() == 2)
        .filter_map(|word| GiftCode::parse(word).ok())
        .collect();
    codes.dedup();
    codes
}

#[tokio::test]
async fn the_printed_card_carries_a_qr_code_and_a_code_that_redeem_the_gift() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let (mut buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let dir = scratch("septante");

    // --- the order page offers the card; the buyer downloads it (for home printing) ---
    let gift = &receipt.gifts[0];
    assert!(!gift.card_unavailable, "{gift:#?}");
    let card_url = gift.card_pdf_url.clone().expect("a card PDF link on the order page");
    let saved = buyer.open(&card_url).await.unwrap();
    let entry = saved.download().expect("the browser saves the card PDF").clone();
    assert_eq!(entry.format, FileFormat::Pdf);
    // The simulated browser keeps no bytes: fetch the same (deterministic) rendering.
    let api = world.partner_api(store.kind);
    let gift_id = gift_id(store, 0);
    let home = api
        .get(&format!("/api/partner/v1/gifts/{gift_id}/card.pdf?format=poche&profile=home-a4"))
        .await
        .unwrap();
    assert_eq!(home.status, 200, "{}", home.text());
    assert_eq!(entry.sha256, sha256_hex(&home.body), "the buyer's file is the server's card");
    let pro = api
        .get(&format!("/api/partner/v1/gifts/{gift_id}/card.pdf?format=poche&profile=pro"))
        .await
        .unwrap();
    assert_eq!(pro.status, 200, "{}", pro.text());

    let expected_qr = ebc_model::claim_url_qr(&world.server.base, &code);
    assert_eq!(expected_qr, gift.claim_url.as_deref().unwrap().to_uppercase());
    let mut typed_from_card: Option<(String, GiftCode)> = None;
    for (name, pdf) in [("home-a4", &home.body), ("pro", &pro.body)] {
        assert!(pdf.starts_with(b"%PDF-"), "{name}");
        let path = dir.join(format!("card-{name}.pdf"));
        std::fs::write(&path, pdf).unwrap();

        // --- the QR code of the inside page (page 2) is the claim URL, upper-cased ---
        if let Some(payloads) = qr_payloads(&path, 2, &dir) {
            assert_eq!(
                payloads.as_slice(),
                std::slice::from_ref(&expected_qr),
                "{name}: QR code(s) on the inside page"
            );
            // Nothing scannable on the outside: the code must not be readable from a
            // closed card lying around.
            let outside = qr_payloads(&path, 1, &dir).unwrap_or_default();
            assert!(outside.is_empty(), "{name}: a QR code on the outside: {outside:?}");
        }

        // --- the text a person reads: the code, the address, the names — inside only ---
        if let Some(inside) = page_text(&path, 2) {
            assert_eq!(codes_in(&inside), [code], "{name}: the printed code\n{inside}");
            let host = world.server.host();
            assert!(inside.contains(host), "{name}: the address {host} is printed\n{inside}");
            for expected in ["Sacha", "Camille", "Joyeux anniversaire"] {
                assert!(inside.contains(expected), "{name}: no {expected:?} inside\n{inside}");
            }
            let outside = page_text(&path, 1).unwrap_or_default();
            assert!(codes_in(&outside).is_empty(), "{name}: the code is printed outside");
            // (The title is on the outside as the cover image and as rotated spine text,
            // neither of which is plain page text; the imprint line is.)
            assert!(outside.contains("eBook Cadeau"), "{name}: the outside text\n{outside}");
            typed_from_card.get_or_insert((host.to_owned(), codes_in(&inside)[0]));
        }
    }

    // --- redeem with nothing but what is printed: the address and the code ---
    let (host, typed_code) = typed_from_card.unwrap_or_else(|| {
        eprintln!("NOTE: redeeming with the code from the order page (pdftotext missing)");
        (world.server.host().to_owned(), code)
    });
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    assert!(kobo.open(&format!("http://{host}/")).await.unwrap().is_page());
    kobo.type_into("code-input", &typed_code.grouped()).unwrap();
    assert!(kobo.click("code-submit").await.unwrap().is_page());
    assert_eq!(kobo.page().unwrap().status(), 200);
    assert_lint_clean(&kobo, "the gift page reached from the printed card");
    let book = kobo.click("download").await.unwrap().download().expect("the EPUB").clone();
    let expected = stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0);
    assert_eq!((book.title.as_str(), book.sha256), ("Candide", sha256_hex(&expected)));

    // --- and by scanning: the QR payload itself opens the gift page on a phone ---
    let mut phone = browser(&world, DeviceProfile::phone());
    assert!(phone.open(&expected_qr).await.unwrap().is_page());
    assert_eq!(phone.page().unwrap().text_of("gift-title"), Some("Candide"));

    // --- a lost card: the reissued code is on the new card, the old card is dead ---
    shop::reload_order(&mut buyer, &receipt).await.unwrap();
    assert!(buyer.click("reissue-gift").await.unwrap().is_page());
    let after = shop::read_order_page(buyer.page().unwrap()).unwrap();
    let new_code = printed_code(&after);
    assert!(after.gifts[0].card_pdf_url.is_some(), "the new card is offered");
    let reprinted = api
        .get(&format!("/api/partner/v1/gifts/{gift_id}/card.pdf?format=poche&profile=home-a4"))
        .await
        .unwrap();
    assert_ne!(reprinted.body, home.body, "the card was rendered again");
    let path = dir.join("card-reissued.pdf");
    std::fs::write(&path, &reprinted.body).unwrap();
    if let Some(inside) = page_text(&path, 2) {
        assert_eq!(codes_in(&inside), [new_code], "the new card prints the new code");
    }
    if let Some(payloads) = qr_payloads(&path, 2, &dir) {
        assert_eq!(payloads, [ebc_model::claim_url_qr(&world.server.base, &new_code)]);
    }
}

/// The card of an ecosystem store names the store in its steps, and every documented
/// format and profile is served to the partner.
#[tokio::test]
async fn the_card_of_a_reseller_gift_names_the_store_and_every_format_is_served() {
    let world = World::start().await.unwrap();
    let store = &world.grande_enseigne;
    let (_buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    assert!(receipt.gifts[0].card_pdf_url.is_some(), "{receipt:#?}");
    let api = world.partner_api(store.kind);
    let gift_id = gift_id(store, 0);
    let dir = scratch("grande-enseigne");
    let card = |query: &str| format!("/api/partner/v1/gifts/{gift_id}/card.pdf{query}");

    let a6 = api.get(&card("?format=a6&profile=home-letter")).await.unwrap();
    assert_eq!(a6.status, 200, "{}", a6.text());
    let path = dir.join("card-a6-letter.pdf");
    std::fs::write(&path, &a6.body).unwrap();
    if let Some(inside) = page_text(&path, 2) {
        assert!(inside.contains("Grande Enseigne"), "the steps name the store\n{inside}");
        assert_eq!(codes_in(&inside), [code]);
    }
    if let Some(payloads) = qr_payloads(&path, 2, &dir) {
        assert_eq!(payloads, [ebc_model::claim_url_qr(&world.server.base, &code)]);
    }

    // Parameters outside the contract are refused; a card too large for its sheet is 422.
    for (query, status, error) in [
        ("?format=a4", 400, "invalid_parameter"),
        ("?profile=home", 400, "invalid_parameter"),
        ("?format=a5&profile=home-a4", 422, "card_unprocessable"),
    ] {
        let reply = api.get(&card(query)).await.unwrap();
        assert_eq!((reply.status, reply.error_code().as_deref()), (status, Some(error)), "{query}");
    }
    // Another partner cannot render this gift's card (it would reveal the code).
    let other = world.partner_api(world.septante.kind);
    assert_eq!(other.get(&card("")).await.unwrap().status, 404);
}
