//! Behavioural tests of palette extraction and edge analysis through the public API,
//! on synthetic images whose colour statistics are known by construction.

use ebc_color::{
    BorderOptions, Edge, MAX_PALETTE_SIZE, PALETTE_MERGE_DELTA_E, Rgb8, Swatch,
    UNIFORM_EDGE_THRESHOLD, contrast_ratio, delta_e2000, edge_palette, edge_uniformity,
    has_uniform_border, palette, pick_text_color, uniform_border_with,
};
use ebc_scene::Pixmap;

const NAVY: Rgb8 = Rgb8::new(27, 42, 73);
const CREAM: Rgb8 = Rgb8::new(242, 232, 207);
const RED: Rgb8 = Rgb8::new(193, 18, 31);
const GREEN: Rgb8 = Rgb8::new(40, 120, 70);

/// Builds an opaque image from a per-pixel colour function.
fn image(w: u32, h: u32, f: impl Fn(u32, u32) -> Rgb8) -> Pixmap {
    let mut data = Vec::with_capacity(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            let c = f(x, y);
            data.extend_from_slice(&[c.r, c.g, c.b, 255]);
        }
    }
    Pixmap::from_rgba8(w, h, data).expect("consistent test image")
}

/// A stateless hash of pixel coordinates: reproducible "noise" for test images.
fn hash(x: u32, y: u32, salt: u64) -> u64 {
    let mut z = (u64::from(y) << 32 | u64::from(x)) ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    z = (z ^ (z >> 33)).wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    z ^ (z >> 33)
}

/// `base` with every channel perturbed by a uniform offset in `-amplitude..=amplitude`.
fn noisy(base: Rgb8, x: u32, y: u32, amplitude: i32) -> Rgb8 {
    let h = hash(x, y, 1);
    let span = (2 * amplitude + 1) as u64;
    let offset = |shift: u32| ((h >> shift) % span) as i32 - amplitude;
    let channel = |v: u8, shift: u32| (i32::from(v) + offset(shift)).clamp(0, 255) as u8;
    Rgb8::new(channel(base.r, 0), channel(base.g, 16), channel(base.b, 32))
}

/// A uniformly random colour per pixel: the busiest possible "photograph".
fn confetti(x: u32, y: u32) -> Rgb8 {
    let h = hash(x, y, 2);
    Rgb8::new(h as u8, (h >> 8) as u8, (h >> 16) as u8)
}

fn share_of(swatches: &[Swatch], color: Rgb8) -> f32 {
    swatches.iter().filter(|s| s.color == color).map(|s| s.share).sum()
}

fn assert_well_formed(swatches: &[Swatch]) {
    let total: f32 = swatches.iter().map(|s| s.share).sum();
    assert!((total - 1.0).abs() < 1e-4, "shares add up to {total}");
    for pair in swatches.windows(2) {
        assert!(pair[0].share >= pair[1].share, "not sorted by share: {swatches:?}");
        assert_ne!(pair[0].color, pair[1].color);
    }
    assert!(swatches.iter().all(|s| s.share > 0.0 && s.share <= 1.0));
}

// --- palette -----------------------------------------------------------------------------

#[test]
fn flat_image_yields_one_exact_swatch_whatever_k() {
    for color in [NAVY, CREAM, Rgb8::BLACK, Rgb8::WHITE, Rgb8::new(1, 254, 3)] {
        let img = image(37, 53, |_, _| color);
        for k in [1, 2, 5, MAX_PALETTE_SIZE, usize::MAX] {
            assert_eq!(palette(&img, k), vec![Swatch { color, share: 1.0 }], "k = {k}");
        }
    }
}

#[test]
fn k_zero_yields_an_empty_palette() {
    let img = image(8, 8, confetti);
    assert!(palette(&img, 0).is_empty());
    assert!(edge_palette(&img, Edge::Left, 0.5, 0).is_empty());
}

#[test]
fn flat_regions_give_exact_colours_and_exact_shares() {
    // 40 x 40: 30 columns red, 10 columns navy.
    let img = image(40, 40, |x, _| if x < 30 { RED } else { NAVY });
    let expected = vec![Swatch { color: RED, share: 0.75 }, Swatch { color: NAVY, share: 0.25 }];
    assert_eq!(palette(&img, 2), expected);
    assert_eq!(palette(&img, 7), expected, "asking for more colours invents none");

    // k = 1 is the (CIELAB) mean colour: between the two, closer to the majority.
    let one = palette(&img, 1);
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].share, 1.0);
    let mean = one[0].color;
    assert!(delta_e2000(mean, RED) < delta_e2000(mean, NAVY));
    assert!(delta_e2000(mean, RED) > 5.0 && delta_e2000(mean, NAVY) > 5.0);
}

#[test]
fn equal_shares_are_ordered_by_colour() {
    let img = image(10, 10, |_, y| if y < 5 { RED } else { NAVY });
    let swatches = palette(&img, 4);
    assert_eq!(
        swatches,
        vec![Swatch { color: NAVY, share: 0.5 }, Swatch { color: RED, share: 0.5 }],
        "NAVY < RED in (r, g, b) order"
    );
}

#[test]
fn large_images_are_sampled_with_accurate_shares() {
    // 1200 x 1800 px in four horizontal bands: 50 % / 30 % / 15 % / 5 %.
    let img = image(1200, 1800, |_, y| match y {
        0..900 => NAVY,
        900..1440 => CREAM,
        1440..1710 => RED,
        _ => Rgb8::BLACK,
    });
    let swatches = palette(&img, 4);
    assert_well_formed(&swatches);
    let colors: Vec<Rgb8> = swatches.iter().map(|s| s.color).collect();
    assert_eq!(colors, vec![NAVY, CREAM, RED, Rgb8::BLACK], "exact colours, by share");
    for (swatch, expected) in swatches.iter().zip([0.50, 0.30, 0.15, 0.05]) {
        assert!((swatch.share - expected).abs() < 0.01, "{swatch:?} vs {expected}");
    }
}

#[test]
fn one_pixel_stripes_do_not_alias() {
    // A regular sampling stride of 6 px would only ever see the even columns.
    let img = image(1024, 512, |x, _| if x % 2 == 0 { NAVY } else { CREAM });
    let swatches = palette(&img, 2);
    assert_eq!(swatches.len(), 2);
    assert!((share_of(&swatches, NAVY) - 0.5).abs() < 0.03, "{swatches:?}");
    assert!((share_of(&swatches, CREAM) - 0.5).abs() < 0.03, "{swatches:?}");
    // Same along the other axis.
    let img = image(512, 1024, |_, y| if y % 2 == 0 { NAVY } else { CREAM });
    let swatches = palette(&img, 2);
    assert!((share_of(&swatches, NAVY) - 0.5).abs() < 0.03, "{swatches:?}");
}

#[test]
fn noisy_regions_are_recovered_close_to_their_true_colours() {
    // 60 % navy, 30 % cream, 10 % red, each pixel perturbed by up to ±6 per channel.
    let truth = [(NAVY, 0.60_f32), (CREAM, 0.30), (RED, 0.10)];
    let img = image(256, 256, |x, y| {
        let base = match (x * 100) / 256 {
            0..60 => NAVY,
            60..90 => CREAM,
            _ => RED,
        };
        noisy(base, x, y, 6)
    });
    let swatches = palette(&img, 3);
    assert_well_formed(&swatches);
    assert_eq!(swatches.len(), 3);
    for (swatch, (color, share)) in swatches.iter().zip(truth) {
        let de = delta_e2000(swatch.color, color);
        assert!(de < 1.5, "{swatch:?} is {de:.2} ΔE00 from {color}");
        assert!((swatch.share - share).abs() < 0.02, "{swatch:?} vs {share}");
    }
}

#[test]
fn indistinguishable_clusters_are_merged() {
    // Two flat colours with ±1 noise: asking for six colours must still report two.
    let img = image(200, 100, |x, y| noisy(if x < 140 { GREEN } else { CREAM }, x, y, 1));
    let swatches = palette(&img, 6);
    assert_well_formed(&swatches);
    assert_eq!(swatches.len(), 2, "{swatches:?}");
    assert!(delta_e2000(swatches[0].color, GREEN) < 1.0);
    assert!(delta_e2000(swatches[1].color, CREAM) < 1.0);
    assert!((swatches[0].share - 0.7).abs() < 0.02);
    // And no two reported swatches are ever closer than the merge threshold.
    let busy = image(160, 160, confetti);
    let many = palette(&busy, 24);
    for (i, a) in many.iter().enumerate() {
        for b in &many[i + 1..] {
            let distance = a.color.to_lab().delta_e76(b.color.to_lab());
            // Centres are rounded to 8 bits after merging, hence the small slack.
            assert!(distance > PALETTE_MERGE_DELTA_E - 1.0, "{a:?} / {b:?}: {distance}");
        }
    }
}

#[test]
fn grey_ramp_is_split_into_ordered_grey_steps() {
    let img = image(256, 64, |x, _| Rgb8::new(x as u8, x as u8, x as u8));
    let swatches = palette(&img, 4);
    assert_well_formed(&swatches);
    assert_eq!(swatches.len(), 4);
    for s in &swatches {
        assert!(s.color.r == s.color.g && s.color.g == s.color.b, "not neutral: {s:?}");
        assert!((0.12..0.40).contains(&s.share), "{s:?}");
    }
    let mut levels: Vec<u8> = swatches.iter().map(|s| s.color.r).collect();
    levels.sort_unstable();
    // One step in each quarter of the ramp, roughly.
    assert!(levels[0] < 64 && levels[1] < 140 && levels[2] > 100 && levels[3] > 190, "{levels:?}");
}

#[test]
fn hostile_k_is_clamped_and_terminates() {
    let img = image(64, 64, confetti); // 4096 distinct colours
    let swatches = palette(&img, usize::MAX);
    assert!(!swatches.is_empty() && swatches.len() <= MAX_PALETTE_SIZE, "{}", swatches.len());
    assert_well_formed(&swatches);
    assert_eq!(swatches, palette(&img, MAX_PALETTE_SIZE));
}

#[test]
fn output_is_deterministic_and_pinned() {
    // A cover-like image: noisy background, a title band, an accent block, a gradient foot.
    let cover = |x: u32, y: u32| -> Rgb8 {
        if (60..120).contains(&y) && (30..270).contains(&x) {
            noisy(CREAM, x, y, 3)
        } else if (300..330).contains(&y) && (120..180).contains(&x) {
            noisy(RED, x, y, 3)
        } else if y >= 400 {
            let t = (y - 400) as u8;
            noisy(Rgb8::new(27 + t, 42 + t, 73 + t), x, y, 2)
        } else {
            noisy(NAVY, x, y, 4)
        }
    };
    let img = image(300, 450, cover);
    let first = palette(&img, 5);
    assert_eq!(first, palette(&img, 5), "same input, same output");
    assert_eq!(first, palette(&img.clone(), 5));
    assert_well_formed(&first);

    // Regression pin: these values must not change across platforms, compilers or
    // dependency updates. If an intentional algorithm change alters them, update the pin
    // and mention it in the changelog — every generated spine colour moves with it.
    let pinned: Vec<(Rgb8, u32)> =
        first.iter().map(|s| (s.color, (s.share * 10_000.0).round() as u32)).collect();
    assert_eq!(pinned, PINNED_COVER_PALETTE.to_vec());
}

/// Expected output of `output_is_deterministic_and_pinned`: colour and share in 1/10 000.
const PINNED_COVER_PALETTE: [(Rgb8, u32); 4] = [
    (Rgb8::new(28, 42, 73), 8088), // navy ground plus the dark end of the foot gradient
    (Rgb8::new(242, 232, 207), 1067), // title band: 60 x 240 px of 300 x 450 = 10.67 %
    (Rgb8::new(60, 75, 106), 712), // the light end of the foot gradient
    (Rgb8::new(193, 18, 31), 133), // accent block: 30 x 60 px = 1.33 %
];

#[test]
fn transparency_is_composited_over_paper_white() {
    let clear = Pixmap::filled(9, 9, [200, 0, 0, 0]);
    assert_eq!(palette(&clear, 3), vec![Swatch { color: Rgb8::WHITE, share: 1.0 }]);

    let half = Pixmap::filled(9, 9, [0, 0, 0, 128]);
    assert_eq!(palette(&half, 3), vec![Swatch { color: Rgb8::new(127, 127, 127), share: 1.0 }]);

    // A logo-like image: opaque red disc on a transparent ground.
    let mut logo = Pixmap::filled(64, 64, [0, 0, 0, 0]);
    for y in 0..64u32 {
        for x in 0..64u32 {
            let (dx, dy) = (x as i32 - 32, y as i32 - 32);
            if dx * dx + dy * dy < 20 * 20 {
                logo.set(x, y, [RED.r, RED.g, RED.b, 255]);
            }
        }
    }
    let swatches = palette(&logo, 2);
    assert_eq!(swatches.len(), 2);
    assert_eq!(swatches[0].color, Rgb8::WHITE);
    assert_eq!(swatches[1].color, RED);
    let disc = std::f32::consts::PI * 400.0 / 4096.0;
    assert!((swatches[1].share - disc).abs() < 0.01);
}

#[test]
fn extreme_aspect_ratios_and_single_pixels_work() {
    assert_eq!(palette(&image(1, 1, |_, _| RED), 3), vec![Swatch { color: RED, share: 1.0 }]);
    let column = image(1, 5000, |_, y| if y < 1000 { RED } else { NAVY });
    let swatches = palette(&column, 2);
    assert_eq!(swatches[0].color, NAVY);
    assert!((swatches[0].share - 0.8).abs() < 0.01, "{swatches:?}");
    let row = image(5000, 1, |x, _| if x < 1000 { RED } else { NAVY });
    assert!((share_of(&palette(&row, 2), RED) - 0.2).abs() < 0.01);
}

// --- edge_palette ------------------------------------------------------------------------

#[test]
fn edge_palette_only_looks_at_the_requested_strip() {
    // 200 x 300: a 20 px navy binding strip on the left, confetti elsewhere, and a 30 px
    // cream band along the bottom (which also covers the bottom of the navy strip).
    let img = image(200, 300, |x, y| {
        if y >= 270 {
            CREAM
        } else if x < 20 {
            NAVY
        } else {
            confetti(x, y)
        }
    });
    assert_eq!(edge_palette(&img, Edge::Bottom, 0.1, 3), vec![Swatch { color: CREAM, share: 1.0 }]);
    let left = edge_palette(&img, Edge::Left, 0.1, 3);
    assert_eq!(left.len(), 2);
    assert_eq!(left[0], Swatch { color: NAVY, share: 0.9 });
    assert_eq!(left[1], Swatch { color: CREAM, share: 0.1 });

    // The right strip sees no navy at all.
    let right = edge_palette(&img, Edge::Right, 0.1, 8);
    assert!(right.iter().all(|s| delta_e2000(s.color, NAVY) > 3.0), "{right:?}");

    // The top strip of a flat two-tone cover sees the binding strip in proportion.
    let flat = image(200, 300, |x, _| if x < 20 { NAVY } else { GREEN });
    assert_eq!(
        edge_palette(&flat, Edge::Top, 0.1, 8),
        vec![Swatch { color: GREEN, share: 0.9 }, Swatch { color: NAVY, share: 0.1 }]
    );
    assert_eq!(
        edge_palette(&flat, Edge::Right, 0.9, 8),
        vec![Swatch { color: GREEN, share: 1.0 }],
        "the right 90 % stops exactly where the navy strip begins"
    );

    // A full-size strip is the whole image.
    assert_eq!(edge_palette(&img, Edge::Top, 1.0, 5), palette(&img, 5));
    assert_eq!(edge_palette(&img, Edge::Left, 1.0, 5), palette(&img, 5));
}

#[test]
fn edge_functions_tolerate_hostile_strip_fractions() {
    let img = image(50, 80, |x, _| if x == 0 { RED } else { NAVY });
    for fraction in [f32::NAN, -5.0, 0.0, f32::NEG_INFINITY, 1e-30] {
        // Degenerates to the single outermost pixel column.
        assert_eq!(
            edge_palette(&img, Edge::Left, fraction, 4),
            vec![Swatch { color: RED, share: 1.0 }],
            "{fraction}"
        );
        assert_eq!(edge_uniformity(&img, Edge::Left, fraction), 1.0);
    }
    for fraction in [1.0, 3.0, f32::INFINITY, f32::MAX] {
        assert_eq!(edge_palette(&img, Edge::Left, fraction, 4), palette(&img, 4), "{fraction}");
        let u = edge_uniformity(&img, Edge::Right, fraction);
        assert!((0.0..=1.0).contains(&u));
    }
}

// --- edge_uniformity ---------------------------------------------------------------------

#[test]
fn uniformity_scale_anchors() {
    let flat = image(120, 180, |_, _| GREEN);
    let busy = image(120, 180, confetti);
    let split = image(120, 180, |_, y| if y < 90 { Rgb8::BLACK } else { Rgb8::WHITE });
    for edge in Edge::ALL {
        assert_eq!(edge_uniformity(&flat, edge, 0.1), 1.0, "{edge:?}");
        assert_eq!(edge_uniformity(&busy, edge, 0.1), 0.0, "{edge:?}");
    }
    // Left/right strips of the split image contain both halves: σ = 50 ΔE → 0.
    assert_eq!(edge_uniformity(&split, Edge::Left, 0.1), 0.0);
    // Its top and bottom strips are flat.
    assert_eq!(edge_uniformity(&split, Edge::Top, 0.1), 1.0);
    assert_eq!(edge_uniformity(&split, Edge::Bottom, 0.4), 1.0);
    assert!(edge_uniformity(&split, Edge::Bottom, 0.9) < 0.1);
}

#[test]
fn uniformity_decreases_with_noise_and_separates_flat_from_textured() {
    let uniformity = |amplitude: i32| {
        let img = image(100, 300, |x, y| noisy(Rgb8::new(128, 110, 96), x, y, amplitude));
        edge_uniformity(&img, Edge::Left, 0.15)
    };
    let amplitudes = [0, 1, 2, 4, 8, 16, 24, 40, 127];
    let values: Vec<f32> = amplitudes.iter().map(|&a| uniformity(a)).collect();
    for (pair, amplitude) in values.windows(2).zip(&amplitudes[1..]) {
        assert!(
            pair[1] < pair[0] || (pair[0] == 0.0 && pair[1] == 0.0),
            "amplitude {amplitude}: {values:?}"
        );
    }
    assert_eq!(values[0], 1.0);
    assert!(values[2] >= UNIFORM_EDGE_THRESHOLD, "±2 (JPEG-like) must count as flat: {values:?}");
    assert!(values[6] < UNIFORM_EDGE_THRESHOLD, "±24 is visible texture: {values:?}");
    assert_eq!(values[8], 0.0, "full-range noise is as busy as it gets: {values:?}");
}

#[test]
fn uniformity_of_gradients_and_partial_strips() {
    // A gentle vertical gradient is neither flat nor busy.
    let gradient = image(60, 256, |_, y| {
        let v = 90 + (y / 4) as u8; // 90..=153
        Rgb8::new(v, v, v)
    });
    let u = edge_uniformity(&gradient, Edge::Left, 0.2);
    assert!((0.5..UNIFORM_EDGE_THRESHOLD).contains(&u), "{u}");
    // …but across the gradient (a thin top strip) it is flat.
    assert!(edge_uniformity(&gradient, Edge::Top, 0.02) >= UNIFORM_EDGE_THRESHOLD);

    // Only the strip matters: flat binding edge, busy artwork.
    let cover = image(200, 300, |x, y| if x < 30 { NAVY } else { confetti(x, y) });
    assert_eq!(edge_uniformity(&cover, Edge::Left, 0.15), 1.0);
    assert!(edge_uniformity(&cover, Edge::Left, 0.5) < 0.3);
    assert_eq!(edge_uniformity(&cover, Edge::Right, 0.15), 0.0);
}

// --- has_uniform_border ------------------------------------------------------------------

/// A 400 x 600 cover with a `frame`-pixel border of `color` around confetti artwork.
fn framed(frame: u32, color: Rgb8, noise: i32) -> Pixmap {
    image(400, 600, |x, y| {
        let inside = x >= frame && x < 400 - frame && y >= frame && y < 600 - frame;
        if inside { confetti(x, y) } else { noisy(color, x, y, noise) }
    })
}

#[test]
fn detects_white_and_coloured_frames() {
    assert_eq!(has_uniform_border(&framed(20, Rgb8::WHITE, 0)), Some(Rgb8::WHITE));
    assert_eq!(has_uniform_border(&framed(20, NAVY, 0)), Some(NAVY));
    assert_eq!(has_uniform_border(&framed(8, CREAM, 0)), Some(CREAM), "frame = band width");
    // With compression-like noise the colour is recovered approximately.
    let found = has_uniform_border(&framed(20, CREAM, 3)).expect("noisy frame is still a frame");
    assert!(delta_e2000(found, CREAM) < 1.0, "{found}");
    let found = has_uniform_border(&framed(20, Rgb8::WHITE, 2)).expect("off-white frame");
    assert!(delta_e2000(found, Rgb8::WHITE) < 1.5, "{found}");
}

#[test]
fn rejects_full_bleed_artwork_and_partial_frames() {
    assert_eq!(has_uniform_border(&image(400, 600, confetti)), None);
    // Two-tone cover: each edge is flat on its own, but they disagree.
    let split = image(400, 600, |x, _| if x < 200 { Rgb8::BLACK } else { Rgb8::WHITE });
    assert_eq!(has_uniform_border(&split), None);
    // A frame thinner than the examined band (2 px < 8 px) does not count.
    assert_eq!(has_uniform_border(&framed(2, Rgb8::WHITE, 0)), None);
    // A frame missing on any one side does not count.
    for missing in Edge::ALL {
        let img = image(400, 600, |x, y| {
            let on = |edge: Edge| match edge {
                Edge::Left => x < 20,
                Edge::Right => x >= 380,
                Edge::Top => y < 20,
                Edge::Bottom => y >= 580,
            };
            if Edge::ALL.iter().any(|&e| e != missing && on(e)) && !on(missing) {
                Rgb8::WHITE
            } else if on(missing) {
                confetti(x, y)
            } else {
                GREEN
            }
        });
        assert_eq!(has_uniform_border(&img), None, "frame missing on {missing:?}");
    }
}

#[test]
fn flat_and_typographic_covers_have_a_uniform_border() {
    assert_eq!(has_uniform_border(&image(300, 450, |_, _| GREEN)), Some(GREEN));
    assert_eq!(has_uniform_border(&image(1, 1, |_, _| RED)), Some(RED));
    assert_eq!(has_uniform_border(&image(1, 40, |_, _| RED)), Some(RED));
    assert_eq!(has_uniform_border(&image(1, 40, |_, y| if y < 20 { RED } else { NAVY })), None);
    // A cream cover with thin red rules well inside the rim and a title block.
    let classic = image(300, 450, |x, y| {
        let rule = |d: u32| {
            let on_x = (x == d || x == 299 - d) && (d..=449 - d).contains(&y);
            let on_y = (y == d || y == 449 - d) && (d..=299 - d).contains(&x);
            on_x || on_y
        };
        if rule(18) || rule(22) {
            RED
        } else if (150..200).contains(&y) && (60..240).contains(&x) {
            Rgb8::BLACK
        } else {
            CREAM
        }
    });
    assert_eq!(has_uniform_border(&classic), Some(CREAM));
}

#[test]
fn small_blemishes_are_tolerated_large_ones_are_not() {
    // Band is 8 px (2 % of 400); the top band has 400 x 8 = 3200 px.
    let with_speck = |speck_w: u32| {
        image(400, 600, |x, y| {
            let inside = (20..380).contains(&x) && (20..580).contains(&y);
            if inside {
                confetti(x, y)
            } else if y < 8 && (100..100 + speck_w).contains(&x) {
                Rgb8::BLACK
            } else {
                Rgb8::WHITE
            }
        })
    };
    assert_eq!(has_uniform_border(&with_speck(8)), Some(Rgb8::WHITE), "2 % of the top edge");
    assert_eq!(has_uniform_border(&with_speck(40)), None, "10 % of the top edge");
}

#[test]
fn border_options_are_honoured_and_hostile_values_are_safe() {
    let thin = framed(2, Rgb8::WHITE, 0);
    let narrow = BorderOptions { band_fraction: 0.005, ..BorderOptions::default() }; // 2 px
    assert_eq!(uniform_border_with(&thin, &narrow), Some(Rgb8::WHITE));

    let noisy_frame = framed(20, CREAM, 6);
    let strict = BorderOptions { tolerance_delta_e: 0.5, ..BorderOptions::default() };
    assert_eq!(uniform_border_with(&noisy_frame, &strict), None);
    let lax = BorderOptions { tolerance_delta_e: 12.0, ..BorderOptions::default() };
    assert!(uniform_border_with(&noisy_frame, &lax).is_some());

    // min_coverage 0 accepts anything and returns the median rim colour.
    let anything = BorderOptions { min_coverage: 0.0, ..BorderOptions::default() };
    assert!(uniform_border_with(&image(40, 60, confetti), &anything).is_some());

    let flat = image(40, 60, |_, _| GREEN);
    let exact = BorderOptions { tolerance_delta_e: 0.0, min_coverage: 1.0, band_fraction: 0.5 };
    assert_eq!(uniform_border_with(&flat, &exact), Some(GREEN));
    for bad in [f32::NAN, -1.0, 0.0, 9.0, f32::INFINITY] {
        let options = BorderOptions { band_fraction: bad, ..BorderOptions::default() };
        assert_eq!(uniform_border_with(&flat, &options), Some(GREEN), "band_fraction {bad}");
    }
    for bad in [f64::NAN, -1.0, f64::NEG_INFINITY] {
        let options = BorderOptions { tolerance_delta_e: bad, ..BorderOptions::default() };
        assert_eq!(uniform_border_with(&flat, &options), None, "tolerance {bad}");
    }
    let options =
        BorderOptions { tolerance_delta_e: f64::INFINITY, min_coverage: 1.0, band_fraction: 0.1 };
    assert!(uniform_border_with(&image(40, 60, confetti), &options).is_some());
    let options = BorderOptions { min_coverage: f64::NAN, ..BorderOptions::default() };
    assert_eq!(uniform_border_with(&flat, &options), None);
    let options = BorderOptions { min_coverage: 2.0, ..BorderOptions::default() };
    assert_eq!(uniform_border_with(&flat, &options), None, "more than 100 % is unreachable");
}

// --- the scenario the spine designer runs -------------------------------------------------

#[test]
fn spine_colours_for_a_typical_cover() {
    // Navy cover, cream title block, small red publisher mark; mild compression noise.
    let cover = image(360, 540, |x, y| {
        let base = if (90..170).contains(&y) && (40..320).contains(&x) {
            CREAM
        } else if (470..500).contains(&y) && (160..200).contains(&x) {
            RED
        } else {
            NAVY
        };
        noisy(base, x, y, 2)
    });

    let colours = palette(&cover, 5);
    assert_eq!(colours.len(), 3, "{colours:?}");
    assert!(delta_e2000(colours[0].color, NAVY) < 1.0);
    assert!(delta_e2000(colours[1].color, CREAM) < 1.0);
    assert!(delta_e2000(colours[2].color, RED) < 1.0);

    // The binding edge is flat navy: continue it onto the spine…
    assert!(edge_uniformity(&cover, Edge::Left, 0.05) >= UNIFORM_EDGE_THRESHOLD);
    let background = edge_palette(&cover, Edge::Left, 0.05, 1)[0].color;
    assert!(delta_e2000(background, NAVY) < 1.0);
    assert_eq!(has_uniform_border(&cover).map(|c| delta_e2000(c, NAVY) < 1.0), Some(true));

    // …and set the title in the cover's own title colour: cream passes AA on navy, the
    // red mark does not and is skipped even when listed first.
    let candidates: Vec<Rgb8> = colours[1..].iter().map(|s| s.color).collect();
    let text = pick_text_color(background, &candidates, 4.5);
    assert_eq!(text, colours[1].color);
    assert!(contrast_ratio(text, background) >= 4.5);
    let reversed: Vec<Rgb8> = candidates.iter().rev().copied().collect();
    assert_eq!(pick_text_color(background, &reversed, 4.5), colours[1].color);
}
