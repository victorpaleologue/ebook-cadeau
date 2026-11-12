//! `ebc fontid`: placeholder until the subcommand is written.

use crate::args::FontidArgs;
use crate::error::{CliError, Status};
use crate::util::Streams;

/// Runs `ebc fontid`.
pub(crate) fn run(_args: &FontidArgs, _streams: &mut Streams<'_>) -> Result<Status, CliError> {
    Err(CliError::failure("not written yet"))
}
