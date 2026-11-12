//! Fitting a line into a box (`fit_size`) and breaking paragraphs (`wrap`).

mod common;

use common::close;
use ebc_text::{Font, MAX_SIZE_PT, ShapeOptions, fit_size, shape, wrap, wrap_ranges};
use unicode_segmentation::UnicodeSegmentation;

fn opts() -> ShapeOptions {
    ShapeOptions::default()
}

fn width(font: &Font, text: &str, size: f64) -> f64 {
    shape(font, text, size, &opts()).width
}

// --- fit_size ----------------------------------------------------------------------

#[test]
fn fitted_size_respects_both_limits_and_is_the_largest_that_does() {
    let titles = ["L'Étranger", "Les Misérables", "Voyage au bout de la nuit — édition intégrale"];
    // (max width mm, max cap height mm, min pt, max pt)
    let boxes = [
        (150.0, 6.0, 6.0, 28.0),
        (40.0, 100.0, 2.0, 72.0),
        (1000.0, 3.0, 4.0, 72.0),
        (60.0, 4.2, 3.0, 40.0),
        (25.0, 2.0, 1.0, 400.0),
    ];
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let metrics = font.metrics();
        for title in titles {
            for (max_width, max_cap, min_pt, max_pt) in boxes {
                let Some(size) =
                    fit_size(&font, title, max_width, max_cap, min_pt, max_pt, &opts())
                else {
                    // Legitimate only if the text does not fit at the minimum size.
                    assert!(
                        width(&font, title, min_pt) > max_width
                            || metrics.cap_height_mm(min_pt) > max_cap,
                        "{label} {title:?}: None although the minimum size fits"
                    );
                    continue;
                };
                assert!((min_pt..=max_pt).contains(&size), "{label} {title:?}: {size}");
                // Exact guarantees, no tolerance.
                assert!(width(&font, title, size) <= max_width, "{label} {title:?}: width");
                assert!(metrics.cap_height_mm(size) <= max_cap, "{label} {title:?}: cap height");
                // Maximal: a hair larger breaks one of the limits (unless capped by max_pt).
                if size < max_pt {
                    let larger = size * (1.0 + 1e-9);
                    assert!(
                        width(&font, title, larger) > max_width
                            || metrics.cap_height_mm(larger) > max_cap,
                        "{label} {title:?}: {size} is not maximal"
                    );
                }
            }
        }
    }
}

#[test]
fn fitted_size_matches_the_closed_form() {
    let font = common::spectral();
    let title = "Voyage au bout de la nuit";
    // Width-limited: width is proportional to the size.
    let per_point = width(&font, title, 1.0);
    let size = fit_size(&font, title, 80.0, f64::INFINITY, 1.0, 500.0, &opts()).expect("fits");
    assert!(close(size, 80.0 / per_point, 1e-9), "{size}");
    // Cap-height-limited.
    let size = fit_size(&font, title, f64::INFINITY, 5.0, 1.0, 500.0, &opts()).expect("fits");
    assert!(close(size, font.metrics().size_for_cap_height(5.0), 1e-9), "{size}");
    // Both: the smaller of the two wins.
    let both = fit_size(&font, title, 80.0, 5.0, 1.0, 500.0, &opts()).expect("fits");
    let expected = (80.0 / per_point).min(font.metrics().size_for_cap_height(5.0));
    assert!(close(both, expected, 1e-9), "{both} vs {expected}");
}

#[test]
fn fit_returns_the_maximum_when_everything_fits_and_none_when_nothing_does() {
    let font = common::atkinson();
    assert_eq!(fit_size(&font, "Zola", 500.0, 50.0, 6.0, 20.0, &opts()), Some(20.0));
    assert_eq!(
        fit_size(&font, "Zola", f64::INFINITY, f64::INFINITY, 6.0, 20.0, &opts()),
        Some(20.0)
    );
    // Too long for 10 mm even at 6 pt.
    assert_eq!(fit_size(&font, "Les Misérables", 10.0, 50.0, 6.0, 20.0, &opts()), None);
    // Capitals of a 6 pt font are 1.4 mm tall: a 1 mm limit cannot be met.
    assert_eq!(fit_size(&font, "Zola", 500.0, 1.0, 6.0, 20.0, &opts()), None);
    assert_eq!(fit_size(&font, "Zola", 0.0, 50.0, 6.0, 20.0, &opts()), None);
    assert_eq!(fit_size(&font, "Zola", 500.0, 0.0, 6.0, 20.0, &opts()), None);

    // The limits are inclusive: a box exactly as wide as the text at the minimum fits.
    let at_min = width(&font, "Zola", 6.0);
    let size = fit_size(&font, "Zola", at_min, 50.0, 6.0, 20.0, &opts()).expect("fits exactly");
    assert!(close(size, 6.0, 1e-9), "{size}");
    // A degenerate range is fine.
    assert_eq!(fit_size(&font, "Zola", 500.0, 50.0, 12.0, 12.0, &opts()), Some(12.0));
    assert_eq!(fit_size(&font, "Zola", 1.0, 50.0, 12.0, 12.0, &opts()), None);
}

#[test]
fn fit_rejects_meaningless_arguments() {
    let font = common::atkinson();
    let fit = |w: f64, h: f64, min: f64, max: f64| fit_size(&font, "Zola", w, h, min, max, &opts());
    assert_eq!(fit(100.0, 10.0, 0.0, 20.0), None, "minimum must be positive");
    assert_eq!(fit(100.0, 10.0, -1.0, 20.0), None);
    assert_eq!(fit(100.0, 10.0, 20.0, 6.0), None, "minimum above maximum");
    assert_eq!(fit(f64::NAN, 10.0, 6.0, 20.0), None);
    assert_eq!(fit(100.0, f64::NAN, 6.0, 20.0), None);
    assert_eq!(fit(100.0, 10.0, f64::NAN, 20.0), None);
    assert_eq!(fit(100.0, 10.0, 6.0, f64::NAN), None);
    assert_eq!(fit(-1.0, 10.0, 6.0, 20.0), None);
    assert_eq!(fit(100.0, -1.0, 6.0, 20.0), None);
    assert_eq!(fit(100.0, 10.0, 6.0, f64::INFINITY), None);
    assert_eq!(fit(100.0, 10.0, f64::NEG_INFINITY, 20.0), None);
    // Sizes are capped at MAX_SIZE_PT, like in `shape`.
    assert_eq!(fit(f64::INFINITY, f64::INFINITY, 1.0, 1e12), Some(MAX_SIZE_PT));
    assert_eq!(fit(f64::INFINITY, f64::INFINITY, MAX_SIZE_PT * 2.0, 1e12), None);
}

#[test]
fn fitting_empty_text_only_looks_at_the_cap_height() {
    let font = common::bebas();
    assert_eq!(fit_size(&font, "", 0.0, 100.0, 6.0, 30.0, &opts()), Some(30.0));
    let size = fit_size(&font, "", 0.0, 4.0, 6.0, 30.0, &opts()).expect("fits");
    assert!(close(font.metrics().cap_height_mm(size), 4.0, 1e-9));
    assert!(close(size, font.metrics().size_for_cap_height(4.0), 1e-9));
}

#[test]
fn fit_honours_shaping_options_and_variation_instances() {
    let font = common::bebas();
    let title = "LA PESTE";
    let plain = fit_size(&font, title, 30.0, 100.0, 1.0, 200.0, &opts()).expect("fits");
    let spaced_opts = ShapeOptions { letter_spacing_em: 0.2, ..opts() };
    let spaced = fit_size(&font, title, 30.0, 100.0, 1.0, 200.0, &spaced_opts).expect("fits");
    assert!(spaced < plain * 0.9, "letter spacing leaves less room: {spaced} vs {plain}");
    assert!(shape(&font, title, spaced, &spaced_opts).width <= 30.0);

    // A heavier (wider) instance of a variable font must be set smaller.
    let variable = Font::from_bytes(
        common::SyntheticFont { variable: true, ..common::SyntheticFont::new() }.build(),
        0,
    )
    .expect("font");
    let regular = fit_size(&variable, "AxHA", 40.0, 100.0, 1.0, 500.0, &opts()).expect("fits");
    let black_font = variable.with_variations(&[("wght", 900.0)]);
    let black = fit_size(&black_font, "AxHA", 40.0, 100.0, 1.0, 500.0, &opts()).expect("fits");
    // Advances 2300 vs 2700 units.
    assert!(close(black / regular, 2300.0 / 2700.0, 1e-6), "{black} vs {regular}");
}

#[test]
fn spine_titles_fit_a_ten_millimetre_spine() {
    // The card's spine is 178 mm long and 10 mm thick. Following
    // docs/research/08 (implementation item 7): capitals at most 7 mm tall, type
    // never below 6 pt; 14 mm are kept clear at each end, leaving 150 mm.
    let font = common::bebas();
    let metrics = font.metrics();
    let fit = |title: &str| fit_size(&font, title, 150.0, 7.0, 6.0, 40.0, &opts());

    // A short title is limited by the thickness of the spine: 7 mm capitals.
    let short = fit("LA PESTE").expect("fits");
    assert!(close(metrics.cap_height_mm(short), 7.0, 1e-9));
    assert!(close(short, 28.346, 0.01), "{short}"); // 7 mm / (0.7 em x 0.3528 mm/pt)
    assert!(shape(&font, "LA PESTE", short, &opts()).width < 150.0);
    // A long title is limited by the length of the spine.
    let long_title = "LE TOUR DU MONDE EN QUATRE-VINGTS JOURS — ÉDITION ILLUSTRÉE ET ANNOTÉE";
    let long = fit(long_title).expect("fits");
    assert!(long < short && long >= 6.0);
    assert!(close(shape(&font, long_title, long, &opts()).width, 150.0, 1e-6));
    // An absurdly long title does not fit at 6 pt: the caller has to shorten it.
    assert_eq!(fit(&long_title.repeat(4)), None);
}

// --- wrap --------------------------------------------------------------------------

/// Plain prose whose only break opportunities are spaces.
const PROSE: &str = "Offrir un livre numérique ne devrait pas se résumer à un courriel perdu \
                     parmi tant de messages. Cette carte se glisse dans une bibliothèque comme \
                     un vrai livre de poche. Ouvrez la jaquette, scannez le code avec votre \
                     téléphone et suivez les trois étapes pour retrouver votre lecture sur \
                     votre liseuse.";

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn wrapped_lines_fit_keep_every_character_and_are_greedy() {
    for (label, bytes) in common::ALL {
        let font = common::font(bytes);
        let size = 10.0;
        let mut previous_count = usize::MAX;
        // The narrowest column still holds the longest word in every font.
        for max_width in [30.0, 40.0, 50.0, 80.0, 120.0, 400.0] {
            let lines = wrap(&font, PROSE, size, max_width, &opts());
            let context = format!("{label} at {max_width} mm");

            // Every line fits — exactly, with the same measure `shape` reports.
            for line in &lines {
                assert!(width(&font, line, size) <= max_width, "{context}: {line:?} overflows");
                assert_eq!(line.trim_end(), line, "{context}: no trailing white space");
                assert!(!line.is_empty(), "{context}: no empty line in plain prose");
            }
            // Nothing is lost or reordered; words are only separated at spaces.
            assert_eq!(lines.join(" "), PROSE, "{context}");
            // Greedy: the first word of the next line would not have fitted.
            for pair in lines.windows(2) {
                let next_word = pair[1].split(' ').next().unwrap_or_default();
                let longer = format!("{} {next_word}", pair[0]);
                assert!(
                    width(&font, &longer, size) > max_width,
                    "{context}: {:?} had room for {next_word:?}",
                    pair[0]
                );
            }
            // More room, fewer (or as many) lines; and a plausible number of them.
            assert!(lines.len() <= previous_count, "{context}");
            previous_count = lines.len();
            let minimum = (width(&font, PROSE, size) / max_width).floor() as usize;
            assert!(lines.len() >= minimum && lines.len() <= 2 * minimum + 2, "{context}");
        }
    }
}

#[test]
fn ranges_cover_the_text_and_match_the_lines() {
    let text = "  Pour Camille,\n\nJoyeux anniversaire\u{a0}! Voici «\u{a0}Vingt mille lieues \
                sous les mers\u{a0}», à lire au coin du feu — ou sous la couette…\n\tMamie  \n";
    let font = common::atkinson();
    for max_width in [0.0, 5.0, 18.0, 30.0, 60.0, 500.0, f64::INFINITY] {
        let ranges = wrap_ranges(&font, text, 11.0, max_width, &opts());
        let lines = wrap(&font, text, 11.0, max_width, &opts());
        assert_eq!(ranges.len(), lines.len());
        let mut at = 0;
        for (range, line) in ranges.iter().zip(&lines) {
            assert_eq!(range.start, at, "contiguous at {max_width}");
            assert!(range.end > range.start);
            assert_eq!(text[range.clone()].trim_end(), line, "line is the trimmed range");
            at = range.end;
        }
        assert_eq!(at, text.len(), "ranges cover the whole text at {max_width}");
        assert_eq!(squeeze(&lines.concat()), squeeze(text), "no character lost at {max_width}");
    }
}

#[test]
fn mandatory_breaks_and_blank_lines_are_kept() {
    let font = common::atkinson();
    let message = "Pour Léa,\n\nJoyeux anniversaire !\r\nMamie\n";
    assert_eq!(
        wrap(&font, message, 11.0, 200.0, &opts()),
        ["Pour Léa,", "", "Joyeux anniversaire !", "Mamie"]
    );
    // Narrow: the second paragraph wraps, the blank line stays. (The "!" stays with
    // its word even though the author typed an ordinary space before it.)
    let column = width(&font, "anniversaire !", 11.0) + 0.2;
    let narrow = wrap(&font, message, 11.0, column, &opts());
    assert_eq!(narrow, ["Pour Léa,", "", "Joyeux", "anniversaire !", "Mamie"]);
    assert!(wrap(&font, "", 11.0, 50.0, &opts()).is_empty());
    assert_eq!(wrap(&font, "\n", 11.0, 50.0, &opts()), [""]);
    assert_eq!(wrap(&font, "   ", 11.0, 50.0, &opts()), [""]);
    assert_eq!(wrap(&font, "  retrait", 11.0, 50.0, &opts()), ["  retrait"]);
}

#[test]
fn french_punctuation_is_not_separated_from_its_word() {
    let font = common::spectral();
    let text = "Quelle surprise\u{a0}! Tu as lu «\u{a0}Germinal\u{a0}»\u{a0}? Non\u{202f}; \
                pas encore\u{a0}: bientôt.";
    // Columns at least as wide as the longest glued group, so that nothing is cut.
    let widest = text.split(' ').map(|group| width(&font, group, 11.0)).fold(0.0, f64::max);
    for extra in [0.1, 4.0, 9.0, 17.0, 30.0, 55.0] {
        let max_width = widest + extra;
        let lines = wrap(&font, text, 11.0, max_width, &opts());
        assert_eq!(lines.join(" "), text, "{max_width}: only broken at ordinary spaces");
        for line in lines {
            let first = line.chars().next().expect("non-empty line");
            let last = line.chars().last().expect("non-empty line");
            assert!(!"!?;:»\u{a0}\u{202f}".contains(first), "{max_width}: line starts {line:?}");
            assert!(!"«\u{a0}\u{202f}".contains(last), "{max_width}: line ends {line:?}");
        }
    }
}

#[test]
fn words_longer_than_the_line_are_cut_between_letters() {
    let font = common::atkinson();
    let word = "Anticonstitutionnellement";
    let size = 12.0;
    let max_width = 18.0;
    assert!(width(&font, word, size) > 2.0 * max_width);
    let lines = wrap(&font, word, size, max_width, &opts());
    assert!(lines.len() >= 3);
    assert_eq!(lines.concat(), word, "nothing lost, nothing added");
    for (k, line) in lines.iter().enumerate() {
        assert!(width(&font, line, size) <= max_width, "{line:?}");
        // As late as possible: one more letter would overflow.
        if let Some(next) = lines.get(k + 1) {
            let one_more = format!("{line}{}", next.chars().next().expect("letter"));
            assert!(width(&font, &one_more, size) > max_width, "{line:?} could take one more");
        }
    }

    // Inside a sentence: the words around it are untouched, and the tail of the cut
    // word shares its line with what follows.
    let sentence = format!("Un mot : {word}, voilà.");
    let max_width = width(&font, "Un mot :", size) + 0.5;
    let lines = wrap(&font, &sentence, size, max_width, &opts());
    assert_eq!(squeeze(&lines.concat()), squeeze(&sentence));
    assert!(lines.iter().all(|l| width(&font, l, size) <= max_width));
    assert_eq!(lines.first().map(String::as_str), Some("Un mot :"));
    assert!(lines.last().is_some_and(|l| l.ends_with("voilà.")), "{lines:?}");
    assert!(lines.len() >= 4, "{lines:?}");
}

#[test]
fn soft_hyphens_become_visible_only_at_a_break() {
    let font = common::atkinson();
    let word = "extra\u{ad}ordinaire\u{ad}ment";
    let size = 12.0;
    // Wide enough for "extraordinaire-" but not for the whole word.
    let max_width = width(&font, "extraordinaire-", size) + 0.1;
    let lines = wrap(&font, word, size, max_width, &opts());
    assert_eq!(lines, ["extra\u{ad}ordinaire-", "ment"]);
    assert!(lines.iter().all(|l| width(&font, l, size) <= max_width));
    // Narrower: both hyphenation points are used.
    let max_width = width(&font, "ordinaire-", size) + 0.1;
    assert_eq!(wrap(&font, word, size, max_width, &opts()), ["extra-", "ordinaire-", "ment"]);
    // Wide: untouched.
    assert_eq!(wrap(&font, word, size, 500.0, &opts()), [word]);
}

#[test]
fn degenerate_widths_never_lose_text_or_split_a_cluster() {
    let font = common::atkinson();
    let text = "Été à Zürich 👩‍👩‍👧‍👦 e\u{301}";
    let clusters: Vec<&str> =
        text.graphemes(true).filter(|g| !g.chars().all(char::is_whitespace)).collect();
    for max_width in [0.0, -10.0, 1e-9, f64::MIN_POSITIVE] {
        let lines = wrap(&font, text, 12.0, max_width, &opts());
        assert_eq!(lines, clusters, "one cluster per line at width {max_width}");
    }
    // The family emoji is 7 code points joined by ZWJ: it must stay in one piece.
    assert!(clusters.contains(&"👩‍👩‍👧‍👦"));
    assert!(clusters.contains(&"e\u{301}"));

    // No limit: only mandatory breaks.
    for max_width in [f64::INFINITY, f64::NAN] {
        assert_eq!(wrap(&font, "a b c\nd e", 12.0, max_width, &opts()), ["a b c", "d e"]);
    }
    // Size zero (or nonsense): everything has zero width and fits.
    for size in [0.0, -3.0, f64::NAN] {
        assert_eq!(wrap(&font, "a b c\nd e", size, 5.0, &opts()), ["a b c", "d e"]);
    }
}

#[test]
fn text_the_font_cannot_draw_still_wraps() {
    // Ideographs are missing from the font (drawn as .notdef boxes) but each one is a
    // break opportunity: the text is distributed over lines and none is lost.
    let font = common::atkinson();
    let text = "你好世界你好世界你好世界";
    let box_width = width(&font, "你", 12.0);
    assert!(box_width > 0.0);
    let lines = wrap(&font, text, 12.0, 3.5 * box_width, &opts());
    assert_eq!(lines, ["你好世", "界你好", "世界你", "好世界"]);
}

#[test]
fn wrapping_honours_letter_spacing() {
    let font = common::atkinson();
    let tight = wrap(&font, PROSE, 10.0, 60.0, &opts());
    let spaced_opts = ShapeOptions { letter_spacing_em: 0.25, ..opts() };
    let spaced = wrap(&font, PROSE, 10.0, 60.0, &spaced_opts);
    assert!(spaced.len() > tight.len(), "{} vs {}", spaced.len(), tight.len());
    assert_eq!(spaced.join(" "), PROSE);
    for pair in spaced.windows(2) {
        assert!(shape(&font, &pair[0], 10.0, &spaced_opts).width <= 60.0);
        let next_word = pair[1].split(' ').next().unwrap_or_default();
        let longer = format!("{} {next_word}", pair[0]);
        assert!(shape(&font, &longer, 10.0, &spaced_opts).width > 60.0, "greedy with spacing");
    }
}

#[test]
fn very_long_input_wraps_without_loss() {
    let font = common::atkinson();
    // 30 000 letters without a break opportunity, then 3 000 short words.
    let blob = "abcdefghij".repeat(3_000);
    let lines = wrap(&font, &blob, 10.0, 60.0, &opts());
    assert!(lines.len() > 500);
    assert_eq!(lines.concat(), blob);
    assert!(lines.iter().all(|l| width(&font, l, 10.0) <= 60.0));

    let words = "mot ".repeat(3_000);
    let lines = wrap(&font, &words, 10.0, 60.0, &opts());
    assert_eq!(lines.join(" "), words.trim_end());
    assert!(lines.iter().all(|l| width(&font, l, 10.0) <= 60.0));
    // An endless line (no limit to speak of) is one line.
    assert_eq!(wrap(&font, &words, 10.0, 1e12, &opts()), [words.trim_end()]);
}
