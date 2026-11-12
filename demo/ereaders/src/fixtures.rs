//! Tiny, deterministic book files for tests and demos.
//!
//! These are **synthetic** files with a few lines of placeholder text — just enough structure
//! for a reading system to accept them (and, through [`EpubBuilder`]'s knobs, to reject them).
//! Output is byte-for-byte reproducible: no clock, no randomness, no compression.

/// CRC-32 (IEEE 802.3, reflected) as used by ZIP.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

struct ZipEntry {
    name: String,
    data: Vec<u8>,
    /// Compression method written in the headers (the data is always stored as-is).
    method: u16,
}

/// Writes a ZIP archive whose entries are all stored (method 0 unless overridden for a test).
fn write_zip(entries: &[ZipEntry]) -> Vec<u8> {
    const DOS_DATE_1980_01_01: u16 = 0x0021;
    let mut out = Vec::new();
    let mut central = Vec::new();
    for entry in entries {
        let offset = u32::try_from(out.len()).unwrap_or(u32::MAX);
        let crc = crc32(&entry.data);
        let size = u32::try_from(entry.data.len()).unwrap_or(u32::MAX);
        let name = entry.name.as_bytes();
        let name_len = u16::try_from(name.len()).unwrap_or(u16::MAX);
        let flags: u16 = if entry.name.is_ascii() { 0 } else { 0x0800 };

        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&entry.method.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&DOS_DATE_1980_01_01.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&name_len.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&entry.data);

        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&flags.to_le_bytes());
        central.extend_from_slice(&entry.method.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&DOS_DATE_1980_01_01.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&name_len.to_le_bytes());
        // extra len (2), comment len (2), disk start (2), internal attrs (2), external attrs (4)
        central.extend_from_slice(&[0u8; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }
    let central_offset = u32::try_from(out.len()).unwrap_or(u32::MAX);
    let central_size = u32::try_from(central.len()).unwrap_or(u32::MAX);
    let count = u16::try_from(entries.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Builds a minimal EPUB 3, with switches that break it in the ways real files break.
#[derive(Debug, Clone)]
pub struct EpubBuilder {
    title: String,
    author: String,
    language: String,
    mimetype_first: bool,
    mimetype_content: String,
    mimetype_method: u16,
    container: bool,
    with_title: bool,
    adobe_rights: bool,
    lcp_marker: bool,
    encryption_algorithm: Option<String>,
    padding: usize,
}

impl EpubBuilder {
    /// A valid, DRM-free EPUB with the given title and author (language `fr`).
    pub fn new(title: &str, author: &str) -> Self {
        Self {
            title: title.to_owned(),
            author: author.to_owned(),
            language: "fr".to_owned(),
            mimetype_first: true,
            mimetype_content: "application/epub+zip".to_owned(),
            mimetype_method: 0,
            container: true,
            with_title: true,
            adobe_rights: false,
            lcp_marker: false,
            encryption_algorithm: None,
            padding: 0,
        }
    }

    /// Sets `dc:language`.
    pub fn language(mut self, language: &str) -> Self {
        self.language = language.to_owned();
        self
    }

    /// `false` moves the `mimetype` entry after `META-INF/container.xml` (invalid OCF).
    pub fn mimetype_first(mut self, first: bool) -> Self {
        self.mimetype_first = first;
        self
    }

    /// Overrides the content of the `mimetype` entry (invalid unless `application/epub+zip`).
    pub fn mimetype_content(mut self, content: &str) -> Self {
        self.mimetype_content = content.to_owned();
        self
    }

    /// Overrides the ZIP compression method declared for `mimetype` (anything but 0 is invalid).
    pub fn mimetype_method(mut self, method: u16) -> Self {
        self.mimetype_method = method;
        self
    }

    /// Omits `META-INF/container.xml`.
    pub fn without_container(mut self) -> Self {
        self.container = false;
        self
    }

    /// Omits `dc:title` from the package document.
    pub fn without_title(mut self) -> Self {
        self.with_title = false;
        self
    }

    /// Adds `META-INF/rights.xml`, the marker of Adobe ADEPT DRM. **Nothing is encrypted.**
    pub fn with_adobe_rights(mut self) -> Self {
        self.adobe_rights = true;
        self
    }

    /// Adds `META-INF/license.lcpl` and an `encryption.xml`, the markers of an LCP-protected
    /// publication. **Nothing is encrypted** — this only lets the simulation tell "protected"
    /// publications from open ones.
    pub fn with_lcp_marker(mut self) -> Self {
        self.lcp_marker = true;
        self
    }

    /// Adds a `META-INF/encryption.xml` declaring one resource with this algorithm URI.
    pub fn with_encryption_algorithm(mut self, algorithm: &str) -> Self {
        self.encryption_algorithm = Some(algorithm.to_owned());
        self
    }

    /// Appends an entry of `bytes` zero bytes, to reach a given file size.
    pub fn padding(mut self, bytes: usize) -> Self {
        self.padding = bytes;
        self
    }

    /// Serialises the EPUB.
    pub fn build(&self) -> Vec<u8> {
        let stored =
            |name: &str, data: Vec<u8>| ZipEntry { name: name.to_owned(), data, method: 0 };
        let mimetype = ZipEntry {
            name: "mimetype".to_owned(),
            data: self.mimetype_content.clone().into_bytes(),
            method: self.mimetype_method,
        };
        let mut entries = Vec::new();
        if self.mimetype_first {
            entries.push(mimetype);
        }
        if self.container {
            entries.push(stored(
                "META-INF/container.xml",
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                    "<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n",
                    "  <rootfiles>\n",
                    "    <rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>\n",
                    "  </rootfiles>\n",
                    "</container>\n"
                )
                .as_bytes()
                .to_vec(),
            ));
        }
        if !self.mimetype_first {
            entries.push(ZipEntry {
                name: "mimetype".to_owned(),
                data: self.mimetype_content.clone().into_bytes(),
                method: self.mimetype_method,
            });
        }
        let title = if self.with_title {
            format!("    <dc:title>{}</dc:title>\n", xml_escape(&self.title))
        } else {
            String::new()
        };
        let opf = format!(
            concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                "<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\">\n",
                "  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n",
                "    <dc:identifier id=\"uid\">urn:ebc-demo:{id:08x}</dc:identifier>\n",
                "{title}",
                "    <dc:creator>{author}</dc:creator>\n",
                "    <dc:language>{language}</dc:language>\n",
                "    <meta property=\"dcterms:modified\">2026-01-01T00:00:00Z</meta>\n",
                "  </metadata>\n",
                "  <manifest>\n",
                "    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n",
                "    <item id=\"c1\" href=\"chapter1.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
                "  </manifest>\n",
                "  <spine>\n",
                "    <itemref idref=\"c1\"/>\n",
                "  </spine>\n",
                "</package>\n"
            ),
            id = crc32(format!("{}\n{}", self.title, self.author).as_bytes()),
            title = title,
            author = xml_escape(&self.author),
            language = xml_escape(&self.language),
        );
        entries.push(stored("OEBPS/content.opf", opf.into_bytes()));
        entries.push(stored(
            "OEBPS/nav.xhtml",
            format!(
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                    "<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\">\n",
                    "<head><title>{title}</title></head>\n",
                    "<body><nav epub:type=\"toc\"><ol><li><a href=\"chapter1.xhtml\">1</a></li></ol></nav></body>\n",
                    "</html>\n"
                ),
                lang = xml_escape(&self.language),
                title = xml_escape(&self.title)
            )
            .into_bytes(),
        ));
        entries.push(stored(
            "OEBPS/chapter1.xhtml",
            format!(
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                    "<html xmlns=\"http://www.w3.org/1999/xhtml\" lang=\"{lang}\">\n",
                    "<head><title>{title}</title></head>\n",
                    "<body><h1>{title}</h1><p>Demo file generated for the eBook Cadeau test suite. ",
                    "It contains no real book.</p></body>\n",
                    "</html>\n"
                ),
                lang = xml_escape(&self.language),
                title = xml_escape(&self.title)
            )
            .into_bytes(),
        ));
        if self.adobe_rights {
            entries.push(stored(
                "META-INF/rights.xml",
                b"<?xml version=\"1.0\"?><adept:rights xmlns:adept=\"http://ns.adobe.com/adept\"/>"
                    .to_vec(),
            ));
        }
        let algorithm = match (&self.encryption_algorithm, self.lcp_marker) {
            (Some(algorithm), _) => Some(algorithm.clone()),
            (None, true) => Some("http://www.w3.org/2001/04/xmlenc#aes256-cbc".to_owned()),
            (None, false) => None,
        };
        if let Some(algorithm) = algorithm {
            let key_info = if self.lcp_marker {
                "<ds:KeyInfo xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\"><ds:RetrievalMethod URI=\"license.lcpl#/encryption/content_key\"/></ds:KeyInfo>"
            } else {
                ""
            };
            entries.push(stored(
                "META-INF/encryption.xml",
                format!(
                    concat!(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                        "<encryption xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\" xmlns:enc=\"http://www.w3.org/2001/04/xmlenc#\">\n",
                        "  <enc:EncryptedData><enc:EncryptionMethod Algorithm=\"{algorithm}\"/>{key_info}",
                        "<enc:CipherData><enc:CipherReference URI=\"OEBPS/chapter1.xhtml\"/></enc:CipherData></enc:EncryptedData>\n",
                        "</encryption>\n"
                    ),
                    algorithm = xml_escape(&algorithm),
                    key_info = key_info
                )
                .into_bytes(),
            ));
        }
        if self.lcp_marker {
            entries.push(stored(
                "META-INF/license.lcpl",
                b"{\"note\":\"SIMULATION - placeholder, not a real LCP licence\"}".to_vec(),
            ));
        }
        if self.padding > 0 {
            entries.push(stored("OEBPS/padding.bin", vec![0u8; self.padding]));
        }
        write_zip(&entries)
    }
}

/// A valid, DRM-free demo EPUB.
pub fn epub(title: &str, author: &str) -> Vec<u8> {
    EpubBuilder::new(title, author).build()
}

/// A minimal one-page PDF whose Info dictionary carries `title`.
pub fn pdf(title: &str) -> Vec<u8> {
    let escaped: String = title
        .chars()
        .flat_map(|c| match c {
            '(' | ')' | '\\' => vec!['\\', c],
            c if c.is_ascii() && !c.is_ascii_control() => vec![c],
            _ => vec!['?'],
        })
        .collect();
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 420 595] >>".to_owned(),
        format!("<< /Title ({escaped}) /Producer (eBook Cadeau demo fixture) >>"),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info 4 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// A PalmDB shell with the `BOOKMOBI` signature — enough for format detection, **not** a
/// readable Mobipocket/KF8 book.
pub fn mobi(title: &str) -> Vec<u8> {
    let mut out = vec![0u8; 128];
    let name: Vec<u8> =
        title.bytes().map(|b| if b.is_ascii_graphic() { b } else { b'_' }).take(31).collect();
    out[..name.len()].copy_from_slice(&name);
    out[60..68].copy_from_slice(b"BOOKMOBI");
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn crc32_matches_the_reference_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn fixture_epub_is_a_zip_other_tools_can_read() {
        let bytes = epub("Vingt mille lieues sous les mers", "Jules Verne");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes.as_slice())).unwrap();
        assert_eq!(archive.name_for_index(0).unwrap().unwrap(), "mimetype");
        let mut mimetype = String::new();
        archive.by_name("mimetype").unwrap().read_to_string(&mut mimetype).unwrap();
        assert_eq!(mimetype, "application/epub+zip");
        let mut opf = String::new();
        archive.by_name("OEBPS/content.opf").unwrap().read_to_string(&mut opf).unwrap();
        assert!(opf.contains("<dc:title>Vingt mille lieues sous les mers</dc:title>"));
    }

    #[test]
    fn fixtures_are_deterministic_and_escape_markup() {
        assert_eq!(epub("A", "B"), epub("A", "B"));
        assert_ne!(epub("A", "B"), epub("A", "C"));
        let tricky = epub("Tom & <Jerry>", "O\"Neil");
        let info = crate::format::inspect(crate::format::FileFormat::Epub, &tricky).unwrap();
        assert_eq!(info.title.as_deref(), Some("Tom & <Jerry>"));
        assert_eq!(info.authors, vec!["O\"Neil".to_owned()]);
        assert_eq!(pdf("x"), pdf("x"));
    }

    #[test]
    fn padding_reaches_the_requested_size() {
        let small = epub("A", "B").len();
        let big = EpubBuilder::new("A", "B").padding(100_000).build();
        assert!(big.len() > small + 100_000);
        assert!(crate::format::inspect(crate::format::FileFormat::Epub, &big).is_ok());
    }
}
