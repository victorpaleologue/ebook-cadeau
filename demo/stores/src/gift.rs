//! The server-side gift pipeline run after payment, and the status refresh run on each
//! order page load.
//!
//! The pipeline is a sequence of idempotent steps — upload (for `hosted_file`), gift
//! creation (`Idempotency-Key` = the order line reference), cover, card PDF, print order —
//! each recorded in the [`GiftRecord`](crate::state::GiftRecord) as soon as it succeeds, so
//! that a failed run can be resumed from where it stopped ("Réessayer" on the order page)
//! without creating anything twice.

use std::sync::Arc;

use crate::catalogue::{self, Book, PUBLISHER};
use crate::config::GiftDelivery;
use crate::epub::{self, Watermark};
use crate::outbound::{
    BookPayload, CardFetch, ContributorPayload, DeliveryPayload, Fulfilment, NewGift,
    PrintOrderRequest, PrintSubmission,
};
use crate::state::{
    AppState, CardDelivery, CardState, GiftDetails, GiftStatus, Order, PrintRecord,
};

/// Card format requested from eBook Cadeau and ordered from the printer.
pub const CARD_FORMAT: &str = "poche";
/// The printer product matching [`CARD_FORMAT`].
pub const CARD_PRODUCT: &str = "card-poche";
/// How long the price of a posted card is cached, in seconds.
const POST_PRICE_TTL_SECS: i64 = 30;

/// The price of one posted card according to the printer's catalogue, or `None` when no
/// printer is configured, it cannot be reached, or it has no single-copy price.
pub async fn post_card_price(state: &AppState) -> Option<u32> {
    let printer = state.printer.as_ref()?;
    if let Some(cached) = state.cached_post_price(POST_PRICE_TTL_SECS) {
        return cached;
    }
    let price = match printer.products().await {
        Ok(products) => products.iter().find(|p| p.id == CARD_PRODUCT).and_then(|p| p.price_for(1)),
        Err(_) => None,
    };
    state.cache_post_price(price);
    price
}

/// Runs (or resumes) the gift pipeline for every gift line of `order_ref`.
pub async fn process_order(state: &AppState, order_ref: &str) {
    let Some(order) = state.order(order_ref) else { return };
    let Some(details) = order.gift.clone() else { return };
    for index in 0..order.lines.len() {
        process_line(state, &order, &details, index).await;
    }
}

/// Records a failure on the gift and stops the pipeline for this line.
fn fail(state: &AppState, line_ref: &str, message: String) {
    state.update_gift(line_ref, |g| g.last_error = Some(message));
}

async fn process_line(state: &AppState, order: &Order, details: &GiftDetails, index: usize) {
    let Some(line) = order.lines.get(index) else { return };
    let Some(line_ref) = line.gift.as_ref().map(|g| g.line_ref.as_str()) else { return };
    // Work on a fresh copy of the record: an earlier, partially successful run may have
    // completed some steps already.
    let current = |state: &AppState| {
        state.order(&order.reference).and_then(|o| o.lines.get(index).and_then(|l| l.gift.clone()))
    };
    let Some(gift) = current(state) else { return };
    if gift.status.is_terminal() {
        return;
    }
    let (Some(partner), Some(book)) = (state.partner.as_ref(), catalogue::find(line.book_id))
    else {
        fail(state, line_ref, "L'option cadeau n'est pas configurée sur cette boutique.".into());
        return;
    };
    let assets = match state.assets.as_ref().map(|a| a.book(book.id)) {
        Ok(Some(assets)) => assets,
        _ => {
            fail(state, line_ref, "Les fichiers du livre sont indisponibles.".into());
            return;
        }
    };
    state.update_gift(line_ref, |g| g.last_error = None);

    // 1. hosted_file: upload a copy stamped for the recipient.
    if details.delivery == GiftDelivery::HostedFile && gift.upload_id.is_none() {
        let watermark = Watermark {
            store_name: state.config.kind.display_name().to_owned(),
            order_ref: line_ref.to_owned(),
            licensee: format!("Offert à {} par {}", details.recipient_name, details.giver_name),
            stamped_at: order.created_at,
        };
        let stamped = match assets.stamped_epub(&watermark) {
            Ok(bytes) => bytes,
            Err(_) => {
                fail(state, line_ref, "Le fichier du livre n'a pas pu être préparé.".into());
                return;
            }
        };
        let key = format!("{line_ref}-upload");
        match partner.upload(&book.file_name(), epub::MEDIA_TYPE, stamped, &key).await {
            Ok(receipt) => {
                state.update_gift(line_ref, |g| g.upload_id = Some(receipt.upload_id));
            }
            Err(e) => {
                fail(state, line_ref, e.user_message());
                return;
            }
        }
    }

    // 2. Create the gift (idempotent on the order line reference).
    let Some(gift) = current(state) else { return };
    if gift.gift_id.is_none() {
        let delivery = match (details.delivery, gift.upload_id.clone()) {
            (GiftDelivery::HostedFile, Some(upload_id)) => {
                DeliveryPayload::HostedFile { upload_id }
            }
            (GiftDelivery::HostedFile, None) => return,
            (GiftDelivery::ResellerRedirect, _) => DeliveryPayload::ResellerRedirect {
                url: format!("{}/cadeau/activer", state.config.public_url),
            },
        };
        let request = NewGift {
            order_ref: line_ref.to_owned(),
            book: book_payload(book),
            giver_name: details.giver_name.clone(),
            recipient_name: details.recipient_name.clone(),
            message: details.message.clone(),
            locale: "fr".to_owned(),
            delivery,
        };
        match partner.create_gift(&request, line_ref).await {
            Ok(created) => {
                state.update_gift(line_ref, |g| {
                    g.status = created
                        .status
                        .as_deref()
                        .and_then(GiftStatus::from_api)
                        .unwrap_or(GiftStatus::Created);
                    g.code_display = created.code_display.or(created.code);
                    g.claim_url = Some(created.claim_url);
                    g.gift_id = Some(created.id);
                });
            }
            Err(e) => {
                fail(state, line_ref, e.user_message());
                return;
            }
        }
    }

    // 3. Send the cover the store is entitled to print (its own artwork).
    let Some(gift) = current(state) else { return };
    let Some(gift_id) = gift.gift_id.clone() else { return };
    if !gift.cover_sent {
        let png = match assets.print_png() {
            Ok(png) => png.to_vec(),
            Err(_) => {
                fail(state, line_ref, "La couverture n'a pas pu être préparée.".into());
                return;
            }
        };
        match partner.put_cover(&gift_id, "image/png", png).await {
            Ok(()) => {
                state.update_gift(line_ref, |g| g.cover_sent = true);
            }
            Err(e) => {
                fail(state, line_ref, e.user_message());
                return;
            }
        }
    }

    // 4. Fetch the card rendered by eBook Cadeau.
    if gift.card == CardState::Pending {
        match partner.card_pdf(&gift_id, CARD_FORMAT, details.card_delivery.card_profile()).await {
            Ok(CardFetch::Ready(pdf)) => {
                state.update_gift(line_ref, |g| g.card = CardState::Ready(Arc::new(pdf)));
            }
            Ok(CardFetch::NotImplemented) => {
                state.update_gift(line_ref, |g| g.card = CardState::Unavailable);
            }
            Err(e) => {
                fail(state, line_ref, e.user_message());
                return;
            }
        }
    }

    // 5. Posted card: hand the PDF to the printer.
    if details.card_delivery == CardDelivery::Post {
        submit_print_order(state, line_ref, details).await;
    }
}

fn book_payload(book: &Book) -> BookPayload {
    BookPayload {
        title: book.full_title(),
        isbn13: book.isbn13.to_owned(),
        contributors: vec![ContributorPayload { name: book.author.to_owned() }],
        publisher: PUBLISHER.to_owned(),
        language: "fr".to_owned(),
    }
}

async fn submit_print_order(state: &AppState, line_ref: &str, details: &GiftDetails) {
    let Some((card, print)) = state.update_gift(line_ref, |g| (g.card.clone(), g.print.clone()))
    else {
        return;
    };
    // Already submitted (accepted, printing, shipped…) or definitively refused.
    if print.as_ref().is_some_and(|p| p.status != "pending") {
        return;
    }
    let pdf = match card {
        CardState::Ready(pdf) => pdf,
        CardState::Unavailable => {
            fail(
                state,
                line_ref,
                "La carte n'a pas pu être générée : l'impression n'a pas été commandée.".into(),
            );
            return;
        }
        CardState::Pending => return,
    };
    let (Some(printer), Some(address)) = (state.printer.as_ref(), details.address.clone()) else {
        fail(state, line_ref, "L'envoi postal n'est pas disponible sur cette boutique.".into());
        return;
    };
    let token = match print {
        Some(existing) => existing.callback_token,
        None => {
            let Ok(token) = state.config.entropy.token(16) else {
                fail(state, line_ref, "Erreur interne.".into());
                return;
            };
            state.update_gift(line_ref, |g| {
                g.print = Some(PrintRecord {
                    order_id: None,
                    status: "pending".to_owned(),
                    tracking: None,
                    price_cents: None,
                    findings: Vec::new(),
                    callback_token: token.clone(),
                });
            });
            token
        }
    };
    let request = PrintOrderRequest {
        product: CARD_PRODUCT.to_owned(),
        quantity: 1,
        address,
        reference: line_ref.to_owned(),
        callback_url: Some(format!("{}/webhooks/printer?token={token}", state.config.public_url)),
    };
    match printer.create_order(&request, pdf.as_ref().clone()).await {
        Ok(PrintSubmission::Accepted(accepted)) => {
            state.update_gift(line_ref, |g| {
                if let Some(print) = g.print.as_mut() {
                    print.tracking = accepted.tracking_text();
                    print.order_id = accepted.order_id;
                    print.price_cents = accepted.price_cents;
                    print.status = known_print_status(&accepted.status).to_owned();
                }
            });
        }
        Ok(PrintSubmission::Rejected { findings }) => {
            state.update_gift(line_ref, |g| {
                if let Some(print) = g.print.as_mut() {
                    print.status = "rejected".to_owned();
                    print.findings = findings;
                }
                g.last_error = Some("L'imprimeur a refusé le fichier de la carte.".to_owned());
            });
        }
        Err(e) => fail(state, line_ref, e.user_message()),
    }
}

/// Maps a status word received from the printer onto the known vocabulary, so that nothing
/// unexpected is ever stored or displayed.
fn known_print_status(status: &str) -> &'static str {
    match status {
        "accepted" => "accepted",
        "printing" => "printing",
        "shipped" => "shipped",
        "cancelled" => "cancelled",
        _ => "accepted",
    }
}

/// Synchronises an order with its collaborators: confirms pending fulfilments, then reads
/// the gift status from the partner API and the print status from the printer. Failures
/// are ignored — the page shows the last known state.
pub async fn refresh_order(state: &AppState, order_ref: &str) {
    confirm_pending_fulfilments(state, Some(order_ref)).await;
    let Some(order) = state.order(order_ref) else { return };
    for gift in order.lines.iter().filter_map(|l| l.gift.as_ref()) {
        if let (Some(partner), Some(gift_id)) = (state.partner.as_ref(), gift.gift_id.as_deref())
            && let Ok(remote) = partner.gift_status(gift_id).await
        {
            state.apply_remote_status(&gift.line_ref, &remote);
        }
        if let (Some(printer), Some(print)) = (state.printer.as_ref(), gift.print.as_ref())
            && let Some(print_id) = print.order_id.as_deref()
            && let Ok(remote) = printer.order(print_id).await
        {
            state.update_gift(&gift.line_ref, |g| {
                if let Some(print) = g.print.as_mut() {
                    print.status = known_print_status(&remote.status).to_owned();
                    if let Some(tracking) = remote.tracking_text() {
                        print.tracking = Some(tracking);
                    }
                }
            });
        }
    }
}

/// Tells eBook Cadeau about every gift activated in this store whose fulfilment it has not
/// acknowledged yet (all orders, or only `order_ref`).
pub async fn confirm_pending_fulfilments(state: &AppState, order_ref: Option<&str>) {
    let Some(partner) = state.partner.as_ref() else { return };
    for (line_ref, gift_id, email) in state.unconfirmed_fulfilments(order_ref) {
        match partner.confirm_fulfilled(&gift_id, &state.account_ref(&email)).await {
            Ok(Fulfilment::Confirmed) => {
                state.update_gift(&line_ref, |g| g.fulfilment_confirmed = true);
            }
            Ok(Fulfilment::AlreadyFulfilled) => {
                // eBook Cadeau has another account on record. The store's own library is
                // the source of truth for who owns the book; stop retrying and keep a note.
                state.update_gift(&line_ref, |g| {
                    g.fulfilment_confirmed = true;
                    g.last_error = Some(
                        "eBook Cadeau avait déjà enregistré ce cadeau pour un autre compte."
                            .to_owned(),
                    );
                });
            }
            Err(_) => {}
        }
    }
}

/// Why a gift order could not be cancelled.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CancelError {
    /// The order has no gift, or is already refunded.
    #[error("cette commande ne peut pas être annulée")]
    NotCancellable,
    /// A gift of the order was already used by its recipient.
    #[error("le cadeau a déjà été utilisé par son destinataire")]
    AlreadyUsed,
    /// eBook Cadeau refused or could not be reached.
    #[error("{0}")]
    Remote(String),
}

/// Cancels every gift of an order (the refund path, §4.2 `cancel`) and marks the order as
/// refunded. Refused as soon as one gift has been redeemed or fulfilled.
///
/// # Errors
/// See [`CancelError`].
pub async fn cancel_order(state: &AppState, order_ref: &str) -> Result<(), CancelError> {
    // Make sure we judge on fresh statuses.
    refresh_order(state, order_ref).await;
    let order = state.order(order_ref).ok_or(CancelError::NotCancellable)?;
    if order.gift.is_none() || order.status != crate::state::OrderStatus::Paid {
        return Err(CancelError::NotCancellable);
    }
    let gifts: Vec<_> = order.lines.iter().filter_map(|l| l.gift.as_ref()).collect();
    if gifts.iter().any(|g| matches!(g.status, GiftStatus::Redeemed | GiftStatus::Fulfilled))
        || gifts.iter().any(|g| g.activated_by.is_some())
    {
        return Err(CancelError::AlreadyUsed);
    }
    for gift in gifts {
        if gift.status.is_terminal() {
            continue;
        }
        if let (Some(partner), Some(gift_id)) = (state.partner.as_ref(), gift.gift_id.as_deref()) {
            partner
                .cancel_gift(gift_id)
                .await
                .map_err(|e| CancelError::Remote(e.user_message()))?;
        }
        state.update_gift(&gift.line_ref, |g| {
            g.status = GiftStatus::Cancelled;
            g.last_error = None;
        });
    }
    state.mark_refunded(order_ref);
    Ok(())
}
