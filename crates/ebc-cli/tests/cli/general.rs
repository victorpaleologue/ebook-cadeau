//! Help, version and usage errors.

use crate::common::{ebc, status, stderr, stdout};

#[test]
fn help_and_version_are_answers() {
    let help = ebc().arg("--help").output().unwrap();
    assert_eq!(status(&help), 0);
    let text = stdout(&help);
    for word in ["isbn", "lookup", "cover", "card", "preflight", "qr", "Exit status", "64"] {
        assert!(text.contains(word), "{word} missing from --help:\n{text}");
    }
    let version = ebc().arg("--version").output().unwrap();
    assert_eq!(status(&version), 0);
    assert!(stdout(&version).starts_with("ebc "));
}

#[test]
fn every_subcommand_has_help_that_names_its_flags() {
    for (command, flags) in [
        ("isbn", vec!["--extract", "--json"]),
        ("lookup", vec!["--provider", "--base-url", "--offline-fixtures", "--json"]),
        ("cover", vec!["--rights-basis", "--max-risk", "--min-ppi", "--format", "--output"]),
        (
            "card",
            vec![
                "--title",
                "--cover-rights",
                "--typographic",
                "--random-code",
                "--base-url",
                "--profile",
                "--icc",
                "--output-condition",
                "--preview-svg",
                "--output",
            ],
        ),
        ("preflight", vec!["--profile", "--format", "--json", "--allow-rgb"]),
        ("qr", vec!["--output", "--ec"]),
    ] {
        let output = ebc().args([command, "--help"]).output().unwrap();
        assert_eq!(status(&output), 0, "{command} --help");
        let text = stdout(&output);
        for flag in flags {
            assert!(text.contains(flag), "{command} --help does not mention {flag}");
        }
    }
}

#[test]
fn usage_errors_exit_64_and_write_nothing_to_stdout() {
    for args in [
        vec![],
        vec!["frobnicate"],
        vec!["isbn"],
        vec!["isbn", "9782070360024", "--extract", "file.txt"],
        vec!["qr", "text"],
        vec!["qr", "text", "-o", "x.svg", "--ec", "Z"],
        vec!["card", "--title", "T", "--base-url", "https://ebk.example", "-o", "x.pdf"],
        vec!["preflight"],
        vec!["lookup"],
        vec!["cover"],
    ] {
        let output = ebc().args(&args).output().unwrap();
        assert_eq!(status(&output), 64, "{args:?}: {}", stderr(&output));
        assert!(stdout(&output).is_empty(), "{args:?} wrote to stdout");
        assert!(!stderr(&output).is_empty(), "{args:?} explains nothing");
    }
}
