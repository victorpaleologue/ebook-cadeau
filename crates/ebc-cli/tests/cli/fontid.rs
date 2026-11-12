//! `ebc fontid`: an own-made cover whose title is set in one of the bundled OFL fonts
//! (`assets/fonts`) must come back as that font's family.

use std::path::{Path, PathBuf};

use crate::common::{ebc, flat_png, status, stderr, stdout};

/// The repository's `assets/fonts` directory (SIL OFL fonts).
fn fonts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("assets").join("fonts")
}

/// A 600 x 900 "cover": `title` in dark ink on an off-white ground, as a PNG file.
fn title_cover(font_file: &str, title: &str) -> Vec<u8> {
    let bytes = std::fs::read(fonts_dir().join(font_file)).expect("bundled font");
    let font = ebc_text::Font::from_bytes(bytes, 0).expect("font parses");
    let mask = ebc_text::rasterize(&font, title, 52.0, &ebc_text::ShapeOptions::default());
    assert!(!mask.is_empty() && mask.width < 560, "the title fits: {} px", mask.width);
    let mut image = image::RgbImage::from_pixel(600, 900, image::Rgb([244, 240, 232]));
    let (left, top) = ((600 - mask.width) / 2, 220);
    for y in 0..mask.height {
        for x in 0..mask.width {
            let ink = u32::from(mask.data[(y * mask.width + x) as usize]);
            let blend = |ground: u8, colour: u32| {
                ((u32::from(ground) * (255 - ink) + colour * ink) / 255) as u8
            };
            image.put_pixel(
                left + x,
                top + y,
                image::Rgb([blend(244, 28), blend(240, 30), blend(232, 46)]),
            );
        }
    }
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).expect("PNG encoding");
    out.into_inner()
}

#[test]
fn the_family_of_the_title_is_found_with_the_bundled_index() {
    let dir = tempfile::tempdir().unwrap();
    for (file, family, title) in [
        ("Spectral-SemiBold.ttf", "Spectral", "Les Vagues du soir"),
        ("BebasNeue-Regular.ttf", "Bebas Neue", "LES VAGUES DU SOIR"),
    ] {
        let cover = dir.path().join("cover.png");
        std::fs::write(&cover, title_cover(file, title)).unwrap();
        let output = ebc()
            .arg("fontid")
            .arg(&cover)
            .args(["--title", title, "--top", "3", "--json"])
            .output()
            .unwrap();
        assert_eq!(status(&output), 0, "{file}: {}", stderr(&output));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let matches = value["matches"].as_array().unwrap();
        assert_eq!(matches.len(), 3, "--top 3");
        assert_eq!(matches[0]["family"], family, "{file}: {value:#}");
        assert!(matches[0]["licence"].as_str().unwrap().contains("OFL"), "{value:#}");
        assert!(value["labelled_glyphs"].as_u64().unwrap() >= 4, "{value:#}");
        assert_eq!(value["region"]["detected"], true);
        assert!(stderr(&output).contains("nearest"), "the result is qualified");
    }
}

#[test]
fn text_output_and_the_second_pass_with_font_files() {
    let dir = tempfile::tempdir().unwrap();
    let cover = dir.path().join("cover.png");
    std::fs::write(&cover, title_cover("Spectral-SemiBold.ttf", "Les Vagues du soir")).unwrap();
    let output = ebc()
        .arg("fontid")
        .arg(&cover)
        .args(["--title", "Les Vagues du soir", "--fonts"])
        .arg(fonts_dir())
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    let first = text.lines().find(|l| l.trim_start().starts_with("1.")).unwrap();
    assert!(first.contains("Spectral"), "{text}");
    assert!(text.contains("re-ranked by rendering"), "{text}");
    assert_eq!(text.lines().filter(|l| l.contains("OFL")).count(), 5, "five by default:\n{text}");

    // A directory without the fonts: the first pass still answers.
    let output = ebc()
        .arg("fontid")
        .arg(&cover)
        .args(["--title", "Les Vagues du soir", "--fonts"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    assert!(stderr(&output).contains("not re-ranked"), "{}", stderr(&output));
    assert!(!stdout(&output).contains("re-ranked by rendering"));
}

#[test]
fn a_cover_without_lettering_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    let blank = dir.path().join("blank.png");
    std::fs::write(&blank, flat_png(300, 450, [250, 250, 250])).unwrap();
    let output = ebc().arg("fontid").arg(&blank).args(["--title", "Candide"]).output().unwrap();
    assert_eq!(status(&output), 3, "{}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("--region"), "{}", stderr(&output));

    let forced = ebc()
        .arg("fontid")
        .arg(&blank)
        .args(["--title", "Candide", "--region", "10,10,200,60"])
        .output()
        .unwrap();
    assert_eq!(status(&forced), 3, "{}", stderr(&forced));
}

#[test]
fn bad_images_indexes_and_arguments_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let cover = dir.path().join("cover.png");
    std::fs::write(&cover, title_cover("Spectral-Regular.ttf", "Candide")).unwrap();
    let garbage = dir.path().join("garbage.bin");
    std::fs::write(&garbage, (0..5000_u32).map(|i| (i * 37 % 251) as u8).collect::<Vec<u8>>())
        .unwrap();
    let truncated = dir.path().join("truncated.png");
    let png = std::fs::read(&cover).unwrap();
    std::fs::write(&truncated, &png[..png.len() / 2]).unwrap();

    for image in [&garbage, &truncated, &dir.path().join("missing.png")] {
        let output = ebc().arg("fontid").arg(image).args(["--title", "Candide"]).output().unwrap();
        assert_eq!(status(&output), 1, "{}: {}", image.display(), stderr(&output));
        assert!(!stderr(&output).contains("panicked"));
    }

    let bad_index = ebc()
        .arg("fontid")
        .arg(&cover)
        .args(["--title", "Candide", "--index"])
        .arg(&garbage)
        .output()
        .unwrap();
    assert_eq!(status(&bad_index), 1);
    assert!(stderr(&bad_index).contains("font index"), "{}", stderr(&bad_index));

    for (extra, reason) in [
        (vec!["--title", "  "], "--title"),
        (vec!["--title", "Candide", "--region", "1,2,3"], "X,Y,WIDTH,HEIGHT"),
        (vec!["--title", "Candide", "--region", "1,2,0,5"], "non-empty"),
        (vec!["--title", "Candide", "--top", "0"], "0"),
        (vec![], "--title"),
    ] {
        let output = ebc().arg("fontid").arg(&cover).args(&extra).output().unwrap();
        assert_eq!(status(&output), 64, "{extra:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(reason), "{extra:?}: {}", stderr(&output));
    }
}
