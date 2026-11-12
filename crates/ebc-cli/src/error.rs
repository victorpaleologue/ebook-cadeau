//! The error type of the command line and the exit statuses it maps to.

use std::path::Path;
use std::process::ExitCode;

/// Exit status: everything went well.
pub(crate) const EXIT_OK: u8 = 0;
/// Exit status: the command could not do its work (I/O, network, refused input).
pub(crate) const EXIT_FAILURE: u8 = 1;
/// Exit status: the check ran and found errors (`ebc preflight`).
pub(crate) const EXIT_CHECK_FAILED: u8 = 2;
/// Exit status: the command worked but found nothing (`ebc lookup`, `ebc cover`,
/// `ebc isbn --extract`, `ebc fontid`).
pub(crate) const EXIT_NOT_FOUND: u8 = 3;
/// Exit status: the command line itself is wrong (BSD `EX_USAGE`). Kept apart from 2 so
/// that a script can tell "preflight found errors" from "I mistyped a flag".
pub(crate) const EXIT_USAGE: u8 = 64;

/// The exit-status table printed at the end of `ebc --help`.
pub(crate) const EXIT_HELP: &str = "\
Exit status:
   0  success
   1  failure (I/O, network, input refused)
   2  `preflight`: the PDF has errors
   3  nothing found (`lookup`, `cover`, `isbn --extract`, `fontid`)
  64  usage error

Results go to standard output, diagnostics and warnings to standard error.";

/// How a command that did not fail ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// The command did what was asked.
    Success,
    /// A check ran to the end and found errors.
    CheckFailed,
    /// The command ran to the end and found nothing.
    NotFound,
}

impl Status {
    /// The process exit status.
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Success => EXIT_OK,
            Self::CheckFailed => EXIT_CHECK_FAILED,
            Self::NotFound => EXIT_NOT_FOUND,
        }
    }
}

/// Why a command stopped.
#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    /// The arguments do not go together or name something unknown.
    #[error("{0}")]
    Usage(String),
    /// The command could not do its work.
    #[error("{0}")]
    Failure(String),
    /// A file could not be read or written.
    #[error("{path}: {source}")]
    File {
        /// The file, as given on the command line.
        path: String,
        /// What the operating system said.
        source: std::io::Error,
    },
    /// Standard output could not be written to.
    #[error("cannot write to standard output: {0}")]
    Stdout(std::io::Error),
}

impl CliError {
    /// A usage error.
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    /// A runtime failure.
    pub(crate) fn failure(message: impl Into<String>) -> Self {
        Self::Failure(message.into())
    }

    /// An I/O error on `path`.
    pub(crate) fn file(path: &Path, source: std::io::Error) -> Self {
        Self::File { path: path.display().to_string(), source }
    }

    /// `true` when standard output was closed by the reader (`ebc … | head`): not an
    /// error worth a message.
    pub(crate) fn is_broken_pipe(&self) -> bool {
        matches!(self, Self::Stdout(error) if error.kind() == std::io::ErrorKind::BrokenPipe)
    }

    /// The process exit status.
    pub(crate) fn code(&self) -> u8 {
        match self {
            Self::Usage(_) => EXIT_USAGE,
            Self::Failure(_) | Self::File { .. } | Self::Stdout(_) => EXIT_FAILURE,
        }
    }
}

/// Converts a status byte into an [`ExitCode`].
pub(crate) fn exit(code: u8) -> ExitCode {
    ExitCode::from(code)
}
