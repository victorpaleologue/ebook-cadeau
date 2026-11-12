//! `ebc cover`: list cover candidates for an ISBN; in operator mode, download one.
//!
//! The posture is the one of `ebc-covers`: a lookup locates and previews, it never
//! yields a right to print. Without `--rights-basis` nothing is saved, ever. With it the
//! tool acts for an operator who holds that right, says so in one line, and saves the
//! first candidate that is a real image — after robots.txt, with the per-host delays.
//!
//! Not implemented: pasted publisher pages (`scrape`), the TDM reservation signals that
//! go with them, and a check of the *resolved* address of each connection. Redirect
//! hops are checked against the catalogue deny-list by the HTTP client and the final URL
//! against the covers guard.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use ebc_covers::placeholder::assess_for_source;
use ebc_covers::quality::{Quality, Thresholds, Verdict, print_quality_with};
use ebc_covers::robots::Robots;
use ebc_covers::sniff::{Format, ImageInfo};
use ebc_covers::{
    AssetOrigin, CandidateRole, CoverCandidate, Policy, PrintableCover, Risk, RightsBasis,
    candidates, guard, probe,
};
use ebc_layout::CardFormat;
use ebc_model::{BookMeta, CoverKind, HttpResponse};
use serde_json::{Value, json};

use crate::args::{CoverArgs, RiskArg};
use crate::cmd::parse_isbn;
use crate::error::{CliError, Status};
use crate::transport::{NetOptions, Transport};
use crate::util::{Streams, format_by_name, is_dash, redact_url};

/// Largest image accepted from the network.
const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;

/// What happened to one candidate.
#[derive(Debug, Clone, PartialEq)]
enum Outcome {
    /// Not a request this tool runs: a preview or a link for a person to open.
    Listed,
    /// A request this run did not make (no `--probe`, or the cascade already ended).
    NotRun,
    /// Left out because an earlier step showed there is no cover here.
    Skipped(String),
    /// The guard, the opt-out list or robots.txt said no.
    Refused(String),
    /// The request could not be made.
    Failed(String),
    /// The answer is not a cover (404, error page, placeholder, too small).
    Miss(String),
    /// A fact-only request answered: there is a cover.
    Hit(String),
    /// The image was written to the output file.
    Saved(String),
}

impl Outcome {
    fn name(&self) -> &'static str {
        match self {
            Self::Listed => "listed",
            Self::NotRun => "not_run",
            Self::Skipped(_) => "skipped",
            Self::Refused(_) => "refused",
            Self::Failed(_) => "failed",
            Self::Miss(_) => "miss",
            Self::Hit(_) => "hit",
            Self::Saved(_) => "saved",
        }
    }

    fn detail(&self) -> Option<&str> {
        match self {
            Self::Listed | Self::NotRun => None,
            Self::Skipped(text)
            | Self::Refused(text)
            | Self::Failed(text)
            | Self::Miss(text)
            | Self::Hit(text)
            | Self::Saved(text) => Some(text),
        }
    }
}

fn role_name(role: CandidateRole) -> &'static str {
    match role {
        CandidateRole::Locate => "locate",
        CandidateRole::Probe => "probe",
        CandidateRole::Preview => "preview",
        CandidateRole::Link => "link",
        CandidateRole::Page => "page",
        CandidateRole::Download => "download",
    }
}

fn kind_name(kind: CoverKind) -> &'static str {
    match kind {
        CoverKind::Front => "front",
        CoverKind::Back => "back",
        CoverKind::Spine => "spine",
        CoverKind::FullWrap => "full_wrap",
    }
}

fn format_name(format: Format) -> &'static str {
    format.mime().trim_start_matches("image/")
}

fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Insufficient => "insufficient",
        Verdict::Low => "low",
        Verdict::Acceptable => "acceptable",
        Verdict::Excellent => "excellent",
    }
}

/// "309 x 500 px, 71 ppi on the 110 x 178 mm front panel: insufficient".
fn describe_quality(info: &ImageInfo, quality: &Quality, panel: &CardFormat) -> String {
    format!(
        "{} x {} px, {:.0} ppi on the {:.0} x {:.0} mm front panel: {}",
        info.width,
        info.height,
        quality.effective_ppi,
        panel.cover.w,
        panel.cover.h,
        verdict_name(quality.verdict)
    )
}

/// Pacing and robots.txt state of one run.
struct Fetcher<'a> {
    transport: &'a Transport,
    policy: &'a Policy,
    robots: HashMap<String, Robots>,
    last_request: HashMap<String, Instant>,
    requests: usize,
}

impl<'a> Fetcher<'a> {
    fn new(transport: &'a Transport, policy: &'a Policy) -> Self {
        Self { transport, policy, robots: HashMap::new(), last_request: HashMap::new(), requests: 0 }
    }

    /// Waits until `delay_ms` has passed since the previous request to `host`. Fixture
    /// runs do not wait: there is no host to spare.
    fn pace(&mut self, host: &str, delay_ms: u32) {
        if self.transport.is_offline() {
            return;
        }
        if let Some(previous) = self.last_request.get(host) {
            let due = *previous + Duration::from_millis(u64::from(delay_ms));
            let wait = due.saturating_duration_since(Instant::now());
            if !wait.is_zero() {
                std::thread::sleep(wait);
            }
        }
    }

    fn execute(&mut self, host: &str, request: &ebc_model::HttpRequest) -> Result<HttpResponse, String> {
        self.requests += 1;
        let result = self.transport.execute(request);
        self.last_request.insert(host.to_owned(), Instant::now());
        result
    }

    /// The robots.txt verdict for the origin of `url`, fetched once per origin. An
    /// origin whose robots.txt cannot be fetched is closed to us.
    fn robots_for(&mut self, url: &url_like::Parts, delay_ms: u32) -> Robots {
        if let Some(known) = self.robots.get(&url.origin) {
            return known.clone();
        }
        let verdict = match self.policy.robots_request(&url.full) {
            Some(request) => {
                self.pace(&url.host, delay_ms);
                match self.execute(&url.host, &request) {
                    Ok(response) => Robots::from_response(response.status, &response.body),
                    Err(_) => Robots::unreachable(),
                }
            }
            None => Robots::unreachable(),
        };
        self.robots.insert(url.origin.clone(), verdict.clone());
        verdict
    }

    /// Runs one candidate's request under the fetcher rules.
    ///
    /// # Errors
    /// The [`Outcome`] that explains why there is no usable response.
    fn fetch(&mut self, candidate: &CoverCandidate) -> Result<HttpResponse, Outcome> {
        let url = &candidate.request.url;
        let parts = url_like::Parts::of(url)
            .map_err(|refusal| Outcome::Refused(refusal.to_string()))?;
        let robots = self.robots_for(&parts, candidate.min_interval_ms);
        self.policy
            .may_fetch(url, Some(&robots))
            .map_err(|refusal| Outcome::Refused(refusal.to_string()))?;
        self.pace(&parts.host, self.policy.delay_ms(candidate, Some(&robots)));
        self.execute(&parts.host, &candidate.request).map_err(Outcome::Failed)
    }
}

/// The pieces of a guarded URL the fetcher keys its state by.
mod url_like {
    use ebc_covers::guard::{self, Refusal};

    /// Origin and host of a URL that passed the guard.
    pub(super) struct Parts {
        pub(super) full: String,
        pub(super) origin: String,
        pub(super) host: String,
    }

    impl Parts {
        /// Checks `url` with the covers guard and splits it.
        pub(super) fn of(url: &str) -> Result<Self, Refusal> {
            let parsed = guard::check_url(url)?;
            Ok(Self {
                full: url.to_owned(),
                origin: guard::origin_of(&parsed),
                host: parsed.host_str().unwrap_or_default().to_ascii_lowercase(),
            })
        }
    }
}

/// Why a response to an image or fact request is not a cover.
fn describe_miss(candidate: &CoverCandidate, response: &HttpResponse) -> String {
    if !response.is_success() {
        return format!("HTTP {}", response.status);
    }
    if !response.final_url.is_empty() && guard::check_url(&response.final_url).is_err() {
        return "redirected to a host that is not allowed".to_owned();
    }
    if response.content_type().is_some_and(|t| t.starts_with("text/")) {
        return "an error page, not an image".to_owned();
    }
    match probe(&response.body) {
        None if response.body.is_empty() => "no cover (placeholder announced)".to_owned(),
        None => "not an image in a recognised format".to_owned(),
        Some(info) => {
            let assessment = assess_for_source(candidate.source, &info, &response.body);
            format!("a \"no cover\" placeholder, not a cover: {:?}", assessment.reasons)
        }
    }
}

/// Pixel size declared by a fact-only answer (Open Library's cover record).
fn declared_size(response: &HttpResponse) -> Option<(u32, u32)> {
    let value: Value = serde_json::from_slice(&response.body).ok()?;
    let side = |name: &str| value.get(name)?.as_u64().and_then(|n| u32::try_from(n).ok());
    side("width").zip(side("height")).filter(|(w, h)| *w > 0 && *h > 0)
}

/// Everything one run needs besides the candidates.
struct Run<'a> {
    basis: Option<RightsBasis>,
    output: Option<&'a Path>,
    probe: bool,
    min_ppi: f64,
    panel: CardFormat,
}

/// What was saved.
struct Saved {
    info: ImageInfo,
    quality: Quality,
    bytes: usize,
    source: &'static str,
}

/// Walks the cascade top to bottom and returns one outcome per candidate.
fn walk(
    run: &Run<'_>,
    steps: &[CoverCandidate],
    fetcher: &mut Fetcher<'_>,
    streams: &mut Streams<'_>,
) -> Result<(Vec<Outcome>, Option<Saved>), CliError> {
    let mut outcomes = Vec::with_capacity(steps.len());
    let mut saved: Option<Saved> = None;
    // (source, ISBN) pairs a fact-only request showed to have no cover.
    let mut absent: Vec<(&'static str, Option<ebc_model::Isbn13>)> = Vec::new();

    for candidate in steps {
        let key = (candidate.source, candidate.isbn);
        let outcome = match candidate.role {
            CandidateRole::Preview | CandidateRole::Link | CandidateRole::Page => Outcome::Listed,
            _ if saved.is_some() => Outcome::NotRun,
            CandidateRole::Locate | CandidateRole::Probe => {
                if !(run.probe || run.basis.is_some()) {
                    Outcome::NotRun
                } else {
                    match fetcher.fetch(candidate) {
                        Err(outcome) => outcome,
                        Ok(response) if candidate.accepts(&response) => {
                            let detail = match declared_size(&response) {
                                Some((w, h)) => {
                                    let info = ImageInfo::new(Format::Jpeg, w, h);
                                    let quality = print_quality_with(
                                        &info,
                                        run.panel.cover,
                                        &Thresholds::PROFESSIONAL,
                                    );
                                    format!(
                                        "a cover exists; declared {}",
                                        describe_quality(&info, &quality, &run.panel)
                                    )
                                }
                                None => "a cover exists".to_owned(),
                            };
                            Outcome::Hit(detail)
                        }
                        Ok(response) => {
                            absent.push(key);
                            Outcome::Miss(describe_miss(candidate, &response))
                        }
                    }
                }
            }
            CandidateRole::Download => match (run.basis, run.output) {
                (Some(basis), Some(output)) if !absent.contains(&key) => {
                    match fetcher.fetch(candidate) {
                        Err(outcome) => outcome,
                        Ok(response) if !candidate.accepts(&response) => {
                            Outcome::Miss(describe_miss(candidate, &response))
                        }
                        Ok(response) => {
                            let size = response.body.len();
                            match PrintableCover::new(
                                response.body,
                                basis,
                                AssetOrigin::OperatorDownload,
                            ) {
                                Err(error) => Outcome::Miss(error.to_string()),
                                Ok(cover) => {
                                    let info = cover.info();
                                    let quality = print_quality_with(
                                        &info,
                                        run.panel.cover,
                                        &Thresholds::PROFESSIONAL,
                                    );
                                    let described = describe_quality(&info, &quality, &run.panel);
                                    if quality.effective_ppi < run.min_ppi {
                                        Outcome::Miss(format!(
                                            "{described}; below --min-ppi {}",
                                            run.min_ppi
                                        ))
                                    } else {
                                        std::fs::write(output, cover.bytes())
                                            .map_err(|e| CliError::file(output, e))?;
                                        saved = Some(Saved {
                                            info,
                                            quality,
                                            bytes: size,
                                            source: candidate.source,
                                        });
                                        Outcome::Saved(format!(
                                            "{} {described}",
                                            format_name(info.format)
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                (Some(_), Some(_)) => Outcome::Skipped("no cover located at this source".to_owned()),
                _ => Outcome::NotRun,
            },
        };
        if !matches!(outcome, Outcome::Listed | Outcome::NotRun) {
            // Progress as it happens: a polite cascade takes seconds per request.
            streams.note(format!(
                "{} {} {}: {}{}",
                candidate.source,
                role_name(candidate.role),
                redact_url(&candidate.request.url),
                outcome.name(),
                outcome.detail().map(|d| format!(" ({d})")).unwrap_or_default()
            ));
        }
        outcomes.push(outcome);
    }
    Ok((outcomes, saved))
}

fn candidate_json(candidate: &CoverCandidate, outcome: &Outcome) -> Value {
    json!({
        "source": candidate.source,
        "source_name": candidate.source_info().map(|s| s.name),
        "role": role_name(candidate.role),
        "risk": candidate.risk.to_string(),
        "kind": kind_name(candidate.kind),
        "method": candidate.request.method.as_str(),
        "url": redact_url(&candidate.request.url),
        "other_edition": candidate.other_edition,
        "declared_px": candidate.declared_px,
        "min_interval_ms": candidate.min_interval_ms,
        "rights_basis": candidate.rights_basis().as_str(),
        "note": candidate.note,
        "outcome": outcome.name(),
        "detail": outcome.detail(),
    })
}

fn print_candidates(
    steps: &[CoverCandidate],
    outcomes: &[Outcome],
    notes: bool,
    streams: &mut Streams<'_>,
) -> Result<(), CliError> {
    for (index, (candidate, outcome)) in steps.iter().zip(outcomes).enumerate() {
        let what = match candidate.role {
            CandidateRole::Preview => "open in a browser to preview",
            CandidateRole::Link => "link to the image at its source",
            CandidateRole::Page => "page to read",
            CandidateRole::Locate => "fact request: is there a cover, how large",
            CandidateRole::Probe => "existence check (HEAD)",
            CandidateRole::Download => "download (operator)",
        };
        streams.line(format!(
            "{:>2}. {:<12} {:<8} risk {:<6} {}{}",
            index + 1,
            candidate.source,
            role_name(candidate.role),
            candidate.risk.to_string(),
            redact_url(&candidate.request.url),
            if candidate.other_edition { "  [other edition]" } else { "" },
        ))?;
        let result = match outcome {
            Outcome::Listed | Outcome::NotRun => String::new(),
            other => format!(
                " → {}{}",
                other.name(),
                other.detail().map(|d| format!(": {d}")).unwrap_or_default()
            ),
        };
        streams.line(format!("    {what}{result}"))?;
        if notes {
            streams.line(format!("    terms: {}", candidate.note))?;
        }
    }
    Ok(())
}

/// The policy for this run.
fn build_policy(args: &CoverArgs, basis: Option<RightsBasis>) -> Result<Policy, CliError> {
    let mut policy = match basis {
        Some(basis) => Policy::operator(basis),
        None => Policy::default(),
    };
    match (args.max_risk, basis) {
        (Some(RiskArg::Low), None) => policy.max_risk = Risk::Low,
        (Some(RiskArg::Medium), None) | (None, _) | (Some(RiskArg::High), Some(_)) => {}
        // Downloads only exist at risk level high; nothing else changes without a basis.
        (Some(RiskArg::High), None) => policy.max_risk = Risk::High,
        (Some(RiskArg::Low | RiskArg::Medium), Some(_)) => {
            return Err(CliError::usage(
                "--rights-basis downloads an image, which every lookup source rates as risk \
                 high: drop --max-risk or pass --max-risk high",
            ));
        }
    }
    for id in &args.sources {
        let id = id.trim().to_ascii_lowercase();
        if !matches!(id.as_str(), "bnf" | "dnb") {
            return Err(CliError::usage(format!(
                "--source {id:?}: only bnf and dnb can be switched on here"
            )));
        }
        policy = policy.with_source(&id);
    }
    if let Some(key) = args.google_books_key.as_deref().filter(|k| !k.trim().is_empty()) {
        policy.google_books_api_key = Some(key.trim().to_owned());
    }
    if let Some(agent) = &args.net.user_agent {
        policy.user_agent = agent.clone();
    }
    let problems = policy.user_agent_problems();
    if !problems.is_empty() {
        return Err(CliError::usage(format!("--user-agent: {}", problems.join("; "))));
    }
    Ok(policy)
}

/// Runs `ebc cover`.
pub(crate) fn run(args: &CoverArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    let isbn = parse_isbn(&args.isbn)?;
    let panel = format_by_name(&args.format)?;
    if !(args.min_ppi.is_finite() && args.min_ppi >= 0.0) {
        return Err(CliError::usage("--min-ppi must be a positive number"));
    }

    let basis = match &args.rights_basis {
        None => None,
        Some(text) => {
            let basis: RightsBasis =
                text.parse().map_err(|e| CliError::usage(format!("--rights-basis: {e}")))?;
            if !AssetOrigin::OperatorDownload.compatible_bases().contains(&basis) {
                return Err(CliError::usage(format!(
                    "--rights-basis {basis}: a downloaded cover can only be saved under \
                     print_licence, rights_holder_declared or public_domain"
                )));
            }
            Some(basis)
        }
    };
    match (&args.output, basis) {
        (Some(_), None) => {
            return Err(CliError::usage(
                "-o needs --rights-basis <basis>: cover candidates are for locating and \
                 previewing only and carry no right to print, so nothing is saved unless \
                 you state the rights you hold (print_licence, rights_holder_declared or \
                 public_domain). Without a cover the card is typographic.",
            ));
        }
        (None, Some(_)) => {
            return Err(CliError::usage("--rights-basis needs -o FILE: where to save the image"));
        }
        (Some(path), Some(_)) if is_dash(path) => {
            return Err(CliError::usage("-o -: the image is saved to a file, not standard output"));
        }
        _ => {}
    }

    let policy = build_policy(args, basis)?;
    if let Some(basis) = basis {
        streams.note(format!(
            "notice: operator mode: you are responsible for holding the rights \
             (`{basis}`) to any image this command saves."
        ));
    }

    let mut book = BookMeta::titled("");
    book.isbn13 = Some(isbn);
    let steps = candidates(&book, &policy);

    let needs_requests = basis.is_some() || args.probe;
    let (outcomes, saved, requests) = if needs_requests {
        let transport = Transport::open(
            args.net.offline_fixtures.as_deref(),
            NetOptions {
                user_agent: policy.user_agent.clone(),
                timeout: Duration::from_secs(args.net.timeout),
                use_proxy: !args.net.no_proxy,
                min_interval: None,
                max_body_bytes: MAX_IMAGE_BYTES,
            },
        )?;
        let run = Run {
            basis,
            output: args.output.as_deref(),
            probe: args.probe,
            min_ppi: args.min_ppi,
            panel,
        };
        let mut fetcher = Fetcher::new(&transport, &policy);
        let (outcomes, saved) = walk(&run, &steps, &mut fetcher, streams)?;
        (outcomes, saved, fetcher.requests)
    } else {
        let outcomes = steps
            .iter()
            .map(|c| match c.role {
                CandidateRole::Preview | CandidateRole::Link | CandidateRole::Page => Outcome::Listed,
                _ => Outcome::NotRun,
            })
            .collect();
        (outcomes, None, 0)
    };

    if args.json {
        let rows: Vec<Value> =
            steps.iter().zip(&outcomes).map(|(c, o)| candidate_json(c, o)).collect();
        streams.json(&json!({
            "isbn": isbn.to_plain(),
            "mode": if basis.is_some() { "operator" } else { "list" },
            "rights_basis": basis.map_or("none", RightsBasis::as_str),
            "max_risk": policy.max_risk.to_string(),
            "user_agent": policy.user_agent,
            "requests": requests,
            "candidates": rows,
            "saved": saved.as_ref().zip(args.output.as_ref()).map(|(s, path)| json!({
                "file": path.display().to_string(),
                "source": s.source,
                "format": format_name(s.info.format),
                "width_px": s.info.width,
                "height_px": s.info.height,
                "bytes": s.bytes,
                "effective_ppi": (s.quality.effective_ppi * 10.0).round() / 10.0,
                "verdict": verdict_name(s.quality.verdict),
                "upscale_factor": (s.quality.upscale_factor * 100.0).round() / 100.0,
            })),
        }))?;
    } else {
        print_candidates(&steps, &outcomes, args.notes, streams)?;
    }

    if steps.is_empty() {
        streams.note(format!(
            "no cover candidate for ISBN {} under this policy (max risk {})",
            isbn.hyphenated(),
            policy.max_risk
        ));
        return Ok(Status::NotFound);
    }
    let Some(basis) = basis else {
        if !args.json {
            streams.line(
                "Rights basis of every candidate: none. They help you find the cover; none \
                 may be printed as found.",
            )?;
            streams.line(
                "To save an image you hold the rights to: --rights-basis <basis> -o FILE.",
            )?;
        }
        return Ok(Status::Success);
    };
    match (saved, args.output.as_ref()) {
        (Some(saved), Some(path)) => {
            if !args.json {
                streams.line(format!(
                    "saved {}: {} {}, {} bytes, from {}, rights basis {basis}",
                    path.display(),
                    format_name(saved.info.format),
                    describe_quality(&saved.info, &saved.quality, &panel),
                    saved.bytes,
                    saved.source
                ))?;
            }
            if saved.quality.verdict.needs_warning() {
                streams.note(format!(
                    "warning: {:.0} ppi is low for print (300 is the target): it would have \
                     to be enlarged {:.1} times",
                    saved.quality.effective_ppi, saved.quality.upscale_factor
                ));
            }
            if !matches!(saved.info.format, Format::Jpeg | Format::Png | Format::WebP) {
                streams.note("warning: `ebc card` reads JPEG, PNG and WebP covers only");
            }
            if path.extension().and_then(|e| e.to_str()).is_some_and(|ext| {
                let ext = ext.to_ascii_lowercase();
                ext != saved.info.format.extension() && !(ext == "jpeg" && saved.info.format == Format::Jpeg)
            }) {
                streams.note(format!(
                    "note: the file is {}, whatever its name says",
                    saved.info.format.mime()
                ));
            }
            Ok(Status::Success)
        }
        _ => {
            streams.note(format!(
                "no usable cover found for ISBN {}: nothing was saved",
                isbn.hyphenated()
            ));
            Ok(Status::NotFound)
        }
    }
}
