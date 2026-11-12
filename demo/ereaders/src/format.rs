//! File formats: recognising them from headers, names and bytes, and validating containers.
//!
//! Validation is deliberately *structural* (is this a ZIP whose first entry is a stored
//! `mimetype`? is there an OPF with a title?) — nothing here renders a book. That is enough to
//! catch the failures that matter for delivery: an HTML error page saved as `.epub`, a truncated
//! download, a DRM-wrapped file sent down a DRM-free channel.

use std::io::Read;

use crate::error::ImportError;
use crate::profile::DrmScheme;

/// Largest `META-INF/container.xml` / `encryption.xml` read from an EPUB.
const MAX_CONTAINER_XML: u64 = 256 * 1024;
/// Largest OPF package document read from an EPUB.
const MAX_OPF: u64 = 4 * 1024 * 1024;
/// Largest number of ZIP entries accepted in an EPUB.
const MAX_ZIP_ENTRIES: usize = 20_000;

/// A file format an e-reader may meet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FileFormat {
    /// EPUB 2/3 (`application/epub+zip`), including Kobo's `.kepub.epub`.
    Epub,
    /// PDF.
    Pdf,
    /// Kindle Format 8 (`.azw3`).
    Azw3,
    /// Mobipocket (`.mobi`, `.prc`, `.azw`).
    Mobi,
    /// Amazon's delivery format after Send-to-Kindle conversion (never served by us).
    Kfx,
    /// Plain text.
    Txt,
    /// FictionBook 2.
    Fb2,
    /// Comic book ZIP.
    Cbz,
    /// Readium LCP licence document (`.lcpl`).
    Lcpl,
    /// Adobe ADEPT fulfilment token (`.acsm`).
    Acsm,
    /// An HTML page (never a book: usually an error page served in place of a file).
    Html,
}

impl FileFormat {
    /// Maps a `Content-Type` value (parameters ignored, case-insensitive) to a format.
    ///
    /// `application/octet-stream` and unknown types give `None`: the caller then falls back to
    /// the file name, as browsers do.
    pub fn from_media_type(content_type: &str) -> Option<Self> {
        let essence = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        Some(match essence.as_str() {
            "application/epub+zip" | "application/kepub+zip" | "application/x-kobo-epub+zip" => {
                Self::Epub
            }
            "application/pdf" | "application/x-pdf" => Self::Pdf,
            "application/vnd.amazon.mobi8-ebook" | "application/x-mobi8-ebook" => Self::Azw3,
            "application/x-mobipocket-ebook" | "application/vnd.amazon.ebook" => Self::Mobi,
            "text/plain" => Self::Txt,
            "application/x-fictionbook+xml" | "application/x-fictionbook" => Self::Fb2,
            "application/vnd.comicbook+zip" | "application/x-cbz" => Self::Cbz,
            "application/vnd.readium.lcp.license.v1.0+json" => Self::Lcpl,
            "application/vnd.adobe.adept+xml" => Self::Acsm,
            "text/html" | "application/xhtml+xml" => Self::Html,
            _ => return None,
        })
    }

    /// Maps a file name to a format by its extension (case-insensitive).
    pub fn from_file_name(file_name: &str) -> Option<Self> {
        let lower = file_name.to_ascii_lowercase();
        let ext = lower.rsplit_once('.').map(|(_, e)| e)?;
        Some(match ext {
            "epub" => Self::Epub,
            "pdf" => Self::Pdf,
            "azw3" | "kf8" => Self::Azw3,
            "mobi" | "prc" | "azw" => Self::Mobi,
            "kfx" => Self::Kfx,
            "txt" => Self::Txt,
            "fb2" => Self::Fb2,
            "cbz" => Self::Cbz,
            "lcpl" => Self::Lcpl,
            "acsm" => Self::Acsm,
            "html" | "htm" | "xhtml" => Self::Html,
            _ => return None,
        })
    }

    /// Looks at the first bytes of a file. `None` when nothing is recognised.
    pub fn sniff(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(b"%PDF-") {
            return Some(Self::Pdf);
        }
        if bytes.starts_with(b"PK\x03\x04") {
            // An EPUB announces itself in its first entry; any other ZIP is treated as a comic
            // archive only if the caller says so — here we only tell EPUB from "some ZIP".
            return if first_entry_is_epub_mimetype(bytes) { Some(Self::Epub) } else { None };
        }
        if bytes.len() >= 68 && (&bytes[60..68] == b"BOOKMOBI" || &bytes[60..68] == b"TEXtREAd") {
            return Some(Self::Mobi);
        }
        let head = &bytes[..bytes.len().min(1024)];
        let text = String::from_utf8_lossy(head);
        let trimmed = text.trim_start_matches('\u{feff}').trim_start();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("<!doctype html") || lower.starts_with("<html") {
            return Some(Self::Html);
        }
        if lower.contains("<fulfillmenttoken") {
            return Some(Self::Acsm);
        }
        if lower.contains("<fictionbook") {
            return Some(Self::Fb2);
        }
        if lower.starts_with("<?xml") && lower.contains("<html") {
            return Some(Self::Html);
        }
        if trimmed.starts_with('{') && looks_like_lcp_licence(bytes) {
            return Some(Self::Lcpl);
        }
        None
    }

    /// The canonical media type.
    pub fn media_type(self) -> &'static str {
        match self {
            Self::Epub => "application/epub+zip",
            Self::Pdf => "application/pdf",
            Self::Azw3 => "application/vnd.amazon.mobi8-ebook",
            Self::Mobi => "application/x-mobipocket-ebook",
            Self::Kfx => "application/vnd.amazon.kfx",
            Self::Txt => "text/plain",
            Self::Fb2 => "application/x-fictionbook+xml",
            Self::Cbz => "application/vnd.comicbook+zip",
            Self::Lcpl => "application/vnd.readium.lcp.license.v1.0+json",
            Self::Acsm => "application/vnd.adobe.adept+xml",
            Self::Html => "text/html",
        }
    }

    /// The usual file extension, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Epub => "epub",
            Self::Pdf => "pdf",
            Self::Azw3 => "azw3",
            Self::Mobi => "mobi",
            Self::Kfx => "kfx",
            Self::Txt => "txt",
            Self::Fb2 => "fb2",
            Self::Cbz => "cbz",
            Self::Lcpl => "lcpl",
            Self::Acsm => "acsm",
            Self::Html => "html",
        }
    }

    /// `true` for formats that are a licence / token rather than a readable book.
    pub fn is_licence(self) -> bool {
        matches!(self, Self::Lcpl | Self::Acsm)
    }
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.extension())
    }
}

/// What structural validation learnt about a book file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BookInfo {
    /// Title from the file's own metadata, when it has one.
    pub title: Option<String>,
    /// Authors from the file's own metadata.
    pub authors: Vec<String>,
    /// BCP 47 language tag from the metadata.
    pub language: Option<String>,
    /// Publication identifier (EPUB `dc:identifier`).
    pub identifier: Option<String>,
    /// DRM markers found in the container.
    pub drm: Option<DrmScheme>,
    /// Non-fatal oddities (shown in the library entry's notes).
    pub warnings: Vec<String>,
}

/// Validates `bytes` as a file of `format` and extracts what metadata the container exposes.
///
/// Never panics on hostile input; every read is bounded.
pub fn inspect(format: FileFormat, bytes: &[u8]) -> Result<BookInfo, ImportError> {
    if bytes.is_empty() {
        return Err(ImportError::Empty);
    }
    match format {
        FileFormat::Epub => inspect_epub(bytes),
        FileFormat::Pdf => inspect_pdf(bytes),
        FileFormat::Mobi | FileFormat::Azw3 => inspect_mobi(format, bytes),
        FileFormat::Txt => inspect_txt(bytes),
        FileFormat::Fb2 => inspect_fb2(bytes),
        FileFormat::Cbz => {
            if bytes.starts_with(b"PK\x03\x04") {
                Ok(BookInfo::default())
            } else {
                Err(invalid(format, "not a ZIP archive"))
            }
        }
        FileFormat::Kfx | FileFormat::Lcpl | FileFormat::Acsm | FileFormat::Html => {
            Err(invalid(format, "not a book file"))
        }
    }
}

fn invalid(format: FileFormat, reason: &str) -> ImportError {
    ImportError::InvalidFile { format, reason: reason.to_owned() }
}

fn epub_err(reason: impl Into<String>) -> ImportError {
    ImportError::InvalidEpub(reason.into())
}

/// The fixed part of a ZIP local file header.
struct LocalHeader {
    flags: u16,
    method: u16,
    compressed_size: u32,
    name_len: usize,
    extra_len: usize,
}

fn read_local_header(bytes: &[u8]) -> Option<LocalHeader> {
    if bytes.len() < 30 || !bytes.starts_with(b"PK\x03\x04") {
        return None;
    }
    let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
    let u32_at =
        |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
    Some(LocalHeader {
        flags: u16_at(6),
        method: u16_at(8),
        compressed_size: u32_at(18),
        name_len: usize::from(u16_at(26)),
        extra_len: usize::from(u16_at(28)),
    })
}

fn first_entry_is_epub_mimetype(bytes: &[u8]) -> bool {
    let Some(header) = read_local_header(bytes) else { return false };
    header.name_len == 8 && bytes.get(30..38) == Some(b"mimetype".as_slice())
}

/// OCF §4.3: the `mimetype` file must be the first entry, stored (not compressed, not
/// encrypted), and contain exactly `application/epub+zip`.
fn check_epub_mimetype_entry(bytes: &[u8], warnings: &mut Vec<String>) -> Result<(), ImportError> {
    let header = read_local_header(bytes).ok_or_else(|| epub_err("not a ZIP archive"))?;
    if header.name_len != 8 || bytes.get(30..38) != Some(b"mimetype".as_slice()) {
        return Err(epub_err("the first ZIP entry is not \"mimetype\""));
    }
    if header.method != 0 {
        return Err(epub_err("the \"mimetype\" entry is compressed; it must be stored"));
    }
    if header.flags & 0x0001 != 0 {
        return Err(epub_err("the \"mimetype\" entry is encrypted"));
    }
    if header.flags & 0x0008 != 0 {
        return Err(epub_err(
            "the \"mimetype\" entry uses a data descriptor; sizes must be inline",
        ));
    }
    if header.extra_len != 0 {
        warnings.push("the \"mimetype\" entry has a ZIP extra field (OCF forbids it)".to_owned());
    }
    let start = 30 + header.name_len + header.extra_len;
    let len = usize::try_from(header.compressed_size).unwrap_or(usize::MAX);
    let end = start.checked_add(len).ok_or_else(|| epub_err("corrupt \"mimetype\" entry"))?;
    let content = bytes.get(start..end).ok_or_else(|| epub_err("truncated \"mimetype\" entry"))?;
    if content != b"application/epub+zip" {
        return Err(epub_err("the \"mimetype\" entry does not contain application/epub+zip"));
    }
    Ok(())
}

fn read_zip_entry(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
    limit: u64,
) -> Result<Option<Vec<u8>>, ImportError> {
    let file = match archive.by_name(name) {
        Ok(file) => file,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(err) => return Err(epub_err(format!("cannot read {name}: {err}"))),
    };
    let mut out = Vec::new();
    // `take` bounds decompression, so a ZIP bomb costs at most `limit + 1` bytes.
    file.take(limit + 1)
        .read_to_end(&mut out)
        .map_err(|err| epub_err(format!("cannot read {name}: {err}")))?;
    if out.len() as u64 > limit {
        return Err(epub_err(format!("{name} is larger than {limit} bytes")));
    }
    Ok(Some(out))
}

fn parse_xml<'a>(text: &'a str, what: &str) -> Result<roxmltree::Document<'a>, ImportError> {
    let options =
        roxmltree::ParsingOptions { allow_dtd: true, nodes_limit: 200_000, ..Default::default() };
    roxmltree::Document::parse_with_options(text, options)
        .map_err(|err| epub_err(format!("{what} is not well-formed XML: {err}")))
}

fn xml_text(bytes: &[u8], what: &str) -> Result<String, ImportError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| epub_err(format!("{what} is not UTF-8")))?
        .trim_start_matches('\u{feff}');
    Ok(text.to_owned())
}

fn collapse_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn inspect_epub(bytes: &[u8]) -> Result<BookInfo, ImportError> {
    let mut info = BookInfo::default();
    check_epub_mimetype_entry(bytes, &mut info.warnings)?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|err| epub_err(format!("unreadable ZIP directory: {err}")))?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(epub_err(format!("more than {MAX_ZIP_ENTRIES} ZIP entries")));
    }

    let container = read_zip_entry(&mut archive, "META-INF/container.xml", MAX_CONTAINER_XML)?
        .ok_or_else(|| epub_err("META-INF/container.xml is missing"))?;
    let container = xml_text(&container, "META-INF/container.xml")?;
    let opf_path = {
        let doc = parse_xml(&container, "META-INF/container.xml")?;
        doc.descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "rootfile")
            .filter_map(|n| n.attribute("full-path"))
            .find(|p| !p.trim().is_empty())
            .map(str::to_owned)
            .ok_or_else(|| epub_err("META-INF/container.xml names no rootfile"))?
    };

    let opf = read_zip_entry(&mut archive, &opf_path, MAX_OPF)?
        .ok_or_else(|| epub_err(format!("package document {opf_path} is missing")))?;
    let opf = xml_text(&opf, "the package document")?;
    {
        let doc = parse_xml(&opf, "the package document")?;
        let root = doc.root_element();
        if root.tag_name().name() != "package" {
            return Err(epub_err("the package document's root element is not <package>"));
        }
        let metadata = root
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "metadata")
            .ok_or_else(|| epub_err("the package document has no <metadata>"))?;
        for node in metadata.descendants().filter(roxmltree::Node::is_element) {
            let text = collapse_ws(
                &node.text().unwrap_or_default().chars().take(2000).collect::<String>(),
            );
            if text.is_empty() {
                continue;
            }
            match node.tag_name().name() {
                "title" if info.title.is_none() => info.title = Some(text),
                "creator" if info.authors.len() < 32 => info.authors.push(text),
                "language" if info.language.is_none() => info.language = Some(text),
                "identifier" if info.identifier.is_none() => info.identifier = Some(text),
                _ => {}
            }
        }
        if info.title.is_none() {
            return Err(epub_err("the package document has no dc:title"));
        }
    }

    info.drm = detect_epub_drm(&mut archive)?;
    Ok(info)
}

/// Algorithms that are font obfuscation, not DRM (IDPF and Adobe).
const FONT_OBFUSCATION: [&str; 2] =
    ["http://www.idpf.org/2008/embedding", "http://ns.adobe.com/pdf/enc#RC"];

fn detect_epub_drm(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
) -> Result<Option<DrmScheme>, ImportError> {
    if archive.index_for_name("META-INF/license.lcpl").is_some() {
        return Ok(Some(DrmScheme::ReadiumLcp));
    }
    if archive.index_for_name("META-INF/rights.xml").is_some() {
        return Ok(Some(DrmScheme::AdobeAdept));
    }
    let Some(encryption) = read_zip_entry(archive, "META-INF/encryption.xml", MAX_CONTAINER_XML)?
    else {
        return Ok(None);
    };
    let text = xml_text(&encryption, "META-INF/encryption.xml")?;
    let doc = parse_xml(&text, "META-INF/encryption.xml")?;
    let mut encrypted_resources = 0usize;
    let mut scheme = None;
    for node in doc.descendants().filter(|n| n.is_element()) {
        if node.tag_name().name() != "EncryptionMethod" {
            continue;
        }
        let algorithm = node.attribute("Algorithm").unwrap_or("");
        if FONT_OBFUSCATION.contains(&algorithm) {
            continue;
        }
        encrypted_resources += 1;
        scheme.get_or_insert(DrmScheme::Unknown);
    }
    // A <KeyInfo> retrieval pointing at an LCP licence marks LCP even without the embedded file.
    if encrypted_resources > 0 && text.contains("license.lcpl") {
        scheme = Some(DrmScheme::ReadiumLcp);
    }
    Ok(scheme)
}

fn inspect_pdf(bytes: &[u8]) -> Result<BookInfo, ImportError> {
    if !bytes.starts_with(b"%PDF-") {
        return Err(ImportError::InvalidPdf("missing %PDF- header".to_owned()));
    }
    let tail = &bytes[bytes.len().saturating_sub(2048)..];
    if !contains(tail, b"%%EOF") {
        return Err(ImportError::InvalidPdf(
            "no %%EOF marker near the end (truncated download?)".to_owned(),
        ));
    }
    let mut info = BookInfo::default();
    // Approximation: only an uncompressed Info dictionary with a literal-string /Title is read.
    info.title = pdf_literal(bytes, b"/Title");
    if let Some(author) = pdf_literal(bytes, b"/Author") {
        info.authors.push(author);
    }
    if contains(&bytes[..bytes.len().min(1 << 20)], b"/Encrypt") {
        info.warnings.push("the PDF declares /Encrypt (password or DRM protected)".to_owned());
        info.drm = Some(DrmScheme::Unknown);
    }
    Ok(info)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    find(haystack, needle).is_some()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Reads `/Key (literal string)` from the first megabyte of a PDF. Handles `\(`, `\)`, `\\` and
/// balanced parentheses; anything else (hex strings, UTF-16, object streams) yields `None`.
fn pdf_literal(bytes: &[u8], key: &[u8]) -> Option<String> {
    let window = &bytes[..bytes.len().min(1 << 20)];
    let at = find(window, key)? + key.len();
    let rest = &window[at..];
    let open = rest.iter().position(|b| !b.is_ascii_whitespace())?;
    if rest.get(open) != Some(&b'(') {
        return None;
    }
    let mut out = Vec::new();
    let mut depth = 1u32;
    let mut escaped = false;
    for &b in rest.get(open + 1..)?.iter().take(1024) {
        if escaped {
            out.push(b);
            escaped = false;
            continue;
        }
        match b {
            b'\\' => escaped = true,
            b'(' => {
                depth += 1;
                out.push(b);
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let text = String::from_utf8(out).ok()?;
                    let text = collapse_ws(&text);
                    return (!text.is_empty()).then_some(text);
                }
                out.push(b);
            }
            _ => out.push(b),
        }
    }
    None
}

fn inspect_mobi(format: FileFormat, bytes: &[u8]) -> Result<BookInfo, ImportError> {
    if bytes.len() < 78 || !(&bytes[60..68] == b"BOOKMOBI" || &bytes[60..68] == b"TEXtREAd") {
        return Err(invalid(format, "missing PalmDB BOOKMOBI signature"));
    }
    // Approximation: the PalmDB name (first 32 bytes) stands in for the title; the real title
    // lives in the EXTH header, which this simulation does not parse.
    let name: Vec<u8> = bytes[..32].iter().copied().take_while(|b| *b != 0).collect();
    let title = String::from_utf8_lossy(&name).replace('_', " ").trim().to_owned();
    Ok(BookInfo { title: (!title.is_empty()).then_some(title), ..BookInfo::default() })
}

fn inspect_txt(bytes: &[u8]) -> Result<BookInfo, ImportError> {
    if bytes.contains(&0) {
        return Err(invalid(FileFormat::Txt, "contains NUL bytes; not a text file"));
    }
    if std::str::from_utf8(bytes).is_err() {
        // Old devices assume a legacy code page; accept, but say so.
        return Ok(BookInfo {
            warnings: vec!["not valid UTF-8; accents may be garbled on the device".to_owned()],
            ..BookInfo::default()
        });
    }
    Ok(BookInfo::default())
}

fn inspect_fb2(bytes: &[u8]) -> Result<BookInfo, ImportError> {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).to_ascii_lowercase();
    if head.contains("<fictionbook") {
        Ok(BookInfo::default())
    } else {
        Err(invalid(FileFormat::Fb2, "no <FictionBook> root element"))
    }
}

fn looks_like_lcp_licence(bytes: &[u8]) -> bool {
    if bytes.len() > 1 << 20 {
        return false;
    }
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(value) => value.get("encryption").is_some() && value.get("links").is_some(),
        Err(_) => false,
    }
}

/// `true` when the name marks a Kobo-flavoured EPUB (`.kepub.epub`), which Kobo devices render
/// with their WebKit engine instead of the Adobe one
/// (research: `ereader-ux.devices.md` §3.2, \[V2\]).
pub fn is_kepub_name(file_name: &str) -> bool {
    file_name.to_ascii_lowercase().ends_with(".kepub.epub")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn media_types_and_extensions_map_to_formats() {
        assert_eq!(
            FileFormat::from_media_type("Application/EPUB+zip; charset=binary"),
            Some(FileFormat::Epub)
        );
        assert_eq!(FileFormat::from_media_type("application/octet-stream"), None);
        assert_eq!(FileFormat::from_media_type(""), None);
        assert_eq!(FileFormat::from_file_name("Livre.KEPUB.EPUB"), Some(FileFormat::Epub));
        assert_eq!(FileFormat::from_file_name("a.azw3"), Some(FileFormat::Azw3));
        assert_eq!(FileFormat::from_file_name("noext"), None);
        assert_eq!(FileFormat::from_file_name(""), None);
        for format in [
            FileFormat::Epub,
            FileFormat::Pdf,
            FileFormat::Azw3,
            FileFormat::Mobi,
            FileFormat::Txt,
            FileFormat::Fb2,
            FileFormat::Cbz,
            FileFormat::Lcpl,
            FileFormat::Acsm,
            FileFormat::Html,
        ] {
            assert_eq!(FileFormat::from_media_type(format.media_type()), Some(format));
            assert_eq!(
                FileFormat::from_file_name(&format!("x.{}", format.extension())),
                Some(format)
            );
        }
    }

    #[test]
    fn sniffing_recognises_real_containers_and_error_pages() {
        let epub = fixtures::epub("Titre", "Autrice");
        assert_eq!(FileFormat::sniff(&epub), Some(FileFormat::Epub));
        assert_eq!(FileFormat::sniff(&fixtures::pdf("T")), Some(FileFormat::Pdf));
        assert_eq!(FileFormat::sniff(&fixtures::mobi("T")), Some(FileFormat::Mobi));
        assert_eq!(
            FileFormat::sniff(b"<!DOCTYPE html><html><body>404</body></html>"),
            Some(FileFormat::Html)
        );
        assert_eq!(FileFormat::sniff(b""), None);
        assert_eq!(FileFormat::sniff(b"PK\x03\x04"), None);
        assert_eq!(FileFormat::sniff(&[0u8; 100]), None);
    }

    #[test]
    fn epub_metadata_is_read_from_the_opf() {
        let info =
            inspect(FileFormat::Epub, &fixtures::epub("Le Horla", "Guy de Maupassant")).unwrap();
        assert_eq!(info.title.as_deref(), Some("Le Horla"));
        assert_eq!(info.authors, vec!["Guy de Maupassant".to_owned()]);
        assert_eq!(info.language.as_deref(), Some("fr"));
        assert_eq!(info.drm, None);
        assert!(info.warnings.is_empty());
    }

    #[test]
    fn epub_with_compressed_or_misplaced_mimetype_is_rejected() {
        let good = fixtures::EpubBuilder::new("T", "A");
        let wrong_first = good.clone().mimetype_first(false).build();
        let err = inspect(FileFormat::Epub, &wrong_first).unwrap_err();
        assert!(
            matches!(&err, ImportError::InvalidEpub(m) if m.contains("first ZIP entry")),
            "{err}"
        );

        let wrong_content = good.clone().mimetype_content("application/zip").build();
        let err = inspect(FileFormat::Epub, &wrong_content).unwrap_err();
        assert!(matches!(&err, ImportError::InvalidEpub(m) if m.contains("application/epub+zip")));

        let deflated = good.clone().mimetype_method(8).build();
        let err = inspect(FileFormat::Epub, &deflated).unwrap_err();
        assert!(matches!(&err, ImportError::InvalidEpub(m) if m.contains("stored")), "{err}");

        let no_container = good.clone().without_container().build();
        let err = inspect(FileFormat::Epub, &no_container).unwrap_err();
        assert!(matches!(&err, ImportError::InvalidEpub(m) if m.contains("container.xml")));

        let no_title = good.clone().without_title().build();
        let err = inspect(FileFormat::Epub, &no_title).unwrap_err();
        assert!(matches!(&err, ImportError::InvalidEpub(m) if m.contains("dc:title")), "{err}");
    }

    #[test]
    fn drm_markers_are_detected_but_font_obfuscation_is_not_drm() {
        let adobe = fixtures::EpubBuilder::new("T", "A").with_adobe_rights().build();
        assert_eq!(inspect(FileFormat::Epub, &adobe).unwrap().drm, Some(DrmScheme::AdobeAdept));

        let lcp = fixtures::EpubBuilder::new("T", "A").with_lcp_marker().build();
        assert_eq!(inspect(FileFormat::Epub, &lcp).unwrap().drm, Some(DrmScheme::ReadiumLcp));

        let fonts = fixtures::EpubBuilder::new("T", "A")
            .with_encryption_algorithm("http://www.idpf.org/2008/embedding")
            .build();
        assert_eq!(inspect(FileFormat::Epub, &fonts).unwrap().drm, None);

        let aes = fixtures::EpubBuilder::new("T", "A")
            .with_encryption_algorithm("http://www.w3.org/2001/04/xmlenc#aes256-cbc")
            .build();
        assert_eq!(inspect(FileFormat::Epub, &aes).unwrap().drm, Some(DrmScheme::Unknown));
    }

    #[test]
    fn truncated_and_garbage_epubs_never_panic() {
        let epub = fixtures::epub("Titre", "Autrice");
        for cut in 0..epub.len() {
            let _ = inspect(FileFormat::Epub, &epub[..cut]);
            let _ = FileFormat::sniff(&epub[..cut]);
        }
        // Flip every byte of the first 200 in turn.
        for i in 0..epub.len().min(200) {
            let mut bad = epub.clone();
            bad[i] ^= 0xff;
            let _ = inspect(FileFormat::Epub, &bad);
        }
        assert_eq!(inspect(FileFormat::Epub, b""), Err(ImportError::Empty));
    }

    #[test]
    fn pdf_needs_header_and_trailer() {
        let pdf = fixtures::pdf("Notice");
        let info = inspect(FileFormat::Pdf, &pdf).unwrap();
        assert_eq!(info.title.as_deref(), Some("Notice"));
        let truncated = &pdf[..pdf.len() - 10];
        assert!(matches!(
            inspect(FileFormat::Pdf, truncated),
            Err(ImportError::InvalidPdf(m)) if m.contains("%%EOF")
        ));
        assert!(matches!(
            inspect(FileFormat::Pdf, b"<html>not a pdf</html>"),
            Err(ImportError::InvalidPdf(m)) if m.contains("header")
        ));
    }

    #[test]
    fn pdf_title_parser_handles_escapes_and_garbage() {
        assert_eq!(
            pdf_literal(b"/Title (A \\(nested\\) (b) c)", b"/Title").unwrap(),
            "A (nested) (b) c"
        );
        assert_eq!(pdf_literal(b"/Title <FEFF0041>", b"/Title"), None);
        assert_eq!(pdf_literal(b"/Title (unterminated", b"/Title"), None);
        assert_eq!(pdf_literal(b"/Title", b"/Title"), None);
        assert_eq!(pdf_literal(b"/Title ()", b"/Title"), None);
    }

    #[test]
    fn kepub_names() {
        assert!(is_kepub_name("Le-Horla.kepub.epub"));
        assert!(!is_kepub_name("Le-Horla.epub"));
    }
}
