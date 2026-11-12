//! Demo: the whole eBook Cadeau constellation in one process, for end-to-end tests and
//! manual demos.
//!
//! [`World`] starts, on loopback ports, the **real** `ebc-server`, the two fictional stores
//! of `demo/stores` ("Septante", a DRM-free store serving its own files as `partner_file`, and "Grande
//! Enseigne", an ecosystem store using `reseller_redirect`), the fictional printer of
//! `demo/printer` and, on request, a mail-to-device gateway of `demo/ereaders` — wired to
//! each other exactly as a deployment would be: the stores sign partner requests with the
//! secrets the server knows, the server delivers webhooks to the stores, redirects
//! recipients to them, sends mail to the gateway and forwards print orders to the printer.
//! Nothing is mocked; every call is real HTTP (or SMTP) on `127.0.0.1`.
//!
//! ```no_run
//! use ebc_demo_ereaders::DeviceProfile;
//! use ebc_demo_integration::{World, shop};
//!
//! # async fn journey() -> Result<(), Box<dyn std::error::Error>> {
//! let world = World::start().await?;                       // ephemeral ports, manual clock
//! let mut buyer = world.browser(DeviceProfile::computer())?;
//! let receipt = shop::buy_gift(&mut buyer, &world.septante.base, &shop::GiftOrder::demo()).await?;
//! let mut kobo = world.browser(DeviceProfile::kobo_clara())?;
//! kobo.open(&world.server.url("/")).await?;
//! kobo.type_into("code-input", receipt.gifts[0].code.as_deref().unwrap_or(""))?;
//! kobo.click("code-submit").await?;
//! assert!(kobo.click("download").await?.download().is_some());
//! # Ok(()) }
//! ```
//!
//! * [`World`], [`WorldBuilder`] — spawn helpers and what the tests need to know: base
//!   URLs, partner secrets, the shared clock, the in-process states.
//! * [`partner`] — a signed partner-API client for direct calls (and for tampering).
//! * [`shop`] — a buyer walking a demo store with a script-less simulated browser.
//!
//! The `ebc-demo` binary starts the same constellation on fixed ports and prints a table
//! of URLs (see `demo/README.md`).
//!
//! Everything here is **fictional and for demonstration**: the stores and the printer are
//! not affiliated with any real company, secrets are public, data lives in memory.

#![forbid(unsafe_code)]

pub mod partner;
pub mod shop;
mod world;

pub use world::{
    PrinterHandle, ServerHandle, StoreHandle, T0, World, WorldBuilder, WorldClock, WorldError,
    WorldPorts,
};
