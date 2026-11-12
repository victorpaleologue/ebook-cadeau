//! Structural sample PDFs for tests and demos of the print provider API.
//!
//! [`SampleCard`] writes a small two-page PDF shaped like an eBook Cadeau card for a given
//! product — page boxes with bleed, CMYK vector artwork, a PDF/X-4 output intent carrying a
//! real (CC0) CMYK profile, and the `EBC.*` geometry keys — and can be told to break one
//! rule at a time, so that tests can check what the printer accepts and what it refuses.
//!
//! These files are **fixtures, not artwork**: flat colour panels without text. They exist
//! so that the printer (and the stores' tests, which need *some* printable PDF from a mock
//! eBook Cadeau server) do not depend on the real card composer.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::catalog::Product;

/// Points per millimetre.
const MM: f64 = 72.0 / 25.4;

/// CC0 CMYK profile from `assets/icc` (CGATS TR 001 / SWOP compatible).
const CMYK_PROFILE: &[u8] = include_bytes!("../../../assets/icc/CGATS001Compat-v2-micro.icc");

/// Side of the sample image, in pixels.
const IMAGE_PX: u32 = 60;

/// Description of a sample card PDF. Start from [`SampleCard::for_product`] (a file the
/// printer accepts) and change one field to get a file it refuses.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleCard {
    /// Trim size of the spread `(width, height)` in millimetres.
    pub trim_mm: (f64, f64),
    /// Spine width in millimetres (drawn, and declared in the `EBC.*` keys).
    pub spine_mm: f64,
    /// Bleed on every side in millimetres.
    pub bleed_mm: f64,
    /// Number of pages.
    pub pages: u32,
    /// Whether to include the PDF/X-4 output intent and version markers.
    pub output_intent: bool,
    /// Paints a patch in `DeviceRGB`.
    pub rgb_fill: bool,
    /// Paints a patch at 50 % opacity.
    pub transparency: bool,
    /// Paints a patch with 400 % total ink.
    pub heavy_ink: bool,
    /// Places a small CMYK image at this effective resolution (pixels per inch).
    pub image_ppi: Option<f64>,
    /// Info `Title`; `None` omits it.
    pub title: Option<String>,
    /// Whether to write the `EBC.*` geometry keys.
    pub geometry_keys: bool,
}

impl SampleCard {
    /// A card the printer accepts for `product`.
    #[must_use]
    pub fn for_product(product: &Product) -> Self {
        Self {
            trim_mm: (product.trim_mm[0], product.trim_mm[1]),
            spine_mm: product.spine_mm,
            bleed_mm: product.bleed_mm,
            pages: product.pages,
            output_intent: true,
            rgb_fill: false,
            transparency: false,
            heavy_ink: false,
            image_ppi: Some(300.0),
            title: Some(format!("Carte de démonstration — {}", product.id)),
            geometry_keys: true,
        }
    }

    /// Width of one cover panel in millimetres.
    fn cover_width(&self) -> f64 {
        (self.trim_mm.0 - self.spine_mm) / 2.0
    }

    /// Serialises the card as a PDF 1.6 file with a classic cross-reference table. The
    /// output is deterministic.
    #[must_use]
    pub fn to_pdf(&self) -> Vec<u8> {
        let mut w = Writer::new();
        let bleed = self.bleed_mm.max(0.0);
        let media_w = (self.trim_mm.0 + 2.0 * bleed) * MM;
        let media_h = (self.trim_mm.1 + 2.0 * bleed) * MM;
        let boxes = format!(
            "/MediaBox [0 0 {media_w:.4} {media_h:.4}] /BleedBox [0 0 {media_w:.4} {media_h:.4}] \
             /TrimBox [{:.4} {:.4} {:.4} {:.4}]",
            bleed * MM,
            bleed * MM,
            (bleed + self.trim_mm.0) * MM,
            (bleed + self.trim_mm.1) * MM,
        );

        let catalog = w.alloc();
        let pages = w.alloc();
        let image = self.image_ppi.map(|_| w.alloc());
        let gstate = self.transparency.then(|| w.alloc());

        let mut resources = String::from("<< ");
        if let Some(image) = image {
            let _ = write!(resources, "/XObject << /Im1 {image} 0 R >> ");
        }
        if let Some(gstate) = gstate {
            let _ = write!(resources, "/ExtGState << /GS1 {gstate} 0 R >> ");
        }
        resources.push_str(">>");

        let mut kids = Vec::new();
        for index in 0..self.pages {
            let page = w.alloc();
            let content = w.alloc();
            let operators = if index == 0 {
                self.outside_content(media_w, media_h, bleed)
            } else {
                Self::inside_content(media_w, media_h)
            };
            w.stream(content, "", operators.as_bytes());
            w.object(
                page,
                &format!(
                    "<< /Type /Page /Parent {pages} 0 R {boxes} /Resources {resources} \
                     /Contents {content} 0 R >>"
                ),
            );
            kids.push(format!("{page} 0 R"));
        }
        w.object(
            pages,
            &format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), self.pages),
        );

        if let Some(image) = image {
            w.stream(
                image,
                &format!(
                    "/Type /XObject /Subtype /Image /Width {IMAGE_PX} /Height {IMAGE_PX} \
                     /ColorSpace /DeviceCMYK /BitsPerComponent 8"
                ),
                &sample_image(),
            );
        }
        if let Some(gstate) = gstate {
            w.object(gstate, "<< /Type /ExtGState /ca 0.5 /CA 0.5 >>");
        }

        let mut catalog_entries = format!("/Type /Catalog /Pages {pages} 0 R");
        if self.output_intent {
            let profile = w.alloc();
            w.stream(profile, "/N 4", CMYK_PROFILE);
            let intent = w.alloc();
            w.object(
                intent,
                &format!(
                    "<< /Type /OutputIntent /S /GTS_PDFX /OutputConditionIdentifier (CGATS TR 001) \
                     /RegistryName (http://www.color.org) /Info (CGATS TR 001 \\(SWOP\\), compact profile) \
                     /DestOutputProfile {profile} 0 R >>"
                ),
            );
            let metadata = w.alloc();
            w.stream(metadata, "/Type /Metadata /Subtype /XML", self.xmp().as_bytes());
            let _ = write!(
                catalog_entries,
                " /OutputIntents [{intent} 0 R] /Metadata {metadata} 0 R"
            );
        }
        w.object(catalog, &format!("<< {catalog_entries} >>"));

        let info = w.alloc();
        let mut info_entries = String::from("/Producer (ebc-demo-printer testkit) /Trapped /False");
        if let Some(title) = &self.title {
            let _ = write!(info_entries, " /Title {}", text_string(title));
        }
        if self.output_intent {
            info_entries.push_str(" /GTS_PDFXVersion (PDF/X-4)");
        }
        if self.geometry_keys {
            let cover = self.cover_width();
            let _ = write!(
                info_entries,
                " /EBC.SpineWidthMm ({}) /EBC.FoldXMm ({},{}) /EBC.CoverWidthMm ({}) \
                 /EBC.CoverHeightMm ({})",
                number(self.spine_mm),
                number(cover),
                number(cover + self.spine_mm),
                number(cover),
                number(self.trim_mm.1),
            );
        }
        w.object(info, &format!("<< {info_entries} >>"));
        w.finish(catalog, info)
    }

    /// Page 1: back | spine | front, flat panels running into the bleed.
    fn outside_content(&self, media_w: f64, media_h: f64, bleed: f64) -> String {
        let cover = self.cover_width();
        let mut c = String::new();
        // Ground over the whole bleed box, then a darker spine and a lighter front panel.
        let _ = writeln!(c, "q 0.10 0.05 0.30 0 k 0 0 {media_w:.4} {media_h:.4} re f Q");
        let _ = writeln!(
            c,
            "q 0 0 0 0.85 k {:.4} 0 {:.4} {media_h:.4} re f Q",
            (bleed + cover) * MM,
            self.spine_mm * MM
        );
        let front_x = bleed + cover + self.spine_mm;
        let _ = writeln!(
            c,
            "q 0.02 0.02 0.10 0 k {:.4} 0 {:.4} {media_h:.4} re f Q",
            front_x * MM,
            (cover + bleed) * MM
        );
        if let Some(ppi) = self.image_ppi {
            // 60 px over `side` millimetres gives the requested effective resolution.
            let side = f64::from(IMAGE_PX) * 25.4 / ppi.max(1.0) * MM;
            let _ = writeln!(
                c,
                "q {side:.4} 0 0 {side:.4} {:.4} {:.4} cm /Im1 Do Q",
                (front_x + 20.0) * MM,
                (bleed + 20.0) * MM
            );
        }
        if self.rgb_fill {
            let _ = writeln!(c, "q 0.8 0.1 0.1 rg {:.4} {:.4} 60 60 re f Q", 30.0 * MM, 30.0 * MM);
        }
        if self.transparency {
            let _ = writeln!(
                c,
                "q /GS1 gs 0 0.4 0.2 0 k {:.4} {:.4} 60 60 re f Q",
                30.0 * MM,
                60.0 * MM
            );
        }
        if self.heavy_ink {
            let _ = writeln!(c, "q 1 1 1 1 k {:.4} {:.4} 60 60 re f Q", 30.0 * MM, 90.0 * MM);
        }
        c
    }

    /// Other pages: white, with two black rules where instructions would go.
    fn inside_content(media_w: f64, media_h: f64) -> String {
        let mut c = String::new();
        let _ = writeln!(c, "q 0 0 0 0 k 0 0 {media_w:.4} {media_h:.4} re f Q");
        let _ = writeln!(
            c,
            "q 0 0 0 1 K 0.75 w {x0:.4} {y:.4} m {x1:.4} {y:.4} l S {x0:.4} {y2:.4} m {x1:.4} {y2:.4} l S Q",
            x0 = media_w * 0.6,
            x1 = media_w * 0.9,
            y = media_h * 0.4,
            y2 = media_h * 0.35,
        );
        c
    }

    fn xmp(&self) -> String {
        let title = xml_escape(self.title.as_deref().unwrap_or("Carte"));
        format!(
            "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
             <x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n\
             <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n\
             <rdf:Description rdf:about=\"\" xmlns:pdfxid=\"http://www.npes.org/pdfx/ns/id/\" \
             xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n\
             <pdfxid:GTS_PDFXVersion>PDF/X-4</pdfxid:GTS_PDFXVersion>\n\
             <dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">{title}</rdf:li></rdf:Alt></dc:title>\n\
             </rdf:Description>\n</rdf:RDF>\n</x:xmpmeta>\n<?xpacket end=\"w\"?>"
        )
    }
}

/// A 60 × 60 px CMYK gradient with low ink coverage, uncompressed.
fn sample_image() -> Vec<u8> {
    let mut data = Vec::with_capacity((IMAGE_PX * IMAGE_PX * 4) as usize);
    for y in 0..IMAGE_PX {
        for x in 0..IMAGE_PX {
            let c = u8::try_from(x * 2).unwrap_or(u8::MAX);
            let yellow = u8::try_from(y * 2).unwrap_or(u8::MAX);
            data.extend_from_slice(&[c, 0, yellow, 0]);
        }
    }
    data
}

/// Formats a millimetre value without a trailing `.0`.
fn number(value: f64) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    if rounded.fract() == 0.0 { format!("{rounded:.0}") } else { format!("{rounded}") }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// A PDF text string: a literal string when the text is ASCII, UTF-16BE in hex otherwise.
fn text_string(text: &str) -> String {
    if text.is_ascii() {
        let mut out = String::from("(");
        for c in text.chars() {
            match c {
                '(' | ')' | '\\' => {
                    out.push('\\');
                    out.push(c);
                }
                c if c.is_ascii_control() => out.push(' '),
                c => out.push(c),
            }
        }
        out.push(')');
        out
    } else {
        let mut out = String::from("<FEFF");
        for unit in text.encode_utf16() {
            let _ = write!(out, "{unit:04X}");
        }
        out.push('>');
        out
    }
}

/// A minimal PDF object writer with a classic cross-reference table.
struct Writer {
    buffer: Vec<u8>,
    /// Byte offset of each object, indexed by object number - 1.
    offsets: Vec<Option<usize>>,
}

impl Writer {
    fn new() -> Self {
        let mut buffer = Vec::with_capacity(32 * 1024);
        buffer.extend_from_slice(b"%PDF-1.6\n%\xE2\xE3\xCF\xD3\n");
        Self { buffer, offsets: Vec::new() }
    }

    /// Reserves the next object number.
    fn alloc(&mut self) -> usize {
        self.offsets.push(None);
        self.offsets.len()
    }

    fn begin(&mut self, id: usize) {
        if let Some(slot) = self.offsets.get_mut(id - 1) {
            *slot = Some(self.buffer.len());
        }
        self.buffer.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
    }

    fn object(&mut self, id: usize, body: &str) {
        self.begin(id);
        self.buffer.extend_from_slice(body.as_bytes());
        self.buffer.extend_from_slice(b"\nendobj\n");
    }

    fn stream(&mut self, id: usize, entries: &str, data: &[u8]) {
        self.begin(id);
        self.buffer
            .extend_from_slice(format!("<< {entries} /Length {} >>\nstream\n", data.len()).as_bytes());
        self.buffer.extend_from_slice(data);
        self.buffer.extend_from_slice(b"\nendstream\nendobj\n");
    }

    fn finish(mut self, root: usize, info: usize) -> Vec<u8> {
        let file_id = {
            let digest = Sha256::digest(&self.buffer);
            let mut hex = String::with_capacity(32);
            for byte in digest.iter().take(16) {
                let _ = write!(hex, "{byte:02X}");
            }
            hex
        };
        let xref_offset = self.buffer.len();
        let mut tail = format!("xref\n0 {}\n0000000000 65535 f \n", self.offsets.len() + 1);
        for offset in &self.offsets {
            // Every allocated object is written by `to_pdf`; a hole would be a bug there,
            // and is recorded as a free entry rather than a bogus offset.
            match offset {
                Some(offset) => {
                    let _ = write!(tail, "{offset:010} 00000 n \n");
                }
                None => tail.push_str("0000000000 65535 f \n"),
            }
        }
        let _ = write!(
            tail,
            "trailer\n<< /Size {} /Root {root} 0 R /Info {info} 0 R /ID [<{file_id}> <{file_id}>] >>\n\
             startxref\n{xref_offset}\n%%EOF\n",
            self.offsets.len() + 1
        );
        self.buffer.extend_from_slice(tail.as_bytes());
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    #[test]
    fn output_is_deterministic_and_looks_like_a_pdf() {
        let product = catalog::find("card-poche").unwrap();
        let a = SampleCard::for_product(product).to_pdf();
        let b = SampleCard::for_product(product).to_pdf();
        assert_eq!(a, b);
        assert!(a.starts_with(b"%PDF-1.6\n"));
        assert!(a.ends_with(b"%%EOF\n"));
        // startxref points at the cross-reference table.
        let text = String::from_utf8_lossy(&a);
        let startxref: usize = text
            .rsplit("startxref\n")
            .next()
            .and_then(|tail| tail.lines().next())
            .and_then(|line| line.parse().ok())
            .unwrap();
        assert_eq!(&a[startxref..startxref + 5], b"xref\n");
        // Every xref entry is exactly 20 bytes and points at "N 0 obj".
        let table = &a[startxref..];
        let table_text = String::from_utf8_lossy(table);
        let mut lines = table_text.lines();
        assert_eq!(lines.next(), Some("xref"));
        let count: usize =
            lines.next().unwrap().strip_prefix("0 ").unwrap().parse().unwrap();
        for (number, line) in lines.take(count).enumerate().skip(1) {
            assert_eq!(line.len(), 19, "{line:?} (plus the newline: 20 bytes)");
            let offset: usize = line[..10].parse().unwrap();
            let expected = format!("{number} 0 obj\n");
            assert_eq!(&a[offset..offset + expected.len()], expected.as_bytes());
        }
    }

    #[test]
    fn text_strings_are_escaped() {
        assert_eq!(text_string("Simple"), "(Simple)");
        assert_eq!(text_string("a(b)c\\d\n"), "(a\\(b\\)c\\\\d )");
        assert_eq!(text_string("é"), "<FEFF00E9>");
        assert_eq!(text_string("𝄞"), "<FEFFD834DD1E>");
        assert_eq!(number(10.0), "10");
        assert_eq!(number(97.5), "97.5");
        assert_eq!(number(110.000_000_1), "110");
    }
}
