//! Round-trip tests: every symbol produced by `ebc-qr` is decoded again by an independent
//! decoder (`rqrr`, a port of quirc) and must give back the input.
//!
//! Two decoding paths are used:
//!
//! * **raster** — the matrix is rendered to a greyscale bitmap with its quiet zone, at
//!   several module sizes, and goes through `rqrr`'s finder-pattern detection, perspective
//!   estimation and sampling, like a camera frame would;
//! * **direct** — the module matrix is handed to the decoder as is, which isolates the
//!   encoder (data, error correction, masking, format/version information) from the
//!   detector and lets every version from 1 to 40 be checked.
//!
//! `rqrr` falls back to a mirrored reading when the normal one fails, so orientation is
//! checked separately in `tests/geometry.rs`.

use ebc_qr::{EcLevel, QUIET_ZONE_MODULES, QrCode, QrError, normalize_url};

/// What the decoder reports about a symbol.
#[derive(Debug, PartialEq, Eq)]
struct Decoded {
    text: String,
    version: usize,
    ec_level: EcLevel,
    mask: u8,
}

/// `rqrr` reports the raw two-bit level indicator of the format information.
fn level_from_format_bits(bits: u16) -> EcLevel {
    match bits {
        0b01 => EcLevel::L,
        0b00 => EcLevel::M,
        0b11 => EcLevel::Q,
        _ => EcLevel::H,
    }
}

fn decoded(meta: rqrr::MetaData, text: String) -> Decoded {
    Decoded {
        text,
        version: meta.version.0,
        ec_level: level_from_format_bits(meta.ecc_level),
        mask: meta.mask as u8,
    }
}

/// Renders the symbol with `quiet` modules of white around it at `module_px` pixels per
/// module and runs the full detect-and-decode pipeline on the bitmap.
fn decode_raster(qr: &QrCode, module_px: u32, quiet: u32) -> Result<Decoded, String> {
    let side = ((qr.size() + 2 * quiet) * module_px) as usize;
    let mut image = rqrr::PreparedImage::prepare_from_greyscale(side, side, |px, py| {
        // Pixels in the quiet zone wrap to huge coordinates, which `module` reports light.
        let x = (px as u32 / module_px).wrapping_sub(quiet);
        let y = (py as u32 / module_px).wrapping_sub(quiet);
        if qr.module(x, y) { 0 } else { 255 }
    });
    let grids = image.detect_grids();
    if grids.len() != 1 {
        return Err(format!("expected exactly one symbol in the bitmap, found {}", grids.len()));
    }
    let (meta, text) = grids[0].decode().map_err(|e| e.to_string())?;
    Ok(decoded(meta, text))
}

/// Decodes the module matrix directly, without rendering.
fn decode_direct(qr: &QrCode) -> Result<Decoded, String> {
    let grid =
        rqrr::SimpleGrid::from_func(qr.size() as usize, |x, y| qr.module(x as u32, y as u32));
    let (meta, text) = rqrr::Grid::new(grid).decode().map_err(|e| e.to_string())?;
    Ok(decoded(meta, text))
}

fn expected(qr: &QrCode, text: &str) -> Decoded {
    Decoded {
        text: text.to_owned(),
        version: usize::from(qr.version()),
        ec_level: qr.ec_level(),
        mask: qr.mask(),
    }
}

const NUMERIC: &[&str] = &["0", "7", "42", "9782070360024", "000000000000000000000000000001"];
const ALPHANUMERIC: &[&str] = &[
    "A",
    "HELLO WORLD",
    "HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD",
    // Every character of the alphanumeric set, in code order.
    "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:",
];
const BYTE: &[&str] = &[
    "a",
    "https://ebk.example/k/7k3m9qx2rd",
    "Hello, World! (lower-case & punctuation: #?=_~@[]{}|\\^`'\")",
    "line one\nline two\ttabbed",
];
const UTF8: &[&str] = &[
    "é",
    "Joyeux anniversaire, Zoë ! Ton livre t’attend.",
    "Ærøskøbing — Straße — ελληνικά — кириллица",
    "日本語のテキスト、한국어, 中文",
    "Un cadeau 🎁📚 pour toi",
];
const MIXED: &[&str] = &[
    "HTTPS://EBK.EXAMPLE/k/7K3M9QX2RD",
    "ISBN 9782070360024 / prix : 8,90 €",
    "abc0123456789012345678901234567890DEF GHI JKL MNO",
    "0123456789012345678901234567890123456789x",
];

fn all_payloads() -> Vec<&'static str> {
    [NUMERIC, ALPHANUMERIC, BYTE, UTF8, MIXED].concat()
}

#[test]
fn raster_round_trip_for_every_mode_level_and_module_size() {
    let mut symbols = 0;
    for text in all_payloads() {
        for level in EcLevel::ALL {
            let qr = QrCode::encode(text, level).expect("short payloads always fit");
            assert!(qr.ec_level() >= level);
            let want = expected(&qr, text);
            for module_px in [3, 4, 5, 8, 11] {
                let got = decode_raster(&qr, module_px, QUIET_ZONE_MODULES);
                assert_eq!(
                    got.as_ref(),
                    Ok(&want),
                    "{text:?} at {level:?}, {module_px} px/module, version {}",
                    qr.version()
                );
            }
            assert_eq!(decode_direct(&qr), Ok(want), "{text:?} at {level:?} (direct)");
            symbols += 1;
        }
    }
    assert_eq!(symbols, all_payloads().len() * 4);
}

#[test]
fn every_requested_level_is_reachable_and_reported() {
    // A payload that fills version 1 exactly at each level (numeric capacities 41/34/27/17)
    // cannot be boosted, so the symbol must carry exactly the requested level.
    for (level, digits) in [(EcLevel::L, 41), (EcLevel::M, 34), (EcLevel::Q, 27), (EcLevel::H, 17)]
    {
        let text = "8".repeat(digits);
        let qr = QrCode::encode(&text, level).expect("fits version 1");
        assert_eq!((qr.version(), qr.ec_level()), (1, level));
        for module_px in [3, 6] {
            let got = decode_raster(&qr, module_px, QUIET_ZONE_MODULES).expect("decodes");
            assert_eq!(got, expected(&qr, &text));
            assert_eq!(got.ec_level, level);
        }
    }
}

#[test]
fn url_encoding_round_trips_to_the_normalised_url() {
    let urls = [
        "https://ebk.example/K/7K3M9QX2RD",
        "https://ebk.example/k/7K3M9QX2RD",
        "https://ebk.example/k/7k3m9qx2rd",
        "https://Librairie-Demo.example:8443/cadeau/Zoë?code=7K3M9QX2RD&lang=fr#ouvrir",
        "http://192.168.1.20:8080/k/0000000000",
        "https://user:Secret@ebk.example/K/ABC",
    ];
    for url in urls {
        let normalised = normalize_url(url).expect("valid URL");
        for level in EcLevel::ALL {
            let qr = QrCode::encode_url(url, level).expect("fits");
            assert_eq!(qr, QrCode::encode(&normalised, level).expect("fits"));
            for module_px in [3, 5, 9] {
                let got = decode_raster(&qr, module_px, QUIET_ZONE_MODULES).expect("decodes");
                assert_eq!(got.text, normalised, "{url} at {level:?}, {module_px} px");
            }
        }
    }
}

#[test]
fn direct_round_trip_covers_every_version_at_every_level() {
    // Byte payloads of growing length (deterministic printable ASCII) sweep the versions.
    let payload = |len: usize| -> String {
        (0..len).map(|i| char::from(b'a' + ((i * 7 + i / 26) % 26) as u8)).collect()
    };
    for level in EcLevel::ALL {
        let mut seen = [false; 41];
        let mut len = 1;
        while len <= 2953 {
            let text = payload(len);
            match QrCode::encode(&text, level) {
                Ok(qr) => {
                    assert_eq!(qr.size(), 17 + 4 * u32::from(qr.version()));
                    assert_eq!(
                        decode_direct(&qr),
                        Ok(expected(&qr, &text)),
                        "{len} bytes at {level:?}"
                    );
                    seen[usize::from(qr.version())] = true;
                }
                Err(e) => {
                    assert_eq!(e, QrError::TooLong { bytes: len, min_ec: level });
                    break;
                }
            }
            // Version capacities grow by at least 5 bytes per step at level H.
            len += if len < 200 { 3 } else { 11 };
        }
        let missing: Vec<usize> = (1..=40).filter(|&v| !seen[v]).collect();
        assert!(missing.is_empty(), "versions never produced at {level:?}: {missing:?}");
    }
}

#[test]
fn raster_round_trip_of_large_symbols() {
    // Versions with version-information blocks (≥ 7) and many alignment patterns.
    let text: String = (0..1500).map(|i| char::from(b'A' + (i % 26) as u8)).collect();
    for (len, level) in
        [(120, EcLevel::M), (400, EcLevel::Q), (900, EcLevel::L), (1500, EcLevel::M)]
    {
        let qr = QrCode::encode(&text[..len], level).expect("fits");
        assert!(qr.version() >= 5, "version {}", qr.version());
        let got = decode_raster(&qr, 4, QUIET_ZONE_MODULES);
        assert_eq!(got, Ok(expected(&qr, &text[..len])), "{len} chars at {level:?}");
    }
}

#[test]
fn capacity_limits_are_the_published_ones() {
    // ISO/IEC 18004 capacities: (version, level, digits, alphanumeric characters, bytes).
    let table = [
        (1, EcLevel::L, 41, 25, 17),
        (1, EcLevel::M, 34, 20, 14),
        (1, EcLevel::Q, 27, 16, 11),
        (1, EcLevel::H, 17, 10, 7),
        (2, EcLevel::M, 63, 38, 26),
        (3, EcLevel::Q, 77, 47, 32),
        (10, EcLevel::L, 652, 395, 271),
        (27, EcLevel::H, 1501, 910, 625),
        (40, EcLevel::L, 7089, 4296, 2953),
        (40, EcLevel::H, 3057, 1852, 1273),
    ];
    for (version, level, digits, alnum, bytes) in table {
        for (unit, capacity) in [("1", digits), ("A", alnum), ("a", bytes)] {
            let at_capacity = QrCode::encode(&unit.repeat(capacity), level)
                .unwrap_or_else(|e| panic!("{capacity} x {unit:?} at {level:?}: {e}"));
            assert_eq!(
                (at_capacity.version(), at_capacity.ec_level()),
                (version, level),
                "{capacity} x {unit:?} at {level:?}"
            );
            let one_more = QrCode::encode(&unit.repeat(capacity + 1), level);
            if version == 40 {
                let bytes = capacity + 1;
                assert_eq!(one_more, Err(QrError::TooLong { bytes, min_ec: level }));
            } else {
                assert_eq!(one_more.map(|qr| qr.version()), Ok(version + 1));
            }
        }
    }
}

#[test]
fn error_correction_is_boosted_only_when_it_is_free() {
    // 10 digits fit version 1 at every level: boosted all the way to H.
    let qr = QrCode::encode("0123456789", EcLevel::L).expect("fits");
    assert_eq!((qr.version(), qr.ec_level()), (1, EcLevel::H));
    // 20 digits fit version 1 up to Q (27) but not H (17).
    let qr = QrCode::encode("01234567890123456789", EcLevel::L).expect("fits");
    assert_eq!((qr.version(), qr.ec_level()), (1, EcLevel::Q));
    // Asking for H makes the symbol grow instead.
    let qr = QrCode::encode("01234567890123456789", EcLevel::H).expect("fits");
    assert_eq!((qr.version(), qr.ec_level()), (2, EcLevel::H));
    // The requested level is always a lower bound.
    for text in all_payloads() {
        let mut previous_version = 0;
        for level in EcLevel::ALL {
            let qr = QrCode::encode(text, level).expect("fits");
            assert!(qr.ec_level() >= level, "{text:?} at {level:?}");
            assert!(qr.version() >= previous_version, "a stronger level never shrinks the symbol");
            previous_version = qr.version();
        }
    }
}

#[test]
fn url_normalisation_buys_smaller_symbols() {
    // The claim URL with an upper-case path is fully alphanumeric: version 2 at M
    // (25 modules) instead of version 3 (29 modules) for the same text in lower case.
    let upper = QrCode::encode_url("https://ebk.example/K/7K3M9QX2RD", EcLevel::M).expect("fits");
    let lower = QrCode::encode("https://ebk.example/k/7k3m9qx2rd", EcLevel::M).expect("fits");
    assert_eq!((upper.version(), upper.size()), (2, 25));
    assert_eq!((lower.version(), lower.size()), (3, 29));

    // With the lower-case "/k/" marker the URL is mixed-mode (217 bits): still version 2.
    let mixed = QrCode::encode_url("https://ebk.example/k/7K3M9QX2RD", EcLevel::M).expect("fits");
    assert_eq!(mixed.version(), 2);
    // Plain `encode` of the same URL must keep the lower-case scheme and host in byte
    // mode: 254 bits at best, which no longer fit version 2 at M (224 bits).
    let plain = QrCode::encode("https://ebk.example/k/7K3M9QX2RD", EcLevel::M).expect("fits");
    assert_eq!(plain.version(), 3);

    // Normalisation never makes a symbol larger, whatever the URL.
    let urls = [
        "https://ebk.example/",
        "https://a.very.long.host.name.example.org/with/a/Mixed/Case/path?and=query",
        "http://x.example/UPPER/ONLY/PATH/0123456789",
        "https://bücher.example/straße",
        "mailto:someone@example.org",
    ];
    for url in urls {
        for level in EcLevel::ALL {
            let normalised = QrCode::encode_url(url, level).expect("fits");
            let verbatim = QrCode::encode(url, level).expect("fits");
            assert!(normalised.version() <= verbatim.version(), "{url} at {level:?}");
        }
    }
}

#[test]
fn mixed_segments_beat_single_mode_encoding() {
    // 60 digits followed by one lower-case letter: one byte segment would need 61 bytes
    // (version 4 at L); numeric + byte needs 4+10+200 + 4+8+8 = 234 bits (version 2 at L).
    let text = format!("{}x", "1234567890".repeat(6));
    let qr = QrCode::encode(&text, EcLevel::L).expect("fits");
    assert_eq!(qr.version(), 2);
    assert_eq!(decode_direct(&qr), Ok(expected(&qr, &text)));
    assert_eq!(decode_raster(&qr, 4, QUIET_ZONE_MODULES), Ok(expected(&qr, &text)));
}

#[test]
fn quiet_zone_matters_to_the_decoder_but_two_modules_still_work() {
    let qr = QrCode::encode("HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD", EcLevel::M).expect("fits");
    for quiet in [2, 4, 10] {
        let got = decode_raster(&qr, 5, quiet).expect("decodes");
        assert_eq!(got.text, "HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD", "quiet zone of {quiet}");
    }
}

#[test]
fn hostile_input_is_rejected_without_panicking() {
    assert_eq!(QrCode::encode("", EcLevel::L), Err(QrError::Empty));
    assert_eq!(QrCode::encode_url("", EcLevel::L), Err(QrError::Empty));
    for level in EcLevel::ALL {
        // Just over the absolute capacity, and far over it.
        for len in [7090, 100_000, 5_000_000] {
            let text = "1".repeat(len);
            assert_eq!(
                QrCode::encode(&text, level),
                Err(QrError::TooLong { bytes: len, min_ec: level })
            );
        }
        // Fits numerically at L only.
        let digits = "9".repeat(7089);
        let result = QrCode::encode(&digits, level);
        assert_eq!(result.is_ok(), level == EcLevel::L, "{level:?}");
        // Bytes that cannot fit: 2954 at L is one too many.
        let bytes = "é".repeat(1477); // 2954 bytes
        assert_eq!(
            QrCode::encode(&bytes, level),
            Err(QrError::TooLong { bytes: 2954, min_ec: level })
        );
    }
    assert!(matches!(
        QrCode::encode_url("ebk.example/k/ABC", EcLevel::M),
        Err(QrError::InvalidUrl(_))
    ));
    assert!(matches!(
        QrCode::encode_url("https://ebk.example/a b", EcLevel::M),
        Err(QrError::InvalidUrl(_))
    ));
    let long_url = format!("https://ebk.example/{}", "a".repeat(3000));
    assert!(matches!(QrCode::encode_url(&long_url, EcLevel::L), Err(QrError::TooLong { .. })));
    // An absurdly long "URL" is turned away by its length before anything is parsed.
    let megabytes = "https://ebk.example/ \n".repeat(200_000);
    assert_eq!(
        QrCode::encode_url(&megabytes, EcLevel::M),
        Err(QrError::TooLong { bytes: megabytes.len(), min_ec: EcLevel::M })
    );

    // Control characters, NUL and other oddities are just bytes to `encode`.
    for text in ["\0", "\u{feff}", "\u{202e}abc", "\r\n", "\u{10ffff}"] {
        let qr = QrCode::encode(text, EcLevel::M).expect("fits");
        assert_eq!(decode_direct(&qr).map(|d| d.text), Ok(text.to_owned()), "{text:?}");
    }
    // Error messages are usable.
    let message = QrCode::encode(&"x".repeat(4000), EcLevel::H).expect_err("too long").to_string();
    assert!(message.contains("4000") && message.contains('H'), "{message}");
}

#[test]
fn encoding_is_deterministic() {
    for text in all_payloads() {
        let a = QrCode::encode(text, EcLevel::M).expect("fits");
        let b = QrCode::encode(text, EcLevel::M).expect("fits");
        assert_eq!(a, b);
        assert_eq!(a.rects(), b.rects());
        assert_eq!(a.to_svg(4, 4), b.to_svg(4, 4));
    }
    // Different payloads give different symbols.
    let a = QrCode::encode("HTTPS://EBK.EXAMPLE/K/7K3M9QX2RD", EcLevel::M).expect("fits");
    let b = QrCode::encode("HTTPS://EBK.EXAMPLE/K/7K3M9QX2RE", EcLevel::M).expect("fits");
    assert_ne!(a, b);
}
