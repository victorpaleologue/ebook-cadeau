//! Nothing panics, whatever the bytes: every input truncated at every offset, bit
//! flips, random bytes behind valid magic numbers, random HTML-ish soup.

mod common;

use common::*;
use ebc_covers::placeholder::{assess_bytes, assess_for_source};
use ebc_covers::robots::Robots;
use ebc_covers::scrape::extract;
use ebc_covers::sniff::{probe, probe_details};
use ebc_covers::tdm::TdmRep;
use ebc_covers::{AssetOrigin, PrintableCover, RightsBasis};
use jpeg_encoder::ColorType;

fn image_corpus() -> Vec<(&'static str, Vec<u8>)> {
    let mut exif = b"Exif\0\0".to_vec();
    exif.extend(tiff_orientation(6, true));
    let props = vec![ispe(160, 240), full(b"pixi", 0, 0, &[3, 8, 8, 8]), ispe(1200, 1800), bx(b"irot", &[3])];
    vec![
        ("png", png_rgb(37, 53)),
        ("png-rgba", png_rgba(9, 11)),
        ("gif", gif(20, 30)),
        ("gif-animated", gif_animated(16, 16)),
        ("webp-lossless", webp_lossless(33, 47)),
        ("webp-lossy", riff_webp(&[(b"VP8 ", vp8_frame_header(550, 368, 1))])),
        ("webp-extended", riff_webp(&[(b"VP8X", vp8x_payload(0x1A, 640, 960)), (b"ALPH", vec![1; 7]), (b"EXIF", exif)])),
        ("jpeg", jpeg_baseline(48, 64)),
        ("jpeg-progressive", jpeg_custom(40, 30, ColorType::Rgb, true, Some(2), Some(&tiff_orientation(8, false)), 300)),
        ("jpeg-cmyk", jpeg_custom(24, 24, ColorType::Cmyk, false, None, None, 0)),
        ("avif", bmff(&[b"avif", b"mif1"], &props, &[3, 2, 4])),
        ("heic", bmff(&[b"heic"], &props, &[])),
    ]
}

/// Runs every byte-level entry point on `bytes`.
fn poke(bytes: &[u8]) {
    let info = probe(bytes);
    let details = probe_details(bytes);
    assert_eq!(info, details.map(|d| d.info));
    if let Some(info) = info {
        assert!(info.width > 0 && info.height > 0);
        let _ = assess_for_source("bnf", &info, bytes);
    }
    let _ = assess_bytes(bytes);
    let _ = PrintableCover::new(bytes.to_vec(), RightsBasis::RightsHolderDeclared, AssetOrigin::LocalFile);
}

#[test]
fn probe_survives_truncation_at_every_offset() {
    for (name, bytes) in image_corpus() {
        let full = probe(&bytes).unwrap_or_else(|| panic!("{name}: complete file must probe"));
        let mut first_hit = None;
        for cut in 0..=bytes.len() {
            poke(&bytes[..cut]);
            if let Some(info) = probe(&bytes[..cut]) {
                // A truncated file never reports a different size for the same image…
                assert_eq!(info, full, "{name} cut at {cut}");
                first_hit.get_or_insert(cut);
            }
        }
        // …and the header is enough: the size is known well before the end.
        let first_hit = first_hit.unwrap_or_else(|| panic!("{name}: never probed"));
        assert!(first_hit < bytes.len(), "{name}: probed only at {first_hit} of {}", bytes.len());
        // Dropping bytes from the front never panics either.
        for skip in 0..bytes.len().min(600) {
            poke(&bytes[skip..]);
        }
    }
}

#[test]
fn probe_survives_bit_flips_and_spliced_garbage() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for (_, bytes) in image_corpus() {
        for _ in 0..400 {
            let mut mutated = bytes.clone();
            for _ in 0..=rng.below(4) {
                let at = rng.below(mutated.len().min(512));
                mutated[at] ^= 1 << rng.below(8);
            }
            poke(&mutated);
            // Overwrite a window with extreme values (huge lengths, zero sizes).
            let at = rng.below(mutated.len());
            let fill = [0x00u8, 0xFF, 0x7F, 0x80][rng.below(4)];
            for b in mutated.iter_mut().skip(at).take(1 + rng.below(12)) {
                *b = fill;
            }
            poke(&mutated);
        }
    }
}

#[test]
fn probe_survives_random_bytes_behind_every_magic_number() {
    let magics: [&[u8]; 12] = [
        b"",
        b"\xFF\xD8\xFF",
        b"\xFF\xD8\xFF\xE1\xFF\xFF",
        b"\x89PNG\r\n\x1a\n",
        b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR",
        b"GIF89a",
        b"GIF87a\x10\x00\x10\x00\xF7",
        b"RIFF\xFF\xFF\xFF\xFFWEBPVP8 ",
        b"RIFF\x00\x00\x00\x00WEBPVP8L",
        b"RIFF\x00\x00\x00\x00WEBPVP8X",
        b"\x00\x00\x00\x1Cftypavif",
        b"\x00\x00\x00\x01ftypheic",
    ];
    let mut rng = Rng(42);
    for magic in magics {
        for round in 0..600 {
            let mut bytes = magic.to_vec();
            bytes.extend(rng.bytes(round % 300));
            poke(&bytes);
        }
    }
    // Structured nonsense: boxes and chunks that point outside the file or at themselves.
    let mut loops = bx(b"ftyp", b"avif\0\0\0\0");
    loops.extend_from_slice(&[0, 0, 0, 0]); // size 0 …
    loops.extend_from_slice(b"meta\0\0\0\0");
    loops.extend_from_slice(&[0, 0, 0, 8]);
    loops.extend_from_slice(b"iprp");
    loops.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
    loops.extend_from_slice(b"ipco");
    poke(&loops);
    let mut huge_ipma = bx(b"ftyp", b"avif\0\0\0\0");
    let mut ipma = vec![0, 0, 0, 1];
    ipma.extend_from_slice(&u32::MAX.to_be_bytes());
    ipma.extend(std::iter::repeat_n(0u8, 4000));
    let mut meta = vec![0u8; 4];
    meta.extend(full(b"pitm", 1, 0, &7u32.to_be_bytes()));
    meta.extend(bx(b"iprp", &[bx(b"ipco", &ispe(5, 5)), bx(b"ipma", &ipma)].concat()));
    huge_ipma.extend(bx(b"meta", &meta));
    poke(&huge_ipma);
    // A GIF made of endless empty extension blocks.
    let mut gif = b"GIF89a\x10\x00\x10\x00\x00\x00\x00".to_vec();
    gif.extend(std::iter::repeat_n([0x21u8, 0xF9, 0x00], 100_000).flatten());
    poke(&gif);
    // A JPEG made of endless fill bytes and of endless tiny segments.
    let mut jpeg = vec![0xFF, 0xD8];
    jpeg.extend(std::iter::repeat_n(0xFFu8, 200_000));
    poke(&jpeg);
    let mut jpeg = vec![0xFF, 0xD8];
    jpeg.extend(std::iter::repeat_n([0xFFu8, 0xE0, 0x00, 0x02], 50_000).flatten());
    poke(&jpeg);
}

fn pages() -> Vec<String> {
    ["publisher_jsonld_book", "publisher_graph_imageobject", "bookseller_product", "malformed", "hostile"]
        .iter()
        .map(|name| fixture(&format!("pages/{name}.html")))
        .collect()
}

#[test]
fn extract_survives_truncation_at_every_offset() {
    for html in pages() {
        let bytes = html.as_bytes();
        let full = extract(&html, "https://www.editeur.example/livre/1");
        for cut in 0..=bytes.len() {
            let text = String::from_utf8_lossy(&bytes[..cut]);
            let page = extract(&text, "https://www.editeur.example/livre/1");
            // Cutting a page can only lose images, never invent a URL the full page
            // does not contain (a cut inside a URL may shorten it).
            assert!(page.images.len() <= full.images.len() + 1, "cut at {cut}");
            for image in &page.images {
                assert!(image.url.starts_with("http://") || image.url.starts_with("https://"));
                assert!((0.0..=1.0).contains(&image.confidence));
            }
        }
        // And from the other end.
        for skip in (0..bytes.len()).step_by(3) {
            let _ = extract(&String::from_utf8_lossy(&bytes[skip..]), "https://www.editeur.example/");
        }
    }
}

#[test]
fn extract_survives_random_input() {
    let mut rng = Rng(7);
    // Raw random bytes, as text.
    for round in 0..500 {
        let bytes = rng.bytes(round % 700);
        let _ = extract(&String::from_utf8_lossy(&bytes), "https://a.example/");
        let _ = extract(&String::from_utf8_lossy(&bytes), &String::from_utf8_lossy(&bytes));
    }
    // Random soup of the tokens the scanner cares about.
    let tokens = [
        "<", ">", "</", "/>", "<!--", "-->", "<!", "<?", "=", "\"", "'", " ", "\n", "&", ";", "&amp;", "&#x", "&#",
        "<meta ", "<link ", "<base ", "<img ", "<a ", "<script", "</script>", "<style>", "</style", "<title>", "</title>",
        "property", "name", "content", "href", "rel", "itemprop", "image", "og:image", "og:image:width", "twitter:image",
        "image_src", "type", "application/ld+json", "{", "}", "[", "]", ":", ",", "\\", "@type", "Book", "@graph",
        "isbn", "9782999900011", "978-2-9999-0001-1", "https://", "//", "..", "/", "é", "\u{202e}", "\0", "\u{feff}",
        "tdm-reservation", "robots", "noai", "1",
    ];
    for round in 0..3000 {
        let mut html = String::new();
        for _ in 0..(5 + round % 120) {
            html.push_str(tokens[rng.below(tokens.len())]);
        }
        let page = extract(&html, "https://a.example/b/c");
        assert!(page.images.len() <= ebc_covers::scrape::MAX_IMAGES);
        // Mutations of the real fixtures.
    }
    for html in pages() {
        let mut bytes = html.into_bytes();
        for _ in 0..300 {
            let at = rng.below(bytes.len());
            bytes[at] = b"<>\"'=&;{}[]/\\ \n\0"[rng.below(16)];
            let _ = extract(&String::from_utf8_lossy(&bytes), "https://a.example/");
        }
    }
}

#[test]
fn robots_and_tdmrep_survive_truncation_and_garbage() {
    let robots = fixture("robots/publisher.txt");
    let bytes = robots.as_bytes();
    for cut in 0..=bytes.len() {
        let parsed = Robots::from_response(200, &bytes[..cut]);
        let _ = parsed.allowed("eBookCadeauBot/0.1", "/media/couvertures/x.jpg");
        let _ = parsed.crawl_delay("eBookCadeauBot");
    }
    let tdmrep = br#"[{"location":"/images/*.jpg$","tdm-reservation":1,"tdm-policy":"https://a.example/p.json"},{"location":"/","tdm-reservation":0}]"#;
    for cut in 0..=tdmrep.len() {
        let parsed = TdmRep::parse(&tdmrep[..cut]);
        let _ = parsed.reservation("/images/a.jpg");
    }
    let mut rng = Rng(99);
    for round in 0..400 {
        let bytes = rng.bytes(round % 500);
        let parsed = Robots::from_response(200, &bytes);
        let _ = parsed.allowed(&String::from_utf8_lossy(&bytes), &String::from_utf8_lossy(&bytes));
        let _ = TdmRep::parse(&bytes).reservation(&String::from_utf8_lossy(&bytes));
    }
}
