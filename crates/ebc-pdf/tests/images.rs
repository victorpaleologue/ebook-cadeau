//! Image XObjects, parsed back with `lopdf`.

mod common;

use std::sync::Arc;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Document, ImageData, ImageId, Item, Page, PdfStandard, PixelFormat, Rect, Size};

const ALL: [PdfStandard; 4] =
    [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4];
const READABLE: WriteOptions = WriteOptions { compress: false, keep_layers: false };

fn raw(format: PixelFormat, width: u32, height: u32, pixels: Vec<u8>) -> ImageData {
    ImageData::Raw { pixels: Arc::from(pixels), width_px: width, height_px: height, format }
}

/// A document with one page showing `image` at a fixed place.
fn doc_with(standard: PdfStandard, data: ImageData, icc: Option<Arc<[u8]>>) -> Document {
    let mut doc = base_doc(standard);
    let image = doc.add_image(data, icc);
    let mut page = Page::new(Size::new(100.0, 80.0), 0.0, 0.0);
    page.items.push(Item::Image { image, rect: Rect::new(10.0, 20.0, 40.0, 30.0) });
    doc.pages.push(page);
    doc
}

/// The stream object of the image named `name` on page 1.
fn image_object<'a>(pdf: &'a lopdf::Document, name: &str) -> &'a lopdf::Stream {
    let xobjects = resources(pdf, 1, "XObject").expect("the page has XObjects");
    get(pdf, xobjects, name).as_stream().expect("image stream")
}

/// Describes the image's colour space: the device space name, or `ICCBased/<N>` together
/// with the profile bytes.
fn color_space(pdf: &lopdf::Document, image: &lopdf::Stream) -> (String, Option<Vec<u8>>) {
    match get(pdf, &image.dict, "ColorSpace") {
        Object::Name(name) => (String::from_utf8_lossy(name).into_owned(), None),
        Object::Array(array) => {
            assert_eq!(array[0].as_name().expect("name"), b"ICCBased");
            let profile = resolve(pdf, &array[1]);
            let n = get(pdf, &profile.as_stream().expect("stream").dict, "N").as_i64().expect("N");
            (format!("ICCBased/{n}"), Some(stream_data(profile)))
        }
        other => panic!("unexpected colour space {other:?}"),
    }
}

fn filters(image: &lopdf::Stream) -> Vec<String> {
    image
        .filters()
        .map(|f| f.iter().map(|n| String::from_utf8_lossy(n).into_owned()).collect())
        .unwrap_or_default()
}

#[test]
fn rgb_jpeg_is_embedded_untouched_with_the_right_colour_space() {
    let data = jpeg_rgb(48, 32, |x, y| [(x * 5) as u8, (y * 7) as u8, 99]);
    let ImageData::Jpeg { bytes: original, .. } = &data else { unreachable!() };
    for standard in [PdfStandard::Plain, PdfStandard::X3_2002, PdfStandard::X4] {
        for options in [READABLE, WriteOptions::default()] {
            let pdf =
                parse(&write_pdf(&doc_with(standard, data.clone(), None), &options).expect("ok"));
            let image = image_object(&pdf, "Im0");
            assert_eq!(get_name(&pdf, &image.dict, "Type"), "XObject");
            assert_eq!(get_name(&pdf, &image.dict, "Subtype"), "Image");
            assert_eq!(get(&pdf, &image.dict, "Width").as_i64().expect("int"), 48);
            assert_eq!(get(&pdf, &image.dict, "Height").as_i64().expect("int"), 32);
            assert_eq!(get(&pdf, &image.dict, "BitsPerComponent").as_i64().expect("int"), 8);
            assert_eq!(filters(image), ["DCTDecode"], "JPEG data is never recompressed");
            assert_eq!(image.content, original.to_vec(), "byte-for-byte passthrough");
            assert!(!image.dict.has(b"Decode") && !image.dict.has(b"SMask"));

            let (space, profile) = color_space(&pdf, image);
            if standard == PdfStandard::Plain {
                assert_eq!(space, "DeviceRGB");
            } else {
                // Colour-managed standards: untagged RGB is declared as sRGB.
                assert_eq!(space, "ICCBased/3", "{standard:?}");
                assert_eq!(profile.expect("profile"), srgb_v2_profile().to_vec());
            }
        }
    }
}

#[test]
fn grey_and_cmyk_jpegs_use_device_spaces_in_every_standard() {
    let grey = jpeg_gray(16, 16, |x, y| (x * 16 + y) as u8);
    let cmyk = jpeg_cmyk(16, 16, [255, 0, 0, 0], false);
    let ycck = jpeg_cmyk(16, 16, [0, 128, 255, 10], true);
    for standard in ALL {
        let pdf =
            parse(&write_pdf(&doc_with(standard, grey.clone(), None), &READABLE).expect("ok"));
        let image = image_object(&pdf, "Im0");
        assert_eq!(color_space(&pdf, image).0, "DeviceGray");
        assert_eq!(filters(image), ["DCTDecode"]);
        assert!(!image.dict.has(b"Decode"));

        for data in [&cmyk, &ycck] {
            let pdf =
                parse(&write_pdf(&doc_with(standard, data.clone(), None), &READABLE).expect("ok"));
            let image = image_object(&pdf, "Im0");
            assert_eq!(color_space(&pdf, image).0, "DeviceCMYK");
            // Adobe-flagged CMYK JPEGs store inverted samples; the Decode array undoes it.
            assert_eq!(
                get_numbers(&pdf, &image.dict, "Decode"),
                [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
                "{standard:?}"
            );
        }
    }
}

/// Removes the Adobe APP14 segment from a JPEG file.
fn without_adobe_marker(jpeg: &[u8]) -> Vec<u8> {
    let start = jpeg.windows(2).position(|w| w == [0xFF, 0xEE]).expect("APP14 marker");
    let length = usize::from(u16::from_be_bytes([jpeg[start + 2], jpeg[start + 3]]));
    assert_eq!(&jpeg[start + 4..start + 9], b"Adobe");
    let mut out = jpeg[..start].to_vec();
    out.extend_from_slice(&jpeg[start + 2 + length..]);
    out
}

#[test]
fn cmyk_jpeg_without_adobe_marker_is_not_inverted() {
    let ImageData::Jpeg { bytes, width_px, height_px, components } =
        jpeg_cmyk(8, 8, [10, 20, 30, 40], false)
    else {
        unreachable!()
    };
    let plain = ImageData::Jpeg {
        bytes: Arc::from(without_adobe_marker(&bytes)),
        width_px,
        height_px,
        components,
    };
    let pdf =
        parse(&write_pdf(&doc_with(PdfStandard::X1a2001, plain, None), &READABLE).expect("ok"));
    let image = image_object(&pdf, "Im0");
    assert_eq!(color_space(&pdf, image).0, "DeviceCMYK");
    assert!(!image.dict.has(b"Decode"));
}

#[test]
fn raw_pixels_are_flate_compressed_losslessly() {
    let w = 37;
    let h = 23;
    let pattern = |bpp: usize| -> Vec<u8> {
        (0..w * h * bpp).map(|i| ((i * 31 + i / 7) % 251) as u8).collect()
    };
    let cases = [
        (PixelFormat::Gray8, 1, "DeviceGray", ALL.as_slice()),
        (PixelFormat::Cmyk8, 4, "DeviceCMYK", ALL.as_slice()),
        (PixelFormat::Rgb8, 3, "DeviceRGB", [PdfStandard::Plain].as_slice()),
        (PixelFormat::Rgb8, 3, "ICCBased/3", [PdfStandard::X3_2002, PdfStandard::X4].as_slice()),
    ];
    for (format, bpp, expected_space, standards) in cases {
        let pixels = pattern(bpp);
        for &standard in standards {
            let data = raw(format, w as u32, h as u32, pixels.clone());
            let doc = doc_with(standard, data, None);

            let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("ok"));
            let image = image_object(&pdf, "Im0");
            assert_eq!(filters(image), ["FlateDecode"]);
            assert!(image.content.len() < pixels.len(), "compressed");
            assert_eq!(image.decompressed_content().expect("inflates"), pixels, "{format:?}");
            assert_eq!(color_space(&pdf, image).0, expected_space, "{format:?} {standard:?}");
            assert_eq!(get(&pdf, &image.dict, "Width").as_i64().expect("int"), w as i64);
            assert_eq!(get(&pdf, &image.dict, "Height").as_i64().expect("int"), h as i64);
            assert_eq!(get(&pdf, &image.dict, "BitsPerComponent").as_i64().expect("int"), 8);

            // Without compression the samples are stored as they are.
            let pdf = parse(&write_pdf(&doc, &READABLE).expect("ok"));
            let image = image_object(&pdf, "Im0");
            assert!(filters(image).is_empty());
            assert_eq!(image.content, pixels);
        }
    }
}

#[test]
fn alpha_channel_becomes_a_soft_mask_where_transparency_is_allowed() {
    let (w, h) = (9u32, 5u32);
    let mut rgba = Vec::new();
    for i in 0..w * h {
        rgba.extend_from_slice(&[
            (i * 3) as u8,
            (i * 5) as u8,
            (i * 7) as u8,
            (i * 255 / (w * h - 1)) as u8,
        ]);
    }
    let rgb: Vec<u8> = rgba.chunks_exact(4).flat_map(|p| p[..3].to_vec()).collect();
    let alpha: Vec<u8> = rgba.chunks_exact(4).map(|p| p[3]).collect();

    for standard in [PdfStandard::X4, PdfStandard::Plain] {
        for options in [READABLE, WriteOptions::default()] {
            let doc = doc_with(standard, raw(PixelFormat::Rgba8, w, h, rgba.clone()), None);
            let pdf = parse(&write_pdf(&doc, &options).expect("ok"));
            let image = image_object(&pdf, "Im0");
            assert_eq!(stream_data(&Object::Stream(image.clone())), rgb, "colour channels");
            let mask = get(&pdf, &image.dict, "SMask").as_stream().expect("soft mask stream");
            assert_eq!(get_name(&pdf, &mask.dict, "Subtype"), "Image");
            assert_eq!(get_name(&pdf, &mask.dict, "ColorSpace"), "DeviceGray");
            assert_eq!(get(&pdf, &mask.dict, "Width").as_i64().expect("int"), i64::from(w));
            assert_eq!(get(&pdf, &mask.dict, "Height").as_i64().expect("int"), i64::from(h));
            assert_eq!(get(&pdf, &mask.dict, "BitsPerComponent").as_i64().expect("int"), 8);
            assert_eq!(stream_data(&Object::Stream(mask.clone())), alpha, "alpha channel");
            assert_eq!(filters(mask).is_empty(), !options.compress);
        }
    }

    // Fully opaque alpha is dropped, so the image is fine even without transparency.
    let mut opaque = rgba.clone();
    for p in opaque.chunks_exact_mut(4) {
        p[3] = 255;
    }
    for standard in [PdfStandard::X3_2002, PdfStandard::X4, PdfStandard::Plain] {
        let doc = doc_with(standard, raw(PixelFormat::Rgba8, w, h, opaque.clone()), None);
        let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("ok"));
        let image = image_object(&pdf, "Im0");
        assert!(!image.dict.has(b"SMask"), "{standard:?}");
        assert_eq!(image.decompressed_content().expect("inflates"), rgb);
    }
}

#[test]
fn image_profiles_are_embedded_and_shared() {
    // Two RGB images tagged with the same (v4) profile, one untagged RGB image and one
    // CMYK image tagged with the output-intent profile.
    let mut doc = base_doc(PdfStandard::X4);
    let v4 = srgb_v4_profile();
    let a = doc.add_image(jpeg_rgb(8, 8, |_, _| [200, 10, 10]), Some(v4.clone()));
    let b =
        doc.add_image(raw(PixelFormat::Rgb8, 1, 1, vec![1, 2, 3]), Some(Arc::from(v4.to_vec())));
    let c = doc.add_image(raw(PixelFormat::Rgb8, 1, 1, vec![4, 5, 6]), None);
    let d = doc.add_image(raw(PixelFormat::Cmyk8, 1, 1, vec![0, 0, 0, 255]), Some(cmyk_profile()));
    let mut page = Page::new(Size::new(100.0, 80.0), 0.0, 0.0);
    for (i, image) in [a, b, c, d].into_iter().enumerate() {
        page.items.push(Item::Image { image, rect: Rect::new(10.0 * i as f64, 0.0, 8.0, 8.0) });
    }
    doc.pages.push(page);
    let pdf = parse(&write_pdf(&doc, &WriteOptions::default()).expect("ok"));

    let profile_ref = |name: &str| {
        let image = image_object(&pdf, name);
        let array = get(&pdf, &image.dict, "ColorSpace").as_array().expect("ICCBased array");
        array[1].as_reference().expect("indirect profile")
    };
    let (ra, rb, rc, rd) =
        (profile_ref("Im0"), profile_ref("Im1"), profile_ref("Im2"), profile_ref("Im3"));
    assert_eq!(ra, rb, "equal profiles are written once, even from different buffers");
    assert_ne!(ra, rc);
    assert_eq!(
        color_space(&pdf, image_object(&pdf, "Im0")),
        ("ICCBased/3".into(), Some(v4.to_vec()))
    );
    assert_eq!(
        color_space(&pdf, image_object(&pdf, "Im2")),
        ("ICCBased/3".into(), Some(srgb_v2_profile().to_vec()))
    );
    assert_eq!(
        color_space(&pdf, image_object(&pdf, "Im3")),
        ("ICCBased/4".into(), Some(cmyk_profile().to_vec()))
    );
    // The CMYK image and the output intent share one profile stream.
    let intents = get(&pdf, catalog(&pdf), "OutputIntents").as_array().expect("array");
    let intent = resolve(&pdf, &intents[0]).as_dict().expect("dict");
    assert_eq!(intent.get(b"DestOutputProfile").and_then(Object::as_reference).expect("ref"), rd);
    let profiles = pdf
        .objects
        .values()
        .filter(|o| o.as_stream().is_ok_and(|s| s.dict.has(b"N") && !s.dict.has(b"Type")))
        .count();
    assert_eq!(profiles, 3, "v4 sRGB, default sRGB, CMYK");
}

#[test]
fn images_are_placed_upright_and_shared_between_pages() {
    let mut doc = base_doc(PdfStandard::Plain);
    let unused = doc.add_image(raw(PixelFormat::Gray8, 1, 1, vec![7]), None);
    let shared = doc.add_image(raw(PixelFormat::Gray8, 2, 2, vec![0, 255, 255, 0]), None);
    let invisible = doc.add_image(raw(PixelFormat::Gray8, 1, 1, vec![9]), None);
    assert_eq!((unused, shared, invisible), (ImageId(0), ImageId(1), ImageId(2)));
    for _ in 0..2 {
        let mut page = Page::new(Size::new(100.0, 80.0), 2.0, 0.0);
        page.items.push(Item::Image { image: shared, rect: Rect::new(10.0, 20.0, 40.0, 30.0) });
        // Zero-area placements draw nothing.
        page.items.push(Item::Image { image: invisible, rect: Rect::new(10.0, 20.0, 0.0, 30.0) });
        page.items.push(Item::Image { image: invisible, rect: Rect::new(10.0, 20.0, 5.0, 0.0) });
        doc.pages.push(page);
    }
    let pdf = parse(&write_pdf(&doc, &READABLE).expect("ok"));

    for n in 1..=2 {
        // The unit square is scaled to the rectangle and flipped, so the first sample row
        // ends up at the top: x = 10, top y = 20, bottom y = 50.
        let content = page_content(&pdf, n);
        assert!(content.contains("q\n40 0 0 -30 10 50 cm\n/Im0 Do\nQ"), "{content}");
        assert_eq!(content.matches(" Do").count(), 1);
        let xobjects = resources(&pdf, n, "XObject").expect("XObjects");
        assert_eq!(xobjects.len(), 1);
    }
    let reference = |n: u32| {
        resources(&pdf, n, "XObject")
            .expect("XObjects")
            .get(b"Im0")
            .expect("Im0")
            .as_reference()
            .expect("ref")
    };
    assert_eq!(reference(1), reference(2), "one XObject for both pages");
    let images = pdf
        .objects
        .values()
        .filter(|o| o.as_stream().is_ok_and(|s| s.dict.has_type(b"XObject")))
        .count();
    assert_eq!(images, 1, "unused and never-painted images are not embedded");
}
