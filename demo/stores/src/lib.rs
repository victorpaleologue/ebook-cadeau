//! Demo: two fictional eBook stores offering the "eBook cadeau" option at checkout.
//!
//! * **Septante** — an independent French store selling DRM-free, watermarked eBooks with
//!   direct downloads. Gifts are delivered as `hosted_file` by default: the store uploads a
//!   copy stamped for the recipient to eBook Cadeau, so the recipient needs no account.
//! * **Grande Enseigne** — a large cultural retailer whose eBooks live in an e-reader
//!   ecosystem account. Nothing is downloaded in the browser; gifts are delivered as
//!   `reseller_redirect` to the store's activation page, and the recipient's e-reader
//!   picks the book up through the device API.
//!
//! Both are **fictional**: names, visual identity, editions, prices and ISBNs are invented,
//! and every page says so. The real-world archetypes they model are named only in
//! `demo/README.md`.
//!
//! The stores are *clients* of the eBook Cadeau partner API (`docs/api.md` §4) and of the
//! print provider API (§5); every outgoing call is in [`outbound`]. Each store is an
//! in-memory [`axum::Router`] built by [`app`], so tests can mount it on an ephemeral port:
//!
//! ```no_run
//! use ebc_demo_stores::{StoreConfig, StoreKind, app};
//!
//! # async fn run() -> std::io::Result<()> {
//! let config = StoreConfig::new(StoreKind::Septante)
//!     .with_public_url("http://127.0.0.1:8101")
//!     .with_partner("http://127.0.0.1:8080", "septante", "demo-secret-septante");
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:8101").await?;
//! axum::serve(listener, app(config)).await
//! # }
//! ```
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use std::sync::Arc;

pub mod assets;
pub mod catalogue;
pub mod checkout;
pub mod config;
pub mod covers;
pub mod crypto;
pub mod epub;
pub mod error;
pub mod gift;
pub mod outbound;
mod routes;
pub mod state;
pub mod time;
pub mod views;

pub use config::{Clock, Entropy, GiftDelivery, PartnerConfig, StoreConfig, StoreKind};
pub use error::StoreError;
pub use state::AppState;

/// Builds the storefront described by `config`, with fresh in-memory state.
///
/// Never fails: configuration problems (an unusable collaborator URL, assets that could not
/// be generated) are reported on the store's home page and disable the affected feature.
pub fn app(config: StoreConfig) -> axum::Router {
    app_with_state(AppState::new(config))
}

/// Builds the storefront around an existing state, for callers that want to inspect it
/// ([`AppState::snapshot`]) without going through the HTTP test hook.
pub fn app_with_state(state: Arc<AppState>) -> axum::Router {
    routes::router(state)
}
