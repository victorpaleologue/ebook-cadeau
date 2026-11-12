//! eBook Cadeau web service — the **core**: gift codes, cards, redemption pages for
//! e-readers and the partner API. The HTTP contract is `docs/api.md` (v0.3); this crate
//! implements its §2 (redemption pages), §4 (partner API) and §6 (configuration).
//!
//! The core never stores a book. Every gift belongs to a registered partner (a store, or
//! a "shelf" service hosting files for their rights holders); a `partner_file` gift's
//! file is served by its partner, which hands out a short-lived download link on each
//! claim (`docs/api.md` §4.5).
//!
//! The crate is a library plus a thin binary, so that integration tests and the demos can
//! mount the service in-process:
//!
//! ```
//! use ebc_server::{AppState, app};
//!
//! let state = AppState::for_tests(); // in-memory, development secret
//! let router: axum::Router = app(state);
//! # let _ = router;
//! ```
//!
//! # Map
//!
//! * [`config`] — environment variables and the partners file.
//! * [`store`] — records, the [`Store`] trait, [`MemoryStore`] and [`FileStore`].
//! * [`codes`] — how codes and tokens are minted, hashed and signed.
//! * [`rate`] — per-client and global budgets for unknown codes.
//! * [`device`] — device classes from `User-Agent` (a hint; pages carry a chooser).
//! * [`card`] — `card.pdf`: the [`CardRenderer`] seam, backed by the card composer
//!   `ebc-card` with the cargo feature `card` (default); `501` without it.
//! * [`clock`] — injected time and randomness.
//! * [`names`] — download file names and `Content-Disposition`.
//! * [`app`] — the router; [`serve`] — serving it with time-outs and a connection cap.
//!
//! # Security model in one paragraph
//!
//! A gift code is a bearer secret of about 47.5 bits printed on a card. It is stored only
//! as an HMAC, guesses are throttled per client network **and globally**, typos (caught
//! by the check letter) are answered without a look-up, and files are only ever served
//! through 128-bit download sessions that expire after 15 minutes (a partner file's
//! session redirects to the partner's own short-lived link). Pages carry no script
//! and set no cookie, so there is no session to ride: state-changing `POST`s are
//! authorised by the code in their own URL. Partner requests are HMAC-signed, checked in
//! constant time, bounded to a 300 s window and idempotent.

#![forbid(unsafe_code)]

pub mod card;
pub mod clock;
pub mod codes;
pub mod config;
pub mod device;
pub mod error;
mod mail;
pub mod names;
mod pages;
pub mod rate;
mod routes;
mod serve;
mod service;
mod state;
pub mod store;
mod text;
mod webhook;

#[cfg(feature = "card")]
pub use card::ComposedCardRenderer;
pub use card::{
    CardFormat, CardProfile, CardRenderError, CardRenderer, CardRequest, CoverRights,
    NoCardRenderer, RenderedCard,
};
pub use clock::{Clock, Entropy, ManualClock, OsEntropy, SystemClock};
pub use config::{Config, Partner, PartnersFile, WebhookPolicy};
pub use device::DeviceClass;
pub use error::{ApiError, ServerError};
pub use routes::app;
pub use serve::serve;
pub use state::{AppState, AppStateBuilder, Inner};
pub use store::{Database, FileStore, MemoryStore, Store, StoreError};
