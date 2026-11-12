//! The recipient side (`docs/api.md` §2): pages, code handling, rate limits, the gift
//! page per device, downloads, claim semantics.

use std::sync::Arc;

use crate::common::{
    Harness, LINK_EXPIRES, Reply, T0, UA_DESKTOP, UA_KINDLE, UA_KOBO, UA_PHONE, assert_page,
    start_shop,
};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use ebc_model::GiftCode;
use serde_json::json;

const GIFT_IDS: [&str; 14] = [
    "gift-title",
    "gift-author",
    "gift-from",
    "gift-message",
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
];

/// A well-formed code that was never issued.
fn unknown_code(n: u64) -> String {
    GiftCode::from_payload(1_000_000 + n).unwrap().compact()
}

fn code_of(gift: &serde_json::Value) -> String {
    gift["code"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn home_help_and_pair_pages_are_localised_script_free_and_carry_their_ids() {
    let h = Harness::new();
    for lang in ["fr", "en", "de", "es", "it", "nl", "pt"] {
        let headers = [("accept-language", lang)];
        let home = h.call(Method::GET, "/", &headers, Body::empty()).await;
        assert_page(&home, StatusCode::OK, lang, &["code-form", "code-input", "code-submit"]);
        assert_eq!(home.attr_of("code-input", "name").as_deref(), Some("code"));
        assert_eq!(home.attr_of("code-form", "method").as_deref(), Some("get"));
        assert_eq!(home.attr_of("code-form", "action").as_deref(), Some("/k"));
        assert_eq!(home.attr_of("code-input", "maxlength"), None, "pasted URLs must fit");

        let help = h.call(Method::GET, "/aide", &headers, Body::empty()).await;
        assert_page(&help, StatusCode::OK, lang, &["help"]);
        assert!(help.text_of("help").contains("ebk.example"), "the help names the host to type");

        let pair = h.call(Method::GET, "/e", &headers, Body::empty()).await;
        assert_page(&pair, StatusCode::OK, lang, &["pair-form", "pair-input", "pair-submit"]);
        assert_eq!(pair.attr_of("pair-input", "name").as_deref(), Some("p"));
        assert_eq!(pair.attr_of("pair-form", "method").as_deref(), Some("post"));
    }
    // French is the fallback, also for languages we do not have.
    assert_page(&h.get("/").await, StatusCode::OK, "fr", &["code-form"]);
    let japanese =
        h.call(Method::GET, "/", &[("accept-language", "ja-JP, zh;q=0.8")], Body::empty()).await;
    assert_page(&japanese, StatusCode::OK, "fr", &["code-form"]);
    let weighted = h
        .call(Method::GET, "/", &[("accept-language", "ja, de;q=0.7, en;q=0.9")], Body::empty())
        .await;
    assert_page(&weighted, StatusCode::OK, "en", &["code-form"]);
}

#[tokio::test]
async fn the_contact_point_is_shown_in_the_footer_and_on_the_help_page_when_configured() {
    let h = Harness::new();
    let help = h.get("/aide").await;
    assert!(!help.has_id("contact") && !help.has_id("contact-point"));

    let h = Harness::with(|c| c.contact_email = Some("contact@ebk.example".to_owned()));
    for lang in ["fr", "en", "de", "es", "it", "nl", "pt"] {
        let headers = [("accept-language", lang)];
        let help = h.call(Method::GET, "/aide", &headers, Body::empty()).await;
        assert_page(&help, StatusCode::OK, lang, &["help", "contact", "contact-point"]);
        assert!(help.text_of("contact-point").contains("contact@ebk.example"));
        assert_eq!(help.attr_of("contact", "href").as_deref(), Some("mailto:contact@ebk.example"));
        let home = h.call(Method::GET, "/", &headers, Body::empty()).await;
        assert_page(&home, StatusCode::OK, lang, &["code-form", "contact"]);
    }
    let code = code_of(&h.file_gift(json!({})).await);
    assert_page(&h.get(&format!("/k/{code}")).await, StatusCode::OK, "fr", &["contact"]);
}

#[tokio::test]
async fn healthz_style_sheet_and_unknown_paths() {
    let h = Harness::new();
    let health = h.get("/healthz").await;
    assert_eq!((health.status, health.text().as_str()), (StatusCode::OK, "ok"));

    let css = h.get("/static/eink.css").await;
    assert_eq!(css.status, StatusCode::OK);
    assert!(css.header("content-type").starts_with("text/css"));
    assert!(css.text().contains("#primary-action"));

    assert_page(&h.get("/nothing/here").await, StatusCode::NOT_FOUND, "fr", &[]);
    let api = h.get("/api/v1/nothing").await;
    assert_eq!(api.status, StatusCode::NOT_FOUND);
    assert_eq!(api.json()["error"]["code"], "not_found");
    // Every response, not only pages, carries the hardening headers.
    for reply in [&health, &api] {
        assert_eq!(reply.header("x-content-type-options"), "nosniff");
        assert_eq!(reply.header("referrer-policy"), "no-referrer");
        assert_eq!(reply.header("cache-control"), "no-store");
        assert!(reply.header("content-security-policy").contains("default-src 'none'"));
    }
}

#[tokio::test]
async fn typed_codes_are_normalised_and_redirected_to_the_gift_page() {
    let h = Harness::new();
    let code = code_of(&h.file_gift(json!({})).await);
    let grouped = GiftCode::parse(&code).unwrap().grouped();
    let target = format!("/k/{code}");

    let variants = [
        code.clone(),
        code.to_lowercase(),
        grouped.clone(),
        grouped.to_lowercase(),
        format!("  {} ", grouped.replace('-', " ")),
        grouped.replace('-', "."),
        grouped.replace('-', "_"),
        // What a QR scanner returns, and what people paste.
        format!("HTTPS://EBK.EXAMPLE/K/{code}"),
        format!("https://ebk.example/k/{}", code.to_lowercase()),
    ];
    for typed in variants {
        let query =
            percent_encoding::utf8_percent_encode(&typed, percent_encoding::NON_ALPHANUMERIC);
        let reply = h.get(&format!("/k?code={query}")).await;
        assert_eq!(reply.location(), target, "{typed:?}");
    }
    assert_eq!(
        h.get(&format!("/K?code={code}&device=kobo")).await.location(),
        format!("{target}?device=kobo")
    );
    assert_eq!(h.get(&format!("/k?code={code}&device=toaster")).await.location(), target);
    // `host/CODE` typed without the /k/ is understood too.
    assert_eq!(h.get(&format!("/{}", grouped.to_lowercase())).await.location(), target);
    // No code: back to the form.
    assert_eq!(h.get("/k?code=").await.location(), "/");
    assert_eq!(h.get("/k").await.location(), "/");

    // The gift page itself accepts any spelling of the code, on /k/ and on /K/.
    for path in
        [format!("/k/{code}"), format!("/K/{code}"), format!("/k/{}", grouped.to_lowercase())]
    {
        let page = h.get(&path).await;
        assert_page(&page, StatusCode::OK, "fr", &GIFT_IDS);
        assert_eq!(page.text_of("gift-title"), "Le Horla");
    }
}

#[tokio::test]
async fn mistyped_codes_are_400_say_so_and_are_never_counted() {
    let h = Harness::with(|c| c.rate_unknown_per_10min = 3);
    let code = code_of(&h.file_gift(json!({})).await);
    let mut swapped = code.clone().into_bytes();
    let position = (0..11).find(|i| swapped[*i] != swapped[*i + 1]).unwrap();
    swapped.swap(position, position + 1);
    let swapped = String::from_utf8(swapped).unwrap();

    let typos = [
        swapped.as_str(),            // two letters exchanged: caught by the check letter
        "BCDF-GH0K-LMNL",            // a zero for an O that cannot exist
        "BCDF-GH1K-LMNL",            // a one
        "BCDFAGHJKLMN",              // a vowel
        "BCDF-GHJK",                 // too short
        "BCDF-GHJK-LMNL-BCDF",       // too long
        "<script>alert(1)</script>", // hostile
        "%00%0d%0a",
    ];
    // Far more typos than the budget of unknown codes allows.
    for round in 0..5 {
        for typo in typos {
            let query =
                percent_encoding::utf8_percent_encode(typo, percent_encoding::NON_ALPHANUMERIC);
            for path in [format!("/k?code={query}"), format!("/k/{query}")] {
                let reply = h.get(&path).await;
                assert_page(
                    &reply,
                    StatusCode::BAD_REQUEST,
                    "fr",
                    &["error", "code-form", "code-input"],
                );
                let error = reply.text_of("error");
                assert!(error.contains("pas compté"), "round {round}, {typo:?}: {error}");
                assert!(!reply.has_header("retry-after"));
            }
        }
    }
    let check = h.get(&format!("/k/{swapped}")).await;
    assert!(check.text_of("error").contains("faute de frappe"), "{}", check.text());
    let digits = h.get("/k?code=BCDF-GH0K-LMNL").await;
    assert!(digits.text_of("error").contains("12 lettres"), "{}", digits.text());
    assert_eq!(
        digits.attr_of("code-input", "value").as_deref(),
        Some("BCDF-GH0K-LMNL"),
        "the typed text is kept"
    );

    // The hostile text comes back escaped, in the field only.
    let hostile = h.get("/k?code=%3Cscript%3Ealert(1)%3C%2Fscript%3E").await;
    assert!(!hostile.text().contains("<script"), "{}", hostile.text());
    assert_eq!(
        hostile.attr_of("code-input", "value").as_deref(),
        Some("<script>alert(1)</script>")
    );

    // After all that the client can still open its gift.
    assert_page(&h.get(&format!("/k/{code}")).await, StatusCode::OK, "fr", &GIFT_IDS);
}

#[tokio::test]
async fn unknown_codes_count_down_then_429_until_the_window_has_passed() {
    let h = Harness::with(|c| c.rate_unknown_per_10min = 3);
    let code = code_of(&h.file_gift(json!({})).await);
    let guesser = "203.0.113.50";

    for (attempt, left) in [(1u64, "2"), (2, "1"), (3, "0")] {
        let reply = h.get_from(&format!("/k/{}", unknown_code(attempt)), guesser).await;
        assert_page(&reply, StatusCode::NOT_FOUND, "fr", &["error", "code-form"]);
        let error = reply.text_of("error");
        assert!(error.contains("inconnu"), "{error}");
        assert!(error.contains(&format!("Essais restants\u{a0}: {left}")), "{error}");
        assert_eq!(error.contains("Réessayez dans 10 min"), left == "0", "{error}");
        h.clock.advance(10);
    }
    // Out of attempts: nothing is looked up any more, not even a valid code, on any route.
    let blocked = h.get_from(&format!("/k/{}", unknown_code(4)), guesser).await;
    assert_page(&blocked, StatusCode::TOO_MANY_REQUESTS, "fr", &["error"]);
    let wait: u64 = blocked.header("retry-after").parse().unwrap();
    assert!((1..=600).contains(&wait), "{wait}");
    assert!(blocked.text_of("error").contains("Réessayez dans 10 min"), "{}", blocked.text());
    for path in [format!("/k/{code}"), format!("/k?code={code}")] {
        assert_eq!(h.get_from(&path, guesser).await.status, StatusCode::TOO_MANY_REQUESTS);
    }
    let post = h
        .post_form_with(&format!("/k/{code}/download"), &[], &[("x-forwarded-for", guesser)])
        .await;
    assert_eq!(post.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(post.has_header("retry-after"));
    // A neighbour on the same /24 shares the budget; another network does not.
    assert_eq!(
        h.get_from(&format!("/k/{code}"), "203.0.113.99").await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(h.get_from(&format!("/k/{code}"), "198.51.100.7").await.status, StatusCode::OK);
    // Prepending forged entries to X-Forwarded-For does not buy a new identity.
    let forged = format!("9.9.9.9, 8.8.8.8, {guesser}");
    assert_eq!(
        h.get_from(&format!("/k/{code}"), &forged).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );

    // Ten minutes after the first failure the oldest attempt has left the window.
    h.clock.set(T0 + 601);
    assert_eq!(h.get_from(&format!("/k/{code}"), guesser).await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_global_budget_stops_distributed_guessing_without_locking_out_card_holders() {
    let h = Harness::with(|c| {
        c.rate_unknown_per_10min = 10;
        c.rate_unknown_global_per_10min = 5;
    });
    let code = code_of(&h.file_gift(json!({})).await);
    // Five wrong guesses, each from another network: every per-client budget is intact.
    for n in 0..5u64 {
        let reply = h.get_from(&format!("/k/{}", unknown_code(n)), &format!("198.51.{n}.1")).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND);
    }
    // The global budget is spent: any further wrong guess is refused, from anywhere …
    let sixth = h.get_from(&format!("/k/{}", unknown_code(77)), "192.0.2.1").await;
    assert_page(&sixth, StatusCode::TOO_MANY_REQUESTS, "fr", &["error"]);
    assert!(sixth.has_header("retry-after"));
    // … and whoever guessed wrong recently is not served at all …
    assert_eq!(
        h.get_from(&format!("/k/{code}"), "198.51.0.1").await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    // … but a recipient with a valid card and a clean record is.
    assert_eq!(h.get_from(&format!("/k/{code}"), "203.0.113.9").await.status, StatusCode::OK);
    h.clock.advance(601);
    assert_eq!(
        h.get_from(&format!("/k/{}", unknown_code(78)), "192.0.2.1").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn the_gift_page_shows_the_gift_and_escapes_hostile_text() {
    let h = Harness::new();
    let hostile_name = "<script>alert('giver')</script>";
    let hostile_message = "\"><img src=x onerror=alert(1)>\nligne 2 & fin";
    let gift = h
        .file_gift(json!({
            "book": { "title": "<b>Titre</b> & \"co\"", "contributors": [{"name": "A. <i>Auteur</i>"}, {"name": "B. Autre"}] },
            "giver_name": hostile_name,
            "message": hostile_message,
        }))
        .await;
    let page = h.get(&format!("/k/{}", code_of(&gift))).await;
    assert_page(&page, StatusCode::OK, "fr", &GIFT_IDS);
    assert_eq!(page.text_of("gift-title"), "<b>Titre</b> & \"co\"");
    assert_eq!(page.text_of("gift-author"), "A. <i>Auteur</i> & B. Autre");
    assert!(page.text_of("gift-from").contains(hostile_name));
    assert!(page.text_of("gift-message").contains("\"><img src=x onerror=alert(1)>"));
    assert!(page.text_of("gift-message").contains("ligne 2 & fin"));
    assert!(page.text_of("claim-count").contains("pas encore"));
    // Nothing of it became markup.
    let html = page.html();
    for css in ["img", "b", "i", "script"] {
        assert_eq!(html.select(&scraper::Selector::parse(css).unwrap()).count(), 0, "{css}");
    }
    assert_eq!(
        html.select(&scraper::Selector::parse("#gift-message br").unwrap()).count(),
        1,
        "line break kept"
    );
    let raw = page.text();
    assert!(
        raw.contains("&lt;script&gt;alert(&#39;giver&#39;)&lt;/script&gt;")
            || raw.contains("&lt;script&gt;alert('giver')&lt;/script&gt;"),
        "{raw}"
    );

    // A gift in another language is served in that language whatever the browser says.
    let german = h.file_gift(json!({ "locale": "de", "giver_name": null })).await;
    let page = h
        .call(
            Method::GET,
            &format!("/k/{}", code_of(&german)),
            &[("accept-language", "fr")],
            Body::empty(),
        )
        .await;
    assert_page(&page, StatusCode::OK, "de", &GIFT_IDS);
    assert!(!page.text_of("gift-from").is_empty(), "an anonymous gift still says it is a gift");
}

#[tokio::test]
async fn each_device_class_gets_its_primary_action_and_the_chooser_overrides_detection() {
    let h = Harness::new();
    let code = code_of(&h.file_gift(json!({})).await);
    let grouped = GiftCode::parse(&code).unwrap().grouped();
    let path = format!("/k/{code}");
    let inside = |reply: &crate::common::Reply, outer: &str, inner: &str| {
        let selector = scraper::Selector::parse(&format!("#{outer} #{inner}")).unwrap();
        reply.html().select(&selector).count() == 1
    };

    // E-readers that save EPUB: one big download button, nothing to type.
    let kobo = h.get_as(&path, UA_KOBO).await;
    assert_page(&kobo, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(inside(&kobo, "primary-action", "download"));
    assert_eq!(
        kobo.attr_of("download-form", "action").as_deref(),
        Some(format!("/k/{code}/download").as_str())
    );
    assert_eq!(kobo.attr_of("download-form", "method").as_deref(), Some("post"));
    assert!(!kobo.has_id("pair-start") && !kobo.has_id("send-form"));
    assert!(kobo.text().contains("<body class=\"eink\">"));

    // Kindle: its browser does not save EPUB, so no download button, Send to Kindle instead.
    let kindle = h.get_as(&path, UA_KINDLE).await;
    assert_page(&kindle, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(!kindle.has_id("download"));
    assert!(kindle.text_of("primary-action").contains("Send to Kindle"));

    // Phone: read here or hand over to an app; the e-reader routes follow.
    let phone = h.get_as(&path, UA_PHONE).await;
    assert_page(&phone, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(inside(&phone, "primary-action", "download"));
    assert!(phone.text_of("download").contains("téléphone"));
    let others = phone.text_of("other-ways");
    assert!(others.contains("ebk.example") && others.contains(&grouped), "{others}");
    assert!(inside(&phone, "other-ways", "pair-start") && inside(&phone, "other-ways", "opds"));
    assert!(others.contains("Send to Kindle"));

    // Computer: download, plus the USB route.
    let desktop = h.get_as(&path, UA_DESKTOP).await;
    assert_page(&desktop, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(inside(&desktop, "primary-action", "download"));
    assert!(desktop.text_of("other-ways").contains("USB"));

    // The chooser: one link per device, pointing back with ?device=.
    for (id, choice) in [
        ("device-kobo", "kobo"),
        ("device-kindle", "kindle"),
        ("device-pocketbook", "pocketbook"),
        ("device-tolino", "tolino"),
        ("device-phone", "phone"),
        ("device-computer", "computer"),
    ] {
        assert_eq!(
            desktop.attr_of(id, "href").as_deref(),
            Some(format!("/k/{code}?device={choice}").as_str())
        );
    }
    assert_eq!(desktop.attr_of("device-computer", "class").as_deref(), Some("on"));

    // A recent Kindle that looks like a desktop browser: the recipient says "Kindle".
    let chosen = h.get_as(&format!("{path}?device=kindle"), UA_DESKTOP).await;
    assert_page(&chosen, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(!chosen.has_id("download"));
    assert_eq!(chosen.attr_of("device-kindle", "class").as_deref(), Some("on"));
    // … and a Kobo that only says "eReader", or a phone user who wants the Kobo page.
    let as_kobo = h.get_as(&format!("{path}?device=kobo"), UA_PHONE).await;
    assert!(inside(&as_kobo, "primary-action", "download") && !as_kobo.has_id("pair-start"));
    // The choice travels in the forms and links (no cookie).
    let hidden =
        scraper::Selector::parse("#download-form input[type=hidden][name=device][value=kobo]")
            .unwrap();
    assert_eq!(as_kobo.html().select(&hidden).count(), 1);
    assert!(as_kobo.text().contains("/aide?device=kobo"));
    assert!(!as_kobo.has_header("set-cookie"));
    // An unknown value is ignored.
    let nonsense = h.get_as(&format!("{path}?device=%3Cscript%3E"), UA_PHONE).await;
    assert!(nonsense.text_of("download").contains("téléphone"));
}

/// The status document of a shelf gift, through the partner API.
async fn status_of(h: &Harness, id: &str) -> serde_json::Value {
    h.shelf(Method::GET, &format!("/api/partner/v1/gifts/{id}"), "").await.json()
}

/// The `/ebc/…` requests (link requests) a shop received.
fn link_requests(shop: &crate::common::Receiver) -> usize {
    shop.requests().iter().filter(|r| r.path_and_query.starts_with("/ebc/")).count()
}

#[tokio::test]
async fn downloading_counts_claims_shows_them_and_stops_at_the_session_limit() {
    let h = Harness::with(|c| c.max_download_sessions = 2);
    let shop = start_shop().await;
    let gift = h.file_gift_at(&shop, "/ebc/file", json!({})).await;
    let code = code_of(&gift);
    let id = gift["id"].as_str().unwrap().to_owned();
    assert_eq!(status_of(&h, &id).await["status"], "created");
    assert_eq!(link_requests(&shop), 0, "nothing is asked of the partner before a claim");

    let first = h.post_form(&format!("/k/{code}/download"), &[]).await.location();
    let parts: Vec<&str> = first.split('/').collect();
    assert_eq!(parts.len(), 4, "{first}");
    assert_eq!((parts[1], parts[3]), ("d", "Le-Horla.epub"));
    assert!(
        parts[2].len() == 32 && parts[2].bytes().all(|b| b.is_ascii_hexdigit()),
        "128-bit token: {first}"
    );
    // The claim asked the partner for a link (signed: the shop checks it) …
    assert_eq!(link_requests(&shop), 1);
    // … and the session URL sends the device to it. The core never relays the bytes.
    let file = h.get(&first).await;
    let link = file.location();
    assert!(link.starts_with(&shop.url("/dl/le-horla.epub")), "{link}");
    assert!(file.body.is_empty() && !file.has_header("content-disposition"));
    // Fetching again within the session is not another claim, and reuses the link.
    assert_eq!(h.get(&first).await.location(), link);
    assert_eq!(link_requests(&shop), 1);

    let page = h.get(&format!("/k/{code}")).await;
    let count = page.text_of("claim-count");
    assert!(count.contains("1 fois") && count.contains("12 novembre 2026"), "{count}");
    let doc = status_of(&h, &id).await;
    assert_eq!(
        (doc["status"].as_str(), doc["downloads"].as_u64(), doc["redeemed_at"].as_i64()),
        (Some("redeemed"), Some(1), Some(T0))
    );

    h.clock.advance(86_400);
    let second = h.post_form(&format!("/K/{}/download", code.to_lowercase()), &[]).await.location();
    assert_ne!(first, second, "every claim gets its own session");
    assert_eq!(h.get(&second).await.status, StatusCode::SEE_OTHER);

    // The limit is reached: no third session, and the page says so instead of offering one.
    let third = h.post_form(&format!("/k/{code}/download"), &[]).await;
    assert_page(&third, StatusCode::GONE, "fr", &["gift-title", "primary-action", "claim-count"]);
    assert!(!third.has_id("download"));
    let page = h.get(&format!("/k/{code}")).await;
    assert_page(&page, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(!page.has_id("download"));
    assert!(page.text_of("primary-action").contains("maximal"));
    let count = page.text_of("claim-count");
    assert!(count.contains("2 fois") && count.contains("13 novembre 2026"), "{count}");
    let doc = status_of(&h, &id).await;
    assert_eq!(
        (doc["status"].as_str(), doc["downloads"].as_u64(), doc["redeemed_at"].as_i64()),
        (Some("exhausted"), Some(2), Some(T0))
    );
    assert_eq!(link_requests(&shop), 2, "the refused third claim asked nothing");
}

#[tokio::test]
async fn partner_file_sessions_redirect_get_and_head_and_only_the_exact_url_works() {
    let h = Harness::new();
    let shop = start_shop().await;
    let gift = h
        .file_gift_at(&shop, "/ebc/file", json!({ "book": { "title": "L’Étranger — été 1942" } }))
        .await;
    let code = code_of(&gift);
    let url = h
        .post_form_with(&format!("/k/{code}/download"), &[], &[("user-agent", UA_DESKTOP)])
        .await
        .location();
    assert!(url.ends_with("/L-Etranger-ete-1942.epub"), "{url}");
    // Browsers check every hop of a form's redirect chain against `form-action`: the gift
    // page and the pair form allow the partners' download hosts, and nothing else.
    let page = h.get(&format!("/k/{code}")).await;
    let csp = page.header("content-security-policy");
    assert!(csp.contains("form-action 'self' http://127.0.0.1:* https://127.0.0.1:*;"), "{csp}");
    let pair_form = h.get("/e").await;
    assert!(pair_form.header("content-security-policy").contains("http://127.0.0.1:*"));
    assert!(!h.get("/").await.header("content-security-policy").contains("127.0.0.1"));

    let get = h.get_as(&url, UA_DESKTOP).await;
    let link = get.location();
    assert!(link.starts_with(&shop.url("/dl/")), "{link}");
    assert_eq!(get.header("cache-control"), "no-store");
    assert_eq!(get.header("referrer-policy"), "no-referrer");
    let head = h.call(Method::HEAD, &url, &[("user-agent", UA_KOBO)], Body::empty()).await;
    assert_eq!((head.status, head.location()), (StatusCode::SEE_OTHER, link.clone()));
    assert!(head.body.is_empty());

    // The partner was asked once, signed, with the gift id and order reference.
    let asked: Vec<_> =
        shop.requests().into_iter().filter(|r| r.path_and_query.starts_with("/ebc/")).collect();
    assert_eq!(asked.len(), 1);
    let query = crate::common::query_of(&asked[0].path_and_query);
    assert_eq!(query["ebc_gift"], gift["id"].as_str().unwrap());
    assert_eq!(query["ebc_order"], "SHELF-000123");
    assert_eq!(query["ebc_ts"], T0.to_string());
    assert_eq!(asked[0].method, "GET");
    assert_eq!(asked[0].headers["accept"], "application/json");

    // Only the exact URL works; nothing in it is a file-system path.
    let token = url.split('/').nth(2).unwrap();
    for bad in [
        format!("/d/{token}/other.epub"),
        format!("/d/{token}/..%2F..%2Fetc%2Fpasswd"),
        format!("/d/{token}/"),
        "/d/00000000000000000000000000000000/L-Etranger-ete-1942.epub".to_owned(),
        "/d/..%2F..%2Fdb.json/L-Etranger-ete-1942.epub".to_owned(),
        format!(
            "/d/{}/L-Etranger-ete-1942.epub",
            token.to_uppercase().replace(|c: char| c.is_ascii_digit(), "g")
        ),
    ] {
        assert_eq!(h.get(&bad).await.status, StatusCode::NOT_FOUND, "{bad}");
    }
    // Sessions last 15 minutes.
    h.clock.advance(15 * 60 - 1);
    assert_eq!(h.get(&url).await.status, StatusCode::SEE_OTHER);
    h.clock.advance(1);
    assert_page(&h.get(&url).await, StatusCode::GONE, "fr", &["error"]);
}

#[tokio::test]
async fn partner_links_are_renewed_when_they_expire_and_a_303_answer_works_too() {
    let h = Harness::new();
    let shop = start_shop().await;
    let gift = h.file_gift_at(&shop, "/ebc/see-other", json!({})).await;
    let url = h.post_form(&format!("/k/{}/download", code_of(&gift)), &[]).await.location();
    // A 303 answer gives no expiry: the link is reused for about a minute, then renewed.
    let link = h.get(&url).await.location();
    assert!(link.starts_with(&shop.url("/dl/")), "{link}");
    assert_eq!(link_requests(&shop), 1);
    h.clock.advance(30);
    assert_eq!(h.get(&url).await.location(), link);
    assert_eq!(link_requests(&shop), 1);
    h.clock.advance(30);
    let renewed = h.get(&url).await.location();
    assert_eq!(link_requests(&shop), 2, "a fresh link was asked for");
    assert!(renewed.ends_with(&format!("t={}", T0 + 60)), "{renewed}");
    // A JSON link carries its own expiry, far away here.
    let gift = h.file_gift_at(&shop, "/ebc/file", json!({})).await;
    let url = h.post_form(&format!("/k/{}/download", code_of(&gift)), &[]).await.location();
    h.clock.advance(14 * 60);
    let _ = h.get(&url).await.location();
    assert_eq!(link_requests(&shop), 3, "still the link of the claim");
    const { assert!(LINK_EXPIRES > T0 + 15 * 60) };
}

#[tokio::test]
async fn a_partner_that_gives_no_usable_link_costs_the_recipient_no_claim() {
    let h = Harness::new();
    let shop = start_shop().await;
    for (path, why) in [
        ("/ebc/elsewhere", "a host outside the partner's redirect_hosts"),
        ("/ebc/wrong-type", "a PDF announced for an EPUB gift"),
        ("/ebc/refuse", "a refusal"),
    ] {
        let gift = h.file_gift_at(&shop, path, json!({})).await;
        let code = code_of(&gift);
        let reply: Reply = h.post_form(&format!("/k/{code}/download"), &[]).await;
        assert_page(&reply, StatusCode::BAD_GATEWAY, "fr", &["primary-action", "error"]);
        assert!(!reply.has_header("location"), "{why}");
        assert!(reply.text_of("error").contains("boutique"), "{why}");
        let doc = status_of(&h, gift["id"].as_str().unwrap()).await;
        assert_eq!(
            (doc["status"].as_str(), doc["downloads"].as_u64()),
            (Some("created"), Some(0)),
            "{why}"
        );
        assert!(h.get(&format!("/k/{code}")).await.text_of("claim-count").contains("pas encore"));
    }
    // An unreachable partner is no link either.
    let unreachable = json!({ "delivery": { "type": "partner_file", "format": "epub", "url": "http://127.0.0.1:9/ebc/file" } });
    let gift = h.file_gift_at(&shop, "/ebc/file", unreachable).await;
    let reply = h.post_form(&format!("/k/{}/download", code_of(&gift)), &[]).await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn reveal_date_expiry_reissue_and_cancellation() {
    let h = Harness::new();

    // Wrapped until the day: the page shows the date and no action; actions are refused.
    let wrapped =
        h.file_gift(json!({ "reveal_at": T0 + 3 * 86_400, "expires_at": T0 + 30 * 86_400 })).await;
    let code = code_of(&wrapped);
    let page = h.get(&format!("/k/{code}")).await;
    assert_page(&page, StatusCode::OK, "fr", &["gift-from", "reveal-date"]);
    assert!(page.text_of("reveal-date").contains("15 novembre 2026"));
    assert!(
        !page.has_id("download") && !page.has_id("gift-title") && !page.has_id("device-chooser")
    );
    for action in ["download", "pair", "send", "redeem"] {
        let refused = h.post_form(&format!("/k/{code}/{action}"), &[]).await;
        assert!(
            refused.status == StatusCode::FORBIDDEN || refused.status == StatusCode::CONFLICT,
            "{action}: {}",
            refused.status
        );
        assert!(!refused.has_header("location"));
    }
    h.clock.set(T0 + 3 * 86_400);
    assert_page(&h.get(&format!("/k/{code}")).await, StatusCode::OK, "fr", &GIFT_IDS);
    assert_eq!(
        h.post_form(&format!("/k/{code}/download"), &[]).await.status,
        StatusCode::SEE_OTHER
    );
    // … and gone once expired.
    h.clock.set(T0 + 30 * 86_400);
    let expired = h.get(&format!("/k/{code}")).await;
    assert_page(&expired, StatusCode::GONE, "fr", &["error"]);
    assert!(expired.text_of("error").contains("date limite"));
    assert_eq!(h.post_form(&format!("/k/{code}/download"), &[]).await.status, StatusCode::GONE);
    h.clock.set(T0);

    // Reissue (lost card): the old code answers 410 and is not a "wrong guess".
    let gift = h.file_gift(json!({})).await;
    let (id, old) = (gift["id"].as_str().unwrap(), code_of(&gift));
    let reissued = h.shelf(Method::POST, &format!("/api/partner/v1/gifts/{id}/reissue"), "").await;
    assert_eq!(reissued.status, StatusCode::OK, "{}", reissued.text());
    let new = reissued.json();
    let new_code = new["code"].as_str().unwrap();
    assert_ne!(new_code, old);
    assert_eq!(new["code_display"], GiftCode::parse(new_code).unwrap().grouped());
    assert_eq!(new["claim_url"], format!("https://ebk.example/k/{new_code}"));
    for _ in 0..15 {
        let gone = h.get(&format!("/k/{old}")).await;
        assert_page(&gone, StatusCode::GONE, "fr", &["error"]);
        assert!(gone.text_of("error").contains("remplacé"));
    }
    assert_eq!(h.post_form(&format!("/k/{old}/download"), &[]).await.status, StatusCode::GONE);
    assert_page(&h.get(&format!("/k/{new_code}")).await, StatusCode::OK, "fr", &GIFT_IDS);

    // Cancellation: the code stops working, live download links die.
    let session = h.post_form(&format!("/k/{new_code}/download"), &[]).await.location();
    assert_eq!(h.get(&session).await.status, StatusCode::SEE_OTHER);
    h.clock.advance(1);
    let cancelled = h.shelf(Method::POST, &format!("/api/partner/v1/gifts/{id}/cancel"), "").await;
    assert_eq!(cancelled.status, StatusCode::OK, "{}", cancelled.text());
    assert_eq!(cancelled.json()["status"], "cancelled");
    let gone = h.get(&format!("/k/{new_code}")).await;
    assert_page(&gone, StatusCode::GONE, "fr", &["error"]);
    assert!(gone.text_of("error").contains("annulé"));
    assert_eq!(h.get(&session).await.status, StatusCode::NOT_FOUND);
    let again = h.shelf(Method::POST, &format!("/api/partner/v1/gifts/{id}/reissue"), "").await;
    assert_eq!(
        (again.status, again.json()["error"]["code"].as_str()),
        (StatusCode::CONFLICT, Some("not_reissuable"))
    );
}

#[tokio::test]
async fn names_and_message_are_deleted_a_year_after_the_first_claim_and_the_gift_still_works() {
    let h = Harness::with(|c| c.retention_after_claim_secs = 365 * 86_400);
    let gift = h.file_gift(json!({})).await;
    let code = code_of(&gift);
    let id = gift["id"].as_str().unwrap();
    let page = h.get(&format!("/k/{code}")).await;
    assert!(page.text_of("gift-from").contains("Camille"));
    // Unclaimed for two years: nothing is deleted (the card may still be in a drawer).
    h.clock.advance(2 * 365 * 86_400);
    let _ = h.file_gift(json!({})).await; // a write runs the purge
    assert!(h.get(&format!("/k/{code}")).await.text_of("gift-message").contains("anniversaire"));

    // First claim, then a year less a second: still there.
    let claimed_at = h.state.now();
    let _ = h.post_form(&format!("/k/{code}/download"), &[]).await.location();
    h.clock.advance(365 * 86_400 - 1);
    let _ = h.file_gift(json!({})).await;
    assert!(h.get(&format!("/k/{code}")).await.text_of("gift-from").contains("Camille"));
    // A year after the first claim: names and message are gone, the gift keeps working.
    h.clock.advance(1);
    let _ = h.file_gift(json!({})).await;
    let page = h.get(&format!("/k/{code}")).await;
    assert_page(&page, StatusCode::OK, "fr", &GIFT_IDS);
    assert!(!page.text_of("gift-from").contains("Camille"), "{}", page.text_of("gift-from"));
    assert_eq!(page.text_of("gift-message"), "");
    assert_eq!(page.text_of("gift-title"), "Le Horla");
    assert!(page.has_id("download"));
    let record = h.state.db.view(|db| db.gifts[id].clone());
    assert!(record.giver_name.is_none() && record.recipient_name.is_none());
    assert_eq!(record.personal_data_deleted_at, Some(claimed_at + 365 * 86_400));
    assert_eq!(
        h.post_form(&format!("/k/{code}/download"), &[]).await.status,
        StatusCode::SEE_OTHER
    );
    assert_eq!(status_of(&h, id).await["downloads"], 2);
}

/// Collects everything the service logs.
#[derive(Clone, Default)]
struct LogCapture(Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for LogCapture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn no_log_line_at_any_level_contains_a_code_or_a_token() {
    // A process-wide subscriber at TRACE level: a thread-local one misses call sites
    // that another test thread registered first. Other tests log into it too, which is
    // fine: the secrets looked for below belong to this test only.
    static CAPTURE: std::sync::OnceLock<LogCapture> = std::sync::OnceLock::new();
    let capture = CAPTURE.get_or_init(|| {
        let capture = LogCapture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        capture
    });

    let h = Harness::new();
    assert_eq!(h.get("/log-probe-start").await.status, StatusCode::NOT_FOUND);
    let gift = h.file_gift(json!({})).await;
    let code = code_of(&gift);
    let grouped = GiftCode::parse(&code).unwrap().grouped();

    // Every route a code or a token travels through, including the failures.
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::OK);
    assert_eq!(h.get(&format!("/K/{}", grouped.to_lowercase())).await.status, StatusCode::OK);
    assert_eq!(h.get(&format!("/{code}")).await.status, StatusCode::SEE_OTHER);
    assert_eq!(h.get(&format!("/k?code={code}")).await.status, StatusCode::SEE_OTHER);
    let session = h.post_form(&format!("/k/{code}/download"), &[]).await.location();
    let token = session.split('/').nth(2).unwrap().to_owned();
    assert_eq!(h.get(&session).await.status, StatusCode::SEE_OTHER);
    assert_eq!(h.get(&format!("/o/{token}/catalog.xml")).await.status, StatusCode::OK);
    let pair = h.post_form(&format!("/k/{code}/pair"), &[]).await.text_of("pair-code");
    assert_eq!(h.post_form("/e", &[("p", pair.as_str())]).await.status, StatusCode::SEE_OTHER);
    assert_eq!(
        h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@kindle.com")]).await.status,
        StatusCode::CONFLICT
    );
    assert_eq!(h.get(&format!("/k/{}", unknown_code(5))).await.status, StatusCode::NOT_FOUND);
    assert_eq!(h.get(&format!("/k/{}X", &code[..11])).await.status, StatusCode::BAD_REQUEST);
    let reissued = h
        .shelf(
            Method::POST,
            &format!("/api/partner/v1/gifts/{}/reissue", gift["id"].as_str().unwrap()),
            "",
        )
        .await;
    let new_code = reissued.json()["code"].as_str().unwrap().to_owned();
    assert_eq!(h.get(&format!("/k/{code}")).await.status, StatusCode::GONE);
    assert_eq!(h.get(&format!("/k/{new_code}")).await.status, StatusCode::OK);

    assert_eq!(h.get("/log-probe-end").await.status, StatusCode::NOT_FOUND);

    let logs = String::from_utf8_lossy(&capture.0.lock().unwrap()).to_lowercase();
    // The requests of this test were captured from start to end, with redacted paths.
    for expected in
        ["/log-probe-start", "/log-probe-end", "/k/…", "/d/…/le-horla.epub", "/o/…/catalog.xml"]
    {
        assert!(logs.contains(expected), "{expected} is missing from the captured logs");
    }
    let secrets = [
        code.clone(),
        grouped,
        new_code,
        unknown_code(5),
        token,
        pair.clone(),
        pair.replace('-', ""),
        "a@kindle.com".to_owned(),
    ];
    for secret in secrets {
        assert!(!logs.contains(&secret.to_lowercase()), "{secret} was logged");
    }
    // Nor the signed request to the partner, nor its download link.
    assert!(!logs.contains("ebc_sig") && !logs.contains("/dl/le-horla.epub"), "partner URL logged");
}
