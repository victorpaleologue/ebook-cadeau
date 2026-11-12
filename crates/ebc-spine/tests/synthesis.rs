//! The typeset spine: geometry, direction, optical centring, fitting, colours and the
//! rule that the default never puts pixels of the cover on the spine.

mod common;

use common::*;
use ebc_color::{Rgb8, contrast_ratio, palette};
use ebc_i18n::SpineDirection::{self, BottomToTop, TopToBottom};
use ebc_layout::{CardFormat, PrintProfile, spine_safe_rect};
use ebc_model::BookMeta;
use ebc_scene::{Color, Document, Item, PathSeg, Pixmap, Point, Rect, Transform};
use ebc_spine::{
    BackgroundKind, BackgroundPaint, BackgroundStyle, MIN_OUTLINED_PT, SpineDesign, SpineError,
    SpineNote, SpineOptions, SpineRequest, SpineSource, TextRole, design, design_with, text_area,
};
use ebc_text::{ShapeOptions, outline_bounds, shape};

const EPS: f64 = 1e-6;
const BOTH: [SpineDirection; 2] = [TopToBottom, BottomToTop];

fn run(req: &SpineRequest<'_>) -> SpineDesign {
    let mut doc = Document::default();
    let spine = design(req, &mut doc).expect("the design succeeds");
    assert!(doc.images.is_empty(), "the default design registers no image");
    spine
}

fn run_with(req: &SpineRequest<'_>, opts: &SpineOptions) -> (SpineDesign, Document) {
    let mut doc = Document::default();
    let spine = design_with(req, opts, &mut doc).expect("the design succeeds");
    (spine, doc)
}

fn title_bounds(spine: &SpineDesign) -> Rect {
    spine.title.as_ref().expect("a title was set").bounds
}

fn covers() -> Vec<(&'static str, Option<Pixmap>)> {
    vec![
        ("none", None),
        ("cream", Some(flat_cover([250, 245, 230]))),
        ("navy", Some(flat_cover([18, 28, 70]))),
        ("mid grey", Some(flat_cover([128, 128, 128]))),
        ("busy", Some(busy_cover(7))),
        ("white frame", Some(framed_cover([255, 255, 255], 14, 3))),
        ("plain band", Some(banded_cover([180, 30, 40], 40, 5))),
    ]
}

// ----- geometry ---------------------------------------------------------------------------

#[test]
fn everything_is_inside_the_panel_and_text_inside_the_safe_band() {
    let book = book("Le Comte de Monte-Cristo", &["Alexandre Dumas"]);
    let (title_font, author_font) = (serif(), serif_regular());
    let (rect, bleed, safe_margin) = poche();
    assert_eq!(rect, Rect::new(110.0, -3.0, 10.0, 184.0));
    let safe = spine_safe_rect(&CardFormat::poche(), &PrintProfile::professional_eu());

    for (name, cover) in covers() {
        for direction in BOTH {
            let req = request(&book, cover.as_ref(), &title_font, &author_font, direction);
            let spine = run(&req);
            let bounds = painted_bounds(&spine.items);

            // The background is the first item and covers the panel exactly, bleed included.
            assert_eq!(bounds[0], rect, "{name}");
            assert!((bounds[0].y - (0.0 - bleed)).abs() < EPS);
            assert!((bounds[0].bottom() - (178.0 + bleed)).abs() < EPS);
            for b in &bounds {
                assert!(rect.contains_rect(b, EPS), "{name}: {b:?} leaves the panel");
            }

            // Title and author are set, inside the band between the safe margins.
            let area = text_area(&req, &SpineOptions::default()).expect("valid geometry");
            assert!((area.x - (rect.x + safe_margin)).abs() < EPS);
            assert!((area.w - (rect.w - 2.0 * safe_margin)).abs() < EPS);
            assert!((area.y - 8.0).abs() < EPS && (area.bottom() - 170.0).abs() < EPS);
            let title = spine.title.as_ref().expect("title");
            let author = spine.author.as_ref().expect("author");
            for placed in [title, author] {
                assert!(area.contains_rect(&placed.bounds, EPS), "{name}: {placed:?}");
                assert!(safe.contains_rect(&placed.bounds, EPS), "{name}: layout's safe rect");
                assert_eq!(bounds[placed.item], placed.bounds, "bounds are those of the item");
            }
            assert_eq!(title.text, "Le Comte de Monte-Cristo");
            assert_eq!(author.text, "Alexandre Dumas");
            assert_eq!(spine.title_size_pt, title.size_pt);
            assert!(spine.notes.len() == spine.details.len());
        }
    }
}

#[test]
fn every_preset_and_a_range_of_spine_widths_keep_text_in_the_band() {
    let book = book("Les Fleurs du mal", &["Charles Baudelaire"]);
    let font = sans();
    let mut previous = 0.0;
    // 5 mm is the narrowest valid spine (two creases need 5 mm between them).
    for width in [5.0, 6.0, 7.0, 8.0, 10.0, 15.0, 25.0, 40.0] {
        for (name, format) in CardFormat::presets() {
            let format = format.with_spine(width).expect("a valid spine width");
            let profile = PrintProfile::professional_eu();
            let rect = format.outside().with_bleed(profile.bleed).spine;
            let req = SpineRequest {
                rect,
                bleed: profile.bleed,
                safe_margin: profile.spine_safe_margin,
                ..request(&book, None, &font, &font, BottomToTop)
            };
            let spine = run(&req);
            for b in painted_bounds(&spine.items) {
                assert!(rect.contains_rect(&b, EPS), "{name} {width}: {b:?}");
            }
            let area = text_area(&req, &SpineOptions::default()).expect("valid");
            match &spine.title {
                Some(title) => {
                    assert!(area.contains_rect(&title.bounds, EPS), "{name} {width}");
                    // Never below the 8 pt floor of outlined text, never above 13 pt.
                    assert!(title.size_pt >= MIN_OUTLINED_PT && title.size_pt <= 13.0);
                }
                None => {
                    // A 5 mm spine leaves a 1 mm band between two 2 mm margins and a
                    // 6 mm one 2 mm: not enough for 8 pt letters with ascenders.
                    assert!(width <= 6.0, "{name} {width}: no title");
                    assert!(has_note(&spine, |n| *n == SpineNote::NoRoomForText));
                    assert_eq!(spine.items.len(), 1, "background only");
                    assert_eq!(spine.title_size_pt, 0.0);
                }
            }
            if name == "poche" {
                assert!(spine.title_size_pt >= previous, "wider spine, smaller title");
                previous = spine.title_size_pt;
            }
        }
    }
    assert_eq!(previous, 13.0, "wide spines reach the maximum size");
}

// ----- direction --------------------------------------------------------------------------

#[test]
fn top_to_bottom_reads_downwards_and_bottom_to_top_upwards() {
    let book = BookMeta::titled("Zadig ou la Destinée");
    let font = serif();
    for direction in BOTH {
        let spine = run(&request(&book, None, &font, &font, direction));
        let title = spine.title.as_ref().expect("title");
        let (path, _) = filled_path(&spine, title.item);
        let first = first_contour_bounds(path); // the "Z"
        let last = last_contour_bounds(path); // the last "e"
        let all = title.bounds;
        assert!(all.h > 5.0 * all.w, "the line runs along the spine: {all:?}");
        match direction {
            TopToBottom => {
                // The first letter starts the line at the top; the last ends it below.
                assert!((first.y - all.y).abs() < EPS, "{first:?} vs {all:?}");
                assert!((last.bottom() - all.bottom()).abs() < 0.5);
                assert!(first.center().y < all.center().y - all.h / 3.0);
                assert!(last.center().y > all.center().y + all.h / 3.0);
            }
            BottomToTop => {
                assert!((first.bottom() - all.bottom()).abs() < EPS, "{first:?} vs {all:?}");
                assert!((last.y - all.y).abs() < 0.5);
                assert!(first.center().y > all.center().y + all.h / 3.0);
                assert!(last.center().y < all.center().y - all.h / 3.0);
            }
        }
    }
}

#[test]
fn the_outline_is_the_horizontal_line_rotated_by_the_conventional_angle() {
    // Rebuild the expected outline independently: shape the title horizontally, rotate it
    // with ebc-scene by the angle ebc-i18n documents for the direction (+90° clockwise
    // for top-to-bottom, −90° for bottom-to-top), and compare point by point up to a
    // translation.
    let book = BookMeta::titled("Stoner");
    let font = serif();
    for direction in BOTH {
        let spine = run(&request(&book, None, &font, &font, direction));
        let title = spine.title.as_ref().expect("title");
        let (path, _) = filled_path(&spine, title.item);

        let flat = shape(&font, "Stoner", title.size_pt, &ShapeOptions::default())
            .to_outline(&font, Point::new(0.0, 0.0));
        let rotated = flat.transformed(&Transform::rotate(direction.rotation_degrees()));
        let expected = outline_bounds(&rotated).expect("ink");
        let (dx, dy) = (title.bounds.x - expected.x, title.bounds.y - expected.y);
        let expected = rotated.transformed(&Transform::translate(dx, dy));
        assert_eq!(path.segs.len(), expected.segs.len());
        let close = |a: Point, b: Point| (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9;
        for (got, want) in path.segs.iter().zip(&expected.segs) {
            let same = match (*got, *want) {
                (PathSeg::MoveTo(a), PathSeg::MoveTo(b))
                | (PathSeg::LineTo(a), PathSeg::LineTo(b)) => close(a, b),
                (PathSeg::QuadTo(a1, a2), PathSeg::QuadTo(b1, b2)) => {
                    close(a1, b1) && close(a2, b2)
                }
                (PathSeg::CubicTo(a1, a2, a3), PathSeg::CubicTo(b1, b2, b3)) => {
                    close(a1, b1) && close(a2, b2) && close(a3, b3)
                }
                (PathSeg::Close, PathSeg::Close) => true,
                _ => false,
            };
            assert!(same, "{direction:?}: {got:?} vs {want:?}");
        }
    }
}

/// All the points (on-curve and control) of a path.
fn points(path: &ebc_scene::Path) -> Vec<Point> {
    let mut out = Vec::new();
    for seg in &path.segs {
        match *seg {
            PathSeg::MoveTo(p) | PathSeg::LineTo(p) => out.push(p),
            PathSeg::QuadTo(c, p) => out.extend([c, p]),
            PathSeg::CubicTo(c1, c2, p) => out.extend([c1, c2, p]),
            PathSeg::Close => {}
        }
    }
    out
}

#[test]
fn the_text_is_rotated_not_mirrored() {
    // A sans-serif "L" is a stem with a foot. Horizontally the empty corner of its box
    // is the top right. Rotated 90° clockwise (top-to-bottom) the stem lies along the
    // top, the foot runs down the left (baseline) side, and the empty corner is the
    // bottom right; rotated counter-clockwise the stem lies along the bottom, the foot
    // runs up the right side, and the empty corner is the top left. A mirrored "L" would
    // have its empty corner elsewhere.
    let book = BookMeta::titled("L");
    let font = caps();
    for direction in BOTH {
        let spine = run(&request(&book, None, &font, &font, direction));
        let title = spine.title.as_ref().expect("title");
        let (path, _) = filled_path(&spine, title.item);
        let b = title.bounds;
        let corner = |right: bool, bottom: bool| -> usize {
            points(path)
                .iter()
                .filter(|p| {
                    let in_x = if right { p.x > b.x + 0.6 * b.w } else { p.x < b.x + 0.4 * b.w };
                    let in_y = if bottom { p.y > b.y + 0.6 * b.h } else { p.y < b.y + 0.4 * b.h };
                    in_x && in_y
                })
                .count()
        };
        let (top_left, top_right) = (corner(false, false), corner(true, false));
        let (bottom_left, bottom_right) = (corner(false, true), corner(true, true));
        match direction {
            TopToBottom => {
                assert_eq!(bottom_right, 0, "{direction:?}");
                assert!(top_left > 0 && top_right > 0 && bottom_left > 0);
            }
            BottomToTop => {
                assert_eq!(top_left, 0, "{direction:?}");
                assert!(top_right > 0 && bottom_left > 0 && bottom_right > 0);
            }
        }
    }
}

#[test]
fn tops_of_the_letters_point_to_the_front_cover_when_reading_downwards() {
    // The spread is back | spine | front: the front cover is to the right (+x).
    // "b" has an ascender, "p" a descender, "x" neither.
    let font = serif();
    let bounds = |text: &str, direction| {
        let book = BookMeta::titled(text);
        let spine = run(&request(&book, None, &font, &font, direction));
        assert_eq!(spine.title_size_pt, 13.0);
        title_bounds(&spine)
    };
    let (x, b, p) = (bounds("x", TopToBottom), bounds("b", TopToBottom), bounds("p", TopToBottom));
    assert!(b.right() > x.right() + 0.5, "ascender towards the front: {b:?} vs {x:?}");
    assert!((b.x - x.x).abs() < 0.2);
    assert!(p.x < x.x - 0.5, "descender towards the back: {p:?} vs {x:?}");
    assert!((p.right() - x.right()).abs() < 0.2);

    let (x, b, p) = (bounds("x", BottomToTop), bounds("b", BottomToTop), bounds("p", BottomToTop));
    assert!(b.x < x.x - 0.5, "ascender towards the back: {b:?} vs {x:?}");
    assert!((b.right() - x.right()).abs() < 0.2);
    assert!(p.right() > x.right() + 0.5, "descender towards the front: {p:?} vs {x:?}");
    assert!((p.x - x.x).abs() < 0.2);
}

// ----- optical centring -------------------------------------------------------------------

#[test]
fn capitals_are_centred_on_the_spine_whatever_else_the_line_contains() {
    let (rect, ..) = poche();
    let centre = rect.center().x;
    for font in [serif(), sans(), caps()] {
        let cap = font.metrics().cap_height_mm(1.0);
        for direction in BOTH {
            // "H" spans exactly baseline to capital height.
            let book = BookMeta::titled("HHHH");
            let spine = run(&request(&book, None, &font, &font, direction));
            let b = title_bounds(&spine);
            assert!((b.center().x - centre).abs() < 0.02, "{font:?} {direction:?}: {b:?}");
            assert!((b.w - cap * spine.title_size_pt).abs() < 0.02, "ink width = cap height");

            // Descenders and accents do not move the capitals: the line's ink is no
            // longer centred, but its capital-height box still is.
            let book = BookMeta::titled("Hpgjy ÉÇ");
            let spine = run(&request(&book, None, &font, &font, direction));
            let b = title_bounds(&spine);
            let half_cap = cap * spine.title_size_pt / 2.0;
            let (baseline, cap_line) = match direction {
                TopToBottom => (centre - half_cap, centre + half_cap),
                BottomToTop => (centre + half_cap, centre - half_cap),
            };
            // Ink reaches beyond the capital line (accent) and beyond the baseline
            // (descender, cedilla), on the expected sides.
            match direction {
                TopToBottom => {
                    assert!(b.x < baseline - 0.2 && b.right() > cap_line + 0.2, "{b:?}");
                }
                BottomToTop => {
                    assert!(b.right() > baseline + 0.2 && b.x < cap_line - 0.2, "{b:?}");
                }
            }
        }
    }
}

#[test]
fn lower_case_without_ascenders_sits_on_the_baseline_side_of_the_centre_line() {
    // Centring on the capital height (not on the ink, not on the em box) leaves a line
    // of x-height letters on the baseline side — as on any printed spine.
    let (rect, ..) = poche();
    let font = sans();
    let book = BookMeta::titled("xxxxx");
    let down = title_bounds(&run(&request(&book, None, &font, &font, TopToBottom)));
    let up = title_bounds(&run(&request(&book, None, &font, &font, BottomToTop)));
    assert!(down.center().x < rect.center().x - 0.2, "{down:?}");
    assert!(up.center().x > rect.center().x + 0.2, "{up:?}");
    // The two directions are mirror images about the centre line.
    assert!(((down.center().x + up.center().x) / 2.0 - rect.center().x).abs() < 1e-6);
}

#[test]
fn author_and_title_are_centred_together_along_the_spine() {
    let book = book("Madame Bovary", &["Gustave Flaubert"]);
    let font = serif();
    let (rect, bleed, _) = poche();
    let (head, foot) = (rect.y + bleed, rect.bottom() - bleed);
    for direction in BOTH {
        let spine = run(&request(&book, None, &font, &font, direction));
        let title = spine.title.as_ref().expect("title");
        let author = spine.author.as_ref().expect("author");
        let ink = title.bounds.union(&author.bounds);
        assert!(((ink.y - head) - (foot - ink.bottom())).abs() < 1e-6, "{ink:?}");
        // Author first in the reading direction, 6 mm before the title, smaller.
        let gap = match direction {
            TopToBottom => title.bounds.y - author.bounds.bottom(),
            BottomToTop => author.bounds.y - title.bounds.bottom(),
        };
        assert!((gap - 6.0).abs() < 1e-6, "{direction:?}: gap {gap}");
        assert!((author.size_pt - 0.75 * title.size_pt).abs() < 1e-6);
        assert!(author.size_pt >= MIN_OUTLINED_PT);
        assert!(author.item < title.item);
    }
}

#[test]
fn author_and_title_share_one_baseline() {
    let book = book("HHHH", &["HH"]);
    let font = sans();
    for direction in BOTH {
        let spine = run(&request(&book, None, &font, &font, direction));
        let title = title_bounds(&spine);
        let author = spine.author.as_ref().expect("author").bounds;
        match direction {
            TopToBottom => assert!((title.x - author.x).abs() < 0.02, "{title:?} {author:?}"),
            BottomToTop => assert!((title.right() - author.right()).abs() < 0.02),
        }
        assert!(author.w < title.w);
    }
}

// ----- fitting ----------------------------------------------------------------------------

const LONG: &str = "La Vie et les aventures étranges et surprenantes de Robinson Crusoé de York \
                    marin qui vécut vingt-huit ans tout seul sur une île déserte de la côte de \
                    l'Amérique près de l'embouchure du grand fleuve Orénoque après avoir été \
                    jeté à la côte au cours d'un naufrage dont il fut le seul survivant";

#[test]
fn longer_titles_never_get_larger_and_never_leave_the_live_length() {
    let font = serif();
    let words: Vec<&str> = LONG.split(' ').collect();
    let mut previous = f64::INFINITY;
    let mut sizes = Vec::new();
    for n in 1..=words.len() {
        let book = BookMeta::titled(words[..n].join(" "));
        let req = request(&book, None, &font, &font, BottomToTop);
        let spine = run(&req);
        let title = spine.title.as_ref().expect("title");
        let area = text_area(&req, &SpineOptions::default()).expect("valid");
        assert!(area.contains_rect(&title.bounds, EPS), "{n} words: {:?}", title.bounds);
        assert!(title.size_pt <= previous + 1e-9, "{n} words: {} > {previous}", title.size_pt);
        assert!((9.0..=13.0).contains(&title.size_pt));
        previous = title.size_pt;
        sizes.push(title.size_pt);
    }
    assert_eq!(sizes[0], 13.0, "a short title is set at the maximum size");
    assert_eq!(*sizes.last().expect("sizes"), 9.0, "the longest is truncated at the minimum");
    assert!(sizes.iter().any(|s| *s > 9.0 && *s < 13.0), "sizes in between are used");
}

#[test]
fn the_ladder_tightens_then_cuts_the_subtitle_then_truncates() {
    let font = serif();
    let fit = |title: &str| {
        let book = BookMeta::titled(title);
        run(&request(&book, None, &font, &font, TopToBottom))
    };
    // Find the longest prefix that fits whole at the minimum size, then add words.
    let words: Vec<&str> = LONG.split(' ').collect();
    let whole = (1..=words.len())
        .take_while(|&n| {
            let spine = fit(&words[..n].join(" "));
            !has_note(&spine, |note| !matches!(note, SpineNote::NoCover))
        })
        .last()
        .expect("some prefix fits");
    assert!(whole > 15 && whole < words.len(), "{whole} words fit whole");

    // One more word: tightening the letter-spacing (at most 2 %) is tried first.
    let next = fit(&words[..=whole].join(" "));
    let tightened = has_note(&next, |n| *n == SpineNote::TitleTightened);
    let truncated = has_note(&next, |n| matches!(n, SpineNote::TitleTruncated { .. }));
    assert!(tightened || truncated);
    if tightened {
        assert_eq!(next.title.as_ref().expect("title").text, words[..=whole].join(" "));
    }

    // Far too long: truncated at a word boundary with an ellipsis, at the minimum size.
    let spine = fit(LONG);
    let title = spine.title.as_ref().expect("title");
    assert_eq!(title.size_pt, 9.0);
    let kept = title.text.strip_suffix('…').expect("ends with an ellipsis");
    assert!(LONG.starts_with(kept), "a prefix of the title: {kept}");
    let rest = &LONG[kept.len()..];
    assert!(rest.starts_with(' '), "cut at a word boundary, before {rest:.20}");
    assert!(has_note(
        &spine,
        |n| matches!(n, SpineNote::TitleTruncated { set } if *set == title.text)
    ));
    assert!(spine.notes.iter().any(|n| n.contains("truncated")), "{:?}", spine.notes);

    // A title with a subtitle after a separator loses the subtitle before any word.
    let with_subtitle = format!("Robinson Crusoé : {LONG}");
    let spine = fit(&with_subtitle);
    assert_eq!(spine.title.as_ref().expect("title").text, "Robinson Crusoé");
    assert!(has_note(&spine, |n| matches!(n, SpineNote::TitleShortened { .. })));
    assert_eq!(spine.title_size_pt, 13.0);
}

#[test]
fn hostile_titles_do_not_panic() {
    let font = serif();
    let mut huge = String::new();
    for i in 0..3_000 {
        huge.push_str(if i % 2 == 0 { "mot " } else { "\u{202E}très\u{200B} " });
    }
    let one_word = "a".repeat(10_000);
    let rtl_override = "\u{202E}evil\u{202C} title";
    for title in [huge.as_str(), one_word.as_str(), rtl_override, "…", ". . .", "a\tb\nc"] {
        let book = BookMeta::titled(title);
        let req = request(&book, None, &font, &font, BottomToTop);
        let spine = run(&req);
        let placed = spine.title.as_ref().expect("something is set");
        let area = text_area(&req, &SpineOptions::default()).expect("valid");
        assert!(area.contains_rect(&placed.bounds, EPS));
        assert!(!placed.text.contains(['\u{202E}', '\u{200B}', '\t', '\n']));
    }
    let book = BookMeta::titled(huge);
    let spine = run(&request(&book, None, &font, &font, BottomToTop));
    assert!(has_note(&spine, |n| *n == SpineNote::TitleTooLong));
    let book = BookMeta::titled(one_word);
    let spine = run(&request(&book, None, &font, &font, BottomToTop));
    let text = &spine.title.as_ref().expect("title").text;
    assert!(text.starts_with("aaaa") && text.ends_with('…') && text.len() < 400);
}

#[test]
fn an_empty_title_is_an_error() {
    let font = serif();
    for title in ["", "   ", "\u{200B}\n\t", "\u{FEFF}"] {
        let book = book(title, &["Someone"]);
        let mut doc = Document::default();
        let result = design(&request(&book, None, &font, &font, TopToBottom), &mut doc);
        assert_eq!(result, Err(SpineError::EmptyTitle), "{title:?}");
    }
}

#[test]
fn the_subtitle_is_dropped_and_nothing_is_recased() {
    let font = serif();
    let mut b = book("Sapiens : Une brève histoire de l'humanité", &[]);
    b.subtitle = Some("Une brève histoire de l'humanité".to_owned());
    let spine = run(&request(&b, None, &font, &font, BottomToTop));
    assert_eq!(spine.title.as_ref().expect("title").text, "Sapiens");
    assert!(spine.author.is_none());
    assert!(!has_note(&spine, |n| *n == SpineNote::AuthorDropped), "there was no author");

    // French typography survives: narrow no-break space, ligature, guillemets, accents.
    let title = "Qui\u{202F}? L’ŒUVRE «\u{00A0}À la recherche\u{00A0}» d’Été";
    let b = book(title, &[]);
    let spine = run(&request(&b, None, &font, &font, BottomToTop));
    assert_eq!(spine.title.as_ref().expect("title").text, title);
    assert_eq!(spine.details, vec![SpineNote::NoCover], "{:?}", spine.notes);
}

#[test]
fn accented_capitals_stay_inside_the_band() {
    // Research test T8: the ink of accents and cedillas counts, not just the cap height.
    for font in [serif(), sans(), caps()] {
        let b = book("ÉCOLE DES FEMMES, ÇA", &[]);
        let req = request(&b, None, &font, &font, BottomToTop);
        let spine = run(&req);
        let bounds = title_bounds(&spine);
        let area = text_area(&req, &SpineOptions::default()).expect("valid");
        assert!(area.contains_rect(&bounds, EPS), "{bounds:?}");
        assert!(bounds.w <= 6.0 + EPS);
        assert!(spine.title_size_pt >= 10.0, "{font:?}: {}", spine.title_size_pt);
    }
}

// ----- author -----------------------------------------------------------------------------

#[test]
fn the_author_gives_way_to_a_long_title() {
    let font = serif();
    let words: Vec<&str> = LONG.split(' ').collect();

    // A title that fills the length at 14 pt with room to spare only below it: the title
    // is reduced, but not below 9 pt.
    let mut seen_reduced = false;
    let mut seen_dropped = false;
    for n in 4..40 {
        let b = book(&words[..n].join(" "), &["Daniel Defoe"]);
        let spine = run(&request(&b, None, &font, &font, BottomToTop));
        let reduced = has_note(&spine, |note| *note == SpineNote::TitleReducedForAuthor);
        let dropped = has_note(&spine, |note| *note == SpineNote::AuthorDropped);
        assert_eq!(dropped, spine.author.is_none(), "{n} words");
        if reduced {
            seen_reduced = true;
            assert!(spine.title_size_pt >= 11.0 && spine.title_size_pt < 13.0);
            assert!(spine.author.is_some());
            assert!(!seen_dropped, "reduction comes before dropping");
        }
        if dropped {
            seen_dropped = true;
            // Without the author the title is as large as it can be.
            let alone = book(&words[..n].join(" "), &[]);
            let alone = run(&request(&alone, None, &font, &font, BottomToTop));
            assert_eq!(spine.title_size_pt, alone.title_size_pt);
        }
    }
    assert!(seen_reduced && seen_dropped);
}

#[test]
fn several_authors_are_reduced_to_the_first_when_the_pair_does_not_fit() {
    let font = serif();
    let b = book(
        "Les Aventures extraordinaires d'un savant russe",
        &["Georges Le Faure", "Henry de Graffigny", "Camille Flammarion"],
    );
    let spine = run(&request(&b, None, &font, &font, BottomToTop));
    let author = spine.author.as_ref().expect("author");
    assert_eq!(author.text, "Georges Le Faure");
    assert!(has_note(
        &spine,
        |n| matches!(n, SpineNote::AuthorShortened { set } if set == "Georges Le Faure")
    ));

    let options = SpineOptions { show_author: false, ..SpineOptions::default() };
    let (spine, _) = run_with(&request(&b, None, &font, &font, BottomToTop), &options);
    assert!(spine.author.is_none());
    assert!(!has_note(&spine, |n| *n == SpineNote::AuthorDropped));
}

// ----- glyph coverage ---------------------------------------------------------------------

#[test]
fn a_title_the_font_cannot_draw_is_reported_not_printed_as_boxes() {
    let latin_only = sans();
    let cyrillic = serif();
    assert!(!latin_only.missing_chars("Война и мир").is_empty(), "the test needs a gap");
    assert!(cyrillic.missing_chars("Война и мир").is_empty());

    // Nothing drawable: background only, with notes.
    let b = book("Война и мир", &["Лев Толстой"]);
    let spine = run(&request(&b, None, &latin_only, &latin_only, TopToBottom));
    assert!(spine.title.is_none() && spine.author.is_none());
    assert_eq!(spine.items.len(), 1);
    assert_eq!(spine.title_size_pt, 0.0);
    assert!(has_note(&spine, |n| *n == SpineNote::TitleNotDrawable));
    assert!(has_note(
        &spine,
        |n| matches!(n, SpineNote::MissingGlyphs { role: TextRole::Title, characters }
            if characters.contains("'В' (U+0412)"))
    ));

    // With a fallback font that covers the script the title is set, and the note says so.
    let options = SpineOptions { fallback_font: Some(cyrillic.clone()), ..Default::default() };
    let (spine, _) = run_with(&request(&b, None, &latin_only, &latin_only, TopToBottom), &options);
    assert_eq!(spine.title.as_ref().expect("title").text, "Война и мир");
    assert_eq!(spine.author.as_ref().expect("author").text, "Лев Толстой");
    assert!(has_note(&spine, |n| matches!(
        n,
        SpineNote::FallbackFont { role: TextRole::Title, .. }
    )));
    assert!(has_note(&spine, |n| matches!(
        n,
        SpineNote::FallbackFont { role: TextRole::Author, .. }
    )));
    assert!(!has_note(&spine, |n| matches!(n, SpineNote::MissingGlyphs { .. })));
    // The outline is the fallback font's.
    let direct = run(&request(&b, None, &cyrillic, &cyrillic, TopToBottom));
    assert_eq!(
        filled_path(&spine, spine.title.as_ref().expect("title").item).0,
        filled_path(&direct, direct.title.as_ref().expect("title").item).0
    );

    // Partly drawable: the rest is left out and listed; no font covers Greek, CJK, emoji.
    for (title, kept) in [("1Q84 村上春樹", "1Q84"), ("Ωmega", "mega"), ("Zoo 🦓🦒", "Zoo")]
    {
        let b = book(title, &[]);
        let (spine, _) = run_with(&request(&b, None, &cyrillic, &cyrillic, TopToBottom), &options);
        assert_eq!(spine.title.as_ref().expect("title").text, kept);
        assert!(has_note(&spine, |n| matches!(n, SpineNote::MissingGlyphs { .. })), "{title}");
    }
}

// ----- colours ----------------------------------------------------------------------------

#[test]
fn light_covers_get_black_text_and_dark_covers_white_text() {
    let b = book("Candide", &["Voltaire"]);
    let font = serif();
    let cases: [([u8; 3], Rgb8, Color); 6] = [
        ([250, 245, 230], Rgb8::BLACK, Color::BLACK),
        ([255, 255, 255], Rgb8::BLACK, Color::BLACK),
        ([255, 221, 0], Rgb8::BLACK, Color::BLACK),
        ([18, 28, 70], Rgb8::WHITE, Color::WHITE),
        ([0, 0, 0], Rgb8::WHITE, Color::WHITE),
        ([150, 20, 30], Rgb8::WHITE, Color::WHITE),
    ];
    for (ground, text, paint) in cases {
        let cover = flat_cover(ground);
        let spine = run(&request(&b, Some(&cover), &font, &font, BottomToTop));
        assert_eq!(spine.background, Rgb8::new(ground[0], ground[1], ground[2]));
        assert_eq!(spine.background_kind, BackgroundKind::EdgeColour);
        assert_eq!(spine.text_color, text, "{ground:?}");
        assert!(spine.contrast >= 4.5, "{ground:?}: {}", spine.contrast);
        assert_eq!(spine.contrast, contrast_ratio(spine.text_color, spine.background));
        // Text is K-only black or paper white — CMYK, never RGB — in both lines.
        for placed in [spine.title.as_ref(), spine.author.as_ref()].into_iter().flatten() {
            assert_eq!(filled_path(&spine, placed.item).1, paint);
        }
        // The ground is the cover's colour, as sRGB like the cover image beside it.
        let (_, fill) = filled_path(&spine, 0);
        assert_eq!(fill, Color::rgb8(ground[0], ground[1], ground[2]));
        assert!(spine.details.is_empty(), "{:?}", spine.notes);
    }
}

#[test]
fn any_flat_ground_reaches_the_minimum_contrast() {
    let b = book("Candide", &[]);
    let font = sans();
    for r in (0..=255u8).step_by(51) {
        for g in (0..=255u8).step_by(51) {
            for bl in (0..=255u8).step_by(51) {
                let options = SpineOptions {
                    background: BackgroundStyle::Solid(Rgb8::new(r, g, bl)),
                    ..Default::default()
                };
                let (spine, _) = run_with(&request(&b, None, &font, &font, TopToBottom), &options);
                assert_eq!(spine.background_kind, BackgroundKind::Custom);
                assert!(spine.contrast >= 4.5, "#{r:02x}{g:02x}{bl:02x}: {}", spine.contrast);
                assert!(!has_note(&spine, |n| matches!(n, SpineNote::LowContrast { .. })));
            }
        }
    }
    // A mid-tone carries the print caveat.
    let options = SpineOptions {
        background: BackgroundStyle::Solid(Rgb8::new(128, 128, 128)),
        ..Default::default()
    };
    let (spine, _) = run_with(&request(&b, None, &font, &font, TopToBottom), &options);
    assert!(has_note(&spine, |n| *n == SpineNote::MidToneBackground));
}

#[test]
fn a_preferred_text_colour_is_used_only_if_it_contrasts_enough() {
    let b = book("Candide", &[]);
    let font = serif();
    let navy = flat_cover([18, 28, 70]);
    let gold = Rgb8::new(240, 200, 80);
    let options = SpineOptions { text_color: Some(gold), ..Default::default() };
    let (spine, _) = run_with(&request(&b, Some(&navy), &font, &font, TopToBottom), &options);
    assert_eq!(spine.text_color, gold);
    assert!(spine.contrast >= 4.5);
    let paint = filled_path(&spine, spine.title.as_ref().expect("title").item).1;
    assert!(matches!(paint, Color::Cmyk { k, .. } if k < 0.2), "a CMYK colour: {paint:?}");
    assert!(spine.details.is_empty());

    let cream = flat_cover([250, 245, 230]);
    let (spine, _) = run_with(&request(&b, Some(&cream), &font, &font, TopToBottom), &options);
    assert_eq!(spine.text_color, Rgb8::BLACK);
    assert!(has_note(
        &spine,
        |n| matches!(n, SpineNote::TextColourRejected { ratio } if *ratio < 2.0)
    ));

    // An unreachable minimum is reported, not hidden.
    let options = SpineOptions { min_contrast: 12.0, ..Default::default() };
    let grey = flat_cover([128, 128, 128]);
    let (spine, _) = run_with(&request(&b, Some(&grey), &font, &font, TopToBottom), &options);
    assert!(has_note(&spine, |n| matches!(n, SpineNote::LowContrast { ratio } if *ratio > 5.0)));
}

#[test]
fn the_ground_can_be_emitted_as_cmyk() {
    let b = book("Candide", &[]);
    let font = serif();
    let cover = flat_cover([200, 30, 40]);
    let options = SpineOptions { background_paint: BackgroundPaint::Cmyk, ..Default::default() };
    let (spine, _) = run_with(&request(&b, Some(&cover), &font, &font, TopToBottom), &options);
    assert!(matches!(filled_path(&spine, 0).1, Color::Cmyk { .. }));
}

// ----- background rule --------------------------------------------------------------------

#[test]
fn a_plain_binding_edge_is_continued_as_its_colour() {
    let b = book("Candide", &[]);
    let font = serif();
    let cover = banded_cover([180, 30, 40], 40, 5);
    let spine = run(&request(&b, Some(&cover), &font, &font, BottomToTop));
    assert_eq!(spine.background_kind, BackgroundKind::EdgeColour);
    assert_eq!(spine.background, Rgb8::new(180, 30, 40));
    assert_eq!(spine.text_color, Rgb8::WHITE);
}

#[test]
fn a_framed_cover_gives_the_frame_colour() {
    let b = book("Candide", &[]);
    let font = serif();
    // The frame (14 px = 4.2 % of the width at the left... but only 2.6 % of the height)
    // is thinner than the 4 % strip in places once the picture's corners are counted;
    // use a 10 px frame (3 %) so that the strip certainly straddles frame and picture.
    for (frame, colour) in [([255, 255, 255], Rgb8::WHITE), ([12, 60, 40], Rgb8::new(12, 60, 40))] {
        let cover = framed_cover(frame, 10, 3);
        let spine = run(&request(&b, Some(&cover), &font, &font, BottomToTop));
        assert_eq!(spine.background_kind, BackgroundKind::BorderColour, "{frame:?}");
        assert_eq!(spine.background, colour);
        assert!(spine.contrast >= 4.5);
    }
    let white = framed_cover([255, 255, 255], 10, 3);
    let spine = run(&request(&b, Some(&white), &font, &font, BottomToTop));
    assert_eq!(spine.text_color, Rgb8::BLACK);
}

#[test]
fn a_busy_cover_gets_a_flat_dominant_colour_never_an_image() {
    let b = book("Le Horla", &["Guy de Maupassant"]);
    let font = serif();
    for seed in 1..=12 {
        let cover = busy_cover(seed);
        let mut doc = Document::default();
        let spine = design(&request(&b, Some(&cover), &font, &font, BottomToTop), &mut doc)
            .expect("design");
        assert_eq!(spine.background_kind, BackgroundKind::DominantColour, "seed {seed}");
        assert!(doc.images.is_empty(), "no pixels of the cover on the spine");
        assert!(!spine.requires_authorisation);
        assert!(!spine.items.iter().any(|item| matches!(item, Item::Image { .. })));
        assert!(!has_note(&spine, |n| *n == SpineNote::AuthorisationRequired));
        // The colour is one of the cover's five main swatches, covering ≥ 10 % of it.
        let swatches = palette(&cover, 5);
        let swatch = swatches
            .iter()
            .find(|s| s.color == spine.background)
            .unwrap_or_else(|| panic!("seed {seed}: {:?} not in {swatches:?}", spine.background));
        assert!(swatch.share >= 0.10);
        assert!(spine.contrast >= 4.5);
        assert!(spine.title.is_some());
    }

    // The explicit style takes the largest swatch.
    let cover = busy_cover(1);
    let options =
        SpineOptions { background: BackgroundStyle::DominantColour, ..Default::default() };
    let (spine, doc) = run_with(&request(&b, Some(&cover), &font, &font, BottomToTop), &options);
    assert_eq!(spine.background, palette(&cover, 5)[0].color);
    assert_eq!(spine.background_kind, BackgroundKind::DominantColour);
    assert_eq!(spine.text_color, Rgb8::WHITE, "the generated cover is mostly dark blue");
    assert!(doc.images.is_empty());
}

#[test]
fn the_default_never_needs_authorisation_whatever_the_cover() {
    let b = book("Candide", &["Voltaire"]);
    let font = serif();
    let tiny = Pixmap::filled(1, 1, [9, 9, 9, 255]);
    let transparent = Pixmap::filled(40, 60, [200, 0, 0, 0]);
    let sliver = Pixmap::filled(20_000, 20, [30, 90, 60, 255]);
    let column = Pixmap::filled(3, 4_000, [240, 240, 200, 255]);
    let mut all: Vec<Pixmap> = covers().into_iter().filter_map(|(_, c)| c).collect();
    all.extend([tiny, transparent, sliver, column]);
    for cover in &all {
        for direction in BOTH {
            let spine = run(&request(&b, Some(cover), &font, &font, direction));
            assert!(spine.background_kind.is_flat());
            assert!(!spine.requires_authorisation);
            assert!(matches!(&spine.items[0], Item::Path { .. }));
            assert!(spine.contrast >= 4.5);
        }
    }
    // A fully transparent cover is paper white.
    let transparent = Pixmap::filled(40, 60, [200, 0, 0, 0]);
    let spine = run(&request(&b, Some(&transparent), &font, &font, TopToBottom));
    assert_eq!(spine.background, Rgb8::WHITE);
}

#[test]
fn without_a_cover_the_spine_is_white_with_a_note() {
    let b = book("Candide", &[]);
    let font = serif();
    let spine = run(&request(&b, None, &font, &font, TopToBottom));
    assert_eq!(spine.background_kind, BackgroundKind::Neutral);
    assert_eq!(spine.background, Rgb8::WHITE);
    assert_eq!(spine.text_color, Rgb8::BLACK);
    assert_eq!(spine.details, vec![SpineNote::NoCover]);
    assert_eq!(spine.notes, vec!["No cover was given: the spine is plain white.".to_owned()]);
}

// ----- the opt-in continuation ------------------------------------------------------------

fn raw_rgb(doc: &Document) -> (&[u8], u32, u32) {
    assert_eq!(doc.images.len(), 1);
    match &doc.images[0].data {
        ebc_scene::ImageData::Raw { pixels, width_px, height_px, format } => {
            assert_eq!(*format, ebc_scene::PixelFormat::Rgb8);
            assert_eq!(pixels.len(), (*width_px * *height_px * 3) as usize);
            (pixels, *width_px, *height_px)
        }
        other => panic!("unexpected image data {other:?}"),
    }
}

#[test]
fn edge_continuation_is_an_explicit_opt_in_that_is_flagged() {
    let b = book("Le Horla", &["Guy de Maupassant"]);
    let font = serif();
    let cover = busy_cover(4);
    let options =
        SpineOptions { background: BackgroundStyle::EdgeContinuation, ..Default::default() };
    let req = request(&b, Some(&cover), &font, &font, BottomToTop);
    let (spine, doc) = run_with(&req, &options);

    assert_eq!(spine.background_kind, BackgroundKind::EdgeContinuation);
    assert!(spine.requires_authorisation);
    assert!(has_note(&spine, |n| *n == SpineNote::AuthorisationRequired));
    assert!(spine.notes.iter().any(|n| n.contains("authorisation")));

    // One image, covering the panel with its bleed, at 300 ppi.
    let (rect, ..) = poche();
    match &spine.items[0] {
        Item::Image { rect: placed, image } => {
            assert_eq!(*placed, rect);
            assert_eq!(*image, doc.images[0].id);
        }
        other => panic!("the background is not an image: {other:?}"),
    }
    let (pixels, w, h) = raw_rgb(&doc);
    assert_eq!(w, 119, "10 mm at 300 ppi");
    assert!((2170..=2180).contains(&h), "184 mm at 300 ppi: {h}");

    // Strongly blurred: neighbouring pixels differ very little, although the cover's
    // blocks differ by hundreds of levels.
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 3) as usize;
        [i32::from(pixels[i]), i32::from(pixels[i + 1]), i32::from(pixels[i + 2])]
    };
    let mut steepest = 0;
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let (p, right, below) = (px(x, y), px(x + 1, y), px(x, y + 1));
            for c in 0..3 {
                steepest = steepest.max((p[c] - right[c]).abs()).max((p[c] - below[c]).abs());
            }
        }
    }
    assert!(steepest <= 6, "steepest step between neighbours: {steepest}");
    // The bleed rows replicate the edge rows.
    assert_eq!(px(5, 0), px(5, 30));
    assert_eq!(px(5, h - 1), px(5, h - 31));

    // The text still reads: against the worst part of the image, or on a band.
    assert!(spine.contrast >= 4.5, "{}", spine.contrast);
    let area = text_area(&req, &options).expect("valid");
    assert!(area.contains_rect(&title_bounds(&spine), EPS));
    for bounds in painted_bounds(&spine.items) {
        assert!(rect.contains_rect(&bounds, EPS));
    }
}

#[test]
fn a_continuation_too_uneven_for_the_text_gets_a_band() {
    // Top half white, bottom half black: neither black nor white text reads on both.
    let mut cover = flat_cover([255, 255, 255]);
    fill(&mut cover, 0, COVER_H / 2, COVER_W, COVER_H, [0, 0, 0]);
    let b = book("Le Rouge et le Noir", &["Stendhal"]);
    let font = serif();
    let options =
        SpineOptions { background: BackgroundStyle::EdgeContinuation, ..Default::default() };
    let (spine, _) = run_with(&request(&b, Some(&cover), &font, &font, BottomToTop), &options);
    assert!(has_note(&spine, |n| *n == SpineNote::ContrastBand));
    assert!(spine.contrast >= 4.5);
    // image, band, author, title
    assert_eq!(spine.items.len(), 4);
    let band = painted_bounds(&spine.items)[1];
    let ink = title_bounds(&spine).union(&spine.author.as_ref().expect("author").bounds);
    assert!(band.contains_rect(&ink, EPS));
    let (rect, ..) = poche();
    assert!(rect.contains_rect(&band, EPS));

    // The same cover under the default rule: not uniform, no frame → a flat colour.
    let spine = run(&request(&b, Some(&cover), &font, &font, BottomToTop));
    assert_eq!(spine.background_kind, BackgroundKind::DominantColour);
}

#[test]
fn continuation_without_a_cover_falls_back_to_a_flat_ground() {
    let b = book("Candide", &[]);
    let font = serif();
    let options =
        SpineOptions { background: BackgroundStyle::EdgeContinuation, ..Default::default() };
    let (spine, doc) = run_with(&request(&b, None, &font, &font, TopToBottom), &options);
    assert!(doc.images.is_empty());
    assert!(!spine.requires_authorisation);
    assert_eq!(spine.background_kind, BackgroundKind::Neutral);
    assert!(has_note(&spine, |n| *n == SpineNote::ContinuationWithoutCover));
}

// ----- robustness -------------------------------------------------------------------------

#[test]
fn the_same_request_gives_the_same_design() {
    let b = book("Les Misérables", &["Victor Hugo"]);
    let font = serif();
    let cover = busy_cover(9);
    for background in [BackgroundStyle::Auto, BackgroundStyle::EdgeContinuation] {
        let options = SpineOptions { background, ..Default::default() };
        let req = request(&b, Some(&cover), &font, &font, BottomToTop);
        let (first, doc1) = run_with(&req, &options);
        let (second, doc2) = run_with(&req, &options);
        assert_eq!(first, second);
        assert_eq!(doc1, doc2);
    }
}

#[test]
fn unusable_geometry_and_options_are_errors() {
    let b = book("Candide", &[]);
    let font = serif();
    let base = request(&b, None, &font, &font, TopToBottom);
    let mut doc = Document::default();
    let rects = [
        Rect::new(f64::NAN, 0.0, 10.0, 184.0),
        Rect::new(0.0, 0.0, f64::INFINITY, 184.0),
        Rect::new(0.0, 0.0, 0.0, 184.0),
        Rect::new(0.0, 0.0, -10.0, 184.0),
        Rect::new(0.0, 0.0, 10.0, 0.0),
        Rect::new(0.0, 0.0, 10.0, 1.0e9),
        Rect::new(1.0e12, 0.0, 10.0, 184.0),
    ];
    for rect in rects {
        let result = design(&SpineRequest { rect, ..base }, &mut doc);
        assert!(matches!(result, Err(SpineError::InvalidGeometry(_))), "{rect:?}");
    }
    for bleed in [-1.0, f64::NAN, 92.0, 500.0] {
        let result = design(&SpineRequest { bleed, ..base }, &mut doc);
        assert!(matches!(result, Err(SpineError::InvalidGeometry(_))), "bleed {bleed}");
    }
    for safe_margin in [-0.1, f64::NAN, f64::INFINITY] {
        let result = design(&SpineRequest { safe_margin, ..base }, &mut doc);
        assert!(matches!(result, Err(SpineError::InvalidGeometry(_))), "margin {safe_margin}");
    }
    let bad_options = [
        SpineOptions { min_size_pt: 20.0, ..Default::default() },
        SpineOptions { min_size_pt: 0.0, ..Default::default() },
        SpineOptions { max_size_pt: f64::NAN, ..Default::default() },
        SpineOptions { max_size_pt: 1.0e9, ..Default::default() },
        SpineOptions { max_tightening_em: -0.1, ..Default::default() },
        SpineOptions { end_margin: -1.0, ..Default::default() },
        SpineOptions { author_scale: 0.0, ..Default::default() },
        SpineOptions { author_max_share: 2.0, ..Default::default() },
        SpineOptions { min_contrast: f64::NAN, ..Default::default() },
        SpineOptions { image_ppi: 0.0, ..Default::default() },
    ];
    for options in bad_options {
        let result = design_with(&base, &options, &mut doc);
        assert!(matches!(result, Err(SpineError::InvalidOptions(_))), "{options:?}");
        assert!(text_area(&base, &options).is_err());
    }
    assert!(doc.images.is_empty());
}

#[test]
fn margins_larger_than_the_spine_leave_a_background_without_text() {
    let b = book("Candide", &["Voltaire"]);
    let font = serif();
    let base = request(&b, None, &font, &font, TopToBottom);
    // No band across, or no length along.
    let across = SpineRequest { safe_margin: 6.0, ..base };
    let spine = run(&across);
    assert!(spine.title.is_none());
    assert!(has_note(&spine, |n| *n == SpineNote::NoRoomForText));
    assert_eq!(painted_bounds(&spine.items), vec![base.rect]);

    let options = SpineOptions { end_margin: 500.0, ..Default::default() };
    let (spine, _) = run_with(&base, &options);
    assert!(spine.title.is_none());
    assert!(spine.notes.iter().any(|n| n.contains("too narrow or too short")));

    // No bleed at all is fine: the background is the trimmed panel.
    let flush = SpineRequest { rect: Rect::new(110.0, 0.0, 10.0, 178.0), bleed: 0.0, ..base };
    let spine = run(&flush);
    assert_eq!(painted_bounds(&spine.items)[0], flush.rect);
    assert!(spine.title.is_some());
}

#[test]
fn a_provided_source_is_not_needed_for_the_default() {
    // `SpineSource::Synthesize` is what `request` builds; make the default explicit.
    let b = book("Candide", &[]);
    let font = serif();
    let req = request(&b, None, &font, &font, TopToBottom);
    assert!(matches!(req.source, SpineSource::Synthesize));
    assert_eq!(SpineOptions::default().background, BackgroundStyle::Auto);
    assert_eq!(BackgroundStyle::default(), BackgroundStyle::Auto);
}

// ----- the 8 pt floor of outlined and reversed text ---------------------------------------

/// `docs/research/03-print-requirements.md` §4.4 and implication 27: outlined text, and
/// reversed text, 8 pt or more. Whatever the title, author, width and options, nothing is
/// set smaller; an author that would be smaller is left out.
#[test]
fn no_text_is_set_below_the_outlined_floor_even_when_options_ask_for_less() {
    let font = serif();
    let navy = flat_cover([18, 28, 70]);
    let loose = SpineOptions {
        min_size_pt: 6.0,
        author_min_size_pt: 5.0,
        min_title_size_with_author_pt: 6.0,
        ..SpineOptions::default()
    };
    let words: Vec<&str> = LONG.split(' ').collect();
    for width in [5.0, 6.0, 7.0, 10.0, 15.0] {
        let format = CardFormat::poche().with_spine(width).expect("valid width");
        let profile = PrintProfile::professional_eu();
        let rect = format.outside().with_bleed(profile.bleed).spine;
        for n in [1, 3, 8, 20, words.len()] {
            let b = book(&words[..n].join(" "), &["Marie-Hélène de La Tour-Dupont"]);
            for opts in [SpineOptions::default(), loose.clone()] {
                let req = SpineRequest {
                    rect,
                    bleed: profile.bleed,
                    safe_margin: profile.spine_safe_margin,
                    ..request(&b, Some(&navy), &font, &font, BottomToTop)
                };
                let (spine, _) = run_with(&req, &opts);
                for text in spine.title.iter().chain(spine.author.iter()) {
                    assert!(text.size_pt >= MIN_OUTLINED_PT, "{width} mm, {n} words: {text:?}");
                }
                assert!(
                    !has_note(&spine, |n| matches!(n, SpineNote::SmallReversedText { .. })),
                    "{width} mm, {n} words"
                );
                if width >= 10.0 {
                    let title = spine.title.as_ref().expect("a 10 mm spine has a title");
                    assert!(title.size_pt >= 9.0 || opts.min_size_pt < 9.0, "{title:?}");
                }
            }
        }
    }
}
