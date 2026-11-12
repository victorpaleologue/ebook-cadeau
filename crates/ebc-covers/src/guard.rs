//! URL and host guard: what may be contacted or linked at all.
//!
//! Every URL that could lead to a request or to a link shown to the user goes through
//! [`check_url`] first — the candidates built by [`crate::candidates`], a page URL pasted
//! by the user, and **every image URL extracted from a page** ([`crate::scrape`]). A page
//! is attacker-controlled input: without this check, `og:image` is a way to make the
//! service or the user's browser contact an internal address or a host the project
//! promised never to touch.
//!
//! The checks are syntactic; they cannot see DNS. A fetcher must therefore **also**
//! check the resolved address at connection time and after every redirect, with
//! [`is_public_ip`] (`docs/research/02-cover-sources.md`, implication 17).
//!
//! The denylist is the "hosts never contacted" list of the research (§7.1 rule 2,
//! implication 7, extended by the fact-check): retailer and aggregator sites and their
//! image CDNs, Goodreads-derived services, library subscription image services, image
//! search engines. The retailer URL patterns behind it are from memory in the research
//! and unverified; the list errs on the side of refusing. No policy setting overrides it.

use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

/// URLs longer than this are refused.
pub const MAX_URL_BYTES: usize = 2048;

/// Why a URL is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Refusal {
    /// Not an absolute URL, or longer than [`MAX_URL_BYTES`].
    Unparseable,
    /// Any scheme other than `https`.
    NotHttps,
    /// The URL carries a user name or a password.
    Credentials,
    /// A port other than 443.
    Port,
    /// The host is an IP address literal (in any notation). Covers are served from
    /// named hosts; a literal is how internal services are reached.
    IpAddress,
    /// A name that designates a local or internal machine (`localhost`, single-label
    /// names, `.local`, `.internal`, `.lan`, `.home.arpa`, …).
    LocalName,
    /// The host is on the built-in denylist; the string names the family
    /// (e.g. `"Amazon"`).
    DeniedHost(&'static str),
    /// The URL is a known "no image" asset.
    PlaceholderUrl,
    /// The host is on the caller's publisher opt-out list.
    OptedOut,
    /// robots.txt of the origin disallows the path for our user agent.
    RobotsDisallow,
    /// robots.txt obedience is on and no robots.txt verdict was supplied.
    RobotsNotChecked,
    /// Lookups do not run in this mode (reseller mode uses the reseller's feed only).
    LookupsDisabled,
    /// The deployment restricts pasted pages to an allow-list of publisher hosts and
    /// this host is not on it.
    NotAllowListed,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unparseable => f.write_str("not an absolute URL of reasonable length"),
            Self::NotHttps => f.write_str("only https URLs are accepted"),
            Self::Credentials => f.write_str("URLs with a user name or password are refused"),
            Self::Port => f.write_str("only the default https port is accepted"),
            Self::IpAddress => f.write_str("IP address hosts are refused"),
            Self::LocalName => f.write_str("local or internal host names are refused"),
            Self::DeniedHost(family) => write!(
                f,
                "{family} hosts are never contacted or linked; supply the image file directly"
            ),
            Self::PlaceholderUrl => f.write_str("the URL is a known \"no image\" asset"),
            Self::OptedOut => f.write_str("the publisher opted out of cover lookups"),
            Self::RobotsDisallow => f.write_str("robots.txt disallows this path"),
            Self::RobotsNotChecked => f.write_str("robots.txt was not checked for this origin"),
            Self::LookupsDisabled => f.write_str("cover lookups do not run in reseller mode"),
            Self::NotAllowListed => f.write_str("this host is not on the list of publisher sites"),
        }
    }
}

/// Host suffixes that are never contacted or linked, with the family they belong to.
///
/// A host matches an entry when it equals the suffix or ends with `.` + suffix.
pub const DENIED_HOST_SUFFIXES: &[(&str, &str)] = &[
    // Amazon: site, legacy and current image hosts (see also the label rule below).
    ("ssl-images-amazon.com", "Amazon"),
    ("images-amazon.com", "Amazon"),
    ("media-amazon.com", "Amazon"),
    ("amazon-adsystem.com", "Amazon"),
    ("audible.com", "Amazon"),
    ("audible.fr", "Amazon"),
    // Goodreads and Goodreads-derived services.
    ("goodreads.com", "Goodreads"),
    ("gr-assets.com", "Goodreads"),
    ("bookcover.longitood.com", "Goodreads"),
    // French retailers and aggregators, and their image CDNs.
    ("fnac-static.com", "Fnac"),
    ("decitre.fr", "Decitre"),
    ("di-static.com", "Decitre"),
    ("furet.com", "Furet du Nord"),
    ("cultura.com", "Cultura"),
    ("epagine.fr", "ePagine/Tite-Live"),
    ("leslibraires.fr", "ePagine/Tite-Live"),
    ("placedeslibraires.fr", "ePagine/Tite-Live"),
    ("babelio.com", "Babelio"),
    // Other retailers named by the research and its fact-check.
    ("ibs.it", "IBS"),
    ("casadellibro.com", "Casa del Libro"),
    ("kobo.com", "Kobo"),
    ("kobobooks.com", "Kobo"),
    ("mzstatic.com", "Apple Books"),
    ("itunes.apple.com", "Apple Books"),
    ("books.apple.com", "Apple Books"),
    // Library subscription image services.
    ("syndetics.com", "Syndetics"),
    ("btol.com", "Content Cafe"),
    // Image search engines.
    ("bing.com", "image search"),
    ("bing.net", "image search"),
    ("duckduckgo.com", "image search"),
    ("qwant.com", "image search"),
    ("yandex.com", "image search"),
    ("yandex.net", "image search"),
    ("yandex.ru", "image search"),
];

/// Brand labels denied under every top-level domain (`amazon.fr`, `www.fnac.be`, …): the
/// host is refused when the label directly left of its public suffix-like tail equals
/// one of these. Approximated as "any label equals the brand", which also refuses
/// `amazon.example.org`; that is the safe side.
const DENIED_LABELS: &[(&str, &str)] = &[("amazon", "Amazon"), ("fnac", "Fnac")];

/// The denylist family of `host`, if it is denied.
#[must_use]
pub fn denied_host_family(host: &str) -> Option<&'static str> {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    for (suffix, family) in DENIED_HOST_SUFFIXES {
        if host_matches_suffix(&host, suffix) {
            return Some(family);
        }
    }
    for (label, family) in DENIED_LABELS {
        if host.split('.').any(|l| l == *label) {
            return Some(family);
        }
    }
    // Google image search (`images.google.*`) and its thumbnail cache
    // (`encrypted-tbnN.gstatic.com`); Google Books itself is not denied.
    if host.starts_with("images.google.")
        || (host.ends_with(".gstatic.com") && host.starts_with("encrypted-tbn"))
    {
        return Some("image search");
    }
    None
}

/// `true` when `host` is on the built-in denylist.
#[must_use]
pub fn is_denied_host(host: &str) -> bool {
    denied_host_family(host).is_some()
}

/// `true` when `host` equals `suffix` or is a subdomain of it (both lower-case).
pub(crate) fn host_matches_suffix(host: &str, suffix: &str) -> bool {
    host == suffix
        || host
            .strip_suffix(suffix)
            .is_some_and(|rest| rest.ends_with('.'))
}

/// Shape checks only: absolute `https` URL, no credentials, default port, a named host
/// that is not local. The denylist is **not** consulted (used for covers a reseller
/// supplies from its own hosts).
///
/// # Errors
/// The first [`Refusal`] that applies.
pub fn check_url_shape(url: &str) -> Result<Url, Refusal> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_URL_BYTES {
        return Err(Refusal::Unparseable);
    }
    let parsed = Url::parse(trimmed).map_err(|_| Refusal::Unparseable)?;
    if parsed.scheme() != "https" {
        return Err(Refusal::NotHttps);
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(Refusal::Credentials);
    }
    if parsed.port().is_some_and(|p| p != 443) {
        return Err(Refusal::Port);
    }
    match parsed.host() {
        None => return Err(Refusal::Unparseable),
        Some(Host::Ipv4(_) | Host::Ipv6(_)) => return Err(Refusal::IpAddress),
        Some(Host::Domain(name)) => {
            if is_local_name(name) {
                return Err(Refusal::LocalName);
            }
        }
    }
    Ok(parsed)
}

/// Full check of a URL before it is requested or shown as a link: [`check_url_shape`],
/// then the built-in denylist and the known placeholder URLs.
///
/// # Errors
/// The first [`Refusal`] that applies.
pub fn check_url(url: &str) -> Result<Url, Refusal> {
    let parsed = check_url_shape(url)?;
    if let Some(family) = parsed.host_str().and_then(denied_host_family) {
        return Err(Refusal::DeniedHost(family));
    }
    if crate::placeholder::is_placeholder_url(parsed.as_str()) {
        return Err(Refusal::PlaceholderUrl);
    }
    Ok(parsed)
}

/// Names that designate the local machine or an internal network.
fn is_local_name(name: &str) -> bool {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if name.is_empty() || !name.contains('.') {
        return true; // single-label: resolved through the local search domain
    }
    // A name made only of digits and dots is an address in disguise that the URL
    // parser did not normalise.
    if name.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return true;
    }
    const LOCAL_SUFFIXES: [&str; 9] = [
        "localhost",
        "local",
        "localdomain",
        "internal",
        "intranet",
        "lan",
        "home.arpa",
        "corp",
        "test",
    ];
    LOCAL_SUFFIXES.iter().any(|suffix| host_matches_suffix(&name, suffix))
}

/// `true` when `ip` is a globally routable unicast address — the check a fetcher must
/// apply to the **resolved** address of every connection and every redirect target.
///
/// Refused: loopback, private (RFC 1918), link-local (which includes the cloud metadata
/// address `169.254.169.254`), carrier-grade NAT, unspecified, multicast, broadcast,
/// documentation and benchmarking ranges, IPv6 unique-local, and IPv4-mapped,
/// IPv4-compatible or NAT64 forms of any refused IPv4 address.
#[must_use]
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_public_v4(v4),
        IpAddr::V6(v6) => is_public_v6(v6),
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..128).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..32).contains(&b))
        || (a == 192 && b == 168)
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224)
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    let embedded_v4 = |hi: u16, lo: u16| {
        let [a, b] = hi.to_be_bytes();
        let [c, d] = lo.to_be_bytes();
        Ipv4Addr::new(a, b, c, d)
    };
    // ::/96 (unspecified, loopback, IPv4-compatible) and ::ffff:0:0/96 (IPv4-mapped).
    if s[..5] == [0, 0, 0, 0, 0] && (s[5] == 0 || s[5] == 0xFFFF) {
        return s[5] == 0xFFFF && is_public_v4(embedded_v4(s[6], s[7]));
    }
    // NAT64 well-known prefix 64:ff9b::/96.
    if s[..6] == [0x64, 0xFF9B, 0, 0, 0, 0] {
        return is_public_v4(embedded_v4(s[6], s[7]));
    }
    !((s[0] & 0xFE00) == 0xFC00 // unique local fc00::/7
        || (s[0] & 0xFFC0) == 0xFE80 // link-local fe80::/10
        || (s[0] & 0xFFC0) == 0xFEC0 // site-local (deprecated)
        || (s[0] & 0xFF00) == 0xFF00 // multicast
        || (s[0] == 0x2001 && s[1] == 0x0DB8) // documentation
        || (s[0] == 0x2002) // 6to4: embeds an arbitrary IPv4 address
        || (s[0] == 0x2001 && s[1] == 0)) // Teredo
}

/// The origin (`https://host[:port]`) of an already checked URL.
#[must_use]
pub fn origin_of(url: &Url) -> String {
    url.origin().ascii_serialization()
}

/// Path and query of a URL, as robots.txt rules are matched against it.
#[must_use]
pub fn path_and_query(url: &Url) -> String {
    match url.query() {
        Some(q) => format!("{}?{q}", url.path()),
        None => url.path().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_https_urls() {
        for ok in [
            "https://covers.openlibrary.org/b/id/123-M.jpg?default=false",
            "https://www.gallimard.example.fr/img/couv.jpg",
            "  https://editeur.example.com:443/a b.jpg ",
        ] {
            assert!(check_url(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn shape_refusals() {
        let cases = [
            ("", Refusal::Unparseable),
            ("/relative/path.jpg", Refusal::Unparseable),
            ("http://example.com/a.jpg", Refusal::NotHttps),
            ("ftp://example.com/a.jpg", Refusal::NotHttps),
            ("data:image/png;base64,AAAA", Refusal::NotHttps),
            ("javascript:alert(1)", Refusal::NotHttps),
            ("file:///etc/passwd", Refusal::NotHttps),
            ("https://user:pw@example.com/a.jpg", Refusal::Credentials),
            ("https://user@example.com/a.jpg", Refusal::Credentials),
            ("https://example.com:8443/a.jpg", Refusal::Port),
            ("https://127.0.0.1/a.jpg", Refusal::IpAddress),
            ("https://2130706433/a.jpg", Refusal::IpAddress),
            ("https://0x7f.1/a.jpg", Refusal::IpAddress),
            ("https://169.254.169.254/latest/meta-data/", Refusal::IpAddress),
            ("https://[::1]/a.jpg", Refusal::IpAddress),
            ("https://[::ffff:10.0.0.1]/a.jpg", Refusal::IpAddress),
            ("https://93.184.216.34/a.jpg", Refusal::IpAddress),
            ("https://localhost/a.jpg", Refusal::LocalName),
            ("https://LOCALHOST./a.jpg", Refusal::LocalName),
            ("https://intranet/a.jpg", Refusal::LocalName),
            ("https://printer.local/a.jpg", Refusal::LocalName),
            ("https://metadata.google.internal/computeMetadata/v1/", Refusal::LocalName),
            ("https://nas.lan/a.jpg", Refusal::LocalName),
            ("https://foo.localhost/a.jpg", Refusal::LocalName),
        ];
        for (url, expected) in cases {
            assert_eq!(check_url(url).err(), Some(expected), "{url}");
        }
        let long = format!("https://example.com/{}", "a".repeat(MAX_URL_BYTES));
        assert_eq!(check_url(&long).err(), Some(Refusal::Unparseable));
    }

    #[test]
    fn denylist_covers_the_research_list() {
        let denied = [
            ("https://images-na.ssl-images-amazon.com/images/P/2070360024.01.LZZZZZZZ.jpg", "Amazon"),
            ("https://m.media-amazon.com/images/I/41abc.jpg", "Amazon"),
            ("https://www.amazon.fr/dp/2070360024", "Amazon"),
            ("https://www.amazon.co.uk/dp/2070360024", "Amazon"),
            ("https://i.gr-assets.com/images/S/x.jpg", "Goodreads"),
            ("https://www.goodreads.com/book/show/1", "Goodreads"),
            ("https://bookcover.longitood.com/bookcover?isbn=9782070360024", "Goodreads"),
            ("https://static.fnac-static.com/multimedia/x.jpg", "Fnac"),
            ("https://www.fnac.com/a1", "Fnac"),
            ("https://livre.fnac.com/a1", "Fnac"),
            ("https://products-images.di-static.com/image/x/9782070360024-475x500-1.jpg", "Decitre"),
            ("https://www.decitre.fr/livres/x.html", "Decitre"),
            ("https://www.furet.com/x", "Furet du Nord"),
            ("https://cdn.cultura.com/cdn-cgi/image/width=180/media/x.jpg", "Cultura"),
            ("https://images.epagine.fr/024/9782070360024_1_75.jpg", "ePagine/Tite-Live"),
            ("https://www.leslibraires.fr/livre/1", "ePagine/Tite-Live"),
            ("https://www.babelio.com/couv/x.jpg", "Babelio"),
            ("https://www.ibs.it/images/9788806219350_0_536_0_75.jpg", "IBS"),
            ("https://imagessl4.casadellibro.com/a/l/t7/24/9782070360024.jpg", "Casa del Libro"),
            ("https://www.kobo.com/fr/fr/ebook/x", "Kobo"),
            ("https://is1-ssl.mzstatic.com/image/thumb/x.jpg", "Apple Books"),
            ("https://secure.syndetics.com/index.aspx?isbn=9782070360024/LC.JPG", "Syndetics"),
            ("https://contentcafe2.btol.com/ContentCafe/Jacket.aspx", "Content Cafe"),
            ("https://images.google.com/imgres?imgurl=x", "image search"),
            ("https://encrypted-tbn0.gstatic.com/images?q=tbn:x", "image search"),
            ("https://tse1.mm.bing.net/th?id=x", "image search"),
        ];
        for (url, family) in denied {
            assert_eq!(check_url(url).err(), Some(Refusal::DeniedHost(family)), "{url}");
        }
        // Near misses are not denied.
        for ok in [
            "https://books.google.com/books?id=x&printsec=frontcover&img=1",
            "https://fonts.gstatic.com/s/x.woff2",
            "https://notamazon.example.org/x.jpg",
            "https://www.kobold-verlag.example.de/x.jpg",
            "https://editions-decitre-example.org/x.jpg",
        ] {
            assert!(check_url(ok).is_ok(), "{ok}");
        }
        assert!(is_denied_host("WWW.Amazon.DE."));
        assert!(!is_denied_host("openlibrary.org"));
    }

    #[test]
    fn shape_check_skips_the_denylist_only() {
        assert!(check_url_shape("https://static.fnac-static.com/multimedia/x.jpg").is_ok());
        assert_eq!(check_url_shape("http://static.fnac-static.com/x.jpg").err(), Some(Refusal::NotHttps));
        assert_eq!(check_url_shape("https://10.0.0.8/x.jpg").err(), Some(Refusal::IpAddress));
    }

    #[test]
    fn placeholder_urls_are_refused() {
        assert_eq!(
            check_url("https://cdn.example.org/images/G/01/x-site/icons/no-img-sm.gif").err(),
            Some(Refusal::PlaceholderUrl)
        );
    }

    #[test]
    fn public_ip_classification() {
        let private = [
            "127.0.0.1", "10.1.2.3", "172.16.0.1", "172.31.255.255", "192.168.1.1",
            "169.254.169.254", "100.64.0.1", "0.0.0.0", "224.0.0.1", "255.255.255.255",
            "192.0.2.1", "198.18.0.1", "::1", "::", "fc00::1", "fd12:3456::1", "fe80::1",
            "ff02::1", "::ffff:127.0.0.1", "::ffff:192.168.0.1", "64:ff9b::a00:1",
            "2001:db8::1", "::10.0.0.1", "2002:a00:1::1",
        ];
        for ip in private {
            assert!(!is_public_ip(ip.parse().expect("ip")), "{ip}");
        }
        let public = [
            "93.184.216.34", "8.8.8.8", "172.32.0.1", "100.128.0.1", "2606:4700::1111",
            "::ffff:8.8.8.8", "64:ff9b::808:808",
        ];
        for ip in public {
            assert!(is_public_ip(ip.parse().expect("ip")), "{ip}");
        }
    }

    #[test]
    fn origin_and_path_helpers() {
        let url = check_url("https://www.editeur.example.fr/livre/1?x=1#top").expect("ok");
        assert_eq!(origin_of(&url), "https://www.editeur.example.fr");
        assert_eq!(path_and_query(&url), "/livre/1?x=1");
    }
}
