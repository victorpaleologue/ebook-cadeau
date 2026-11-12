//! eBook Cadeau **shelf**: the service that hosts rights holders' eBook files and gives
//! them with eBook Cadeau cards.
//!
//! The core service (`ebc-server`) never holds a book. A shelf stores the files of their
//! rights holders — an author or publisher for their own books (`EBC_SHELF_MODE=own`), or
//! many rights holders (`multi`) — and registers with the core **as a partner**, exactly
//! like a store (`EBC_PARTNERS` at the core, the signed partner API of `docs/api.md` §4).
//! It creates `partner_file` gifts and answers the core's signed link requests on each
//! claim with a short-lived download link on its own host (§4.5).
//!
//! The shelf is the part that is a **host** in law (DSA art. 6, French LCEN art. 6), so
//! the hosting duties live here (`docs/research/09-legal-gifting-and-commerce.md` §2–§3,
//! §9):
//!
//! * accounts by verified e-mail (sign-in links), with the identification data the LCEN
//!   asks hosts to keep;
//! * an upload gate through `ebc-epub` (sniffing, DRM refusal, size limit), a mandatory
//!   rights declaration recorded with time and account, a blocklist by SHA-256 and ISBN,
//!   quotas;
//! * notice and action (DSA art. 16), statements of reasons (art. 17), an internal
//!   complaint, a repeat-infringer policy, terms, contact points and a legal notice;
//! * retention periods for identity, connection data, declarations, notices and
//!   delivery logs.
//!
//! Like the core it is a library plus a thin binary:
//!
//! ```
//! let (state, _outbox) = ebc_shelf::AppState::for_tests(); // in memory, test secrets
//! let router: axum::Router = ebc_shelf::app(state);
//! # let _ = router;
//! ```
//!
//! # Map
//!
//! * [`config`] — environment variables; [`store`] — records, [`MemoryStore`],
//!   [`FileStore`]; [`core`] — the signed client of the core's partner API;
//!   [`mail`] — sign-in links and notices by SMTP; [`clock`] — injected time and
//!   randomness; [`crypto`] — signatures.
//! * [`app`] — the router; [`serve`] — serving it with time-outs and a connection cap.
//!
//! What the code does **not** settle (counsel, operator): see the crate's README.

#![forbid(unsafe_code)]

pub mod clock;
pub mod config;
pub mod core;
pub mod crypto;
pub mod mail;
mod pages;
mod routes;
mod serve;
mod service;
mod state;
pub mod store;
mod text;

pub use clock::{Clock, Entropy, FnClock, ManualClock, OsEntropy, SystemClock};
pub use config::{Config, LegalInfo, Mode, Retention};
pub use mail::{Mailer, MemoryMailer, OutgoingMail, SmtpMailer};
pub use routes::app;
pub use serve::serve;
pub use service::{DECLARATION_VERSION, LICENCES};
pub use state::{AppState, AppStateBuilder, Inner, ShelfError};
pub use store::{Db, FileStore, MemoryStore, Store, StoreError};
