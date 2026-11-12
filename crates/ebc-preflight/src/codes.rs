//! The stable machine codes carried by [`crate::Finding::code`].
//!
//! Codes never change meaning; new checks get new codes. The default severity is given in
//! each description — some depend on the [`crate::Expectations`] or on whether the file
//! claims PDF/X conformance.

// ---- structure -------------------------------------------------------------------------

/// The file is encrypted (`Encrypt` in the trailer). Error.
pub const ENCRYPTED: &str = "ENCRYPTED";
/// The cross-reference table was damaged and had to be rebuilt to read the file (error), or
/// an offset to it was wrong and it was found nearby (warning).
pub const XREF_DAMAGED: &str = "XREF_DAMAGED";
/// The document has no pages. Error.
pub const NO_PAGES: &str = "NO_PAGES";
/// The page tree is malformed (cycle, too deep, node that is not a dictionary). Error.
pub const PAGE_TREE_INVALID: &str = "PAGE_TREE_INVALID";
/// JavaScript is present. Error.
pub const JAVASCRIPT: &str = "JAVASCRIPT";
/// The catalog or a page carries an `OpenAction` / `AA` action. Error.
pub const ACTION: &str = "ACTION";
/// A page has annotations (links, comments, widgets …). Error inside the bleed box,
/// warning when entirely outside it; printer's-mark and trap-network annotations are ignored.
pub const ANNOTATION: &str = "ANNOTATION";
/// The document has an interactive form. Error when it has fields, warning when empty.
pub const ACROFORM: &str = "ACROFORM";
/// The document embeds files (attachments). Error.
pub const EMBEDDED_FILE: &str = "EMBEDDED_FILE";
/// The document uses optional content (layers). Error when an output intent is required
/// (professional printing: layers must be flattened, `docs/research/03-print-requirements.md`
/// §3.2 and check C3); otherwise a warning under PDF/X-1a and PDF/X-3, else info.
pub const OPTIONAL_CONTENT: &str = "OPTIONAL_CONTENT";
/// The file was saved with incremental updates: earlier revisions remain inside it. Warning.
pub const INCREMENTAL_UPDATE: &str = "INCREMENTAL_UPDATE";
/// The document is too complex to analyse completely within the built-in limits, so the
/// report is partial. Error (a warning when only the list of image placements was cut short).
pub const LIMIT_EXCEEDED: &str = "LIMIT_EXCEEDED";

// ---- page boxes ------------------------------------------------------------------------

/// A page has no (valid) `MediaBox`. Error.
pub const BOX_MEDIA_MISSING: &str = "BOX_MEDIA_MISSING";
/// A page has no `TrimBox`. Error.
pub const BOX_TRIM_MISSING: &str = "BOX_TRIM_MISSING";
/// A page has no `BleedBox` although bleed is expected. Warning (the box that takes its
/// place is used for the bleed measurement).
pub const BOX_BLEED_MISSING: &str = "BOX_BLEED_MISSING";
/// A page box is malformed (not four finite numbers, or empty). Error.
pub const BOX_INVALID: &str = "BOX_INVALID";
/// The boxes are not nested `TrimBox ⊆ BleedBox ⊆ MediaBox`. Error.
pub const BOX_NOT_NESTED: &str = "BOX_NOT_NESTED";
/// The MediaBox is larger than the BleedBox (a slug for marks) although the upload must be
/// exactly trim + bleed. Error.
pub const BOX_MEDIA_NOT_BLEED: &str = "BOX_MEDIA_NOT_BLEED";
/// A `CropBox` cuts into the bleed box: viewers and some RIPs would clip the bleed. Warning.
pub const BOX_CROP_CLIPS_BLEED: &str = "BOX_CROP_CLIPS_BLEED";
/// A page has both a `TrimBox` and an `ArtBox`, which PDF/X forbids. Warning.
pub const BOX_TRIM_AND_ART: &str = "BOX_TRIM_AND_ART";
/// Artwork reaches the trim edge but does not extend far enough into the bleed: a white
/// sliver may show after cutting. Warning.
pub const BLEED_NOT_PAINTED: &str = "BLEED_NOT_PAINTED";
/// The bleed is smaller than expected on at least one side. Error.
pub const BOX_BLEED_TOO_SMALL: &str = "BOX_BLEED_TOO_SMALL";
/// The trim size differs from the expected size. Error.
pub const BOX_TRIM_SIZE: &str = "BOX_TRIM_SIZE";
/// Pages do not all have the same trim / media size. Error.
pub const PAGE_SIZE_MISMATCH: &str = "PAGE_SIZE_MISMATCH";
/// A page is rotated with `Rotate`. Warning.
pub const PAGE_ROTATED: &str = "PAGE_ROTATED";
/// A page uses `UserUnit` ≠ 1. Warning.
pub const PAGE_USER_UNIT: &str = "PAGE_USER_UNIT";

// ---- fonts -----------------------------------------------------------------------------

/// A font has no embedded font program. Error.
pub const FONT_NOT_EMBEDDED: &str = "FONT_NOT_EMBEDDED";
/// A Type 3 font is used (glyphs are arbitrary drawings, often low-resolution bitmaps). Warning.
pub const FONT_TYPE3: &str = "FONT_TYPE3";
/// A font is embedded in full rather than as a subset. Info.
pub const FONT_NOT_SUBSET: &str = "FONT_NOT_SUBSET";
/// A font dictionary is malformed. Error.
pub const FONT_INVALID: &str = "FONT_INVALID";

// ---- images ----------------------------------------------------------------------------

/// An image is placed at an effective resolution below the minimum. Error.
pub const IMAGE_LOW_RES: &str = "IMAGE_LOW_RES";
/// An image dictionary is malformed or the image is placed with a degenerate transform. Warning.
pub const IMAGE_INVALID: &str = "IMAGE_INVALID";
/// An image is scaled by different factors horizontally and vertically (stretched). Warning.
pub const IMAGE_DISTORTED: &str = "IMAGE_DISTORTED";
/// More than 0.5 % of the pixels of a losslessly stored CMYK image exceed the total ink
/// limit. Warning.
pub const IMAGE_INK_TOTAL_TOO_HIGH: &str = "IMAGE_INK_TOTAL_TOO_HIGH";

// ---- colour ----------------------------------------------------------------------------

/// RGB (or other three-component, device-independent) colour is used. Error unless allowed.
pub const COLOR_RGB: &str = "COLOR_RGB";
/// A spot colour (`Separation` / `DeviceN` with non-process colorants) is used. Warning.
pub const COLOR_SPOT: &str = "COLOR_SPOT";
/// Registration colour (`Separation /All`) is painted inside the bleed box (error), or its
/// position could not be determined (warning).
pub const COLOR_REGISTRATION: &str = "COLOR_REGISTRATION";
/// A colour space could not be resolved. Warning.
pub const COLOR_UNKNOWN: &str = "COLOR_UNKNOWN";
/// Printer's marks (anything in the registration colour, `Separation /All`) in a file
/// for a printer that wants none. Error.
pub const MARKS_NOT_ALLOWED: &str = "MARKS_NOT_ALLOWED";
/// A flat CMYK colour in the printed area has a process channel above 0 and below 5 %,
/// a tint that prints unreliably (Pixartprinting: no tint below 5 %). Warning, when an
/// output intent is required.
pub const INK_LIGHT_TINT: &str = "INK_LIGHT_TINT";
/// A CMYK fill or stroke exceeds the total ink limit. Error.
pub const INK_TOTAL_TOO_HIGH: &str = "INK_TOTAL_TOO_HIGH";

// ---- transparency ----------------------------------------------------------------------

/// Transparency is used (soft mask, opacity below 1, blend mode, transparency group).
/// Error unless allowed.
pub const TRANSPARENCY: &str = "TRANSPARENCY";

// ---- content streams -------------------------------------------------------------------

/// A content stream could not be read or parsed completely. Error.
pub const CONTENT_INVALID: &str = "CONTENT_INVALID";
/// `q` / `Q` operators are not balanced. Warning.
pub const CONTENT_UNBALANCED_Q: &str = "CONTENT_UNBALANCED_Q";
/// A resource named by an operator is missing from the resource dictionary. Warning.
pub const RESOURCE_MISSING: &str = "RESOURCE_MISSING";
/// A PostScript XObject is used. Error.
pub const POSTSCRIPT_XOBJECT: &str = "POSTSCRIPT_XOBJECT";
/// A stroke in the printed area is thinner than 0.25 pt (warning) or has zero width, which
/// means "as thin as the device can draw" (error).
pub const STROKE_HAIRLINE: &str = "STROKE_HAIRLINE";

// ---- output intent and PDF/X -----------------------------------------------------------

/// The document has no output intent. Error when one is required.
pub const NO_OUTPUT_INTENT: &str = "NO_OUTPUT_INTENT";
/// The output intent is malformed (wrong subtype, missing identifier, several PDF/X intents,
/// unreadable or missing profile). Error when an output intent is required, else warning.
pub const OUTPUT_INTENT_INVALID: &str = "OUTPUT_INTENT_INVALID";
/// The `N` of the output profile does not match the profile's ICC header. Error.
pub const OUTPUT_INTENT_PROFILE_MISMATCH: &str = "OUTPUT_INTENT_PROFILE_MISMATCH";
/// The file makes no PDF/X claim. Warning when an output intent is required, else info.
pub const PDFX_NOT_CLAIMED: &str = "PDFX_NOT_CLAIMED";
/// The PDF/X version markers contradict each other or the file header version. Error.
pub const PDFX_VERSION_MISMATCH: &str = "PDFX_VERSION_MISMATCH";
/// The content contradicts the PDF/X level the file claims (RGB under PDF/X-1a, transparency
/// under PDF/X-1a or PDF/X-3) although the expectations would have accepted it. Error.
pub const PDFX_CONTENT_VIOLATION: &str = "PDFX_CONTENT_VIOLATION";
/// The file claims PDF/X but has no file identifier (`ID` in the trailer). Warning.
pub const NO_FILE_ID: &str = "NO_FILE_ID";
/// The file claims PDF/X but carries no creation or modification date. Error.
pub const NO_DOCUMENT_DATES: &str = "NO_DOCUMENT_DATES";
/// The Info dictionary has no `Title`. Error under PDF/X, else warning.
pub const NO_TITLE: &str = "NO_TITLE";
/// Info `Trapped` is missing or not `True`/`False`. Error under PDF/X, else warning.
pub const TRAPPED_NOT_SET: &str = "TRAPPED_NOT_SET";

// ---- eBook Cadeau geometry -------------------------------------------------------------

/// An `EBC.*` Info key is not a valid number (or list of numbers). Error.
pub const EBC_KEY_INVALID: &str = "EBC_KEY_INVALID";
/// The `EBC.*` Info keys contradict the trim box or each other. Error.
pub const EBC_GEOMETRY_MISMATCH: &str = "EBC_GEOMETRY_MISMATCH";
/// A spine width is expected but the file does not declare one. Error.
pub const EBC_SPINE_MISSING: &str = "EBC_SPINE_MISSING";
/// The declared spine width differs from the expected one. Error.
pub const EBC_SPINE_MISMATCH: &str = "EBC_SPINE_MISMATCH";
/// A QR-like vector graphic (or the declared `EBC.QrModuleMm`) has modules smaller than the
/// expected minimum. Error.
pub const QR_MODULE_TOO_SMALL: &str = "QR_MODULE_TOO_SMALL";
/// A minimum QR module size is expected but no QR-like vector graphic was recognised. Info.
pub const QR_NOT_FOUND: &str = "QR_NOT_FOUND";

/// Every code this crate can emit, sorted alphabetically.
pub const ALL: &[&str] = &[
    ACROFORM,
    ACTION,
    ANNOTATION,
    BLEED_NOT_PAINTED,
    BOX_BLEED_MISSING,
    BOX_BLEED_TOO_SMALL,
    BOX_CROP_CLIPS_BLEED,
    BOX_INVALID,
    BOX_MEDIA_MISSING,
    BOX_MEDIA_NOT_BLEED,
    BOX_NOT_NESTED,
    BOX_TRIM_AND_ART,
    BOX_TRIM_MISSING,
    BOX_TRIM_SIZE,
    COLOR_REGISTRATION,
    COLOR_RGB,
    COLOR_SPOT,
    COLOR_UNKNOWN,
    CONTENT_INVALID,
    CONTENT_UNBALANCED_Q,
    EBC_GEOMETRY_MISMATCH,
    EBC_KEY_INVALID,
    EBC_SPINE_MISMATCH,
    EBC_SPINE_MISSING,
    EMBEDDED_FILE,
    ENCRYPTED,
    FONT_INVALID,
    FONT_NOT_EMBEDDED,
    FONT_NOT_SUBSET,
    FONT_TYPE3,
    IMAGE_DISTORTED,
    IMAGE_INK_TOTAL_TOO_HIGH,
    IMAGE_INVALID,
    IMAGE_LOW_RES,
    INCREMENTAL_UPDATE,
    INK_LIGHT_TINT,
    INK_TOTAL_TOO_HIGH,
    JAVASCRIPT,
    LIMIT_EXCEEDED,
    MARKS_NOT_ALLOWED,
    NO_DOCUMENT_DATES,
    NO_FILE_ID,
    NO_OUTPUT_INTENT,
    NO_PAGES,
    NO_TITLE,
    OPTIONAL_CONTENT,
    OUTPUT_INTENT_INVALID,
    OUTPUT_INTENT_PROFILE_MISMATCH,
    PAGE_ROTATED,
    PAGE_SIZE_MISMATCH,
    PAGE_TREE_INVALID,
    PAGE_USER_UNIT,
    PDFX_CONTENT_VIOLATION,
    PDFX_NOT_CLAIMED,
    PDFX_VERSION_MISMATCH,
    POSTSCRIPT_XOBJECT,
    QR_MODULE_TOO_SMALL,
    QR_NOT_FOUND,
    RESOURCE_MISSING,
    STROKE_HAIRLINE,
    TRANSPARENCY,
    TRAPPED_NOT_SET,
    XREF_DAMAGED,
];
