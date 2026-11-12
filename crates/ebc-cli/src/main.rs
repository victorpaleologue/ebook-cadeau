//! `ebc`, the command-line tool of eBook Cadeau: ISBN utilities, metadata lookup, cover
//! candidates, the printable gift card, PDF preflight, QR codes and font identification.
//!
//! Every subcommand is a thin layer over one library crate (`ebc-isbn`, `ebc-catalog`,
//! `ebc-covers`, `ebc-card`, `ebc-preflight`, `ebc-qr`, `ebc-fontid`). Results go to
//! standard output, diagnostics to standard error; see [`error`] for the exit statuses.

#![forbid(unsafe_code)]

mod args;
mod cmd;
mod error;
mod fixtures;
mod transport;
mod util;

use std::io::Write as _;
use std::process::ExitCode;

use clap::Parser as _;
use clap::error::ErrorKind;

use crate::args::{Cli, Command};
use crate::error::{EXIT_OK, EXIT_USAGE, exit};
use crate::util::Streams;

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            // `--help` and `--version` are answers, not errors.
            let informational =
                matches!(error.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion);
            let _ = error.print();
            return exit(if informational { EXIT_OK } else { EXIT_USAGE });
        }
    };

    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut err = stderr.lock();
    let result = {
        let mut streams = Streams { out: &mut out, err: &mut err };
        match &cli.command {
            Command::Isbn(args) => cmd::isbn::run(args, &mut streams),
            Command::Lookup(args) => cmd::lookup::run(args, &mut streams),
            Command::Cover(args) => cmd::cover::run(args, &mut streams),
            Command::Card(args) => cmd::card::run(args, &mut streams),
            Command::Preflight(args) => cmd::preflight::run(args, &mut streams),
            Command::Qr(args) => cmd::qr::run(args, &mut streams),
            #[cfg(feature = "fontid")]
            Command::Fontid(args) => cmd::fontid::run(args, &mut streams),
        }
    };
    let result = result.and_then(|status| {
        out.flush().map_err(error::CliError::Stdout)?;
        Ok(status)
    });
    match result {
        Ok(status) => exit(status.code()),
        Err(error) if error.is_broken_pipe() => exit(EXIT_OK),
        Err(error) => {
            let _ = writeln!(err, "ebc: {error}");
            exit(error.code())
        }
    }
}
