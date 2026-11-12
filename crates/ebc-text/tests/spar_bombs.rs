//! Sparring tests: tiny hostile fonts that make the crate burn unbounded CPU or memory.
//!
//! The crate documents font files as hostile input ("Untrusted fonts" in the crate docs)
//! and contains *panics*. These tests attack the other half of that promise: a font of a
//! few hundred bytes must not be able to hang `Font::from_bytes`, `to_outline`,
//! `ink_bounds` or `rasterize`, nor make them emit millions of path segments.
//!
//! The work each test asks for is finite (the helper threads terminate on their own), but
//! it grows exponentially with a parameter a font author controls, so the same fonts with
//! a larger fan-out never finish at all.
#![allow(missing_docs)]

mod common;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ebc_scene::Point;
use ebc_text::{Font, ShapeOptions, shape};

fn push_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn push_i16(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Runs `f` on a helper thread and returns its result, or `None` if it is not done after
/// `limit`. (The helper thread is left to finish on its own.)
fn within<T: Send + 'static>(limit: Duration, f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(limit).ok()
}

fn head(units_per_em: u16, long_loca: bool) -> Vec<u8> {
    let mut head = Vec::new();
    push_u32(&mut head, 0x0001_0000);
    push_u32(&mut head, 0x0001_0000);
    push_u32(&mut head, 0);
    push_u32(&mut head, 0x5F0F_3CF5);
    push_u16(&mut head, 0);
    push_u16(&mut head, units_per_em);
    head.extend_from_slice(&[0; 16]);
    for v in [0i16, -200, 700, 800] {
        push_i16(&mut head, v);
    }
    push_u16(&mut head, 0);
    push_u16(&mut head, 8);
    push_i16(&mut head, 2);
    push_i16(&mut head, i16::from(long_loca));
    push_i16(&mut head, 0);
    head
}

fn hhea(glyph_count: u16) -> Vec<u8> {
    let mut hhea = Vec::new();
    push_u32(&mut hhea, 0x0001_0000);
    push_i16(&mut hhea, 800);
    push_i16(&mut hhea, -200);
    push_i16(&mut hhea, 0);
    push_u16(&mut hhea, 600);
    hhea.extend_from_slice(&[0; 22]);
    push_u16(&mut hhea, glyph_count);
    hhea
}

fn hmtx(glyph_count: u16) -> Vec<u8> {
    let mut hmtx = Vec::new();
    for _ in 0..glyph_count {
        push_u16(&mut hmtx, 600);
        push_i16(&mut hmtx, 0);
    }
    hmtx
}

/// A format 0 `cmap` mapping "H" to glyph 1 and the space to glyph `space`.
fn cmap(space: u8) -> Vec<u8> {
    let mut cmap = Vec::new();
    push_u16(&mut cmap, 0);
    push_u16(&mut cmap, 1);
    push_u16(&mut cmap, 3);
    push_u16(&mut cmap, 1);
    push_u32(&mut cmap, 12);
    push_u16(&mut cmap, 0);
    push_u16(&mut cmap, 262);
    push_u16(&mut cmap, 0);
    let mut map = [0u8; 256];
    map[b'H' as usize] = 1;
    map[b' ' as usize] = space;
    cmap.extend_from_slice(&map);
    cmap
}

/// An `OS/2` table, version 4, `fsType` 0, with a capital height — so that loading the
/// font does not need to look at any outline.
fn os2_with_cap_height() -> Vec<u8> {
    let mut os2 = vec![0u8; 96];
    os2[0..2].copy_from_slice(&4u16.to_be_bytes());
    os2[4..6].copy_from_slice(&400u16.to_be_bytes());
    os2[86..88].copy_from_slice(&500i16.to_be_bytes());
    os2[88..90].copy_from_slice(&700i16.to_be_bytes());
    os2
}

/// A TrueType font whose glyph 1 ("H") is a composite of `fan_out` copies of glyph 2,
/// which is a composite of `fan_out` copies of glyph 3, … `levels` composites deep. The
/// last glyph is a plain rectangle (`solid_leaf`) or an empty glyph.
///
/// Every component is legal on its own; the glyph simply stands for
/// `fan_out ^ levels` copies of the leaf. `levels` must stay below ttf-parser's nesting
/// limit of 32.
fn composite_bomb(levels: u16, fan_out: u16, solid_leaf: bool, with_os2: bool) -> Vec<u8> {
    common::build_sfnt(composite_bomb_tables(levels, fan_out, solid_leaf, with_os2))
}

/// The tables of [`composite_bomb`], for tests that add some of their own.
fn composite_bomb_tables(
    levels: u16,
    fan_out: u16,
    solid_leaf: bool,
    with_os2: bool,
) -> Vec<(&'static [u8; 4], Vec<u8>)> {
    assert!(levels < 31);
    let glyph_count = levels + 3; // .notdef, `levels` composites, leaf, space
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    let end_glyph = |glyf: &mut Vec<u8>, loca: &mut Vec<u8>| {
        while !glyf.len().is_multiple_of(4) {
            glyf.push(0);
        }
        push_u32(loca, glyf.len() as u32);
    };
    push_u32(&mut loca, 0);
    end_glyph(&mut glyf, &mut loca); // glyph 0: empty
    for level in 0..levels {
        push_i16(&mut glyf, -1); // composite
        for v in [100i16, 0, 500, 700] {
            push_i16(&mut glyf, v);
        }
        let child = level + 2;
        for k in 0..fan_out {
            let more = if k + 1 < fan_out { 0x0020 } else { 0 };
            push_u16(&mut glyf, 0x0002 | more); // ARGS_ARE_XY_VALUES, byte arguments
            push_u16(&mut glyf, child);
            glyf.extend_from_slice(&[0, 0]); // dx, dy
        }
        end_glyph(&mut glyf, &mut loca);
    }
    if solid_leaf {
        push_i16(&mut glyf, 1);
        for v in [100i16, 0, 500, 700] {
            push_i16(&mut glyf, v);
        }
        push_u16(&mut glyf, 3);
        push_u16(&mut glyf, 0);
        glyf.extend_from_slice(&[0x01; 4]);
        for dx in [100i16, 0, 400, 0] {
            push_i16(&mut glyf, dx);
        }
        for dy in [0i16, 700, 0, -700] {
            push_i16(&mut glyf, dy);
        }
    }
    end_glyph(&mut glyf, &mut loca); // the leaf
    end_glyph(&mut glyf, &mut loca); // the space

    let mut maxp = Vec::new();
    push_u32(&mut maxp, 0x0001_0000);
    push_u16(&mut maxp, glyph_count);
    maxp.extend_from_slice(&[0; 26]);

    let mut tables: Vec<(&'static [u8; 4], Vec<u8>)> = vec![
        (b"cmap", cmap((glyph_count - 1) as u8)),
        (b"glyf", glyf),
        (b"head", head(1000, true)),
        (b"hhea", hhea(glyph_count)),
        (b"hmtx", hmtx(glyph_count)),
        (b"loca", loca),
        (b"maxp", maxp),
    ];
    if with_os2 {
        tables.push((b"OS/2", os2_with_cap_height()));
    }
    tables
}

/// A CFF `INDEX` with two-byte offsets.
fn cff_index(items: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    push_u16(&mut out, items.len() as u16);
    if items.is_empty() {
        return out;
    }
    out.push(2);
    let mut offset = 1usize;
    push_u16(&mut out, offset as u16);
    for item in items {
        offset += item.len();
        push_u16(&mut out, u16::try_from(offset).expect("INDEX fits two-byte offsets"));
    }
    for item in items {
        out.extend_from_slice(item);
    }
    out
}

/// An OpenType/CFF font whose glyph 1 ("H") calls global subroutine 0 `fan_out` times,
/// which calls subroutine 1 `fan_out` times, … `levels` subroutines deep (the Type 2
/// nesting limit is 10). The innermost subroutine does nothing.
///
/// The font has no `OS/2` table, like `common::synthetic_cff_font`, unless `with_os2`.
fn cff_subroutine_bomb(levels: u8, fan_out: usize, with_os2: bool) -> Vec<u8> {
    const CALLGSUBR: u8 = 29;
    const RETURN: u8 = 11;
    const ENDCHAR: u8 = 14;
    assert!((1..=10).contains(&levels));
    // With fewer than 1240 subroutines the index bias is 107; operands in -107..=107 are
    // one byte: value + 139.
    let call = |index: u8| [index + 32, CALLGSUBR]; // (index - 107) + 139
    let mut subrs = Vec::new();
    for level in 0..levels {
        let mut body = Vec::new();
        if level + 1 < levels {
            for _ in 0..fan_out {
                body.extend_from_slice(&call(level + 1));
            }
        }
        body.push(RETURN);
        subrs.push(body);
    }
    let mut glyph = Vec::new();
    for _ in 0..fan_out {
        glyph.extend_from_slice(&call(0));
    }
    glyph.push(ENDCHAR);
    let char_strings = cff_index(&[vec![ENDCHAR], glyph, vec![ENDCHAR]]);
    let global_subrs = cff_index(&subrs);

    let mut cff = vec![1, 0, 4, 1];
    cff.extend_from_slice(&cff_index(&[b"Syn".to_vec()]));
    let top_dict_index_len = 2 + 1 + 2 * 2 + 6;
    let char_strings_offset = cff.len() + top_dict_index_len + 2 + global_subrs.len();
    let mut top_dict = vec![29];
    top_dict.extend_from_slice(&(char_strings_offset as u32).to_be_bytes());
    top_dict.push(17); // CharStrings
    cff.extend_from_slice(&cff_index(&[top_dict]));
    cff.extend_from_slice(&cff_index(&[])); // strings
    cff.extend_from_slice(&global_subrs);
    assert_eq!(cff.len(), char_strings_offset);
    cff.extend_from_slice(&char_strings);

    let mut maxp = Vec::new();
    push_u32(&mut maxp, 0x0000_5000);
    push_u16(&mut maxp, 3);

    let mut tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"CFF ", cff),
        (b"cmap", cmap(2)),
        (b"head", head(1000, false)),
        (b"hhea", hhea(3)),
        (b"hmtx", hmtx(3)),
        (b"maxp", maxp),
    ];
    if with_os2 {
        tables.push((b"OS/2", os2_with_cap_height()));
    }
    let mut font = common::build_sfnt(tables);
    font[..4].copy_from_slice(b"OTTO");
    font
}

// ---------------------------------------------------------------------------------------
// Sanity: the generators produce fonts the crate accepts and draws correctly when small.
// ---------------------------------------------------------------------------------------

#[test]
fn small_composite_fonts_work_and_draw_every_component() {
    // 3 levels, fan-out 2: "H" is 8 copies of the rectangle.
    let font = Font::from_bytes(composite_bomb(3, 2, true, true), 0).expect("parses");
    assert_eq!(font.glyph_id('H'), Some(1));
    let run = shape(&font, "H", 72.0, &ShapeOptions::default());
    assert_eq!(run.glyphs.len(), 1);
    let path = run.to_outline(&font, Point::new(0.0, 0.0));
    let single = Font::from_bytes(composite_bomb(1, 1, true, true), 0).expect("parses");
    let one_rectangle = shape(&single, "H", 72.0, &ShapeOptions::default())
        .to_outline(&single, Point::new(0.0, 0.0))
        .segs
        .len();
    assert!(one_rectangle >= 5, "move, lines, close");
    assert_eq!(path.segs.len(), 8 * one_rectangle, "8 copies of the rectangle");
    let ink = run.ink_bounds(&font, Point::new(0.0, 0.0)).expect("ink");
    assert!(common::close(ink.w, 0.4 * 25.4, 1e-9) && common::close(ink.h, 0.7 * 25.4, 1e-9));
}

#[test]
fn small_cff_subroutine_fonts_load() {
    let font = Font::from_bytes(cff_subroutine_bomb(3, 2, false), 0).expect("parses");
    assert_eq!(font.glyph_id('H'), Some(1));
    // Nothing is drawn by the subroutines: no ink, and no failure either.
    let run = shape(&font, "H", 72.0, &ShapeOptions::default());
    assert_eq!(run.glyphs.len(), 1);
    assert!(run.to_outline(&font, Point::new(0.0, 0.0)).is_empty());
}

// ---------------------------------------------------------------------------------------
// The attacks
// ---------------------------------------------------------------------------------------

/// A TrueType glyph cannot have more than 65 535 points (point numbers are 16-bit), so
/// no legitimate outline of ONE glyph needs more than about 2 x 65 536 path segments.
/// A font of about 1 kB must not be able to make `to_outline` emit a million of them: with
/// a few more levels the same font asks for gigabytes (2^30 rectangles at 30 levels).
#[test]
fn a_composite_glyph_cannot_multiply_into_millions_of_segments() {
    let bytes = composite_bomb(18, 2, true, true);
    assert!(bytes.len() < 2048, "the font is {} bytes", bytes.len());
    let font = Font::from_bytes(bytes, 0).expect("parses");
    let run = shape(&font, "H", 12.0, &ShapeOptions::default());
    assert_eq!(run.glyphs.len(), 1);
    let path = run.to_outline(&font, Point::new(0.0, 0.0));
    assert!(
        path.segs.len() <= 2 * 65_536 + 1024,
        "one glyph of a {}-byte font became {} path segments ({} MB of PathSeg)",
        font.data().len(),
        path.segs.len(),
        path.segs.len() * std::mem::size_of::<ebc_scene::PathSeg>() / (1 << 20),
    );
}

/// The same font shape with empty leaves and 4^14 = 268 435 456 component visits: no
/// memory is used, only time. Drawing one letter must not take seconds (and with a
/// fan-out of 10 and 30 levels it would not finish in the lifetime of the universe).
#[test]
fn outlining_a_composite_glyph_takes_bounded_time() {
    let bytes = composite_bomb(14, 4, false, true);
    assert!(bytes.len() < 2048, "the font is {} bytes", bytes.len());
    let font = Font::from_bytes(bytes, 0).expect("parses");
    let run = shape(&font, "H", 12.0, &ShapeOptions::default());
    let started = Instant::now();
    let done = within(Duration::from_secs(3), move || {
        run.to_outline(&font, Point::new(0.0, 0.0)).segs.len()
    });
    assert!(
        done.is_some(),
        "to_outline of ONE glyph of a 1.3 kB font was still running after {:?}",
        started.elapsed()
    );
}

/// Without an `OS/2` capital height, `Font::from_bytes` measures the outline of "H" —
/// so the same glyph hangs the *loading* of the font, before any policy check
/// (`allows_outlines`, …) can reject it.
#[test]
fn loading_a_font_with_a_composite_bomb_takes_bounded_time() {
    let bytes = composite_bomb(14, 4, false, false);
    assert!(bytes.len() < 2048, "the font is {} bytes", bytes.len());
    let started = Instant::now();
    let done = within(Duration::from_secs(3), move || Font::from_bytes(bytes, 0).is_ok());
    assert!(
        done.is_some(),
        "Font::from_bytes of a 1.2 kB font was still running after {:?}",
        started.elapsed()
    );
}

/// Same attack through the rasteriser entry point.
#[cfg(feature = "raster")]
#[test]
fn rasterising_a_composite_glyph_takes_bounded_time() {
    let bytes = composite_bomb(14, 4, false, true);
    let font = Font::from_bytes(bytes, 0).expect("parses");
    let started = Instant::now();
    let done = within(Duration::from_secs(3), move || {
        ebc_text::try_rasterize(&font, "H", 32.0, &ShapeOptions::default()).is_ok()
    });
    assert!(done.is_some(), "try_rasterize was still running after {:?}", started.elapsed());
}

/// Type 2 charstrings: ten nested subroutine levels, each calling the next 8 times, is
/// 8^10 = 1 073 741 824 calls from a font of about 750 bytes. (With 100 calls per level
/// — 3 kB — it is 10^20.) The font has no `OS/2` table, so `Font::from_bytes` itself
/// runs the charstring of "H" to find the capital height.
#[test]
fn loading_a_font_with_a_cff_subroutine_bomb_takes_bounded_time() {
    let bytes = cff_subroutine_bomb(10, 8, false);
    assert!(bytes.len() < 2048, "the font is {} bytes", bytes.len());
    let started = Instant::now();
    let done = within(Duration::from_secs(3), move || Font::from_bytes(bytes, 0).is_ok());
    assert!(
        done.is_some(),
        "Font::from_bytes of a {}-byte CFF font was still running after {:?}",
        cff_subroutine_bomb(10, 8, false).len(),
        started.elapsed()
    );
}

/// The same CFF font with an `OS/2` table loads at once; the bomb then goes off in
/// `to_outline` / `ink_bounds`.
#[test]
fn outlining_a_cff_subroutine_bomb_takes_bounded_time() {
    let font = Font::from_bytes(cff_subroutine_bomb(10, 8, true), 0).expect("parses");
    let run = shape(&font, "H", 12.0, &ShapeOptions::default());
    assert_eq!(run.glyphs.len(), 1);
    let started = Instant::now();
    let done = within(Duration::from_secs(3), move || {
        run.ink_bounds(&font, Point::new(0.0, 0.0)).is_some()
    });
    assert!(done.is_some(), "ink_bounds was still running after {:?}", started.elapsed());
}

/// Not an assertion: prints how the cost grows with the depth of the composite tree and
/// of the subroutine tree. Run with
/// `cargo test -p ebc-text --test spar_bombs -- --ignored --nocapture growth`.
#[test]
#[ignore = "measurement only"]
fn growth_of_the_cost_with_the_nesting_depth() {
    for levels in [8u16, 10, 12, 13] {
        let font = Font::from_bytes(composite_bomb(levels, 4, false, true), 0).expect("parses");
        let run = shape(&font, "H", 12.0, &ShapeOptions::default());
        let started = Instant::now();
        let segs = run.to_outline(&font, Point::new(0.0, 0.0)).segs.len();
        eprintln!(
            "glyf: {levels} levels x fan-out 4, {} bytes: {:?} ({segs} segments)",
            font.data().len(),
            started.elapsed()
        );
    }
    for fan_out in [3usize, 4, 5, 6] {
        let bytes = cff_subroutine_bomb(10, fan_out, false);
        let len = bytes.len();
        let started = Instant::now();
        let ok = Font::from_bytes(bytes, 0).is_ok();
        eprintln!(
            "CFF: 10 levels x fan-out {fan_out}, {len} bytes: from_bytes {:?} (ok: {ok})",
            started.elapsed()
        );
    }
}

// ---------------------------------------------------------------------------------------
// Added with the fix: what the bounded outline path promises beyond "it returns".
// ---------------------------------------------------------------------------------------

use ebc_text::{FontError, ShapedGlyph, ShapedText};

/// An `fvar` table with one axis, "wght" 100..900, default 400, and no named instance.
fn fvar_weight_axis() -> Vec<u8> {
    let mut fvar = Vec::new();
    push_u32(&mut fvar, 0x0001_0000);
    for v in [16u16, 2, 1, 20, 0, 8] {
        push_u16(&mut fvar, v); // axes offset, reserved, axis count and size, instances
    }
    fvar.extend_from_slice(b"wght");
    for v in [100u32, 400, 900] {
        push_u32(&mut fvar, v << 16);
    }
    push_u16(&mut fvar, 0);
    push_u16(&mut fvar, 256);
    fvar
}

fn one_glyph(id: u16) -> ShapedText {
    let glyph = ShapedGlyph { id, x: 0.0, y: 0.0, advance: 0.0, text: String::new() };
    ShapedText { glyphs: vec![glyph], width: 0.0, size_pt: 72.0 }
}

#[test]
fn an_oversized_glyph_is_reported_and_the_rest_of_the_font_still_draws() {
    let font = Font::from_bytes(composite_bomb(18, 2, true, true), 0).expect("parses");
    assert!(font.has_oversized_glyphs());
    let origin = Point::new(0.0, 0.0);
    let run = shape(&font, "H", 12.0, &ShapeOptions::default());
    assert!(run.to_outline(&font, origin).is_empty(), "the bomb draws nothing");
    assert_eq!(run.try_to_outline(&font, origin), Err(FontError::OutlineTooComplex { glyph: 1 }));
    assert_eq!(run.ink_bounds(&font, origin), None);
    #[cfg(feature = "raster")]
    assert_eq!(
        ebc_text::try_rasterize(&font, "H", 32.0, &ShapeOptions::default()),
        Err(FontError::OutlineTooComplex { glyph: 1 })
    );

    // Glyph 1 + k is the composite k levels down: 2^(18 − k) rectangles. The ones that
    // stay within 65 536 points and component references are drawn in full.
    let rectangle = one_glyph(19).to_outline(&font, origin).segs.len();
    assert!(rectangle >= 5);
    for (levels_down, copies) in [(17u16, 2usize), (10, 256), (6, 4_096)] {
        let path = one_glyph(1 + levels_down)
            .try_to_outline(&font, origin)
            .expect("an ordinary composite glyph");
        assert_eq!(path.segs.len(), copies * rectangle, "{levels_down} levels down");
    }
    // 2^14 rectangles are 16 384 x 4 points plus 32 766 references: too many.
    assert_eq!(
        one_glyph(1 + 4).try_to_outline(&font, origin),
        Err(FontError::OutlineTooComplex { glyph: 5 })
    );

    // A font without such glyphs says so.
    let fine = Font::from_bytes(composite_bomb(3, 2, true, true), 0).expect("parses");
    assert!(!fine.has_oversized_glyphs());
    for (label, bytes) in common::ALL {
        assert!(!common::font(bytes).has_oversized_glyphs(), "{label}");
    }
}

/// A glyph just under the per-glyph limit, repeated: the whole call has a budget too.
#[test]
fn a_long_text_cannot_multiply_an_expensive_glyph() {
    // 30 + 30² + 30³ = 27 930 references to an empty glyph: legal on its own.
    let font = Font::from_bytes(composite_bomb(3, 30, false, true), 0).expect("parses");
    assert!(!font.has_oversized_glyphs());
    assert_eq!(
        one_glyph(1).try_to_outline(&font, Point::new(0.0, 0.0)),
        Ok(ebc_scene::Path::new())
    );

    let text = "H".repeat(20_000); // 20 000 x 55 861 = 10^9 component visits if unchecked
    let started = Instant::now();
    let done = within(Duration::from_secs(10), move || {
        let run = shape(&font, &text, 12.0, &ShapeOptions::default());
        assert_eq!(run.glyphs.len(), 20_000);
        let outline = run.try_to_outline(&font, Point::new(0.0, 0.0));
        #[cfg(feature = "raster")]
        let raster = ebc_text::try_rasterize(&font, &text, 16.0, &ShapeOptions::default());
        #[cfg(not(feature = "raster"))]
        let raster = outline.clone().map(|_| ());
        (outline, raster.is_err(), run.to_outline(&font, Point::new(0.0, 0.0)).is_empty())
    });
    let (outline, raster_failed, plain_is_empty) =
        done.unwrap_or_else(|| panic!("still running after {:?}", started.elapsed()));
    assert_eq!(outline, Err(FontError::OutlineTooComplex { glyph: 1 }));
    assert!(raster_failed && plain_is_empty);
}

/// The shaper itself outlines glyphs in one situation — a variable font without `HVAR`,
/// at a non-default instance, for glyphs without `gvar` data — once per glyph of the
/// text. Neither a bomb nor a merely expensive glyph may be reachable that way.
#[test]
fn shaping_an_instance_of_a_variable_bomb_takes_bounded_time() {
    for (levels, fan_out) in [(14, 4), (3, 30)] {
        let mut tables = composite_bomb_tables(levels, fan_out, false, true);
        tables.push((b"fvar", fvar_weight_axis()));
        let font = Font::from_bytes(common::build_sfnt(tables), 0).expect("parses");
        assert!(font.is_variable());
        let bold = font.with_variations(&[("wght", 700.0)]);
        assert_ne!(bold, font, "the instance is not the default one");
        let started = Instant::now();
        let done = within(Duration::from_secs(5), move || {
            let text = "H".repeat(5_000);
            let opts = ShapeOptions::default();
            let run = shape(&bold, &text, 12.0, &opts);
            let lines = ebc_text::wrap(&bold, &text, 12.0, 50.0, &opts);
            let size = ebc_text::fit_size(&bold, &text, 100.0, 5.0, 1.0, 50.0, &opts);
            (run.glyphs.len(), lines.concat().len(), size.is_some(), bold.missing_chars(&text))
        });
        let (glyphs, wrapped, _, missing) = done.unwrap_or_else(|| {
            panic!("{levels} x {fan_out}: shaping still running after {:?}", started.elapsed())
        });
        assert_eq!((glyphs, wrapped), (5_000, 5_000), "{levels} x {fan_out}");
        assert!(missing.is_empty());
    }
}

#[test]
fn postscript_bombs_are_reported_and_ordinary_subroutines_still_work() {
    let origin = Point::new(0.0, 0.0);
    let bomb = Font::from_bytes(cff_subroutine_bomb(10, 8, true), 0).expect("parses");
    let run = shape(&bomb, "H", 12.0, &ShapeOptions::default());
    assert_eq!(run.try_to_outline(&bomb, origin), Err(FontError::OutlineTooComplex { glyph: 1 }));
    assert!(run.to_outline(&bomb, origin).is_empty());

    // Without an OS/2 table the bomb is met while loading: the font still loads, with
    // the default capital height of 0.7 em.
    let bomb = Font::from_bytes(cff_subroutine_bomb(10, 8, false), 0).expect("parses");
    assert_eq!(bomb.metrics().cap_height, 700);
    let glyf_bomb = Font::from_bytes(composite_bomb(14, 4, false, false), 0).expect("parses");
    assert_eq!(glyf_bomb.metrics().cap_height, 700);

    // 8^4 = 4 096 calls that draw nothing are within the budget of one glyph…
    let busy = Font::from_bytes(cff_subroutine_bomb(4, 8, true), 0).expect("parses");
    assert_eq!(one_glyph(1).try_to_outline(&busy, origin), Ok(ebc_scene::Path::new()));
    // …but not 20 000 times over.
    let text = "H".repeat(20_000);
    let started = Instant::now();
    let done = within(Duration::from_secs(10), move || {
        shape(&busy, &text, 12.0, &ShapeOptions::default()).try_to_outline(&busy, origin)
    });
    let outline = done.unwrap_or_else(|| panic!("still running after {:?}", started.elapsed()));
    assert_eq!(outline, Err(FontError::OutlineTooComplex { glyph: 1 }));

    // The ordinary CFF fixture is unaffected: "O" is four curves, "H" a rectangle.
    let font = Font::from_bytes(common::synthetic_cff_font(), 0).expect("parses");
    let run = shape(&font, "OH", 72.0, &ShapeOptions::default());
    let path = run.try_to_outline(&font, origin).expect("ordinary glyphs");
    let curves = path.segs.iter().filter(|s| matches!(s, ebc_scene::PathSeg::CubicTo(..))).count();
    assert_eq!(curves, 4);
    let ink = run.ink_bounds(&font, origin).expect("ink");
    assert!(common::close(ink.h, 0.7 * 25.4, 1e-6), "{ink:?}");
}
