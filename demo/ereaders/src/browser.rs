//! `EinkBrowser`: a deliberately limited web client driven like a person would drive it.
//!
//! * It sends the profile's `User-Agent` and **never executes JavaScript**: anything that only
//!   works through script fails with [`BrowserError::NeedsJavaScript`].
//! * It understands links, forms (GET and POST, urlencoded), bounded redirects, cookies (if the
//!   profile has them), `<meta http-equiv="refresh">` (on request) and "back".
//! * A response that is not a page is a download: the browser decides from the headers whether
//!   it would save that kind of file at all (a Kindle refuses EPUB before reading a byte), then
//!   hands the file to the device's [`Library`].
//! * Every interaction is counted in taps ([`TapLog`]), so tests can assert UX budgets.
//!
//! Client-side validation (`required`, `pattern`, input types) is **not** enforced, as on old
//! WebKit (*assumption*: these engines predate reliable constraint validation); `maxlength`
//! is, because every browser enforces it — which is how an 11-character `ABCDE-12345` typed
//! into a `maxlength="10"` field silently loses its last digit.

use std::collections::HashMap;

use reqwest::Method;
use url::Url;

use crate::error::{BrowserError, DownloadRefusal, Error, NetError, Result};
use crate::format::FileFormat;
use crate::library::{ImportSource, IncomingFile, Library, LibraryEntry};
use crate::net::{self, Body, CookieJar, Exchange, Head, Http, NetConfig, Request};
use crate::page::{FieldKind, Form, FormMethod, Page};
use crate::profile::DeviceProfile;
use crate::typing::{InputMode, TapLog, TypingCost, typed_url};

/// Settings of a simulated browser session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserConfig {
    /// Network limits.
    pub net: NetConfig,
    /// Refuse ports other than 80/443 when the profile has the
    /// [`standard_ports_only`](crate::profile::Quirks::standard_ports_only) quirk. Off by
    /// default because tests bind ephemeral ports.
    pub enforce_port_quirk: bool,
    /// `Accept-Language` sent with every request (the device's interface language).
    pub accept_language: String,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            net: NetConfig::default(),
            enforce_port_quirk: false,
            accept_language: "fr-FR,fr;q=0.9,en;q=0.5".to_owned(),
        }
    }
}

/// A licence file (`.lcpl`, `.acsm`) the browser handed to the reading system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenceFile {
    /// `Lcpl` or `Acsm`.
    pub format: FileFormat,
    /// File name.
    pub file_name: String,
    /// URL it was downloaded from.
    pub url: Url,
    /// The document.
    pub bytes: Vec<u8>,
}

/// What an interaction led to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A page is displayed: see [`EinkBrowser::page`].
    Page,
    /// A file was downloaded and is now in the library. The previous page stays displayed.
    Download(LibraryEntry),
    /// A licence file was downloaded; pass it to the reading system
    /// (for LCP: [`crate::lcp::SimulatedLcpReader`]). The previous page stays displayed.
    Licence(LicenceFile),
}

impl Outcome {
    /// The downloaded book, if the outcome is a download.
    pub fn download(&self) -> Option<&LibraryEntry> {
        match self {
            Self::Download(entry) => Some(entry),
            _ => None,
        }
    }

    /// `true` when a page is displayed as a result.
    pub fn is_page(&self) -> bool {
        matches!(self, Self::Page)
    }
}

enum Activation {
    Navigate(Url),
    Submit { form: usize, submitter: Option<usize> },
    Reset { form: usize },
    Stay,
}

/// A simulated e-reader browser session.
#[derive(Debug)]
pub struct EinkBrowser {
    profile: DeviceProfile,
    http: Http,
    config: BrowserConfig,
    jar: CookieJar,
    current: Option<Page>,
    history: Vec<Page>,
    typed: HashMap<(usize, usize), String>,
    checked: HashMap<(usize, usize), bool>,
    selected: HashMap<(usize, usize), usize>,
    library: Library,
    taps: TapLog,
    launched: bool,
    exchanges: Vec<Exchange>,
}

const MAX_HISTORY: usize = 50;

impl EinkBrowser {
    /// A browser for `profile` with default settings.
    pub fn new(profile: DeviceProfile) -> Result<Self> {
        Self::with_config(profile, BrowserConfig::default())
    }

    /// A browser for `profile` with explicit settings.
    pub fn with_config(profile: DeviceProfile, config: BrowserConfig) -> Result<Self> {
        let http = Http::new(profile.user_agent, config.net.clone())?;
        Ok(Self {
            profile,
            http,
            config,
            jar: CookieJar::default(),
            current: None,
            history: Vec::new(),
            typed: HashMap::new(),
            checked: HashMap::new(),
            selected: HashMap::new(),
            library: Library::new(),
            taps: TapLog::default(),
            launched: false,
            exchanges: Vec::new(),
        })
    }

    /// The device profile.
    pub fn profile(&self) -> &DeviceProfile {
        &self.profile
    }

    /// The displayed page.
    pub fn page(&self) -> Result<&Page> {
        self.current.as_ref().ok_or_else(|| BrowserError::NoPage.into())
    }

    /// The device's library.
    pub fn library(&self) -> &Library {
        &self.library
    }

    /// Mutable access to the library (to pre-load books in a test).
    pub fn library_mut(&mut self) -> &mut Library {
        &mut self.library
    }

    /// Every counted interaction so far.
    pub fn taps(&self) -> &TapLog {
        &self.taps
    }

    /// Total taps so far.
    pub fn total_taps(&self) -> u32 {
        self.taps.total()
    }

    /// Every HTTP exchange made so far (redirect hops included), oldest first.
    pub fn exchanges(&self) -> &[Exchange] {
        &self.exchanges
    }

    /// Cookies currently held, as `(name, value)`.
    pub fn cookies(&self) -> Vec<(String, String)> {
        self.jar.pairs()
    }

    /// Exchanges that sent a `Referer` naming a different origin than the one requested — the
    /// way a gift code in a URL leaks to third parties from browsers that ignore
    /// `Referrer-Policy`.
    pub fn cross_origin_referers(&self) -> Vec<&Exchange> {
        self.exchanges
            .iter()
            .filter(|x| {
                x.referer
                    .as_deref()
                    .and_then(|r| Url::parse(r).ok())
                    .is_some_and(|r| !net::same_origin(&r, &x.url))
            })
            .collect()
    }

    /// Types `url` into the address bar and goes there.
    ///
    /// Taps: launching the browser (first call only), one tap on the address bar, the keys
    /// (scheme not typed — see [`TypingCost::of_url`]) and one tap on "Go".
    pub async fn open(&mut self, url: &str) -> Result<Outcome> {
        let parsed = net::parse_url(url)?;
        if !self.launched {
            self.launched = true;
            self.taps.record("open the web browser", self.profile.browser_launch_taps);
        }
        let cost = TypingCost::of_url(url.trim(), &self.profile.keyboard);
        self.taps.record(format!("type the address {}", typed_url(url.trim())), cost.taps + 2);
        self.navigate(Request::get(parsed), false).await
    }

    /// Types `text` into a field found by `id`, then `name`, then label text.
    ///
    /// The value replaces what the field contained and is truncated to `maxlength` like a real
    /// browser would. Taps: one to focus the field, one backspace per character already there,
    /// then the keys.
    pub fn type_into(&mut self, field: &str, text: &str) -> Result<TypingCost> {
        let (form, index) = self.find_field(field)?;
        let (what, kind, disabled, readonly, maxlength, initial) = {
            let control = &self.page()?.forms()[form].fields[index];
            (
                describe_field(control),
                control.kind,
                control.disabled,
                control.readonly,
                control.maxlength,
                control.value.clone(),
            )
        };
        let refuse = |reason: &str| -> Error {
            BrowserError::NotTypeable { field: what.clone(), reason: reason.to_owned() }.into()
        };
        if !kind.is_typeable() {
            return Err(refuse("it is not a text field"));
        }
        if disabled {
            return Err(refuse("it is disabled"));
        }
        if readonly {
            return Err(refuse("it is read-only"));
        }
        let previous = self.typed.get(&(form, index)).cloned().unwrap_or(initial);
        let mut value: String = text.to_owned();
        if let Some(max) = maxlength {
            value = value.chars().take(max).collect();
        }
        let cost = TypingCost::of(text, &self.profile.keyboard, InputMode::Text);
        let backspaces = u32::try_from(previous.chars().count()).unwrap_or(u32::MAX);
        self.taps.record(
            format!("type {text:?} into {what}"),
            cost.taps.saturating_add(backspaces).saturating_add(1),
        );
        self.typed.insert((form, index), value);
        Ok(cost)
    }

    /// The current value of a field (what would be submitted).
    pub fn field_value(&self, field: &str) -> Result<String> {
        let (form, index) = self.find_field(field)?;
        let control = &self.page()?.forms()[form].fields[index];
        Ok(self.typed.get(&(form, index)).cloned().unwrap_or_else(|| control.value.clone()))
    }

    /// Ticks a checkbox or picks a radio button (found by `id` or label text). One tap.
    pub fn check(&mut self, field: &str) -> Result<()> {
        self.set_checked(field, true)
    }

    /// Unticks a checkbox. One tap.
    pub fn uncheck(&mut self, field: &str) -> Result<()> {
        self.set_checked(field, false)
    }

    fn set_checked(&mut self, field: &str, on: bool) -> Result<()> {
        let (form, index) = self.find_field(field)?;
        let (what, kind, siblings) = {
            let fields = &self.page()?.forms()[form].fields;
            let control = &fields[index];
            let siblings: Vec<usize> = fields
                .iter()
                .enumerate()
                .filter(|(_, f)| f.kind == FieldKind::Radio && f.name == control.name)
                .map(|(i, _)| i)
                .collect();
            (describe_field(control), control.kind, siblings)
        };
        match kind {
            FieldKind::Checkbox => {
                self.checked.insert((form, index), on);
            }
            FieldKind::Radio if on => {
                for sibling in siblings {
                    self.checked.insert((form, sibling), sibling == index);
                }
            }
            _ => {
                return Err(BrowserError::NotActionable { element: what }.into());
            }
        }
        self.taps.record(format!("tick {what}"), 1);
        Ok(())
    }

    /// Picks an option of a `<select>` by visible text or value. Two taps (open, pick).
    pub fn select_option(&mut self, field: &str, option: &str) -> Result<()> {
        let (form, index) = self.find_field(field)?;
        let (what, position) = {
            let control = &self.page()?.forms()[form].fields[index];
            let what = describe_field(control);
            if control.kind != FieldKind::Select {
                return Err(BrowserError::NotActionable { element: what }.into());
            }
            let wanted = normalise(option);
            let position = control
                .options
                .iter()
                .position(|o| normalise(&o.text) == wanted || o.value == option)
                .ok_or_else(|| BrowserError::NotFound { query: option.to_owned() })?;
            (what, position)
        };
        self.selected.insert((form, index), position);
        self.taps.record(format!("choose {option:?} in {what}"), 2);
        Ok(())
    }

    /// Submits a form found by `id` or `name` (an empty key means "the only form"), using its
    /// first submit button. One tap.
    pub async fn submit(&mut self, form: &str) -> Result<Outcome> {
        let (index, submitter, what) = {
            let page = self.page()?;
            let index = if form.is_empty() && page.forms().len() == 1 {
                0
            } else {
                page.form_index(form)
                    .ok_or_else(|| BrowserError::NotFound { query: form.to_owned() })?
            };
            (index, page.forms()[index].default_submit(), page.forms()[index].describe())
        };
        self.taps.record(format!("submit {what}"), 1);
        self.submit_form(index, submitter).await
    }

    /// Taps a link or button found by `id`, else by its visible text (case-insensitive). One tap.
    pub async fn click(&mut self, target: &str) -> Result<Outcome> {
        let activation = self.resolve_click(target)?;
        self.taps.record(format!("tap {target:?}"), 1);
        match activation {
            Activation::Navigate(url) => self.navigate(Request::get(url), true).await,
            Activation::Submit { form, submitter } => self.submit_form(form, submitter).await,
            Activation::Reset { form } => {
                self.typed.retain(|(f, _), _| *f != form);
                self.checked.retain(|(f, _), _| *f != form);
                self.selected.retain(|(f, _), _| *f != form);
                Ok(Outcome::Page)
            }
            Activation::Stay => Ok(Outcome::Page),
        }
    }

    /// Follows the page's `<meta http-equiv="refresh">` (no waiting is simulated, no taps).
    pub async fn follow_meta_refresh(&mut self) -> Result<Outcome> {
        let url = self.page()?.meta_refresh().ok_or(BrowserError::NoMetaRefresh)?.url.clone();
        self.navigate(Request::get(url), true).await
    }

    /// Goes back to the previous page as it was displayed (no request is made — a
    /// simplification: real browsers may re-fetch). One tap.
    pub fn back(&mut self) -> Result<&Page> {
        let previous = self.history.pop().ok_or(BrowserError::NoHistory)?;
        self.taps.record("go back", 1);
        self.clear_form_state();
        self.current = Some(previous);
        self.page()
    }

    fn clear_form_state(&mut self) {
        self.typed.clear();
        self.checked.clear();
        self.selected.clear();
    }

    fn find_field(&self, query: &str) -> Result<(usize, usize)> {
        let page = self.page()?;
        let all = || {
            page.forms()
                .iter()
                .enumerate()
                .flat_map(|(fi, form)| form.fields.iter().enumerate().map(move |(i, f)| (fi, i, f)))
        };
        if let Some((form, index, _)) = all().find(|(_, _, f)| f.id.as_deref() == Some(query)) {
            return Ok((form, index));
        }
        let unique = |matches: Vec<(usize, usize)>| -> Result<Option<(usize, usize)>> {
            match matches.len() {
                0 => Ok(None),
                1 => Ok(Some(matches[0])),
                count => Err(BrowserError::Ambiguous { query: query.to_owned(), count }.into()),
            }
        };
        let by_name: Vec<_> = all()
            .filter(|(_, _, f)| f.name.as_deref() == Some(query) && f.kind != FieldKind::Hidden)
            .map(|(form, index, _)| (form, index))
            .collect();
        if let Some(found) = unique(by_name)? {
            return Ok(found);
        }
        let wanted = normalise(query);
        let by_label: Vec<_> = all()
            .filter(|(_, _, f)| f.label.as_deref().is_some_and(|l| normalise(l) == wanted))
            .map(|(form, index, _)| (form, index))
            .collect();
        if let Some(found) = unique(by_label)? {
            return Ok(found);
        }
        Err(BrowserError::NotFound { query: query.to_owned() }.into())
    }

    fn resolve_click(&self, target: &str) -> Result<Activation> {
        let page = self.page()?;
        let by_id = page.nodes.iter().position(|n| n.id.as_deref() == Some(target));
        let node_index = match by_id {
            Some(index) => index,
            None => {
                let wanted = normalise(target);
                let matches: Vec<usize> = page
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| {
                        let label = match n.tag.as_str() {
                            "a" | "button" => page.text_in(&n.text).to_owned(),
                            "input" => {
                                match n.attr("type").unwrap_or("").to_ascii_lowercase().as_str() {
                                    "submit" | "button" | "reset" => {
                                        n.attr("value").unwrap_or("").to_owned()
                                    }
                                    "image" => n.attr("alt").unwrap_or("").to_owned(),
                                    _ => return false,
                                }
                            }
                            _ => return false,
                        };
                        !wanted.is_empty() && normalise(&label) == wanted
                    })
                    .map(|(i, _)| i)
                    .collect();
                match matches.len() {
                    0 => return Err(BrowserError::NotFound { query: target.to_owned() }.into()),
                    1 => matches[0],
                    count => {
                        return Err(
                            BrowserError::Ambiguous { query: target.to_owned(), count }.into()
                        );
                    }
                }
            }
        };
        let node = &page.nodes[node_index];
        let element = node.describe();
        let needs_js = |reason: &str| -> Error {
            BrowserError::NeedsJavaScript { element: element.clone(), reason: reason.to_owned() }
                .into()
        };
        let onclick = node.attr("onclick").is_some();
        match node.tag.as_str() {
            "a" => {
                let Some(href) = node.attr("href").map(str::trim) else {
                    return Err(if onclick {
                        needs_js("the link has onclick but no href")
                    } else {
                        BrowserError::NotActionable { element }.into()
                    });
                };
                let lower = href.to_ascii_lowercase();
                if lower.starts_with("javascript:") {
                    return Err(needs_js("the link's href is a javascript: URL"));
                }
                if href.is_empty() || href.starts_with('#') {
                    return if onclick {
                        Err(needs_js("the link is a placeholder driven by onclick"))
                    } else {
                        Ok(Activation::Stay)
                    };
                }
                let url = page.base_url().join(href).map_err(|e| NetError::InvalidUrl {
                    url: href.to_owned(),
                    reason: e.to_string(),
                })?;
                if !matches!(url.scheme(), "http" | "https") {
                    return Err(BrowserError::UnsupportedLinkScheme {
                        scheme: url.scheme().to_owned(),
                        href: href.to_owned(),
                    }
                    .into());
                }
                Ok(Activation::Navigate(url))
            }
            "button" | "input" => {
                let Some((form, index)) = node.field else {
                    let is_input_text = node.tag == "input"
                        && !matches!(
                            node.attr("type").unwrap_or("").to_ascii_lowercase().as_str(),
                            "submit" | "button" | "image" | "reset"
                        );
                    if is_input_text {
                        return Err(BrowserError::NotActionable { element }.into());
                    }
                    return Err(if node.attr("form").is_some() {
                        BrowserError::FormNotSubmittable {
                            form: element,
                            reason: "the button sits outside its <form>; the form= attribute is ignored by old WebKit".to_owned(),
                        }
                        .into()
                    } else {
                        needs_js("a button outside any <form> does nothing without script")
                    });
                };
                let control = &page.forms()[form].fields[index];
                if control.disabled {
                    return Err(BrowserError::NotActionable { element }.into());
                }
                match control.kind {
                    FieldKind::Submit | FieldKind::Image => {
                        Ok(Activation::Submit { form, submitter: Some(index) })
                    }
                    FieldKind::Reset => Ok(Activation::Reset { form }),
                    FieldKind::Button => Err(needs_js("type=button does nothing without script")),
                    _ => Err(BrowserError::NotActionable { element }.into()),
                }
            }
            _ if onclick => Err(needs_js("the element is only clickable through onclick")),
            _ => Err(BrowserError::NotActionable { element }.into()),
        }
    }

    fn form_data(
        &self,
        form_index: usize,
        form: &Form,
        submitter: Option<usize>,
    ) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        for (index, field) in form.fields.iter().enumerate() {
            let Some(name) = field.name.as_deref() else { continue };
            if field.disabled {
                continue;
            }
            let key = (form_index, index);
            match field.kind {
                FieldKind::Text | FieldKind::Password | FieldKind::Hidden | FieldKind::Textarea => {
                    let value =
                        self.typed.get(&key).cloned().unwrap_or_else(|| field.value.clone());
                    pairs.push((name.to_owned(), value));
                }
                FieldKind::Checkbox | FieldKind::Radio => {
                    if self.checked.get(&key).copied().unwrap_or(field.checked) {
                        let value =
                            if field.value.is_empty() { "on" } else { field.value.as_str() };
                        pairs.push((name.to_owned(), value.to_owned()));
                    }
                }
                FieldKind::Submit | FieldKind::Image => {
                    if submitter == Some(index) {
                        pairs.push((name.to_owned(), field.value.clone()));
                    }
                }
                FieldKind::Select => {
                    let chosen = self
                        .selected
                        .get(&key)
                        .copied()
                        .or_else(|| field.options.iter().position(|o| o.selected))
                        .unwrap_or(0);
                    if let Some(option) = field.options.get(chosen) {
                        pairs.push((name.to_owned(), option.value.clone()));
                    }
                }
                FieldKind::Reset | FieldKind::Button | FieldKind::File => {}
            }
        }
        pairs
    }

    async fn submit_form(
        &mut self,
        form_index: usize,
        submitter: Option<usize>,
    ) -> Result<Outcome> {
        let page = self.page()?;
        let form = &page.forms()[form_index];
        let what = form.describe();
        let refuse = |reason: String| -> Error {
            BrowserError::FormNotSubmittable { form: what.clone(), reason }.into()
        };
        let action = form.action.as_deref().map(str::trim).unwrap_or("");
        if action.to_ascii_lowercase().starts_with("javascript:") {
            return Err(BrowserError::NeedsJavaScript {
                element: what,
                reason: "the form's action is a javascript: URL".to_owned(),
            }
            .into());
        }
        let mut url = if action.is_empty() {
            page.url().clone()
        } else {
            page.base_url().join(action).map_err(|e| refuse(format!("bad action URL: {e}")))?
        };
        if !matches!(url.scheme(), "http" | "https") {
            return Err(refuse(format!("the action uses the {}: scheme", url.scheme())));
        }
        let pairs = self.form_data(form_index, form, submitter);
        let encoded =
            url::form_urlencoded::Serializer::new(String::new()).extend_pairs(&pairs).finish();
        let request = match form.method {
            FormMethod::Post => {
                if form.enctype != "application/x-www-form-urlencoded" {
                    return Err(refuse(format!(
                        "enctype {} is not supported (only application/x-www-form-urlencoded)",
                        form.enctype
                    )));
                }
                url.set_fragment(None);
                Request {
                    method: Method::POST,
                    url,
                    headers: Vec::new(),
                    body: Some(Body {
                        content_type: "application/x-www-form-urlencoded".to_owned(),
                        bytes: encoded.into_bytes(),
                    }),
                }
            }
            // GET, and unknown methods, which browsers treat as GET.
            FormMethod::Get | FormMethod::Other(_) => {
                url.set_query(Some(&encoded));
                url.set_fragment(None);
                Request::get(url)
            }
        };
        self.navigate(request, true).await
    }

    fn referer_for(&self, target: &Url) -> Option<String> {
        let page = self.current.as_ref()?;
        let mut full = page.url().clone();
        full.set_fragment(None);
        let origin = || format!("{}/", full.origin().ascii_serialization());
        if self.profile.quirks.ignores_referrer_policy {
            return Some(full.to_string());
        }
        let policy = page
            .header("referrer-policy")
            .and_then(|p| p.split(',').next_back())
            .map(|p| p.trim().to_ascii_lowercase())
            .unwrap_or_default();
        let same = net::same_origin(&full, target);
        match policy.as_str() {
            "no-referrer" => None,
            "same-origin" => same.then(|| full.to_string()),
            "origin" | "strict-origin" => Some(origin()),
            "unsafe-url" | "no-referrer-when-downgrade" => Some(full.to_string()),
            _ => Some(if same { full.to_string() } else { origin() }),
        }
    }

    async fn navigate(&mut self, mut request: Request, from_page: bool) -> Result<Outcome> {
        if self.config.enforce_port_quirk && self.profile.quirks.standard_ports_only {
            let port = request.url.port_or_known_default().unwrap_or(0);
            if !matches!(port, 80 | 443) {
                return Err(BrowserError::NonStandardPort { device: self.profile.id, port }.into());
            }
        }
        if from_page {
            if let Some(referer) = self.referer_for(&request.url) {
                request.headers.push(("referer", referer));
            }
        }
        request.headers.push(("accept", "text/html,application/xhtml+xml,*/*;q=0.8".to_owned()));
        if !self.config.accept_language.is_empty() {
            request.headers.push(("accept-language", self.config.accept_language.clone()));
        }
        let jar = if self.profile.cookies { Some(&mut self.jar) } else { None };
        let (head, response) = self.http.fetch(request, jar, &mut self.exchanges).await?;
        self.handle_response(head, response).await
    }

    async fn handle_response(
        &mut self,
        head: Head,
        response: reqwest::Response,
    ) -> Result<Outcome> {
        let content_type = head.header("content-type").unwrap_or("").to_owned();
        let essence = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        let attachment = head
            .header("content-disposition")
            .is_some_and(|d| d.trim_start().to_ascii_lowercase().starts_with("attachment"));
        let is_html = matches!(essence.as_str(), "text/html" | "application/xhtml+xml" | "");
        let is_inline_text = essence == "text/plain";

        if (is_html || is_inline_text) && !attachment {
            let limit = self.profile.max_page_bytes;
            let body = match self.http.read(&head, response, limit as u64).await {
                Err(NetError::BodyTooLarge { url, .. }) => {
                    return Err(BrowserError::PageTooHeavy { url, limit }.into());
                }
                other => other?,
            };
            let page = if is_inline_text {
                let escaped = String::from_utf8_lossy(&body)
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                let mut page = Page::from_html(head.url.as_str(), &format!("<pre>{escaped}</pre>"));
                page.set_response(head.status, head.headers);
                page
            } else {
                Page::from_response(head.url, head.status, head.headers, &body)
            };
            self.show(page);
            return Ok(Outcome::Page);
        }

        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }

        let file_name = net::response_file_name(&head);
        let declared = FileFormat::from_media_type(&content_type)
            .or_else(|| FileFormat::from_file_name(&file_name));
        let refuse = |reason: DownloadRefusal| -> Error {
            BrowserError::DownloadRefused {
                device: self.profile.id,
                file_name: file_name.clone(),
                format: declared,
                reason,
            }
            .into()
        };
        let Some(format) = declared else {
            return Err(refuse(DownloadRefusal::UnknownType));
        };
        if !self.profile.browser_downloads(format) {
            return Err(refuse(DownloadRefusal::UnsupportedType));
        }
        if self.profile.quirks.ascii_download_names && !is_plain_file_name(&file_name) {
            return Err(refuse(DownloadRefusal::FileName));
        }
        let bytes = self.http.read(&head, response, self.profile.max_download_bytes).await?;
        self.taps.record(
            format!("confirm the download of {file_name}"),
            self.profile.download_confirm_taps,
        );
        if format.is_licence() {
            return Ok(Outcome::Licence(LicenceFile { format, file_name, url: head.url, bytes }));
        }
        let entry = self.library.import(
            &self.profile,
            IncomingFile { bytes: &bytes, file_name: &file_name, declared: Some(format) },
            ImportSource::BrowserDownload { url: head.url.to_string() },
        )?;
        Ok(Outcome::Download(entry))
    }

    fn show(&mut self, page: Page) {
        if let Some(previous) = self.current.take() {
            if self.history.len() >= MAX_HISTORY {
                self.history.remove(0);
            }
            self.history.push(previous);
        }
        self.clear_form_state();
        self.current = Some(page);
    }
}

/// File names an old Kindle-class browser can save: letters, digits and `. _ - " ' ( )`
/// (research: `ereader-ux.devices.md` §3.1 \[V1\], after send2ereader's Kindle workaround).
fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '"' | '\'' | '(' | ')')
        })
}

fn describe_field(field: &crate::page::Field) -> String {
    match (&field.id, &field.name, &field.label) {
        (Some(id), _, _) => format!("the field #{id}"),
        (None, Some(name), _) => format!("the field named {name:?}"),
        (None, None, Some(label)) => format!("the field labelled {label:?}"),
        (None, None, None) => "an unnamed field".to_owned(),
    }
}

fn normalise(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_file_names() {
        assert!(is_plain_file_name("le-horla_(1887).azw3"));
        assert!(!is_plain_file_name("le horla.azw3"));
        assert!(!is_plain_file_name("déjà.azw3"));
        assert!(!is_plain_file_name(""));
        assert!(!is_plain_file_name("a/b.azw3"));
    }

    #[test]
    fn text_matching_ignores_case_and_spacing() {
        assert_eq!(normalise("  Télécharger\n le   LIVRE "), "télécharger le livre");
    }
}
