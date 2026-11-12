//! URL normalisation for compact QR codes.

use crate::QrError;

/// Rewrites a URL into the equivalent form that encodes most compactly in a QR code.
///
/// # The rule
///
/// QR alphanumeric mode (5½ bits per character instead of 8) only has **upper-case**
/// letters. RFC 3986 makes exactly two parts of a URL case-insensitive — the scheme
/// (§3.1) and the host (§3.2.2) — so exactly those two are converted to upper case:
///
/// ```text
/// https://ebk.example/k/7K3M9QX2RD   →   HTTPS://EBK.EXAMPLE/k/7K3M9QX2RD
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
/// * URLs without an authority (`mailto:`, `urn:` …) only have their scheme upper-cased.
///
/// Phone cameras resolve `HTTPS://EBK.EXAMPLE/…` exactly like the lower-case form.
///
/// # Errors
///
/// * [`QrError::Empty`] for an empty string;
/// * [`QrError::InvalidUrl`] if the text contains whitespace or control characters, has no
///   valid scheme (`scheme:` — a bare `example.com/x` is rejected, since phones would not
///   treat it as a link reliably), or is an `http(s)` URL without a host.
///
/// ```
/// use ebc_qr::normalize_url;
/// assert_eq!(
///     normalize_url("https://ebk.example/K/7K3M9QX2RD").as_deref(),
///     Ok("HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD")
/// );
/// assert_eq!(
///     normalize_url("https://Shop.example/gift?c=aBc#top").as_deref(),
///     Ok("HTTPS://SHOP.EXAMPLE/gift?c=aBc#top")
/// );
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

    let mut out = String::with_capacity(url.len());
    out.push_str(&scheme.to_ascii_uppercase());
    out.push(':');

    let Some(after_slashes) = rest.strip_prefix("//") else {
        if is_http {
            return Err(QrError::InvalidUrl("an http(s) URL needs \"//\" and a host"));
        }
        out.push_str(rest);
        return Ok(out);
    };
    let authority_end = after_slashes.find(['/', '?', '#']).unwrap_or(after_slashes.len());
    let (authority, tail) = after_slashes.split_at(authority_end);
    let (userinfo, host_port) = match authority.rfind('@') {
        Some(at) => authority.split_at(at + 1),
        None => ("", authority),
    };
    if is_http && (host_port.is_empty() || host_port.starts_with(':')) {
        return Err(QrError::InvalidUrl("an http(s) URL needs a host"));
    }

    out.push_str("//");
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
            // No authority: only the scheme changes.
            ("mailto:Zoe@Example.com", "MAILTO:Zoe@Example.com"),
            ("urn:isbn:9782070360024", "URN:isbn:9782070360024"),
            ("tel:+33123456789", "TEL:+33123456789"),
            ("file:///home/a/b.epub", "FILE:///home/a/b.epub"),
            ("ebk+app://Open/Item", "EBK+APP://OPEN/Item"),
        ];
        for (input, expected) in cases {
            assert_eq!(normalize_url(input).as_deref(), Ok(expected), "{input}");
        }
    }

    #[test]
    fn is_idempotent_and_never_changes_anything_but_ascii_case_before_the_path() {
        let inputs = [
            "https://ebk.example/k/aBcDeF?x=Y#z",
            "https://a.b.c.example:444/",
            "ftp://u:p@files.example/dir/File.TXT",
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
        assert_eq!(normalize_url("a:").as_deref(), Ok("A:"));
        assert_eq!(normalize_url("x://@").as_deref(), Ok("X://@"));
    }
}
