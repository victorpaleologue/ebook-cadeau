//! `ebc cover`: listing without the network, and the operator cascade against a fixture
//! directory (robots.txt, placeholder detection, print quality).

use std::path::Path;

use crate::common::{
    Answer, ISBN, cover_jpeg, ebc, flat_png, status, stderr, stdout, write_fixtures,
};

const ROBOTS: &str = "https://covers.openlibrary.org/robots.txt";
const LOCATE: &str = "https://covers.openlibrary.org/b/isbn/9782070360024.json";
const IMAGE: &str = "https://covers.openlibrary.org/b/isbn/9782070360024.jpg?default=false";

/// A fixture directory in which Open Library has `image` as the cover.
fn open_library_with(dir: &Path, robots: &str, image: &[u8], declared: (u32, u32)) {
    let facts = format!(r#"{{"id": 8231856, "width": {}, "height": {}}}"#, declared.0, declared.1);
    write_fixtures(
        dir,
        &[
            Answer::exact(ROBOTS, 200, "text/plain", robots.as_bytes()),
            Answer::exact(LOCATE, 200, "application/json", facts.as_bytes()),
            Answer::exact(IMAGE, 200, "image/jpeg", image),
        ],
    );
}

fn operator(fixtures: &Path, out: &Path) -> assert_cmd::Command {
    let mut command = ebc();
    command
        .args(["cover", ISBN, "--rights-basis", "rights_holder_declared", "--offline-fixtures"])
        .arg(fixtures)
        .arg("-o")
        .arg(out);
    command
}

#[test]
fn candidates_are_listed_without_any_request_and_carry_no_rights() {
    let output = ebc().args(["cover", ISBN]).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("https://covers.openlibrary.org/b/isbn/9782070360024-M.jpg"), "{text}");
    assert!(text.contains("preview") && text.contains("link"));
    assert!(text.contains("Rights basis of every candidate: none"), "{text}");
    assert!(!text.contains("download"), "no download without a rights basis:\n{text}");
    assert!(!stderr(&output).contains("operator"));

    let json = ebc().args(["cover", ISBN, "--json", "--notes"]).output().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value["mode"], "list");
    assert_eq!(value["requests"], 0);
    assert!(value["saved"].is_null());
    let candidates = value["candidates"].as_array().unwrap();
    assert!(!candidates.is_empty());
    for candidate in candidates {
        assert_eq!(candidate["rights_basis"], "none");
        assert_ne!(candidate["role"], "download");
        assert!(["listed", "not_run"].contains(&candidate["outcome"].as_str().unwrap()));
        assert!(!candidate["note"].as_str().unwrap().is_empty());
    }
}

#[test]
fn nothing_is_saved_without_an_explicit_rights_basis() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");
    let output = ebc().args(["cover", ISBN, "-o"]).arg(&out).output().unwrap();
    assert_eq!(status(&output), 64);
    let message = stderr(&output);
    assert!(message.contains("--rights-basis") && message.contains("typographic"), "{message}");
    assert!(!out.exists());

    for (basis, reason) in [
        ("none", "print_licence"),
        ("user_private_copy", "print_licence"),
        ("promo_all_media", "print_licence"),
        ("i-found-it-online", "unknown rights basis"),
        ("", "rights basis"),
    ] {
        let output =
            ebc().args(["cover", ISBN, "--rights-basis", basis, "-o"]).arg(&out).output().unwrap();
        assert_eq!(status(&output), 64, "{basis}: {}", stderr(&output));
        assert!(stderr(&output).contains(reason), "{basis}: {}", stderr(&output));
        assert!(!out.exists());
    }
    let output = ebc().args(["cover", ISBN, "--rights-basis", "public_domain"]).output().unwrap();
    assert_eq!(status(&output), 64, "a basis without -o");
}

#[test]
fn operator_mode_saves_the_cover_and_says_who_is_responsible() {
    let fixtures = tempfile::tempdir().unwrap();
    let image = cover_jpeg(1300, 2100);
    open_library_with(fixtures.path(), "User-agent: *\nAllow: /\n", &image, (1300, 2100));
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");

    let output = operator(fixtures.path(), &out).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    assert_eq!(std::fs::read(&out).unwrap(), image, "the bytes are saved untouched");
    let notice = stderr(&output);
    assert!(notice.contains("operator mode") && notice.contains("you are responsible"), "{notice}");
    assert_eq!(notice.lines().filter(|l| l.starts_with("notice:")).count(), 1, "one line");
    let text = stdout(&output);
    assert!(text.contains("download"), "{text}");
    assert!(text.contains("1300 x 2100 px") && text.contains("300 ppi"), "{text}");
    assert!(text.contains("rights basis rights_holder_declared"), "{text}");
}

#[test]
fn operator_mode_reports_as_json() {
    let fixtures = tempfile::tempdir().unwrap();
    open_library_with(fixtures.path(), "", &cover_jpeg(650, 1050), (650, 1050));
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");
    let output = operator(fixtures.path(), &out).arg("--json").output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["mode"], "operator");
    assert_eq!(value["rights_basis"], "rights_holder_declared");
    assert_eq!(value["requests"], 3, "robots.txt, the facts, the image");
    assert_eq!(value["saved"]["width_px"], 650);
    assert_eq!(value["saved"]["verdict"], "low");
    assert!(stderr(&output).contains("warning"), "150 ppi deserves a warning");
    let outcomes: Vec<&str> = value["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["outcome"].as_str().unwrap())
        .collect();
    assert_eq!(outcomes, ["hit", "listed", "listed", "saved"]);
    // Open Library asks for at most one ISBN-keyed request every 3 s.
    assert_eq!(value["candidates"][0]["delay_ms"], 3000);
    assert_eq!(value["candidates"][3]["delay_ms"], 3000);
    assert!(value["candidates"][1]["delay_ms"].is_null(), "previews are not requested");
}

#[test]
fn a_crawl_delay_in_robots_txt_lengthens_the_pause() {
    let fixtures = tempfile::tempdir().unwrap();
    let robots = "User-agent: *\nCrawl-delay: 10\nAllow: /\n";
    open_library_with(fixtures.path(), robots, &cover_jpeg(650, 1050), (650, 1050));
    let dir = tempfile::tempdir().unwrap();
    let output =
        operator(fixtures.path(), &dir.path().join("c.jpg")).arg("--json").output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["candidates"][0]["delay_ms"], 10_000);
    assert_eq!(value["candidates"][3]["delay_ms"], 10_000);
    assert!(value["user_agent"].as_str().unwrap().starts_with("eBookCadeauBot/"));
}

#[test]
fn a_placeholder_is_never_saved() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");
    for (name, body) in [
        ("a one-pixel image", flat_png(1, 1, [255, 255, 255])),
        ("a tiny tile", flat_png(40, 60, [200, 200, 200])),
        ("an error page", b"<html><body>No cover available</body></html>".to_vec()),
        ("nothing", Vec::new()),
        ("garbage", (0..4096_u32).map(|i| (i * 31 % 251) as u8).collect()),
    ] {
        let fixtures = tempfile::tempdir().unwrap();
        open_library_with(fixtures.path(), "", &body, (1, 1));
        let output = operator(fixtures.path(), &out).output().unwrap();
        assert_eq!(status(&output), 3, "{name}: {}", stderr(&output));
        assert!(!out.exists(), "{name} was saved");
        assert!(stderr(&output).contains("nothing was saved"), "{name}: {}", stderr(&output));
    }
}

#[test]
fn a_missing_cover_stops_the_cascade_before_the_download() {
    let fixtures = tempfile::tempdir().unwrap();
    write_fixtures(
        fixtures.path(),
        &[
            Answer::exact(ROBOTS, 404, "text/plain", b""),
            Answer::exact(LOCATE, 404, "application/json", b"{}"),
            Answer::exact(IMAGE, 200, "image/jpeg", &cover_jpeg(650, 1050)),
        ],
    );
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");
    let output = operator(fixtures.path(), &out).arg("--json").output().unwrap();
    assert_eq!(status(&output), 3, "{}", stderr(&output));
    assert!(!out.exists());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["requests"], 2, "robots.txt and the facts; the image is not requested");
    assert_eq!(value["candidates"][3]["outcome"], "skipped");
}

#[test]
fn robots_txt_is_obeyed() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");
    let image = cover_jpeg(650, 1050);

    // Disallowed for everyone, and disallowed for this bot by name.
    for robots in [
        "User-agent: *\nDisallow: /b/\n",
        "User-agent: *\nAllow: /\n\nUser-agent: eBookCadeauBot\nDisallow: /\n",
    ] {
        let fixtures = tempfile::tempdir().unwrap();
        open_library_with(fixtures.path(), robots, &image, (650, 1050));
        let output = operator(fixtures.path(), &out).arg("--json").output().unwrap();
        assert_eq!(status(&output), 3, "{}", stderr(&output));
        assert!(!out.exists());
        assert!(stderr(&output).contains("robots.txt disallows"), "{}", stderr(&output));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["requests"], 1, "only robots.txt itself was requested");
    }

    // robots.txt unreachable (no fixture) or failing: the origin is closed to us.
    for robots in [None, Some(503)] {
        let fixtures = tempfile::tempdir().unwrap();
        let mut answers = vec![
            Answer::exact(LOCATE, 200, "application/json", br#"{"width": 650, "height": 1050}"#),
            Answer::exact(IMAGE, 200, "image/jpeg", &image),
        ];
        if let Some(code) = robots {
            answers.push(Answer::exact(ROBOTS, code, "text/plain", b"busy"));
        }
        write_fixtures(fixtures.path(), &answers);
        let output = operator(fixtures.path(), &out).output().unwrap();
        assert_eq!(status(&output), 3, "{robots:?}: {}", stderr(&output));
        assert!(!out.exists());
        assert!(stderr(&output).contains("refused"), "{}", stderr(&output));
    }
}

#[test]
fn an_image_below_the_requested_resolution_is_not_saved() {
    let fixtures = tempfile::tempdir().unwrap();
    open_library_with(fixtures.path(), "", &cover_jpeg(650, 1050), (650, 1050));
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cover.jpg");
    let output = operator(fixtures.path(), &out).args(["--min-ppi", "250"]).output().unwrap();
    assert_eq!(status(&output), 3, "{}", stderr(&output));
    assert!(!out.exists());
    assert!(stderr(&output).contains("below --min-ppi 250"), "{}", stderr(&output));

    // The same image is fine for a smaller card.
    let output = operator(fixtures.path(), &out)
        .args(["--min-ppi", "250", "--format", "60x97"])
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    assert!(out.exists());
}

#[test]
fn probing_asks_for_facts_only() {
    let fixtures = tempfile::tempdir().unwrap();
    open_library_with(fixtures.path(), "", &cover_jpeg(650, 1050), (309, 500));
    let output = ebc()
        .args(["cover", ISBN, "--probe", "--json", "--offline-fixtures"])
        .arg(fixtures.path())
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["requests"], 2, "robots.txt and the cover record; never the image");
    assert_eq!(value["candidates"][0]["outcome"], "hit");
    let detail = value["candidates"][0]["detail"].as_str().unwrap();
    assert!(detail.contains("309 x 500 px") && detail.contains("insufficient"), "{detail}");
    assert!(value["saved"].is_null());
}

#[test]
fn keys_are_redacted_and_bad_arguments_refused() {
    let secret = "s3cr3t-K3Y-do-not-print";
    let output =
        ebc().args(["cover", ISBN, "--json"]).env("EBC_GOOGLE_BOOKS_KEY", secret).output().unwrap();
    assert_eq!(status(&output), 0);
    let text = stdout(&output);
    assert!(text.contains("googleapis.com") && text.contains("key=REDACTED"), "{text}");
    assert!(!text.contains(secret) && !stderr(&output).contains(secret));

    for (args, code, reason) in [
        (vec!["cover", "978207036002"], 1, "not a valid ISBN"),
        (vec!["cover", ISBN, "--user-agent", "Mozilla/5.0 (X11; Linux)"], 64, "browser"),
        (vec!["cover", ISBN, "--source", "amazon"], 64, "bnf and dnb"),
        (vec!["cover", ISBN, "--format", "huge"], 64, "unknown format"),
        (vec!["cover", ISBN, "--max-risk", "extreme"], 64, "extreme"),
        (
            vec![
                "cover",
                ISBN,
                "--rights-basis",
                "public_domain",
                "-o",
                "x.jpg",
                "--max-risk",
                "low",
            ],
            64,
            "risk",
        ),
    ] {
        let output = ebc().args(&args).output().unwrap();
        assert_eq!(status(&output), code, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(reason), "{args:?}: {}", stderr(&output));
    }

    let low = ebc().args(["cover", ISBN, "--max-risk", "low"]).output().unwrap();
    assert_eq!(status(&low), 3, "no lookup source is low-risk");
    let bnf = ebc().args(["cover", ISBN, "--source", "bnf"]).output().unwrap();
    assert!(stdout(&bnf).contains("catalogue.bnf.fr"), "{}", stdout(&bnf));
}
