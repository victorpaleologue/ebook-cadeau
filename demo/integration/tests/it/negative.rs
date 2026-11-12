//! (h) What must *not* work, and what must keep working when something is wrong: typos,
//! unknown codes and the rate limit, cancelled gifts, reissued codes, tampered signatures
//! (webhooks and partner requests), hostile names and messages, and a `User-Agent` that
//! misleads the server.

use std::time::Duration;

use ebc_demo_ereaders::flow::{self, FlowEnd, RedeemInputs};
use ebc_demo_ereaders::opds::FeedKind;
use ebc_demo_ereaders::{DeviceProfile, EinkBrowser, FileFormat, MailOutcome, OpdsClient};
use ebc_demo_integration::partner::{plain_client, read_reply};
use ebc_demo_integration::shop::{self, GiftOrder};
use ebc_demo_integration::{T0, World};
use ebc_model::GiftCode;
use serde_json::json;

use crate::support::{
    assert_lint_clean, browser, buy_candide, gift_id, http_get, http_post_form, printed_code,
    sha256_hex, stamped_epub, zip_text,
};

/// Types `text` into the code form of the page displayed (the home page or one of its
/// error variants) and submits it.
async fn submit_code(device: &mut EinkBrowser, text: &str) -> u16 {
    device.type_into("code-input", text).unwrap();
    assert!(device.click("code-submit").await.unwrap().is_page());
    device.page().unwrap().status()
}

/// A well-formed code that was never issued (and is not `issued`).
fn unknown_code(n: u64, issued: &GiftCode) -> GiftCode {
    (0..)
        .map(|k| GiftCode::from_payload((n + k) * 1_000_003 + 17).unwrap())
        .find(|code| code != issued)
        .unwrap()
}

#[tokio::test]
async fn typos_are_free_but_unknown_codes_are_counted_and_then_rate_limited() {
    let world = World::start().await.unwrap();
    let (_buyer, receipt) = buy_candide(&world, &world.septante).await;
    let code = printed_code(&receipt);
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&world.server.url("/")).await.unwrap();

    // --- typos: a wrong check letter is caught without a look-up and never counted ---
    let compact = code.compact();
    let last = compact.chars().last().unwrap();
    let other = GiftCode::ALPHABET.chars().find(|c| *c != last).unwrap();
    let typo = format!("{}{other}", &compact[..compact.len() - 1]);
    assert!(GiftCode::parse(&typo).is_err());
    for attempt in 0..15 {
        assert_eq!(submit_code(&mut kobo, &typo).await, 400, "typo #{attempt}");
    }
    let page = kobo.page().unwrap();
    assert_lint_clean(&kobo, "the typo page");
    assert!(page.has("error") && page.has("code-form"), "the form is offered again");
    assert_eq!(kobo.field_value("code-input").unwrap(), typo, "what was typed is kept");
    // Too short, digits, punctuation: malformed, also free.
    for malformed in ["bcdf", "1234-5678-9012", "!!!!", &compact[..11]] {
        assert_eq!(submit_code(&mut kobo, malformed).await, 400, "{malformed:?}");
    }

    // --- unknown codes: 404, with the attempts left, ten times ---
    for n in 0..10u32 {
        let guess = unknown_code(u64::from(n), &code);
        assert_eq!(submit_code(&mut kobo, &guess.grouped()).await, 404, "guess #{n}");
        let page = kobo.page().unwrap();
        assert_eq!(page.url().path(), format!("/k/{}", guess.compact()));
        let left = (9 - n).to_string();
        assert!(page.text_of("error").unwrap().contains(&left), "{}", page.text());
        assert!(page.header("retry-after").is_none());
    }
    assert_lint_clean(&kobo, "the unknown-code page");

    // --- the eleventh: 429 with Retry-After — and nothing is looked up any more, not
    // even the genuine code, until the window has passed ---
    assert_eq!(submit_code(&mut kobo, &unknown_code(99, &code).grouped()).await, 429);
    let page = kobo.page().unwrap();
    let retry_after: u64 = page.header("retry-after").expect("Retry-After").parse().unwrap();
    assert!((1..=600).contains(&retry_after), "{retry_after}");
    assert!(page.has("error") && page.has("code-form"));
    assert_lint_clean(&kobo, "the rate-limit page");
    assert_eq!(submit_code(&mut kobo, &compact).await, 429, "blocked means blocked");
    // The pair form shares the budget: a guesser cannot switch to the shorter code.
    kobo.open(&world.server.url("/e")).await.unwrap();
    kobo.type_into("pair-input", "bcd-fgh").unwrap();
    kobo.click("pair-submit").await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 429);
    // The typo check still answers (it costs nothing): a typo is told apart even now…
    // …and the partner API, which is authenticated, is not affected at all.
    let api = world.partner_api(world.septante.kind);
    let status = api.gift_status(&gift_id(&world.septante, 0)).await.unwrap();
    assert_eq!(status.json()["status"], "created");

    // --- once the window has passed, the card works ---
    world.advance(i64::try_from(retry_after).unwrap() + 1);
    kobo.open(&world.server.url("/")).await.unwrap();
    assert_eq!(submit_code(&mut kobo, &compact).await, 200);
    assert!(kobo.click("download").await.unwrap().download().is_some());
}

#[tokio::test]
async fn a_cancelled_gift_answers_410_everywhere() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let (mut buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let gift_url = world.server.url(&format!("/k/{}", code.compact()));
    let phone = DeviceProfile::phone();

    // A pair code is waiting when the buyer changes their mind.
    let mut recipient = browser(&world, DeviceProfile::phone());
    recipient.open(&gift_url).await.unwrap();
    recipient.click("pair-start").await.unwrap();
    let pair = recipient.page().unwrap().text_of("pair-code").unwrap().to_owned();

    // The buyer cancels on the order page: refunded, the code is voided at the server.
    shop::reload_order(&mut buyer, &receipt).await.unwrap();
    assert!(buyer.click("cancel-gift").await.unwrap().is_page());
    let after = shop::read_order_page(buyer.page().unwrap()).unwrap();
    assert_eq!(
        (after.order_status.as_str(), after.gifts[0].status.as_str()),
        ("refunded", "cancelled")
    );
    let status = world.partner_api(store.kind).gift_status(&gift_id(store, 0)).await.unwrap();
    assert_eq!(status.json()["status"], "cancelled");

    // The card is dead: the page, every action, the pair code.
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&gift_url).await.unwrap();
    let page = kobo.page().unwrap();
    assert_eq!(page.status(), 410, "{}", page.text());
    assert_lint_clean(&kobo, "the cancelled-gift page");
    assert!(!page.has("download") && !page.has("gift-title"), "nothing about the gift is shown");
    for action in ["download", "pair", "send", "redeem"] {
        let reply = http_post_form(&format!("{gift_url}/{action}"), phone.user_agent, &[]).await;
        assert_eq!(reply.status, 410, "POST {action}: {}", reply.text());
    }
    kobo.open(&world.server.url("/e")).await.unwrap();
    kobo.type_into("pair-input", &pair).unwrap();
    assert!(kobo.click("pair-submit").await.unwrap().is_page(), "no file for a cancelled gift");
    assert_ne!(kobo.page().unwrap().status(), 200);
    assert!(kobo.library().is_empty());

    // The store heard about it by webhook too, and a second cancellation is refused.
    assert!(world.webhooks_delivered(Duration::from_secs(5)).await);
    let events = store.snapshot()["webhooks"].clone();
    assert!(events.as_array().unwrap().iter().any(|e| e["event"] == "gift.cancelled"), "{events}");
    assert!(!buyer.page().unwrap().has("cancel-gift"));
}

#[tokio::test]
async fn a_reissued_code_voids_the_lost_card_and_the_new_one_works() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let (mut buyer, receipt) = buy_candide(&world, store).await;
    let old = printed_code(&receipt);
    let expected =
        sha256_hex(&stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0));

    // Whoever found the lost card has a pair code ready.
    let mut finder = browser(&world, DeviceProfile::phone());
    finder.open(&world.server.url(&format!("/k/{}", old.compact()))).await.unwrap();
    finder.click("pair-start").await.unwrap();
    let stale_pair = finder.page().unwrap().text_of("pair-code").unwrap().to_owned();

    // The buyer asks the store for a new code (docs/api.md §4.2 `reissue`).
    shop::reload_order(&mut buyer, &receipt).await.unwrap();
    assert!(buyer.click("reissue-gift").await.unwrap().is_page());
    let after = shop::read_order_page(buyer.page().unwrap()).unwrap();
    let new = printed_code(&after);
    assert_ne!(new, old);
    assert_eq!(
        after.gifts[0].claim_url.as_deref(),
        Some(world.server.url(&format!("/k/{}", new.compact())).as_str())
    );
    assert!(buyer.page().unwrap().has("code-reissued"));
    assert_eq!(after.gifts[0].status, "created", "the gift itself is unchanged");

    // The old code: 410, not 404 — the page says a new code was issued, and the attempt
    // is not counted as a guess. Its pair code died with it.
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&world.server.url("/")).await.unwrap();
    for _ in 0..12 {
        kobo.open(&world.server.url("/")).await.unwrap();
        assert_eq!(submit_code(&mut kobo, &old.grouped()).await, 410);
    }
    assert_lint_clean(&kobo, "the voided-code page");
    kobo.open(&world.server.url("/e")).await.unwrap();
    kobo.type_into("pair-input", &stale_pair).unwrap();
    kobo.click("pair-submit").await.unwrap();
    assert_eq!(kobo.page().unwrap().status(), 404);
    assert!(kobo.library().is_empty());

    // The new code delivers the same watermarked copy.
    kobo.open(&world.server.url("/")).await.unwrap();
    assert_eq!(submit_code(&mut kobo, &new.grouped()).await, 200);
    let outcome = kobo.click("download").await.unwrap();
    assert_eq!(outcome.download().expect("the EPUB").sha256, expected);

    // A second reissue voids the second code as well; three codes, one gift.
    // The world clock is frozen: without advancing it, the second signed reissue would be
    // byte-identical to the first (same timestamp, same signature) and the server would
    // rightly answer it as a replay of the first (docs/api.md §4.1).
    world.advance(1);
    assert!(buyer.click("reissue-gift").await.unwrap().is_page());
    let third = printed_code(&shop::read_order_page(buyer.page().unwrap()).unwrap());
    let ua = DeviceProfile::kobo_clara().user_agent;
    for (code, status) in [(&old, 410), (&new, 410), (&third, 200)] {
        let reply = http_get(&world.server.url(&format!("/k/{}", code.compact())), ua).await;
        assert_eq!(reply.status, status, "{}", code.grouped());
    }
    assert_eq!(store.snapshot()["gifts"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn tampered_signatures_are_refused_by_the_store_and_by_the_server() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let (_buyer, _receipt) = buy_candide(&world, store).await;
    let gift_id = gift_id(store, 0);
    let now = world.clock.now();

    // ---------- webhooks: server → store (docs/api.md §4.4) ----------
    let signer = world.partner_api(store.kind);
    let foreign = world.partner_api(world.grande_enseigne.kind);
    let event = json!({ "id": "evt_test", "event": "gift.redeemed", "gift_id": gift_id, "order_ref": null, "at": now })
        .to_string()
        .into_bytes();
    let deliver = |timestamp: Option<String>, signature: Option<String>, body: Vec<u8>| {
        let url = store.url("/webhooks/ebc");
        async move {
            let mut request = plain_client()
                .unwrap()
                .post(url)
                .header("content-type", "application/json")
                .body(body);
            if let Some(timestamp) = timestamp {
                request = request.header("X-EBC-Timestamp", timestamp);
            }
            if let Some(signature) = signature {
                request = request.header("X-EBC-Signature", signature);
            }
            read_reply(request.send().await.unwrap()).await.unwrap()
        }
    };
    let genuine = signer.signature(now, "POST", "/webhooks/ebc", &event);
    let gift_status = || store.snapshot()["gifts"][0]["status"].clone();
    assert_eq!(gift_status(), "created");

    // The body was altered after signing (another gift, another event).
    let altered =
        String::from_utf8(event.clone()).unwrap().replace("gift.redeemed", "gift.cancelled");
    let reply = deliver(Some(now.to_string()), Some(genuine.clone()), altered.into_bytes()).await;
    assert_eq!((reply.status, reply.error_code().as_deref()), (401, Some("bad_signature")));
    // Signed with another partner's secret.
    let wrong_key = foreign.signature(now, "POST", "/webhooks/ebc", &event);
    let reply = deliver(Some(now.to_string()), Some(wrong_key), event.clone()).await;
    assert_eq!((reply.status, reply.error_code().as_deref()), (401, Some("bad_signature")));
    // Signed for another path, a flipped hex digit, a missing prefix.
    let other_path = signer.signature(now, "POST", "/webhooks/other", &event);
    let flipped = if genuine.ends_with('0') {
        format!("{}1", &genuine[..genuine.len() - 1])
    } else {
        format!("{}0", &genuine[..genuine.len() - 1])
    };
    for bad in [other_path, flipped, genuine.trim_start_matches("v1=").to_owned(), String::new()] {
        let reply = deliver(Some(now.to_string()), Some(bad.clone()), event.clone()).await;
        assert_eq!(reply.status, 401, "{bad:?}");
    }
    // A captured delivery replayed after its time (or post-dated).
    for skew in [-301, 301] {
        let stale = signer.signature(now + skew, "POST", "/webhooks/ebc", &event);
        let reply = deliver(Some((now + skew).to_string()), Some(stale), event.clone()).await;
        assert_eq!((reply.status, reply.error_code().as_deref()), (401, Some("stale_timestamp")));
    }
    // No authentication at all.
    assert_eq!(deliver(None, None, event.clone()).await.status, 401);
    assert_eq!(deliver(Some(now.to_string()), None, event.clone()).await.status, 401);
    // Nothing of the above touched the gift…
    assert_eq!(gift_status(), "created");
    assert!(store.snapshot()["webhooks"].as_array().unwrap().is_empty());
    // …and the genuine delivery is accepted (so the refusals were about the signature).
    let reply = deliver(Some(now.to_string()), Some(genuine), event.clone()).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    assert_eq!(reply.json()["known"], true);
    assert_eq!(gift_status(), "redeemed");

    // ---------- partner requests: store → server (docs/api.md §4.1) ----------
    let path = format!("/api/partner/v1/gifts/{gift_id}");
    let cancel = format!("{path}/cancel");
    assert_eq!(signer.get(&path).await.unwrap().status, 200);
    // A signature for one request does not authorise another one.
    let read_only = signer.signature(now, "GET", &path, b"");
    let reply = signer.send_raw("POST", &cancel, now, &read_only, Vec::new(), &[]).await.unwrap();
    assert_eq!((reply.status, reply.error_code().as_deref()), (401, Some("bad_signature")));
    // Another partner's signature, an unknown partner.
    let reply = foreign.send("POST", &cancel, Vec::new(), &[]).await.unwrap();
    assert_eq!(reply.status, 404, "another partner's gift is indistinguishable from none");
    let stranger = world.partner_api_as("inconnu", "whatever-secret-0123456789");
    let reply = stranger.get(&path).await.unwrap();
    assert_eq!((reply.status, reply.error_code().as_deref()), (401, Some("bad_signature")));
    let impostor = world.partner_api_as(&store.partner_id, "not-the-secret-0123456789");
    assert_eq!(impostor.get(&path).await.unwrap().status, 401);
    // A stale timestamp, correctly signed.
    let stale = signer.signature(now - 301, "POST", &cancel, b"");
    let reply = signer.send_raw("POST", &cancel, now - 301, &stale, Vec::new(), &[]).await.unwrap();
    assert_eq!((reply.status, reply.error_code().as_deref()), (401, Some("stale_timestamp")));
    // A body swapped under a signature.
    let body = json!({ "account_ref": "acct_1" }).to_string().into_bytes();
    let signed = signer.signature(now, "POST", &format!("{path}/fulfilled"), &body);
    let swapped = json!({ "account_ref": "acct_2" }).to_string().into_bytes();
    let reply = signer
        .send_raw("POST", &format!("{path}/fulfilled"), now, &signed, swapped, &[])
        .await
        .unwrap();
    assert_eq!(reply.status, 401);
    // An Idempotency-Key replayed with another body.
    let creation = |title: &str| {
        json!({
            "order_ref": "IDEM-1", "book": { "title": title },
            "delivery": { "type": "reseller_redirect", "url": store.url("/cadeau/activer") },
        })
    };
    let first = signer
        .post_json("/api/partner/v1/gifts", &creation("Candide"), Some("idem-1"))
        .await
        .unwrap();
    assert_eq!(first.status, 201, "{}", first.text());
    let second = signer
        .post_json("/api/partner/v1/gifts", &creation("Zadig"), Some("idem-1"))
        .await
        .unwrap();
    assert_eq!(
        (second.status, second.error_code().as_deref()),
        (409, Some("idempotency_conflict"))
    );
    // After all that, the gift is still there and untouched by the refused cancellations.
    assert_eq!(signer.get(&path).await.unwrap().json()["status"], "created");
}

#[tokio::test]
async fn hostile_names_and_messages_never_execute_or_break_a_page() {
    let world = World::start().await.unwrap();
    let store = &world.septante;
    let giver = "<script>alert('giver')</script>";
    let recipient = "\"><img src=x onerror=alert(1)>";
    let message =
        "</p><script src=//evil.example/x.js></script> {{7*7}} ${7*7} <b>gras</b> & 'fin'";
    let order = GiftOrder { giver, recipient, message, ..GiftOrder::demo() };
    let mut buyer = browser(&world, DeviceProfile::computer());
    let receipt = shop::buy_gift(&mut buyer, &store.base, &order).await.unwrap();
    assert_eq!(receipt.gifts[0].status, "created", "{receipt:#?}");
    let code = printed_code(&receipt);

    // No element was injected anywhere; the text is shown as text.
    let no_injection = |device: &EinkBrowser, what: &str| {
        let page = device.page().unwrap();
        let html = page.html().to_ascii_lowercase();
        assert!(!html.contains("<script"), "{what}: a script element");
        assert!(!html.contains("<img src=x"), "{what}: an injected image");
        assert!(!html.contains("onerror=alert(1)>"), "{what}: a live event handler");
        assert!(!html.contains("<b>gras</b>"), "{what}: injected markup");
    };

    // --- the store's order page ---
    let page = buyer.page().unwrap();
    assert_eq!(page.text_of("gift-giver"), Some(giver));
    assert_eq!(page.text_of("gift-recipient"), Some(recipient));
    no_injection(&buyer, "the store's order page");

    // --- the server's pages, on an e-reader and on a phone ---
    for profile in [DeviceProfile::kobo_clara(), DeviceProfile::phone()] {
        let mut device = browser(&world, profile);
        device.open(&world.server.url(&format!("/k/{}", code.compact()))).await.unwrap();
        let page = device.page().unwrap();
        assert_eq!(page.status(), 200);
        assert!(page.text_of("gift-from").unwrap().contains(giver), "{}", page.text());
        assert_eq!(page.text_of("gift-message"), Some(message));
        no_injection(&device, "the gift page");
        assert_lint_clean(&device, "the gift page with hostile text");
        for id in ["primary-action", "device-chooser", "claim-count", "download"] {
            assert!(page.has(id), "the hostile text broke the page: no #{id}");
        }
        // The page still works.
        let outcome = device.click("download").await.unwrap();
        assert_eq!(outcome.download().expect("the EPUB").title, "Candide");
    }

    // --- the pair page and the OPDS feeds (XML and JSON escaping) ---
    let mut phone = browser(&world, DeviceProfile::phone());
    phone.open(&world.server.url(&format!("/k/{}", code.compact()))).await.unwrap();
    phone.click("pair-start").await.unwrap();
    no_injection(&phone, "the pair page");
    assert!(phone.page().unwrap().has("pair-code"));
    let minted = http_post_form(
        &world.server.url(&format!("/k/{}/download", code.compact())),
        DeviceProfile::computer().user_agent,
        &[("opds", "1")],
    )
    .await;
    let atom = world.server.url(minted.header("location").unwrap());
    let mut app = OpdsClient::new(DeviceProfile::thorium_desktop()).unwrap();
    let feed = app.fetch(&atom).await.unwrap();
    assert_eq!((feed.kind, feed.publications.len()), (FeedKind::Opds1, 1));
    let feed = app.fetch(&atom.replace("catalog.xml", "catalog.json")).await.unwrap();
    assert_eq!((feed.kind, feed.publications[0].title.as_str()), (FeedKind::Opds2, "Candide"));

    // --- the watermark page inside the EPUB ---
    let stamped = stamped_epub(store, "candide", &receipt.order_ref, recipient, giver, T0);
    let watermark = zip_text(&stamped, "OEBPS/watermark.xhtml");
    assert!(!watermark.contains("<script") && !watermark.contains("<img"), "{watermark}");
    assert!(watermark.contains("&lt;script&gt;"));
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&world.server.url(&format!("/k/{}", code.compact()))).await.unwrap();
    let entry = kobo.click("download").await.unwrap().download().unwrap().clone();
    assert_eq!(entry.sha256, sha256_hex(&stamped), "the device got exactly that file");
}

#[tokio::test]
async fn when_the_user_agent_misleads_the_server_the_device_chooser_fixes_it() {
    let address = "sacha_42@kindle.com";
    let world = World::builder()
        .mail_device(DeviceProfile::kindle_paperwhite(), address)
        .start()
        .await
        .unwrap();
    let store = &world.septante;
    let mail = world.mail.as_ref().unwrap();
    mail.approve_sender(&world.server.state.config.send_from);
    let (_buyer, receipt) = buy_candide(&world, store).await;
    let code = printed_code(&receipt);
    let gift_url = world.server.url(&format!("/k/{}", code.compact()));
    let expected =
        sha256_hex(&stamped_epub(store, "candide", &receipt.order_ref, "Sacha", "Camille", T0));

    // --- a 2024 Kindle whose User-Agent no longer says "Kindle": the server takes it for
    // some other e-reader and offers an EPUB download its browser cannot save ---
    let profile = DeviceProfile::kindle_2024();
    assert!(!profile.user_agent.contains("Kindle"));
    let mut kindle = browser(&world, profile);
    kindle.open(&gift_url).await.unwrap();
    assert!(kindle.page().unwrap().has("download"), "sniffing got it wrong, as expected");
    assert!(kindle.click("download").await.is_err(), "a Kindle does not save EPUB");
    assert!(kindle.library().is_empty());
    // The person taps "Kindle" in the device chooser: the right page, and the choice is
    // carried in the links and forms (no cookie).
    assert!(kindle.click("device-kindle").await.unwrap().is_page());
    let page = kindle.page().unwrap();
    assert_eq!(page.url().query(), Some("device=kindle"));
    assert!(!page.has("download") && page.has("send-form"));
    assert_lint_clean(&kindle, "the chosen Kindle page");
    let carried = page.form("send-form").unwrap().field("device").map(|f| f.value.clone());
    assert_eq!(carried.as_deref(), Some("kindle"));
    assert!(kindle.cookies().is_empty());

    // The whole walk in one call, as a person would do it on a second such Kindle.
    let mut second = browser(&world, DeviceProfile::kindle_2024());
    let inputs = RedeemInputs { device_email: Some(address.to_owned()), ..RedeemInputs::default() };
    let report = flow::redeem(&mut second, &gift_url, &inputs).await.unwrap();
    assert!(matches!(report.end, FlowEnd::Sent { .. }), "{report:#?}");
    assert!(
        report.steps.iter().any(|s| s.action.contains("device-kindle")),
        "the walk used the device chooser: {:#?}",
        report.steps
    );
    assert!(report.steps.iter().all(|s| s.lint_errors.is_empty()), "{:#?}", report.steps);
    let records = mail.records();
    assert_eq!(records.len(), 1);
    let MailOutcome::Processed(attachments) = &records[0].outcome else {
        panic!("the mail was not processed: {:#?}", records[0]);
    };
    let delivered = attachments[0].result.as_ref().unwrap();
    assert_eq!((delivered.sha256.as_str(), delivered.format), (expected.as_str(), FileFormat::Kfx));

    // --- a Kobo whose firmware only says "eReader": still an e-reader to the server, the
    // printed code typed on the home page just works ---
    let mut libra = browser(&world, DeviceProfile::kobo_ereader_ua());
    let inputs = RedeemInputs { code: Some(code.grouped()), ..RedeemInputs::default() };
    let report = flow::redeem(&mut libra, &world.server.url("/"), &inputs).await.unwrap();
    let FlowEnd::Downloaded(entry) = &report.end else { panic!("{report:#?}") };
    assert_eq!(entry.sha256, expected);

    // --- the override wins over any User-Agent, in both directions, and junk is ignored ---
    let kobo_ua = DeviceProfile::kobo_clara().user_agent;
    let as_kindle = http_get(&format!("{gift_url}?device=kindle"), kobo_ua).await.text();
    assert!(!as_kindle.contains("id=\"download\"") && as_kindle.contains("id=\"send-form\""));
    let kindle_ua = DeviceProfile::kindle_paperwhite().user_agent;
    let as_kobo = http_get(&format!("{gift_url}?device=kobo"), kindle_ua).await.text();
    assert!(as_kobo.contains("id=\"download\""));
    let junk = http_get(&format!("{gift_url}?device=%3Cscript%3E"), kindle_ua).await;
    assert_eq!(junk.status, 200);
    assert!(!junk.text().contains("id=\"download\""), "junk falls back to detection");
    // The QR form of the URL (upper case) accepts the override too.
    let upper = ebc_model::claim_url_qr(&world.server.base, &code);
    assert_eq!(http_get(&format!("{upper}?device=kobo"), kindle_ua).await.status, 200);
}
