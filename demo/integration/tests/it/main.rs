//! End-to-end journeys over real HTTP: the real `ebc-server`, both demo stores, the demo
//! printer and the simulated devices, all in-process on ephemeral loopback ports.

#![allow(clippy::unwrap_used)]

mod support;

mod card;
mod etagere;
mod grande_enseigne;
mod licence;
mod negative;
mod printing;
mod septante;
mod shelf;
