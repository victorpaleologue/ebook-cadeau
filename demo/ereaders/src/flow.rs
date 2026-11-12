//! Walks the redemption flow of `docs/api.md` §2 the way a person holding the card would.
//!
//! [`redeem`] drives an [`EinkBrowser`] through the stable element ids of the contract:
//! type the code into `code-input` (home page), or the pair number into `pair-input` (`/e`),
//! then press `download` or `redeem`, or fill `device-email` and send. It stops at the first
//! terminal event — a book in the library, a licence file, a "sent" confirmation, leaving for
//! another site (the reseller's store), an error page — and reports every step with the taps
//! it cost. It backs the `ebc-ereader open` command and gives integration tests a one-call
//! "does this gift reach this device?".

use url::Url;

use crate::browser::{EinkBrowser, LicenceFile, Outcome};
use crate::error::{BrowserError, Error, Result};
use crate::library::LibraryEntry;
use crate::lint::LintConfig;
use crate::net;

/// Most actions attempted before giving up (guards against pages that loop).
const MAX_STEPS: usize = 12;

/// What the person has at hand.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedeemInputs {
    /// The code printed on the card (typed as given).
    pub code: Option<String>,
    /// The pair number shown on the phone.
    pub pair_number: Option<String>,
    /// The device's e-mail address for send-to-device.
    pub device_email: Option<String>,
}

/// One step of the walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowStep {
    /// What was done.
    pub action: String,
    /// Page displayed afterwards.
    pub url: String,
    /// Its HTTP status.
    pub status: u16,
    /// Lint error codes of that page (empty when it is clean or when no new page was shown).
    pub lint_errors: Vec<&'static str>,
    /// Taps spent since the session began.
    pub taps_total: u32,
}

/// How the walk ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowEnd {
    /// The book is in the device's library.
    Downloaded(LibraryEntry),
    /// A licence file was handed to the reading system.
    Licence(LicenceFile),
    /// The send-to-device form was submitted; `message` is the page's `send-result` text.
    Sent {
        /// Text of the `send-result` element.
        message: String,
    },
    /// The flow continued on another site (the reseller's store).
    LeftSite {
        /// Where the browser ended up.
        url: Url,
    },
    /// Nothing more could be done automatically.
    Stopped {
        /// Why.
        reason: String,
    },
}

impl FlowEnd {
    /// `true` when the gift reached the device (or its reading system, or was sent to it).
    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Downloaded(_) | Self::Licence(_) | Self::Sent { .. })
    }
}

/// The whole walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowReport {
    /// Steps, in order.
    pub steps: Vec<FlowStep>,
    /// The end.
    pub end: FlowEnd,
    /// Total taps of the session.
    pub taps: u32,
}

fn step(browser: &EinkBrowser, action: String, new_page: bool) -> FlowStep {
    let (url, status, lint_errors) = match browser.page() {
        Ok(page) => {
            let lint_errors = if new_page {
                let report = page.lint(&LintConfig::for_profile(browser.profile()));
                let mut codes: Vec<&'static str> = report.errors().map(|f| f.code).collect();
                codes.dedup();
                codes
            } else {
                Vec::new()
            };
            (page.url().to_string(), page.status(), lint_errors)
        }
        Err(_) => (String::new(), 0, Vec::new()),
    };
    FlowStep { action, url, status, lint_errors, taps_total: browser.total_taps() }
}

fn excerpt(text: &str) -> String {
    let mut out: String = text.chars().take(160).collect();
    if text.chars().count() > 160 {
        out.push('…');
    }
    out
}

/// Opens `url` on the device and follows the redemption flow as far as it goes.
///
/// Transport errors and broken pages are returned as `Err`; a flow that simply cannot go on
/// (wrong code, device without a usable delivery) ends with [`FlowEnd::Stopped`].
pub async fn redeem(
    browser: &mut EinkBrowser,
    url: &str,
    inputs: &RedeemInputs,
) -> Result<FlowReport> {
    let start = net::parse_url(url)?;
    let mut steps = Vec::new();
    let mut outcome = browser.open(url).await?;
    steps.push(step(browser, format!("open {url}"), outcome.is_page()));

    let (mut code_used, mut pair_used, mut email_used) = (false, false, false);
    let (mut download_refused, mut redeem_refused) = (None::<String>, None::<String>);

    let end = 'walk: {
        for _ in 0..MAX_STEPS {
            match outcome {
                Outcome::Download(entry) => break 'walk FlowEnd::Downloaded(entry),
                Outcome::Licence(file) => break 'walk FlowEnd::Licence(file),
                Outcome::Page => {}
            }
            let page = browser.page()?;
            if !net::same_origin(&start, page.url()) {
                break 'walk FlowEnd::LeftSite { url: page.url().clone() };
            }
            if page.status() >= 400 {
                break 'walk FlowEnd::Stopped {
                    reason: format!("HTTP {}: {}", page.status(), excerpt(page.text())),
                };
            }
            if let Some(message) = page.text_of("send-result") {
                break 'walk FlowEnd::Sent { message: message.to_owned() };
            }
            let has = |id: &str| page.has(id);
            let (action, next): (String, Result<Outcome>) = if let (true, false, Some(code)) =
                (has("code-input"), code_used, inputs.code.as_deref())
            {
                code_used = true;
                let submit = has("code-submit");
                browser.type_into("code-input", code)?;
                let next = if submit {
                    browser.click("code-submit").await
                } else {
                    browser.submit("code-form").await
                };
                (format!("type the code {code} and continue"), next)
            } else if let (true, false, Some(number)) =
                (has("pair-input"), pair_used, inputs.pair_number.as_deref())
            {
                pair_used = true;
                let submit = has("pair-submit");
                browser.type_into("pair-input", number)?;
                let next = if submit {
                    browser.click("pair-submit").await
                } else {
                    browser.submit("pair-form").await
                };
                (format!("type the pair number {number} and continue"), next)
            } else if has("download") && download_refused.is_none() {
                ("press \"download\"".to_owned(), browser.click("download").await)
            } else if has("redeem") && redeem_refused.is_none() {
                ("press \"redeem\"".to_owned(), browser.click("redeem").await)
            } else if let (true, false, Some(address)) =
                (has("device-email"), email_used, inputs.device_email.as_deref())
            {
                email_used = true;
                browser.type_into("device-email", address)?;
                (format!("send to {address}"), browser.submit("send-form").await)
            } else {
                let offers = page.text_of("other-ways").map(excerpt).unwrap_or_default();
                let mut reason =
                    String::from("nothing on the page can be activated with what was given");
                if let Some(refusal) = download_refused.as_ref().or(redeem_refused.as_ref()) {
                    reason = format!("{refusal}; no other delivery could be used");
                }
                if !offers.is_empty() {
                    reason.push_str(&format!(" (other ways offered: {offers})"));
                }
                break 'walk FlowEnd::Stopped { reason };
            };
            match next {
                Ok(next) => {
                    steps.push(step(browser, action, next.is_page()));
                    outcome = next;
                }
                // The device refuses this delivery: note it and look for another way on the page.
                Err(
                    err @ (Error::Browser(BrowserError::DownloadRefused { .. }) | Error::Import(_)),
                ) => {
                    let message = err.to_string();
                    steps.push(step(browser, format!("{action} — refused: {message}"), false));
                    if action.contains("redeem") {
                        redeem_refused = Some(message);
                    } else {
                        download_refused = Some(message);
                    }
                    outcome = Outcome::Page;
                }
                Err(err) => return Err(err),
            }
        }
        FlowEnd::Stopped { reason: format!("gave up after {MAX_STEPS} steps") }
    };
    Ok(FlowReport { steps, end, taps: browser.total_taps() })
}
