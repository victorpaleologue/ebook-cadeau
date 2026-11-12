//! `ebc isbn`: validate, convert and hyphenate; or extract every ISBN of a text file.

use ebc_isbn::{Isbn13, extract_all, range_data_date};
use serde_json::{Value, json};

use crate::args::IsbnArgs;
use crate::cmd::parse_isbn;
use crate::error::{CliError, Status};
use crate::util::{Streams, read_input};

/// Largest text file `--extract` reads.
const MAX_TEXT_BYTES: u64 = 64 * 1024 * 1024;

fn describe(isbn: &Isbn13) -> Value {
    let isbn10 = isbn.to_isbn10().ok();
    let parts = isbn.parts();
    json!({
        "isbn13": isbn.to_plain(),
        "hyphenated": isbn.hyphenated(),
        "isbn10": isbn10.map(|i| i.to_plain()),
        "isbn10_hyphenated": isbn10.map(|i| i.hyphenated()),
        "gs1_prefix": isbn.gs1_prefix(),
        "group": isbn.registration_group().map(|g| json!({ "prefix": g.prefix, "name": g.name })),
        "registrant": parts.as_ref().map(|p| p.registrant.clone()),
        "publication": parts.as_ref().map(|p| p.publication.clone()),
        "check_digit": isbn.check_digit(),
        "range_data_date": range_data_date(),
    })
}

fn print_one(isbn: &Isbn13, streams: &mut Streams<'_>) -> Result<(), CliError> {
    streams.line(format!("isbn13      {}", isbn.to_plain()))?;
    streams.line(format!("hyphenated  {}", isbn.hyphenated()))?;
    match isbn.to_isbn10() {
        Ok(isbn10) => {
            streams.line(format!("isbn10      {}", isbn10.to_plain()))?;
            streams.line(format!("            {}", isbn10.hyphenated()))?;
        }
        Err(_) => streams.line("isbn10      none (979 prefix)")?,
    }
    match isbn.registration_group() {
        Some(group) => streams.line(format!("group       {} ({})", group.name, group.prefix))?,
        None => streams.line("group       unknown")?,
    }
    match isbn.parts() {
        Some(parts) => {
            streams.line(format!("registrant  {}", parts.registrant))?;
            streams.line(format!("publication {}", parts.publication))?;
        }
        None => streams.note(format!(
            "note: no registrant range known for this ISBN (range table of {}); not hyphenated",
            range_data_date()
        )),
    }
    Ok(())
}

/// Runs `ebc isbn`.
pub(crate) fn run(args: &IsbnArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    if let Some(file) = &args.extract {
        let bytes = read_input(file, MAX_TEXT_BYTES)?;
        let found = extract_all(&String::from_utf8_lossy(&bytes));
        if args.json {
            streams.json(&Value::Array(found.iter().map(describe).collect()))?;
        } else {
            for isbn in &found {
                streams.line(isbn.hyphenated())?;
            }
        }
        if found.is_empty() {
            streams.note(format!("no ISBN found in {}", file.display()));
            return Ok(Status::NotFound);
        }
        return Ok(Status::Success);
    }
    let text = args.text.as_deref().unwrap_or_default();
    let isbn = parse_isbn(text)?;
    if args.json { streams.json(&describe(&isbn))? } else { print_one(&isbn, streams)? }
    Ok(Status::Success)
}
