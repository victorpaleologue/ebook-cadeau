//! JPEG header inspection.
//!
//! JPEG files are embedded as they are (`DCTDecode`), so the writer never decodes pixels.
//! It only walks the marker segments up to the first scan to check that the file really
//! is the image the document describes and to find the Adobe `APP14` marker, which tells
//! whether CMYK samples are stored inverted.

/// What the header of a JPEG file says about the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JpegInfo {
    /// Width in pixels.
    pub(crate) width: u32,
    /// Height in pixels.
    pub(crate) height: u32,
    /// Number of colour components in the frame.
    pub(crate) components: u8,
    /// The colour transform byte of the Adobe `APP14` marker, if the marker is present
    /// (0 = none/CMYK/RGB, 1 = YCbCr, 2 = YCCK).
    pub(crate) adobe_transform: Option<u8>,
}

const SOI: u8 = 0xD8;
const EOI: u8 = 0xD9;
const SOS: u8 = 0xDA;
const APP14: u8 = 0xEE;

/// Parses the marker segments of a JPEG file up to the first scan.
///
/// Only the encodings that PDF's `DCTDecode` filter understands are accepted: baseline,
/// extended sequential and progressive Huffman-coded DCT with 8-bit samples.
pub(crate) fn sniff(data: &[u8]) -> Result<JpegInfo, &'static str> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != SOI {
        return Err("not a JPEG file (missing SOI marker)");
    }
    let mut pos = 2usize;
    let mut frame: Option<(u32, u32, u8)> = None;
    let mut adobe_transform = None;

    loop {
        // Every marker starts with 0xFF, optionally preceded by more 0xFF fill bytes.
        if data.get(pos) != Some(&0xFF) {
            return Err("corrupt JPEG marker stream");
        }
        while data.get(pos) == Some(&0xFF) {
            pos += 1;
        }
        let Some(&marker) = data.get(pos) else {
            return Err("truncated JPEG file");
        };
        pos += 1;

        match marker {
            // Stand-alone markers without a length field.
            0x01 | 0xD0..=0xD7 => continue,
            SOI => return Err("corrupt JPEG marker stream"),
            EOI | SOS => break,
            _ => {}
        }

        let (Some(&hi), Some(&lo)) = (data.get(pos), data.get(pos + 1)) else {
            return Err("truncated JPEG file");
        };
        let length = usize::from(u16::from_be_bytes([hi, lo]));
        if length < 2 {
            return Err("corrupt JPEG segment length");
        }
        let Some(segment) = data.get(pos + 2..pos + length) else {
            return Err("truncated JPEG file");
        };
        pos += length;

        match marker {
            // SOF0 (baseline), SOF1 (extended sequential), SOF2 (progressive).
            0xC0..=0xC2 => {
                if frame.is_some() {
                    return Err("JPEG with more than one frame");
                }
                if segment.len() < 6 {
                    return Err("corrupt JPEG frame header");
                }
                if segment[0] != 8 {
                    return Err("only 8-bit JPEG samples are supported");
                }
                let height = u32::from(u16::from_be_bytes([segment[1], segment[2]]));
                let width = u32::from(u16::from_be_bytes([segment[3], segment[4]]));
                let components = segment[5];
                if width == 0 || height == 0 {
                    return Err("JPEG with a zero dimension");
                }
                if segment.len() < 6 + 3 * usize::from(components) {
                    return Err("corrupt JPEG frame header");
                }
                frame = Some((width, height, components));
            }
            // Lossless, hierarchical and arithmetic-coded frames (DHT = C4, JPG = C8 and
            // DAC = CC are tables, not frames).
            0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {
                return Err("unsupported JPEG encoding (lossless, hierarchical or arithmetic)");
            }
            APP14 if segment.len() >= 12 && segment.starts_with(b"Adobe") => {
                adobe_transform = Some(segment[11]);
            }
            _ => {}
        }
    }

    let Some((width, height, components)) = frame else {
        return Err("JPEG without a frame header before the first scan");
    };
    Ok(JpegInfo { width, height, components, adobe_transform })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(marker: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![0xFF, marker];
        out.extend_from_slice(&u16::try_from(payload.len() + 2).expect("short").to_be_bytes());
        out.extend_from_slice(payload);
        out
    }

    fn sof(marker: u8, precision: u8, width: u16, height: u16, components: u8) -> Vec<u8> {
        let mut payload = vec![precision];
        payload.extend_from_slice(&height.to_be_bytes());
        payload.extend_from_slice(&width.to_be_bytes());
        payload.push(components);
        for i in 0..components {
            payload.extend_from_slice(&[i + 1, 0x11, 0]);
        }
        segment(marker, &payload)
    }

    fn file(segments: &[Vec<u8>]) -> Vec<u8> {
        let mut out = vec![0xFF, SOI];
        for s in segments {
            out.extend_from_slice(s);
        }
        out.extend_from_slice(&[0xFF, SOS, 0, 2, 0xFF, EOI]);
        out
    }

    #[test]
    fn reads_frame_header_and_adobe_marker() {
        let jfif = segment(0xE0, b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
        let info = sniff(&file(&[jfif.clone(), sof(0xC0, 8, 640, 480, 3)])).expect("valid");
        assert_eq!(
            info,
            JpegInfo { width: 640, height: 480, components: 3, adobe_transform: None }
        );

        let adobe = segment(APP14, b"Adobe\0\x64\0\0\0\0\x02");
        let info = sniff(&file(&[adobe, sof(0xC2, 8, 1, 65535, 4)])).expect("valid");
        assert_eq!(
            info,
            JpegInfo { width: 1, height: 65535, components: 4, adobe_transform: Some(2) }
        );

        // The Adobe marker may also follow the frame header; fill bytes are legal.
        let mut data = vec![0xFF, SOI];
        data.extend_from_slice(&sof(0xC1, 8, 2, 3, 1));
        data.extend_from_slice(&[0xFF, 0xFF, 0xFF]);
        data.extend_from_slice(&segment(APP14, b"Adobe\0\x64\0\0\0\0\0")[1..]);
        data.extend_from_slice(&[0xFF, SOS, 0, 2]);
        let info = sniff(&data).expect("valid");
        assert_eq!(info.adobe_transform, Some(0));
        assert_eq!(info.components, 1);
    }

    #[test]
    fn rejects_what_dctdecode_cannot_read() {
        assert!(sniff(&file(&[sof(0xC0, 12, 8, 8, 3)])).is_err(), "12-bit");
        assert!(sniff(&file(&[sof(0xC3, 8, 8, 8, 3)])).is_err(), "lossless");
        assert!(sniff(&file(&[sof(0xC9, 8, 8, 8, 3)])).is_err(), "arithmetic");
        assert!(sniff(&file(&[sof(0xC0, 8, 0, 8, 3)])).is_err(), "zero width");
        assert!(sniff(&file(&[sof(0xC0, 8, 8, 0, 3)])).is_err(), "zero height");
        assert!(sniff(&file(&[])).is_err(), "no frame");
        assert!(
            sniff(&file(&[sof(0xC0, 8, 8, 8, 3), sof(0xC0, 8, 8, 8, 3)])).is_err(),
            "two frames"
        );
    }

    #[test]
    fn never_panics_on_hostile_input() {
        assert!(sniff(&[]).is_err());
        assert!(sniff(&[0xFF]).is_err());
        assert!(sniff(b"\x89PNG\r\n\x1a\n").is_err());
        assert!(sniff(&[0xFF, SOI, 0xFF]).is_err());
        assert!(sniff(&[0xFF, SOI, 0xFF, 0xE0, 0x00]).is_err());
        assert!(sniff(&[0xFF, SOI, 0xFF, 0xE0, 0x00, 0x01]).is_err(), "length below 2");
        assert!(sniff(&[0xFF, SOI, 0xFF, 0xE0, 0xFF, 0xFF, 1, 2, 3]).is_err(), "overlong");
        assert!(sniff(&[0xFF, SOI, 0x12, 0x34]).is_err(), "garbage after SOI");
        assert!(sniff(&[0xFF, SOI, 0xFF, SOI]).is_err(), "second SOI");

        // Every truncation of a valid header is an error, never a panic.
        let valid = file(&[segment(APP14, b"Adobe\0\x64\0\0\0\0\x02"), sof(0xC0, 8, 9, 9, 4)]);
        assert!(sniff(&valid).is_ok());
        for len in 0..valid.len() - 6 {
            assert!(sniff(&valid[..len]).is_err(), "prefix of {len} bytes");
        }
        // Flipping any single byte must not panic either.
        for i in 0..valid.len() {
            let mut mutated = valid.clone();
            mutated[i] ^= 0xFF;
            let _ = sniff(&mutated);
        }
    }
}
