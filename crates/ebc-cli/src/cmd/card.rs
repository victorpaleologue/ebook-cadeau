//! `ebc card`: compose the gift card as a print-ready PDF.

use std::time::{SystemTime, UNIX_EPOCH};

use ebc_card::rights::route_of;
use ebc_card::{
    CardRequest, CoverInput, MAX_COVER_BYTES, OutputProfile, PrintRoute, RightsBasis,
    bundled_fonts, preview_svg, render_pdf,
};
use ebc_model::{BookMeta, Contributor, GiftCode};
use ebc_scene::PdfStandard;
use serde_json::json;

use crate::args::CardArgs;
use crate::cmd::parse_isbn;
use crate::error::{CliError, Status};
use crate::util::{
    Streams, format_by_name, is_dash, locale_by_tag, profile_by_name, read_input, write_output,
};

/// Largest ICC profile read from `--icc`.
const MAX_ICC_BYTES: u64 = 32 * 1024 * 1024;

/// What `--cover FILE` without `--cover-rights` is answered with.
const NO_RIGHTS_BASIS: &str = "\
--cover needs --cover-rights <basis>: a cover image is only printed with a rights basis.
  print_licence           per-title print licence from the publisher (any profile)
  rights_holder_declared  you are the author or publisher, or hold a licence (any profile)
  public_domain           the artwork is in the public domain (any profile)
  promo_all_media         reseller declaration; home profiles only, reduced size
  user_private_copy       your own copy, your own printer; home profiles only
Without one the card is typographic (title and author as text on a flat ground): pass
--typographic to compose that card and ignore the image.";

/// The gift code: the one given, or a fresh one from the system's secure random source.
fn gift_code(args: &CardArgs) -> Result<GiftCode, CliError> {
    if let Some(text) = &args.code {
        // The rejected text is not echoed: a mistyped code is still most of a secret.
        return GiftCode::parse(text).map_err(|e| CliError::failure(format!("--code: {e}")));
    }
    let mut random = [0u8; 32];
    getrandom::fill(&mut random)
        .map_err(|e| CliError::failure(format!("no secure random source: {e}")))?;
    GiftCode::from_random_bytes(&random).map_err(|e| CliError::failure(e.to_string()))
}

/// Creation time: `--created-unix`, else `SOURCE_DATE_EPOCH`, else the clock.
fn created_unix(args: &CardArgs) -> Result<i64, CliError> {
    if let Some(seconds) = args.created_unix {
        return Ok(seconds);
    }
    if let Ok(text) = std::env::var("SOURCE_DATE_EPOCH") {
        return text.trim().parse::<i64>().map_err(|_| {
            CliError::failure("SOURCE_DATE_EPOCH is set but is not a number of seconds")
        });
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    Ok(i64::try_from(now).unwrap_or(i64::MAX))
}

/// The cover and its rights basis, or none. `Err` when an image is offered without a
/// basis and without `--typographic`.
fn cover_input(args: &CardArgs, streams: &mut Streams<'_>) -> Result<CoverInput, CliError> {
    let Some(path) = &args.cover else { return Ok(CoverInput::None) };
    if args.typographic {
        streams.note(format!(
            "note: --typographic: {} is not read; the card carries no cover image",
            path.display()
        ));
        return Ok(CoverInput::None);
    }
    let Some(basis) = &args.cover_rights else {
        return Err(CliError::usage(NO_RIGHTS_BASIS));
    };
    let rights: RightsBasis =
        basis.parse().map_err(|e| CliError::usage(format!("--cover-rights: {e}")))?;
    let bytes = read_input(path, MAX_COVER_BYTES as u64)?;
    let Some(info) = ebc_covers::probe(&bytes) else {
        return Err(CliError::failure(format!(
            "{}: not an image in a recognised format",
            path.display()
        )));
    };
    let assessment = ebc_covers::placeholder::assess(&info, &bytes);
    if assessment.is_placeholder() {
        return Err(CliError::failure(format!(
            "{}: this looks like a \"no cover available\" placeholder, not a cover ({:?})",
            path.display(),
            assessment.reasons
        )));
    }
    Ok(CoverInput::Encoded { bytes, rights })
}

/// Runs `ebc card`.
pub(crate) fn run(args: &CardArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    let profile = profile_by_name(&args.profile)?;
    let format = format_by_name(&args.format)?;
    let locale = locale_by_tag(&args.locale)?;

    let mut book = BookMeta::titled(args.title.trim());
    book.subtitle = args.subtitle.clone();
    book.contributors = args
        .authors
        .iter()
        .filter(|name| !name.trim().is_empty())
        .map(|name| Contributor::author(name.trim()))
        .collect();
    book.publisher = args.publisher.clone();
    book.language = args.book_language.clone();
    if let Some(isbn) = &args.isbn {
        book.isbn13 = Some(parse_isbn(isbn)?);
    }

    let cover = cover_input(args, streams)?;
    let rights = cover.rights();
    let has_cover = cover.is_printable();
    let code = gift_code(args)?;

    let mut request = CardRequest::new(book, code, &args.base_url);
    request.cover = cover;
    request.cover_credit = args.cover_credit.clone();
    request.trim_authorised = args.trim_authorised;
    request.giver = args.giver.clone();
    request.recipient = args.recipient.clone();
    request.message = args.message.clone();
    request.store_name = args.store.clone();
    request.locale = locale;
    request.format = format;
    request.guides = args.guides;
    request.created_unix = Some(created_unix(args)?);
    if let (Some(path), Some(condition)) = (&args.icc, &args.output_condition) {
        if profile.standard == PdfStandard::Plain {
            streams.note(format!(
                "note: profile {} writes a plain PDF without an output intent: --icc is not used",
                profile.name
            ));
        }
        let icc = read_input(path, MAX_ICC_BYTES)?;
        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        request.output_profile = Some(
            OutputProfile::new(icc, condition, &name)
                .map_err(|e| CliError::failure(format!("{}: {e}", path.display())))?,
        );
    }
    let route = match route_of(&profile) {
        PrintRoute::HomePrint => "home print",
        PrintRoute::ProfessionalOrder => "professional print",
    };
    request.profile = profile;

    let fonts = bundled_fonts().map_err(|e| CliError::failure(e.to_string()))?;
    let (pdf, warnings) = render_pdf(&request, &fonts)
        .map_err(|e| CliError::failure(format!("the card cannot be composed: {e}")))?;

    let mut previews = Vec::new();
    if let Some(dir) = &args.preview_svg {
        let pages = preview_svg(&request, &fonts)
            .map_err(|e| CliError::failure(format!("the previews cannot be drawn: {e}")))?;
        std::fs::create_dir_all(dir).map_err(|e| CliError::file(dir, e))?;
        for (name, svg) in ["outside.svg", "inside.svg"].iter().zip(&pages) {
            let path = dir.join(name);
            std::fs::write(&path, svg).map_err(|e| CliError::file(&path, e))?;
            previews.push(path.display().to_string());
        }
    }

    write_output(&args.output, &pdf, streams)?;
    for warning in &warnings {
        streams.note(format!("warning: {warning}"));
    }

    // With `-o -` the PDF itself is on standard output: the summary goes to standard
    // error so that the stream stays a valid PDF.
    let to_stdout = is_dash(&args.output);
    let kind = if has_cover { "cover" } else { "typographic" };
    let summary = json!({
        "output": if to_stdout { "-".to_owned() } else { args.output.display().to_string() },
        "bytes": pdf.len(),
        "pages": 2,
        "profile": request.profile.name,
        "route": route,
        "card": kind,
        "cover_rights": rights.as_str(),
        "locale": locale.code(),
        "previews": previews,
        "warnings": warnings.iter().map(ToString::to_string).collect::<Vec<_>>(),
        // The code is a bearer secret: it is only reported when this run created it.
        "code": args.random_code.then(|| request.code.grouped()),
    });
    if args.json {
        if to_stdout {
            streams.note(summary.to_string());
        } else {
            streams.json(&summary)?;
        }
        return Ok(Status::Success);
    }
    let lines = [
        Some(format!(
            "wrote {}: {} bytes, 2 pages, {kind} card, profile {} ({route})",
            if to_stdout {
                "standard output".to_owned()
            } else {
                args.output.display().to_string()
            },
            pdf.len(),
            request.profile.name,
        )),
        args.random_code.then(|| format!("code: {}", request.code.grouped())),
        (!previews.is_empty()).then(|| format!("previews: {}", previews.join(", "))),
    ];
    for line in lines.into_iter().flatten() {
        if to_stdout { streams.note(line) } else { streams.line(line)? }
    }
    Ok(Status::Success)
}
