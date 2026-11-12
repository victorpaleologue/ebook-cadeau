//! Domain model for eBook Cadeau: book metadata, gifts, redemption codes, locales and
//! sans-IO HTTP types.
//!
//! Everything here is plain data with `serde` support and no I/O, so it can cross the
//! wasm boundary and be stored as JSON. Nothing reads a clock or a random source: time
//! stamps and random bytes are passed in by the caller.
//!
//! * [`book`] — bibliographic metadata ([`BookMeta`]).
//! * [`code`] — the redemption code printed on the card ([`GiftCode`]: twelve consonants
//!   with a check letter that detects every single-letter typo and every swap of two
//!   neighbouring letters), the code that pairs a phone with an e-reader ([`PairCode`]),
//!   and the URL printed on the card and encoded in its QR code ([`claim_url`],
//!   [`claim_url_qr`]).
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
pub use code::{CLAIM_PATH, CodeError, GiftCode, PairCode, claim_url, claim_url_qr};
pub use ebc_isbn::{Isbn10, Isbn13};
pub use gift::{
    Delivery, FileFormat, Gift, GiftError, GiftId, GiftStatus, LicenceFormat, PartnerFileLink,
};
pub use http::{HttpRequest, HttpResponse, Method};
pub use locale::Locale;
