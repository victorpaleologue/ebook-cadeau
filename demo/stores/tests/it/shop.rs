//! Storefront basics over real HTTP: catalogue, covers, accounts, a purchase without the
//! gift option in each store, downloads, and the e-reader device API.

use crate::common::{Browser, World, open_epub, plain_client, text_of};
use axum::http::StatusCode;
use ebc_demo_stores::StoreKind;
use ebc_demo_stores::catalogue;
use serde_json::{Value, json};

const BANNER: &str = "Démo — boutique fictive, sans lien avec une enseigne réelle";

#[tokio::test]
async fn every_page_is_french_carries_the_banner_and_forbids_scripts() {
    let world = World::new().await;
    for kind in StoreKind::ALL {
        let store = world.store(kind).await;
        let browser = store.browser();
        browser.register("lectrice@example.org", "motdepasse").await;
        browser.add_to_cart("candide").await;
        for path in [
            "/",
            "/livre/candide",
            "/panier",
            "/commande",
            "/commandes",
            "/bibliotheque",
            "/cadeau/activer",
            "/livre/inconnu",
            "/n-existe-pas",
        ] {
            let page = browser.get(path).await;
            assert_eq!(page.text("demo-banner").as_deref(), Some(BANNER), "{kind:?} {path}");
            assert_eq!(page.count("html[lang=\"fr\"]"), 1, "{kind:?} {path}");
            assert_eq!(page.count("script"), 0, "{kind:?} {path}: pages work without JavaScript");
            assert!(page.header("content-type").unwrap().starts_with("text/html"));
            let csp = page.header("content-security-policy").unwrap();
            assert!(csp.contains("default-src 'none'") && !csp.contains("script-src"));
            assert_eq!(page.header("x-content-type-options"), Some("nosniff"));
            assert_eq!(page.header("cache-control"), Some("no-store"));
        }
        // The fictional name is on the page; the real-world archetypes are not.
        let home = browser.get("/").await.html().to_lowercase();
        assert!(home.contains(&kind.display_name().to_lowercase()));
        for real in ["fnac", "7switch", "kobo", "amazon", "kindle"] {
            assert!(!home.contains(real), "{kind:?}: the page names {real}");
        }
    }
}

#[tokio::test]
async fn catalogue_and_product_pages_show_the_demo_editions() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    let home = browser.get("/").await;
    assert_eq!(home.status, StatusCode::OK);
    assert_eq!(home.count("#catalogue li"), 6);
    for book in catalogue::books() {
        assert_eq!(
            home.attr(&format!("book-{}", book.id), "href"),
            Some(format!("/livre/{}", book.id))
        );
        let page = browser.get(&format!("/livre/{}", book.id)).await;
        assert_eq!(page.status, StatusCode::OK);
        assert_eq!(page.text("book-title"), Some(book.full_title()));
        assert!(page.text("book-author").unwrap().starts_with(book.author));
        assert_eq!(page.text("book-isbn").as_deref(), Some(book.isbn13));
        assert!(page.html().contains("ISBN de démonstration"));
        assert_eq!(page.text("book-price"), Some(catalogue::format_price(book.price_cents)));
        assert!(page.has("add-to-cart"));
        ebc_isbn::Isbn13::parse(book.isbn13).expect("a syntactically valid ISBN-13");
    }
    let missing = browser.get("/livre/les-trois-mousquetaires").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert!(missing.has("not-found"));
}

#[tokio::test]
async fn covers_are_served_as_svg_and_as_print_ready_png() {
    let world = World::new().await;
    let store = world.store(StoreKind::GrandeEnseigne).await;
    let browser = store.browser();
    for book in catalogue::books() {
        let svg = browser.get(&format!("/covers/{}.svg", book.id)).await;
        assert_eq!(svg.status, StatusCode::OK);
        assert_eq!(svg.header("content-type"), Some("image/svg+xml"));
        assert!(svg.html().starts_with("<svg"));
        assert!(svg.header("cache-control").unwrap().contains("max-age"));
    }
    let png = browser.get("/covers/madame-bovary.png").await;
    assert_eq!(png.status, StatusCode::OK);
    assert_eq!(png.header("content-type"), Some("image/png"));
    let decoder = png::Decoder::new(std::io::Cursor::new(&png.body));
    let reader = decoder.read_info().unwrap();
    let info = reader.info();
    assert!(info.width >= 1300 && info.height >= 2100, "{}x{}", info.width, info.height);
    assert_eq!(info.color_type, png::ColorType::Rgb, "opaque, ready for print");

    for bad in [
        "/covers/candide.gif",
        "/covers/candide",
        "/covers/nope.png",
        "/covers/..%2F..%2Fetc%2Fpasswd.svg",
        "/covers/.svg",
    ] {
        assert_eq!(browser.get(bad).await.status, StatusCode::NOT_FOUND, "{bad}");
    }
}

#[tokio::test]
async fn septante_purchase_without_gift_ends_in_a_watermarked_download() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();

    // catalogue → product → cart
    let cart = browser.add_to_cart("candide").await;
    assert_eq!(cart.path, "/panier");
    assert!(cart.has("line-candide"));
    assert_eq!(cart.text("cart-total").as_deref(), Some("2,99\u{a0}€"));
    assert_eq!(cart.text("cart-count").as_deref(), Some("1"));
    browser.add_to_cart("le-horla").await;
    browser.add_to_cart("le-horla").await;
    let cart = browser.post("/panier/retirer", &[("book", "candide")]).await;
    assert!(!cart.has("line-candide") && cart.has("line-le-horla"));
    assert_eq!(cart.text("cart-total").as_deref(), Some("0,99\u{a0}€"), "an eBook is bought once");
    browser.add_to_cart("candide").await;

    // checkout needs an account, then comes back
    let checkout = browser.get("/commande").await;
    assert_eq!(checkout.hops[0].1, "/compte/connexion?next=%2Fcommande");
    assert!(checkout.has("login-form") && checkout.has("register-form"));
    let checkout = browser
        .post(
            "/compte/inscription",
            &[("email", "Camille@Example.org"), ("password", "motdepasse"), ("next", "/commande")],
        )
        .await;
    assert_eq!(checkout.path, "/commande");
    assert_eq!(checkout.text("account-email").as_deref(), Some("camille@example.org"));
    assert!(checkout.has("gift-option"), "the gift option is offered, unticked");
    assert_eq!(checkout.attr("gift-option", "checked"), None);
    assert!(checkout.has("pay"));

    // pay without the gift option
    let order = browser.post("/commande/payer", &[]).await;
    assert_eq!(order.status, StatusCode::OK);
    let reference = order.order_ref();
    assert_eq!(reference, "SEP-2026-000001");
    assert_eq!(order.path, format!("/commande/{reference}"));
    assert_eq!(order.text("order-total").as_deref(), Some("3,98\u{a0}€"));
    assert!(order.has("purchase-done") && order.has("library-link"));
    assert!(!order.has("gift-status") && !order.has("card-pdf"));
    assert!(world.ebc.requests().is_empty(), "no gift, no call to the partner API");

    // the cart is empty, the library is not
    assert!(browser.get("/panier").await.has("cart-empty"));
    let library = browser.get("/bibliotheque").await;
    assert!(library.has("owned-candide") && library.has("owned-le-horla"));
    assert_eq!(
        library.attr("download-candide", "href").as_deref(),
        Some("/bibliotheque/candide.epub")
    );

    // the download is a valid EPUB stamped for the buyer
    let download = browser.get("/bibliotheque/candide.epub").await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.header("content-type"), Some("application/epub+zip"));
    assert_eq!(
        download.header("content-disposition"),
        Some("attachment; filename=\"candide.epub\"")
    );
    let (names, files) = open_epub(&download.body);
    assert!(names.contains(&"OEBPS/watermark.xhtml".to_owned()));
    let watermark = text_of(&files, "OEBPS/watermark.xhtml");
    assert!(watermark.contains("Exemplaire de camille@example.org"));
    assert!(watermark.contains(&reference));
    let opf = text_of(&files, "OEBPS/package.opf");
    assert!(opf.contains("<dc:identifier id=\"pub-id\">urn:isbn:9786990000015</dc:identifier>"));
    assert!(opf.contains("<dc:title>Candide</dc:title>"));
    assert!(opf.contains(">Voltaire</dc:creator>"));
    assert!(opf.contains("<dc:language>fr</dc:language>"));
    assert!(opf.contains(&format!("<meta property=\"demo:order\">{reference}</meta>")));
    assert!(text_of(&files, "OEBPS/chapter-1.xhtml").contains("Il y avait en Westphalie"));
    assert!(text_of(&files, "OEBPS/nav.xhtml").contains("epub:type=\"toc\""));

    // a book that was not bought cannot be downloaded
    assert_eq!(browser.get("/bibliotheque/madame-bovary.epub").await.status, StatusCode::NOT_FOUND);
    assert_eq!(browser.get("/bibliotheque/candide.pdf").await.status, StatusCode::NOT_FOUND);

    // the product page now knows the title is owned; buying it again adds nothing
    assert!(browser.get("/livre/candide").await.has("already-owned"));
    browser.add_to_cart("candide").await;
    browser.post("/commande/payer", &[]).await;
    assert_eq!(store.snapshot()["accounts"][0]["library"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn grande_enseigne_purchase_lands_in_the_account_and_on_the_e_reader() {
    let world = World::new().await;
    let store = world.store(StoreKind::GrandeEnseigne).await;
    let browser = store.browser();
    let order = browser.buy("lecteur@example.org", &["les-miserables-1"], &[]).await;
    assert_eq!(order.order_ref(), "GE-2026-000001");
    assert!(order.text("purchase-done").unwrap().contains("liseuse"));

    // no file in the browser
    let library = browser.get("/bibliotheque").await;
    assert!(library.has("owned-les-miserables-1") && library.has("sync-note"));
    assert!(!library.has("download-les-miserables-1"));
    assert_eq!(library.count("a[href$=\".epub\"]"), 0);
    let attempt = browser.get("/bibliotheque/les-miserables-1.epub").await;
    assert_eq!(attempt.status, StatusCode::NOT_FOUND);
    assert!(attempt.has("no-download"));

    // the e-reader links to the account and syncs
    let http = plain_client();
    let link = |body: Value| {
        let http = http.clone();
        let url = format!("{}/api/device/link", store.base);
        async move { http.post(url).json(&body).send().await.unwrap() }
    };
    let wrong = link(json!({"email": "lecteur@example.org", "password": "mauvais"})).await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(wrong.json::<Value>().await.unwrap()["error"]["code"], "invalid_credentials");
    let unknown = link(json!({"email": "personne@example.org", "password": "motdepasse"})).await;
    assert_eq!(unknown.status(), StatusCode::UNAUTHORIZED);
    for garbage in [json!({"email": "lecteur@example.org"}), json!([]), json!("x")] {
        assert_eq!(link(garbage).await.status(), StatusCode::BAD_REQUEST);
    }
    let linked = link(json!({"email": "Lecteur@Example.org", "password": "motdepasse"})).await;
    assert_eq!(linked.status(), StatusCode::OK);
    let token = linked.json::<Value>().await.unwrap()["device_token"].as_str().unwrap().to_owned();
    assert_eq!(token.len(), 64);

    let library_url = format!("{}/api/device/library", store.base);
    let anonymous = http.get(&library_url).send().await.unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous.headers()["www-authenticate"], "Bearer");
    let forged = http.get(&library_url).bearer_auth("0".repeat(64)).send().await.unwrap();
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);
    let books: Value =
        http.get(&library_url).bearer_auth(&token).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        books,
        json!([{
            "id": "les-miserables-1",
            "title": "Les Misérables — Tome 1 : Fantine",
            "author": "Victor Hugo",
            "format": "epub",
            "download_url": "/api/device/books/les-miserables-1.epub",
        }])
    );

    let download_url = format!("{}{}", store.base, books[0]["download_url"].as_str().unwrap());
    assert_eq!(http.get(&download_url).send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    let epub = http.get(&download_url).bearer_auth(&token).send().await.unwrap();
    assert_eq!(epub.status(), StatusCode::OK);
    assert_eq!(epub.headers()["content-type"], "application/epub+zip");
    let (names, files) = open_epub(&epub.bytes().await.unwrap());
    assert!(!names.contains(&"OEBPS/watermark.xhtml".to_owned()));
    assert!(text_of(&files, "OEBPS/chapter-1.xhtml").contains("évêque de Digne"));
    let other = format!("{}/api/device/books/candide.epub", store.base);
    let not_owned = http.get(&other).bearer_auth(&token).send().await.unwrap();
    assert_eq!(not_owned.status(), StatusCode::NOT_FOUND);
    assert_eq!(not_owned.json::<Value>().await.unwrap()["error"]["code"], "not_in_library");
}

#[tokio::test]
async fn the_device_api_exists_only_in_the_ecosystem_store() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let response = plain_client()
        .post(format!("{}/api/device/link", store.base))
        .json(&json!({"email": "a@b.example", "password": "motdepasse"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn accounts_sign_in_out_and_reject_bad_input() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();

    for (email, password, status) in [
        ("pas-une-adresse", "motdepasse", StatusCode::UNPROCESSABLE_ENTITY),
        ("a@b.example", "court", StatusCode::UNPROCESSABLE_ENTITY),
        ("<script>@b.example\r\nX: y", "motdepasse", StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        let page = browser.register(email, password).await;
        assert_eq!(page.status, status, "{email}");
        assert!(page.has("auth-error"));
        assert_eq!(page.count("script"), 0);
        assert!(!page.has("account-email"));
    }
    let created = browser.register("a@b.example", "motdepasse").await;
    assert_eq!(created.text("account-email").as_deref(), Some("a@b.example"));
    let duplicate = store.browser().register("A@B.example", "autremotdepasse").await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);

    let out = browser.post("/compte/deconnexion", &[]).await;
    assert!(out.has("nav-login") && !out.has("account-email"));
    let wrong = browser.login("a@b.example", "mauvais-mot-de-passe").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong.attr("login-email", "value").as_deref(), Some("a@b.example"));
    let nobody = browser.login("personne@b.example", "motdepasse").await;
    assert_eq!(nobody.status, StatusCode::UNAUTHORIZED);
    assert_eq!(nobody.text("auth-error"), wrong.text("auth-error"), "no account enumeration");
    let back = browser.login("a@b.example", "motdepasse").await;
    assert_eq!(back.text("account-email").as_deref(), Some("a@b.example"));

    // Pages that need an account send anonymous visitors to the sign-in page and back.
    let anonymous = store.browser();
    for path in ["/commande", "/commandes", "/bibliotheque", "/commande/SEP-2026-000001"] {
        let page = anonymous.get_once(path).await;
        assert_eq!(page.status, StatusCode::SEE_OTHER, "{path}");
        assert!(
            page.header("location").unwrap().starts_with("/compte/connexion?next=%2F"),
            "{path}"
        );
    }
}

#[tokio::test]
async fn the_next_parameter_never_redirects_off_site() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let hostile =
        ["https://evil.example/", "//evil.example", "/\\evil.example", "javascript:alert(1)"];
    for (index, next) in hostile.into_iter().enumerate() {
        // After registration…
        let browser = store.browser();
        let email = format!("visiteur{index}@example.org");
        let page = browser
            .post(
                "/compte/inscription",
                &[("email", &email), ("password", "motdepasse"), ("next", next)],
            )
            .await;
        // (`Browser::follow` also asserts that every hop stays on the store's origin.)
        assert_eq!(page.hops, [(StatusCode::SEE_OTHER, "/".to_owned())], "{next}");
        // …after sign-in…
        let browser = store.browser();
        let page = browser
            .post(
                "/compte/connexion",
                &[("email", &email), ("password", "motdepasse"), ("next", next)],
            )
            .await;
        assert_eq!(page.hops, [(StatusCode::SEE_OTHER, "/".to_owned())], "{next}");
        // …and after sign-out.
        let page = browser.post("/compte/deconnexion", &[("next", next)]).await;
        assert_eq!(page.hops, [(StatusCode::SEE_OTHER, "/".to_owned())], "{next}");
    }
    // The sign-in page itself never echoes an off-site destination into its forms.
    let page = store.browser().get_once("/compte/connexion?next=https://evil.example/").await;
    assert_eq!(page.attr("login-form", "action").as_deref(), Some("/compte/connexion"));
    assert_eq!(page.count("input[name=\"next\"][value=\"/\"]"), 2);
    assert!(!page.html().contains("evil.example"));
}

#[tokio::test]
async fn session_cookies_are_scoped_and_rotated_at_sign_in() {
    let world = World::new().await;
    let septante = world.store(StoreKind::Septante).await;
    let grande = world.store(StoreKind::GrandeEnseigne).await;
    let http = plain_client();

    let response = http
        .post(format!("{}/panier/ajouter", septante.base))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("book=candide")
        .send()
        .await
        .unwrap();
    let cookie = response.headers()["set-cookie"].to_str().unwrap().to_owned();
    assert!(cookie.starts_with("septante_sid="));
    for attribute in ["HttpOnly", "SameSite=Lax", "Path=/"] {
        assert!(cookie.contains(attribute), "{cookie}");
    }
    let anonymous_sid = cookie.split(';').next().unwrap().to_owned();

    // Signing in issues a new identifier and the old one stops working.
    septante.state.register("a@b.example", "motdepasse").unwrap();
    let login = http
        .post(format!("{}/compte/connexion", septante.base))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("cookie", &anonymous_sid)
        .body("email=a%40b.example&password=motdepasse")
        .send()
        .await
        .unwrap();
    let signed_in_sid =
        login.headers()["set-cookie"].to_str().unwrap().split(';').next().unwrap().to_owned();
    assert_ne!(signed_in_sid, anonymous_sid);
    let cart_with = |sid: String| {
        let http = http.clone();
        let url = format!("{}/panier", septante.base);
        async move { http.get(url).header("cookie", sid).send().await.unwrap().text().await.unwrap() }
    };
    assert!(
        cart_with(signed_in_sid.clone()).await.contains("id=\"line-candide\""),
        "the cart follows"
    );
    assert!(cart_with(anonymous_sid).await.contains("id=\"cart-empty\""));

    // A client-chosen identifier is never adopted (no session fixation).
    let fixed = "septante_sid=00000000000000000000000000000000".to_owned();
    assert!(cart_with(fixed.clone()).await.contains("id=\"cart-empty\""));
    let response = http
        .post(format!("{}/panier/ajouter", septante.base))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("cookie", &fixed)
        .body("book=candide")
        .send()
        .await
        .unwrap();
    let issued = response.headers()["set-cookie"].to_str().unwrap();
    assert!(!issued.starts_with(&fixed));

    // The other store uses another cookie name and does not know Septante's sessions.
    let other = http
        .get(format!("{}/panier", grande.base))
        .header("cookie", signed_in_sid.replace("septante_sid", "ge_sid"))
        .send()
        .await
        .unwrap();
    assert!(other.text().await.unwrap().contains("id=\"cart-empty\""));
    assert_eq!(grande.snapshot()["accounts"], json!([]));
}

#[tokio::test]
async fn hostile_form_input_is_harmless() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let browser = store.browser();
    assert_eq!(
        browser.add_to_cart("<script>alert(1)</script>").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(browser.add_to_cart("../../etc/passwd").await.status, StatusCode::NOT_FOUND);
    assert_eq!(browser.add_to_cart("").await.status, StatusCode::NOT_FOUND);
    // Removing something that is not there, or with no session at all, is a no-op.
    assert_eq!(browser.post("/panier/retirer", &[("book", "\u{0}")]).await.status, StatusCode::OK);
    // A body that is not a form at all.
    let response = plain_client()
        .post(format!("{}/panier/ajouter", store.base))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![0xff, 0xfe, 0x00, 0x80])
        .send()
        .await
        .unwrap();
    assert!(response.status().is_client_error());
    // An oversized body is refused without being buffered whole.
    let response = plain_client()
        .post(format!("{}/compte/inscription", store.base))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!("email=a%40b.example&password={}", "x".repeat(3 << 20)))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(store.snapshot()["accounts"], json!([]));
}

#[tokio::test]
async fn orders_are_private_to_their_account() {
    let world = World::new().await;
    let store = world.store(StoreKind::Septante).await;
    let owner = store.browser();
    let order = owner.buy("owner@example.org", &["candide"], &[]).await;
    let reference = order.order_ref();

    let intruder = store.browser();
    intruder.register("intrus@example.org", "motdepasse").await;
    let page = intruder.get(&format!("/commande/{reference}")).await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    assert!(!page.html().contains("owner@example.org"));
    assert_eq!(
        intruder.get(&format!("/commande/{reference}/carte-1.pdf")).await.status,
        StatusCode::NOT_FOUND
    );
    for action in ["relancer", "annuler"] {
        let response = intruder.post(&format!("/commande/{reference}/{action}"), &[]).await;
        assert_eq!(response.status, StatusCode::NOT_FOUND, "{action}");
    }
    assert!(intruder.get("/commandes").await.has("orders-empty"));
    assert_eq!(owner.get("/commandes").await.count("#orders tr"), 1);
    assert_eq!(owner.get(&format!("/commande/{reference}")).await.status, StatusCode::OK);
}

#[tokio::test]
async fn test_hooks_expose_state_only_when_enabled() {
    let world = World::new().await;
    let open = world.store(StoreKind::Septante).await;
    Browser::new(&open.base).buy("a@b.example", &["candide"], &[]).await;
    let response = plain_client().get(format!("{}/__test/state", open.base)).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let state: Value = response.json().await.unwrap();
    assert_eq!(state["store"], "septante");
    assert_eq!(state["orders"][0]["reference"], "SEP-2026-000001");
    assert_eq!(state["orders"][0]["lines"][0]["book_id"], "candide");
    assert_eq!(state["accounts"][0]["email"], "a@b.example");
    assert_eq!(state["gifts"], json!([]));
    let dump = state.to_string();
    assert!(!dump.contains("password") && !dump.contains("salt") && !dump.contains("motdepasse"));

    let closed = world.store_with(StoreKind::Septante, |c| c.with_test_hooks(false)).await;
    let response =
        plain_client().get(format!("{}/__test/state", closed.base)).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
