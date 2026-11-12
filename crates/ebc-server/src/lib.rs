//! eBook Cadeau web service: gifts, redemption pages for e-readers, file delivery and the
//! partner API. The HTTP contract is `docs/api.md` (v0.2); this crate implements its §2
//! (redemption pages), §3 (creator API), §4 (partner API) and §6 (configuration).
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
//! * [`upload`] — sniffing of uploads, DRM refusal, rights declaration.
//! * [`card`] — the [`CardRenderer`] seam for `card.pdf` (answers `501` by default).
//! * [`clock`] — injected time and randomness.
//! * [`names`] — download file names and `Content-Disposition`.
//! * [`app`] — the router.
//!
//! # Security model in one paragraph
//!
//! A gift code is a bearer secret of about 47.5 bits printed on a card. It is stored only
//! as an HMAC, guesses are throttled per client network **and globally**, typos (caught
//! by the check letter) are answered without a look-up, and files are only ever served
//! through 128-bit download sessions that expire after 15 minutes. Pages carry no script
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
mod service;
mod state;
pub mod store;
mod text;
pub mod upload;
mod webhook;

pub use card::{
    CardFormat, CardProfile, CardRenderError, CardRenderer, CardRequest, NoCardRenderer,
};
pub use clock::{Clock, Entropy, ManualClock, OsEntropy, SystemClock};
pub use config::{Config, Partner, PartnersFile, WebhookPolicy};
pub use device::DeviceClass;
pub use error::{ApiError, ServerError};
pub use routes::app;
pub use state::{AppState, AppStateBuilder, Inner};
pub use store::{Database, FileStore, MemoryStore, Store, StoreError};
