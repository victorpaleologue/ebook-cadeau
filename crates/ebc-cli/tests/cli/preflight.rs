//! `ebc preflight`: exit status 0 for a pass, 2 for errors, 1 for an unreadable file.

use std::path::{Path, PathBuf};

use crate::common::{BASE_URL, CREATED, code, ebc, status, stderr, stdout};

/// Composes a typographic card with `ebc card` and returns its path.
fn make_card(dir: &Path, profile: &str) -> PathBuf {
    let pdf = dir.join(format!("{profile}.pdf"));
    let output = ebc()
        .args(["card", "--title", "Les Vagues du soir", "--author", "Jeanne Roy"])
        .args(["--code", &code(), "--base-url", BASE_URL, "--created-unix", CREATED])
        .args(["--profile", profile, "-o"])
        .arg(&pdf)
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    pdf
}

#[test]
fn a_good_card_passes_in_text_and_json() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = make_card(dir.path(), "upload");
    let text = ebc().arg("preflight").arg(&pdf).output().unwrap();
    assert_eq!(status(&text), 0, "{}", stdout(&text));
    assert!(stdout(&text).contains("PASS — 0 error(s)"), "{}", stdout(&text));

    let json = ebc().arg("preflight").arg(&pdf).arg("--json").output().unwrap();
    assert_eq!(status(&json), 0);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value["passed"], true);
    assert_eq!(value["document"]["page_count"], 2);

    let piped =
        ebc().args(["preflight", "-"]).write_stdin(std::fs::read(&pdf).unwrap()).output().unwrap();
    assert_eq!(status(&piped), 0, "{}", stdout(&piped));
}

#[test]
fn a_file_made_for_another_profile_or_format_exits_2() {
    let dir = tempfile::tempdir().unwrap();
    let home = make_card(dir.path(), "home-a4");
    let output = ebc().arg("preflight").arg(&home).args(["--profile", "pro"]).output().unwrap();
    assert_eq!(status(&output), 2);
    let text = stdout(&output);
    assert!(text.contains("FAIL") && text.contains("BOX_BLEED_TOO_SMALL"), "{text}");
    assert!(text.contains("NO_OUTPUT_INTENT"), "{text}");

    let json =
        ebc().arg("preflight").arg(&home).args(["--profile", "pro", "--json"]).output().unwrap();
    assert_eq!(status(&json), 2);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value["passed"], false);
    assert!(value["findings"].as_array().unwrap().iter().any(|f| f["severity"] == "error"));

    let upload = make_card(dir.path(), "upload");
    let wrong_size = ebc().arg("preflight").arg(&upload).args(["--format", "a5"]).output().unwrap();
    assert_eq!(status(&wrong_size), 2);
    assert!(stdout(&wrong_size).contains("BOX_TRIM_SIZE"), "{}", stdout(&wrong_size));
    // Against the profile alone, without card geometry, the same file is fine.
    let generic = ebc().arg("preflight").arg(&upload).arg("--generic").output().unwrap();
    assert_eq!(status(&generic), 0, "{}", stdout(&generic));
}

#[test]
fn unreadable_input_exits_1_and_never_panics() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = std::fs::read(make_card(dir.path(), "home-a4")).unwrap();

    let not_pdf = dir.path().join("notes.txt");
    std::fs::write(&not_pdf, "hello").unwrap();
    let output = ebc().arg("preflight").arg(&not_pdf).output().unwrap();
    assert_eq!(status(&output), 1);
    assert!(stderr(&output).contains("not a PDF"), "{}", stderr(&output));
    assert!(stdout(&output).is_empty());

    let missing = ebc().arg("preflight").arg(dir.path().join("nope.pdf")).output().unwrap();
    assert_eq!(status(&missing), 1);

    // Damaged files: a report with errors (2) or a refusal (1), never a crash and never
    // a pass.
    let mut flipped = pdf.clone();
    for byte in flipped.iter_mut().skip(200).step_by(97) {
        *byte ^= 0x55;
    }
    for (name, bytes) in [
        ("truncated", pdf[..pdf.len() / 2].to_vec()),
        ("header-only", b"%PDF-1.7\n".to_vec()),
        ("flipped", flipped),
    ] {
        let file = dir.path().join(format!("{name}.pdf"));
        std::fs::write(&file, &bytes).unwrap();
        let output =
            ebc().arg("preflight").arg(&file).args(["--profile", "home-a4"]).output().unwrap();
        assert!([1, 2].contains(&status(&output)), "{name}: status {}", status(&output));
        assert!(!stderr(&output).contains("panicked"), "{name}: {}", stderr(&output));
    }

    let bad = ebc().arg("preflight").arg(&not_pdf).args(["--profile", "offset"]).output().unwrap();
    assert_eq!(status(&bad), 64);
}
