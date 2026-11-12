//! Helpers shared by the journeys: lint assertions, the expected watermarked file,
//! keystroke records, plain HTTP calls.

use std::io::Read;

use ebc_demo_ereaders::typing::TapLog;
use ebc_demo_ereaders::{DeviceProfile, EinkBrowser, LintConfig};
use ebc_demo_integration::partner::{ApiReply, plain_client, read_reply};
use ebc_demo_integration::shop::{self, GiftOrder, Receipt};
use ebc_demo_integration::{StoreHandle, World};
use ebc_demo_stores::assets::Assets;
use ebc_demo_stores::epub::Watermark;
use ebc_model::GiftCode;
use sha2::{Digest, Sha256};

/// A browser of `profile` in `world`.
pub(crate) fn browser(world: &World, profile: DeviceProfile) -> EinkBrowser {
    world.browser(profile).unwrap()
}

/// Asserts that the page displayed by `browser` works on its device: nothing that needs
/// JavaScript, nothing loaded from elsewhere, labelled controls, and the weight budget of
/// `docs/api.md` §2 (30 kB).
pub(crate) fn assert_lint_clean(browser: &EinkBrowser, what: &str) {
    let page = browser.page().unwrap();
    let report = page.lint(&LintConfig::for_profile(browser.profile()));
    assert!(
        report.passed(),
        "{what} ({}) fails the e-reader lint: {:#?}",
        page.url(),
        report.findings
    );
    assert!(page.weight_bytes() < 30_000, "{what} weighs {} bytes", page.weight_bytes());
    assert!(!page.html().to_ascii_lowercase().contains("<script"), "{what} carries a script");
}

/// Asserts the response headers `docs/api.md` §2 requires on gift pages.
pub(crate) fn assert_gift_headers(browser: &EinkBrowser, what: &str) {
    let page = browser.page().unwrap();
    let report = page.check_gift_headers();
    assert!(report.passed(), "{what} ({}): {:#?}", page.url(), report.findings);
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The text of one file of an EPUB (a ZIP archive).
pub(crate) fn zip_text(epub: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(epub)).unwrap();
    let mut file = archive.by_name(name).unwrap_or_else(|e| panic!("no {name} in the EPUB: {e}"));
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}

/// The copy of `book` the store stamps for a gift of `giver` to `recipient` ordered under
/// `line_ref` at `stamped_at`: what the recipient's device must end up with, byte for
/// byte.
pub(crate) fn stamped_epub(
    store: &StoreHandle,
    book: &str,
    line_ref: &str,
    recipient: &str,
    giver: &str,
    stamped_at: i64,
) -> Vec<u8> {
    let watermark = Watermark {
        store_name: store.kind.display_name().to_owned(),
        order_ref: line_ref.to_owned(),
        licensee: format!("Offert à {recipient} par {giver}"),
        stamped_at,
    };
    Assets::get().unwrap().book(book).unwrap().stamped_epub(&watermark).unwrap()
}

/// The unstamped edition of `book`.
pub(crate) fn plain_epub(book: &str) -> Vec<u8> {
    Assets::get().unwrap().book(book).unwrap().epub.clone()
}

/// Camille buys "Candide" for Sacha at `store`, card printed at home.
pub(crate) async fn buy_candide(world: &World, store: &StoreHandle) -> (EinkBrowser, Receipt) {
    let mut buyer = browser(world, DeviceProfile::computer());
    let receipt = shop::buy_gift(&mut buyer, &store.base, &GiftOrder::demo()).await.unwrap();
    assert_eq!(receipt.order_status, "paid");
    assert_eq!(receipt.gifts.len(), 1);
    assert_eq!(receipt.gifts[0].status, "created", "{receipt:#?}");
    (buyer, receipt)
}

/// The gift code printed on the (first) card of an order, checked against v0.2.
pub(crate) fn printed_code(receipt: &Receipt) -> GiftCode {
    let printed = receipt.gifts[0].code.as_deref().expect("the order page shows the code");
    let code = GiftCode::parse(printed).expect("a 12-letter code with a valid check letter");
    assert_eq!(printed, code.grouped(), "printed as XXXX-XXXX-XXXX");
    assert_eq!(printed.len(), 14);
    code
}

/// The gift id the store holds for its `index`-th gift.
pub(crate) fn gift_id(store: &StoreHandle, index: usize) -> String {
    store.snapshot()["gifts"][index]["gift_id"].as_str().expect("a registered gift").to_owned()
}

/// Writes the tap log of a device journey to `target/tmp/keystrokes-{name}.txt` (and to
/// the test output), so that the UX cost of each path is on record.
pub(crate) fn record_taps(name: &str, device: &str, taps: &TapLog) {
    let mut report = format!("{name} — {device}\n");
    for step in taps.steps() {
        report.push_str(&format!("{:>4}  {}\n", step.taps, step.action));
    }
    report.push_str(&format!("{:>4}  TOTAL\n", taps.total()));
    eprintln!("{report}");
    let path =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("keystrokes-{name}.txt"));
    std::fs::write(path, report).unwrap();
}

/// A plain `GET` (no redirects followed) with a given `User-Agent`.
pub(crate) async fn http_get(url: &str, user_agent: &str) -> ApiReply {
    let response =
        plain_client().unwrap().get(url).header("user-agent", user_agent).send().await.unwrap();
    read_reply(response).await.unwrap()
}

/// A plain urlencoded `POST` (no redirects followed) with a given `User-Agent`.
pub(crate) async fn http_post_form(
    url: &str,
    user_agent: &str,
    fields: &[(&str, &str)],
) -> ApiReply {
    let body =
        fields.iter().map(|(name, value)| format!("{name}={value}")).collect::<Vec<_>>().join("&");
    let response = plain_client()
        .unwrap()
        .post(url)
        .header("user-agent", user_agent)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap();
    read_reply(response).await.unwrap()
}

/// A call to the open creator API of the server (`docs/api.md` §3), with the manage token of
/// a gift as `Authorization: Bearer` when one is given.
pub(crate) async fn creator_call(
    world: &World,
    method: &str,
    path: &str,
    token: Option<&str>,
    headers: &[(&str, &str)],
    body: Vec<u8>,
) -> ApiReply {
    let method = reqwest::Method::from_bytes(method.as_bytes()).unwrap();
    let mut request = plain_client().unwrap().request(method, world.server.url(path)).body(body);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    read_reply(request.send().await.unwrap()).await.unwrap()
}
