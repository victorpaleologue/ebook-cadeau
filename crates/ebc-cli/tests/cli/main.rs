//! End-to-end tests of the `ebc` binary: every subcommand's happy path and failure
//! paths, offline. One test executable for the whole crate.

#![allow(clippy::unwrap_used)]

mod card;
mod common;
mod cover;
#[cfg(feature = "fontid")]
mod fontid;
mod general;
mod isbn;
mod lookup;
mod preflight;
mod qr;
