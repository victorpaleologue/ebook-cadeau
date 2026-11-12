//! `Page::lint()`: what breaks (or degrades) a page on an old e-reader browser.
//!
//! The rules encode `docs/api.md` §2 ("HTML must render in decade-old WebKit: no flexbox/grid
//! dependence, no web fonts, page weight < 30 kB", "pages work without JavaScript") and the UX
//! lessons of `docs/research/notes/ereader-ux.devices.md` §7 (static-first pages, no script
//! requirement, no animation). They are heuristics over the markup — nothing is rendered and
//! no style sheet is fetched — so a clean report is necessary, not sufficient.
//!
//! | Code | Severity | Meaning |
//! |---|---|---|
//! | `js-required` | error | an action only works through script (`javascript:` link, `type=button`, `onclick` on a `<div>`, a control outside any form) |
//! | `js-module` | error | `<script type="module">`: unknown to old WebKit, so the script never runs |
//! | `js-only-page` | error | scripts but nothing to read or activate without them, or a "please enable JavaScript" notice |
//! | `js-present` | warning | a script is present (fine as progressive enhancement, never as a requirement) |
//! | `js-inline` | warning | inline script or `on*` handler: blocked by the CSP that api.md §2 mandates |
//! | `form-action` | error | a form whose action is not reachable without script |
//! | `form-unsupported` | error | `multipart/form-data`, `text/plain`, file upload, or a method other than GET/POST |
//! | `form-no-submit` | warning | a form without a submit button (relies on the keyboard's Go key) |
//! | `form-implicit-action` | warning | a form without `action` (submits to the page's own URL) |
//! | `form-attr` | warning | a control attached with the HTML5 `form=` attribute (ignored by old WebKit) |
//! | `external-resource` | error | a sub-resource from another origin (old TLS stacks, captive networks, referrer leaks) |
//! | `page-weight` | error | the HTML document exceeds the weight budget |
//! | `lang-missing` | error | no `<html lang>` |
//! | `charset-missing` | error / warning | no declared charset (error when the page has non-ASCII bytes) |
//! | `label-missing` | error | a visible input without a label |
//! | `contrast` | error / warning | inline-declared text colours below 4.5:1 (error) or 7:1 (warning) |
//! | `css-modern-layout` | warning | flexbox / grid / sticky positioning in inline CSS |
//! | `css-web-font` | warning | `@font-face` or `@import` in inline CSS |
//! | `css-animation` | warning | animation / transition (e-ink ghosting) |
//! | `fixed-width` | warning | an element wider than the device's viewport class |
//! | `viewport-missing` | warning | no `<meta name="viewport">` |
//! | `doctype-missing` | warning | no doctype (quirks mode) |
//! | `input-type` | warning | an input type old browsers render as plain text |
//! | `link-new-window` | warning | `target="_blank"` on a single-window browser |
//! | `sets-cookie` | warning | the response sets a cookie (api.md §2: none are needed) |
//! | `too-many-elements` | warning | the page has more elements than the simulator indexes |

use url::Url;

use crate::net::same_origin;
use crate::page::{FieldKind, FormMethod, Page};
use crate::profile::DeviceProfile;

/// Severity of a lint finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Degrades the experience.
    Warning,
    /// Breaks the page or the flow on at least one class of e-reader.
    Error,
}

/// One thing the lint found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    /// Severity.
    pub severity: Severity,
    /// Stable rule code (see the module table).
    pub code: &'static str,
    /// Human-readable explanation.
    pub message: String,
    /// The element concerned, e.g. `<button id="download">`.
    pub element: Option<String>,
}

/// Lint thresholds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintConfig {
    /// Budget for the HTML document, in bytes. Default 30 000 (`docs/api.md` §2).
    pub weight_budget_bytes: usize,
    /// Narrowest viewport the page must fit, in CSS px.
    pub viewport_css_px: u32,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            weight_budget_bytes: 30_000,
            viewport_css_px: crate::profile::ScreenClass::Small.css_width_px(),
        }
    }
}

impl LintConfig {
    /// The default budget with the viewport of a given device.
    pub fn for_profile(profile: &DeviceProfile) -> Self {
        Self { viewport_css_px: profile.screen.css_width_px(), ..Self::default() }
    }
}

/// The result of linting a page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintReport {
    /// Findings, errors first.
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    /// `true` when there is no error-level finding.
    pub fn passed(&self) -> bool {
        self.errors().next().is_none()
    }

    /// Error-level findings.
    pub fn errors(&self) -> impl Iterator<Item = &LintFinding> {
        self.findings.iter().filter(|f| f.severity == Severity::Error)
    }

    /// Warning-level findings.
    pub fn warnings(&self) -> impl Iterator<Item = &LintFinding> {
        self.findings.iter().filter(|f| f.severity == Severity::Warning)
    }

    /// Whether a rule fired.
    pub fn has(&self, code: &str) -> bool {
        self.findings.iter().any(|f| f.code == code)
    }

    /// The codes that fired, deduplicated, errors first.
    pub fn codes(&self) -> Vec<&'static str> {
        let mut codes: Vec<&'static str> = Vec::new();
        for finding in &self.findings {
            if !codes.contains(&finding.code) {
                codes.push(finding.code);
            }
        }
        codes
    }

    fn push(&mut self, severity: Severity, code: &'static str, message: String, element: Option<&str>) {
        if self.findings.len() < 500 {
            self.findings.push(LintFinding {
                severity,
                code,
                message,
                element: element.map(str::to_owned),
            });
        }
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>, element: Option<&str>) {
        self.push(Severity::Error, code, message.into(), element);
    }

    fn warn(&mut self, code: &'static str, message: impl Into<String>, element: Option<&str>) {
        self.push(Severity::Warning, code, message.into(), element);
    }
}

impl std::fmt::Display for LintReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.findings.is_empty() {
            return f.write_str("no findings");
        }
        for finding in &self.findings {
            let level = match finding.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            write!(f, "{level}[{}]: {}", finding.code, finding.message)?;
            if let Some(element) = &finding.element {
                write!(f, " — {element}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Page {
    /// Checks the page against what old e-reader browsers can handle.
    pub fn lint(&self, config: &LintConfig) -> LintReport {
        let mut report = LintReport::default();
        self.lint_scripts(&mut report);
        self.lint_forms(&mut report);
        self.lint_resources(&mut report);
        self.lint_document(config, &mut report);
        self.lint_style(config, &mut report);
        report.findings.sort_by(|a, b| b.severity.cmp(&a.severity));
        report
    }

    /// Checks the response headers `docs/api.md` §2 requires on every gift page:
    /// `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, `X-Robots-Tag: noindex`,
    /// `X-Content-Type-Options: nosniff` and a `Content-Security-Policy` that does not allow
    /// inline script. All findings are errors with codes `header-*`.
    pub fn check_gift_headers(&self) -> LintReport {
        let mut report = LintReport::default();
        let header = |name: &str| self.header(name).map(str::to_ascii_lowercase).unwrap_or_default();
        if !header("cache-control").contains("no-store") {
            report.error("header-cache-control", "Cache-Control must contain no-store", None);
        }
        if header("referrer-policy").trim() != "no-referrer" {
            report.error("header-referrer-policy", "Referrer-Policy must be no-referrer", None);
        }
        if !header("x-robots-tag").contains("noindex") {
            report.error("header-x-robots-tag", "X-Robots-Tag must contain noindex", None);
        }
        if header("x-content-type-options").trim() != "nosniff" {
            report.error("header-x-content-type-options", "X-Content-Type-Options must be nosniff", None);
        }
        let csp = header("content-security-policy");
        if csp.is_empty() {
            report.error("header-csp", "Content-Security-Policy is missing", None);
        } else if csp_allows_inline_script(&csp) {
            report.error("header-csp", "Content-Security-Policy allows inline script", None);
        }
        report
    }

    fn lint_scripts(&self, report: &mut LintReport) {
        let facts = &self.facts;
        for (element, reason) in &facts.needs_js {
            report.error(
                "js-required",
                format!("this only works with JavaScript: {reason}"),
                Some(element),
            );
        }
        for element in &facts.orphan_controls {
            report.error(
                "js-required",
                "a form control outside any <form> can only be used through script",
                Some(element),
            );
        }
        for script in &facts.scripts {
            let is_data_block = !script.kind.is_empty()
                && !script.kind.contains("javascript")
                && !script.kind.contains("ecmascript")
                && script.kind != "module";
            if is_data_block {
                continue;
            }
            if script.kind == "module" {
                report.error(
                    "js-module",
                    "<script type=\"module\"> is not understood by old WebKit; the script never runs",
                    None,
                );
            } else if script.src.is_some() {
                report.warn(
                    "js-present",
                    "a script is loaded; the page must stay fully usable without it",
                    None,
                );
            }
            if script.src.is_none() && script.inline_bytes > 0 {
                report.warn(
                    "js-inline",
                    "inline script is blocked by the Content-Security-Policy required by docs/api.md §2",
                    None,
                );
            }
        }
        for element in &facts.handlers {
            report.warn(
                "js-inline",
                "inline event handler (on…=) is blocked by the required Content-Security-Policy",
                Some(element),
            );
        }
        let has_script = facts.scripts.iter().any(|s| {
            s.kind.is_empty() || s.kind == "module" || s.kind.contains("javascript")
        });
        if has_script {
            let asks_for_js = facts.noscript_text.iter().any(|t| {
                let lower = t.to_lowercase();
                lower.contains("javascript") && !lower.contains("sans javascript")
                    && !lower.contains("without javascript")
            });
            let usable_forms = self.forms().iter().any(|f| f.default_submit().is_some());
            let usable_links = self.links().any(|a| {
                a.attr("href").is_some_and(|h| {
                    let h = h.trim().to_ascii_lowercase();
                    !h.is_empty() && !h.starts_with('#') && !h.starts_with("javascript:")
                })
            });
            let visible = self.text().chars().count();
            if asks_for_js {
                report.error(
                    "js-only-page",
                    "the page tells visitors without JavaScript to enable it",
                    None,
                );
            } else if !usable_forms && !usable_links && visible < 40 {
                report.error(
                    "js-only-page",
                    "without JavaScript the page shows (almost) nothing and offers nothing to activate",
                    None,
                );
            }
        }
    }

    fn lint_forms(&self, report: &mut LintReport) {
        for form in self.forms() {
            let what = form.describe();
            let action = form.action.as_deref().map(str::trim);
            match action {
                Some(a) if a.to_ascii_lowercase().starts_with("javascript:") => {
                    report.error("form-action", "the form's action is a javascript: URL", Some(&what));
                }
                Some("#") if form.onsubmit => {
                    report.error(
                        "form-action",
                        "the form has a placeholder action and relies on onsubmit",
                        Some(&what),
                    );
                }
                None | Some("") if form.onsubmit => {
                    report.error(
                        "form-action",
                        "the form has no action and relies on onsubmit",
                        Some(&what),
                    );
                }
                None | Some("") => {
                    report.warn(
                        "form-implicit-action",
                        "the form has no action; it submits to the page's own URL",
                        Some(&what),
                    );
                }
                Some(a) => {
                    if let Ok(target) = self.base_url().join(a) {
                        if !matches!(target.scheme(), "http" | "https") {
                            report.error(
                                "form-action",
                                format!("the form's action uses the {}: scheme", target.scheme()),
                                Some(&what),
                            );
                        }
                    }
                }
            }
            if let FormMethod::Other(method) = &form.method {
                report.error(
                    "form-unsupported",
                    format!("method=\"{method}\" is not GET or POST"),
                    Some(&what),
                );
            }
            if form.enctype != "application/x-www-form-urlencoded" {
                report.error(
                    "form-unsupported",
                    format!("enctype {} is not supported by this flow (e-readers cannot upload files)", form.enctype),
                    Some(&what),
                );
            }
            if form.default_submit().is_none() {
                report.warn(
                    "form-no-submit",
                    "the form has no submit button; it can only be sent with the keyboard's Go key",
                    Some(&what),
                );
            }
            for field in &form.fields {
                let element = match (&field.id, &field.name) {
                    (Some(id), _) => format!("<input id=\"{id}\">"),
                    (None, Some(name)) => format!("<input name=\"{name}\">"),
                    (None, None) => "<input>".to_owned(),
                };
                if field.kind == FieldKind::File {
                    report.error("form-unsupported", "file upload is not available on e-readers", Some(&element));
                }
                let needs_label = matches!(
                    field.kind,
                    FieldKind::Text
                        | FieldKind::Password
                        | FieldKind::Checkbox
                        | FieldKind::Radio
                        | FieldKind::Select
                        | FieldKind::Textarea
                        | FieldKind::File
                );
                if needs_label && field.label.as_deref().is_none_or(|l| l.trim().is_empty()) {
                    let hint = if field.placeholder {
                        " (a placeholder is not a label: it vanishes when typing starts)"
                    } else {
                        ""
                    };
                    report.error(
                        "label-missing",
                        format!("the control has no <label>{hint}"),
                        Some(&element),
                    );
                }
                if field.kind.is_submit() && field.text.trim().is_empty() && field.label.is_none() {
                    report.error("label-missing", "the submit button has no text", Some(&element));
                }
            }
        }
        for element in &self.facts.form_attr_controls {
            report.warn(
                "form-attr",
                "the form= attribute is ignored by old WebKit; put the control inside its <form>",
                Some(element),
            );
        }
        for (element, kind) in &self.facts.modern_inputs {
            report.warn(
                "input-type",
                format!("type=\"{kind}\" falls back to a plain text field on old browsers"),
                Some(element),
            );
        }
    }

    fn lint_resources(&self, report: &mut LintReport) {
        for (element, raw) in &self.facts.resources {
            let trimmed = raw.trim();
            if trimmed.to_ascii_lowercase().starts_with("data:") {
                continue;
            }
            let Ok(target) = self.base_url().join(trimmed) else { continue };
            if is_external(self.url(), &target) {
                report.error(
                    "external-resource",
                    format!(
                        "loads {} from another origin ({})",
                        clip(trimmed, 80),
                        target.host_str().unwrap_or("?")
                    ),
                    Some(element),
                );
            }
        }
        for element in &self.facts.blank_targets {
            report.warn(
                "link-new-window",
                "target=\"_blank\" has no meaning on a single-window e-reader browser",
                Some(element),
            );
        }
    }

    fn lint_document(&self, config: &LintConfig, report: &mut LintReport) {
        if self.weight_bytes() > config.weight_budget_bytes {
            report.error(
                "page-weight",
                format!(
                    "the HTML weighs {} bytes; the budget is {} bytes",
                    self.weight_bytes(),
                    config.weight_budget_bytes
                ),
                None,
            );
        }
        if self.lang().is_none() {
            report.error("lang-missing", "<html> has no lang attribute", None);
        }
        if !self.charset_in_header() && !self.facts.meta_charset {
            if self.has_non_ascii() {
                report.error(
                    "charset-missing",
                    "no charset is declared and the page contains non-ASCII bytes: old browsers will show mojibake",
                    None,
                );
            } else {
                report.warn("charset-missing", "no charset is declared (header or <meta charset>)", None);
            }
        }
        if !self.facts.has_viewport {
            report.warn(
                "viewport-missing",
                "no <meta name=\"viewport\">: small screens render the page zoomed out",
                None,
            );
        }
        if !self.facts.has_doctype {
            report.warn("doctype-missing", "no <!DOCTYPE html>: the page renders in quirks mode", None);
        }
        if self.header_values("set-cookie").next().is_some() {
            report.warn(
                "sets-cookie",
                "the response sets a cookie; gift pages must not depend on cookies (docs/api.md §2)",
                None,
            );
        }
        if self.facts.truncated_index {
            report.warn(
                "too-many-elements",
                "the page has more elements than the simulator indexes; later ones are ignored",
                None,
            );
        }
    }

    fn lint_style(&self, config: &LintConfig, report: &mut LintReport) {
        for fact in &self.facts.contrast {
            let message = format!(
                "text #{:02x}{:02x}{:02x} on #{:02x}{:02x}{:02x} has a contrast of {:.1}:1",
                fact.fg.0, fact.fg.1, fact.fg.2, fact.bg.0, fact.bg.1, fact.bg.2, fact.ratio
            );
            if fact.ratio < 4.5 {
                report.error("contrast", format!("{message} (minimum 4.5:1)"), Some(&fact.element));
            } else {
                report.warn(
                    "contrast",
                    format!("{message}; 16-level e-ink screens need about 7:1"),
                    Some(&fact.element),
                );
            }
        }
        let compact: String = self.facts.css.chars().filter(|c| !c.is_whitespace()).collect();
        if ["display:flex", "display:inline-flex", "display:grid", "display:inline-grid", "position:sticky"]
            .iter()
            .any(|needle| compact.contains(needle))
        {
            report.warn(
                "css-modern-layout",
                "flexbox / grid / sticky positioning is not supported by decade-old WebKit",
                None,
            );
        }
        if compact.contains("@font-face") || compact.contains("@import") {
            report.warn("css-web-font", "@font-face / @import: web fonts and extra style sheets are not wanted here", None);
        }
        if compact.contains("animation:") || compact.contains("transition:") || compact.contains("@keyframes") {
            report.warn("css-animation", "animations and transitions ghost on e-ink screens", None);
        }
        for (element, px) in &self.facts.fixed_widths {
            if *px > config.viewport_css_px {
                report.warn(
                    "fixed-width",
                    format!("fixed width of {px}px exceeds the {}px viewport", config.viewport_css_px),
                    Some(element),
                );
            }
        }
    }
}

fn is_external(page: &Url, target: &Url) -> bool {
    matches!(target.scheme(), "http" | "https") && !same_origin(page, target)
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        let mut out: String = text.chars().take(max).collect();
        out.push('…');
        out
    }
}

/// Whether a CSP lets inline script run: `script-src` (or, failing that, `default-src`) is
/// absent or contains `'unsafe-inline'`.
fn csp_allows_inline_script(csp: &str) -> bool {
    let directive = |name: &str| {
        csp.split(';').map(str::trim).find_map(|d| {
            let (key, value) = d.split_once(char::is_whitespace).unwrap_or((d, ""));
            (key == name).then(|| value.to_owned())
        })
    };
    match directive("script-src").or_else(|| directive("default-src")) {
        Some(value) => value.contains("'unsafe-inline'"),
        None => true,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn lint(html: &str) -> LintReport {
        Page::from_html("http://ebk.example/k/ABCDE12345", html).lint(&LintConfig::default())
    }

    const GOOD: &str = r#"<!DOCTYPE html>
<html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>Votre livre</title><link rel="stylesheet" href="/static/eink.css"></head>
<body>
<h1 id="gift-title">Le Horla</h1><p id="gift-author">Guy de Maupassant</p>
<div id="primary-action">
<form id="download-form" method="post" action="/k/ABCDE12345/download">
<button id="download" type="submit">Télécharger le livre</button></form>
</div>
<form id="send-form" method="post" action="/k/ABCDE12345/send">
<label for="device-email">Adresse de votre liseuse</label>
<input id="device-email" name="device_email" type="email">
<input type="submit" value="Envoyer"></form>
<p id="other-ways"><a href="/e">Sur ma liseuse</a></p>
</body></html>"#;

    #[test]
    fn a_plain_form_page_passes_cleanly() {
        let report = lint(GOOD);
        assert!(report.passed(), "{report}");
        assert!(report.findings.is_empty(), "{report}");
        assert_eq!(report.to_string(), "no findings");
    }

    #[test]
    fn pages_that_need_javascript_fail() {
        let spa = lint(r#"<!DOCTYPE html><html lang="fr"><head><meta charset="utf-8"></head>
            <body><div id="app"></div><script src="/bundle.js"></script></body></html>"#);
        assert!(!spa.passed());
        assert!(spa.has("js-only-page"), "{spa}");

        let onclick = lint(r##"<!DOCTYPE html><html lang="fr"><head><meta charset="utf-8"></head><body>
            <p>Votre livre vous attend, appuyez sur le bouton pour le recevoir sur la liseuse.</p>
            <button id="download" onclick="go()">Télécharger</button>
            <a id="alt" href="javascript:send()">Envoyer</a>
            <a id="alt2" href="#" onclick="x()">Autre</a>
            <span id="alt3" onclick="x()">Encore</span>
            <form id="f" onsubmit="return send()"><label>Code <input name="code"></label><input type="submit" value="OK"></form>
            <form id="g" action="/x"><label>Code <input name="code"></label><button type="button">OK</button></form>
            </body></html>"##);
        assert!(!onclick.passed());
        let js: Vec<_> = onclick.errors().filter(|f| f.code == "js-required").collect();
        let elements: Vec<_> = js.iter().filter_map(|f| f.element.as_deref()).collect();
        assert!(elements.contains(&"<button id=\"download\">"), "{elements:?}");
        assert!(elements.contains(&"<a id=\"alt\">"));
        assert!(elements.contains(&"<a id=\"alt2\">"));
        assert!(elements.contains(&"<span id=\"alt3\">"));
        assert!(onclick.errors().any(|f| f.code == "form-action" && f.element.as_deref() == Some("<form id=\"f\">")));
        assert!(onclick.warnings().any(|f| f.code == "form-no-submit" && f.element.as_deref() == Some("<form id=\"g\">")));
        assert!(onclick.has("js-inline"));

        let module = lint(r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"></head><body>
            <p>Some long enough text so the page is not considered empty at all.</p>
            <script type="module" src="/app.mjs"></script></body></html>"#);
        assert!(module.errors().any(|f| f.code == "js-module"), "{module}");

        let notice = lint(r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"></head><body>
            <noscript>Please enable JavaScript to continue.</noscript><a href="/x">Home page link</a>
            <script src="/app.js"></script></body></html>"#);
        assert!(notice.errors().any(|f| f.code == "js-only-page"), "{notice}");
    }

    #[test]
    fn progressive_enhancement_is_only_a_warning() {
        let report = lint(&GOOD.replace("</body>", "<script src=\"/static/enhance.js\"></script><noscript>Tout fonctionne sans JavaScript.</noscript></body>"));
        assert!(report.passed(), "{report}");
        assert_eq!(report.codes(), vec!["js-present"]);
        // JSON-LD and other data blocks are not scripts.
        let report = lint(&GOOD.replace("</body>", "<script type=\"application/ld+json\">{}</script></body>"));
        assert!(report.findings.is_empty(), "{report}");
    }

    #[test]
    fn external_resources_weight_lang_and_labels_are_errors() {
        let report = lint(r#"<!DOCTYPE html><html><head><meta charset="utf-8">
            <link rel="stylesheet" href="https://fonts.example/css?family=X">
            <style>@font-face{font-family:X;src:url(//cdn.example/x.woff)} .a{display:flex; transition: all 1s}</style></head>
            <body><img src="https://tracker.example/pixel.gif" width="900"><img src="/cover.jpg"><img src="data:image/gif;base64,AAAA">
            <form action="/k"><input name="code" placeholder="Code"><input type="file" name="f" aria-label="File">
            <input type="submit" value="OK"></form>
            <a href="https://store.example/" target="_blank">Store (a link is not a resource)</a>
            </body></html>"#);
        let external: Vec<_> = report.errors().filter(|f| f.code == "external-resource").collect();
        assert_eq!(external.len(), 3, "{report}");
        assert!(report.errors().any(|f| f.code == "lang-missing"));
        let label = report.errors().find(|f| f.code == "label-missing").unwrap();
        assert!(label.message.contains("placeholder"));
        assert!(report.errors().any(|f| f.code == "form-unsupported"));
        for code in ["css-modern-layout", "css-web-font", "css-animation", "fixed-width", "link-new-window", "viewport-missing"] {
            assert!(report.warnings().any(|f| f.code == code), "missing {code}: {report}");
        }
        // Errors are listed first.
        let first_warning = report.findings.iter().position(|f| f.severity == Severity::Warning).unwrap();
        assert!(report.findings[first_warning..].iter().all(|f| f.severity == Severity::Warning));

        let heavy = Page::from_html("http://ebk.example/", &format!("{GOOD}<!--{}-->", "x".repeat(40_000)));
        let report = heavy.lint(&LintConfig::default());
        assert!(report.errors().any(|f| f.code == "page-weight"), "{report}");
        let relaxed = LintConfig { weight_budget_bytes: 100_000, ..LintConfig::default() };
        assert!(heavy.lint(&relaxed).passed());
    }

    #[test]
    fn inline_contrast_is_checked_where_the_text_is() {
        let report = lint(&GOOD.replace(
            "<p id=\"other-ways\">",
            "<p id=\"faint\" style=\"color:#ccc\">gris clair</p>\
             <div style=\"background:#000\"><p id=\"ok\" style=\"color:#fff\">blanc sur noir</p></div>\
             <div id=\"bad-bg\" style=\"background-color: navy\">noir sur bleu</div>\
             <p id=\"mid\" style=\"color:#666\">gris moyen</p>\
             <p style=\"color:#eee\"></p><p id=\"other-ways\">",
        ));
        let contrast: Vec<_> = report.findings.iter().filter(|f| f.code == "contrast").collect();
        let of = |id: &str| {
            contrast.iter().find(|f| f.element.as_deref() == Some(&format!("<{id}>") as &str)).map(|f| f.severity)
        };
        assert_eq!(of("p id=\"faint\""), Some(Severity::Error), "{report}");
        assert_eq!(of("div id=\"bad-bg\""), Some(Severity::Error));
        assert_eq!(of("p id=\"mid\""), Some(Severity::Warning));
        assert_eq!(of("p id=\"ok\""), None);
        assert_eq!(contrast.len(), 3, "{report}");
    }

    #[test]
    fn charset_rule_depends_on_the_bytes() {
        let ascii = Page::from_response(
            Url::parse("http://ebk.example/").unwrap(),
            200,
            vec![("content-type".to_owned(), "text/html".to_owned())],
            b"<!DOCTYPE html><html lang=en><head><meta name=viewport content=x></head><body>plain</body></html>",
        );
        let report = ascii.lint(&LintConfig::default());
        assert!(report.passed(), "{report}");
        assert!(report.warnings().any(|f| f.code == "charset-missing"));
        let accented = Page::from_response(
            Url::parse("http://ebk.example/").unwrap(),
            200,
            vec![("content-type".to_owned(), "text/html".to_owned())],
            "<!DOCTYPE html><html lang=fr><head></head><body>déjà</body></html>".as_bytes(),
        );
        assert!(accented.lint(&LintConfig::default()).errors().any(|f| f.code == "charset-missing"));
    }

    #[test]
    fn gift_headers_are_checked_against_the_contract() {
        let headers = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
            pairs.iter().map(|(n, v)| ((*n).to_owned(), (*v).to_owned())).collect()
        };
        let url = Url::parse("http://ebk.example/k/X").unwrap();
        let good = Page::from_response(
            url.clone(),
            200,
            headers(&[
                ("content-type", "text/html; charset=utf-8"),
                ("cache-control", "no-store"),
                ("referrer-policy", "no-referrer"),
                ("x-robots-tag", "noindex, nofollow"),
                ("x-content-type-options", "nosniff"),
                ("content-security-policy", "default-src 'self'; img-src 'self' data:"),
            ]),
            GOOD.as_bytes(),
        );
        assert!(good.check_gift_headers().findings.is_empty());
        let bad = Page::from_response(
            url,
            200,
            headers(&[
                ("content-type", "text/html; charset=utf-8"),
                ("cache-control", "max-age=3600"),
                ("content-security-policy", "default-src 'self'; script-src 'self' 'unsafe-inline'"),
                ("set-cookie", "sid=1"),
            ]),
            GOOD.as_bytes(),
        );
        let report = bad.check_gift_headers();
        assert_eq!(
            report.codes(),
            vec![
                "header-cache-control",
                "header-referrer-policy",
                "header-x-robots-tag",
                "header-x-content-type-options",
                "header-csp"
            ]
        );
        assert!(bad.lint(&LintConfig::default()).warnings().any(|f| f.code == "sets-cookie"));
        assert!(csp_allows_inline_script("img-src 'self'"));
        assert!(!csp_allows_inline_script("default-src 'none'"));
        assert!(!csp_allows_inline_script("default-src 'unsafe-inline'; script-src 'self'"));
    }
}
