//! Randomised whole-code tests of [`GiftCode`] and [`PairCode`], through the public API
//! only.
//!
//! The quasigroup properties behind the error-detection guarantees are proved
//! exhaustively in the unit tests of `ebc_model::code`; here every position of many
//! random codes is attacked. The generator is deterministic (no clock, no OS randomness).

use std::collections::HashSet;

use ebc_model::{CodeError, GiftCode, PairCode, claim_url, claim_url_qr};

/// SplitMix64.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn bytes16(&mut self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&self.next().to_be_bytes());
        bytes[8..].copy_from_slice(&self.next().to_be_bytes());
        bytes
    }

    fn code(&mut self) -> GiftCode {
        GiftCode::from_random_bytes(&self.bytes16()).expect("sixteen bytes are enough")
    }

    fn pair(&mut self) -> PairCode {
        PairCode::from_random_bytes(&self.bytes16()).expect("sixteen bytes are enough")
    }
}

const ALPHABET: &[u8] = GiftCode::ALPHABET.as_bytes();

fn parse_bytes(bytes: &[u8]) -> Result<GiftCode, CodeError> {
    GiftCode::parse(std::str::from_utf8(bytes).expect("ascii"))
}

#[test]
fn every_single_symbol_substitution_is_detected() {
    let mut rng = Rng(0xc0de);
    let mut checked = 0usize;
    for _ in 0..4_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for position in 0..GiftCode::LEN {
            for replacement in ALPHABET {
                if original[position] == *replacement {
                    continue;
                }
                let mut mutated = original.clone();
                mutated[position] = *replacement;
                assert_eq!(
                    parse_bytes(&mutated),
                    Err(CodeError::Check),
                    "{} -> {}",
                    code,
                    String::from_utf8_lossy(&mutated)
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 4_000 * 12 * 19);
}

#[test]
fn every_adjacent_transposition_is_detected() {
    let mut rng = Rng(0x5eed);
    let mut checked = 0usize;
    for _ in 0..20_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for position in 0..GiftCode::LEN - 1 {
            if original[position] == original[position + 1] {
                continue;
            }
            let mut mutated = original.clone();
            mutated.swap(position, position + 1);
            assert_eq!(
                parse_bytes(&mutated),
                Err(CodeError::Check),
                "{} -> {}",
                code,
                String::from_utf8_lossy(&mutated)
            );
            checked += 1;
        }
    }
    // Eleven adjacent pairs per code, minus the ~1/20 that hold twice the same letter.
    assert!(checked > 20_000 * 11 * 9 / 10, "only {checked} transpositions checked");
}

#[test]
fn every_jump_transposition_is_detected() {
    // abc -> cba: two places apart.
    let mut rng = Rng(0xfeed);
    let mut checked = 0usize;
    for _ in 0..20_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for position in 0..GiftCode::LEN - 2 {
            if original[position] == original[position + 2] {
                continue;
            }
            let mut mutated = original.clone();
            mutated.swap(position, position + 2);
            assert_eq!(parse_bytes(&mutated), Err(CodeError::Check), "{code} at {position}");
            checked += 1;
        }
    }
    assert!(checked > 20_000 * 10 * 9 / 10);
}

#[test]
fn every_adjacent_twin_error_is_detected() {
    // aa -> bb, wherever a doubled letter occurs naturally.
    let mut rng = Rng(0x7717);
    let mut checked = 0usize;
    for _ in 0..40_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for position in 0..GiftCode::LEN - 1 {
            if original[position] != original[position + 1] {
                continue;
            }
            for replacement in ALPHABET {
                if original[position] == *replacement {
                    continue;
                }
                let mut mutated = original.clone();
                mutated[position] = *replacement;
                mutated[position + 1] = *replacement;
                assert_eq!(parse_bytes(&mutated), Err(CodeError::Check), "{code} at {position}");
                checked += 1;
            }
        }
    }
    assert!(checked > 200_000, "only {checked} twin errors checked");
}

#[test]
fn distant_swaps_are_usually_but_not_always_detected() {
    // Documented limit: swaps three or more places apart are outside the guarantee.
    // Measure how many slip through, so that the documentation stays honest.
    let mut rng = Rng(0xd157);
    let (mut tried, mut undetected) = (0usize, 0usize);
    let mut undetected_distances = HashSet::new();
    for _ in 0..5_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for a in 0..GiftCode::LEN {
            for b in (a + 3)..GiftCode::LEN {
                if original[a] == original[b] {
                    continue;
                }
                let mut mutated = original.clone();
                mutated.swap(a, b);
                tried += 1;
                match parse_bytes(&mutated) {
                    Err(CodeError::Check) => {}
                    Ok(other) => {
                        assert_ne!(other, code);
                        undetected += 1;
                        undetected_distances.insert(b - a);
                    }
                    Err(error) => panic!("unexpected {error:?}"),
                }
            }
        }
    }
    assert!(undetected > 0, "the documented limit no longer exists: update the docs");
    assert!(undetected * 100 < tried * 15, "{undetected} of {tried} distant swaps undetected");
    // Only distances that are multiples of 3 (GF(4) weights repeat) or of 4 (ℤ/5 weights
    // repeat) can slip through.
    let mut distances: Vec<usize> = undetected_distances.into_iter().collect();
    distances.sort_unstable();
    assert_eq!(distances, [3, 4, 6, 8, 9]);
}

#[test]
fn exactly_one_check_symbol_fits_any_eleven_data_symbols() {
    let mut rng = Rng(0xabcd);
    for _ in 0..5_000 {
        let code = rng.code();
        let compact = code.compact();
        let mut valid = Vec::new();
        for check in ALPHABET {
            let candidate = format!("{}{}", &compact[..11], char::from(*check));
            match GiftCode::parse(&candidate) {
                Ok(parsed) => valid.push(parsed),
                Err(error) => assert_eq!(error, CodeError::Check),
            }
        }
        assert_eq!(valid, [code]);
    }
}

#[test]
fn dropped_and_doubled_symbols_are_length_errors_not_guesses() {
    let mut rng = Rng(0x1e57);
    for _ in 0..2_000 {
        let compact = rng.code().compact();
        let position = rng.below(GiftCode::LEN);
        let mut dropped = compact.clone();
        dropped.remove(position);
        assert_eq!(GiftCode::parse(&dropped), Err(CodeError::Length { expected: 12, found: 11 }));
        let mut doubled = compact.clone();
        let symbol = compact.as_bytes()[position];
        doubled.insert(position, char::from(symbol));
        assert_eq!(GiftCode::parse(&doubled), Err(CodeError::Length { expected: 12, found: 13 }));
    }
}

#[test]
fn codes_round_trip_through_every_representation() {
    let mut rng = Rng(42);
    for _ in 0..20_000 {
        let code = rng.code();
        let compact = code.compact();
        let grouped = code.grouped();
        assert_eq!(compact.len(), GiftCode::LEN);
        assert!(compact.bytes().all(|b| ALPHABET.contains(&b)));
        assert_eq!(grouped, format!("{}-{}-{}", &compact[..4], &compact[4..8], &compact[8..]));
        assert_eq!(code.to_string(), grouped);
        assert_eq!(GiftCode::parse(&compact), Ok(code));
        assert_eq!(GiftCode::parse(&grouped), Ok(code));
        assert_eq!(GiftCode::parse(&grouped.to_lowercase()), Ok(code));
        assert_eq!(grouped.parse::<GiftCode>(), Ok(code));
        assert_eq!(GiftCode::from_payload(code.payload()), Ok(code));
        assert!(code.payload() < GiftCode::SPACE);
        assert!(code.symbols().iter().all(|s| *s < 20));
        let url = claim_url("https://ebk.example", &code);
        assert_eq!(GiftCode::from_claim_url(&url), Ok(code));
        let qr = claim_url_qr("https://ebk.example", &code);
        assert_eq!(qr, url.to_uppercase());
        assert_eq!(GiftCode::from_claim_url(&qr), Ok(code));
        let json = serde_json::to_string(&code).expect("serialises");
        assert_eq!(json.len(), GiftCode::LEN + 2);
        assert_eq!(serde_json::from_str::<GiftCode>(&json).expect("parses"), code);
    }
}

#[test]
fn distinct_payloads_give_distinct_codes() {
    let mut rng = Rng(7);
    let mut seen = HashSet::new();
    let mut payloads = HashSet::new();
    for _ in 0..50_000 {
        let code = rng.code();
        if payloads.insert(code.payload()) {
            assert!(seen.insert(code.compact()), "two payloads share the code {code}");
        }
    }
    // 50 000 draws from 2^47.5: a repeat would mean the mapping loses entropy.
    assert_eq!(payloads.len(), 50_000);
}

#[test]
fn symbols_are_spread_evenly() {
    // A loose sanity check that neither the sampling nor the check symbol is skewed: each
    // of the 20 letters should appear close to 1/20 of the time at every position.
    let mut rng = Rng(99);
    let draws = 60_000usize;
    let mut counts = [[0usize; 20]; GiftCode::LEN];
    for _ in 0..draws {
        for (position, symbol) in rng.code().symbols().iter().enumerate() {
            counts[position][usize::from(*symbol)] += 1;
        }
    }
    let expected = draws / 20;
    for (position, row) in counts.iter().enumerate() {
        for (symbol, count) in row.iter().enumerate() {
            assert!(
                count.abs_diff(expected) < expected / 5,
                "symbol {symbol} appears {count} times at position {position} (expected ≈{expected})"
            );
        }
    }
}

#[test]
fn more_random_bytes_never_hurt() {
    // Eight bytes are enough; with more, the first accepted chunk decides.
    let mut rng = Rng(0xb17e5);
    for _ in 0..2_000 {
        let bytes = rng.bytes16();
        let from_16 = GiftCode::from_random_bytes(&bytes).expect("16 bytes");
        let from_8 = GiftCode::from_random_bytes(&bytes[..8]).expect("8 bytes");
        // A first chunk is rejected only once in ~90 000 draws; these all agree.
        assert_eq!(from_16, from_8);
        let mut longer = bytes.to_vec();
        longer.extend_from_slice(&[0xaa; 5]);
        assert_eq!(GiftCode::from_random_bytes(&longer), Ok(from_16));
    }
    for len in 0..8 {
        assert_eq!(
            GiftCode::from_random_bytes(&vec![0x55; len]),
            Err(CodeError::NotEnoughRandomness { needed: 8, got: len })
        );
        assert_eq!(
            PairCode::from_random_bytes(&vec![0x55; len]),
            Err(CodeError::NotEnoughRandomness { needed: 8, got: len })
        );
    }
}

#[test]
fn few_codes_contain_unfortunate_letters() {
    let mut rng = Rng(0xbad);
    let flagged = (0..50_000).filter(|_| rng.code().has_unfortunate_letters()).count();
    // Documented as about 1.4 %.
    assert!((300..1_200).contains(&flagged), "{flagged} of 50 000 flagged");
}

#[test]
fn arbitrary_input_never_panics_and_accepted_input_is_canonicalised() {
    let pieces = [
        "B",
        "c",
        "D",
        "z",
        "Z",
        "k",
        "K",
        "L",
        "A",
        "e",
        "I",
        "o",
        "U",
        "y",
        "0",
        "1",
        "9",
        "-",
        " ",
        ".",
        "_",
        "\u{2013}",
        "\u{a0}",
        "\n",
        "\t",
        "\u{feff}",
        "é",
        "漢",
        "\u{0}",
        "/",
        ":",
        "?",
        "https://ebk.example/k/",
        "HTTPS://EBK.EXAMPLE/K/",
        "BBBB",
        "ZZZZ",
    ];
    let mut rng = Rng(0xf00d);
    for _ in 0..50_000 {
        let mut text = String::new();
        for _ in 0..rng.below(18) {
            text.push_str(pieces[rng.below(pieces.len())]);
        }
        let _ = GiftCode::from_claim_url(&text);
        let _ = PairCode::parse(&text);
        if let Ok(code) = GiftCode::parse(&text) {
            assert_eq!(GiftCode::parse(&code.compact()), Ok(code));
            assert!(!text.contains(['A', 'e', 'I', 'o', 'U', 'y', '0', '1', '9', 'é', '/', ':']));
        }
    }
    // Start from valid codes and decorate them: these must all be accepted.
    for _ in 0..5_000 {
        let code = rng.code();
        let mut text = String::new();
        for c in code.compact().chars() {
            if rng.below(3) == 0 {
                text.push_str(["-", " ", ".", "_", "\u{2011}", "\u{200b}"][rng.below(6)]);
            }
            text.push(if rng.below(2) == 0 { c.to_ascii_lowercase() } else { c });
        }
        assert_eq!(GiftCode::parse(&text), Ok(code), "decorated {text:?}");
    }
}

#[test]
fn pair_codes_round_trip_and_spread_evenly() {
    let mut rng = Rng(0x9a12);
    let draws = 40_000usize;
    let mut counts = [[0usize; 20]; PairCode::LEN];
    let mut distinct = HashSet::new();
    for _ in 0..draws {
        let pair = rng.pair();
        let compact = pair.compact();
        let grouped = pair.grouped();
        assert_eq!(compact.len(), PairCode::LEN);
        assert_eq!(grouped, format!("{}-{}", &compact[..3], &compact[3..]));
        assert_eq!(pair.to_string(), grouped);
        assert_eq!(PairCode::parse(&compact), Ok(pair));
        assert_eq!(PairCode::parse(&grouped.to_lowercase()), Ok(pair));
        let json = serde_json::to_string(&pair).expect("serialises");
        assert_eq!(serde_json::from_str::<PairCode>(&json).expect("parses"), pair);
        for (position, symbol) in pair.symbols().iter().enumerate() {
            assert!(*symbol < 20);
            counts[position][usize::from(*symbol)] += 1;
        }
        distinct.insert(compact);
    }
    // 40 000 draws from 64 million: a handful of repeats at most (birthday bound ≈ 12).
    assert!(distinct.len() > draws - 100, "{} distinct pair codes", distinct.len());
    let expected = draws / 20;
    for row in &counts {
        for count in row {
            assert!(count.abs_diff(expected) < expected / 5, "{count} vs ≈{expected}");
        }
    }
    // A pair code is never mistaken for a gift code, nor the reverse.
    let pair = rng.pair();
    let code = rng.code();
    assert_eq!(GiftCode::parse(&pair.compact()), Err(CodeError::Length { expected: 12, found: 6 }));
    assert_eq!(PairCode::parse(&code.compact()), Err(CodeError::Length { expected: 6, found: 12 }));
}
