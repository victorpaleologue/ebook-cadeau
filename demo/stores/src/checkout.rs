//! Validation of the checkout form. Pure functions: no state, no I/O.

use crate::outbound::PostalAddress;
use crate::state::{CardDelivery, GiftDetails};
use crate::views::{CheckoutForm, CheckoutOffer};

/// Longest giver / recipient name, in characters.
pub const MAX_NAME_CHARS: usize = 80;
/// Longest gift message, in characters.
pub const MAX_MESSAGE_CHARS: usize = 500;
/// Most lines in a gift message.
pub const MAX_MESSAGE_LINES: usize = 8;

/// Cleans a one-line field: control characters become spaces, runs of white space collapse
/// to one space, both ends are trimmed.
#[must_use]
pub fn clean_line(input: &str) -> String {
    let spaced: String =
        input.chars().map(|c| if c.is_control() || is_invisible(c) { ' ' } else { c }).collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Cleans a multi-line field: line endings are normalised to `\n`, other control characters
/// are removed, each line is trimmed at the end, and leading / trailing blank lines dropped.
#[must_use]
pub fn clean_text(input: &str) -> String {
    let unified = input.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = unified
        .split('\n')
        .map(|line| {
            let kept: String = line
                .chars()
                .map(|c| if c == '\t' { ' ' } else { c })
                .filter(|c| !c.is_control() && !is_invisible(*c))
                .collect();
            kept.trim_end().to_owned()
        })
        .collect();
    lines.join("\n").trim_matches('\n').to_owned()
}

/// Bidirectional overrides and other invisible format characters that could make a name
/// render misleadingly on the card or in a page.
fn is_invisible(c: char) -> bool {
    matches!(
        c,
        '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{2069}' | '\u{feff}'
    )
}

/// Validates the checkout form against what the store can offer.
///
/// Returns `Ok(None)` for a purchase without the gift option, `Ok(Some(details))` for a
/// gift, or the list of problems (French, one sentence each) to show above the form. The
/// returned details carry `offer.delivery` and the price of a posted card.
///
/// # Errors
/// One message per invalid field.
pub fn validate(
    form: &CheckoutForm,
    offer: &CheckoutOffer,
) -> Result<Option<GiftDetails>, Vec<String>> {
    if !form.gift {
        return Ok(None);
    }
    let mut errors = Vec::new();
    if !offer.gift_available {
        return Err(vec!["L'option cadeau n'est pas disponible pour le moment.".to_owned()]);
    }
    let giver_name = clean_line(&form.giver_name);
    let recipient_name = clean_line(&form.recipient_name);
    let message = clean_text(&form.message);
    if giver_name.is_empty() {
        errors.push("Indiquez de la part de qui est le cadeau.".to_owned());
    } else if giver_name.chars().count() > MAX_NAME_CHARS {
        errors.push(format!("Votre nom est trop long ({MAX_NAME_CHARS} caractères au plus)."));
    }
    if recipient_name.is_empty() {
        errors.push("Indiquez à qui le livre est offert.".to_owned());
    } else if recipient_name.chars().count() > MAX_NAME_CHARS {
        errors.push(format!(
            "Le nom du destinataire est trop long ({MAX_NAME_CHARS} caractères au plus)."
        ));
    }
    if message.chars().count() > MAX_MESSAGE_CHARS {
        errors
            .push(format!("Votre message est trop long ({MAX_MESSAGE_CHARS} caractères au plus)."));
    } else if message.lines().count() > MAX_MESSAGE_LINES {
        errors.push(format!("Votre message compte trop de lignes ({MAX_MESSAGE_LINES} au plus)."));
    }

    let card_delivery = match form.card_delivery.as_str() {
        // An absent choice means the free default.
        "" => Some(CardDelivery::Home),
        other => CardDelivery::from_form(other),
    };
    let Some(card_delivery) = card_delivery else {
        errors.push("Choisissez comment recevoir la carte.".to_owned());
        return Err(errors);
    };

    let mut address = None;
    let mut card_price_cents = 0;
    if card_delivery == CardDelivery::Post {
        match offer.post_price_cents {
            Some(price) => card_price_cents = price,
            None => errors.push("L'envoi postal est indisponible pour le moment.".to_owned()),
        }
        let name = clean_line(&form.ship_name);
        let line1 = clean_line(&form.ship_street);
        let postal_code = clean_line(&form.ship_postcode);
        let city = clean_line(&form.ship_city);
        if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
            errors.push("Indiquez le destinataire du courrier.".to_owned());
        }
        if line1.is_empty() || line1.chars().count() > 120 {
            errors.push("Indiquez l'adresse d'envoi.".to_owned());
        }
        if postal_code.len() != 5 || !postal_code.bytes().all(|b| b.is_ascii_digit()) {
            errors.push("Le code postal doit compter cinq chiffres (envoi en France).".to_owned());
        }
        if city.is_empty() || city.chars().count() > MAX_NAME_CHARS {
            errors.push("Indiquez la ville.".to_owned());
        }
        address = Some(PostalAddress { name, line1, postal_code, city, country: "FR".to_owned() });
    }

    if errors.is_empty() {
        Ok(Some(GiftDetails {
            giver_name,
            recipient_name,
            message,
            card_delivery,
            address,
            delivery: offer.delivery,
            card_price_cents,
        }))
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GiftDelivery;

    fn offer(post: Option<u32>) -> CheckoutOffer {
        CheckoutOffer {
            gift_available: true,
            post_price_cents: post,
            delivery: GiftDelivery::HostedFile,
        }
    }

    fn gift_form() -> CheckoutForm {
        CheckoutForm {
            gift: true,
            giver_name: "Camille".into(),
            recipient_name: "Sacha".into(),
            message: "Joyeux anniversaire !".into(),
            card_delivery: "home".into(),
            ..CheckoutForm::default()
        }
    }

    #[test]
    fn no_gift_ignores_every_other_field() {
        let form = CheckoutForm {
            gift: false,
            giver_name: "x".repeat(10_000),
            card_delivery: "teleport".into(),
            ..CheckoutForm::default()
        };
        assert_eq!(validate(&form, &offer(None)), Ok(None));
    }

    #[test]
    fn a_home_gift_is_free_and_needs_no_address() {
        let details = validate(&gift_form(), &offer(Some(590))).unwrap().unwrap();
        assert_eq!(details.card_delivery, CardDelivery::Home);
        assert_eq!(details.card_price_cents, 0);
        assert_eq!(details.address, None);
        assert_eq!(details.delivery, GiftDelivery::HostedFile);
        // No radio at all (a client that did not send the field) means the free default.
        let form = CheckoutForm { card_delivery: String::new(), ..gift_form() };
        assert_eq!(
            validate(&form, &offer(None)).unwrap().unwrap().card_delivery,
            CardDelivery::Home
        );
    }

    #[test]
    fn a_posted_gift_needs_a_french_address_and_a_price() {
        let form = CheckoutForm {
            card_delivery: "post".into(),
            ship_name: " Sacha  Martin ".into(),
            ship_street: "12 rue des Lilas".into(),
            ship_postcode: "75011".into(),
            ship_city: "Paris".into(),
            ..gift_form()
        };
        let details = validate(&form, &offer(Some(590))).unwrap().unwrap();
        assert_eq!(details.card_price_cents, 590);
        assert_eq!(
            details.address,
            Some(PostalAddress {
                name: "Sacha Martin".into(),
                line1: "12 rue des Lilas".into(),
                postal_code: "75011".into(),
                city: "Paris".into(),
                country: "FR".into(),
            })
        );

        // The printer is down: the choice cannot be honoured.
        let errors = validate(&form, &offer(None)).unwrap_err();
        assert_eq!(errors, ["L'envoi postal est indisponible pour le moment."]);

        // Missing and malformed address fields are all reported at once.
        let bad = CheckoutForm {
            ship_name: String::new(),
            ship_street: "\u{0}\t".into(),
            ship_postcode: "7501A".into(),
            ship_city: "  ".into(),
            ..form
        };
        assert_eq!(validate(&bad, &offer(Some(590))).unwrap_err().len(), 4);
        for postcode in ["7501", "750111", "７５０１１", "75 01"] {
            let f = CheckoutForm { ship_postcode: postcode.into(), ..bad.clone() };
            assert!(validate(&f, &offer(Some(590))).is_err(), "{postcode}");
        }
    }

    #[test]
    fn names_are_required_and_bounded() {
        let form =
            CheckoutForm { giver_name: "  ".into(), recipient_name: String::new(), ..gift_form() };
        assert_eq!(validate(&form, &offer(None)).unwrap_err().len(), 2);

        let exactly = CheckoutForm { recipient_name: "é".repeat(MAX_NAME_CHARS), ..gift_form() };
        assert!(validate(&exactly, &offer(None)).is_ok(), "the limit counts characters, not bytes");
        let over = CheckoutForm { recipient_name: "é".repeat(MAX_NAME_CHARS + 1), ..gift_form() };
        assert_eq!(validate(&over, &offer(None)).unwrap_err().len(), 1);

        let long = CheckoutForm { message: "m".repeat(MAX_MESSAGE_CHARS + 1), ..gift_form() };
        assert!(validate(&long, &offer(None)).is_err());
        let tall = CheckoutForm { message: "a\n".repeat(MAX_MESSAGE_LINES + 1), ..gift_form() };
        assert!(validate(&tall, &offer(None)).is_err());
        let empty_message = CheckoutForm { message: " \r\n ".into(), ..gift_form() };
        assert_eq!(validate(&empty_message, &offer(None)).unwrap().unwrap().message, "");
    }

    #[test]
    fn unknown_card_choice_and_unavailable_gift_are_refused() {
        let form = CheckoutForm { card_delivery: "drone".into(), ..gift_form() };
        assert!(validate(&form, &offer(Some(590))).is_err());
        let off = CheckoutOffer { gift_available: false, ..offer(None) };
        assert_eq!(
            validate(&gift_form(), &off).unwrap_err(),
            ["L'option cadeau n'est pas disponible pour le moment."]
        );
    }

    #[test]
    fn hostile_text_is_neutralised_not_rejected() {
        assert_eq!(clean_line("  Sa\u{0}cha\r\n\tMartin\u{202e}  "), "Sa cha Martin");
        assert_eq!(clean_line("\u{200b}\u{feff}"), "");
        assert_eq!(clean_line("<script>alert(1)</script>"), "<script>alert(1)</script>");
        assert_eq!(clean_text("Bonne\r\nlecture\r\u{7}!\n\n"), "Bonne\nlecture\n!");
        assert_eq!(clean_text("a\tb \n"), "a b");
        assert_eq!(clean_text("\n\n"), "");
        // Markup is kept verbatim (escaping is the templates' job) and survives validation.
        let form = CheckoutForm {
            recipient_name: "<img src=x onerror=alert(1)>".into(),
            message: "\"><script>alert(2)</script>".into(),
            ..gift_form()
        };
        let details = validate(&form, &offer(None)).unwrap().unwrap();
        assert_eq!(details.recipient_name, "<img src=x onerror=alert(1)>");
        assert_eq!(details.message, "\"><script>alert(2)</script>");
    }
}
