//! Adapting a provided spine or a full wraparound cover (explicit opt-ins that need the
//! rights-holder's authorisation), and falling back to a typeset spine.
//!
//! All images are generated; widths are given "at the height of the card": the images
//! are as high as the card's spine (178 mm at 300 ppi), so a "35 mm spine" is 35 mm wide
//! once scaled to the panel.

mod common;

use common::*;
use ebc_color::Rgb8;
use ebc_i18n::SpineDirection::BottomToTop;
use ebc_model::BookMeta;
use ebc_scene::{Color, Document, ImageData, Item, Pixmap, Rect};
use ebc_spine::{
    Adaptation, BackgroundKind, Incompatibility, SpineDesign, SpineError, SpineNote, SpineRequest,
    SpineSource, design,
};
use ebc_text::Font;

const EPS: f64 = 1e-6;
const CARD_H: f64 = 178.0;
const RED: [u8; 3] = [190, 40, 45];

fn run(book: &BookMeta, font: &Font, source: SpineSource<'_>) -> (SpineDesign, Document) {
    let mut doc = Document::default();
    let req = SpineRequest { source, ..request(book, None, font, font, BottomToTop) };
    let spine = design(&req, &mut doc).expect("the design succeeds");
    (spine, doc)
}

fn provided(image: &Pixmap) -> (SpineDesign, Document) {
    run(&book("Germinal", &["Émile Zola"]), &serif(), SpineSource::Provided(image))
}

fn adaptation(spine: &SpineDesign) -> Adaptation {
    spine
        .details
        .iter()
        .find_map(|n| match n {
            SpineNote::ProvidedSpineAdapted(how) => Some(*how),
            _ => None,
        })
        .unwrap_or_else(|| panic!("not adapted: {:?}", spine.notes))
}

fn incompatibility(spine: &SpineDesign) -> Incompatibility {
    spine
        .details
        .iter()
        .find_map(|n| match n {
            SpineNote::ProvidedSpineIncompatible(why) => Some(*why),
            _ => None,
        })
        .unwrap_or_else(|| panic!("not declared incompatible: {:?}", spine.notes))
}

/// Checks everything common to an adapted spine and returns where the image was placed
/// and its pixel size.
fn check_adapted(spine: &SpineDesign, doc: &Document) -> (Rect, (u32, u32)) {
    let (rect, bleed, _) = poche();
    let trim = Rect::new(rect.x, rect.y + bleed, rect.w, rect.h - 2.0 * bleed);
    assert_eq!(spine.background_kind, BackgroundKind::ProvidedSpine);
    assert!(spine.requires_authorisation);
    assert!(has_note(spine, |n| *n == SpineNote::AuthorisationRequired));
    assert!(spine.notes.iter().any(|n| n.contains("authorisation")), "{:?}", spine.notes);
    assert!(spine.title.is_none() && spine.author.is_none(), "nothing is typeset on it");
    assert_eq!(spine.title_size_pt, 0.0);
    assert_eq!(doc.images.len(), 1);

    let bounds = painted_bounds(&spine.items);
    for b in &bounds {
        assert!(rect.contains_rect(b, EPS), "{b:?} leaves the panel");
    }
    // The flat fills come first and cover the whole panel, bleed included; the image is
    // last and stays inside the trimmed panel.
    let (image_item, fills) = spine.items.split_last().expect("items");
    let Item::Image { image, rect: placed } = image_item else {
        panic!("the last item is not the image: {image_item:?}");
    };
    assert_eq!(*image, doc.images[0].id);
    assert!(trim.contains_rect(placed, EPS));
    let covered = fills
        .iter()
        .map(|item| match item {
            Item::Path { path, fill: Some((Color::Rgb { .. }, _)), .. } => {
                ebc_text::outline_bounds(path).expect("a rectangle")
            }
            other => panic!("unexpected item under the image: {other:?}"),
        })
        .reduce(|a, b| a.union(&b))
        .expect("fills");
    assert_eq!(covered, rect);
    (*placed, doc.images[0].data.dimensions_px())
}

fn trim() -> Rect {
    Rect::new(110.0, 0.0, 10.0, CARD_H)
}

// ----- compatible -------------------------------------------------------------------------

#[test]
fn a_spine_of_the_right_width_is_used_almost_as_is() {
    let image = spine_image(10.0, CARD_H, RED, 4.0);
    let (spine, doc) = provided(&image);
    let (placed, px) = check_adapted(&spine, &doc);
    assert_eq!(placed, trim());
    assert_eq!(px, (image.width(), image.height()), "the pixels are not resampled");
    assert!(matches!(adaptation(&spine), Adaptation::Stretched { percent } if percent.abs() < 0.5));
    assert_eq!(spine.background, Rgb8::new(RED[0], RED[1], RED[2]));
    assert_eq!(spine.text_color, Rgb8::WHITE);

    // The registered pixels are the image's.
    let ImageData::Raw { pixels, width_px, .. } = &doc.images[0].data else { panic!("raw") };
    let at = |x: u32, y: u32| {
        let i = ((y * width_px + x) * 3) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };
    assert_eq!(at(0, 0), RED);
    let centre = image.get(image.width() / 2, image.height() / 5 + 1).expect("pixel");
    assert_eq!(at(image.width() / 2, image.height() / 5 + 1), [centre[0], centre[1], centre[2]]);
    assert_ne!(at(image.width() / 2, image.height() / 5 + 1), RED, "lettering is there");
}

#[test]
fn up_to_eight_per_cent_is_absorbed_by_stretching_across() {
    for (width, sign) in [(10.7, -1.0), (9.3, 1.0)] {
        let image = spine_image(width, CARD_H, RED, 4.0);
        let (spine, doc) = provided(&image);
        let (placed, _) = check_adapted(&spine, &doc);
        assert_eq!(placed, trim(), "{width} mm fills the trimmed panel exactly");
        let Adaptation::Stretched { percent } = adaptation(&spine) else {
            panic!("{width} mm: {:?}", spine.notes)
        };
        assert!(percent * sign > 5.0 && percent.abs() <= 8.7, "{width} mm: {percent}");
        assert!(spine.notes[0].contains("scaled across"), "{:?}", spine.notes);
    }
}

#[test]
fn a_narrower_spine_with_plain_sides_is_padded_not_stretched() {
    let image = spine_image(7.0, CARD_H, RED, 3.0);
    let (spine, doc) = provided(&image);
    let (placed, px) = check_adapted(&spine, &doc);
    assert!(
        matches!(adaptation(&spine), Adaptation::Padded { added_mm } if (added_mm - 3.0).abs() < 0.1)
    );
    // Uniform scale: 7 mm wide, centred, full height.
    assert!((placed.w - 7.0).abs() < 0.05 && (placed.center().x - 115.0).abs() < EPS);
    assert!((placed.y - 0.0).abs() < EPS && (placed.h - CARD_H).abs() < EPS);
    assert_eq!(px, (image.width(), image.height()));
    // The padding is the spine's own side colour.
    let side_fills = spine
        .items
        .iter()
        .filter(|item| {
            matches!(item, Item::Path { fill: Some((c, _)), .. }
                if *c == Color::rgb8(RED[0], RED[1], RED[2]))
        })
        .count();
    assert_eq!(side_fills, 4, "head, foot and both sides are in the spine's colour");
}

#[test]
fn a_wider_spine_with_plain_margins_is_cropped_around_its_content() {
    // The "35 mm" case with the title set along the spine: 5 mm of lettering in 35 mm of
    // plain ground. 25 mm of ground are cropped; nothing is scaled across.
    let image = spine_image(35.0, CARD_H, RED, 5.0);
    let (spine, doc) = provided(&image);
    let (placed, px) = check_adapted(&spine, &doc);
    assert_eq!(placed, trim());
    let Adaptation::Cropped { removed_mm } = adaptation(&spine) else {
        panic!("{:?}", spine.notes)
    };
    assert!((removed_mm - 25.0).abs() < 0.1, "{removed_mm}");
    assert_eq!(px.1, image.height());
    assert!((f64::from(px.0) - 10.0 * SPINE_PX_PER_MM).abs() <= 1.0, "10 mm of pixels: {px:?}");
    assert!(spine.notes[0].contains("cropped"), "{:?}", spine.notes);

    // Off-centre lettering stays in the middle of the crop.
    let mut off_centre = Pixmap::filled(image.width(), image.height(), [240, 235, 220, 255]);
    let (x0, x1) = ((22.0 * SPINE_PX_PER_MM) as u32, (27.0 * SPINE_PX_PER_MM) as u32);
    letters(&mut off_centre, x0, x1, 300, 1800);
    let (spine, doc) = provided(&off_centre);
    check_adapted(&spine, &doc);
    assert!(matches!(adaptation(&spine), Adaptation::Cropped { .. }));
    let ImageData::Raw { pixels, width_px, .. } = &doc.images[0].data else { panic!("raw") };
    let row = 310 * *width_px as usize * 3;
    let dark: Vec<usize> = (0..*width_px as usize).filter(|x| pixels[row + x * 3] < 100).collect();
    let (first, last) = (dark[0], dark[dark.len() - 1]);
    let margin_left = first;
    let margin_right = *width_px as usize - 1 - last;
    assert!(margin_left.abs_diff(margin_right) <= 3, "{margin_left} vs {margin_right}");
    assert!((57..=61).contains(&dark.len()), "5 mm of lettering: {}", dark.len());
}

#[test]
fn a_slightly_wider_spine_that_cannot_be_cropped_is_reduced_to_fit() {
    // 12 mm of artwork up to the edges: scaled down uniformly to 10 mm; the head and
    // foot (about 15 mm each) are filled with its end colours.
    let image = busy_spine_image(12.0, CARD_H, 5.0, 11);
    let (spine, doc) = provided(&image);
    let (placed, px) = check_adapted(&spine, &doc);
    let Adaptation::Shrunk { percent } = adaptation(&spine) else { panic!("{:?}", spine.notes) };
    assert!((percent - 16.7).abs() < 0.3, "{percent}");
    assert!((placed.w - 10.0).abs() < EPS);
    assert!((placed.h - CARD_H / 1.2).abs() < 0.5, "{placed:?}");
    assert!((placed.center().y - CARD_H / 2.0).abs() < EPS);
    assert!((placed.w / placed.h - f64::from(px.0) / f64::from(px.1)).abs() < 1e-3, "uniform");

    // Plain margins but content wider than the 6 mm band: the same.
    let image = spine_image(12.0, CARD_H, RED, 8.0);
    let (spine, doc) = provided(&image);
    check_adapted(&spine, &doc);
    assert!(matches!(adaptation(&spine), Adaptation::Shrunk { .. }));

    // Plain margins and content within the band: cropped instead.
    let image = spine_image(12.0, CARD_H, RED, 5.0);
    let (spine, doc) = provided(&image);
    check_adapted(&spine, &doc);
    assert!(
        matches!(adaptation(&spine), Adaptation::Cropped { removed_mm } if (removed_mm - 2.0).abs() < 0.1)
    );
}

#[test]
fn the_spine_columns_of_a_full_wrap_are_used() {
    // back (110 mm, green) | spine (10 mm, red, lettered) | front (110 mm, blue).
    let px = |mm: f64| (mm * SPINE_PX_PER_MM).round() as u32;
    let (w, h) = (px(230.0), px(CARD_H));
    let mut wrap = Pixmap::filled(w, h, [30, 120, 60, 255]);
    fill(&mut wrap, px(110.0), 0, px(120.0), h, RED);
    fill(&mut wrap, px(120.0), 0, w, h, [20, 40, 160]);
    letters(&mut wrap, px(113.0), px(117.0), h / 5, 4 * h / 5);
    let book = book("Germinal", &[]);
    let font = serif();
    let source = SpineSource::FullWrap { image: &wrap, spine_px: (px(110.0), px(120.0)) };
    let (spine, doc) = run(&book, &font, source);
    let (placed, size) = check_adapted(&spine, &doc);
    assert_eq!(placed, trim());
    assert_eq!(size, (px(120.0) - px(110.0), h));
    assert!(matches!(adaptation(&spine), Adaptation::Stretched { .. }));
    let ImageData::Raw { pixels, .. } = &doc.images[0].data else { panic!("raw") };
    assert_eq!(&pixels[..3], &RED, "the first pixel is the spine's, not the back cover's");
    assert_eq!(&pixels[pixels.len() - 3..], &RED);

    // Ranges that are empty, reversed or outside the image are typed errors.
    for spine_px in [(5, 5), (10, 5), (0, w + 1), (w, w + 10), (u32::MAX, 0)] {
        let mut doc = Document::default();
        let req = SpineRequest {
            source: SpineSource::FullWrap { image: &wrap, spine_px },
            ..request(&book, None, &font, &font, BottomToTop)
        };
        assert_eq!(
            design(&req, &mut doc),
            Err(SpineError::InvalidSpineRange { start: spine_px.0, end: spine_px.1, width: w }),
        );
        assert!(doc.images.is_empty());
    }
}

#[test]
fn an_adapted_spine_needs_no_title() {
    let image = spine_image(10.0, CARD_H, RED, 4.0);
    let (spine, doc) = run(&BookMeta::titled(""), &serif(), SpineSource::Provided(&image));
    check_adapted(&spine, &doc);
}

// ----- incompatible: a spine is typeset instead -------------------------------------------

/// Checks the fallback: a typeset, flat spine that needs no authorisation.
fn check_typeset(spine: &SpineDesign, doc: &Document, ground: Option<[u8; 3]>) {
    assert!(doc.images.is_empty(), "no pixel of the image is used");
    assert!(!spine.requires_authorisation);
    assert!(!has_note(spine, |n| *n == SpineNote::AuthorisationRequired));
    assert!(matches!(spine.details[0], SpineNote::ProvidedSpineIncompatible(_)));
    assert!(spine.notes[0].starts_with("The spine supplied was not used ("), "{:?}", spine.notes);
    assert!(spine.notes[0].ends_with("the spine was typeset instead."));
    assert_eq!(spine.title.as_ref().expect("the title is typeset").text, "Germinal");
    assert_eq!(spine.author.as_ref().expect("the author is typeset").text, "Émile Zola");
    assert!(spine.contrast >= 4.5);
    match ground {
        Some([r, g, b]) => {
            assert_eq!(spine.background_kind, BackgroundKind::ProvidedSpineColour);
            assert_eq!(spine.background, Rgb8::new(r, g, b));
        }
        None => assert_eq!(spine.background_kind, BackgroundKind::Neutral),
    }
}

#[test]
fn a_three_millimetre_spine_is_not_stretched_to_ten() {
    // With (tiny) lettering: much too narrow.
    let image = spine_image(3.0, CARD_H, RED, 1.5);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, Some(RED));
    assert!(matches!(
        incompatibility(&spine),
        Incompatibility::TooNarrow { width_mm } if (width_mm - 3.0).abs() < 0.1
    ));
    assert!(spine.notes[0].contains("3.0 mm wide"), "{:?}", spine.notes);

    // Blank, as real 3 mm spines are: nothing to keep but the colour.
    let image = spine_image(3.0, CARD_H, RED, 0.0);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, Some(RED));
    assert_eq!(incompatibility(&spine), Incompatibility::NoText);

    // Up to half the width is still refused, even with legible lettering.
    let image = spine_image(4.8, CARD_H, RED, 3.0);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, Some(RED));
    assert!(matches!(incompatibility(&spine), Incompatibility::TooNarrow { .. }));
}

#[test]
fn a_thirty_five_millimetre_spine_with_a_title_across_it_is_not_reused() {
    // Lettering 25 mm across: cropping to 10 mm would cut it, shrinking by 71 % would
    // make it illegible.
    let image = spine_image(35.0, CARD_H, [240, 235, 220], 25.0);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, Some([240, 235, 220]));
    assert!(matches!(
        incompatibility(&spine),
        Incompatibility::TooWide { width_mm } if (width_mm - 35.0).abs() < 0.1
    ));
    assert_eq!(spine.text_color, Rgb8::BLACK);

    // Artwork up to the edges: no plain ground to reuse either.
    let image = busy_spine_image(35.0, CARD_H, 5.0, 3);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, None);
    assert!(matches!(incompatibility(&spine), Incompatibility::TooWide { .. }));
}

#[test]
fn other_incompatibilities_are_explained() {
    // Narrower than the spine with artwork up to the edges: cannot be padded.
    let image = busy_spine_image(7.0, CARD_H, 3.0, 5);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, None);
    assert!(matches!(incompatibility(&spine), Incompatibility::BusySides { .. }));

    // Lettering that would be 1 mm high once cropped.
    let image = spine_image(35.0, CARD_H, RED, 1.0);
    let (spine, doc) = provided(&image);
    check_typeset(&spine, &doc, Some(RED));
    assert!(matches!(
        incompatibility(&spine),
        Incompatibility::TextTooSmall { height_mm } if (0.7..1.4).contains(&height_mm)
    ));

    // Too few pixels: a 10 mm spine scanned at 100 ppi.
    let small = Pixmap::filled(39, 701, [RED[0], RED[1], RED[2], 255]);
    let (spine, doc) = provided(&small);
    check_typeset(&spine, &doc, Some(RED));
    assert!(matches!(
        incompatibility(&spine),
        Incompatibility::LowResolution { ppi } if (ppi - 100.0).abs() < 1.0
    ));
    assert!(spine.notes[0].contains("100 ppi"), "{:?}", spine.notes);

    // Degenerate images.
    for image in [Pixmap::filled(1, 1, [0, 0, 0, 255]), Pixmap::filled(5000, 2, [9, 9, 9, 0])] {
        let (spine, doc) = provided(&image);
        assert!(doc.images.is_empty());
        assert!(matches!(incompatibility(&spine), Incompatibility::LowResolution { .. }));
        assert!(spine.title.is_some());
    }
}

#[test]
fn the_fallback_still_needs_a_title_and_uses_the_cover_when_the_spine_has_no_plain_ground() {
    let image = busy_spine_image(35.0, CARD_H, 5.0, 3);
    let font = serif();
    let mut doc = Document::default();
    let untitled = BookMeta::titled(" ");
    let req = SpineRequest {
        source: SpineSource::Provided(&image),
        ..request(&untitled, None, &font, &font, BottomToTop)
    };
    assert_eq!(design(&req, &mut doc), Err(SpineError::EmptyTitle));

    let book = book("Germinal", &[]);
    let cover = flat_cover([18, 28, 70]);
    let req = SpineRequest {
        source: SpineSource::Provided(&image),
        ..request(&book, Some(&cover), &font, &font, BottomToTop)
    };
    let spine = design(&req, &mut doc).expect("design");
    assert_eq!(spine.background_kind, BackgroundKind::EdgeColour);
    assert_eq!(spine.background, Rgb8::new(18, 28, 70));
    assert!(doc.images.is_empty());
}

#[test]
fn adaptation_is_deterministic() {
    for image in [spine_image(35.0, CARD_H, RED, 5.0), busy_spine_image(12.0, CARD_H, 5.0, 11)] {
        let (first, doc1) = provided(&image);
        let (second, doc2) = provided(&image);
        assert_eq!(first, second);
        assert_eq!(doc1, doc2);
    }
}
