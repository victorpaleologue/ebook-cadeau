//! A customer of a demo store, driven like a person would: a script-less simulated
//! browser clicks through catalogue → product → cart → sign-up → checkout with the gift
//! option → payment, and reads the order confirmation page.
//!
//! The element ids used here are those of `demo/stores` (`src/views.rs`); the same walk
//! works in both stores.

use ebc_demo_ereaders::{EinkBrowser, Outcome, Page};

use crate::world::WorldError;

/// Postal address of a card that is printed and posted (France only in the demo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostTo<'a> {
    /// Addressee.
    pub name: &'a str,
    /// Street line.
    pub street: &'a str,
    /// Postal code.
    pub postcode: &'a str,
    /// City.
    pub city: &'a str,
}

/// What the buyer chooses at checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GiftOrder<'a> {
    /// E-mail address of the buyer's (new) store account.
    pub email: &'a str,
    /// Its password (at least 8 characters).
    pub password: &'a str,
    /// Catalogue ids of the books to offer, e.g. `candide`.
    pub books: &'a [&'a str],
    /// "De la part de".
    pub giver: &'a str,
    /// "Pour".
    pub recipient: &'a str,
    /// Personal message (may be empty).
    pub message: &'a str,
    /// `None`: the buyer prints the card at home. `Some`: printed and posted.
    pub post_to: Option<PostTo<'a>>,
}

impl GiftOrder<'_> {
    /// Camille offers "Candide" to Sacha, card printed at home.
    #[must_use]
    pub fn demo() -> Self {
        Self {
            email: "camille@example.org",
            password: "motdepasse",
            books: &["candide"],
            giver: "Camille",
            recipient: "Sacha",
            message: "Joyeux anniversaire !",
            post_to: None,
        }
    }
}

/// One gift as the order page shows it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GiftCard {
    /// `data-status` of the gift: `pending`, `created`, `redeemed`, `exhausted`,
    /// `fulfilled`, `cancelled`, `expired`.
    pub status: String,
    /// The claim URL to print or pass on.
    pub claim_url: Option<String>,
    /// The code as printed, `XXXX-XXXX-XXXX`.
    pub code: Option<String>,
    /// Absolute URL of the card PDF, when the store has one.
    pub card_pdf_url: Option<String>,
    /// Whether the page says no printable card is available (the claim URL and code are
    /// the fallback).
    pub card_unavailable: bool,
    /// `data-status` of the print order of a posted card.
    pub print_status: Option<String>,
    /// The error the page reports about this gift.
    pub error: Option<String>,
    /// Download sessions the store knows about.
    pub downloads: Option<u32>,
}

/// The order confirmation page, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// The order reference, e.g. `SEP-2026-000001`.
    pub order_ref: String,
    /// Absolute URL of the order page.
    pub order_url: String,
    /// `paid` or `refunded`.
    pub order_status: String,
    /// The gifts of the order, in line order.
    pub gifts: Vec<GiftCard>,
}

fn journey(what: impl Into<String>) -> WorldError {
    WorldError::Journey(what.into())
}

/// Requires that the last interaction displayed a page with status 200.
fn shown<'a>(
    browser: &'a EinkBrowser,
    outcome: &Outcome,
    step: &str,
) -> Result<&'a Page, WorldError> {
    if !outcome.is_page() {
        return Err(journey(format!("{step}: expected a page, got {outcome:?}")));
    }
    let page = browser.page()?;
    if page.status() != 200 {
        return Err(journey(format!(
            "{step}: HTTP {} at {}: {}",
            page.status(),
            page.url(),
            page.text().chars().take(300).collect::<String>()
        )));
    }
    Ok(page)
}

/// Creates a store account from the sign-in page currently displayed (the store shows it
/// whenever an account is needed) and lands wherever the store sends the new customer.
///
/// # Errors
/// [`WorldError::Device`] / [`WorldError::Journey`] when the page is not the sign-in page
/// or the account is refused.
pub async fn register_here(
    browser: &mut EinkBrowser,
    email: &str,
    password: &str,
) -> Result<(), WorldError> {
    browser.type_into("register-email", email)?;
    browser.type_into("register-password", password)?;
    let outcome = browser.click("register-submit").await?;
    shown(browser, &outcome, "create the account")?;
    Ok(())
}

/// Buys `order.books` as a gift in the store at `store_base`, starting from its home
/// page, and returns what the order confirmation page says.
///
/// The walk: home → each product page → "add to cart" → cart → "checkout" → create an
/// account → tick the gift option, fill the names and the message, choose home printing or
/// the posted card → "pay".
///
/// # Errors
/// [`WorldError::Device`] when the simulated browser cannot perform a step (which
/// includes anything that would need JavaScript), [`WorldError::Journey`] when a page is
/// not the expected one.
pub async fn buy_gift(
    browser: &mut EinkBrowser,
    store_base: &str,
    order: &GiftOrder<'_>,
) -> Result<Receipt, WorldError> {
    for book in order.books {
        let outcome = browser.open(&format!("{store_base}/")).await?;
        shown(browser, &outcome, "open the store")?;
        let outcome = browser.click(&format!("book-{book}")).await?;
        shown(browser, &outcome, "open the product page")?;
        let outcome = browser.click("add-to-cart").await?;
        shown(browser, &outcome, "add to the cart")?;
    }
    let outcome = browser.click("checkout-link").await?;
    let page = shown(browser, &outcome, "go to checkout")?;
    if page.has("register-form") {
        register_here(browser, order.email, order.password).await?;
    }
    if !browser.page()?.has("gift-option") {
        return Err(journey(format!(
            "the checkout page at {} does not offer the gift option",
            browser.page()?.url()
        )));
    }
    browser.check("gift-option")?;
    browser.type_into("giver-name", order.giver)?;
    browser.type_into("recipient-name", order.recipient)?;
    browser.type_into("gift-message", order.message)?;
    match &order.post_to {
        None => browser.check("card-delivery-home")?,
        Some(address) => {
            browser.check("card-delivery-post")?;
            browser.type_into("ship-name", address.name)?;
            browser.type_into("ship-street", address.street)?;
            browser.type_into("ship-postcode", address.postcode)?;
            browser.type_into("ship-city", address.city)?;
        }
    }
    let outcome = browser.click("pay").await?;
    let page = shown(browser, &outcome, "pay")?;
    read_order_page(page)
}

/// Opens an order page again (the store refreshes the gift status from the server on each
/// load) and reads it.
///
/// # Errors
/// As [`buy_gift`].
pub async fn reload_order(
    browser: &mut EinkBrowser,
    receipt: &Receipt,
) -> Result<Receipt, WorldError> {
    let outcome = browser.open(&receipt.order_url).await?;
    read_order_page(shown(browser, &outcome, "open the order page")?)
}

/// Reads an order confirmation page of a demo store.
///
/// # Errors
/// [`WorldError::Journey`] when the page is not an order page.
pub fn read_order_page(page: &Page) -> Result<Receipt, WorldError> {
    let order_ref = page
        .text_of("order-ref")
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| {
            journey(format!(
                "{} is not an order page: {}",
                page.url(),
                page.text().chars().take(300).collect::<String>()
            ))
        })?;
    let attr = |id: &str, name: &str| {
        page.element(id).and_then(|element| element.attr(name)).map(str::to_owned)
    };
    let text = |id: &str| page.text_of(id).map(|text| text.trim().to_owned());
    let mut gifts = Vec::new();
    // The first gift carries the bare ids; further ones get a numeric suffix (`-2`, …).
    for index in 1usize.. {
        let id = |name: &str| if index == 1 { name.to_owned() } else { format!("{name}-{index}") };
        let Some(status) = attr(&id("gift-status"), "data-status") else { break };
        let card_pdf_url = attr(&id("card-pdf"), "href")
            .and_then(|href| page.url().join(&href).ok())
            .map(|url| url.to_string());
        gifts.push(GiftCard {
            status,
            claim_url: attr(&id("claim-url"), "href").or_else(|| text(&id("claim-url"))),
            code: text(&id("claim-code")),
            card_pdf_url,
            card_unavailable: page.has(&id("card-unavailable")),
            print_status: attr(&id("print-status"), "data-status"),
            error: text(&id("gift-error")),
            downloads: text(&id("gift-downloads")).and_then(|n| n.parse().ok()),
        });
        if gifts.len() >= 64 {
            break;
        }
    }
    Ok(Receipt {
        order_ref,
        order_url: page.url().to_string(),
        order_status: attr("order-status", "data-status").unwrap_or_default(),
        gifts,
    })
}
