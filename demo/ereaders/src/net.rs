//! HTTP plumbing shared by every simulated client.
//!
//! One rule: **nothing is unbounded**. Every exchange has a deadline, every body a size cap,
//! every redirect chain a length limit — so a hostile or broken server yields a typed
//! [`NetError`] instead of a hang or an out-of-memory.

use std::time::Duration;

use reqwest::Method;
use url::Url;

use crate::error::NetError;

/// Network limits of a simulated client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetConfig {
    /// Time allowed to establish a connection.
    pub connect_timeout: Duration,
    /// Time allowed for one whole exchange, body included.
    pub request_timeout: Duration,
    /// Longest redirect chain followed.
    pub max_redirects: usize,
    /// Use `HTTP_PROXY` / `HTTPS_PROXY` from the environment. Off by default so tests against
    /// `127.0.0.1` never leave the machine.
    pub use_env_proxy: bool,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_redirects: 10,
            use_env_proxy: false,
        }
    }
}

/// One HTTP exchange made by a simulated client, for assertions on what was sent where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exchange {
    /// `GET`, `POST`…
    pub method: String,
    /// Requested URL.
    pub url: Url,
    /// Status received.
    pub status: u16,
    /// `Referer` header sent, if any.
    pub referer: Option<String>,
    /// Whether an `Authorization` header was sent.
    pub authorized: bool,
}

pub(crate) struct Body {
    pub(crate) content_type: String,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) struct Request {
    pub(crate) method: Method,
    pub(crate) url: Url,
    /// Extra headers (lower-case names).
    pub(crate) headers: Vec<(&'static str, String)>,
    pub(crate) body: Option<Body>,
}

impl Request {
    pub(crate) fn get(url: Url) -> Self {
        Self { method: Method::GET, url, headers: Vec::new(), body: None }
    }

    pub(crate) fn header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }
}

/// Status line and headers of a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Head {
    pub(crate) url: Url,
    pub(crate) status: u16,
    /// Lower-case names, in order of arrival.
    pub(crate) headers: Vec<(String, String)>,
}

impl Head {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    pub(crate) fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Parses what a person typed or a document linked to. A missing scheme means `http://`,
/// as in a browser's address bar. Only `http` and `https` are accepted.
pub(crate) fn parse_url(input: &str) -> Result<Url, NetError> {
    let trimmed = input.trim();
    let invalid = |reason: String| NetError::InvalidUrl { url: input.to_owned(), reason };
    if trimmed.is_empty() {
        return Err(invalid("empty".to_owned()));
    }
    if trimmed.len() > 8192 {
        return Err(invalid("longer than 8192 bytes".to_owned()));
    }
    let candidate = if trimmed.contains("://") {
        trimmed.to_owned()
    } else {
        format!("http://{trimmed}")
    };
    let url = Url::parse(&candidate).map_err(|e| invalid(e.to_string()))?;
    check_scheme(&url)?;
    if url.host_str().is_none_or(str::is_empty) {
        return Err(invalid("no host".to_owned()));
    }
    Ok(url)
}

pub(crate) fn check_scheme(url: &Url) -> Result<(), NetError> {
    match url.scheme() {
        "http" | "https" => Ok(()),
        other => Err(NetError::UnsupportedScheme { url: url.to_string(), scheme: other.to_owned() }),
    }
}

pub(crate) fn same_origin(a: &Url, b: &Url) -> bool {
    a.scheme() == b.scheme()
        && a.host_str() == b.host_str()
        && a.port_or_known_default() == b.port_or_known_default()
}

/// A minimal cookie jar: host-only cookies with `Path` and `Secure`, no expiry dates
/// (`Max-Age=0` deletes). Enough to tell whether a flow *depends* on cookies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CookieJar {
    cookies: Vec<Cookie>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cookie {
    host: String,
    path: String,
    name: String,
    value: String,
    secure: bool,
}

impl CookieJar {
    const MAX_COOKIES: usize = 64;
    const MAX_COOKIE_BYTES: usize = 4096;

    fn store(&mut self, url: &Url, set_cookie: &str) {
        if set_cookie.len() > Self::MAX_COOKIE_BYTES {
            return;
        }
        let Some(host) = url.host_str() else { return };
        let mut parts = set_cookie.split(';');
        let Some((name, value)) = parts.next().and_then(|pair| pair.split_once('=')) else {
            return;
        };
        let (name, value) = (name.trim(), value.trim());
        if name.is_empty() {
            return;
        }
        let mut path = "/".to_owned();
        let mut secure = false;
        let mut delete = false;
        for attribute in parts {
            let (key, val) = match attribute.split_once('=') {
                Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim()),
                None => (attribute.trim().to_ascii_lowercase(), ""),
            };
            match key.as_str() {
                "path" if val.starts_with('/') => path = val.to_owned(),
                "secure" => secure = true,
                "max-age" => delete = val.parse::<i64>().is_ok_and(|age| age <= 0),
                _ => {}
            }
        }
        self.cookies.retain(|c| !(c.host == host && c.path == path && c.name == name));
        if !delete && self.cookies.len() < Self::MAX_COOKIES {
            self.cookies.push(Cookie {
                host: host.to_owned(),
                path,
                name: name.to_owned(),
                value: value.to_owned(),
                secure,
            });
        }
    }

    fn header_for(&self, url: &Url) -> Option<String> {
        let host = url.host_str()?;
        let pairs: Vec<String> = self
            .cookies
            .iter()
            .filter(|c| c.host == host)
            .filter(|c| !c.secure || url.scheme() == "https")
            .filter(|c| path_matches(url.path(), &c.path))
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();
        (!pairs.is_empty()).then(|| pairs.join("; "))
    }

    pub(crate) fn pairs(&self) -> Vec<(String, String)> {
        self.cookies.iter().map(|c| (c.name.clone(), c.value.clone())).collect()
    }
}

fn path_matches(request_path: &str, cookie_path: &str) -> bool {
    if cookie_path == "/" || request_path == cookie_path {
        return true;
    }
    request_path
        .strip_prefix(cookie_path)
        .is_some_and(|rest| cookie_path.ends_with('/') || rest.starts_with('/'))
}

/// A bounded HTTP client that never follows redirects on its own.
pub(crate) struct Http {
    client: reqwest::Client,
    config: NetConfig,
}

impl std::fmt::Debug for Http {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Http").field("config", &self.config).finish_non_exhaustive()
    }
}

impl Http {
    pub(crate) fn new(user_agent: &str, config: NetConfig) -> Result<Self, NetError> {
        // reqwest is built with `rustls-no-provider` (shared with demo/stores): a process-wide
        // crypto provider must exist before any client is built, or reqwest panics. Installing
        // twice is harmless (the second call returns an error we ignore).
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut builder = reqwest::Client::builder()
            .user_agent(user_agent)
            .redirect(reqwest::redirect::Policy::none())
            .referer(false)
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .http1_only()
            .pool_max_idle_per_host(2);
        if !config.use_env_proxy {
            builder = builder.no_proxy();
        }
        let client = builder.build().map_err(|e| NetError::Client(e.to_string()))?;
        Ok(Self { client, config })
    }

    /// Sends `request`, following redirects up to the configured bound. Returns the final
    /// response with its body still unread, so the caller can pick a size cap that fits what
    /// the response turned out to be.
    pub(crate) async fn fetch(
        &self,
        mut request: Request,
        mut jar: Option<&mut CookieJar>,
        log: &mut Vec<Exchange>,
    ) -> Result<(Head, reqwest::Response), NetError> {
        let first_url = request.url.to_string();
        let mut hops = 0usize;
        loop {
            check_scheme(&request.url)?;
            let mut builder = self.client.request(request.method.clone(), request.url.clone());
            for (name, value) in &request.headers {
                builder = builder.header(*name, value);
            }
            if let Some(cookie) = jar.as_deref().and_then(|jar| jar.header_for(&request.url)) {
                builder = builder.header("cookie", cookie);
            }
            if let Some(body) = &request.body {
                builder = builder.header("content-type", &body.content_type).body(body.bytes.clone());
            }
            let url_text = request.url.to_string();
            let sent = tokio::time::timeout(
                self.config.request_timeout + Duration::from_millis(250),
                builder.send(),
            )
            .await;
            let response = match sent {
                Err(_) => return Err(NetError::Timeout { url: url_text }),
                Ok(Err(err)) => return Err(map_reqwest(&err, &url_text)),
                Ok(Ok(response)) => response,
            };
            let head = Head {
                url: request.url.clone(),
                status: response.status().as_u16(),
                headers: response
                    .headers()
                    .iter()
                    .map(|(name, value)| {
                        (name.as_str().to_owned(), String::from_utf8_lossy(value.as_bytes()).into_owned())
                    })
                    .collect(),
            };
            log.push(Exchange {
                method: request.method.as_str().to_owned(),
                url: request.url.clone(),
                status: head.status,
                referer: request
                    .headers
                    .iter()
                    .find(|(name, _)| *name == "referer")
                    .map(|(_, value)| value.clone()),
                authorized: request.headers.iter().any(|(name, _)| *name == "authorization"),
            });
            if let Some(jar) = jar.as_deref_mut() {
                for (name, value) in &head.headers {
                    if name == "set-cookie" {
                        jar.store(&head.url, value);
                    }
                }
            }
            if !matches!(head.status, 301 | 302 | 303 | 307 | 308) {
                return Ok((head, response));
            }
            // Redirect. The body of a 3xx is never read.
            drop(response);
            hops += 1;
            if hops > self.config.max_redirects {
                return Err(NetError::TooManyRedirects {
                    url: first_url,
                    limit: self.config.max_redirects,
                });
            }
            let location = head
                .header("location")
                .filter(|l| !l.trim().is_empty() && l.len() <= 8192)
                .ok_or_else(|| NetError::BadRedirect { url: url_text.clone() })?;
            let next = head
                .url
                .join(location.trim())
                .map_err(|_| NetError::BadRedirect { url: url_text.clone() })?;
            check_scheme(&next)?;
            let to_get = head.status == 303
                || (matches!(head.status, 301 | 302) && request.method == Method::POST);
            if to_get && request.method != Method::HEAD {
                request.method = Method::GET;
                request.body = None;
            }
            if !same_origin(&request.url, &next) {
                // Credentials never follow a redirect to another origin.
                request.headers.retain(|(name, _)| *name != "authorization");
            }
            request.url = next;
        }
    }

    /// Reads a response body, refusing anything above `limit` bytes.
    pub(crate) async fn read(
        &self,
        head: &Head,
        mut response: reqwest::Response,
        limit: u64,
    ) -> Result<Vec<u8>, NetError> {
        let url = head.url.to_string();
        let too_large = || NetError::BodyTooLarge { url: url.clone(), limit };
        if response.content_length().is_some_and(|len| len > limit) {
            return Err(too_large());
        }
        let read_all = async {
            let mut out: Vec<u8> = Vec::new();
            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) => {
                        if (out.len() + chunk.len()) as u64 > limit {
                            return Err(too_large());
                        }
                        out.extend_from_slice(&chunk);
                    }
                    Ok(None) => return Ok(out),
                    Err(err) => return Err(map_reqwest(&err, &url)),
                }
            }
        };
        match tokio::time::timeout(self.config.request_timeout + Duration::from_millis(250), read_all)
            .await
        {
            Ok(result) => result,
            Err(_) => Err(NetError::Timeout { url: url.clone() }),
        }
    }

    /// `fetch` + `read` for clients that just want a document.
    pub(crate) async fn fetch_bytes(
        &self,
        request: Request,
        limit: u64,
        log: &mut Vec<Exchange>,
    ) -> Result<(Head, Vec<u8>), NetError> {
        let (head, response) = self.fetch(request, None, log).await?;
        let body = self.read(&head, response, limit).await?;
        Ok((head, body))
    }
}

fn map_reqwest(err: &reqwest::Error, url: &str) -> NetError {
    if err.is_timeout() {
        return NetError::Timeout { url: url.to_owned() };
    }
    // Walk to the root cause: reqwest's own message is often just "error sending request".
    let mut reason = err.to_string();
    let mut source = std::error::Error::source(err);
    let mut depth = 0;
    while let Some(inner) = source {
        reason = inner.to_string();
        source = inner.source();
        depth += 1;
        if depth > 8 {
            break;
        }
    }
    NetError::Request { url: url.to_owned(), reason }
}

/// File name proposed by a response: `Content-Disposition`'s plain `filename=` (old browsers
/// ignore the RFC 5987 `filename*=` form — *assumption*, it post-dates their engines), else the
/// last path segment of the URL, percent-decoded.
pub(crate) fn response_file_name(head: &Head) -> String {
    if let Some(name) = head.header("content-disposition").and_then(disposition_file_name) {
        return name;
    }
    let segment = head.url.path_segments().and_then(|mut s| s.next_back()).unwrap_or("");
    let decoded = percent_decode(segment);
    if decoded.is_empty() { "download".to_owned() } else { decoded }
}

fn disposition_file_name(value: &str) -> Option<String> {
    for part in value.split(';').skip(1) {
        let Some((key, val)) = part.split_once('=') else { continue };
        if !key.trim().eq_ignore_ascii_case("filename") {
            continue;
        }
        let val = val.trim();
        let unquoted = val.strip_prefix('"').and_then(|v| v.strip_suffix('"')).unwrap_or(val);
        let name = unquoted.replace("\\\"", "\"");
        // Never trust a path: keep the last component only.
        let name = name.rsplit(['/', '\\']).next().unwrap_or("").trim().to_owned();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn percent_decode(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok());
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn head(url: &str, headers: &[(&str, &str)]) -> Head {
        Head {
            url: Url::parse(url).unwrap(),
            status: 200,
            headers: headers.iter().map(|(n, v)| ((*n).to_owned(), (*v).to_owned())).collect(),
        }
    }

    #[test]
    fn typed_addresses_get_a_scheme_and_odd_ones_are_rejected() {
        assert_eq!(parse_url("ebk.example/k/ABC").unwrap().as_str(), "http://ebk.example/k/ABC");
        assert_eq!(parse_url("  https://ebk.example ").unwrap().as_str(), "https://ebk.example/");
        assert_eq!(parse_url("127.0.0.1:8080").unwrap().port(), Some(8080));
        assert!(matches!(parse_url(""), Err(NetError::InvalidUrl { .. })));
        assert!(matches!(parse_url("   "), Err(NetError::InvalidUrl { .. })));
        assert!(matches!(parse_url("http://"), Err(NetError::InvalidUrl { .. })));
        assert!(matches!(
            parse_url("file:///etc/passwd"),
            Err(NetError::UnsupportedScheme { scheme, .. }) if scheme == "file"
        ));
        assert!(matches!(parse_url("javascript://x"), Err(NetError::UnsupportedScheme { .. })));
        assert!(matches!(parse_url(&"a".repeat(9000)), Err(NetError::InvalidUrl { .. })));
    }

    #[test]
    fn origins_compare_scheme_host_and_effective_port() {
        let a = Url::parse("http://ebk.example/a").unwrap();
        assert!(same_origin(&a, &Url::parse("http://ebk.example:80/b").unwrap()));
        assert!(!same_origin(&a, &Url::parse("https://ebk.example/b").unwrap()));
        assert!(!same_origin(&a, &Url::parse("http://ebk.example:8080/b").unwrap()));
        assert!(!same_origin(&a, &Url::parse("http://store.example/b").unwrap()));
    }

    #[test]
    fn cookie_jar_scopes_by_host_path_and_secure() {
        let mut jar = CookieJar::default();
        let url = Url::parse("http://ebk.example/k/ABC").unwrap();
        jar.store(&url, "sid=1; Path=/; HttpOnly");
        jar.store(&url, "scoped=2; Path=/k");
        jar.store(&url, "tls=3; Secure");
        jar.store(&url, "=nameless");
        jar.store(&url, "garbage");
        assert_eq!(jar.header_for(&url).unwrap(), "sid=1; scoped=2");
        let other_path = Url::parse("http://ebk.example/e").unwrap();
        assert_eq!(jar.header_for(&other_path).unwrap(), "sid=1");
        let sibling = Url::parse("http://ebk.example/kx").unwrap();
        assert_eq!(jar.header_for(&sibling).unwrap(), "sid=1");
        let https = Url::parse("https://ebk.example/k/ABC").unwrap();
        assert_eq!(jar.header_for(&https).unwrap(), "sid=1; scoped=2; tls=3");
        assert_eq!(jar.header_for(&Url::parse("http://evil.example/").unwrap()), None);
        // Replacement and deletion.
        jar.store(&url, "sid=9; Path=/");
        assert!(jar.header_for(&other_path).unwrap().contains("sid=9"));
        jar.store(&url, "sid=; Path=/; Max-Age=0");
        assert_eq!(jar.header_for(&other_path), None);
        // Bounded.
        for i in 0..200 {
            jar.store(&url, &format!("c{i}=v"));
        }
        assert!(jar.pairs().len() <= CookieJar::MAX_COOKIES);
        jar.store(&url, &format!("big={}", "x".repeat(10_000)));
        assert!(!jar.pairs().iter().any(|(n, _)| n == "big"));
    }

    #[test]
    fn file_names_come_from_plain_filename_then_the_url() {
        let h = head(
            "http://ebk.example/d/s/Le%20Horla.epub",
            &[("content-disposition", "attachment; filename=\"le-horla.epub\"; filename*=UTF-8''Le%20Horla%20%C3%A9.epub")],
        );
        assert_eq!(response_file_name(&h), "le-horla.epub");
        let h = head("http://ebk.example/d/s/Le%20Horla.epub", &[("content-disposition", "attachment")]);
        assert_eq!(response_file_name(&h), "Le Horla.epub");
        let h = head(
            "http://ebk.example/d/s/x",
            &[("content-disposition", "attachment; filename=\"../../etc/passwd\"")],
        );
        assert_eq!(response_file_name(&h), "passwd");
        let h = head("http://ebk.example/", &[]);
        assert_eq!(response_file_name(&h), "download");
        let h = head("http://ebk.example/a%ZZb%", &[]);
        assert_eq!(response_file_name(&h), "a%ZZb%");
        let h = head("http://ebk.example/x", &[("content-disposition", "attachment; filename=")]);
        assert_eq!(response_file_name(&h), "x");
    }
}
