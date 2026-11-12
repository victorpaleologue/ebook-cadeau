//! Journeys of a `partner_file` gift bought at "Septante" (the DRM-free store, which
//! serves its own files): the printed code typed on a Kobo, the pair code from a phone to
//! a PocketBook, Send to Kindle, and the OPDS feeds of a download session. The core never
//! holds the book: each claim asks the store for a short-lived link, and the store stamps
//! the copy for the recipient at that moment.

use std::time::Duration;

use ebc_demo_ereaders::flow;
use ebc_demo_ereaders::mail::HoldReason;
use ebc_demo_ereaders::opds::FeedKind;
use ebc_demo_ereaders::typing::TypingCost;
use ebc_demo_ereaders::{
    BrowserError, DeviceFamily, DeviceProfile, Error, FileFormat, ImportSource, MailOutcome,
    OpdsClient,
};
use ebc_demo_integration::{T0, World, shop};
use ebc_model::PairCode;

use crate::support::{
    assert_gift_headers, assert_lint_clean, browser, buy_candide, gift_id, http_post_form,
    plain_epub, printed_code, record_taps, sha256_hex, stamped_epub, zip_text,
};

/// (a) Browse → cart → checkout with the gift option → pay → the store creates the gift
/// through the signed partner API (a `partner_file` pointing at its own file endpoint) →
/// the order page shows
/// the claim URL and the code → a Kobo **without any phone** opens the server's home
/// page, types the printed code and downloads the book into its library.
#[tokio::test]
async fn a_kobo_types_the_printed_code_and_downloads_the_watermarked_epub() {
    let world = World::start().await.unwrap();
    let store = &world.septante;

    // --- the buyer, on a computer, without JavaScript ---
    let (mut buyer, receipt) = buy_candide(&world, store).await;
    let visited: Vec<String> =
        buyer.exchanges().iter().map(|x| format!("{} {}", x.method, x.url.path())).collect();
    for step in [
        "GET /",
        "GET /livre/candide",
        "POST /panier/ajouter",
        "GET /panier",
        "GET /commande",
        "POST /compte/inscription",
        "POST /commande/payer",
    ] {
        assert!(visited.iter().any(|v| v == step), "the buyer never did {step}: {visited:#?}");
    }
    let gift = &receipt.gifts[0];
    let code = printed_code(&receipt);
    assert_eq!(
        gift.claim_url.as_deref(),
        Some(format!("{}/k/{}", world.server.base, code.compact()).as_str())
    );
    assert_eq!((gift.error.as_deref(), gift.print_status.as_deref()), (None, None));

    // --- what the store did behind the page: gift, cover — all signed; no file sent ---
    let snapshot = store.snapshot();
    let record = &snapshot["gifts"][0];
    assert_eq!(record["file_links"], 0, "nothing is asked of the store before a claim");
    assert!(record["gift_id"].as_str().is_some_and(|id| id.starts_with("g_")), "{record}");
    assert_eq!(record["cover_sent"], true);
    let gift_id = gift_id(store, 0);
    let api = world.partner_api(store.kind);
    let status = api.gift_status(&gift_id).await.unwrap().json();
    assert_eq!(status["status"], "created");
    assert_eq!(status["order_ref"], receipt.order_ref.as_str());

    // --- the card: rendered by the server, offered on the order page (its content is
    // checked in `card.rs`) ---
    let card = api
        .get(&format!("/api/partner/v1/gifts/{gift_id}/card.pdf?format=poche&profile=home-a4"))
        .await
        .unwrap();
    assert_eq!(card.status, 200, "{}", card.text());
    assert_eq!(card.header("content-type"), Some("application/pdf"));
    let card_link = format!("{}/carte-1.pdf", receipt.order_url);
    assert!(!gift.card_unavailable, "{gift:#?}");
    assert_eq!(gift.card_pdf_url.as_deref(), Some(card_link.as_str()));
    let saved = buyer.open(&card_link).await.unwrap();
    let entry = saved.download().expect("the browser saves the card PDF");
    assert_eq!(entry.format, FileFormat::Pdf);
    assert_eq!(
        entry.sha256,
        sha256_hex(&card.body),
        "the card the server rendered for home printing"
    );

    // --- the recipient: a Kobo, no phone, no account ---
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    assert!(kobo.open(&world.server.url("/")).await.unwrap().is_page());
    assert_lint_clean(&kobo, "the home page");
    // Typed the cheapest way the card allows: letters only, any case, no separators.
    let typed = code.compact().to_lowercase();
    let cost = kobo.type_into("code-input", &typed).unwrap();
    assert_eq!((cost.taps, cost.layer_switches, cost.shift_taps), (12, 0, 0), "{cost:?}");
    assert!(kobo.click("code-submit").await.unwrap().is_page());
    let page = kobo.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert_eq!(page.url().path(), format!("/k/{}", code.compact()));
    assert_lint_clean(&kobo, "the gift page");
    assert_gift_headers(&kobo, "the gift page");
    assert_eq!(page.text_of("gift-title"), Some("Candide"));
    assert!(page.text_of("gift-author").unwrap().contains("Voltaire"));
    assert!(page.text_of("gift-from").unwrap().contains("Camille"));
    assert_eq!(page.text_of("gift-message"), Some("Joyeux anniversaire !"));
    for id in [
        "primary-action",
        "other-ways",
        "device-chooser",
        "device-kobo",
        "device-kindle",
        "device-pocketbook",
        "device-tolino",
        "device-phone",
        "device-computer",
        "claim-count",
        "download-form",
    ] {
        assert!(page.has(id), "the gift page has no #{id}");
    }
    let outcome = kobo.click("download").await.unwrap();
    let entry = outcome.download().expect("the Kobo saves the EPUB to its library").clone();
    assert_eq!((entry.title.as_str(), entry.format), ("Candide", FileFormat::Epub));
    assert_eq!(entry.authors, ["Voltaire"]);
    assert_eq!(entry.drm, None);
    assert!(matches!(entry.source, ImportSource::BrowserDownload { .. }));
    assert_eq!(kobo.library().len(), 1);
    assert!(kobo.cookies().is_empty(), "redeeming needs no cookie");
    // The Kobo ignores Referrer-Policy: following the redirect to the store's download
    // link, it names the gift page (and so the code) in `Referer`. That host is the
    // partner's own (`redirect_hosts`), which issued the code; no other origin hears it.
    let leaks = kobo.cross_origin_referers();
    assert!(
        leaks.iter().all(|x| x.url.origin() == reqwest::Url::parse(&store.base).unwrap().origin()),
        "the code leaks through Referer to a third party: {leaks:#?}"
    );
    // The book itself came from the store, through the core's session URL.
    let hops: Vec<String> = kobo.exchanges().iter().map(|x| x.url.path().to_owned()).collect();
    assert!(hops.iter().any(|p| p.starts_with("/d/")), "{hops:#?}");
    assert!(hops.iter().any(|p| p.starts_with("/cadeau/telechargement/")), "{hops:#?}");

    // The file is the copy the store stamped for Sacha — not the plain edition.
    let expected = stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0);
    assert_eq!(entry.sha256, sha256_hex(&expected));
    assert_eq!(entry.size_bytes, expected.len() as u64);
    assert_ne!(entry.sha256, sha256_hex(&plain_epub("candide")));
    let watermark = zip_text(&expected, "OEBPS/watermark.xhtml");
    assert!(watermark.contains("Offert à Sacha par Camille"), "{watermark}");
    assert!(watermark.contains(&receipt.order_ref) && watermark.contains("Septante"));

    // --- keystroke cost, on record ---
    record_taps("septante-kobo-printed-code", kobo.profile().name, kobo.taps());
    // Launching the beta browser (4) + address bar and "Go" (2) + the code field (1) +
    // 12 letters + "continue" (1) + "download" (1) + the device's own confirmation (1),
    // plus the address itself. With the production host this is what a person pays:
    let address = TypingCost::of_url("https://ebk.example/", &kobo.profile().keyboard);
    let fixed = kobo.total_taps() - typed_address_taps(&kobo);
    eprintln!("taps besides the address: {fixed}; a host like ebk.example adds {}", address.taps);
    assert!(fixed <= 24, "{fixed} taps besides the address");
    assert!(fixed + address.taps <= 40, "the no-phone path costs {} taps", fixed + address.taps);

    // --- everybody sees what happened ---
    assert!(world.webhooks_delivered(Duration::from_secs(5)).await);
    let events: Vec<String> = store.snapshot()["webhooks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["gift_id"] == gift_id.as_str())
        .map(|e| e["event"].as_str().unwrap().to_owned())
        .collect();
    assert!(events.contains(&"gift.redeemed".to_owned()), "{events:?}");
    assert!(events.contains(&"gift.downloaded".to_owned()), "{events:?}");
    let after = shop::reload_order(&mut buyer, &receipt).await.unwrap();
    assert_eq!(after.gifts[0].status, "redeemed");
    assert_eq!(after.gifts[0].downloads, Some(1));
    // The holder of the card sees the claim count too (misuse is visible).
    kobo.open(&world.server.url(&format!("/k/{}", code.compact()))).await.unwrap();
    let count = kobo.page().unwrap().text_of("claim-count").unwrap().to_owned();
    assert!(count.contains('1'), "{count}");
}

/// Taps spent typing addresses (everything recorded as "type the address …").
fn typed_address_taps(browser: &ebc_demo_ereaders::EinkBrowser) -> u32 {
    browser
        .taps()
        .steps()
        .iter()
        .filter(|step| step.action.starts_with("type the address"))
        .map(|step| step.taps)
        .sum()
}

/// (b) The same gift through a phone: the phone scans the QR code and asks for a pair
/// code; a PocketBook types it at `/e` and gets the book. A pair code works once and for
/// 15 minutes.
#[tokio::test]
async fn a_phone_pairs_a_pocketbook_and_pair_codes_are_single_use_and_expire() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let (_buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let expected =
        sha256_hex(&stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0));

    // The QR code carries the claim URL fully upper-cased: `/K/` is `/k/`.
    let qr = ebc_model::claim_url_qr(&world.server.base, &code);
    assert!(qr.contains("/K/") && qr.starts_with("HTTP://"), "{qr}");
    let mut phone = browser(&world, DeviceProfile::phone());
    assert!(phone.open(&qr).await.unwrap().is_page());
    let page = phone.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert_lint_clean(&phone, "the gift page on a phone");
    assert_gift_headers(&phone, "the gift page on a phone");
    assert!(page.has("download"), "a phone can read the book itself");
    // The e-reader instructions name the short address and the code to type there.
    let other_ways = page.text_of("other-ways").unwrap();
    assert!(other_ways.contains(world.server.host()), "{other_ways}");
    assert!(other_ways.contains(&code.grouped()), "{other_ways}");

    assert!(phone.click("pair-start").await.unwrap().is_page());
    assert_lint_clean(&phone, "the pair-code page");
    let page = phone.page().unwrap();
    let shown = page.text_of("pair-code").unwrap().to_owned();
    let pair = PairCode::parse(&shown).expect("6 letters");
    assert_eq!(shown, pair.grouped(), "shown as XXX-XXX");
    assert_eq!(page.text_of("pair-url"), Some(format!("{}/e", world.server.host()).as_str()));
    assert!(page.text_of("pair-expires").unwrap().contains("15"), "the page says how long");

    // The PocketBook: open /e, type six letters, get the book.
    let mut pocketbook = browser(&world, DeviceProfile::pocketbook());
    assert!(pocketbook.open(&world.server.url("/e")).await.unwrap().is_page());
    assert_lint_clean(&pocketbook, "the pair form");
    let cost = pocketbook.type_into("pair-input", &pair.compact().to_lowercase()).unwrap();
    assert_eq!((cost.taps, cost.layer_switches), (6, 0));
    let outcome = pocketbook.click("pair-submit").await.unwrap();
    let entry = outcome.download().expect("the PocketBook saves the EPUB");
    assert_eq!((entry.title.as_str(), entry.sha256.as_str()), ("Candide", expected.as_str()));
    record_taps("septante-pocketbook-pair-code", pocketbook.profile().name, pocketbook.taps());

    // Reuse: the same pair code on another device is refused like an unknown one.
    let mut tolino = browser(&world, DeviceProfile::tolino());
    tolino.open(&world.server.url("/e")).await.unwrap();
    tolino.type_into("pair-input", &shown).unwrap();
    assert!(tolino.click("pair-submit").await.unwrap().is_page());
    let refused = tolino.page().unwrap();
    assert_eq!(refused.status(), 404, "{}", refused.text());
    assert!(refused.has("error") && refused.has("pair-form"), "the form is offered again");
    assert_lint_clean(&tolino, "the refused pair page");
    assert!(tolino.library().is_empty());

    // Expiry: a second pair code, typed a quarter of an hour too late.
    assert!(phone.click("pair-renew").await.unwrap().is_page());
    let second = phone.page().unwrap().text_of("pair-code").unwrap().to_owned();
    assert_ne!(second, shown);
    world.advance(15 * 60 + 1);
    tolino.type_into("pair-input", &second).unwrap();
    assert!(tolino.click("pair-submit").await.unwrap().is_page());
    assert_eq!(tolino.page().unwrap().status(), 404);
    assert!(tolino.library().is_empty());

    // A malformed pair code is a typo, not a guess: 400, and the form says so.
    tolino.type_into("pair-input", "12-34").unwrap();
    tolino.click("pair-submit").await.unwrap();
    assert_eq!(tolino.page().unwrap().status(), 400);

    // A fresh pair code works again — the gift itself is untouched by all this.
    assert!(phone.click("pair-renew").await.unwrap().is_page());
    let third = phone.page().unwrap().text_of("pair-code").unwrap().to_owned();
    tolino.type_into("pair-input", &third).unwrap();
    let outcome = tolino.click("pair-submit").await.unwrap();
    // The store stamps every claim when it happens: this copy carries the later time.
    let later =
        stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0 + 15 * 60 + 1);
    assert_ne!(sha256_hex(&later), expected);
    assert_eq!(outcome.download().expect("the tolino saves the EPUB").sha256, sha256_hex(&later));
}

/// (c) Kindle: its browser is not offered an EPUB download; from a phone, "Send to
/// Kindle" delivers the EPUB by SMTP to the device's address — parked while the sender is
/// not approved, imported once it is.
#[tokio::test]
async fn a_kindle_gets_the_book_by_send_to_kindle_once_the_sender_is_approved() {
    let address = "sacha_42@kindle.com";
    let world = World::builder()
        .mail_device(DeviceProfile::kindle_paperwhite(), address)
        .start()
        .await
        .unwrap();
    let store = &world.septante;
    let mail = world.mail.as_ref().unwrap();
    let sender = world.server.state.config.send_from.clone();
    let (_buyer, receipt) = buy_candide(&world, store).await;
    let claim_url = receipt.gifts[0].claim_url.clone().unwrap();
    let expected =
        sha256_hex(&stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0));

    // --- on the Kindle itself: no EPUB download, the Send-to-Kindle explanation instead ---
    let mut kindle = browser(&world, DeviceProfile::kindle_paperwhite());
    assert!(kindle.open(&claim_url).await.unwrap().is_page());
    let page = kindle.page().unwrap();
    assert_eq!(page.status(), 200);
    assert_lint_clean(&kindle, "the gift page on a Kindle");
    assert!(!page.has("download") && !page.has("download-form"), "an EPUB a Kindle cannot save");
    assert!(page.has("send-form") && page.has("device-email"));
    assert!(
        page.text_of("primary-action").unwrap().contains(&sender),
        "names the sender to approve"
    );
    // Even if the page of another device class is forced, the Kindle's browser refuses
    // the EPUB before saving a byte.
    kindle.open(&format!("{claim_url}?device=kobo")).await.unwrap();
    let refusal = kindle.click("download").await.unwrap_err();
    assert!(matches!(refusal, Error::Browser(BrowserError::DownloadRefused { .. })), "{refusal:?}");
    assert!(kindle.library().is_empty());

    // --- from the phone: the e-mail form ---
    let mut phone = browser(&world, DeviceProfile::phone());
    phone.open(&claim_url).await.unwrap();
    assert!(phone.page().unwrap().text().contains(&sender), "the phone page names the sender");
    phone.type_into("device-email", address).unwrap();
    assert!(phone.click("send").await.unwrap().is_page());
    let page = phone.page().unwrap();
    assert_eq!(page.status(), 200, "{}", page.text());
    assert!(page.has("send-result"));
    assert_lint_clean(&phone, "the send confirmation");

    // The mail left the server and reached the gateway — which parks it silently,
    // because the recipient has not approved our sender yet.
    let records = mail.records();
    assert_eq!(records.len(), 1, "{records:#?}");
    assert_eq!(records[0].from, sender);
    assert_eq!(records[0].outcome, MailOutcome::Held(HoldReason::SenderNotApproved));
    assert!(mail.library().is_empty());

    // The recipient approves the sender and confirms the parked delivery: imported.
    mail.approve_sender(&sender);
    let released = mail.release(records[0].id).unwrap();
    let delivered = released.delivered();
    assert_eq!(delivered.len(), 1, "{released:#?}");
    assert_eq!(delivered[0].title, "Candide");
    assert_eq!(delivered[0].sha256, expected, "the watermarked EPUB, attached unchanged");
    assert_eq!(delivered[0].original_format, FileFormat::Epub);
    assert_eq!(delivered[0].format, FileFormat::Kfx, "Send to Kindle converts EPUB");
    assert!(matches!(delivered[0].source, ImportSource::Mail { .. }));
    assert_eq!(mail.library().len(), 1);

    // From now on a sending arrives directly.
    phone.open(&claim_url).await.unwrap();
    phone.type_into("device-email", address).unwrap();
    phone.click("send").await.unwrap();
    assert_eq!(phone.page().unwrap().status(), 200);
    let records = mail.records();
    assert_eq!(records.len(), 2);
    assert!(matches!(records[1].outcome, MailOutcome::Processed(_)), "{:#?}", records[1]);
    assert_eq!(mail.library().len(), 1, "the same file is not duplicated");

    // Only device addresses of the configured domains are accepted; nothing is sent.
    phone.open(&claim_url).await.unwrap();
    phone.type_into("device-email", "sacha@example.org").unwrap();
    phone.click("send").await.unwrap();
    let page = phone.page().unwrap();
    assert_eq!(page.status(), 400);
    assert!(page.text_of("send-result").unwrap().contains("@kindle.com"));
    assert_eq!(mail.records().len(), 2);
}

/// (d) OPDS: a reading application fetches the OPDS 1.2 and 2.0 feeds of a download
/// session and acquires the book.
#[tokio::test]
async fn a_reading_app_acquires_the_book_through_both_opds_feeds() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let (_buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let expected =
        sha256_hex(&stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0));

    // The computer's gift page offers "open in a reading application (OPDS)".
    let mut computer = browser(&world, DeviceProfile::computer());
    computer.open(&world.server.url(&format!("/k/{}", code.compact()))).await.unwrap();
    assert!(computer.page().unwrap().has("opds"), "no OPDS button on the computer page");
    let profile = DeviceProfile::computer();
    let minted = http_post_form(
        &world.server.url(&format!("/k/{}/download", code.compact())),
        profile.user_agent,
        &[("opds", "1")],
    )
    .await;
    assert_eq!(minted.status, 303, "{}", minted.text());
    let location = minted.header("location").unwrap();
    assert!(location.starts_with("/o/") && location.ends_with("/catalog.xml"), "{location}");
    let atom_url = world.server.url(location);
    let json_url = atom_url.replace("catalog.xml", "catalog.json");

    // OPDS 1.2 (Atom).
    let mut app = OpdsClient::new(DeviceProfile::thorium_desktop()).unwrap();
    let feed = app.fetch(&atom_url).await.unwrap();
    assert_eq!(feed.kind, FeedKind::Opds1);
    assert_eq!(feed.publications.len(), 1, "{feed:#?}");
    let publication = &feed.publications[0];
    assert_eq!(
        (publication.title.as_str(), publication.authors.as_slice()),
        ("Candide", &["Voltaire".to_owned()][..])
    );
    assert!(publication.acquisition("application/epub+zip").is_some());
    let outcome = app.acquire(publication).await.unwrap();
    let entry = outcome.download().expect("the app imports the EPUB");
    assert_eq!(entry.sha256, expected);
    assert!(matches!(entry.source, ImportSource::Opds { .. }));

    // OPDS 2.0 (JSON), in another application.
    let mut other = OpdsClient::new(DeviceProfile::thorium_desktop()).unwrap();
    let feed = other.fetch(&json_url).await.unwrap();
    assert_eq!(feed.kind, FeedKind::Opds2);
    assert_eq!(feed.publications.len(), 1, "{feed:#?}");
    assert_eq!(feed.publications[0].title, "Candide");
    let outcome = other.acquire(&feed.publications[0]).await.unwrap();
    assert_eq!(outcome.download().expect("the app imports the EPUB").sha256, expected);

    // One download session was minted for all of this: one claim.
    let status = world.partner_api(store.kind).gift_status(&gift_id(store, 0)).await.unwrap();
    assert_eq!(status.json()["downloads"], 1);

    // The feed dies with its session (15 minutes).
    world.advance(15 * 60 + 1);
    let mut late = OpdsClient::new(DeviceProfile::thorium_desktop()).unwrap();
    assert!(late.fetch(&atom_url).await.is_err());
    assert!(late.fetch(&json_url).await.is_err());
}

/// Every simulated device either ends up with the book or is told why not — from the
/// same card, starting on the server's home page with the printed code.
#[tokio::test]
async fn every_simulated_device_gets_the_book_or_is_told_why() {
    let world = World::builder()
        .server_config(|config| config.max_download_sessions = 50)
        .start()
        .await
        .unwrap();
    let store = &world.septante;
    let (_buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let expected =
        sha256_hex(&stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0));
    let inputs = flow::RedeemInputs { code: Some(code.grouped()), ..flow::RedeemInputs::default() };

    let mut got_it = Vec::new();
    let mut told_why = Vec::new();
    for profile in DeviceProfile::all() {
        let id = profile.id;
        let is_kindle = profile.family == DeviceFamily::Kindle;
        // Old Android tolino firmware reports browser downloads as damaged.
        let takes_epub = profile.browser_downloads(FileFormat::Epub)
            && !profile.quirks.download_reported_damaged;
        let mut device = browser(&world, profile);
        let report = flow::redeem(&mut device, &world.server.url("/"), &inputs).await.unwrap();
        for step in &report.steps {
            assert!(step.lint_errors.is_empty(), "{id}: {} — {:?}", step.action, step.lint_errors);
        }
        match &report.end {
            flow::FlowEnd::Downloaded(entry) => {
                assert!(takes_epub, "{id} saved an EPUB it cannot save");
                assert_eq!(entry.sha256, expected, "{id}");
                assert_eq!(device.library().len(), 1, "{id}");
                got_it.push(id);
            }
            flow::FlowEnd::Stopped { reason } => {
                assert!(!takes_epub, "{id} should have got the book: {reason}");
                assert!(device.library().is_empty(), "{id}");
                // The device is left on a page that names another way: Send to Kindle
                // for a Kindle, the USB copy from a computer otherwise.
                let page = device.page().unwrap();
                assert_eq!(page.status(), 200, "{id}: {reason}");
                let hint = if is_kindle { "Kindle" } else { "USB" };
                assert!(page.text().contains(hint), "{id}: {}", page.text());
                told_why.push(id);
            }
            other => panic!("{id}: unexpected end {other:?}"),
        }
        record_taps(&format!("matrix-{id}"), device.profile().name, device.taps());
    }
    assert_eq!(
        got_it,
        [
            "kobo-clara",
            "kobo-ereader-ua",
            "pocketbook",
            "vivlio",
            "tolino",
            "generic-eink",
            "thorium-desktop",
            "phone",
            "computer"
        ]
    );
    assert_eq!(told_why, ["kindle-paperwhite", "kindle-2024", "tolino-legacy-android"]);
}

/// A partner file can be downloaded again by whoever holds the card — up to the session
/// limit. Then the gift is `exhausted`: the page says so, and the store shows it.
#[tokio::test]
async fn the_download_limit_exhausts_the_gift_and_everybody_is_told() {
    let world = World::builder()
        .server_config(|config| config.max_download_sessions = 2)
        .start()
        .await
        .unwrap();
    let store = &world.septante;
    let (mut buyer, receipt) = buy_candide(&world, store).await;
    let gift_url = receipt.gifts[0].claim_url.clone().unwrap();

    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    for round in 1..=2 {
        kobo.open(&gift_url).await.unwrap();
        assert!(kobo.click("download").await.unwrap().download().is_some(), "download #{round}");
    }
    assert_eq!(kobo.library().len(), 1, "the same file twice is one book");

    // The third visit: no download any more, and the page says why.
    kobo.open(&gift_url).await.unwrap();
    let page = kobo.page().unwrap();
    assert_eq!(page.status(), 200);
    assert!(!page.has("download") && !page.has("pair-start"), "{}", page.text());
    assert!(page.text_of("claim-count").unwrap().contains('2'));
    assert_lint_clean(&kobo, "the exhausted gift page");
    let pressed = http_post_form(
        &format!("{gift_url}/download"),
        DeviceProfile::kobo_clara().user_agent,
        &[],
    )
    .await;
    assert_ne!(pressed.status, 303, "an exhausted gift mints no session: {}", pressed.text());

    // The partner API and the store's order page use the v0.2 status `exhausted`; the
    // store treats it as used (no refund, no new code).
    let status = world.partner_api(store.kind).gift_status(&gift_id(store, 0)).await.unwrap();
    assert_eq!(status.json()["status"], "exhausted");
    assert_eq!(status.json()["downloads"], 2);
    let after = shop::reload_order(&mut buyer, &receipt).await.unwrap();
    assert_eq!(after.gifts[0].status, "exhausted");
    assert_eq!(after.gifts[0].downloads, Some(2));
    let page = buyer.page().unwrap();
    assert!(!page.has("cancel-gift") && !page.has("reissue-gift"));
}

/// One order, two books: two gifts, two cards, two codes — each code delivers its own
/// book, stamped with its own order-line reference.
#[tokio::test]
async fn an_order_of_two_books_makes_two_independent_gifts() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let order = shop::GiftOrder { books: &["candide", "le-horla"], ..shop::GiftOrder::demo() };
    let mut buyer = browser(&world, DeviceProfile::computer());
    let receipt = shop::buy_gift(&mut buyer, &store.base, &order).await.unwrap();
    assert_eq!(receipt.gifts.len(), 2, "{receipt:#?}");
    let codes: Vec<String> = receipt.gifts.iter().map(|g| g.code.clone().unwrap()).collect();
    assert_ne!(codes[0], codes[1]);
    assert!(receipt.gifts.iter().all(|g| g.status == "created"));

    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    for (index, (book, title)) in
        [("candide", "Candide"), ("le-horla", "Le Horla")].iter().enumerate()
    {
        kobo.open(&world.server.url("/")).await.unwrap();
        kobo.type_into("code-input", &codes[index]).unwrap();
        kobo.click("code-submit").await.unwrap();
        assert_eq!(kobo.page().unwrap().text_of("gift-title"), Some(*title));
        let entry = kobo.click("download").await.unwrap().download().unwrap().clone();
        let line_ref = format!("{}-{}", receipt.order_ref, index + 1);
        let expected = stamped_epub(store, book, &line_ref, "Sacha", "Camille", T0);
        assert_eq!(entry.sha256, sha256_hex(&expected), "{title}");
    }
    assert_eq!(kobo.library().len(), 2);
    // Each gift counts its own claims.
    let api = world.partner_api(store.kind);
    for index in 0..2 {
        let status = api.gift_status(&gift_id(store, index)).await.unwrap().json();
        assert_eq!(status["downloads"], 1, "{status}");
        assert_eq!(status["order_ref"], format!("{}-{}", receipt.order_ref, index + 1).as_str());
    }
}
