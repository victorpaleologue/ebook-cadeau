# Demo constellation

Everything under `demo/` is **fictional and for demonstration**: the stores and the printer
are not affiliated with any real company, no payment is taken, nothing is printed, secrets
are public and all data lives in memory. Every page of the stores says so.

| Crate | What it is | Binary | Default port |
|---|---|---|---|
| `demo/stores` (`ebc-demo-stores`) | Two fictional eBook stores offering the "eBook cadeau" option at checkout | `ebc-demo-stores` | 8101, 8102 |
| `demo/printer` (`ebc-demo-printer`) | "Imprimerie du Dos Carré", a fictional print-on-demand API that preflights card PDFs (`docs/api.md` §5) | `ebc-demo-printer` | 8103 |
| `demo/ereaders` (`ebc-demo-ereaders`) | Simulated e-readers: script-less browsers per device profile, OPDS client, mail-to-device SMTP sink, account sync, simulated LCP reader | `ebc-ereader` (CLI) | — |
| `demo/integration` (`ebc-demo-integration`) | The whole constellation in one process, plus the end-to-end journeys | `ebc-demo` | 8080 + the above |

The two stores model two *kinds* of reseller (the archetypes are named here only, never in
the user interface):

* **Septante** — an independent store selling DRM-free, watermarked EPUBs with direct
  downloads (the way stores of the 7switch kind work). Gifts are `hosted_file`: the store
  uploads a copy stamped for the recipient; the recipient needs no account.
* **Grande Enseigne** — a large cultural retailer whose eBooks live in an e-reader
  ecosystem account (the way a Fnac/Kobo-like pairing works). Gifts are `reseller_redirect`:
  the recipient adds the book to their store account and their e-reader syncs it.

## The whole thing at once: `ebc-demo`

```sh
cargo run -p ebc-demo-integration --bin ebc-demo
```

starts the **real** `ebc-server`, both stores, the printer and a Send-to-Kindle mail sink,
wired to each other (signed partner API, webhooks, redemption redirects, print orders,
SMTP), and prints where everything listens:

```text
  eBook Cadeau (server)           http://127.0.0.1:8080  code form at /, help at /aide, pairing at /e
  Septante (démo)                 http://127.0.0.1:8101  fictional DRM-free store — gifts as hosted_file
  Grande Enseigne (démo)          http://127.0.0.1:8102  fictional ecosystem store — gifts as reseller_redirect
  Imprimerie du Dos Carré (démo)  http://127.0.0.1:8103  fictional printer — GET /v1/products, /__test/orders
  Mail-to-device gateway          smtp://127.0.0.1:…     SMTP sink for demo@kindle.com
```

| Variable | Default | Meaning |
|---|---|---|
| `EBC_DEMO_SERVER_PORT` | `8080` | eBook Cadeau server (also its public URL) |
| `EBC_DEMO_SEPTANTE_PORT` | `8101` | Septante |
| `EBC_DEMO_GRANDE_ENSEIGNE_PORT` | `8102` | Grande Enseigne |
| `EBC_DEMO_PRINTER_PORT` | `8103` | printer |
| `EBC_DEMO_KINDLE_ADDRESS` | `demo@kindle.com` | device address the mail sink answers for (the sender is pre-approved) |
| `EBC_DEMO_PRINTER_TICK_MS` | `2000` | time between two steps of a print order (`0`: only `POST /__test/advance` moves orders) |

A port set to `0` is picked by the operating system. Test hooks are on: `GET {store}/__test/state`
(orders, gifts, accounts, received webhooks as JSON) and `GET {printer}/__test/orders`.

A walk-through:

1. Open Septante, put a book in the cart, check out, tick **"C'est pour offrir"**, pay
   (any e-mail address; no card is asked for).
2. The order page shows the claim URL and the code (`XXXX-XXXX-XXXX`). The printable card
   is not available yet (see *Limits*), so the page says to pass on the link and the code.
3. Redeem it as an e-reader without a phone would — by typing the code on the home page:

   ```sh
   cargo run -p ebc-demo-ereaders --bin ebc-ereader -- --profile kobo open http://127.0.0.1:8080/ --code XXXX-XXXX-XXXX
   ```

   or as a Kindle, which is not offered the EPUB and goes through Send to Kindle:

   ```sh
   cargo run -p ebc-demo-ereaders --bin ebc-ereader -- --profile kindle open http://127.0.0.1:8080/k/XXXXXXXXXXXX --device-email demo@kindle.com
   ```

   `ebc-ereader profiles` lists the simulated devices, `ebc-ereader lint URL` checks a page
   the way an old e-reader browser would. A real browser works too, of course: open the
   claim URL, try the device chooser, ask for a pair code and type it at `/e`.
4. Do the same at Grande Enseigne: the gift page sends you to the store, where you create
   an account and accept the book; the order page of the buyer then says it is in the
   recipient's library, and the device API (`POST /api/device/link`, `GET /api/device/library`)
   serves it to a linked reader.
5. On an order page, **"Obtenir un nouveau code"** replaces a lost card's code (the old one
   answers `410`), **"Annuler le cadeau"** refunds an unused gift.

For browser-driven tests (Playwright): the ports are fixed, the pages carry stable element
ids (`docs/api.md` §2 for the server; `demo/stores/src/views.rs` for the stores), nothing
needs JavaScript, and the test hooks above expose the state.

## Running the pieces separately

```sh
EBC_PARTNERS=crates/ebc-server/partners.dev.toml EBC_PRINTER_URL=http://127.0.0.1:8103 cargo run -p ebc-server
DEMO_TEST_HOOKS=1 EBC_PRINTER_URL=http://127.0.0.1:8103 cargo run -p ebc-demo-stores    # both stores
DEMO_TEST_HOOKS=1 cargo run -p ebc-demo-printer
```

The stores' default partner identifiers and secrets (`septante` / `grande-enseigne`,
`dev-secret-…`) are those of `crates/ebc-server/partners.dev.toml`, whose `redirect_hosts`
and webhook URLs are the stores' default addresses — so the three commands above work
together unchanged. The stores read `EBC_URL` (default `http://127.0.0.1:8080`),
`EBC_PRINTER_URL`, `DEMO_{SEPTANTE,GRANDE_ENSEIGNE}_{BIND,PUBLIC_URL,PARTNER_ID,PARTNER_SECRET,DELIVERY}`
and `DEMO_TEST_HOOKS`; the printer reads `DEMO_PRINTER_BIND`, `DEMO_PRINTER_TICK_MS` and
`DEMO_TEST_HOOKS`.

## End-to-end tests

```sh
cargo test -p ebc-demo-integration        # about 2 s; offline, loopback sockets only
```

Each test starts its own constellation on ephemeral ports with a shared manual clock
(`ebc_demo_integration::World`) and drives complete journeys over real HTTP and SMTP —
none of our own components is mocked:

| Journey | Test module |
|---|---|
| Septante: browse → cart → gift checkout → signed partner API + watermarked upload → a Kobo with **no phone** types the printed code and downloads the stamped EPUB (page lint, keystroke cost) | `tests/it/septante.rs` |
| Phone + pair code to a PocketBook; reuse and expiry refused | `septante.rs` |
| Kindle: no EPUB download offered; Send to Kindle by SMTP, parked until the sender is approved | `septante.rs` |
| OPDS 1.2 and 2.0 feeds of a download session | `septante.rs` |
| Every device profile gets the book or is told why; download limit → `exhausted`; two books, two gifts | `septante.rs` |
| Grande Enseigne: signed redirect → account → fulfilment confirmed → account sync; second account refused | `tests/it/grande_enseigne.rs` |
| `reseller_licence` with an in-test licence endpoint and the simulated LCP reader | `tests/it/licence.rs` |
| Printer: card with bleed accepted, without bleed rejected (via a store's client and via the server's print orders); posted-card checkout | `tests/it/printing.rs` |
| Typos, unknown codes and the rate limit, cancelled and reissued codes, tampered signatures, hostile text, misleading `User-Agent` + device chooser | `tests/it/negative.rs` |

Keystroke costs of the device journeys are written to `target/tmp/keystrokes-*.txt`.
`demo/stores` additionally tests itself against a strict mock of the partner API
(`cargo test -p ebc-demo-stores`), `demo/ereaders` and `demo/printer` have their own unit
and API tests.

## Limits (what is stubbed or unverified)

* **No printable card yet.** The server's `card.pdf` answers `501` until the card composer
  (`ebc-card`) is plugged into its `CardRenderer`; the stores then show the claim URL and
  code instead, and a "printed and posted" order reports that nothing was sent to the
  printer. `tests/it/printing.rs` covers the printer leg with a structural PDF built with
  `ebc-layout` + `ebc-pdf` (flat panels — a fixture, not a card), exercises the
  store → `card.pdf` → printer → callback chain with that fixture behind the renderer seam,
  and keeps the real full-chain test `#[ignore = "needs ebc-card wired into the server"]`.
* **LCP is simulated**: the licence flow has the right shape but no cryptography
  (`ebc_demo_ereaders::lcp`). Neither store issues licence files; the `reseller_licence`
  journey uses an in-test partner.
* **Device profiles are hypotheses**, built from `docs/research/` and never validated on
  hardware; each capability carries its source or says it is an assumption
  (`ebc-ereader profiles --claims`). Send to Kindle's EPUB conversion is only labelled.
* The demo runs on `127.0.0.1` with ports; a real deployment needs a short, port-less host
  (the Kobo browser is reported to struggle with non-standard ports, and the QR code
  carries the URL in upper case).
