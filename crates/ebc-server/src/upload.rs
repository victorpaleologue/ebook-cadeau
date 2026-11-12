//! Validation of uploaded files (`docs/api.md` §3): the type is **sniffed**, never trusted
//! from a header; EPUBs carrying DRM are refused; the uploader must declare on what basis
//! the file may be distributed.

use std::io::{Cursor, Read};

use serde::{Deserialize, Serialize};

/// The declaration every upload must carry in `X-Rights-Basis`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsBasis {
    /// The uploader wrote the book.
    OwnWork,
    /// The book is in the public domain.
    PublicDomain,
    /// The book is under an open licence that allows redistribution.
    OpenLicence,
    /// The uploader is a distributor licensed by the rights holder (partners).
    LicensedDistributor,
}

impl RightsBasis {
    /// Every accepted value.
    pub const ALL: [RightsBasis; 4] =
        [Self::OwnWork, Self::PublicDomain, Self::OpenLicence, Self::LicensedDistributor];

    /// The header value: `own_work`, `public_domain`, `open_licence`,
    /// `licensed_distributor`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OwnWork => "own_work",
            Self::PublicDomain => "public_domain",
            Self::OpenLicence => "open_licence",
            Self::LicensedDistributor => "licensed_distributor",
        }
    }

    /// Parses a header value (exact, lower case; surrounding spaces ignored).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        Self::ALL.into_iter().find(|basis| basis.as_str() == value)
    }
}

/// What an uploaded file turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    /// EPUB (an OCF ZIP container).
    Epub,
    /// PDF.
    Pdf,
}

impl FileKind {
    /// The media type served with the file.
    #[must_use]
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Epub => "application/epub+zip",
            Self::Pdf => "application/pdf",
        }
    }

    /// The file extension, without dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Epub => "epub",
            Self::Pdf => "pdf",
        }
    }

    /// The kind for a media type we serve.
    #[must_use]
    pub fn from_media_type(media_type: &str) -> Option<Self> {
        match media_type {
            "application/epub+zip" => Some(Self::Epub),
            "application/pdf" => Some(Self::Pdf),
            _ => None,
        }
    }
}

/// Why an upload is refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum UploadError {
    /// The body is empty.
    #[error("the file is empty")]
    Empty,
    /// Neither an EPUB nor a PDF, whatever the headers said.
    #[error("only EPUB and PDF files are accepted (the content is neither)")]
    UnsupportedType,
    /// Starts like an EPUB but the container cannot be read.
    #[error("the EPUB container is damaged: {0}")]
    InvalidEpub(String),
    /// The file is protected by DRM and could not be opened by the recipient.
    #[error("the file is protected by DRM ({0}); only unprotected files can be gifted here")]
    DrmProtected(String),
}

/// Offset of the file name in the first local file header of a ZIP archive.
const ZIP_NAME_OFFSET: usize = 30;
/// The media type an OCF container declares in its first entry.
const EPUB_MIMETYPE: &[u8] = b"application/epub+zip";
/// Font obfuscation is not DRM: IDPF and Adobe algorithms (EPUB OCF §4.4; Adobe).
const FONT_OBFUSCATION: [&str; 2] =
    ["http://www.idpf.org/2008/embedding", "http://ns.adobe.com/pdf/enc#RC"];
/// Most bytes read from one metadata entry of the container.
const MAX_META_BYTES: u64 = 1 << 20;

/// Identifies an uploaded file from its bytes and refuses what cannot be gifted.
///
/// * **EPUB** — a ZIP archive whose *first* entry is named `mimetype`, is stored
///   uncompressed and contains `application/epub+zip` (the OCF rule that makes the type
///   sniffable at a fixed offset). The archive must be readable and must not carry DRM:
///   `META-INF/rights.xml` (Adobe ADEPT, and others), an embedded Readium LCP licence
///   (`META-INF/license.lcpl`), or a `META-INF/encryption.xml` that uses anything but font
///   obfuscation (or that cannot be parsed, since DRM cannot then be ruled out).
/// * **PDF** — starts with `%PDF-`. A PDF whose trailer region carries an `/Encrypt`
///   entry is refused as protected. This is a heuristic over the first and last 4 kB,
///   not a PDF parser.
///
/// Never panics on hostile input and reads at most 1 MiB out of any archive entry.
///
/// # Errors
/// See [`UploadError`].
pub fn sniff(bytes: &[u8]) -> Result<FileKind, UploadError> {
    if bytes.is_empty() {
        return Err(UploadError::Empty);
    }
    if bytes.starts_with(b"%PDF-") {
        return sniff_pdf(bytes);
    }
    if looks_like_epub(bytes) {
        return sniff_epub(bytes);
    }
    Err(UploadError::UnsupportedType)
}

/// The fixed-offset OCF signature: local file header, method 0, name `mimetype`, content.
fn looks_like_epub(bytes: &[u8]) -> bool {
    let header_ok = bytes.starts_with(b"PK\x03\x04")
        // compression method (offset 8, little endian) = 0: stored
        && bytes.get(8..10) == Some(&[0, 0])
        // file name length (offset 26) = 8
        && bytes.get(26..28) == Some(&[8, 0])
        && bytes.get(ZIP_NAME_OFFSET..ZIP_NAME_OFFSET + 8) == Some(b"mimetype");
    if !header_ok {
        return false;
    }
    // The OCF forbids an extra field here, but some tools write one: skip it.
    let extra = bytes.get(28..30).map_or(0, |b| usize::from(u16::from_le_bytes([b[0], b[1]])));
    let start = ZIP_NAME_OFFSET + 8 + extra;
    bytes.get(start..start + EPUB_MIMETYPE.len()) == Some(EPUB_MIMETYPE)
}

fn sniff_epub(bytes: &[u8]) -> Result<FileKind, UploadError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| UploadError::InvalidEpub(e.to_string()))?;
    let names: Vec<Vec<u8>> = archive.file_names_raw().map(<[u8]>::to_vec).collect();
    let has = |wanted: &str| names.iter().any(|name| name.eq_ignore_ascii_case(wanted.as_bytes()));

    if has("META-INF/rights.xml") {
        return Err(UploadError::DrmProtected("META-INF/rights.xml".to_owned()));
    }
    if has("META-INF/license.lcpl") {
        return Err(UploadError::DrmProtected("Readium LCP licence".to_owned()));
    }
    if has("META-INF/encryption.xml") {
        let name = names
            .iter()
            .find(|name| name.eq_ignore_ascii_case(b"META-INF/encryption.xml"))
            .and_then(|name| std::str::from_utf8(name).ok())
            .unwrap_or("META-INF/encryption.xml")
            .to_owned();
        let mut xml = String::new();
        let entry = archive.by_name(&name).map_err(|e| UploadError::InvalidEpub(e.to_string()))?;
        entry
            .take(MAX_META_BYTES)
            .read_to_string(&mut xml)
            .map_err(|_| UploadError::DrmProtected("unreadable encryption.xml".to_owned()))?;
        check_encryption_xml(&xml)?;
    }
    if !has("META-INF/container.xml") {
        return Err(UploadError::InvalidEpub("META-INF/container.xml is missing".to_owned()));
    }
    Ok(FileKind::Epub)
}

/// Accepts an `encryption.xml` only if every encrypted resource uses font obfuscation.
fn check_encryption_xml(xml: &str) -> Result<(), UploadError> {
    let options = roxmltree::ParsingOptions { allow_dtd: false, nodes_limit: 200_000, ..Default::default() };
    let document = roxmltree::Document::parse_with_options(xml, options)
        .map_err(|_| UploadError::DrmProtected("unreadable encryption.xml".to_owned()))?;
    let elements = || document.descendants().filter(roxmltree::Node::is_element);
    let mut encrypted = 0usize;
    let mut methods = 0usize;
    for node in elements() {
        match node.tag_name().name() {
            "EncryptedData" => encrypted += 1,
            // Only the method of the resource itself matters; `EncryptedKey` (key
            // transport) never appears with font obfuscation and is DRM by itself.
            "EncryptedKey" => {
                return Err(UploadError::DrmProtected("encrypted content key".to_owned()));
            }
            "EncryptionMethod" => {
                methods += 1;
                let algorithm = node.attribute("Algorithm").unwrap_or("");
                if !FONT_OBFUSCATION.contains(&algorithm) {
                    let shown: String = algorithm.chars().take(80).collect();
                    return Err(UploadError::DrmProtected(format!("encryption algorithm {shown:?}")));
                }
            }
            _ => {}
        }
    }
    if methods < encrypted {
        return Err(UploadError::DrmProtected("encrypted resource without a declared method".to_owned()));
    }
    Ok(())
}

fn sniff_pdf(bytes: &[u8]) -> Result<FileKind, UploadError> {
    const WINDOW: usize = 4096;
    let head = &bytes[..bytes.len().min(WINDOW)];
    let tail = &bytes[bytes.len().saturating_sub(WINDOW)..];
    let encrypted = |region: &[u8]| region.windows(8).any(|w| w == b"/Encrypt");
    if encrypted(head) || encrypted(tail) {
        return Err(UploadError::DrmProtected("encrypted PDF".to_owned()));
    }
    Ok(FileKind::Pdf)
}

/// What a cover image turned out to be (`PUT …/cover` accepts JPEG, PNG and WebP).
///
/// Returns the media type, or `None` for anything else.
#[must_use]
pub fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some("image/png")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some("image/webp")
    } else {
        None
    }
}

/// Decodes the `X-File-Name` header (percent-encoded UTF-8) into a safe plain file name:
/// the last path component, without control characters, trimmed, at most 120 characters,
/// with the extension forced to match the sniffed kind. Returns `None` when nothing
/// usable is left (the caller then derives a name from the book title).
#[must_use]
pub fn decode_file_name(header: &str, kind: FileKind) -> Option<String> {
    let decoded = percent_encoding::percent_decode_str(header).decode_utf8().ok()?;
    let last = decoded.rsplit(['/', '\\']).next().unwrap_or("");
    let cleaned: String = last.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    let stem = match cleaned.rsplit_once('.') {
        Some((stem, ext)) if ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphanumeric()) => stem,
        _ => cleaned,
    };
    let stem: String = stem.trim().chars().take(120).collect();
    let stem = stem.trim();
    if stem.is_empty() {
        return None;
    }
    Some(format!("{stem}.{}", kind.extension()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod testing {
    //! EPUB fixtures built in memory for tests.
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    pub(crate) const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#;

    /// Builds a ZIP from `(name, content, deflate)` entries, in order.
    pub(crate) fn zip_of(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, content, deflate) in entries {
            let method = if *deflate { CompressionMethod::Deflated } else { CompressionMethod::Stored };
            zip.start_file(*name, SimpleFileOptions::DEFAULT.compression_method(method)).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    /// A minimal valid EPUB with the given extra entries.
    pub(crate) fn epub_with(extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut entries: Vec<(&str, &[u8], bool)> = vec![
            ("mimetype", b"application/epub+zip", false),
            ("META-INF/container.xml", CONTAINER_XML.as_bytes(), true),
            ("OEBPS/package.opf", b"<package/>", true),
            ("OEBPS/chapter-1.xhtml", b"<html><body><p>Il etait une fois.</p></body></html>", true),
        ];
        for (name, content) in extra {
            entries.push((name, content, true));
        }
        zip_of(&entries)
    }

    /// A minimal valid EPUB.
    pub(crate) fn epub() -> Vec<u8> {
        epub_with(&[])
    }

    /// A minimal PDF-looking file.
    pub(crate) fn pdf() -> Vec<u8> {
        b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n".to_vec()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::testing::{CONTAINER_XML, epub, epub_with, pdf, zip_of};
    use super::*;

    #[test]
    fn epub_and_pdf_are_recognised_from_their_bytes() {
        assert_eq!(sniff(&epub()), Ok(FileKind::Epub));
        assert_eq!(sniff(&pdf()), Ok(FileKind::Pdf));
        assert_eq!(FileKind::Epub.media_type(), "application/epub+zip");
        assert_eq!(FileKind::Pdf.media_type(), "application/pdf");
        assert_eq!(FileKind::from_media_type("application/epub+zip"), Some(FileKind::Epub));
        assert_eq!(FileKind::from_media_type("text/html"), None);
    }

    #[test]
    fn other_content_is_refused_whatever_it_claims() {
        let plain_zip = zip_of(&[("hello.txt", b"hello", true)]);
        let docx_like = zip_of(&[
            ("[Content_Types].xml", b"<Types/>", true),
            ("mimetype", b"application/epub+zip", false),
        ]);
        let wrong_mimetype = zip_of(&[
            ("mimetype", b"application/vnd.oasis.opendocument.text", false),
            ("META-INF/container.xml", CONTAINER_XML.as_bytes(), true),
        ]);
        let compressed_mimetype = zip_of(&[
            ("mimetype", b"application/epub+zip", true),
            ("META-INF/container.xml", CONTAINER_XML.as_bytes(), true),
        ]);
        let html = b"<!DOCTYPE html><html><script>alert(1)</script></html>".to_vec();
        let exe = b"MZ\x90\x00\x03\x00\x00\x00".to_vec();
        let late_pdf = b"junk before %PDF-1.7".to_vec();
        for (what, bytes) in [
            ("plain zip", plain_zip),
            ("mimetype not first", docx_like),
            ("wrong mimetype", wrong_mimetype),
            ("compressed mimetype", compressed_mimetype),
            ("html", html),
            ("exe", exe),
            ("pdf marker not at start", late_pdf),
            ("mobi", b"BOOKMOBI".to_vec()),
            ("one byte", vec![0x50]),
        ] {
            assert_eq!(sniff(&bytes), Err(UploadError::UnsupportedType), "{what}");
        }
        assert_eq!(sniff(b""), Err(UploadError::Empty));
    }

    #[test]
    fn damaged_epubs_are_refused_without_panicking() {
        let good = epub();
        // Every truncation is either refused or (never) accepted as something else.
        for cut in (0..good.len()).step_by(7) {
            let result = sniff(&good[..cut]);
            assert!(result.is_err(), "truncated at {cut}: {result:?}");
        }
        // The right signature followed by garbage.
        let mut garbage = good[..58].to_vec();
        garbage.extend(std::iter::repeat_n(0xAAu8, 500));
        assert!(matches!(sniff(&garbage), Err(UploadError::InvalidEpub(_))));
        // A container without container.xml is not an EPUB we can serve.
        let no_container = zip_of(&[("mimetype", b"application/epub+zip", false)]);
        assert!(matches!(sniff(&no_container), Err(UploadError::InvalidEpub(_))));
        // Flipping bytes anywhere never panics.
        for position in (0..good.len()).step_by(11) {
            let mut mutated = good.clone();
            mutated[position] ^= 0xff;
            let _ = sniff(&mutated);
        }
    }

    const ADEPT_ENCRYPTION: &str = r#"<?xml version="1.0"?>
<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <EncryptedData xmlns="http://www.w3.org/2001/04/xmlenc#">
    <EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes128-cbc"/>
    <CipherData><CipherReference URI="OEBPS/chapter-1.xhtml"/></CipherData>
  </EncryptedData>
</encryption>"#;

    const LCP_ENCRYPTION: &str = r#"<?xml version="1.0"?>
<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#">
  <enc:EncryptedData>
    <enc:EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes256-cbc"/>
    <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#"><ds:RetrievalMethod URI="license.lcpl#/encryption/content_key" Type="http://readium.org/2014/01/lcp#EncryptedContentKey"/></ds:KeyInfo>
    <enc:CipherData><enc:CipherReference URI="OEBPS/chapter-1.xhtml"/></enc:CipherData>
  </enc:EncryptedData>
</encryption>"#;

    const FONT_OBFUSCATION_ONLY: &str = r#"<?xml version="1.0"?>
<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#">
  <enc:EncryptedData>
    <enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/>
    <enc:CipherData><enc:CipherReference URI="OEBPS/fonts/a.otf"/></enc:CipherData>
  </enc:EncryptedData>
  <enc:EncryptedData>
    <enc:EncryptionMethod Algorithm="http://ns.adobe.com/pdf/enc#RC"/>
    <enc:CipherData><enc:CipherReference URI="OEBPS/fonts/b.otf"/></enc:CipherData>
  </enc:EncryptedData>
</encryption>"#;

    #[test]
    fn drm_is_refused_but_font_obfuscation_is_not() {
        let drm = |extra: &[(&str, &[u8])]| matches!(sniff(&epub_with(extra)), Err(UploadError::DrmProtected(_)));
        assert!(drm(&[("META-INF/encryption.xml", ADEPT_ENCRYPTION.as_bytes())]));
        assert!(drm(&[("META-INF/encryption.xml", LCP_ENCRYPTION.as_bytes())]));
        assert!(drm(&[("META-INF/rights.xml", b"<adept:rights xmlns:adept=\"http://ns.adobe.com/adept\"/>")]));
        assert!(drm(&[("META-INF/license.lcpl", b"{}")]));
        assert!(drm(&[("meta-inf/RIGHTS.XML", b"<rights/>")]), "names are matched case-insensitively");
        // rights.xml wins even next to a harmless encryption.xml.
        assert!(drm(&[
            ("META-INF/encryption.xml", FONT_OBFUSCATION_ONLY.as_bytes()),
            ("META-INF/rights.xml", b"<rights/>"),
        ]));
        assert_eq!(
            sniff(&epub_with(&[("META-INF/encryption.xml", FONT_OBFUSCATION_ONLY.as_bytes())])),
            Ok(FileKind::Epub)
        );
        // An empty encryption.xml protects nothing.
        assert_eq!(
            sniff(&epub_with(&[(
                "META-INF/encryption.xml",
                b"<encryption xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\"/>"
            )])),
            Ok(FileKind::Epub)
        );
    }

    #[test]
    fn encryption_xml_that_cannot_be_vetted_counts_as_drm() {
        let drm = |xml: &str| {
            matches!(
                sniff(&epub_with(&[("META-INF/encryption.xml", xml.as_bytes())])),
                Err(UploadError::DrmProtected(_))
            )
        };
        // Mixed: one obfuscated font, one encrypted chapter.
        let mixed = FONT_OBFUSCATION_ONLY.replacen(
            "http://ns.adobe.com/pdf/enc#RC",
            "http://www.w3.org/2001/04/xmlenc#aes256-cbc",
            1,
        );
        assert!(drm(&mixed));
        assert!(drm("<encryption><EncryptedData><CipherData/></EncryptedData></encryption>"));
        assert!(drm("<encryption><EncryptedData><EncryptionMethod/></EncryptedData></encryption>"));
        assert!(drm("<encryption><EncryptedKey/></encryption>"));
        assert!(drm("this is not xml"));
        assert!(drm("<encryption><EncryptedData>"));
        assert!(drm("<!DOCTYPE x [<!ENTITY a \"aaaa\">]><encryption>&a;</encryption>"));
        assert!(drm("\u{feff}\u{0}\u{1}"));
        // A look-alike algorithm URI is not font obfuscation.
        assert!(drm(&FONT_OBFUSCATION_ONLY.replace("2008/embedding", "2008/embedding/")));
    }

    #[test]
    fn encrypted_pdfs_are_refused() {
        let mut encrypted = pdf();
        let at = encrypted.len() - 8;
        encrypted.splice(at..at, b"/Encrypt 5 0 R ".iter().copied());
        assert!(matches!(sniff(&encrypted), Err(UploadError::DrmProtected(_))));
        // The word in the middle of a large document body is not a trailer entry.
        let mut body = b"%PDF-1.7\n".to_vec();
        body.extend(std::iter::repeat_n(b' ', 6_000));
        body.extend_from_slice(b"(how to /Encrypt a file)");
        body.extend(std::iter::repeat_n(b' ', 6_000));
        body.extend_from_slice(b"\n%%EOF\n");
        assert_eq!(sniff(&body), Ok(FileKind::Pdf));
        assert_eq!(sniff(b"%PDF-"), Ok(FileKind::Pdf));
    }

    #[test]
    fn images_are_sniffed() {
        assert_eq!(sniff_image(&[0xff, 0xd8, 0xff, 0xe0, 0, 0]), Some("image/jpeg"));
        assert_eq!(sniff_image(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"), Some("image/png"));
        assert_eq!(sniff_image(b"RIFF\x24\0\0\0WEBPVP8 "), Some("image/webp"));
        for other in [&b"GIF89a"[..], b"RIFF\x24\0\0\0WAVE", b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>", b"", b"\xff\xd8"] {
            assert_eq!(sniff_image(other), None);
        }
    }

    #[test]
    fn rights_basis_values() {
        for basis in RightsBasis::ALL {
            assert_eq!(RightsBasis::parse(basis.as_str()), Some(basis));
            assert_eq!(serde_json::to_value(basis).unwrap(), basis.as_str());
        }
        assert_eq!(RightsBasis::parse(" public_domain "), Some(RightsBasis::PublicDomain));
        for bad in ["", "PUBLIC_DOMAIN", "purchased", "own work", "own_work,public_domain"] {
            assert_eq!(RightsBasis::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn file_names_are_decoded_and_made_safe() {
        let name = |header: &str| decode_file_name(header, FileKind::Epub);
        assert_eq!(name("L%27%C3%89tranger.epub").as_deref(), Some("L'Étranger.epub"));
        assert_eq!(name("le-horla.EPUB").as_deref(), Some("le-horla.epub"));
        assert_eq!(name("book").as_deref(), Some("book.epub"));
        assert_eq!(name("book.pdf").as_deref(), Some("book.epub"), "extension follows the content");
        assert_eq!(name("..%2F..%2Fetc%2Fpasswd").as_deref(), Some("passwd.epub"));
        assert_eq!(name("C%3A%5CUsers%5Cme%5Clivre.epub").as_deref(), Some("livre.epub"));
        assert_eq!(name("a%0D%0AContent-Type%3A%20x.epub").as_deref(), Some("aContent-Type: x.epub"));
        assert_eq!(name("  spaced  .epub").as_deref(), Some("spaced.epub"));
        assert_eq!(name("v1.2 final.epub").as_deref(), Some("v1.2 final.epub"));
        for unusable in ["", ".", "..", "%2F", ".epub", "%00", "%FF%FE", "   "] {
            assert_eq!(name(unusable), None, "{unusable:?}");
        }
        let long = name(&"é".repeat(500)).unwrap();
        assert_eq!(long.chars().count(), 120 + ".epub".len());
        assert_eq!(decode_file_name("x.epub", FileKind::Pdf).as_deref(), Some("x.pdf"));
    }
}
