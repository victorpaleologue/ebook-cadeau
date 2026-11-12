//! One module per subcommand.

pub(crate) mod card;
pub(crate) mod cover;
#[cfg(feature = "fontid")]
pub(crate) mod fontid;
pub(crate) mod isbn;
pub(crate) mod lookup;
pub(crate) mod preflight;
pub(crate) mod qr;

use ebc_model::Isbn13;

use crate::error::CliError;

/// Parses an ISBN given on the command line.
pub(crate) fn parse_isbn(text: &str) -> Result<Isbn13, CliError> {
    Isbn13::parse(text).map_err(|e| CliError::failure(format!("{text:?} is not a valid ISBN: {e}")))
}
