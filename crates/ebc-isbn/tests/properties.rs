//! Property-style tests over the public API, driven by a small deterministic generator
//! (no external crates, no clock, no network).

use ebc_isbn::{Isbn10, Isbn13, IsbnError, ean13_check_digit, extract_all, isbn10_check_char};

/// SplitMix64: tiny, deterministic, good enough to spread test inputs.
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

    /// A random valid ISBN-13 as thirteen ASCII digits. One in four uses prefix 979.
    fn isbn13(&mut self) -> String {
        let mut first12 = [0u8; 12];
        first12[..3].copy_from_slice(if self.below(4) == 0 { b"979" } else { b"978" });
        for slot in first12.iter_mut().skip(3) {
            *slot = b'0' + self.below(10) as u8;
        }
        format!("{}{}", String::from_utf8_lossy(&first12), ean13_check_digit(&first12))
    }

    /// A random valid ISBN-10 as ten ASCII characters.
    fn isbn10(&mut self) -> String {
        let mut first9 = [0u8; 9];
        for slot in &mut first9 {
            *slot = b'0' + self.below(10) as u8;
        }
        format!("{}{}", String::from_utf8_lossy(&first9), isbn10_check_char(&first9))
    }
}

const CASES: usize = 20_000;

#[test]
fn isbn13_round_trips_through_every_textual_form() {
    let mut rng = Rng(1);
    let mut hyphenated_count = 0usize;
    for _ in 0..CASES {
        let plain = rng.isbn13();
        let isbn = Isbn13::parse(&plain).expect("generated ISBNs are valid");
        assert_eq!(isbn.to_plain(), plain);
        assert_eq!(isbn.to_string(), plain);
        assert_eq!(plain.parse::<Isbn13>(), Ok(isbn));

        let hyphenated = isbn.hyphenated();
        assert_eq!(hyphenated.replace('-', ""), plain, "hyphenation must only insert hyphens");
        assert_eq!(Isbn13::parse(&hyphenated), Ok(isbn));
        assert_eq!(Isbn13::parse(&format!("ISBN {hyphenated}")), Ok(isbn));
        assert_eq!(extract_all(&format!("see {hyphenated}, p. 12")), [isbn]);

        match isbn.parts() {
            Some(parts) => {
                hyphenated_count += 1;
                assert_eq!(hyphenated.matches('-').count(), 4);
                let group = isbn.registration_group().expect("parts imply a known group");
                assert_eq!(format!("{}-{}", parts.gs1_prefix, parts.group), group.prefix);
                assert_eq!(parts.gs1_prefix, isbn.gs1_prefix());
                assert!(!parts.registrant.is_empty() && !parts.publication.is_empty());
                assert_eq!(parts.group.len() + parts.registrant.len() + parts.publication.len(), 9);
                assert_eq!(parts.check_digit, char::from(b'0' + isbn.check_digit()));
            }
            None => assert_eq!(hyphenated, plain),
        }
    }
    // Most of the 978 space is assigned, so the table must hyphenate a clear majority.
    assert!(hyphenated_count > CASES / 2, "only {hyphenated_count} of {CASES} hyphenated");
}

#[test]
fn isbn10_and_isbn13_convert_both_ways() {
    let mut rng = Rng(2);
    for _ in 0..CASES {
        let plain10 = rng.isbn10();
        let ten = Isbn10::parse(&plain10).expect("generated ISBNs are valid");
        assert_eq!(ten.to_string(), plain10);
        let thirteen = ten.to_isbn13();
        assert_eq!(thirteen.gs1_prefix(), 978);
        assert_eq!(thirteen.to_isbn10(), Ok(ten));
        assert_eq!(Isbn13::parse(&plain10), Ok(thirteen));
        assert_eq!(Isbn10::parse(&thirteen.to_plain()), Ok(ten));
        assert_eq!(Isbn10::parse(&ten.hyphenated()), Ok(ten));
        assert_eq!(Isbn10::parse(&plain10.to_lowercase()), Ok(ten));
        assert_eq!(ten.hyphenated().replace('-', ""), plain10);
        assert_eq!(&thirteen.to_plain()[3..12], &plain10[..9]);
    }
    for _ in 0..CASES {
        let isbn = Isbn13::parse(&rng.isbn13()).expect("valid");
        match isbn.to_isbn10() {
            Ok(ten) => {
                assert_eq!(isbn.gs1_prefix(), 978);
                assert_eq!(ten.to_isbn13(), isbn);
            }
            Err(error) => {
                assert_eq!(isbn.gs1_prefix(), 979);
                assert_eq!(error, IsbnError::NoIsbn10);
            }
        }
    }
}

#[test]
fn every_single_digit_error_in_an_isbn13_is_rejected() {
    let mut rng = Rng(3);
    for _ in 0..500 {
        let plain = rng.isbn13();
        for position in 0..13 {
            for digit in b'0'..=b'9' {
                let mut bytes = plain.clone().into_bytes();
                if bytes[position] == digit {
                    continue;
                }
                bytes[position] = digit;
                let mutated = String::from_utf8(bytes).expect("ascii");
                match Isbn13::parse(&mutated) {
                    Err(IsbnError::CheckDigit { .. } | IsbnError::Prefix) => {}
                    other => panic!("{plain} -> {mutated} gave {other:?}"),
                }
            }
        }
    }
}

#[test]
fn every_single_error_and_transposition_in_an_isbn10_is_rejected() {
    let mut rng = Rng(4);
    for _ in 0..500 {
        let plain = rng.isbn10();
        for position in 0..10 {
            for symbol in b"0123456789X" {
                let mut bytes = plain.clone().into_bytes();
                if bytes[position] == *symbol {
                    continue;
                }
                bytes[position] = *symbol;
                let mutated = String::from_utf8(bytes).expect("ascii");
                assert!(Isbn10::parse(&mutated).is_err(), "{plain} -> {mutated}");
            }
        }
        for a in 0..10 {
            for b in (a + 1)..10 {
                let mut bytes = plain.clone().into_bytes();
                if bytes[a] == bytes[b] {
                    continue;
                }
                bytes.swap(a, b);
                let mutated = String::from_utf8(bytes).expect("ascii");
                assert!(Isbn10::parse(&mutated).is_err(), "{plain} -> {mutated}");
            }
        }
    }
}

#[test]
fn extract_all_finds_isbns_scattered_in_noise() {
    let noise = [
        "",
        " ",
        "\n",
        " — ",
        "; ",
        " (poche) ",
        "<td>",
        "</td>",
        "\"isbn\":\"",
        "\",",
        "ISBN ",
        "ISBN-13: ",
        "EAN\u{a0}: ",
        "prix : ",
        " € ",
        "/livre/",
        ".html ",
        "é",
        "書",
        " p. ",
    ];
    let mut rng = Rng(5);
    for _ in 0..2_000 {
        let mut text = String::new();
        let mut expected: Vec<Isbn13> = Vec::new();
        for _ in 0..rng.below(6) {
            text.push_str(noise[rng.below(noise.len())]);
            text.push_str(noise[rng.below(noise.len())]);
            let isbn = Isbn13::parse(&rng.isbn13()).expect("valid");
            // Half hyphenated (when the range is known), half plain.
            text.push_str(&if rng.below(2) == 0 { isbn.hyphenated() } else { isbn.to_plain() });
            if !expected.contains(&isbn) {
                expected.push(isbn);
            }
            // A separator that never glues two numbers together.
            text.push_str([" ; ", "\n", "</p>", " | "][rng.below(4)]);
        }
        assert_eq!(extract_all(&text), expected, "text {text:?}");
    }
}

#[test]
fn arbitrary_text_never_panics_and_yields_only_valid_isbns() {
    let alphabet = [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "9",
        "7",
        "8",
        "978",
        "979",
        "X",
        "x",
        "-",
        "-",
        " ",
        " ",
        "\u{a0}",
        "\u{2011}",
        "\u{2013}",
        "\u{202f}",
        "\n",
        "\t",
        "I",
        "S",
        "B",
        "N",
        "ISBN",
        "isbn-13",
        "ISBN-10",
        ":",
        ".",
        "/",
        "é",
        "漢",
        "\u{0}",
        "\u{feff}",
        "urn:isbn:",
        "EAN",
        "(",
        ")",
        "#",
        "=",
    ];
    let mut rng = Rng(6);
    let mut found_some = 0usize;
    for _ in 0..20_000 {
        let mut text = String::new();
        for _ in 0..rng.below(40) {
            text.push_str(alphabet[rng.below(alphabet.len())]);
        }
        // Parsing arbitrary text must return, not panic.
        let parsed13 = Isbn13::parse(&text);
        let parsed10 = Isbn10::parse(&text);
        if let (Ok(thirteen), Ok(ten)) = (parsed13, parsed10) {
            assert_eq!(ten.to_isbn13(), thirteen);
        }
        let found = extract_all(&text);
        found_some += found.len();
        let mut unique = found.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), found.len(), "duplicates in {text:?}");
        for isbn in found {
            assert_eq!(Isbn13::parse(&isbn.to_plain()), Ok(isbn));
            // Every hit is backed by its digits appearing in order in the text.
            let digits: String = text.chars().filter(char::is_ascii_digit).collect();
            let body = &isbn.to_plain()[3..12];
            assert!(digits.contains(body), "{isbn} not in {text:?}");
        }
    }
    // The generator is biased towards ISBN-like fragments; it must hit real ones sometimes,
    // otherwise this test exercises nothing.
    assert!(found_some > 0);
}

#[test]
fn check_digit_function_agrees_with_parsing() {
    let mut rng = Rng(7);
    for _ in 0..CASES {
        let plain = rng.isbn13();
        let isbn = Isbn13::parse(&plain).expect("valid");
        let digits = isbn.digits();
        let mut first12 = [0u8; 12];
        first12.copy_from_slice(&digits[..12]);
        assert_eq!(ean13_check_digit(&first12), digits[12]);
        assert_eq!(ean13_check_digit(&first12), isbn.check_digit());
        // ASCII input gives the same answer as digit values.
        let mut ascii = [0u8; 12];
        ascii.copy_from_slice(&plain.as_bytes()[..12]);
        assert_eq!(ean13_check_digit(&ascii), digits[12]);
    }
}

#[test]
fn ordering_and_hashing_follow_the_digits() {
    use std::collections::HashSet;
    let a = Isbn13::parse("9782070360024").expect("valid");
    let b = Isbn13::parse("2-07-036002-4").expect("valid");
    let c = Isbn13::parse("9791090636071").expect("valid");
    assert!(a < c);
    let set: HashSet<Isbn13> = [a, b, c].into_iter().collect();
    assert_eq!(set.len(), 2);
    let tens: HashSet<Isbn10> = ["2070360024", "2-07-036002-4", "080442957x"]
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    assert_eq!(tens.len(), 2);
}
