//! Text-and-data-mining opt-out signals (W3C Community Group "TDM Reservation
//! Protocol", TDMRep) and the non-standard `noai` / `noimageai` robots tags.
//!
//! A fetcher that wants to *analyse* a page or an image (colours, hash, font class)
//! must check these first; any reservation means no fetch for analysis and no facts
//! derived (`docs/research/02-cover-sources.md`, §7.4, implications 23 and 31).
//!
//! Precedence, as the protocol specifies it: the site-wide file
//! `/.well-known/tdmrep.json`, then the `tdm-reservation` response header, then the
//! HTML `<meta name="tdm-reservation">` tag — each later source supersedes the earlier
//! ones, and an absent value never resets an earlier one. Any value other than `0` or
//! `1` is treated as unset. The HTML tag covers the document only, **not the images it
//! references**: an image is judged on its own response headers and its own origin's
//! file.
//!
//! The shape of `tdmrep.json` (an array of `{ "location", "tdm-reservation",
//! "tdm-policy" }` objects, `*` and `$` in `location` as in robots.txt, first matching
//! rule applies) is implemented from the research's reading of the specification; it
//! was not tested against a live deployment.

use serde_json::Value;

use crate::robots::{compile_pattern, normalise_path, pattern_matches};

/// Only this many leading bytes of a `tdmrep.json` are parsed.
pub const MAX_TDMREP_BYTES: usize = 500 * 1024;
/// At most this many rules of a `tdmrep.json` are kept.
const MAX_TDMREP_RULES: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    pattern: String,
    anchored: bool,
    reservation: Option<bool>,
    policy: Option<String>,
}

/// A parsed `/.well-known/tdmrep.json`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TdmRep {
    rules: Vec<Rule>,
}

/// Reads a `tdm-reservation` value: `1` → reserved, `0` → not reserved, anything else
/// → unset.
#[must_use]
pub fn parse_reservation(value: &str) -> Option<bool> {
    match value.trim() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

fn reservation_of(value: &Value) -> Option<bool> {
    match value {
        Value::Number(n) => match n.as_u64() {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        },
        Value::String(s) => parse_reservation(s),
        _ => None,
    }
}

impl TdmRep {
    /// Parses the body of a `tdmrep.json`. Anything that is not the expected array
    /// yields an empty declaration (no rule): a malformed file declares nothing.
    #[must_use]
    pub fn parse(body: &[u8]) -> Self {
        if body.len() > MAX_TDMREP_BYTES {
            return Self::default();
        }
        let Ok(Value::Array(items)) = serde_json::from_slice::<Value>(body) else {
            return Self::default();
        };
        let rules = items
            .iter()
            .filter_map(|item| {
                let object = item.as_object()?;
                let location = object.get("location")?.as_str()?;
                let (mut pattern, anchored) = compile_pattern(location.trim());
                if !pattern.starts_with('/') && !pattern.starts_with('*') {
                    pattern.insert(0, '/');
                }
                Some(Rule {
                    pattern,
                    anchored,
                    reservation: object.get("tdm-reservation").and_then(reservation_of),
                    policy: object
                        .get("tdm-policy")
                        .and_then(Value::as_str)
                        .map(|s| s.chars().take(2048).collect()),
                })
            })
            .take(MAX_TDMREP_RULES)
            .collect();
        Self { rules }
    }

    fn rule_for(&self, path: &str) -> Option<&Rule> {
        let path = normalise_path(path);
        self.rules.iter().find(|rule| pattern_matches(&rule.pattern, rule.anchored, &path))
    }

    /// The reservation the file declares for `path` (path and query, or a full URL):
    /// the first rule whose `location` matches. `None` when no rule matches or the
    /// matching rule carries no valid value.
    #[must_use]
    pub fn reservation(&self, path: &str) -> Option<bool> {
        self.rule_for(path).and_then(|rule| rule.reservation)
    }

    /// The `tdm-policy` URL declared for `path`, if any.
    #[must_use]
    pub fn policy(&self, path: &str) -> Option<&str> {
        self.rule_for(path).and_then(|rule| rule.policy.as_deref())
    }
}

fn header<'a>(
    headers: &'a [(String, String)],
    name: &str,
) -> impl Iterator<Item = &'a str> + use<'a> {
    let name = name.to_ascii_lowercase();
    headers.iter().filter(move |(n, _)| n.eq_ignore_ascii_case(&name)).map(|(_, v)| v.as_str())
}

/// The `tdm-reservation` response header, if present with a valid value.
#[must_use]
pub fn header_reservation(headers: &[(String, String)]) -> Option<bool> {
    header(headers, "tdm-reservation").find_map(parse_reservation)
}

/// `true` when a robots directive list (the value of an `X-Robots-Tag` header or of a
/// `<meta name="robots">` tag) contains `noai` or `noimageai`.
#[must_use]
pub fn directives_say_noai(value: &str) -> bool {
    value
        .split([',', ' ', '\t', ';'])
        .map(|token| token.rsplit(':').next().unwrap_or_default().trim())
        .any(|token| token.eq_ignore_ascii_case("noai") || token.eq_ignore_ascii_case("noimageai"))
}

/// `true` when an `X-Robots-Tag` response header carries `noai` or `noimageai`.
#[must_use]
pub fn headers_say_noai(headers: &[(String, String)]) -> bool {
    header(headers, "x-robots-tag").any(directives_say_noai)
}

/// The opt-out signals gathered for one resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Signals {
    /// From the origin's `tdmrep.json` ([`TdmRep::reservation`]).
    pub file: Option<bool>,
    /// From the response's `tdm-reservation` header ([`header_reservation`]).
    pub header: Option<bool>,
    /// From the HTML `<meta name="tdm-reservation">` tag
    /// ([`crate::scrape::PageInfo::tdm_reservation`]); pages only, never images.
    pub meta: Option<bool>,
    /// `noai` / `noimageai` seen in `X-Robots-Tag` or in a robots meta tag.
    pub noai: bool,
}

impl Signals {
    /// The effective `tdm-reservation`: the last source in the order file, header,
    /// meta that carries a value.
    #[must_use]
    pub fn effective(&self) -> Option<bool> {
        self.meta.or(self.header).or(self.file)
    }

    /// `true` when analysis is closed for the resource: an effective reservation of 1,
    /// or a `noai` / `noimageai` tag.
    #[must_use]
    pub fn reserved(&self) -> bool {
        self.noai || self.effective() == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(n, v)| ((*n).to_owned(), (*v).to_owned())).collect()
    }

    #[test]
    fn file_rules_first_match_applies() {
        let file = TdmRep::parse(
            br#"[
              {"location": "/images/libre/*", "tdm-reservation": 0},
              {"location": "/images/*.jpg$", "tdm-reservation": 1, "tdm-policy": "https://example.org/policy.json"},
              {"location": "/odd", "tdm-reservation": 2},
              {"location": "/text", "tdm-reservation": "1"},
              {"no-location": true},
              "junk"
            ]"#,
        );
        assert_eq!(file.reservation("/images/libre/a.jpg"), Some(false));
        assert_eq!(file.reservation("/images/a.jpg"), Some(true));
        assert_eq!(file.policy("/images/a.jpg"), Some("https://example.org/policy.json"));
        assert_eq!(file.reservation("/images/a.jpg?x=1"), None);
        assert_eq!(file.reservation("https://example.org/images/b.jpg"), Some(true));
        assert_eq!(file.reservation("/odd"), None);
        assert_eq!(file.reservation("/text/page"), Some(true));
        assert_eq!(file.reservation("/elsewhere"), None);
    }

    #[test]
    fn malformed_files_declare_nothing() {
        for body in [&b""[..], b"{}", b"[", b"null", b"<html>", b"[[[[", b"\xff\xfe"] {
            assert_eq!(TdmRep::parse(body), TdmRep::default());
        }
        let huge = vec![b' '; MAX_TDMREP_BYTES + 1];
        assert_eq!(TdmRep::parse(&huge), TdmRep::default());
        let deep = "[".repeat(100_000);
        assert_eq!(TdmRep::parse(deep.as_bytes()), TdmRep::default());
    }

    #[test]
    fn header_and_tag_values() {
        assert_eq!(header_reservation(&headers(&[("TDM-Reservation", " 1 ")])), Some(true));
        assert_eq!(header_reservation(&headers(&[("tdm-reservation", "0")])), Some(false));
        assert_eq!(header_reservation(&headers(&[("tdm-reservation", "yes")])), None);
        assert_eq!(header_reservation(&headers(&[("content-type", "1")])), None);
        assert!(headers_say_noai(&headers(&[("X-Robots-Tag", "noindex, NoAI")])));
        assert!(headers_say_noai(&headers(&[("x-robots-tag", "googlebot: noimageai")])));
        assert!(!headers_say_noai(&headers(&[("x-robots-tag", "noindex, nofollow")])));
        assert!(!directives_say_noai("noairplane"));
    }

    #[test]
    fn precedence_file_header_meta() {
        let s = |file, header, meta| Signals { file, header, meta, noai: false };
        assert_eq!(s(None, None, None).effective(), None);
        assert!(!s(None, None, None).reserved());
        assert!(s(Some(true), None, None).reserved());
        // A header supersedes the file; a meta tag supersedes both.
        assert!(!s(Some(true), Some(false), None).reserved());
        assert!(s(Some(true), Some(false), Some(true)).reserved());
        assert!(!s(Some(true), Some(true), Some(false)).reserved());
        // An absent later value never resets an earlier one.
        assert!(s(Some(true), None, None).reserved());
        assert!(s(None, Some(true), None).reserved());
        // noai closes analysis whatever TDMRep says.
        assert!(Signals { file: Some(false), header: None, meta: None, noai: true }.reserved());
    }
}
