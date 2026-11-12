//! Domain model for eBook Cadeau: book metadata, locales and sans-IO HTTP types.
//!
//! Everything here is plain data with `serde` support and no I/O, so it can cross the
//! wasm boundary and be stored as JSON.
//!
//! The `book`, `http` and `locale` modules are the **contract** other crates build
//! against. See `docs/architecture.md` for what remains to be added (`code`, `gift`).

#![forbid(unsafe_code)]

pub mod book;
pub mod http;
pub mod locale;

pub use book::{
    BookFormat, BookMeta, Contributor, ContributorRole, CoverKind, CoverRef, SourceRef,
};
pub use ebc_isbn::Isbn13;
pub use http::{HttpRequest, HttpResponse, Method};
pub use locale::Locale;
