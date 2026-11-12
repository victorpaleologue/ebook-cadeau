//! The device's library: what actually ended up readable on the e-reader.
//!
//! Every delivery channel (browser download, mail-to-device, account sync, OPDS, LCP) ends in
//! [`Library::import`], which applies the same checks a reading system would: is this a format
//! I open, is the container sound, is it locked by a DRM I cannot unlock.

use sha2::{Digest, Sha256};

use crate::error::ImportError;
use crate::format::{self, FileFormat};
use crate::profile::{DeviceProfile, DrmScheme};

/// How a file reached the device.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportSource {
    /// Saved by the built-in browser.
    BrowserDownload {
        /// Final URL of the file.
        url: String,
    },
    /// Delivered by a mail-to-device service.
    Mail {
        /// Sender address.
        from: String,
        /// Service id (`send-to-kindle`…).
        service: &'static str,
    },
    /// Pulled by account sync.
    Sync {
        /// The store's identifier of the book.
        id: String,
    },
    /// Downloaded from an OPDS acquisition link.
    Opds {
        /// The acquisition URL.
        url: String,
    },
    /// Fetched by the **simulated** LCP reader after the passphrase check.
    SimulatedLcp {
        /// Licence id.
        licence_id: String,
    },
    /// Copied over USB (not simulated; available for tests that pre-load a library).
    Sideload,
}

/// One book on the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    /// Title from the file's metadata, or the file name stem when it has none.
    pub title: String,
    /// Authors from the file's metadata.
    pub authors: Vec<String>,
    /// Language from the file's metadata.
    pub language: Option<String>,
    /// Format as stored on the device.
    pub format: FileFormat,
    /// Format of the file as it was sent (differs after a service-side conversion).
    pub original_format: FileFormat,
    /// File name on the device.
    pub file_name: String,
    /// Size of the received file.
    pub size_bytes: u64,
    /// Lower-case hex SHA-256 of the received bytes.
    pub sha256: String,
    /// How it got here.
    pub source: ImportSource,
    /// DRM scheme the file is (notionally) protected with.
    pub drm: Option<DrmScheme>,
    /// `true` for `.kepub.epub` on a Kobo (rendered by the Kobo WebKit engine).
    pub kepub: bool,
    /// Remarks a person would notice ("no cover", "accents may be garbled"…).
    pub notes: Vec<String>,
}

/// A file arriving on the device.
#[derive(Debug, Clone, Copy)]
pub struct IncomingFile<'a> {
    /// The bytes.
    pub bytes: &'a [u8],
    /// The name it arrives under.
    pub file_name: &'a str,
    /// What the sender said it is (`Content-Type` or attachment type), if anything usable.
    pub declared: Option<FileFormat>,
}

/// The books on a simulated device.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Library {
    entries: Vec<LibraryEntry>,
}

impl Library {
    /// An empty library.
    pub fn new() -> Self {
        Self::default()
    }

    /// All books, oldest first.
    pub fn entries(&self) -> &[LibraryEntry] {
        &self.entries
    }

    /// Number of books.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when there are no books.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The first book with exactly this title.
    pub fn find_by_title(&self, title: &str) -> Option<&LibraryEntry> {
        self.entries.iter().find(|e| e.title == title)
    }

    /// Whether a book with exactly this title is present.
    pub fn contains_title(&self, title: &str) -> bool {
        self.find_by_title(title).is_some()
    }

    /// Validates a file the way `profile`'s reading system would and adds it.
    ///
    /// Importing the same bytes twice does not create a duplicate: the existing entry is
    /// returned (a simplification — real devices differ here).
    pub fn import(
        &mut self,
        profile: &DeviceProfile,
        file: IncomingFile<'_>,
        source: ImportSource,
    ) -> Result<LibraryEntry, ImportError> {
        self.import_with(profile, file, source, |_| {})
    }

    /// [`Library::import`] with a last adjustment of the entry before it is stored.
    pub(crate) fn import_with(
        &mut self,
        profile: &DeviceProfile,
        file: IncomingFile<'_>,
        source: ImportSource,
        adjust: impl FnOnce(&mut LibraryEntry),
    ) -> Result<LibraryEntry, ImportError> {
        let (format, info) = examine(file, profile.max_download_bytes)?;
        if !profile.reads(format) {
            return Err(ImportError::UnsupportedFormat { format, device: profile.id });
        }
        if let Some(scheme) = info.drm {
            let via_lcp = matches!(source, ImportSource::SimulatedLcp { .. })
                && scheme == DrmScheme::ReadiumLcp
                && profile.unlocks(DrmScheme::ReadiumLcp);
            if !via_lcp {
                return Err(ImportError::DrmProtected { scheme });
            }
        }
        if profile.quirks.download_reported_damaged
            && matches!(source, ImportSource::BrowserDownload { .. })
        {
            return Err(ImportError::DamagedOnDevice { device: profile.id });
        }
        let mut notes = Vec::new();
        if matches!(source, ImportSource::BrowserDownload { .. }) {
            if let Some(note) = profile.quirks.download_note {
                notes.push(note.to_owned());
            }
        }
        let mut entry = build_entry(file, format, &info, source, notes);
        entry.kepub = entry.kepub && profile.family == crate::profile::DeviceFamily::Kobo;
        adjust(&mut entry);
        Ok(self.push_unique(entry))
    }

    /// Adds an entry produced by a delivery service that converts files on its side
    /// (Send to Kindle). The caller has already validated the original file.
    pub(crate) fn push_unique(&mut self, entry: LibraryEntry) -> LibraryEntry {
        if let Some(existing) =
            self.entries.iter().find(|e| e.sha256 == entry.sha256 && e.format == entry.format)
        {
            return existing.clone();
        }
        self.entries.push(entry.clone());
        entry
    }
}

/// Builds the library record of a validated file. `kepub` is set from the file name alone;
/// the caller clears it for devices that do not treat `.kepub.epub` specially.
pub(crate) fn build_entry(
    file: IncomingFile<'_>,
    format: FileFormat,
    info: &format::BookInfo,
    source: ImportSource,
    extra_notes: Vec<String>,
) -> LibraryEntry {
    let mut notes = info.warnings.clone();
    notes.extend(extra_notes);
    LibraryEntry {
        title: info.title.clone().unwrap_or_else(|| stem(file.file_name)),
        authors: info.authors.clone(),
        language: info.language.clone(),
        format,
        original_format: format,
        file_name: file.file_name.to_owned(),
        size_bytes: file.bytes.len() as u64,
        sha256: sha256_hex(file.bytes),
        source,
        drm: info.drm,
        kepub: format == FileFormat::Epub && format::is_kepub_name(file.file_name),
        notes,
    }
}

/// Determines what a file really is and validates its container.
///
/// The bytes win over the label: an HTML error page sent as `application/epub+zip` is reported
/// as a [`ImportError::ContentMismatch`], which is the failure a person would see as "the book
/// does not open".
pub(crate) fn examine(
    file: IncomingFile<'_>,
    max_bytes: u64,
) -> Result<(FileFormat, format::BookInfo), ImportError> {
    if file.bytes.is_empty() {
        return Err(ImportError::Empty);
    }
    let size = file.bytes.len() as u64;
    if size > max_bytes {
        return Err(ImportError::TooLarge { size, limit: max_bytes });
    }
    let sniffed = FileFormat::sniff(file.bytes);
    let named = FileFormat::from_file_name(file.file_name);
    let declared = file.declared.or(named);
    let format = match (sniffed, declared) {
        // AZW3 and MOBI share the PalmDB signature; the label decides between them.
        (Some(FileFormat::Mobi), Some(FileFormat::Azw3)) => FileFormat::Azw3,
        (Some(actual), Some(declared)) if actual != declared => {
            return Err(ImportError::ContentMismatch { declared, actual: Some(actual) });
        }
        (Some(actual), _) => actual,
        // Formats without a magic number are accepted on their label and validated below.
        (None, Some(label @ (FileFormat::Txt | FileFormat::Cbz | FileFormat::Fb2))) => label,
        (None, Some(declared)) => {
            return Err(ImportError::ContentMismatch { declared, actual: None });
        }
        (None, None) => return Err(ImportError::Unrecognized),
    };
    let info = format::inspect(format, file.bytes)?;
    Ok((format, info))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn stem(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    let cut = if lower.ends_with(".kepub.epub") {
        file_name.len() - ".kepub.epub".len()
    } else {
        file_name.rfind('.').unwrap_or(file_name.len())
    };
    let stem = file_name.get(..cut).unwrap_or(file_name);
    let pretty = stem.replace(['_', '-'], " ");
    let pretty = pretty.trim();
    if pretty.is_empty() { "Untitled".to_owned() } else { pretty.to_owned() }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::fixtures;

    fn incoming<'a>(bytes: &'a [u8], name: &'a str, declared: Option<FileFormat>) -> IncomingFile<'a> {
        IncomingFile { bytes, file_name: name, declared }
    }

    fn download() -> ImportSource {
        ImportSource::BrowserDownload { url: "http://ebk.example/d/s/livre.epub".to_owned() }
    }

    #[test]
    fn a_kobo_imports_an_epub_with_its_metadata() {
        let mut library = Library::new();
        let epub = fixtures::epub("Le Horla", "Guy de Maupassant");
        let entry = library
            .import(
                &DeviceProfile::kobo_clara(),
                incoming(&epub, "le-horla.kepub.epub", Some(FileFormat::Epub)),
                download(),
            )
            .unwrap();
        assert_eq!(entry.title, "Le Horla");
        assert_eq!(entry.authors, vec!["Guy de Maupassant".to_owned()]);
        assert_eq!(entry.format, FileFormat::Epub);
        assert!(entry.kepub);
        assert_eq!(entry.size_bytes, epub.len() as u64);
        assert_eq!(entry.sha256.len(), 64);
        assert_eq!(library.len(), 1);
        assert!(library.contains_title("Le Horla"));
        // Same bytes again: no duplicate.
        library
            .import(&DeviceProfile::kobo_clara(), incoming(&epub, "x.epub", None), download())
            .unwrap();
        assert_eq!(library.len(), 1);
    }

    #[test]
    fn a_kindle_refuses_epub_with_a_typed_error() {
        let mut library = Library::new();
        let epub = fixtures::epub("Le Horla", "Guy de Maupassant");
        let err = library
            .import(
                &DeviceProfile::kindle_paperwhite(),
                incoming(&epub, "le-horla.epub", Some(FileFormat::Epub)),
                ImportSource::Sideload,
            )
            .unwrap_err();
        assert_eq!(
            err,
            ImportError::UnsupportedFormat { format: FileFormat::Epub, device: "kindle-paperwhite" }
        );
        assert!(library.is_empty());
    }

    #[test]
    fn an_error_page_saved_as_epub_is_a_content_mismatch() {
        let mut library = Library::new();
        let html = b"<!DOCTYPE html><html><body><h1>503</h1></body></html>";
        let err = library
            .import(
                &DeviceProfile::kobo_clara(),
                incoming(html, "livre.epub", Some(FileFormat::Epub)),
                download(),
            )
            .unwrap_err();
        assert_eq!(
            err,
            ImportError::ContentMismatch {
                declared: FileFormat::Epub,
                actual: Some(FileFormat::Html)
            }
        );
        let err = library
            .import(
                &DeviceProfile::kobo_clara(),
                incoming(&[1, 2, 3, 4], "livre.epub", None),
                download(),
            )
            .unwrap_err();
        assert_eq!(err, ImportError::ContentMismatch { declared: FileFormat::Epub, actual: None });
        let err = library
            .import(&DeviceProfile::kobo_clara(), incoming(&[1, 2, 3, 4], "blob", None), download())
            .unwrap_err();
        assert_eq!(err, ImportError::Unrecognized);
        let err = library
            .import(&DeviceProfile::kobo_clara(), incoming(&[], "a.epub", None), download())
            .unwrap_err();
        assert_eq!(err, ImportError::Empty);
    }

    #[test]
    fn drm_wrapped_files_do_not_open_from_a_plain_download() {
        let mut library = Library::new();
        let adobe = fixtures::EpubBuilder::new("T", "A").with_adobe_rights().build();
        let err = library
            .import(&DeviceProfile::pocketbook(), incoming(&adobe, "t.epub", None), download())
            .unwrap_err();
        assert_eq!(err, ImportError::DrmProtected { scheme: DrmScheme::AdobeAdept });

        let lcp = fixtures::EpubBuilder::new("T", "A").with_lcp_marker().build();
        let err = library
            .import(&DeviceProfile::pocketbook(), incoming(&lcp, "t.epub", None), download())
            .unwrap_err();
        assert_eq!(err, ImportError::DrmProtected { scheme: DrmScheme::ReadiumLcp });
        // Through the simulated LCP channel, on an LCP-capable device, it is accepted…
        let entry = library
            .import(
                &DeviceProfile::pocketbook(),
                incoming(&lcp, "t.epub", None),
                ImportSource::SimulatedLcp { licence_id: "l1".to_owned() },
            )
            .unwrap();
        assert_eq!(entry.drm, Some(DrmScheme::ReadiumLcp));
        // …but not on a Kobo, which has no LCP.
        let mut kobo = Library::new();
        let err = kobo
            .import(
                &DeviceProfile::kobo_clara(),
                incoming(&lcp, "t.epub", None),
                ImportSource::SimulatedLcp { licence_id: "l1".to_owned() },
            )
            .unwrap_err();
        assert_eq!(err, ImportError::DrmProtected { scheme: DrmScheme::ReadiumLcp });
    }

    #[test]
    fn device_quirks_apply_to_browser_downloads_only() {
        let epub = fixtures::epub("T", "A");
        let legacy = DeviceProfile::tolino_legacy_android();
        let mut library = Library::new();
        let err =
            library.import(&legacy, incoming(&epub, "t.epub", None), download()).unwrap_err();
        assert_eq!(err, ImportError::DamagedOnDevice { device: "tolino-legacy-android" });
        library.import(&legacy, incoming(&epub, "t.epub", None), ImportSource::Sideload).unwrap();

        let azw3 = fixtures::mobi("Titre");
        let mut kindle = Library::new();
        let entry = kindle
            .import(
                &DeviceProfile::kindle_paperwhite(),
                incoming(&azw3, "titre.azw3", None),
                download(),
            )
            .unwrap();
        assert_eq!(entry.format, FileFormat::Azw3);
        assert!(entry.notes.iter().any(|n| n.contains("not in the Kindle library")));
    }

    #[test]
    fn size_limits_and_title_fallbacks() {
        let profile = DeviceProfile { max_download_bytes: 100, ..DeviceProfile::kobo_clara() };
        let epub = fixtures::epub("T", "A");
        let err = Library::new()
            .import(&profile, incoming(&epub, "t.epub", None), download())
            .unwrap_err();
        assert!(matches!(err, ImportError::TooLarge { limit: 100, .. }));

        let mut library = Library::new();
        let entry = library
            .import(
                &DeviceProfile::kobo_clara(),
                incoming(b"Bonjour, ceci est un texte.", "mes_notes-de-lecture.txt", None),
                ImportSource::Sideload,
            )
            .unwrap();
        assert_eq!(entry.title, "mes notes de lecture");
        assert_eq!(stem(".epub"), "Untitled");
        assert_eq!(stem("Livre.kepub.epub"), "Livre");
        assert_eq!(stem(""), "Untitled");
        assert_eq!(stem("sans extension"), "sans extension");
    }
}
