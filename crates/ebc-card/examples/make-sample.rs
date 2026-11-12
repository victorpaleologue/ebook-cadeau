//! Writes one sample card from the command line.
//!
//! ```sh
//! cargo run -p ebc-card --example make-sample -- -o card.pdf
//! cargo run -p ebc-card --example make-sample -- --profile home_a4 --locale en \
//!     --generated-cover --rights rights_holder_declared --giver Alice --recipient Bob -o card.pdf
//! cargo run -p ebc-card --example make-sample -- --cover my-cover.jpg --rights public_domain \
//!     --title "Candide" --author Voltaire --svg previews/ -o candide.pdf
//! ```
//!
//! Options: `--format poche|a6|digest|a5`, `--profile professional_eu|upload_eu|home_a4|home_letter|…`,
//! `--locale fr|en|de|es|it|nl|pt`, `--title`, `--subtitle`, `--author`, `--publisher`,
//! `--giver`, `--recipient`, `--message`, `--store`, `--code XXXX-XXXX-XXXX`, `--base-url`,
//! `--cover FILE` or `--generated-cover` (an own-made image), `--rights BASIS`,
//! `--credit TEXT`, `--trim-authorised`, `--spine typeset|cover_colour|edge_continuation`,
//! `--adaptation-authorised`, `--guides`, `--svg DIR`, `-o FILE`.
//!
//! A cover is embedded only with an explicit `--rights` basis other than `none`; without
//! it the typographic card is produced. The demo book, names and address are invented.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use ebc_card::{
    CardRequest, CoverInput, RightsBasis, SpineChoice, bundled_fonts, preview_svg, render_pdf,
};
use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::{BookMeta, Contributor, GiftCode, Locale};

#[allow(dead_code, unreachable_pub)]
#[path = "../tests/common/art.rs"]
mod art;

/// 2026-11-06 12:00:00 UTC: a fixed creation time, so that samples are reproducible.
const CREATED_UNIX: i64 = 1_794_000_000;

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut book = BookMeta::titled("Les Vagues du soir");
    book.subtitle = Some("Roman".to_owned());
    book.publisher = Some("Éditions du Lac (fictif)".to_owned());
    book.language = Some("fr".to_owned());
    let mut author = Some("Jeanne Roy".to_owned());
    let mut code = GiftCode::from_payload(123_456_789_012).map_err(|e| e.to_string())?;
    let mut base_url = "https://ebk.example".to_owned();
    let mut format = CardFormat::poche();
    let mut profile = PrintProfile::upload_eu();
    let mut locale = Locale::Fr;
    let mut cover_file: Option<PathBuf> = None;
    let mut generated = false;
    let mut rights: Option<RightsBasis> = None;
    let mut out: Option<PathBuf> = None;
    let mut svg_dir: Option<PathBuf> = None;
    let mut later: Vec<Box<dyn FnOnce(&mut CardRequest)>> = Vec::new();

    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "-o" | "--output" => out = Some(PathBuf::from(value("-o")?)),
            "--svg" => svg_dir = Some(PathBuf::from(value("--svg")?)),
            "--format" => {
                let name = value("--format")?;
                format = CardFormat::by_name(&name).ok_or(format!("unknown format {name:?}"))?;
            }
            "--profile" => {
                let name = value("--profile")?;
                profile = PrintProfile::by_name(&name).ok_or(format!("unknown profile {name:?}"))?;
            }
            "--locale" => {
                let tag = value("--locale")?;
                locale = Locale::from_tag(&tag).ok_or(format!("unsupported locale {tag:?}"))?;
            }
            "--title" => book.title = value("--title")?,
            "--subtitle" => book.subtitle = Some(value("--subtitle")?),
            "--author" => author = Some(value("--author")?),
            "--publisher" => book.publisher = Some(value("--publisher")?),
            "--code" => {
                code = GiftCode::parse(&value("--code")?).map_err(|e| e.to_string())?;
            }
            "--base-url" => base_url = value("--base-url")?,
            "--cover" => cover_file = Some(PathBuf::from(value("--cover")?)),
            "--generated-cover" => generated = true,
            "--rights" => rights = Some(value("--rights")?.parse().map_err(|e| format!("{e}"))?),
            "--credit" => {
                let v = value("--credit")?;
                later.push(Box::new(move |r| r.cover_credit = Some(v)));
            }
            "--giver" => {
                let v = value("--giver")?;
                later.push(Box::new(move |r| r.giver = Some(v)));
            }
            "--recipient" => {
                let v = value("--recipient")?;
                later.push(Box::new(move |r| r.recipient = Some(v)));
            }
            "--message" => {
                let v = value("--message")?;
                later.push(Box::new(move |r| r.message = Some(v)));
            }
            "--store" => {
                let v = value("--store")?;
                later.push(Box::new(move |r| r.store_name = Some(v)));
            }
            "--spine" => {
                let v = value("--spine")?;
                let choice = match v.as_str() {
                    "typeset" => SpineChoice::Typeset,
                    "cover_colour" => SpineChoice::CoverColour,
                    "edge_continuation" => SpineChoice::EdgeContinuation,
                    other => return Err(format!("unknown spine style {other:?}")),
                };
                later.push(Box::new(move |r| r.spine = choice));
            }
            "--trim-authorised" => later.push(Box::new(|r| r.trim_authorised = true)),
            "--adaptation-authorised" => later.push(Box::new(|r| r.adaptation_authorised = true)),
            "--guides" => later.push(Box::new(|r| r.guides = true)),
            "-h" | "--help" => {
                return Err("see the documentation at the top of examples/make-sample.rs".to_owned());
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    let out = out.ok_or("give the output file with -o FILE")?;
    if let Some(name) = author {
        book.contributors.push(Contributor::author(name));
    }

    let mut request = CardRequest::new(book, code, &base_url);
    request.format = format;
    request.profile = profile;
    request.locale = locale;
    request.created_unix = Some(CREATED_UNIX);
    for set in later {
        set(&mut request);
    }

    let mut err = std::io::stderr();
    let wants_cover = cover_file.is_some() || generated;
    match (wants_cover, rights) {
        (true, Some(rights)) if rights != RightsBasis::None => {
            request.cover = match cover_file {
                Some(path) => CoverInput::Encoded {
                    bytes: std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
                    rights,
                },
                None => CoverInput::Pixels { pixmap: art::cover(1240, 2008, false, true), rights },
            };
            let _ = writeln!(
                err,
                "note: the cover is embedded under rights basis `{rights}`; you are responsible for holding these rights."
            );
        }
        (true, _) => {
            let _ = writeln!(
                err,
                "note: no --rights basis given: the cover is not used, the typographic card is produced."
            );
        }
        (false, _) => {}
    }

    let fonts = bundled_fonts().map_err(|e| e.to_string())?;
    let (pdf, warnings) = render_pdf(&request, &fonts).map_err(|e| e.to_string())?;
    std::fs::write(&out, &pdf).map_err(|e| format!("{}: {e}", out.display()))?;
    for warning in &warnings {
        let _ = writeln!(err, "warning: {warning}");
    }
    if let Some(dir) = svg_dir {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let pages = preview_svg(&request, &fonts).map_err(|e| e.to_string())?;
        for (svg, name) in pages.iter().zip(["outside.svg", "inside.svg"]) {
            let path = dir.join(name);
            std::fs::write(&path, svg).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    let _ = writeln!(std::io::stdout(), "{} ({} bytes)", out.display(), pdf.len());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(std::io::stderr(), "make-sample: {message}");
            ExitCode::FAILURE
        }
    }
}
