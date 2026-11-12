//! Adversarial ("sparring") tests for the palette extraction and edge analysis of
//! `ebc-color`, through the public API only.

use std::time::{Duration, Instant};

use ebc_color::{
    BorderOptions, Edge, MAX_PALETTE_SIZE, PALETTE_MERGE_DELTA_E, Rgb8, Swatch, edge_palette,
    edge_uniformity, has_uniform_border, palette, uniform_border_with,
};
use ebc_scene::Pixmap;

/// xorshift64*: a tiny deterministic generator for test inputs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn rgb(&mut self) -> Rgb8 {
        let v = self.next();
        Rgb8::new((v >> 16) as u8, (v >> 32) as u8, (v >> 48) as u8)
    }
}

fn image(w: u32, h: u32, mut f: impl FnMut(u32, u32) -> [u8; 4]) -> Pixmap {
    let mut data = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            data.extend_from_slice(&f(x, y));
        }
    }
    Pixmap::from_rgba8(w, h, data).expect("consistent test image")
}

fn opaque(c: Rgb8) -> [u8; 4] {
    [c.r, c.g, c.b, 255]
}

/// The structural promises of the documentation of `palette` and `Swatch`.
fn assert_well_formed(swatches: &[Swatch], k: usize, context: &str) {
    assert!(swatches.len() <= k.min(MAX_PALETTE_SIZE), "{context}: {} > k = {k}", swatches.len());
    if k >= 1 {
        assert!(!swatches.is_empty(), "{context}: k >= 1 must give at least one swatch");
    }
    let total: f64 = swatches.iter().map(|s| f64::from(s.share)).sum();
    if !swatches.is_empty() {
        assert!((total - 1.0).abs() < 1e-4, "{context}: shares add up to {total}");
    }
    for s in swatches {
        assert!(s.share.is_finite() && s.share > 0.0 && s.share <= 1.0, "{context}: {s:?}");
    }
    for pair in swatches.windows(2) {
        assert!(pair[0].share >= pair[1].share, "{context}: not sorted by share: {swatches:?}");
    }
    for (i, a) in swatches.iter().enumerate() {
        for b in &swatches[i + 1..] {
            assert_ne!(a.color, b.color, "{context}: the same colour is listed twice");
        }
    }
}

// --- structural invariants on hostile images ------------------------------------------------

#[test]
fn palettes_of_random_images_are_well_formed() {
    let mut rng = Rng(0xA11CE);
    for round in 0..400 {
        let w = 1 + rng.below(if round % 10 == 0 { 400 } else { 60 }) as u32;
        let h = 1 + rng.below(if round % 10 == 1 { 400 } else { 60 }) as u32;
        // Between 1 and ~40 base colours, optionally with noise and transparency.
        let bases: Vec<Rgb8> = (0..1 + rng.below(40)).map(|_| rng.rgb()).collect();
        let noise = [0u64, 0, 2, 8, 256][rng.below(5) as usize];
        let with_alpha = round % 4 == 0;
        let mut pixels = Rng(rng.next() | 1);
        let img = image(w, h, |_, _| {
            let base = bases[pixels.below(bases.len() as u64) as usize];
            let jitter = |v: u8, r: &mut Rng| {
                if noise == 0 { v } else { v.wrapping_add((r.below(noise)) as u8) }
            };
            let a = if with_alpha { pixels.below(256) as u8 } else { 255 };
            [
                jitter(base.r, &mut pixels),
                jitter(base.g, &mut pixels),
                jitter(base.b, &mut pixels),
                a,
            ]
        });
        for k in [0usize, 1, 2, 3, 5, 8, 64, 65, 1_000_000, usize::MAX] {
            let swatches = palette(&img, k);
            assert_well_formed(&swatches, k, &format!("round {round}, {w}x{h}, k = {k}"));
            assert_eq!(swatches, palette(&img, k), "palette must be deterministic");
        }
        for edge in Edge::ALL {
            for fraction in [0.0_f32, 0.01, 0.33, 1.0, f32::NAN, -1.0, f32::INFINITY] {
                let swatches = edge_palette(&img, edge, fraction, 4);
                assert_well_formed(&swatches, 4, &format!("round {round} {edge:?} {fraction}"));
                let u = edge_uniformity(&img, edge, fraction);
                assert!((0.0..=1.0).contains(&u), "uniformity {u}");
            }
        }
        let _ = has_uniform_border(&img);
    }
}

#[test]
fn degenerate_image_shapes_do_not_panic() {
    for (w, h) in [(1, 1), (1, 2), (2, 1), (1, 16_385), (16_385, 1), (3, 5_462), (127, 130)] {
        let img = image(w, h, |x, y| opaque(Rgb8::new(x as u8, y as u8, (x ^ y) as u8)));
        for k in [1, 4, 64] {
            assert_well_formed(&palette(&img, k), k, &format!("{w}x{h}"));
        }
        for edge in Edge::ALL {
            let _ = edge_palette(&img, edge, 0.5, 3);
            let u = edge_uniformity(&img, edge, 0.02);
            assert!((0.0..=1.0).contains(&u));
        }
        let _ = has_uniform_border(&img);
    }
}

#[test]
fn fully_transparent_pixels_are_paper_white_whatever_their_rgb() {
    let mut rng = Rng(9);
    let img = image(50, 50, |_, _| {
        let c = rng.rgb();
        [c.r, c.g, c.b, 0]
    });
    assert_eq!(palette(&img, 5), vec![Swatch { color: Rgb8::WHITE, share: 1.0 }]);
    assert_eq!(has_uniform_border(&img), Some(Rgb8::WHITE));
    for edge in Edge::ALL {
        assert_eq!(edge_uniformity(&img, edge, 0.1), 1.0);
    }
}

/// Small images are read in full, so the palette can only depend on the multiset of
/// pixels, not on where they are.
#[test]
fn palette_of_a_small_image_ignores_pixel_order() {
    let mut rng = Rng(31337);
    for _ in 0..40 {
        let (w, h) = (1 + rng.below(100) as u32, 1 + rng.below(100) as u32);
        let bases: Vec<Rgb8> = (0..2 + rng.below(6)).map(|_| rng.rgb()).collect();
        let mut pixels: Vec<Rgb8> = (0..w * h)
            .map(|_| {
                let b = bases[rng.below(bases.len() as u64) as usize];
                Rgb8::new(b.r ^ (rng.below(4) as u8), b.g, b.b ^ (rng.below(4) as u8))
            })
            .collect();
        let build = |pixels: &[Rgb8], w: u32, h: u32| {
            image(w, h, |x, y| opaque(pixels[(y * w + x) as usize]))
        };
        let original = palette(&build(&pixels, w, h), 5);
        // Reverse, and transpose the shape.
        pixels.reverse();
        assert_eq!(palette(&build(&pixels, w, h), 5), original);
        assert_eq!(palette(&build(&pixels, h, w), 5), original);
    }
}

// --- worst-case cost --------------------------------------------------------------------------

/// "Cost is bounded regardless of the image size": the most expensive input (the full
/// sample budget of distinct colours, the largest k) must finish in interactive time.
#[test]
fn worst_case_palette_is_fast_enough_for_a_browser() {
    let img = image(128, 128, |x, y| {
        // 16 384 distinct colours spread over the whole cube.
        let i = y * 128 + x;
        let v = i.wrapping_mul(2_654_435_761);
        [(v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8, 255]
    });
    let start = Instant::now();
    let swatches = palette(&img, usize::MAX);
    let elapsed = start.elapsed();
    assert_well_formed(&swatches, usize::MAX, "confetti");
    // Generous: this is a debug build. A release build takes a few tens of milliseconds.
    assert!(elapsed < Duration::from_secs(30), "worst-case palette took {elapsed:?}");

    // A huge, mostly empty image costs no more than a small one.
    let big = Pixmap::filled(6_000, 6_000, [12, 34, 56, 255]);
    let start = Instant::now();
    assert_eq!(palette(&big, 8), vec![Swatch { color: Rgb8::new(12, 34, 56), share: 1.0 }]);
    assert_eq!(has_uniform_border(&big), Some(Rgb8::new(12, 34, 56)));
    assert!(start.elapsed() < Duration::from_secs(10));
}

// --- accuracy of the shares ---------------------------------------------------------------------

/// The share is documented as the "share of the analysed pixels belonging to this cluster".
/// When the region is sampled on a grid, the cells of the last column/row are narrower than
/// the others, yet every sample counts the same — so whatever sits in the last few pixels
/// is over-represented. The effect is small on a whole cover (about 100 columns) but large
/// on the thin edge strips the spine designer looks at.
#[test]
fn shares_are_not_biased_towards_the_last_partial_sampling_cell() {
    // A 7 px wide, 4500 px tall strip: six white columns and one red column on the right.
    let strip =
        image(7, 4_500, |x, _| opaque(if x == 6 { Rgb8::new(200, 0, 0) } else { Rgb8::WHITE }));
    let swatches = palette(&strip, 2);
    assert_eq!(swatches.len(), 2, "{swatches:?}");
    let red = swatches.iter().find(|s| s.color != Rgb8::WHITE).expect("the red column");
    let truth = 1.0 / 7.0;
    assert!(
        (red.share - truth).abs() < 0.03,
        "one column in seven is red ({truth:.3}), reported share {:.3}",
        red.share
    );
}

/// The same bias seen through `edge_palette` on a realistic print-resolution cover: the
/// 0.3 % binding strip of a 3000 x 4500 px cover is 9 px wide; a one-pixel dark rule on its
/// inner side is 1/9 of the strip.
#[test]
fn edge_palette_share_of_a_thin_rule_in_a_print_resolution_cover() {
    let mut cover = Pixmap::filled(3_000, 4_500, [250, 246, 235, 255]);
    for y in 0..4_500 {
        cover.set(8, y, [20, 20, 60, 255]);
    }
    let swatches = edge_palette(&cover, Edge::Left, 0.003, 2);
    assert_eq!(swatches.len(), 2, "{swatches:?}");
    let rule = swatches[1];
    assert_eq!(rule.color, Rgb8::new(20, 20, 60));
    let truth = 1.0 / 9.0;
    assert!(
        (rule.share - truth).abs() < 0.03,
        "the rule is {truth:.3} of the strip, reported share {:.3}",
        rule.share
    );
}

// --- merging ---------------------------------------------------------------------------------

/// "Clusters closer than `PALETTE_MERGE_DELTA_E` are merged": two reported swatches must
/// therefore never be closer than that. A single greedy pass does not guarantee it, because
/// a kept cluster moves when a smaller one is merged into it.
#[test]
fn reported_swatches_are_never_closer_than_the_merge_threshold() {
    // Three greys: C (heaviest), A just over 5 ΔE darker (so it is kept), and X between
    // them but slightly nearer to A, which pulls A towards C when it is merged.
    let lightness = |v: u8| Rgb8::new(v, v, v).to_lab().l;
    let c = 128u8;
    let a = (0..c).rev().find(|&v| lightness(c) - lightness(v) >= PALETTE_MERGE_DELTA_E + 0.05);
    let a = a.expect("a darker grey exists");
    let x = (a..c)
        .rev()
        .find(|&v| lightness(v) - lightness(a) < lightness(c) - lightness(v))
        .expect("a grey nearer to A than to C");
    assert!(lightness(x) - lightness(a) < PALETTE_MERGE_DELTA_E);

    // 100 px of C, 99 of A, 98 of X.
    let img = image(297, 1, |i, _| {
        let v = if i < 100 {
            c
        } else if i < 199 {
            a
        } else {
            x
        };
        opaque(Rgb8::new(v, v, v))
    });
    let swatches = palette(&img, 8);
    for (i, s) in swatches.iter().enumerate() {
        for t in &swatches[i + 1..] {
            let distance = s.color.to_lab().delta_e76(t.color.to_lab());
            assert!(
                distance >= PALETTE_MERGE_DELTA_E - 0.5,
                "{} and {} are only {distance:.2} ΔE apart but listed as two swatches \
                 (greys {c}, {a}, {x}): {swatches:?}",
                s.color,
                t.color
            );
        }
    }
}

/// The same promise on random low-colour images (flat-colour covers, logos, scans of
/// typographic covers), where every colour starts as its own cluster.
#[test]
fn merged_palettes_of_low_colour_images_keep_their_distance() {
    let mut rng = Rng(0xD15_7A9CE);
    let mut violations = Vec::new();
    for round in 0..300 {
        let base = rng.rgb();
        // Up to 20 colours within a few code values of each other and of the base.
        let colours: Vec<Rgb8> = (0..3 + rng.below(18))
            .map(|_| {
                let d = |v: u8, r: &mut Rng| v.saturating_add(r.below(40) as u8);
                Rgb8::new(d(base.r, &mut rng), d(base.g, &mut rng), d(base.b, &mut rng))
            })
            .collect();
        let mut pick = Rng(rng.next() | 1);
        let img = image(40, 40, |_, _| {
            // Skewed weights: low indices are much more frequent.
            let i = (pick.below(colours.len() as u64) * pick.below(colours.len() as u64))
                / colours.len() as u64;
            opaque(colours[i as usize])
        });
        let swatches = palette(&img, 32);
        assert_well_formed(&swatches, 32, &format!("round {round}"));
        for (i, s) in swatches.iter().enumerate() {
            for t in &swatches[i + 1..] {
                let distance = s.color.to_lab().delta_e76(t.color.to_lab());
                if distance < PALETTE_MERGE_DELTA_E - 0.5 {
                    violations.push((round, s.color, t.color, distance));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "{} palettes list two swatches closer than the merge threshold, e.g. {:?}",
        violations.len(),
        &violations[..violations.len().min(3)]
    );
}

// --- uniform border ----------------------------------------------------------------------------

/// The documentation of `uniform_border_with` says: "a NaN or negative tolerance, or a NaN
/// coverage, rejects every image".
#[test]
fn nan_or_negative_tolerance_rejects_every_image_as_documented() {
    let flat = Pixmap::filled(40, 60, [40, 120, 70, 255]);
    for tolerance in [f64::NAN, -1.0, f64::NEG_INFINITY] {
        for min_coverage in [1.0, 0.97, 0.5, 0.0, -1.0, f64::NEG_INFINITY] {
            let options = BorderOptions {
                tolerance_delta_e: tolerance,
                min_coverage,
                ..BorderOptions::default()
            };
            assert_eq!(
                uniform_border_with(&flat, &options),
                None,
                "tolerance {tolerance}, min_coverage {min_coverage}"
            );
        }
    }
}

#[test]
fn border_detection_is_consistent_with_its_definition_on_framed_images() {
    // A frame of exactly the examined thickness, on images whose bands need subsampling.
    for (w, h) in [(400u32, 600u32), (1_000, 1_500), (1_240, 1_754), (1_600, 2_400)] {
        let band = (f64::from(w.min(h)) * 0.02).round() as u32;
        let mut rng = Rng(u64::from(w) * 31 + u64::from(h));
        let framed = |frame: u32, rng: &mut Rng| {
            image(w, h, |x, y| {
                let inside = x >= frame && x < w - frame && y >= frame && y < h - frame;
                opaque(if inside { rng.rgb() } else { Rgb8::new(250, 250, 245) })
            })
        };
        assert_eq!(
            has_uniform_border(&framed(band, &mut rng)),
            Some(Rgb8::new(250, 250, 245)),
            "{w}x{h}: a frame as thick as the band ({band} px) is a uniform border"
        );
        assert_eq!(
            has_uniform_border(&framed(band + 7, &mut rng)),
            Some(Rgb8::new(250, 250, 245)),
            "{w}x{h}: a thicker frame"
        );
        // Half the band is artwork: every edge is far below 97 % coverage.
        assert_eq!(has_uniform_border(&framed(band / 2, &mut rng)), None, "{w}x{h}: half frame");
        assert_eq!(has_uniform_border(&framed(0, &mut rng)), None, "{w}x{h}: full bleed");
    }
}

/// 97 % coverage is the documented threshold: a frame whose innermost pixel row/column is
/// artwork on one edge only covers (band − 1)/band of that band — below 97 % for bands
/// thinner than 34 px, so it must be rejected, and consistently so for every edge.
#[test]
fn a_frame_one_pixel_too_thin_is_rejected_on_every_edge_alike() {
    let (w, h) = (1_240u32, 1_754u32);
    let band = 25; // round(1240 x 0.02)
    let frame = Rgb8::new(250, 250, 245);
    let mut verdicts = Vec::new();
    for thin in Edge::ALL {
        let mut rng = Rng(5);
        let img = image(w, h, |x, y| {
            let depth = |edge: Edge| match edge {
                Edge::Left => x,
                Edge::Right => w - 1 - x,
                Edge::Top => y,
                Edge::Bottom => h - 1 - y,
            };
            let inside =
                Edge::ALL.iter().all(|&e| depth(e) >= if e == thin { band - 1 } else { band + 10 });
            opaque(if inside { rng.rgb() } else { frame })
        });
        verdicts.push((thin, has_uniform_border(&img)));
    }
    // 24/25 = 96 % < 97 %.
    for (edge, verdict) in &verdicts {
        assert_eq!(*verdict, None, "frame one pixel short on {edge:?}: {verdicts:?}");
    }
}

/// Mirroring a cover left-to-right cannot change whether it has a uniform border. Here the
/// frame is exactly as thick as the examined band (25 px), except that along 60 % of one
/// vertical edge the artwork reaches one pixel into the band: 1 − 0.6/25 = 97.6 % of that
/// band is frame-coloured, which is above the documented 97 % threshold — so the answer is
/// "uniform border" whichever side the intrusion is on.
#[test]
fn border_verdict_does_not_depend_on_which_side_a_blemish_is_on() {
    let (w, h) = (1_240u32, 1_754u32);
    let band = 25; // round(1240 x 0.02)
    let frame = Rgb8::new(250, 250, 245);
    let build = |mirrored: bool| {
        let mut rng = Rng(11);
        image(w, h, |x, y| {
            let x = if mirrored { w - 1 - x } else { x };
            let artwork = rng.rgb();
            let inside = (band..w - band).contains(&x) && (band..h - band).contains(&y);
            let intrusion = x == band - 1 && (300..300 + h * 6 / 10).contains(&y);
            opaque(if inside || intrusion { artwork } else { frame })
        })
    };
    let left = has_uniform_border(&build(false));
    let right = has_uniform_border(&build(true));
    assert_eq!(left, right, "intrusion on the left: {left:?}; mirrored to the right: {right:?}");
    assert_eq!(left, Some(frame), "97.6 % of the band is frame-coloured");
}
