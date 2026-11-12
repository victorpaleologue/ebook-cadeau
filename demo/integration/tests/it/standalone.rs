//! The standalone ("creator") mode of `docs/api.md` §3, without any store: somebody
//! uploads their own DRM-free file, gets a code and a manage token, and the recipient
//! redeems it on an e-reader. Covers the optional reveal date, the giver's status view,
//! the reissue of a lost card and the cancellation.

use ebc_demo_ereaders::fixtures::{self, EpubBuilder};
use ebc_demo_ereaders::{DeviceProfile, FileFormat};
use ebc_demo_integration::{T0, World};
use ebc_model::GiftCode;
use serde_json::json;

use crate::support::{assert_lint_clean, browser, creator_call, http_post_form, sha256_hex};

#[tokio::test]
async fn a_standalone_gift_stays_wrapped_until_its_reveal_date_and_its_giver_manages_it() {
    let world = World::start().await.unwrap();
    let epub = fixtures::epub("Le Horla", "Guy de Maupassant");
    let upload_headers = [("x-rights-basis", "public_domain"), ("x-file-name", "le-horla.epub")];

    // --- uploads: the rights declaration is mandatory, DRM is refused ---
    let undeclared = creator_call(&world, "POST", "/api/v1/uploads", None, &[], epub.clone()).await;
    assert!((400..500).contains(&undeclared.status), "{}", undeclared.text());
    for protected in [
        EpubBuilder::new("T", "A").with_adobe_rights().build(),
        EpubBuilder::new("T", "A").with_lcp_marker().build(),
    ] {
        let refused =
            creator_call(&world, "POST", "/api/v1/uploads", None, &upload_headers, protected).await;
        assert_eq!(refused.error_code().as_deref(), Some("drm_protected"), "{}", refused.text());
    }
    let not_a_book = creator_call(
        &world,
        "POST",
        "/api/v1/uploads",
        None,
        &upload_headers,
        b"<html><script>alert(1)</script></html>".to_vec(),
    )
    .await;
    assert_eq!(not_a_book.status, 415, "{}", not_a_book.text());
    let upload =
        creator_call(&world, "POST", "/api/v1/uploads", None, &upload_headers, epub.clone()).await;
    assert_eq!(upload.status, 201, "{}", upload.text());
    assert_eq!(upload.json()["sha256"], sha256_hex(&epub).as_str());
    assert_eq!(upload.json()["media_type"], "application/epub+zip");

    // --- the gift, to be revealed tomorrow, in English ---
    let reveal_at = T0 + 86_400;
    let created = creator_call(
        &world,
        "POST",
        "/api/v1/gifts",
        None,
        &[("content-type", "application/json")],
        json!({
            "book": { "title": "Le Horla", "contributors": [{"name": "Guy de Maupassant"}], "language": "fr" },
            "giver_name": "Camille", "recipient_name": "Sacha", "message": "Open it on your birthday!",
            "locale": "en", "reveal_at": reveal_at,
            "delivery": { "type": "hosted_file", "upload_id": upload.json()["upload_id"] },
        })
        .to_string()
        .into_bytes(),
    )
    .await;
    assert_eq!(created.status, 201, "{}", created.text());
    let gift = created.json();
    let id = gift["id"].as_str().unwrap().to_owned();
    let token = gift["manage_token"].as_str().unwrap().to_owned();
    let code = GiftCode::parse(gift["code"].as_str().unwrap()).unwrap();
    assert_eq!(gift["code_display"], code.grouped().as_str());
    let claim_url = gift["claim_url"].as_str().unwrap().to_owned();
    assert_eq!(claim_url, ebc_model::claim_url(&world.server.base, &code));

    // --- before the date: the page shows who it is from and when, and no action ---
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&claim_url).await.unwrap();
    let page = kobo.page().unwrap();
    assert_eq!(page.status(), 200);
    assert_eq!(page.lang(), Some("en"), "pages are served in the gift's locale");
    assert!(page.has("reveal-date") && page.text_of("gift-from").unwrap().contains("Camille"));
    assert!(!page.has("download") && !page.has("gift-title"), "the title stays a surprise");
    assert_lint_clean(&kobo, "the wrapped gift page");
    let ua = DeviceProfile::kobo_clara().user_agent;
    for action in ["download", "pair"] {
        let early = http_post_form(&format!("{claim_url}/{action}"), ua, &[]).await;
        assert_ne!(early.status, 303, "{action} before the reveal date");
        assert!(!early.text().contains("id=\"pair-code\""), "{action} before the reveal date");
    }

    // --- the day comes ---
    world.advance(86_400 + 1);
    kobo.open(&claim_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().text_of("gift-title"), Some("Le Horla"));
    let entry = kobo.click("download").await.unwrap().download().unwrap().clone();
    assert_eq!((entry.title.as_str(), entry.format), ("Le Horla", FileFormat::Epub));
    assert_eq!(entry.sha256, sha256_hex(&epub), "the uploaded file, byte for byte");

    // --- the giver follows the gift with the manage token (and nobody else can) ---
    let path = format!("/api/v1/gifts/{id}");
    let status = creator_call(&world, "GET", &path, Some(&token), &[], Vec::new()).await;
    assert_eq!(status.status, 200, "{}", status.text());
    assert_eq!(
        (status.json()["status"].as_str(), status.json()["downloads"].as_u64()),
        (Some("redeemed"), Some(1))
    );
    for wrong in [None, Some("0".repeat(64))] {
        let reply = creator_call(&world, "GET", &path, wrong.as_deref(), &[], Vec::new()).await;
        assert!(matches!(reply.status, 401 | 403 | 404), "{}", reply.status);
    }

    // --- a lost card: the giver gets a new code, the old one answers 410 ---
    let reissued =
        creator_call(&world, "POST", &format!("{path}/reissue"), Some(&token), &[], Vec::new())
            .await;
    assert_eq!(reissued.status, 200, "{}", reissued.text());
    let new_code = GiftCode::parse(reissued.json()["code"].as_str().unwrap()).unwrap();
    assert_ne!(new_code, code);
    kobo.open(&claim_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 410);
    let new_url = reissued.json()["claim_url"].as_str().unwrap().to_owned();
    kobo.open(&new_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 200);
    assert!(kobo.page().unwrap().has("download"));

    // --- the giver cancels: the code stops working and the file is gone ---
    let cancelled = creator_call(&world, "DELETE", &path, Some(&token), &[], Vec::new()).await;
    assert!(cancelled.status == 200 || cancelled.status == 204, "{}", cancelled.text());
    kobo.open(&new_url).await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 410);
    let status = creator_call(&world, "GET", &path, Some(&token), &[], Vec::new()).await;
    assert_eq!(status.json()["status"], "cancelled");
}
