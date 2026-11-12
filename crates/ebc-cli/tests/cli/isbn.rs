//! `ebc isbn`.

use crate::common::{ebc, status, stderr, stdout};

#[test]
fn an_isbn10_is_validated_converted_and_hyphenated() {
    let output = ebc().args(["isbn", "ISBN 2-07-036002-4"]).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    for expected in
        ["9782070360024", "978-2-07-036002-4", "2070360024", "2-07-036002-4", "French language"]
    {
        assert!(text.contains(expected), "{expected} missing:\n{text}");
    }
}

#[test]
fn json_output_describes_the_isbn() {
    let output = ebc().args(["isbn", "--json", "979-10-321-0276-3"]).output().unwrap();
    assert_eq!(status(&output), 0);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["isbn13"], "9791032102763");
    assert_eq!(value["hyphenated"], "979-10-321-0276-3");
    assert!(value["isbn10"].is_null(), "979 has no ISBN-10 form");
    assert_eq!(value["group"]["prefix"], "979-10");
}

#[test]
fn an_invalid_isbn_fails_with_the_reason() {
    for (input, reason) in [
        ("9782070360025", "check digit"),
        ("12345", "digits"),
        ("9772070360021", "978 or 979"),
        ("", "digits"),
    ] {
        let output = ebc().args(["isbn", input]).output().unwrap();
        assert_eq!(status(&output), 1, "{input:?}");
        assert!(stdout(&output).is_empty());
        assert!(stderr(&output).contains(reason), "{input:?}: {}", stderr(&output));
    }
}

#[test]
fn hostile_input_is_refused_without_a_panic() {
    let long = "9".repeat(100_000);
    for input in [long.as_str(), "978\u{0}2070360024", "９７８２０７０３６００２４", "97820703600２4\u{202e}"] {
        let output = ebc().args(["isbn", "--", input]).output();
        // An argument with a NUL byte cannot even be passed to a process.
        let Ok(output) = output else { continue };
        assert!([0, 1].contains(&status(&output)), "status {}", status(&output));
        assert!(!stderr(&output).contains("panicked"));
    }
}

#[test]
fn extract_finds_every_isbn_in_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("list.txt");
    std::fs::write(
        &file,
        "EAN 9782070360024, also sold as ISBN-10: 0-306-40615-2.\nNot one: 9782070360025.\n\xff\xfe",
    )
    .unwrap();
    let output = ebc().arg("isbn").arg("--extract").arg(&file).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    assert_eq!(stdout(&output), "978-2-07-036002-4\n978-0-306-40615-7\n");

    let json = ebc().arg("isbn").arg("--json").arg("--extract").arg(&file).output().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 2);
    assert_eq!(value[1]["isbn10"], "0306406152");
}

#[test]
fn extract_reads_standard_input() {
    let output = ebc()
        .args(["isbn", "--extract", "-"])
        .write_stdin("see 978-2-07-036002-4 for details")
        .output()
        .unwrap();
    assert_eq!(status(&output), 0);
    assert_eq!(stdout(&output), "978-2-07-036002-4\n");
}

#[test]
fn extract_without_a_hit_exits_3_and_a_missing_file_exits_1() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("empty.txt");
    std::fs::write(&file, "nothing to see here 1234567890").unwrap();
    let none = ebc().arg("isbn").arg("--extract").arg(&file).output().unwrap();
    assert_eq!(status(&none), 3);
    assert!(stdout(&none).is_empty());
    assert!(stderr(&none).contains("no ISBN found"));

    let missing = ebc().arg("isbn").arg("--extract").arg(dir.path().join("nope")).output().unwrap();
    assert_eq!(status(&missing), 1);
    assert!(stderr(&missing).contains("nope"));
}
