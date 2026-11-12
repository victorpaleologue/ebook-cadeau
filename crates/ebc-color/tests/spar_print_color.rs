//! Spar review (print domain). Intended location: `crates/ebc-color/tests/spar_print_color.rs`.
use ebc_color::{Rgb8, to_cmyk_ink_safe};
use ebc_scene::Color;

/// 03-print-requirements.md §3.4 and implication 24: flat colours are cleaned — channels
/// below 2.5 % go to 0, channels from 2.5 % to 5 % go to 5 % (Pixartprinting: no tint
/// below 5 %). A cream ground today comes out with 2 % K and 4 % M.
#[test]
fn flat_colours_have_no_channel_between_0_and_5_percent() {
    for rgb in [Rgb8::new(250, 240, 220), Rgb8::new(245, 245, 250), Rgb8::new(230, 225, 210)] {
        let Color::Cmyk { c, m, y, k } = to_cmyk_ink_safe(rgb, 300.0) else { panic!() };
        for v in [c, m, y, k] {
            assert!(v == 0.0 || v >= 0.05 - 1e-6, "{rgb:?} -> {c} {m} {y} {k}");
        }
    }
}
