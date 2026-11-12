//! HTML pages (maud templates, auto-escaping). Plain forms only: every flow works without
//! JavaScript. Element `id`s used by the integration and end-to-end tests are listed in
//! `demo/README.md`; treat them as part of the interface.

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::catalogue::{self, Book, ISBN_LABEL, format_price};
use crate::config::{GiftDelivery, StoreKind};
use crate::state::{
    CardDelivery, CardState, GiftRecord, GiftStatus, LibraryEntry, Order, OrderStatus, PrintRecord,
};
use crate::time::french_date;

/// The notice every page carries.
pub const DEMO_BANNER: &str = "Démo — boutique fictive, sans lien avec une enseigne réelle";

/// What the page chrome needs to know about the visitor.
#[derive(Debug, Clone, Copy)]
pub struct Chrome<'a> {
    /// Which store is rendering.
    pub store: StoreKind,
    /// Signed-in account.
    pub account: Option<&'a str>,
    /// Number of books in the cart.
    pub cart_count: usize,
}

impl StoreKind {
    /// How this store writes "eBook" (a per-reseller choice, see the terminology research).
    #[must_use]
    pub const fn ebook_noun(self) -> &'static str {
        match self {
            Self::Septante => "ebook",
            Self::GrandeEnseigne => "Ebook",
        }
    }

    /// The store's tagline.
    #[must_use]
    pub const fn tagline(self) -> &'static str {
        match self {
            Self::Septante => "Librairie numérique indépendante — ebooks sans DRM",
            Self::GrandeEnseigne => "Livres, culture et liseuses — vos Ebooks dans votre compte",
        }
    }

    /// Label of the gift checkbox at checkout (verb form, as real stores label it).
    #[must_use]
    pub fn gift_option_label(self) -> String {
        format!("Offrir cet {}", self.ebook_noun())
    }

    /// Label of the gift checkbox when the cart holds several books.
    #[must_use]
    pub fn gift_option_label_plural(self) -> String {
        format!("Offrir ces {}s (une carte par livre)", self.ebook_noun())
    }
}

/// Help text under the gift checkbox.
pub const GIFT_OPTION_HELP: &str = "Avec une carte « eBook Cadeau » à imprimer : la couverture du livre, un dos d'un \
     centimètre pour la ranger dans une bibliothèque et, à l'intérieur, un QR code et un code \
     pour recevoir le livre sur liseuse, tablette ou téléphone.";
/// Label of the "print at home" card choice.
pub const CARD_HOME_LABEL: &str = "Carte à imprimer chez moi (gratuit)";
/// Label of the "printed and posted" card choice.
pub const CARD_POST_LABEL: &str = "Carte imprimée et envoyée par la poste";

const BASE_CSS: &str = "\
*{box-sizing:border-box}\
body{margin:0;line-height:1.5}\
#demo-banner{background:#111;color:#fff;text-align:center;padding:.4em 1em;font:600 .85rem/1.4 system-ui,sans-serif;letter-spacing:.02em}\
header.site{padding:1em 1.5em;display:flex;flex-wrap:wrap;align-items:baseline;gap:.5em 1.5em}\
header.site .brand{font-size:1.7rem;font-weight:700;text-decoration:none}\
header.site .tagline{font-size:.9rem;opacity:.8}\
header.site nav{margin-left:auto;display:flex;flex-wrap:wrap;gap:1em;align-items:baseline}\
header.site form{display:inline}\
main{max-width:60rem;margin:0 auto;padding:1.5em}\
footer.site{max-width:60rem;margin:2em auto;padding:1em 1.5em;font-size:.85rem;opacity:.75;border-top:1px solid #8884}\
h1{font-size:1.6rem;margin:.2em 0 .8em}\
h2{font-size:1.2rem;margin:1.4em 0 .5em}\
ul.books{list-style:none;padding:0;display:flex;flex-wrap:wrap;gap:1.5em}\
ul.books li{width:11rem}\
ul.books a{text-decoration:none;display:block}\
ul.books img,.cover{width:100%;height:auto;display:block;box-shadow:0 2px 8px #0004}\
ul.books .t{display:block;font-weight:700;margin-top:.6em}\
.product{display:flex;flex-wrap:wrap;gap:2em}\
.product .cover-box{width:16rem;max-width:100%}\
.product .info{flex:1;min-width:16rem}\
.price{font-weight:700}\
.muted{opacity:.75;font-size:.9rem}\
table.lines{border-collapse:collapse;width:100%}\
table.lines td,table.lines th{padding:.5em .6em;border-bottom:1px solid #8884;text-align:left;vertical-align:top}\
table.lines td.num,table.lines th.num{text-align:right;white-space:nowrap}\
fieldset{border:1px solid #8886;border-radius:.4em;margin:1.2em 0;padding:1em}\
legend{font-weight:700;padding:0 .4em}\
label{display:block;margin:.6em 0 .2em}\
label.inline{display:inline;margin:0}\
input[type=text],input[type=email],input[type=password],textarea{width:100%;max-width:28rem;padding:.5em;font:inherit;border:1px solid #8888;border-radius:.3em;background:#fff;color:#111}\
button,a.button{font:inherit;font-weight:700;padding:.55em 1.1em;border:0;border-radius:.3em;cursor:pointer;text-decoration:none;display:inline-block}\
button.link{background:none;padding:0;font-weight:400;text-decoration:underline}\
.notice{padding:.8em 1em;border-radius:.3em;margin:1em 0;border:1px solid #8886}\
.error{border-color:#b00020;color:#7a0016;background:#fdecee}\
.ok{border-color:#1b7f3b;background:#eaf7ee;color:#0f4f24}\
.gift{border:1px solid #8886;border-radius:.4em;padding:1em;margin:1em 0}\
.code{font:700 1.3rem/1.2 ui-monospace,Menlo,Consolas,monospace;letter-spacing:.08em}\
.cols{display:flex;flex-wrap:wrap;gap:2em}\
.cols>*{flex:1;min-width:16rem}\
";

const SEPTANTE_CSS: &str = "\
body{background:#fbf7f0;color:#2b2118;font-family:Georgia,'Times New Roman',serif}\
header.site{border-bottom:3px double #b5482f}\
header.site .brand{color:#b5482f;font-style:italic}\
a{color:#8f3521}\
button,a.button{background:#b5482f;color:#fff}\
button.link{color:#8f3521}\
";

const GRANDE_ENSEIGNE_CSS: &str = "\
body{background:#f4f6f8;color:#14232b;font-family:'Helvetica Neue',Arial,sans-serif}\
header.site{background:#0b3b4f;color:#fff}\
header.site a{color:#fff}\
header.site .brand{text-transform:uppercase;letter-spacing:.06em}\
header.site button.link{color:#fff}\
a{color:#0b5d7a}\
button,a.button{background:#ef6f6c;color:#1a0c0b}\
button.link{color:#0b5d7a}\
";

/// Wraps `body` in the store's page chrome (banner, header, footer).
#[must_use]
pub fn page(chrome: &Chrome<'_>, title: &str, body: Markup) -> Markup {
    let store = chrome.store;
    let theme = match store {
        StoreKind::Septante => SEPTANTE_CSS,
        StoreKind::GrandeEnseigne => GRANDE_ENSEIGNE_CSS,
    };
    html! {
        (DOCTYPE)
        html lang="fr" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="robots" content="noindex, nofollow";
                title { (title) " — " (store.display_name()) " (démo)" }
                style { (PreEscaped(BASE_CSS)) (PreEscaped(theme)) }
            }
            body class=(format!("store-{}", store.slug())) {
                div #demo-banner role="note" { (DEMO_BANNER) }
                header.site {
                    a.brand #brand href="/" { (store.display_name()) }
                    span.tagline { (store.tagline()) }
                    nav {
                        a #nav-catalogue href="/" { "Catalogue" }
                        a #nav-cart href="/panier" { "Panier (" span #cart-count { (chrome.cart_count) } ")" }
                        @if let Some(email) = chrome.account {
                            a #nav-library href="/bibliotheque" { "Ma bibliothèque" }
                            a #nav-orders href="/commandes" { "Mes commandes" }
                            span #account-email { (email) }
                            form method="post" action="/compte/deconnexion" {
                                button.link #logout type="submit" { "Déconnexion" }
                            }
                        } @else {
                            a #nav-login href="/compte/connexion" { "Connexion" }
                        }
                    }
                }
                main { (body) }
                footer.site {
                    p {
                        (store.display_name()) " est une boutique de démonstration du projet eBook Cadeau. "
                        "Les éditions, prix et " (ISBN_LABEL) " sont inventés ; les textes sont des extraits "
                        "du domaine public ; aucun paiement n'est réellement effectué."
                    }
                }
            }
        }
    }
}

fn cover_img(book: &Book) -> Markup {
    html! {
        img.cover src=(format!("/covers/{}.svg", book.id)) width="220" height="356"
            alt=(format!("Couverture : {} — {}", book.full_title(), book.author));
    }
}

/// The catalogue (home page).
#[must_use]
pub fn home(chrome: &Chrome<'_>, warnings: &[String]) -> Markup {
    page(
        chrome,
        "Catalogue",
        html! {
            h1 { "Classiques à (re)lire" }
            @for warning in warnings {
                p.notice.error { "Configuration : " (warning) }
            }
            ul.books #catalogue {
                @for book in catalogue::books() {
                    li {
                        a id=(format!("book-{}", book.id)) href=(format!("/livre/{}", book.id)) {
                            (cover_img(book))
                            span.t { (book.full_title()) }
                            span { (book.author) } " "
                            span.price { (format_price(book.price_cents)) }
                        }
                    }
                }
            }
        },
    )
}

/// A product page.
#[must_use]
pub fn product(chrome: &Chrome<'_>, book: &Book, owned: bool, in_cart: bool) -> Markup {
    let store = chrome.store;
    page(
        chrome,
        &book.full_title(),
        html! {
            div.product {
                div.cover-box { (cover_img(book)) }
                div.info {
                    h1 #book-title { (book.full_title()) }
                    p #book-author { (book.author) " — " (book.year) }
                    p { (book.description) }
                    p.price #book-price { (format_price(book.price_cents)) }
                    p.muted {
                        @if store.offers_downloads() {
                            "EPUB sans DRM, marqué à votre nom (tatouage numérique visible)."
                        } @else {
                            "Ebook ajouté à votre compte ; il se synchronise sur votre liseuse et vos applications."
                        }
                    }
                    p.muted { (ISBN_LABEL) " : " span #book-isbn { (book.isbn13) } }
                    @if owned {
                        p.notice.ok #already-owned { "Ce titre est déjà dans votre bibliothèque. Vous pouvez toujours l'offrir." }
                    }
                    @if in_cart {
                        p { a.button #go-to-cart href="/panier" { "Voir le panier" } }
                    } @else {
                        form #add-to-cart-form method="post" action="/panier/ajouter" {
                            input type="hidden" name="book" value=(book.id);
                            button #add-to-cart type="submit" { "Ajouter au panier" }
                        }
                    }
                }
            }
        },
    )
}

fn lines_table(books: &[&'static Book], removable: bool) -> Markup {
    let total: u32 = books.iter().map(|b| b.price_cents).sum();
    html! {
        table.lines #cart {
            thead { tr { th { "Titre" } th { "Auteur" } th.num { "Prix" } @if removable { th {} } } }
            tbody {
                @for book in books {
                    tr id=(format!("line-{}", book.id)) {
                        td { a href=(format!("/livre/{}", book.id)) { (book.full_title()) } }
                        td { (book.author) }
                        td.num { (format_price(book.price_cents)) }
                        @if removable {
                            td {
                                form method="post" action="/panier/retirer" {
                                    input type="hidden" name="book" value=(book.id);
                                    button.link type="submit" id=(format!("remove-{}", book.id)) { "Retirer" }
                                }
                            }
                        }
                    }
                }
            }
            tfoot {
                tr {
                    th colspan="2" { "Total livres (TVA 5,5 % incluse)" }
                    th.num #cart-total { (format_price(total)) }
                    @if removable { th {} }
                }
            }
        }
    }
}

/// The cart.
#[must_use]
pub fn cart(chrome: &Chrome<'_>, books: &[&'static Book]) -> Markup {
    page(
        chrome,
        "Panier",
        html! {
            h1 { "Votre panier" }
            @if books.is_empty() {
                p #cart-empty { "Votre panier est vide." }
                p { a href="/" { "Retour au catalogue" } }
            } @else {
                (lines_table(books, true))
                p { a.button #checkout-link href="/commande" { "Passer la commande" } }
            }
        },
    )
}

/// The sign-in / create-account page. `next` is where to go afterwards (already vetted).
#[must_use]
pub fn auth(
    chrome: &Chrome<'_>,
    next: &str,
    error: Option<&str>,
    email: &str,
    intro: Option<Markup>,
) -> Markup {
    page(
        chrome,
        "Connexion",
        html! {
            h1 { "Identifiez-vous" }
            @if let Some(intro) = intro { (intro) }
            @if let Some(error) = error {
                p.notice.error #auth-error role="alert" { (error) }
            }
            div.cols {
                form #login-form method="post" action="/compte/connexion" {
                    h2 { "J'ai déjà un compte" }
                    input type="hidden" name="next" value=(next);
                    label for="login-email" { "Adresse e-mail" }
                    input #login-email type="email" name="email" value=(email) required autocomplete="username";
                    label for="login-password" { "Mot de passe" }
                    input #login-password type="password" name="password" required autocomplete="current-password";
                    p { button #login-submit type="submit" { "Me connecter" } }
                }
                form #register-form method="post" action="/compte/inscription" {
                    h2 { "Je crée mon compte" }
                    input type="hidden" name="next" value=(next);
                    label for="register-email" { "Adresse e-mail" }
                    input #register-email type="email" name="email" value=(email) required autocomplete="username";
                    label for="register-password" { "Mot de passe (8 caractères au moins)" }
                    input #register-password type="password" name="password" required minlength="8" autocomplete="new-password";
                    p { button #register-submit type="submit" { "Créer mon compte" } }
                    p.muted { "Compte de démonstration : rien n'est conservé au redémarrage, n'utilisez pas un vrai mot de passe." }
                }
            }
        },
    )
}

/// Values typed in the checkout form (kept to re-fill it after a validation error).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckoutForm {
    /// Gift checkbox.
    pub gift: bool,
    /// Buyer's name for the card.
    pub giver_name: String,
    /// Recipient's name.
    pub recipient_name: String,
    /// Personal message.
    pub message: String,
    /// `home` or `post`.
    pub card_delivery: String,
    /// Addressee of a posted card.
    pub ship_name: String,
    /// Street.
    pub ship_street: String,
    /// Postal code.
    pub ship_postcode: String,
    /// City.
    pub ship_city: String,
}

/// What the checkout page can offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckoutOffer {
    /// Whether the gift option is available (partner API configured).
    pub gift_available: bool,
    /// Price of one posted card; `None` when the printer is not available.
    pub post_price_cents: Option<u32>,
    /// How gifts are delivered by this store.
    pub delivery: GiftDelivery,
}

/// The checkout page.
#[must_use]
pub fn checkout(
    chrome: &Chrome<'_>,
    books: &[&'static Book],
    form: &CheckoutForm,
    errors: &[String],
    offer: &CheckoutOffer,
) -> Markup {
    let store = chrome.store;
    let label =
        if books.len() > 1 { store.gift_option_label_plural() } else { store.gift_option_label() };
    let post_selected = form.card_delivery == "post" && offer.post_price_cents.is_some();
    page(
        chrome,
        "Commande",
        html! {
            h1 { "Votre commande" }
            @if !errors.is_empty() {
                div.notice.error #checkout-errors role="alert" {
                    ul { @for error in errors { li { (error) } } }
                }
            }
            (lines_table(books, false))
            form #checkout-form method="post" action="/commande/payer" {
                fieldset #gift-block {
                    legend { "C'est pour offrir ?" }
                    @if offer.gift_available {
                        p {
                            input #gift-option type="checkbox" name="gift" value="1" checked[form.gift];
                            " "
                            label.inline for="gift-option" { (label) }
                        }
                        p.muted #gift-option-help { (GIFT_OPTION_HELP) }
                        p.muted #gift-delivery-note {
                            @match offer.delivery {
                                GiftDelivery::PartnerFile => {
                                    "La personne qui reçoit la carte télécharge le livre sans créer de compte."
                                }
                                GiftDelivery::ResellerRedirect => {
                                    "La personne qui reçoit la carte ajoute le livre à son compte "
                                    (store.display_name()) " (elle peut le créer à ce moment-là)."
                                }
                            }
                        }
                        label for="giver-name" { "De la part de" }
                        input #giver-name type="text" name="giver-name" maxlength="80" value=(form.giver_name);
                        label for="recipient-name" { "Pour" }
                        input #recipient-name type="text" name="recipient-name" maxlength="80" value=(form.recipient_name);
                        label for="gift-message" { "Votre message (facultatif)" }
                        textarea #gift-message name="gift-message" maxlength="500" rows="3" { (form.message) }
                        p {
                            input #card-delivery-home type="radio" name="card-delivery" value="home" checked[!post_selected];
                            " "
                            label.inline for="card-delivery-home" { (CARD_HOME_LABEL) }
                        }
                        p {
                            input #card-delivery-post type="radio" name="card-delivery" value="post"
                                checked[post_selected] disabled[offer.post_price_cents.is_none()];
                            " "
                            label.inline for="card-delivery-post" {
                                (CARD_POST_LABEL)
                                @if let Some(price) = offer.post_price_cents {
                                    " — " span #card-post-price { (format_price(price)) }
                                    " par carte (TVA 20 % incluse), en France"
                                }
                            }
                        }
                        @if offer.post_price_cents.is_none() {
                            p.muted #card-post-unavailable { "L'envoi postal est indisponible pour le moment." }
                        } @else {
                            div #ship-fields {
                                p.muted { "Adresse d'envoi de la carte (uniquement pour l'envoi postal) :" }
                                label for="ship-name" { "Destinataire du courrier" }
                                input #ship-name type="text" name="ship-name" maxlength="80" value=(form.ship_name) autocomplete="name";
                                label for="ship-street" { "Adresse" }
                                input #ship-street type="text" name="ship-street" maxlength="120" value=(form.ship_street) autocomplete="street-address";
                                label for="ship-postcode" { "Code postal" }
                                input #ship-postcode type="text" name="ship-postcode" maxlength="10" value=(form.ship_postcode) autocomplete="postal-code";
                                label for="ship-city" { "Ville" }
                                input #ship-city type="text" name="ship-city" maxlength="80" value=(form.ship_city) autocomplete="address-level2";
                            }
                        }
                    } @else {
                        p.muted #gift-unavailable { "L'option cadeau n'est pas disponible sur cette boutique pour le moment." }
                    }
                }
                p.muted { "Paiement simulé : aucune carte bancaire n'est demandée." }
                p { button #pay type="submit" { "Payer la commande" } }
            }
        },
    )
}

/// French label of a gift status.
#[must_use]
pub const fn gift_status_label(status: GiftStatus) -> &'static str {
    match status {
        GiftStatus::Pending => "En préparation",
        GiftStatus::Created => "Prêt à offrir — pas encore utilisé",
        GiftStatus::Redeemed => "Utilisé par son destinataire",
        GiftStatus::Exhausted => "Utilisé — limite de téléchargements atteinte",
        GiftStatus::Fulfilled => "Ajouté à la bibliothèque de son destinataire",
        GiftStatus::Cancelled => "Annulé et remboursé",
        GiftStatus::Expired => "Expiré",
    }
}

/// French label of a print order status.
#[must_use]
pub fn print_status_label(status: &str) -> &'static str {
    match status {
        "pending" => "En attente d'envoi à l'imprimeur",
        "accepted" => "Fichier accepté par l'imprimeur",
        "printing" => "En cours d'impression",
        "shipped" => "Expédiée",
        "cancelled" => "Annulée par l'imprimeur",
        "rejected" => "Fichier refusé par l'imprimeur",
        _ => "Statut inconnu",
    }
}

/// Returns the URL when it is a plain http(s) link that is safe to put in an `href`.
#[must_use]
pub fn safe_http_url(url: &str) -> Option<&str> {
    let lower = url.trim_start().to_ascii_lowercase();
    ((lower.starts_with("http://") || lower.starts_with("https://"))
        && !url.chars().any(char::is_control))
    .then_some(url)
}

fn gift_block(order: &Order, index: usize, book: Option<&Book>, gift: &GiftRecord) -> Markup {
    // The first gift of an order carries the bare ids; further ones get a numeric suffix.
    let id = |name: &str| {
        if index == 0 { name.to_owned() } else { format!("{name}-{}", index + 1) }
    };
    let pdf_href = format!("/commande/{}/carte-{}.pdf", order.reference, index + 1);
    html! {
        section.gift id=(id("gift")) {
            h2 { "Cadeau : " (book.map_or_else(|| "livre".to_owned(), Book::full_title)) }
            p {
                "Statut : "
                strong id=(id("gift-status")) data-status=(gift.status.as_str()) { (gift_status_label(gift.status)) }
                @if let Some(n) = gift.downloads.filter(|n| *n > 0) {
                    " — " span id=(id("gift-downloads")) { (n) } " téléchargement(s)"
                }
            }
            @if let Some(error) = &gift.last_error {
                p.notice.error id=(id("gift-error")) role="alert" { (error) }
            }
            @match &gift.card {
                CardState::Ready(_) => {
                    p {
                        a.button id=(id("card-pdf")) href=(pdf_href) { "Télécharger la carte à imprimer (PDF)" }
                    }
                }
                CardState::Unavailable => {
                    p.notice id=(id("card-unavailable")) {
                        "La carte à imprimer n'est pas disponible pour le moment. "
                        "Vous pouvez transmettre le lien et le code ci-dessous."
                    }
                }
                CardState::Pending => {}
            }
            @if let Some(claim_url) = &gift.claim_url {
                p {
                    "Lien du cadeau : "
                    @if let Some(url) = safe_http_url(claim_url) {
                        a id=(id("claim-url")) href=(url) rel="noreferrer" { (url) }
                    } @else {
                        span id=(id("claim-url")) { (claim_url) }
                    }
                }
            }
            @if let Some(code) = &gift.code_display {
                p { "Code : " span.code id=(id("claim-code")) { (code) } }
            }
            @if gift.reissues > 0 {
                p.notice id=(id("code-reissued")) {
                    "Ce code remplace le précédent, qui ne fonctionne plus. "
                    "Une carte déjà imprimée doit être réimprimée."
                }
            }
            @if order.status == OrderStatus::Paid && gift.gift_id.is_some()
                && gift.status.is_claimable() && gift.activated_by.is_none()
            {
                form id=(id("reissue-form")) method="post" action=(format!("/commande/{}/nouveau-code", order.reference)) {
                    input type="hidden" name="gift" value=(index + 1);
                    p.muted { "Carte perdue ? Un nouveau code annule l'ancien ; le livre reste offert." }
                    button.link id=(id("reissue-gift")) type="submit" { "Obtenir un nouveau code" }
                }
            }
            @if let Some(print) = &gift.print {
                (print_block(print, &id))
            }
        }
    }
}

fn print_block(print: &PrintRecord, id: &dyn Fn(&str) -> String) -> Markup {
    html! {
        p {
            "Carte imprimée : "
            strong id=(id("print-status")) data-status=(print.status) { (print_status_label(&print.status)) }
            @if let Some(tracking) = &print.tracking {
                " — suivi " span id=(id("print-tracking")) { (tracking) }
            }
        }
        @if !print.findings.is_empty() {
            ul id=(id("print-findings")) {
                @for finding in &print.findings { li { (finding) } }
            }
        }
    }
}

/// The order confirmation page.
#[must_use]
pub fn order(chrome: &Chrome<'_>, order: &Order, notice: Option<&str>) -> Markup {
    let store = chrome.store;
    let gifts: Vec<(usize, &GiftRecord)> =
        order.lines.iter().filter_map(|l| l.gift.as_ref()).enumerate().collect();
    let needs_retry = order.status == OrderStatus::Paid
        && gifts.iter().any(|(_, g)| {
            !g.status.is_terminal()
                && (g.gift_id.is_none()
                    || !g.cover_sent
                    || g.card == CardState::Pending
                    || g.print.as_ref().is_some_and(|p| p.status == "pending"))
        });
    let cancellable = order.status == OrderStatus::Paid
        && !gifts.is_empty()
        && gifts.iter().all(|(_, g)| {
            matches!(g.status, GiftStatus::Pending | GiftStatus::Created)
                && g.activated_by.is_none()
        });
    let books_total: u32 = order.lines.iter().map(|l| l.price_cents).sum();
    page(
        chrome,
        "Confirmation de commande",
        html! {
            h1 { "Merci pour votre commande" }
            @if let Some(notice) = notice {
                p.notice.error #order-notice role="alert" { (notice) }
            }
            p {
                "Commande " strong #order-ref { (order.reference) }
                " du " (french_date(order.created_at)) " — "
                span #order-status data-status=(if order.status == OrderStatus::Paid { "paid" } else { "refunded" }) {
                    @if order.status == OrderStatus::Paid { "payée" } @else { "remboursée" }
                }
            }
            table.lines #order-lines {
                tbody {
                    @for line in &order.lines {
                        @let book = catalogue::find(line.book_id);
                        tr {
                            td { (book.map_or_else(|| line.book_id.to_owned(), Book::full_title)) }
                            td.num { (format_price(line.price_cents)) }
                        }
                    }
                    tr { td.muted { "Sous-total livres (TVA 5,5 % incluse)" } td.num { (format_price(books_total)) } }
                    @if let Some(details) = &order.gift {
                        @if details.card_delivery == CardDelivery::Post {
                            tr #order-card-line {
                                td { (CARD_POST_LABEL) " × " (order.lines.len()) " (TVA 20 % incluse)" }
                                td.num { (format_price(order.total_cents.saturating_sub(books_total))) }
                            }
                        } @else {
                            tr #order-card-line { td { (CARD_HOME_LABEL) } td.num { (format_price(0)) } }
                        }
                    }
                }
                tfoot { tr { th { "Total payé" } th.num #order-total { (format_price(order.total_cents)) } } }
            }
            @if let Some(details) = &order.gift {
                p #gift-summary {
                    "Offert à " strong #gift-recipient { (details.recipient_name) }
                    " de la part de " strong #gift-giver { (details.giver_name) } "."
                }
                @if !details.message.is_empty() {
                    blockquote #gift-message-text { (details.message) }
                }
                @for (index, gift) in &gifts {
                    @let book = order.lines.iter().filter(|l| l.gift.is_some()).nth(*index).and_then(|l| catalogue::find(l.book_id));
                    (gift_block(order, *index, book, gift))
                }
                @if needs_retry {
                    form method="post" action=(format!("/commande/{}/relancer", order.reference)) {
                        button #retry type="submit" { "Réessayer" }
                    }
                }
                @if cancellable {
                    form method="post" action=(format!("/commande/{}/annuler", order.reference)) {
                        p.muted { "Tant que le cadeau n'a pas été utilisé, vous pouvez l'annuler et être remboursé." }
                        button.link #cancel-gift type="submit" { "Annuler le cadeau" }
                    }
                }
            } @else {
                p.notice.ok #purchase-done {
                    @if store.offers_downloads() {
                        "Vos livres sont disponibles au téléchargement dans votre bibliothèque."
                    } @else {
                        "Vos Ebooks ont été ajoutés à votre compte : ils apparaîtront sur votre liseuse à la prochaine synchronisation."
                    }
                }
                p { a.button #library-link href="/bibliotheque" { "Ma bibliothèque" } }
            }
        },
    )
}

/// The list of an account's orders.
#[must_use]
pub fn orders(chrome: &Chrome<'_>, orders: &[Order]) -> Markup {
    page(
        chrome,
        "Mes commandes",
        html! {
            h1 { "Mes commandes" }
            @if orders.is_empty() {
                p #orders-empty { "Vous n'avez pas encore passé de commande." }
            } @else {
                table.lines #orders {
                    tbody {
                        @for order in orders {
                            tr {
                                td { a href=(format!("/commande/{}", order.reference)) { (order.reference) } }
                                td { (french_date(order.created_at)) }
                                td { @if order.gift.is_some() { "cadeau" } @else { "pour moi" } }
                                td.num { (format_price(order.total_cents)) }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// The account's library.
#[must_use]
pub fn library(chrome: &Chrome<'_>, entries: &[LibraryEntry]) -> Markup {
    let store = chrome.store;
    page(
        chrome,
        "Ma bibliothèque",
        html! {
            h1 { "Ma bibliothèque" }
            @if entries.is_empty() {
                p #library-empty { "Votre bibliothèque est vide." }
            } @else {
                @if !store.offers_downloads() {
                    p.notice #sync-note {
                        "Vos Ebooks sont liés à votre compte : ils ne se téléchargent pas depuis ce site. "
                        "Synchronisez votre liseuse ou votre application de lecture pour les retrouver."
                    }
                }
                table.lines #library {
                    tbody {
                        @for entry in entries {
                            @if let Some(book) = catalogue::find(entry.book_id) {
                                tr id=(format!("owned-{}", book.id)) {
                                    td { (book.full_title()) }
                                    td { (book.author) }
                                    td.muted {
                                        @if let Some(gift) = &entry.gift {
                                            "Offert par " (gift.giver_name)
                                        } @else {
                                            "Acheté le " (french_date(entry.acquired_at))
                                        }
                                    }
                                    td {
                                        @if store.offers_downloads() {
                                            a id=(format!("download-{}", book.id)) href=(format!("/bibliotheque/{}.epub", book.id)) {
                                                "Télécharger (EPUB)"
                                            }
                                        } @else {
                                            span.muted { "Sur votre liseuse" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Introduction shown above the sign-in forms when a gift is waiting.
#[must_use]
pub fn activation_intro(book: &Book, giver_name: &str) -> Markup {
    html! {
        div.notice.ok #activation {
            p {
                strong #activation-giver { (giver_name) } " vous offre "
                strong #activation-book { (book.full_title()) } " de " (book.author) "."
            }
            p { "Connectez-vous ou créez un compte pour l'ajouter à votre bibliothèque." }
        }
    }
}

/// Confirmation page shown to a signed-in recipient before the book is added.
#[must_use]
pub fn activation_confirm(
    chrome: &Chrome<'_>,
    book: &Book,
    giver_name: &str,
    already_owned: bool,
) -> Markup {
    page(
        chrome,
        "Votre cadeau",
        html! {
            h1 { "Un livre vous est offert" }
            div.product #activation {
                div.cover-box { (cover_img(book)) }
                div.info {
                    p {
                        strong #activation-giver { (giver_name) } " vous offre "
                        strong #activation-book { (book.full_title()) } " de " (book.author) "."
                    }
                    @if already_owned {
                        p.notice #activation-already-owned {
                            "Ce titre est déjà dans votre bibliothèque. Vous pouvez tout de même accepter le cadeau."
                        }
                    }
                    p { "Il sera ajouté au compte " strong #activation-account { (chrome.account.unwrap_or("")) } "." }
                    form #activation-form method="post" action="/cadeau/activer" {
                        button #activate type="submit" { "Ajouter à ma bibliothèque" }
                    }
                    form method="post" action="/compte/deconnexion" {
                        input type="hidden" name="next" value="/cadeau/activer";
                        p.muted { "Ce n'est pas vous ? " button.link #activation-switch type="submit" { "Changer de compte" } }
                    }
                }
            }
        },
    )
}

/// Page shown once the gifted book is in the recipient's library.
#[must_use]
pub fn activation_done(chrome: &Chrome<'_>, book: &Book, already_owned: bool) -> Markup {
    let store = chrome.store;
    page(
        chrome,
        "Cadeau ajouté",
        html! {
            h1 { "C'est ajouté !" }
            div.notice.ok #activation-done {
                p { strong #activation-book { (book.full_title()) } " est dans votre bibliothèque." }
                @if already_owned {
                    p { "Vous possédiez déjà ce titre : votre bibliothèque est inchangée." }
                }
                @if store.offers_downloads() {
                    p { a #activation-download href=(format!("/bibliotheque/{}.epub", book.id)) { "Télécharger le livre (EPUB)" } }
                } @else {
                    p #activation-sync {
                        "Allumez votre liseuse et lancez une synchronisation : le livre apparaît dans votre bibliothèque. "
                        "Il est aussi disponible dans l'application de lecture, avec le même compte."
                    }
                }
            }
            p { a.button #library-link href="/bibliotheque" { "Ma bibliothèque" } }
        },
    )
}

/// An error page. `id` identifies the problem for tests (e.g. `activation-error`).
#[must_use]
pub fn problem(chrome: &Chrome<'_>, id: &str, code: &str, title: &str, message: &str) -> Markup {
    page(
        chrome,
        title,
        html! {
            h1 { (title) }
            p.notice.error id=(id) data-code=(code) role="alert" { (message) }
            p { a href="/" { "Retour au catalogue" } }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_http_links_are_linked() {
        assert_eq!(safe_http_url("https://ebk.example/k/ABC"), Some("https://ebk.example/k/ABC"));
        assert_eq!(safe_http_url("HTTP://x"), Some("HTTP://x"));
        assert_eq!(safe_http_url("javascript:alert(1)"), None);
        assert_eq!(safe_http_url("data:text/html,x"), None);
        assert_eq!(safe_http_url("//evil.example"), None);
        assert_eq!(safe_http_url("https://x\n.example"), None);
        assert_eq!(safe_http_url(""), None);
    }

    #[test]
    fn every_page_carries_the_banner_and_escapes_text() {
        let chrome = Chrome {
            store: StoreKind::Septante,
            account: Some("<b>x</b>@y.example"),
            cart_count: 2,
        };
        let html = problem(&chrome, "oops", "c", "Titre <i>", "Message & <script>").into_string();
        assert!(html.contains(DEMO_BANNER));
        assert!(html.contains("lang=\"fr\""));
        assert!(html.contains("&lt;b&gt;x&lt;/b&gt;@y.example"));
        assert!(html.contains("Message &amp; &lt;script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<i>"));
    }

    #[test]
    fn wording_follows_the_store() {
        assert_eq!(StoreKind::Septante.gift_option_label(), "Offrir cet ebook");
        assert_eq!(StoreKind::GrandeEnseigne.gift_option_label(), "Offrir cet Ebook");
        assert!(StoreKind::Septante.gift_option_label_plural().contains("une carte par livre"));
    }
}
