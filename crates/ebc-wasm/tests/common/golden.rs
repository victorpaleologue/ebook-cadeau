//! The golden cards: three fixed requests rendered to PDF, shared by the `golden` test
//! and the `golden-digests` example (through `#[path]`).

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use ebc_wasm::api::card::{RenderedCard, render_card};
use sha2::{Digest, Sha256};

/// The cases: fixture name, with the cover fixture, with a CMYK output profile.
pub(crate) const CASES: [(&str, bool, bool); 3] =
    [("typographic", false, false), ("cover", true, false), ("pdfx", false, true)];

/// `crates/ebc-wasm/tests/fixtures`.
pub(crate) fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Renders one case natively.
pub(crate) fn render(name: &str, with_cover: bool, with_icc: bool) -> RenderedCard {
    let dir = fixtures();
    let request = fs::read_to_string(dir.join(format!("golden-{name}.json")))
        .unwrap_or_else(|e| panic!("golden-{name}.json: {e}"));
    let cover = with_cover.then(|| {
        fs::read(dir.join("golden-cover.png")).expect("tests/fixtures/golden-cover.png")
    });
    let icc = with_icc.then(|| {
        fs::read(dir.join("../../../../assets/icc/CGATS001Compat-v2-micro.icc"))
            .expect("the CC0 CMYK profile of assets/icc")
    });
    render_card(&request, cover, icc.as_deref()).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// Lower-case hexadecimal SHA-256.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// One line per case, `"<sha256>  <name>\n"`, in the format of `sha256sum`.
pub(crate) fn digests() -> String {
    CASES.iter().fold(String::new(), |mut out, (name, cover, icc)| {
        let _ = writeln!(out, "{}  {name}", sha256_hex(&render(name, *cover, *icc).pdf));
        out
    })
}
