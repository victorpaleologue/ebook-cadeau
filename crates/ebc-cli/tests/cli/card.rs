//! `ebc card`, and its round trip through `ebc preflight`.

use std::path::Path;

use crate::common::{BASE_URL, CREATED, code, cover_jpeg, ebc, flat_png, status, stderr, stdout};

/// A complete typographic card request writing to `out`.
fn card(out: &Path) -> assert_cmd::Command {
    let mut command = ebc();
    command
        .args(["card", "--title", "Les Vagues du soir", "--subtitle", "Roman"])
        .args(["--author", "Jeanne Roy", "--publisher", "Éditions du Lac (fictif)"])
        .args(["--code", &code(), "--base-url", BASE_URL, "--created-unix", CREATED])
        .args(["--giver", "Alice", "--recipient", "Bob"])
        .args(["--message", "Joyeux anniversaire ! J’ai adoré ce roman."])
        .arg("-o")
        .arg(out);
    command
}

fn preflight(pdf: &Path, profile: &str) -> assert_cmd::Command {
    let mut command = ebc();
    command.arg("preflight").arg(pdf).args(["--profile", profile]);
    command
}

#[test]
fn a_card_of_every_profile_passes_preflight() {
    let dir = tempfile::tempdir().unwrap();
    for profile in ["upload", "pro", "home-a4", "home-letter"] {
        let pdf = dir.path().join(format!("{profile}.pdf"));
        let made = card(&pdf).args(["--profile", profile]).output().unwrap();
        assert_eq!(status(&made), 0, "{profile}: {}", stderr(&made));
        assert!(stdout(&made).contains("typographic card"), "{}", stdout(&made));
        assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF-"));

        let checked = preflight(&pdf, profile).output().unwrap();
        assert_eq!(status(&checked), 0, "{profile}:\n{}", stdout(&checked));
        assert!(stdout(&checked).contains("PASS"), "{}", stdout(&checked));
    }
}

#[test]
fn other_formats_and_locales_pass_too() {
    let dir = tempfile::tempdir().unwrap();
    for (format, profile, locale) in
        [("a6", "upload", "de"), ("a5", "home-a3", "en"), ("120x180x12", "pro", "nl")]
    {
        let pdf = dir.path().join("card.pdf");
        let made = card(&pdf)
            .args(["--format", format, "--profile", profile, "--locale", locale])
            .output()
            .unwrap();
        assert_eq!(status(&made), 0, "{format}/{profile}: {}", stderr(&made));
        let checked = preflight(&pdf, profile).args(["--format", format]).output().unwrap();
        assert_eq!(status(&checked), 0, "{format}/{profile}:\n{}", stdout(&checked));
    }
}

#[test]
fn a_cover_needs_a_rights_basis() {
    let dir = tempfile::tempdir().unwrap();
    let cover = dir.path().join("cover.jpg");
    std::fs::write(&cover, cover_jpeg(1300, 2100)).unwrap();
    let pdf = dir.path().join("card.pdf");

    let refused = card(&pdf).arg("--cover").arg(&cover).output().unwrap();
    assert_eq!(status(&refused), 64);
    let message = stderr(&refused);
    assert!(message.contains("--cover-rights") && message.contains("typographic"), "{message}");
    assert!(!pdf.exists(), "nothing is written when the request is refused");

    let typographic = card(&pdf).arg("--cover").arg(&cover).arg("--typographic").output().unwrap();
    assert_eq!(status(&typographic), 0, "{}", stderr(&typographic));
    assert!(stdout(&typographic).contains("typographic card"));
    let without_image = std::fs::read(&pdf).unwrap();

    let none =
        card(&pdf).arg("--cover").arg(&cover).args(["--cover-rights", "none"]).output().unwrap();
    assert_eq!(status(&none), 0, "{}", stderr(&none));
    assert!(stderr(&none).contains("no rights basis"), "{}", stderr(&none));
    assert_eq!(std::fs::read(&pdf).unwrap(), without_image, "`none` prints no cover either");

    let unknown =
        card(&pdf).arg("--cover").arg(&cover).args(["--cover-rights", "mine"]).output().unwrap();
    assert_eq!(status(&unknown), 64);
    assert!(stderr(&unknown).contains("unknown rights basis"));
}

#[test]
fn a_cover_with_rights_is_printed_and_the_card_passes_preflight() {
    let dir = tempfile::tempdir().unwrap();
    let cover = dir.path().join("cover.jpg");
    std::fs::write(&cover, cover_jpeg(1300, 2100)).unwrap();
    let pdf = dir.path().join("card.pdf");

    let with_cover = |profile: &str, basis: &str| {
        card(&pdf)
            .arg("--cover")
            .arg(&cover)
            .args(["--cover-rights", basis, "--profile", profile])
            .args(["--cover-credit", "© Éditions du Lac (fictif)"])
            .output()
            .unwrap()
    };

    let made = with_cover("pro", "rights_holder_declared");
    assert_eq!(status(&made), 0, "{}", stderr(&made));
    assert!(stdout(&made).contains("cover card"), "{}", stdout(&made));
    // The cover is converted to the output intent's CMYK (docs/research/03 §3.4): no
    // "left as sRGB" warning, and the card passes the profile's own CMYK-only preflight.
    assert!(!stderr(&made).contains("sRGB"), "{}", stderr(&made));
    let strict = preflight(&pdf, "pro").output().unwrap();
    assert_eq!(status(&strict), 0, "{}", stdout(&strict));
    let upload = with_cover("upload", "rights_holder_declared");
    assert_eq!(status(&upload), 0, "{}", stderr(&upload));
    let checked = preflight(&pdf, "upload").output().unwrap();
    assert_eq!(status(&checked), 0, "{}", stdout(&checked));
    // A file with marks and a slug is not an upload (check B4).
    let marked = with_cover("pro", "rights_holder_declared");
    assert_eq!(status(&marked), 0, "{}", stderr(&marked));
    let refused = preflight(&pdf, "upload").output().unwrap();
    assert_eq!(status(&refused), 2, "{}", stdout(&refused));
    assert!(stdout(&refused).contains("MARKS_NOT_ALLOWED"), "{}", stdout(&refused));
    assert!(stdout(&refused).contains("BOX_MEDIA_NOT_BLEED"), "{}", stdout(&refused));

    let home = with_cover("home-a4", "user_private_copy");
    assert_eq!(status(&home), 0, "{}", stderr(&home));
    assert!(stderr(&home).contains("own printer"), "{}", stderr(&home));
    let checked = preflight(&pdf, "home-a4").output().unwrap();
    assert_eq!(status(&checked), 0, "{}", stdout(&checked));
}

#[test]
fn home_only_rights_are_refused_on_professional_profiles() {
    let dir = tempfile::tempdir().unwrap();
    let cover = dir.path().join("cover.png");
    std::fs::write(&cover, cover_jpeg(1300, 2100)).unwrap();
    for basis in ["user_private_copy", "promo_all_media"] {
        for profile in ["upload", "pro"] {
            let pdf = dir.path().join(format!("{basis}-{profile}.pdf"));
            let output = card(&pdf)
                .arg("--cover")
                .arg(&cover)
                .args(["--cover-rights", basis, "--profile", profile])
                .output()
                .unwrap();
            assert_eq!(status(&output), 1, "{basis} on {profile}");
            assert!(stderr(&output).contains(basis), "{}", stderr(&output));
            assert!(!pdf.exists());
        }
    }
}

#[test]
fn hostile_cover_files_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("card.pdf");
    for (name, bytes) in [
        ("empty", Vec::new()),
        ("text", b"<html>not an image</html>".to_vec()),
        ("placeholder", flat_png(1, 1, [255, 255, 255])),
        ("garbage", (0..10_000_u32).map(|i| (i * 131 % 253) as u8).collect()),
        // A JPEG header that announces 65 535 x 65 535 pixels and nothing else.
        (
            "bomb",
            [
                &[0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0x03][..],
                &[0; 64],
            ]
            .concat(),
        ),
    ] {
        let cover = dir.path().join(name);
        std::fs::write(&cover, &bytes).unwrap();
        let output = card(&pdf)
            .arg("--cover")
            .arg(&cover)
            .args(["--cover-rights", "public_domain"])
            .output()
            .unwrap();
        assert_eq!(status(&output), 1, "{name}: {}", stderr(&output));
        assert!(!stderr(&output).contains("panicked"), "{name}");
        assert!(!pdf.exists(), "{name}");
    }
    // A truncated JPEG: the decoder of `ebc-card` may salvage the top of the image or
    // refuse it; either way there is no crash.
    let jpeg = cover_jpeg(400, 600);
    let cover = dir.path().join("truncated.jpg");
    std::fs::write(&cover, &jpeg[..jpeg.len() / 3]).unwrap();
    let output = card(&pdf)
        .arg("--cover")
        .arg(&cover)
        .args(["--cover-rights", "public_domain"])
        .output()
        .unwrap();
    assert!([0, 1].contains(&status(&output)), "status {}", status(&output));
    assert!(!stderr(&output).contains("panicked"));

    let missing = card(&pdf)
        .arg("--cover")
        .arg(dir.path().join("nope.jpg"))
        .args(["--cover-rights", "public_domain"])
        .output()
        .unwrap();
    assert_eq!(status(&missing), 1);
    assert!(stderr(&missing).contains("nope.jpg"));
}

#[test]
fn the_output_is_reproducible_and_the_code_stays_private() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = (dir.path().join("a.pdf"), dir.path().join("b.pdf"));
    let first = card(&a).output().unwrap();
    let second = card(&b).env("SOURCE_DATE_EPOCH", "1").output().unwrap();
    assert_eq!((status(&first), status(&second)), (0, 0));
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap(), "--created-unix wins");

    let compact = code().replace('-', "");
    for output in [&first, &second] {
        for text in [stdout(output), stderr(output)] {
            assert!(!text.contains(&code()) && !text.contains(&compact), "{text}");
        }
    }
}

#[test]
fn a_random_code_is_drawn_and_reported_once() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("card.pdf");
    let mut codes = Vec::new();
    for _ in 0..2 {
        let output = ebc()
            .args(["card", "--title", "Candide", "--author", "Voltaire", "--random-code"])
            .args(["--base-url", BASE_URL, "--profile", "home-a4", "--json", "-o"])
            .arg(&pdf)
            .output()
            .unwrap();
        assert_eq!(status(&output), 0, "{}", stderr(&output));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["card"], "typographic");
        assert_eq!(value["pages"], 2);
        assert_eq!(value["bytes"], std::fs::metadata(&pdf).unwrap().len());
        let code = value["code"].as_str().unwrap().to_owned();
        assert!(ebc_model::GiftCode::parse(&code).is_ok(), "{code}");
        codes.push(code);
    }
    assert_ne!(codes[0], codes[1], "two runs, two codes");

    let given = card(&pdf).arg("--json").output().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&given.stdout).unwrap();
    assert!(value["code"].is_null(), "a code given on the command line is not echoed");
}

#[test]
fn previews_and_standard_output() {
    let dir = tempfile::tempdir().unwrap();
    let previews = dir.path().join("previews");
    let pdf = dir.path().join("card.pdf");
    let output = card(&pdf).arg("--preview-svg").arg(&previews).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    for name in ["outside.svg", "inside.svg"] {
        let svg = std::fs::read_to_string(previews.join(name)).unwrap();
        assert!(svg.trim_start().starts_with("<svg"), "{name}");
    }

    let piped = card(Path::new("-")).output().unwrap();
    assert_eq!(status(&piped), 0);
    assert_eq!(piped.stdout, std::fs::read(&pdf).unwrap(), "stdout is the PDF and nothing else");
    assert!(stderr(&piped).contains("wrote standard output"));
}

#[test]
fn bad_requests_are_explained() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("card.pdf");
    let base = |extra: &[&str]| {
        let mut command = ebc();
        command.args(["card", "--base-url", BASE_URL, "-o"]).arg(&pdf).args(extra);
        command.output().unwrap()
    };
    let good = code();
    for (args, expected, reason) in [
        (vec!["--title", "T", "--code", "BCDF-GHJK-LMNB"], 1, "mistyped"),
        (vec!["--title", "T", "--code", "1234"], 1, "--code"),
        (vec!["--title", "   ", "--code", &good], 1, "no title"),
        (vec!["--title", "T", "--code", &good, "--profile", "offset"], 64, "unknown profile"),
        (vec!["--title", "T", "--code", &good, "--format", "1x1"], 64, "format"),
        (vec!["--title", "T", "--code", &good, "--locale", "tlh"], 64, "unsupported locale"),
        (vec!["--title", "T", "--code", &good, "--isbn", "123"], 1, "not a valid ISBN"),
        (vec!["--title", "T", "--code", &good, "--random-code"], 64, "cannot be used with"),
        (vec!["--title", "T", "--code", &good, "--cover-rights", "public_domain"], 64, "--cover"),
        (vec!["--title", "T", "--code", &good, "--icc", "x.icc"], 64, "--output-condition"),
    ] {
        let output = base(&args);
        assert_eq!(status(&output), expected, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(reason), "{args:?}: {}", stderr(&output));
        assert!(!pdf.exists(), "{args:?} wrote a file");
    }

    let icc = dir.path().join("fake.icc");
    std::fs::write(&icc, b"not an ICC profile").unwrap();
    let output = base(&["--title", "T", "--code", &good, "--output-condition", "FOGRA51", "--icc"]);
    assert_eq!(status(&output), 64, "--icc without a value");
    let mut command = ebc();
    command
        .args(["card", "--title", "T", "--code", &good, "--base-url", BASE_URL])
        .args(["--output-condition", "FOGRA51", "--icc"])
        .arg(&icc)
        .arg("-o")
        .arg(&pdf);
    let output = command.output().unwrap();
    assert_eq!(status(&output), 1, "{}", stderr(&output));
    assert!(stderr(&output).contains("fake.icc"), "{}", stderr(&output));

    let unwritable = ebc()
        .args(["card", "--title", "T", "--code", &good, "--base-url", BASE_URL, "-o"])
        .arg(dir.path().join("no-such-dir").join("card.pdf"))
        .output()
        .unwrap();
    assert_eq!(status(&unwritable), 1);
}

#[test]
fn a_given_icc_profile_becomes_the_output_intent() {
    // The CMYK profile ebc-card bundles, passed as if it were the printer's own.
    let icc = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc");
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("card.pdf");
    let made = card(&pdf)
        .args(["--profile", "pro", "--output-condition", "FOGRA39L-test", "--icc"])
        .arg(&icc)
        .output()
        .unwrap();
    assert_eq!(status(&made), 0, "{}", stderr(&made));

    let checked = preflight(&pdf, "pro").arg("--json").output().unwrap();
    assert_eq!(status(&checked), 0, "{}", stdout(&checked));
    let report: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
    let text = report.to_string();
    assert!(text.contains("\"identifier\":\"FOGRA39L-test\""), "{text}");
    assert!(text.contains("ISOcoated_v2_300_bas"), "{text}");

    // On a home profile the plain PDF has no output intent: the profile is not used.
    let home = card(&pdf)
        .args(["--profile", "home-a4", "--output-condition", "FOGRA39", "--icc"])
        .arg(&icc)
        .output()
        .unwrap();
    assert_eq!(status(&home), 0, "{}", stderr(&home));
    assert!(stderr(&home).contains("--icc is not used"), "{}", stderr(&home));
}
