//! A robots.txt parser and matcher following RFC 9309 (Robots Exclusion Protocol).
//!
//! What is implemented:
//!
//! * **groups**: one or more `User-agent` lines followed by rules; a `User-agent` line
//!   after a rule starts a new group; rules before any `User-agent` line are ignored;
//! * **group selection** by product token, case-insensitively; all groups naming the
//!   token are combined; when none does, the `*` groups apply; when there is none
//!   either, everything is allowed;
//! * **longest match**: the matching rule with the most octets wins; on a tie `Allow`
//!   wins; no matching rule means allowed; `/robots.txt` itself is always allowed;
//! * **special characters**: `*` (any sequence) and a final `$` (end of the path);
//! * **percent-encoding normalisation** before comparison: non-ASCII octets are
//!   percent-encoded, percent-encoded *unreserved* characters are decoded, hex digits are
//!   upper-cased, other escapes are kept; a literal `*` or `$` in a URL matches `%2A` /
//!   `%24` in a rule;
//! * the first 500 KiB are parsed, the rest is ignored (the minimum the RFC asks for);
//! * `Crawl-delay`, which is **not** part of the RFC, as a polite extra.
//!
//! Where a choice exists this module takes the side that fetches less: a `Disallow`
//! value without a leading `/` is treated as if it had one (an `Allow` is not);
//! over-long `Allow` rules are dropped while over-long `Disallow` rules are kept as
//! broader prefixes; a file with an absurd number of rules disallows everything.
//!
//! [`Robots::from_response`] encodes how the fetch outcome is read (the conservative
//! reading chosen by `docs/research/02-cover-sources.md`, implication 22; the 429 case
//! is Google's practice, not RFC text): 2xx → parse; 4xx other than 429 → no
//! restriction; 429, 5xx, anything else or an unreachable host → disallow everything.
//!
//! robots.txt is not a licence: "allowed" here only means the site did not object to
//! the fetch. It grants no right in what is fetched.
//!
//! ```
//! use ebc_covers::robots::Robots;
//!
//! let robots = Robots::parse("User-agent: *\nDisallow: /private/\nAllow: /private/cover.jpg\n");
//! assert!(robots.allowed("eBookCadeauBot/0.1 (+https://example.org/bot)", "/livre/1"));
//! assert!(!robots.allowed("eBookCadeauBot", "/private/x"));
//! assert!(robots.allowed("eBookCadeauBot", "/private/cover.jpg"));
//! ```

/// Only this many leading bytes of a robots.txt are parsed (500 kibibytes).
pub const MAX_ROBOTS_BYTES: usize = 500 * 1024;
/// A file with more rules than this is treated as "disallow everything".
pub const MAX_RULES: usize = 10_000;
/// Longest rule pattern kept, in bytes after normalisation.
const MAX_PATTERN_BYTES: usize = 2048;
/// Longest path matched, in bytes after normalisation; longer paths are cut.
const MAX_PATH_BYTES: usize = 8192;

/// One `Allow` or `Disallow` line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    allow: bool,
    /// Normalised pattern; `*` is the wildcard; a final `$` has been moved to `anchored`.
    pattern: String,
    anchored: bool,
}

impl Rule {
    /// The specificity used for longest-match: octets of the rule path.
    fn specificity(&self) -> usize {
        self.pattern.len() + usize::from(self.anchored)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Group {
    /// Lower-cased product tokens; `"*"` for the wildcard.
    agents: Vec<String>,
    rules: Vec<Rule>,
}

/// A parsed robots.txt, or one of the two blanket verdicts.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Robots {
    groups: Vec<Group>,
    /// `Crawl-delay` values with the agents they were declared for.
    delays: Vec<(Vec<String>, f32)>,
    /// `Some(true)`: allow everything; `Some(false)`: disallow everything.
    blanket: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Directive {
    UserAgent,
    Allow,
    Disallow,
    CrawlDelay,
    Other,
}

fn directive(key: &str) -> Directive {
    let key = key.trim().to_ascii_lowercase();
    match key.as_str() {
        "user-agent" | "useragent" | "user agent" => Directive::UserAgent,
        "allow" => Directive::Allow,
        // Frequent misspellings are read as `Disallow`: that only ever restricts us.
        "disallow" | "dissallow" | "dissalow" | "disalow" | "diasllow" | "disallaw" => {
            Directive::Disallow
        }
        "crawl-delay" | "crawldelay" | "crawl delay" => Directive::CrawlDelay,
        _ => Directive::Other,
    }
}

/// The product token of a `User-agent` line value or of a full user-agent string:
/// `"*"`, or the leading run of letters, `_` and `-`, lower-cased.
fn product_token(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('*') {
        return "*".to_owned();
    }
    value
        .chars()
        .take_while(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

const fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

const fn is_reserved(b: u8) -> bool {
    matches!(
        b,
        b':' | b'/' | b'?' | b'#' | b'[' | b']' | b'@' | b'!' | b'$' | b'&' | b'\'' | b'(' | b')'
            | b'*' | b'+' | b',' | b';' | b'='
    )
}

const fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn push_escaped(out: &mut String, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(char::from(HEX[usize::from(b >> 4)]));
    out.push(char::from(HEX[usize::from(b & 0x0F)]));
}

/// Brings a rule path or a URL path to the comparison form described in the module
/// docs. The result is pure ASCII and at most `limit` bytes long (cut on a boundary
/// that never splits an escape). With `pattern` set, `*` stays a wildcard; otherwise a
/// literal `*` or `$` is escaped.
fn normalise(input: &str, pattern: bool, limit: usize) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len().min(limit));
    let mut i = 0;
    while i < bytes.len() && out.len() + 3 <= limit {
        let b = bytes[i];
        if b == b'%' {
            let decoded = match (bytes.get(i + 1), bytes.get(i + 2)) {
                (Some(&h), Some(&l)) => hex_value(h).zip(hex_value(l)).map(|(h, l)| h << 4 | l),
                _ => None,
            };
            match decoded {
                Some(v) if is_unreserved(v) => out.push(char::from(v)),
                Some(v) => push_escaped(&mut out, v),
                None => {
                    out.push('%');
                    i += 1;
                    continue;
                }
            }
            i += 3;
            continue;
        }
        if is_unreserved(b) || (b == b'*' && pattern) {
            out.push(char::from(b));
        } else if b == b'*' || b == b'$' {
            push_escaped(&mut out, b);
        } else if is_reserved(b) {
            out.push(char::from(b));
        } else {
            // Non-ASCII octets, spaces, controls and the other excluded characters.
            push_escaped(&mut out, b);
        }
        i += 1;
    }
    out
}

/// `true` when `pattern` (normalised, `*` wildcards) matches `path` (normalised) from
/// its start; with `anchored`, the match must also consume the whole path.
pub(crate) fn pattern_matches(pattern: &str, anchored: bool, path: &str) -> bool {
    let mut segments = pattern.split('*');
    let first = segments.next().unwrap_or_default();
    if !path.starts_with(first) {
        return false;
    }
    let mut rest = &path[first.len()..];
    let mut segments = segments.peekable();
    if segments.peek().is_none() {
        // No wildcard at all.
        return !anchored || rest.is_empty();
    }
    while let Some(segment) = segments.next() {
        if segments.peek().is_none() && anchored {
            return rest.ends_with(segment);
        }
        match rest.find(segment) {
            Some(at) => rest = &rest[at + segment.len()..],
            None => return false,
        }
    }
    true
}

/// Splits a raw rule path into its normalised pattern and the "anchored by a final `$`"
/// flag.
pub(crate) fn compile_pattern(raw: &str) -> (String, bool) {
    let (raw, anchored) = match raw.strip_suffix('$') {
        Some(stripped) => (stripped, true),
        None => (raw, false),
    };
    (normalise(raw, true, usize::MAX), anchored)
}

/// Normalises a URL path for matching: cuts the fragment, makes sure it starts with `/`.
pub(crate) fn normalise_path(path: &str) -> String {
    let path = path.trim();
    // A full URL is accepted: keep what follows the authority.
    let path = match path.find("://") {
        Some(scheme_end)
            if scheme_end > 0
                && path[..scheme_end]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.')) =>
        {
            let after = &path[scheme_end + 3..];
            after.find(['/', '?']).map_or("/", |at| &after[at..])
        }
        _ => path,
    };
    let path = path.split('#').next().unwrap_or_default();
    let normalised = normalise(path, false, MAX_PATH_BYTES);
    if normalised.starts_with('/') { normalised } else { format!("/{normalised}") }
}

impl Robots {
    /// Parses the text of a robots.txt. Never fails: unknown or malformed lines are
    /// skipped. Only the first [`MAX_ROBOTS_BYTES`] are read.
    #[must_use]
    pub fn parse(txt: &str) -> Self {
        let mut end = txt.len().min(MAX_ROBOTS_BYTES);
        while !txt.is_char_boundary(end) {
            end -= 1;
        }
        let txt = txt[..end].trim_start_matches('\u{FEFF}');

        let mut out = Self::default();
        let mut collecting_agents = false;
        let mut delay_agents: Vec<String> = Vec::new();
        let mut collecting_delay_agents = false;
        let mut rule_count = 0usize;

        for line in txt.split(['\n', '\r']) {
            let line = line.split('#').next().unwrap_or_default();
            let Some((key, value)) = line.split_once(':') else { continue };
            let value = value.trim();
            match directive(key) {
                Directive::UserAgent => {
                    let token = product_token(value);
                    if !collecting_agents {
                        out.groups.push(Group::default());
                        collecting_agents = true;
                    }
                    if let Some(group) = out.groups.last_mut() {
                        group.agents.push(token.clone());
                    }
                    if !collecting_delay_agents {
                        delay_agents.clear();
                        collecting_delay_agents = true;
                    }
                    delay_agents.push(token);
                }
                kind @ (Directive::Allow | Directive::Disallow) => {
                    collecting_agents = false;
                    collecting_delay_agents = false;
                    let allow = kind == Directive::Allow;
                    let Some(group) = out.groups.last_mut() else { continue };
                    if value.is_empty() {
                        continue; // "Disallow:" restricts nothing
                    }
                    let (mut pattern, anchored) = compile_pattern(value);
                    if !pattern.starts_with('/') && !pattern.starts_with('*') {
                        if allow {
                            continue; // can never match a path
                        }
                        pattern.insert(0, '/');
                    }
                    let mut anchored = anchored;
                    if pattern.len() > MAX_PATTERN_BYTES {
                        if allow {
                            continue;
                        }
                        // A shorter prefix disallows more, never less. The cut may fall
                        // inside an escape; drop the dangling part.
                        pattern.truncate(MAX_PATTERN_BYTES);
                        if let Some(at) = pattern.rfind('%').filter(|at| at + 3 > pattern.len()) {
                            pattern.truncate(at);
                        }
                        anchored = false;
                    }
                    rule_count += 1;
                    if rule_count > MAX_RULES {
                        return Self::disallow_all();
                    }
                    group.rules.push(Rule { allow, pattern, anchored });
                }
                Directive::CrawlDelay => {
                    collecting_delay_agents = false;
                    if delay_agents.is_empty() {
                        continue;
                    }
                    if let Some(seconds) = value.parse::<f32>().ok().filter(|s| s.is_finite() && *s >= 0.0)
                    {
                        out.delays.push((delay_agents.clone(), seconds));
                    }
                }
                Directive::Other => {}
            }
        }
        out
    }

    /// A verdict that allows everything (no robots.txt, or a 4xx other than 429).
    #[must_use]
    pub fn allow_all() -> Self {
        Self { blanket: Some(true), ..Self::default() }
    }

    /// A verdict that disallows everything (robots.txt unreachable, 5xx or 429).
    #[must_use]
    pub fn disallow_all() -> Self {
        Self { blanket: Some(false), ..Self::default() }
    }

    /// Reads the outcome of fetching `/robots.txt` (after redirects): see the module
    /// docs. `body` is decoded as UTF-8 with replacement.
    #[must_use]
    pub fn from_response(status: u16, body: &[u8]) -> Self {
        match status {
            200..=299 => {
                let end = body.len().min(MAX_ROBOTS_BYTES);
                Self::parse(&String::from_utf8_lossy(&body[..end]))
            }
            429 => Self::disallow_all(),
            400..=499 => Self::allow_all(),
            _ => Self::disallow_all(),
        }
    }

    /// The verdict for a robots.txt that could not be fetched at all (timeout, DNS or
    /// TLS failure): disallow everything.
    #[must_use]
    pub fn unreachable() -> Self {
        Self::disallow_all()
    }

    /// `true` when this is the blanket "disallow everything" verdict.
    #[must_use]
    pub fn is_disallow_all(&self) -> bool {
        self.blanket == Some(false)
    }

    /// The groups that apply to `user_agent`: those naming its product token, else the
    /// `*` groups.
    fn groups_for<'a>(&'a self, token: &str) -> impl Iterator<Item = &'a Group> + use<'a> {
        let specific =
            !token.is_empty() && self.groups.iter().any(|g| g.agents.iter().any(|a| a == token));
        let wanted = if specific { token.to_owned() } else { "*".to_owned() };
        self.groups.iter().filter(move |g| g.agents.iter().any(|a| *a == wanted))
    }

    /// Whether `user_agent` may fetch `path`.
    ///
    /// `user_agent` is a product token (`"eBookCadeauBot"`) or a full user-agent string
    /// starting with one (`"eBookCadeauBot/1.0 (+https://…)"`). `path` is the path and
    /// query of the URL (`"/livre/1?x=2"`); a full URL is accepted too.
    #[must_use]
    pub fn allowed(&self, user_agent: &str, path: &str) -> bool {
        if let Some(verdict) = self.blanket {
            return verdict;
        }
        let path = normalise_path(path);
        if path == "/robots.txt" {
            return true;
        }
        let token = product_token(user_agent);
        let mut best_allow: Option<usize> = None;
        let mut best_disallow: Option<usize> = None;
        for rule in self.groups_for(&token).flat_map(|g| g.rules.iter()) {
            let best = if rule.allow { &mut best_allow } else { &mut best_disallow };
            let specificity = rule.specificity();
            if best.is_some_and(|b| b >= specificity) {
                continue;
            }
            if pattern_matches(&rule.pattern, rule.anchored, &path) {
                *best = Some(specificity);
            }
        }
        match (best_allow, best_disallow) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(allow), Some(disallow)) => allow >= disallow,
        }
    }

    /// The `Crawl-delay` (seconds) declared for `user_agent`, from the groups that apply
    /// to it; the longest when several are declared. Not part of RFC 9309.
    #[must_use]
    pub fn crawl_delay(&self, user_agent: &str) -> Option<f32> {
        let token = product_token(user_agent);
        let specific =
            !token.is_empty() && self.groups.iter().any(|g| g.agents.iter().any(|a| *a == token));
        let wanted = if specific { token.as_str() } else { "*" };
        self.delays
            .iter()
            .filter(|(agents, _)| agents.iter().any(|a| a == wanted))
            .map(|(_, seconds)| *seconds)
            .fold(None, |best: Option<f32>, s| Some(best.map_or(s, |b| b.max(s))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 9309, section 5.1 "Simple Example".
    const RFC_SIMPLE: &str = "\
User-Agent: *
Disallow: *.gif$
Disallow: /example/
Allow: /publications/

User-Agent: foobot
Disallow:/
Allow:/example/page.html
Allow:/example/allowed.gif

User-Agent: barbot
User-Agent: bazbot
Disallow: /example/page.html

User-Agent: quxbot

EOF
";

    #[test]
    fn rfc_9309_simple_example() {
        let r = Robots::parse(RFC_SIMPLE);
        // The `*` group.
        for ua in ["otherbot", "OtherBot/2.0 (+https://example.org)", ""] {
            assert!(!r.allowed(ua, "/image.gif"), "{ua}");
            assert!(!r.allowed(ua, "/a/b/c.gif"), "{ua}");
            assert!(r.allowed(ua, "/image.gif?x=1"), "{ua}: `$` anchors the end");
            assert!(!r.allowed(ua, "/example/"), "{ua}");
            assert!(!r.allowed(ua, "/example/page.html"), "{ua}");
            assert!(r.allowed(ua, "/publications/"), "{ua}");
            assert!(r.allowed(ua, "/publications/report.pdf"), "{ua}");
            assert!(r.allowed(ua, "/publications/figure.gif"), "{ua}: the longer Allow wins");
            assert!(r.allowed(ua, "/"), "{ua}");
            assert!(r.allowed(ua, "/example"), "{ua}");
        }
        // foobot: its own group only.
        assert!(!r.allowed("foobot", "/"));
        assert!(!r.allowed("FooBot", "/publications/"));
        assert!(r.allowed("foobot", "/example/page.html"));
        assert!(r.allowed("foobot", "/example/allowed.gif"));
        assert!(!r.allowed("foobot", "/example/other.gif"));
        // barbot and bazbot share a group.
        for ua in ["barbot", "bazbot", "BazBot/1.0"] {
            assert!(!r.allowed(ua, "/example/page.html"), "{ua}");
            assert!(r.allowed(ua, "/example/allowed.gif"), "{ua}");
            assert!(r.allowed(ua, "/image.gif"), "{ua}: the * group does not apply");
        }
        // quxbot: a group without rules allows everything.
        assert!(r.allowed("quxbot", "/example/page.html"));
        assert!(r.allowed("quxbot", "/image.gif"));
    }

    #[test]
    fn rfc_9309_longest_match_example() {
        // RFC 9309, section 5.2.
        let r = Robots::parse(
            "User-Agent: foobot\nAllow: /example/page/\nDisallow: /example/page/disallowed.gif\n",
        );
        assert!(!r.allowed("foobot", "/example/page/disallowed.gif"));
        assert!(r.allowed("foobot", "/example/page/"));
        assert!(r.allowed("foobot", "/example/page/allowed.gif"));
        assert!(r.allowed("foobot", "https://example.com/example/page/index.html"));
        assert!(!r.allowed("foobot", "https://example.com/example/page/disallowed.gif?x#frag"));
    }

    #[test]
    fn rfc_9309_percent_encoding_table() {
        // Section 2.2.2: the path as it must be compared.
        assert_eq!(normalise("/foo/bar?baz=quz", false, 9999), "/foo/bar?baz=quz");
        assert_eq!(
            normalise("/foo/bar?baz=https%3A%2F%2Ffoo.bar", false, 9999),
            "/foo/bar?baz=https%3A%2F%2Ffoo.bar"
        );
        assert_eq!(normalise("/foo/bar/\u{30C4}", false, 9999), "/foo/bar/%E3%83%84");
        assert_eq!(normalise("/foo/bar/%E3%83%84", false, 9999), "/foo/bar/%E3%83%84");
        assert_eq!(normalise("/foo/bar/%e3%83%84", false, 9999), "/foo/bar/%E3%83%84");
        assert_eq!(normalise("/foo/bar/%62%61%7A", false, 9999), "/foo/bar/baz");

        let r = Robots::parse("User-agent: *\nDisallow: /foo/bar/\u{30C4}\nDisallow: /caf%c3%a9\nDisallow: /b%61z\n");
        assert!(!r.allowed("x", "/foo/bar/%E3%83%84"));
        assert!(!r.allowed("x", "/foo/bar/\u{30C4}/more"));
        assert!(!r.allowed("x", "/caf\u{e9}"));
        assert!(!r.allowed("x", "/caf%C3%A9"));
        assert!(!r.allowed("x", "/baz"));
        assert!(!r.allowed("x", "/%62az"));
        assert!(r.allowed("x", "/foo/bar/"));
    }

    #[test]
    fn rfc_9309_special_characters_table() {
        // Section 2.2.3: literal `*` and `$` are matched through their escapes.
        let r = Robots::parse(
            "User-agent: *\nDisallow: /path/file-with-a-%2A.html\nDisallow: /path/foo-%24\nAllow: /this/path/exactly$\nDisallow: /this/\n",
        );
        assert!(!r.allowed("x", "/path/file-with-a-*.html"));
        assert!(!r.allowed("x", "/path/file-with-a-%2A.html"));
        assert!(r.allowed("x", "/path/file-with-a-b.html"));
        assert!(!r.allowed("x", "/path/foo-$"));
        assert!(r.allowed("x", "/path/foo-"));
        assert!(r.allowed("x", "/this/path/exactly"));
        assert!(!r.allowed("x", "/this/path/exactly/not"));
        // A `$` that is not final is a literal.
        let r = Robots::parse("User-agent: *\nDisallow: /a$b\n");
        assert!(!r.allowed("x", "/a$b"));
        assert!(!r.allowed("x", "/a%24bc"));
        assert!(r.allowed("x", "/a"));
    }

    #[test]
    fn google_documented_path_matching() {
        let cases: [(&str, &[&str], &[&str]); 7] = [
            (
                "/fish",
                &["/fish", "/fish.html", "/fish/salmon.html", "/fishheads", "/fishheads/yummy.html", "/fish.php?id=anything"],
                &["/Fish.asp", "/catfish", "/?id=fish", "/desired-fish"],
            ),
            (
                "/fish*",
                &["/fish", "/fish.html", "/fish/salmon.html", "/fishheads", "/fish.php?id=anything"],
                &["/Fish.asp", "/catfish", "/?id=fish", "/desired-fish"],
            ),
            (
                "/fish/",
                &["/fish/", "/fish/?id=anything", "/fish/salmon.htm"],
                &["/fish", "/fish.html", "/animals/fish/", "/Fish/Salmon.asp"],
            ),
            (
                "/*.php",
                &["/index.php", "/filename.php", "/folder/filename.php", "/folder/filename.php?parameters", "/folder/any.php.file.html", "/filename.php/"],
                &["/", "/windows.PHP"],
            ),
            (
                "/*.php$",
                &["/filename.php", "/folder/filename.php"],
                &["/filename.php?parameters", "/filename.php/", "/filename.php5", "/windows.PHP"],
            ),
            ("/fish*.php", &["/fish.php", "/fishheads/catfish.php?parameters"], &["/Fish.PHP"]),
            ("/", &["/", "/anything"], &[]),
        ];
        for (pattern, matching, not_matching) in cases {
            let r = Robots::parse(&format!("User-agent: *\nDisallow: {pattern}\n"));
            for path in matching {
                assert!(!r.allowed("anybot", path), "{pattern} should match {path}");
            }
            for path in not_matching {
                assert!(r.allowed("anybot", path), "{pattern} should not match {path}");
            }
        }
    }

    #[test]
    fn google_documented_precedence() {
        let verdict = |rules: &str, path: &str| {
            Robots::parse(&format!("User-agent: *\n{rules}")).allowed("anybot", path)
        };
        assert!(verdict("Allow: /p\nDisallow: /\n", "/page"));
        assert!(verdict("Allow: /folder\nDisallow: /folder\n", "/folder/page"));
        assert!(!verdict("Allow: /page\nDisallow: /*.htm\n", "/page.htm"));
        assert!(verdict("Allow: /page\nDisallow: /*.ph\n", "/page.php5"));
        assert!(verdict("Allow: /$\nDisallow: /\n", "/"));
        assert!(!verdict("Allow: /$\nDisallow: /\n", "/page.htm"));
        // Order of the lines is irrelevant.
        assert!(verdict("Disallow: /\nAllow: /p\n", "/page"));
    }

    #[test]
    fn group_selection_and_merging() {
        let txt = "\
User-agent: ebookcadeaubot
Disallow: /a

User-agent: *
Disallow: /b

User-agent: OtherBot
User-agent: eBookCadeauBot
Disallow: /c
";
        let r = Robots::parse(txt);
        let ua = "eBookCadeauBot/0.1.0 (+https://example.org/bot; bot@example.org)";
        // Both groups naming the token are combined; the * group is ignored.
        assert!(!r.allowed(ua, "/a"));
        assert!(!r.allowed(ua, "/c"));
        assert!(r.allowed(ua, "/b"));
        assert!(!r.allowed("otherbot", "/c"));
        assert!(r.allowed("otherbot", "/a"));
        assert!(!r.allowed("thirdbot", "/b"));
        assert!(r.allowed("thirdbot", "/a"));
        // A token is matched whole, not as a prefix or substring.
        assert!(!r.allowed("ebookcadeau", "/b"));
        assert!(r.allowed("ebookcadeau", "/a"));
    }

    #[test]
    fn defaults_and_edge_cases() {
        // Empty file, no groups, rules before any user-agent: everything allowed.
        assert!(Robots::parse("").allowed("x", "/anything"));
        assert!(Robots::parse("Disallow: /\n").allowed("x", "/anything"));
        assert!(Robots::parse("# just a comment\n\n").allowed("x", "/"));
        // Empty Disallow restricts nothing.
        assert!(Robots::parse("User-agent: *\nDisallow:\n").allowed("x", "/a"));
        // /robots.txt is always fetchable.
        let closed = Robots::parse("User-agent: *\nDisallow: /\n");
        assert!(closed.allowed("x", "/robots.txt"));
        assert!(!closed.allowed("x", "/robots.txt.bak"));
        assert!(!closed.allowed("x", ""));
        assert!(!closed.allowed("x", "page"));
        // Comments, BOM, CR line endings, odd spacing and key case.
        let r = Robots::parse(
            "\u{FEFF}USER-AGENT : * # everyone\r\nDISALLOW :   /private/   # secret\rallow:/private/ok\r\n",
        );
        assert!(!r.allowed("x", "/private/x"));
        assert!(r.allowed("x", "/private/ok"));
        // A Disallow without a leading slash is read as a path; an Allow is dropped.
        let r = Robots::parse("User-agent: *\nDisallow: private\nAllow: private/ok\n");
        assert!(!r.allowed("x", "/private/ok"));
        // Misspelt Disallow still restricts.
        assert!(!Robots::parse("User-agent: *\nDisalow: /x\n").allowed("a", "/x"));
        // Unknown records do not split a group.
        let r = Robots::parse("User-agent: a\nSitemap: https://example.org/s.xml\nUser-agent: b\nDisallow: /\n");
        assert!(!r.allowed("a", "/x"));
        assert!(!r.allowed("b", "/x"));
    }

    #[test]
    fn crawl_delay_extra() {
        let r = Robots::parse(
            "User-agent: a\nCrawl-delay: 5\n\nUser-agent: b\nCrawl-delay: 10\nDisallow: /x\n\nUser-agent: *\nCrawl-delay: 1.5\n",
        );
        assert_eq!(r.crawl_delay("a"), Some(5.0));
        assert_eq!(r.crawl_delay("B/1.0"), Some(10.0));
        assert_eq!(r.crawl_delay("other"), Some(1.5));
        // The delay line does not split the rule group (a and b share the Disallow).
        assert!(!r.allowed("a", "/x"));
        assert!(!r.allowed("b", "/x"));
        // Garbage values are ignored.
        let r = Robots::parse("User-agent: *\nCrawl-delay: soon\nCrawl-delay: -3\nCrawl-delay: NaN\nCrawl-delay: inf\n");
        assert_eq!(r.crawl_delay("x"), None);
        assert_eq!(Robots::parse("Crawl-delay: 3\n").crawl_delay("x"), None);
        // A specific group without a delay does not inherit the * delay.
        let r = Robots::parse("User-agent: *\nCrawl-delay: 9\n\nUser-agent: a\nDisallow: /x\n");
        assert_eq!(r.crawl_delay("a"), None);
    }

    #[test]
    fn fetch_outcomes() {
        let body = b"User-agent: *\nDisallow: /no\n";
        let ok = Robots::from_response(200, body);
        assert!(!ok.allowed("x", "/no") && ok.allowed("x", "/yes"));
        for status in [400, 401, 403, 404, 410] {
            let r = Robots::from_response(status, body);
            assert!(r.allowed("x", "/no"), "{status}");
            assert!(!r.is_disallow_all());
        }
        for status in [429, 500, 502, 503, 301, 100, 0, 999] {
            let r = Robots::from_response(status, body);
            assert!(!r.allowed("x", "/yes"), "{status}");
            assert!(r.is_disallow_all());
        }
        assert!(!Robots::unreachable().allowed("x", "/"));
        assert!(Robots::allow_all().allowed("x", "/"));
        // Invalid UTF-8 does not prevent parsing.
        let r = Robots::from_response(200, b"User-agent: *\nDisallow: /caf\xE9\n\xFF\xFE\nDisallow: /b\n");
        assert!(!r.allowed("x", "/b"));
    }

    #[test]
    fn size_and_count_limits() {
        // Rules beyond the first 500 KiB are not read.
        let mut txt = String::from("User-agent: *\nDisallow: /early\n");
        while txt.len() < MAX_ROBOTS_BYTES + 100 {
            txt.push_str("# padding padding padding padding padding padding padding\n");
        }
        txt.push_str("Disallow: /late\n");
        let r = Robots::parse(&txt);
        assert!(!r.allowed("x", "/early"));
        assert!(r.allowed("x", "/late"));
        // An absurd number of rules closes the site.
        let mut many = String::from("User-agent: *\n");
        for i in 0..=MAX_RULES {
            many.push_str(&format!("Allow: /a{i}\n"));
        }
        assert!(Robots::parse(&many).is_disallow_all());
        // Over-long rules: Disallow kept as a prefix, Allow dropped.
        let long = "x".repeat(5000);
        let r = Robots::parse(&format!("User-agent: *\nDisallow: /{long}\nAllow: /{long}y\n"));
        assert!(!r.allowed("x", &format!("/{long}")));
        assert!(!r.allowed("x", &format!("/{}", "x".repeat(3000))));
        assert!(r.allowed("x", "/xxx"));
        // Very long paths do not blow up.
        let r = Robots::parse("User-agent: *\nDisallow: /*a*a*a*a*a*a*a*b$\n");
        assert!(r.allowed("x", &format!("/{}", "a".repeat(100_000))));
    }

    #[test]
    fn garbage_never_panics() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let alphabet: Vec<char> =
            "User-agent:DisallowAllow*$/%#\n\r :\u{e9}\u{30C4}\0abc123crawl-delay.".chars().collect();
        for round in 0..300 {
            let mut txt = String::new();
            for _ in 0..(round * 7 % 400) {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                txt.push(alphabet[(seed % alphabet.len() as u64) as usize]);
            }
            let r = Robots::parse(&txt);
            let _ = r.allowed(&txt, &txt);
            let _ = r.allowed("bot", "/%zz%2/\u{e9}%");
            let _ = r.crawl_delay(&txt);
        }
    }
}
