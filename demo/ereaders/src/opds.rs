//! `OpdsClient`: what a reading app does with an OPDS catalogue.
//!
//! Supports OPDS 1.2 (Atom XML) and OPDS 2.0 (JSON) acquisition feeds: fetch a feed, list its
//! publications and their acquisition links by media type (including the
//! `indirectAcquisition` chain), and download a publication into the reader's [`Library`].
//!
//! Evidence (`docs/research/notes/ereader-ux.ecosystems.md` §4.1): media types and acquisition
//! relations \[V1\]; `indirectAcquisition` \[V2\]; no stock Kindle or Kobo firmware speaks OPDS
//! \[M\] — so this client stands for an *app* (Thorium, KOReader…), which is why
//! [`DeviceProfile::native_opds`] is false for the e-ink profiles. Authentication for OPDS,
//! search, facets, pagination beyond exposing `next`, and the `opds://` callback beyond a
//! simple scheme mapping are **not** implemented. HTTP Basic is supported because it is the
//! only scheme KOReader offers (`ereader-ux.devices.md` §4 \[V1\]).

use reqwest::Method;
use url::Url;

use crate::browser::{LicenceFile, Outcome};
use crate::error::{Error, NetError, OpdsError, Result};
use crate::format::FileFormat;
use crate::library::{ImportSource, IncomingFile, Library};
use crate::net::{self, Exchange, Http, NetConfig, Request};
use crate::profile::{DeviceProfile, DrmScheme};

/// Relation prefix of every OPDS acquisition link.
pub const REL_ACQUISITION: &str = "http://opds-spec.org/acquisition";
/// Media type of an OPDS 1.2 acquisition feed.
pub const TYPE_OPDS1_ACQUISITION: &str = "application/atom+xml;profile=opds-catalog;kind=acquisition";
/// Media type of an OPDS 2.0 feed.
pub const TYPE_OPDS2: &str = "application/opds+json";

/// Limits of the OPDS client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpdsConfig {
    /// Network limits.
    pub net: NetConfig,
    /// Largest feed document accepted.
    pub max_feed_bytes: u64,
    /// Most publications (and navigation entries) kept from one feed.
    pub max_entries: usize,
}

impl Default for OpdsConfig {
    fn default() -> Self {
        Self { net: NetConfig::default(), max_feed_bytes: 2 * 1024 * 1024, max_entries: 2000 }
    }
}

/// Which OPDS dialect a feed is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedKind {
    /// OPDS 1.x, Atom XML.
    Opds1,
    /// OPDS 2.0, JSON.
    Opds2,
}

/// A link that yields (or leads to) the publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acquisition {
    /// Absolute URL.
    pub url: Url,
    /// Relation, e.g. `http://opds-spec.org/acquisition/open-access`.
    pub rel: String,
    /// Media type of what the link returns directly.
    pub media_type: String,
    /// Media types of the `indirectAcquisition` chain, outermost first: what the reader gets
    /// after going through `media_type` (e.g. an LCP licence leading to an EPUB).
    pub indirect: Vec<String>,
}

impl Acquisition {
    /// The media type the reader finally ends up with.
    pub fn final_media_type(&self) -> &str {
        self.indirect.last().map_or(self.media_type.as_str(), String::as_str)
    }

    /// The relation's last path segment: `acquisition`, `open-access`, `buy`, `borrow`,
    /// `sample`, `subscribe`.
    pub fn kind(&self) -> &str {
        self.rel.rsplit('/').next().unwrap_or("")
    }
}

/// A publication listed in a feed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Publication {
    /// Identifier (`atom:id` / `metadata.identifier`).
    pub id: Option<String>,
    /// Title.
    pub title: String,
    /// Authors.
    pub authors: Vec<String>,
    /// Language.
    pub language: Option<String>,
    /// Summary / description.
    pub summary: Option<String>,
    /// Acquisition links.
    pub acquisitions: Vec<Acquisition>,
}

impl Publication {
    /// The first acquisition link whose *direct* media type matches (parameters ignored).
    pub fn acquisition(&self, media_type: &str) -> Option<&Acquisition> {
        self.acquisitions.iter().find(|a| same_type(&a.media_type, media_type))
    }

    /// The acquisition a given reading system would pick: an open or generic link whose final
    /// format it reads, directly if possible, else through a licence it can unlock. Purchase,
    /// loan and subscription links are never picked (their flows are not simulated).
    pub fn best_for(&self, profile: &DeviceProfile) -> Option<&Acquisition> {
        let usable = |a: &&Acquisition| matches!(a.kind(), "acquisition" | "open-access" | "sample");
        let reads = |media_type: &str| {
            FileFormat::from_media_type(media_type).is_some_and(|f| !f.is_licence() && profile.reads(f))
        };
        let rank = |a: &Acquisition| u8::from(a.kind() == "sample");
        let mut direct: Vec<&Acquisition> =
            self.acquisitions.iter().filter(usable).filter(|a| a.indirect.is_empty() && reads(&a.media_type)).collect();
        direct.sort_by_key(|a| rank(a));
        if let Some(found) = direct.first() {
            return Some(found);
        }
        self.acquisitions.iter().filter(usable).find(|a| {
            FileFormat::from_media_type(&a.media_type) == Some(FileFormat::Lcpl)
                && profile.unlocks(DrmScheme::ReadiumLcp)
                && (a.indirect.is_empty() || reads(a.final_media_type()))
        })
    }
}

/// A link to another feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Navigation {
    /// Title.
    pub title: String,
    /// Absolute URL.
    pub url: Url,
}

/// A parsed OPDS feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feed {
    /// Dialect.
    pub kind: FeedKind,
    /// URL the feed was read from.
    pub url: Url,
    /// Feed title.
    pub title: String,
    /// Publications, in feed order.
    pub publications: Vec<Publication>,
    /// Links to other feeds.
    pub navigation: Vec<Navigation>,
    /// `rel="next"` page, if any.
    pub next: Option<Url>,
}

impl Feed {
    /// The first publication with exactly this title.
    pub fn publication(&self, title: &str) -> Option<&Publication> {
        self.publications.iter().find(|p| p.title == title)
    }
}

fn same_type(a: &str, b: &str) -> bool {
    let essence = |t: &str| t.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    essence(a) == essence(b)
}

fn clean(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(2000).collect()
}

/// Parses a feed document. `content_type` is a hint; the content decides.
///
/// Pure and bounded: never panics, keeps at most `max_entries` entries.
pub fn parse_feed(
    bytes: &[u8],
    content_type: Option<&str>,
    base: &Url,
    max_entries: usize,
) -> Result<Feed, OpdsError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| OpdsError::Malformed("the document is not UTF-8".to_owned()))?
        .trim_start_matches('\u{feff}');
    let first = text.trim_start().chars().next();
    let json_hint = content_type.is_some_and(|t| t.to_ascii_lowercase().contains("json"));
    match first {
        Some('{') => parse_opds2(text, base, max_entries),
        Some('<') => parse_opds1(text, base, max_entries),
        Some(_) if json_hint => parse_opds2(text, base, max_entries),
        Some(_) => Err(OpdsError::NotOpds("neither XML nor JSON".to_owned())),
        None => Err(OpdsError::NotOpds("empty document".to_owned())),
    }
}

fn resolve(base: &Url, href: &str) -> Option<Url> {
    let href = href.trim();
    if href.is_empty() || href.len() > 8192 {
        return None;
    }
    let url = base.join(href).ok()?;
    matches!(url.scheme(), "http" | "https").then_some(url)
}

fn parse_opds1(text: &str, base: &Url, max_entries: usize) -> Result<Feed, OpdsError> {
    let options =
        roxmltree::ParsingOptions { allow_dtd: false, nodes_limit: 400_000, ..Default::default() };
    let doc = roxmltree::Document::parse_with_options(text, options)
        .map_err(|e| OpdsError::Malformed(e.to_string()))?;
    let root = doc.root_element();
    let base = root
        .attribute(("http://www.w3.org/XML/1998/namespace", "base"))
        .and_then(|b| resolve(base, b))
        .unwrap_or_else(|| base.clone());
    let child_text = |node: roxmltree::Node<'_, '_>, name: &str| {
        node.children()
            .find(|c| c.is_element() && c.tag_name().name() == name)
            .map(|c| clean(&c.descendants().filter_map(|d| d.text()).collect::<String>()))
            .filter(|t| !t.is_empty())
    };
    let mut feed = Feed {
        kind: FeedKind::Opds1,
        url: base.clone(),
        title: String::new(),
        publications: Vec::new(),
        navigation: Vec::new(),
        next: None,
    };
    let entries: Vec<roxmltree::Node<'_, '_>> = match root.tag_name().name() {
        "feed" => {
            feed.title = child_text(root, "title").unwrap_or_default();
            for link in root.children().filter(|c| c.is_element() && c.tag_name().name() == "link") {
                if link.attribute("rel") == Some("next") {
                    feed.next = link.attribute("href").and_then(|h| resolve(&base, h));
                }
            }
            root.children().filter(|c| c.is_element() && c.tag_name().name() == "entry").collect()
        }
        // An OPDS "entry document": a single publication.
        "entry" => vec![root],
        other => {
            return Err(OpdsError::NotOpds(format!("root element is <{other}>, not an Atom feed")));
        }
    };
    for entry in entries.into_iter().take(max_entries) {
        let title = child_text(entry, "title").unwrap_or_default();
        let mut publication = Publication {
            id: child_text(entry, "id"),
            title: title.clone(),
            authors: entry
                .children()
                .filter(|c| c.is_element() && c.tag_name().name() == "author")
                .filter_map(|a| child_text(a, "name"))
                .take(32)
                .collect(),
            language: child_text(entry, "language"),
            summary: child_text(entry, "summary").or_else(|| child_text(entry, "content")),
            acquisitions: Vec::new(),
        };
        let mut nav: Option<Url> = None;
        for link in entry.children().filter(|c| c.is_element() && c.tag_name().name() == "link").take(64) {
            let rel = link.attribute("rel").unwrap_or("");
            let media_type = link.attribute("type").unwrap_or("");
            let Some(url) = link.attribute("href").and_then(|h| resolve(&base, h)) else { continue };
            if rel.starts_with(REL_ACQUISITION) {
                let mut indirect = Vec::new();
                let mut cursor = link;
                // Follow the first branch of nested <opds:indirectAcquisition> elements.
                while let Some(inner) = cursor
                    .children()
                    .find(|c| c.is_element() && c.tag_name().name() == "indirectAcquisition")
                {
                    if let Some(t) = inner.attribute("type") {
                        indirect.push(t.to_owned());
                    }
                    cursor = inner;
                    if indirect.len() >= 8 {
                        break;
                    }
                }
                publication.acquisitions.push(Acquisition {
                    url,
                    rel: rel.to_owned(),
                    media_type: media_type.to_owned(),
                    indirect,
                });
            } else if media_type.to_ascii_lowercase().contains("profile=opds-catalog") && nav.is_none() {
                nav = Some(url);
            }
        }
        if !publication.acquisitions.is_empty() {
            feed.publications.push(publication);
        } else if let Some(url) = nav {
            feed.navigation.push(Navigation { title, url });
        }
    }
    Ok(feed)
}

fn json_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(clean(s)).filter(|s| !s.is_empty()),
        // Localised strings: {"fr": "…", "en": "…"} — take French, then English, then any.
        serde_json::Value::Object(map) => map
            .get("fr")
            .or_else(|| map.get("en"))
            .or_else(|| map.get("name"))
            .or_else(|| map.values().next())
            .and_then(json_string),
        _ => None,
    }
}

fn json_names(value: Option<&serde_json::Value>) -> Vec<String> {
    match value {
        Some(serde_json::Value::Array(items)) => items.iter().take(32).filter_map(json_name).collect(),
        Some(other) => json_name(other).into_iter().collect(),
        None => Vec::new(),
    }
}

fn json_name(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => map.get("name").and_then(json_string),
        other => json_string(other),
    }
}

fn json_rels(link: &serde_json::Value) -> Vec<String> {
    match link.get("rel") {
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        Some(serde_json::Value::Array(items)) => {
            items.iter().take(16).filter_map(|v| v.as_str().map(str::to_owned)).collect()
        }
        _ => Vec::new(),
    }
}

fn json_indirect(link: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = link.get("properties").and_then(|p| p.get("indirectAcquisition"));
    while let Some(first) = cursor.and_then(|c| c.as_array()).and_then(|a| a.first()) {
        if let Some(t) = first.get("type").and_then(|t| t.as_str()) {
            out.push(t.to_owned());
        }
        cursor = first.get("child");
        if out.len() >= 8 {
            break;
        }
    }
    out
}

fn parse_opds2(text: &str, base: &Url, max_entries: usize) -> Result<Feed, OpdsError> {
    let root: serde_json::Value =
        serde_json::from_str(text).map_err(|e| OpdsError::Malformed(e.to_string()))?;
    let object = root.as_object().ok_or_else(|| OpdsError::NotOpds("not a JSON object".to_owned()))?;
    let has_collections = ["publications", "navigation", "groups"].iter().any(|k| object.contains_key(*k));
    if !object.contains_key("metadata") && !has_collections {
        return Err(OpdsError::NotOpds("no metadata, publications, navigation or groups".to_owned()));
    }
    let mut feed = Feed {
        kind: FeedKind::Opds2,
        url: base.clone(),
        title: root.get("metadata").and_then(|m| m.get("title")).and_then(json_string).unwrap_or_default(),
        publications: Vec::new(),
        navigation: Vec::new(),
        next: None,
    };
    let links = |value: &serde_json::Value| -> Vec<serde_json::Value> {
        value.get("links").and_then(|l| l.as_array()).cloned().unwrap_or_default()
    };
    for link in links(&root).iter().take(64) {
        if json_rels(link).iter().any(|r| r == "next") {
            feed.next = link.get("href").and_then(|h| h.as_str()).and_then(|h| resolve(base, h));
        }
    }
    // A publication document on its own (application/opds-publication+json).
    let lone_publication =
        !has_collections && root.get("links").is_some() && root.get("metadata").is_some();
    let mut collections: Vec<&serde_json::Value> = vec![&root];
    if let Some(groups) = root.get("groups").and_then(|g| g.as_array()) {
        collections.extend(groups.iter().take(256));
    }
    for collection in collections {
        if let Some(items) = collection.get("navigation").and_then(|n| n.as_array()) {
            for item in items {
                if feed.navigation.len() >= max_entries {
                    break;
                }
                let url = item.get("href").and_then(|h| h.as_str()).and_then(|h| resolve(base, h));
                if let Some(url) = url {
                    feed.navigation.push(Navigation {
                        title: item.get("title").and_then(json_string).unwrap_or_default(),
                        url,
                    });
                }
            }
        }
        let publications: Vec<&serde_json::Value> = if lone_publication {
            vec![&root]
        } else {
            collection
                .get("publications")
                .and_then(|p| p.as_array())
                .map(|a| a.iter().collect())
                .unwrap_or_default()
        };
        for item in publications {
            if feed.publications.len() >= max_entries {
                break;
            }
            let metadata = item.get("metadata");
            let get = |key: &str| metadata.and_then(|m| m.get(key));
            let mut publication = Publication {
                id: get("identifier").and_then(json_string),
                title: get("title").and_then(json_string).unwrap_or_default(),
                authors: json_names(get("author")),
                language: match get("language") {
                    Some(serde_json::Value::Array(items)) => items.first().and_then(json_string),
                    Some(other) => json_string(other),
                    None => None,
                },
                summary: get("description").and_then(json_string),
                acquisitions: Vec::new(),
            };
            for link in links(item).iter().take(64) {
                if link.get("templated").and_then(serde_json::Value::as_bool) == Some(true) {
                    continue;
                }
                let Some(rel) = json_rels(link).into_iter().find(|r| r.starts_with(REL_ACQUISITION))
                else {
                    continue;
                };
                let Some(url) = link.get("href").and_then(|h| h.as_str()).and_then(|h| resolve(base, h))
                else {
                    continue;
                };
                publication.acquisitions.push(Acquisition {
                    url,
                    rel,
                    media_type: link.get("type").and_then(|t| t.as_str()).unwrap_or("").to_owned(),
                    indirect: json_indirect(link),
                });
            }
            if !publication.acquisitions.is_empty() {
                feed.publications.push(publication);
            }
        }
        if lone_publication {
            break;
        }
    }
    Ok(feed)
}

/// Maps the `opds://` callback scheme to a fetchable URL: `https://`, or `http://` for
/// `localhost` and IP-literal hosts so local demos work.
///
/// *Assumption*: the buying best-practice draft (ecosystems §4.1 \[V1\]) defines the scheme but
/// not the transport; reading apps are assumed to use HTTPS.
pub fn normalise_opds_url(input: &str) -> Result<Url, NetError> {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("opds://").map(|_| &trimmed["opds://".len()..]) {
        let secure = net::parse_url(&format!("https://{rest}"))?;
        let local = match secure.host() {
            Some(url::Host::Domain(d)) => d == "localhost",
            Some(url::Host::Ipv4(_) | url::Host::Ipv6(_)) => true,
            None => false,
        };
        return if local { net::parse_url(&format!("http://{rest}")) } else { Ok(secure) };
    }
    net::parse_url(trimmed)
}

/// An OPDS-capable reading app.
#[derive(Debug)]
pub struct OpdsClient {
    profile: DeviceProfile,
    http: Http,
    config: OpdsConfig,
    library: Library,
    exchanges: Vec<Exchange>,
    basic_auth: Option<(String, String, Url)>,
}

impl OpdsClient {
    /// A client reading with `profile`'s reading system.
    pub fn new(profile: DeviceProfile) -> Result<Self> {
        Self::with_config(profile, OpdsConfig::default())
    }

    /// A client with explicit limits.
    pub fn with_config(profile: DeviceProfile, config: OpdsConfig) -> Result<Self> {
        let http = Http::new(profile.user_agent, config.net.clone())?;
        Ok(Self { profile, http, config, library: Library::new(), exchanges: Vec::new(), basic_auth: None })
    }

    /// Uses HTTP Basic credentials — sent only to the origin of `catalogue_url`.
    pub fn with_basic_auth(mut self, catalogue_url: &str, user: &str, password: &str) -> Result<Self> {
        let origin = normalise_opds_url(catalogue_url)?;
        self.basic_auth = Some((user.to_owned(), password.to_owned(), origin));
        Ok(self)
    }

    /// The reader's library.
    pub fn library(&self) -> &Library {
        &self.library
    }

    /// Every HTTP exchange made so far.
    pub fn exchanges(&self) -> &[Exchange] {
        &self.exchanges
    }

    fn request(&self, url: Url, accept: &str) -> Request {
        let mut request = Request { method: Method::GET, url, headers: Vec::new(), body: None }
            .header("accept", accept);
        if let Some((user, password, origin)) = &self.basic_auth {
            if net::same_origin(origin, &request.url) {
                request = request.header("authorization", basic(user, password));
            }
        }
        request
    }

    /// Fetches and parses a feed. Accepts `http(s)://` and `opds://` URLs.
    pub async fn fetch(&mut self, url: &str) -> Result<Feed> {
        let url = normalise_opds_url(url)?;
        let request = self.request(
            url,
            "application/opds+json, application/atom+xml;profile=opds-catalog, application/atom+xml;q=0.8",
        );
        let (head, body) =
            self.http.fetch_bytes(request, self.config.max_feed_bytes, &mut self.exchanges).await?;
        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }
        Ok(parse_feed(&body, head.header("content-type"), &head.url, self.config.max_entries)?)
    }

    /// Downloads an acquisition link. A book goes into the library
    /// ([`Outcome::Download`]); a licence document is returned for the reading system
    /// ([`Outcome::Licence`]). Never returns [`Outcome::Page`]: an HTML answer (a login or
    /// payment page) is an error, since those flows are not simulated.
    pub async fn download(&mut self, acquisition: &Acquisition) -> Result<Outcome> {
        let request = self.request(acquisition.url.clone(), "*/*");
        let (head, response) = self.http.fetch(request, None, &mut self.exchanges).await?;
        if !head.is_success() {
            return Err(NetError::Status { url: head.url.to_string(), status: head.status }.into());
        }
        let bytes = self.http.read(&head, response, self.profile.max_download_bytes).await?;
        let file_name = net::response_file_name(&head);
        let declared = head
            .header("content-type")
            .and_then(FileFormat::from_media_type)
            .or_else(|| FileFormat::from_media_type(&acquisition.media_type))
            .or_else(|| FileFormat::from_file_name(&file_name));
        if let Some(format) = declared.filter(|f| f.is_licence()) {
            return Ok(Outcome::Licence(LicenceFile { format, file_name, url: head.url, bytes }));
        }
        let entry = self.library.import(
            &self.profile,
            IncomingFile { bytes: &bytes, file_name: &file_name, declared },
            ImportSource::Opds { url: head.url.to_string() },
        )?;
        Ok(Outcome::Download(entry))
    }

    /// Picks the acquisition this reading system would use ([`Publication::best_for`]) and
    /// downloads it.
    pub async fn acquire(&mut self, publication: &Publication) -> Result<Outcome> {
        let acquisition = publication.best_for(&self.profile).cloned().ok_or_else(|| {
            Error::from(OpdsError::NoAcquisition {
                wanted: self.profile.reading_formats.iter().map(|f| f.media_type().to_owned()).collect(),
            })
        })?;
        self.download(&acquisition).await
    }
}

/// `Authorization: Basic …` value.
fn basic(user: &str, password: &str) -> String {
    format!("Basic {}", base64_encode(format!("{user}:{password}").as_bytes()))
}

/// Standard base64 with padding (kept local: the crate needs nothing else from a base64 library).
pub(crate) fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [chunk[0], chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(char::from(ALPHABET[(n >> 18) as usize & 63]));
        out.push(char::from(ALPHABET[(n >> 12) as usize & 63]));
        out.push(if chunk.len() > 1 { char::from(ALPHABET[(n >> 6) as usize & 63]) } else { '=' });
        out.push(if chunk.len() > 2 { char::from(ALPHABET[n as usize & 63]) } else { '=' });
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("http://ebk.example/o/s3ss10n/catalog.xml").unwrap()
    }

    const ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/terms/" xmlns:opds="http://opds-spec.org/2010/catalog">
  <id>urn:uuid:1</id><title>Votre  cadeau</title><updated>2026-11-12T00:00:00Z</updated>
  <link rel="self" href="catalog.xml" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  <link rel="next" href="catalog.xml?page=2" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  <entry>
    <title>Le Horla</title><id>urn:isbn:9782070360024</id>
    <author><name>Guy de Maupassant</name></author><dc:language>fr</dc:language>
    <summary>Une nouvelle fantastique.</summary>
    <link rel="http://opds-spec.org/image/thumbnail" href="/cover.jpg" type="image/jpeg"/>
    <link rel="http://opds-spec.org/acquisition" href="../../d/s3ss10n/le-horla.epub" type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition/sample" href="/sample.epub" type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition" href="/licence.lcpl" type="application/vnd.readium.lcp.license.v1.0+json">
      <opds:indirectAcquisition type="application/epub+zip"/>
    </link>
    <link rel="http://opds-spec.org/acquisition/buy" href="/buy" type="text/html">
      <opds:indirectAcquisition type="application/vnd.adobe.adept+xml"><opds:indirectAcquisition type="application/epub+zip"/></opds:indirectAcquisition>
    </link>
  </entry>
  <entry><title>Nouveautés</title><id>urn:nav</id>
    <link href="/new.xml" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/></entry>
  <entry><title>Sans lien</title><id>urn:none</id></entry>
</feed>"#;

    const JSON: &str = r#"{
  "metadata": {"title": "Votre cadeau"},
  "links": [{"rel": "self", "href": "catalog.json", "type": "application/opds+json"},
            {"rel": ["next"], "href": "catalog.json?page=2", "type": "application/opds+json"}],
  "navigation": [{"href": "/new.json", "title": "Nouveautés", "type": "application/opds+json"}],
  "publications": [{
    "metadata": {"@type": "http://schema.org/Book", "title": {"fr": "Le Horla", "en": "The Horla"},
                 "author": [{"name": "Guy de Maupassant"}, "Anonyme"], "identifier": "urn:isbn:9782070360024",
                 "language": ["fr"], "description": "Une nouvelle fantastique."},
    "links": [
      {"rel": "http://opds-spec.org/acquisition/open-access", "href": "/d/s/le-horla.epub", "type": "application/epub+zip"},
      {"rel": ["http://opds-spec.org/acquisition"], "href": "/licence.lcpl",
       "type": "application/vnd.readium.lcp.license.v1.0+json",
       "properties": {"indirectAcquisition": [{"type": "application/epub+zip"}]}},
      {"rel": "http://opds-spec.org/acquisition", "href": "/t{?x}", "type": "application/epub+zip", "templated": true},
      {"rel": "self", "href": "/pub.json", "type": "application/opds-publication+json"}
    ],
    "images": [{"href": "/cover.jpg", "type": "image/jpeg"}]
  }],
  "groups": [{"metadata": {"title": "Autres"}, "publications": [
     {"metadata": {"title": "Sans acquisition"}, "links": []},
     {"metadata": {"title": "Boule de Suif", "author": "Guy de Maupassant"},
      "links": [{"rel": "http://opds-spec.org/acquisition", "href": "bds.pdf", "type": "application/pdf"}]}]}]
}"#;

    #[test]
    fn opds1_feed_lists_publications_and_acquisitions() {
        let feed = parse_feed(ATOM.as_bytes(), Some("application/atom+xml"), &base(), 100).unwrap();
        assert_eq!(feed.kind, FeedKind::Opds1);
        assert_eq!(feed.title, "Votre cadeau");
        assert_eq!(feed.next.as_ref().unwrap().as_str(), "http://ebk.example/o/s3ss10n/catalog.xml?page=2");
        assert_eq!(feed.publications.len(), 1);
        let book = feed.publication("Le Horla").unwrap();
        assert_eq!(book.id.as_deref(), Some("urn:isbn:9782070360024"));
        assert_eq!(book.authors, vec!["Guy de Maupassant".to_owned()]);
        assert_eq!(book.language.as_deref(), Some("fr"));
        assert_eq!(book.summary.as_deref(), Some("Une nouvelle fantastique."));
        assert_eq!(book.acquisitions.len(), 4);
        let epub = book.acquisition("application/epub+zip").unwrap();
        assert_eq!(epub.url.as_str(), "http://ebk.example/d/s3ss10n/le-horla.epub");
        assert_eq!(epub.kind(), "acquisition");
        let lcp = book.acquisition("application/vnd.readium.lcp.license.v1.0+json").unwrap();
        assert_eq!(lcp.indirect, vec!["application/epub+zip".to_owned()]);
        assert_eq!(lcp.final_media_type(), "application/epub+zip");
        let buy = book.acquisition("text/html").unwrap();
        assert_eq!(buy.kind(), "buy");
        assert_eq!(buy.indirect.len(), 2);
        assert_eq!(feed.navigation, vec![Navigation {
            title: "Nouveautés".to_owned(),
            url: Url::parse("http://ebk.example/new.xml").unwrap()
        }]);
    }

    #[test]
    fn opds2_feed_lists_publications_groups_and_navigation() {
        let feed = parse_feed(JSON.as_bytes(), Some(TYPE_OPDS2), &base(), 100).unwrap();
        assert_eq!(feed.kind, FeedKind::Opds2);
        assert_eq!(feed.title, "Votre cadeau");
        assert!(feed.next.is_some());
        assert_eq!(feed.navigation.len(), 1);
        assert_eq!(feed.publications.len(), 2);
        let book = &feed.publications[0];
        assert_eq!(book.title, "Le Horla");
        assert_eq!(book.authors, vec!["Guy de Maupassant".to_owned(), "Anonyme".to_owned()]);
        assert_eq!(book.language.as_deref(), Some("fr"));
        assert_eq!(book.acquisitions.len(), 2, "templated and non-acquisition links are skipped");
        assert_eq!(book.acquisitions[0].kind(), "open-access");
        assert_eq!(book.acquisitions[1].indirect, vec!["application/epub+zip".to_owned()]);
        let pdf = feed.publication("Boule de Suif").unwrap();
        assert_eq!(pdf.authors, vec!["Guy de Maupassant".to_owned()]);
        assert_eq!(pdf.acquisitions[0].url.as_str(), "http://ebk.example/o/s3ss10n/bds.pdf");
    }

    #[test]
    fn each_reading_system_picks_what_it_can_open() {
        let feed = parse_feed(ATOM.as_bytes(), None, &base(), 100).unwrap();
        let book = &feed.publications[0];
        let kobo = book.best_for(&DeviceProfile::kobo_clara()).unwrap();
        assert_eq!(kobo.media_type, "application/epub+zip");
        assert_eq!(kobo.kind(), "acquisition", "the full book is preferred to the sample");
        assert!(book.best_for(&DeviceProfile::kindle_paperwhite()).is_none());

        let lcp_only = Publication {
            title: "Protégé".to_owned(),
            acquisitions: book.acquisitions.iter().filter(|a| a.media_type.contains("lcp") || a.kind() == "buy").cloned().collect(),
            ..Publication::default()
        };
        assert!(lcp_only.best_for(&DeviceProfile::kobo_clara()).is_none());
        let picked = lcp_only.best_for(&DeviceProfile::thorium_desktop()).unwrap();
        assert!(picked.media_type.contains("lcp"));
    }

    #[test]
    fn entry_documents_and_lone_publications_are_feeds_of_one() {
        let entry = r#"<entry xmlns="http://www.w3.org/2005/Atom"><title>Seul</title>
            <link rel="http://opds-spec.org/acquisition/open-access" href="seul.epub" type="application/epub+zip"/></entry>"#;
        let feed = parse_feed(entry.as_bytes(), None, &base(), 10).unwrap();
        assert_eq!(feed.publications.len(), 1);
        let publication = r#"{"metadata": {"title": "Seul"}, "links": [
            {"rel": "http://opds-spec.org/acquisition", "href": "seul.epub", "type": "application/epub+zip"}]}"#;
        let feed = parse_feed(publication.as_bytes(), None, &base(), 10).unwrap();
        assert_eq!(feed.publications.len(), 1);
        assert_eq!(feed.publications[0].title, "Seul");
    }

    #[test]
    fn hostile_feeds_are_rejected_without_panicking() {
        let b = base();
        let malformed: Vec<(&[u8], &str)> = vec![
            (b"", "empty"),
            (b"   ", "blank"),
            (b"hello", "text"),
            (b"\xff\xfe\x00", "not utf-8"),
            (b"<feed", "truncated xml"),
            (b"<html><body>404</body></html>", "html"),
            (b"{", "truncated json"),
            (b"[]", "json array"),
            (b"{\"a\": 1}", "json without opds members"),
            (b"null", "json null"),
            (b"<!DOCTYPE feed [<!ENTITY a \"aaaa\">]><feed xmlns=\"http://www.w3.org/2005/Atom\">&a;</feed>", "dtd"),
        ];
        for (bytes, what) in malformed {
            assert!(parse_feed(bytes, None, &b, 10).is_err(), "{what} should be rejected");
        }
        // Wrong shapes inside a valid document are skipped, not fatal.
        let odd = r#"{"metadata": 7, "publications": [1, "x", null, {"metadata": [], "links": "no"},
            {"metadata": {"title": 5, "author": {"name": {"zz": "Zed"}}}, "links": [
              {"rel": 9, "href": "a.epub"}, {"rel": "http://opds-spec.org/acquisition", "href": 3},
              {"rel": "http://opds-spec.org/acquisition", "href": "javascript:alert(1)"},
              {"rel": "http://opds-spec.org/acquisition", "href": "ok.epub", "type": 12,
               "properties": {"indirectAcquisition": "nope"}}]}],
            "navigation": "nope", "groups": {"not": "an array"}, "links": [7]}"#;
        let feed = parse_feed(odd.as_bytes(), None, &b, 10).unwrap();
        assert_eq!(feed.publications.len(), 1);
        assert_eq!(feed.publications[0].title, "");
        assert_eq!(feed.publications[0].authors, vec!["Zed".to_owned()]);
        assert_eq!(feed.publications[0].acquisitions.len(), 1);
        // Entry caps hold.
        let many: String = (0..500)
            .map(|i| format!("<entry><title>t{i}</title><link rel=\"http://opds-spec.org/acquisition\" href=\"{i}.epub\" type=\"application/epub+zip\"/></entry>"))
            .collect();
        let feed = parse_feed(
            format!("<feed xmlns=\"http://www.w3.org/2005/Atom\">{many}</feed>").as_bytes(),
            None,
            &b,
            50,
        )
        .unwrap();
        assert_eq!(feed.publications.len(), 50);
        // Deeply nested JSON and XML are errors, not stack overflows.
        let deep_json = "[".repeat(100_000);
        assert!(parse_feed(deep_json.as_bytes(), Some("application/json"), &b, 10).is_err());
        let deep_xml = "<a>".repeat(100_000);
        assert!(parse_feed(deep_xml.as_bytes(), None, &b, 10).is_err());
    }

    #[test]
    fn opds_scheme_maps_to_https_except_for_local_hosts() {
        assert_eq!(normalise_opds_url("opds://store.example/shelf").unwrap().as_str(), "https://store.example/shelf");
        assert_eq!(normalise_opds_url("OPDS://127.0.0.1:8080/o/x/catalog.xml").unwrap().as_str(), "http://127.0.0.1:8080/o/x/catalog.xml");
        assert_eq!(normalise_opds_url("opds://localhost/c").unwrap().scheme(), "http");
        assert_eq!(normalise_opds_url("http://store.example/c").unwrap().scheme(), "http");
        assert!(normalise_opds_url("opds://").is_err());
        assert!(normalise_opds_url("ftp://store.example/c").is_err());
    }

    #[test]
    fn base64_matches_rfc4648_vectors() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(input.as_bytes()), expected);
        }
        assert_eq!(basic("Aladdin", "open sesame"), "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
    }
}
