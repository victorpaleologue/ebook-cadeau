//! Domain model for eBook Cadeau: book metadata, gifts, redemption codes, locales and
//! sans-IO HTTP types.
//!
//! Everything here is plain data with `serde` support and no I/O, so it can cross the
//! wasm boundary and be stored as JSON. Nothing reads a clock or a random source: time
//! stamps and random bytes are passed in by the caller.
//!
//! * [`book`] — bibliographic metadata ([`BookMeta`]).
//! * [`code`] — the redemption code printed on the card ([`GiftCode`], with a check
//!   symbol that detects every single-character typo and every swap of two characters)
//!   and the URL encoded in the QR code ([`claim_url`]).
//! * [`gift`] — a gift record ([`Gift`]), how it is delivered ([`Delivery`]) and where it
//!   is in its life ([`GiftStatus`]).
//! * [`http`] — sans-IO HTTP messages ([`HttpRequest`], [`HttpResponse`]).
//! * [`locale`] — supported user-interface languages ([`Locale`]).

#![forbid(unsafe_code)]

pub mod book;
pub mod code;
pub mod gift;
pub mod http;
pub mod locale;

pub use book::{
    BookFormat, BookMeta, Contributor, ContributorRole, CoverKind, CoverRef, SourceRef,
};
pub use code::{CLAIM_PATH, CodeError, GiftCode, claim_url};
pub use ebc_isbn::{Isbn10, Isbn13};
pub use gift::{Delivery, Gift, GiftError, GiftId, GiftStatus, LicenceFormat};
pub use http::{HttpRequest, HttpResponse, Method};
pub use locale::Locale;
