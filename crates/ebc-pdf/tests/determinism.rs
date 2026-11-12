//! The same document always gives the same bytes.

mod common;

use common::*;
use ebc_pdf::{WriteOptions, write_pdf};
use ebc_scene::{Color, Item, PdfStandard, Rect};

const ALL: [PdfStandard; 4] =
    [PdfStandard::Plain, PdfStandard::X1a2001, PdfStandard::X3_2002, PdfStandard::X4];
const OPTIONS: [WriteOptions; 4] = [
    WriteOptions { compress: true, keep_layers: true },
    WriteOptions { compress: true, keep_layers: false },
    WriteOptions { compress: false, keep_layers: true },
    WriteOptions { compress: false, keep_layers: false },
];

#[test]
fn writing_twice_gives_identical_bytes() {
    for standard in ALL {
        let doc = card::document(standard);
        for options in OPTIONS {
            let first = write_pdf(&doc, &options).expect("writable");
            let second = write_pdf(&doc, &options).expect("writable");
            assert!(first == second, "{standard:?} {options:?}: output differs between runs");
        }
    }
}

#[test]
fn rebuilding_the_document_from_scratch_gives_identical_bytes() {
    // New allocations, new `Arc`s, new font and image buffers: nothing may depend on
    // addresses or on hash-map iteration order.
    for standard in ALL {
        let first =
            write_pdf(&card::document(standard), &WriteOptions::default()).expect("writable");
        let clone = write_pdf(&card::document(standard).clone(), &WriteOptions::default())
            .expect("writable");
        let on_another_thread = std::thread::spawn(move || {
            write_pdf(&card::document(standard), &WriteOptions::default()).expect("writable")
        })
        .join()
        .expect("thread finishes");
        assert!(first == clone, "{standard:?}");
        assert!(first == on_another_thread, "{standard:?}");
    }
}

#[test]
fn nothing_but_the_input_ends_up_in_the_file() {
    // The only dates in the file are the one we passed in (2026-11-12T08:30:00Z).
    let doc = card::document(PdfStandard::X4);
    let pdf =
        write_pdf(&doc, &WriteOptions { compress: false, keep_layers: false }).expect("writable");
    let text = String::from_utf8_lossy(&pdf);
    let dates: Vec<&str> = text.match_indices("D:2").map(|(i, _)| &text[i..i + 17]).collect();
    assert_eq!(dates, ["D:20261112083000Z", "D:20261112083000Z"]);
    assert_eq!(
        text.matches("2026-11-12T08:30:00Z").count(),
        3,
        "XMP create, modify, metadata dates"
    );

    // Without a date (plain PDF) there is none at all.
    let mut undated = card::document(PdfStandard::Plain);
    undated.info.created_unix = None;
    let pdf = write_pdf(&undated, &WriteOptions { compress: false, keep_layers: false })
        .expect("writable");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("CreationDate")
            && !text.contains("ModDate")
            && !text.contains("xmp:CreateDate")
    );
    let again = write_pdf(&undated, &WriteOptions { compress: false, keep_layers: false })
        .expect("writable");
    assert!(pdf == again);
}

#[test]
fn different_input_gives_different_output() {
    let base = card::document(PdfStandard::X4);
    let reference = write_pdf(&base, &WriteOptions::default()).expect("writable");

    let mut recoloured = base.clone();
    recoloured.pages[1].items.push(Item::filled_rect(Rect::new(1.0, 1.0, 1.0, 1.0), Color::BLACK));
    let mut later = base.clone();
    later.info.created_unix = Some(CREATED + 60);
    let mut renamed = base.clone();
    renamed.info.custom[0].1 = "digest".into();

    let mut outputs = vec![reference];
    for (what, doc) in [("content", recoloured), ("date", later), ("custom key", renamed)] {
        let pdf = write_pdf(&doc, &WriteOptions::default()).expect("writable");
        assert!(outputs.iter().all(|other| *other != pdf), "{what} change is not reflected");
        outputs.push(pdf);
    }
    assert!(
        outputs[0]
            != write_pdf(&base, &WriteOptions { compress: true, keep_layers: true })
                .expect("writable"),
        "options matter too"
    );
}
