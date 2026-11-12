//! Spar review (print domain). Intended location: `crates/ebc-layout/tests/spar_print_layout.rs`.
use ebc_layout::CardFormat;
use ebc_scene::Size;

/// 03-print-requirements.md §6.2 and implication 6: two parallel creases never closer than
/// 5 mm; a spine narrower than that cannot be creased as two hinges.
#[test]
fn creases_closer_than_5_mm_are_refused() {
    for spine in [3.0, 4.0, 4.9] {
        assert!(
            CardFormat::custom(Size::new(110.0, 178.0), spine).is_err(),
            "spine {spine} mm accepted: creases {spine} mm apart"
        );
    }
}
