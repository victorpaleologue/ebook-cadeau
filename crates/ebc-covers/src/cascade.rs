//! The acquisition cascade: **where a cover might be found and previewed**, in which
//! order, at what risk, under which terms.
//!
//! [`candidates`] turns a [`BookMeta`] and a [`Policy`] into an ordered list of
//! [`CoverCandidate`]s — requests for the host to run and links for the user to follow.
//! It encodes steps 7–10 of the cascade of `docs/research/02-cover-sources.md` (§1.2,
//! "Implications" 5–16) as corrected by `docs/research/factcheck/cover-sources.md`:
//!
//! | # | Source id | What is produced | Locate / preview | Store | Print | Default |
//! |---|---|---|---|---|---|---|
//! | 0 | (record) | covers already referenced by the book record, routed to their source | per source | | | on |
//! | 1 | `openlibrary` | JSON facts, `-M` preview, link to the original | Medium | High | High | on |
//! | 2 | `googlebooks` | volume lookup, cleaned thumbnail, larger-zoom link | Medium | High | High | only with an API key |
//! | 3 | `bnf` (978-2, 979-10) | SRU lookup or `HEAD` probe, link | Medium | High | High | **off** until its image terms are read |
//! | 4 | `dnb` (978-3) | `HEAD` probe, link | Medium | High | High | **off** until its image terms are read |
//! | — | `publisher_page` | a page URL pasted by the user ([`page_candidate`]) | Medium | High | High | on |
//! | — | `reseller_feed` | reseller mode: the feed's own covers, nothing else | Low | Low | Medium | reseller mode |
//!
//! ("Low–Medium" in the research is stored as Medium; the fact-check rates Open
//! Library's preview Medium until counsel has answered.) Steps 1–6 of the research's
//! cascade — feed covers with a print clause, rights-holder uploads, the user's own
//! EPUB or file, public-domain artwork — are not lookups: their bytes are already in
//! the caller's hands and go straight into [`crate::PrintableCover`]. Step 12, the
//! typographic card, needs no cover at all and is the default output.
//!
//! **What a candidate is not.** A candidate never becomes a printable file through
//! this crate: its rights basis is always [`RightsBasis::None`]
//! ([`CoverCandidate::rights_basis`]), it carries no pixels, and the hosted service
//! only ever shows it — an inline preview loaded by the user's browser from an
//! allow-listed host ([`CandidateRole::Preview`]), or a plain link
//! ([`CandidateRole::Link`]). No image proxy, no server-side copy.
//!
//! **High risk exists only as an explicit opt-in.** The one High-risk act a candidate
//! can describe is downloading the bytes ([`CandidateRole::Download`]); it appears only
//! when the operator of a command-line or self-hosted tool set
//! [`Policy::operator_rights_basis`] *and* raised [`Policy::max_risk`] to High. Retailer
//! and aggregator hosts, Amazon, Goodreads-derived services and subscription image
//! services are not opt-in: the research rules them out in every mode, so there is no
//! URL builder for them, [`crate::guard`] refuses their hosts, and [`SOURCES`] lists
//! them only so that a tool can explain why.
//!
//! Everything about terms, quotas and live behaviour is **unverified**: the research
//! could not open any provider's terms. URL grammars come from source code (Open
//! Library's cover server, Inventaire's BnF probe, calibre-dnb, VuFind).

use core::fmt;

use ebc_isbn::Isbn13;
use ebc_model::{BookMeta, CoverKind, CoverRef, HttpRequest, HttpResponse, Method};
use url::Url;

use crate::CoversError;
use crate::guard::{self, Refusal};
use crate::rights::RightsBasis;
use crate::robots::Robots;
use crate::scrape::PageInfo;

/// Exposure of the service for one act (France first), as assessed by the research.
/// These are assessments, not sourced rulings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Risk {
    /// The act rests on a contract, an open licence, or the user's own file.
    Low,
    /// Third-party rights are not cleared, or the terms were not read; acceptable for
    /// locating and previewing only.
    Medium,
    /// Outside any known terms, or barred by them. Never done by the hosted service.
    High,
}

impl fmt::Display for Risk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        })
    }
}

/// What a source's output may be used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Use {
    /// Shown as an `<img>` loaded by the user's browser from the source, pixels never
    /// read by our code, at most 400 CSS px on the long edge.
    PreviewInline,
    /// Shown as a plain link with the source's name.
    LinkOnly,
    /// Fetched by the user's browser for a transient analysis that returns facts only
    /// (colours, hash); subject to the opt-out signals of [`crate::tdm`].
    AnalyseTransient,
    /// Bytes may be downloaded by an operator's command-line or self-hosted tool that
    /// was given an explicit rights basis.
    DownloadCli,
}

/// Whether a source is used at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceStatus {
    /// Part of the default cascade.
    Enabled,
    /// Used only when a key is configured ([`Policy::google_books_api_key`]).
    RequiresKey,
    /// Implemented, but off until someone has read and recorded its image terms; the
    /// caller must add the id to [`Policy::allowed_sources`].
    DisabledUntilTermsRead,
    /// Paid B2B service: reachable only through the reseller's own contract and feed.
    /// No request is ever built here.
    ContractOnly,
    /// Ruled out in every mode by the research. No request is ever built here and no
    /// policy setting can enable it.
    Never,
}

/// Static description of a cover source: who it is, what its terms are believed to be,
/// and how risky each act is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceInfo {
    /// Stable identifier, e.g. `"openlibrary"`.
    pub id: &'static str,
    /// Display name, for attribution next to a preview.
    pub name: &'static str,
    /// Home page (for the link back).
    pub homepage: &'static str,
    /// Where the terms are published, when a URL is known.
    pub terms_url: Option<&'static str>,
    /// ISO date on which the terms were last read **at source**. `None` everywhere
    /// today: the research could not open any of them.
    pub terms_last_read: Option<&'static str>,
    /// An API key is needed.
    pub requires_key: bool,
    /// Cross-origin reads from a browser: `Some(true)` verified, `Some(false)` known
    /// not to work, `None` unverified.
    pub browser_cors: Option<bool>,
    /// Minimum interval between two requests to this source, in milliseconds.
    pub min_interval_ms: u32,
    /// Hosts requests and previews go to (the `img-src` allow-list entries).
    pub hosts: &'static [&'static str],
    /// What the output may be used for.
    pub uses: &'static [Use],
    /// Risk of locating and previewing.
    pub preview_risk: Risk,
    /// Risk of storing image bytes on our side.
    pub store_risk: Risk,
    /// Risk of printing the image.
    pub print_risk: Risk,
    /// Whether the source is used.
    pub status: SourceStatus,
    /// The terms that make the source acceptable or not, and what is unverified.
    pub note: &'static str,
}

impl SourceInfo {
    /// `true` when the source's output may be used for `wanted`.
    #[must_use]
    pub fn allows(&self, wanted: Use) -> bool {
        self.uses.contains(&wanted)
    }
}

/// Every source the research considered, usable or not.
pub const SOURCES: &[SourceInfo] = &[
    SourceInfo {
        id: "reseller_feed",
        name: "Reseller feed",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 0,
        hosts: &[],
        uses: &[Use::PreviewInline, Use::LinkOnly],
        preview_risk: Risk::Low,
        store_risk: Risk::Low,
        print_risk: Risk::Medium,
        status: SourceStatus::Enabled,
        note: "Cover supplied by the reseller from its ONIX or distributor feed. Display on the \
               reseller's pages is inside its licence. Printing needs the per-title flag: \
               print_licence (express print clause) or promo_all_media (written declaration, \
               reduced size); without one the basis is none and the card is typographic.",
    },
    SourceInfo {
        id: "openlibrary",
        name: "Open Library",
        homepage: "https://openlibrary.org",
        terms_url: Some("https://openlibrary.org/dev/docs/api/covers"),
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 3000,
        hosts: &["covers.openlibrary.org", "archive.org"],
        uses: &[Use::PreviewInline, Use::LinkOnly, Use::AnalyseTransient, Use::DownloadCli],
        preview_risk: Risk::Medium,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Enabled,
        note: "Keyless cover service of the Internet Archive. The images are user uploads and \
               imports with no licence and a takedown list: locate and preview by inline link \
               only, with the source name and a link back; never cached, never printed by the \
               hosted service. ISBN-keyed requests at most one every 3 s per client, id-keyed \
               one per second (documentation figure from memory; the server allows 400/min). \
               Covers are thumbnails by default; the unsuffixed URL is the original upload. \
               CORS `*` is in the server code; production and the archive.org redirect are \
               unverified. Terms not read at source.",
    },
    SourceInfo {
        id: "googlebooks",
        name: "Google Books",
        homepage: "https://books.google.com",
        terms_url: Some("https://developers.google.com/books/terms"),
        terms_last_read: None,
        requires_key: true,
        browser_cors: None,
        min_interval_ms: 1000,
        hosts: &["www.googleapis.com", "books.google.com"],
        uses: &[Use::PreviewInline, Use::LinkOnly],
        preview_risk: Risk::Medium,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::RequiresKey,
        note: "Used only with an API key, as its terms are believed to require: thumbnail with \
               \"Google Books\" attribution and a link to the volume, results kept for the \
               session only, never cached, never downloaded, never printed. Quota (about \
               1,000 requests a day) and the terms' wording are from memory, unverified. A \
               missing cover is a dummy image, not a 404: reject volumes without imageLinks.",
    },
    SourceInfo {
        id: "bnf",
        name: "Bibliothèque nationale de France",
        homepage: "https://catalogue.bnf.fr",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 2000,
        hosts: &["catalogue.bnf.fr"],
        uses: &[Use::LinkOnly],
        preview_risk: Risk::Medium,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::DisabledUntilTermsRead,
        note: "Catalogue cover service for 978-2 and 979-10 ISBNs. The metadata is under the \
               Licence Ouverte; the images are believed not to be, and their origin and reuse \
               terms are unknown. Disabled until those terms are read; then link only. A HEAD \
               answer of 200 whose Content-Length is 4566 or 4658 is the placeholder.",
    },
    SourceInfo {
        id: "dnb",
        name: "Deutsche Nationalbibliothek",
        homepage: "https://portal.dnb.de",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 2000,
        hosts: &["portal.dnb.de"],
        uses: &[Use::LinkOnly],
        preview_risk: Risk::Medium,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::DisabledUntilTermsRead,
        note: "Catalogue cover service for 978-3 ISBNs. Origin of the images (the URL path \
               suggests the German trade catalogue) and reuse terms are unknown. Disabled \
               until those terms are read; then link only. Any HTTP error on HEAD means no \
               cover.",
    },
    SourceInfo {
        id: "publisher_page",
        name: "Publisher's page",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: Some(false),
        min_interval_ms: 5000,
        hosts: &[],
        uses: &[Use::LinkOnly, Use::DownloadCli],
        preview_risk: Risk::Medium,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Enabled,
        note: "A page URL pasted by the user. Fetched server-side only after the host guard, \
               robots.txt and the TDM reservation signals allow it; at most 3 requests per \
               user action and 5 s between actions on one host. The image it names is shown \
               as a plain link, never inlined, cached or downloaded: per-site terms are \
               unknown and several French publishing groups reserve text and data mining.",
    },
    SourceInfo {
        id: "reference",
        name: "Cover reference in the book record",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 5000,
        hosts: &[],
        uses: &[Use::LinkOnly],
        preview_risk: Risk::Medium,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Enabled,
        note: "An image URL carried by a catalogue record, from a host with no recorded \
               terms. Shown as a plain link only.",
    },
    SourceInfo {
        id: "librarything",
        name: "LibraryThing",
        homepage: "https://www.librarything.com",
        terms_url: None,
        terms_last_read: None,
        requires_key: true,
        browser_cors: None,
        min_interval_ms: 0,
        hosts: &[],
        uses: &[],
        preview_risk: Risk::High,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Never,
        note: "Not used: developer key, unclear terms, nothing verified.",
    },
    SourceInfo {
        id: "trade_databases",
        name: "Trade databases (Electre, Dilicom, Tite-Live, ORB, VLB)",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: true,
        browser_cors: None,
        min_interval_ms: 0,
        hosts: &[],
        uses: &[],
        preview_risk: Risk::High,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::ContractOnly,
        note: "Paid B2B services licensed for catalogue display by subscribers. Usable only \
               through a reseller's own contract, i.e. as its feed; never queried by a lookup.",
    },
    SourceInfo {
        id: "library_subscriptions",
        name: "Library image subscriptions (Syndetics, Content Cafe)",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: true,
        browser_cors: None,
        min_interval_ms: 0,
        hosts: &[],
        uses: &[],
        preview_risk: Risk::High,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Never,
        note: "Subscription services for library catalogue display. Never used.",
    },
    SourceInfo {
        id: "amazon",
        name: "Amazon (image URLs, product pages, product APIs) and Goodreads",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 0,
        hosts: &[],
        uses: &[],
        preview_risk: Risk::High,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Never,
        note: "Never, in any mode, spines and variants included: the associates terms are \
               believed to allow images only to send traffic to Amazon and to forbid offline \
               use; Goodreads images are Amazon's. Hosts are on the denylist.",
    },
    SourceInfo {
        id: "retailers",
        name: "Retailer and aggregator pages and image CDNs",
        homepage: "",
        terms_url: None,
        terms_last_read: None,
        requires_key: false,
        browser_cors: None,
        min_interval_ms: 0,
        hosts: &[],
        uses: &[],
        preview_risk: Risk::High,
        store_risk: Risk::High,
        print_risk: Risk::High,
        status: SourceStatus::Never,
        note: "Never, in any mode (Fnac, Decitre, Furet du Nord, Cultura, ePagine/leslibraires, \
               Babelio and the like, and image search results): EU database makers' rights, \
               site terms and parasitisme exposure; a real-time ISBN-to-cover relay over a \
               retailer's catalogue is the pattern courts have condemned. Hosts are on the \
               denylist; a user who has the right supplies the file directly.",
    },
];

/// The descriptor of the source with this id.
#[must_use]
pub fn source(id: &str) -> Option<&'static SourceInfo> {
    SOURCES.iter().find(|s| s.id == id)
}

/// Looks a built-in source up; the ids used inside this module always exist, and the
/// fallback keeps the function total without a panic path.
fn builtin(id: &'static str) -> &'static SourceInfo {
    source(id).unwrap_or(&SOURCES[0])
}

/// What the request of a candidate is for, which also says where it may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CandidateRole {
    /// A metadata request that returns **facts** (cover id, pixel size, record id) as
    /// JSON or XML. Runs in the user's browser where CORS allows, else through a relay
    /// that returns text metadata only.
    Locate,
    /// A `HEAD` request: existence by status and `Content-Length`. No body.
    Probe,
    /// A small image to show inline: the URL goes into an `<img>` in the user's
    /// browser (with `referrerpolicy="no-referrer"`), is never fetched by our servers,
    /// never drawn to a canvas, never handed to WebAssembly.
    Preview,
    /// The image at its source, to show as a plain link ("open the image on …").
    Link,
    /// The HTML of a page the user pasted; fetched server-side under the fetcher rules,
    /// then given to [`crate::scrape::extract`].
    Page,
    /// Download of the image bytes by an operator tool that holds an explicit rights
    /// basis. Never produced for the hosted service.
    Download,
}

/// One step of the cascade: a request, its source and what it may be used for.
///
/// There is deliberately no field for image bytes and none for a rights basis.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverCandidate {
    /// The request to run (or, for [`CandidateRole::Preview`] and
    /// [`CandidateRole::Link`], the URL to show).
    pub request: HttpRequest,
    /// Source id; see [`source`].
    pub source: &'static str,
    /// Which part of the book the image shows.
    pub kind: CoverKind,
    /// Risk of the act this candidate describes: the source's preview risk for
    /// locating, probing, previewing and linking; its store risk for a download.
    pub risk: Risk,
    /// The terms that make this step acceptable, and what the caller must do or not do.
    pub note: &'static str,
    /// What the request is for.
    pub role: CandidateRole,
    /// The ISBN the request is about, when it is keyed by ISBN.
    pub isbn: Option<Isbn13>,
    /// The request is about a related ISBN (another edition or format), not the book's
    /// own: flag the result "other edition" and let the user confirm.
    pub other_edition: bool,
    /// Pixel size declared by the record the URL came from, if any. Unverified.
    pub declared_px: Option<(u32, u32)>,
    /// Minimum interval, in milliseconds, between this request and the previous one to
    /// the same host ([`Policy::delay_ms`] adds the robots.txt `Crawl-delay`).
    pub min_interval_ms: u32,
}

impl CoverCandidate {
    /// The rights basis of anything found through a candidate: always
    /// [`RightsBasis::None`]. A lookup never yields a right to print.
    #[must_use]
    pub const fn rights_basis(&self) -> RightsBasis {
        RightsBasis::None
    }

    /// The source descriptor.
    #[must_use]
    pub fn source_info(&self) -> Option<&'static SourceInfo> {
        source(self.source)
    }

    /// Whether `response` — the answer to this candidate's request, after redirects —
    /// is a hit, so that the cascade can stop here.
    ///
    /// * the status must be 2xx (Open Library answers 404 thanks to `default=false`,
    ///   the DNB answers an error, when there is no cover);
    /// * the final URL must still pass the host guard (a redirect to a denied or
    ///   internal host is not a hit);
    /// * a BnF probe must not announce one of the two placeholder lengths;
    /// * an answer that announces `text/html` to an image request is an error page;
    /// * when a body is present for an image request (operator downloads), it must be
    ///   a recognised image and not a placeholder ([`crate::placeholder`]).
    ///
    /// A `true` here says nothing about rights: the candidate's basis is still `none`.
    #[must_use]
    pub fn accepts(&self, response: &HttpResponse) -> bool {
        if !response.is_success() {
            return false;
        }
        if !response.final_url.is_empty() {
            let checked = if self.source == "reseller_feed" {
                guard::check_url_shape(&response.final_url)
            } else {
                guard::check_url(&response.final_url)
            };
            if checked.is_err() {
                return false;
            }
        }
        if matches!(self.role, CandidateRole::Locate | CandidateRole::Page) {
            return true;
        }
        if response.content_type().is_some_and(|t| t.starts_with("text/")) {
            return false;
        }
        let announced = response.header("content-length").and_then(|v| v.trim().parse::<u64>().ok());
        if self.source == "bnf"
            && announced.is_some_and(crate::placeholder::is_bnf_placeholder_length)
        {
            return false;
        }
        if response.body.is_empty() {
            return true;
        }
        match crate::sniff::probe(&response.body) {
            Some(info) => {
                !crate::placeholder::assess_for_source(self.source, &info, &response.body).is_placeholder()
            }
            None => false,
        }
    }
}

/// Which product flow the lookup serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// An individual, author or publisher making a card: lookups help find the cover.
    #[default]
    Standalone,
    /// A reseller integration: the reseller's feed is the only source; lookups never
    /// run.
    Reseller,
}

/// Publishers that asked for no lookup and no preview of their titles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OptOutList {
    /// ISBN prefixes (digits; hyphens and spaces ignored), typically a registrant
    /// prefix such as `"978-2-07"`.
    pub isbn_prefixes: Vec<String>,
    /// Publisher or imprint names, compared case-insensitively.
    pub publishers: Vec<String>,
    /// Hosts (and their subdomains) that must not be contacted or linked.
    pub hosts: Vec<String>,
}

impl OptOutList {
    /// `true` when the book's ISBN (or a related one), publisher or imprint is listed.
    #[must_use]
    pub fn blocks_book(&self, book: &BookMeta) -> bool {
        let isbn_listed = |isbn: &Isbn13| {
            let plain = isbn.to_plain();
            self.isbn_prefixes.iter().any(|prefix| {
                let digits: String = prefix.chars().filter(char::is_ascii_digit).collect();
                !digits.is_empty() && plain.starts_with(&digits)
            })
        };
        let name_listed = |name: &str| {
            let name = name.trim();
            !name.is_empty() && self.publishers.iter().any(|p| p.trim().eq_ignore_ascii_case(name) || p.trim().to_lowercase() == name.to_lowercase())
        };
        book.isbn13.as_ref().is_some_and(isbn_listed)
            || book.related_isbns.iter().any(isbn_listed)
            || book.publisher.as_deref().is_some_and(name_listed)
            || book.imprint.as_deref().is_some_and(name_listed)
    }

    /// `true` when `host` or a parent domain of it is listed.
    #[must_use]
    pub fn blocks_host(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        self.hosts.iter().any(|listed| {
            let listed = listed.trim().trim_end_matches('.').to_ascii_lowercase();
            !listed.is_empty() && guard::host_matches_suffix(&host, &listed)
        })
    }
}

/// How lookups behave: which sources, up to which risk, and how the fetcher identifies
/// and paces itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// Standalone (default) or reseller mode.
    pub mode: Mode,
    /// Candidates riskier than this are not produced. Default [`Risk::Medium`]; the
    /// hosted service must never raise it.
    pub max_risk: Risk,
    /// Source ids that may be used, **in lookup order** (the research wants the order
    /// configurable so that a measured hit rate can change it). Default:
    /// `reseller_feed`, `openlibrary`, `googlebooks`, `publisher_page`, `reference`.
    /// Adding `bnf` or `dnb` records the
    /// operator's decision to enable them (see [`SourceStatus::DisabledUntilTermsRead`]);
    /// ids whose status is `Never` or `ContractOnly` have no effect.
    pub allowed_sources: Vec<String>,
    /// The `User-Agent` of every request: a fixed product token and version, a URL
    /// describing the bot and a contact address. Never a browser's. See
    /// [`Policy::user_agent_problems`].
    pub user_agent: String,
    /// Obey robots.txt (default `true`). Turning it off is for an operator fetching
    /// from hosts it controls; the hosted service must leave it on.
    pub obey_robots: bool,
    /// Minimum interval between two requests to a host that has no source-specific
    /// figure, in milliseconds. Default 5000.
    pub per_host_delay_ms: u32,
    /// At most this many requests per user action on a pasted page's host (robots.txt,
    /// tdmrep.json, the page). Default 3.
    pub max_requests_per_action: u32,
    /// Google Books API key; without one the source is skipped.
    pub google_books_api_key: Option<String>,
    /// Also look related ISBNs up (e.g. the print edition of an eBook), flagged
    /// [`CoverCandidate::other_edition`]. Default 1 — "retried once".
    pub max_related_isbns: usize,
    /// Set by an operator's command-line or self-hosted tool that holds the rights: the
    /// basis under which downloaded bytes will be used. Together with a `max_risk` of
    /// High it makes [`CandidateRole::Download`] candidates appear. `None` (default)
    /// and [`RightsBasis::None`] produce none. The tool must print that the operator is
    /// responsible for the rights.
    pub operator_rights_basis: Option<RightsBasis>,
    /// Publisher opt-outs, consulted before any lookup.
    pub opt_out: OptOutList,
    /// Hosts (and their subdomains) a pasted page may be on. Empty (default): any host
    /// that passes the guard. The denylist only names the retailers the research
    /// knew; a deployment that wants "publisher pages only" lists publisher domains
    /// here.
    pub page_hosts: Vec<String>,
}

/// Default user agent: product token (contains "bot", as Open Library's rules for
/// self-identifying clients expect), version and the project URL. A deployment must
/// replace it with one that carries its own bot-information URL and contact address.
pub const DEFAULT_USER_AGENT: &str = concat!(
    "eBookCadeauBot/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/victorpaleologue/ebook-cadeau)"
);

impl Default for Policy {
    fn default() -> Self {
        Self {
            mode: Mode::Standalone,
            max_risk: Risk::Medium,
            allowed_sources: ["reseller_feed", "openlibrary", "googlebooks", "publisher_page", "reference"]
                .map(String::from)
                .to_vec(),
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            obey_robots: true,
            per_host_delay_ms: 5000,
            max_requests_per_action: 3,
            google_books_api_key: None,
            max_related_isbns: 1,
            operator_rights_basis: None,
            opt_out: OptOutList::default(),
            page_hosts: Vec::new(),
        }
    }
}

impl Policy {
    /// The policy of the hosted service in standalone mode (same as `default()`).
    #[must_use]
    pub fn hosted() -> Self {
        Self::default()
    }

    /// Reseller mode: the feed's covers only, no lookup.
    #[must_use]
    pub fn reseller() -> Self {
        Self { mode: Mode::Reseller, ..Self::default() }
    }

    /// An operator tool that holds `basis` and may download: `max_risk` High, downloads
    /// on. `basis` must not be `none` for downloads to appear.
    #[must_use]
    pub fn operator(basis: RightsBasis) -> Self {
        Self { max_risk: Risk::High, operator_rights_basis: Some(basis), ..Self::default() }
    }

    /// Adds a source id to [`Policy::allowed_sources`] (e.g. `"bnf"`, `"dnb"`).
    #[must_use]
    pub fn with_source(mut self, id: &str) -> Self {
        if !self.allowed_sources.iter().any(|s| s == id) {
            self.allowed_sources.push(id.to_owned());
        }
        self
    }

    /// `true` when the source may be used under this policy: it is in
    /// [`Policy::allowed_sources`], its status permits it (a key is configured where
    /// one is needed; `Never` and `ContractOnly` are always refused) and its preview
    /// risk does not exceed [`Policy::max_risk`].
    #[must_use]
    pub fn allows_source(&self, id: &str) -> bool {
        let Some(info) = source(id) else { return false };
        let status_ok = match info.status {
            SourceStatus::Enabled | SourceStatus::DisabledUntilTermsRead => true,
            SourceStatus::RequiresKey => {
                self.google_books_api_key.as_deref().is_some_and(|k| !k.trim().is_empty())
            }
            SourceStatus::ContractOnly | SourceStatus::Never => false,
        };
        status_ok && info.preview_risk <= self.max_risk && self.allowed_sources.iter().any(|s| s == id)
    }

    /// `true` when downloads may be described: an operator rights basis other than
    /// `none`, and a `max_risk` of High.
    #[must_use]
    pub fn downloads_enabled(&self) -> bool {
        self.max_risk >= Risk::High
            && self.operator_rights_basis.is_some_and(|basis| basis != RightsBasis::None)
    }

    /// The product token robots.txt groups are matched against: the leading run of
    /// letters, `_` and `-` of the user agent (e.g. `"eBookCadeauBot"`).
    #[must_use]
    pub fn product_token(&self) -> &str {
        let ua = self.user_agent.trim_start();
        let end = ua
            .char_indices()
            .find(|(_, c)| !(c.is_ascii_alphabetic() || *c == '_' || *c == '-'))
            .map_or(ua.len(), |(at, _)| at);
        &ua[..end]
    }

    /// Reasons the user agent breaks the fetcher rules; empty when it is acceptable.
    ///
    /// Checked: starts with a product token; carries a bot-information URL or a contact
    /// address; does not imitate a browser; does not contain `CloudFront` or
    /// `Bytespider` (dropped by Open Library); is a single printable ASCII line.
    #[must_use]
    pub fn user_agent_problems(&self) -> Vec<&'static str> {
        let ua = self.user_agent.as_str();
        let lower = ua.to_ascii_lowercase();
        let mut problems = Vec::new();
        if self.product_token().len() < 3 {
            problems.push("must start with a product token (letters, `_`, `-`)");
        }
        if !(lower.contains("http") || lower.contains('@')) {
            problems.push("must carry a bot-information URL or a contact address");
        }
        if ["mozilla/", "chrome/", "safari/", "firefox/", "applewebkit", "gecko/", "edg/", "opera/"]
            .iter()
            .any(|browser| lower.contains(browser))
        {
            problems.push("must not imitate a browser");
        }
        if lower.contains("cloudfront") || lower.contains("bytespider") {
            problems.push("must not contain CloudFront or Bytespider");
        }
        if ua.is_empty() || ua.len() > 256 || !ua.bytes().all(|b| (0x20..0x7F).contains(&b)) {
            problems.push("must be one line of printable ASCII, at most 256 bytes");
        }
        problems
    }

    /// Whether `url` may be fetched by our own fetcher: the host guard
    /// ([`guard::check_url`]), the publisher opt-out hosts, then robots.txt.
    ///
    /// `robots` is the verdict for the URL's origin ([`Robots::from_response`]). With
    /// [`Policy::obey_robots`] on, a missing verdict is a refusal: a fetcher that could
    /// not read robots.txt must not fetch.
    ///
    /// # Errors
    /// The [`Refusal`] that applies.
    pub fn may_fetch(&self, url: &str, robots: Option<&Robots>) -> Result<Url, Refusal> {
        let parsed = guard::check_url(url)?;
        if parsed.host_str().is_some_and(|host| self.opt_out.blocks_host(host)) {
            return Err(Refusal::OptedOut);
        }
        if self.obey_robots {
            let robots = robots.ok_or(Refusal::RobotsNotChecked)?;
            if !robots.allowed(&self.user_agent, &guard::path_and_query(&parsed)) {
                return Err(Refusal::RobotsDisallow);
            }
        }
        Ok(parsed)
    }

    /// The delay to respect before `candidate`'s request, in milliseconds: the
    /// candidate's own interval, or the robots.txt `Crawl-delay` when that is longer
    /// (capped at one hour).
    #[must_use]
    pub fn delay_ms(&self, candidate: &CoverCandidate, robots: Option<&Robots>) -> u32 {
        let crawl_delay = robots
            .filter(|_| self.obey_robots)
            .and_then(|r| r.crawl_delay(&self.user_agent))
            .map_or(0, |seconds| (f64::from(seconds) * 1000.0).clamp(0.0, 3_600_000.0) as u32);
        candidate.min_interval_ms.max(crawl_delay)
    }

    /// The request for the robots.txt of `url`'s origin, when `url` passes the guard.
    #[must_use]
    pub fn robots_request(&self, url: &str) -> Option<HttpRequest> {
        let parsed = guard::check_url(url).ok()?;
        Some(self.get(&format!("{}/robots.txt", guard::origin_of(&parsed)), "text/plain"))
    }

    /// The request for the TDMRep file of `url`'s origin, when `url` passes the guard.
    #[must_use]
    pub fn tdmrep_request(&self, url: &str) -> Option<HttpRequest> {
        let parsed = guard::check_url(url).ok()?;
        Some(self.get(&format!("{}/.well-known/tdmrep.json", guard::origin_of(&parsed)), "application/json"))
    }

    fn get(&self, url: &str, accept: &str) -> HttpRequest {
        HttpRequest::get(url).with_header("User-Agent", self.user_agent.clone()).with_header("Accept", accept)
    }

    fn head(&self, url: &str) -> HttpRequest {
        HttpRequest { method: Method::Head, ..self.get(url, "image/*") }
    }
}

// ------------------------------------------------------------------------------------
// URL builders
// ------------------------------------------------------------------------------------

/// Open Library size suffixes: bounding boxes of 116 × 58 (`S`), 180 × 360 (`M`) and
/// 500 × 500 px (`L`); `Original` is the unsuffixed URL, the upload as received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenLibrarySize {
    /// `-S`
    Small,
    /// `-M` — the preview size.
    Medium,
    /// `-L`
    Large,
    /// No suffix: the original upload.
    Original,
}

impl OpenLibrarySize {
    const fn suffix(self) -> &'static str {
        match self {
            Self::Small => "-S",
            Self::Medium => "-M",
            Self::Large => "-L",
            Self::Original => "",
        }
    }
}

/// Open Library cover image by ISBN. Always carries `?default=false`, so that a missing
/// cover is a 404 and not a placeholder image.
#[must_use]
pub fn openlibrary_isbn_image_url(isbn: &Isbn13, size: OpenLibrarySize) -> String {
    format!("https://covers.openlibrary.org/b/isbn/{}{}.jpg?default=false", isbn.to_plain(), size.suffix())
}

/// Open Library cover facts by ISBN: redirects to the id record, whose JSON holds
/// `id`, `width`, `height`; 404 when there is no cover.
#[must_use]
pub fn openlibrary_isbn_json_url(isbn: &Isbn13) -> String {
    format!("https://covers.openlibrary.org/b/isbn/{}.json", isbn.to_plain())
}

/// Open Library cover image by cover id (long cache lifetime, the form to use once the
/// id is known). Always carries `?default=false`.
#[must_use]
pub fn openlibrary_id_image_url(cover_id: u64, size: OpenLibrarySize) -> String {
    format!("https://covers.openlibrary.org/b/id/{cover_id}{}.jpg?default=false", size.suffix())
}

/// The cover id in an Open Library cover URL of the form
/// `https://covers.openlibrary.org/b/id/{id}[-S|-M|-L].jpg`.
#[must_use]
pub fn openlibrary_cover_id(url: &str) -> Option<u64> {
    let parsed = Url::parse(url.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str() != Some("covers.openlibrary.org") {
        return None;
    }
    let rest = parsed.path().strip_prefix("/b/id/")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let tail = &rest[digits.len()..];
    let known_tail = ["", ".jpg", "-S.jpg", "-M.jpg", "-L.jpg", ".json"].contains(&tail);
    digits.parse::<u64>().ok().filter(|id| *id > 0 && known_tail)
}

/// Cleans a Google Books image link (`https://books.google.com/books/content?id=…` or
/// `…/books?id=…&printsec=frontcover&img=1&zoom=N`): forces `https`, removes the
/// page-curl effect (`edge=curl`) and, with `zoom`, replaces the zoom level (5 is the
/// smallest thumbnail, 1 the usual one, 0 the largest Google will serve).
///
/// Returns `None` when the URL is not a Google Books image link.
#[must_use]
pub fn google_books_image_url(url: &str, zoom: Option<u8>) -> Option<String> {
    let mut parsed = Url::parse(url.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str() != Some("books.google.com")
        || !parsed.path().starts_with("/books")
    {
        return None;
    }
    let mut pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(key, _)| key != "edge")
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .take(32)
        .collect();
    if !pairs.iter().any(|(key, value)| key == "id" && !value.is_empty()) {
        return None;
    }
    if let Some(zoom) = zoom {
        pairs.retain(|(key, _)| key != "zoom");
        pairs.push(("zoom".to_owned(), zoom.min(9).to_string()));
    }
    parsed.query_pairs_mut().clear().extend_pairs(pairs);
    parsed.set_scheme("https").ok()?;
    parsed.set_fragment(None);
    Some(parsed.into())
}

/// The BnF catalogue cover URL for an ARK (`ark:/12148/cb…`), or `None` when the ARK
/// does not have that shape. Probe it with `HEAD`: status 200 and a `Content-Length`
/// other than 4566 or 4658 ([`crate::placeholder::is_bnf_placeholder_length`]).
#[must_use]
pub fn bnf_cover_url(ark: &str) -> Option<String> {
    let ark = bnf_ark_in(ark)?;
    Some(format!("https://catalogue.bnf.fr/couverture?appName=NE&idArk={ark}&couverture=1"))
}

/// The first BnF catalogue ARK (`ark:/12148/cb` followed by letters and digits) in a
/// text — a record URL or an SRU response.
#[must_use]
pub fn bnf_ark_in(text: &str) -> Option<String> {
    const PREFIX: &str = "ark:/12148/cb";
    let start = text.find(PREFIX)?;
    let id: String = text[start + PREFIX.len()..]
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .take(24)
        .collect();
    (id.len() >= 6).then(|| format!("{PREFIX}{id}"))
}

fn isbn_in_group(isbn: &Isbn13, prefixes: &[&str]) -> bool {
    let plain = isbn.to_plain();
    prefixes.iter().any(|prefix| plain.starts_with(prefix))
}

// ------------------------------------------------------------------------------------
// The cascade
// ------------------------------------------------------------------------------------

const NOTE_FEED: &str = "Cover supplied by the reseller's feed. Its rights basis is whatever the \
    reseller's per-title flag says (none unless declared); this candidate only locates it.";
const NOTE_OL_LOCATE: &str = "Open Library cover facts (id, width, height) as JSON; 404 means no \
    cover. Facts only. Run from the user's browser; at most one ISBN-keyed request every 3 s.";
const NOTE_OL_PREVIEW: &str = "Open Library preview (fits 180 x 360 px): show inline from the \
    source with its name and a link back, never proxy, cache or read the pixels. `default=false` \
    makes a missing cover a 404. No licence on the images: preview only, never printed.";
const NOTE_OL_LINK: &str = "Open Library original upload: show as a link (\"open the image on Open \
    Library\"). A user who holds the rights may save it and supply the file under their own \
    declaration. May redirect to archive.org.";
const NOTE_OL_DOWNLOAD: &str = "Download of the Open Library original by an operator tool under \
    the operator's explicit rights basis; the operator is responsible for the rights. Follow up \
    to 5 redirects, archive.org allowed. Never done by the hosted service.";
const NOTE_GOOGLE_LOCATE: &str = "Google Books volume lookup with the configured API key. Keep \
    the result for the session only; reject volumes without imageLinks (a missing cover is a \
    dummy image). Do not log the URL: it carries the key.";
const NOTE_GOOGLE_PREVIEW: &str = "Google Books thumbnail, page curl removed: show inline with \
    \"Google Books\" attribution and a link to the volume; never cache, download or print.";
const NOTE_GOOGLE_LINK: &str = "Largest rendition Google Books serves for this volume (zoom=0), \
    as a link only. Its terms are not believed to allow any use beyond display with attribution.";
const NOTE_BNF_LOCATE: &str = "BnF catalogue SRU lookup by ISBN, to learn the record's ARK \
    (text metadata under the Licence Ouverte). Follow with a HEAD on bnf_cover_url(ark).";
const NOTE_BNF_PROBE: &str = "BnF catalogue cover, HEAD only: a cover exists when the status is \
    200 and Content-Length is neither 4566 nor 4658. Image terms unread: link only.";
const NOTE_BNF_LINK: &str = "BnF catalogue cover, as a link only: the reuse terms of these \
    images have not been read.";
const NOTE_DNB_PROBE: &str = "DNB cover service, HEAD only: any HTTP error means no cover. Image \
    origin and terms unread: link only.";
const NOTE_DNB_LINK: &str = "DNB cover, as a link only: the reuse terms of these images have not \
    been read.";
const NOTE_REFERENCE: &str = "Image URL carried by the book record, from a host with no recorded \
    terms: show as a link only, never inline, never fetched by our servers.";
const NOTE_PAGE: &str = "Publisher page pasted by the user: fetch server-side only after \
    robots.txt and the TDM reservation signals allow it, 2 MiB body cap, at most 5 redirects \
    re-checked against the host guard; pass the HTML to scrape::extract and show what it finds \
    as plain links. The page must state the book's ISBN.";
const NOTE_PAGE_IMAGE: &str = "Image named by the publisher's page: show as a plain link with the \
    publisher's name, never inline. Per-site terms are unknown.";
const NOTE_PAGE_DOWNLOAD: &str = "Download of an image named by a publisher's page, by an operator \
    tool under the operator's explicit rights basis (e.g. a publisher fetching its own covers); \
    the operator is responsible for the rights. Check robots.txt first.";

struct Builder<'a> {
    policy: &'a Policy,
    out: Vec<CoverCandidate>,
}

impl Builder<'_> {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        source_id: &'static str,
        role: CandidateRole,
        request: HttpRequest,
        kind: CoverKind,
        note: &'static str,
        isbn: Option<Isbn13>,
        other_edition: bool,
        declared_px: Option<(u32, u32)>,
        min_interval_ms: u32,
    ) {
        let info = builtin(source_id);
        let risk = if role == CandidateRole::Download { info.store_risk } else { info.preview_risk };
        if risk > self.policy.max_risk {
            return;
        }
        if role == CandidateRole::Download
            && !(self.policy.downloads_enabled() && info.allows(Use::DownloadCli))
        {
            return;
        }
        if self.out.iter().any(|c| c.role == role && c.request.url == request.url && c.request.method == request.method) {
            return;
        }
        self.out.push(CoverCandidate {
            request,
            source: source_id,
            kind,
            risk,
            note,
            role,
            isbn,
            other_edition,
            declared_px,
            min_interval_ms,
        });
    }

    fn declared(cover: &CoverRef) -> Option<(u32, u32)> {
        cover.width_px.zip(cover.height_px).filter(|(w, h)| *w > 0 && *h > 0)
    }

    /// Step 0: covers the book record already references, routed to their source.
    fn references(&mut self, book: &BookMeta) {
        let policy = self.policy;
        for cover in book.covers.iter().take(16) {
            let declared = Self::declared(cover);
            // Known sources first: their URLs are rebuilt in a clean `https` form, so a
            // record that carries `http://` links (Google's API does) is still usable.
            if let Some(id) = openlibrary_cover_id(&cover.url) {
                if policy.allows_source("openlibrary") {
                    self.openlibrary_by_id(id, cover.kind, declared);
                }
                continue;
            }
            if let Some(thumbnail) = google_books_image_url(&cover.url, None) {
                if policy.allows_source("googlebooks") && guard::check_url(&thumbnail).is_ok() {
                    let interval = builtin("googlebooks").min_interval_ms;
                    let request = policy.get(&thumbnail, "image/*");
                    self.push("googlebooks", CandidateRole::Preview, request, cover.kind, NOTE_GOOGLE_PREVIEW, None, false, declared, interval);
                    if let Some(larger) = google_books_image_url(&thumbnail, Some(0)) {
                        let request = policy.get(&larger, "image/*");
                        self.push("googlebooks", CandidateRole::Link, request, cover.kind, NOTE_GOOGLE_LINK, None, false, None, interval);
                    }
                }
                continue;
            }
            let Ok(url) = guard::check_url(&cover.url) else { continue };
            if url.host_str().is_some_and(|host| policy.opt_out.blocks_host(host)) {
                continue;
            }
            if policy.allows_source("reference") {
                let request = policy.get(url.as_str(), "image/*");
                self.push("reference", CandidateRole::Link, request, cover.kind, NOTE_REFERENCE, None, false, declared, policy.per_host_delay_ms);
            }
        }
    }

    /// Id-keyed Open Library candidates: no ISBN request needed, 1 request per second.
    fn openlibrary_by_id(&mut self, id: u64, kind: CoverKind, declared: Option<(u32, u32)>) {
        let policy = self.policy;
        let steps = [
            (CandidateRole::Preview, OpenLibrarySize::Medium, NOTE_OL_PREVIEW),
            (CandidateRole::Link, OpenLibrarySize::Original, NOTE_OL_LINK),
            (CandidateRole::Download, OpenLibrarySize::Original, NOTE_OL_DOWNLOAD),
        ];
        for (role, size, note) in steps {
            let request = policy.get(&openlibrary_id_image_url(id, size), "image/*");
            self.push("openlibrary", role, request, kind, note, None, false, declared, 1000);
        }
    }

    fn openlibrary(&mut self, isbn: &Isbn13, other_edition: bool) {
        let policy = self.policy;
        let interval = builtin("openlibrary").min_interval_ms;
        self.push(
            "openlibrary",
            CandidateRole::Locate,
            policy.get(&openlibrary_isbn_json_url(isbn), "application/json"),
            CoverKind::Front,
            NOTE_OL_LOCATE,
            Some(*isbn),
            other_edition,
            None,
            interval,
        );
        let steps = [
            (CandidateRole::Preview, OpenLibrarySize::Medium, NOTE_OL_PREVIEW),
            (CandidateRole::Link, OpenLibrarySize::Original, NOTE_OL_LINK),
            (CandidateRole::Download, OpenLibrarySize::Original, NOTE_OL_DOWNLOAD),
        ];
        for (role, size, note) in steps {
            let request = policy.get(&openlibrary_isbn_image_url(isbn, size), "image/*");
            self.push("openlibrary", role, request, CoverKind::Front, note, Some(*isbn), other_edition, None, interval);
        }
    }

    fn googlebooks(&mut self, isbn: &Isbn13, other_edition: bool) {
        let policy = self.policy;
        let Some(key) = policy.google_books_api_key.as_deref() else { return };
        let Ok(mut url) = Url::parse("https://www.googleapis.com/books/v1/volumes") else { return };
        url.query_pairs_mut()
            .append_pair("q", &format!("isbn:{}", isbn.to_plain()))
            .append_pair("maxResults", "1")
            .append_pair("key", key.trim());
        self.push(
            "googlebooks",
            CandidateRole::Locate,
            policy.get(url.as_str(), "application/json"),
            CoverKind::Front,
            NOTE_GOOGLE_LOCATE,
            Some(*isbn),
            other_edition,
            None,
            builtin("googlebooks").min_interval_ms,
        );
    }

    fn bnf(&mut self, book: &BookMeta, isbn: &Isbn13, other_edition: bool) {
        if !isbn_in_group(isbn, &["9782", "97910"]) {
            return;
        }
        let policy = self.policy;
        let interval = builtin("bnf").min_interval_ms;
        // The ARK is already known when the catalogue lookup went through the BnF.
        let known_ark = (!other_edition)
            .then(|| {
                book.sources
                    .iter()
                    .filter(|s| s.provider == "bnf")
                    .find_map(|s| s.record_url.as_deref().and_then(bnf_ark_in))
            })
            .flatten();
        match known_ark.as_deref().and_then(bnf_cover_url) {
            Some(cover_url) => {
                self.push("bnf", CandidateRole::Probe, policy.head(&cover_url), CoverKind::Front, NOTE_BNF_PROBE, Some(*isbn), other_edition, None, interval);
                self.push("bnf", CandidateRole::Link, policy.get(&cover_url, "image/*"), CoverKind::Front, NOTE_BNF_LINK, Some(*isbn), other_edition, None, interval);
            }
            None => {
                let url = format!(
                    "https://catalogue.bnf.fr/api/SRU?version=1.2&operation=searchRetrieve&query=bib.isbn%20all%20%22{}%22&recordSchema=dublincore&maximumRecords=1",
                    isbn.to_plain()
                );
                self.push("bnf", CandidateRole::Locate, policy.get(&url, "application/xml"), CoverKind::Front, NOTE_BNF_LOCATE, Some(*isbn), other_edition, None, interval);
            }
        }
    }

    fn dnb(&mut self, isbn: &Isbn13, other_edition: bool) {
        if !isbn_in_group(isbn, &["9783"]) {
            return;
        }
        let policy = self.policy;
        let interval = builtin("dnb").min_interval_ms;
        let url = format!("https://portal.dnb.de/opac/mvb/cover?isbn={}", isbn.to_plain());
        self.push("dnb", CandidateRole::Probe, policy.head(&url), CoverKind::Front, NOTE_DNB_PROBE, Some(*isbn), other_edition, None, interval);
        self.push("dnb", CandidateRole::Link, policy.get(&url, "image/*"), CoverKind::Front, NOTE_DNB_LINK, Some(*isbn), other_edition, None, interval);
    }
}

/// The ordered cascade of lookup steps for `book` under `policy`.
///
/// Order: covers the record already references (each routed to its source, so that a
/// known Open Library cover id or Google image link costs no extra lookup), then, for
/// the book's own ISBN and after it for up to [`Policy::max_related_isbns`] related
/// ISBNs, the lookup sources **in the order of [`Policy::allowed_sources`]** — by
/// default Open Library, then Google Books (with a key); BnF (978-2, 979-10) and DNB
/// (978-3) follow when the caller enabled them. Run it top to bottom and stop at the
/// first accepted hit ([`CoverCandidate::accepts`]): each user action should cost the
/// sources as few requests as possible.
///
/// Returns nothing when the publisher opted out ([`Policy::opt_out`]) — the output is
/// then the typographic card — and, in [`Mode::Reseller`], only the covers the record
/// (the reseller's feed) references: lookups never run in reseller mode.
///
/// URLs on the denylist, non-`https` URLs and known placeholder URLs in the record are
/// dropped silently. Every candidate's rights basis is `none`.
///
/// ```
/// use ebc_covers::{candidates, CandidateRole, Policy, RightsBasis};
/// use ebc_model::BookMeta;
///
/// let mut book = BookMeta::titled("L'Étranger");
/// book.isbn13 = Some("9782070360024".parse().unwrap());
/// let steps = candidates(&book, &Policy::default());
/// assert_eq!(steps[0].source, "openlibrary");
/// assert_eq!(steps[0].request.url, "https://covers.openlibrary.org/b/isbn/9782070360024.json");
/// assert!(steps.iter().all(|c| c.rights_basis() == RightsBasis::None));
/// assert!(steps.iter().all(|c| c.role != CandidateRole::Download));
/// ```
#[must_use]
pub fn candidates(book: &BookMeta, policy: &Policy) -> Vec<CoverCandidate> {
    if policy.opt_out.blocks_book(book) {
        return Vec::new();
    }
    let mut builder = Builder { policy, out: Vec::new() };

    if policy.mode == Mode::Reseller {
        if policy.allows_source("reseller_feed") {
            for cover in book.covers.iter().take(16) {
                // The reseller's own hosts are not judged against the retailer denylist.
                let Ok(url) = guard::check_url_shape(&cover.url) else { continue };
                if crate::placeholder::is_placeholder_url(url.as_str()) {
                    continue;
                }
                let request = policy.get(url.as_str(), "image/*");
                builder.push("reseller_feed", CandidateRole::Link, request, cover.kind, NOTE_FEED, None, false, Builder::declared(cover), 0);
            }
        }
        return builder.out;
    }

    builder.references(book);

    let own = book.isbn13.iter().map(|isbn| (isbn, false));
    let related = book
        .related_isbns
        .iter()
        .filter(|isbn| Some(**isbn) != book.isbn13)
        .take(policy.max_related_isbns)
        .map(|isbn| (isbn, true));
    for (isbn, other_edition) in own.chain(related) {
        for id in &policy.allowed_sources {
            if !policy.allows_source(id) {
                continue;
            }
            match id.as_str() {
                "openlibrary" => builder.openlibrary(isbn, other_edition),
                "googlebooks" => builder.googlebooks(isbn, other_edition),
                "bnf" => builder.bnf(book, isbn, other_edition),
                "dnb" => builder.dnb(isbn, other_edition),
                _ => {}
            }
        }
    }
    builder.out
}

/// The candidate for a product page URL pasted by the user (cascade step 10).
///
/// The URL must pass the host guard — `https`, a named public host, not on the denylist
/// of retailers and aggregators — and the publisher opt-out list. The fetcher must then
/// check robots.txt ([`Policy::robots_request`], [`Policy::may_fetch`]) and the TDM
/// reservation signals ([`Policy::tdmrep_request`], [`crate::tdm`]) before fetching, and
/// hand the HTML to [`crate::scrape::extract`].
///
/// The denylist only knows the hosts the research named: any other host passes, a blog
/// or an unlisted retailer included. A deployment that wants "publisher pages only"
/// sets [`Policy::page_hosts`] to its allow-list of publisher domains (fact-check,
/// omission 7).
///
/// # Errors
/// [`CoversError::UrlRefused`] with the reason: the guard's refusal,
/// [`Refusal::OptedOut`], [`Refusal::NotAllowListed`], or [`Refusal::LookupsDisabled`]
/// in reseller mode or when the `publisher_page` source is not allowed.
pub fn page_candidate(url: &str, policy: &Policy) -> Result<CoverCandidate, CoversError> {
    if policy.mode == Mode::Reseller || !policy.allows_source("publisher_page") {
        return Err(CoversError::UrlRefused(Refusal::LookupsDisabled));
    }
    let parsed = guard::check_url(url).map_err(CoversError::UrlRefused)?;
    if parsed.host_str().is_some_and(|host| policy.opt_out.blocks_host(host)) {
        return Err(CoversError::UrlRefused(Refusal::OptedOut));
    }
    if !policy.page_hosts.is_empty() {
        let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
        let listed = policy.page_hosts.iter().any(|allowed| {
            let allowed = allowed.trim().trim_end_matches('.').to_ascii_lowercase();
            !allowed.is_empty() && guard::host_matches_suffix(&host, &allowed)
        });
        if !listed {
            return Err(CoversError::UrlRefused(Refusal::NotAllowListed));
        }
    }
    let info = builtin("publisher_page");
    Ok(CoverCandidate {
        request: policy.get(parsed.as_str(), "text/html"),
        source: "publisher_page",
        kind: CoverKind::Front,
        risk: info.preview_risk,
        note: NOTE_PAGE,
        role: CandidateRole::Page,
        isbn: None,
        other_edition: false,
        declared_px: None,
        min_interval_ms: policy.per_host_delay_ms.max(info.min_interval_ms),
    })
}

/// Turns what [`crate::scrape::extract`] found on a pasted page into candidates: one
/// plain [`CandidateRole::Link`] per usable image, most likely cover first, plus — for
/// an operator tool with downloads enabled — a [`CandidateRole::Download`].
///
/// Images whose URL was refused by the guard, or whose host opted out, are left out.
/// `other_edition` is set when the page states a related ISBN but not the book's own;
/// when it states neither ([`crate::scrape::IsbnMatch::NotFound`]) the caller should
/// flag the result "ISBN not found on page" before showing anything.
#[must_use]
pub fn page_image_candidates(page: &PageInfo, book: &BookMeta, policy: &Policy) -> Vec<CoverCandidate> {
    if policy.mode == Mode::Reseller
        || !policy.allows_source("publisher_page")
        || policy.opt_out.blocks_book(book)
    {
        return Vec::new();
    }
    let other_edition = page.confirms(book) == crate::scrape::IsbnMatch::OtherEdition;
    let mut builder = Builder { policy, out: Vec::new() };
    let interval = policy.per_host_delay_ms.max(builtin("publisher_page").min_interval_ms);
    for image in page.usable().take(16) {
        let Ok(url) = guard::check_url(&image.url) else { continue };
        if url.host_str().is_some_and(|host| policy.opt_out.blocks_host(host)) {
            continue;
        }
        let declared = image.width.zip(image.height);
        for (role, note) in [(CandidateRole::Link, NOTE_PAGE_IMAGE), (CandidateRole::Download, NOTE_PAGE_DOWNLOAD)] {
            let request = policy.get(url.as_str(), "image/*");
            builder.push("publisher_page", role, request, CoverKind::Front, note, book.isbn13, other_edition, declared, interval);
        }
    }
    builder.out
}
