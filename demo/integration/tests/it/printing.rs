//! (g) Printing. The server renders `card.pdf` with the card composer (`ebc-card`), and a
//! "printed and posted" order sends that PDF to the printer, which preflights it:
//!
//! * the full chain — checkout → the store fetches `card.pdf` → the printer accepts it →
//!   its status changes come back to the order page;
//! * the printer's verdicts on a card-shaped PDF built here with `ebc-layout` (page boxes,
//!   bleed, `EBC.*` geometry keys) and `ebc-pdf` (PDF/X-4): accepted with bleed, refused
//!   without, both when a store's printer client submits it and when the server forwards
//!   a creator's print order;
//! * a server without a card renderer (`501`): the stores fall back to the link and the
//!   code, and order nothing from the printer.

use std::sync::Arc;
use std::time::Duration;

use ebc_demo_ereaders::DeviceProfile;
use ebc_demo_integration::shop::{self, GiftOrder, PostTo, Receipt};
use ebc_demo_integration::{T0, World};
use ebc_demo_stores::outbound::{PostalAddress, PrintOrderRequest, PrintSubmission, PrinterClient};
use ebc_layout::{CardFormat, Face, PrintProfile, info_entries, new_page_for};
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Color, DocInfo, Document, Item, OutputIntent, PdfStandard};
use serde_json::json;

use crate::support::{browser, creator_call, gift_id, plain_epub, sha256_hex};

/// A two-page "poche" card (2 × 110 × 178 mm covers, 10 mm spine) as flat CMYK panels,
/// written as PDF/X-4 with `bleed_mm` of bleed on every side. A fixture, not artwork: no
/// text, no cover, no QR code.
fn card_pdf(bleed_mm: f64) -> Vec<u8> {
    let format = CardFormat::poche();
    let mut profile = PrintProfile::upload_eu(); // trim + bleed, no marks
    profile.bleed = bleed_mm;
    let profile_bytes: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/icc/CGATS001Compat-v2-micro.icc"
    ));
    let mut document = Document {
        info: DocInfo {
            title: "Carte cadeau — Candide (fixture de test)".to_owned(),
            author: Some("eBook Cadeau (tests d'intégration)".to_owned()),
            created_unix: Some(T0),
            language: Some("fr-FR".to_owned()),
            custom: info_entries(&format, &profile),
            ..DocInfo::default()
        },
        standard: PdfStandard::X4,
        output_intent: Some(OutputIntent {
            condition_identifier: "CGATS TR 001".to_owned(),
            info: "SWOP (CGATS TR 001) compatible".to_owned(),
            registry_name: "http://www.color.org".to_owned(),
            icc_profile: Some(Arc::from(profile_bytes)),
            components: 4,
        }),
        ..Document::default()
    };
    for face in [Face::Outside, Face::Inside] {
        let mut page = new_page_for(face, &format, &profile).unwrap();
        // A ground that runs to the edge of the bleed, and the spine in a darker tint.
        page.items.push(Item::filled_rect(page.bleed_box(), Color::cmyk(0.04, 0.02, 0.12, 0.0)));
        let spine = if face == Face::Outside { format.spine() } else { format.inside_spine() };
        page.items.push(Item::filled_rect(spine, Color::cmyk(0.0, 0.0, 0.0, 0.85)));
        document.pages.push(page);
    }
    write_pdf(&document, &WriteOptions::default()).unwrap()
}

fn address() -> PostalAddress {
    PostalAddress {
        name: "Sacha Martin".to_owned(),
        line1: "12 rue des Lilas".to_owned(),
        postal_code: "69003".to_owned(),
        city: "Lyon".to_owned(),
        country: "FR".to_owned(),
    }
}

#[tokio::test]
async fn the_printer_accepts_a_card_with_bleed_and_rejects_one_without() {
    let world = World::start().await.unwrap();
    let good = card_pdf(3.0);
    let bare = card_pdf(0.0);
    assert!(good.starts_with(b"%PDF-") && bare.starts_with(b"%PDF-"));

    // --- 1. submitted by a store's own printer client (the code `demo/stores` runs) ---
    let printer = PrinterClient::new(&world.printer.base, Duration::from_secs(10)).unwrap();
    let products = printer.products().await.unwrap();
    assert!(products.iter().any(|p| p.id == "card-poche" && p.trim_mm == [230.0, 178.0]));
    let order = |reference: &str| PrintOrderRequest {
        product: "card-poche".to_owned(),
        quantity: 1,
        address: address(),
        reference: reference.to_owned(),
        callback_url: None,
    };
    let accepted = printer.create_order(&order("IT-0001"), good.clone()).await.unwrap();
    let PrintSubmission::Accepted(status) = accepted else {
        panic!("the printer refused a card with 3 mm bleed: {accepted:#?}");
    };
    assert_eq!(status.status, "accepted");
    assert_eq!(status.price_cents, Some(590));
    let rejected = printer.create_order(&order("IT-0002"), bare.clone()).await.unwrap();
    let PrintSubmission::Rejected { findings } = rejected else {
        panic!("the printer accepted a card without bleed: {rejected:#?}");
    };
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.to_ascii_lowercase().contains("bleed")), "{findings:#?}");
    let orders = world.printer.state.orders();
    assert_eq!(orders.len(), 1, "only the accepted card became an order");
    assert_eq!(
        (orders[0].reference.as_str(), orders[0].pdf_sha256.as_str()),
        ("IT-0001", sha256_hex(&good).as_str())
    );

    // --- 2. forwarded by the server for a standalone creator (docs/api.md §3) ---
    let upload = creator_call(
        &world,
        "POST",
        "/api/v1/uploads",
        None,
        &[("x-rights-basis", "public_domain"), ("x-file-name", "candide.epub")],
        plain_epub("candide"),
    )
    .await;
    assert_eq!(upload.status, 201, "{}", upload.text());
    let gift = creator_call(
        &world,
        "POST",
        "/api/v1/gifts",
        None,
        &[("content-type", "application/json")],
        json!({
            "book": { "title": "Candide", "contributors": [{"name": "Voltaire"}] },
            "giver_name": "Camille", "recipient_name": "Sacha", "locale": "fr",
            "delivery": { "type": "hosted_file", "upload_id": upload.json()["upload_id"] },
        })
        .to_string()
        .into_bytes(),
    )
    .await;
    assert_eq!(gift.status, 201, "{}", gift.text());
    let gift = gift.json();
    let (id, token) = (gift["id"].as_str().unwrap(), gift["manage_token"].as_str().unwrap());
    let card_path = format!("/api/v1/gifts/{id}/card.pdf");
    let orders_path = format!("/api/v1/gifts/{id}/print-orders");
    let print = |pdf: &[u8]| {
        json!({
            "product": "card-poche", "pdf_sha256": sha256_hex(pdf),
            "address": { "name": "Sacha Martin", "line1": "12 rue des Lilas", "postal_code": "69003", "city": "Lyon", "country": "FR" },
        })
        .to_string()
        .into_bytes()
    };
    let json_header = [("content-type", "application/json")];

    // Without the manage token nothing is uploaded or ordered.
    let anonymous = creator_call(&world, "PUT", &card_path, None, &[], bare.clone()).await;
    assert_eq!(anonymous.status, 401, "{}", anonymous.text());

    // The card without bleed: the printer's findings come back through the server.
    let put = creator_call(&world, "PUT", &card_path, Some(token), &[], bare.clone()).await;
    assert_eq!(put.status, 200, "{}", put.text());
    let refused =
        creator_call(&world, "POST", &orders_path, Some(token), &json_header, print(&bare)).await;
    assert_eq!((refused.status, refused.error_code().as_deref()), (422, Some("preflight_failed")));
    let findings = refused.json()["error"]["findings"].as_array().cloned().unwrap_or_default();
    assert!(
        findings.iter().any(|f| f["severity"] == "error"
            && f["message"].as_str().is_some_and(|m| m.to_ascii_lowercase().contains("bleed"))),
        "{findings:#?}"
    );
    assert_eq!(world.printer.state.orders().len(), 1);

    // Ordering a file other than the one uploaded is refused before the printer sees it.
    let mismatch =
        creator_call(&world, "POST", &orders_path, Some(token), &json_header, print(&good)).await;
    assert_eq!((mismatch.status, mismatch.error_code().as_deref()), (409, Some("pdf_mismatch")));

    // The good card: accepted, priced, and at the printer with the very same bytes.
    let put = creator_call(&world, "PUT", &card_path, Some(token), &[], good.clone()).await;
    assert_eq!(put.json()["sha256"], sha256_hex(&good).as_str());
    let ordered =
        creator_call(&world, "POST", &orders_path, Some(token), &json_header, print(&good)).await;
    assert_eq!(ordered.status, 201, "{}", ordered.text());
    let receipt = ordered.json();
    assert_eq!(
        (receipt["status"].as_str(), receipt["price_cents"].as_u64()),
        (Some("accepted"), Some(590))
    );
    assert_eq!(receipt["currency"], "EUR");
    let orders = world.printer.state.orders();
    assert_eq!(orders.len(), 2);
    let at_printer =
        orders.iter().find(|o| o.order_id == receipt["order_id"].as_str().unwrap()).unwrap();
    assert_eq!(
        (at_printer.reference.as_str(), at_printer.pdf_sha256.as_str()),
        (id, sha256_hex(&good).as_str())
    );
    assert_eq!((at_printer.address.city.as_str(), at_printer.quantity), ("Lyon", 1));

    // The order then lives its life at the printer.
    world.printer.state.advance_all().await;
    assert_eq!(
        world.printer.state.order(&at_printer.order_id).unwrap().status.as_str(),
        "printing"
    );
}

fn posted_order() -> GiftOrder<'static> {
    GiftOrder {
        post_to: Some(PostTo {
            name: "Sacha Martin",
            street: "12 rue des Lilas",
            postcode: "69003",
            city: "Lyon",
        }),
        ..GiftOrder::demo()
    }
}

async fn buy_posted(world: &World) -> (ebc_demo_ereaders::EinkBrowser, Receipt) {
    let mut buyer = browser(world, DeviceProfile::computer());
    // The checkout page quotes the printer's price for one posted card.
    let receipt = shop::buy_gift(&mut buyer, &world.septante.base, &posted_order()).await.unwrap();
    assert_eq!(receipt.gifts.len(), 1);
    (buyer, receipt)
}

/// A server built without the card composer answers `501`: the stores show the claim URL
/// and the code instead of a card, and a "printed and posted" order says that nothing was
/// sent to the printer.
#[tokio::test]
async fn without_a_card_renderer_the_stores_fall_back_to_the_link_and_the_code() {
    let world =
        World::builder().card_renderer(Arc::new(ebc_server::NoCardRenderer)).start().await.unwrap();
    let (mut buyer, receipt) = buy_posted(&world).await;
    let gift = &receipt.gifts[0];
    assert_eq!(gift.status, "created", "the gift itself exists either way: {gift:#?}");
    assert!(gift.claim_url.is_some() && gift.code.is_some(), "the fallback: link and code");
    // The posted card was priced from the printer's catalogue (5,90 € on top of the book).
    let total = buyer.page().unwrap().text_of("order-total").unwrap().to_owned();
    assert!(total.starts_with("8,89"), "{total}");

    let card = world
        .partner_api(world.septante.kind)
        .get(&format!(
            "/api/partner/v1/gifts/{}/card.pdf?format=poche&profile=pro",
            gift_id(&world.septante, 0)
        ))
        .await
        .unwrap();
    assert_eq!((card.status, card.error_code().as_deref()), (501, Some("not_implemented")));
    assert!(gift.card_unavailable && gift.card_pdf_url.is_none(), "{gift:#?}");
    assert_eq!(gift.print_status, None, "nothing was sent to the printer");
    assert!(
        gift.error.as_deref().is_some_and(|e| e.contains("impression n'a pas été commandée")),
        "{gift:#?}"
    );
    assert!(world.printer.state.orders().is_empty());
    assert!(buyer.open(&format!("{}/carte-1.pdf", receipt.order_url)).await.unwrap().is_page());
    assert_eq!(buyer.page().unwrap().status(), 404);
}

/// The full chain: checkout with the posted card → the store fetches `card.pdf` (the real
/// card, composed by `ebc-card` with the store's cover) → submits it to the printer,
/// whose preflight accepts it → the printer's status changes come back to the order page.
#[tokio::test]
async fn a_posted_card_reaches_the_printer_and_its_status_comes_back() {
    let world = World::start().await.unwrap();
    let (mut buyer, receipt) = buy_posted(&world).await;
    let gift = &receipt.gifts[0];
    assert_eq!(gift.error, None, "{gift:#?}");
    assert!(gift.card_pdf_url.is_some(), "the buyer can also download the card");
    assert_eq!(gift.print_status.as_deref(), Some("accepted"));

    let orders = world.printer.state.orders();
    assert_eq!(orders.len(), 1, "{orders:#?}");
    let order = orders[0].clone();
    assert_eq!((order.product, order.quantity), ("card-poche", 1));
    assert_eq!(order.reference, receipt.order_ref);
    assert_eq!(
        (order.address.name.as_str(), order.address.city.as_str(), order.address.country.as_str()),
        ("Sacha Martin", "Lyon", "FR")
    );
    assert!(order.callback_url.as_deref().is_some_and(|u| u.starts_with(&world.septante.base)));
    assert!(order.pdf_bytes > 10_000, "a real card, not a stub: {} bytes", order.pdf_bytes);
    eprintln!(
        "the printer accepted the card ({} bytes) with {} preflight warning(s): {:#?}",
        order.pdf_bytes,
        order.preflight_warnings.len(),
        order.preflight_warnings
    );
    // What the printer holds is what the server renders for the professional profile:
    // rendering is deterministic.
    let card = world
        .partner_api(world.septante.kind)
        .get(&format!(
            "/api/partner/v1/gifts/{}/card.pdf?format=poche&profile=pro",
            gift_id(&world.septante, 0)
        ))
        .await
        .unwrap();
    assert_eq!(card.status, 200, "{}", card.text());
    assert_eq!(order.pdf_sha256, sha256_hex(&card.body));
    // The buyer's copy is a PDF too.
    let saved = buyer.open(gift.card_pdf_url.as_deref().unwrap()).await.unwrap();
    assert_eq!(saved.download().expect("the card PDF").format, ebc_demo_ereaders::FileFormat::Pdf);

    // accepted → printing → shipped, each step called back to the store.
    for expected in ["printing", "shipped"] {
        world.printer.state.advance_all().await;
        let after = shop::reload_order(&mut buyer, &receipt).await.unwrap();
        assert_eq!(after.gifts[0].print_status.as_deref(), Some(expected));
    }
    assert!(buyer.page().unwrap().has("print-tracking"));
}
