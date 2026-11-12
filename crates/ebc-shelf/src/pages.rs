//! The pages, rendered with `maud` (every value is escaped by the template engine).
//! No script anywhere: every essential page is a plain HTML form. One small same-origin
//! style sheet. Element `id`s are used by the tests and listed in the README.

use ebc_epub::RightsBasis;
use ebc_model::Locale;
use maud::{DOCTYPE, Markup, html};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::clock::{iso_date, iso_minute};
use crate::config::{Config, Mode};
use crate::service::{LICENCES, size_text};
use crate::store::{
    Account, AccountStatus, Appeal, BlockEntry, BlockKind, Book, Gift, GiftState, Identity, Notice,
    NoticeCategory, NoticeDecision, PartyKind, Removal, Role,
};
use crate::text::{S, s, sf};

/// The style sheet served at `/static/shelf.css`.
pub(crate) const STYLE: &str = "\
html{font-family:Georgia,serif;font-size:17px;line-height:1.45;color:#111;background:#fff}\n\
body{margin:0 auto;padding:12px 16px;max-width:46em}\n\
header{border-bottom:2px solid #111;padding-bottom:6px;margin-bottom:12px}\n\
header a.name{font-weight:bold;font-size:1.2em;text-decoration:none;color:#111}\n\
nav a,nav form{display:inline-block;margin:4px 14px 4px 0}\n\
h1{font-size:1.5em;margin:.4em 0}\n\
h2{font-size:1.2em;margin:1.3em 0 .4em}\n\
a{color:#0a3d91}\n\
label{display:block;margin-top:.7em}\n\
input[type=text],input[type=email],input[type=number],input[type=date],textarea,select{font-size:1em;padding:6px;border:1px solid #333;width:100%;max-width:32em;box-sizing:border-box;display:block;margin:4px 0}\n\
textarea{min-height:6em}\n\
fieldset{border:1px solid #888;margin:12px 0;padding:6px 12px}\n\
label.choice{display:block;margin:6px 0}\n\
button{font-size:1em;font-weight:bold;padding:9px 14px;border:2px solid #111;background:#111;color:#fff;margin:8px 8px 8px 0;cursor:pointer}\n\
button.alt{background:#fff;color:#111}\n\
nav button{padding:2px 8px;margin:0;font-weight:normal}\n\
.note{font-size:.92em;color:#333}\n\
.err{border:2px solid #a00;color:#a00;padding:8px;font-weight:bold}\n\
.ok{border:2px solid #060;color:#060;padding:8px}\n\
.draft{border:2px dashed #a00;padding:6px;font-weight:bold}\n\
.code{font-family:monospace;font-size:1.15em;font-weight:bold;letter-spacing:.06em}\n\
table{border-collapse:collapse;width:100%}\n\
td,th{border-bottom:1px solid #bbb;padding:6px 4px;text-align:left;vertical-align:top}\n\
.inline{display:inline}\n\
footer{margin-top:30px;border-top:1px solid #111;padding-top:8px;font-size:.9em}\n\
footer a{margin-right:1em;display:inline-block;padding:4px 0}\n";

/// Characters escaped in a URL fragment value.
const FRAGMENT_VALUE: &percent_encoding::AsciiSet = NON_ALPHANUMERIC;

/// What every page needs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Chrome<'a> {
    pub(crate) locale: Locale,
    pub(crate) config: &'a Config,
    pub(crate) account: Option<&'a Account>,
    pub(crate) csrf: Option<&'a str>,
    pub(crate) path: &'a str,
}

impl Chrome<'_> {
    fn t(&self, msg: S) -> &'static str {
        s(self.locale, msg)
    }

    fn csrf_field(&self) -> Markup {
        html! { @if let Some(token) = self.csrf { input type="hidden" name="csrf" value=(token); } }
    }
}

/// The page skeleton.
pub(crate) fn layout(chrome: Chrome<'_>, title: &str, body: Markup) -> String {
    let locale = chrome.locale;
    html! {
        (DOCTYPE)
        html lang=(locale.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="robots" content="noindex, nofollow";
                title { (title) " — " (chrome.config.name) }
                link rel="stylesheet" href="/static/shelf.css";
            }
            body {
                header {
                    a.name href="/" { (chrome.config.name) }
                    nav {
                        @if let Some(account) = chrome.account {
                            a id="nav-account" href="/compte" { (chrome.t(S::NavAccount)) }
                            @if account.role == Role::Operator {
                                a id="nav-moderation" href="/moderation" { (chrome.t(S::NavModeration)) }
                            }
                            form.inline method="post" action="/deconnexion" {
                                (chrome.csrf_field())
                                button.alt id="signout" type="submit" { (chrome.t(S::NavSignOut)) }
                            }
                        } @else {
                            a id="nav-signin" href="/connexion" { (chrome.t(S::NavSignIn)) }
                        }
                        a id="nav-notice" href="/signaler" { (chrome.t(S::NavNotice)) }
                    }
                }
                main { (body) }
                footer {
                    a href="/conditions" { (chrome.t(S::NavTerms)) }
                    a href="/mentions-legales" { (chrome.t(S::NavLegal)) }
                    a href="/contact" { (chrome.t(S::NavContact)) }
                    a href="/signaler" { (chrome.t(S::NavNotice)) }
                    br;
                    @for other in Locale::ALL {
                        @if other == locale {
                            strong { (other.code().to_uppercase()) } " "
                        } @else {
                            a lang=(other.code()) href=(format!("{}?lang={}", chrome.path, other.code())) { (other.code().to_uppercase()) } " "
                        }
                    }
                }
            }
        }
    }
    .into_string()
}

/// A simple page: title, paragraphs, an optional link.
pub(crate) fn message(
    chrome: Chrome<'_>,
    title: &str,
    paragraphs: &[String],
    link: Option<(&str, &str)>,
) -> String {
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            @for paragraph in paragraphs { p id="message" { (paragraph) } }
            @if let Some((href, text)) = link { p { a id="message-link" href=(href) { (text) } } }
        },
    )
}

/// `GET /`.
pub(crate) fn home(chrome: Chrome<'_>) -> String {
    let config = chrome.config;
    layout(
        chrome,
        &config.name,
        html! {
            h1 { (config.name) }
            p id="intro" { (chrome.t(S::HomeIntro)) }
            p { (chrome.t(if config.mode == Mode::Own { S::HomeOwn } else { S::HomeMulti })) }
            p.note id="not-purchases" { (chrome.t(S::NotPurchases)) }
            p { a id="home-signin" href=(if chrome.account.is_some() { "/compte" } else { "/connexion" }) {
                (chrome.t(if chrome.account.is_some() { S::NavAccount } else { S::NavSignIn }))
            } }
            p.note { (chrome.t(S::HomeRecipient)) }
        },
    )
}

/// `GET /connexion`.
pub(crate) fn sign_in(chrome: Chrome<'_>, error: Option<&str>, prefill: &str) -> String {
    let minutes = (chrome.config.login_ttl_secs / 60).to_string();
    let title = chrome.t(S::SignInTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            @if let Some(error) = error { p.err id="error" role="alert" { (error) } }
            p { (sf(chrome.locale, S::SignInIntro, &[("minutes", &minutes)])) }
            form id="signin-form" method="post" action="/connexion" {
                label for="email" { (chrome.t(S::EmailLabel)) }
                input id="email" name="email" type="email" autocomplete="email" required value=(prefill);
                button id="signin-submit" type="submit" { (chrome.t(S::SendLink)) }
            }
        },
    )
}

/// After a sign-in request.
pub(crate) fn link_sent(chrome: Chrome<'_>) -> String {
    let minutes = (chrome.config.login_ttl_secs / 60).to_string();
    let title = chrome.t(S::LinkSentTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p id="link-sent" { (sf(chrome.locale, S::LinkSent, &[("minutes", &minutes)])) }
        },
    )
}

/// `GET /connexion/{token}`: a button, so that mail scanners following links do not use
/// the link up.
pub(crate) fn confirm_sign_in(chrome: Chrome<'_>, token: &str, email: &str) -> String {
    let title = chrome.t(S::SignInTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p { (sf(chrome.locale, S::ConfirmIntro, &[("email", email)])) }
            form id="confirm-form" method="post" action=(format!("/connexion/{token}")) {
                button id="confirm-signin" type="submit" { (chrome.t(S::ConfirmButton)) }
            }
        },
    )
}

fn removal_text(chrome: Chrome<'_>, removal: &Removal) -> &'static str {
    chrome.t(match removal {
        Removal::Deleted => S::RemovedDeleted,
        Removal::TakenDown { .. } => S::RemovedTakenDown,
        Removal::Expired => S::RemovedExpired,
        Removal::AccountClosed => S::RemovedClosed,
    })
}

/// `GET /compte`.
pub(crate) fn account(
    chrome: Chrome<'_>,
    account: &Account,
    books: &[Book],
    decisions: &[Notice],
    used: u64,
) -> String {
    let title = chrome.t(S::NavAccount);
    let locale = chrome.locale;
    let can_publish = crate::service::may_publish(account);
    layout(
        chrome,
        title,
        html! {
            h1 id="account" { (title) }
            p { (sf(locale, S::SignedInAs, &[("email", &account.email)])) }
            @if matches!(account.status, AccountStatus::Suspended { .. }) {
                p.err id="suspended" { (chrome.t(S::Suspended)) }
            }
            @if account.identity.is_none() {
                p.err id="identity-missing" { (chrome.t(S::IdentityMissing)) }
                p { a id="identity-link" href="/compte/identite" { (chrome.t(S::CompleteIdentity)) } }
            } @else {
                p { a id="identity-link" href="/compte/identite" { (chrome.t(S::EditIdentity)) } }
            }
            h2 { (chrome.t(S::MyBooks)) }
            @if can_publish {
                p { a id="upload-link" href="/livres/nouveau" { (chrome.t(S::UploadBook)) } }
            }
            p.note id="storage" { (sf(locale, S::StorageUsed, &[("used", &size_text(locale, used)), ("quota", &size_text(locale, chrome.config.account_quota_bytes))])) }
            @if books.is_empty() {
                p { (chrome.t(S::NoBooks)) }
            } @else {
                ul id="books" {
                    @for book in books {
                        li {
                            a id=(format!("book-{}", book.id)) href=(format!("/livres/{}", book.id)) { (book.title) }
                            " — " (book.author)
                            @if let Some(removal) = &book.removed { " (" (removal_text(chrome, removal)) ")" }
                        }
                    }
                }
            }
            @if !decisions.is_empty() {
                h2 { (chrome.t(S::Decisions)) }
                ul id="decisions" {
                    @for notice in decisions {
                        li { a id=(format!("decision-{}", notice.id)) href=(format!("/compte/decisions/{}", notice.id)) {
                            (sf(locale, S::DecisionTitle, &[("id", &notice.id)]))
                        } }
                    }
                }
            }
            h2 { (chrome.t(S::CloseAccount)) }
            p.note { (chrome.t(S::CloseAccountHelp)) }
            form id="close-form" method="post" action="/compte/fermer" {
                (chrome.csrf_field())
                label.choice { input id="close-confirm" type="checkbox" name="confirm" value="1"; " " (chrome.t(S::CloseConfirm)) }
                button.alt id="close-submit" type="submit" { (chrome.t(S::CloseAccount)) }
            }
        },
    )
}

/// `GET /compte/identite`.
pub(crate) fn identity_form(
    chrome: Chrome<'_>,
    identity: Option<&Identity>,
    error: Option<&str>,
) -> String {
    let title = chrome.t(S::IdentityTitle);
    let value = |f: fn(&Identity) -> Option<&str>| identity.and_then(f).unwrap_or("").to_owned();
    let organisation = identity.is_some_and(|i| i.kind == PartyKind::Organisation);
    let optional = chrome.t(S::Optional);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            @if let Some(error) = error { p.err id="error" role="alert" { (error) } }
            p.note { (chrome.t(S::IdentityIntro)) }
            form id="identity-form" method="post" action="/compte/identite" {
                (chrome.csrf_field())
                fieldset {
                    legend { (chrome.t(S::KindLegend)) }
                    label.choice { input id="kind-person" type="radio" name="kind" value="person" checked[!organisation]; " " (chrome.t(S::KindPerson)) }
                    label.choice { input id="kind-organisation" type="radio" name="kind" value="organisation" checked[organisation]; " " (chrome.t(S::KindOrganisation)) }
                }
                label for="name" { (chrome.t(S::NameLabel)) }
                input id="name" name="name" type="text" required maxlength="200" value=(value(|i| Some(i.name.as_str())));
                label for="representative" { (chrome.t(S::RepresentativeLabel)) }
                input id="representative" name="representative" type="text" maxlength="200" value=(value(|i| i.representative.as_deref()));
                label for="address" { (chrome.t(S::AddressLabel)) }
                textarea id="address" name="address" required maxlength="500" { (value(|i| Some(i.postal_address.as_str()))) }
                label for="phone" { (chrome.t(S::PhoneLabel)) " " (optional) }
                input id="phone" name="phone" type="text" maxlength="40" value=(value(|i| i.phone.as_deref()));
                label for="birth_date" { (chrome.t(S::BirthDateLabel)) " " (optional) }
                input id="birth_date" name="birth_date" type="text" placeholder="AAAA-MM-JJ" maxlength="10" value=(value(|i| i.birth_date.as_deref()));
                label for="birth_place" { (chrome.t(S::BirthPlaceLabel)) " " (optional) }
                input id="birth_place" name="birth_place" type="text" maxlength="200" value=(value(|i| i.birth_place.as_deref()));
                button id="identity-submit" type="submit" { (chrome.t(S::Save)) }
            }
        },
    )
}

/// Values typed in the upload form, shown again after a refusal.
#[derive(Debug, Clone, Default)]
pub(crate) struct UploadValues {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) isbn: String,
    pub(crate) language: String,
    pub(crate) basis: String,
    pub(crate) death_year: String,
    pub(crate) licence: String,
    pub(crate) licensor: String,
}

/// `GET /livres/nouveau`.
pub(crate) fn upload_form(
    chrome: Chrome<'_>,
    values: &UploadValues,
    error: Option<&str>,
) -> String {
    let title = chrome.t(S::UploadBook);
    let max = size_text(chrome.locale, chrome.config.max_file_bytes);
    let optional = chrome.t(S::Optional);
    let bases = [
        (RightsBasis::OwnWork, S::BasisOwnWork),
        (RightsBasis::PublicDomain, S::BasisPublicDomain),
        (RightsBasis::OpenLicence, S::BasisOpenLicence),
        (RightsBasis::LicensedDistributor, S::BasisLicensedDistributor),
    ];
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            @if let Some(error) = error { p.err id="error" role="alert" { (error) } }
            p.note id="not-purchases" { (chrome.t(S::NotPurchases)) }
            form id="upload-form" method="post" action="/livres" enctype="multipart/form-data" {
                @if let Some(token) = chrome.csrf { input id="upload-csrf" type="hidden" name="csrf" value=(token); }
                label for="file" { (chrome.t(S::FileLabel)) }
                input id="file" name="file" type="file" required accept=".epub,.pdf,application/epub+zip,application/pdf";
                p.note { (sf(chrome.locale, S::UploadLimits, &[("max", &max)])) }
                label for="title" { (chrome.t(S::TitleLabel)) }
                input id="title" name="title" type="text" required maxlength="300" value=(values.title);
                label for="author" { (chrome.t(S::AuthorLabel)) }
                input id="author" name="author" type="text" required maxlength="300" value=(values.author);
                label for="isbn" { (chrome.t(S::IsbnLabel)) " " (optional) }
                input id="isbn" name="isbn" type="text" maxlength="20" value=(values.isbn);
                label for="language" { (chrome.t(S::LanguageLabel)) " " (optional) }
                input id="language" name="language" type="text" maxlength="35" value=(values.language);
                fieldset {
                    legend { (chrome.t(S::BasisLegend)) }
                    @for (basis, text) in bases {
                        label.choice {
                            input id=(format!("basis-{}", basis.as_str())) type="radio" name="basis" value=(basis.as_str()) checked[values.basis == basis.as_str()];
                            " " (chrome.t(text))
                        }
                    }
                    label for="death_year" { (chrome.t(S::DeathYearLabel)) }
                    input id="death_year" name="death_year" type="text" inputmode="numeric" maxlength="4" value=(values.death_year);
                    label for="licence" { (chrome.t(S::LicenceLabel)) }
                    select id="licence" name="licence" {
                        option value="" { "—" }
                        @for (id, name) in LICENCES {
                            option value=(id) selected[values.licence == id] { (name) }
                        }
                    }
                    label for="licensor" { (chrome.t(S::LicensorLabel)) }
                    input id="licensor" name="licensor" type="text" maxlength="300" value=(values.licensor);
                }
                label.choice {
                    input id="declared" type="checkbox" name="declared" value="1" required;
                    " " (chrome.t(S::DeclarationText))
                }
                button id="upload-submit" type="submit" { (chrome.t(S::UploadButton)) }
            }
        },
    )
}

fn basis_text(locale: Locale, basis: RightsBasis) -> &'static str {
    s(
        locale,
        match basis {
            RightsBasis::OwnWork => S::BasisOwnWork,
            RightsBasis::PublicDomain => S::BasisPublicDomain,
            RightsBasis::OpenLicence => S::BasisOpenLicence,
            RightsBasis::LicensedDistributor => S::BasisLicensedDistributor,
        },
    )
}

fn gift_state_text(chrome: Chrome<'_>, gift: &Gift) -> &'static str {
    chrome.t(match gift.state {
        GiftState::Live => S::GiftLive,
        GiftState::Pending { .. } => S::GiftPending,
        GiftState::CancelPending { .. } | GiftState::Cancelled { .. } => S::GiftCancelled,
    })
}

/// The designer link for a gift: the core's card designer with the code, title and
/// author in the URL **fragment**, which browsers never send to a server, so the code
/// stays out of every access log (`web/README.md`, "Prefilled from a link").
pub(crate) fn designer_link(
    core_public_url: &str,
    code: &str,
    book: &Book,
    locale: Locale,
) -> String {
    let enc = |v: &str| utf8_percent_encode(v, FRAGMENT_VALUE).to_string();
    format!(
        "{}/app/#code={}&title={}&author={}&lang={}",
        core_public_url.trim_end_matches('/'),
        enc(code),
        enc(&book.title),
        enc(&book.author),
        locale.code()
    )
}

/// `GET /livres/{id}`.
pub(crate) fn book(
    chrome: Chrome<'_>,
    book: &Book,
    gifts: &[Gift],
    owner: bool,
    can_publish: bool,
    error: Option<&str>,
) -> String {
    let locale = chrome.locale;
    let max = chrome.config.max_gifts_per_request.to_string();
    let optional = chrome.t(S::Optional);
    let declaration = format!(
        "{}{}",
        basis_text(locale, book.declaration.basis),
        if book.declaration.details.is_empty() {
            String::new()
        } else {
            format!(" — {}", book.declaration.details)
        }
    );
    layout(
        chrome,
        &book.title,
        html! {
            h1 id="book-title" { (book.title) }
            p { (book.author) @if let Some(isbn) = &book.isbn13 { " — ISBN " (isbn) } }
            @if let Some(error) = error { p.err id="error" role="alert" { (error) } }
            @if let Some(removal) = &book.removed {
                p.err id="book-status" data-state="removed" { (sf(locale, S::BookRemoved, &[("reason", removal_text(chrome, removal))])) }
            } @else {
                p.note id="book-status" data-state="live" {
                    (sf(locale, S::BookFile, &[("name", &book.file_name), ("size", &size_text(locale, book.size_bytes)), ("hash", &book.sha256[..16])]))
                }
            }
            p.note id="declaration" { (sf(locale, S::BookDeclared, &[("date", &iso_minute(book.declaration.declared_at)), ("basis", &declaration)])) }
            h2 { (chrome.t(S::Gifts)) }
            @if gifts.is_empty() {
                p { (chrome.t(S::NoGifts)) }
            } @else {
                table id="gifts" {
                    @for gift in gifts {
                        tr id=(format!("gift-{}", gift.id)) data-state=(match gift.state { GiftState::Live => "live", GiftState::Pending { .. } => "pending", _ => "cancelled" }) {
                            td {
                                @if let Some(recipient) = &gift.recipient_name { (chrome.t(S::RecipientLabel)) " " (recipient) br; }
                                (gift_state_text(chrome, gift))
                                " · " (sf(locale, S::Deliveries, &[("n", &gift.deliveries.to_string())]))
                            }
                            td {
                                @if let Some(code) = &gift.code {
                                    (chrome.t(S::CodeLabel)) " " span.code id=(format!("code-{}", gift.id)) { (code) } br;
                                }
                                @if let Some(url) = &gift.claim_url { span.note id=(format!("claim-{}", gift.id)) { (url) } }
                            }
                            td {
                                @if owner && gift.is_live() {
                                    a id=(format!("card-home-{}", gift.id)) href=(format!("/cadeaux/{}/carte.pdf?profile=home-a4", gift.id)) { (chrome.t(S::CardHome)) } br;
                                    a id=(format!("card-pro-{}", gift.id)) href=(format!("/cadeaux/{}/carte.pdf?profile=pro", gift.id)) { (chrome.t(S::CardPro)) } br;
                                    @if let Some(code) = &gift.code {
                                        a id=(format!("designer-{}", gift.id)) href=(designer_link(&chrome.config.core_public_url, code, book, Locale::from_tag(&gift.locale).unwrap_or_default())) rel="noreferrer" { (chrome.t(S::Designer)) } br;
                                    }
                                    form.inline method="post" action=(format!("/cadeaux/{}/nouveau-code", gift.id)) {
                                        (chrome.csrf_field())
                                        button.alt id=(format!("reissue-{}", gift.id)) type="submit" { (chrome.t(S::Reissue)) }
                                    }
                                    form.inline method="post" action=(format!("/cadeaux/{}/annuler", gift.id)) {
                                        (chrome.csrf_field())
                                        button.alt id=(format!("cancel-{}", gift.id)) type="submit" { (chrome.t(S::CancelGift)) }
                                    }
                                }
                                @if owner && matches!(gift.state, GiftState::Pending { .. }) && book.is_live() {
                                    form.inline method="post" action=(format!("/cadeaux/{}/reessayer", gift.id)) {
                                        (chrome.csrf_field())
                                        button.alt id=(format!("retry-{}", gift.id)) type="submit" { (chrome.t(S::Retry)) }
                                    }
                                    form.inline method="post" action=(format!("/cadeaux/{}/annuler", gift.id)) {
                                        (chrome.csrf_field())
                                        button.alt id=(format!("cancel-{}", gift.id)) type="submit" { (chrome.t(S::CancelGift)) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            @if owner && can_publish && book.is_live() {
                h2 { (chrome.t(S::NewGifts)) }
                form id="gift-form" method="post" action=(format!("/livres/{}/cadeaux", book.id)) {
                    (chrome.csrf_field())
                    label for="count" { (sf(locale, S::CountLabel, &[("max", &max)])) }
                    input id="count" name="count" type="number" min="1" max=(max) value="1";
                    label for="giver" { (chrome.t(S::GiverLabel)) " " (optional) }
                    input id="giver" name="giver" type="text" maxlength="80";
                    label for="recipient" { (chrome.t(S::RecipientLabel)) " " (optional) }
                    input id="recipient" name="recipient" type="text" maxlength="80";
                    label for="message" { (chrome.t(S::MessageLabel)) " " (optional) }
                    textarea id="message" name="message" maxlength="1000" {}
                    label for="locale" { (chrome.t(S::CardLanguageLabel)) }
                    select id="locale" name="locale" {
                        @for option in Locale::ALL { option value=(option.code()) selected[option == locale] { (option.code().to_uppercase()) } }
                    }
                    label for="reveal" { (chrome.t(S::RevealLabel)) " " (optional) }
                    input id="reveal" name="reveal" type="text" placeholder="2026-12-24" maxlength="10";
                    button id="gift-submit" type="submit" { (chrome.t(S::CreateGifts)) }
                }
            }
            @if owner && book.is_live() {
                h2 { (chrome.t(S::DeleteBook)) }
                p.note { (chrome.t(S::DeleteBookHelp)) }
                form id="delete-form" method="post" action=(format!("/livres/{}/supprimer", book.id)) {
                    (chrome.csrf_field())
                    button.alt id="delete-submit" type="submit" { (chrome.t(S::DeleteBook)) }
                }
            }
        },
    )
}

/// Values typed in the notice form.
#[derive(Debug, Clone, Default)]
pub(crate) struct NoticeValues {
    pub(crate) category: String,
    pub(crate) location: String,
    pub(crate) explanation: String,
    pub(crate) name: String,
    pub(crate) email: String,
}

/// `GET /signaler`.
pub(crate) fn notice_form(
    chrome: Chrome<'_>,
    values: &NoticeValues,
    error: Option<&str>,
) -> String {
    let title = chrome.t(S::NoticeTitle);
    let other = values.category == "other";
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            @if let Some(error) = error { p.err id="error" role="alert" { (error) } }
            p { (chrome.t(S::NoticeIntro)) }
            form id="notice-form" method="post" action="/signaler" {
                fieldset {
                    legend { (chrome.t(S::CategoryLegend)) }
                    label.choice { input id="category-copyright" type="radio" name="category" value="copyright" checked[!other]; " " (chrome.t(S::CategoryCopyright)) }
                    label.choice { input id="category-other" type="radio" name="category" value="other" checked[other]; " " (chrome.t(S::CategoryOther)) }
                }
                label for="location" { (chrome.t(S::LocationLabel)) }
                input id="location" name="location" type="text" required maxlength="2000" value=(values.location);
                label for="explanation" { (chrome.t(S::ExplanationLabel)) }
                textarea id="explanation" name="explanation" required maxlength="5000" { (values.explanation) }
                label for="name" { (chrome.t(S::NotifierNameLabel)) }
                input id="name" name="name" type="text" maxlength="200" autocomplete="name" value=(values.name);
                label for="email" { (chrome.t(S::EmailLabel)) }
                input id="email" name="email" type="email" maxlength="254" autocomplete="email" value=(values.email);
                label.choice { input id="exception" type="checkbox" name="exception" value="1"; " " (chrome.t(S::ExceptionLabel)) }
                label.choice { input id="good_faith" type="checkbox" name="good_faith" value="1" required; " " (chrome.t(S::GoodFaithLabel)) }
                button id="notice-submit" type="submit" { (chrome.t(S::SendNotice)) }
            }
        },
    )
}

/// After a notice.
pub(crate) fn notice_received(chrome: Chrome<'_>, id: &str) -> String {
    let title = chrome.t(S::NoticeReceivedTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p.ok id="notice-received" { (sf(chrome.locale, S::NoticeReceived, &[("id", id)])) }
            p { "Référence / reference : " strong id="notice-id" { (id) } }
        },
    )
}

/// `GET /compte/decisions/{id}`: the statement of reasons and the complaint form.
pub(crate) fn decision(
    chrome: Chrome<'_>,
    notice: &Notice,
    title: &str,
    appeal: Option<&Appeal>,
    deadline: i64,
    now: i64,
) -> String {
    let locale = chrome.locale;
    let heading = sf(locale, S::DecisionTitle, &[("id", &notice.id)]);
    let NoticeDecision::Upheld { at, statement, .. } = &notice.decision else {
        return message(chrome, &heading, &[], None);
    };
    layout(
        chrome,
        &heading,
        html! {
            h1 { (heading) }
            p id="decision-summary" { (sf(locale, S::DecisionUpheld, &[("title", title), ("date", &iso_date(*at))])) }
            p id="decision-reasons" { (sf(locale, S::Reasons, &[("statement", statement)])) }
            @match appeal {
                Some(appeal) => {
                    @match appeal.granted {
                        None => p id="appeal-status" data-state="open" { (sf(locale, S::AppealFiled, &[("date", &iso_date(appeal.filed_at))])) },
                        Some(true) => p.ok id="appeal-status" data-state="granted" { (sf(locale, S::AppealGranted, &[("date", &iso_date(appeal.decided_at.unwrap_or(now))), ("answer", appeal.answer.as_deref().unwrap_or(""))])) },
                        Some(false) => p id="appeal-status" data-state="dismissed" { (sf(locale, S::AppealDismissed, &[("date", &iso_date(appeal.decided_at.unwrap_or(now))), ("answer", appeal.answer.as_deref().unwrap_or(""))])) },
                    }
                }
                None if now <= deadline => {
                    p { (sf(locale, S::AppealIntro, &[("date", &iso_date(deadline))])) }
                    form id="appeal-form" method="post" action=(format!("/compte/decisions/{}", notice.id)) {
                        (chrome.csrf_field())
                        label for="text" { (chrome.t(S::AppealLabel)) }
                        textarea id="text" name="text" required maxlength="5000" {}
                        button id="appeal-submit" type="submit" { (chrome.t(S::SendAppeal)) }
                    }
                }
                None => p id="appeal-status" data-state="closed" { (chrome.t(S::AppealClosed)) },
            }
        },
    )
}

/// One row of the moderation overview.
pub(crate) struct ModerationView<'a> {
    pub(crate) open: Vec<&'a Notice>,
    pub(crate) decided: Vec<&'a Notice>,
    pub(crate) appeals: Vec<(&'a Appeal, Option<&'a Notice>)>,
    pub(crate) blocklist: &'a [BlockEntry],
    pub(crate) suspended: Vec<&'a Account>,
}

fn decision_word(chrome: Chrome<'_>, notice: &Notice) -> &'static str {
    match notice.decision {
        NoticeDecision::Open => "—",
        NoticeDecision::Upheld { .. } => chrome.t(S::Uphold),
        NoticeDecision::Rejected { .. } => chrome.t(S::Reject),
    }
}

/// `GET /moderation`.
pub(crate) fn moderation(chrome: Chrome<'_>, view: &ModerationView<'_>) -> String {
    let locale = chrome.locale;
    let title = chrome.t(S::NavModeration);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            h2 { (chrome.t(S::OpenNotices)) }
            @if view.open.is_empty() { p id="no-open-notices" { (chrome.t(S::NothingOpen)) } }
            ul id="open-notices" {
                @for notice in &view.open {
                    li { a id=(format!("notice-{}", notice.id)) href=(format!("/moderation/signalements/{}", notice.id)) { (notice.id) }
                        " — " (sf(locale, S::ReceivedOn, &[("date", &iso_minute(notice.received_at))])) }
                }
            }
            h2 { (chrome.t(S::OpenAppeals)) }
            @if view.appeals.is_empty() { p { (chrome.t(S::NothingOpen)) } }
            @for (appeal, notice) in &view.appeals {
                div id=(format!("appeal-{}", appeal.id)) {
                    p { strong { (appeal.id) } " — " (appeal.notice_id) " — " (iso_minute(appeal.filed_at)) }
                    @if let Some(NoticeDecision::Upheld { statement, .. }) = notice.map(|n| &n.decision) { p.note { (sf(locale, S::Reasons, &[("statement", statement)])) } }
                    p { (appeal.text) }
                    form method="post" action=(format!("/moderation/reclamations/{}", appeal.id)) {
                        (chrome.csrf_field())
                        label for=(format!("answer-{}", appeal.id)) { (chrome.t(S::AnswerLabel)) }
                        textarea id=(format!("answer-{}", appeal.id)) name="answer" required maxlength="5000" {}
                        button id=(format!("grant-{}", appeal.id)) type="submit" name="grant" value="1" { (chrome.t(S::GrantAppeal)) }
                        button.alt id=(format!("dismiss-{}", appeal.id)) type="submit" name="grant" value="0" { (chrome.t(S::DismissAppeal)) }
                    }
                }
            }
            h2 { (chrome.t(S::ClosedNotices)) }
            ul id="decided-notices" {
                @for notice in &view.decided {
                    li { a href=(format!("/moderation/signalements/{}", notice.id)) { (notice.id) } " — " (decision_word(chrome, notice)) }
                }
            }
            h2 { (chrome.t(S::SuspendedAccounts)) }
            ul id="suspended-accounts" {
                @for account in &view.suspended { li { (account.id) " — " (account.email) " (" (account.upheld_notices) " / " (account.false_declarations) ")" } }
            }
            h2 { (chrome.t(S::Blocklist)) }
            table id="blocklist" {
                @for entry in view.blocklist {
                    tr { td { (match entry.kind { BlockKind::Sha256 => "SHA-256", BlockKind::Isbn => "ISBN" }) } td.code { (entry.value) } td { (entry.notice_id) } td { (iso_date(entry.added_at)) } }
                }
            }
        },
    )
}

/// `GET /moderation/signalements/{id}`.
pub(crate) fn moderation_notice(
    chrome: Chrome<'_>,
    notice: &Notice,
    book: Option<&Book>,
    uploader: Option<&Account>,
    error: Option<&str>,
) -> String {
    let locale = chrome.locale;
    let title = notice.id.clone();
    layout(
        chrome,
        &title,
        html! {
            h1 { (chrome.t(S::NoticeTitle)) " " (notice.id) }
            @if let Some(error) = error { p.err id="error" role="alert" { (error) } }
            p { (sf(locale, S::ReceivedOn, &[("date", &iso_minute(notice.received_at))])) " — "
                (match notice.category { NoticeCategory::Copyright => chrome.t(S::CategoryCopyright), NoticeCategory::OtherIllegal => chrome.t(S::CategoryOther) }) }
            p id="notice-location" { (sf(locale, S::LocationIs, &[("location", &notice.location)])) }
            p id="notice-explanation" { (notice.explanation) }
            p { @if notice.exception { (chrome.t(S::NotifierException)) } @else { (sf(locale, S::NotifierIs, &[("name", &notice.notifier_name), ("email", &notice.notifier_email)])) } }
            @if let Some(book) = book {
                p id="resolved" { (sf(locale, S::ResolvedTo, &[("title", &book.title), ("id", &book.id)])) }
                p.note { (sf(locale, S::BookDeclared, &[("date", &iso_minute(book.declaration.declared_at)), ("basis", &format!("{} {}", basis_text(locale, book.declaration.basis), book.declaration.details))])) }
                @if let Some(account) = uploader {
                    @let identity = account.identity.as_ref();
                    p.note id="uploader" { (sf(locale, S::UploaderIs, &[
                        ("name", identity.map_or("", |i| i.name.as_str())),
                        ("address", identity.map_or("", |i| i.postal_address.as_str())),
                        ("email", &account.email),
                    ])) }
                }
            } @else if matches!(notice.decision, NoticeDecision::Open) {
                p.err id="unresolved" { (chrome.t(S::Unresolved)) }
            }
            @match &notice.decision {
                NoticeDecision::Open => {
                    form id="decision-form" method="post" action=(format!("/moderation/signalements/{}", notice.id)) {
                        (chrome.csrf_field())
                        fieldset {
                            legend { (chrome.t(S::DecisionLegend)) }
                            label.choice { input id="decision-uphold" type="radio" name="decision" value="uphold"; " " (chrome.t(S::Uphold)) }
                            label.choice { input id="decision-reject" type="radio" name="decision" value="reject"; " " (chrome.t(S::Reject)) }
                        }
                        label for="book_id" { (chrome.t(S::BookIdLabel)) }
                        input id="book_id" name="book_id" type="text" maxlength="40" value=(book.map_or("", |b| b.id.as_str()));
                        label for="statement" { (chrome.t(S::StatementLabel)) }
                        textarea id="statement" name="statement" required maxlength="5000" {}
                        label.choice { input id="false_declaration" type="checkbox" name="false_declaration" value="1"; " " (chrome.t(S::FalseDeclLabel)) }
                        button id="decision-submit" type="submit" { (chrome.t(S::Decide)) }
                    }
                }
                NoticeDecision::Upheld { at, statement, .. } => {
                    p id="decision" data-state="upheld" { (chrome.t(S::Uphold)) " — " (iso_minute(*at)) } p { (statement) }
                }
                NoticeDecision::Rejected { at, reason } => {
                    p id="decision" data-state="rejected" { (chrome.t(S::Reject)) " — " (iso_minute(*at)) } p { (reason) }
                }
            }
        },
    )
}

/// `GET /conditions` (DSA art. 14).
pub(crate) fn terms(chrome: Chrome<'_>) -> String {
    let config = chrome.config;
    let locale = chrome.locale;
    let r = &config.retention;
    let title = chrome.t(S::TermsTitle);
    let days = |d: i64| d.to_string();
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p.draft id="draft" { (chrome.t(S::Draft)) }
            h2 { (chrome.t(S::TermsWhatHead)) } p { (chrome.t(S::TermsWhat)) }
            h2 { (chrome.t(S::TermsWhoHead)) } p { (chrome.t(S::TermsWho)) } p.note { (chrome.t(S::NotPurchases)) }
            h2 { (chrome.t(S::TermsChecksHead)) } p { (sf(locale, S::TermsChecks, &[("max", &size_text(locale, config.max_file_bytes))])) }
            h2 { (chrome.t(S::TermsModerationHead)) } p { (chrome.t(S::TermsModeration)) }
            h2 { (chrome.t(S::TermsRepeatHead)) }
            p id="repeat-policy" { (sf(locale, S::TermsRepeat, &[("upheld", &config.suspend_after_upheld.to_string()), ("false", &config.suspend_after_false_declarations.to_string())])) }
            h2 { (chrome.t(S::TermsDataHead)) }
            p id="retention" { (sf(locale, S::TermsData, &[
                ("identity", &days(r.identity_days)), ("connection", &days(r.connection_days)),
                ("declaration", &days(r.declaration_days)), ("notices", &days(r.notice_days)),
                ("security", &days(r.security_log_days)), ("idle", &days(r.idle_file_days)),
            ])) }
            @if let Some(email) = &config.contact_email { p { (sf(locale, S::ContactText, &[("email", email)])) } }
        },
    )
}

/// `GET /mentions-legales` (LCEN art. 6 III).
pub(crate) fn legal(chrome: Chrome<'_>) -> String {
    let legal = &chrome.config.legal;
    let title = chrome.t(S::LegalTitle);
    let missing = chrome.t(S::ToComplete);
    let show = |value: &Option<String>| value.clone().unwrap_or_else(|| missing.to_owned());
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p.draft id="draft" { (chrome.t(S::Draft)) }
            h2 { (chrome.t(S::LegalOperator)) }
            p id="operator" { (show(&legal.operator_name)) br; (show(&legal.operator_address)) br; (show(&legal.operator_registration)) }
            h2 { (chrome.t(S::LegalDirector)) }
            p id="director" { (show(&legal.publication_director)) }
            h2 { (chrome.t(S::NavContact)) }
            p { @match &chrome.config.contact_email { Some(email) => a href=(format!("mailto:{email}")) { (email) }, None => (missing) } }
            h2 { (chrome.t(S::LegalHost)) }
            p id="host" { (show(&legal.host_name)) br; (show(&legal.host_address)) br; (show(&legal.host_phone)) }
        },
    )
}

/// `GET /contact` (DSA arts 11–12).
pub(crate) fn contact(chrome: Chrome<'_>) -> String {
    let title = chrome.t(S::ContactTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            @match &chrome.config.contact_email {
                Some(email) => p id="contact-point" { (sf(chrome.locale, S::ContactText, &[("email", email)])) " " a href=(format!("mailto:{email}")) { (email) } },
                None => p id="contact-point" { (chrome.t(S::ContactMissing)) },
            }
            p { a href="/signaler" { (chrome.t(S::NoticeTitle)) } }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Declaration;

    fn book() -> Book {
        Book {
            id: "b_1".to_owned(),
            account_id: "a_1".to_owned(),
            sha256: "0".repeat(64),
            size_bytes: 1,
            kind: ebc_epub::FileKind::Epub,
            file_name: "x.epub".to_owned(),
            title: "Le Horla & <autres>".to_owned(),
            author: "Guy de Maupassant".to_owned(),
            isbn13: None,
            embedded_isbns: Vec::new(),
            language: None,
            declaration: Declaration {
                basis: RightsBasis::PublicDomain,
                details: "†1893".to_owned(),
                text_version: "v".to_owned(),
                declared_at: 0,
                account_id: "a_1".to_owned(),
                declarant: String::new(),
            },
            created_at: 0,
            removed: None,
            removed_at: None,
        }
    }

    #[test]
    fn the_designer_link_keeps_the_code_in_the_fragment() {
        let link = designer_link("https://ebk.example/", "BCDF-GHJK-LMNP", &book(), Locale::En);
        assert_eq!(
            link,
            "https://ebk.example/app/#code=BCDF%2DGHJK%2DLMNP&title=Le%20Horla%20%26%20%3Cautres%3E&author=Guy%20de%20Maupassant&lang=en"
        );
        assert!(!link.contains('?'), "nothing in the query string, which servers log");
    }

    #[test]
    fn pages_escape_what_users_typed() {
        let config = Config::for_tests();
        let chrome =
            Chrome { locale: Locale::Fr, config: &config, account: None, csrf: None, path: "/" };
        let page =
            super::book(chrome, &book(), &[], false, false, Some("<script>alert(1)</script>"));
        assert!(!page.contains("<script>"));
        assert!(page.contains("Le Horla &amp; &lt;autres&gt;"));
        assert!(!page.to_lowercase().contains("<script"));
    }
}
