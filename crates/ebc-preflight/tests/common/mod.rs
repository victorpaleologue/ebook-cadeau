//! Fixture PDFs for the integration tests, built with `lopdf` (never with `ebc-pdf`: the
//! preflight must stay independent of the writer it referees).
//!
//! [`Fixture::reference`] is a two-page "poche" card that a professional printer would
//! accept: 230 × 178 mm trim (two 110 mm covers and a 10 mm spine), 3 mm bleed, CMYK only,
//! an embedded subset font, a 300 ppi cover image, a vector QR code, a PDF/X-4 output intent
//! and the `EBC.*` geometry keys. Tests mutate it and expect exactly the matching finding.

#![allow(dead_code)]

use std::collections::BTreeSet;

use ebc_preflight::{Expectations, Report};
use lopdf::xref::XrefType;
use lopdf::{Dictionary, Document, Object, ObjectId, Stream, dictionary};

/// Points per millimetre.
pub(crate) const MM: f64 = 72.0 / 25.4;

pub(crate) const COVER_W: f64 = 110.0;
pub(crate) const COVER_H: f64 = 178.0;
pub(crate) const SPINE: f64 = 10.0;
pub(crate) const TRIM_W: f64 = 2.0 * COVER_W + SPINE;
pub(crate) const BLEED: f64 = 3.0;
pub(crate) const MEDIA_W: f64 = TRIM_W + 2.0 * BLEED;
pub(crate) const MEDIA_H: f64 = COVER_H + 2.0 * BLEED;

/// Cover image pixels: 113 × 184 mm (front cover plus bleed on three sides) at 300 ppi.
pub(crate) const COVER_PX: (i64, i64) = (1335, 2174);

/// A complete, valid baseline JPEG: 8 × 8 pixels, CMYK (Adobe marker), one flat colour.
///
/// It stands in for the cover photo. The preflight never decodes image data — it reads the
/// pixel dimensions from the image dictionary — but external validators (`qpdf --check`)
/// do decode it, so the bytes must be a real JPEG.
#[rustfmt::skip]
pub(crate) const TINY_CMYK_JPEG: [u8; 176] = [
    0xFF, 0xD8, 0xFF, 0xEE, 0x00, 0x0E, 0x41, 0x64, 0x6F, 0x62, 0x65, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDB,
    0x00, 0x43, 0x00, 0x50, 0x37, 0x3C, 0x46, 0x3C, 0x32, 0x50, 0x46, 0x41, 0x46, 0x5A, 0x55, 0x50, 0x5F, 0x78, 0xC8, 0x82,
    0x78, 0x6E, 0x6E, 0x78, 0xF5, 0xAF, 0xB9, 0x91, 0xC8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC0, 0x00, 0x14, 0x08, 0x00, 0x08, 0x00, 0x08, 0x04, 0x43, 0x11, 0x00,
    0x4D, 0x11, 0x00, 0x59, 0x11, 0x00, 0x4B, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x16, 0x00, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x03, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x0E, 0x04,
    0x43, 0x00, 0x4D, 0x00, 0x59, 0x00, 0x4B, 0x00, 0x00, 0x3F, 0x00, 0x88, 0x29, 0xAB, 0xFF, 0xD9,
];

/// QR module size on the inside page, in millimetres.
pub(crate) const QR_MODULE: f64 = 0.5;

/// Millimetres to points, as a PDF real.
pub(crate) fn pt(mm: f64) -> Object {
    Object::Real((mm * MM) as f32)
}

/// A rectangle array from millimetres.
pub(crate) fn rect_mm(x0: f64, y0: f64, x1: f64, y1: f64) -> Object {
    Object::Array(vec![pt(x0), pt(y0), pt(x1), pt(y1)])
}

pub(crate) fn name(n: &str) -> Object {
    Object::Name(n.as_bytes().to_vec())
}

/// A PDF text string: PDFDocEncoding when the text is ASCII, UTF-16BE with a byte-order mark
/// otherwise.
pub(crate) fn string(s: &str) -> Object {
    if s.is_ascii() {
        return Object::string_literal(s);
    }
    let mut bytes = vec![0xFE, 0xFF];
    for unit in s.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    Object::String(bytes, lopdf::StringFormat::Hexadecimal)
}

/// What the reference card is expected to satisfy.
pub(crate) fn card_expectations() -> Expectations {
    Expectations::professional().with_trim(TRIM_W, COVER_H).with_spine(SPINE).with_min_qr_module(0.4)
}

/// The distinct finding codes of a report.
pub(crate) fn codes(report: &Report) -> BTreeSet<&'static str> {
    report.findings.iter().map(|f| f.code).collect()
}

/// A set of codes, for comparisons.
pub(crate) fn set(codes: &[&'static str]) -> BTreeSet<&'static str> {
    codes.iter().copied().collect()
}

/// A minimal but well-formed ICC header (132 bytes: header plus an empty tag table).
pub(crate) fn icc_profile(class: &[u8; 4], color_space: &[u8; 4]) -> Vec<u8> {
    let mut data = vec![0_u8; 132];
    data[0..4].copy_from_slice(&132_u32.to_be_bytes());
    data[8..12].copy_from_slice(&[2, 0x40, 0, 0]);
    data[12..16].copy_from_slice(class);
    data[16..20].copy_from_slice(color_space);
    data[20..24].copy_from_slice(b"Lab ");
    data[36..40].copy_from_slice(b"acsp");
    data
}

pub(crate) fn xmp(pdfx_version: &str) -> Vec<u8> {
    format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n\
         <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n\
         <rdf:Description rdf:about=\"\" xmlns:pdfxid=\"http://www.npes.org/pdfx/ns/id/\" \
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n\
         <pdfxid:GTS_PDFXVersion>{pdfx_version}</pdfxid:GTS_PDFXVersion>\n\
         <dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">Le Petit Prince</rdf:li></rdf:Alt></dc:title>\n\
         </rdf:Description>\n</rdf:RDF>\n</x:xmpmeta>\n<?xpacket end=\"w\"?>"
    )
    .into_bytes()
}

/// A deterministic 25 × 25 module matrix with QR finder and timing patterns.
pub(crate) fn qr_matrix() -> Vec<Vec<bool>> {
    const SIZE: usize = 25;
    let mut m = vec![vec![false; SIZE]; SIZE];
    for (ox, oy) in [(0, 0), (SIZE - 7, 0), (0, SIZE - 7)] {
        for y in 0..7 {
            for x in 0..7 {
                let ring = x == 0 || y == 0 || x == 6 || y == 6;
                let core = (2..=4).contains(&x) && (2..=4).contains(&y);
                m[oy + y][ox + x] = ring || core;
            }
        }
    }
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let in_finder = (x < 8 && y < 8) || (x >= SIZE - 8 && y < 8) || (x < 8 && y >= SIZE - 8);
            if in_finder {
                continue;
            }
            if y == 6 || x == 6 {
                m[y][x] = (x + y) % 2 == 0;
                continue;
            }
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            m[y][x] = state & 1 == 1;
        }
    }
    m
}

/// Content operators drawing the QR matrix as merged horizontal runs (`re` … `f`), with its
/// lower-left corner at `(x_mm, y_mm)`.
pub(crate) fn qr_content(x_mm: f64, y_mm: f64, module_mm: f64) -> String {
    let matrix = qr_matrix();
    let size = matrix.len();
    let mut out = String::from("q 0 0 0 1 k\n");
    for (row, cells) in matrix.iter().enumerate() {
        let mut x = 0;
        while x < size {
            if !cells[x] {
                x += 1;
                continue;
            }
            let start = x;
            while x < size && cells[x] {
                x += 1;
            }
            // Row 0 is the top of the symbol.
            let top = (size - row) as f64;
            out.push_str(&format!(
                "{:.4} {:.4} {:.4} {:.4} re\n",
                (x_mm + start as f64 * module_mm) * MM,
                (y_mm + (top - 1.0) * module_mm) * MM,
                (x - start) as f64 * module_mm * MM,
                module_mm * MM,
            ));
        }
    }
    out.push_str("f Q\n");
    out
}

/// The reference card and the ids of the objects tests like to mutate.
pub(crate) struct Fixture {
    pub(crate) doc: Document,
    pub(crate) catalog: ObjectId,
    pub(crate) info: ObjectId,
    pub(crate) pages: ObjectId,
    pub(crate) page1: ObjectId,
    pub(crate) page2: ObjectId,
    pub(crate) content1: ObjectId,
    pub(crate) content2: ObjectId,
    pub(crate) cover_image: ObjectId,
    pub(crate) font: ObjectId,
    pub(crate) cid_font: ObjectId,
    pub(crate) font_descriptor: ObjectId,
    pub(crate) font_file: ObjectId,
    pub(crate) output_intent: ObjectId,
    pub(crate) icc: ObjectId,
    pub(crate) metadata: ObjectId,
}

impl Fixture {
    /// The clean reference card: preflights without a single finding.
    pub(crate) fn reference() -> Self {
        let mut doc = Document::with_version("1.6");
        doc.reference_table.cross_reference_type = XrefType::CrossReferenceTable;

        // -- font: an embedded TrueType subset as a composite font --
        let font_file = doc.add_object(Stream::new(dictionary! { "Length1" => 64 }, vec![0xA5; 64]));
        let font_descriptor = doc.add_object(dictionary! {
            "Type" => "FontDescriptor",
            "FontName" => "EBCAAA+AtkinsonHyperlegible-Regular",
            "Flags" => 4,
            "FontBBox" => vec![(-100).into(), (-250).into(), 1100.into(), 950.into()],
            "ItalicAngle" => 0, "Ascent" => 950, "Descent" => -250, "CapHeight" => 668, "StemV" => 80,
            "FontFile2" => font_file,
        });
        let cid_font = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "CIDFontType2",
            "BaseFont" => "EBCAAA+AtkinsonHyperlegible-Regular",
            "CIDSystemInfo" => dictionary! { "Registry" => string("Adobe"), "Ordering" => string("Identity"), "Supplement" => 0 },
            "FontDescriptor" => font_descriptor,
            "DW" => 600, "CIDToGIDMap" => "Identity",
        });
        let font = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "Type0",
            "BaseFont" => "EBCAAA+AtkinsonHyperlegible-Regular",
            "Encoding" => "Identity-H",
            "DescendantFonts" => vec![cid_font.into()],
        });

        // -- cover image: CMYK, 300 ppi over the front cover plus bleed --
        let cover_image = doc.add_object(Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image",
                "Width" => COVER_PX.0, "Height" => COVER_PX.1,
                "ColorSpace" => "DeviceCMYK", "BitsPerComponent" => 8,
                "Filter" => "DCTDecode",
            },
            TINY_CMYK_JPEG.to_vec(),
        ));

        // -- page 1: outside spread (back | spine | front) --
        let front_x = BLEED + COVER_W + SPINE;
        let content1 = format!(
            "q 0.10 0.05 0.30 0 k 0 0 {mw:.4} {mh:.4} re f Q\n\
             q {iw:.4} 0 0 {mh:.4} {ix:.4} 0 cm /Im1 Do Q\n\
             q 0 0 0 0.9 k {sx:.4} 0 {sw:.4} {mh:.4} re f Q\n\
             BT /F1 9 Tf 0 0 0 0 k 0 -1 1 0 {tx:.4} {ty:.4} Tm <000100020003> Tj ET\n",
            mw = MEDIA_W * MM,
            mh = MEDIA_H * MM,
            iw = (COVER_W + BLEED) * MM,
            ix = front_x * MM,
            sx = (BLEED + COVER_W) * MM,
            sw = SPINE * MM,
            tx = (BLEED + COVER_W + 3.5) * MM,
            ty = (MEDIA_H - 20.0) * MM,
        );
        let content1 = doc.add_object(Stream::new(dictionary! {}, content1.into_bytes()));

        // -- page 2: inside spread with the QR code --
        let content2 = format!(
            "q 0 0 0 0 k 0 0 {mw:.4} {mh:.4} re f Q\n\
             {qr}\
             BT /F1 14 Tf 0 0 0 1 k 1 0 0 1 {tx:.4} {ty:.4} Tm <0004000500060007> Tj ET\n\
             q 0 0 0 1 K 0.5 w {lx:.4} {ly:.4} m {lx2:.4} {ly:.4} l S Q\n",
            mw = MEDIA_W * MM,
            mh = MEDIA_H * MM,
            qr = qr_content(150.0, 80.0, QR_MODULE),
            tx = 150.0 * MM,
            ty = 70.0 * MM,
            lx = 140.0 * MM,
            lx2 = 220.0 * MM,
            ly = 60.0 * MM,
        );
        let content2 = doc.add_object(Stream::new(dictionary! {}, content2.into_bytes()));

        let pages = doc.new_object_id();
        let boxes = |page: &mut Dictionary| {
            page.set("MediaBox", rect_mm(0.0, 0.0, MEDIA_W, MEDIA_H));
            page.set("BleedBox", rect_mm(0.0, 0.0, MEDIA_W, MEDIA_H));
            page.set("TrimBox", rect_mm(BLEED, BLEED, BLEED + TRIM_W, BLEED + COVER_H));
        };
        let mut page1 = dictionary! {
            "Type" => "Page", "Parent" => pages, "Contents" => content1,
            "Resources" => dictionary! {
                "Font" => dictionary! { "F1" => font },
                "XObject" => dictionary! { "Im1" => cover_image },
            },
        };
        boxes(&mut page1);
        let page1 = doc.add_object(page1);
        let mut page2 = dictionary! {
            "Type" => "Page", "Parent" => pages, "Contents" => content2,
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
        };
        boxes(&mut page2);
        let page2 = doc.add_object(page2);
        doc.objects.insert(
            pages,
            Object::Dictionary(dictionary! {
                "Type" => "Pages", "Kids" => vec![page1.into(), page2.into()], "Count" => 2,
            }),
        );

        // -- output intent, metadata, catalog, info --
        let icc = doc.add_object(Stream::new(dictionary! { "N" => 4 }, icc_profile(b"prtr", b"CMYK")));
        let output_intent = doc.add_object(dictionary! {
            "Type" => "OutputIntent", "S" => "GTS_PDFX",
            "OutputConditionIdentifier" => string("FOGRA39"),
            "RegistryName" => string("http://www.color.org"),
            "Info" => string("Coated FOGRA39 (ISO 12647-2:2004)"),
            "DestOutputProfile" => icc,
        });
        let metadata =
            doc.add_object(Stream::new(dictionary! { "Type" => "Metadata", "Subtype" => "XML" }, xmp("PDF/X-4")));
        let catalog = doc.add_object(dictionary! {
            "Type" => "Catalog", "Pages" => pages,
            "OutputIntents" => vec![output_intent.into()],
            "Metadata" => metadata,
        });
        let info = doc.add_object(dictionary! {
            "Title" => string("Le Petit Prince — carte cadeau"),
            "Producer" => string("ebc-preflight test fixture"),
            "Trapped" => "False",
            "GTS_PDFXVersion" => string("PDF/X-4"),
            "EBC.SpineWidthMm" => string("10"),
            "EBC.FoldXMm" => string("110,120"),
            "EBC.CoverWidthMm" => string("110"),
            "EBC.CoverHeightMm" => string("178"),
        });
        doc.trailer.set("Root", catalog);
        doc.trailer.set("Info", info);
        let id = Object::String(b"ebc-fixture-0001".to_vec(), lopdf::StringFormat::Hexadecimal);
        doc.trailer.set("ID", vec![id.clone(), id]);

        Self {
            doc,
            catalog,
            info,
            pages,
            page1,
            page2,
            content1,
            content2,
            cover_image,
            font,
            cid_font,
            font_descriptor,
            font_file,
            output_intent,
            icc,
            metadata,
        }
    }

    /// Serialises the document with a classic cross-reference table.
    pub(crate) fn bytes(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        self.doc.save_to(&mut out).expect("writing to a Vec cannot fail");
        out
    }

    /// Serialises the document with object streams and a cross-reference stream.
    pub(crate) fn bytes_modern(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        self.doc.save_modern(&mut out).expect("writing to a Vec cannot fail");
        out
    }

    /// Preflights the document against the card expectations.
    pub(crate) fn check(&mut self) -> Report {
        self.check_with(&card_expectations())
    }

    pub(crate) fn check_with(&mut self, expect: &Expectations) -> Report {
        let bytes = self.bytes();
        ebc_preflight::check(&bytes, expect).expect("the fixture must parse")
    }

    /// The dictionary of an indirect dictionary or stream object.
    pub(crate) fn dict(&mut self, id: ObjectId) -> &mut Dictionary {
        match self.doc.objects.get_mut(&id).expect("object exists") {
            Object::Dictionary(dict) => dict,
            Object::Stream(stream) => &mut stream.dict,
            other => panic!("object {id:?} is not a dictionary: {other:?}"),
        }
    }

    /// A page's (direct) resource dictionary.
    pub(crate) fn resources(&mut self, page: ObjectId) -> &mut Dictionary {
        match self.dict(page).get_mut(b"Resources").expect("page has resources") {
            Object::Dictionary(dict) => dict,
            other => panic!("resources are not a direct dictionary: {other:?}"),
        }
    }

    /// Adds `resources[category][name] = object` to a page.
    pub(crate) fn add_resource(&mut self, page: ObjectId, category: &str, key: &str, object: impl Into<Object>) {
        let resources = self.resources(page);
        if !resources.has(category.as_bytes()) {
            resources.set(category, Dictionary::new());
        }
        match resources.get_mut(category.as_bytes()).expect("just inserted") {
            Object::Dictionary(dict) => dict.set(key, object),
            other => panic!("{category} is not a direct dictionary: {other:?}"),
        }
    }

    /// The raw bytes of a content stream.
    pub(crate) fn content(&self, id: ObjectId) -> Vec<u8> {
        match self.doc.objects.get(&id).expect("object exists") {
            Object::Stream(stream) => stream.content.clone(),
            other => panic!("object {id:?} is not a stream: {other:?}"),
        }
    }

    /// Replaces a content stream.
    pub(crate) fn set_content(&mut self, id: ObjectId, content: impl Into<Vec<u8>>) {
        match self.doc.objects.get_mut(&id).expect("object exists") {
            Object::Stream(stream) => stream.set_plain_content(content.into()),
            other => panic!("object {id:?} is not a stream: {other:?}"),
        }
    }

    /// Appends operators to a content stream.
    pub(crate) fn append_content(&mut self, id: ObjectId, extra: &str) {
        let mut content = self.content(id);
        content.push(b'\n');
        content.extend_from_slice(extra.as_bytes());
        self.set_content(id, content);
    }

    /// Sets a string value in the Info dictionary.
    pub(crate) fn set_info(&mut self, key: &str, value: &str) {
        let info = self.info;
        self.dict(info).set(key, string(value));
    }
}

/// A tiny deterministic pseudo-random generator (xorshift64*), so that fuzz-style tests are
/// reproducible without a dependency.
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    pub(crate) fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| (self.next() >> 32) as u8).collect()
    }
}

/// Whether an external command-line tool is available (tests that need one skip otherwise).
pub(crate) fn has_tool(tool: &str) -> bool {
    std::process::Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// A scratch directory unique to one test, under the system temporary directory.
pub(crate) fn scratch_dir(test: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ebc-preflight-{}-{test}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}
