//! `ebc qr`: the SVG is rasterised here and decoded back with `rqrr`.

use crate::common::{ebc, status, stderr, stdout};

/// Rasterises the SVG written by `ebc qr` (a white square and one path of
/// `M x,y h w v h h -w z` rectangles in module units) at `scale` pixels per module.
fn rasterise(svg: &str, scale: usize) -> (usize, Vec<u8>) {
    let attribute = |name: &str| {
        let start = svg.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
        &svg[start..start + svg[start..].find('"').unwrap()]
    };
    let side: usize = attribute("viewBox").split(' ').nth(2).unwrap().parse().unwrap();
    let mut pixels = vec![255_u8; side * scale * side * scale];
    let data = attribute(" d");
    for rect in data.split('M').filter(|r| !r.is_empty()) {
        let (origin, rest) = rect.split_once('h').unwrap();
        let (x, y) = origin.split_once(',').unwrap();
        let (w, rest) = rest.split_once('v').unwrap();
        let (h, _) = rest.split_once('h').unwrap();
        let (x, y, w, h): (usize, usize, usize, usize) =
            (x.parse().unwrap(), y.parse().unwrap(), w.parse().unwrap(), h.parse().unwrap());
        for py in y * scale..(y + h) * scale {
            for px in x * scale..(x + w) * scale {
                pixels[py * side * scale + px] = 0;
            }
        }
    }
    (side * scale, pixels)
}

fn decode(svg: &str) -> String {
    let (side, pixels) = rasterise(svg, 6);
    let mut image =
        rqrr::PreparedImage::prepare_from_greyscale(side, side, |x, y| pixels[y * side + x]);
    let grids = image.detect_grids();
    assert_eq!(grids.len(), 1, "exactly one QR code in the image");
    grids[0].decode().unwrap().1
}

#[test]
fn the_code_decodes_back_to_the_text() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("qr.svg");
    let text = "https://ebk.example/k/BCDFGHJKLMNP";
    let output = ebc().args(["qr", text, "-o"]).arg(&file).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    assert!(stdout(&output).is_empty(), "the summary goes to standard error");
    assert!(!stderr(&output).contains("BCDFGHJKLMNP"), "the payload is not echoed");
    let svg = std::fs::read_to_string(&file).unwrap();
    assert!(svg.starts_with("<svg "));
    assert_eq!(decode(&svg), text);
}

#[test]
fn a_compact_url_decodes_to_the_equivalent_upper_case_url() {
    let plain = ebc().args(["qr", "https://ebk.example/K/BCDFGHJKLMNP", "-o", "-"]).output().unwrap();
    let compact = ebc()
        .args(["qr", "https://ebk.example/K/BCDFGHJKLMNP", "--compact-url", "--ec", "Q", "-o", "-"])
        .output()
        .unwrap();
    assert_eq!((status(&plain), status(&compact)), (0, 0), "{}", stderr(&compact));
    assert_eq!(decode(&stdout(&plain)), "https://ebk.example/K/BCDFGHJKLMNP");
    assert_eq!(decode(&stdout(&compact)), "HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNP");
    assert!(stderr(&compact).contains("error correction"));
}

#[test]
fn unicode_text_survives() {
    let text = "Bonne lecture, Zoé ! — 読書";
    let output = ebc().args(["qr", text, "--module-px", "3", "-o", "-"]).output().unwrap();
    assert_eq!(status(&output), 0);
    assert_eq!(decode(&stdout(&output)), text);
}

#[test]
fn what_cannot_be_encoded_or_written_fails() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("qr.svg");
    let too_long = "x".repeat(8000);
    for text in ["", too_long.as_str()] {
        let output = ebc().args(["qr", text, "-o"]).arg(&file).output().unwrap();
        assert_eq!(status(&output), 1, "{}", stderr(&output));
        assert!(stderr(&output).contains("QR code"), "{}", stderr(&output));
        assert!(!file.exists(), "nothing is written on failure");
    }
    let nowhere = dir.path().join("no-such-dir").join("qr.svg");
    let output = ebc().args(["qr", "text", "-o"]).arg(&nowhere).output().unwrap();
    assert_eq!(status(&output), 1);
    assert!(stderr(&output).contains("no-such-dir"));

    let output =
        ebc().args(["qr", "ftp//not a url", "--compact-url", "-o"]).arg(&file).output().unwrap();
    assert!([0, 1].contains(&status(&output)));
}
