//! (e) A `reseller_redirect` gift bought at "Grande Enseigne" (the ecosystem store): the
//! recipient opens the claim URL on a phone, is sent to the store with a signed link,
//! creates an account there, the store confirms the fulfilment to the server, and an
//! e-reader linked to that account receives the book at its next sync.

use std::time::Duration;

use ebc_demo_ereaders::{DeviceProfile, EinkBrowser, ImportSource, SyncDevice};
use ebc_demo_integration::{World, shop};

use crate::support::{
    assert_gift_headers, assert_lint_clean, browser, buy_candide, gift_id, plain_epub,
    printed_code, sha256_hex,
};

/// Presses "add to my library" on the gift page and lands on the store's sign-in page.
async fn follow_the_gift_to_the_store(phone: &mut EinkBrowser, claim_url: &str, store_base: &str) {
    assert!(phone.open(claim_url).await.unwrap().is_page());
    let page = phone.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert!(page.has("redeem"), "the primary action of a reseller gift");
    assert!(phone.click("redeem").await.unwrap().is_page());
    let page = phone.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert_eq!(page.url().as_str(), format!("{store_base}/cadeau/activer"));
    assert!(page.has("register-form") && page.has("activation"), "{}", page.text());
}

#[tokio::test]
async fn a_gift_is_activated_in_one_store_account_and_syncs_to_its_e_reader() {
    let world = World::start().await.unwrap();
    let store = &world.grande_enseigne;
    let (mut buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let claim_url = receipt.gifts[0].claim_url.clone().unwrap();
    assert_eq!(claim_url, format!("{}/k/{}", world.server.base, code.compact()));
    let gift_id = gift_id(store, 0);
    // The book never leaves the store's ecosystem: no file link was ever handed out.
    assert_eq!(store.snapshot()["gifts"][0]["file_links"], 0);
    let api = world.partner_api(store.kind);

    // --- on an e-reader the page says to use a phone; nothing is downloadable there ---
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&claim_url).await.unwrap();
    assert_lint_clean(&kobo, "the reseller gift page on a Kobo");
    assert_gift_headers(&kobo, "the reseller gift page on a Kobo");
    let page = kobo.page().unwrap();
    assert!(page.has("redeem") && !page.has("download") && !page.has("pair-start"));
    assert!(page.text_of("primary-action").unwrap().contains("Grande Enseigne"));
    // Old e-reader browsers ignore `Referrer-Policy`: when such a device follows the gift
    // to the store, the claim URL travels in `Referer`. It must reach nobody but the
    // store that issued the gift (which knows the code already) — the page links nowhere
    // else.
    assert!(kobo.profile().quirks.ignores_referrer_policy);
    assert!(kobo.click("redeem").await.unwrap().is_page());
    assert_eq!(kobo.page().unwrap().url().as_str(), format!("{}/cadeau/activer", store.base));
    let leaks = kobo.cross_origin_referers();
    assert!(!leaks.is_empty(), "the simulated Kobo sends Referer whatever the page says");
    assert!(
        leaks.iter().all(|x| x.url.as_str().starts_with(&store.base)),
        "the claim URL leaked beyond the issuing store: {leaks:#?}"
    );

    // --- two phones follow the gift to the store before anybody has an account ---
    let mut sacha = browser(&world, DeviceProfile::phone());
    follow_the_gift_to_the_store(&mut sacha, &claim_url, &store.base).await;
    assert_lint_clean(&sacha, "the store's sign-in page");
    let mut intruder = browser(&world, DeviceProfile::phone());
    follow_the_gift_to_the_store(&mut intruder, &claim_url, &store.base).await;

    // The redirect carried the signed parameters of docs/api.md §4.3, computed with the
    // secret the store shares with the server; the store then dropped them from the URL.
    let landing = sacha
        .exchanges()
        .iter()
        .find(|x| x.url.path() == "/cadeau/activer" && x.url.query().is_some())
        .expect("the signed landing on the store")
        .url
        .clone();
    let parameter = |name: &str| {
        landing.query_pairs().find(|(key, _)| key == name).map(|(_, value)| value.into_owned())
    };
    assert_eq!(parameter("ebc_gift").as_deref(), Some(gift_id.as_str()));
    assert_eq!(parameter("ebc_order").as_deref(), Some(receipt.order_ref.as_str()));
    let ts: i64 = parameter("ebc_ts").unwrap().parse().unwrap();
    assert_eq!(ts, world.clock.now());
    assert_eq!(
        parameter("ebc_sig").unwrap(),
        ebc_server::codes::redirect_signature(
            &store.partner_secret,
            &gift_id,
            &receipt.order_ref,
            ts
        )
    );
    assert!(sacha.cross_origin_referers().is_empty(), "the gift URL leaked to the store");

    // A link whose signature covers another gift, or one that is too old, is refused.
    let mut forger = browser(&world, DeviceProfile::phone());
    forger.open(&landing.as_str().replace(&gift_id, "g_0000000000000000")).await.unwrap();
    assert_eq!(forger.page().unwrap().status(), 403);
    let stale = ebc_server::codes::redirect_signature(
        &store.partner_secret,
        &gift_id,
        &receipt.order_ref,
        ts - 901,
    );
    forger
        .open(&format!(
            "{}/cadeau/activer?ebc_gift={gift_id}&ebc_order={}&ebc_ts={}&ebc_sig={stale}",
            store.base,
            receipt.order_ref,
            ts - 901
        ))
        .await
        .unwrap();
    assert_eq!(forger.page().unwrap().status(), 410);

    // --- Sacha creates an account and accepts the gift ---
    shop::register_here(&mut sacha, "sacha@example.org", "liseuse-2026").await.unwrap();
    let page = sacha.page().unwrap();
    assert_eq!(page.text_of("activation-book"), Some("Candide"));
    assert_eq!(page.text_of("activation-giver"), Some("Camille"));
    assert_eq!(page.text_of("activation-account"), Some("sacha@example.org"));
    assert!(sacha.click("activate").await.unwrap().is_page());
    let page = sacha.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert_eq!(page.url().path(), "/cadeau/merci");
    assert!(page.has("activation-done") && page.has("activation-sync"));

    // The store told the server (§4.4).
    let record = store.snapshot()["gifts"][0].clone();
    assert_eq!(record["status"], "fulfilled");
    assert_eq!(record["fulfilment_confirmed"], true);
    assert_eq!(record["activated_by"], "sacha@example.org");
    let status = api.gift_status(&gift_id).await.unwrap().json();
    assert_eq!(status["status"], "fulfilled", "{status}");
    assert!(status["redeemed_at"].is_i64());

    // --- the e-reader linked to that account syncs and has the book ---
    let mut reader = SyncDevice::new(DeviceProfile::kobo_clara(), &store.base).unwrap();
    reader.link("sacha@example.org", "liseuse-2026").await.unwrap();
    let report = reader.sync().await.unwrap();
    assert_eq!(report.added.len(), 1, "{report:#?}");
    assert!(report.skipped.is_empty(), "{report:#?}");
    let entry = &report.added[0];
    assert_eq!(
        (entry.title.as_str(), entry.authors.as_slice()),
        ("Candide", &["Voltaire".to_owned()][..])
    );
    assert_eq!(entry.sha256, sha256_hex(&plain_epub("candide")));
    assert!(matches!(entry.source, ImportSource::Sync { .. }));
    assert!(reader.library().contains_title("Candide"));
    // A second sync brings nothing new.
    let again = reader.sync().await.unwrap();
    assert_eq!((again.added.len(), again.already_present), (0, 1));

    // --- a second activation, with another account, is refused by the store ---
    intruder.type_into("register-email", "mallory@example.org").unwrap();
    intruder.type_into("register-password", "motdepasse").unwrap();
    assert!(intruder.click("register-submit").await.unwrap().is_page());
    let page = intruder.page().unwrap();
    assert_eq!(page.status(), 409, "{}", page.text());
    assert_eq!(
        page.element("activation-error").and_then(|e| e.attr("data-code")),
        Some("already_activated")
    );
    assert!(!page.has("activate"));
    let mut other_reader = SyncDevice::new(DeviceProfile::kobo_clara(), &store.base).unwrap();
    other_reader.link("mallory@example.org", "motdepasse").await.unwrap();
    assert!(other_reader.sync().await.unwrap().added.is_empty());
    assert!(other_reader.library().is_empty());
    // …and by the server, should a store ever confirm a second account.
    let second = api
        .post_json(
            &format!("/api/partner/v1/gifts/{gift_id}/fulfilled"),
            &serde_json::json!({ "account_ref": "acct_someone_else" }),
            None,
        )
        .await
        .unwrap();
    assert_eq!((second.status, second.error_code().as_deref()), (409, Some("already_fulfilled")));

    // --- the server's gift page now only says the book is in an account ---
    let mut late = browser(&world, DeviceProfile::phone());
    late.open(&claim_url).await.unwrap();
    let page = late.page().unwrap();
    assert_eq!(page.status(), 200);
    assert_lint_clean(&late, "the fulfilled gift page");
    assert!(!page.has("redeem") && !page.has("download") && !page.has("pair-start"));
    assert!(page.text_of("primary-action").unwrap().contains("Grande Enseigne"));
    assert_eq!(
        page.element("store-link").and_then(|e| e.attr("href")),
        Some(format!("{}/", store.base).as_str())
    );
    // Pressing a stale "redeem" button from an old tab changes nothing.
    let stale_press = crate::support::http_post_form(
        &world.server.url(&format!("/k/{}/redeem", code.compact())),
        DeviceProfile::phone().user_agent,
        &[],
    )
    .await;
    assert_ne!(stale_press.status, 303, "a fulfilled gift must not redirect again");

    // --- and the buyer sees it too ---
    assert!(world.webhooks_delivered(Duration::from_secs(5)).await);
    let after = shop::reload_order(&mut buyer, &receipt).await.unwrap();
    assert_eq!(after.gifts[0].status, "fulfilled");
    assert!(
        !buyer.page().unwrap().has("cancel-gift") && !buyer.page().unwrap().has("reissue-gift")
    );
}
