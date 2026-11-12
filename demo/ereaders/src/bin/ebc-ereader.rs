//! `ebc-ereader`: drive a simulated e-reader from the command line.
//!
//! ```text
//! ebc-ereader --profile kobo open http://127.0.0.1:8080/ --code BCDF-GHJK-LMNP
//! ebc-ereader --profile pocketbook open http://127.0.0.1:8080/e --pair BCD-FGH
//! ebc-ereader --profile kindle open http://127.0.0.1:8080/k/BCDFGHJKLMNP --device-email me@kindle.com
//! ebc-ereader profiles --claims
//! ebc-ereader lint http://127.0.0.1:8080/
//! ebc-ereader --profile pocketbook cost "bcdf-ghjk-lmnp"
//! ```
//!
//! Exit status: 0 when the gift reached the device (or the lint passed), 1 when the flow
//! stopped short (or the lint found errors), 2 on usage or transport errors.

#![forbid(unsafe_code)]

use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use ebc_demo_ereaders::flow::{self, FlowEnd, RedeemInputs};
use ebc_demo_ereaders::profile::Basis;
use ebc_demo_ereaders::typing::{InputMode, TypingCost};
use ebc_demo_ereaders::{BrowserConfig, DeviceProfile, EinkBrowser, LintConfig, Outcome};

#[derive(Parser)]
#[command(
    name = "ebc-ereader",
    version,
    about = "Simulated e-readers for eBook Cadeau demos (no JavaScript, profile User-Agent, tap counting)"
)]
struct Cli {
    /// Device profile: kobo, kindle, pocketbook, vivlio, tolino, generic, phone, computer, or any id listed by `profiles`.
    #[arg(long, short, global = true, default_value = "kobo")]
    profile: String,
    /// Seconds allowed per HTTP exchange.
    #[arg(long, global = true, default_value_t = 30)]
    timeout: u64,
    /// Use HTTP(S)_PROXY from the environment.
    #[arg(long, global = true)]
    env_proxy: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Type a URL on the device and walk the redemption flow, printing each step.
    Open {
        /// The address typed on the device (claim URL, short URL or the service's home page).
        url: String,
        /// The gift code printed on the card (XXXX-XXXX-XXXX), typed into the home page's form.
        #[arg(long)]
        code: Option<String>,
        /// The pair code shown on the phone (XXX-XXX), typed on the /e page.
        #[arg(long)]
        pair: Option<String>,
        /// The device's e-mail address for the send-to-device form.
        #[arg(long)]
        device_email: Option<String>,
    },
    /// List the built-in device profiles and what they can do.
    Profiles {
        /// Also print the evidence behind every capability.
        #[arg(long)]
        claims: bool,
    },
    /// Fetch one page as the device and report what would break on old e-reader browsers.
    Lint {
        /// The page to check.
        url: String,
    },
    /// Count the taps needed to type a text on the device's on-screen keyboard.
    Cost {
        /// The text (or URL with --url).
        text: String,
        /// Treat the text as a URL typed into the address bar.
        #[arg(long)]
        url: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = std::io::stdout().lock();
    let result = match run(&cli, &mut out) {
        Ok(code) => code,
        Err(message) => {
            let _ = writeln!(std::io::stderr(), "ebc-ereader: {message}");
            ExitCode::from(2)
        }
    };
    let _ = out.flush();
    result
}

fn run(cli: &Cli, out: &mut impl Write) -> Result<ExitCode, String> {
    let io = |e: std::io::Error| e.to_string();
    let profile = DeviceProfile::by_name(&cli.profile).ok_or_else(|| {
        let known: Vec<&str> = DeviceProfile::all().iter().map(|p| p.id).collect();
        format!("unknown profile {:?}; known profiles: {}", cli.profile, known.join(", "))
    })?;
    let mut config = BrowserConfig::default();
    config.net.request_timeout = Duration::from_secs(cli.timeout.clamp(1, 3600));
    config.net.use_env_proxy = cli.env_proxy;

    match &cli.command {
        Command::Profiles { claims } => {
            for profile in DeviceProfile::all() {
                writeln!(out, "{:<24} {}", profile.id, profile.name).map_err(io)?;
                writeln!(out, "    user-agent : {}", profile.user_agent).map_err(io)?;
                writeln!(
                    out,
                    "    browser    : downloads {} | cookies {} | page limit {} kB",
                    list(profile.browser_downloads),
                    if profile.cookies { "on" } else { "off" },
                    profile.max_page_bytes / 1024
                )
                .map_err(io)?;
                writeln!(out, "    reads      : {}", list(profile.reading_formats)).map_err(io)?;
                let mail = profile.mail_service.map_or("none".to_owned(), |s| {
                    format!("{} (@{}; accepts {})", s.name, s.domains.join(", @"), list(s.accepted))
                });
                writeln!(out, "    mail       : {mail}").map_err(io)?;
                if *claims {
                    for claim in profile.claims {
                        let basis = match claim.basis {
                            Basis::Research { section, grade } => {
                                format!("research: {section} [{grade:?}]")
                            }
                            Basis::Assumption(why) => format!("ASSUMPTION: {why}"),
                        };
                        writeln!(
                            out,
                            "    - {:?}: {} — {basis}",
                            claim.capability, claim.statement
                        )
                        .map_err(io)?;
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Cost { text, url } => {
            let cost = if *url {
                TypingCost::of_url(text, &profile.keyboard)
            } else {
                TypingCost::of(text, &profile.keyboard, InputMode::Text)
            };
            writeln!(
                out,
                "{} taps on the {} keyboard of {} ({} characters, {} layer switches, {} shift taps, {} long presses)",
                cost.taps, profile.keyboard.name, profile.id, cost.characters, cost.layer_switches,
                cost.shift_taps, cost.long_presses
            )
            .map_err(io)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Lint { url } => {
            let runtime = runtime()?;
            let mut browser =
                EinkBrowser::with_config(profile, config).map_err(|e| e.to_string())?;
            let outcome = runtime.block_on(browser.open(url)).map_err(|e| e.to_string())?;
            if !matches!(outcome, Outcome::Page) {
                return Err("the URL is a download, not a page".to_owned());
            }
            let page = browser.page().map_err(|e| e.to_string())?;
            let report = page.lint(&LintConfig::for_profile(browser.profile()));
            writeln!(
                out,
                "{} — HTTP {}, {} bytes, lang {}",
                page.url(),
                page.status(),
                page.weight_bytes(),
                page.lang().unwrap_or("(none)")
            )
            .map_err(io)?;
            write!(out, "{report}").map_err(io)?;
            if report.findings.is_empty() {
                writeln!(out).map_err(io)?;
            }
            Ok(if report.passed() { ExitCode::SUCCESS } else { ExitCode::from(1) })
        }
        Command::Open { url, code, pair, device_email } => {
            let runtime = runtime()?;
            let mut browser =
                EinkBrowser::with_config(profile, config).map_err(|e| e.to_string())?;
            writeln!(out, "device : {} ({})", browser.profile().name, browser.profile().id)
                .map_err(io)?;
            let inputs = RedeemInputs {
                code: code.clone(),
                pair_code: pair.clone(),
                device_email: device_email.clone(),
            };
            let report = runtime
                .block_on(flow::redeem(&mut browser, url, &inputs))
                .map_err(|e| e.to_string())?;
            writeln!(out, "\nsteps").map_err(io)?;
            for (index, step) in report.steps.iter().enumerate() {
                writeln!(out, "  {}. {}", index + 1, step.action).map_err(io)?;
                writeln!(
                    out,
                    "     -> HTTP {} {}  [{} taps so far]",
                    step.status, step.url, step.taps_total
                )
                .map_err(io)?;
                if !step.lint_errors.is_empty() {
                    writeln!(out, "     lint errors: {}", step.lint_errors.join(", "))
                        .map_err(io)?;
                }
            }
            writeln!(out, "\ntaps").map_err(io)?;
            for tap in browser.taps().steps() {
                writeln!(out, "  {:>3}  {}", tap.taps, tap.action).map_err(io)?;
            }
            writeln!(out, "  {:>3}  total", report.taps).map_err(io)?;
            writeln!(out, "\nresult").map_err(io)?;
            match &report.end {
                FlowEnd::Downloaded(entry) => {
                    writeln!(
                        out,
                        "  downloaded \"{}\" ({}, {} bytes)",
                        entry.title, entry.format, entry.size_bytes
                    )
                    .map_err(io)?;
                }
                FlowEnd::Licence(file) => {
                    writeln!(
                        out,
                        "  licence file {} ({}) handed to the reading system",
                        file.file_name, file.format
                    )
                    .map_err(io)?;
                }
                FlowEnd::Sent { message } => writeln!(out, "  sent: {message}").map_err(io)?,
                FlowEnd::LeftSite { url } => {
                    writeln!(out, "  the flow continues on another site: {url}").map_err(io)?;
                }
                FlowEnd::Stopped { reason } => writeln!(out, "  stopped: {reason}").map_err(io)?,
            }
            writeln!(out, "\nlibrary ({} book(s))", browser.library().len()).map_err(io)?;
            for entry in browser.library().entries() {
                writeln!(
                    out,
                    "  - \"{}\" by {} [{}] {}",
                    entry.title,
                    if entry.authors.is_empty() {
                        "unknown".to_owned()
                    } else {
                        entry.authors.join(", ")
                    },
                    entry.format,
                    entry.file_name
                )
                .map_err(io)?;
                for note in &entry.notes {
                    writeln!(out, "      note: {note}").map_err(io)?;
                }
            }
            Ok(if report.end.succeeded() { ExitCode::SUCCESS } else { ExitCode::from(1) })
        }
    }
}

fn list(formats: &[ebc_demo_ereaders::FileFormat]) -> String {
    if formats.is_empty() {
        "nothing".to_owned()
    } else {
        formats.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
    }
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("cannot start the async runtime: {e}"))
}
