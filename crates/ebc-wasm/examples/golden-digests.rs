//! Prints the SHA-256 of the three golden cards as the **native** library renders them,
//! one `"<sha256>  <name>"` line each.
//!
//! `scripts/build-wasm.sh` stores the output next to the WebAssembly package
//! (`web/pkg/golden-native.sha256`), and `web/tests/smoke.test.mjs` checks that the
//! package renders the same fixtures to the same bytes.
//!
//! ```sh
//! cargo run -p ebc-wasm --example golden-digests
//! ```

#[path = "../tests/common/golden.rs"]
mod golden;

use std::io::Write as _;

fn main() -> std::io::Result<()> {
    std::io::stdout().write_all(golden::digests().as_bytes())
}
