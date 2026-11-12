//! URL normalisation for compact QR codes.

use crate::QrError;

/// Rewrites an `http(s)` URL into the equivalent form that encodes most compactly in a QR
/// code; every other URL is returned unchanged.
///
/// # The rule
///
/// QR alphanumeric mode (5½ bits per character instead of 8) only has **upper-case**
/// letters. RFC 3986 makes exactly two parts of a URL case-insensitive — the scheme
/// (§3.1) and the host (§3.2.2) — so for `http` and `https` URLs exactly those two are
/// converted to upper case:
///
/// ```text
/// https://ebk.example/k/7K3M9QX2RD   →   HTTPS://EBK.EXAMPLE/k/7K3M9QX2RD
/// https://ebk.example/K/BCDFGHJKLMNP →   HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNP   (all alphanumeric)
/// ```
///
/// Everything else is copied **byte for byte**:
///
/// * the path, query and fragment are case-sensitive, so they are never rewritten. A path
///   that already consists only of QR alphanumeric characters (`0-9 A-Z $ % * + - . / :`
///   and space) is by definition already upper-case and simply benefits from alphanumeric
///   mode; one that contains lower-case letters is left alone and the encoder switches to
///   byte mode for those parts (mixed segments). To get a fully alphanumeric code, make
///   the *server* accept an upper-case path (e.g. `/K/…`) and pass that in;
/// * user information (`user:password@`) is case-sensitive and kept as is;
/// * an IP-literal host (`[…]`) is kept as is, because an IPv6 zone identifier may be
///   case-sensitive;
/// * only ASCII letters are changed; internationalised host names keep their non-ASCII
///   characters (IDNA lower-cases the result anyway);
/// * **URLs of any other scheme are not touched at all** — neither `mailto:`/`urn:`/`tel:`
///   nor custom or deep-link schemes such as `opds://authorize/…` or `myapp://Open/Item`.
///   RFC 3986 calls scheme and host case-insensitive, but the software that consumes
///   such links does not agree: the WHATWG URL parser keeps the host of a non-special
///   scheme verbatim (an "opaque host"), apps compare the "host" of their own scheme as a
///   literal keyword, and Android matches intent-filter schemes and hosts
///   case-sensitively (the last point is from memory of the Android documentation, not
///   re-verified). They are still validated (see Errors).
///
/// # Backslashes
///
/// Browsers — the WHATWG URL parser, which is what opens the link once the code has been
/// scanned — read `\` as `/` in `http(s)` URLs, whereas RFC 3986 does not allow it at
/// all. In `https://shop.example\@files.example/Abc` a browser sees the host
/// `shop.example` and the path `/@files.example/Abc`; an RFC parser sees the user
/// information `shop.example\` and the host `files.example`. No rewriting is safe under
/// both readings, so an `http(s)` URL with a backslash **before its first `/`, `?` or `#`**
/// is rejected. A backslash further on is in a part that is copied verbatim anyway.
///
/// # What is not verified
///
/// That phone cameras open `HTTPS://EBK.EXAMPLE/…` exactly like the lower-case form is
/// what RFC 3986 and the common scanner libraries (ZXing) lead one to expect, and browsers
/// lower-case scheme and host when they parse the link — but it has **not yet been tested
/// on stock iOS and Android camera apps** (the research notes carry the same caveat). One
/// known difference: an upper-case host does not match an Android App Link filter, so the
/// link opens in the browser rather than in an app registered for it.
///
/// # Errors
///
/// * [`QrError::Empty`] for an empty string;
/// * [`QrError::InvalidUrl`] if the text contains whitespace or control characters, has no
///   valid scheme (`scheme:` — a bare `example.com/x` is rejected, since phones would not
///   treat it as a link reliably), is an `http(s)` URL without a host, or is an `http(s)`
///   URL with a backslash before its path (see above).
///
/// ```
/// use ebc_qr::normalize_url;
/// assert_eq!(
///     normalize_url("https://ebk.example/K/BCDFGHJKLMNP").as_deref(),
///     Ok("HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNP")
/// );
/// assert_eq!(
///     normalize_url("https://Shop.example/gift?c=aBc#top").as_deref(),
///     Ok("HTTPS://SHOP.EXAMPLE/gift?c=aBc#top")
/// );
/// assert_eq!(normalize_url("opds://Authorize/x").as_deref(), Ok("opds://Authorize/x"));
/// assert!(normalize_url("https://ebk.example\\k\\abc").is_err());
/// ```
pub fn normalize_url(url: &str) -> Result<String, QrError> {
    if url.is_empty() {
        return Err(QrError::Empty);
    }
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(QrError::InvalidUrl("it contains whitespace or control characters"));
    }

    let Some((scheme, rest)) = url.split_once(':') else {
        return Err(QrError::InvalidUrl("it has no scheme (such as \"https:\")"));
    };
    let mut scheme_chars = scheme.chars();
    let scheme_ok = scheme_chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && scheme_chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !scheme_ok {
        return Err(QrError::InvalidUrl("it has no valid scheme (such as \"https:\")"));
    }
    let is_http = scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https");
    if !is_http {
        // Not ours to rewrite: see "any other scheme" in the documentation.
        return Ok(url.to_owned());
    }

    let Some(after_slashes) = rest.strip_prefix("//") else {
        return Err(QrError::InvalidUrl("an http(s) URL needs \"//\" and a host"));
    };
    let authority_end = after_slashes.find(['/', '?', '#']).unwrap_or(after_slashes.len());
    let (authority, tail) = after_slashes.split_at(authority_end);
    if authority.contains('\\') {
        return Err(QrError::InvalidUrl(
            "it has a backslash before the path, which browsers read as \"/\": \
             where the host ends is ambiguous",
        ));
    }
    let (userinfo, host_port) = match authority.rfind('@') {
        Some(at) => authority.split_at(at + 1),
        None => ("", authority),
    };
    if host_port.is_empty() || host_port.starts_with(':') {
        return Err(QrError::InvalidUrl("an http(s) URL needs a host"));
    }

    let mut out = String::with_capacity(url.len());
    out.push_str(&scheme.to_ascii_uppercase());
    out.push_str("://");
    out.push_str(userinfo);
    if host_port.starts_with('[') {
        out.push_str(host_port);
    } else {
        out.push_str(&host_port.to_ascii_uppercase());
    }
    out.push_str(tail);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_cases_scheme_and_host_only() {
        let cases = [
            ("https://ebk.example/k/7K3M9QX2RD", "HTTPS://EBK.EXAMPLE/k/7K3M9QX2RD"),
            ("https://ebk.example/K/7K3M9QX2RD", "HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD"),
            ("HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD", "HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD"),
            ("http://example.com", "HTTP://EXAMPLE.COM"),
            ("http://example.com/", "HTTP://EXAMPLE.COM/"),
            (
                "https://Example.com:8443/a/B?q=Zz&x=1#Frag",
                "HTTPS://EXAMPLE.COM:8443/a/B?q=Zz&x=1#Frag",
            ),
            ("https://example.com?q=lower", "HTTPS://EXAMPLE.COM?q=lower"),
            ("https://example.com#frag", "HTTPS://EXAMPLE.COM#frag"),
            // Query and fragment may contain "@", ":" and "//" without confusing the split.
            ("https://example.com/p?mail=a@b.c", "HTTPS://EXAMPLE.COM/p?mail=a@b.c"),
            ("https://example.com/r?u=http://x.y/z", "HTTPS://EXAMPLE.COM/r?u=http://x.y/z"),
            ("https://example.com/a:b//c", "HTTPS://EXAMPLE.COM/a:b//c"),
            // User information is case-sensitive and kept; the last "@" delimits it.
            ("https://User:pA@ss@host.example/x", "HTTPS://User:pA@ss@HOST.EXAMPLE/x"),
            // Percent-encoded and internationalised hosts: ASCII letters only.
            ("https://b%c3%bccher.example/", "HTTPS://B%C3%BCCHER.EXAMPLE/"),
            ("https://bücher.example/ü", "HTTPS://BüCHER.EXAMPLE/ü"),
            ("https://xn--bcher-kva.example/", "HTTPS://XN--BCHER-KVA.EXAMPLE/"),
            // IP literals are left alone (zone identifiers can be case-sensitive).
            ("http://[fe80::1%25eth0]:8080/x", "HTTP://[fe80::1%25eth0]:8080/x"),
            ("http://192.168.0.1/admin", "HTTP://192.168.0.1/admin"),
            // A backslash after the authority is in the part copied verbatim.
            ("https://ebk.example/k\\aBc?x=\\y#\\z", "HTTPS://EBK.EXAMPLE/k\\aBc?x=\\y#\\z"),
            ("https://ebk.example?q=a\\b", "HTTPS://EBK.EXAMPLE?q=a\\b"),
        ];
        for (input, expected) in cases {
            assert_eq!(normalize_url(input).as_deref(), Ok(expected), "{input}");
        }
    }

    /// Audit finding 1: only http and https are rewritten. (This replaces the earlier
    /// expectations `"ebk+app://Open/Item" -> "EBK+APP://OPEN/Item"`, `mailto:` ->
    /// `MAILTO:` and so on, which pinned the defect.)
    #[test]
    fn other_schemes_are_returned_byte_for_byte() {
        let untouched = [
            "mailto:Zoe@Example.com",
            "urn:isbn:9782070360024",
            "tel:+33123456789",
            "file:///home/a/b.epub",
            "ebk+app://Open/Item",
            "kobo://Library/open?id=Abc",
            "opds://authorize/",
            "opds://example.com/user/shelf",
            "intent://Scan/#Intent;scheme=zxing;package=com.example;end",
            "ftp://u:p@files.example/dir/File.TXT",
            "ws://Sock.example/x",
            "httpx://Host.example/x",
            "https+foo://Host.example/x",
            "xhttp://Host.example/x",
            // Backslashes are not special outside http(s) either.
            "myapp://Open\\Item",
            "a:",
            "x://@",
        ];
        for input in untouched {
            assert_eq!(normalize_url(input).as_deref(), Ok(input), "{input}");
        }
        // Whatever the case of the scheme, "http" and "https" are recognised…
        for input in ["hTTp://a.example/x", "HTTPS://a.example/x", "Https://a.example/x"] {
            let out = normalize_url(input).expect("valid");
            assert!(
                out.starts_with("HTTP://A.EXAMPLE/x") || out.starts_with("HTTPS://A.EXAMPLE/x")
            );
        }
        // …and other schemes are still validated.
        for invalid in ["mailto:a b@example.org", "opds://authorize/\n", "1x:y", "+x:y"] {
            assert!(matches!(normalize_url(invalid), Err(QrError::InvalidUrl(_))), "{invalid:?}");
        }
    }

    /// Audit finding 2: a backslash must not let the path be taken for the host.
    #[test]
    fn backslash_before_the_path_of_an_http_url_is_rejected() {
        let ambiguous = [
            "https://ebk.example\\k\\abc",
            "https://ebk.example\\k\\7k3m9qx2rd",
            "http://ebk.example\\Gift\\aBcDeF?x=y",
            "https://ebk.example\\@evil.example/Path",
            "https://shop.example\\@files.example/Abc",
            "https://shop.example\\x@files.example",
            "https://user\\name@host.example/",
            "https://ebk.example\\",
            "https://ebk.example:443\\k",
            "https://\\ebk.example/k",
            "HTTP://[::1]\\x",
            // WHATWG also accepts backslashes in place of the "//".
            "https:\\\\ebk.example\\k",
            "https:/\\ebk.example/k",
            "https:\\/ebk.example/k",
        ];
        for input in ambiguous {
            assert!(
                matches!(normalize_url(input), Err(QrError::InvalidUrl(_))),
                "{input:?} -> {:?}",
                normalize_url(input)
            );
        }
    }

    /// The property the module promises, on generated input: for an accepted http(s) URL
    /// only ASCII case changes, and nothing changes from the first `/`, `\`, `?` or `#`
    /// after the `//` — the earliest point at which any parser (RFC 3986 or WHATWG) can
    /// see the path begin.
    #[test]
    fn generated_http_urls_never_have_their_path_query_or_fragment_altered() {
        let pieces = [
            "ebk.example",
            "Shop.Example",
            "user",
            "pA:ss",
            "@",
            "@",
            ":",
            "8443",
            "[::1]",
            "/",
            "/",
            "\\",
            "\\",
            "?",
            "#",
            "k",
            "K",
            "aBc",
            "7k3m9qx2rd",
            "%2f",
            "%5C",
            ".",
            "-",
            "=",
            "&",
            "é",
            "~",
        ];
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = |bound: usize| {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            (state.wrapping_mul(0x2545_F491_4F6C_DD1D) % bound as u64) as usize
        };
        let (mut accepted, mut rejected_backslash) = (0, 0);
        for round in 0..30_000 {
            let mut url = String::from(["https://", "http://", "HTTPS://", "hTtP://"][round % 4]);
            for _ in 0..1 + next(8) {
                url.push_str(pieces[next(pieces.len())]);
            }
            let after = &url[url.find("//").expect("prefix") + 2..];
            let cut = after.find(['/', '\\', '?', '#']).unwrap_or(after.len());
            let tail = &after[cut..];
            match normalize_url(&url) {
                Ok(out) => {
                    accepted += 1;
                    assert!(!tail.starts_with('\\'), "{url:?} accepted as {out:?}");
                    assert!(out.ends_with(tail), "{url:?} -> {out:?}");
                    assert_eq!(out.len(), url.len());
                    assert!(out.eq_ignore_ascii_case(&url));
                    assert_eq!(normalize_url(&out).as_deref(), Ok(out.as_str()));
                    // User information is untouched as well.
                    if let Some(at) = after[..cut].rfind('@') {
                        assert!(out.contains(&after[..=at]), "{url:?} -> {out:?}");
                    }
                }
                Err(QrError::InvalidUrl(_)) => {
                    if tail.starts_with('\\') {
                        rejected_backslash += 1;
                    }
                }
                Err(other) => panic!("{url:?}: {other:?}"),
            }
        }
        assert!(accepted > 10_000 && rejected_backslash > 1_000, "{accepted} {rejected_backslash}");
    }

    #[test]
    fn is_idempotent_and_never_changes_anything_but_ascii_case_before_the_path() {
        let inputs = [
            "https://ebk.example/k/aBcDeF?x=Y#z",
            "https://a.b.c.example:444/",
            "http://u:p@files.example/dir/File.TXT",
            "https://bücher.example/straße",
        ];
        for input in inputs {
            let once = normalize_url(input).expect("valid");
            assert_eq!(normalize_url(&once).as_deref(), Ok(once.as_str()), "{input}");
            assert_eq!(once.len(), input.len());
            assert!(once.eq_ignore_ascii_case(input));
            // The part from the first "/" after the authority is byte-identical.
            let tail_start = input.find("//").expect("authority") + 2;
            let tail = &input[tail_start + input[tail_start..].find('/').expect("path")..];
            assert!(once.ends_with(tail), "{once} / {tail}");
        }
    }

    #[test]
    fn rejects_what_is_not_a_link() {
        assert_eq!(normalize_url(""), Err(QrError::Empty));
        let invalid = [
            "ebk.example/k/ABC",          // no scheme
            "/k/ABC",                     // relative reference
            "://ebk.example",             // empty scheme
            "1http://ebk.example",        // scheme must start with a letter
            "ht tp://ebk.example",        // space
            "https://ebk.example/a b",    // space in path
            "https://ebk.example/\n",     // control character
            " https://ebk.example/",      // leading space
            "https://ebk.example/\u{a0}", // non-breaking space
            "https://ebk.example/\u{0}",
            "https://ebk.example/\u{7f}",
            "https://",          // no host
            "https:///path",     // no host
            "https://:443/",     // no host, only a port
            "https://user@/x",   // no host after user information
            "https:ebk.example", // http(s) without "//"
            "http:/ebk.example",
            "héttps://ebk.example", // non-ASCII scheme
            "ht_tp://ebk.example",
        ];
        for input in invalid {
            assert!(
                matches!(normalize_url(input), Err(QrError::InvalidUrl(_))),
                "{input:?} -> {:?}",
                normalize_url(input)
            );
        }
    }

    #[test]
    fn hostile_input_does_not_panic() {
        let long = format!("https://ebk.example/{}", "a".repeat(100_000));
        assert_eq!(normalize_url(&long).map(|s| s.len()), Ok(long.len()));
        for weird in
            [":", "::", "a:", "a:/", "a://", "a://@", "a://@@@", "a:?", "a:#", "a://[", "a://]:[@"]
        {
            let _ = normalize_url(weird);
        }
        assert_eq!(normalize_url("a:").as_deref(), Ok("a:"));
        assert_eq!(normalize_url("x://@").as_deref(), Ok("x://@"));
    }
}
