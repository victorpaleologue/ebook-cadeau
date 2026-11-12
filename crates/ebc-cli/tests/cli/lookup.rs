//! `ebc lookup`: against a fixture server on the loopback interface (`--base-url`) and
//! against a fixture directory (`--offline-fixtures`). No request leaves the machine.

use crate::common::{
    Answer, ISBN, Server, closed_port, ebc, fixture_bytes, status, stderr, stdout, write_fixtures,
};

fn open_library(server: &Server) -> String {
    format!("openlibrary={}", server.base)
}

#[cfg(feature = "net")]
#[test]
fn a_record_is_fetched_from_a_local_server_with_attribution_and_licence() {
    let server =
        Server::start(vec![("/api/books", (200, fixture_bytes("openlibrary/etranger.json")))]);
    let output = ebc()
        .args(["lookup", ISBN, "--provider", "openlibrary", "--base-url", &open_library(&server)])
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    for expected in [
        "978-2-07-036002-4",
        "L'étranger",
        "Albert Camus",
        "Gallimard",
        "openlibrary",
        "found (1 request(s)): Open Library",
        "licence: CC0",
        "terms: https://openlibrary.org/developers/api",
    ] {
        assert!(text.contains(expected), "{expected:?} missing:\n{text}");
    }
    let heads = server.heads();
    assert_eq!(heads.len(), 1, "one request for one lookup");
    assert!(heads[0].contains(&format!("bibkeys=ISBN%3A{ISBN}")) || heads[0].contains(ISBN));
    let agent = heads[0].to_ascii_lowercase();
    assert!(agent.contains("user-agent: ebook-cadeau/"), "the client names itself:\n{}", heads[0]);
}

#[cfg(feature = "net")]
#[test]
fn json_output_carries_the_merged_record_and_the_providers() {
    let server =
        Server::start(vec![("/api/books", (200, fixture_bytes("openlibrary/etranger.json")))]);
    let output = ebc()
        .args(["lookup", "--json", "2-07-036002-4", "--provider", "openlibrary"])
        .args(["--base-url", &open_library(&server)])
        .output()
        .unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["isbn"], ISBN);
    assert_eq!(value["found"], true);
    assert_eq!(value["record"]["title"], "L'étranger");
    assert_eq!(value["record"]["contributors"][0]["name"], "Albert Camus");
    assert_eq!(value["providers"][0]["id"], "openlibrary");
    assert_eq!(value["providers"][0]["status"], "found");
    assert!(value["providers"][0]["licence"].as_str().unwrap().contains("CC0"));
}

#[cfg(feature = "net")]
#[test]
fn an_unknown_isbn_exits_3() {
    let server =
        Server::start(vec![("/api/books", (200, fixture_bytes("openlibrary/not-found.json")))]);
    let output = ebc()
        .args(["lookup", ISBN, "--provider", "openlibrary", "--base-url", &open_library(&server)])
        .output()
        .unwrap();
    assert_eq!(status(&output), 3, "{}", stderr(&output));
    assert!(stdout(&output).contains("not found"));
    assert!(stderr(&output).contains("no provider knows"));
}

#[cfg(feature = "net")]
#[test]
fn a_failing_service_exits_1_and_says_which() {
    let server = Server::start(vec![("/api/books", (503, b"<html>down</html>".to_vec()))]);
    let output = ebc()
        .args(["lookup", ISBN, "--provider", "openlibrary", "--base-url", &open_library(&server)])
        .output()
        .unwrap();
    assert_eq!(status(&output), 1);
    let errors = stderr(&output);
    assert!(errors.contains("openlibrary") && errors.contains("503"), "{errors}");

    let nobody = closed_port();
    let output = ebc()
        .args(["lookup", ISBN, "--provider", "openlibrary"])
        .args(["--base-url", &format!("openlibrary={nobody}"), "--timeout", "3"])
        .output()
        .unwrap();
    assert_eq!(status(&output), 1, "{}", stderr(&output));
    assert!(!stderr(&output).contains("panicked"));
}

#[cfg(feature = "net")]
#[test]
fn an_api_key_never_reaches_the_output() {
    let secret = "s3cr3t-K3Y-do-not-print";
    let server = Server::start(vec![(
        "/books/v1/volumes",
        (403, br#"{"error":{"code":403,"message":"API key not valid."}}"#.to_vec()),
    )]);
    let output = ebc()
        .args(["lookup", ISBN, "--provider", "googlebooks", "--json"])
        .args(["--base-url", &format!("googlebooks={}", server.base)])
        .env("EBC_GOOGLE_BOOKS_KEY", secret)
        .output()
        .unwrap();
    assert_eq!(status(&output), 1, "{}", stderr(&output));
    assert!(server.heads()[0].contains(secret), "the key is sent to the service");
    assert!(!stdout(&output).contains(secret), "{}", stdout(&output));
    assert!(!stderr(&output).contains(secret), "{}", stderr(&output));

    // Without a key the provider is refused before any request.
    let output = ebc().args(["lookup", ISBN, "--provider", "googlebooks"]).output().unwrap();
    assert_eq!(status(&output), 64);
    assert!(stderr(&output).contains("EBC_GOOGLE_BOOKS_KEY"));
}

#[test]
fn offline_fixtures_answer_instead_of_the_network() {
    let dir = tempfile::tempdir().unwrap();
    write_fixtures(
        dir.path(),
        &[Answer::containing(
            "openlibrary.org/api/books",
            200,
            "application/json",
            &fixture_bytes("openlibrary/etranger.json"),
        )],
    );
    // Default routing for 978-2: the BnF first (no fixture: a transport error, reported
    // and survived), then Open Library.
    let output =
        ebc().args(["lookup", ISBN, "--offline-fixtures"]).arg(dir.path()).output().unwrap();
    assert_eq!(status(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Albert Camus") && text.contains("Gallimard"), "{text}");
    assert!(text.contains("bnf") && text.contains("error"), "{text}");
    assert!(stderr(&output).contains("no fixture"), "{}", stderr(&output));

    let empty = tempfile::tempdir().unwrap();
    write_fixtures(empty.path(), &[]);
    let output =
        ebc().args(["lookup", ISBN, "--offline-fixtures"]).arg(empty.path()).output().unwrap();
    assert_eq!(status(&output), 1, "every provider failed: {}", stderr(&output));
}

#[test]
fn a_fixture_index_cannot_reach_outside_its_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("index.json"),
        r#"[{"url_contains": "openlibrary.org", "file": "../../../../etc/hostname"}]"#,
    )
    .unwrap();
    let output = ebc()
        .args(["lookup", ISBN, "--provider", "openlibrary", "--offline-fixtures"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(status(&output), 1);
    assert!(stderr(&output).contains("inside the fixture directory"), "{}", stderr(&output));

    let broken = tempfile::tempdir().unwrap();
    std::fs::write(broken.path().join("index.json"), "{not json").unwrap();
    for directory in [broken.path(), &broken.path().join("missing")] {
        let output =
            ebc().args(["lookup", ISBN, "--offline-fixtures"]).arg(directory).output().unwrap();
        assert_eq!(status(&output), 1);
        assert!(stderr(&output).contains("index.json"), "{}", stderr(&output));
    }
}

#[test]
fn bad_arguments_are_explained() {
    for (args, code, reason) in [
        (vec!["lookup", "9782070360025"], 1, "check digit"),
        (vec!["lookup", ISBN, "--provider", "amazon"], 64, "unknown provider"),
        (vec!["lookup", ISBN, "--base-url", "openlibrary"], 64, "PROVIDER=URL"),
        (vec!["lookup", ISBN, "--base-url", "nobody=http://127.0.0.1:1"], 64, "unknown provider"),
        (vec!["lookup", ISBN, "--base-url", "openlibrary=ftp://x"], 64, "http://"),
    ] {
        let output = ebc().args(&args).output().unwrap();
        assert_eq!(status(&output), code, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(reason), "{args:?}: {}", stderr(&output));
        assert!(stdout(&output).is_empty());
    }
}

#[test]
fn the_providers_are_listed_with_their_licences() {
    let output = ebc().args(["lookup", "--list-providers"]).output().unwrap();
    assert_eq!(status(&output), 0);
    let text = stdout(&output);
    for id in ebc_catalog::PROVIDER_IDS {
        assert!(text.contains(id), "{id} missing");
    }
    assert!(text.contains("Licence Ouverte") && text.contains("CC0"));
    let json = ebc().args(["lookup", "--list-providers", "--json"]).output().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 13);
}
