//! Small helpers shared by the subcommands: output streams, file access, and the names
//! of formats, profiles and locales.

use std::io::{Read, Write};
use std::path::Path;

use ebc_layout::{CardFormat, PrintProfile};
use ebc_model::Locale;
use ebc_scene::Size;

use crate::error::CliError;

/// Where a command writes: results to `out`, diagnostics to `err`.
pub(crate) struct Streams<'a> {
    /// Standard output: results only.
    pub(crate) out: &'a mut dyn Write,
    /// Standard error: notices, warnings, progress.
    pub(crate) err: &'a mut dyn Write,
}

impl Streams<'_> {
    /// Writes one line of result.
    pub(crate) fn line(&mut self, text: impl AsRef<str>) -> Result<(), CliError> {
        writeln!(self.out, "{}", text.as_ref()).map_err(CliError::Stdout)
    }

    /// Writes one diagnostic line. A closed standard error is not worth failing for.
    pub(crate) fn note(&mut self, text: impl AsRef<str>) {
        let _ = writeln!(self.err, "{}", text.as_ref());
    }

    /// Writes a value as pretty-printed JSON followed by a newline.
    pub(crate) fn json(&mut self, value: &serde_json::Value) -> Result<(), CliError> {
        let text = serde_json::to_string_pretty(value)
            .map_err(|e| CliError::failure(format!("cannot serialise the result: {e}")))?;
        self.line(text)
    }
}

/// `true` when `path` is `-`, the conventional name of a standard stream.
pub(crate) fn is_dash(path: &Path) -> bool {
    path.as_os_str() == "-"
}

/// Reads a whole file; `-` reads standard input. `limit` caps the size in bytes.
pub(crate) fn read_input(path: &Path, limit: u64) -> Result<Vec<u8>, CliError> {
    let mut bytes = Vec::new();
    let read = if is_dash(path) {
        std::io::stdin().lock().take(limit + 1).read_to_end(&mut bytes)
    } else {
        std::fs::File::open(path).and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
    };
    read.map_err(|e| CliError::file(path, e))?;
    if bytes.len() as u64 > limit {
        return Err(CliError::failure(format!(
            "{}: larger than the {} MiB this command accepts",
            path.display(),
            limit / (1024 * 1024)
        )));
    }
    Ok(bytes)
}

/// Writes a result file; `-` writes to standard output.
pub(crate) fn write_output(
    path: &Path,
    bytes: &[u8],
    streams: &mut Streams<'_>,
) -> Result<(), CliError> {
    if is_dash(path) {
        streams.out.write_all(bytes).and_then(|()| streams.out.flush()).map_err(CliError::Stdout)
    } else {
        std::fs::write(path, bytes).map_err(|e| CliError::file(path, e))
    }
}

/// Short names this tool adds to the profile names of `ebc-layout`.
const PROFILE_ALIASES: [(&str, &str); 3] =
    [("pro", "professional_eu"), ("professional", "professional_eu"), ("upload", "upload_eu")];

/// The print profile called `name`: `pro` (marks and slug), `upload` (bleed only, for
/// web-to-print portals), `home-a4`, `home-letter`, `home-a3`, `home-sra3`, or any name
/// `ebc-layout` knows.
pub(crate) fn profile_by_name(name: &str) -> Result<PrintProfile, CliError> {
    let wanted = name.trim().to_ascii_lowercase().replace('-', "_");
    let wanted = PROFILE_ALIASES
        .iter()
        .find(|(alias, _)| *alias == wanted)
        .map_or(wanted.as_str(), |(_, full)| *full);
    PrintProfile::by_name(wanted).ok_or_else(|| {
        CliError::usage(format!(
            "unknown profile {name:?}: use pro, upload, home-a4, home-letter, home-a3 or home-sra3"
        ))
    })
}

/// The card format called `name`: a preset (`poche`, `a6`, `digest`, `a5`) or a custom
/// size in millimetres, `WIDTHxHEIGHT` or `WIDTHxHEIGHTxSPINE` (one cover panel).
pub(crate) fn format_by_name(name: &str) -> Result<CardFormat, CliError> {
    if let Some(format) = CardFormat::by_name(name) {
        return Ok(format);
    }
    let unknown = || {
        CliError::usage(format!(
            "unknown format {name:?}: use {} or WIDTHxHEIGHT[xSPINE] in millimetres",
            CardFormat::PRESET_NAMES.join(", ")
        ))
    };
    let numbers: Vec<f64> = name
        .trim()
        .split(['x', 'X', '×'])
        .map(|part| part.trim().parse::<f64>())
        .collect::<Result<_, _>>()
        .map_err(|_| unknown())?;
    let (cover, spine) = match numbers.as_slice() {
        [w, h] => (Size::new(*w, *h), CardFormat::SPINE_DEFAULT_MM),
        [w, h, spine] => (Size::new(*w, *h), *spine),
        _ => return Err(unknown()),
    };
    CardFormat::custom(cover, spine).map_err(|e| CliError::usage(format!("format {name:?}: {e}")))
}

/// The locale for a language tag (`fr`, `en-GB`, …).
pub(crate) fn locale_by_tag(tag: &str) -> Result<Locale, CliError> {
    Locale::from_tag(tag).ok_or_else(|| {
        let known: Vec<&str> = Locale::ALL.iter().map(|l| l.code()).collect();
        CliError::usage(format!("unsupported locale {tag:?}: use one of {}", known.join(", ")))
    })
}

/// Replaces the value of secret-bearing query parameters (`key`, `api_key`, `token`,
/// `apikey`, `access_token`) in a URL, for display.
pub(crate) fn redact_url(url: &str) -> String {
    let Some((head, query)) = url.split_once('?') else { return url.to_owned() };
    let (query, fragment) = match query.split_once('#') {
        Some((q, f)) => (q, Some(f)),
        None => (query, None),
    };
    let cleaned: Vec<String> = query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((name, _))
                if ["key", "api_key", "apikey", "token", "access_token"]
                    .contains(&name.to_ascii_lowercase().as_str()) =>
            {
                format!("{name}=REDACTED")
            }
            _ => pair.to_owned(),
        })
        .collect();
    let mut out = format!("{head}?{}", cleaned.join("&"));
    if let Some(fragment) = fragment {
        out.push('#');
        out.push_str(fragment);
    }
    out
}
