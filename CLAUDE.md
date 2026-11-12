# eBook Cadeau — working rules for agents and contributors

eBook Cadeau turns a gifted eBook into a physical object: a print-ready PDF "book jacket" card
(front = the book's cover, a **10 mm spine** carrying the title, a back, and an inside with a QR
code + short code that gets the eBook onto the recipient's e-reader). Read
`docs/architecture.md` before touching code; research backing the decisions is in
`docs/research/`.

## Layout

- `crates/` — Rust libraries and binaries (`ebc-*`). Pure libraries must compile to
  `wasm32-unknown-unknown`; anything CPU-heavy runs in the browser through `ebc-wasm`.
- `demo/` — fictional stores, printer and e-readers used by the integration tests.
- `web/` — static frontend (ES modules + the wasm package). `e2e/` — Playwright tests.
- `tools/` — scripts (font corpus, business model, training). `docs/` — research and design.

## Rules

1. **Stay in your lane.** Only edit the directories you were assigned. Never edit another
   crate's sources to make yours compile — report the mismatch instead. `crates/ebc-scene`
   is the shared contract: do not change its public API without saying so explicitly.
2. **No git operations** (add/commit/push/stash/checkout) unless your brief says you own them.
   Other agents work in the same tree at the same time.
3. **Dependencies**: add third-party crates with `cargo add -p <your-crate> <dep>` in your own
   crate's `Cargo.toml`. Internal crates are declared in `[workspace.dependencies]` — use
   `ebc-foo = { workspace = true }`. Licences must be permissive (MIT, Apache-2.0, BSD, ISC,
   Zlib, MPL-2.0, Unicode, CC0). No GPL/AGPL/LGPL code or data. Prefer pure-Rust crates
   that build for wasm; no C toolchains, no `openssl`.
4. **Build only your crate**: `cargo build -p <crate>`, `cargo test -p <crate>`,
   `cargo clippy -p <crate> --all-targets -- -D warnings`, `cargo fmt -p <crate>`.
   Cargo may print "Blocking waiting for file lock" while another agent builds — just wait
   (allow up to 10 minutes per command). If the workspace manifest is momentarily broken by
   someone else, wait 30 s and retry.
5. **Quality bar**: `#![forbid(unsafe_code)]`; no `unwrap()`/`expect()` on fallible runtime
   paths in library code (tests are fine); typed errors with `thiserror`; public items
   documented; no panics on untrusted input (ISBNs, HTML, images, PDFs, HTTP bodies) —
   add tests with hostile input. Deterministic output: no wall-clock or randomness inside
   libraries unless injected by the caller.
6. **Tests are offline.** No network in tests. Use recorded fixtures under
   `crates/<crate>/tests/fixtures/`. Live tests must be `#[ignore]` and opt-in via env var.
7. **Units and coordinates**: millimetres, origin top-left of the trim box, y down
   (see `ebc-scene` docs). Colour for text/QR/marks is CMYK or grey, never RGB.
8. **Honesty**: if something is stubbed, approximate or unverified, say so in the code
   comment and in your final report. Do not claim tests pass without running them.
9. **Naming**: demos must use clearly fictional store names with a visible
   "demo — not affiliated" notice. Do not copy real logos or trade dress.
10. **Network**: the sandbox blocks most hosts. Reachable: crates.io, npm, PyPI,
    `raw.githubusercontent.com`, `fonts.gstatic.com`, Ubuntu apt. WebFetch mostly fails;
    WebSearch works. Dependency sources are readable under `~/.cargo/registry/src/`.

## Commands

```sh
cargo test --workspace                 # all Rust tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
