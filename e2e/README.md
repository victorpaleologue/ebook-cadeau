# Browser end-to-end tests

Real Chromium (through Playwright) driving the **demo constellation** — the eBook Cadeau
server, the two fictional stores "Septante" and "Grande Enseigne" (demo — not affiliated
with any real store), the fictional printer and the Kindle mail sink, all started by the
`ebc-demo` binary — plus the card designer web app. They complement the Rust journeys of
`demo/integration` (simulated, script-less e-readers over HTTP) with what only a browser
shows: rendering, emulated phones, JavaScript switched off, downloads, screenshots.

```sh
e2e/run.sh                              # builds what is needed, then runs everything (~15 s warm)
e2e/run.sh tests/septante.test.mjs      # one file (paths relative to e2e/)
EBC_E2E_SKIP_BUILD=1 e2e/run.sh         # reuse the binaries and web/pkg as they are
e2e/print-check.sh                      # sample cards for every format x profile, checked by tools
```

## What it needs

* **Node 22** and **Playwright** with Chromium. In the development container both are
  preinstalled (`require('playwright')` resolves through `NODE_PATH`, browsers in
  `PLAYWRIGHT_BROWSERS_PATH`): nothing to install. Elsewhere: `cd e2e && npm ci && npx
  playwright install chromium` (the version is pinned in `package.json` / `package-lock.json`;
  `e2e/node_modules` wins over a global install).
* The Rust toolchain and the `wasm-bindgen` CLI of the version in `Cargo.lock`
  (`scripts/build-wasm.sh` says which). `run.sh` builds, in the debug profile, `ebc-demo`
  (`demo/integration`), `ebc-server`, `ebc` (`crates/ebc-cli`), and `web/pkg` with
  `scripts/build-wasm.sh`.
* Optional: `pdftoppm`, `pdftotext`, `zbarimg` (`scripts/install-print-tools.sh`). Without
  them the QR-code and printed-text checks of the card are **skipped with a note**;
  `EBC_E2E_REQUIRE_TOOLS=1` (set in CI) turns a missing tool into a failure.

No test touches the network: every browser context aborts requests to anything but
`127.0.0.1` and fails on them, and on console errors.

## Scenarios

Each file starts its own constellation on ports chosen by the system (`EBC_DEMO_*_PORT=0`,
URLs read from the banner `ebc-demo` prints), so files run in parallel; the scenarios
inside a file run in order and share state (a later one is *skipped*, not failed, when an
earlier one did not produce what it needs).

| File | Scenario |
|---|---|
| `tests/septante.test.mjs` | **1.** Desktop buyer at Septante: catalogue → product → cart → account → checkout; asserts the gift wording ("C'est pour offrir ?", "Offrir cet ebook", help and delivery note, the two card choices), ticks the gift option, giver/recipient/message, *print at home*, pays; the order page shows the card PDF link, the code and the claim URL; the PDF is downloaded, its inside page rendered (`pdftoppm`, 300 dpi) and its QR code decoded (`zbarimg`): it must be the claim URL upper-cased, with no QR code outside; the code, names, message and host are read back with `pdftotext`. **2.** The recipient on a phone (Pixel 7): opens the decoded upper-case URL, sees title, author, giver, message, the device chooser (phone detected), downloads the EPUB (served by Septante through the server's redirect, `partner_file`; the watermark names recipient and giver), asks for a pair code (`XXX-XXX`, `/e`, 15 minutes). **3a.** A Kobo (its real user agent, 480 × 640, **JavaScript disabled**): home page, types the code in lower case without dashes, gift page in e-ink layout with the Kobo recognised, downloads the EPUB. **3b.** A PocketBook (JS disabled) at `/e`: a wrong pair code is refused with the form shown again, the real one downloads the EPUB, a second use is refused. The home page, `/e` and the gift page (phone and Kobo): no `<script>`, HTML + CSS under the 30 kB budget of `docs/api.md` §2, no sideways scrolling; on the Kobo a download button at least 48 px high. |
| `tests/grande-enseigne.test.mjs` | **4.** Gift purchase at Grande Enseigne ("Offrir cet Ebook", account-based delivery note); the recipient (iPhone 15) opens the claim URL, presses "add to my library", follows the signed redirect (`ebc_gift`, `ebc_order`, `ebc_ts`, `ebc_sig`; no `Referer` carrying the claim URL to the store), creates an account, accepts the book; the library lists it as a gift; the gift page then offers no second redemption; the buyer's order page turns to *fulfilled* and loses "cancel". |
| `tests/designer.test.mjs` | **5.** The card designer at `/app/` (served by `ebc-server` with `EBC_WEB_DIR=web`, with its CSP): French by default, typographic card, title/author/message typed, live preview of both sides (described by `alt`), the demonstration code, the PDF downloaded and checked with **`ebc preflight`** for the selected format and profile, its QR code decoded; English via the language menu (typed values kept); German via `?lang=de` at phone width without sideways scrolling; accessibility audit in French and English. **6.** No file hosting in the core (`docs/api.md` v0.3): the designer shows no upload choice, the store-code entry and the note pointing buyers to the store's gift option stay, no print-order form without a partner gift, and the server answers `404` to `/api/v1/uploads`, `/api/v1/gifts` and `/api/partner/v1/uploads`. |
| `tests/audit.test.mjs` | Checks the accessibility audit itself on a clean and a deliberately broken page. |

**6. Visual record.** Every scenario writes full-page screenshots (deviceScaleFactor 1)
and the downloaded files to `e2e/artifacts/` (git-ignored, cleared by `run.sh`; CI uploads
it when the job fails): the store pages, the checkout, the order page, the card PDF and its
rendered pages, the phone and e-reader pages, the store's account pages, the designer.

## The accessibility audit

`audit()` in `lib/harness.mjs` is a small home-made check, **not axe-core**: `lang` and
`title`, a label (or `aria-label`/`aria-labelledby`/`title`) on every visible form
control, an accessible name on every button and link, an `alt` attribute on every image,
and the WCAG 2 contrast ratio of every visible text node against its effective background
colour (4.5:1, 3:1 for large text; ancestor opacity included; disabled controls exempt;
text over a background *image* is not judged). It fails the designer and the recipient's
gift page; on the store pages it reports findings without failing (`known()`), since the
stores are demo fixtures. It does not check focus order, keyboard traps, ARIA semantics or
anything a screen reader would reveal.

## Known limits

* `ebc-demo` does not read `EBC_WEB_DIR` (it builds its server configuration with
  `Config::for_tests()`), so `/app/` answers 404 there; the designer test then starts a
  separate `ebc-server` with `EBC_WEB_DIR` and says so in a `NOTE`. It switches to the
  constellation by itself once `ebc-demo` serves `/app/`.
* E-reader browsers are approximated by Chromium with their user agent, a small viewport
  and JavaScript off: old WebKit engines, e-ink refresh, on-screen keyboards and how the
  devices store downloads are not reproduced (the Rust journeys model taps and libraries).
* Phones are Playwright device descriptors (viewport, user agent, touch) on Chromium,
  including the iPhone one: no Safari/WebKit.
* Print checks are by tools only (`ebc preflight`, poppler, qpdf, ghostscript); no printer
  has seen these PDFs.
