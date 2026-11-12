//! Integration tests of `ebc-server`: the service mounted in-process, offline (loopback
//! sockets only). One test binary on purpose: every extra binary links the whole
//! dependency tree again.

#![allow(clippy::unwrap_used, missing_docs, dead_code, unreachable_pub)]

mod common;
mod creator_api;
mod delivery;
mod partner_api;
mod redemption;
