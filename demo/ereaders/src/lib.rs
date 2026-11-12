//! Simulated e-readers for the eBook Cadeau integration and end-to-end tests.
//!
//! The redemption flow of a gifted eBook only *really* works if it works on the devices people
//! read on: e-ink readers with decade-old browsers, no camera, no JavaScript worth relying on,
//! and ecosystems that each accept different files through different doors. This crate
//! simulates those devices closely enough for tests to prove the flow end to end:
//!
//! | Component | Simulates |
//! |---|---|
//! | [`EinkBrowser`] | the built-in browser: profile `User-Agent`, **no JavaScript**, links, forms, bounded redirects, downloads into the device [`Library`] |
//! | [`Page::lint`] | what breaks on old e-reader browsers (script requirements, external resources, weight, labels…) |
//! | [`TypingCost`] | taps needed on an on-screen e-ink keyboard, for UX budgets |
//! | [`OpdsClient`] | a reading app's OPDS 1.2 (Atom) / 2.0 (JSON) client |
//! | [`MailDevice`] | "send to my device by e-mail" (Send to Kindle, Send-to-PocketBook) behind a tiny SMTP server |
//! | [`SyncDevice`] | an ecosystem-account reader that pulls purchases from a store's device API |
//! | [`SimulatedLcpReader`] | the *shape* of the Readium LCP flow — **no real cryptography** |
//!
//! The two other devices of the journeys are profiles too — [`DeviceProfile::phone`] (scans
//! the QR code, asks for a pair code, activates a gift in a store account, sends to a Kindle)
//! and [`DeviceProfile::computer`] (buys the gift) — driven by the same script-less browser.
//!
//! The simulators follow `docs/api.md` **v0.2**: 12-letter gift codes (`XXXX-XXXX-XXXX`),
//! 6-letter pair codes typed into the field `p` of `/e`, and the explicit device chooser
//! (`?device=`), which [`flow::redeem`] uses when the server guessed the device wrong.
//!
//! What each device family can and cannot do is encoded in [`DeviceProfile`]; every capability
//! carries a [`profile::Claim`] citing `docs/research/notes/ereader-ux.*.md` or marking itself
//! as an assumption. **No profile was validated on hardware.**
//!
//! # Example
//!
//! ```no_run
//! use ebc_demo_ereaders::{DeviceProfile, EinkBrowser};
//!
//! # async fn demo() -> Result<(), ebc_demo_ereaders::Error> {
//! let mut kobo = EinkBrowser::new(DeviceProfile::kobo_clara())?;
//! kobo.open("http://127.0.0.1:8080/").await?;          // typed on the e-ink keyboard
//! kobo.type_into("code-input", "bcdf-ghjk-lmnp")?;   // letters only: no layer switch
//! kobo.submit("code-form").await?;                      // -> /k/BCDFGHJKLMNP
//! let outcome = kobo.click("download").await?;          // POST -> 303 -> the EPUB
//! assert!(outcome.download().is_some());
//! assert!(kobo.total_taps() < 60);
//! # Ok(()) }
//! ```
//!
//! Everything is bounded: requests have deadlines, bodies have size caps, redirect chains have
//! a length limit, and no parser panics on hostile input.

#![forbid(unsafe_code)]

pub mod browser;
mod css;
pub mod error;
pub mod fixtures;
pub mod flow;
pub mod format;
pub mod lcp;
pub mod library;
pub mod lint;
pub mod mail;
pub mod net;
pub mod opds;
pub mod page;
pub mod profile;
pub mod sync;
pub mod typing;

pub use browser::{BrowserConfig, EinkBrowser, LicenceFile, Outcome};
pub use error::{
    BrowserError, DownloadRefusal, Error, ImportError, LcpError, MailError, NetError, OpdsError,
    Result, SyncError,
};
pub use format::FileFormat;
pub use lcp::{
    LcpReader, LicenceStep, SimulatedLcpReader, SimulatedLicence, SimulatedLicenceBuilder,
};
pub use library::{ImportSource, IncomingFile, Library, LibraryEntry};
pub use lint::{LintConfig, LintFinding, LintReport, Severity};
pub use mail::{MailDevice, MailDeviceConfig, MailOutcome, MailRecord, MailboxMessage};
pub use net::{Exchange, NetConfig};
pub use opds::{OpdsClient, OpdsConfig};
pub use page::Page;
pub use profile::{DeviceFamily, DeviceProfile, DrmScheme, MailServiceSpec};
pub use sync::{SyncDevice, SyncReport};
pub use typing::{Keyboard, TapLog, TypingCost};
