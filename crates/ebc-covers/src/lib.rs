//! Cover images for the gift card: **locating and previewing** candidates, judging image
//! files, and recording the rights under which a cover may be printed.
//!
//! The crate is sans-IO: it never opens a socket or a file. It *describes* requests
//! ([`ebc_model::HttpRequest`]) and *interprets* bytes the host fetched or the user supplied.
//!
//! # Two kinds of output, kept apart by construction
//!
//! Following `docs/research/02-cover-sources.md` (read with its fact-check) and
//! `docs/research/07-legal-covers-and-reseller-rights.md`:
//!
//! * a [`CoverCandidate`] is the result of a **lookup**: a request, the source it belongs
//!   to, a risk level and a note on the terms. Its rights basis is always
//!   [`RightsBasis::None`] and it has no field that can hold pixels. Candidates exist to
//!   help a person find the cover (inline preview from an allow-listed host, or a plain
//!   link) — never to feed the composer;
//! * a [`PrintableCover`] is image bytes **plus a rights basis other than `none`**. It can
//!   only be built from bytes the caller already holds (a reseller feed, a file the user
//!   picked, a cover read from the user's own EPUB, a rights holder's upload) and the
//!   constructor refuses `none`. [`RightsBasis::gate`] then says which print routes the
//!   basis opens.
//!
//! No function of this crate turns a candidate into a printable cover. An operator tool
//! that downloads a candidate's bytes must be told the rights basis explicitly
//! ([`Policy::operator_rights_basis`]) and builds the [`PrintableCover`] itself.
//!
//! # Modules
//!
//! * [`rights`] — [`RightsBasis`], print-route gates, [`PrintableCover`];
//! * [`cascade`] — [`candidates`], [`Policy`], the source descriptors and the host guard;
//! * [`scrape`] — tolerant extraction of image URLs and ISBNs from a product page;
//! * [`robots`] — RFC 9309 parser and matcher; [`tdm`] — TDMRep opt-out signals;
//! * [`sniff`] — header-only format and pixel-size probing;
//! * [`placeholder`] — "no cover available" detection;
//! * [`quality`] — print-quality rating; [`mod@upscale`] — Lanczos-3 resampling.
//!
//! Nothing here was checked against the live services: URL grammars come from the
//! research (source code of the services and of library systems), terms and quotas are
//! unverified, and every source descriptor says so.

#![forbid(unsafe_code)]

pub mod cascade;
pub mod guard;
pub mod placeholder;
pub mod quality;
pub mod rights;
pub mod robots;
pub mod scrape;
pub mod sniff;
pub mod tdm;
pub mod upscale;

pub use cascade::{
    CandidateRole, CoverCandidate, Mode, OptOutList, Policy, Risk, SOURCES, SourceInfo,
    SourceStatus, Use, candidates, page_candidate, page_image_candidates, source,
};
pub use ebc_model::CoverKind;
pub use guard::{Refusal, check_url, is_denied_host, is_public_ip};
pub use placeholder::{Assessment, is_placeholder, is_placeholder_pixels};
pub use quality::{Quality, Thresholds, Verdict, print_quality, print_quality_with};
pub use rights::{
    AssetOrigin, Condition, Grant, Placement, PrintRoute, PrintableCover, RightsBasis,
};
pub use sniff::{Format, ImageInfo, probe};
pub use upscale::{UnsharpMask, UpscaleOptions, upscale, upscale_by};

/// Errors of this crate.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum CoversError {
    /// An image or a requested output size has a zero dimension.
    #[error("image has a zero dimension")]
    EmptyImage,
    /// The requested output exceeds the configured limits.
    #[error(
        "image of {width} x {height} px exceeds the limits ({max_side_px} px per side, {max_pixels} pixels)"
    )]
    ImageTooLarge {
        /// Requested width in pixels.
        width: u32,
        /// Requested height in pixels.
        height: u32,
        /// Largest side allowed.
        max_side_px: u32,
        /// Largest pixel count allowed.
        max_pixels: u64,
    },
    /// A scale factor is not a positive finite number.
    #[error("scale factor must be a positive finite number")]
    InvalidFactor,
    /// A cover cannot be made printable without a rights basis.
    #[error("a cover with rights basis `none` cannot enter a print file")]
    NoRightsBasis,
    /// The rights basis does not open the requested print route.
    #[error("rights basis `{basis}` does not allow {route}")]
    RouteNotAllowed {
        /// The rights basis that was offered.
        basis: RightsBasis,
        /// The route that was asked for.
        route: PrintRoute,
    },
    /// The origin of the bytes and the rights basis do not go together.
    #[error("a cover from {origin} cannot carry rights basis `{basis}`")]
    OriginMismatch {
        /// Where the bytes came from.
        origin: AssetOrigin,
        /// The rights basis that was offered.
        basis: RightsBasis,
    },
    /// The bytes are not an image in a recognised format.
    #[error("not an image in a recognised format (JPEG, PNG, GIF, WebP, AVIF/HEIF)")]
    NotAnImage,
    /// The bytes are a "no cover available" placeholder.
    #[error("the image is a placeholder, not a cover")]
    Placeholder,
    /// A URL was refused before any request (scheme, private address, denied host, …).
    #[error("URL refused: {0}")]
    UrlRefused(Refusal),
    /// An unknown rights-basis name.
    #[error("unknown rights basis {0:?}")]
    UnknownRightsBasis(String),
}
