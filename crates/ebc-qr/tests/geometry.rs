//! Tests of the module matrix and of everything derived from it for print: rectangle
//! merging, the vector path, the SVG preview and the size helpers.

use ebc_qr::{
    EcLevel, MIN_PRINT_MODULE_MM, Placement, QUIET_ZONE_MODULES, QrCode,
    RECOMMENDED_PRINT_MODULE_MM,
};
use ebc_scene::{PathSeg, Point, Rect};

/// Deterministic pseudo-random generator for test payloads.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}

/// A random payload from one of four alphabets (numeric, alphanumeric, ASCII, Unicode).
fn random_payload(rng: &mut XorShift) -> String {
    let alphabet: Vec<char> = match rng.next(4) {
        0 => "0123456789".chars().collect(),
        1 => "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:".chars().collect(),
        2 => (0x20u8..0x7f).map(char::from).collect(),
        _ => "aàâçéèêëîïôùûüÿœæ €—🎁📚日本語0123ABC".chars().collect(),
    };
    let longest = if rng.next(5) == 0 { 300 } else { 80 };
    let len = 1 + rng.next(longest);
    (0..len).map(|_| alphabet[rng.next(alphabet.len())]).collect()
}

fn random_symbols(count: usize, seed: u64) -> Vec<QrCode> {
    let mut rng = XorShift(seed);
    (0..count)
        .map(|_| {
            let text = random_payload(&mut rng);
            let level = EcLevel::ALL[rng.next(4)];
            QrCode::encode(&text, level).expect("random payloads are short enough")
        })
        .collect()
}

fn claim_code() -> QrCode {
    QrCode::encode_url("https://ebk.example/K/7K3M9QX2RD", EcLevel::M).expect("fits")
}

// --- the matrix ---------------------------------------------------------------------------

#[test]
fn size_follows_the_version_and_out_of_range_modules_are_light() {
    for qr in random_symbols(60, 0xA5A5_0001) {
        let n = qr.size();
        assert_eq!(n, 17 + 4 * u32::from(qr.version()));
        assert!((1..=40).contains(&qr.version()));
        assert!(qr.mask() <= 7);
        for (x, y) in [(n, 0), (0, n), (n, n), (u32::MAX, 0), (0, u32::MAX), (u32::MAX, u32::MAX)] {
            assert!(!qr.module(x, y), "({x}, {y}) is outside the symbol");
        }
        let dark =
            (0..n).flat_map(|y| (0..n).map(move |x| (x, y))).filter(|&(x, y)| qr.module(x, y));
        assert_eq!(dark.count(), qr.dark_count());
        // A sane symbol is roughly half dark (masking balances it).
        let ratio = qr.dark_count() as f64 / f64::from(n * n);
        assert!((0.35..0.65).contains(&ratio), "dark ratio {ratio}");
    }
}

/// Checks a 7 x 7 finder pattern whose top-left corner is at `(x0, y0)`.
fn assert_finder(qr: &QrCode, x0: u32, y0: u32) {
    for dy in 0..7u32 {
        for dx in 0..7u32 {
            let ring = dx.min(6 - dx).min(dy).min(6 - dy);
            let expect_dark = ring != 1; // dark border, light ring, dark 3 x 3 core
            assert_eq!(
                qr.module(x0 + dx, y0 + dy),
                expect_dark,
                "finder at ({x0},{y0}) +({dx},{dy})"
            );
        }
    }
}

#[test]
fn function_patterns_are_where_the_standard_puts_them() {
    for qr in random_symbols(80, 0xA5A5_0002) {
        let n = qr.size();
        // Finder patterns in three corners, with their light separators.
        assert_finder(&qr, 0, 0);
        assert_finder(&qr, n - 7, 0);
        assert_finder(&qr, 0, n - 7);
        for i in 0..8 {
            for (x, y) in [(7, i), (i, 7), (n - 8, i), (n - 8 + i, 7), (7, n - 8 + i), (i, n - 8)] {
                assert!(!qr.module(x, y), "separator module ({x},{y}) must be light");
            }
        }
        // The fourth corner carries no finder pattern (this is what gives a symbol its
        // orientation).
        let looks_like_finder = (0..7u32).all(|d| {
            let ring = |dx: u32, dy: u32| dx.min(6 - dx).min(dy).min(6 - dy);
            (0..7u32).all(|e| qr.module(n - 7 + d, n - 7 + e) == (ring(d, e) != 1))
        });
        assert!(!looks_like_finder);
        // Timing patterns: alternating along row 6 and column 6, starting dark.
        for i in 8..n - 8 {
            assert_eq!(qr.module(i, 6), i % 2 == 0, "horizontal timing at {i}");
            assert_eq!(qr.module(6, i), i % 2 == 0, "vertical timing at {i}");
        }
        // The always-dark module next to the bottom-left finder.
        assert!(qr.module(8, n - 8));
        // Alignment pattern (versions ≥ 2): centre at (n − 7, n − 7), dark–light–dark rings.
        if qr.version() >= 2 {
            let c = n - 7;
            for dy in 0..5u32 {
                for dx in 0..5u32 {
                    let ring = dx.abs_diff(2).max(dy.abs_diff(2));
                    assert_eq!(
                        qr.module(c - 2 + dx, c - 2 + dy),
                        ring != 1,
                        "alignment +({dx},{dy})"
                    );
                }
            }
        }
    }
}

/// Reads the 15 format bits from their first copy (around the top-left finder), in the
/// positions the standard assigns to bits 0–14, and removes the fixed mask.
fn read_format_bits(qr: &QrCode) -> u16 {
    const XS: [u32; 15] = [8, 8, 8, 8, 8, 8, 8, 8, 7, 5, 4, 3, 2, 1, 0];
    const YS: [u32; 15] = [0, 1, 2, 3, 4, 5, 7, 8, 8, 8, 8, 8, 8, 8, 8];
    let mut bits = 0u16;
    for (i, (&x, &y)) in XS.iter().zip(&YS).enumerate() {
        bits |= u16::from(qr.module(x, y)) << i;
    }
    bits ^ 0x5412
}

/// Remainder of the (15, 5) BCH code used for the format information.
fn bch_remainder(data: u16) -> u16 {
    let mut rem = u32::from(data);
    for _ in 0..10 {
        rem = (rem << 1) ^ ((rem >> 9) * 0x537);
    }
    (rem & 0x3ff) as u16
}

#[test]
fn format_information_states_the_level_and_mask_in_the_right_orientation() {
    // This fails if the matrix were transposed or mirrored, which a round trip through a
    // decoder that tolerates mirrored symbols would not notice.
    for qr in random_symbols(120, 0xA5A5_0003) {
        let format = read_format_bits(&qr);
        let data = format >> 10;
        assert_eq!(format & 0x3ff, bch_remainder(data), "format bits are not a BCH codeword");
        let level = match data >> 3 {
            0b01 => EcLevel::L,
            0b00 => EcLevel::M,
            0b11 => EcLevel::Q,
            _ => EcLevel::H,
        };
        assert_eq!(level, qr.ec_level());
        assert_eq!((data & 7) as u8, qr.mask());

        // The second copy: bits 0–7 right to left under the top-right finder, bits 8–14
        // top to bottom beside the bottom-left finder.
        let n = qr.size();
        let mut second = 0u16;
        for i in 0..8u32 {
            second |= u16::from(qr.module(n - 1 - i, 8)) << i;
        }
        for i in 8..15u32 {
            second |= u16::from(qr.module(8, n - 15 + i)) << i;
        }
        assert_eq!(second ^ 0x5412, format, "the two copies of the format information differ");
    }
}

#[test]
fn hello_world_symbol_is_pinned() {
    // Regression pin of a complete symbol ("HELLO WORLD", version 1, level Q — the worked
    // example of many QR tutorials), so that an encoder update that changes mask selection
    // or module placement is noticed. '#' = dark.
    let qr = QrCode::encode("HELLO WORLD", EcLevel::Q).expect("fits");
    assert_eq!((qr.version(), qr.size(), qr.ec_level()), (1, 21, EcLevel::Q));
    let rows: Vec<String> = (0..21)
        .map(|y| (0..21).map(|x| if qr.module(x, y) { '#' } else { '.' }).collect())
        .collect();
    assert_eq!(rows.join("\n"), HELLO_WORLD_1Q.trim());
}

/// Output of this crate at the time of writing (qrcodegen 1.8.0, mask chosen by the
/// standard penalty rules); verified to decode with `rqrr`, not copied from a reference.
const HELLO_WORLD_1Q: &str = "
#######.##....#######
#.....#.#..#..#.....#
#.###.#.#..##.#.###.#
#.###.#.#.....#.###.#
#.###.#.#.#...#.###.#
#.....#...#...#.....#
#######.#.#.#.#######
........#............
.##.#.##....#.#.#####
.#......####....#...#
..##.###.##...#.##...
.##.##.#..##.#.#.###.
#...#.#.#.###.###.#.#
........##.#..#...#.#
#######.#.#....#.##..
#.....#..#.##.##.#...
#.###.#.#.#...#######
#.###.#..#.#.#.#...#.
#.###.#.#..#.###.#..#
#.....#.#.####...#.##
#######....#.###....#
";

// --- rects --------------------------------------------------------------------------------

#[test]
fn rects_cover_exactly_the_dark_modules_without_overlap() {
    let mut total_rects = 0;
    let mut total_dark = 0;
    for qr in random_symbols(400, 0xA5A5_0004) {
        let n = qr.size() as usize;
        let mut coverage = vec![0u8; n * n];
        let rects = qr.rects();
        for &(x, y, w, h) in &rects {
            assert!(w >= 1 && h >= 1, "degenerate rect {:?}", (x, y, w, h));
            assert!(x + w <= qr.size() && y + h <= qr.size(), "rect outside the symbol");
            for yy in y..y + h {
                for xx in x..x + w {
                    coverage[yy as usize * n + xx as usize] += 1;
                }
            }
        }
        for y in 0..qr.size() {
            for x in 0..qr.size() {
                let expected = u8::from(qr.module(x, y));
                assert_eq!(coverage[y as usize * n + x as usize], expected, "module ({x},{y})");
            }
        }
        let area: u32 = rects.iter().map(|&(_, _, w, h)| w * h).sum();
        assert_eq!(area as usize, qr.dark_count());
        total_rects += rects.len();
        total_dark += qr.dark_count();
    }
    // Merging must actually pay off: well under half as many rectangles as dark modules.
    assert!(total_rects * 2 < total_dark, "{total_rects} rects for {total_dark} dark modules");
}

#[test]
fn rects_are_maximal_runs_sorted_and_stacked() {
    for qr in random_symbols(150, 0xA5A5_0005) {
        let rects = qr.rects();
        let n = qr.size();
        // Sorted by (y, x), no duplicates.
        for pair in rects.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            assert!((a.1, a.0) < (b.1, b.0), "{a:?} before {b:?}");
        }
        for &(x, y, w, h) in &rects {
            // Horizontally maximal in every row: light (or the edge) on both sides.
            for row in y..y + h {
                assert!(x == 0 || !qr.module(x - 1, row), "{:?} could extend left", (x, y, w, h));
                assert!(
                    x + w == n || !qr.module(x + w, row),
                    "{:?} could extend right",
                    (x, y, w, h)
                );
            }
            // Vertically maximal: the row above/below does not hold the identical run.
            let same_run = |row: u32| {
                (x..x + w).all(|xx| qr.module(xx, row))
                    && (x == 0 || !qr.module(x - 1, row))
                    && (x + w == n || !qr.module(x + w, row))
            };
            assert!(y == 0 || !same_run(y - 1), "{:?} could extend up", (x, y, w, h));
            assert!(y + h == n || !same_run(y + h), "{:?} could extend down", (x, y, w, h));
        }
        // The count equals the number of maximal horizontal runs minus the stacked ones:
        // at most one rect per run, at least one per distinct run column pattern.
        let runs: usize = (0..n)
            .map(|y| {
                (0..n).filter(|&x| qr.module(x, y) && (x == 0 || !qr.module(x - 1, y))).count()
            })
            .sum();
        assert!(rects.len() <= runs);
    }
}

#[test]
fn finder_patterns_become_few_rectangles() {
    // The top-left finder alone: top bar, two sides (stacked 5 high), 3 x 3 core, bottom bar.
    let qr = claim_code();
    let rects = qr.rects();
    assert!(rects.contains(&(0, 0, 7, 1)), "top bar");
    assert!(rects.contains(&(2, 2, 3, 3)), "core, stacked over three rows");
    assert!(rects.contains(&(0, 6, 7, 1)), "bottom bar");
    // The sides of the ring are single modules in rows 1–5: same x and width in
    // consecutive rows, so each side stacks into one 1 x 5 rectangle.
    assert!(rects.contains(&(0, 1, 1, 5)), "left side: {:?}", &rects[..12]);
    assert!(rects.contains(&(6, 1, 1, 5)), "right side: {:?}", &rects[..12]);
}

// --- to_path ------------------------------------------------------------------------------

#[test]
fn path_is_one_closed_rectangle_per_rect_at_exact_multiples_of_the_module() {
    let origin = Point::new(12.3, 45.6);
    let module = 0.437;
    for qr in random_symbols(40, 0xA5A5_0006) {
        let rects = qr.rects();
        let path = qr.to_path(origin, module);
        assert_eq!(path.segs.len(), rects.len() * 5);
        let at = |x: u32, y: u32| {
            Point::new(origin.x + f64::from(x) * module, origin.y + f64::from(y) * module)
        };
        let mut area = 0.0;
        for (segs, &(x, y, w, h)) in path.segs.chunks(5).zip(&rects) {
            // Bit-exact coordinates: abutting rectangles share identical edge values.
            assert_eq!(
                segs,
                [
                    PathSeg::MoveTo(at(x, y)),
                    PathSeg::LineTo(at(x + w, y)),
                    PathSeg::LineTo(at(x + w, y + h)),
                    PathSeg::LineTo(at(x, y + h)),
                    PathSeg::Close,
                ]
            );
            area += f64::from(w * h) * module * module;
        }
        assert!((area - qr.dark_count() as f64 * module * module).abs() < 1e-6);

        // Bounds: finder patterns touch the left, top, right and bottom edges of the symbol.
        let bounds = path.control_bounds().expect("non-empty path");
        let side = f64::from(qr.size()) * module;
        assert!((bounds.x - origin.x).abs() < 1e-9 && (bounds.y - origin.y).abs() < 1e-9);
        assert!((bounds.w - side).abs() < 1e-9 && (bounds.h - side).abs() < 1e-9);
        // The quiet zone is the caller's: the path stays strictly inside the symbol square.
        assert!(Rect::new(origin.x, origin.y, side, side).contains_rect(&bounds, 1e-9));
    }
}

#[test]
fn path_winding_is_consistent_so_any_fill_rule_works() {
    // Every sub-path runs right, down, left (clockwise on a y-down page): with identical
    // winding and no overlap, non-zero and even-odd fills agree.
    let qr = claim_code();
    let path = qr.to_path(Point::new(0.0, 0.0), 1.0);
    for segs in path.segs.chunks(5) {
        let [
            PathSeg::MoveTo(a),
            PathSeg::LineTo(b),
            PathSeg::LineTo(c),
            PathSeg::LineTo(d),
            PathSeg::Close,
        ] = segs
        else {
            panic!("unexpected sub-path {segs:?}");
        };
        assert!(b.x > a.x && b.y == a.y, "first edge goes right");
        assert!(c.x == b.x && c.y > b.y, "second edge goes down");
        assert!(d.x == a.x && d.y == c.y, "third edge goes left");
    }
}

#[test]
fn path_rejects_unusable_geometry_instead_of_emitting_garbage() {
    let qr = claim_code();
    for module in [0.0, -0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(qr.to_path(Point::new(10.0, 10.0), module).is_empty(), "module {module}");
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(qr.to_path(Point::new(bad, 0.0), 0.5).is_empty());
        assert!(qr.to_path(Point::new(0.0, bad), 0.5).is_empty());
    }
    // Negative page coordinates (inside the bleed) and tiny modules are legitimate.
    assert!(!qr.to_path(Point::new(-3.0, -3.0), 1e-6).is_empty());
}

// --- size helpers -------------------------------------------------------------------------

#[test]
fn print_size_rules() {
    let qr = claim_code(); // version 2: 25 modules
    // The published rules (ISO/IEC 18004 quiet zone, GS1 minimum X-dimension), checked at
    // compile time so that a careless edit of the constants cannot slip through.
    const {
        assert!(QUIET_ZONE_MODULES == 4);
        assert!(MIN_PRINT_MODULE_MM > 0.39 && RECOMMENDED_PRINT_MODULE_MM >= MIN_PRINT_MODULE_MM);
    }

    assert_eq!(qr.side_mm(1.0, 4), 33.0);
    assert_eq!(qr.side_mm(0.5, 0), 12.5);
    assert!((qr.side_mm(MIN_PRINT_MODULE_MM, QUIET_ZONE_MODULES) - 13.2).abs() < 1e-12);
    // The 30 mm box of the research notes gives comfortable 0.9 mm modules.
    let module = qr.module_mm_for_side(30.0, QUIET_ZONE_MODULES);
    assert!((module - 30.0 / 33.0).abs() < 1e-12);
    assert!(module >= RECOMMENDED_PRINT_MODULE_MM);
    // The two helpers are inverses.
    for side in [10.0, 25.4, 33.0, 80.0] {
        for quiet in [0, 2, 4, 8] {
            let m = qr.module_mm_for_side(side, quiet);
            assert!((qr.side_mm(m, quiet) - side).abs() < 1e-9);
        }
    }
    // A dense symbol squeezed into a small box is flagged by the constant.
    let dense = QrCode::encode(&"x".repeat(500), EcLevel::M).expect("fits");
    assert!(dense.module_mm_for_side(25.0, QUIET_ZONE_MODULES) < MIN_PRINT_MODULE_MM);
    // Hostile arguments cannot overflow.
    assert!(qr.side_mm(1.0, u32::MAX).is_finite());
    assert!(qr.module_mm_for_side(30.0, u32::MAX) > 0.0);
}

#[test]
fn fit_in_centres_the_symbol_and_its_quiet_zone() {
    let qr = claim_code(); // 25 modules + 2 x 4 = 33
    // Square box.
    let Placement { origin, module_mm } = qr.fit_in(Rect::new(40.0, 60.0, 33.0, 33.0), 4);
    assert!((module_mm - 1.0).abs() < 1e-12);
    assert!((origin.x - 44.0).abs() < 1e-12 && (origin.y - 64.0).abs() < 1e-12);

    // Wide box: limited by the height, centred horizontally.
    let outer = Rect::new(10.0, 20.0, 99.0, 66.0);
    let placement = qr.fit_in(outer, 4);
    assert!((placement.module_mm - 2.0).abs() < 1e-12);
    assert!((placement.origin.x - (10.0 + 16.5 + 8.0)).abs() < 1e-12);
    assert!((placement.origin.y - 28.0).abs() < 1e-12);

    // The drawn symbol plus its quiet zone stays inside the box, for any box.
    for (w, h, quiet) in [(30.0, 30.0, 4), (20.0, 45.0, 4), (70.0, 25.0, 2), (15.0, 15.0, 0)] {
        let outer = Rect::new(5.0, 7.0, w, h);
        let p = qr.fit_in(outer, quiet);
        let bounds = qr.to_path(p.origin, p.module_mm).control_bounds().expect("non-empty");
        let with_quiet = bounds.inflate(f64::from(quiet) * p.module_mm);
        assert!(outer.contains_rect(&with_quiet, 1e-9), "{with_quiet:?} outside {outer:?}");
        let centre = with_quiet.center();
        assert!((centre.x - outer.center().x).abs() < 1e-9);
        assert!((centre.y - outer.center().y).abs() < 1e-9);
        assert!((with_quiet.w - w.min(h)).abs() < 1e-9, "fills the shorter side");
    }
}

// --- to_svg -------------------------------------------------------------------------------

/// Parses the `M x,y h w v h h -w z` commands of the SVG path back into rectangles.
fn rects_from_svg(svg: &str) -> Vec<(u64, u64, u64, u64)> {
    let start = svg.find(" d=\"").expect("path data") + 4;
    let data = &svg[start..start + svg[start..].find('"').expect("closing quote")];
    data.split('M')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (xy, rest) = part.split_once('h').expect("h command");
            let (x, y) = xy.split_once(',').expect("x,y");
            let (w, rest) = rest.split_once('v').expect("v command");
            let (h, rest) = rest.split_once("h-").expect("h- command");
            let back = rest.strip_suffix('z').expect("closed sub-path");
            assert_eq!(back, w, "the bottom edge returns by the width");
            let n = |s: &str| s.parse::<u64>().expect("integer coordinate");
            (n(x), n(y), n(w), n(h))
        })
        .collect()
}

fn attribute<'a>(svg: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = svg.find(&key).unwrap_or_else(|| panic!("attribute {name}")) + key.len();
    &svg[start..start + svg[start..].find('"').expect("closing quote")]
}

#[test]
fn svg_draws_exactly_the_matrix_inside_its_quiet_zone() {
    for qr in random_symbols(40, 0xA5A5_0007) {
        for (module_px, quiet) in [(1, 0), (4, 4), (10, 2)] {
            let svg = qr.to_svg(module_px, quiet);
            assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
            assert!(svg.ends_with("</svg>"));
            let side = u64::from(qr.size() + 2 * quiet);
            assert_eq!(attribute(&svg, "viewBox"), format!("0 0 {side} {side}"));
            assert_eq!(attribute(&svg, "width"), (side * u64::from(module_px)).to_string());
            assert_eq!(attribute(&svg, "height"), (side * u64::from(module_px)).to_string());
            assert!(
                svg.contains(&format!(
                    "<rect width=\"{side}\" height=\"{side}\" fill=\"#ffffff\"/>"
                ))
            );

            // Paint the path back onto a grid and compare with the modules.
            let mut grid = vec![false; (side * side) as usize];
            for (x, y, w, h) in rects_from_svg(&svg) {
                for yy in y..y + h {
                    for xx in x..x + w {
                        let cell = &mut grid[(yy * side + xx) as usize];
                        assert!(!*cell, "overlap at ({xx},{yy})");
                        *cell = true;
                    }
                }
            }
            for y in 0..side {
                for x in 0..side {
                    let (mx, my) = ((x as u32).wrapping_sub(quiet), (y as u32).wrapping_sub(quiet));
                    assert_eq!(grid[(y * side + x) as usize], qr.module(mx, my), "({x},{y})");
                }
            }
        }
    }
}

#[test]
fn svg_is_well_formed_and_survives_hostile_parameters() {
    let qr = claim_code();
    let svg = qr.to_svg(8, 4);
    // Tags balance and nothing needs escaping: digits, path letters, commas and minus only.
    assert_eq!(svg.matches('<').count(), svg.matches('>').count());
    assert_eq!(svg.matches("<svg").count(), 1);
    assert_eq!(svg.matches("<path").count(), 1);
    assert!(attribute(&svg, "d").chars().all(|c| c.is_ascii_digit() || "Mhvz,-".contains(c)));
    assert!(!svg.contains("NaN") && !svg.contains("inf"));

    // A module size of 0 is treated as 1.
    assert_eq!(qr.to_svg(0, 4), qr.to_svg(1, 4));
    // Absurd sizes neither overflow nor allocate proportionally.
    let huge = qr.to_svg(u32::MAX, u32::MAX);
    let side = 25 + 2 * u128::from(u32::MAX);
    assert_eq!(attribute(&huge, "viewBox"), format!("0 0 {side} {side}"));
    assert_eq!(attribute(&huge, "width"), (side * u128::from(u32::MAX)).to_string());
    assert!(huge.len() < 20_000, "{} bytes", huge.len());
}

#[test]
fn debug_output_is_a_summary_not_a_matrix_dump() {
    let text = format!("{:?}", claim_code());
    assert!(
        text.contains("version: 2") && text.contains("size: 25") && text.contains("ec_level: M")
    );
    assert!(text.len() < 120, "{text}");
}
