//! Adversarial ("sparring") tests for the encoder side of `ebc-qr`: is the symbol really the
//! smallest possible, does every hostile text survive a round trip, are the documented
//! error-correction rules kept?
//!
//! The reference for "smallest possible" is an independent, exhaustive O(n²) dynamic
//! programme over all segmentations written here from the bit counts of ISO/IEC 18004, fed
//! straight into `qrcodegen` — it shares no code with `ebc_qr::segment`.

use ebc_qr::{EcLevel, MAX_PAYLOAD_BYTES, QrCode, QrError};
use qrcodegen::{QrCodeEcc, QrSegment, Version};

/// xorshift64*: a tiny deterministic generator for test inputs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn pick(&mut self, set: &[char]) -> char {
        set[self.below(set.len())]
    }
}

const ALNUM: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum M {
    Num,
    Alnum,
    Byte,
}

fn accepts(mode: M, c: char) -> bool {
    match mode {
        M::Num => c.is_ascii_digit(),
        M::Alnum => ALNUM.contains(c),
        M::Byte => true,
    }
}

/// Character-count field widths for versions 1–9, 10–26, 27–40 (ISO/IEC 18004 table 3).
fn count_bits(mode: M, group: usize) -> u64 {
    match mode {
        M::Num => [10, 12, 14][group],
        M::Alnum => [9, 11, 13][group],
        M::Byte => [8, 16, 16][group],
    }
}

fn segment_bits(mode: M, chars: u64, bytes: u64, group: usize) -> u64 {
    let data = match mode {
        M::Num => 10 * (chars / 3) + [0, 4, 7][(chars % 3) as usize],
        M::Alnum => 11 * (chars / 2) + 6 * (chars % 2),
        M::Byte => 8 * bytes,
    };
    4 + count_bits(mode, group) + data
}

/// The cheapest segmentation of `text` for a version group, by trying every possible last
/// segment at every position. Returns `(bits, segments)`.
fn reference_segmentation(text: &str, group: usize) -> (u64, Vec<(M, String)>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut best = vec![u64::MAX; n + 1];
    let mut back: Vec<(usize, M)> = vec![(0, M::Byte); n + 1];
    best[0] = 0;
    for end in 1..=n {
        for mode in [M::Num, M::Alnum, M::Byte] {
            let mut bytes = 0u64;
            for start in (0..end).rev() {
                if !accepts(mode, chars[start]) {
                    break;
                }
                bytes += chars[start].len_utf8() as u64;
                let cost = best[start] + segment_bits(mode, (end - start) as u64, bytes, group);
                if cost < best[end] {
                    best[end] = cost;
                    back[end] = (start, mode);
                }
            }
        }
    }
    let mut segments = Vec::new();
    let mut end = n;
    while end > 0 {
        let (start, mode) = back[end];
        segments.push((mode, chars[start..end].iter().collect::<String>()));
        end = start;
    }
    segments.reverse();
    (best[n], segments)
}

fn backend_level(level: EcLevel) -> QrCodeEcc {
    match level {
        EcLevel::L => QrCodeEcc::Low,
        EcLevel::M => QrCodeEcc::Medium,
        EcLevel::Q => QrCodeEcc::Quartile,
        EcLevel::H => QrCodeEcc::High,
    }
}

/// The smallest version (and the boosted level) that can hold `text`, per the reference
/// segmentation; `None` if it does not fit at all.
fn reference_version(text: &str, level: EcLevel) -> Option<(u8, QrCodeEcc)> {
    for (group, (low, high)) in [(1u8, 9u8), (10, 26), (27, 40)].into_iter().enumerate() {
        let (_, segments) = reference_segmentation(text, group);
        let segments: Vec<QrSegment> = segments
            .iter()
            .map(|(mode, part)| match mode {
                M::Num => QrSegment::make_numeric(part),
                M::Alnum => QrSegment::make_alphanumeric(part),
                M::Byte => QrSegment::make_bytes(part.as_bytes()),
            })
            .collect();
        let code = qrcodegen::QrCode::encode_segments_advanced(
            &segments,
            backend_level(level),
            Version::new(low),
            Version::new(high),
            None,
            true,
        );
        if let Ok(code) = code {
            return Some((code.version().value(), code.error_correction_level()));
        }
    }
    None
}

fn decode(qr: &QrCode) -> Result<String, String> {
    let grid =
        rqrr::SimpleGrid::from_func(qr.size() as usize, |x, y| qr.module(x as u32, y as u32));
    rqrr::Grid::new(grid).decode().map(|(_, text)| text).map_err(|e| e.to_string())
}

/// A text made of runs of different character classes, the worst case for segmentation.
fn random_mixed_text(rng: &mut Rng, max_runs: usize, max_run: usize) -> String {
    let digits: Vec<char> = "0123456789".chars().collect();
    let upper: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:".chars().collect();
    let lower: Vec<char> = "abcdefghijklmnopqrstuvwxyz_?=&#@~".chars().collect();
    let wide: Vec<char> = "éßñ€—日本한🎁\u{301}\u{200d}\u{feff}\u{10ffff}\0\n".chars().collect();
    let mut text = String::new();
    for _ in 0..1 + rng.below(max_runs) {
        let set = match rng.below(8) {
            0..=2 => &digits,
            3..=4 => &upper,
            5..=6 => &lower,
            _ => &wide,
        };
        for _ in 0..1 + rng.below(max_run) {
            text.push(rng.pick(set));
        }
    }
    text
}

// --- smallest version ------------------------------------------------------------------------

#[test]
fn version_is_the_smallest_possible_for_random_mixed_texts() {
    let mut rng = Rng(0x5E6_3E47);
    let mut compared = 0;
    for _ in 0..400 {
        let text = random_mixed_text(&mut rng, 9, 14);
        for level in EcLevel::ALL {
            let ours = QrCode::encode(&text, level).expect("short text fits");
            let (version, boosted) = reference_version(&text, level).expect("reference fits");
            assert_eq!(
                (ours.version(), backend_level(ours.ec_level())),
                (version, boosted),
                "{text:?} at {level:?}"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 1_600);
}

/// Grow a text one character at a time through several version boundaries: a segmentation
/// that wastes even a few bits shows up as a version bump one character too early.
#[test]
fn version_boundaries_are_crossed_exactly_when_the_reference_crosses_them() {
    let mut rng = Rng(0xB007_DA47);
    for round in 0..12 {
        let full = random_mixed_text(&mut rng, 30, 9);
        let chars: Vec<char> = full.chars().take(160).collect();
        let level = EcLevel::ALL[round % 4];
        let mut previous = 0;
        for len in 1..=chars.len() {
            let text: String = chars[..len].iter().collect();
            let ours = QrCode::encode(&text, level).expect("fits");
            let (version, boosted) = reference_version(&text, level).expect("reference fits");
            assert_eq!(
                (ours.version(), backend_level(ours.ec_level())),
                (version, boosted),
                "prefix of {len} chars of {full:?} at {level:?}"
            );
            assert!(ours.version() >= previous, "a longer text cannot need a smaller symbol");
            previous = ours.version();
        }
    }
}

#[test]
fn larger_version_groups_are_also_optimal() {
    // Texts in the 10–26 and 27–40 ranges, where the length fields are wider.
    let mut rng = Rng(0xB16);
    for round in 0..6 {
        let mut text = String::new();
        let target = [300, 700, 1_200, 1_800, 2_400, 2_900][round];
        while text.len() < target {
            text.push_str(&random_mixed_text(&mut rng, 6, 40));
        }
        for level in [EcLevel::L, EcLevel::Q] {
            let ours = QrCode::encode(&text, level);
            let reference = reference_version(&text, level);
            match (ours, reference) {
                (Ok(ours), Some((version, boosted))) => {
                    assert_eq!(
                        (ours.version(), backend_level(ours.ec_level())),
                        (version, boosted),
                        "{} bytes at {level:?}",
                        text.len()
                    );
                    assert_eq!(decode(&ours).as_deref(), Ok(text.as_str()));
                }
                (Err(QrError::TooLong { bytes, min_ec }), None) => {
                    assert_eq!((bytes, min_ec), (text.len(), level));
                }
                (ours, reference) => {
                    panic!("{} bytes at {level:?}: {ours:?} vs {reference:?}", text.len())
                }
            }
        }
    }
}

// --- round trips of hostile text -----------------------------------------------------------------

#[test]
fn random_mixed_and_unicode_texts_decode_to_themselves() {
    let mut rng = Rng(0xDEC0DE);
    for round in 0..300 {
        let text = random_mixed_text(&mut rng, 7, 10);
        let level = EcLevel::ALL[round % 4];
        let qr = QrCode::encode(&text, level).expect("fits");
        assert!(qr.ec_level() >= level);
        assert_eq!(qr.size(), 17 + 4 * u32::from(qr.version()));
        assert_eq!(decode(&qr).as_deref(), Ok(text.as_str()), "{text:?} at {level:?}");
    }
}

#[test]
fn segment_boundaries_inside_grapheme_clusters_and_at_odd_lengths() {
    let cases = [
        "1\u{301}2\u{301}3",    // combining marks between digits
        "A\u{200d}B\u{200d}C",  // zero-width joiners between capitals
        "👨‍👩‍👧‍👦0123456789012345👨‍👩‍👧‍👦", // ZWJ emoji sequence around digits
        "e\u{301}0e\u{301}00e\u{301}000e\u{301}0000",
        "0",
        "00",
        "000",
        "0000",
        "A0",
        "0A",
        "A00000000A",
        "a0000000a",
        "a00000000a",
        "aAAAAAAa",
        "aAAAAAAAAAAAAa",
        ":::::::::::::::::::::",
        "                    ",
        "%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%",
        "\u{10ffff}\u{10ffff}\u{10ffff}",
        "\u{0}\u{0}\u{0}0000000000000000\u{0}",
        "٣٤٥٦٧٨٩٠١٢٣٤٥٦٧٨٩٠",   // Arabic-Indic digits are not QR digits
        "１２３４５６７８９０", // nor are full-width ones
        "ＡＢＣ",
        "ǅǈǋ", // title-case letters
        "ß",   // upper-cases to two letters in Unicode
        "ﬁ",   // ligature
        "İi̇I", // dotted/dotless i
    ];
    for text in cases {
        for level in EcLevel::ALL {
            let qr = QrCode::encode(text, level).expect("fits");
            assert_eq!(decode(&qr).as_deref(), Ok(text), "{text:?} at {level:?}");
            let (version, boosted) = reference_version(text, level).expect("fits");
            assert_eq!(
                (qr.version(), backend_level(qr.ec_level())),
                (version, boosted),
                "{text:?}"
            );
        }
    }
}

// --- error-correction rules --------------------------------------------------------------------

#[test]
fn boosting_never_costs_a_version_and_never_stops_short() {
    let mut rng = Rng(0xEC);
    for _ in 0..200 {
        let text = random_mixed_text(&mut rng, 6, 12);
        let base = QrCode::encode(&text, EcLevel::L).expect("fits");
        let mut previous_version = 0;
        for level in EcLevel::ALL {
            let qr = QrCode::encode(&text, level).expect("fits");
            assert!(qr.ec_level() >= level, "{text:?}: asked {level:?}, got {:?}", qr.ec_level());
            assert!(qr.version() >= previous_version, "a stronger level cannot shrink the symbol");
            previous_version = qr.version();
            // The reported level is reachable on its own in the same version…
            let again = QrCode::encode(&text, qr.ec_level()).expect("fits");
            assert_eq!(again.version(), qr.version());
            assert_eq!(again, qr, "asking for the boosted level gives the same symbol");
            // …and the next stronger one is not (otherwise the boost stopped short).
            if let Some(&stronger) = EcLevel::ALL.iter().find(|&&l| l > qr.ec_level()) {
                let bigger = QrCode::encode(&text, stronger).expect("fits");
                assert!(
                    bigger.version() > qr.version(),
                    "{text:?}: boost stopped at {:?}",
                    qr.ec_level()
                );
            }
        }
        assert_eq!(base.version(), QrCode::encode(&text, base.ec_level()).expect("fits").version());
    }
}

// --- limits --------------------------------------------------------------------------------------

#[test]
fn length_limits_are_exact_at_every_level() {
    // ISO/IEC 18004 table 7, version 40: digits / alphanumeric characters / bytes.
    let capacity = [
        (EcLevel::L, 7089, 4296, 2953),
        (EcLevel::M, 5596, 3391, 2331),
        (EcLevel::Q, 3993, 2420, 1663),
        (EcLevel::H, 3057, 1852, 1273),
    ];
    for (level, digits, alnum, bytes) in capacity {
        for (unit, limit) in [("7", digits), ("K", alnum), ("k", bytes)] {
            let fits = QrCode::encode(&unit.repeat(limit), level).expect("exactly at capacity");
            assert_eq!(fits.version(), 40, "{unit} x {limit} at {level:?}");
            assert_eq!(fits.ec_level(), level, "nothing to boost at capacity");
            assert_eq!(
                QrCode::encode(&unit.repeat(limit + 1), level),
                Err(QrError::TooLong { bytes: limit + 1, min_ec: level }),
                "{unit} x {} at {level:?}",
                limit + 1
            );
        }
    }
    // Multi-byte characters: the limit is in bytes, and the error reports bytes.
    let euro = "€".repeat(985); // 2955 bytes
    assert_eq!(
        QrCode::encode(&euro, EcLevel::L),
        Err(QrError::TooLong { bytes: 2955, min_ec: EcLevel::L })
    );
    assert!(QrCode::encode(&"€".repeat(984), EcLevel::L).is_ok());
    // Mixed content right at the up-front cut-off.
    let mut edge = "9".repeat(MAX_PAYLOAD_BYTES - 1);
    edge.push('a');
    assert_eq!(edge.len(), MAX_PAYLOAD_BYTES);
    assert!(matches!(QrCode::encode(&edge, EcLevel::L), Err(QrError::TooLong { .. })));
}

#[test]
fn encoding_the_largest_symbols_stays_fast() {
    use std::time::{Duration, Instant};
    let start = Instant::now();
    // Worst cases for the segmentation: alternating classes, so every boundary is a choice.
    let alternating: String = (0..2_900).map(|i| if i % 2 == 0 { '7' } else { 'a' }).collect();
    let _ = QrCode::encode(&alternating, EcLevel::L);
    let blocks: String = (0..4_000).map(|i| if (i / 7) % 2 == 0 { '7' } else { 'K' }).collect();
    let qr = QrCode::encode(&blocks, EcLevel::L).expect("fits");
    assert_eq!(decode(&qr).as_deref(), Ok(blocks.as_str()));
    let rects = qr.rects();
    assert!(rects.len() < qr.dark_count());
    let _ = qr.to_svg(4, 4);
    // Rejections must be cheap too.
    for _ in 0..50 {
        let _ = QrCode::encode(&"€".repeat(2_363), EcLevel::H);
    }
    assert!(start.elapsed() < Duration::from_secs(60), "took {:?}", start.elapsed());
}
