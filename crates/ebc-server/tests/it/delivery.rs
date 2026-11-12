//! Getting the book onto a device by other routes (`docs/api.md` §2): pairing a phone
//! with an e-reader, sending to a device address by e-mail, OPDS feeds, licence files
//! issued by a store.

use crate::common::{
    Harness, Receiver, SmtpSink, UA_KINDLE, UA_KOBO, UA_PHONE, assert_page, epub, partner,
};
use axum::http::{Method, StatusCode};
use mail_parser::MimeHeaders;
use serde_json::json;

fn code_of(gift: &serde_json::Value) -> String {
    gift["code"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn a_phone_pairs_an_e_reader_with_a_short_single_use_code() {
    let h = Harness::new();
    let code = code_of(&h.hosted_gift(json!({})).await);

    // On the phone.
    let shown =
        h.post_form_with(&format!("/k/{code}/pair"), &[], &[("user-agent", UA_PHONE)]).await;
    assert_page(&shown, StatusCode::OK, "fr", &["pair-code", "pair-url", "pair-expires"]);
    let pair = shown.text_of("pair-code");
    assert_eq!(pair.len(), 7, "{pair}");
    assert!(
        pair.chars().enumerate().all(|(i, c)| if i == 3 {
            c == '-'
        } else {
            "BCDFGHJKLMNPQRSTVWXZ".contains(c)
        }),
        "{pair}"
    );
    assert_eq!(shown.text_of("pair-url"), "ebk.example/e");
    let expires = shown.text_of("pair-expires");
    assert!(expires.contains("15 minutes") && expires.contains("00:15 UTC"), "{expires}");

    // On the e-reader: lower case and no hyphen are fine.
    let typed = pair.replace('-', "").to_lowercase();
    let redirect = h
        .post_form_with("/e", &[("p", typed.as_str())], &[("user-agent", UA_KOBO)])
        .await
        .location();
    assert!(redirect.starts_with("/d/") && redirect.ends_with("/Le-Horla.epub"), "{redirect}");
    let file = h.get_as(&redirect, UA_KOBO).await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body.as_ref(), epub().as_slice());

    // Single use.
    let reused = h.post_form("/e", &[("p", pair.as_str())]).await;
    assert_page(&reused, StatusCode::NOT_FOUND, "fr", &["error", "pair-form", "pair-input"]);
    assert!(reused.text_of("error").contains("Essais restants"));

    // One live code per gift: a new one replaces the previous one.
    let first = h.post_form(&format!("/k/{code}/pair"), &[]).await.text_of("pair-code");
    let second = h.post_form(&format!("/k/{code}/pair"), &[]).await.text_of("pair-code");
    assert_ne!(first, second);
    assert_eq!(
        h.post_form_with("/e", &[("p", first.as_str())], &[("x-forwarded-for", "192.0.2.10")])
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    // Valid for 15 minutes, not a second more.
    h.clock.advance(15 * 60);
    assert_eq!(
        h.post_form_with("/e", &[("p", second.as_str())], &[("x-forwarded-for", "192.0.2.20")])
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let third = h.post_form(&format!("/k/{code}/pair"), &[]).await.text_of("pair-code");
    h.clock.advance(15 * 60 - 1);
    assert_eq!(
        h.post_form_with("/e", &[("p", third.as_str())], &[("x-forwarded-for", "192.0.2.30")])
            .await
            .status,
        StatusCode::SEE_OTHER
    );

    // The claims are visible to whoever holds the card.
    assert!(h.get(&format!("/k/{code}")).await.text_of("claim-count").contains("2 fois"));
}

#[tokio::test]
async fn wrong_pair_codes_are_budgeted_and_malformed_ones_are_not_counted() {
    let h = Harness::with(|c| c.rate_unknown_per_10min = 3);
    let code = code_of(&h.hosted_gift(json!({})).await);
    let pair = h.post_form(&format!("/k/{code}/pair"), &[]).await.text_of("pair-code");
    let reader = [("x-forwarded-for", "198.51.100.33")];

    // Not six letters: a typo, said so, never counted.
    for typo in ["", "BCD", "BCD-FG", "BCDFGHJ", "123456", "BCDFGA", "<b>x</b>"] {
        for _ in 0..3 {
            let reply = h.post_form_with("/e", &[("p", typo)], &reader).await;
            assert_page(&reply, StatusCode::BAD_REQUEST, "fr", &["error", "pair-form"]);
            assert!(reply.text_of("error").contains("pas compté"), "{typo:?}");
        }
    }
    // A missing or unreadable form is a typo too, not a crash.
    let bare = h.call(Method::POST, "/e", &reader, axum::body::Body::empty()).await;
    assert_eq!(bare.status, StatusCode::BAD_REQUEST);

    // Six letters that are not a live code: counted, then 429 with Retry-After.
    let wrong = if pair == "BCD-FGH" { "ZZZ-ZZZ" } else { "BCD-FGH" };
    for left in ["2", "1", "0"] {
        let reply = h.post_form_with("/e", &[("p", wrong)], &reader).await;
        assert_page(&reply, StatusCode::NOT_FOUND, "fr", &["error", "pair-form"]);
        assert!(reply.text_of("error").contains(&format!("Essais restants\u{a0}: {left}")));
    }
    let blocked = h.post_form_with("/e", &[("p", pair.as_str())], &reader).await;
    assert_page(&blocked, StatusCode::TOO_MANY_REQUESTS, "fr", &["error", "pair-form"]);
    assert!((1..=600).contains(&blocked.header("retry-after").parse::<u64>().unwrap()));
    // The budget is shared with gift codes: this client cannot guess those either.
    assert_eq!(
        h.call(Method::GET, &format!("/k/{code}"), &reader, axum::body::Body::empty()).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    // The right code still works for the person who has it.
    assert_eq!(h.post_form("/e", &[("p", pair.as_str())]).await.status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn wrong_pair_codes_from_everywhere_exhaust_the_global_budget() {
    let h = Harness::with(|c| {
        c.rate_unknown_per_10min = 5;
        c.rate_unknown_global_per_10min = 4;
    });
    for n in 0..4 {
        let from = format!("203.0.{n}.1");
        assert_eq!(
            h.post_form_with("/e", &[("p", "BCDFGH")], &[("x-forwarded-for", from.as_str())])
                .await
                .status,
            StatusCode::NOT_FOUND
        );
    }
    let reply =
        h.post_form_with("/e", &[("p", "BCDFGH")], &[("x-forwarded-for", "192.0.2.77")]).await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(reply.has_header("retry-after"));
}

#[tokio::test]
async fn send_to_device_mails_the_file_to_allowed_domains_only() {
    let sink = SmtpSink::start().await;
    let url = sink.url();
    let h = Harness::with(|c| {
        c.smtp_url = Some(url);
        c.send_from = "livraison@ebk.example".to_owned();
    });
    let gift = h.hosted_gift(json!({ "book": { "title": "Le Horla, suivi de « Lui ? »" } })).await;
    let code = code_of(&gift);

    // A Kindle gets the e-mail form as its primary action, with the sender to approve.
    let page = h.get_as(&format!("/k/{code}"), UA_KINDLE).await;
    assert_page(&page, StatusCode::OK, "fr", &["send-form", "device-email", "primary-action"]);
    assert_eq!(page.attr_of("device-email", "name").as_deref(), Some("device_email"));
    assert_eq!(
        page.attr_of("send-form", "action").as_deref(),
        Some(format!("/k/{code}/send").as_str())
    );
    let action = page.text_of("primary-action");
    assert!(action.contains("livraison@ebk.example") && action.contains("@kindle.com"), "{action}");
    // On a phone the form is one of the other ways.
    assert_page(
        &h.get_as(&format!("/k/{code}"), UA_PHONE).await,
        StatusCode::OK,
        "fr",
        &["send-form", "download"],
    );

    // Addresses outside the device domains, and anything that is not a plain address.
    let refused = [
        "sacha@gmail.com",
        "sacha@kindle.com.evil.example",
        "sacha@kindle.com\r\nBcc: victim@example.org",
        "sacha@kindle.com, victim@example.org",
        "Sacha <sacha@kindle.com>",
        "",
    ];
    for address in refused {
        let reply = h.post_form(&format!("/k/{code}/send"), &[("device_email", address)]).await;
        assert_page(&reply, StatusCode::BAD_REQUEST, "fr", &["send-result"]);
        assert!(reply.text_of("send-result").contains("@kindle.com, @pbsync.com"), "{address:?}");
    }
    assert!(sink.messages().is_empty(), "nothing may be sent for refused addresses");
    assert!(h.get(&format!("/k/{code}")).await.text_of("claim-count").contains("pas encore"));

    // A device address: the file goes out as an attachment with a plain ASCII name.
    let sent =
        h.post_form(&format!("/k/{code}/send"), &[("device_email", " Sacha_42@Kindle.com ")]).await;
    assert_page(&sent, StatusCode::OK, "fr", &["send-result"]);
    assert!(sent.text_of("send-result").contains("envoyé"));
    let mails = sink.messages();
    assert_eq!(mails.len(), 1);
    assert_eq!(mails[0].mail_from, "<livraison@ebk.example>");
    assert_eq!(mails[0].rcpt_to, ["<Sacha_42@kindle.com>"]);
    let message = mail_parser::MessageParser::default().parse(&mails[0].data).unwrap();
    assert_eq!(message.to().unwrap().first().unwrap().address(), Some("Sacha_42@kindle.com"));
    assert_eq!(message.from().unwrap().first().unwrap().address(), Some("livraison@ebk.example"));
    assert_eq!(message.subject(), Some("Le-Horla-suivi-de-Lui"));
    assert!(message.message_id().unwrap().ends_with("@ebk.example"));
    assert!(message.body_text(0).unwrap().contains("Le Horla, suivi de « Lui ? »"));
    let attachments: Vec<_> = message.attachments().collect();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].attachment_name(), Some("Le-Horla-suivi-de-Lui.epub"));
    let content_type = attachments[0].content_type().unwrap();
    assert_eq!((content_type.ctype(), content_type.subtype()), ("application", Some("epub+zip")));
    assert_eq!(attachments[0].contents(), epub().as_slice(), "the attachment is the stored file");

    // The sending is a claim like any other, and the address is kept nowhere.
    assert!(h.get(&format!("/k/{code}")).await.text_of("claim-count").contains("1 fois"));
    let stored = h.state.db.view(|db| serde_json::to_string(db).unwrap()).to_lowercase();
    assert!(
        !stored.contains("sacha_42") && !stored.contains("kindle.com"),
        "the device address was stored"
    );

    // At most three sendings a day per gift.
    for _ in 0..2 {
        assert_eq!(
            h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@pbsync.com")])
                .await
                .status,
            StatusCode::OK
        );
    }
    let limited =
        h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@pbsync.com")]).await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(sink.messages().len(), 3);
    h.clock.advance(86_400);
    assert_eq!(
        h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@pbsync.com")]).await.status,
        StatusCode::OK
    );
    assert_eq!(sink.messages().len(), 4);
}

#[tokio::test]
async fn send_to_device_is_off_without_smtp_and_fails_cleanly_when_the_server_is_down() {
    // Not configured: no form, and the route refuses.
    let h = Harness::new();
    let code = code_of(&h.hosted_gift(json!({})).await);
    assert!(!h.get_as(&format!("/k/{code}"), UA_PHONE).await.has_id("send-form"));
    let refused =
        h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@kindle.com")]).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);

    // Configured but unreachable (a port nobody listens on): 502, no claim counted.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let dead = format!("smtp://{}", listener.local_addr().unwrap());
    drop(listener);
    let h = Harness::with(|c| {
        c.smtp_url = Some(dead);
        c.outbound_timeout_ms = 2_000;
    });
    let code = code_of(&h.hosted_gift(json!({})).await);
    let failed = h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@kindle.com")]).await;
    assert_page(&failed, StatusCode::BAD_GATEWAY, "fr", &["send-result"]);
    assert!(failed.text_of("send-result").contains("échoué"));
    assert!(h.get(&format!("/k/{code}")).await.text_of("claim-count").contains("pas encore"));
    // Failed sendings hand their reservation back: they do not use up the daily limit.
    for _ in 0..3 {
        let again =
            h.post_form(&format!("/k/{code}/send"), &[("device_email", "a@kindle.com")]).await;
        assert_eq!(again.status, StatusCode::BAD_GATEWAY);
    }
    let counted = h
        .state
        .db
        .view(|db| db.gifts.values().map(|g| (g.claims, g.sends.len())).collect::<Vec<_>>());
    assert_eq!(counted, [(0, 0)]);
}

#[tokio::test]
async fn opds_feeds_are_well_formed_and_lead_to_the_file() {
    let h = Harness::new();
    let gift = h
        .hosted_gift(json!({
            "book": {
                "title": "Tom & <Jerry> \"l’intégrale\"", "isbn13": "9782070360024",
                "contributors": [{"name": "Hanna & Barbera"}], "publisher": "Éditions <Démo>", "language": "fr"
            }
        }))
        .await;
    let code = code_of(&gift);
    let feed_url = h.post_form(&format!("/k/{code}/download"), &[("opds", "1")]).await.location();
    assert!(feed_url.starts_with("/o/") && feed_url.ends_with("/catalog.xml"), "{feed_url}");
    let token = feed_url.split('/').nth(2).unwrap().to_owned();

    // OPDS 1.2: an Atom acquisition feed with the one publication.
    let atom = h.get(&feed_url).await;
    assert_eq!(atom.status, StatusCode::OK);
    assert_eq!(
        atom.header("content-type"),
        "application/atom+xml;profile=opds-catalog;kind=acquisition"
    );
    let text = atom.text();
    let document =
        roxmltree::Document::parse(&text).unwrap_or_else(|e| panic!("not XML ({e}):\n{text}"));
    const ATOM: &str = "http://www.w3.org/2005/Atom";
    let root = document.root_element();
    assert_eq!((root.tag_name().namespace(), root.tag_name().name()), (Some(ATOM), "feed"));
    let child = |node: roxmltree::Node<'_, '_>, name: &str| -> String {
        node.children()
            .find(|n| n.has_tag_name((ATOM, name)))
            .and_then(|n| n.text())
            .unwrap_or_default()
            .to_owned()
    };
    assert!(!child(root, "id").is_empty() && child(root, "updated").ends_with('Z'));
    let entries: Vec<_> = root.children().filter(|n| n.has_tag_name((ATOM, "entry"))).collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(child(entries[0], "title"), "Tom & <Jerry> \"l’intégrale\"");
    assert_eq!(child(entries[0], "id"), "urn:isbn:9782070360024");
    let author = entries[0].children().find(|n| n.has_tag_name((ATOM, "author"))).unwrap();
    assert_eq!(child(author, "name"), "Hanna & Barbera");
    let acquisition = entries[0]
        .children()
        .find(|n| {
            n.has_tag_name((ATOM, "link"))
                && n.attribute("rel") == Some("http://opds-spec.org/acquisition")
        })
        .unwrap();
    assert_eq!(acquisition.attribute("type"), Some("application/epub+zip"));
    let href = acquisition.attribute("href").unwrap().to_owned();
    assert_eq!(href, format!("/d/{token}/Tom-Jerry-l-integrale.epub"));
    let file = h.get(&href).await;
    assert_eq!((file.status, file.body.as_ref()), (StatusCode::OK, epub().as_slice()));

    // OPDS 2.0: the same publication as JSON.
    let feed = h.get(&format!("/o/{token}/catalog.json")).await;
    assert_eq!(feed.status, StatusCode::OK);
    assert_eq!(feed.header("content-type"), "application/opds+json");
    let feed = feed.json();
    assert_eq!(feed["metadata"]["title"], "Tom & <Jerry> \"l’intégrale\"");
    assert_eq!(feed["links"][0]["rel"], "self");
    let publications = feed["publications"].as_array().unwrap();
    assert_eq!(publications.len(), 1);
    assert_eq!(publications[0]["metadata"]["title"], "Tom & <Jerry> \"l’intégrale\"");
    assert_eq!(publications[0]["metadata"]["author"][0], "Hanna & Barbera");
    assert_eq!(publications[0]["metadata"]["identifier"], "urn:isbn:9782070360024");
    assert_eq!(publications[0]["links"][0]["rel"], "http://opds-spec.org/acquisition");
    assert_eq!(publications[0]["links"][0]["type"], "application/epub+zip");
    assert_eq!(publications[0]["links"][0]["href"], href);

    // Feeds exist only for live sessions.
    for path in [
        "/o/00000000000000000000000000000000/catalog.xml",
        "/o/x/catalog.json",
        "/o/..%2Fd/catalog.xml",
    ] {
        assert_eq!(h.get(path).await.status, StatusCode::NOT_FOUND, "{path}");
    }
    assert_eq!(
        h.get(&format!("/o/{token}/cover")).await.status,
        StatusCode::NOT_FOUND,
        "no cover was supplied"
    );
    h.clock.advance(15 * 60);
    assert_eq!(h.get(&feed_url).await.status, StatusCode::GONE);
}

#[tokio::test]
async fn a_store_licence_is_fetched_with_signed_parameters_and_served_through_a_session() {
    const LICENCE: &[u8] = br#"{"id":"lic-1","provider":"https://store.test","encryption":{}}"#;
    let store = Receiver::start(|_| (StatusCode::OK, LICENCE.to_vec())).await;
    let host = store.host();
    let h = Harness::with(|c| c.partners = vec![partner(&[host.as_str()], None)]);
    let gift = h
        .partner_gift(json!({ "type": "reseller_licence", "format": "lcpl", "url": store.url("/lcp/licence?shop=1") }), json!({}))
        .await;
    let (id, code) = (gift["id"].as_str().unwrap(), code_of(&gift));

    let page = h.get_as(&format!("/k/{code}"), UA_PHONE).await;
    assert_page(
        &page,
        StatusCode::OK,
        "fr",
        &["redeem", "redeem-form", "primary-action", "pair-start"],
    );
    assert!(!page.has_id("download"));
    assert!(store.requests().is_empty(), "nothing is fetched before the recipient asks");

    // The store is down at first: a clean 502 page, no claim counted.
    store.fail_first.store(1, std::sync::atomic::Ordering::SeqCst);
    let failed = h.post_form(&format!("/k/{code}/redeem"), &[]).await;
    assert_page(&failed, StatusCode::BAD_GATEWAY, "fr", &["error", "redeem"]);
    assert!(failed.text_of("claim-count").contains("pas encore"));

    let location = h.post_form(&format!("/k/{code}/redeem"), &[]).await.location();
    assert!(location.starts_with("/d/") && location.ends_with("/Madame-Bovary.lcpl"), "{location}");
    let licence = h.get(&location).await;
    assert_eq!(licence.status, StatusCode::OK);
    assert_eq!(licence.header("content-type"), "application/vnd.readium.lcp.license.v1.0+json");
    assert_eq!(licence.body.as_ref(), LICENCE);

    // The store was asked with the signed parameters of §4.3, on the stored URL.
    let request = store.requests().pop().unwrap();
    assert_eq!(request.method, "GET");
    let url = reqwest::Url::parse(&format!("http://store.test{}", request.path_and_query)).unwrap();
    assert_eq!(url.path(), "/lcp/licence");
    let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();
    assert_eq!(query["shop"], "1");
    assert_eq!(query["ebc_gift"], id);
    assert_eq!(query["ebc_order"], "ORD-2026-000123");
    let ts: i64 = query["ebc_ts"].parse().unwrap();
    assert_eq!(ts, h.state.now());
    assert_eq!(
        query["ebc_sig"],
        ebc_server::codes::redirect_signature(
            crate::common::PARTNER_SECRET,
            id,
            "ORD-2026-000123",
            ts
        )
    );

    // Pairing works for licences too: the e-reader gets its own session.
    let pair = h.post_form(&format!("/k/{code}/pair"), &[]).await.text_of("pair-code");
    let location = h.post_form("/e", &[("p", pair.as_str())]).await.location();
    assert_eq!(h.get(&location).await.body.as_ref(), LICENCE);
    // The file routes of a hosted gift do not apply.
    assert_eq!(h.post_form(&format!("/k/{code}/download"), &[]).await.status, StatusCode::CONFLICT);
}
