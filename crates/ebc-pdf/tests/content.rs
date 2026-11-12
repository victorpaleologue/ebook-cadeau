//! Page content: paths, strokes, colours, groups and layers, read back from the
//! uncompressed content stream.

mod common;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{
    Color, Document, FillRule, Item, Page, Path, PdfStandard, Point, Rect, Size, Stroke, StrokeCap,
    StrokeJoin, Transform,
};

const READABLE: WriteOptions = WriteOptions { compress: false, keep_layers: false };
const LAYERED: WriteOptions = WriteOptions { compress: false, keep_layers: true };

/// Writes a one-page document (100 x 80 mm, no bleed) holding `items` and returns the
/// parsed PDF and the content stream without its leading page transform.
fn render(
    standard: PdfStandard,
    options: &WriteOptions,
    items: Vec<Item>,
) -> (lopdf::Document, String) {
    let mut doc = base_doc(standard);
    let mut page = Page::new(Size::new(100.0, 80.0), 0.0, 0.0);
    page.items = items;
    doc.pages.push(page);
    let pdf = parse(&write_pdf(&doc, options).expect("writable"));
    let content = page_content(&pdf, 1);
    let (first, rest) = content.split_once('\n').unwrap_or((content.as_str(), ""));
    assert!(first.ends_with(" cm"), "{first}");
    (pdf, rest.to_owned())
}

fn body(standard: PdfStandard, items: Vec<Item>) -> String {
    render(standard, &READABLE, items).1
}

fn triangle() -> Path {
    let mut path = Path::new();
    path.move_to(10.0, 10.0).line_to(30.0, 10.0).line_to(20.0, 25.5).close();
    path
}

#[test]
fn paint_operators_match_fill_rule_and_stroke() {
    let cyan = Color::cmyk(1.0, 0.0, 0.0, 0.0);
    let stroke = Stroke::solid(Color::BLACK, 1.0);
    let path = "10 10 m\n30 10 l\n20 25.5 l\nh\n";
    let cases = [
        (Some((cyan, FillRule::NonZero)), None, "f"),
        (Some((cyan, FillRule::EvenOdd)), None, "f*"),
        (None, Some(stroke.clone()), "S"),
        (Some((cyan, FillRule::NonZero)), Some(stroke.clone()), "B"),
        (Some((cyan, FillRule::EvenOdd)), Some(stroke.clone()), "B*"),
    ];
    for (fill, stroke, operator) in cases {
        let content = body(
            PdfStandard::X4,
            vec![Item::Path { path: triangle(), fill, stroke: stroke.clone() }],
        );
        assert!(content.ends_with(&format!("{path}{operator}")), "{operator}: {content}");
        assert_eq!(content.contains("1 0 0 0 k"), fill.is_some(), "{content}");
        assert_eq!(content.contains("0 0 0 1 K"), stroke.is_some(), "{content}");
    }
    // Neither fill nor stroke, or no segments: nothing is written.
    assert_eq!(
        body(PdfStandard::X4, vec![Item::Path { path: triangle(), fill: None, stroke: None }]),
        ""
    );
    assert_eq!(body(PdfStandard::X4, vec![Item::fill(Path::new(), cyan)]), "");
}

#[test]
fn curves_are_written_as_cubic_beziers() {
    let mut path = Path::new();
    path.move_to(0.0, 0.0)
        .quad_to(30.0, 0.0, 30.0, 30.0)
        .cubic_to(30.0, 40.0, 20.0, 50.0, 10.0, 50.0)
        .close()
        .line_to(5.0, 5.0);
    let content = body(PdfStandard::X4, vec![Item::fill(path, Color::BLACK)]);
    // Quadratic (0,0) (30,0) (30,30) elevated: controls at 2/3 towards the control point.
    assert!(
        content.contains("0 0 m\n20 0 30 10 30 30 c\n30 40 20 50 10 50 c\nh\n5 5 l\nf"),
        "{content}"
    );

    // A circle is four cubic segments and survives unchanged.
    let content = body(
        PdfStandard::X4,
        vec![Item::fill(Path::circle(Point::new(50.0, 40.0), 10.0), Color::BLACK)],
    );
    assert_eq!(content.matches(" c\n").count(), 4, "{content}");
    assert!(
        content.starts_with("0 0 0 1 k\n60 40 m\n60 45.52285 55.52285 50 50 50 c\n"),
        "{content}"
    );
}

#[test]
fn stroke_parameters_are_converted_and_not_repeated() {
    let line = |y: f64| Path::line(Point::new(10.0, y), Point::new(90.0, y));
    let hairline = Stroke::solid(Color::BLACK, 0.25);
    let fancy = Stroke {
        color: Color::gray(0.4),
        width_pt: 2.0,
        dash_mm: vec![3.0, 1.5],
        cap: StrokeCap::Round,
        join: StrokeJoin::Bevel,
    };
    let square = Stroke { cap: StrokeCap::Square, join: StrokeJoin::Round, ..hairline.clone() };
    let content = body(
        PdfStandard::X4,
        vec![
            Item::stroke(line(10.0), hairline.clone()),
            Item::stroke(line(20.0), hairline.clone()),
            Item::stroke(line(30.0), fancy),
            Item::stroke(line(40.0), square),
        ],
    );
    let expected = "\
0 0 0 1 K
0.08819 w
0 J
0 j
[] 0 d
10 10 m
90 10 l
S
10 20 m
90 20 l
S
0.4 G
0.70556 w
1 J
2 j
[3 1.5] 0 d
10 30 m
90 30 l
S
0 0 0 1 K
0.08819 w
2 J
1 j
[] 0 d
10 40 m
90 40 l
S";
    assert_eq!(content, expected);
}

#[test]
fn process_and_grey_colours_use_device_operators_in_every_standard() {
    for standard in
        [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4]
    {
        let (pdf, content) = render(
            standard,
            &READABLE,
            vec![
                Item::filled_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::BLACK),
                Item::filled_rect(
                    Rect::new(0.0, 0.0, 10.0, 10.0),
                    Color::cmyk(0.12345, 0.5, 1.0, 0.0),
                ),
                Item::filled_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::gray(0.5)),
                Item::filled_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE),
                // Out-of-range and NaN components are clamped like the constructors do.
                Item::filled_rect(
                    Rect::new(0.0, 0.0, 10.0, 10.0),
                    Color::Cmyk { c: 7.0, m: -1.0, y: f32::NAN, k: 0.25 },
                ),
                Item::filled_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::Gray(-3.0)),
            ],
        );
        let colours: Vec<&str> =
            content.lines().filter(|l| l.ends_with(" k") || l.ends_with(" g")).collect();
        assert_eq!(
            colours,
            ["0 0 0 1 k", "0.1235 0.5 1 0 k", "0.5 g", "0 0 0 0 k", "1 0 0 0.25 k", "0 g"],
            "{standard:?}"
        );
        assert!(resources(&pdf, 1, "ColorSpace").is_none(), "device colours need no resources");
    }
}

#[test]
fn rgb_is_device_rgb_in_plain_pdf_and_icc_based_srgb_in_colour_managed_standards() {
    let items = || {
        vec![
            Item::filled_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::rgb8(255, 128, 0)),
            Item::stroke(
                Path::line(Point::new(0.0, 0.0), Point::new(5.0, 5.0)),
                Stroke::solid(Color::rgb(0.0, 0.25, 1.0), 1.0),
            ),
        ]
    };
    let (pdf, content) = render(PdfStandard::Plain, &READABLE, items());
    assert!(content.contains("1 0.502 0 rg"), "{content}");
    assert!(content.contains("0 0.25 1 RG"), "{content}");
    assert!(resources(&pdf, 1, "ColorSpace").is_none());

    for standard in [PdfStandard::X3_2002, PdfStandard::X4] {
        let (pdf, content) = render(standard, &READABLE, items());
        assert!(content.contains("/sRGB cs\n1 0.502 0 scn"), "{content}");
        assert!(content.contains("/sRGB CS\n0 0.25 1 SCN"), "{content}");
        assert!(
            !content.lines().any(|l| l.ends_with(" rg") || l.ends_with(" RG")),
            "no device RGB: {content}"
        );
        let spaces = resources(&pdf, 1, "ColorSpace").expect("colour space resources");
        let srgb = get(&pdf, spaces, "sRGB").as_array().expect("array");
        assert_eq!(srgb[0].as_name().expect("name"), b"ICCBased");
        let profile = resolve(&pdf, &srgb[1]);
        assert_eq!(
            get(&pdf, &profile.as_stream().expect("stream").dict, "N").as_i64().expect("N"),
            3
        );
        assert_eq!(stream_data(profile), srgb_v2_profile().to_vec());
        assert!(
            !profile.as_stream().expect("stream").dict.has(b"Alternate"),
            "no device-dependent alternate in a PDF/X file"
        );
    }
}

#[test]
fn registration_colour_is_a_separation_named_all() {
    for standard in
        [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4]
    {
        let (pdf, content) = render(
            standard,
            &READABLE,
            vec![
                Item::stroke(
                    Path::line(Point::new(-5.0, 0.0), Point::new(-2.0, 0.0)),
                    Stroke::solid(Color::Registration, 0.25),
                ),
                Item::fill(Path::circle(Point::new(-4.0, -4.0), 1.0), Color::Registration),
            ],
        );
        assert!(content.contains("/Registration CS\n1 SCN"), "{content}");
        assert!(content.contains("/Registration cs\n1 scn"), "{content}");
        let spaces = resources(&pdf, 1, "ColorSpace").expect("colour space resources");
        let space = get(&pdf, spaces, "Registration").as_array().expect("array");
        assert_eq!(space[0].as_name().expect("name"), b"Separation");
        assert_eq!(space[1].as_name().expect("name"), b"All");
        assert_eq!(space[2].as_name().expect("name"), b"DeviceCMYK");
        let tint = resolve(&pdf, &space[3]).as_dict().expect("tint transform");
        assert_eq!(get(&pdf, tint, "FunctionType").as_i64().expect("int"), 2);
        assert_eq!(get_numbers(&pdf, tint, "Domain"), [0.0, 1.0]);
        assert_eq!(get_numbers(&pdf, tint, "C0"), [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(get_numbers(&pdf, tint, "C1"), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(get(&pdf, tint, "N").as_float().expect("number"), 1.0);
    }
}

#[test]
fn groups_save_transform_clip_and_restore() {
    let child = || Item::filled_rect(Rect::new(0.0, 0.0, 20.0, 4.0), Color::BLACK);
    let content = body(
        PdfStandard::X4,
        vec![
            // Rotated 90 degrees clockwise about (30, 50), clipped to a square.
            Item::Group {
                transform: Transform::rotate_about(90.0, Point::new(30.0, 50.0)),
                clip: Some(Path::rect(Rect::new(0.0, 0.0, 10.0, 10.0))),
                layer: None,
                items: vec![child()],
            },
            // Identity transform: no cm.
            Item::clipped(Rect::new(1.0, 2.0, 3.0, 4.0), vec![child()]),
            // Plain translation and scale.
            Item::group(
                Transform::translate(5.0, 6.0).then_inner(Transform::scale(2.0, 0.5)),
                vec![child()],
            ),
        ],
    );
    let expected = "\
q
0 1 -1 0 80 20 cm
0 0 m
10 0 l
10 10 l
0 10 l
h
W
n
0 0 0 1 k
0 0 m
20 0 l
20 4 l
0 4 l
h
f
Q
q
1 2 m
4 2 l
4 6 l
1 6 l
h
W
n
0 0 0 1 k
0 0 m
20 0 l
20 4 l
0 4 l
h
f
Q
q
2 0 0 0.5 5 6 cm
0 0 0 1 k
0 0 m
20 0 l
20 4 l
0 4 l
h
f
Q";
    assert_eq!(content, expected);
}

#[test]
fn invisible_groups_are_skipped() {
    let child = || Item::filled_rect(Rect::new(0.0, 0.0, 20.0, 4.0), Color::BLACK);
    let content = body(
        PdfStandard::X4,
        vec![
            Item::group(Transform::IDENTITY, vec![]),
            Item::group(Transform::scale(0.0, 1.0), vec![child()]),
            Item::group(
                Transform { a: 1.0, b: 2.0, c: 2.0, d: 4.0, e: 0.0, f: 0.0 },
                vec![child()],
            ),
            Item::Group {
                transform: Transform::IDENTITY,
                clip: Some(Path::new()),
                layer: None,
                items: vec![child()],
            },
        ],
    );
    assert_eq!(content, "");
}

#[test]
fn graphics_state_is_tracked_across_groups() {
    let magenta = Color::cmyk(0.0, 1.0, 0.0, 0.0);
    let rect = |x: f64| Rect::new(x, 0.0, 5.0, 5.0);
    // The colour set inside a group does not leak out of it ...
    let content = body(
        PdfStandard::X4,
        vec![
            Item::group(
                Transform::translate(1.0, 1.0),
                vec![Item::filled_rect(rect(0.0), magenta)],
            ),
            Item::filled_rect(rect(10.0), magenta),
        ],
    );
    assert_eq!(content.matches("0 1 0 0 k").count(), 2, "set again after Q: {content}");

    // ... and a colour set before a group is still current inside and after it.
    let content = body(
        PdfStandard::X4,
        vec![
            Item::filled_rect(rect(0.0), magenta),
            Item::group(
                Transform::translate(1.0, 1.0),
                vec![
                    Item::filled_rect(rect(0.0), magenta),
                    Item::filled_rect(rect(0.0), Color::BLACK),
                ],
            ),
            Item::filled_rect(rect(10.0), magenta),
        ],
    );
    assert_eq!(content.matches("0 1 0 0 k").count(), 1, "{content}");
    assert_eq!(content.matches("0 0 0 1 k").count(), 1, "{content}");
    // q and Q are balanced.
    assert_eq!(
        content.matches("\nq\n").count() + usize::from(content.starts_with("q\n")),
        content.matches("\nQ").count()
    );
}

fn layered_items() -> Vec<Item> {
    let mark = |x: f64| {
        Item::stroke(
            Path::line(Point::new(x, -6.0), Point::new(x, -3.0)),
            Stroke::solid(Color::Registration, 0.25),
        )
    };
    vec![
        Item::layer(
            "artwork",
            vec![Item::filled_rect(Rect::new(0.0, 0.0, 100.0, 80.0), Color::gray(0.9))],
        ),
        Item::layer(
            "marks",
            vec![mark(0.0), Item::layer("fold marks — plis", vec![mark(45.0), mark(55.0)])],
        ),
        Item::layer(
            "artwork",
            vec![Item::filled_rect(Rect::new(10.0, 10.0, 5.0, 5.0), Color::BLACK)],
        ),
    ]
}

#[test]
fn named_groups_become_optional_content_when_layers_are_kept() {
    for standard in [PdfStandard::X4, PdfStandard::Plain] {
        let (pdf, content) = render(standard, &LAYERED, layered_items());

        // Three distinct names -> three OCGs, listed in order of first appearance.
        let properties = get_dict(&pdf, catalog(&pdf), "OCProperties");
        let ocgs = get(&pdf, properties, "OCGs").as_array().expect("array");
        let names: Vec<String> = ocgs
            .iter()
            .map(|o| {
                let ocg = resolve(&pdf, o).as_dict().expect("OCG dictionary");
                assert_eq!(get_name(&pdf, ocg, "Type"), "OCG");
                get_text(&pdf, ocg, "Name")
            })
            .collect();
        assert_eq!(names, ["artwork", "marks", "fold marks — plis"]);
        let default = get_dict(&pdf, properties, "D");
        assert!(!get_text(&pdf, default, "Name").is_empty(), "PDF/X-4 wants a named configuration");
        assert_eq!(get(&pdf, default, "Order").as_array().expect("array"), ocgs);
        assert!(!default.has(b"AS"), "PDF/X-4 forbids automatic state changes");

        // The page maps /OC0../OC2 to those groups and brackets the content with them.
        let page_properties =
            get_dict(&pdf, get_dict(&pdf, page(&pdf, 1), "Resources"), "Properties");
        for (i, ocg) in ocgs.iter().enumerate() {
            assert_eq!(page_properties.get(format!("OC{i}").as_bytes()).expect("entry"), ocg);
        }
        let brackets: Vec<&str> =
            content.lines().filter(|l| l.ends_with("BDC") || *l == "EMC").collect();
        assert_eq!(
            brackets,
            [
                "/OC /OC0 BDC",
                "EMC",
                "/OC /OC1 BDC",
                "/OC /OC2 BDC",
                "EMC",
                "EMC",
                "/OC /OC0 BDC",
                "EMC"
            ],
            "{standard:?}"
        );
    }
}

#[test]
fn layers_are_flattened_when_not_wanted_or_not_allowed() {
    let cases = [
        (PdfStandard::X4, READABLE),
        (PdfStandard::Plain, READABLE),
        (PdfStandard::X1a2001, LAYERED),
        (PdfStandard::X3_2002, LAYERED),
    ];
    let (_, reference) = render(PdfStandard::X4, &LAYERED, layered_items());
    let drawing = |content: &str| -> Vec<String> {
        content.lines().filter(|l| !l.ends_with("BDC") && *l != "EMC").map(str::to_owned).collect()
    };
    for (standard, options) in cases {
        let (pdf, content) = render(standard, &options, layered_items());
        assert!(!catalog(&pdf).has(b"OCProperties"), "{standard:?}");
        assert!(!get_dict(&pdf, page(&pdf, 1), "Resources").has(b"Properties"), "{standard:?}");
        assert!(!content.contains("BDC") && !content.contains("EMC"), "{standard:?}");
        // Flattening only removes the brackets; the drawing is the same.
        assert_eq!(drawing(&content), drawing(&reference), "{standard:?}");
    }
}

#[test]
fn every_page_gets_its_own_content_and_resources() {
    let mut doc: Document = base_doc(PdfStandard::X4);
    let mut first = Page::new(Size::new(100.0, 80.0), 0.0, 0.0);
    first.items.push(Item::filled_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::rgb8(1, 2, 3)));
    let mut second = Page::new(Size::new(50.0, 60.0), 1.0, 2.0);
    second.items.push(Item::filled_rect(Rect::new(0.0, 0.0, 2.0, 2.0), Color::Registration));
    doc.pages = vec![first, second, Page::new(Size::new(10.0, 10.0), 0.0, 0.0)];
    let pdf = parse(&write_pdf(&doc, &READABLE).expect("writable"));
    assert_eq!(pdf.get_pages().len(), 3);

    let spaces = |n: u32| -> Vec<String> {
        resources(&pdf, n, "ColorSpace")
            .map(|d| d.iter().map(|(k, _)| String::from_utf8_lossy(k).into_owned()).collect())
            .unwrap_or_default()
    };
    assert_eq!(spaces(1), ["sRGB"]);
    assert_eq!(spaces(2), ["Registration"]);
    assert!(spaces(3).is_empty());
    assert!(page_content(&pdf, 1).contains("0 0 m\n1 0 l"));
    assert!(page_content(&pdf, 2).contains("0 0 m\n2 0 l"));
    // An empty page still has a (transform-only) content stream and a resource dictionary.
    assert!(page_content(&pdf, 3).trim_end().ends_with(" cm"));
    assert!(page(&pdf, 3).has(b"Resources"));
    // Each page is a child of the single page tree node.
    let tree = get_dict(&pdf, catalog(&pdf), "Pages");
    assert_eq!(get(&pdf, tree, "Count").as_i64().expect("int"), 3);
    assert_eq!(get(&pdf, tree, "Kids").as_array().expect("array").len(), 3);
}
