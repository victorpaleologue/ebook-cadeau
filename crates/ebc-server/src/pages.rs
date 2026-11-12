//! The HTML pages recipients see (`docs/api.md` §2), rendered with `maud` (every value
//! is escaped by the template engine).
//!
//! Written for decade-old e-reader browsers: no script anywhere, no cookie, one small
//! same-origin style sheet ([`STYLE`], no flexbox, grid, web font or animation), plain
//! links and forms, large controls. Element `id`s are part of the contract.

use ebc_i18n::{Msg, format_unix_date, t, tf};
use ebc_model::{Delivery, GiftCode, GiftStatus, Locale};
use maud::{DOCTYPE, Markup, html};

use crate::device::DeviceClass;
use crate::store::GiftRecord;
use crate::text::{S, s, sf};

/// The style sheet served at `/static/eink.css`.
pub(crate) const STYLE: &str = "\
html{font-family:Georgia,serif;font-size:18px;line-height:1.4;color:#000;background:#fff}\n\
body{margin:0 auto;padding:12px;max-width:34em}\n\
h1{font-size:1.5em;margin:.3em 0}\n\
h2{font-size:1.15em;margin:1.2em 0 .3em}\n\
h3{font-size:1em;margin:1em 0 .2em}\n\
p{margin:.5em 0}\n\
a{color:#000}\n\
label{display:block;margin-top:.6em}\n\
input[type=text],input[type=email]{font-size:1.2em;padding:8px;border:2px solid #000;width:90%;max-width:20em;display:block;margin:6px 0 10px}\n\
button{font-size:1.1em;font-weight:bold;padding:12px 16px;min-height:48px;border:2px solid #000;background:#000;color:#fff;margin:6px 0}\n\
.eink button{min-height:56px}\n\
.alt button{background:#fff;color:#000}\n\
.code{font-family:monospace;font-size:1.5em;font-weight:bold;letter-spacing:.08em}\n\
.note{font-size:.9em}\n\
.err{border:2px solid #000;padding:10px;font-weight:bold}\n\
#gift-message{border-left:4px solid #000;padding-left:10px;font-style:italic}\n\
#gift-message:empty,#gift-author:empty{display:none}\n\
#primary-action{border:2px solid #000;padding:10px;margin:14px 0}\n\
#device-chooser a{display:inline-block;padding:10px 12px;margin:4px 6px 4px 0;border:1px solid #000}\n\
#device-chooser a.on{background:#000;color:#fff}\n\
footer{margin-top:28px;border-top:1px solid #000;padding-top:8px;font-size:.9em}\n\
footer a{margin-right:1.2em;display:inline-block;padding:8px 0}\n";

/// What every page needs to know about the request.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Chrome<'a> {
    /// Language of the page.
    pub(crate) locale: Locale,
    /// Device class the page is laid out for (detected or chosen).
    pub(crate) device: DeviceClass,
    /// The class the visitor chose explicitly (`?device=`), carried in links and forms.
    pub(crate) chosen: Option<DeviceClass>,
    /// The host people type, e.g. `ebk.example`.
    pub(crate) host: &'a str,
}

impl Chrome<'_> {
    /// The `device` value to carry, when one was chosen.
    fn carried(&self) -> Option<&'static str> {
        self.chosen.and_then(DeviceClass::choice)
    }

    /// `path`, with the chosen device as query string.
    fn link(&self, path: &str) -> String {
        match self.carried() {
            Some(choice) => format!("{path}?device={choice}"),
            None => path.to_owned(),
        }
    }

    fn hidden_device(&self) -> Markup {
        html! {
            @if let Some(choice) = self.carried() {
                input type="hidden" name="device" value=(choice);
            }
        }
    }
}

/// The page skeleton.
fn layout(chrome: Chrome<'_>, title: &str, body: Markup) -> String {
    let locale = chrome.locale;
    html! {
        (DOCTYPE)
        html lang=(locale.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="robots" content="noindex, nofollow";
                title { (title) " — " (t(locale, Msg::ProductName)) }
                link rel="stylesheet" href="/static/eink.css";
            }
            body class=[chrome.device.is_eink().then_some("eink")] {
                (body)
                footer {
                    a href=(chrome.link("/")) { (s(locale, S::EnterCode)) }
                    a href=(chrome.link("/aide")) { (s(locale, S::Help)) }
                }
            }
        }
    }
    .into_string()
}

/// The code form shared by the home page and the code error pages.
fn code_form(chrome: Chrome<'_>, prefill: Option<&str>) -> Markup {
    let locale = chrome.locale;
    html! {
        form id="code-form" method="get" action="/k" {
            label for="code-input" { (t(locale, Msg::RedeemIntro)) }
            input id="code-input" name="code" type="text" value=[prefill]
                autocapitalize="characters" autocomplete="off" autocorrect="off" spellcheck="false";
            p.note { (s(locale, S::CodeHint)) }
            (chrome.hidden_device())
            button id="code-submit" type="submit" { (t(locale, Msg::RedeemCta)) }
        }
    }
}

fn error_box(messages: &[String]) -> Markup {
    html! {
        @if !messages.is_empty() {
            div.err id="error" role="alert" {
                @for message in messages { p { (message) } }
            }
        }
    }
}

/// `GET /`, and the answer to a mistyped, unknown or rate-limited code: the code form,
/// with `errors` above it. `prefill` puts the typed text back in the field.
pub(crate) fn home(chrome: Chrome<'_>, errors: &[String], prefill: Option<&str>) -> String {
    let title = t(chrome.locale, Msg::RedeemTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            (error_box(errors))
            (code_form(chrome, prefill))
        },
    )
}

/// A page that only says something, e.g. a cancelled gift or an expired download link.
pub(crate) fn message(
    chrome: Chrome<'_>,
    title: &str,
    messages: &[String],
    back: Option<(&str, &str)>,
) -> String {
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            (error_box(messages))
            @if let Some((href, label)) = back {
                p { a href=(href) { (label) } }
            }
        },
    )
}

/// `GET /aide`: the three ways to get the book.
pub(crate) fn help(chrome: Chrome<'_>) -> String {
    let locale = chrome.locale;
    let title = s(locale, S::HelpTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            div id="help" {
                h2 { (s(locale, S::HelpPhoneHead)) }
                p { (s(locale, S::HelpPhoneText)) }
                h2 { (t(locale, Msg::DeviceEreader)) " (Kobo, PocketBook, Vivlio, tolino)" }
                p { (tf(locale, Msg::EreaderBrowserHint, &[("url", chrome.host)])) }
                p.note { (s(locale, S::CodeHint)) }
                h2 { (s(locale, S::OnKindle)) }
                p { (s(locale, S::KindleExplain)) }
                h2 { (t(locale, Msg::DeviceComputer)) }
                p { (s(locale, S::UsbHint)) }
            }
        },
    )
}

/// Send-to-device settings shown with the e-mail form.
#[derive(Debug, Clone)]
pub(crate) struct SendView<'a> {
    /// The sender address to approve.
    pub(crate) from: &'a str,
    /// The accepted domains, e.g. `@kindle.com, @pbsync.com`.
    pub(crate) domains: String,
}

/// Everything the gift page shows.
pub(crate) struct GiftView<'a> {
    pub(crate) chrome: Chrome<'a>,
    pub(crate) gift: &'a GiftRecord,
    pub(crate) code: &'a GiftCode,
    /// Status at the time of the request.
    pub(crate) status: GiftStatus,
    /// Whether the reveal date has passed.
    pub(crate) revealed: bool,
    /// Trading name of the store, for reseller gifts.
    pub(crate) store_name: &'a str,
    /// Origin of the store (`https://store.example/`), for the link shown once fulfilled.
    pub(crate) store_home: Option<String>,
    /// `Some` when send-to-device is configured.
    pub(crate) send: Option<SendView<'a>>,
    /// Messages to show above the actions (a failed action).
    pub(crate) notices: &'a [String],
}

fn post_form(
    view: &GiftView<'_>,
    id: &str,
    action: &str,
    button_id: &str,
    label: &str,
    extra: Markup,
) -> Markup {
    html! {
        form id=(id) method="post" action=(format!("/k/{}/{action}", view.code.compact())) {
            (view.chrome.hidden_device())
            (extra)
            button id=(button_id) type="submit" { (label) }
        }
    }
}

fn send_form(view: &GiftView<'_>, send: &SendView<'_>) -> Markup {
    let locale = view.chrome.locale;
    html! {
        p.note { (sf(locale, S::ApproveSender, &[("sender", send.from)])) }
        form id="send-form" method="post" action=(format!("/k/{}/send", view.code.compact())) {
            (view.chrome.hidden_device())
            label for="device-email" { (sf(locale, S::DeviceEmailLabel, &[("domains", &send.domains)])) }
            input id="device-email" name="device_email" type="email" autocomplete="off"
                autocapitalize="none" spellcheck="false";
            button id="send" type="submit" { (t(locale, Msg::SendCta)) }
        }
    }
}

fn pair_offer(view: &GiftView<'_>) -> Markup {
    let locale = view.chrome.locale;
    html! {
        p { (s(locale, S::PairIntro)) }
        div.alt { (post_form(view, "pair-start-form", "pair", "pair-start", s(locale, S::PairCta), html! {})) }
    }
}

/// The action block and the alternatives of a hosted file.
fn hosted_actions(view: &GiftView<'_>, media_type: &str) -> (Markup, Markup) {
    let locale = view.chrome.locale;
    let device = view.chrome.device;
    let format = if media_type == "application/pdf" { "PDF" } else { "EPUB" };
    let kindle_route = device == DeviceClass::Kindle && !device.browser_saves(media_type);
    let phone_or_computer = matches!(device, DeviceClass::Mobile | DeviceClass::Desktop);
    let primary = html! {
        @if kindle_route {
            h2 { (s(locale, S::OnKindle)) }
            p { (s(locale, S::KindleExplain)) }
            @if let Some(send) = &view.send { (send_form(view, send)) }
        } @else {
            @let label = if device == DeviceClass::Mobile { s(locale, S::ReadOnPhone) } else { t(locale, Msg::DownloadCta) };
            (post_form(view, "download-form", "download", "download", label, html! {}))
            p.note { (s(locale, S::RetryDownload)) }
        }
        p.note { (sf(locale, S::FormatOpen, &[("format", format)])) }
    };
    let others = html! {
        @if phone_or_computer {
            h3 { (t(locale, Msg::DeviceEreader)) " (Kobo, PocketBook, Vivlio, tolino)" }
            p { (tf(locale, Msg::EreaderBrowserHint, &[("url", view.chrome.host)])) }
            p { (sf(locale, S::YourCode, &[("code", &view.code.grouped())])) }
            (pair_offer(view))
            h3 { (s(locale, S::OnKindle)) }
            p { (s(locale, S::KindleExplain)) }
            @if let Some(send) = &view.send { (send_form(view, send)) }
            @if device == DeviceClass::Desktop {
                h3 { (t(locale, Msg::DeviceComputer)) }
                p { (s(locale, S::UsbHint)) }
            }
            div.alt {
                (post_form(view, "opds-form", "download", "opds", s(locale, S::OpdsCta),
                    html! { input type="hidden" name="opds" value="1"; }))
            }
        } @else if !kindle_route {
            h3 { (t(locale, Msg::DeviceComputer)) }
            p { (s(locale, S::UsbHint)) }
        }
    };
    (primary, others)
}

/// `GET /k/{code}`: the gift page.
pub(crate) fn gift(view: &GiftView<'_>) -> String {
    let chrome = view.chrome;
    let locale = chrome.locale;
    let gift = view.gift;
    let from = match gift.giver_name.as_deref() {
        Some(name) => tf(locale, Msg::RedeemGiftFrom, &[("name", name)]),
        None => t(locale, Msg::RedeemGiftAnonymous).to_owned(),
    };

    if !view.revealed {
        let date = gift.reveal_at.map(|at| format_unix_date(locale, at)).unwrap_or_default();
        let title = s(locale, S::WrappedTitle);
        return layout(
            chrome,
            title,
            html! {
                h1 { (title) }
                p id="gift-from" { (from) }
                p id="reveal-date" { (sf(locale, S::RevealOn, &[("date", &date)])) }
            },
        );
    }

    let store = view.store_name;
    let (primary, others) = match &gift.delivery {
        _ if view.status == GiftStatus::Exhausted => {
            (html! { p { (s(locale, S::Exhausted)) } }, html! {})
        }
        Delivery::HostedFile { media_type, .. } => hosted_actions(view, media_type),
        Delivery::ResellerRedirect { .. } if view.status == GiftStatus::Fulfilled => (
            html! {
                p { (sf(locale, S::Fulfilled, &[("store", store)])) }
                @if let Some(home) = &view.store_home {
                    p { a id="store-link" rel="noreferrer" href=(home) { (tf(locale, Msg::ContinueAtStore, &[("store", store)])) } }
                }
            },
            html! {},
        ),
        Delivery::ResellerRedirect { .. } => (
            html! {
                (post_form(view, "redeem-form", "redeem", "redeem",
                    &sf(locale, S::AddToStoreLibrary, &[("store", store)]), html! {}))
                p.note { (sf(locale, S::FormatStore, &[("store", store)])) }
                @if chrome.device.is_eink() { p { (s(locale, S::ResellerEinkHint)) } }
            },
            html! {},
        ),
        Delivery::ResellerLicence { .. } => (
            html! {
                (post_form(view, "redeem-form", "redeem", "redeem", t(locale, Msg::RedeemCta), html! {}))
                p.note { (sf(locale, S::FormatStore, &[("store", store)])) }
            },
            html! {
                @if !chrome.device.is_eink() {
                    h3 { (t(locale, Msg::DeviceEreader)) }
                    (pair_offer(view))
                }
            },
        ),
    };
    let claims = match (gift.claims, gift.last_claim_at) {
        (0, _) | (_, None) => s(locale, S::NeverClaimed).to_owned(),
        (count, Some(at)) => sf(
            locale,
            S::ClaimedTimes,
            &[("count", &count.to_string()), ("date", &format_unix_date(locale, at))],
        ),
    };
    let choices: [(DeviceClass, &str, &str); 6] = [
        (DeviceClass::Kobo, "device-kobo", "Kobo"),
        (DeviceClass::Kindle, "device-kindle", "Kindle"),
        (DeviceClass::Pocketbook, "device-pocketbook", "PocketBook / Vivlio"),
        (DeviceClass::Tolino, "device-tolino", "tolino"),
        (DeviceClass::Mobile, "device-phone", s(locale, S::Phone)),
        (DeviceClass::Desktop, "device-computer", s(locale, S::Computer)),
    ];

    layout(
        chrome,
        &gift.book.title,
        html! {
            p id="gift-from" { (from) }
            h1 id="gift-title" { (gift.book.title) }
            p id="gift-author" { (gift.book.author_line().unwrap_or_default()) }
            p id="gift-message" {
                @if let Some(message) = &gift.message {
                    @for (index, line) in message.lines().enumerate() {
                        @if index > 0 { br; }
                        (line)
                    }
                }
            }
            (error_box(view.notices))
            div id="primary-action" { (primary) }
            div id="other-ways" {
                h2 { (s(locale, S::OtherWays)) }
                (others)
                p { a href=(chrome.link("/aide")) { (s(locale, S::HelpTitle)) } }
            }
            div id="device-chooser" {
                h2 { (t(locale, Msg::ChooseDevice)) }
                @for (class, id, label) in choices {
                    @let href = format!("/k/{}?device={}", view.code.compact(), class.choice().unwrap_or("computer"));
                    a id=(id) href=(href) class=[(chrome.device == class).then_some("on")] { (label) }
                    " "
                }
            }
            p.note id="claim-count" { (claims) }
        },
    )
}

/// `HH:MM` (UTC) of a Unix time.
fn clock_time(unix: i64) -> String {
    let seconds = unix.rem_euclid(86_400);
    format!("{:02}:{:02}", seconds / 3600, seconds % 3600 / 60)
}

/// The answer to `POST /k/{code}/pair`: the pair code, where to type it, until when.
pub(crate) fn pair_created(
    chrome: Chrome<'_>,
    code: &GiftCode,
    pair: &str,
    expires_at: i64,
    ttl_secs: i64,
) -> String {
    let locale = chrome.locale;
    let title = s(locale, S::PairTitle);
    let url = format!("{}/e", chrome.host);
    let minutes = (ttl_secs / 60).max(1).to_string();
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p.code id="pair-code" { (pair) }
            p { (sf(locale, S::PairSteps, &[("url", &url)])) }
            p.code id="pair-url" { (url) }
            p id="pair-expires" {
                (sf(locale, S::PairExpires, &[("time", &clock_time(expires_at)), ("minutes", &minutes)]))
            }
            div.alt {
                form id="pair-renew-form" method="post" action=(format!("/k/{}/pair", code.compact())) {
                    (chrome.hidden_device())
                    button id="pair-renew" type="submit" { (s(locale, S::PairCta)) }
                }
            }
            p { a href=(chrome.link(&format!("/k/{}", code.compact()))) { (s(locale, S::BackToGift)) } }
        },
    )
}

/// `GET /e` and the error answers of `POST /e`: the pair-code form.
pub(crate) fn pair_form(chrome: Chrome<'_>, errors: &[String]) -> String {
    let locale = chrome.locale;
    let title = s(locale, S::PairFormTitle);
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            (error_box(errors))
            form id="pair-form" method="post" action="/e" {
                label for="pair-input" { (s(locale, S::PairLabel)) }
                input id="pair-input" name="p" type="text" autocapitalize="characters" autocomplete="off"
                    autocorrect="off" spellcheck="false";
                (chrome.hidden_device())
                button id="pair-submit" type="submit" { (t(locale, Msg::DownloadCta)) }
            }
            p { a href=(chrome.link("/")) { (s(locale, S::UseCardCode)) } }
        },
    )
}

/// The answer to `POST /k/{code}/send`.
pub(crate) fn send_result(
    chrome: Chrome<'_>,
    code: &GiftCode,
    title: &str,
    text: &str,
    ok: bool,
) -> String {
    let locale = chrome.locale;
    layout(
        chrome,
        title,
        html! {
            h1 { (title) }
            p id="send-result" class=[(!ok).then_some("err")] { (text) }
            p { a href=(chrome.link(&format!("/k/{}", code.compact()))) { (s(locale, S::BackToGift)) } }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_time_is_utc_hours_and_minutes() {
        assert_eq!(clock_time(0), "00:00");
        assert_eq!(clock_time(1_794_441_600 + 14 * 3600 + 32 * 60 + 59), "14:32");
        assert_eq!(clock_time(-60), "23:59");
    }

    #[test]
    fn the_style_sheet_avoids_what_old_e_reader_browsers_lack() {
        let css = STYLE.to_ascii_lowercase();
        for banned in [
            "flex",
            "grid",
            "@font-face",
            "@import",
            "animation",
            "transition",
            "position:fixed",
            "sticky",
            "url(",
        ] {
            assert!(!css.contains(banned), "{banned}");
        }
        assert!(STYLE.len() < 2_500);
    }
}
