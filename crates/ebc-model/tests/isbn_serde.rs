//! `ebc-model` enables the `serde` feature of `ebc-isbn`; check the wire format here,
//! where `serde_json` is available.

use ebc_model::{Isbn10, Isbn13};

#[test]
fn isbn13_serialises_as_plain_digits() {
    let isbn: Isbn13 = "978-2-07-036002-4".parse().expect("valid");
    assert_eq!(serde_json::to_string(&isbn).expect("serialises"), "\"9782070360024\"");
    for input in
        ["\"9782070360024\"", "\"978-2-07-036002-4\"", "\"2070360024\"", "\"ISBN 2-07-036002-4\""]
    {
        assert_eq!(serde_json::from_str::<Isbn13>(input).expect("parses"), isbn, "{input}");
    }
    for bad in ["\"9782070360025\"", "\"\"", "9782070360024", "null", "[]", "\"4006381333931\""] {
        assert!(serde_json::from_str::<Isbn13>(bad).is_err(), "{bad}");
    }
}

#[test]
fn isbn10_serialises_as_ten_characters() {
    let isbn: Isbn10 = "0-8044-2957-x".parse().expect("valid");
    assert_eq!(serde_json::to_string(&isbn).expect("serialises"), "\"080442957X\"");
    for input in ["\"080442957X\"", "\"080442957x\"", "\"0-8044-2957-X\"", "\"9780804429573\""] {
        assert_eq!(serde_json::from_str::<Isbn10>(input).expect("parses"), isbn, "{input}");
    }
    for bad in ["\"0804429573\"", "\"9791090636071\"", "\"\"", "804429573", "null"] {
        assert!(serde_json::from_str::<Isbn10>(bad).is_err(), "{bad}");
    }
}

#[test]
fn isbns_round_trip_inside_collections() {
    let isbns: Vec<Isbn13> = ["9782070360024", "9791090636071", "9780306406157"]
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    assert_eq!(isbns.len(), 3);
    let json = serde_json::to_string(&isbns).expect("serialises");
    assert_eq!(json, r#"["9782070360024","9791090636071","9780306406157"]"#);
    assert_eq!(serde_json::from_str::<Vec<Isbn13>>(&json).expect("parses"), isbns);
}
