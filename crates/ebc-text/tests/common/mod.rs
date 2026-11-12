//! Shared test helpers: the bundled OFL fonts and a tiny synthetic font builder.
#![allow(dead_code, unreachable_pub, missing_docs)]

use ebc_text::Font;

pub const ATKINSON: &[u8] =
    include_bytes!("../../../../assets/fonts/AtkinsonHyperlegible-Regular.ttf");
pub const ATKINSON_BOLD: &[u8] =
    include_bytes!("../../../../assets/fonts/AtkinsonHyperlegible-Bold.ttf");
pub const ATKINSON_ITALIC: &[u8] =
    include_bytes!("../../../../assets/fonts/AtkinsonHyperlegible-Italic.ttf");
pub const SPECTRAL: &[u8] = include_bytes!("../../../../assets/fonts/Spectral-Regular.ttf");
pub const SPECTRAL_ITALIC: &[u8] = include_bytes!("../../../../assets/fonts/Spectral-Italic.ttf");
pub const SPECTRAL_SEMIBOLD: &[u8] =
    include_bytes!("../../../../assets/fonts/Spectral-SemiBold.ttf");
pub const BEBAS: &[u8] = include_bytes!("../../../../assets/fonts/BebasNeue-Regular.ttf");
pub const COURIER_BOLD: &[u8] = include_bytes!("../../../../assets/fonts/CourierPrime-Bold.ttf");

/// Every bundled font with a short label for assertion messages.
pub const ALL: [(&str, &[u8]); 8] = [
    ("Atkinson Regular", ATKINSON),
    ("Atkinson Bold", ATKINSON_BOLD),
    ("Atkinson Italic", ATKINSON_ITALIC),
    ("Spectral Regular", SPECTRAL),
    ("Spectral Italic", SPECTRAL_ITALIC),
    ("Spectral SemiBold", SPECTRAL_SEMIBOLD),
    ("Bebas Neue", BEBAS),
    ("Courier Prime Bold", COURIER_BOLD),
];

pub fn font(bytes: &[u8]) -> Font {
    Font::from_bytes(bytes, 0).expect("bundled font parses")
}

pub fn atkinson() -> Font {
    font(ATKINSON)
}
pub fn spectral() -> Font {
    font(SPECTRAL)
}
pub fn bebas() -> Font {
    font(BEBAS)
}
pub fn courier() -> Font {
    font(COURIER_BOLD)
}

pub fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

/// A small deterministic PRNG (xorshift64*), so that fuzz-style tests are reproducible.
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n.max(1) as u64) as usize
    }
}

/// Finds a table in an sfnt file and returns `(offset, length)` of its data.
pub fn find_table(font: &[u8], tag: &[u8; 4]) -> Option<(usize, usize)> {
    let count = u16::from_be_bytes([font[4], font[5]]) as usize;
    (0..count).map(|i| 12 + i * 16).find(|&r| &font[r..r + 4] == tag).map(|r| {
        let offset = u32::from_be_bytes([font[r + 8], font[r + 9], font[r + 10], font[r + 11]]);
        let length = u32::from_be_bytes([font[r + 12], font[r + 13], font[r + 14], font[r + 15]]);
        (offset as usize, length as usize)
    })
}

/// Returns a copy of `font` whose table `tag` is hidden under another tag, i.e. a font
/// without that table. (Parsers do not verify sfnt checksums.)
pub fn without_table(font: &[u8], tag: &[u8; 4]) -> Vec<u8> {
    let mut out = font.to_vec();
    let count = u16::from_be_bytes([font[4], font[5]]) as usize;
    for record in (0..count).map(|i| 12 + i * 16) {
        if &out[record..record + 4] == tag {
            out[record..record + 4].copy_from_slice(b"zzzz");
        }
    }
    out
}

/// Returns a copy of `font` with `OS/2.fsType` (and optionally the table version) set.
pub fn with_fs_type(font: &[u8], fs_type: u16, version: Option<u16>) -> Vec<u8> {
    let (offset, length) = find_table(font, b"OS/2").expect("font has an OS/2 table");
    assert!(length >= 10);
    let mut out = font.to_vec();
    out[offset + 8..offset + 10].copy_from_slice(&fs_type.to_be_bytes());
    if let Some(version) = version {
        out[offset..offset + 2].copy_from_slice(&version.to_be_bytes());
    }
    out
}

// ---------------------------------------------------------------------------------------
// Synthetic TrueType fonts
// ---------------------------------------------------------------------------------------

/// One `name` table record of a synthetic font.
pub struct NameRecord {
    pub platform: u16,
    pub encoding: u16,
    pub language: u16,
    pub name_id: u16,
    pub bytes: Vec<u8>,
}

impl NameRecord {
    /// A Windows / Unicode BMP / US English record.
    pub fn windows(name_id: u16, text: &str) -> Self {
        let bytes = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        Self { platform: 3, encoding: 1, language: 0x0409, name_id, bytes }
    }

    /// A Macintosh / Roman / English record with raw Mac Roman bytes.
    pub fn mac(name_id: u16, bytes: &[u8]) -> Self {
        Self { platform: 1, encoding: 0, language: 0, name_id, bytes: bytes.to_vec() }
    }
}

/// Description of a synthetic font. All glyphs are axis-aligned rectangles.
///
/// Glyphs: 0 `.notdef` (empty), 1 "A" and 2 "H" (100..500 x 0..700, advance 600),
/// 3 "x" (100..400 x 0..500, advance 500), 4 space (empty, advance 300).
#[derive(Default)]
pub struct SyntheticFont {
    /// `Some((version, fs_type, with_heights))`: add an `OS/2` table. `with_heights`
    /// fills `sxHeight`/`sCapHeight` (version >= 2 only) with 480/680.
    pub os2: Option<(u16, u16, bool)>,
    pub names: Vec<NameRecord>,
    /// Add a `wght` axis (100..900, default 400). At the maximum, every rectangle is
    /// 100 units wider (and its advance 100 larger); at the minimum, 50 units narrower.
    pub variable: bool,
    pub units_per_em: u16,
    /// `hhea` ascender / descender (0, 0 to test the fallbacks).
    pub ascender: i16,
    pub descender: i16,
}

impl SyntheticFont {
    pub fn new() -> Self {
        Self { units_per_em: 1000, ascender: 800, descender: -200, ..Self::default() }
    }

    pub fn build(&self) -> Vec<u8> {
        self.build_at(0)
    }

    /// Builds a TrueType collection containing the given faces, in order.
    pub fn collection(faces: &[&SyntheticFont]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"ttcf");
        push_u32(&mut out, 0x0001_0000);
        push_u32(&mut out, faces.len() as u32);
        let header = 12 + 4 * faces.len();
        let mut bodies: Vec<Vec<u8>> = Vec::new();
        let mut at = header;
        for face in faces {
            push_u32(&mut out, at as u32);
            let body = face.build_at(at);
            at += body.len();
            bodies.push(body);
        }
        for body in bodies {
            out.extend_from_slice(&body);
        }
        out
    }

    fn build_at(&self, base: usize) -> Vec<u8> {
        // (xmin, ymin, xmax, ymax, advance); None = empty glyph.
        type Glyph = (Option<(i16, i16, i16, i16)>, u16);
        let glyphs: [Glyph; 5] = [
            (None, 500),
            (Some((100, 0, 500, 700)), 600),
            (Some((100, 0, 500, 700)), 600),
            (Some((100, 0, 400, 500)), 500),
            (None, 300),
        ];

        let mut glyf = Vec::new();
        let mut loca = Vec::new();
        for (rect, _) in &glyphs {
            loca.extend_from_slice(&((glyf.len() / 2) as u16).to_be_bytes());
            if let Some((x0, y0, x1, y1)) = *rect {
                push_i16(&mut glyf, 1); // one contour
                for v in [x0, y0, x1, y1] {
                    push_i16(&mut glyf, v);
                }
                push_u16(&mut glyf, 3); // last point of the contour
                push_u16(&mut glyf, 0); // no instructions
                glyf.extend_from_slice(&[0x01; 4]); // four on-curve points, 16-bit deltas
                // Clockwise (TrueType outer contour): bottom-left, top-left, top-right,
                // bottom-right; coordinates are deltas from the previous point.
                for dx in [x0, 0, x1 - x0, 0] {
                    push_i16(&mut glyf, dx);
                }
                for dy in [y0, y1 - y0, 0, y0 - y1] {
                    push_i16(&mut glyf, dy);
                }
                if glyf.len() % 2 == 1 {
                    glyf.push(0);
                }
            }
        }
        loca.extend_from_slice(&((glyf.len() / 2) as u16).to_be_bytes());

        let mut head = Vec::new();
        push_u32(&mut head, 0x0001_0000); // version
        push_u32(&mut head, 0x0001_0000); // font revision
        push_u32(&mut head, 0); // checksum adjustment
        push_u32(&mut head, 0x5F0F_3CF5); // magic
        push_u16(&mut head, 0); // flags
        push_u16(&mut head, self.units_per_em);
        head.extend_from_slice(&[0; 16]); // created, modified
        for v in [0i16, -200, 700, 800] {
            push_i16(&mut head, v); // font bounding box
        }
        push_u16(&mut head, 0); // mac style
        push_u16(&mut head, 8); // lowest rec ppem
        push_i16(&mut head, 2); // direction hint
        push_i16(&mut head, 0); // short loca
        push_i16(&mut head, 0); // glyph data format

        let mut hhea = Vec::new();
        push_u32(&mut hhea, 0x0001_0000);
        push_i16(&mut hhea, self.ascender);
        push_i16(&mut hhea, self.descender);
        push_i16(&mut hhea, 0); // line gap
        push_u16(&mut hhea, 600); // advance width max
        hhea.extend_from_slice(&[0; 22]); // bearings, extent, caret, reserved, format
        push_u16(&mut hhea, glyphs.len() as u16); // number of h metrics

        let mut maxp = Vec::new();
        push_u32(&mut maxp, 0x0001_0000);
        push_u16(&mut maxp, glyphs.len() as u16);
        maxp.extend_from_slice(&[0; 26]);

        let mut hmtx = Vec::new();
        for (rect, advance) in &glyphs {
            push_u16(&mut hmtx, *advance);
            push_i16(&mut hmtx, rect.map_or(0, |r| r.0));
        }

        // cmap: one Windows / Unicode BMP subtable in format 0 (byte encoding).
        let mut cmap = Vec::new();
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 1);
        push_u16(&mut cmap, 3);
        push_u16(&mut cmap, 1);
        push_u32(&mut cmap, 12);
        push_u16(&mut cmap, 0); // format 0
        push_u16(&mut cmap, 262);
        push_u16(&mut cmap, 0);
        let mut map = [0u8; 256];
        map[b'A' as usize] = 1;
        map[b'H' as usize] = 2;
        map[b'x' as usize] = 3;
        map[b' ' as usize] = 4;
        cmap.extend_from_slice(&map);

        let mut tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
            (b"cmap", cmap),
            (b"glyf", glyf),
            (b"head", head),
            (b"hhea", hhea),
            (b"hmtx", hmtx),
            (b"loca", loca),
            (b"maxp", maxp),
        ];

        if let Some((version, fs_type, with_heights)) = self.os2 {
            let length = match version {
                0 => 78,
                1 => 86,
                2..=4 => 96,
                _ => 100,
            };
            let mut os2 = vec![0u8; length];
            os2[0..2].copy_from_slice(&version.to_be_bytes());
            os2[4..6].copy_from_slice(&400u16.to_be_bytes()); // weight class
            os2[6..8].copy_from_slice(&5u16.to_be_bytes()); // width class
            os2[8..10].copy_from_slice(&fs_type.to_be_bytes());
            if version >= 2 && with_heights {
                os2[86..88].copy_from_slice(&480i16.to_be_bytes());
                os2[88..90].copy_from_slice(&680i16.to_be_bytes());
            }
            tables.push((b"OS/2", os2));
        }

        if !self.names.is_empty() {
            let mut name = Vec::new();
            let mut storage = Vec::new();
            push_u16(&mut name, 0);
            push_u16(&mut name, self.names.len() as u16);
            push_u16(&mut name, (6 + 12 * self.names.len()) as u16);
            for record in &self.names {
                push_u16(&mut name, record.platform);
                push_u16(&mut name, record.encoding);
                push_u16(&mut name, record.language);
                push_u16(&mut name, record.name_id);
                push_u16(&mut name, record.bytes.len() as u16);
                push_u16(&mut name, storage.len() as u16);
                storage.extend_from_slice(&record.bytes);
            }
            name.extend_from_slice(&storage);
            tables.push((b"name", name));
        }

        if self.variable {
            let mut fvar = Vec::new();
            push_u32(&mut fvar, 0x0001_0000);
            push_u16(&mut fvar, 16); // offset to the axes
            push_u16(&mut fvar, 2); // reserved
            push_u16(&mut fvar, 1); // axis count
            push_u16(&mut fvar, 20); // axis size
            push_u16(&mut fvar, 0); // instance count
            push_u16(&mut fvar, 8); // instance size
            fvar.extend_from_slice(b"wght");
            for value in [100i32, 400, 900] {
                push_u32(&mut fvar, (value << 16) as u32); // 16.16 fixed
            }
            push_u16(&mut fvar, 0); // flags
            push_u16(&mut fvar, 256); // axis name id
            tables.push((b"fvar", fvar));

            // gvar: per glyph, two tuples (peak +1 and peak -1) moving the right-hand
            // points and the advance phantom point.
            let mut data = Vec::new();
            let mut offsets = Vec::new();
            for (rect, _) in &glyphs {
                offsets.push(data.len());
                if rect.is_none() {
                    continue;
                }
                // Points: 0 BL, 1 TL, 2 TR, 3 BR, then phantoms 4 (left), 5 (right),
                // 6 (top), 7 (bottom).
                let tuple = |delta: i8| -> Vec<u8> {
                    let mut t = vec![0x00]; // point count 0 = all points
                    t.push(0x07); // 8 x deltas as bytes
                    t.extend_from_slice(&[0, 0, delta as u8, delta as u8, 0, delta as u8, 0, 0]);
                    t.push(0x87); // 8 zero y deltas
                    t
                };
                let (wide, narrow) = (tuple(100), tuple(-50));
                push_u16(&mut data, 2); // two tuples, no shared point numbers
                push_u16(&mut data, 4 + 2 * 6); // offset to the serialized data
                for (body, peak) in [(&wide, 0x4000u16), (&narrow, 0xC000)] {
                    push_u16(&mut data, body.len() as u16);
                    push_u16(&mut data, 0x8000 | 0x2000); // embedded peak, private points
                    push_u16(&mut data, peak); // F2Dot14 +1.0 / -1.0
                }
                data.extend_from_slice(&wide);
                data.extend_from_slice(&narrow);
                if data.len() % 2 == 1 {
                    data.push(0);
                }
            }
            offsets.push(data.len());

            let mut gvar = Vec::new();
            let header = 20 + 2 * offsets.len();
            push_u32(&mut gvar, 0x0001_0000);
            push_u16(&mut gvar, 1); // axis count
            push_u16(&mut gvar, 0); // shared tuple count
            push_u32(&mut gvar, header as u32); // shared tuples offset
            push_u16(&mut gvar, glyphs.len() as u16);
            push_u16(&mut gvar, 0); // short offsets
            push_u32(&mut gvar, header as u32); // glyph variation data offset
            for offset in &offsets {
                push_u16(&mut gvar, (offset / 2) as u16);
            }
            gvar.extend_from_slice(&data);
            tables.push((b"gvar", gvar));
        }

        build_sfnt_at(tables, base)
    }
}

fn push_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn push_i16(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Assembles an sfnt (TrueType) file from tables. Checksums are left at zero.
pub fn build_sfnt(tables: Vec<(&[u8; 4], Vec<u8>)>) -> Vec<u8> {
    build_sfnt_at(tables, 0)
}

/// Like [`build_sfnt`], for a face stored `base` bytes into a font collection.
pub fn build_sfnt_at(mut tables: Vec<(&[u8; 4], Vec<u8>)>, base: usize) -> Vec<u8> {
    tables.sort_by_key(|(tag, _)| **tag);
    let count = tables.len() as u16;
    let mut out = Vec::new();
    push_u32(&mut out, 0x0001_0000);
    push_u16(&mut out, count);
    out.extend_from_slice(&[0; 6]); // search range, entry selector, range shift
    let mut offset = base + 12 + 16 * tables.len();
    for (tag, data) in &tables {
        out.extend_from_slice(*tag);
        push_u32(&mut out, 0);
        push_u32(&mut out, offset as u32);
        push_u32(&mut out, data.len() as u32);
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in &tables {
        out.extend_from_slice(data);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    out
}
