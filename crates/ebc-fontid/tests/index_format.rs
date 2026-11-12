//! The font index: building, licences, serialisation and hostile input.

mod common;

use common::{bundled_fonts, fresh_index};
use ebc_fontid::{
    BUNDLED_FONT_FILES, BUNDLED_INDEX_BYTES, FontIdError, FontIndex, IndexEntry, IndexEntryMeta,
    OPEN_LICENCES, REFERENCE_CHARS, bundled_index,
};

fn fnv1a(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5u32, |h, &b| (h ^ u32::from(b)).wrapping_mul(0x0100_0193))
}

/// Replaces the trailing checksum so that a tampered body passes the checksum test.
fn reseal(bytes: &mut [u8]) {
    let body = bytes.len() - 4;
    let sum = fnv1a(&bytes[..body]).to_le_bytes();
    bytes[body..].copy_from_slice(&sum);
}

#[test]
fn bundled_index_lists_the_eight_open_fonts() {
    let index = bundled_index().expect("embedded index parses");
    assert_eq!(index.len(), 8);
    assert_eq!(index.chars().iter().collect::<String>(), REFERENCE_CHARS);
    let mut families: Vec<&str> = index.entries().iter().map(|e| e.family.as_str()).collect();
    families.dedup();
    assert_eq!(families, ["Atkinson Hyperlegible", "Spectral", "Bebas Neue", "Courier Prime"]);
    for (entry, file) in index.entries().iter().zip(BUNDLED_FONT_FILES) {
        assert_eq!(entry.licence, "OFL-1.1", "{entry:?}");
        assert!(OPEN_LICENCES.contains(&entry.licence.as_str()));
        assert_eq!(entry.source_url, format!("assets/fonts/{file}"));
        assert_eq!(entry.coverage(), 65, "{entry:?}");
        assert!(!entry.style.is_empty());
    }
    let styles: Vec<&str> = index.entries().iter().map(|e| e.style.as_str()).collect();
    assert_eq!(&styles[..6], ["Regular", "Bold", "Italic", "Regular", "SemiBold", "Italic"]);
    // Small enough to embed.
    assert!(BUNDLED_INDEX_BYTES.len() < 100_000, "{}", BUNDLED_INDEX_BYTES.len());
}

#[test]
fn building_is_deterministic_and_round_trips() {
    let fonts = bundled_fonts();
    let index = fresh_index(&fonts[..3]);
    let bytes = index.to_bytes();
    assert_eq!(bytes, fresh_index(&fonts[..3]).to_bytes(), "same input, same bytes");
    let back = FontIndex::from_bytes(&bytes).expect("round trip");
    assert_eq!(back, index);
    assert_eq!(back.to_bytes(), bytes);
    assert_eq!(back.entries()[1].meta(), fonts[1].0);
    // Subsets and joins keep entries intact.
    let two = FontIndex::from_entries(index.entries()[1..].iter().cloned());
    assert_eq!(two.len(), 2);
    assert_eq!(two.entries()[0], index.entries()[1]);
    let alone = IndexEntry::from_font(fonts[1].0.clone(), &fonts[1].1);
    assert_eq!(alone, index.entries()[1], "an entry described alone equals the indexed one");
    let joined = FontIndex::from_entries([alone, index.entries()[0].clone()]);
    assert_eq!(FontIndex::from_bytes(&joined.to_bytes()).expect("round trip"), joined);
    assert!(FontIndex::from_entries([]).is_empty());
}

#[test]
fn only_openly_licensed_fonts_enter_a_checked_index() {
    let fonts = bundled_fonts();
    for licence in ["", "unknown", "GPL-3.0-only", "proprietary", "ofl-1.1", "OFL-1.1 "] {
        let mut bad = fonts[..2].to_vec();
        bad[1].0.licence = licence.to_string();
        match FontIndex::build_checked(&bad) {
            Err(FontIdError::LicenceNotAllowed { family, licence: l, .. }) => {
                assert_eq!(family, bad[1].0.family);
                assert_eq!(l, licence);
            }
            other => panic!("licence {licence:?} must be refused, got {other:?}"),
        }
    }
    let mut apache = fonts[..1].to_vec();
    apache[0].0.licence = "Apache-2.0".to_string();
    assert_eq!(FontIndex::build_checked(&apache).expect("allowed").len(), 1);
    // Metadata read from the font alone does not carry an SPDX identifier.
    let from_font = IndexEntryMeta::from_font(&fonts[0].1, "x");
    assert_eq!(from_font.family, "Atkinson Hyperlegible");
    assert!(FontIndex::build_checked(&[(from_font, fonts[0].1.clone())]).is_err());
    assert!(FontIndex::build_checked(&[]).expect("nothing to refuse").is_empty());
}

#[test]
fn every_truncation_is_an_error_not_a_panic() {
    let fonts = bundled_fonts();
    let bytes = fresh_index(&fonts[..2]).to_bytes();
    assert!(bytes.len() > 10_000);
    for len in 0..bytes.len() {
        // Every header offset, then every 7th byte of the body (and the last 64).
        if len > 400 && len % 7 != 0 && len + 64 < bytes.len() {
            continue;
        }
        let cut = &bytes[..len];
        assert!(FontIndex::from_bytes(cut).is_err(), "truncated to {len} bytes");
        // Truncated but with a matching checksum: the structure checks must hold too.
        if len >= 24 {
            let mut sealed = cut.to_vec();
            reseal(&mut sealed);
            assert!(FontIndex::from_bytes(&sealed).is_err(), "resealed at {len} bytes");
        }
    }
    assert_eq!(FontIndex::from_bytes(b""), Err(FontIdError::BadMagic));
    assert_eq!(FontIndex::from_bytes(b"EBCFIDX"), Err(FontIdError::BadMagic));
    assert_eq!(FontIndex::from_bytes(b"not an index at all"), Err(FontIdError::BadMagic));
}

#[test]
fn corrupted_fields_are_rejected() {
    let fonts = bundled_fonts();
    let good = fresh_index(&fonts[..1]).to_bytes();

    // A flipped bit anywhere is caught by the checksum (or an earlier check).
    for position in (0..good.len()).step_by(97) {
        let mut bad = good.clone();
        bad[position] ^= 0x10;
        assert!(FontIndex::from_bytes(&bad).is_err(), "bit flip at {position}");
    }

    let tamper = |edit: &dyn Fn(&mut Vec<u8>)| {
        let mut bad = good.clone();
        edit(&mut bad);
        reseal(&mut bad);
        FontIndex::from_bytes(&bad)
    };
    // Version and layout.
    assert_eq!(
        tamper(&|b| b[8] = 9),
        Err(FontIdError::UnsupportedVersion { found: 9, supported: FontIndex::FORMAT_VERSION })
    );
    assert_eq!(tamper(&|b| b[10] = 5), Err(FontIdError::UnsupportedLayout));
    // A character count that does not fit the data.
    assert!(tamper(&|b| b[14..16].copy_from_slice(&u16::MAX.to_le_bytes())).is_err());
    assert!(tamper(&|b| b[14..16].copy_from_slice(&0u16.to_le_bytes())).is_err());
    // An entry count of four billion must fail fast, without allocating for it.
    assert!(matches!(
        tamper(&|b| b[16..20].copy_from_slice(&u32::MAX.to_le_bytes())),
        Err(FontIdError::Truncated { .. })
    ));
    // An invalid code point and a repeated character in the table.
    assert_eq!(
        tamper(&|b| b[20..24].copy_from_slice(&0xD800u32.to_le_bytes())),
        Err(FontIdError::Corrupt("invalid character"))
    );
    assert_eq!(
        tamper(&|b| b[20..24].copy_from_slice(&u32::from('B').to_le_bytes())),
        Err(FontIdError::Corrupt("repeated character"))
    );
    // A string length that runs past the end, and a string that is not UTF-8.
    let strings = 20 + 65 * 4;
    assert!(matches!(
        tamper(&|b| b[strings..strings + 2].copy_from_slice(&u16::MAX.to_le_bytes())),
        Err(FontIdError::Truncated { .. })
    ));
    assert_eq!(
        tamper(&|b| b[strings + 2] = 0xff),
        Err(FontIdError::Corrupt("string is not valid UTF-8"))
    );
    // Trailing garbage after the last entry.
    assert_eq!(
        tamper(&|b| {
            let end = b.len() - 4;
            b.splice(end..end, [1, 2, 3]);
        }),
        Err(FontIdError::Corrupt("trailing bytes after the last entry"))
    );
    // The untouched bytes still parse.
    assert!(tamper(&|_| {}).is_ok());
}

#[test]
fn random_bytes_never_panic() {
    let mut state = 0x1234_5678_9abc_def0u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for round in 0..400 {
        let len = (next() % 600) as usize;
        let mut bytes: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        if round % 2 == 0 && len >= 24 {
            // A valid magic, version and layout in front, a valid checksum behind.
            bytes[..8].copy_from_slice(b"EBCFIDX\0");
            bytes[8..10].copy_from_slice(&1u16.to_le_bytes());
            bytes[10..14].copy_from_slice(&[4, 8, 5, 3]);
            reseal(&mut bytes);
        }
        let _ = FontIndex::from_bytes(&bytes);
    }
}
