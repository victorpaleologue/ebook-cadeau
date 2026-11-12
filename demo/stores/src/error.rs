//! The crate's error type.

/// Errors raised while configuring a store or generating its static assets.
///
/// HTTP-level failures of outgoing calls are reported separately as
/// [`crate::outbound::OutboundError`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// A configuration value is unusable.
    #[error("invalid configuration: {0}")]
    Config(String),
    /// The random number generator failed.
    #[error("the random number generator failed")]
    Entropy,
    /// An EPUB could not be assembled.
    #[error("EPUB generation failed: {0}")]
    Epub(String),
    /// A cover could not be rendered.
    #[error("cover rendering failed: {0}")]
    Cover(String),
    /// The HTTP client could not be built.
    #[error("HTTP client could not be built: {0}")]
    HttpClient(String),
}
