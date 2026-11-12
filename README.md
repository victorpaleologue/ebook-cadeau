# eBook Cadeau

Service web pour générer des cartes cadeau imprimables qui accompagnent des eBooks.
*A web service that turns a gifted eBook into something you can wrap.*

An eBook has no object to hand over. eBook Cadeau generates a **print-ready PDF** of a
book-shaped card: the book's cover on the front, a **1 cm spine** with the title so it can
stand on a shelf, and inside, a **QR code and a short code** that put the eBook on the
recipient's e-reader in a few steps.

> **Status: first complete version.** The workspace builds, 1 814 tests pass, and a card goes
> end to end: a store sells an eBook as a gift, the card PDF is generated with its QR code,
> and a simulated e-reader redeems it. Nothing has been tested with real printers, e-readers
> or live ISBN services yet. [`docs/STATUS.md`](docs/STATUS.md) has the details;
> [`docs/architecture.md`](docs/architecture.md) the design; [`docs/research/`](docs/research/)
> the studies; [`docs/business/`](docs/business/) the business model; [`docs/samples/`](docs/samples/)
> sample cards.

## Try it

```sh
cargo run -p ebc-demo-integration --bin ebc-demo     # server, two fictional stores, printer
scripts/build-wasm.sh && EBC_WEB_DIR=web cargo run -p ebc-server   # card designer at /app/
cargo run -p ebc-cli -- card --title "Les Vagues du soir" --author "Jeanne Roy" \
    --random-code --base-url https://ebk.example --typographic -o card.pdf
```

## Repository layout

| Path | Content |
|---|---|
| `crates/` | Rust libraries (`ebc-*`) — pure, wasm-compatible — plus the server and CLI |
| `web/` | Static frontend running the heavy work in the browser through WebAssembly |
| `demo/` | Fictional eBook stores, a fictional printer and simulated e-readers for tests |
| `e2e/` | Playwright end-to-end tests |
| `docs/` | Architecture, research (ISBN sources, print, printers, UX, legal, business) |
| `tools/` | Scripts: font corpus, business model, training |
| `assets/fonts/` | Bundled fonts, all under the SIL Open Font License |

## Development

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Licence

BSD 3-Clause — see [`LICENSE`](LICENSE). Bundled fonts keep their own licences
(`assets/fonts/OFL-*.txt`).
