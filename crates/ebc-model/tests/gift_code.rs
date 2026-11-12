//! Randomised whole-code tests of [`GiftCode`]: the error-detection guarantees of the
//! check symbol, exercised through the public API only.
//!
//! The quasigroup properties behind these guarantees are proved exhaustively in the unit
//! tests of `ebc_model::code`; here every position of many random codes is attacked.
//! The generator is deterministic (no clock, no OS randomness).

use ebc_model::{CodeError, GiftCode, claim_url};

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

    fn code(&mut self) -> GiftCode {
        let bytes = self.next().to_be_bytes();
        GiftCode::from_random_bytes(&bytes).expect("eight bytes are enough")
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
    for _ in 0..3_000 {
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
    assert_eq!(checked, 3_000 * 10 * 31);
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
    // Nine adjacent pairs per code, minus the ~1/32 that hold twice the same symbol.
    assert!(checked > 20_000 * 9 * 9 / 10, "only {checked} transpositions checked");
}

#[test]
fn transpositions_at_any_distance_are_detected() {
    // Stronger than required: the linear structure of the check gives every pair of
    // positions a distinct weight.
    let mut rng = Rng(0xfeed);
    for _ in 0..5_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for a in 0..GiftCode::LEN {
            for b in (a + 1)..GiftCode::LEN {
                if original[a] == original[b] {
                    continue;
                }
                let mut mutated = original.clone();
                mutated.swap(a, b);
                assert_eq!(parse_bytes(&mutated), Err(CodeError::Check), "{code} swap {a},{b}");
            }
        }
    }
}

#[test]
fn twin_and_jump_twin_errors_are_detected() {
    // aa -> bb (twin) and a?a -> b?b (jump twin), wherever they occur naturally.
    let mut rng = Rng(0x7717);
    let (mut twins, mut jump_twins) = (0usize, 0usize);
    for _ in 0..40_000 {
        let code = rng.code();
        let original = code.compact().into_bytes();
        for distance in [1usize, 2] {
            for position in 0..GiftCode::LEN - distance {
                if original[position] != original[position + distance] {
                    continue;
                }
                for replacement in ALPHABET {
                    if original[position] == *replacement {
                        continue;
                    }
                    let mut mutated = original.clone();
                    mutated[position] = *replacement;
                    mutated[position + distance] = *replacement;
                    assert_eq!(
                        parse_bytes(&mutated),
                        Err(CodeError::Check),
                        "{code} at {position}"
                    );
                    if distance == 1 {
                        twins += 1;
                    } else {
                        jump_twins += 1;
                    }
                }
            }
        }
    }
    assert!(twins > 100_000 && jump_twins > 100_000, "{twins} twins, {jump_twins} jump twins");
}

#[test]
fn exactly_one_check_symbol_fits_any_nine_data_symbols() {
    let mut rng = Rng(0xabcd);
    for _ in 0..5_000 {
        let code = rng.code();
        let compact = code.compact();
        let mut valid = Vec::new();
        for check in ALPHABET {
            let candidate = format!("{}{}", &compact[..9], char::from(*check));
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
        assert_eq!(GiftCode::parse(&dropped), Err(CodeError::Length { found: 9 }));
        let mut doubled = compact.clone();
        let symbol = compact.as_bytes()[position];
        doubled.insert(position, char::from(symbol));
        assert_eq!(GiftCode::parse(&doubled), Err(CodeError::Length { found: 11 }));
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
        assert_eq!(grouped, format!("{}-{}", &compact[..5], &compact[5..]));
        assert_eq!(code.to_string(), grouped);
        assert_eq!(GiftCode::parse(&compact), Ok(code));
        assert_eq!(GiftCode::parse(&grouped), Ok(code));
        assert_eq!(GiftCode::parse(&grouped.to_lowercase()), Ok(code));
        assert_eq!(grouped.parse::<GiftCode>(), Ok(code));
        assert_eq!(GiftCode::from_payload(code.payload()), Ok(code));
        assert!(code.payload() < 1 << GiftCode::ENTROPY_BITS);
        assert_eq!(code.symbols().len(), GiftCode::LEN);
        assert!(code.symbols().iter().all(|s| *s < 32));
        let url = claim_url("https://ebk.example", &code);
        assert_eq!(GiftCode::from_claim_url(&url), Ok(code));
        assert_eq!(GiftCode::from_claim_url(&url.to_uppercase()), Ok(code));
        let json = serde_json::to_string(&code).expect("serialises");
        assert_eq!(json.len(), GiftCode::LEN + 2);
        assert_eq!(serde_json::from_str::<GiftCode>(&json).expect("parses"), code);
    }
}

#[test]
fn distinct_random_bytes_give_distinct_codes() {
    use std::collections::HashSet;
    let mut rng = Rng(7);
    let mut seen = HashSet::new();
    let mut payloads = HashSet::new();
    for _ in 0..50_000 {
        let code = rng.code();
        if payloads.insert(code.payload()) {
            assert!(seen.insert(code.compact()), "two payloads share the code {code}");
        }
    }
    // 50 000 draws from 2^45: a repeat would mean the mapping loses entropy.
    assert_eq!(payloads.len(), 50_000);
}

#[test]
fn symbols_are_spread_evenly() {
    // A loose sanity check that neither the bit slicing nor the check symbol is skewed:
    // each of the 32 symbols should appear close to 1/32 of the time at every position.
    let mut rng = Rng(99);
    let draws = 64_000usize;
    let mut counts = [[0usize; 32]; GiftCode::LEN];
    for _ in 0..draws {
        for (position, symbol) in rng.code().symbols().iter().enumerate() {
            counts[position][usize::from(*symbol)] += 1;
        }
    }
    let expected = draws / 32;
    for (position, row) in counts.iter().enumerate() {
        for (symbol, count) in row.iter().enumerate() {
            assert!(
                count.abs_diff(expected) < expected / 4,
                "symbol {symbol} appears {count} times at position {position} (expected ≈{expected})"
            );
        }
    }
}

#[test]
fn arbitrary_input_never_panics_and_accepted_input_is_canonicalised() {
    let pieces = [
        "0",
        "1",
        "7",
        "9",
        "A",
        "b",
        "Z",
        "z",
        "I",
        "l",
        "O",
        "o",
        "U",
        "u",
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
        "k",
        "https://ebk.example/k/",
        "00000",
        "ZZZZZ",
    ];
    let mut rng = Rng(0xf00d);
    let mut accepted = 0usize;
    for _ in 0..50_000 {
        let mut text = String::new();
        for _ in 0..rng.below(16) {
            text.push_str(pieces[rng.below(pieces.len())]);
        }
        let _ = GiftCode::from_claim_url(&text);
        if let Ok(code) = GiftCode::parse(&text) {
            accepted += 1;
            assert_eq!(GiftCode::parse(&code.compact()), Ok(code));
            assert!(!text.contains(['U', 'u', 'é', '/', ':']));
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
        accepted += 1;
    }
    assert!(accepted >= 5_000);
}
