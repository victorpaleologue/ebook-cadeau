//! Helpers shared by the integration tests.
#![allow(dead_code)]

pub(crate) mod card;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use ebc_scene::{
    Color, Document, FontId, Glyph, ImageData, Item, MM_PER_PT, OutputIntent, PdfStandard, Point,
};

/// 2026-11-12T08:30:00Z.
pub(crate) const CREATED: i64 = 1_794_472_200;

pub(crate) fn asset(relative: &str) -> Arc<[u8]> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets").join(relative);
    Arc::from(std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
}

pub(crate) fn fixture(name: &str) -> Arc<[u8]> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    Arc::from(std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
}

pub(crate) fn font(name: &str) -> Arc<[u8]> {
    asset(&format!("fonts/{name}"))
}

pub(crate) fn cmyk_profile() -> Arc<[u8]> {
    asset("icc/CGATS001Compat-v2-micro.icc")
}

pub(crate) fn srgb_v2_profile() -> Arc<[u8]> {
    asset("icc/sRGB-v2-micro.icc")
}

pub(crate) fn srgb_v4_profile() -> Arc<[u8]> {
    asset("icc/sRGB-v4.icc")
}

pub(crate) fn output_intent() -> OutputIntent {
    OutputIntent {
        condition_identifier: "CGATS TR 001".into(),
        info: "SWOP (CGATS TR 001) compatible".into(),
        registry_name: "http://www.color.org".into(),
        icc_profile: Some(cmyk_profile()),
        components: 4,
    }
}

/// An empty document that satisfies the document-level rules of `standard`.
pub(crate) fn base_doc(standard: PdfStandard) -> Document {
    let mut doc = Document { standard, ..Document::default() };
    doc.info.title = "Carte cadeau — L'Étranger".into();
    doc.info.author = Some("Albert Camus".into());
    doc.info.creator = Some("ebc-pdf tests".into());
    doc.info.created_unix = Some(CREATED);
    doc.info.language = Some("fr-FR".into());
    doc.output_intent = Some(output_intent());
    doc
}

/// Lays `text` out left to right with the font's own advances (no shaping, no kerning):
/// enough to exercise the writer. Returns the glyphs, their texts and the run width (mm).
pub(crate) fn layout(font_data: &[u8], text: &str, size_pt: f64) -> (Vec<Glyph>, Vec<String>, f64) {
    let face = ttf_parser::Face::parse(font_data, 0).expect("test font parses");
    let scale = size_pt * MM_PER_PT / f64::from(face.units_per_em());
    let mut x = 0.0;
    let mut glyphs = Vec::new();
    let mut texts = Vec::new();
    for c in text.chars() {
        let id = face.glyph_index(c).unwrap_or_else(|| panic!("font lacks {c:?}"));
        glyphs.push(Glyph { id: id.0, x, y: 0.0 });
        texts.push(c.to_string());
        x += f64::from(face.glyph_hor_advance(id).unwrap_or(0)) * scale;
    }
    (glyphs, texts, x)
}

/// A glyph run item for `text` starting at `origin`.
pub(crate) fn text_item(
    font: FontId,
    font_data: &[u8],
    text: &str,
    size_pt: f64,
    origin: Point,
    color: Color,
) -> Item {
    let (glyphs, glyph_text, _) = layout(font_data, text, size_pt);
    Item::Glyphs { font, size_pt, color, origin, glyphs, glyph_text }
}

// ----- test images ---------------------------------------------------------------------

/// An RGB JPEG whose pixels come from `pixel(x, y)`.
pub(crate) fn jpeg_rgb(width: u32, height: u32, pixel: impl Fn(u32, u32) -> [u8; 3]) -> ImageData {
    let mut data = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            data.extend_from_slice(&pixel(x, y));
        }
    }
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 95)
        .encode(&data, width, height, image::ExtendedColorType::Rgb8)
        .expect("jpeg encoding");
    ImageData::Jpeg { bytes: Arc::from(bytes), width_px: width, height_px: height, components: 3 }
}

/// A greyscale JPEG.
pub(crate) fn jpeg_gray(width: u32, height: u32, pixel: impl Fn(u32, u32) -> u8) -> ImageData {
    let mut data = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            data.push(pixel(x, y));
        }
    }
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 95)
        .encode(&data, width, height, image::ExtendedColorType::L8)
        .expect("jpeg encoding");
    ImageData::Jpeg { bytes: Arc::from(bytes), width_px: width, height_px: height, components: 1 }
}

/// An Adobe-style CMYK JPEG (inverted samples, APP14 marker) of one flat ink colour
/// given as C, M, Y, K with 255 = 100 % ink. `ycck` selects the YCCK transform.
pub(crate) fn jpeg_cmyk(width: u16, height: u16, ink: [u8; 4], ycck: bool) -> ImageData {
    jpeg_cmyk_with(width, height, |_, _| ink, ycck)
}

/// An Adobe-style CMYK JPEG whose ink values come from `pixel(x, y)`.
pub(crate) fn jpeg_cmyk_with(
    width: u16,
    height: u16,
    pixel: impl Fn(u16, u16) -> [u8; 4],
    ycck: bool,
) -> ImageData {
    let mut data = Vec::with_capacity(usize::from(width) * usize::from(height) * 4);
    for y in 0..height {
        for x in 0..width {
            data.extend_from_slice(&pixel(x, y));
        }
    }
    let mut bytes = Vec::new();
    let color =
        if ycck { jpeg_encoder::ColorType::CmykAsYcck } else { jpeg_encoder::ColorType::Cmyk };
    jpeg_encoder::Encoder::new(&mut bytes, 95)
        .encode(&data, width, height, color)
        .expect("jpeg encoding");
    ImageData::Jpeg {
        bytes: Arc::from(bytes),
        width_px: u32::from(width),
        height_px: u32::from(height),
        components: 4,
    }
}

// ----- external tools ------------------------------------------------------------------

/// Full path of `name` if it is on `PATH`.
pub(crate) fn tool(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths).map(|dir| dir.join(name)).find(|p| p.is_file())
}

/// Runs `name` with `args`; returns `None` (after printing a note) when the tool is not
/// installed, otherwise `(success, stdout, stderr)`.
pub(crate) fn run(name: &str, args: &[&str]) -> Option<(bool, String, String)> {
    let Some(path) = tool(name) else {
        eprintln!("note: `{name}` is not installed — skipping this check");
        return None;
    };
    let output = Command::new(path).args(args).output().expect("tool starts");
    Some((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// A per-test scratch directory under `target/tmp`.
pub(crate) fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ebc-pdf").join(test);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Writes `pdf` to `<scratch>/<name>` and returns the path as a string.
pub(crate) fn save(test: &str, name: &str, pdf: &[u8]) -> String {
    let path = scratch(test).join(name);
    std::fs::write(&path, pdf).expect("write pdf");
    path.to_string_lossy().into_owned()
}

/// Renders page `page` (1-based) at `dpi` with `pdftoppm` and returns the RGB pixels,
/// or `None` when poppler is not installed.
pub(crate) fn render(test: &str, pdf_path: &str, page: u32, dpi: u32) -> Option<image::RgbImage> {
    let stem = Path::new(pdf_path).file_stem().map(|s| s.to_string_lossy().into_owned());
    let prefix = scratch(test).join(format!("{}-page{page}-{dpi}dpi", stem.unwrap_or_default()));
    let prefix_str = prefix.to_string_lossy().into_owned();
    let page_str = page.to_string();
    let dpi_str = dpi.to_string();
    let (ok, _, stderr) = run(
        "pdftoppm",
        &[
            "-png",
            "-r",
            &dpi_str,
            "-f",
            &page_str,
            "-l",
            &page_str,
            "-singlefile",
            pdf_path,
            &prefix_str,
        ],
    )?;
    assert!(ok, "pdftoppm failed: {stderr}");
    let png = prefix.with_extension("png");
    Some(image::open(&png).expect("rendered page decodes").to_rgb8())
}

/// The pixel under the page-millimetre position `(x_mm, y_mm)` (origin: top-left corner
/// of the trim box) in a rendering made at `dpi`, for a page whose trim box is inset by
/// `margin_mm` (bleed + slug) from the media box.
pub(crate) fn pixel_at(
    img: &image::RgbImage,
    dpi: u32,
    margin_mm: f64,
    x_mm: f64,
    y_mm: f64,
) -> [u8; 3] {
    let scale = f64::from(dpi) / 25.4;
    let x = ((x_mm + margin_mm) * scale).floor() as u32;
    let y = ((y_mm + margin_mm) * scale).floor() as u32;
    img.get_pixel(x.min(img.width() - 1), y.min(img.height() - 1)).0
}

/// `true` when two colours differ by at most `tolerance` per channel.
pub(crate) fn close(a: [u8; 3], b: [u8; 3], tolerance: u8) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= tolerance)
}

// ----- parsing the output back with lopdf ------------------------------------------------

pub(crate) use lopdf::{Dictionary, Object};

pub(crate) fn parse(pdf: &[u8]) -> lopdf::Document {
    lopdf::Document::load_mem(pdf).expect("lopdf parses the output")
}

/// Follows indirect references.
pub(crate) fn resolve<'a>(doc: &'a lopdf::Document, object: &'a Object) -> &'a Object {
    doc.dereference(object).expect("reference resolves").1
}

/// `dict[key]`, dereferenced. Panics with the key name when it is missing.
pub(crate) fn get<'a>(doc: &'a lopdf::Document, dict: &'a Dictionary, key: &str) -> &'a Object {
    let object = dict.get(key.as_bytes()).unwrap_or_else(|_| panic!("missing /{key} in {dict:?}"));
    resolve(doc, object)
}

pub(crate) fn get_dict<'a>(
    doc: &'a lopdf::Document,
    dict: &'a Dictionary,
    key: &str,
) -> &'a Dictionary {
    match get(doc, dict, key) {
        Object::Dictionary(d) => d,
        Object::Stream(s) => &s.dict,
        other => panic!("/{key} is not a dictionary: {other:?}"),
    }
}

pub(crate) fn get_name(doc: &lopdf::Document, dict: &Dictionary, key: &str) -> String {
    String::from_utf8_lossy(get(doc, dict, key).as_name().expect("a name")).into_owned()
}

pub(crate) fn get_numbers(doc: &lopdf::Document, dict: &Dictionary, key: &str) -> Vec<f64> {
    get(doc, dict, key)
        .as_array()
        .expect("an array")
        .iter()
        .map(|o| f64::from(resolve(doc, o).as_float().expect("a number")))
        .collect()
}

/// Decodes a PDF text string (UTF-16BE with a byte-order mark, or ASCII).
pub(crate) fn text_string(object: &Object) -> String {
    let bytes = object.as_str().expect("a string");
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let units: Vec<u16> =
            rest.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        String::from_utf16(&units).expect("valid UTF-16")
    } else {
        assert!(bytes.is_ascii(), "non-ASCII string without BOM: {bytes:?}");
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub(crate) fn get_text(doc: &lopdf::Document, dict: &Dictionary, key: &str) -> String {
    text_string(get(doc, dict, key))
}

/// The decoded data of a stream object (Flate is undone; other filters are left alone).
pub(crate) fn stream_data(object: &Object) -> Vec<u8> {
    let stream = object.as_stream().expect("a stream");
    match stream.filters() {
        Ok(filters) if filters == [b"FlateDecode".as_slice()] => {
            stream.decompressed_content().expect("inflates")
        }
        _ => stream.content.clone(),
    }
}

pub(crate) fn get_stream_data(doc: &lopdf::Document, dict: &Dictionary, key: &str) -> Vec<u8> {
    stream_data(get(doc, dict, key))
}

/// The dictionary of page `number` (1-based).
pub(crate) fn page(doc: &lopdf::Document, number: u32) -> &Dictionary {
    let pages = doc.get_pages();
    let id = pages.get(&number).unwrap_or_else(|| panic!("no page {number}"));
    doc.get_dictionary(*id).expect("page dictionary")
}

/// The decoded content stream of page `number` (1-based) as text.
pub(crate) fn page_content(doc: &lopdf::Document, number: u32) -> String {
    let data = get_stream_data(doc, page(doc, number), "Contents");
    String::from_utf8_lossy(&data).into_owned()
}

/// The page's resource sub-dictionary `kind` (e.g. `"Font"`), if present.
pub(crate) fn resources<'a>(
    doc: &'a lopdf::Document,
    number: u32,
    kind: &str,
) -> Option<&'a Dictionary> {
    let res = get_dict(doc, page(doc, number), "Resources");
    res.has(kind.as_bytes()).then(|| get_dict(doc, res, kind))
}

/// The document information dictionary.
pub(crate) fn info(doc: &lopdf::Document) -> &Dictionary {
    get_dict(doc, &doc.trailer, "Info")
}

/// The document catalog.
pub(crate) fn catalog(doc: &lopdf::Document) -> &Dictionary {
    doc.catalog().expect("catalog")
}

/// The XMP packet as text.
pub(crate) fn xmp(doc: &lopdf::Document) -> String {
    let object = get(doc, catalog(doc), "Metadata");
    String::from_utf8(object.as_stream().expect("a stream").content.clone()).expect("UTF-8 XMP")
}

/// Asserts that two lists of numbers agree within `tolerance`.
#[track_caller]
pub(crate) fn assert_close(actual: &[f64], expected: &[f64], tolerance: f64) {
    assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= tolerance, "{actual:?} vs {expected:?}");
    }
}

/// Millimetres to points.
pub(crate) fn pt(mm: f64) -> f64 {
    mm * 72.0 / 25.4
}

/// A copy of a font with its `OS/2.fsType` field replaced.
pub(crate) fn with_fs_type(font: &[u8], fs_type: u16) -> Arc<[u8]> {
    let mut data = font.to_vec();
    let tables = usize::from(u16::from_be_bytes([data[4], data[5]]));
    for i in 0..tables {
        let record = 12 + 16 * i;
        if &data[record..record + 4] == b"OS/2" {
            let offset = u32::from_be_bytes([
                data[record + 8],
                data[record + 9],
                data[record + 10],
                data[record + 11],
            ]) as usize;
            data[offset + 8..offset + 10].copy_from_slice(&fs_type.to_be_bytes());
            return Arc::from(data);
        }
    }
    panic!("font has no OS/2 table");
}

// ----- fonts and text in the parsed output -----------------------------------------------

use std::collections::BTreeMap;

/// Everything the tests want to know about one embedded font.
pub(crate) struct FontInfo {
    /// `BaseFont` of the Type 0 dictionary.
    pub(crate) type0_name: String,
    /// `BaseFont` of the descendant CID font.
    pub(crate) base_font: String,
    /// `Subtype` of the descendant CID font.
    pub(crate) cid_subtype: String,
    /// CID → width (thousandths of an em) from `/W`.
    pub(crate) widths: BTreeMap<u16, f64>,
    /// CID → text from the `ToUnicode` CMap.
    pub(crate) to_unicode: BTreeMap<u16, String>,
    /// The font descriptor.
    pub(crate) descriptor: Dictionary,
    /// Key of the font program in the descriptor (`FontFile2` or `FontFile3`).
    pub(crate) program_key: String,
    /// The font program's stream dictionary.
    pub(crate) program_dict: Dictionary,
    /// The decoded font program.
    pub(crate) program: Vec<u8>,
    /// `true` when the CID font has `/CIDToGIDMap /Identity`.
    pub(crate) identity_cid_to_gid: bool,
}

/// Reads the font registered as `name` (e.g. `"F0"`) in the resources of page `number`.
pub(crate) fn font_info(doc: &lopdf::Document, number: u32, name: &str) -> FontInfo {
    let fonts = resources(doc, number, "Font").expect("the page has fonts");
    let type0 = get_dict(doc, fonts, name);
    assert_eq!(get_name(doc, type0, "Type"), "Font");
    assert_eq!(get_name(doc, type0, "Subtype"), "Type0");
    assert_eq!(get_name(doc, type0, "Encoding"), "Identity-H");
    let descendants = get(doc, type0, "DescendantFonts").as_array().expect("array");
    assert_eq!(descendants.len(), 1);
    let cid_font = resolve(doc, &descendants[0]).as_dict().expect("CID font dictionary");
    assert_eq!(get_name(doc, cid_font, "Type"), "Font");
    let system = get_dict(doc, cid_font, "CIDSystemInfo");
    assert_eq!(get(doc, system, "Registry").as_str().expect("string"), b"Adobe");
    assert_eq!(get(doc, system, "Ordering").as_str().expect("string"), b"Identity");
    assert_eq!(get(doc, system, "Supplement").as_i64().expect("integer"), 0);

    let mut widths = BTreeMap::new();
    if cid_font.has(b"W") {
        let w = get(doc, cid_font, "W").as_array().expect("array");
        let mut i = 0;
        while i < w.len() {
            let first = u16::try_from(w[i].as_i64().expect("first CID")).expect("16-bit CID");
            let run = w[i + 1].as_array().expect("this writer only emits `c [w ...]` runs");
            assert!(run.len() <= 8191, "array too long for PDF 1.3 consumers");
            for (k, width) in run.iter().enumerate() {
                let cid = first + u16::try_from(k).expect("small run");
                let previous = widths.insert(cid, f64::from(width.as_float().expect("number")));
                assert!(previous.is_none(), "CID {cid} has two widths");
            }
            i += 2;
        }
    }

    let mut to_unicode = BTreeMap::new();
    if type0.has(b"ToUnicode") {
        let cmap = String::from_utf8(get_stream_data(doc, type0, "ToUnicode")).expect("ASCII CMap");
        assert!(cmap.contains("begincmap") && cmap.contains("endcmap"));
        assert!(cmap.contains("<0000> <FFFF>"), "two-byte code space");
        let mut inside = false;
        for line in cmap.lines() {
            if line.ends_with("beginbfchar") {
                let count: usize = line.split(' ').next().expect("count").parse().expect("number");
                assert!(count <= 100, "at most 100 mappings per section");
                inside = true;
            } else if line == "endbfchar" {
                inside = false;
            } else if inside {
                let (code, text) = line.split_once(' ').expect("`<code> <text>`");
                let code = code.trim_matches(|c| c == '<' || c == '>');
                let text = text.trim_matches(|c| c == '<' || c == '>');
                let cid = u16::from_str_radix(code, 16).expect("hex CID");
                let units: Vec<u16> = (0..text.len() / 4)
                    .map(|k| u16::from_str_radix(&text[k * 4..k * 4 + 4], 16).expect("hex"))
                    .collect();
                let previous = to_unicode.insert(cid, String::from_utf16(&units).expect("UTF-16"));
                assert!(previous.is_none(), "CID {cid} is mapped twice");
            }
        }
    }

    let descriptor = get_dict(doc, cid_font, "FontDescriptor").clone();
    assert_eq!(get_name(doc, &descriptor, "Type"), "FontDescriptor");
    let program_key = ["FontFile2", "FontFile3", "FontFile"]
        .into_iter()
        .find(|k| descriptor.has(k.as_bytes()))
        .expect("the font program is embedded")
        .to_owned();
    let program_object = get(doc, &descriptor, &program_key);
    let identity_cid_to_gid = cid_font.has(b"CIDToGIDMap")
        && get(doc, cid_font, "CIDToGIDMap").as_name().expect("name") == b"Identity";

    FontInfo {
        type0_name: get_name(doc, type0, "BaseFont"),
        base_font: get_name(doc, cid_font, "BaseFont"),
        cid_subtype: get_name(doc, cid_font, "Subtype"),
        widths,
        to_unicode,
        program_key,
        program_dict: program_object.as_stream().expect("stream").dict.clone(),
        program: stream_data(program_object),
        descriptor,
        identity_cid_to_gid,
    }
}

/// A glyph as a PDF consumer would place it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlacedGlyph {
    /// Resource name of the font.
    pub(crate) font: String,
    /// Font size in content units (millimetres on our pages).
    pub(crate) size: f64,
    pub(crate) cid: u16,
    /// Glyph origin in content units (the scene's page millimetres for top-level text).
    pub(crate) x: f64,
    pub(crate) y: f64,
}

/// Interprets the text operators of a content stream the way a PDF consumer does
/// (`Tf`, `Tm`, `TJ`/`Tj` with the advance widths from `widths`), ignoring `cm`.
pub(crate) fn placed_glyphs(content: &[u8], widths: &dyn Fn(&str, u16) -> f64) -> Vec<PlacedGlyph> {
    let operations = lopdf::content::Content::decode(content).expect("content parses").operations;
    let number = |o: &Object| f64::from(o.as_float().expect("number"));
    let mut out = Vec::new();
    let (mut font, mut size) = (String::new(), 0.0);
    let mut matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut tx = 0.0;
    let mut in_text = false;
    for op in &operations {
        match op.operator.as_str() {
            "BT" => {
                assert!(!in_text, "nested BT");
                in_text = true;
                matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                tx = 0.0;
            }
            "ET" => {
                assert!(in_text, "ET without BT");
                in_text = false;
            }
            "Tf" => {
                font =
                    String::from_utf8_lossy(op.operands[0].as_name().expect("name")).into_owned();
                size = number(&op.operands[1]);
            }
            "Tm" => {
                assert!(in_text);
                for (slot, operand) in matrix.iter_mut().zip(&op.operands) {
                    *slot = number(operand);
                }
                tx = 0.0;
            }
            "TJ" | "Tj" => {
                assert!(in_text, "text shown outside BT/ET");
                let elements: Vec<Object> = match &op.operands[0] {
                    Object::Array(a) => a.clone(),
                    other => vec![other.clone()],
                };
                assert!(elements.len() <= 8191, "array too long for PDF 1.3 consumers");
                for element in &elements {
                    if let Ok(bytes) = element.as_str() {
                        assert!(bytes.len() <= 32767, "string too long");
                        assert_eq!(bytes.len() % 2, 0, "two bytes per CID");
                        for pair in bytes.chunks_exact(2) {
                            let cid = u16::from_be_bytes([pair[0], pair[1]]);
                            out.push(PlacedGlyph {
                                font: font.clone(),
                                size,
                                cid,
                                x: matrix[4] + tx * matrix[0],
                                y: matrix[5] + tx * matrix[1],
                            });
                            tx += widths(&font, cid) / 1000.0 * size;
                        }
                    } else {
                        tx -= number(element) / 1000.0 * size;
                    }
                }
            }
            "Td" | "TD" | "T*" | "'" | "\"" | "Tc" | "Tw" | "Tz" | "TL" | "Ts" => {
                panic!("unexpected text operator {}", op.operator)
            }
            _ => {}
        }
    }
    assert!(!in_text, "unterminated text object");
    out
}
