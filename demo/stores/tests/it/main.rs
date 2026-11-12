//! Integration tests of the two demo stores, as one test binary (keeps link time and disk
//! use down): the storefront over real HTTP ([`shop`]), checkout with the gift option
//! against a strict mock of the eBook Cadeau partner API and the real demo printer
//! ([`gifts`]), and the recipient's side of a `reseller_redirect` gift plus webhooks
//! ([`activation`]). The journeys against the **real** `ebc-server` live in
//! `demo/integration`.

#![allow(clippy::unwrap_used)]

mod activation;
mod common;
mod gifts;
mod shop;
