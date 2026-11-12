//! The real shelf (`ebc-shelf`, "Étagère des autrices") in the constellation, registered
//! with the server as a partner exactly like a store:
//!
//! * an author signs in with the link the mail sink received, identifies herself,
//!   uploads her own EPUB with a rights declaration and makes a gift; the recipient types
//!   the code on a Kobo, or pairs a PocketBook from a phone, and gets the file the shelf
//!   hosts, byte for byte;
//! * a DRM-protected EPUB is refused; a notice is upheld: the file is deleted, the gift's
//!   code answers `410` at the server, the hash is blocklisted and a re-upload refused;
//! * two upheld notices suspend an account;
//! * a single-author shelf (`own` mode) refuses a second account;
//! * the §4.5 endpoint refuses forged, stale and foreign link requests.

use ebc_demo_ereaders::fixtures::{self, EpubBuilder};
use ebc_demo_ereaders::{DeviceProfile, EinkBrowser};
use ebc_demo_integration::partner::{plain_client, read_reply};
use ebc_demo_integration::{SHELF_OPERATOR, SHELF_PARTNER_SECRET, World};
use ebc_model::{GiftCode, PairCode};

use crate::support::{assert_lint_clean, browser, http_get, sha256_hex};

/// Signs in at the shelf with the link from the mail sink and fills the identification.
async fn author(world: &World, email: &str) -> EinkBrowser {
    let mut author = browser(world, DeviceProfile::computer());
    author.open(&world.shelf.url("/connexion")).await.unwrap();
    assert_lint_clean(&author, "the sign-in page");
    author.type_into("email", email).unwrap();
    author.submit("signin-form").await.unwrap();
    assert!(author.page().unwrap().has("link-sent"), "{}", author.page().unwrap().text());
    let link = world.link_in_mail(email, "/connexion/").expect("the sign-in link was mailed");
    assert!(link.starts_with(&world.shelf.base), "{link}");
    author.open(&link).await.unwrap();
    author.click("confirm-signin").await.unwrap();
    let page = author.page().unwrap();
    assert!(page.has("account") && page.has("identity-missing"), "{}", page.text());
    author.click("identity-link").await.unwrap();
    author.type_into("name", "Jeanne Roy").unwrap();
    author.type_into("address", "12 rue des Lilas, 69003 Lyon").unwrap();
    author.submit("identity-form").await.unwrap();
    assert!(author.page().unwrap().has("upload-link"), "{}", author.page().unwrap().text());
    author
}

/// What the shelf answered to an upload.
struct Uploaded {
    status: u16,
    refusal: Option<String>,
    book: Option<String>,
}

/// Uploads `bytes` with a rights declaration (`multipart/form-data`, which e-ink browsers
/// cannot send: a computer's browser does it, with the session cookie and form token).
async fn upload(
    world: &World,
    author: &mut EinkBrowser,
    title: &str,
    isbn: &str,
    bytes: &[u8],
) -> Uploaded {
    author.open(&world.shelf.url("/livres/nouveau")).await.unwrap();
    let csrf = author.field_value("upload-csrf").unwrap();
    let cookie =
        author.cookies().into_iter().find(|(name, _)| name == "ebc_shelf").expect("signed in").1;
    let form = reqwest::multipart::Form::new()
        .text("csrf", csrf)
        .text("title", title.to_owned())
        .text("author", "Jeanne Roy")
        .text("isbn", isbn.to_owned())
        .text("basis", "own_work")
        .text("declared", "1")
        .part("file", reqwest::multipart::Part::bytes(bytes.to_vec()).file_name("livre.epub"));
    let response = plain_client()
        .unwrap()
        .post(world.shelf.url("/livres"))
        .header("cookie", format!("ebc_shelf={cookie}"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    let reply = read_reply(response).await.unwrap();
    Uploaded {
        status: reply.status,
        refusal: reply.header("x-ebc-refusal").map(str::to_owned),
        book: reply.header("location").and_then(|l| l.strip_prefix("/livres/")).map(str::to_owned),
    }
}

/// Makes one gift of `book` through the book page; returns `(shelf gift id, code)`.
async fn gift(world: &World, author: &mut EinkBrowser, book: &str) -> (String, GiftCode) {
    author.open(&world.shelf.url(&format!("/livres/{book}"))).await.unwrap();
    author.type_into("giver", "Jeanne").unwrap();
    author.type_into("recipient", "Sacha").unwrap();
    author.type_into("message", "Mon premier roman, pour toi.").unwrap();
    author.submit("gift-form").await.unwrap();
    let gift = world.shelf.state.snapshot().gifts.into_iter().rfind(|g| g.book_id == book).unwrap();
    let page = author.page().unwrap();
    let shown = page
        .text_of(&format!("code-{}", gift.id))
        .expect("the code is on the book page")
        .to_owned();
    (gift.id, GiftCode::parse(&shown).unwrap())
}

#[tokio::test]
async fn an_author_hosts_her_own_book_and_a_kobo_and_a_pocketbook_get_it_unchanged() {
    let world = World::start().await.unwrap();
    let mut jeanne = author(&world, "jeanne@exemple.fr").await;
    let epub = fixtures::epub("Les Vagues du soir", "Jeanne Roy");
    let uploaded = upload(&world, &mut jeanne, "Les Vagues du soir", "", &epub).await;
    assert_eq!((uploaded.status, uploaded.refusal), (303, None));
    let book = uploaded.book.unwrap();
    jeanne.open(&world.shelf.url(&format!("/livres/{book}"))).await.unwrap();
    assert!(jeanne.page().unwrap().text_of("declaration").unwrap().contains("auteur"));
    let (gift_id, code) = gift(&world, &mut jeanne, &book).await;
    let page = jeanne.page().unwrap();
    assert!(page.has(&format!("designer-{gift_id}")) && page.has(&format!("card-home-{gift_id}")));

    // The card, rendered by the server from the shelf's gift (signed partner request).
    jeanne.click(&format!("card-home-{gift_id}")).await.unwrap();
    let cookie = jeanne.cookies().into_iter().find(|(n, _)| n == "ebc_shelf").unwrap().1;
    let card = plain_client()
        .unwrap()
        .get(world.shelf.url(&format!("/cadeaux/{gift_id}/carte.pdf?profile=home-a4")))
        .header("cookie", format!("ebc_shelf={cookie}"))
        .send()
        .await
        .unwrap();
    let card = read_reply(card).await.unwrap();
    assert_eq!(card.status, 200, "{}", card.text());
    assert!(card.body.starts_with(b"%PDF-"));

    // A Kobo with no phone: the code typed on the server's home page; the server asks the
    // shelf for a link and redirects the Kobo to it.
    let mut kobo = browser(&world, DeviceProfile::kobo_clara());
    kobo.open(&world.server.url("/")).await.unwrap();
    kobo.type_into("code-input", &code.compact().to_lowercase()).unwrap();
    kobo.click("code-submit").await.unwrap();
    assert_eq!(kobo.page().unwrap().text_of("gift-title"), Some("Les Vagues du soir"));
    let entry =
        kobo.click("download").await.unwrap().download().expect("the Kobo saves the EPUB").clone();
    assert_eq!(entry.sha256, sha256_hex(&epub), "byte-identical to the upload");
    let hops: Vec<String> = kobo.exchanges().iter().map(|x| x.url.to_string()).collect();
    assert!(hops.iter().any(|u| u.starts_with(&world.shelf.url("/f/"))), "{hops:#?}");

    // A phone asks for a pair code; a PocketBook types it at /e.
    let mut phone = browser(&world, DeviceProfile::phone());
    phone.open(&ebc_model::claim_url_qr(&world.server.base, &code)).await.unwrap();
    phone.click("pair-start").await.unwrap();
    let pair = PairCode::parse(phone.page().unwrap().text_of("pair-code").unwrap()).unwrap();
    let mut pocketbook = browser(&world, DeviceProfile::pocketbook());
    pocketbook.open(&world.server.url("/e")).await.unwrap();
    pocketbook.type_into("pair-input", &pair.compact()).unwrap();
    let entry = pocketbook
        .click("pair-submit")
        .await
        .unwrap()
        .download()
        .expect("the PocketBook saves it")
        .clone();
    assert_eq!(entry.sha256, sha256_hex(&epub));

    // The shelf counted two deliveries; the server two claims.
    let shelf_gift =
        world.shelf.state.snapshot().gifts.into_iter().find(|g| g.id == gift_id).unwrap();
    assert_eq!(shelf_gift.deliveries, 2);
    assert!(
        world
            .shelf
            .state
            .snapshot()
            .deliveries
            .iter()
            .all(|d| d.client.as_deref().is_some_and(|c| !c.contains('.')))
    );
}

#[tokio::test]
async fn drm_is_refused_and_an_upheld_notice_turns_the_code_off_and_keeps_the_file_out() {
    let world = World::start().await.unwrap();
    let mut jeanne = author(&world, "jeanne@exemple.fr").await;
    let drm = EpubBuilder::new("Acheté en librairie", "X").with_adobe_rights().build();
    let refused = upload(&world, &mut jeanne, "Acheté en librairie", "", &drm).await;
    assert_eq!((refused.status, refused.refusal.as_deref()), (415, Some("drm_protected")));

    let epub = fixtures::epub("Le Roman d’un autre", "Quelqu’un");
    let book = upload(&world, &mut jeanne, "Le Roman d’un autre", "978-2-07-036002-4", &epub)
        .await
        .book
        .unwrap();
    let (_, code) = gift(&world, &mut jeanne, &book).await;
    let claim = world.server.url(&format!("/k/{}", code.compact()));
    assert_eq!(http_get(&claim, "Mozilla/5.0").await.status, 200);

    // Anyone may notify, without an account; the code on the card locates the book.
    let mut notifier = browser(&world, DeviceProfile::computer());
    notifier.open(&world.shelf.url("/signaler")).await.unwrap();
    assert_lint_clean(&notifier, "the notice form");
    notifier.type_into("location", &format!("Le code de la carte : {}", code.grouped())).unwrap();
    notifier
        .type_into("explanation", "Ce roman est le mien ; il est publié chez mon éditeur.")
        .unwrap();
    notifier.type_into("name", "Autre Autrice").unwrap();
    notifier.type_into("email", "ayant-droit@exemple.fr").unwrap();
    notifier.check("good_faith").unwrap();
    notifier.submit("notice-form").await.unwrap();
    let notice = notifier.page().unwrap().text_of("notice-id").expect("a reference").to_owned();
    assert!(
        world.inbox("ayant-droit@exemple.fr").iter().any(|m| m.subject.contains(&notice)),
        "acknowledged"
    );

    // The operator upholds it.
    let mut operator = author(&world, SHELF_OPERATOR).await;
    operator.open(&world.shelf.url(&format!("/moderation/signalements/{notice}"))).await.unwrap();
    assert!(operator.page().unwrap().has("resolved"));
    operator.check("decision-uphold").unwrap();
    operator.type_into("statement", "Œuvre d’un tiers sous contrat d’édition.").unwrap();
    operator.submit("decision-form").await.unwrap();
    assert_eq!(operator.page().unwrap().text_of("decision").map(|_| ()), Some(()));

    // The code now answers 410 at the server; nothing is left on the shelf.
    assert_eq!(http_get(&claim, "Mozilla/5.0").await.status, 410);
    let db = world.shelf.state.snapshot();
    assert!(db.books.iter().all(|b| !b.is_live()));
    assert_eq!(world.shelf.state.live_links(), 0);
    let sor = world.inbox("jeanne@exemple.fr").into_iter().last().unwrap();
    assert!(sor.text.contains("art. 17") && sor.text.contains("art. 21"), "{}", sor.text);
    // Re-upload: refused by hash, and by ISBN with another file.
    let again = upload(&world, &mut jeanne, "Le Roman d’un autre", "", &epub).await;
    assert_eq!(again.refusal.as_deref(), Some("blocked_file"));
    let other_file = fixtures::epub("Le Roman d’un autre (2)", "Quelqu’un");
    let again =
        upload(&world, &mut jeanne, "Le Roman d’un autre", "9782070360024", &other_file).await;
    assert_eq!(again.refusal.as_deref(), Some("blocked_isbn"));
}

#[tokio::test]
async fn two_upheld_notices_suspend_the_account() {
    let world = World::start().await.unwrap();
    let mut author = author(&world, "recidive@exemple.fr").await;
    let mut operator = self::author(&world, SHELF_OPERATOR).await;
    for n in 1..=2 {
        let title = format!("Copie {n}");
        let book = upload(&world, &mut author, &title, "", &fixtures::epub(&title, "X"))
            .await
            .book
            .unwrap();
        let mut notifier = browser(&world, DeviceProfile::computer());
        notifier.open(&world.shelf.url("/signaler")).await.unwrap();
        notifier.type_into("location", &world.shelf.url(&format!("/livres/{book}"))).unwrap();
        notifier.type_into("explanation", "Contrefaçon.").unwrap();
        notifier.type_into("name", "N").unwrap();
        notifier.type_into("email", "n@exemple.fr").unwrap();
        notifier.check("good_faith").unwrap();
        notifier.submit("notice-form").await.unwrap();
        let notice = notifier.page().unwrap().text_of("notice-id").unwrap().to_owned();
        operator
            .open(&world.shelf.url(&format!("/moderation/signalements/{notice}")))
            .await
            .unwrap();
        operator.check("decision-uphold").unwrap();
        operator.type_into("statement", "Contrefaçon établie.").unwrap();
        operator.submit("decision-form").await.unwrap();
    }
    author.open(&world.shelf.url("/compte")).await.unwrap();
    let page = author.page().unwrap();
    assert!(page.has("suspended") && !page.has("upload-link"), "{}", page.text());
    author.open(&world.shelf.url("/livres/nouveau")).await.unwrap();
    let page = author.page().unwrap();
    assert!(page.has("account") && !page.has("upload-form"), "no upload form any more");
    assert!(
        world.inbox("recidive@exemple.fr").into_iter().last().unwrap().text.contains("suspendu")
    );
}

#[tokio::test]
async fn a_single_author_shelf_refuses_a_second_account() {
    let world = World::builder()
        .shelf_config(|config| config.mode = ebc_shelf::Mode::Own)
        .start()
        .await
        .unwrap();
    let mut stranger = browser(&world, DeviceProfile::computer());
    stranger.open(&world.shelf.url("/connexion")).await.unwrap();
    stranger.type_into("email", "jeanne@exemple.fr").unwrap();
    stranger.submit("signin-form").await.unwrap();
    let page = stranger.page().unwrap();
    assert_eq!(page.status(), 403);
    assert!(page.has("error") && !page.has("link-sent"));
    assert!(world.inbox("jeanne@exemple.fr").is_empty(), "no link sent");
    // The owner (the operator address) signs in and hosts her books.
    let owner = author(&world, SHELF_OPERATOR).await;
    assert!(owner.page().unwrap().has("upload-link"));
}

#[tokio::test]
async fn the_link_endpoint_answers_only_the_servers_fresh_signature() {
    let world = World::start().await.unwrap();
    let mut jeanne = author(&world, "jeanne@exemple.fr").await;
    let epub = fixtures::epub("Signé", "Jeanne Roy");
    let book = upload(&world, &mut jeanne, "Signé", "", &epub).await.book.unwrap();
    let (gift_id, _) = gift(&world, &mut jeanne, &book).await;
    let core_id = world
        .shelf
        .state
        .snapshot()
        .gifts
        .into_iter()
        .find(|g| g.id == gift_id)
        .unwrap()
        .core_id
        .unwrap();
    let now = world.clock.now();
    let ask = |secret: &str, gift: &str, order: &str, ts: i64| {
        let sig = ebc_shelf::crypto::link_request_signature(secret, gift, order, ts);
        world.shelf.url(&format!(
            "/api/ebc/link?ebc_gift={gift}&ebc_order={order}&ebc_ts={ts}&ebc_sig={sig}"
        ))
    };
    let status = |url: String| async move { http_get(&url, "ebc-test").await.status };
    assert_eq!(status(ask("not-the-secret-0123456789", &core_id, &gift_id, now)).await, 403);
    assert_eq!(status(ask(SHELF_PARTNER_SECRET, &core_id, &gift_id, now - 901)).await, 401);
    assert_eq!(status(ask(SHELF_PARTNER_SECRET, &core_id, "c_0000000000000000", now)).await, 404);
    assert_eq!(status(world.shelf.url(&format!("/api/ebc/link?ebc_gift={core_id}"))).await, 400);
    // The genuine request (what the server sends on a claim) gets a link on the shelf's host.
    let answer = http_get(&ask(SHELF_PARTNER_SECRET, &core_id, &gift_id, now), "ebc-test").await;
    assert_eq!(answer.status, 200, "{}", answer.text());
    let url = answer.json()["download_url"].as_str().unwrap().to_owned();
    assert!(url.starts_with(&world.shelf.url("/f/")));
    let file = http_get(&url, "ebc-test").await;
    assert_eq!(file.status, 200);
    assert_eq!(sha256_hex(&file.body), sha256_hex(&epub));
    assert!(file.header("content-disposition").unwrap_or("").starts_with("attachment;"));
}
