//! Adversarial ("sparring") tests for URL normalisation and the geometry helpers of
//! `ebc-qr`, through the public API only.

use ebc_qr::{EcLevel, Placement, QUIET_ZONE_MODULES, QrCode, QrError, normalize_url};
use ebc_scene::{PathSeg, Point, Rect};

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

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn decode(qr: &QrCode) -> Result<String, String> {
    let grid =
        rqrr::SimpleGrid::from_func(qr.size() as usize, |x, y| qr.module(x as u32, y as u32));
    rqrr::Grid::new(grid).decode().map(|(_, text)| text).map_err(|e| e.to_string())
}

fn claim_code() -> QrCode {
    QrCode::encode_url("https://ebk.example/k/7K3M9QX2RD", EcLevel::M).expect("fits")
}

// --- normalize_url: fuzzing -----------------------------------------------------------------

/// What RFC 3986 §3 says the normalised form must be, written independently: only the
/// scheme and a non-bracketed host[:port] change, and only in ASCII case.
///
/// Corrected after the review: this model originally upper-cased the scheme and host of
/// *every* scheme, which is the behaviour audit finding 1
/// (`docs/status/review-findings/color-qr.audit.json`) reports as a defect — WHATWG
/// parsers keep the host of a non-special scheme verbatim (`new URL("ebk+app://Open/Item")
/// .host == "Open"`), and deep-link handlers (`opds://authorize/`, Android intent filters)
/// compare scheme and host literally. Only `http` and `https` may be rewritten; any other
/// accepted URL must come back byte for byte, which is the stricter expectation.
fn rfc3986_expected(url: &str) -> Option<String> {
    let colon = url.find(':')?;
    let (scheme, rest) = (&url[..colon], &url[colon + 1..]);
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Some(url.to_owned());
    }
    let mut out = scheme.to_ascii_uppercase();
    out.push(':');
    let Some(after) = rest.strip_prefix("//") else {
        out.push_str(rest);
        return Some(out);
    };
    let end = after.find(['/', '?', '#']).unwrap_or(after.len());
    let authority = &after[..end];
    let split = authority.rfind('@').map_or(0, |at| at + 1);
    let (userinfo, host) = authority.split_at(split);
    out.push_str("//");
    out.push_str(userinfo);
    if host.starts_with('[') {
        out.push_str(host);
    } else {
        out.push_str(&host.to_ascii_uppercase());
    }
    out.push_str(&after[end..]);
    Some(out)
}

#[test]
fn normalisation_of_random_url_like_strings_only_touches_scheme_and_host() {
    #[rustfmt::skip]
    let pieces = [
        "http", "https", "HTTPS", "hTTp", "ftp", "mailto", "urn", "a", "Z", "z9", "+", "-", ".",
        ":", ":", "/", "/", "//", "//", "@", "@", "[", "]", "?", "#", "%", "%41", "%c3%a9",
        "ebk.example", "Shop.Example", "xn--bcher-kva", "bücher", "ÉCOLE", "straße", "::1",
        "8443", "k", "K", "7K3M9QX2RD", "aBc", "=", "&", "~", "_", "!", "$", "'", "(", ")", "*",
        ",", ";", "\u{200b}", "\u{feff}", "🎁", "İ", "ı", "ǅ", "ﬁ", "\u{301}", "|", "^", "`",
        "{", "}", "<", ">", "\"",
    ];
    let schemes = ["http", "https", "HTTPS", "hTTp", "ftp", "mailto", "urn", "a", "z9+x-y.z"];
    let separators = ["://", "://", "://", "://", ":", ":/", ":///", "://@", "://["];
    let mut rng = Rng(0x0BAD_CAFE);
    let (mut accepted, mut decoded) = (0, 0);
    for round in 0..40_000 {
        // Three quarters start like a URL; the rest are pure piece soup.
        let mut url = String::new();
        if round % 4 != 0 {
            url.push_str(schemes[rng.below(schemes.len())]);
            url.push_str(separators[rng.below(separators.len())]);
        }
        for _ in 0..rng.below(12) {
            url.push_str(pieces[rng.below(pieces.len())]);
        }
        match normalize_url(&url) {
            Ok(out) => {
                accepted += 1;
                assert_eq!(out.len(), url.len(), "{url:?} -> {out:?}");
                assert!(out.eq_ignore_ascii_case(&url), "{url:?} -> {out:?}");
                assert_eq!(normalize_url(&out).as_deref(), Ok(out.as_str()), "not idempotent");
                assert_eq!(Some(&out), rfc3986_expected(&url).as_ref(), "{url:?}");
                if accepted % 20 == 0 {
                    let qr = QrCode::encode_url(&url, EcLevel::M).expect("short URL fits");
                    assert_eq!(decode(&qr).as_deref(), Ok(out.as_str()), "{url:?}");
                    assert!(
                        qr.version() <= QrCode::encode(&url, EcLevel::M).expect("fits").version()
                    );
                    decoded += 1;
                }
            }
            Err(QrError::InvalidUrl(reason)) => assert!(!reason.is_empty()),
            Err(QrError::Empty) => assert!(url.is_empty()),
            Err(other) => panic!("{url:?}: unexpected error {other:?}"),
        }
    }
    assert!(accepted > 15_000 && decoded > 750, "{accepted} accepted, {decoded} decoded");
}

#[test]
fn every_kind_of_whitespace_and_control_character_is_rejected_anywhere() {
    let mut nasty: Vec<char> = (0u32..0x20).filter_map(char::from_u32).collect();
    nasty.extend((0x7Fu32..=0x9F).filter_map(char::from_u32));
    nasty.extend([' ', '\u{a0}', '\u{1680}', '\u{2000}', '\u{2009}', '\u{2028}', '\u{2029}']);
    nasty.extend(['\u{202f}', '\u{205f}', '\u{3000}', '\u{85}']);
    for c in nasty {
        for url in [
            format!("{c}https://ebk.example/k/ABC"),
            format!("ht{c}tps://ebk.example/k/ABC"),
            format!("https://ebk{c}.example/k/ABC"),
            format!("https://ebk.example/k/{c}ABC"),
            format!("https://ebk.example/k/ABC{c}"),
            format!("mailto:a{c}b@example.org"),
        ] {
            assert!(
                matches!(normalize_url(&url), Err(QrError::InvalidUrl(_))),
                "U+{:04X} accepted in {url:?}",
                c as u32
            );
            assert!(matches!(QrCode::encode_url(&url, EcLevel::L), Err(QrError::InvalidUrl(_))));
        }
    }
}

// --- normalize_url: backslashes -----------------------------------------------------------------

/// Browsers (the WHATWG URL parser, which is what a phone uses once the code is scanned)
/// treat `\` exactly like `/` in http(s) URLs: `https://ebk.example\k\aBc` is
/// `https://ebk.example/k/aBc`. RFC 3986 does not allow `\` in a URL at all. Either way the
/// text after the first backslash is **not** part of the host, so it must not be
/// upper-cased: doing so changes a case-sensitive path and the code opens a different
/// page than the URL it was made from.
fn assert_backslash_tail_untouched(url: &str) {
    let cut = url.find('\\').expect("test URL contains a backslash");
    match normalize_url(url) {
        Err(QrError::InvalidUrl(_)) => {} // rejecting is the simplest correct answer
        Ok(out) => assert_eq!(
            &out[cut..],
            &url[cut..],
            "{url:?} became {out:?}: everything from the backslash on is path (or was never \
             a valid URL) and must be copied byte for byte"
        ),
        Err(other) => panic!("{url:?}: unexpected error {other:?}"),
    }
}

#[test]
fn backslash_path_separators_do_not_get_upper_cased_with_the_host() {
    assert_backslash_tail_untouched("https://ebk.example\\k\\7k3m9qx2rd");
    assert_backslash_tail_untouched("http://ebk.example\\Gift\\aBcDeF?x=y");
}

#[test]
fn backslash_before_an_at_sign_does_not_move_the_host() {
    // WHATWG: host = shop.example, path = /@files.example/Abc. The implementation takes
    // "shop.example\" for user information and upper-cases "files.example", which browsers
    // see as part of the (case-sensitive) path.
    assert_backslash_tail_untouched("https://shop.example\\@files.example/Abc");
    assert_backslash_tail_untouched("https://shop.example\\x@files.example");
}

// --- to_path -----------------------------------------------------------------------------------

fn points(path: &ebc_scene::Path) -> Vec<Point> {
    path.segs
        .iter()
        .filter_map(|seg| match *seg {
            PathSeg::MoveTo(p) | PathSeg::LineTo(p) => Some(p),
            PathSeg::Close => None,
            ref other => panic!("unexpected segment {other:?}"),
        })
        .collect()
}

/// "…yields an empty path rather than a panic or garbage geometry": finite arguments whose
/// product overflows must not produce infinite or NaN coordinates either — a PDF content
/// stream cannot carry them.
#[test]
fn path_coordinates_are_always_finite() {
    let qr = claim_code();
    let hostile = [
        (Point::new(0.0, 0.0), f64::MAX),
        (Point::new(0.0, 0.0), 1e307),
        (Point::new(1e308, 0.0), 1e307),
        (Point::new(0.0, -1e308), 1e307),
        (Point::new(f64::MAX, f64::MAX), 1e292),
        (Point::new(f64::MIN, f64::MIN), f64::MAX),
    ];
    for (origin, module) in hostile {
        let path = qr.to_path(origin, module);
        for p in points(&path) {
            assert!(
                p.x.is_finite() && p.y.is_finite(),
                "to_path({origin:?}, {module:e}) produced the non-finite point {p:?}"
            );
        }
    }
}

#[test]
fn path_of_the_largest_symbol_is_exact() {
    let text: String = (0..2_953).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
    let qr = QrCode::encode(&text, EcLevel::L).expect("exactly at capacity");
    assert_eq!((qr.version(), qr.size()), (40, 177));
    let rects = qr.rects();
    // Exact cover, no overlap.
    let mut painted = vec![0u8; 177 * 177];
    for &(x, y, w, h) in &rects {
        assert!(w >= 1 && h >= 1 && x + w <= 177 && y + h <= 177);
        for yy in y..y + h {
            for xx in x..x + w {
                painted[(yy * 177 + xx) as usize] += 1;
            }
        }
    }
    for y in 0..177 {
        for x in 0..177 {
            assert_eq!(painted[(y * 177 + x) as usize], u8::from(qr.module(x, y)), "({x}, {y})");
        }
    }
    assert_eq!(painted.iter().map(|&v| usize::from(v)).sum::<usize>(), qr.dark_count());

    // An awkward module size and origin: every corner is still origin + n x module, and
    // rectangles that touch share bit-identical coordinates.
    let (origin, module) = (Point::new(-3.3, 7.1), 0.1 + 0.2);
    let path = qr.to_path(origin, module);
    assert_eq!(path.segs.len(), rects.len() * 5);
    for (chunk, &(x, y, w, h)) in path.segs.chunks(5).zip(&rects) {
        let at = |cx: u32, cy: u32| {
            Point::new(origin.x + f64::from(cx) * module, origin.y + f64::from(cy) * module)
        };
        assert_eq!(chunk[0], PathSeg::MoveTo(at(x, y)));
        assert_eq!(chunk[1], PathSeg::LineTo(at(x + w, y)));
        assert_eq!(chunk[2], PathSeg::LineTo(at(x + w, y + h)));
        assert_eq!(chunk[3], PathSeg::LineTo(at(x, y + h)));
        assert_eq!(chunk[4], PathSeg::Close);
    }
    let bounds = path.control_bounds().expect("non-empty");
    assert!((bounds.w - 177.0 * module).abs() < 1e-9 && (bounds.h - 177.0 * module).abs() < 1e-9);
}

// --- fit_in / size helpers ---------------------------------------------------------------------

#[test]
fn fit_in_keeps_symbol_and_quiet_zone_inside_random_boxes() {
    let mut rng = Rng(0xF17);
    let symbols = [
        claim_code(),
        QrCode::encode("1", EcLevel::L).expect("fits"),
        QrCode::encode(&"x".repeat(900), EcLevel::H).expect("fits"),
    ];
    for _ in 0..3_000 {
        let qr = &symbols[rng.below(symbols.len())];
        let outer = Rect::new(
            rng.unit() * 400.0 - 200.0,
            rng.unit() * 400.0 - 200.0,
            0.5 + rng.unit() * 300.0,
            0.5 + rng.unit() * 300.0,
        );
        let quiet = rng.below(12) as u32;
        let Placement { origin, module_mm } = qr.fit_in(outer, quiet);
        assert!(module_mm > 0.0 && module_mm.is_finite());
        let side = qr.side_mm(module_mm, quiet);
        assert!((side - outer.w.min(outer.h)).abs() < 1e-9, "fills the shorter side");
        let bounds = qr.to_path(origin, module_mm).control_bounds().expect("non-empty");
        let with_quiet = bounds.inflate(f64::from(quiet) * module_mm);
        assert!(outer.contains_rect(&with_quiet, 1e-9), "{with_quiet:?} outside {outer:?}");
        assert!((with_quiet.center().x - outer.center().x).abs() < 1e-9);
        assert!((with_quiet.center().y - outer.center().y).abs() < 1e-9);
    }
}

#[test]
fn fit_in_and_size_helpers_never_panic_and_never_draw_outside_a_degenerate_box() {
    let qr = claim_code();
    let specials = [
        0.0,
        -0.0,
        -1.0,
        1e-300,
        5e-324,
        1e300,
        f64::MAX,
        f64::MIN,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        30.0,
    ];
    for &x in &specials {
        for &w in &specials {
            for &h in &specials {
                for quiet in [0, 4, u32::MAX] {
                    let outer = Rect::new(x, 10.0, w, h);
                    let placement = qr.fit_in(outer, quiet);
                    let path = qr.to_path(placement.origin, placement.module_mm);
                    // A box without a positive finite extent cannot hold a symbol.
                    let usable = w > 0.0 && h > 0.0 && w.is_finite() && h.is_finite();
                    if !usable || !x.is_finite() {
                        assert!(
                            path.is_empty(),
                            "fit_in({outer:?}, {quiet}) = {placement:?} still draws something"
                        );
                    }
                    let _ = qr.side_mm(w, quiet);
                    let _ = qr.module_mm_for_side(h, quiet);
                }
            }
        }
    }
    assert!(qr.side_mm(f64::NAN, 4).is_nan());
    assert_eq!(qr.side_mm(0.0, u32::MAX), 0.0);
    assert_eq!(qr.module_mm_for_side(0.0, 0), 0.0);
}

// --- to_svg ------------------------------------------------------------------------------------

#[test]
fn svg_is_well_formed_for_extreme_parameters() {
    let qr = claim_code();
    for (module_px, quiet) in [
        (0, 0),
        (1, 0),
        (0, u32::MAX),
        (u32::MAX, 0),
        (u32::MAX, u32::MAX),
        (7, QUIET_ZONE_MODULES),
    ] {
        let svg = qr.to_svg(module_px, quiet);
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.ends_with("</svg>"));
        assert_eq!(svg.matches('<').count(), svg.matches('>').count());
        assert_eq!(svg.matches('"').count() % 2, 0);
        assert!(!svg.contains("NaN") && !svg.contains("inf") && !svg.contains("--"));
        let side = u128::from(qr.size()) + 2 * u128::from(quiet);
        let side_px = side * u128::from(module_px.max(1));
        assert!(svg.contains(&format!("width=\"{side_px}\" height=\"{side_px}\"")), "{svg}");
        assert!(svg.contains(&format!("viewBox=\"0 0 {side} {side}\"")));
        // Every "M x,y" start point lies inside the symbol area.
        let d = svg.split(" d=\"").nth(1).and_then(|s| s.split('"').next()).expect("path data");
        let mut subpaths = 0;
        for sub in d.split('M').skip(1) {
            let (xy, _) = sub.split_once('h').expect("h after M");
            let (x, y) = xy.split_once(',').expect("x,y");
            let (x, y): (u128, u128) = (x.parse().expect("x"), y.parse().expect("y"));
            let q = u128::from(quiet);
            assert!(
                x >= q && x < q + u128::from(qr.size()) && y >= q && y < q + u128::from(qr.size())
            );
            subpaths += 1;
        }
        assert_eq!(subpaths, qr.rects().len());
    }
}
