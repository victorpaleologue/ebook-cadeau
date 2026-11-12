//! Demo: "Imprimerie du Dos Carré (démo)", a fictional print-on-demand API that preflights
//! and accepts eBook Cadeau card PDFs.
//!
//! It implements the print provider interface of `docs/api.md` §5:
//!
//! | Route | Purpose |
//! |---|---|
//! | `GET /v1/products` | the catalogue ([`catalog`]) |
//! | `POST /v1/orders` | multipart `order` (JSON) + `file` (PDF); the PDF is preflighted against the product with the `ebc-preflight` crate ([`preflight`]): `201` or `422 preflight_failed` with findings |
//! | `GET /v1/orders/{id}` | status and tracking |
//! | `POST /v1/orders/{id}/cancel` | cancel while still `accepted` (extension) |
//! | `GET /__test/orders`, `POST /__test/advance` | test hooks, only when enabled |
//!
//! Orders move through a time-compressed life cycle (`accepted → printing → shipped`, one
//! step per tick) and status changes are POSTed to the order's optional `callback_url`
//! ([`orders`]). Nothing is printed, nobody is charged, and the prices are placeholders.
//!
//! ```no_run
//! use ebc_demo_printer::{PrinterConfig, app};
//!
//! # async fn run() -> std::io::Result<()> {
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:8103").await?;
//! axum::serve(listener, app(PrinterConfig::default())).await
//! # }
//! ```
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use std::sync::Arc;

pub mod catalog;
pub mod orders;
pub mod preflight;
mod routes;
pub mod testkit;

pub use orders::{Clock, PrinterConfig, PrinterState};

/// Default TCP port of the demo binary.
pub const DEFAULT_PORT: u16 = 8103;

/// Builds the printer API with fresh in-memory state.
pub fn app(config: PrinterConfig) -> axum::Router {
    app_with_state(PrinterState::new(config))
}

/// Builds the printer API around an existing state, for callers that want to drive the
/// life cycle directly ([`PrinterState::advance_all`]).
pub fn app_with_state(state: Arc<PrinterState>) -> axum::Router {
    routes::router(state)
}
