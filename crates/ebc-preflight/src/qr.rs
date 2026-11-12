//! Recognises QR-code-like vector graphics so that their module size can be checked.
//!
//! A vector QR code is a set of axis-aligned dark rectangles (single modules or merged
//! runs) on a common grid, whose outline is a square of `21 + 4·n` modules. The three finder
//! patterns guarantee that the dark modules span the full square, and the timing pattern
//! guarantees at least one rectangle that is exactly one module thick. This is a heuristic:
//! it works on geometry only and does not decode the symbol.

use crate::report::{QrCandidate, round3};

/// Fewest rectangles a run must have to be considered (a version 1 symbol drawn as merged
/// horizontal runs has well over fifty).
pub(crate) const MIN_RECTS: usize = 20;
/// Most rectangles kept per run: a version 40 symbol has 177² = 31 329 modules.
pub(crate) const MAX_RECTS: usize = 40_000;

/// An axis-aligned rectangle in points: `(x0, y0, x1, y1)` with `x0 <= x1`, `y0 <= y1`.
pub(crate) type Rect = (f64, f64, f64, f64);

/// Analyses a run of same-colour rectangle fills. `mm_per_unit` converts the rectangles'
/// unit (points × `UserUnit`) to millimetres.
pub(crate) fn detect(rects: &[Rect], mm_per_unit: f64) -> Option<QrCandidate> {
    if rects.len() < MIN_RECTS || rects.len() > MAX_RECTS {
        return None;
    }
    let mut module = f64::INFINITY;
    let (mut x0, mut y0, mut x1, mut y1) =
        (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for &(rx0, ry0, rx1, ry1) in rects {
        let (w, h) = (rx1 - rx0, ry1 - ry0);
        if !(w.is_finite() && h.is_finite()) || w <= 0.0 || h <= 0.0 {
            return None;
        }
        module = module.min(w).min(h);
        x0 = x0.min(rx0);
        y0 = y0.min(ry0);
        x1 = x1.max(rx1);
        y1 = y1.max(ry1);
    }
    // Below a hundredth of a point nothing is printable, and ratios become noise.
    if !module.is_finite() || module < 0.01 {
        return None;
    }
    let (width, height) = (x1 - x0, y1 - y0);
    if (width - height).abs() > module * 0.5 {
        return None;
    }
    let modules_f = (width / module).round();
    if !(21.0..=177.0).contains(&modules_f) {
        return None;
    }
    // In range, whole and positive: the cast is exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let modules = modules_f as u32;
    if (modules - 21) % 4 != 0 {
        return None;
    }
    // The module pitch, which may differ slightly from the thinnest rectangle when the
    // producer overlaps modules to avoid hairline gaps.
    let pitch = width / modules_f;
    let on_grid = |v: f64| {
        let cells = v / pitch;
        (cells - cells.round()).abs() <= 0.12
    };
    for &(rx0, ry0, rx1, ry1) in rects {
        if !(on_grid(rx0 - x0) && on_grid(ry0 - y0) && on_grid(rx1 - rx0) && on_grid(ry1 - ry0)) {
            return None;
        }
    }
    Some(QrCandidate {
        module_mm: round3(pitch * mm_per_unit),
        side_mm: round3(width * mm_per_unit),
        modules,
    })
}

/// Whether a closed subpath given by its vertices is an axis-aligned rectangle; returns it
/// normalised. Accepts four vertices, or five when the last repeats the first.
pub(crate) fn axis_aligned_rect(points: &[(f64, f64)]) -> Option<Rect> {
    const EPS: f64 = 1.0e-3;
    let points = match points {
        [first, .., last] if points.len() == 5 => {
            if (first.0 - last.0).abs() > EPS || (first.1 - last.1).abs() > EPS {
                return None;
            }
            &points[..4]
        }
        _ if points.len() == 4 => points,
        _ => return None,
    };
    let horizontal = |a: (f64, f64), b: (f64, f64)| (a.1 - b.1).abs() <= EPS && (a.0 - b.0).abs() > EPS;
    let vertical = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() <= EPS && (a.1 - b.1).abs() > EPS;
    let (p0, p1, p2, p3) = (points[0], points[1], points[2], points[3]);
    let hvhv = horizontal(p0, p1) && vertical(p1, p2) && horizontal(p2, p3) && vertical(p3, p0);
    let vhvh = vertical(p0, p1) && horizontal(p1, p2) && vertical(p2, p3) && horizontal(p3, p0);
    if !(hvhv || vhvh) {
        return None;
    }
    let xs = [p0.0, p1.0, p2.0, p3.0];
    let ys = [p0.1, p1.1, p2.1, p3.1];
    let min = |v: [f64; 4]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let max = |v: [f64; 4]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some((min(xs), min(ys), max(xs), max(ys)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {
    use super::*;

    /// A deterministic 21×21 matrix with the fixed patterns of a version 1 QR code (finder
    /// patterns, timing patterns) and pseudo-random data modules.
    pub(crate) fn fake_qr_matrix(size: usize) -> Vec<Vec<bool>> {
        let mut m = vec![vec![false; size]; size];
        let mut finder = |ox: usize, oy: usize| {
            for y in 0..7 {
                for x in 0..7 {
                    let ring = x == 0 || y == 0 || x == 6 || y == 6;
                    let core = (2..=4).contains(&x) && (2..=4).contains(&y);
                    m[oy + y][ox + x] = ring || core;
                }
            }
        };
        finder(0, 0);
        finder(size - 7, 0);
        finder(0, size - 7);
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        for y in 0..size {
            for x in 0..size {
                let in_finder = (x < 8 && y < 8) || (x >= size - 8 && y < 8) || (x < 8 && y >= size - 8);
                if in_finder {
                    continue;
                }
                if y == 6 || x == 6 {
                    m[y][x] = (x + y) % 2 == 0;
                    continue;
                }
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                m[y][x] = state & 1 == 1;
            }
        }
        m
    }

    /// Horizontal dark runs as rectangles `(x0, y0, x1, y1)` in the given unit.
    pub(crate) fn runs(matrix: &[Vec<bool>], origin: (f64, f64), module: f64) -> Vec<Rect> {
        let mut out = Vec::new();
        for (y, row) in matrix.iter().enumerate() {
            let mut x = 0;
            while x < row.len() {
                if row[x] {
                    let start = x;
                    while x < row.len() && row[x] {
                        x += 1;
                    }
                    out.push((
                        origin.0 + start as f64 * module,
                        origin.1 + y as f64 * module,
                        origin.0 + x as f64 * module,
                        origin.1 + (y + 1) as f64 * module,
                    ));
                } else {
                    x += 1;
                }
            }
        }
        out
    }

    #[test]
    fn recognises_a_qr_like_grid_and_measures_its_module() {
        let matrix = fake_qr_matrix(21);
        // 0.5 mm modules, expressed in points.
        let module_pt = 0.5 * 72.0 / 25.4;
        let rects = runs(&matrix, (100.0, 200.0), module_pt);
        assert!(rects.len() >= MIN_RECTS);
        let qr = detect(&rects, 25.4 / 72.0).unwrap();
        assert_eq!(qr.modules, 21);
        assert!((qr.module_mm - 0.5).abs() < 1e-3, "{qr:?}");
        assert!((qr.side_mm - 10.5).abs() < 1e-3, "{qr:?}");
        // Larger symbol.
        let rects = runs(&fake_qr_matrix(33), (0.0, 0.0), 1.0);
        assert_eq!(detect(&rects, 1.0).unwrap().modules, 33);
    }

    #[test]
    fn tolerates_slightly_overlapping_modules() {
        let matrix = fake_qr_matrix(25);
        let rects: Vec<Rect> = runs(&matrix, (10.0, 10.0), 2.0)
            .into_iter()
            .map(|(x0, y0, x1, y1)| (x0 - 0.02, y0 - 0.02, x1 + 0.02, y1 + 0.02))
            .collect();
        let qr = detect(&rects, 1.0).unwrap();
        assert_eq!(qr.modules, 25);
        assert!((qr.module_mm - 2.0).abs() < 0.01, "{qr:?}");
    }

    #[test]
    fn rejects_things_that_are_not_qr_codes() {
        // Too few rectangles.
        assert!(detect(&[(0.0, 0.0, 1.0, 1.0); 5], 1.0).is_none());
        // A colour bar: a row of squares is not square overall.
        let bar: Vec<Rect> = (0..30).map(|i| (f64::from(i) * 2.0, 0.0, f64::from(i) * 2.0 + 1.0, 1.0)).collect();
        assert!(detect(&bar, 1.0).is_none());
        // A square grid with a side that is not 21 + 4n modules.
        let grid: Vec<Rect> = (0..22)
            .flat_map(|y| (0..22).map(move |x| (f64::from(x), f64::from(y), f64::from(x) + 1.0, f64::from(y) + 1.0)))
            .collect();
        assert!(detect(&grid, 1.0).is_none());
        // Rectangles off the module grid.
        let mut rects = runs(&fake_qr_matrix(21), (0.0, 0.0), 1.0);
        rects[10].0 += 0.4;
        assert!(detect(&rects, 1.0).is_none());
        // Degenerate and non-finite rectangles.
        let mut rects = runs(&fake_qr_matrix(21), (0.0, 0.0), 1.0);
        rects[3] = (1.0, 1.0, 1.0, 2.0);
        assert!(detect(&rects, 1.0).is_none());
        rects[3] = (f64::NAN, 1.0, 2.0, 2.0);
        assert!(detect(&rects, 1.0).is_none());
    }

    #[test]
    fn rectangle_subpaths() {
        let r = axis_aligned_rect(&[(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]).unwrap();
        assert_eq!(r, (0.0, 0.0, 2.0, 1.0));
        // Clockwise, starting elsewhere, explicitly closed.
        let r = axis_aligned_rect(&[(2.0, 1.0), (2.0, 0.0), (0.0, 0.0), (0.0, 1.0), (2.0, 1.0)]).unwrap();
        assert_eq!(r, (0.0, 0.0, 2.0, 1.0));
        // Not closed back to the start.
        assert!(axis_aligned_rect(&[(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0), (0.5, 0.5)]).is_none());
        // A parallelogram, a triangle, a line, a zero-width rectangle.
        assert!(axis_aligned_rect(&[(0.0, 0.0), (2.0, 0.0), (3.0, 1.0), (1.0, 1.0)]).is_none());
        assert!(axis_aligned_rect(&[(0.0, 0.0), (2.0, 0.0), (2.0, 1.0)]).is_none());
        assert!(axis_aligned_rect(&[(0.0, 0.0), (2.0, 0.0)]).is_none());
        assert!(axis_aligned_rect(&[(0.0, 0.0), (0.0, 0.0), (0.0, 1.0), (0.0, 1.0)]).is_none());
    }
}
